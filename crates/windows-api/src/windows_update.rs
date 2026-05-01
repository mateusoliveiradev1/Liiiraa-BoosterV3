//! Fixture-backed adapter for Windows Update and Delivery Optimization controls.

use std::fmt;

use optimizer_core::{
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
    windows_update::{
        build_windows_update_plan, is_windows_update_mutation_target, is_windows_update_tweak_id,
        plan_blocks_global_windows_update_disable, WindowsUpdatePlanRequest,
        WindowsUpdateServiceState, TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE,
        UPDATE_DISABLE_GLOBAL_TWEAK_ID,
    },
};

use crate::{ServiceScanItem, SystemScanReport, WindowsRollbackFixture};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsUpdateSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Windows Update or Delivery Optimization targets written or verified.
    pub targets: Vec<String>,
}

impl WindowsUpdateSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T046 Windows Update safety plan from read-only scan data.
#[must_use]
pub fn build_windows_update_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let mut request = WindowsUpdatePlanRequest::new(plan_id);
    request.service_state = service_state_from_scan(report);

    build_windows_update_plan(&request)
}

/// Applies T046 fixture changes after denying global Windows Update disable paths.
pub fn apply_windows_update_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<WindowsUpdateSettingsSummary, WindowsUpdateSettingsError> {
    validate_no_global_disable(plan)?;

    let mut summary = WindowsUpdateSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                WindowsUpdateSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            fixture.set_value(change.target.clone(), desired.to_owned());
            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Verifies T046 fixture changes after denying global Windows Update disable paths.
pub fn verify_windows_update_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<WindowsUpdateSettingsSummary, WindowsUpdateSettingsError> {
    validate_no_global_disable(plan)?;

    let mut summary = WindowsUpdateSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                WindowsUpdateSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(WindowsUpdateSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Returns true when the plan contains no global Windows Update disable path.
#[must_use]
pub fn windows_update_plan_blocks_global_disable(plan: &TweakPlan) -> bool {
    plan_blocks_global_windows_update_disable(plan)
}

fn validate_no_global_disable(plan: &TweakPlan) -> Result<(), WindowsUpdateSettingsError> {
    if plan_blocks_global_windows_update_disable(plan) {
        Ok(())
    } else {
        Err(WindowsUpdateSettingsError::global_disable_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), WindowsUpdateSettingsError> {
    if is_windows_update_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(WindowsUpdateSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), WindowsUpdateSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(WindowsUpdateSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_windows_update_mutation_target(&change.target) {
        return Err(WindowsUpdateSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

fn service_state_from_scan(report: &SystemScanReport) -> WindowsUpdateServiceState {
    let relevant = report
        .services
        .iter()
        .filter(|service| is_update_service(service))
        .collect::<Vec<_>>();

    if relevant.is_empty() {
        return WindowsUpdateServiceState::Unknown;
    }

    if relevant.iter().any(|service| {
        service
            .start_mode
            .as_deref()
            .is_some_and(|mode| mode.eq_ignore_ascii_case("disabled"))
    }) {
        WindowsUpdateServiceState::Disabled
    } else {
        WindowsUpdateServiceState::Enabled
    }
}

fn is_update_service(service: &ServiceScanItem) -> bool {
    matches!(
        service.name.to_ascii_lowercase().as_str(),
        "wuauserv" | "bits" | "dosvc" | "usosvc"
    )
}

/// Stable failure reason for fixture-backed Windows Update operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsUpdateSettingsErrorReason {
    /// Plan item was not part of the T046 update scope.
    UnsupportedTweak,
    /// Plan item targeted a setting outside the T046 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// The plan attempted global Windows Update disable.
    GlobalDisableDenied,
}

impl WindowsUpdateSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::GlobalDisableDenied => "global_disable_denied",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-Windows-Update tweak",
            Self::UnsupportedTarget => "Plan targets a setting outside the T046 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Windows Update fixture readback did not match the plan",
            Self::GlobalDisableDenied => "Global Windows Update disable is denied",
        }
    }
}

/// Structured error for fixture-backed Windows Update operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsUpdateSettingsError {
    reason: WindowsUpdateSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl WindowsUpdateSettingsError {
    fn new(
        reason: WindowsUpdateSettingsErrorReason,
        tweak_id: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            reason,
            tweak_id,
            target,
        }
    }

    fn unsupported_tweak(tweak_id: impl Into<String>) -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn global_disable_denied() -> Self {
        Self::new(
            WindowsUpdateSettingsErrorReason::GlobalDisableDenied,
            Some(UPDATE_DISABLE_GLOBAL_TWEAK_ID.to_owned()),
            Some(TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE.to_owned()),
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> WindowsUpdateSettingsErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for WindowsUpdateSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for WindowsUpdateSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlanItem, TweakRisk,
        },
        windows_update::{
            WindowsUpdateControlConsent, TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_MODE,
            TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT,
            TARGET_DELIVERY_OPTIMIZATION_UPLOAD_KBPS,
            TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_END, TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START,
            TARGET_WINDOWS_UPDATE_NO_AUTO_REBOOT_WITH_USERS,
            UPDATE_AUTO_RESTART_GUARD_TWEAK_ID,
            UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID,
        },
    };

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");

    fn item<'a>(
        plan: &'a TweakPlan,
        tweak_id: &str,
    ) -> &'a optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn scan_fixture_builds_update_recommendations_without_global_disable() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_windows_update_plan_from_scan("plan-t046-fixture", &report);
        let delivery = item(&plan, UPDATE_DELIVERY_OPTIMIZATION_LIMIT_TWEAK_ID);
        let restart = item(&plan, UPDATE_AUTO_RESTART_GUARD_TWEAK_ID);

        assert_eq!(delivery.action, PlanAction::Recommend);
        assert_eq!(restart.action, PlanAction::Recommend);
        assert!(windows_update_plan_blocks_global_disable(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_update_values() {
        let mut request = WindowsUpdatePlanRequest::new("plan-update-apply");
        request.service_state = WindowsUpdateServiceState::Enabled;
        request.delivery_optimization_consent = WindowsUpdateControlConsent::Granted;
        request.auto_restart_consent = WindowsUpdateControlConsent::Granted;
        request.delivery_download_percent = 25;
        request.delivery_upload_kbps = 2_048;
        request.active_hours_start = 19;
        request.active_hours_end = 2;
        let plan = build_windows_update_plan(&request);
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT, "100")
            .with_value(TARGET_DELIVERY_OPTIMIZATION_UPLOAD_KBPS, "0")
            .with_value(TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_MODE, "3")
            .with_value(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START, "8")
            .with_value(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_END, "17")
            .with_value(TARGET_WINDOWS_UPDATE_NO_AUTO_REBOOT_WITH_USERS, "0");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("update backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_windows_update_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");

        assert_eq!(applied.item_count, 2);
        assert_eq!(
            fixture.value(TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT),
            Some("25")
        );
        assert_eq!(
            fixture.value(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START),
            Some("19")
        );

        verify_windows_update_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore update fixture state");
        }

        assert_eq!(
            fixture.value(TARGET_DELIVERY_OPTIMIZATION_DOWNLOAD_PERCENT),
            Some("100")
        );
        assert_eq!(
            fixture.value(TARGET_WINDOWS_UPDATE_ACTIVE_HOURS_START),
            Some("8")
        );
    }

    #[test]
    fn fixture_rejects_global_windows_update_disable_even_if_marked_apply() {
        let plan = TweakPlan {
            id: "plan-malicious-update-disable".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: UPDATE_DISABLE_GLOBAL_TWEAK_ID.to_owned(),
                category: TweakCategory::BlockedGuardrail,
                action: PlanAction::Apply,
                mode: TweakMode::Blocked,
                risk: TweakRisk::Critical,
                changes: vec![PlannedChange {
                    target: TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: None,
                    desired_value: Some("disabled".to_owned()),
                    scope: SessionScope::Blocked,
                }],
                backup: BackupRequirement::NotRequired,
                rollback: RollbackPlan {
                    kind: RollbackKind::NotNeededReadonly,
                    steps: Vec::new(),
                    requires_admin: false,
                    reboot: RebootPolicy::None,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::None,
                requires_admin: false,
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new();

        let error = apply_windows_update_plan_to_fixture(&mut fixture, &plan)
            .expect_err("global Windows Update disable must be denied");

        assert_eq!(
            error.reason(),
            WindowsUpdateSettingsErrorReason::GlobalDisableDenied
        );
        assert_eq!(error.target(), Some(TARGET_WINDOWS_UPDATE_GLOBAL_DISABLE));
    }
}
