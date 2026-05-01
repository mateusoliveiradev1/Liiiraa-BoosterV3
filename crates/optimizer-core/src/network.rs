//! Safe planning for network adapter power management plus Lab-only advanced NIC controls.

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
/// Tweak ID for benchmark-gated Receive Side Scaling enablement.
pub const NET_RSS_ENSURE_TWEAK_ID: &str = "net.rss.ensure";
/// Tweak ID for benchmark-gated Receive Segment Coalescing profiling.
pub const NET_RSC_PROFILE_TWEAK_ID: &str = "net.rsc.profile";
/// Tweak ID for VPN/capture-tool RSC and offload diagnostics.
pub const NET_RSC_VPN_DIAGNOSIS_TWEAK_ID: &str = "net.rsc.vpn-diagnosis";
/// Tweak ID for keeping checksum and large-send offloads conservative by default.
pub const NET_OFFLOADS_KEEP_DEFAULT_TWEAK_ID: &str = "net.offloads.keep-default";
/// Tweak ID for benchmark-gated interrupt moderation tuning.
pub const NET_INTERRUPT_MODERATION_LAB_TWEAK_ID: &str = "net.interrupt-moderation.lab";
/// Prefix for adapter-specific logical network targets.
pub const NETWORK_ADAPTER_TARGET_PREFIX: &str = "netadapter:";

const POWER_SAVING_SUFFIX: &str = "/power-management/allow-computer-to-turn-off-device";
const ADVANCED_PROPERTY_PREFIX: &str = "/advanced/";
const DESIRED_POWER_SAVING_STATE: &str = "disabled";
const DESIRED_EEE_STATE: &str = "Disabled";
const DESIRED_ADVANCED_ENABLED_STATE: &str = "Enabled";
const DESIRED_ADVANCED_DISABLED_STATE: &str = "Disabled";
const ADVANCED_NETWORK_BENCHMARK_WARNING: &str = concat!(
    "Baseline benchmark is required before applying advanced network tuning; compare ",
    "latency, jitter, throughput, and frametime stability before and after."
);
const ADVANCED_NETWORK_RESTART_WARNING: &str = concat!(
    "Adapter restart or a brief link interruption may be required after changing advanced ",
    "network adapter properties."
);

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

/// Request used to build the T055 advanced NIC Lab plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkAdvancedTuningPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Consent for Receive Side Scaling enablement.
    pub rss_consent: NetworkControlConsent,
    /// Consent for Receive Segment Coalescing profiling.
    pub rsc_consent: NetworkControlConsent,
    /// Consent for VPN/capture-tool offload diagnostics.
    pub offload_diagnostics_consent: NetworkControlConsent,
    /// Consent for interrupt moderation profiling.
    pub interrupt_moderation_consent: NetworkControlConsent,
    /// Whether a baseline benchmark exists before applying advanced NIC changes.
    pub baseline_benchmark_captured: bool,
    /// Whether the user accepted adapter restart or link-flap risk.
    pub adapter_restart_accepted: bool,
    /// Whether the user identified VPN, packet-capture, or driver-specific network symptoms.
    pub diagnostic_issue_confirmed: bool,
    /// Physical adapters discovered during scan.
    pub adapters: Vec<NetworkAdapterInspection>,
}

impl NetworkAdvancedTuningPlanRequest {
    /// Creates a conservative advanced NIC request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            rss_consent: NetworkControlConsent::NotGranted,
            rsc_consent: NetworkControlConsent::NotGranted,
            offload_diagnostics_consent: NetworkControlConsent::NotGranted,
            interrupt_moderation_consent: NetworkControlConsent::NotGranted,
            baseline_benchmark_captured: false,
            adapter_restart_accepted: false,
            diagnostic_issue_confirmed: false,
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

/// Builds a dry-run plan for T055 advanced NIC Lab tuning.
#[must_use]
pub fn build_network_advanced_tuning_plan(
    request: &NetworkAdvancedTuningPlanRequest,
) -> TweakPlan {
    let items = vec![
        rss_ensure_item(request),
        rsc_profile_item(request),
        rsc_vpn_diagnosis_item(request),
        offloads_keep_default_item(request),
        interrupt_moderation_lab_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Lab")
                || warning.contains("benchmark")
                || warning.contains("adapter")
                || warning.contains("offload")
                || warning.contains("RSC")
                || warning.contains("RSS")
                || warning.contains("Interrupt")
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

/// Returns true when the ID belongs to the T055 advanced NIC tuning scope.
#[must_use]
pub fn is_network_advanced_tuning_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        NET_RSS_ENSURE_TWEAK_ID
            | NET_RSC_PROFILE_TWEAK_ID
            | NET_RSC_VPN_DIAGNOSIS_TWEAK_ID
            | NET_OFFLOADS_KEEP_DEFAULT_TWEAK_ID
            | NET_INTERRUPT_MODERATION_LAB_TWEAK_ID
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

/// Builds the adapter-specific target for an allowlisted advanced NIC property.
#[must_use]
pub fn network_adapter_advanced_property_target(
    adapter_id: &str,
    property: &NetworkAdapterAdvancedProperty,
) -> Option<String> {
    advanced_property_kind(property).map(|_| {
        format!(
            "{NETWORK_ADAPTER_TARGET_PREFIX}{}{ADVANCED_PROPERTY_PREFIX}{}",
            target_slug(adapter_id),
            target_slug(&property.display_name)
        )
    })
}

/// Returns true when a target is an adapter-specific T047 mutation target.
#[must_use]
pub fn is_network_adapter_power_mutation_target(target: &str) -> bool {
    is_network_adapter_power_saving_target(target) || is_network_adapter_eee_mutation_target(target)
}

/// Returns true when a target is an adapter-specific T055 advanced NIC property target.
#[must_use]
pub fn is_network_advanced_tuning_target(target: &str) -> bool {
    target.starts_with(NETWORK_ADAPTER_TARGET_PREFIX)
        && !target.contains('*')
        && advanced_property_kind_from_target(target).is_some()
}

/// Returns true when a T055 tweak ID is paired with an allowed advanced NIC target.
#[must_use]
pub fn network_advanced_tweak_targets_property(tweak_id: &str, target: &str) -> bool {
    let Some(kind) = advanced_property_kind_from_target(target) else {
        return false;
    };

    matches!(
        (tweak_id, kind),
        (NET_RSS_ENSURE_TWEAK_ID, AdvancedNetworkPropertyKind::Rss)
            | (NET_RSC_PROFILE_TWEAK_ID, AdvancedNetworkPropertyKind::Rsc)
            | (NET_RSC_VPN_DIAGNOSIS_TWEAK_ID, AdvancedNetworkPropertyKind::Rsc)
            | (NET_RSC_VPN_DIAGNOSIS_TWEAK_ID, AdvancedNetworkPropertyKind::Offload)
            | (
                NET_INTERRUPT_MODERATION_LAB_TWEAK_ID,
                AdvancedNetworkPropertyKind::InterruptModeration
            )
    )
}

/// Returns true when Safe/default planning does not apply advanced NIC changes.
#[must_use]
pub fn network_advanced_plan_is_not_safe_default(plan: &TweakPlan) -> bool {
    plan.requested_mode != TweakMode::Safe
        || plan.items.iter().all(|item| item.action != PlanAction::Apply)
}

/// Returns true when apply items are Lab, explicitly consented, and benchmark-framed.
#[must_use]
pub fn network_advanced_apply_requires_lab_consent_and_benchmark(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if item.action != PlanAction::Apply {
            return true;
        }

        item.mode == TweakMode::Lab
            && item.risk == TweakRisk::High
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit consent"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("Baseline benchmark"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("Adapter restart"))
    })
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

fn rss_ensure_item(request: &NetworkAdvancedTuningPlanRequest) -> TweakPlanItem {
    advanced_lab_item(
        AdvancedLabItemInput {
            tweak_id: NET_RSS_ENSURE_TWEAK_ID,
            kind: AdvancedNetworkPropertyKind::Rss,
            desired_value: DESIRED_ADVANCED_ENABLED_STATE,
            consent: request.rss_consent,
            summary: "Receive Side Scaling",
            no_support_warning: "No exact Receive Side Scaling adapter property was exposed.",
            issue_context_required: false,
        },
        request,
    )
}

fn rsc_profile_item(request: &NetworkAdvancedTuningPlanRequest) -> TweakPlanItem {
    advanced_lab_item(
        AdvancedLabItemInput {
            tweak_id: NET_RSC_PROFILE_TWEAK_ID,
            kind: AdvancedNetworkPropertyKind::Rsc,
            desired_value: DESIRED_ADVANCED_DISABLED_STATE,
            consent: request.rsc_consent,
            summary: "Receive Segment Coalescing",
            no_support_warning: "No exact Receive Segment Coalescing adapter property was exposed.",
            issue_context_required: false,
        },
        request,
    )
}

fn rsc_vpn_diagnosis_item(request: &NetworkAdvancedTuningPlanRequest) -> TweakPlanItem {
    let changes = advanced_property_changes(
        &request.adapters,
        AdvancedNetworkPropertyKind::Offload,
        DESIRED_ADVANCED_DISABLED_STATE,
    );
    let warnings = advanced_lab_warnings(
        request,
        request.offload_diagnostics_consent,
        "RSC/offload VPN or capture-tool diagnosis",
        changes.is_empty(),
        "No exact RSC or checksum/large-send offload adapter property was exposed.",
        true,
    );
    let action = advanced_lab_action(
        request,
        request.offload_diagnostics_consent,
        changes.is_empty(),
        true,
    );

    advanced_plan_item(AdvancedPlanItemInput {
        tweak_id: NET_RSC_VPN_DIAGNOSIS_TWEAK_ID,
        action,
        mode: TweakMode::Lab,
        risk: TweakRisk::High,
        changes,
        rollback_kind: RollbackKind::ExactValue,
        requires_admin: true,
        warnings,
    })
}

fn offloads_keep_default_item(request: &NetworkAdvancedTuningPlanRequest) -> TweakPlanItem {
    let detected_count = request
        .adapters
        .iter()
        .flat_map(|adapter| adapter.advanced_properties.iter())
        .filter(|property| {
            advanced_property_kind(property) == Some(AdvancedNetworkPropertyKind::Offload)
        })
        .count();
    let mut warnings = vec![
        "Checksum and large-send offloads stay at adapter defaults unless a Lab diagnostic plan \
         owns a benchmarked change."
            .to_owned(),
        "No offload changes are included in Safe/default optimization.".to_owned(),
        "No game files, memory, BattlEye files, or anti-cheat processes are modified.".to_owned(),
    ];

    if detected_count == 0 {
        warnings.push("No checksum or large-send offload adapter property was exposed.".to_owned());
    }

    advanced_plan_item(AdvancedPlanItemInput {
        tweak_id: NET_OFFLOADS_KEEP_DEFAULT_TWEAK_ID,
        action: PlanAction::DetectOnly,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes: Vec::new(),
        rollback_kind: RollbackKind::NotNeededReadonly,
        requires_admin: false,
        warnings,
    })
}

fn interrupt_moderation_lab_item(request: &NetworkAdvancedTuningPlanRequest) -> TweakPlanItem {
    advanced_lab_item(
        AdvancedLabItemInput {
            tweak_id: NET_INTERRUPT_MODERATION_LAB_TWEAK_ID,
            kind: AdvancedNetworkPropertyKind::InterruptModeration,
            desired_value: DESIRED_ADVANCED_DISABLED_STATE,
            consent: request.interrupt_moderation_consent,
            summary: "Interrupt moderation",
            no_support_warning: "No exact interrupt moderation adapter property was exposed.",
            issue_context_required: false,
        },
        request,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdvancedNetworkPropertyKind {
    Rss,
    Rsc,
    Offload,
    InterruptModeration,
}

#[derive(Clone, Copy)]
struct AdvancedLabItemInput<'a> {
    tweak_id: &'a str,
    kind: AdvancedNetworkPropertyKind,
    desired_value: &'a str,
    consent: NetworkControlConsent,
    summary: &'a str,
    no_support_warning: &'a str,
    issue_context_required: bool,
}

fn advanced_lab_item(
    input: AdvancedLabItemInput<'_>,
    request: &NetworkAdvancedTuningPlanRequest,
) -> TweakPlanItem {
    let changes = advanced_property_changes(&request.adapters, input.kind, input.desired_value);
    let warnings = advanced_lab_warnings(
        request,
        input.consent,
        input.summary,
        changes.is_empty(),
        input.no_support_warning,
        input.issue_context_required,
    );
    let action = advanced_lab_action(
        request,
        input.consent,
        changes.is_empty(),
        input.issue_context_required,
    );

    advanced_plan_item(AdvancedPlanItemInput {
        tweak_id: input.tweak_id,
        action,
        mode: TweakMode::Lab,
        risk: TweakRisk::High,
        changes,
        rollback_kind: RollbackKind::ExactValue,
        requires_admin: true,
        warnings,
    })
}

fn advanced_lab_action(
    request: &NetworkAdvancedTuningPlanRequest,
    consent: NetworkControlConsent,
    no_changes: bool,
    issue_context_required: bool,
) -> PlanAction {
    if no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode != TweakMode::Lab
        || !consent.is_granted()
        || !request.baseline_benchmark_captured
        || !request.adapter_restart_accepted
        || (issue_context_required && !request.diagnostic_issue_confirmed)
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn advanced_lab_warnings(
    request: &NetworkAdvancedTuningPlanRequest,
    consent: NetworkControlConsent,
    summary: &str,
    no_changes: bool,
    no_support_warning: &str,
    issue_context_required: bool,
) -> Vec<String> {
    let mut warnings = vec![
        format!("{summary} is Lab-only and requires explicit consent."),
        ADVANCED_NETWORK_BENCHMARK_WARNING.to_owned(),
        ADVANCED_NETWORK_RESTART_WARNING.to_owned(),
        "Advanced NIC changes are adapter-specific; wildcard writes and viral TCP packs are not \
         allowed."
            .to_owned(),
        "No game files, memory, BattlEye files, or anti-cheat processes are modified.".to_owned(),
    ];

    if request.requested_mode != TweakMode::Lab {
        warnings.push("Advanced NIC tuning stays off in Safe/default planning.".to_owned());
    }

    if !consent.is_granted() {
        warnings.push(format!("{summary} consent has not been granted."));
    }

    if !request.baseline_benchmark_captured {
        warnings.push(
            "Capture a baseline benchmark before applying this network tweak.".to_owned(),
        );
    }

    if !request.adapter_restart_accepted {
        warnings.push("Adapter restart risk has not been accepted.".to_owned());
    }

    if issue_context_required && !request.diagnostic_issue_confirmed {
        warnings.push(
            "VPN, capture-tool, or adapter-driver issue context is required before offload \
             diagnostics."
                .to_owned(),
        );
    }

    if no_changes {
        warnings.push(no_support_warning.to_owned());
    }

    warnings
}

fn advanced_property_changes(
    adapters: &[NetworkAdapterInspection],
    kind: AdvancedNetworkPropertyKind,
    desired_value: &str,
) -> Vec<PlannedChange> {
    adapters
        .iter()
        .flat_map(|adapter| {
            adapter
                .advanced_properties
                .iter()
                .filter(move |property| advanced_property_kind(property) == Some(kind))
                .filter(move |property| property_needs_advanced_change(property, desired_value))
                .filter_map(move |property| {
                    network_adapter_advanced_property_target(&adapter.adapter_id, property).map(
                        |target| PlannedChange {
                            target,
                            operation: TweakOperationKind::Write,
                            previous_value: property.current_value.clone(),
                            desired_value: Some(desired_value.to_owned()),
                            scope: SessionScope::Persistent,
                        },
                    )
                })
        })
        .collect()
}

fn property_needs_advanced_change(
    property: &NetworkAdapterAdvancedProperty,
    desired_value: &str,
) -> bool {
    let Some(current_value) = property.current_value.as_deref() else {
        return false;
    };

    if is_disabled_value(desired_value) {
        !is_disabled_value(current_value)
    } else if is_enabled_value(desired_value) {
        !is_enabled_value(current_value)
    } else {
        normalized(current_value) != normalized(desired_value)
    }
}

struct AdvancedPlanItemInput {
    tweak_id: &'static str,
    action: PlanAction,
    mode: TweakMode,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    rollback_kind: RollbackKind,
    requires_admin: bool,
    warnings: Vec<String>,
}

fn advanced_plan_item(input: AdvancedPlanItemInput) -> TweakPlanItem {
    let backup = backup_requirement(input.action, input.rollback_kind, &input.changes);
    let rollback = rollback_plan(
        input.action,
        input.rollback_kind,
        &input.changes,
        input.requires_admin,
    );

    TweakPlanItem {
        tweak_id: input.tweak_id.to_owned(),
        category: TweakCategory::Network,
        action: input.action,
        mode: input.mode,
        risk: input.risk,
        changes: input.changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin: input.requires_admin,
        warnings: input.warnings,
    }
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
    let backup = backup_requirement(action, RollbackKind::ExactValue, &changes);
    let rollback = rollback_plan(action, RollbackKind::ExactValue, &changes, requires_admin);

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

fn backup_requirement(
    action: PlanAction,
    rollback_kind: RollbackKind,
    changes: &[PlannedChange],
) -> BackupRequirement {
    if action == PlanAction::Apply && rollback_kind.needs_backup_before_apply() {
        BackupRequirement::Required {
            kind: rollback_kind,
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
    rollback_kind: RollbackKind,
    changes: &[PlannedChange],
    requires_admin: bool,
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: rollback_kind,
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

fn advanced_property_kind_from_target(target: &str) -> Option<AdvancedNetworkPropertyKind> {
    if !adapter_segment_is_specific(target) {
        return None;
    }

    target
        .rsplit_once(ADVANCED_PROPERTY_PREFIX)
        .and_then(|(_, property)| advanced_property_kind_from_slug(property))
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

fn advanced_property_kind(
    property: &NetworkAdapterAdvancedProperty,
) -> Option<AdvancedNetworkPropertyKind> {
    property
        .registry_keyword
        .as_deref()
        .and_then(advanced_property_kind_from_slug)
        .or_else(|| advanced_property_kind_from_slug(&property.display_name))
}

fn advanced_property_kind_from_slug(value: &str) -> Option<AdvancedNetworkPropertyKind> {
    let value = normalized(value);

    if matches!(value.as_str(), "rss" | "receivesidescaling") {
        return Some(AdvancedNetworkPropertyKind::Rss);
    }

    if matches!(value.as_str(), "rsc" | "receivesegmentcoalescing")
        || value.contains("receivesegmentcoalescing")
        || value.contains("recvsegmentcoalescing")
    {
        return Some(AdvancedNetworkPropertyKind::Rsc);
    }

    if value.contains("offload")
        || value.contains("largesend")
        || value.contains("lso")
        || value.contains("checksum")
    {
        return Some(AdvancedNetworkPropertyKind::Offload);
    }

    if value.contains("interruptmoderation") {
        return Some(AdvancedNetworkPropertyKind::InterruptModeration);
    }

    None
}

fn is_disabled_value(value: &str) -> bool {
    matches!(
        normalized(value).as_str(),
        "disabled" | "disable" | "off" | "false" | "0" | "none"
    )
}

fn is_enabled_value(value: &str) -> bool {
    matches!(
        normalized(value).as_str(),
        "enabled" | "enable" | "on" | "true" | "1"
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

    fn advanced_adapter() -> NetworkAdapterInspection {
        let mut adapter =
            NetworkAdapterInspection::new("Ethernet", "Realtek Gaming 2.5GbE Family Controller");
        adapter.adapter_type = Some("Ethernet 802.3".to_owned());
        adapter.advanced_properties = vec![
            NetworkAdapterAdvancedProperty::new("Receive Side Scaling")
                .with_registry_keyword("*RSS")
                .with_current_value("Disabled"),
            NetworkAdapterAdvancedProperty::new("Receive Segment Coalescing")
                .with_registry_keyword("*RSC")
                .with_current_value("Enabled"),
            NetworkAdapterAdvancedProperty::new("Large Send Offload v2 (IPv4)")
                .with_registry_keyword("*LsoV2IPv4")
                .with_current_value("Enabled"),
            NetworkAdapterAdvancedProperty::new("Interrupt Moderation")
                .with_registry_keyword("*InterruptModeration")
                .with_current_value("Enabled"),
            NetworkAdapterAdvancedProperty::new("Jumbo Packet")
                .with_registry_keyword("*JumboPacket")
                .with_current_value("1514 Bytes"),
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

    #[test]
    fn advanced_network_tuning_stays_conservative_by_default() {
        let mut request = NetworkAdvancedTuningPlanRequest::new("plan-advanced-safe");
        request.adapters = vec![advanced_adapter()];

        let plan = build_network_advanced_tuning_plan(&request);
        let rss = item(&plan, NET_RSS_ENSURE_TWEAK_ID);
        let offloads = item(&plan, NET_OFFLOADS_KEEP_DEFAULT_TWEAK_ID);

        assert!(!plan.has_apply_items());
        assert!(network_advanced_plan_is_not_safe_default(&plan));
        assert_eq!(rss.action, PlanAction::Recommend);
        assert_eq!(rss.mode, TweakMode::Lab);
        assert_eq!(rss.backup, BackupRequirement::NotRequired);
        assert_eq!(offloads.action, PlanAction::DetectOnly);
        assert_eq!(offloads.mode, TweakMode::Safe);
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("Safe/default")));
    }

    #[test]
    fn advanced_network_tuning_requires_lab_consent_and_benchmark() {
        let mut request = NetworkAdvancedTuningPlanRequest::new("plan-advanced-lab");
        request.requested_mode = TweakMode::Lab;
        request.adapters = vec![advanced_adapter()];

        let no_consent_plan = build_network_advanced_tuning_plan(&request);
        assert_eq!(
            item(&no_consent_plan, NET_RSC_PROFILE_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.rss_consent = NetworkControlConsent::Granted;
        request.rsc_consent = NetworkControlConsent::Granted;
        request.offload_diagnostics_consent = NetworkControlConsent::Granted;
        request.interrupt_moderation_consent = NetworkControlConsent::Granted;
        request.diagnostic_issue_confirmed = true;
        let no_baseline_plan = build_network_advanced_tuning_plan(&request);
        assert_eq!(
            item(&no_baseline_plan, NET_INTERRUPT_MODERATION_LAB_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.baseline_benchmark_captured = true;
        request.adapter_restart_accepted = true;
        let apply_plan = build_network_advanced_tuning_plan(&request);
        let rss = item(&apply_plan, NET_RSS_ENSURE_TWEAK_ID);
        let rsc = item(&apply_plan, NET_RSC_PROFILE_TWEAK_ID);
        let offload = item(&apply_plan, NET_RSC_VPN_DIAGNOSIS_TWEAK_ID);
        let interrupt = item(&apply_plan, NET_INTERRUPT_MODERATION_LAB_TWEAK_ID);

        assert_eq!(rss.action, PlanAction::Apply);
        assert_eq!(rsc.action, PlanAction::Apply);
        assert_eq!(offload.action, PlanAction::Apply);
        assert_eq!(interrupt.action, PlanAction::Apply);
        assert_eq!(
            rss.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: "netadapter:ethernet/advanced/receive-side-scaling".to_owned(),
            }
        );
        assert_eq!(rss.changes[0].desired_value.as_deref(), Some("Enabled"));
        assert!(rsc.changes[0]
            .target
            .ends_with("/advanced/receive-segment-coalescing"));
        assert!(offload.changes[0]
            .target
            .ends_with("/advanced/large-send-offload-v2-ipv4"));
        assert!(interrupt.changes[0]
            .target
            .ends_with("/advanced/interrupt-moderation"));
        assert!(network_advanced_apply_requires_lab_consent_and_benchmark(
            &apply_plan
        ));
    }

    #[test]
    fn advanced_network_targets_are_adapter_specific_and_allowlisted() {
        let mut request = NetworkAdvancedTuningPlanRequest::new("plan-advanced-targets");
        request.requested_mode = TweakMode::Lab;
        request.rss_consent = NetworkControlConsent::Granted;
        request.rsc_consent = NetworkControlConsent::Granted;
        request.offload_diagnostics_consent = NetworkControlConsent::Granted;
        request.interrupt_moderation_consent = NetworkControlConsent::Granted;
        request.baseline_benchmark_captured = true;
        request.adapter_restart_accepted = true;
        request.diagnostic_issue_confirmed = true;
        request.adapters = vec![advanced_adapter()];

        let plan = build_network_advanced_tuning_plan(&request);

        assert!(plan
            .items
            .iter()
            .flat_map(|item| item.changes.iter().map(move |change| (&item.tweak_id, change)))
            .all(|(tweak_id, change)| network_advanced_tweak_targets_property(
                tweak_id,
                &change.target
            )));
        assert!(!is_network_advanced_tuning_target(
            "netadapter:*/advanced/interrupt-moderation"
        ));
        assert!(!is_network_advanced_tuning_target(
            "netadapter:ethernet/advanced/jumbo-packet"
        ));
    }
}
