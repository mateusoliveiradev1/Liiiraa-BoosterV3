//! Fixture-backed adapter for Defender-safe performance actions.

use std::fmt;

use optimizer_core::{
    defender::{
        build_defender_performance_plan, is_defender_mutation_target,
        is_defender_performance_tweak_id, plan_blocks_global_defender_disable,
        DefenderPerformancePlanRequest, DefenderProtectionState, DefenderScheduleState,
        DefenderTamperState, DEFENDER_DISABLE_GLOBAL_TWEAK_ID, TARGET_DEFENDER_GLOBAL_DISABLE,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{ScheduledTaskScanItem, SystemScanReport, WindowsRollbackFixture};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Defender targets written or verified.
    pub targets: Vec<String>,
}

impl DefenderSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T045 Defender-safe performance plan from read-only scan data.
#[must_use]
pub fn build_defender_performance_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let defender = &report.security.defender;
    let mut request = DefenderPerformancePlanRequest::new(plan_id);
    request.antivirus = DefenderProtectionState::from_option(defender.antivirus_enabled);
    request.real_time_protection =
        DefenderProtectionState::from_option(defender.real_time_protection_enabled);
    request.tamper_protection = DefenderTamperState::from_option(defender.tamper_protected);
    request.schedule_state = schedule_state_from_scan(report);

    if request.schedule_state == DefenderScheduleState::OverlapsGamingHours {
        request.preferred_scan_window = Some("03:00-05:00".to_owned());
    }

    build_defender_performance_plan(&request)
}

/// Applies T045 Defender-safe fixture changes.
pub fn apply_defender_performance_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<DefenderSettingsSummary, DefenderSettingsError> {
    validate_no_global_disable(plan)?;

    let mut summary = DefenderSettingsSummary::empty();

    for item in plan
        .items
        .iter()
        .filter(|item| item.action == PlanAction::Apply)
    {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                DefenderSettingsError::missing_desired_value(
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

/// Verifies T045 Defender-safe fixture changes.
pub fn verify_defender_performance_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<DefenderSettingsSummary, DefenderSettingsError> {
    validate_no_global_disable(plan)?;

    let mut summary = DefenderSettingsSummary::empty();

    for item in plan
        .items
        .iter()
        .filter(|item| item.action == PlanAction::Apply)
    {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                DefenderSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(DefenderSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Returns true when the plan contains no global Defender disable path.
#[must_use]
pub fn defender_plan_blocks_global_disable(plan: &TweakPlan) -> bool {
    plan_blocks_global_defender_disable(plan)
}

fn validate_no_global_disable(plan: &TweakPlan) -> Result<(), DefenderSettingsError> {
    if plan_blocks_global_defender_disable(plan) {
        Ok(())
    } else {
        Err(DefenderSettingsError::global_disable_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), DefenderSettingsError> {
    if is_defender_performance_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(DefenderSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), DefenderSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(DefenderSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_defender_mutation_target(&change.target) {
        return Err(DefenderSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

fn schedule_state_from_scan(report: &SystemScanReport) -> DefenderScheduleState {
    if let Some(schedule_time) = report.security.defender.scan_schedule_time.as_deref() {
        return schedule_state_from_time(schedule_time).unwrap_or(DefenderScheduleState::Unknown);
    }

    let defender_task = report
        .scheduled_tasks
        .iter()
        .find(|task| is_defender_scheduled_scan_task(task));

    match defender_task {
        Some(task) => task
            .next_run_time
            .as_deref()
            .and_then(schedule_state_from_time)
            .unwrap_or(DefenderScheduleState::Unknown),
        None => DefenderScheduleState::Missing,
    }
}

fn is_defender_scheduled_scan_task(task: &ScheduledTaskScanItem) -> bool {
    let task_name = task.task_name.to_ascii_lowercase();
    let task_path = task.task_path.to_ascii_lowercase();

    task_name.contains("scheduled scan") && task_path.contains("windows defender")
}

fn schedule_state_from_time(value: &str) -> Option<DefenderScheduleState> {
    parse_hour(value).map(|hour| {
        if likely_gaming_hour(hour) {
            DefenderScheduleState::OverlapsGamingHours
        } else {
            DefenderScheduleState::OutsideGamingHours
        }
    })
}

fn parse_hour(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    let hour_index = bytes
        .iter()
        .position(|byte| matches!(*byte, b'T' | b' '))
        .map_or(0, |index| index + 1);

    if bytes.len() < hour_index + 2 {
        return None;
    }

    let hour = std::str::from_utf8(&bytes[hour_index..hour_index + 2])
        .ok()?
        .parse::<u8>()
        .ok()?;

    (hour < 24).then_some(hour)
}

fn likely_gaming_hour(hour: u8) -> bool {
    hour >= 18 || hour <= 1
}

/// Stable failure reason for fixture-backed Defender operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderSettingsErrorReason {
    /// Plan item was not part of the T045 Defender scope.
    UnsupportedTweak,
    /// Plan item targeted a Defender setting outside the allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// The plan attempted global Defender disable.
    GlobalDisableDenied,
}

impl DefenderSettingsErrorReason {
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
            Self::UnsupportedTweak => "Plan contains a non-Defender-safe tweak",
            Self::UnsupportedTarget => "Plan targets a Defender setting outside the T045 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Defender fixture readback did not match the plan",
            Self::GlobalDisableDenied => "Global Defender disable is denied",
        }
    }
}

/// Structured error for fixture-backed Defender operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefenderSettingsError {
    reason: DefenderSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl DefenderSettingsError {
    fn new(
        reason: DefenderSettingsErrorReason,
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
            DefenderSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            DefenderSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            DefenderSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            DefenderSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            DefenderSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn global_disable_denied() -> Self {
        Self::new(
            DefenderSettingsErrorReason::GlobalDisableDenied,
            Some(DEFENDER_DISABLE_GLOBAL_TWEAK_ID.to_owned()),
            Some(TARGET_DEFENDER_GLOBAL_DISABLE.to_owned()),
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> DefenderSettingsErrorReason {
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

impl fmt::Display for DefenderSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}",
            self.reason.as_str(),
            self.reason.message()
        )?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for DefenderSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        defender::{
            build_defender_performance_plan, DefenderControlConsent, DefenderExclusionCandidate,
            DefenderExclusionKind, TARGET_DEFENDER_EXCLUSION_LIST, TARGET_DEFENDER_SCHEDULE_WINDOW,
        },
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlanItem, TweakRisk,
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
    fn scan_fixture_builds_schedule_recommendation_without_disabling_defender() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_defender_performance_plan_from_scan("plan-defender-fixture", &report);
        let schedule = item(&plan, optimizer_core::defender::DEFENDER_SCHEDULE_TWEAK_ID);

        assert_eq!(schedule.action, PlanAction::Recommend);
        assert!(defender_plan_blocks_global_disable(&plan));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("Defender scheduled scan")));
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_schedule_and_exclusion_list() {
        let mut request = DefenderPerformancePlanRequest::new("plan-defender-apply");
        request.antivirus = DefenderProtectionState::Enabled;
        request.real_time_protection = DefenderProtectionState::Enabled;
        request.schedule_state = DefenderScheduleState::OverlapsGamingHours;
        request.schedule_consent = DefenderControlConsent::Granted;
        request.preferred_scan_window = Some("03:00-05:00".to_owned());
        request.exclusion_candidates = vec![DefenderExclusionCandidate::new(
            "C:\\Games\\SteamLibrary\\steamapps\\common\\PUBG",
            DefenderExclusionKind::GameInstallDirectory,
        )
        .verified(true)
        .with_warning_consent(DefenderControlConsent::Granted)];
        let plan = build_defender_performance_plan(&request);
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_DEFENDER_SCHEDULE_WINDOW, "20:00-22:00")
            .with_value(TARGET_DEFENDER_EXCLUSION_LIST, "");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("Defender backups should capture");
        let applied = apply_defender_performance_plan_to_fixture(&mut fixture, &plan)
            .expect("Defender fixture apply should succeed");

        assert_eq!(applied.item_count, 2);
        assert_eq!(
            fixture.value(TARGET_DEFENDER_SCHEDULE_WINDOW),
            Some("03:00-05:00")
        );
        assert_eq!(
            fixture.value(TARGET_DEFENDER_EXCLUSION_LIST),
            Some("C:\\Games\\SteamLibrary\\steamapps\\common\\PUBG")
        );

        verify_defender_performance_plan_fixture(&fixture, &plan)
            .expect("Defender fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore Defender fixture state");
        }

        assert_eq!(
            fixture.value(TARGET_DEFENDER_SCHEDULE_WINDOW),
            Some("20:00-22:00")
        );
        assert_eq!(fixture.value(TARGET_DEFENDER_EXCLUSION_LIST), Some(""));
    }

    #[test]
    fn fixture_rejects_global_defender_disable_even_if_marked_apply() {
        let plan = TweakPlan {
            id: "plan-malicious-defender-disable".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: DEFENDER_DISABLE_GLOBAL_TWEAK_ID.to_owned(),
                category: TweakCategory::BlockedGuardrail,
                action: PlanAction::Apply,
                mode: TweakMode::Blocked,
                risk: TweakRisk::Critical,
                changes: vec![PlannedChange {
                    target: TARGET_DEFENDER_GLOBAL_DISABLE.to_owned(),
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

        let error = apply_defender_performance_plan_to_fixture(&mut fixture, &plan)
            .expect_err("global Defender disable must be denied");

        assert_eq!(
            error.reason(),
            DefenderSettingsErrorReason::GlobalDisableDenied
        );
        assert_eq!(error.target(), Some(TARGET_DEFENDER_GLOBAL_DISABLE));
    }
}
