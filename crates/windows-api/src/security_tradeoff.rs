//! Windows scan and fixture adapters for VBS, HVCI, VMP, and Hyper-V tradeoffs.

use std::fmt;

use optimizer_core::{
    security_tradeoff::{
        build_security_tradeoff_plan, is_security_tradeoff_mutation_target,
        is_security_tradeoff_tweak_id, security_tradeoff_plan_is_not_safe_default,
        security_tradeoff_plan_requires_explicit_consent, SecurityFeatureDesiredState,
        SecurityFeatureState, SecurityTradeoffConsent, SecurityTradeoffPlanRequest,
        VirtualizationStackUse, SECURITY_HVCI_TRADEOFF_TWEAK_ID,
        SECURITY_VMP_TRADEOFF_TWEAK_ID,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::scan::DeviceGuardScan;
use crate::{
    SystemScanReport, WindowsOptionalFeatureScanItem, WindowsRollbackFixture,
};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityTradeoffSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Security tradeoff targets written or verified.
    pub targets: Vec<String>,
    /// Whether any applied or verified target requires a reboot boundary.
    pub reboot_required: bool,
}

impl SecurityTradeoffSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
            reboot_required: false,
        }
    }
}

/// Builds a T050 read-only security tradeoff plan from scan data.
#[must_use]
pub fn build_security_tradeoff_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let request = security_tradeoff_request_from_scan(plan_id, report);

    build_security_tradeoff_plan(&request)
}

/// Builds a consented T050 plan from scan data and explicit desired states.
#[must_use]
pub fn build_consented_security_tradeoff_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    desired_hvci_state: Option<SecurityFeatureDesiredState>,
    desired_vmp_state: Option<SecurityFeatureDesiredState>,
    desired_hyperv_state: Option<SecurityFeatureDesiredState>,
    virtualization_stack_use: VirtualizationStackUse,
) -> TweakPlan {
    let mut request = security_tradeoff_request_from_scan(plan_id, report);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Competitive;
    request.desired_hvci_state = desired_hvci_state;
    request.desired_vmp_state = desired_vmp_state;
    request.desired_hyperv_state = desired_hyperv_state;
    request.virtualization_stack_use = virtualization_stack_use;
    request.hvci_consent = SecurityTradeoffConsent::Granted;
    request.vmp_consent = SecurityTradeoffConsent::Granted;
    request.hyperv_consent = SecurityTradeoffConsent::Granted;

    build_security_tradeoff_plan(&request)
}

/// Applies T050 security tradeoff fixture changes.
pub fn apply_security_tradeoff_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<SecurityTradeoffSettingsSummary, SecurityTradeoffSettingsError> {
    validate_explicit_tradeoff_plan(plan)?;

    let mut summary = SecurityTradeoffSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;
        summary.reboot_required |=
            item.reboot == optimizer_core::tweak_contracts::RebootPolicy::Required;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                SecurityTradeoffSettingsError::missing_desired_value(
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

/// Verifies T050 security tradeoff fixture changes.
pub fn verify_security_tradeoff_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<SecurityTradeoffSettingsSummary, SecurityTradeoffSettingsError> {
    validate_explicit_tradeoff_plan(plan)?;

    let mut summary = SecurityTradeoffSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;
        summary.reboot_required |=
            item.reboot == optimizer_core::tweak_contracts::RebootPolicy::Required;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                SecurityTradeoffSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(SecurityTradeoffSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Returns true when the plan contains no Safe/default security tradeoff apply.
#[must_use]
pub fn security_tradeoff_plan_is_conservative(plan: &TweakPlan) -> bool {
    security_tradeoff_plan_is_not_safe_default(plan)
        && security_tradeoff_plan_requires_explicit_consent(plan)
}

fn security_tradeoff_request_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> SecurityTradeoffPlanRequest {
    let mut request = SecurityTradeoffPlanRequest::new(plan_id);
    request.vbs = SecurityFeatureState::from_device_guard_vbs_status(
        report
            .security
            .device_guard
            .virtualization_based_security_status,
    );
    request.credential_guard = device_guard_service_state(&report.security.device_guard, 1);
    request.hvci = SecurityFeatureState::from_registry_dword(report.security.hvci.enabled)
        .or_else(device_guard_service_state(&report.security.device_guard, 2));
    request.hvci_locked = report.security.hvci.locked.is_some_and(|value| value != 0);
    request.vmp = optional_feature_state(
        &report.security.optional_features,
        "VirtualMachinePlatform",
    );
    request.hyperv = optional_feature_state(
        &report.security.optional_features,
        "Microsoft-Hyper-V-All",
    );
    request.wsl = optional_feature_state(
        &report.security.optional_features,
        "Microsoft-Windows-Subsystem-Linux",
    );
    request.virtualization_stack_use = virtualization_stack_use_from_scan(&request);
    request.pending_reboot = report.reboot_required.is_reboot_required();
    request
}

trait FeatureStateExt {
    fn or_else(self, fallback: SecurityFeatureState) -> Self;
}

impl FeatureStateExt for SecurityFeatureState {
    fn or_else(self, fallback: SecurityFeatureState) -> Self {
        if self == Self::Unknown {
            fallback
        } else {
            self
        }
    }
}

fn device_guard_service_state(
    device_guard: &DeviceGuardScan,
    service_code: u32,
) -> SecurityFeatureState {
    if device_guard.security_services_running.contains(&service_code) {
        SecurityFeatureState::Enabled
    } else if device_guard.security_services_configured.contains(&service_code) {
        SecurityFeatureState::Disabled
    } else {
        SecurityFeatureState::Unknown
    }
}

fn optional_feature_state(
    features: &[WindowsOptionalFeatureScanItem],
    feature_name: &str,
) -> SecurityFeatureState {
    features
        .iter()
        .find(|feature| feature.name.eq_ignore_ascii_case(feature_name))
        .map_or(SecurityFeatureState::Unknown, |feature| {
            SecurityFeatureState::from_optional_feature_install_state(feature.install_state)
        })
}

fn virtualization_stack_use_from_scan(
    request: &SecurityTradeoffPlanRequest,
) -> VirtualizationStackUse {
    if request.wsl == SecurityFeatureState::Enabled
        || request.hyperv == SecurityFeatureState::Enabled
    {
        VirtualizationStackUse::Needed
    } else {
        VirtualizationStackUse::Unknown
    }
}

fn validate_explicit_tradeoff_plan(
    plan: &TweakPlan,
) -> Result<(), SecurityTradeoffSettingsError> {
    if security_tradeoff_plan_is_conservative(plan) {
        Ok(())
    } else {
        Err(SecurityTradeoffSettingsError::safe_default_denied())
    }
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), SecurityTradeoffSettingsError> {
    if is_security_tradeoff_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(SecurityTradeoffSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), SecurityTradeoffSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(SecurityTradeoffSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_security_tradeoff_mutation_target(&change.target) {
        return Err(SecurityTradeoffSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed security tradeoff operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityTradeoffSettingsErrorReason {
    /// Plan item was not part of the T050 security tradeoff scope.
    UnsupportedTweak,
    /// Plan item targeted a setting outside the T050 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// Plan attempted a Safe/default security tradeoff apply.
    SafeDefaultDenied,
}

impl SecurityTradeoffSettingsErrorReason {
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
            Self::UnsupportedTweak => "Plan contains a non-security-tradeoff tweak",
            Self::UnsupportedTarget => "Plan targets a setting outside the T050 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Security tradeoff fixture readback did not match the plan",
            Self::SafeDefaultDenied => {
                "Security tradeoffs must not apply from Safe/default planning"
            }
        }
    }
}

/// Structured error for fixture-backed security tradeoff operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityTradeoffSettingsError {
    reason: SecurityTradeoffSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl SecurityTradeoffSettingsError {
    fn new(
        reason: SecurityTradeoffSettingsErrorReason,
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
            SecurityTradeoffSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SecurityTradeoffSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SecurityTradeoffSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SecurityTradeoffSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            SecurityTradeoffSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn safe_default_denied() -> Self {
        Self::new(
            SecurityTradeoffSettingsErrorReason::SafeDefaultDenied,
            None,
            None,
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> SecurityTradeoffSettingsErrorReason {
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

impl fmt::Display for SecurityTradeoffSettingsError {
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

impl std::error::Error for SecurityTradeoffSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        security_tradeoff::{TARGET_HVCI_ENABLED, TARGET_VMP_FEATURE},
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
    fn scan_fixture_builds_security_tradeoff_detection_without_safe_apply() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_security_tradeoff_plan_from_scan("plan-t050-fixture", &report);
        let hvci = item(&plan, SECURITY_HVCI_TRADEOFF_TWEAK_ID);
        let vmp = item(&plan, SECURITY_VMP_TRADEOFF_TWEAK_ID);

        assert_eq!(hvci.action, PlanAction::DetectOnly);
        assert_eq!(vmp.action, PlanAction::DetectOnly);
        assert!(security_tradeoff_plan_is_conservative(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_consented_security_tradeoffs() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_consented_security_tradeoff_plan_from_scan(
            "plan-t050-consented",
            &report,
            Some(SecurityFeatureDesiredState::Disabled),
            Some(SecurityFeatureDesiredState::Disabled),
            None,
            VirtualizationStackUse::NotNeeded,
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
                    .expect("security changes should include previous value"),
            );
        }

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("security backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_security_tradeoff_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");

        assert_eq!(applied.item_count, 2);
        assert!(applied.reboot_required);
        assert_eq!(fixture.value(TARGET_HVCI_ENABLED), Some("0"));
        assert_eq!(fixture.value(TARGET_VMP_FEATURE), Some("disabled"));

        verify_security_tradeoff_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore security fixture state");
        }

        assert_eq!(fixture.value(TARGET_HVCI_ENABLED), Some("1"));
        assert_eq!(fixture.value(TARGET_VMP_FEATURE), Some("enabled"));
    }

    #[test]
    fn fixture_rejects_safe_mode_security_tradeoff_apply() {
        let plan = TweakPlan {
            id: "plan-malicious-security".to_owned(),
            requested_mode: TweakMode::Safe,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: SECURITY_HVCI_TRADEOFF_TWEAK_ID.to_owned(),
                category: TweakCategory::SecurityTradeoff,
                action: PlanAction::Apply,
                mode: TweakMode::Competitive,
                risk: TweakRisk::High,
                changes: vec![PlannedChange {
                    target: TARGET_HVCI_ENABLED.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("enabled".to_owned()),
                    desired_value: Some("0".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: TARGET_HVCI_ENABLED.to_owned(),
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
                warnings: vec!["requires explicit consent".to_owned()],
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new().with_value(TARGET_HVCI_ENABLED, "enabled");

        let error = apply_security_tradeoff_plan_to_fixture(&mut fixture, &plan)
            .expect_err("safe/default security tradeoff apply must be denied");

        assert_eq!(
            error.reason(),
            SecurityTradeoffSettingsErrorReason::SafeDefaultDenied
        );
    }
}
