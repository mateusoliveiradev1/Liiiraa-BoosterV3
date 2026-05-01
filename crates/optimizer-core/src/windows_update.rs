//! Safe planning for Windows Update and Delivery Optimization controls.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for Delivery Optimization bandwidth and peer-sharing limits.
pub const UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID: &str =
    "update.delivery-optimization.limit";
/// Tweak ID for active-hours restart protection.
pub const UPDATE_AUTO_RESTART_GUARD_TWEAK_ID: &str = "update.auto-restart.guard";
/// Tweak ID for reducing Windows Update driver replacement.
pub const UPDATE_DRIVER_SOURCE_POLICY_TWEAK_ID: &str = "update.driver-source-policy";
/// Tweak ID for the blocked global Windows Update disable guardrail.
pub const UPDATE_DISABLE_GLOBAL_TWEAK_ID: &str = "update.disable-global";
/// Blocked guardrail ID used by the V1 matrix.
pub const BLOCKED_WINDOWS_UPDATE_DISABLE_GUARDRAIL_ID: &str = "blocked.windows-update.disable";

/// Logical target for Delivery Optimization download throttling.
pub const TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT: &str =
    "registry:hklm/software/policies/microsoft/windows/deliveryoptimization/domaxbackgrounddownloadbandwidthpercentage";
/// Logical target for Delivery Optimization peer upload throttling.
pub const TARGET_DELIVERY_OPTIMIZATION_UPLOAD_KBPS: &str =
    "registry:hklm/software/policies/microsoft/windows/deliveryoptimization/domaxuploadbandwidth";
/// Logical target for Delivery Optimization peer scope.
pub const TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_MODE: &str =
    "registry:hklm/software/policies/microsoft/windows/deliveryoptimization/dodownloadmode";
/// Logical target for Windows Update active-hours start.
pub const TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START: &str =
    "registry:hklm/software/policies/microsoft/windows/windowsupdate/activehoursstart";
/// Logical target for Windows Update active-hours end.
pub const TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_END: &str =
    "registry:hklm/software/policies/microsoft/windows/windowsupdate/activehoursend";
/// Logical target for avoiding auto reboot while a user is signed in.
pub const TARGET_WINDOWS_UPDATE_NO_AUTO_REBOOT_WITH_USERS: &str =
    "registry:hklm/software/policies/microsoft/windows/windowsupdate/au/noautorebootwithloggedonusers";
/// Logical target for excluding drivers from Windows quality updates.
pub const TARGET_WINDOWS_UPDATE_EXCLUDE_DRIVERS: &str =
    "registry:hklm/software/policies/microsoft/windows/windowsupdate/excludewudriversinqualityupdate";
/// Logical denial target for global Windows Update disable requests.
pub const TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE: &str = "windows-update:global-disable";

const DEFAULT_DELIVERY_DOWNLOAD_PERCENT: u8 = 20;
const DEFAULT_DELIVERY_UPLOAD_KBPS: u32 = 1_024;
const MIN_DELIVERY_DOWNLOAD_PERCENT: u8 = 5;
const MIN_DELIVERY_UPLOAD_KBPS: u32 = 64;
const DEFAULT_ACTIVE_HOURS_START: u8 = 18;
const DEFAULT_ACTIVE_HOURS_END: u8 = 2;

/// Explicit consent state for Windows Update controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsUpdateControlConsent {
    /// The user has not accepted this optional control.
    NotGranted,
    /// The user explicitly accepted this optional control.
    Granted,
}

impl WindowsUpdateControlConsent {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Observed Windows Update service health.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsUpdateServiceState {
    /// Windows Update services appear enabled or demand-start capable.
    Enabled,
    /// One or more core Windows Update services appear globally disabled.
    Disabled,
    /// Scan data could not prove the service posture.
    Unknown,
}

/// Request used to build the T046 update safety plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsUpdatePlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Delivery Optimization download limit percentage.
    pub delivery_download_percent: u8,
    /// Delivery Optimization upload limit in KB/s.
    pub delivery_upload_kbps: u32,
    /// Consent for Delivery Optimization limits.
    pub delivery_optimization_consent: WindowsUpdateControlConsent,
    /// Preferred active-hours start hour, 0-23.
    pub active_hours_start: u8,
    /// Preferred active-hours end hour, 0-23.
    pub active_hours_end: u8,
    /// Consent for active-hours restart guard.
    pub auto_restart_consent: WindowsUpdateControlConsent,
    /// Consent for Windows Update driver-source policy.
    pub driver_source_policy_consent: WindowsUpdateControlConsent,
    /// Whether the user committed to maintaining GPU/chipset drivers from vendors.
    pub vendor_driver_maintenance_committed: bool,
    /// Whether global Windows Update disable was requested.
    pub global_disable_requested: bool,
    /// Observed Windows Update service posture.
    pub service_state: WindowsUpdateServiceState,
}

impl WindowsUpdatePlanRequest {
    /// Creates a safe default request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            delivery_download_percent: DEFAULT_DELIVERY_DOWNLOAD_PERCENT,
            delivery_upload_kbps: DEFAULT_DELIVERY_UPLOAD_KBPS,
            delivery_optimization_consent: WindowsUpdateControlConsent::NotGranted,
            active_hours_start: DEFAULT_ACTIVE_HOURS_START,
            active_hours_end: DEFAULT_ACTIVE_HOURS_END,
            auto_restart_consent: WindowsUpdateControlConsent::NotGranted,
            driver_source_policy_consent: WindowsUpdateControlConsent::NotGranted,
            vendor_driver_maintenance_committed: false,
            global_disable_requested: false,
            service_state: WindowsUpdateServiceState::Unknown,
        }
    }
}

/// Builds a dry-run plan for T046 Windows Update and Delivery Optimization controls.
#[must_use]
pub fn build_windows_update_plan(request: &WindowsUpdatePlanRequest) -> TweakPlan {
    let items = vec![
        delivery_optimization_item(request),
        auto_restart_guard_item(request),
        driver_source_policy_item(request),
        global_disable_guardrail_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Windows Update") || warning.contains("Delivery Optimization")
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

/// Returns true when the ID belongs to T046 Windows Update planning.
#[must_use]
pub fn is_windows_update_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID
            | UPDATE_AUTO_RESTART_GUARD_TWEAK_ID
            | UPDATE_DRIVER_SOURCE_POLICY_TWEAK_ID
            | UPDATE_DISABLE_GLOBAL_TWEAK_ID
            | BLOCKED_WINDOWS_UPDATE_DISABLE_GUARDRAIL_ID
    )
}

/// Returns true when a target is a T046 mutable setting allowlisted for apply.
#[must_use]
pub fn is_windows_update_mutation_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT
            | TARGET_DELIVERY_OPTIMIZATION_UPLOAD_KBPS
            | TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_MODE
            | TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START
            | TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_END
            | TARGET_WINDOWS_UPDATE_NO_AUTO_REBOOT_WITH_USERS
            | TARGET_WINDOWS_UPDATE_EXCLUDE_DRIVERS
    )
}

/// Returns true when a plan has no path to global Windows Update disable.
#[must_use]
pub fn plan_blocks_global_windows_update_disable(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        let is_disable_guardrail = matches!(
            item.tweak_id.as_str(),
            UPDATE_DISABLE_GLOBAL_TWEAK_ID | BLOCKED_WINDOWS_UPDATE_DISABLE_GUARDRAIL_ID
        );
        let applies_global_disable = item.action == PlanAction::Apply
            && item
                .changes
                .iter()
                .any(|change| change.target == TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE);

        !applies_global_disable && (!is_disable_guardrail || item.action != PlanAction::Apply)
    })
}

fn delivery_optimization_item(request: &WindowsUpdatePlanRequest) -> TweakPlanItem {
    let mut warnings = service_warnings(request);
    let download_percent = sanitize_download_percent(request.delivery_download_percent);
    let upload_kbps = sanitize_upload_kbps(request.delivery_upload_kbps);

    if download_percent != request.delivery_download_percent {
        warnings.push(format!(
            "Delivery Optimization download limits must stay at or above {MIN_DELIVERY_DOWNLOAD_PERCENT}%."
        ));
    }

    if upload_kbps != request.delivery_upload_kbps {
        warnings.push(format!(
            "Delivery Optimization upload limits must stay at or above {MIN_DELIVERY_UPLOAD_KBPS} KB/s."
        ));
    }

    if !request.delivery_optimization_consent.is_granted() {
        warnings.push("Delivery Optimization bandwidth limits are prompt-only.".to_owned());
    }

    let action = if request.delivery_optimization_consent.is_granted() {
        PlanAction::Apply
    } else {
        PlanAction::Recommend
    };

    plan_item(
        UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID,
        action,
        TweakMode::Safe,
        TweakRisk::Low,
        vec![
            write_change(
                TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT,
                &download_percent.to_string(),
            ),
            write_change(TARGET_DELIVERY_OPTIMIZATION_UPLOAD_KBPS, &upload_kbps.to_string()),
            write_change(TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_MODE, "1"),
        ],
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn auto_restart_guard_item(request: &WindowsUpdatePlanRequest) -> TweakPlanItem {
    let mut warnings = service_warnings(request);
    let start = sanitize_hour(request.active_hours_start);
    let end = sanitize_hour(request.active_hours_end);

    if start == end {
        warnings.push(
            "Windows Update active hours need a non-empty window; using the default gaming window."
                .to_owned(),
        );
    }

    if !request.auto_restart_consent.is_granted() {
        warnings.push("Windows Update restart guard is prompt-only.".to_owned());
    }

    let (start, end) = if start == end {
        (DEFAULT_ACTIVE_HOURS_START, DEFAULT_ACTIVE_HOURS_END)
    } else {
        (start, end)
    };
    let action = if request.auto_restart_consent.is_granted() {
        PlanAction::Apply
    } else {
        PlanAction::Recommend
    };

    plan_item(
        UPDATE_AUTO_RESTART_GUARD_TWEAK_ID,
        action,
        TweakMode::Safe,
        TweakRisk::Low,
        vec![
            write_change(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START, &start.to_string()),
            write_change(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_END, &end.to_string()),
            write_change(TARGET_WINDOWS_UPDATE_NO_AUTO_REBOOT_WITH_USERS, "1"),
        ],
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn driver_source_policy_item(request: &WindowsUpdatePlanRequest) -> TweakPlanItem {
    let mut warnings = service_warnings(request);
    warnings.push(
        "Driver source policy never blocks Windows security or quality updates.".to_owned(),
    );

    if request.requested_mode == TweakMode::Safe {
        warnings.push(
            "Windows Update driver replacement policy is Competitive and stays off in Safe mode."
                .to_owned(),
        );
    }

    if !request.vendor_driver_maintenance_committed {
        warnings.push(
            "Use this only when the user commits to maintaining GPU and chipset drivers from vendors."
                .to_owned(),
        );
    }

    if !request.driver_source_policy_consent.is_granted() {
        warnings.push("Windows Update driver source policy requires explicit consent.".to_owned());
    }

    let can_apply = request.requested_mode != TweakMode::Safe
        && request.driver_source_policy_consent.is_granted()
        && request.vendor_driver_maintenance_committed;
    let action = if can_apply {
        PlanAction::Apply
    } else {
        PlanAction::Recommend
    };

    plan_item(
        UPDATE_DRIVER_SOURCE_POLICY_TWEAK_ID,
        action,
        TweakMode::Competitive,
        TweakRisk::Medium,
        vec![write_change(TARGET_WINDOWS_UPDATE_EXCLUDE_DRIVERS, "1")],
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn global_disable_guardrail_item(request: &WindowsUpdatePlanRequest) -> TweakPlanItem {
    let mut warnings = service_warnings(request);

    if request.global_disable_requested {
        warnings.push(
            "Global Windows Update disable is denied; use bandwidth, active-hours, or driver-source controls only."
                .to_owned(),
        );
    }

    let changes = if request.global_disable_requested {
        vec![deny_change(TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE, "disabled")]
    } else {
        Vec::new()
    };

    plan_item(
        UPDATE_DISABLE_GLOBAL_TWEAK_ID,
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
            TweakCategory::BackgroundWork
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
                summary: "Restore previous Windows Update setting.".to_owned(),
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

fn write_change(target: &str, value: &str) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value: None,
        desired_value: Some(value.to_owned()),
        scope: SessionScope::Persistent,
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

const fn sanitize_download_percent(value: u8) -> u8 {
    if value < MIN_DELIVERY_DOWNLOAD_PERCENT {
        MIN_DELIVERY_DOWNLOAD_PERCENT
    } else {
        value
    }
}

const fn sanitize_upload_kbps(value: u32) -> u32 {
    if value < MIN_DELIVERY_UPLOAD_KBPS {
        MIN_DELIVERY_UPLOAD_KBPS
    } else {
        value
    }
}

const fn sanitize_hour(value: u8) -> u8 {
    if value > 23 {
        DEFAULT_ACTIVE_HOURS_START
    } else {
        value
    }
}

fn service_warnings(request: &WindowsUpdatePlanRequest) -> Vec<String> {
    match request.service_state {
        WindowsUpdateServiceState::Enabled => Vec::new(),
        WindowsUpdateServiceState::Disabled => vec![concat!(
            "Windows Update services appear disabled; Liiiraa will not preserve or apply ",
            "global update disable as an optimization."
        )
        .to_owned()],
        WindowsUpdateServiceState::Unknown => vec![concat!(
            "Windows Update service posture is unknown; keep update controls typed and ",
            "avoid service disable paths."
        )
        .to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn delivery_optimization_limits_are_prompted_until_user_consents() {
        let mut request = WindowsUpdatePlanRequest::new("plan-update-delivery");
        request.service_state = WindowsUpdateServiceState::Enabled;
        request.delivery_download_percent = 0;
        request.delivery_upload_kbps = 0;

        let recommend_plan = build_windows_update_plan(&request);
        let delivery = item(
            &recommend_plan,
            UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID,
        );

        assert_eq!(delivery.action, PlanAction::Recommend);
        assert_eq!(delivery.backup, BackupRequirement::NotRequired);
        assert_eq!(
            delivery.changes[0].desired_value.as_deref(),
            Some("5")
        );
        assert!(delivery
            .warnings
            .iter()
            .any(|warning| warning.contains("prompt-only")));

        request.delivery_optimization_consent = WindowsUpdateControlConsent::Granted;
        let apply_plan = build_windows_update_plan(&request);
        let delivery = item(&apply_plan, UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID);

        assert_eq!(delivery.action, PlanAction::Apply);
        assert_eq!(
            delivery.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT.to_owned(),
            }
        );
        assert_eq!(delivery.changes.len(), 3);
        assert!(plan_blocks_global_windows_update_disable(&apply_plan));
    }

    #[test]
    fn active_hours_restart_guard_is_rollback_capable_when_consented() {
        let mut request = WindowsUpdatePlanRequest::new("plan-update-restarts");
        request.service_state = WindowsUpdateServiceState::Enabled;
        request.auto_restart_consent = WindowsUpdateControlConsent::Granted;
        request.active_hours_start = 19;
        request.active_hours_end = 1;

        let plan = build_windows_update_plan(&request);
        let guard = item(&plan, UPDATE_AUTO_RESTART_GUARD_TWEAK_ID);

        assert_eq!(guard.action, PlanAction::Apply);
        assert_eq!(guard.rollback.kind, RollbackKind::ExactValue);
        assert_eq!(guard.rollback.steps.len(), 3);
        assert_eq!(
            guard.changes[0].target,
            TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START
        );
        assert_eq!(guard.changes[0].desired_value.as_deref(), Some("19"));
        assert_eq!(guard.changes[1].desired_value.as_deref(), Some("1"));
    }

    #[test]
    fn driver_source_policy_never_applies_in_safe_mode() {
        let mut request = WindowsUpdatePlanRequest::new("plan-update-drivers");
        request.service_state = WindowsUpdateServiceState::Enabled;
        request.driver_source_policy_consent = WindowsUpdateControlConsent::Granted;
        request.vendor_driver_maintenance_committed = true;

        let safe_plan = build_windows_update_plan(&request);
        let driver_policy = item(&safe_plan, UPDATE_DRIVER_SOURCE_POLICY_TWEAK_ID);

        assert_eq!(driver_policy.action, PlanAction::Recommend);
        assert_eq!(driver_policy.backup, BackupRequirement::NotRequired);
        assert!(driver_policy
            .warnings
            .iter()
            .any(|warning| warning.contains("security or quality updates")));

        request.requested_mode = TweakMode::Competitive;
        let competitive_plan = build_windows_update_plan(&request);
        let driver_policy = item(&competitive_plan, UPDATE_DRIVER_SOURCE_POLICY_TWEAK_ID);

        assert_eq!(driver_policy.action, PlanAction::Apply);
        assert_eq!(
            driver_policy.changes[0].target,
            TARGET_WINDOWS_UPDATE_EXCLUDE_DRIVERS
        );
    }

    #[test]
    fn global_windows_update_disable_is_always_denied() {
        let mut request = WindowsUpdatePlanRequest::new("plan-update-disable");
        request.service_state = WindowsUpdateServiceState::Enabled;
        request.global_disable_requested = true;

        let plan = build_windows_update_plan(&request);
        let guardrail = item(&plan, UPDATE_DISABLE_GLOBAL_TWEAK_ID);

        assert_eq!(guardrail.action, PlanAction::Deny);
        assert_eq!(guardrail.mode, TweakMode::Blocked);
        assert_eq!(guardrail.risk, TweakRisk::Critical);
        assert_eq!(guardrail.changes[0].target, TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE);
        assert_eq!(guardrail.changes[0].operation, TweakOperationKind::Deny);
        assert!(plan_blocks_global_windows_update_disable(&plan));
        assert!(!plan.has_apply_items());
    }
}
