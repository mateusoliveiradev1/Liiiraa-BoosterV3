//! Competitive planning for MMCSS and foreground scheduler registry tweaks.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for MMCSS `SystemResponsiveness`.
pub const WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID: &str =
    "win.mmcss.system-responsiveness";
/// Tweak ID for foreground scheduler quantum boost.
pub const WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID: &str =
    "win.scheduler.foreground-boost";

/// Logical target for MMCSS `SystemResponsiveness`.
pub const TARGET_MMCSS_SYSTEM_RESPONSIVENESS: &str = concat!(
    "registry:hklm/software/microsoft/windows-nt/currentversion/multimedia/",
    "systemprofile/systemresponsiveness"
);
/// Logical target for `Win32PrioritySeparation`.
pub const TARGET_WIN32_PRIORITY_SEPARATION: &str =
    "registry:hklm/system/currentcontrolset/control/prioritycontrol/win32priorityseparation";

/// Desired MMCSS `SystemResponsiveness` value for the Competitive benchmark path.
pub const DESIRED_MMCSS_SYSTEM_RESPONSIVENESS: u32 = 10;
/// Desired `Win32PrioritySeparation` value for foreground boost benchmarking.
pub const DESIRED_WIN32_PRIORITY_SEPARATION: u32 = 38;

const BENCHMARK_WARNING: &str = concat!(
    "Baseline benchmark is required before applying scheduler tweaks; compare frametime ",
    "stability before and after instead of assuming universal FPS gains."
);

/// Current registry state for one scheduler DWORD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerRegistryDwordState {
    /// The registry value exists and was read.
    Value(u32),
    /// The registry value is absent.
    Missing,
    /// The scan could not prove the value.
    Unknown,
}

impl SchedulerRegistryDwordState {
    /// Converts an optional registry DWORD into a conservative state.
    #[must_use]
    pub const fn from_option(value: Option<u32>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Unknown,
        }
    }

    const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    fn matches_desired(self, desired: u32) -> bool {
        matches!(self, Self::Value(value) if value == desired)
    }

    fn previous_value(self) -> Option<String> {
        match self {
            Self::Value(value) => Some(value.to_string()),
            Self::Missing | Self::Unknown => None,
        }
    }

    const fn rollback_kind(self) -> RollbackKind {
        match self {
            Self::Value(_) => RollbackKind::ExactValue,
            Self::Missing => RollbackKind::DeleteCreatedValue,
            Self::Unknown => RollbackKind::NotNeededReadonly,
        }
    }
}

/// Explicit consent state for Competitive scheduler tweaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerControlConsent {
    /// The user has not accepted this scheduler tweak.
    NotGranted,
    /// The user explicitly accepted this scheduler tweak.
    Granted,
}

impl SchedulerControlConsent {
    fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Request used to build the T051 scheduler Competitive plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerCompetitivePlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Current MMCSS `SystemResponsiveness` registry state.
    pub mmcss_system_responsiveness: SchedulerRegistryDwordState,
    /// Current `Win32PrioritySeparation` registry state.
    pub win32_priority_separation: SchedulerRegistryDwordState,
    /// Consent for the MMCSS tweak.
    pub mmcss_consent: SchedulerControlConsent,
    /// Consent for the foreground scheduler tweak.
    pub foreground_boost_consent: SchedulerControlConsent,
    /// Whether a baseline benchmark exists before applying the tweak.
    pub baseline_benchmark_captured: bool,
}

impl SchedulerCompetitivePlanRequest {
    /// Creates a conservative scheduler request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            mmcss_system_responsiveness: SchedulerRegistryDwordState::Unknown,
            win32_priority_separation: SchedulerRegistryDwordState::Unknown,
            mmcss_consent: SchedulerControlConsent::NotGranted,
            foreground_boost_consent: SchedulerControlConsent::NotGranted,
            baseline_benchmark_captured: false,
        }
    }
}

/// Builds a dry-run plan for T051 scheduler Competitive tweaks.
#[must_use]
pub fn build_scheduler_competitive_plan(
    request: &SchedulerCompetitivePlanRequest,
) -> TweakPlan {
    let items = vec![
        mmcss_system_responsiveness_item(request),
        foreground_boost_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Competitive")
                || warning.contains("consent")
                || warning.contains("benchmark")
                || warning.contains("unknown")
        })
        .cloned()
        .collect();

    TweakPlan {
        id: request.plan_id.clone(),
        requested_mode: request.requested_mode,
        catalog_schema_version: SUPPORTED_CATALOG_SCHEMA_VERSION.to_owned(),
        items,
        warnings,
    }
}

/// Returns true when the ID belongs to the T051 scheduler scope.
#[must_use]
pub fn is_scheduler_competitive_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID
            | WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID
    )
}

/// Returns true when a logical registry target belongs to the T051 scheduler scope.
#[must_use]
pub fn is_scheduler_registry_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_MMCSS_SYSTEM_RESPONSIVENESS | TARGET_WIN32_PRIORITY_SEPARATION
    )
}

/// Returns true when a tweak ID is paired with its exact scheduler target.
#[must_use]
pub fn scheduler_tweak_targets_registry_value(tweak_id: &str, target: &str) -> bool {
    matches!(
        (tweak_id, target),
        (WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID, TARGET_MMCSS_SYSTEM_RESPONSIVENESS)
            | (WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID, TARGET_WIN32_PRIORITY_SEPARATION)
    )
}

/// Returns true when Safe/default planning does not apply scheduler registry changes.
#[must_use]
pub fn scheduler_plan_is_not_safe_default(plan: &TweakPlan) -> bool {
    plan.requested_mode != TweakMode::Safe
        || plan.items.iter().all(|item| item.action != PlanAction::Apply)
}

/// Returns true when apply items are Competitive, consented, and benchmark-framed.
#[must_use]
pub fn scheduler_plan_requires_explicit_consent_and_benchmark(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if item.action != PlanAction::Apply {
            return true;
        }

        item.mode == TweakMode::Competitive
            && item.risk == TweakRisk::Medium
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit consent"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("Baseline benchmark"))
    })
}

fn mmcss_system_responsiveness_item(
    request: &SchedulerCompetitivePlanRequest,
) -> TweakPlanItem {
    scheduler_item(
        SchedulerItemInput {
            tweak_id: WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID,
            target: TARGET_MMCSS_SYSTEM_RESPONSIVENESS,
            current: request.mmcss_system_responsiveness,
            desired: DESIRED_MMCSS_SYSTEM_RESPONSIVENESS,
            consent: request.mmcss_consent,
            summary: "MMCSS SystemResponsiveness",
        },
        request,
    )
}

fn foreground_boost_item(request: &SchedulerCompetitivePlanRequest) -> TweakPlanItem {
    scheduler_item(
        SchedulerItemInput {
            tweak_id: WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID,
            target: TARGET_WIN32_PRIORITY_SEPARATION,
            current: request.win32_priority_separation,
            desired: DESIRED_WIN32_PRIORITY_SEPARATION,
            consent: request.foreground_boost_consent,
            summary: "Win32PrioritySeparation foreground boost",
        },
        request,
    )
}

#[derive(Clone, Copy)]
struct SchedulerItemInput<'a> {
    tweak_id: &'a str,
    target: &'a str,
    current: SchedulerRegistryDwordState,
    desired: u32,
    consent: SchedulerControlConsent,
    summary: &'a str,
}

fn scheduler_item(
    input: SchedulerItemInput<'_>,
    request: &SchedulerCompetitivePlanRequest,
) -> TweakPlanItem {
    let changes = if input.current.matches_desired(input.desired) || input.current.is_unknown() {
        Vec::new()
    } else {
        vec![PlannedChange {
            target: input.target.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: input.current.previous_value(),
            desired_value: Some(input.desired.to_string()),
            scope: SessionScope::Persistent,
        }]
    };
    let warnings = scheduler_warnings(request, input);
    let action = scheduler_action(request, input, changes.is_empty());
    let rollback_kind = input.current.rollback_kind();
    let backup = backup_requirement(action, rollback_kind, input.target);
    let rollback = rollback_plan(action, rollback_kind, &changes);

    TweakPlanItem {
        tweak_id: input.tweak_id.to_owned(),
        category: TweakCategory::PowerAndLatency,
        action,
        mode: TweakMode::Competitive,
        risk: TweakRisk::Medium,
        changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin: true,
        warnings,
    }
}

fn scheduler_action(
    request: &SchedulerCompetitivePlanRequest,
    input: SchedulerItemInput<'_>,
    no_changes: bool,
) -> PlanAction {
    if input.current.is_unknown() || no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode == TweakMode::Safe
        || !input.consent.is_granted()
        || !request.baseline_benchmark_captured
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn scheduler_warnings(
    request: &SchedulerCompetitivePlanRequest,
    input: SchedulerItemInput<'_>,
) -> Vec<String> {
    let mut warnings = vec![
        format!("{} requires explicit consent.", input.summary),
        BENCHMARK_WARNING.to_owned(),
        "Scheduler tweaks affect global Windows scheduling state, not a per-game profile."
            .to_owned(),
        "No game, BattlEye, or anti-cheat files or processes are modified.".to_owned(),
    ];

    if request.requested_mode == TweakMode::Safe {
        warnings.push("Scheduler registry tweaks are Competitive and stay off in Safe mode.".to_owned());
    }

    if !input.consent.is_granted() {
        warnings.push(format!("{} consent has not been granted.", input.summary));
    }

    if !request.baseline_benchmark_captured {
        warnings.push("Capture a baseline benchmark before applying this tweak.".to_owned());
    }

    match input.current {
        SchedulerRegistryDwordState::Unknown => {
            warnings.push("Current registry value is unknown; inspect and back it up before apply.".to_owned());
        }
        SchedulerRegistryDwordState::Missing => {
            warnings.push("Registry value is missing; rollback will delete the created value.".to_owned());
        }
        SchedulerRegistryDwordState::Value(value) if value == input.desired => {
            warnings.push("Registry value already matches the Competitive target.".to_owned());
        }
        SchedulerRegistryDwordState::Value(_) => {}
    }

    warnings
}

fn backup_requirement(
    action: PlanAction,
    rollback_kind: RollbackKind,
    target: &str,
) -> BackupRequirement {
    if action == PlanAction::Apply && rollback_kind.needs_backup_before_apply() {
        BackupRequirement::Required {
            kind: rollback_kind,
            target: target.to_owned(),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(
    action: PlanAction,
    rollback_kind: RollbackKind,
    changes: &[PlannedChange],
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: rollback_kind,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous scheduler registry value.".to_owned(),
                target: change.target.clone(),
                operation: if rollback_kind == RollbackKind::DeleteCreatedValue {
                    TweakOperationKind::Delete
                } else {
                    TweakOperationKind::Write
                },
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin: true,
        reboot: RebootPolicy::None,
        manual_instructions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SchedulerCompetitivePlanRequest {
        let mut request = SchedulerCompetitivePlanRequest::new("plan-scheduler");
        request.mmcss_system_responsiveness = SchedulerRegistryDwordState::Value(20);
        request.win32_priority_separation = SchedulerRegistryDwordState::Value(2);
        request
    }

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn safe_plan_recommends_scheduler_tweaks_without_apply() {
        let mut request = request();
        request.mmcss_consent = SchedulerControlConsent::Granted;
        request.foreground_boost_consent = SchedulerControlConsent::Granted;
        request.baseline_benchmark_captured = true;

        let plan = build_scheduler_competitive_plan(&request);
        let mmcss = item(&plan, WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID);

        assert_eq!(mmcss.action, PlanAction::Recommend);
        assert_eq!(mmcss.mode, TweakMode::Competitive);
        assert_eq!(mmcss.backup, BackupRequirement::NotRequired);
        assert!(!plan.has_apply_items());
        assert!(scheduler_plan_is_not_safe_default(&plan));
    }

    #[test]
    fn competitive_apply_requires_consent_and_baseline_benchmark() {
        let mut request = request();
        request.requested_mode = TweakMode::Competitive;

        let no_consent_plan = build_scheduler_competitive_plan(&request);
        assert_eq!(
            item(&no_consent_plan, WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.mmcss_consent = SchedulerControlConsent::Granted;
        request.foreground_boost_consent = SchedulerControlConsent::Granted;
        let no_baseline_plan = build_scheduler_competitive_plan(&request);
        assert_eq!(
            item(&no_baseline_plan, WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.baseline_benchmark_captured = true;
        let apply_plan = build_scheduler_competitive_plan(&request);
        let foreground = item(&apply_plan, WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID);

        assert_eq!(foreground.action, PlanAction::Apply);
        assert_eq!(
            foreground.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_WIN32_PRIORITY_SEPARATION.to_owned(),
            }
        );
        assert_eq!(foreground.changes[0].desired_value.as_deref(), Some("38"));
        assert!(scheduler_plan_requires_explicit_consent_and_benchmark(&apply_plan));
    }

    #[test]
    fn missing_registry_value_uses_delete_created_rollback() {
        let mut request = request();
        request.requested_mode = TweakMode::Competitive;
        request.mmcss_system_responsiveness = SchedulerRegistryDwordState::Missing;
        request.win32_priority_separation = SchedulerRegistryDwordState::Value(38);
        request.mmcss_consent = SchedulerControlConsent::Granted;
        request.baseline_benchmark_captured = true;

        let plan = build_scheduler_competitive_plan(&request);
        let mmcss = item(&plan, WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID);

        assert_eq!(mmcss.action, PlanAction::Apply);
        assert_eq!(
            mmcss.backup,
            BackupRequirement::Required {
                kind: RollbackKind::DeleteCreatedValue,
                target: TARGET_MMCSS_SYSTEM_RESPONSIVENESS.to_owned(),
            }
        );
        assert_eq!(mmcss.rollback.kind, RollbackKind::DeleteCreatedValue);
        assert_eq!(mmcss.rollback.steps[0].operation, TweakOperationKind::Delete);
    }

    #[test]
    fn unknown_or_matching_registry_values_do_not_apply() {
        let mut request = request();
        request.requested_mode = TweakMode::Competitive;
        request.mmcss_system_responsiveness = SchedulerRegistryDwordState::Unknown;
        request.win32_priority_separation =
            SchedulerRegistryDwordState::Value(DESIRED_WIN32_PRIORITY_SEPARATION);
        request.mmcss_consent = SchedulerControlConsent::Granted;
        request.foreground_boost_consent = SchedulerControlConsent::Granted;
        request.baseline_benchmark_captured = true;

        let plan = build_scheduler_competitive_plan(&request);

        assert_eq!(
            item(&plan, WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, WIN_SCHEDULER_FOREGROUND_BOOST_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert!(!plan.has_apply_items());
    }
}
