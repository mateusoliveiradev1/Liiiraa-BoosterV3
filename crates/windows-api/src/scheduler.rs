//! Registry-fixture adapter for T051 scheduler Competitive tweaks.

use std::fmt;

use optimizer_core::{
    scheduler::{
        build_scheduler_competitive_plan, is_scheduler_competitive_tweak_id,
        scheduler_plan_is_not_safe_default,
        scheduler_plan_requires_explicit_consent_and_benchmark,
        scheduler_tweak_targets_registry_value, SchedulerCompetitivePlanRequest,
        SchedulerControlConsent, SchedulerRegistryDwordState,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{SystemScanReport, WindowsRollbackFixture};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Scheduler registry targets written or verified.
    pub targets: Vec<String>,
}

impl SchedulerSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T051 read-only scheduler plan from scan data.
#[must_use]
pub fn build_scheduler_competitive_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let request = scheduler_request_from_scan(plan_id, report);

    build_scheduler_competitive_plan(&request)
}

/// Builds a consented T051 scheduler plan from scan data.
#[must_use]
pub fn build_consented_scheduler_competitive_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    baseline_benchmark_captured: bool,
) -> TweakPlan {
    let mut request = scheduler_request_from_scan(plan_id, report);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Competitive;
    request.mmcss_consent = SchedulerControlConsent::Granted;
    request.foreground_boost_consent = SchedulerControlConsent::Granted;
    request.baseline_benchmark_captured = baseline_benchmark_captured;

    build_scheduler_competitive_plan(&request)
}

/// Applies T051 scheduler registry changes to an in-memory Windows fixture.
pub fn apply_scheduler_competitive_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<SchedulerSettingsSummary, SchedulerSettingsError> {
    validate_explicit_scheduler_plan(plan)?;

    let mut summary = SchedulerSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                SchedulerSettingsError::missing_desired_value(
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

/// Verifies T051 scheduler registry changes against an in-memory fixture.
pub fn verify_scheduler_competitive_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<SchedulerSettingsSummary, SchedulerSettingsError> {
    validate_explicit_scheduler_plan(plan)?;

    let mut summary = SchedulerSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                SchedulerSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(SchedulerSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn scheduler_request_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> SchedulerCompetitivePlanRequest {
    let mut request = SchedulerCompetitivePlanRequest::new(plan_id);
    request.mmcss_system_responsiveness =
        SchedulerRegistryDwordState::from_option(report.scheduler.mmcss_system_responsiveness);
    request.win32_priority_separation =
        SchedulerRegistryDwordState::from_option(report.scheduler.win32_priority_separation);
    request
}

fn validate_explicit_scheduler_plan(plan: &TweakPlan) -> Result<(), SchedulerSettingsError> {
    if scheduler_plan_is_not_safe_default(plan)
        && scheduler_plan_requires_explicit_consent_and_benchmark(plan)
    {
        Ok(())
    } else {
        Err(SchedulerSettingsError::safe_default_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), SchedulerSettingsError> {
    if is_scheduler_competitive_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(SchedulerSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), SchedulerSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(SchedulerSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !scheduler_tweak_targets_registry_value(tweak_id, &change.target) {
        return Err(SchedulerSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed scheduler registry operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerSettingsErrorReason {
    /// Plan item was not part of the T051 scheduler scope.
    UnsupportedTweak,
    /// Plan item targeted a registry value outside the T051 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// Plan attempted a Safe/default scheduler apply.
    SafeDefaultDenied,
}

impl SchedulerSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::SafeDefaultDenied => "safe_default_denied",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-scheduler tweak",
            Self::UnsupportedTarget => "Plan targets a registry value outside the T051 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Scheduler registry fixture readback did not match the plan",
            Self::SafeDefaultDenied => {
                "Scheduler Competitive tweaks must not apply from Safe/default planning"
            }
        }
    }
}

/// Structured error for fixture-backed scheduler registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerSettingsError {
    reason: SchedulerSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl SchedulerSettingsError {
    fn new(
        reason: SchedulerSettingsErrorReason,
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
            SchedulerSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SchedulerSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SchedulerSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SchedulerSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SchedulerSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn safe_default_denied() -> Self {
        Self::new(SchedulerSettingsErrorReason::SafeDefaultDenied, None, None)
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> SchedulerSettingsErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected registry target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for SchedulerSettingsError {
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

impl std::error::Error for SchedulerSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        scheduler::{
            TARGET_MMCSS_SYSTEM_RESPONSIVENESS, TARGET_WIN32_PRIORITY_SEPARATION,
            WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID,
        },
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            SessionScope, TweakCategory, TweakMode, TweakPlanItem, TweakRisk,
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
    fn scan_fixture_builds_scheduler_detection_without_safe_apply() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_scheduler_competitive_plan_from_scan("plan-t051-fixture", &report);

        assert!(!plan.has_apply_items());
        assert!(scheduler_plan_is_not_safe_default(&plan));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("Competitive")));
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_scheduler_values() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan =
            build_consented_scheduler_competitive_plan_from_scan("plan-t051-consented", &report, true);
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
                    .expect("scheduler changes should include previous value"),
            );
        }

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("scheduler backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_scheduler_competitive_plan_to_fixture(&mut fixture, &plan)
            .expect("scheduler fixture apply should succeed");

        assert_eq!(applied.item_count, 2);
        assert_eq!(fixture.value(TARGET_MMCSS_SYSTEM_RESPONSIVENESS), Some("10"));
        assert_eq!(fixture.value(TARGET_WIN32_PRIORITY_SEPARATION), Some("38"));

        verify_scheduler_competitive_plan_fixture(&fixture, &plan)
            .expect("scheduler fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore scheduler fixture state");
        }

        assert_eq!(fixture.value(TARGET_MMCSS_SYSTEM_RESPONSIVENESS), Some("20"));
        assert_eq!(fixture.value(TARGET_WIN32_PRIORITY_SEPARATION), Some("2"));
    }

    #[test]
    fn fixture_rejects_safe_mode_scheduler_apply() {
        let plan = TweakPlan {
            id: "plan-malicious-scheduler".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID.to_owned(),
                category: TweakCategory::PowerAndLatency,
                action: PlanAction::Apply,
                mode: TweakMode::Competitive,
                risk: TweakRisk::Medium,
                changes: vec![PlannedChange {
                    target: TARGET_MMCSS_SYSTEM_RESPONSIVENESS.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("20".to_owned()),
                    desired_value: Some("10".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: TARGET_MMCSS_SYSTEM_RESPONSIVENESS.to_owned(),
                },
                rollback: RollbackPlan {
                    kind: RollbackKind::ExactValue,
                    steps: Vec::new(),
                    requires_admin: true,
                    reboot: RebootPolicy::None,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::None,
                requires_admin: true,
                warnings: vec![
                    "MMCSS SystemResponsiveness requires explicit consent.".to_owned(),
                    "Baseline benchmark is required before applying scheduler tweaks.".to_owned(),
                ],
            }],
            warnings: Vec::new(),
        };
        let mut fixture =
            WindowsRollbackFixture::new().with_value(TARGET_MMCSS_SYSTEM_RESPONSIVENESS, "20");

        let error = apply_scheduler_competitive_plan_to_fixture(&mut fixture, &plan)
            .expect_err("safe/default scheduler apply must be denied");

        assert_eq!(error.reason(), SchedulerSettingsErrorReason::SafeDefaultDenied);
    }

    #[test]
    fn fixture_rejects_cross_wired_scheduler_target() {
        let plan = TweakPlan {
            id: "plan-cross-wired-scheduler".to_owned(),
            requested_mode: TweakMode::Competitive,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: WIN_MMCSS_SYSTEM_RESPONSIVENESS_TWEAK_ID.to_owned(),
                category: TweakCategory::PowerAndLatency,
                action: PlanAction::Apply,
                mode: TweakMode::Competitive,
                risk: TweakRisk::Medium,
                changes: vec![PlannedChange {
                    target: TARGET_WIN32_PRIORITY_SEPARATION.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("2".to_owned()),
                    desired_value: Some("38".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: TARGET_WIN32_PRIORITY_SEPARATION.to_owned(),
                },
                rollback: RollbackPlan {
                    kind: RollbackKind::ExactValue,
                    steps: Vec::new(),
                    requires_admin: true,
                    reboot: RebootPolicy::None,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::None,
                requires_admin: true,
                warnings: vec![
                    "MMCSS SystemResponsiveness requires explicit consent.".to_owned(),
                    "Baseline benchmark is required before applying scheduler tweaks.".to_owned(),
                ],
            }],
            warnings: Vec::new(),
        };
        let mut fixture =
            WindowsRollbackFixture::new().with_value(TARGET_WIN32_PRIORITY_SEPARATION, "2");

        let error = apply_scheduler_competitive_plan_to_fixture(&mut fixture, &plan)
            .expect_err("cross-wired scheduler target must be denied");

        assert_eq!(error.reason(), SchedulerSettingsErrorReason::UnsupportedTarget);
    }
}
