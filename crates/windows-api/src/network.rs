//! Windows scan and fixture adapters for T047 network adapter power controls.

use std::fmt;

use optimizer_core::{
    network::{
        build_network_adapter_power_plan, is_network_adapter_eee_mutation_target,
        is_network_adapter_power_saving_target, is_network_adapter_power_tweak_id,
        AdapterPowerSavingState, NetworkAdapterAdvancedProperty, NetworkAdapterInspection,
        NetworkAdapterPowerPlanRequest, NetworkControlConsent,
        NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID, NET_EEE_GREEN_OFF_TWEAK_ID,
    },
    power_plan::{DevicePowerClass, PowerSourceState},
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{
    NetworkAdapterAdvancedPropertyScanItem, NetworkAdapterScanItem, SystemScanReport,
    WindowsRollbackFixture,
};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapterSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Adapter-specific targets written or verified.
    pub targets: Vec<String>,
}

impl NetworkAdapterSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T047 network adapter power plan from read-only scan data.
#[must_use]
pub fn build_network_adapter_power_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    device_class: DevicePowerClass,
    power_source: PowerSourceState,
) -> TweakPlan {
    let mut request = NetworkAdapterPowerPlanRequest::new(plan_id);
    request.device_class = device_class;
    request.power_source = power_source;
    request.adapters = report
        .network_adapters
        .iter()
        .map(adapter_from_scan)
        .collect();

    build_network_adapter_power_plan(&request)
}

/// Builds a consented T047 network adapter power plan from read-only scan data.
#[must_use]
pub fn build_consented_network_adapter_power_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    device_class: DevicePowerClass,
    power_source: PowerSourceState,
) -> TweakPlan {
    let mut request = NetworkAdapterPowerPlanRequest::new(plan_id);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Competitive;
    request.device_class = device_class;
    request.power_source = power_source;
    request.adapter_power_saving_consent = NetworkControlConsent::Granted;
    request.eee_green_consent = NetworkControlConsent::Granted;
    request.adapters = report
        .network_adapters
        .iter()
        .map(adapter_from_scan)
        .collect();

    build_network_adapter_power_plan(&request)
}

/// Applies T047 network adapter setting changes to an in-memory Windows fixture.
pub fn apply_network_adapter_power_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<NetworkAdapterSettingsSummary, NetworkAdapterSettingsError> {
    let mut summary = NetworkAdapterSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                NetworkAdapterSettingsError::missing_desired_value(
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

/// Verifies T047 network adapter setting changes against an in-memory fixture.
pub fn verify_network_adapter_power_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<NetworkAdapterSettingsSummary, NetworkAdapterSettingsError> {
    let mut summary = NetworkAdapterSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                NetworkAdapterSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(NetworkAdapterSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn adapter_from_scan(item: &NetworkAdapterScanItem) -> NetworkAdapterInspection {
    let adapter_id = item
        .net_connection_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or(item.mac_address.as_deref())
        .unwrap_or(&item.name);
    let mut adapter = NetworkAdapterInspection::new(adapter_id, item.name.clone());
    adapter.adapter_type = item.adapter_type.clone();
    adapter.power_saving = match item.power_management.allow_computer_to_turn_off_device {
        Some(true) => AdapterPowerSavingState::Enabled,
        Some(false) => AdapterPowerSavingState::Disabled,
        None => AdapterPowerSavingState::Unknown,
    };
    adapter.advanced_properties = item
        .advanced_properties
        .iter()
        .map(property_from_scan)
        .collect();
    adapter
}

fn property_from_scan(
    item: &NetworkAdapterAdvancedPropertyScanItem,
) -> NetworkAdapterAdvancedProperty {
    let mut property = NetworkAdapterAdvancedProperty::new(item.display_name.clone());

    if let Some(registry_keyword) = item
        .registry_keyword
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        property = property.with_registry_keyword(registry_keyword);
    }

    if let Some(current_value) = item
        .display_value
        .as_deref()
        .or(item.registry_value.as_deref())
        .filter(|value| !value.trim().is_empty())
    {
        property = property.with_current_value(current_value);
    }

    property
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), NetworkAdapterSettingsError> {
    if is_network_adapter_power_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(NetworkAdapterSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), NetworkAdapterSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(NetworkAdapterSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    let supported_target = match tweak_id {
        NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID => {
            is_network_adapter_power_saving_target(&change.target)
        }
        NET_EEE_GREEN_OFF_TWEAK_ID => is_network_adapter_eee_mutation_target(&change.target),
        _ => false,
    };

    if !supported_target {
        return Err(NetworkAdapterSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

/// Stable failure reason for fixture-backed network adapter operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkAdapterSettingsErrorReason {
    /// Plan item was not part of the T047 network adapter scope.
    UnsupportedTweak,
    /// Plan item targeted a non-adapter-specific or non-T047 setting.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
}

impl NetworkAdapterSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-network-adapter-power tweak",
            Self::UnsupportedTarget => "Plan targets a network setting outside the T047 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Network adapter fixture readback did not match the plan",
        }
    }
}

/// Structured error for fixture-backed network adapter operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapterSettingsError {
    reason: NetworkAdapterSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl NetworkAdapterSettingsError {
    fn new(
        reason: NetworkAdapterSettingsErrorReason,
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
            NetworkAdapterSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NetworkAdapterSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NetworkAdapterSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NetworkAdapterSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NetworkAdapterSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> NetworkAdapterSettingsErrorReason {
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

impl fmt::Display for NetworkAdapterSettingsError {
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

impl std::error::Error for NetworkAdapterSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        network::network_adapter_power_saving_target,
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
    fn scan_fixture_builds_adapter_specific_power_plan() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_network_adapter_power_plan_from_scan(
            "plan-t047-fixture",
            &report,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
        );
        let power = item(&plan, NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID);
        let eee = item(&plan, NET_EEE_GREEN_OFF_TWEAK_ID);

        assert_eq!(power.action, PlanAction::Apply);
        assert_eq!(eee.action, PlanAction::Recommend);
        assert!(power
            .changes
            .iter()
            .all(|change| change.target.starts_with("netadapter:ethernet/")));
        assert_eq!(eee.changes.len(), 1);
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_adapter_values() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_consented_network_adapter_power_plan_from_scan(
            "plan-t047-consented",
            &report,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
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
                    .expect("apply changes should include previous value"),
            );
        }

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("network backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_network_adapter_power_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");
        assert_eq!(applied.item_count, 2);
        assert_eq!(
            fixture.value(&network_adapter_power_saving_target("Ethernet")),
            Some("disabled")
        );

        verify_network_adapter_power_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore adapter fixture state");
        }

        assert_eq!(
            fixture.value(&network_adapter_power_saving_target("Ethernet")),
            Some("enabled")
        );
    }

    #[test]
    fn fixture_rejects_non_adapter_specific_advanced_target() {
        let plan = TweakPlan {
            id: "plan-malicious-network".to_owned(),
            requested_mode: TweakMode::Competitive,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: NET_EEE_GREEN_OFF_TWEAK_ID.to_owned(),
                category: TweakCategory::Network,
                action: PlanAction::Apply,
                mode: TweakMode::Competitive,
                risk: TweakRisk::Medium,
                changes: vec![PlannedChange {
                    target: "netadapter:*/advanced/receive-side-scaling".to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: Some("Enabled".to_owned()),
                    desired_value: Some("Disabled".to_owned()),
                    scope: SessionScope::Persistent,
                }],
                backup: BackupRequirement::Required {
                    kind: RollbackKind::ExactValue,
                    target: "netadapter:*/advanced/receive-side-scaling".to_owned(),
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
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new();

        let error = apply_network_adapter_power_plan_to_fixture(&mut fixture, &plan)
            .expect_err("wildcard adapter target must be denied");

        assert_eq!(
            error.reason(),
            NetworkAdapterSettingsErrorReason::UnsupportedTarget
        );
    }
}
