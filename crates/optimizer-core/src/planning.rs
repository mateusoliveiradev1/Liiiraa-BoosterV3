//! Dry-run tweak plan builder and deterministic dependency ordering.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

use crate::{
    catalog::TweakRegistry,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlanId, PlannedChange, RebootPolicy, RollbackKind,
        RollbackPlan, RollbackStep, SessionScope, TweakDefinition, TweakId, TweakMode,
        TweakOperationKind, TweakPlan, TweakPlanItem,
    },
};

/// Directed dependency between two selected tweaks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakDependency {
    /// Tweak that must run after `depends_on`.
    pub tweak_id: TweakId,
    /// Prerequisite tweak that must appear earlier in the plan.
    pub depends_on: TweakId,
    /// Human-facing reason for the ordering.
    pub reason: String,
}

impl TweakDependency {
    /// Creates a dependency edge for dry-run ordering.
    #[must_use]
    pub fn new(
        tweak_id: impl Into<String>,
        depends_on: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            tweak_id: tweak_id.into(),
            depends_on: depends_on.into(),
            reason: reason.into(),
        }
    }
}

/// Input for building a non-mutating tweak plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DryRunPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: PlanId,
    /// Highest mode the user explicitly requested for this plan.
    pub requested_mode: TweakMode,
    /// Optional selected tweak IDs. Empty means all catalog definitions in registry order.
    pub selected_tweak_ids: Vec<TweakId>,
    /// Explicit dependencies that must shape plan order.
    pub dependencies: Vec<TweakDependency>,
}

impl DryRunPlanRequest {
    /// Builds a request for every definition in the registry.
    #[must_use]
    pub fn all(plan_id: impl Into<String>, requested_mode: TweakMode) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode,
            selected_tweak_ids: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Builds a request for a selected set of tweak IDs.
    #[must_use]
    pub fn selected(
        plan_id: impl Into<String>,
        requested_mode: TweakMode,
        selected_tweak_ids: Vec<TweakId>,
    ) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode,
            selected_tweak_ids,
            dependencies: Vec::new(),
        }
    }

    /// Adds one dependency edge to this request.
    #[must_use]
    pub fn with_dependency(mut self, dependency: TweakDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }
}

/// Reason a dry-run plan could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanBuildErrorReason {
    /// A selected or dependency tweak ID was not in the registry.
    UnknownTweakId,
    /// A dependency refers to a prerequisite outside the selected plan.
    MissingDependency,
    /// Dependency ordering contains a cycle.
    DependencyCycle,
}

impl PlanBuildErrorReason {
    /// Returns a stable reason string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownTweakId => "unknown_tweak_id",
            Self::MissingDependency => "missing_dependency",
            Self::DependencyCycle => "dependency_cycle",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnknownTweakId => "Tweak ID is not present in the registry",
            Self::MissingDependency => "Dependency prerequisite is not selected",
            Self::DependencyCycle => "Plan dependency graph contains a cycle",
        }
    }
}

/// Structured dry-run plan build error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanBuildError {
    reason: PlanBuildErrorReason,
    tweak_id: Option<TweakId>,
    detail: Option<String>,
}

impl PlanBuildError {
    fn with_tweak(reason: PlanBuildErrorReason, tweak_id: impl Into<String>) -> Self {
        Self {
            reason,
            tweak_id: Some(tweak_id.into()),
            detail: None,
        }
    }

    fn with_detail(reason: PlanBuildErrorReason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            tweak_id: None,
            detail: Some(detail.into()),
        }
    }

    /// Returns the plan build failure reason.
    #[must_use]
    pub const fn reason(&self) -> PlanBuildErrorReason {
        self.reason
    }

    /// Returns the associated tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns extra dependency detail, when known.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for PlanBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, " [{detail}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for PlanBuildError {}

/// Builds a dry-run plan from validated registry definitions.
///
/// This function is pure: it reads catalog contracts, resolves ordering, and
/// returns a plan without running detection, backup, apply, verify, or rollback
/// operations.
pub fn build_dry_run_plan(
    registry: &TweakRegistry,
    request: &DryRunPlanRequest,
) -> Result<TweakPlan, PlanBuildError> {
    let selected_ids = selected_tweak_ids(registry, request)?;
    let ordered_ids = order_tweak_ids(&selected_ids, &request.dependencies)?;
    let ordered_definitions = ordered_ids
        .iter()
        .map(|tweak_id| {
            registry
                .get(tweak_id)
                .ok_or_else(|| {
                    PlanBuildError::with_tweak(
                        PlanBuildErrorReason::UnknownTweakId,
                        tweak_id.as_str(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut applied_definitions = BTreeMap::new();
    let mut plan_warnings = Vec::new();
    let mut items = Vec::with_capacity(ordered_definitions.len());

    for definition in ordered_definitions {
        let (item, conflict_warnings) =
            plan_item_for_definition(definition, request.requested_mode, &applied_definitions);

        plan_warnings.extend(conflict_warnings);

        if item.action == PlanAction::Apply {
            applied_definitions.insert(definition.id.clone(), definition);
        }

        items.push(item);
    }

    Ok(TweakPlan {
        id: request.plan_id.clone(),
        requested_mode: request.requested_mode,
        catalog_schema_version: registry.schema_version().to_owned(),
        items,
        warnings: plan_warnings,
    })
}

fn selected_tweak_ids(
    registry: &TweakRegistry,
    request: &DryRunPlanRequest,
) -> Result<Vec<TweakId>, PlanBuildError> {
    if request.selected_tweak_ids.is_empty() {
        return Ok(registry.iter().map(|definition| definition.id.clone()).collect());
    }

    let mut selected = Vec::with_capacity(request.selected_tweak_ids.len());
    let mut seen = BTreeSet::new();

    for tweak_id in &request.selected_tweak_ids {
        if registry.get(tweak_id).is_none() {
            return Err(PlanBuildError::with_tweak(
                PlanBuildErrorReason::UnknownTweakId,
                tweak_id.as_str(),
            ));
        }

        if seen.insert(tweak_id.clone()) {
            selected.push(tweak_id.clone());
        }
    }

    Ok(selected)
}

fn order_tweak_ids(
    selected_ids: &[TweakId],
    dependencies: &[TweakDependency],
) -> Result<Vec<TweakId>, PlanBuildError> {
    let selected_set = selected_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut dependents_by_prerequisite = BTreeMap::<TweakId, Vec<TweakId>>::new();
    let mut indegree = selected_ids
        .iter()
        .map(|tweak_id| (tweak_id.clone(), 0_usize))
        .collect::<BTreeMap<_, _>>();

    for dependency in dependencies {
        if !selected_set.contains(&dependency.tweak_id) {
            return Err(PlanBuildError::with_tweak(
                PlanBuildErrorReason::UnknownTweakId,
                dependency.tweak_id.as_str(),
            ));
        }

        if !selected_set.contains(&dependency.depends_on) {
            return Err(PlanBuildError::with_detail(
                PlanBuildErrorReason::MissingDependency,
                format!("{} depends on {}", dependency.tweak_id, dependency.depends_on),
            ));
        }

        dependents_by_prerequisite
            .entry(dependency.depends_on.clone())
            .or_default()
            .push(dependency.tweak_id.clone());

        if let Some(count) = indegree.get_mut(&dependency.tweak_id) {
            *count += 1;
        }
    }

    let original_position = selected_ids
        .iter()
        .enumerate()
        .map(|(index, tweak_id)| (tweak_id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut ready = selected_ids
        .iter()
        .filter(|tweak_id| indegree.get(*tweak_id) == Some(&0))
        .cloned()
        .collect::<VecDeque<_>>();
    let mut ordered = Vec::with_capacity(selected_ids.len());

    while let Some(tweak_id) = ready.pop_front() {
        ordered.push(tweak_id.clone());

        if let Some(dependents) = dependents_by_prerequisite.get(&tweak_id) {
            for dependent in dependents {
                if let Some(count) = indegree.get_mut(dependent) {
                    *count -= 1;

                    if *count == 0 {
                        insert_ready_by_original_position(
                            &mut ready,
                            dependent.clone(),
                            &original_position,
                        );
                    }
                }
            }
        }
    }

    if ordered.len() == selected_ids.len() {
        Ok(ordered)
    } else {
        Err(PlanBuildError::with_detail(
            PlanBuildErrorReason::DependencyCycle,
            selected_ids.join(","),
        ))
    }
}

fn insert_ready_by_original_position(
    ready: &mut VecDeque<TweakId>,
    tweak_id: TweakId,
    original_position: &BTreeMap<TweakId, usize>,
) {
    let position = original_position.get(&tweak_id).copied().unwrap_or(usize::MAX);
    let insert_at = ready
        .iter()
        .position(|queued_id| {
            original_position
                .get(queued_id)
                .copied()
                .unwrap_or(usize::MAX)
                > position
        })
        .unwrap_or(ready.len());

    ready.insert(insert_at, tweak_id);
}

fn plan_item_for_definition(
    definition: &TweakDefinition,
    requested_mode: TweakMode,
    applied_definitions: &BTreeMap<TweakId, &TweakDefinition>,
) -> (TweakPlanItem, Vec<String>) {
    let mut warnings = definition_warnings(definition);
    let mut plan_warnings = Vec::new();
    let mut action = choose_action(definition, requested_mode, &mut warnings);

    if action == PlanAction::Apply {
        let conflicts = applied_conflicts(definition, applied_definitions);

        if !conflicts.is_empty() {
            action = PlanAction::Recommend;

            let warning = format!(
                "{} conflicts with already planned tweak(s): {}",
                definition.id,
                conflicts.join(", ")
            );

            warnings.push(warning.clone());
            plan_warnings.push(warning);
        }
    }

    let changes = planned_changes(definition, action);
    let backup = backup_requirement(definition, action, &changes);
    let rollback = rollback_plan(definition);

    (
        TweakPlanItem {
            tweak_id: definition.id.clone(),
            category: definition.category.clone(),
            action,
            mode: definition.mode,
            risk: definition.risk,
            changes,
            backup,
            rollback,
            reboot: definition.reboot,
            requires_admin: definition.requires_admin,
            warnings,
        },
        plan_warnings,
    )
}

fn choose_action(
    definition: &TweakDefinition,
    requested_mode: TweakMode,
    warnings: &mut Vec<String>,
) -> PlanAction {
    if definition.is_blocked_guardrail() {
        warnings.push("Blocked guardrail cannot be applied.".to_owned());
        return PlanAction::Deny;
    }

    if !mode_is_allowed(requested_mode, definition.mode) {
        warnings.push(format!(
            "{} mode requires explicit opt-in beyond {} planning.",
            definition.mode.as_str(),
            requested_mode.as_str()
        ));
        return PlanAction::Recommend;
    }

    if definition.mode.requires_explicit_opt_in() {
        warnings.push(format!(
            "{} tweak included because that mode was requested.",
            definition.mode.as_str()
        ));
    }

    if definition.is_mutable()
        && definition.mode == TweakMode::Safe
        && !definition.rollback_kind.needs_backup_before_apply()
    {
        warnings.push("Safe mutable tweak requires rollback before apply.".to_owned());
        return PlanAction::Deny;
    }

    if !definition.is_mutable() {
        return PlanAction::DetectOnly;
    }

    if definition.default_enabled || definition.mode == requested_mode {
        PlanAction::Apply
    } else {
        PlanAction::Recommend
    }
}

fn mode_is_allowed(requested_mode: TweakMode, definition_mode: TweakMode) -> bool {
    match requested_mode {
        TweakMode::Safe => definition_mode == TweakMode::Safe,
        TweakMode::Competitive => {
            matches!(definition_mode, TweakMode::Safe | TweakMode::Competitive)
        }
        TweakMode::Lab => matches!(
            definition_mode,
            TweakMode::Safe | TweakMode::Competitive | TweakMode::Lab
        ),
        TweakMode::Blocked => definition_mode == TweakMode::Blocked,
    }
}

fn definition_warnings(definition: &TweakDefinition) -> Vec<String> {
    let mut warnings = Vec::new();

    warnings.extend(definition.known_side_effects.iter().cloned());
    warnings.extend(definition.anti_cheat_notes.iter().cloned());

    if definition.measurement_plan.benchmark_required {
        warnings.push("Benchmark required before recommendation.".to_owned());
    }

    if definition.game_closed_required {
        warnings.push("Game must be closed before apply.".to_owned());
    }

    if definition.requires_admin {
        warnings.push("Administrator privileges required for apply.".to_owned());
    }

    if definition.reboot == RebootPolicy::Required {
        warnings.push("Reboot required before verification can complete.".to_owned());
    }

    warnings
}

fn applied_conflicts(
    definition: &TweakDefinition,
    applied_definitions: &BTreeMap<TweakId, &TweakDefinition>,
) -> Vec<TweakId> {
    applied_definitions
        .iter()
        .filter_map(|(applied_id, applied_definition)| {
            if definition.conflicts_with.contains(applied_id)
                || applied_definition.conflicts_with.contains(&definition.id)
            {
                Some(applied_id.clone())
            } else {
                None
            }
        })
        .collect()
}

fn planned_changes(definition: &TweakDefinition, action: PlanAction) -> Vec<PlannedChange> {
    if action == PlanAction::DetectOnly {
        return Vec::new();
    }

    if action == PlanAction::Deny {
        return denied_changes(definition);
    }

    definition
        .apply
        .operations
        .iter()
        .map(|operation| PlannedChange {
            target: operation.target.clone(),
            operation: operation.kind,
            previous_value: None,
            desired_value: operation.value.clone(),
            scope: definition.session_scope,
        })
        .collect()
}

fn denied_changes(definition: &TweakDefinition) -> Vec<PlannedChange> {
    if definition.apply.operations.is_empty() {
        return vec![PlannedChange {
            target: definition.id.clone(),
            operation: TweakOperationKind::Deny,
            previous_value: None,
            desired_value: None,
            scope: SessionScope::Blocked,
        }];
    }

    definition
        .apply
        .operations
        .iter()
        .map(|operation| PlannedChange {
            target: operation.target.clone(),
            operation: TweakOperationKind::Deny,
            previous_value: None,
            desired_value: None,
            scope: SessionScope::Blocked,
        })
        .collect()
}

fn backup_requirement(
    definition: &TweakDefinition,
    action: PlanAction,
    changes: &[PlannedChange],
) -> BackupRequirement {
    if action == PlanAction::Apply
        && definition.is_mutable()
        && definition.rollback_kind.needs_backup_before_apply()
    {
        BackupRequirement::Required {
            kind: definition.rollback_kind,
            target: changes
                .first()
                .map_or_else(|| definition.id.clone(), |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(definition: &TweakDefinition) -> RollbackPlan {
    if definition.rollback_kind == RollbackKind::NotNeededReadonly {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: definition.rollback_kind,
        steps: definition
            .rollback
            .operations
            .iter()
            .map(|operation| RollbackStep {
                summary: definition.rollback.summary.clone(),
                target: operation.target.clone(),
                operation: operation.kind,
                expected_state: operation.value.clone(),
            })
            .collect(),
        requires_admin: definition.requires_admin,
        reboot: definition.reboot,
        manual_instructions: if definition.rollback_kind == RollbackKind::ManualInstructions {
            vec![definition.rollback.summary.clone()]
        } else {
            Vec::new()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog::{
            load_tweak_catalog, CatalogSource, TweakCatalog, SUPPORTED_CATALOG_SCHEMA_VERSION,
        },
        tweak_contracts::{
            CompatibilityRule, CompatibilityTarget, EvidenceLevel, ExpectedImpact,
            ImpactDirection, LaptopPolicy, MeasurementPlan, PowerSourcePolicy, SourceLink,
            TweakCategory, TweakOperation, TweakOperationKind, TweakRisk, TweakStep,
            TweakStepKind, TweakTestPlan,
        },
    };

    fn registry(definitions: Vec<TweakDefinition>) -> TweakRegistry {
        load_tweak_catalog(TweakCatalog::new(
            SUPPORTED_CATALOG_SCHEMA_VERSION,
            CatalogSource::embedded("catalog:embedded:v1"),
            definitions,
        ))
        .expect("fixture catalog should load")
    }

    fn operation(target: &str, value: &str) -> TweakOperation {
        TweakOperation {
            kind: TweakOperationKind::Write,
            target: target.to_owned(),
            value: Some(value.to_owned()),
        }
    }

    fn source_links() -> Vec<SourceLink> {
        vec![SourceLink {
            title: "Tweak definition standard".to_owned(),
            url: "local:tweak-definition-standard".to_owned(),
            evidence: EvidenceLevel::Official,
        }]
    }

    fn base_definition(id: &str) -> TweakDefinition {
        let read_detect = TweakStep::read_only(TweakStepKind::Detect, "Read current state.");
        let read_precheck = TweakStep::read_only(TweakStepKind::Precheck, "Check prerequisites.");
        let read_plan = TweakStep::read_only(TweakStepKind::Plan, "Build dry-run plan.");
        let read_verify = TweakStep::read_only(TweakStepKind::Verify, "Verify desired state.");

        TweakDefinition {
            id: id.to_owned(),
            title: id.to_owned(),
            summary: "Fixture tweak.".to_owned(),
            category: TweakCategory::WindowsGaming,
            mode: TweakMode::Safe,
            risk: TweakRisk::Low,
            default_enabled: true,
            session_scope: SessionScope::Persistent,
            rollback_kind: RollbackKind::ExactValue,
            requires_admin: false,
            reboot: RebootPolicy::None,
            supported_os: vec!["windows-10+".to_owned()],
            supported_hardware: vec![CompatibilityRule {
                target: CompatibilityTarget::Windows,
                expression: "windows-10+".to_owned(),
                reason: "Fixture target.".to_owned(),
            }],
            supported_drivers: Vec::new(),
            unsupported_when: Vec::new(),
            conflicts_with: Vec::new(),
            laptop_policy: LaptopPolicy::SameAsDesktop,
            power_source_policy: PowerSourcePolicy::Any,
            source_links: source_links(),
            evidence_level: EvidenceLevel::Official,
            measurement_plan: MeasurementPlan {
                benchmark_required: false,
                metrics: Vec::new(),
                notes: Vec::new(),
            },
            expected_impact: ExpectedImpact {
                metric: "frametime consistency".to_owned(),
                direction: ImpactDirection::Stabilize,
                evidence: EvidenceLevel::Official,
                summary: "Fixture impact.".to_owned(),
            },
            known_side_effects: Vec::new(),
            anti_cheat_notes: Vec::new(),
            game_closed_required: false,
            user_disclosure: "Fixture disclosure.".to_owned(),
            r#do: vec!["Plan exact state.".to_owned()],
            dont: vec!["Do not mutate during dry run.".to_owned()],
            detect: read_detect,
            precheck: read_precheck,
            plan: read_plan,
            backup: TweakStep {
                kind: TweakStepKind::Backup,
                summary: "Back up previous value.".to_owned(),
                operations: vec![TweakOperation {
                    kind: TweakOperationKind::Read,
                    target: format!("backup:{id}"),
                    value: None,
                }],
                mutates_system: false,
            },
            apply: TweakStep {
                kind: TweakStepKind::Apply,
                summary: "Write desired value.".to_owned(),
                operations: vec![operation(&format!("target:{id}"), "enabled")],
                mutates_system: true,
            },
            verify: read_verify,
            rollback: TweakStep {
                kind: TweakStepKind::Rollback,
                summary: "Restore previous value.".to_owned(),
                operations: vec![operation(&format!("target:{id}"), "previous")],
                mutates_system: true,
            },
            tests: TweakTestPlan {
                cases: Vec::new(),
                fixtures: Vec::new(),
                requires_live_windows: false,
            },
        }
    }

    #[test]
    fn builds_dry_run_plan_with_backup_before_apply_contract() {
        let registry = registry(vec![base_definition("game.capture.off")]);
        let request = DryRunPlanRequest::all("plan-001", TweakMode::Safe);

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");

        assert_eq!(plan.id, "plan-001");
        assert_eq!(plan.catalog_schema_version, SUPPORTED_CATALOG_SCHEMA_VERSION);
        assert_eq!(plan.items.len(), 1);

        let item = &plan.items[0];
        assert_eq!(item.action, PlanAction::Apply);
        assert_eq!(
            item.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: "target:game.capture.off".to_owned()
            }
        );
        assert_eq!(item.changes[0].target, "target:game.capture.off");
        assert_eq!(item.rollback.steps[0].expected_state.as_deref(), Some("previous"));
    }

    #[test]
    fn orders_dependencies_before_dependents() {
        let registry = registry(vec![
            base_definition("windows.capture.disable"),
            base_definition("pubg.capture.profile"),
        ]);
        let request = DryRunPlanRequest::selected(
            "plan-ordered",
            TweakMode::Safe,
            vec![
                "pubg.capture.profile".to_owned(),
                "windows.capture.disable".to_owned(),
            ],
        )
        .with_dependency(TweakDependency::new(
            "pubg.capture.profile",
            "windows.capture.disable",
            "Profile assumes Windows capture is planned first.",
        ));

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");

        assert_eq!(
            plan.items
                .iter()
                .map(|item| item.tweak_id.as_str())
                .collect::<Vec<_>>(),
            vec!["windows.capture.disable", "pubg.capture.profile"]
        );
    }

    #[test]
    fn reports_dependency_cycles() {
        let registry = registry(vec![base_definition("a"), base_definition("b")]);
        let request = DryRunPlanRequest::selected(
            "plan-cycle",
            TweakMode::Safe,
            vec!["a".to_owned(), "b".to_owned()],
        )
        .with_dependency(TweakDependency::new("a", "b", "fixture"))
        .with_dependency(TweakDependency::new("b", "a", "fixture"));

        let error = build_dry_run_plan(&registry, &request).expect_err("cycle should fail");

        assert_eq!(error.reason(), PlanBuildErrorReason::DependencyCycle);
    }

    #[test]
    fn safe_plan_recommends_competitive_and_denies_blocked_guardrails() {
        let mut competitive = base_definition("nvidia.low-latency");
        competitive.mode = TweakMode::Competitive;
        competitive.default_enabled = false;

        let mut blocked = base_definition("blocked.defender.disable");
        blocked.mode = TweakMode::Blocked;
        blocked.risk = TweakRisk::Critical;
        blocked.session_scope = SessionScope::Blocked;
        blocked.default_enabled = false;

        let registry = registry(vec![competitive, blocked]);
        let request = DryRunPlanRequest::all("plan-safe", TweakMode::Safe);

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");

        assert_eq!(plan.items[0].action, PlanAction::Deny);
        assert_eq!(plan.items[0].changes[0].operation, TweakOperationKind::Deny);
        assert_eq!(plan.items[0].changes[0].scope, SessionScope::Blocked);
        assert_eq!(plan.items[1].action, PlanAction::Recommend);
        assert!(plan.has_denials());
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn safe_plan_excludes_competitive_and_lab_modes_from_apply() {
        let safe = base_definition("game.capture.background.off");

        let mut competitive = base_definition("nvidia.low-latency.on");
        competitive.mode = TweakMode::Competitive;
        competitive.default_enabled = true;

        let mut lab = base_definition("nvidia.rebar.hidden-override");
        lab.mode = TweakMode::Lab;
        lab.default_enabled = true;

        let registry = registry(vec![safe, competitive, lab]);
        let request = DryRunPlanRequest::all("plan-safe", TweakMode::Safe);

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");

        assert_eq!(plan.items[0].action, PlanAction::Apply);
        assert_eq!(plan.items[1].action, PlanAction::Recommend);
        assert_eq!(plan.items[2].action, PlanAction::Recommend);
        assert!(plan.items[1]
            .warnings
            .iter()
            .any(|warning| warning.contains("requires explicit opt-in")));
        assert!(plan.items[2]
            .warnings
            .iter()
            .any(|warning| warning.contains("requires explicit opt-in")));
    }

    #[test]
    fn blocked_guardrail_without_apply_operations_still_has_denial_target() {
        let mut blocked = base_definition("blocked.bulk-reg-pack");
        blocked.mode = TweakMode::Blocked;
        blocked.session_scope = SessionScope::Blocked;
        blocked.apply.operations.clear();
        blocked.apply.mutates_system = false;
        blocked.default_enabled = false;

        let registry = registry(vec![blocked]);
        let request = DryRunPlanRequest::all("plan-safe", TweakMode::Safe);

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");
        let item = &plan.items[0];

        assert_eq!(item.action, PlanAction::Deny);
        assert_eq!(item.changes.len(), 1);
        assert_eq!(item.changes[0].target, "blocked.bulk-reg-pack");
        assert_eq!(item.changes[0].operation, TweakOperationKind::Deny);
        assert_eq!(item.changes[0].scope, SessionScope::Blocked);
    }

    #[test]
    fn conflicting_apply_items_downgrade_later_item_to_recommendation() {
        let mut first = base_definition("capture.disable");
        first.conflicts_with.push("capture.enable".to_owned());

        let second = base_definition("capture.enable");
        let registry = registry(vec![first, second]);
        let request = DryRunPlanRequest::selected(
            "plan-conflicts",
            TweakMode::Safe,
            vec!["capture.disable".to_owned(), "capture.enable".to_owned()],
        );

        let plan = build_dry_run_plan(&registry, &request).expect("plan should build");

        assert_eq!(plan.items[0].action, PlanAction::Apply);
        assert_eq!(plan.items[1].action, PlanAction::Recommend);
        assert_eq!(plan.warnings.len(), 1);
        assert!(plan.warnings[0].contains("capture.enable conflicts"));
    }
}
