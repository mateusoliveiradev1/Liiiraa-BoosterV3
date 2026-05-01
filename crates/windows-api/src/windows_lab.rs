//! Fixture adapter for T056 Windows Lab timer and memory compression experiments.

use std::fmt;

use optimizer_core::{
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
    windows_lab::{
        build_windows_lab_experiment_plan, is_windows_lab_tweak_id,
        windows_lab_apply_requires_opt_in, windows_lab_plan_is_not_safe_default,
        windows_lab_tweak_targets_value, MemoryCompressionState, TimerResolutionState,
        WindowsLabConsent, WindowsLabExperimentPlanRequest,
    },
};

use crate::{SystemScanReport, WindowsRollbackFixture};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsLabSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Windows Lab targets written or verified.
    pub targets: Vec<String>,
}

impl WindowsLabSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a conservative T056 Windows Lab plan from scan data.
#[must_use]
pub fn build_windows_lab_experiment_plan_from_scan(
    plan_id: impl Into<String>,
    _report: &SystemScanReport,
) -> TweakPlan {
    build_windows_lab_experiment_plan(&WindowsLabExperimentPlanRequest::new(plan_id))
}

/// Builds a fully opted-in T056 fixture plan from explicit Lab state.
#[must_use]
pub fn build_consented_windows_lab_experiment_plan_from_state(
    plan_id: impl Into<String>,
    timer_resolution_state: TimerResolutionState,
    memory_compression_state: MemoryCompressionState,
) -> TweakPlan {
    let mut request = WindowsLabExperimentPlanRequest::new(plan_id);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Lab;
    request.timer_resolution_state = timer_resolution_state;
    request.memory_compression_state = memory_compression_state;
    request.timer_resolution_consent = WindowsLabConsent::Granted;
    request.memory_compression_consent = WindowsLabConsent::Granted;
    request.baseline_benchmark_captured = true;
    request.restore_point_confirmed = true;
    request.session_boundary_accepted = true;
    request.memory_headroom_confirmed = true;

    build_windows_lab_experiment_plan(&request)
}

/// Applies T056 Windows Lab changes to an in-memory Windows fixture.
pub fn apply_windows_lab_experiment_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<WindowsLabSettingsSummary, WindowsLabSettingsError> {
    validate_explicit_windows_lab_plan(plan)?;

    let mut summary = WindowsLabSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                WindowsLabSettingsError::missing_desired_value(
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

/// Verifies T056 Windows Lab changes against an in-memory Windows fixture.
pub fn verify_windows_lab_experiment_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<WindowsLabSettingsSummary, WindowsLabSettingsError> {
    validate_explicit_windows_lab_plan(plan)?;

    let mut summary = WindowsLabSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                WindowsLabSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(WindowsLabSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn validate_explicit_windows_lab_plan(
    plan: &TweakPlan,
) -> Result<(), WindowsLabSettingsError> {
    if windows_lab_plan_is_not_safe_default(plan) && windows_lab_apply_requires_opt_in(plan) {
        Ok(())
    } else {
        Err(WindowsLabSettingsError::lab_gate_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), WindowsLabSettingsError> {
    if is_windows_lab_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(WindowsLabSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), WindowsLabSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(WindowsLabSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !windows_lab_tweak_targets_value(tweak_id, &change.target) {
        return Err(WindowsLabSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed Windows Lab operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsLabSettingsErrorReason {
    /// Plan item was not part of the T056 Windows Lab scope.
    UnsupportedTweak,
    /// Plan item targeted a value outside the T056 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// Plan attempted Lab changes without the required opt-in gates.
    LabGateDenied,
}

impl WindowsLabSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::LabGateDenied => "lab_gate_denied",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-Windows-Lab tweak",
            Self::UnsupportedTarget => "Plan targets a value outside the T056 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Windows Lab fixture readback did not match the plan",
            Self::LabGateDenied => {
                "Windows Lab experiments require Lab mode, explicit consent, restore point, and benchmark proof"
            }
        }
    }
}

/// Structured error for fixture-backed Windows Lab operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowsLabSettingsError {
    reason: WindowsLabSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl WindowsLabSettingsError {
    fn new(
        reason: WindowsLabSettingsErrorReason,
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
            WindowsLabSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsLabSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsLabSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsLabSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            WindowsLabSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn lab_gate_denied() -> Self {
        Self::new(WindowsLabSettingsErrorReason::LabGateDenied, None, None)
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> WindowsLabSettingsErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected Windows Lab target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for WindowsLabSettingsError {
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

impl std::error::Error for WindowsLabSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            SessionScope, TweakCategory, TweakMode, TweakPlanItem, TweakRisk,
        },
        windows_lab::{
            TARGET_MEMORY_COMPRESSION, TARGET_TIMER_RESOLUTION_SESSION,
            WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID, WIN_TIMER_RESOLUTION_LAB_TWEAK_ID,
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
    fn scan_fixture_builds_windows_lab_plan_without_safe_apply() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_windows_lab_experiment_plan_from_scan("plan-t056-fixture", &report);

        assert!(!plan.has_apply_items());
        assert!(windows_lab_plan_is_not_safe_default(&plan));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("Safe/default")));
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_windows_lab_values() {
        let plan = build_consented_windows_lab_experiment_plan_from_state(
            "plan-t056-consented",
            TimerResolutionState::Default,
            MemoryCompressionState::Enabled,
        );
        let mut fixture = WindowsRollbackFixture::new();

        for change in plan
            .items
            .iter()
            .filter(|item| item.action == PlanAction::Apply)
            .flat_map(|item| item.changes.iter())
        {
            fixture.set_value(
                change.target.clone(),
                change
                    .previous_value
                    .clone()
                    .expect("Windows Lab changes should include previous value"),
            );
        }

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("Windows Lab backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_windows_lab_experiment_plan_to_fixture(&mut fixture, &plan)
            .expect("Windows Lab fixture apply should succeed");
        assert_eq!(applied.item_count, 2);
        assert_eq!(
            fixture.value(TARGET_TIMER_RESOLUTION_SESSION),
            Some("1ms-session")
        );
        assert_eq!(fixture.value(TARGET_MEMORY_COMPRESSION), Some("disabled"));

        verify_windows_lab_experiment_plan_fixture(&fixture, &plan)
            .expect("Windows Lab fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore Windows Lab fixture state");
        }

        assert_eq!(fixture.value(TARGET_TIMER_RESOLUTION_SESSION), Some("default"));
        assert_eq!(fixture.value(TARGET_MEMORY_COMPRESSION), Some("enabled"));
    }

    #[test]
    fn fixture_rejects_safe_mode_windows_lab_apply() {
        let plan = windows_lab_plan(
            TweakMode::Safe,
            WIN_TIMER_RESOLUTION_LAB_TWEAK_ID,
            TARGET_TIMER_RESOLUTION_SESSION,
        );
        let mut fixture =
            WindowsRollbackFixture::new().with_value(TARGET_TIMER_RESOLUTION_SESSION, "default");

        let error = apply_windows_lab_experiment_plan_to_fixture(&mut fixture, &plan)
            .expect_err("Safe/default Windows Lab apply must be denied");

        assert_eq!(error.reason(), WindowsLabSettingsErrorReason::LabGateDenied);
    }

    #[test]
    fn fixture_rejects_unowned_windows_lab_target() {
        let plan = windows_lab_plan(
            TweakMode::Lab,
            WIN_MEMORY_COMPRESSION_LAB_TWEAK_ID,
            "registry:hklm/system/currentcontrolset/control/session-manager/memory-management",
        );
        let mut fixture = WindowsRollbackFixture::new().with_value(
            "registry:hklm/system/currentcontrolset/control/session-manager/memory-management",
            "enabled",
        );

        let error = apply_windows_lab_experiment_plan_to_fixture(&mut fixture, &plan)
            .expect_err("unowned Windows Lab target must be denied");

        assert_eq!(error.reason(), WindowsLabSettingsErrorReason::UnsupportedTarget);
    }

    fn windows_lab_plan(
        requested_mode: TweakMode,
        tweak_id: &str,
        target: &str,
    ) -> TweakPlan {
        TweakPlan {
            id: "plan-windows-lab-fixture".to_owned(),
            requested_mode,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: tweak_id.to_owned(),
                category: TweakCategory::PowerAndLatency,
                action: PlanAction::Apply,
                mode: TweakMode::Lab,
                risk: TweakRisk::High,
                changes: vec![PlannedChange {
                    target: target.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("enabled".to_owned()),
                    desired_value: Some("disabled".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: target.to_owned(),
                },
                rollback: RollbackPlan {
                    kind: RollbackKind::ExactValue,
                    steps: vec![optimizer_core::tweak_contracts::RollbackStep {
                        summary: "Restore previous Windows Lab experiment state.".to_owned(),
                        target: target.to_owned(),
                        operation: TweakOperationKind::Write,
                        expected_state: Some("enabled".to_owned()),
                    }],
                    requires_admin: true,
                    reboot: RebootPolicy::None,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::None,
                requires_admin: true,
                warnings: vec![
                    "Memory compression is Lab-only and requires explicit consent.".to_owned(),
                    "Baseline benchmark is required before applying Windows Lab experiments."
                        .to_owned(),
                    "Create or confirm a restore point and capture backups before applying this Lab experiment."
                        .to_owned(),
                ],
            }],
            warnings: Vec::new(),
        }
    }
}
