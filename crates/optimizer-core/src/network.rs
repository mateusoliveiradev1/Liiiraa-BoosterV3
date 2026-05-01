//! Safe planning for network adapter power management and EEE/Green Ethernet controls.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    power_plan::{DevicePowerClass, PowerSourceState},
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for disabling adapter power-saving where the adapter exposes support.
pub const NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID: &str = "net.adapter.power-saving.off";
/// Tweak ID for disabling Energy Efficient Ethernet, Green Ethernet, or Energy Detect.
pub const NET_EEE_GREEN_OFF_TWEAK_ID: &str = "net.eee.green.off";
/// Prefix for adapter-specific logical network targets.
pub const NETWORK_ADAPTER_TARGET_PREFIX: &str = "netadapter:";

const POWER_SAVING_SUFFIX: &str = "/power-management/allow-computer-to-turn-off-device";
const ADVANCED_PROPERTY_PREFIX: &str = "/advanced/";
const DESIRED_POWER_SAVING_STATE: &str = "disabled";
const DESIRED_EEE_STATE: &str = "Disabled";

/// Explicit consent state for prompt-only network controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkControlConsent {
    /// The user has not accepted this optional network control.
    NotGranted,
    /// The user explicitly accepted this optional network control.
    Granted,
}

impl NetworkControlConsent {
    /// Returns true when consent was granted.
    #[must_use]
    pub const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Adapter power-saving state discovered from Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterPowerSavingState {
    /// Windows reports that the adapter can be powered down to save energy.
    Enabled,
    /// Adapter power-saving is already disabled.
    Disabled,
    /// The adapter or driver does not expose the setting.
    Unsupported,
    /// The scan could not determine the setting.
    Unknown,
}

impl AdapterPowerSavingState {
    const fn as_previous_value(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
        }
    }

    const fn needs_disable(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// One advanced adapter property discovered by name on a specific adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapterAdvancedProperty {
    /// Exact display name exposed by Windows or the adapter driver.
    pub display_name: String,
    /// Optional registry keyword associated with the exact property.
    pub registry_keyword: Option<String>,
    /// Current display value for the property, when exposed.
    pub current_value: Option<String>,
}

impl NetworkAdapterAdvancedProperty {
    /// Creates an advanced adapter property with an exact display name.
    #[must_use]
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            registry_keyword: None,
            current_value: None,
        }
    }

    /// Adds the adapter driver's exact registry keyword.
    #[must_use]
    pub fn with_registry_keyword(mut self, registry_keyword: impl Into<String>) -> Self {
        self.registry_keyword = Some(registry_keyword.into());
        self
    }

    /// Adds the current adapter property value.
    #[must_use]
    pub fn with_current_value(mut self, current_value: impl Into<String>) -> Self {
        self.current_value = Some(current_value.into());
        self
    }
}

/// Read-only inspection for one physical network adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapterInspection {
    /// Stable adapter identifier, such as the connection name or interface description.
    pub adapter_id: String,
    /// Human-facing adapter display name.
    pub display_name: String,
    /// Adapter type reported by Windows.
    pub adapter_type: Option<String>,
    /// Power management state for allowing Windows to power down the adapter.
    pub power_saving: AdapterPowerSavingState,
    /// Exact advanced properties exposed by this adapter.
    pub advanced_properties: Vec<NetworkAdapterAdvancedProperty>,
}

impl NetworkAdapterInspection {
    /// Creates a network adapter inspection with unknown power state.
    #[must_use]
    pub fn new(adapter_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            display_name: display_name.into(),
            adapter_type: None,
            power_saving: AdapterPowerSavingState::Unknown,
            advanced_properties: Vec::new(),
        }
    }
}

/// Request used to build the T047 adapter power plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdapterPowerPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Desktop/laptop classification from scan data or caller policy.
    pub device_class: DevicePowerClass,
    /// Current AC/battery state.
    pub power_source: PowerSourceState,
    /// Consent for laptop adapter power-saving changes.
    pub adapter_power_saving_consent: NetworkControlConsent,
    /// Consent for EEE/Green Ethernet changes.
    pub eee_green_consent: NetworkControlConsent,
    /// Physical adapters discovered during scan.
    pub adapters: Vec<NetworkAdapterInspection>,
}

impl NetworkAdapterPowerPlanRequest {
    /// Creates a conservative network adapter power request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            device_class: DevicePowerClass::Desktop,
            power_source: PowerSourceState::Unknown,
            adapter_power_saving_consent: NetworkControlConsent::NotGranted,
            eee_green_consent: NetworkControlConsent::NotGranted,
            adapters: Vec::new(),
        }
    }
}

/// Builds a dry-run plan for T047 NIC power-saving and EEE/Green Ethernet controls.
#[must_use]
pub fn build_network_adapter_power_plan(request: &NetworkAdapterPowerPlanRequest) -> TweakPlan {
    let items = vec![adapter_power_saving_item(request), eee_green_item(request)];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("adapter")
                || warning.contains("EEE")
                || warning.contains("Green Ethernet")
                || warning.contains("battery")
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

/// Returns true when the ID belongs to the T047 network adapter power scope.
#[must_use]
pub fn is_network_adapter_power_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID | NET_EEE_GREEN_OFF_TWEAK_ID
    )
}

/// Builds the adapter-specific logical target for adapter power saving.
#[must_use]
pub fn network_adapter_power_saving_target(adapter_id: &str) -> String {
    format!(
        "{NETWORK_ADAPTER_TARGET_PREFIX}{}{POWER_SAVING_SUFFIX}",
        target_slug(adapter_id)
    )
}

/// Builds the adapter-specific target for an exact EEE/Green Ethernet property.
#[must_use]
pub fn network_adapter_eee_property_target(
    adapter_id: &str,
    property_display_name: &str,
) -> Option<String> {
    eee_property_slug(property_display_name).map(|property| {
        format!(
            "{NETWORK_ADAPTER_TARGET_PREFIX}{}{ADVANCED_PROPERTY_PREFIX}{property}",
            target_slug(adapter_id)
        )
    })
}

/// Returns true when a target is an adapter-specific T047 mutation target.
#[must_use]
pub fn is_network_adapter_power_mutation_target(target: &str) -> bool {
    is_network_adapter_power_saving_target(target) || is_network_adapter_eee_mutation_target(target)
}

/// Returns true when a target is an adapter-specific power-saving target.
#[must_use]
pub fn is_network_adapter_power_saving_target(target: &str) -> bool {
    target.starts_with(NETWORK_ADAPTER_TARGET_PREFIX)
        && !target.contains('*')
        && is_adapter_power_saving_target(target)
}

/// Returns true when a target is an exact EEE/Green Ethernet property target.
#[must_use]
pub fn is_network_adapter_eee_mutation_target(target: &str) -> bool {
    target.starts_with(NETWORK_ADAPTER_TARGET_PREFIX)
        && !target.contains('*')
        && is_adapter_eee_property_target(target)
}

/// Returns true when the plan never attempts broad or advanced non-T047 network writes.
#[must_use]
pub fn network_plan_uses_only_adapter_specific_targets(plan: &TweakPlan) -> bool {
    plan.items
        .iter()
        .flat_map(|item| item.changes.iter())
        .filter(|change| change.operation == TweakOperationKind::Write)
        .all(|change| is_network_adapter_power_mutation_target(&change.target))
}

fn adapter_power_saving_item(request: &NetworkAdapterPowerPlanRequest) -> TweakPlanItem {
    let changes = request
        .adapters
        .iter()
        .filter(|adapter| adapter.power_saving.needs_disable())
        .map(adapter_power_saving_change)
        .collect::<Vec<_>>();
    let mut warnings = adapter_power_saving_warnings(request, changes.is_empty());
    let (mode, risk) = adapter_power_saving_mode(request.device_class);
    let action = if changes.is_empty() {
        PlanAction::DetectOnly
    } else if request.device_class == DevicePowerClass::Desktop {
        PlanAction::Apply
    } else if request.requested_mode == TweakMode::Safe
        || !request.adapter_power_saving_consent.is_granted()
    {
        warnings.push(
            "Laptop adapter power-saving changes are Competitive and require explicit consent."
                .to_owned(),
        );
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    };

    plan_item(
        NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID,
        action,
        mode,
        risk,
        changes,
        true,
        warnings,
    )
}

fn eee_green_item(request: &NetworkAdapterPowerPlanRequest) -> TweakPlanItem {
    let detected_count = request
        .adapters
        .iter()
        .flat_map(|adapter| adapter.advanced_properties.iter())
        .filter(|property| eee_property_slug(&property.display_name).is_some())
        .count();
    let changes = request
        .adapters
        .iter()
        .flat_map(eee_green_changes)
        .collect::<Vec<_>>();
    let mut warnings = Vec::new();

    if detected_count == 0 {
        warnings.push(
            "No exact EEE, Green Ethernet, or Energy Detect adapter property was exposed."
                .to_owned(),
        );
    }

    if request.requested_mode == TweakMode::Safe {
        warnings.push(
            "EEE/Green Ethernet changes are Competitive and stay off in Safe mode.".to_owned(),
        );
    }

    if !request.eee_green_consent.is_granted() {
        warnings.push(
            "EEE/Green Ethernet changes require explicit adapter-specific consent.".to_owned(),
        );
    }

    let action = if changes.is_empty() {
        PlanAction::DetectOnly
    } else if request.requested_mode != TweakMode::Safe && request.eee_green_consent.is_granted() {
        PlanAction::Apply
    } else {
        PlanAction::Recommend
    };

    plan_item(
        NET_EEE_GREEN_OFF_TWEAK_ID,
        action,
        TweakMode::Competitive,
        TweakRisk::Medium,
        changes,
        true,
        warnings,
    )
}

fn adapter_power_saving_mode(device_class: DevicePowerClass) -> (TweakMode, TweakRisk) {
    if device_class == DevicePowerClass::Laptop {
        (TweakMode::Competitive, TweakRisk::Medium)
    } else {
        (TweakMode::Safe, TweakRisk::Low)
    }
}

fn adapter_power_saving_warnings(
    request: &NetworkAdapterPowerPlanRequest,
    no_changes: bool,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if request.adapters.is_empty() {
        warnings.push("No physical network adapters were detected.".to_owned());
    }

    if no_changes && !request.adapters.is_empty() {
        warnings.push(
            "No adapter exposed enabled power saving that needs a change.".to_owned(),
        );
    }

    if request.device_class == DevicePowerClass::Laptop {
        warnings.push(
            "Disabling adapter power saving on laptops can increase battery drain and heat."
                .to_owned(),
        );
    }

    if request.power_source == PowerSourceState::Battery {
        warnings.push("The device is on battery; keep network power changes opt-in.".to_owned());
    }

    warnings
}

fn adapter_power_saving_change(adapter: &NetworkAdapterInspection) -> PlannedChange {
    PlannedChange {
        target: network_adapter_power_saving_target(&adapter.adapter_id),
        operation: TweakOperationKind::Write,
        previous_value: Some(adapter.power_saving.as_previous_value().to_owned()),
        desired_value: Some(DESIRED_POWER_SAVING_STATE.to_owned()),
        scope: SessionScope::Persistent,
    }
}

fn eee_green_changes(adapter: &NetworkAdapterInspection) -> Vec<PlannedChange> {
    adapter
        .advanced_properties
        .iter()
        .filter(|property| {
            eee_property_slug(&property.display_name).is_some()
                && property
                    .current_value
                    .as_deref()
                    .is_some_and(|value| !is_disabled_value(value))
        })
        .filter_map(|property| {
            network_adapter_eee_property_target(&adapter.adapter_id, &property.display_name).map(
                |target| PlannedChange {
                    target,
                    operation: TweakOperationKind::Write,
                    previous_value: property.current_value.clone(),
                    desired_value: Some(DESIRED_EEE_STATE.to_owned()),
                    scope: SessionScope::Persistent,
                },
            )
        })
        .collect()
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    mode: TweakMode,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    requires_admin: bool,
    warnings: Vec<String>,
) -> TweakPlanItem {
    let backup = backup_requirement(action, &changes);
    let rollback = rollback_plan(action, &changes, requires_admin);

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::Network,
        action,
        mode,
        risk,
        changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin,
        warnings,
    }
}

fn backup_requirement(action: PlanAction, changes: &[PlannedChange]) -> BackupRequirement {
    if action == PlanAction::Apply && !changes.is_empty() {
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
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: RollbackKind::ExactValue,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous network adapter setting.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin,
        reboot: RebootPolicy::None,
        manual_instructions: Vec::new(),
    }
}

fn is_adapter_power_saving_target(target: &str) -> bool {
    target.ends_with(POWER_SAVING_SUFFIX) && adapter_segment_is_specific(target)
}

fn is_adapter_eee_property_target(target: &str) -> bool {
    if !adapter_segment_is_specific(target) {
        return false;
    }

    target
        .rsplit_once(ADVANCED_PROPERTY_PREFIX)
        .is_some_and(|(_, property)| {
            matches!(
                property,
                "eee" | "energy-efficient-ethernet" | "green-ethernet" | "energy-detect"
            )
        })
}

fn adapter_segment_is_specific(target: &str) -> bool {
    target
        .strip_prefix(NETWORK_ADAPTER_TARGET_PREFIX)
        .and_then(|rest| rest.split('/').next())
        .is_some_and(|adapter| !adapter.is_empty() && adapter != "all")
}

fn eee_property_slug(display_name: &str) -> Option<String> {
    match normalized(display_name).as_str() {
        "eee" => Some("eee".to_owned()),
        "energyefficientethernet" => Some("energy-efficient-ethernet".to_owned()),
        "greenethernet" => Some("green-ethernet".to_owned()),
        "energydetect" => Some("energy-detect".to_owned()),
        _ => None,
    }
}

fn is_disabled_value(value: &str) -> bool {
    matches!(
        normalized(value).as_str(),
        "disabled" | "disable" | "off" | "false" | "0" | "none"
    )
}

fn target_slug(value: &str) -> String {
    let mut slug = String::new();

    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => slug.push(byte as char),
            b'A'..=b'Z' => slug.push((byte + 32) as char),
            b'.' | b'-' | b'_' => slug.push(byte as char),
            _ if !slug.ends_with('-') => slug.push('-'),
            _ => {}
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "unknown-adapter".to_owned()
    } else {
        slug.to_owned()
    }
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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

    fn adapter() -> NetworkAdapterInspection {
        let mut adapter =
            NetworkAdapterInspection::new("Ethernet", "Realtek Gaming 2.5GbE Family Controller");
        adapter.adapter_type = Some("Ethernet 802.3".to_owned());
        adapter.power_saving = AdapterPowerSavingState::Enabled;
        adapter.advanced_properties = vec![
            NetworkAdapterAdvancedProperty::new("Energy Efficient Ethernet")
                .with_registry_keyword("EEE")
                .with_current_value("Enabled"),
            NetworkAdapterAdvancedProperty::new("Receive Side Scaling")
                .with_registry_keyword("*RSS")
                .with_current_value("Enabled"),
        ];
        adapter
    }

    #[test]
    fn desktop_adapter_power_saving_applies_by_default_with_backup() {
        let mut request = NetworkAdapterPowerPlanRequest::new("plan-network");
        request.device_class = DevicePowerClass::Desktop;
        request.power_source = PowerSourceState::Ac;
        request.adapters = vec![adapter()];

        let plan = build_network_adapter_power_plan(&request);
        let item = item(&plan, NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID);

        assert_eq!(item.action, PlanAction::Apply);
        assert_eq!(item.mode, TweakMode::Safe);
        assert_eq!(
            item.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: network_adapter_power_saving_target("Ethernet"),
            }
        );
        assert_eq!(item.rollback.steps.len(), 1);
        assert!(network_plan_uses_only_adapter_specific_targets(&plan));
    }

    #[test]
    fn laptop_adapter_power_saving_requires_competitive_consent() {
        let mut request = NetworkAdapterPowerPlanRequest::new("plan-laptop-network");
        request.device_class = DevicePowerClass::Laptop;
        request.power_source = PowerSourceState::Battery;
        request.adapters = vec![adapter()];

        let safe_plan = build_network_adapter_power_plan(&request);
        let safe_item = item(&safe_plan, NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID);

        assert_eq!(safe_item.action, PlanAction::Recommend);
        assert_eq!(safe_item.mode, TweakMode::Competitive);
        assert_eq!(safe_item.backup, BackupRequirement::NotRequired);

        request.requested_mode = TweakMode::Competitive;
        request.adapter_power_saving_consent = NetworkControlConsent::Granted;
        let consented_plan = build_network_adapter_power_plan(&request);
        let consented_item = item(&consented_plan, NET_ADAPTER_POWER_SAVING_OFF_TWEAK_ID);

        assert_eq!(consented_item.action, PlanAction::Apply);
        assert!(consented_item
            .warnings
            .iter()
            .any(|warning| warning.contains("battery")));
    }

    #[test]
    fn eee_green_requires_exact_property_and_consent() {
        let mut request = NetworkAdapterPowerPlanRequest::new("plan-eee");
        request.adapters = vec![adapter()];

        let safe_plan = build_network_adapter_power_plan(&request);
        let safe_item = item(&safe_plan, NET_EEE_GREEN_OFF_TWEAK_ID);

        assert_eq!(safe_item.action, PlanAction::Recommend);
        assert_eq!(safe_item.backup, BackupRequirement::NotRequired);
        assert_eq!(safe_item.changes.len(), 1);
        assert!(safe_item.changes[0].target.ends_with(
            "/advanced/energy-efficient-ethernet"
        ));

        request.requested_mode = TweakMode::Competitive;
        request.eee_green_consent = NetworkControlConsent::Granted;
        let consented_plan = build_network_adapter_power_plan(&request);
        let consented_item = item(&consented_plan, NET_EEE_GREEN_OFF_TWEAK_ID);

        assert_eq!(consented_item.action, PlanAction::Apply);
        assert_eq!(consented_item.changes.len(), 1);
        assert!(consented_item
            .changes
            .iter()
            .all(|change| !change.target.contains("receive-side-scaling")));
        assert_eq!(consented_item.rollback.steps.len(), 1);
    }

    #[test]
    fn non_exact_eee_like_property_does_not_generate_write() {
        let mut adapter = adapter();
        adapter.advanced_properties = vec![
            NetworkAdapterAdvancedProperty::new("Energy Efficient Ethernet Mode")
                .with_current_value("Enabled"),
        ];
        let mut request = NetworkAdapterPowerPlanRequest::new("plan-no-wildcards");
        request.requested_mode = TweakMode::Competitive;
        request.eee_green_consent = NetworkControlConsent::Granted;
        request.adapters = vec![adapter];

        let plan = build_network_adapter_power_plan(&request);
        let item = item(&plan, NET_EEE_GREEN_OFF_TWEAK_ID);

        assert_eq!(item.action, PlanAction::DetectOnly);
        assert!(item.changes.is_empty());
        assert!(network_plan_uses_only_adapter_specific_targets(&plan));
    }
}
