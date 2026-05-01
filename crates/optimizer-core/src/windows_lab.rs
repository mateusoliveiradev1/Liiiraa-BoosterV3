//! Lab-only Windows experiments for timer resolution and memory compression.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for a session-scoped timer resolution experiment.
pub const WIN_TIMER_RESOLUTION_LAB_TWEAK_ID: &str = "win.timer-resolution.lab";
/// Tweak ID for a memory compression Lab experiment.
pub const WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID: &str = "win.memory-compression.lab";

/// Logical target for the session timer resolution request.
pub const TARGET_TIMER_RESOLUTION_SESSION: &str = "windows:timer-resolution/session";
/// Logical target for Windows Memory Manager compression policy.
pub const TARGET_MEMORY_COMPRESSION: &str = "powershell:mmagent/memory-compression";

const DESIRED_TIMER_RESOLUTION_STATE: &str = "1ms-session";
const DESIRED_MEMORY_COMPRESSION_STATE: &str = "disabled";
const LAB_BENCHMARK_WARNING: &str = concat!(
    "Baseline benchmark is required before applying Windows Lab experiments; compare ",
    "frametime stability, latency-sensitive workload behavior, thermals, and idle power before ",
    "and after."
);
const RESTORE_POINT_WARNING: &str =
    "Create or confirm a restore point and capture backups before applying this Lab experiment.";

/// Explicit consent state for Lab-only Windows experiments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsLabConsent {
    /// The user has not accepted the Lab experiment.
    NotGranted,
    /// The user explicitly accepted the Lab experiment.
    Granted,
}

impl WindowsLabConsent {
    /// Returns true when consent was granted.
    #[must_use]
    pub const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Current timer resolution state known to the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerResolutionState {
    /// No Liiiraa timer resolution request is active.
    Default,
    /// A low-latency timer request is already active.
    Requested,
    /// The adapter could not determine timer resolution state.
    Unknown,
}

impl TimerResolutionState {
    const fn needs_session_request(self) -> bool {
        matches!(self, Self::Default)
    }

    const fn previous_value(self) -> Option<&'static str> {
        match self {
            Self::Default => Some("default"),
            Self::Requested => Some(DESIRED_TIMER_RESOLUTION_STATE),
            Self::Unknown => None,
        }
    }
}

/// Current memory compression state known to the planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryCompressionState {
    /// Windows memory compression appears enabled.
    Enabled,
    /// Windows memory compression appears disabled.
    Disabled,
    /// The adapter could not determine memory compression state.
    Unknown,
}

impl MemoryCompressionState {
    const fn needs_disable_experiment(self) -> bool {
        matches!(self, Self::Enabled)
    }

    const fn previous_value(self) -> Option<&'static str> {
        match self {
            Self::Enabled => Some("enabled"),
            Self::Disabled => Some("disabled"),
            Self::Unknown => None,
        }
    }
}

/// Request used to build the T056 Windows Lab experiment plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsLabExperimentPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Current timer resolution state.
    pub timer_resolution_state: TimerResolutionState,
    /// Current memory compression state.
    pub memory_compression_state: MemoryCompressionState,
    /// Consent for the timer resolution experiment.
    pub timer_resolution_consent: WindowsLabConsent,
    /// Consent for the memory compression experiment.
    pub memory_compression_consent: WindowsLabConsent,
    /// Whether a baseline benchmark exists before applying the experiment.
    pub baseline_benchmark_captured: bool,
    /// Whether a restore point or equivalent rollback checkpoint is confirmed.
    pub restore_point_confirmed: bool,
    /// Whether the user accepted that timer resolution is session-scoped.
    pub session_boundary_accepted: bool,
    /// Whether memory headroom was confirmed before changing compression.
    pub memory_headroom_confirmed: bool,
}

impl WindowsLabExperimentPlanRequest {
    /// Creates a conservative Windows Lab request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            timer_resolution_state: TimerResolutionState::Unknown,
            memory_compression_state: MemoryCompressionState::Unknown,
            timer_resolution_consent: WindowsLabConsent::NotGranted,
            memory_compression_consent: WindowsLabConsent::NotGranted,
            baseline_benchmark_captured: false,
            restore_point_confirmed: false,
            session_boundary_accepted: false,
            memory_headroom_confirmed: false,
        }
    }
}

/// Builds a dry-run plan for T056 Windows Lab experiments.
#[must_use]
pub fn build_windows_lab_experiment_plan(
    request: &WindowsLabExperimentPlanRequest,
) -> TweakPlan {
    let items = vec![
        timer_resolution_lab_item(request),
        memory_compression_lab_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Lab")
                || warning.contains("benchmark")
                || warning.contains("consent")
                || warning.contains("restore point")
                || warning.contains("Safe/default")
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

/// Returns true when the ID belongs to the T056 Windows Lab scope.
#[must_use]
pub fn is_windows_lab_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        WIN_TIMER_RESOLUTION_LAB_TWEAK_ID | WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID
    )
}

/// Returns true when the target belongs to the T056 Windows Lab scope.
#[must_use]
pub fn is_windows_lab_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_TIMER_RESOLUTION_SESSION | TARGET_MEMORY_COMPRESSION
    )
}

/// Returns true when a T056 tweak ID is paired with its allowed target.
#[must_use]
pub fn windows_lab_tweak_targets_value(tweak_id: &str, target: &str) -> bool {
    matches!(
        (tweak_id, target),
        (WIN_TIMER_RESOLUTION_LAB_TWEAK_ID, TARGET_TIMER_RESOLUTION_SESSION)
            | (WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID, TARGET_MEMORY_COMPRESSION)
    )
}

/// Returns true when Safe/default planning does not apply Windows Lab changes.
#[must_use]
pub fn windows_lab_plan_is_not_safe_default(plan: &TweakPlan) -> bool {
    plan.requested_mode != TweakMode::Safe
        || plan.items.iter().all(|item| item.action != PlanAction::Apply)
}

/// Returns true when apply items are Lab, explicitly consented, and benchmark-framed.
#[must_use]
pub fn windows_lab_apply_requires_opt_in(plan: &TweakPlan) -> bool {
    if plan.has_apply_items() && plan.requested_mode != TweakMode::Lab {
        return false;
    }

    plan.items.iter().all(|item| {
        if item.action != PlanAction::Apply {
            return true;
        }

        item.mode == TweakMode::Lab
            && item.risk == TweakRisk::High
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit consent"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("Baseline benchmark"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("restore point"))
    })
}

fn timer_resolution_lab_item(request: &WindowsLabExperimentPlanRequest) -> TweakPlanItem {
    let changes = if request.timer_resolution_state.needs_session_request() {
        vec![PlannedChange {
            target: TARGET_TIMER_RESOLUTION_SESSION.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: request
                .timer_resolution_state
                .previous_value()
                .map(str::to_owned),
            desired_value: Some(DESIRED_TIMER_RESOLUTION_STATE.to_owned()),
            scope: SessionScope::SessionOnly,
        }]
    } else {
        Vec::new()
    };
    let warnings = timer_resolution_warnings(request, changes.is_empty());
    let action = lab_action(
        request,
        request.timer_resolution_consent,
        changes.is_empty(),
        LabGateKind::TimerSession,
    );

    lab_plan_item(LabPlanItemInput {
        tweak_id: WIN_TIMER_RESOLUTION_LAB_TWEAK_ID,
        category: TweakCategory::PowerAndLatency,
        action,
        changes,
        requires_admin: false,
        reboot: RebootPolicy::None,
        warnings,
    })
}

fn memory_compression_lab_item(request: &WindowsLabExperimentPlanRequest) -> TweakPlanItem {
    let changes = if request
        .memory_compression_state
        .needs_disable_experiment()
    {
        vec![PlannedChange {
            target: TARGET_MEMORY_COMPRESSION.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: request
                .memory_compression_state
                .previous_value()
                .map(str::to_owned),
            desired_value: Some(DESIRED_MEMORY_COMPRESSION_STATE.to_owned()),
            scope: SessionScope::Persistent,
        }]
    } else {
        Vec::new()
    };
    let warnings = memory_compression_warnings(request, changes.is_empty());
    let action = lab_action(
        request,
        request.memory_compression_consent,
        changes.is_empty(),
        LabGateKind::MemoryCompression,
    );

    lab_plan_item(LabPlanItemInput {
        tweak_id: WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID,
        category: TweakCategory::BackgroundWork,
        action,
        changes,
        requires_admin: true,
        reboot: RebootPolicy::Recommended,
        warnings,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabGateKind {
    TimerSession,
    MemoryCompression,
}

fn lab_action(
    request: &WindowsLabExperimentPlanRequest,
    consent: WindowsLabConsent,
    no_changes: bool,
    kind: LabGateKind,
) -> PlanAction {
    if no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode != TweakMode::Lab
        || !consent.is_granted()
        || !request.baseline_benchmark_captured
        || !request.restore_point_confirmed
        || (kind == LabGateKind::TimerSession && !request.session_boundary_accepted)
        || (kind == LabGateKind::MemoryCompression && !request.memory_headroom_confirmed)
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn timer_resolution_warnings(
    request: &WindowsLabExperimentPlanRequest,
    no_changes: bool,
) -> Vec<String> {
    let mut warnings = base_lab_warnings(
        request,
        request.timer_resolution_consent,
        "Timer resolution",
    );
    warnings.push(
        "Timer resolution is session-scoped; Liiiraa must release the request when the Lab run ends."
            .to_owned(),
    );
    warnings.push(
        "Permanent timer services and BCD timer packs are blocked outside this guarded Lab path."
            .to_owned(),
    );

    if !request.session_boundary_accepted {
        warnings.push("Session boundary and automatic release have not been accepted.".to_owned());
    }

    if no_changes {
        warnings.push(
            "Timer resolution state is unknown or already requested; no apply step was planned."
                .to_owned(),
        );
    }

    warnings
}

fn memory_compression_warnings(
    request: &WindowsLabExperimentPlanRequest,
    no_changes: bool,
) -> Vec<String> {
    let mut warnings = base_lab_warnings(
        request,
        request.memory_compression_consent,
        "Memory compression",
    );
    warnings.push(
        "Memory compression changes are global Windows memory-manager experiments, not default FPS tweaks."
            .to_owned(),
    );

    if !request.memory_headroom_confirmed {
        warnings.push(
            "Stable memory headroom has not been confirmed for this compression experiment."
                .to_owned(),
        );
    }

    if no_changes {
        warnings.push(
            "Memory compression state is unknown or already matches the Lab target; no apply step was planned."
                .to_owned(),
        );
    }

    warnings
}

fn base_lab_warnings(
    request: &WindowsLabExperimentPlanRequest,
    consent: WindowsLabConsent,
    summary: &str,
) -> Vec<String> {
    let mut warnings = vec![
        format!("{summary} is Lab-only and requires explicit consent."),
        LAB_BENCHMARK_WARNING.to_owned(),
        RESTORE_POINT_WARNING.to_owned(),
        "No game files, game memory, BattlEye files, or anti-cheat processes are modified."
            .to_owned(),
    ];

    if request.requested_mode != TweakMode::Lab {
        warnings.push("Windows Lab experiments stay off in Safe/default planning.".to_owned());
    }

    if !consent.is_granted() {
        warnings.push(format!("{summary} consent has not been granted."));
    }

    if !request.baseline_benchmark_captured {
        warnings.push("Capture a baseline benchmark before applying this Lab experiment.".to_owned());
    }

    if !request.restore_point_confirmed {
        warnings.push("Restore point confirmation is required before apply.".to_owned());
    }

    warnings
}

struct LabPlanItemInput {
    tweak_id: &'static str,
    category: TweakCategory,
    action: PlanAction,
    changes: Vec<PlannedChange>,
    requires_admin: bool,
    reboot: RebootPolicy,
    warnings: Vec<String>,
}

fn lab_plan_item(input: LabPlanItemInput) -> TweakPlanItem {
    let backup = backup_requirement(input.action, &input.changes);
    let rollback = rollback_plan(
        input.action,
        &input.changes,
        input.requires_admin,
        input.reboot,
    );

    TweakPlanItem {
        tweak_id: input.tweak_id.to_owned(),
        category: input.category,
        action: input.action,
        mode: TweakMode::Lab,
        risk: TweakRisk::High,
        changes: input.changes,
        backup,
        rollback,
        reboot: input.reboot,
        requires_admin: input.requires_admin,
        warnings: input.warnings,
    }
}

fn backup_requirement(action: PlanAction, changes: &[PlannedChange]) -> BackupRequirement {
    if action == PlanAction::Apply {
        BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: changes
                .first()
                .map_or_else(String::new, |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(
    action: PlanAction,
    changes: &[PlannedChange],
    requires_admin: bool,
    reboot: RebootPolicy,
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: RollbackKind::ExactValue,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous Windows Lab experiment state.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin,
        reboot,
        manual_instructions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> WindowsLabExperimentPlanRequest {
        let mut request = WindowsLabExperimentPlanRequest::new("plan-windows-lab");
        request.timer_resolution_state = TimerResolutionState::Default;
        request.memory_compression_state = MemoryCompressionState::Enabled;
        request
    }

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn safe_plan_never_applies_lab_experiments() {
        let mut request = request();
        request.timer_resolution_consent = WindowsLabConsent::Granted;
        request.memory_compression_consent = WindowsLabConsent::Granted;
        request.baseline_benchmark_captured = true;
        request.restore_point_confirmed = true;
        request.session_boundary_accepted = true;
        request.memory_headroom_confirmed = true;

        let plan = build_windows_lab_experiment_plan(&request);
        let timer = item(&plan, WIN_TIMER_RESOLUTION_LAB_TWEAK_ID);
        let memory = item(&plan, WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID);

        assert_eq!(timer.action, PlanAction::Recommend);
        assert_eq!(memory.action, PlanAction::Recommend);
        assert_eq!(timer.mode, TweakMode::Lab);
        assert_eq!(memory.mode, TweakMode::Lab);
        assert!(!plan.has_apply_items());
        assert!(windows_lab_plan_is_not_safe_default(&plan));
    }

    #[test]
    fn lab_apply_requires_consent_benchmark_restore_and_specific_gates() {
        let mut request = request();
        request.requested_mode = TweakMode::Lab;

        let no_consent = build_windows_lab_experiment_plan(&request);
        assert_eq!(
            item(&no_consent, WIN_TIMER_RESOLUTION_LAB_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.timer_resolution_consent = WindowsLabConsent::Granted;
        request.memory_compression_consent = WindowsLabConsent::Granted;
        request.baseline_benchmark_captured = true;
        let no_restore = build_windows_lab_experiment_plan(&request);
        assert_eq!(
            item(&no_restore, WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.restore_point_confirmed = true;
        request.session_boundary_accepted = true;
        request.memory_headroom_confirmed = true;
        let apply_plan = build_windows_lab_experiment_plan(&request);
        let timer = item(&apply_plan, WIN_TIMER_RESOLUTION_LAB_TWEAK_ID);
        let memory = item(&apply_plan, WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID);

        assert_eq!(timer.action, PlanAction::Apply);
        assert_eq!(memory.action, PlanAction::Apply);
        assert_eq!(timer.changes[0].scope, SessionScope::SessionOnly);
        assert_eq!(memory.changes[0].scope, SessionScope::Persistent);
        assert_eq!(
            memory.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_MEMORY_COMPRESSION.to_owned(),
            }
        );
        assert!(windows_lab_apply_requires_opt_in(&apply_plan));

        let mut wrong_mode_plan = apply_plan.clone();
        wrong_mode_plan.requested_mode = TweakMode::Competitive;
        assert!(!windows_lab_apply_requires_opt_in(&wrong_mode_plan));
    }

    #[test]
    fn unknown_or_matching_lab_state_does_not_apply() {
        let mut request = request();
        request.requested_mode = TweakMode::Lab;
        request.timer_resolution_state = TimerResolutionState::Requested;
        request.memory_compression_state = MemoryCompressionState::Unknown;
        request.timer_resolution_consent = WindowsLabConsent::Granted;
        request.memory_compression_consent = WindowsLabConsent::Granted;
        request.baseline_benchmark_captured = true;
        request.restore_point_confirmed = true;
        request.session_boundary_accepted = true;
        request.memory_headroom_confirmed = true;

        let plan = build_windows_lab_experiment_plan(&request);

        assert_eq!(
            item(&plan, WIN_TIMER_RESOLUTION_LAB_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert!(!plan.has_apply_items());
    }
}
