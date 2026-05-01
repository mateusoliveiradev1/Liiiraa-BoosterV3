//! Defender-safe performance planning for schedules and narrow exclusions.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for narrow Microsoft Defender exclusions.
pub const DEFENDER_NARROW_EXCLUSION_TWEAK_ID: &str = "security.defender.exclusion.narrow";
/// Tweak ID for read-only Defender Tamper Protection detection.
pub const DEFENDER_TAMPER_DETECT_TWEAK_ID: &str = "security.defender.tamper-detect";
/// Tweak ID for moving Defender scans outside gaming hours.
pub const DEFENDER_SCHEDULE_TWEAK_ID: &str = "security.defender.schedule";
/// Tweak ID for the blocked global Defender disable guardrail.
pub const DEFENDER_DISABLE_GLOBAL_TWEAK_ID: &str = "security.defender.disable-global";
/// Blocked guardrail ID used by the V1 matrix.
pub const BLOCKED_DEFENDER_DISABLE_GUARDRAIL_ID: &str = "blocked.defender.disable";
/// Blocked guardrail ID for broad or wildcard Defender exclusions.
pub const BLOCKED_DEFENDER_WILDCARD_EXCLUSION_TWEAK_ID: &str = "blocked.exclusions-wildcard";

/// Logical target for Defender scan scheduling.
pub const TARGET_DEFENDER_SCHEDULE_WINDOW: &str = "defender:schedule/scan-window";
/// Logical target for the backed-up Defender exclusion path set.
pub const TARGET_DEFENDER_EXCLUSION_LIST: &str = "defender:exclusion/list";
/// Logical denial target for global Defender disable requests.
pub const TARGET_DEFENDER_GLOBAL_DISABLE: &str = "defender:global-disable";

const DEFAULT_SCAN_WINDOW: &str = "02:00-05:00";
const DEFENDER_EXCLUSION_WARNING: &str = concat!(
    "Defender exclusions reduce scanning for the selected path; use only verified game ",
    "or library folders and keep real-time protection enabled."
);

/// Defender protection state from scan data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderProtectionState {
    /// Protection is enabled.
    Enabled,
    /// Protection is disabled.
    Disabled,
    /// Protection state could not be read.
    Unknown,
}

impl DefenderProtectionState {
    /// Converts an optional Windows bool into a conservative state.
    #[must_use]
    pub const fn from_option(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Enabled,
            Some(false) => Self::Disabled,
            None => Self::Unknown,
        }
    }

    const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// Defender Tamper Protection state from scan data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderTamperState {
    /// Tamper Protection is enabled.
    Enabled,
    /// Tamper Protection is disabled.
    Disabled,
    /// Tamper Protection state could not be read.
    Unknown,
}

impl DefenderTamperState {
    /// Converts an optional Windows bool into a conservative state.
    #[must_use]
    pub const fn from_option(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Enabled,
            Some(false) => Self::Disabled,
            None => Self::Unknown,
        }
    }
}

/// Defender scheduled scan relationship to user gaming hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderScheduleState {
    /// Existing scan timing appears outside gaming hours.
    OutsideGamingHours,
    /// Existing scan timing overlaps likely gaming hours.
    OverlapsGamingHours,
    /// Defender schedule information was present but not precise enough.
    Unknown,
    /// No Defender scheduled scan was detected.
    Missing,
}

/// Explicit consent state for Defender settings that need warning copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderControlConsent {
    /// The user has not accepted the warned action.
    NotGranted,
    /// The user explicitly accepted the warned action.
    Granted,
}

impl DefenderControlConsent {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Narrow path class allowed for Defender exclusions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderExclusionKind {
    /// A verified game executable.
    GameExecutable,
    /// A verified game install directory.
    GameInstallDirectory,
    /// A verified launcher library directory.
    GameLibraryDirectory,
}

/// Candidate Defender exclusion path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderExclusionCandidate {
    /// Absolute Windows path supplied by detection or the user.
    pub path: String,
    /// Narrow path class.
    pub kind: DefenderExclusionKind,
    /// Whether the app verified this path belongs to the game/library context.
    pub verified: bool,
    /// Whether the user accepted the warning for this exclusion.
    pub warning_consent: DefenderControlConsent,
}

impl DefenderExclusionCandidate {
    /// Creates an unverified, unconsented candidate.
    #[must_use]
    pub fn new(path: impl Into<String>, kind: DefenderExclusionKind) -> Self {
        Self {
            path: path.into(),
            kind,
            verified: false,
            warning_consent: DefenderControlConsent::NotGranted,
        }
    }

    /// Marks the candidate as verified by game/library detection.
    #[must_use]
    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = verified;
        self
    }

    /// Adds explicit warning consent.
    #[must_use]
    pub fn with_warning_consent(mut self, consent: DefenderControlConsent) -> Self {
        self.warning_consent = consent;
        self
    }
}

/// Request for T045 Defender-safe performance planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderPerformancePlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Defender antivirus state.
    pub antivirus: DefenderProtectionState,
    /// Defender real-time protection state.
    pub real_time_protection: DefenderProtectionState,
    /// Defender Tamper Protection state.
    pub tamper_protection: DefenderTamperState,
    /// Existing Defender schedule state.
    pub schedule_state: DefenderScheduleState,
    /// Consent to move Defender scans outside gaming hours.
    pub schedule_consent: DefenderControlConsent,
    /// Desired scan window, such as `02:00-05:00`.
    pub preferred_scan_window: Option<String>,
    /// Narrow exclusion candidates.
    pub exclusion_candidates: Vec<DefenderExclusionCandidate>,
    /// Whether a global Defender disable was requested.
    pub global_disable_requested: bool,
}

impl DefenderPerformancePlanRequest {
    /// Creates a safe default request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            antivirus: DefenderProtectionState::Unknown,
            real_time_protection: DefenderProtectionState::Unknown,
            tamper_protection: DefenderTamperState::Unknown,
            schedule_state: DefenderScheduleState::Unknown,
            schedule_consent: DefenderControlConsent::NotGranted,
            preferred_scan_window: None,
            exclusion_candidates: Vec::new(),
            global_disable_requested: false,
        }
    }
}

/// Builds a dry-run plan for T045 Defender-safe performance actions.
#[must_use]
pub fn build_defender_performance_plan(request: &DefenderPerformancePlanRequest) -> TweakPlan {
    let items = vec![
        tamper_detect_item(request),
        schedule_item(request),
        narrow_exclusion_item(request),
        broad_exclusion_guardrail_item(request),
        global_disable_guardrail_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Defender")
                || warning.contains("exclusion")
                || warning.contains("Tamper")
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

/// Returns true when the ID belongs to T045 Defender-safe planning.
#[must_use]
pub fn is_defender_performance_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        DEFENDER_NARROW_EXCLUSION_TWEAK_ID
            | DEFENDER_TAMPER_DETECT_TWEAK_ID
            | DEFENDER_SCHEDULE_TWEAK_ID
            | DEFENDER_DISABLE_GLOBAL_TWEAK_ID
            | BLOCKED_DEFENDER_DISABLE_GUARDRAIL_ID
            | BLOCKED_DEFENDER_WILDCARD_EXCLUSION_TWEAK_ID
    )
}

/// Returns true when a logical target is a T045 mutable Defender target.
#[must_use]
pub fn is_defender_mutation_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_DEFENDER_SCHEDULE_WINDOW | TARGET_DEFENDER_EXCLUSION_LIST
    )
}

/// Returns true when a Defender plan contains no path to global disable.
#[must_use]
pub fn plan_blocks_global_defender_disable(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        let is_disable_guardrail = matches!(
            item.tweak_id.as_str(),
            DEFENDER_DISABLE_GLOBAL_TWEAK_ID | BLOCKED_DEFENDER_DISABLE_GUARDRAIL_ID
        );
        let applies_global_disable = item.action == PlanAction::Apply
            && item
                .changes
                .iter()
                .any(|change| change.target == TARGET_DEFENDER_GLOBAL_DISABLE);

        !applies_global_disable && (!is_disable_guardrail || item.action != PlanAction::Apply)
    })
}

fn tamper_detect_item(request: &DefenderPerformancePlanRequest) -> TweakPlanItem {
    let mut warnings = protection_warnings(request);

    match request.tamper_protection {
        DefenderTamperState::Enabled => warnings.push(
            "Defender Tamper Protection is enabled; some Defender settings may require Windows Security confirmation."
                .to_owned(),
        ),
        DefenderTamperState::Unknown => warnings.push(
            "Defender Tamper Protection state is unknown; keep Defender actions prompt-only."
                .to_owned(),
        ),
        DefenderTamperState::Disabled => {}
    }

    TweakPlanItem {
        tweak_id: DEFENDER_TAMPER_DETECT_TWEAK_ID.to_owned(),
        category: TweakCategory::SecurityTradeoff,
        action: PlanAction::DetectOnly,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes: Vec::new(),
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

fn schedule_item(request: &DefenderPerformancePlanRequest) -> TweakPlanItem {
    let mut warnings = protection_warnings(request);
    let desired_window = request
        .preferred_scan_window
        .as_deref()
        .filter(|window| valid_scan_window(window))
        .unwrap_or(DEFAULT_SCAN_WINDOW);
    let schedule_needs_help = !matches!(
        request.schedule_state,
        DefenderScheduleState::OutsideGamingHours
    );
    let can_apply = schedule_needs_help && request.schedule_consent.is_granted();

    match request.schedule_state {
        DefenderScheduleState::OutsideGamingHours => warnings.push(
            "Defender scheduled scan already appears outside likely gaming hours.".to_owned(),
        ),
        DefenderScheduleState::OverlapsGamingHours => warnings.push(
            "Defender scheduled scan overlaps likely gaming hours; offer an off-hours schedule."
                .to_owned(),
        ),
        DefenderScheduleState::Unknown => warnings.push(
            "Defender scan schedule could not be proven; ask before changing scan timing."
                .to_owned(),
        ),
        DefenderScheduleState::Missing => warnings.push(
            "Defender scheduled scan was not detected; offer a protected off-hours schedule."
                .to_owned(),
        ),
    }

    if !request.schedule_consent.is_granted() && schedule_needs_help {
        warnings.push("Defender scan scheduling is prompt-only.".to_owned());
    }

    let changes = if schedule_needs_help {
        vec![write_change(
            TARGET_DEFENDER_SCHEDULE_WINDOW,
            desired_window,
            SessionScope::Persistent,
        )]
    } else {
        Vec::new()
    };

    plan_item(
        DEFENDER_SCHEDULE_TWEAK_ID,
        if can_apply {
            PlanAction::Apply
        } else if schedule_needs_help {
            PlanAction::Recommend
        } else {
            PlanAction::DetectOnly
        },
        TweakMode::Safe,
        TweakRisk::Low,
        changes,
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn narrow_exclusion_item(request: &DefenderPerformancePlanRequest) -> TweakPlanItem {
    let accepted_paths = request
        .exclusion_candidates
        .iter()
        .filter(|candidate| exclusion_candidate_is_safe(candidate))
        .filter(|candidate| candidate.warning_consent.is_granted())
        .map(|candidate| candidate.path.clone())
        .collect::<Vec<_>>();
    let safe_paths = request
        .exclusion_candidates
        .iter()
        .filter(|candidate| exclusion_candidate_is_safe(candidate))
        .map(|candidate| candidate.path.clone())
        .collect::<Vec<_>>();
    let mut warnings = protection_warnings(request);

    if !safe_paths.is_empty() {
        warnings.push(DEFENDER_EXCLUSION_WARNING.to_owned());
    }

    let action = if !accepted_paths.is_empty() {
        PlanAction::Apply
    } else if !safe_paths.is_empty() {
        warnings.push("Narrow Defender exclusions require explicit warning consent.".to_owned());
        PlanAction::Recommend
    } else {
        PlanAction::DetectOnly
    };

    let changes = if action == PlanAction::DetectOnly {
        Vec::new()
    } else {
        vec![write_change(
            TARGET_DEFENDER_EXCLUSION_LIST,
            &paths_value(if accepted_paths.is_empty() {
                &safe_paths
            } else {
                &accepted_paths
            }),
            SessionScope::ProfileScoped,
        )]
    };

    plan_item(
        DEFENDER_NARROW_EXCLUSION_TWEAK_ID,
        action,
        TweakMode::Safe,
        TweakRisk::Medium,
        changes,
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn broad_exclusion_guardrail_item(request: &DefenderPerformancePlanRequest) -> TweakPlanItem {
    let blocked = request
        .exclusion_candidates
        .iter()
        .filter(|candidate| !candidate_path_is_narrow(&candidate.path))
        .collect::<Vec<_>>();
    let changes = blocked
        .iter()
        .map(|candidate| deny_change("defender:exclusion/broad", &candidate.path))
        .collect::<Vec<_>>();
    let warnings = if blocked.is_empty() {
        Vec::new()
    } else {
        vec![concat!(
            "Broad or wildcard Defender exclusions are denied; never exclude drive roots, ",
            "user profile roots, Downloads, temp, or Windows system folders."
        )
        .to_owned()]
    };

    plan_item(
        BLOCKED_DEFENDER_WILDCARD_EXCLUSION_TWEAK_ID,
        if blocked.is_empty() {
            PlanAction::DetectOnly
        } else {
            PlanAction::Deny
        },
        TweakMode::Blocked,
        TweakRisk::Critical,
        changes,
        false,
        warnings,
        RollbackKind::NotNeededReadonly,
    )
}

fn global_disable_guardrail_item(request: &DefenderPerformancePlanRequest) -> TweakPlanItem {
    let warnings = if request.global_disable_requested {
        vec![
            "Global Defender disable is denied; use scheduling or narrow exclusions only."
                .to_owned(),
        ]
    } else {
        Vec::new()
    };
    let changes = if request.global_disable_requested {
        vec![deny_change(TARGET_DEFENDER_GLOBAL_DISABLE, "disabled")]
    } else {
        Vec::new()
    };

    plan_item(
        DEFENDER_DISABLE_GLOBAL_TWEAK_ID,
        if request.global_disable_requested {
            PlanAction::Deny
        } else {
            PlanAction::DetectOnly
        },
        TweakMode::Blocked,
        TweakRisk::Critical,
        changes,
        false,
        warnings,
        RollbackKind::NotNeededReadonly,
    )
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    mode: TweakMode,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    requires_admin: bool,
    warnings: Vec<String>,
    rollback_kind: RollbackKind,
) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: if mode == TweakMode::Blocked {
            TweakCategory::BlockedGuardrail
        } else {
            TweakCategory::SecurityTradeoff
        },
        action,
        mode,
        risk,
        backup: backup_requirement(action, rollback_kind, &changes),
        rollback: rollback_plan(rollback_kind, &changes, requires_admin),
        changes,
        reboot: RebootPolicy::None,
        requires_admin,
        warnings,
    }
}

fn backup_requirement(
    action: PlanAction,
    kind: RollbackKind,
    changes: &[PlannedChange],
) -> BackupRequirement {
    if action == PlanAction::Apply && kind.needs_backup_before_apply() {
        BackupRequirement::Required {
            kind,
            target: changes
                .first()
                .map_or_else(String::new, |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(
    kind: RollbackKind,
    changes: &[PlannedChange],
    requires_admin: bool,
) -> RollbackPlan {
    if changes.is_empty() || !kind.needs_backup_before_apply() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous Defender-safe setting.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: None,
            })
            .collect(),
        requires_admin,
        reboot: RebootPolicy::None,
        manual_instructions: Vec::new(),
    }
}

fn write_change(target: &str, value: &str, scope: SessionScope) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value: None,
        desired_value: Some(value.to_owned()),
        scope,
    }
}

fn deny_change(target: &str, value: &str) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Deny,
        previous_value: None,
        desired_value: Some(value.to_owned()),
        scope: SessionScope::Blocked,
    }
}

fn protection_warnings(request: &DefenderPerformancePlanRequest) -> Vec<String> {
    let mut warnings = Vec::new();

    if !request.antivirus.is_enabled() {
        warnings.push(
            "Defender antivirus is not confirmed enabled; Liiiraa will not disable Defender for performance."
                .to_owned(),
        );
    }

    if !request.real_time_protection.is_enabled() {
        warnings.push(
            "Defender real-time protection is not confirmed enabled; performance options must not weaken protection."
                .to_owned(),
        );
    }

    warnings
}

fn exclusion_candidate_is_safe(candidate: &DefenderExclusionCandidate) -> bool {
    candidate.verified && candidate_path_is_narrow(&candidate.path)
}

fn candidate_path_is_narrow(path: &str) -> bool {
    let normalized = normalize_windows_path(path);
    let Some(path) = normalized.as_deref() else {
        return false;
    };

    if path.contains('*') || path.contains('?') {
        return false;
    }

    let parts = path
        .split('\\')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() < 3 {
        return false;
    }

    if matches!(parts.get(1), Some(&"windows")) {
        return false;
    }

    if matches!(parts.get(1), Some(&"users")) && parts.len() <= 3 {
        return false;
    }

    if parts
        .iter()
        .any(|part| matches!(*part, "downloads" | "temp" | "tmp"))
    {
        return false;
    }

    if parts.len() <= 2
        && matches!(
            parts.get(1),
            Some(&"program files") | Some(&"program files (x86)") | Some(&"programdata")
        )
    {
        return false;
    }

    true
}

fn normalize_windows_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    let bytes = trimmed.as_bytes();

    if bytes.len() < 4
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':'
        || !matches!(bytes[2], b'\\' | b'/')
    {
        return None;
    }

    Some(trimmed.replace('/', "\\").to_ascii_lowercase())
}

fn valid_scan_window(window: &str) -> bool {
    let bytes = window.as_bytes();

    bytes.len() == "00:00-00:00".len()
        && matches!(bytes.get(2), Some(&b':'))
        && matches!(bytes.get(5), Some(&b'-'))
        && matches!(bytes.get(8), Some(&b':'))
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5 | 8) || byte.is_ascii_digit())
}

fn paths_value(paths: &[String]) -> String {
    paths.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("tweak item should exist")
    }

    #[test]
    fn schedule_overlap_recommends_until_user_accepts_off_hours_window() {
        let mut request = DefenderPerformancePlanRequest::new("plan-defender-schedule");
        request.antivirus = DefenderProtectionState::Enabled;
        request.real_time_protection = DefenderProtectionState::Enabled;
        request.schedule_state = DefenderScheduleState::OverlapsGamingHours;
        request.preferred_scan_window = Some("03:00-05:00".to_owned());

        let recommend_plan = build_defender_performance_plan(&request);
        let schedule = item(&recommend_plan, DEFENDER_SCHEDULE_TWEAK_ID);

        assert_eq!(schedule.action, PlanAction::Recommend);
        assert_eq!(schedule.backup, BackupRequirement::NotRequired);
        assert!(schedule
            .warnings
            .iter()
            .any(|warning| warning.contains("prompt-only")));

        request.schedule_consent = DefenderControlConsent::Granted;
        let apply_plan = build_defender_performance_plan(&request);
        let schedule = item(&apply_plan, DEFENDER_SCHEDULE_TWEAK_ID);

        assert_eq!(schedule.action, PlanAction::Apply);
        assert_eq!(
            schedule.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_DEFENDER_SCHEDULE_WINDOW.to_owned(),
            }
        );
        assert_eq!(schedule.changes[0].desired_value.as_deref(), Some("03:00-05:00"));
    }

    #[test]
    fn narrow_exclusion_requires_verified_path_warning_and_consent() {
        let path = "C:\\Games\\SteamLibrary\\steamapps\\common\\PUBG\\TslGame.exe";
        let mut request = DefenderPerformancePlanRequest::new("plan-defender-exclusion");
        request.antivirus = DefenderProtectionState::Enabled;
        request.real_time_protection = DefenderProtectionState::Enabled;
        request.exclusion_candidates = vec![DefenderExclusionCandidate::new(
            path,
            DefenderExclusionKind::GameExecutable,
        )
        .verified(true)];

        let recommend_plan = build_defender_performance_plan(&request);
        let exclusion = item(&recommend_plan, DEFENDER_NARROW_EXCLUSION_TWEAK_ID);

        assert_eq!(exclusion.action, PlanAction::Recommend);
        assert!(exclusion
            .warnings
            .iter()
            .any(|warning| warning.contains("reduce scanning")));

        request.exclusion_candidates[0].warning_consent = DefenderControlConsent::Granted;
        let apply_plan = build_defender_performance_plan(&request);
        let exclusion = item(&apply_plan, DEFENDER_NARROW_EXCLUSION_TWEAK_ID);

        assert_eq!(exclusion.action, PlanAction::Apply);
        assert_eq!(exclusion.risk, TweakRisk::Medium);
        assert_eq!(exclusion.changes[0].target, TARGET_DEFENDER_EXCLUSION_LIST);
        assert_eq!(exclusion.changes[0].desired_value.as_deref(), Some(path));
    }

    #[test]
    fn broad_or_wildcard_exclusions_are_denied() {
        let mut request = DefenderPerformancePlanRequest::new("plan-defender-broad");
        request.exclusion_candidates = vec![
            DefenderExclusionCandidate::new("C:\\", DefenderExclusionKind::GameLibraryDirectory)
                .verified(true)
                .with_warning_consent(DefenderControlConsent::Granted),
            DefenderExclusionCandidate::new(
                "C:\\Windows\\System32",
                DefenderExclusionKind::GameInstallDirectory,
            )
            .verified(true)
            .with_warning_consent(DefenderControlConsent::Granted),
            DefenderExclusionCandidate::new(
                "D:\\SteamLibrary\\*",
                DefenderExclusionKind::GameLibraryDirectory,
            )
            .verified(true)
            .with_warning_consent(DefenderControlConsent::Granted),
        ];

        let plan = build_defender_performance_plan(&request);
        let narrow = item(&plan, DEFENDER_NARROW_EXCLUSION_TWEAK_ID);
        let blocked = item(&plan, BLOCKED_DEFENDER_WILDCARD_EXCLUSION_TWEAK_ID);

        assert_eq!(narrow.action, PlanAction::DetectOnly);
        assert_eq!(blocked.action, PlanAction::Deny);
        assert_eq!(blocked.mode, TweakMode::Blocked);
        assert!(blocked
            .changes
            .iter()
            .all(|change| change.operation == TweakOperationKind::Deny));
    }

    #[test]
    fn global_defender_disable_is_always_denied() {
        let mut request = DefenderPerformancePlanRequest::new("plan-defender-disable");
        request.global_disable_requested = true;

        let plan = build_defender_performance_plan(&request);
        let guardrail = item(&plan, DEFENDER_DISABLE_GLOBAL_TWEAK_ID);

        assert_eq!(guardrail.action, PlanAction::Deny);
        assert_eq!(guardrail.mode, TweakMode::Blocked);
        assert_eq!(guardrail.risk, TweakRisk::Critical);
        assert_eq!(guardrail.changes[0].target, TARGET_DEFENDER_GLOBAL_DISABLE);
        assert!(plan_blocks_global_defender_disable(&plan));
        assert!(!plan.has_apply_items());
    }
}
