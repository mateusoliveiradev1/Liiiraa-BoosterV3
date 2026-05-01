//! Registry-fixture adapter for T052 Windows graphics setting planning.

use std::fmt;

use optimizer_core::{
    graphics_settings::{
        build_graphics_settings_plan, graphics_hags_apply_requires_consent_and_benchmark,
        graphics_plan_has_no_safe_hags_apply, graphics_tweak_targets_setting,
        is_graphics_settings_tweak_id, GraphicsControlConsent, GraphicsPreferenceState,
        GraphicsRegistryDwordState, GraphicsSettingsPlanRequest, HagsBenchmarkTarget,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{SystemScanReport, WindowsRollbackFixture};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Graphics setting targets written or verified.
    pub targets: Vec<String>,
}

impl GraphicsSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T052 graphics settings plan from scan data.
#[must_use]
pub fn build_graphics_settings_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    pubg_executable_path: Option<String>,
) -> TweakPlan {
    let request = graphics_request_from_scan(plan_id, report, pubg_executable_path);

    build_graphics_settings_plan(&request)
}

/// Builds a consented T052 graphics settings plan from scan data.
#[must_use]
pub fn build_consented_graphics_settings_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    pubg_executable_path: Option<String>,
    enable_windowed_optimizations: bool,
    enable_variable_refresh_rate: bool,
    hags_target: HagsBenchmarkTarget,
    baseline_benchmark_captured: bool,
) -> TweakPlan {
    let mut request = graphics_request_from_scan(plan_id, report, pubg_executable_path);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Competitive;
    request.windowed_optimizations_consent = if enable_windowed_optimizations {
        GraphicsControlConsent::Granted
    } else {
        GraphicsControlConsent::NotGranted
    };
    request.variable_refresh_rate_consent = if enable_variable_refresh_rate {
        GraphicsControlConsent::Granted
    } else {
        GraphicsControlConsent::NotGranted
    };
    request.hags_consent = GraphicsControlConsent::Granted;
    request.hags_target = hags_target;
    request.baseline_benchmark_captured = baseline_benchmark_captured;

    build_graphics_settings_plan(&request)
}

/// Applies T052 graphics setting registry changes to an in-memory Windows fixture.
pub fn apply_graphics_settings_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<GraphicsSettingsSummary, GraphicsSettingsError> {
    validate_graphics_plan(plan)?;

    let mut summary = GraphicsSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                GraphicsSettingsError::missing_desired_value(
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

/// Verifies T052 graphics setting registry changes against an in-memory fixture.
pub fn verify_graphics_settings_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<GraphicsSettingsSummary, GraphicsSettingsError> {
    validate_graphics_plan(plan)?;

    let mut summary = GraphicsSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                GraphicsSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(GraphicsSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn graphics_request_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    pubg_executable_path: Option<String>,
) -> GraphicsSettingsPlanRequest {
    let mut request = GraphicsSettingsPlanRequest::new(plan_id);
    request.pubg_graphics_preference =
        pubg_preference_state_from_scan(report, pubg_executable_path.as_deref());
    request.pubg_executable_path = pubg_executable_path;
    request.high_performance_gpu_available = report.graphics.high_performance_gpu_available;
    request.windowed_optimizations =
        GraphicsRegistryDwordState::from_option(report.graphics.windowed_optimizations.value);
    request.windowed_optimizations_supported = report.graphics.windowed_optimizations.supported;
    request.variable_refresh_rate =
        GraphicsRegistryDwordState::from_option(report.graphics.variable_refresh_rate.value);
    request.variable_refresh_rate_supported = report.graphics.variable_refresh_rate.supported;
    request.hags = GraphicsRegistryDwordState::from_option(report.graphics.hags.value);
    request.hags_supported = report.graphics.hags.supported;
    request
}

fn pubg_preference_state_from_scan(
    report: &SystemScanReport,
    pubg_executable_path: Option<&str>,
) -> GraphicsPreferenceState {
    let Some(pubg_executable_path) = pubg_executable_path else {
        return GraphicsPreferenceState::Unknown;
    };

    report
        .graphics
        .app_preferences
        .iter()
        .find(|preference| preference.executable_path.eq_ignore_ascii_case(pubg_executable_path))
        .map_or(GraphicsPreferenceState::Missing, |preference| {
            GraphicsPreferenceState::Value(preference.preference.clone())
        })
}

fn validate_graphics_plan(plan: &TweakPlan) -> Result<(), GraphicsSettingsError> {
    if graphics_plan_has_no_safe_hags_apply(plan)
        && graphics_hags_apply_requires_consent_and_benchmark(plan)
    {
        Ok(())
    } else {
        Err(GraphicsSettingsError::safe_hags_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), GraphicsSettingsError> {
    if is_graphics_settings_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(GraphicsSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), GraphicsSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(GraphicsSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !graphics_tweak_targets_setting(tweak_id, &change.target) {
        return Err(GraphicsSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed graphics setting operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsSettingsErrorReason {
    /// Plan item was not part of the T052 graphics settings scope.
    UnsupportedTweak,
    /// Plan item targeted a registry value outside the T052 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// Plan attempted to apply HAGS from Safe/default planning.
    SafeHagsDenied,
}

impl GraphicsSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::SafeHagsDenied => "safe_hags_denied",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-graphics-settings tweak",
            Self::UnsupportedTarget => "Plan targets a registry value outside the T052 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Graphics setting fixture readback did not match the plan",
            Self::SafeHagsDenied => {
                "HAGS Competitive changes must not apply from Safe/default planning"
            }
        }
    }
}

/// Structured error for fixture-backed graphics setting operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsSettingsError {
    reason: GraphicsSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl GraphicsSettingsError {
    fn new(
        reason: GraphicsSettingsErrorReason,
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
            GraphicsSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GraphicsSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GraphicsSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GraphicsSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            GraphicsSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn safe_hags_denied() -> Self {
        Self::new(GraphicsSettingsErrorReason::SafeHagsDenied, None, None)
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> GraphicsSettingsErrorReason {
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

impl fmt::Display for GraphicsSettingsError {
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

impl std::error::Error for GraphicsSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        graphics_settings::{
            graphics_preference_target, GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID,
            GAME_HAGS_BENCHMARK_TWEAK_ID, GAME_VRR_DETECT_PLAN_TWEAK_ID,
            GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID, TARGET_HAGS_MODE,
            TARGET_VARIABLE_REFRESH_RATE, TARGET_WINDOWED_OPTIMIZATIONS,
        },
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            SessionScope, TweakCategory, TweakMode, TweakPlanItem, TweakRisk,
        },
    };

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");
    const PUBG_EXE: &str = "C:\\Program Files\\PUBG\\TslGame.exe";

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
    fn scan_fixture_builds_graphics_plan_without_safe_hags_apply() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_graphics_settings_plan_from_scan(
            "plan-t052-fixture",
            &report,
            Some(PUBG_EXE.to_owned()),
        );

        assert_eq!(
            item(&plan, GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID).action,
            PlanAction::Apply
        );
        assert_eq!(
            item(&plan, GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID).action,
            PlanAction::Recommend
        );
        assert_eq!(
            item(&plan, GAME_VRR_DETECT_PLAN_TWEAK_ID).action,
            PlanAction::Recommend
        );
        assert_eq!(
            item(&plan, GAME_HAGS_BENCHMARK_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert!(graphics_plan_has_no_safe_hags_apply(&plan));
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_graphics_settings() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_consented_graphics_settings_plan_from_scan(
            "plan-t052-consented",
            &report,
            Some(PUBG_EXE.to_owned()),
            true,
            true,
            HagsBenchmarkTarget::Enable,
            true,
        );
        let pubg_target = graphics_preference_target(PUBG_EXE);
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
                    .expect("fixture changes should include previous values"),
            );
        }

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("graphics backups should capture");
        assert_eq!(backups.len(), 4);

        let applied = apply_graphics_settings_plan_to_fixture(&mut fixture, &plan)
            .expect("graphics fixture apply should succeed");
        assert_eq!(applied.item_count, 4);
        assert_eq!(fixture.value(&pubg_target), Some("GpuPreference=2;"));
        assert_eq!(fixture.value(TARGET_WINDOWED_OPTIMIZATIONS), Some("1"));
        assert_eq!(fixture.value(TARGET_VARIABLE_REFRESH_RATE), Some("1"));
        assert_eq!(fixture.value(TARGET_HAGS_MODE), Some("2"));

        verify_graphics_settings_plan_fixture(&fixture, &plan)
            .expect("graphics fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore graphics fixture state");
        }

        assert_eq!(fixture.value(&pubg_target), Some("GpuPreference=1;"));
        assert_eq!(fixture.value(TARGET_WINDOWED_OPTIMIZATIONS), Some("0"));
        assert_eq!(fixture.value(TARGET_VARIABLE_REFRESH_RATE), Some("0"));
        assert_eq!(fixture.value(TARGET_HAGS_MODE), Some("1"));
    }

    #[test]
    fn fixture_rejects_safe_mode_hags_apply() {
        let plan = TweakPlan {
            id: "plan-malicious-hags".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: GAME_HAGS_BENCHMARK_TWEAK_ID.to_owned(),
                category: TweakCategory::WindowsGaming,
                action: PlanAction::Apply,
                mode: TweakMode::Competitive,
                risk: TweakRisk::Medium,
                changes: vec![PlannedChange {
                    target: TARGET_HAGS_MODE.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("1".to_owned()),
                    desired_value: Some("2".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: TARGET_HAGS_MODE.to_owned(),
                },
                rollback: RollbackPlan {
                    kind: RollbackKind::ExactValue,
                    steps: Vec::new(),
                    requires_admin: true,
                    reboot: RebootPolicy::Required,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::Required,
                requires_admin: true,
                warnings: vec![
                    "HAGS benchmark path requires explicit consent.".to_owned(),
                    "Baseline benchmark is required before applying graphics setting changes."
                        .to_owned(),
                ],
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new().with_value(TARGET_HAGS_MODE, "1");

        let error = apply_graphics_settings_plan_to_fixture(&mut fixture, &plan)
            .expect_err("safe/default HAGS apply must be denied");

        assert_eq!(error.reason(), GraphicsSettingsErrorReason::SafeHagsDenied);
    }

    #[test]
    fn fixture_rejects_cross_wired_graphics_target() {
        let plan = TweakPlan {
            id: "plan-cross-wired-graphics".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID.to_owned(),
                category: TweakCategory::WindowsGaming,
                action: PlanAction::Apply,
                mode: TweakMode::Safe,
                risk: TweakRisk::Low,
                changes: vec![PlannedChange {
                    target: TARGET_VARIABLE_REFRESH_RATE.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("0".to_owned()),
                    desired_value: Some("1".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: TARGET_VARIABLE_REFRESH_RATE.to_owned(),
                },
                rollback: RollbackPlan {
                    kind: RollbackKind::ExactValue,
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
        let mut fixture =
            WindowsRollbackFixture::new().with_value(TARGET_VARIABLE_REFRESH_RATE, "0");

        let error = apply_graphics_settings_plan_to_fixture(&mut fixture, &plan)
            .expect_err("cross-wired graphics target must be denied");

        assert_eq!(error.reason(), GraphicsSettingsErrorReason::UnsupportedTarget);
    }
}
