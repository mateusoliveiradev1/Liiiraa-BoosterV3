//! Competitive planning for VBS, HVCI, VMP, and Hyper-V security tradeoffs.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for read-only VBS, HVCI, Credential Guard, and VMP detection.
pub const SECURITY_VBS_DETECT_TWEAK_ID: &str = "security.vbs.detect";
/// Tweak ID for the Memory Integrity / HVCI tradeoff.
pub const SECURITY_HVCI_TRADEOFF_TWEAK_ID: &str = "security.hvci.tradeoff";
/// Tweak ID for the Virtual Machine Platform tradeoff.
pub const SECURITY_VMP_TRADEOFF_TWEAK_ID: &str = "security.vmp.tradeoff";
/// Tweak ID for the broader Hyper-V stack tradeoff.
pub const SECURITY_HYPERV_TRADEOFF_TWEAK_ID: &str = "security.hyperv.tradeoff";

/// Logical target for the HVCI registry switch.
pub const TARGET_HVCI_ENABLED: &str =
    "registry:hklm/system/currentcontrolset/control/deviceguard/scenarios/hypervisorenforcedcodeintegrity/enabled";
/// Logical target for the Virtual Machine Platform optional feature.
pub const TARGET_VMP_FEATURE: &str = "windows-feature:virtualmachineplatform";
/// Logical target for the Hyper-V optional feature.
pub const TARGET_HYPERV_FEATURE: &str = "windows-feature:microsoft-hyper-v-all";

const SECURITY_TRADEOFF_WARNING: &str = concat!(
    "This is a security tradeoff: changing virtualization security can reduce protection ",
    "or break virtualization workloads."
);

/// Coarse enabled/disabled state for a Windows security or optional feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityFeatureState {
    /// The feature appears enabled.
    Enabled,
    /// The feature appears disabled.
    Disabled,
    /// The feature state could not be proven.
    Unknown,
}

impl SecurityFeatureState {
    /// Converts an optional bool into a conservative feature state.
    #[must_use]
    pub const fn from_option(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Enabled,
            Some(false) => Self::Disabled,
            None => Self::Unknown,
        }
    }

    /// Converts an optional DWORD where non-zero means enabled.
    #[must_use]
    pub const fn from_registry_dword(value: Option<u32>) -> Self {
        match value {
            Some(0) => Self::Disabled,
            Some(_) => Self::Enabled,
            None => Self::Unknown,
        }
    }

    /// Converts `Win32_DeviceGuard.VirtualizationBasedSecurityStatus`.
    #[must_use]
    pub const fn from_device_guard_vbs_status(value: Option<u32>) -> Self {
        match value {
            Some(2) => Self::Enabled,
            Some(0 | 1) => Self::Disabled,
            Some(_) | None => Self::Unknown,
        }
    }

    /// Converts `Win32_OptionalFeature.InstallState`.
    #[must_use]
    pub const fn from_optional_feature_install_state(value: Option<u32>) -> Self {
        match value {
            Some(1) => Self::Enabled,
            Some(2 | 3) => Self::Disabled,
            Some(_) | None => Self::Unknown,
        }
    }

    const fn as_previous_value(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unknown => "unknown",
        }
    }

    const fn as_registry_previous_value(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "0",
            Self::Unknown => "unknown",
        }
    }

    const fn matches_desired(self, desired: SecurityFeatureDesiredState) -> bool {
        matches!(
            (self, desired),
            (Self::Enabled, SecurityFeatureDesiredState::Enabled)
                | (Self::Disabled, SecurityFeatureDesiredState::Disabled)
        )
    }
}

/// Desired state for a security tradeoff target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityFeatureDesiredState {
    /// Enable the target feature.
    Enabled,
    /// Disable the target feature.
    Disabled,
}

impl SecurityFeatureDesiredState {
    /// Returns a stable string for feature targets.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    const fn registry_value(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "0",
        }
    }
}

/// Explicit consent state for security tradeoffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityTradeoffConsent {
    /// The user has not accepted the warned action.
    NotGranted,
    /// The user explicitly accepted the warned action.
    Granted,
}

impl SecurityTradeoffConsent {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Whether the user still needs the local virtualization stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualizationStackUse {
    /// WSL, Hyper-V, VM tooling, emulators, or similar workloads are needed.
    Needed,
    /// The user confirmed those workloads are not needed.
    NotNeeded,
    /// The app could not prove whether the stack is needed.
    Unknown,
}

impl VirtualizationStackUse {
    const fn blocks_disable(self) -> bool {
        !matches!(self, Self::NotNeeded)
    }
}

/// Request used to build the T050 security tradeoff plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityTradeoffPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Virtualization-Based Security status.
    pub vbs: SecurityFeatureState,
    /// Credential Guard status inferred from Device Guard services.
    pub credential_guard: SecurityFeatureState,
    /// Memory Integrity / HVCI state.
    pub hvci: SecurityFeatureState,
    /// Whether HVCI is locked by policy or firmware.
    pub hvci_locked: bool,
    /// Virtual Machine Platform optional feature state.
    pub vmp: SecurityFeatureState,
    /// Hyper-V optional feature state.
    pub hyperv: SecurityFeatureState,
    /// Windows Subsystem for Linux optional feature state.
    pub wsl: SecurityFeatureState,
    /// Whether local virtualization workloads are still needed.
    pub virtualization_stack_use: VirtualizationStackUse,
    /// Count of known incompatible drivers for enabling HVCI.
    pub incompatible_driver_count: u16,
    /// Whether Windows already has a pending reboot.
    pub pending_reboot: bool,
    /// Desired HVCI state, when the user chose one.
    pub desired_hvci_state: Option<SecurityFeatureDesiredState>,
    /// Desired VMP state, when the user chose one.
    pub desired_vmp_state: Option<SecurityFeatureDesiredState>,
    /// Desired Hyper-V state, when the user chose one.
    pub desired_hyperv_state: Option<SecurityFeatureDesiredState>,
    /// Consent for HVCI tradeoff changes.
    pub hvci_consent: SecurityTradeoffConsent,
    /// Consent for VMP tradeoff changes.
    pub vmp_consent: SecurityTradeoffConsent,
    /// Consent for Hyper-V tradeoff changes.
    pub hyperv_consent: SecurityTradeoffConsent,
}

impl SecurityTradeoffPlanRequest {
    /// Creates a conservative read-only security tradeoff request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            vbs: SecurityFeatureState::Unknown,
            credential_guard: SecurityFeatureState::Unknown,
            hvci: SecurityFeatureState::Unknown,
            hvci_locked: false,
            vmp: SecurityFeatureState::Unknown,
            hyperv: SecurityFeatureState::Unknown,
            wsl: SecurityFeatureState::Unknown,
            virtualization_stack_use: VirtualizationStackUse::Unknown,
            incompatible_driver_count: 0,
            pending_reboot: false,
            desired_hvci_state: None,
            desired_vmp_state: None,
            desired_hyperv_state: None,
            hvci_consent: SecurityTradeoffConsent::NotGranted,
            vmp_consent: SecurityTradeoffConsent::NotGranted,
            hyperv_consent: SecurityTradeoffConsent::NotGranted,
        }
    }
}

/// Builds a dry-run plan for T050 security tradeoffs.
#[must_use]
pub fn build_security_tradeoff_plan(request: &SecurityTradeoffPlanRequest) -> TweakPlan {
    let items = vec![
        vbs_detect_item(request),
        hvci_tradeoff_item(request),
        vmp_tradeoff_item(request),
        hyperv_tradeoff_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("security")
                || warning.contains("reboot")
                || warning.contains("Virtual")
                || warning.contains("HVCI")
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

/// Returns true when the ID belongs to the T050 security tradeoff scope.
#[must_use]
pub fn is_security_tradeoff_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        SECURITY_VBS_DETECT_TWEAK_ID
            | SECURITY_HVCI_TRADEOFF_TWEAK_ID
            | SECURITY_VMP_TRADEOFF_TWEAK_ID
            | SECURITY_HYPERV_TRADEOFF_TWEAK_ID
    )
}

/// Returns true when the target is an allowlisted T050 mutation target.
#[must_use]
pub fn is_security_tradeoff_mutation_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_HVCI_ENABLED | TARGET_VMP_FEATURE | TARGET_HYPERV_FEATURE
    )
}

/// Returns true when no security tradeoff is applied from a Safe/default request.
#[must_use]
pub fn security_tradeoff_plan_is_not_safe_default(plan: &TweakPlan) -> bool {
    plan.requested_mode != TweakMode::Safe
        || plan.items.iter().all(|item| item.action != PlanAction::Apply)
}

/// Returns true when apply items stay inside explicit-consent tradeoff policy.
#[must_use]
pub fn security_tradeoff_plan_requires_explicit_consent(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if item.action != PlanAction::Apply {
            return true;
        }

        item.mode == TweakMode::Competitive
            && item.reboot == RebootPolicy::Required
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit consent"))
    })
}

fn vbs_detect_item(request: &SecurityTradeoffPlanRequest) -> TweakPlanItem {
    let mut warnings = Vec::new();

    if request.vbs == SecurityFeatureState::Unknown {
        warnings.push("VBS status is unknown; keep security tradeoffs prompt-only.".to_owned());
    }

    if request.credential_guard == SecurityFeatureState::Enabled {
        warnings.push(
            "Credential Guard appears enabled; virtualization security changes need extra review."
                .to_owned(),
        );
    }

    TweakPlanItem {
        tweak_id: SECURITY_VBS_DETECT_TWEAK_ID.to_owned(),
        category: TweakCategory::SecurityTradeoff,
        action: PlanAction::DetectOnly,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes: Vec::new(),
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

fn hvci_tradeoff_item(request: &SecurityTradeoffPlanRequest) -> TweakPlanItem {
    let desired = request.desired_hvci_state;
    let changes = desired
        .filter(|desired| !request.hvci.matches_desired(*desired))
        .map(|desired| {
            write_change(
                TARGET_HVCI_ENABLED,
                desired.registry_value(),
                request.hvci.as_registry_previous_value(),
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    let mut warnings = common_tradeoff_warnings(request);

    warnings.push("Memory Integrity / HVCI changes require explicit consent.".to_owned());

    if request.hvci_locked {
        warnings.push(
            "HVCI is locked by policy or firmware; do not plan an automatic change.".to_owned(),
        );
    }

    if desired == Some(SecurityFeatureDesiredState::Enabled)
        && request.incompatible_driver_count > 0
    {
        warnings.push(format!(
            "{} incompatible driver(s) were detected; resolve them before enabling HVCI.",
            request.incompatible_driver_count
        ));
    }

    let blocker = request.hvci_locked
        || (desired == Some(SecurityFeatureDesiredState::Enabled)
            && request.incompatible_driver_count > 0);
    let action = tradeoff_action(request, request.hvci_consent, changes.is_empty(), blocker);

    plan_item(SECURITY_HVCI_TRADEOFF_TWEAK_ID, action, changes, warnings)
}

fn vmp_tradeoff_item(request: &SecurityTradeoffPlanRequest) -> TweakPlanItem {
    let desired = request.desired_vmp_state;
    let changes = desired
        .filter(|desired| !request.vmp.matches_desired(*desired))
        .map(|desired| {
            write_change(
                TARGET_VMP_FEATURE,
                desired.as_str(),
                request.vmp.as_previous_value(),
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    let mut warnings = common_tradeoff_warnings(request);

    warnings.push("Virtual Machine Platform changes require explicit consent.".to_owned());
    add_virtualization_stack_warnings(request, desired, &mut warnings);

    let blocker = desired == Some(SecurityFeatureDesiredState::Disabled)
        && request.virtualization_stack_use.blocks_disable();
    let action = tradeoff_action(request, request.vmp_consent, changes.is_empty(), blocker);

    plan_item(SECURITY_VMP_TRADEOFF_TWEAK_ID, action, changes, warnings)
}

fn hyperv_tradeoff_item(request: &SecurityTradeoffPlanRequest) -> TweakPlanItem {
    let desired = request.desired_hyperv_state;
    let changes = desired
        .filter(|desired| !request.hyperv.matches_desired(*desired))
        .map(|desired| {
            write_change(
                TARGET_HYPERV_FEATURE,
                desired.as_str(),
                request.hyperv.as_previous_value(),
            )
        })
        .into_iter()
        .collect::<Vec<_>>();
    let mut warnings = common_tradeoff_warnings(request);

    warnings.push("Hyper-V stack changes require explicit consent.".to_owned());
    add_virtualization_stack_warnings(request, desired, &mut warnings);

    let blocker = desired == Some(SecurityFeatureDesiredState::Disabled)
        && request.virtualization_stack_use.blocks_disable();
    let action = tradeoff_action(request, request.hyperv_consent, changes.is_empty(), blocker);

    plan_item(SECURITY_HYPERV_TRADEOFF_TWEAK_ID, action, changes, warnings)
}

fn tradeoff_action(
    request: &SecurityTradeoffPlanRequest,
    consent: SecurityTradeoffConsent,
    no_changes: bool,
    blocker: bool,
) -> PlanAction {
    if no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode == TweakMode::Safe
        || !consent.is_granted()
        || request.pending_reboot
        || blocker
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn common_tradeoff_warnings(request: &SecurityTradeoffPlanRequest) -> Vec<String> {
    let mut warnings = vec![SECURITY_TRADEOFF_WARNING.to_owned()];

    if request.requested_mode == TweakMode::Safe {
        warnings.push("Security tradeoffs are Competitive and stay off in Safe mode.".to_owned());
    }

    if request.pending_reboot {
        warnings.push(
            "A reboot is already pending; finish that reboot before security tradeoff apply."
                .to_owned(),
        );
    }

    warnings
}

fn add_virtualization_stack_warnings(
    request: &SecurityTradeoffPlanRequest,
    desired: Option<SecurityFeatureDesiredState>,
    warnings: &mut Vec<String>,
) {
    if desired != Some(SecurityFeatureDesiredState::Disabled) {
        return;
    }

    match request.virtualization_stack_use {
        VirtualizationStackUse::Needed => warnings.push(
            "WSL, VM, emulator, or Hyper-V workloads appear needed; do not disable the virtualization stack."
                .to_owned(),
        ),
        VirtualizationStackUse::Unknown => warnings.push(
            "Virtualization workload usage is unknown; require the user to confirm WSL, VMs, and emulators are not needed."
                .to_owned(),
        ),
        VirtualizationStackUse::NotNeeded => {}
    }

    if request.wsl == SecurityFeatureState::Enabled {
        warnings.push("WSL appears enabled; VMP disable would break WSL workflows.".to_owned());
    }

    if request.hyperv == SecurityFeatureState::Enabled
        && desired == Some(SecurityFeatureDesiredState::Disabled)
    {
        warnings.push("Hyper-V appears enabled; VMP changes may affect VM workflows.".to_owned());
    }
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    changes: Vec<PlannedChange>,
    warnings: Vec<String>,
) -> TweakPlanItem {
    let reboot = if changes.is_empty() {
        RebootPolicy::None
    } else {
        RebootPolicy::Required
    };
    let backup = backup_requirement(action, &changes);
    let rollback = rollback_plan(action, &changes);

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::SecurityTradeoff,
        action,
        mode: TweakMode::Competitive,
        risk: TweakRisk::High,
        changes,
        backup,
        rollback,
        reboot,
        requires_admin: reboot == RebootPolicy::Required,
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

fn rollback_plan(action: PlanAction, changes: &[PlannedChange]) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: RollbackKind::ExactValue,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous virtualization security setting.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin: true,
        reboot: RebootPolicy::Required,
        manual_instructions: Vec::new(),
    }
}

fn write_change(target: &str, desired: &str, previous: &str) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value: Some(previous.to_owned()),
        desired_value: Some(desired.to_owned()),
        scope: SessionScope::Persistent,
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
    fn safe_mode_never_applies_hvci_tradeoff_even_with_consent() {
        let mut request = SecurityTradeoffPlanRequest::new("plan-safe-security");
        request.hvci = SecurityFeatureState::Enabled;
        request.desired_hvci_state = Some(SecurityFeatureDesiredState::Disabled);
        request.hvci_consent = SecurityTradeoffConsent::Granted;

        let plan = build_security_tradeoff_plan(&request);
        let hvci = item(&plan, SECURITY_HVCI_TRADEOFF_TWEAK_ID);

        assert_eq!(hvci.action, PlanAction::Recommend);
        assert_eq!(hvci.mode, TweakMode::Competitive);
        assert_eq!(hvci.backup, BackupRequirement::NotRequired);
        assert!(!plan.has_apply_items());
        assert!(security_tradeoff_plan_is_not_safe_default(&plan));
    }

    #[test]
    fn hvci_tradeoff_requires_competitive_mode_and_explicit_consent() {
        let mut request = SecurityTradeoffPlanRequest::new("plan-hvci");
        request.requested_mode = TweakMode::Competitive;
        request.hvci = SecurityFeatureState::Enabled;
        request.desired_hvci_state = Some(SecurityFeatureDesiredState::Disabled);

        let recommend_plan = build_security_tradeoff_plan(&request);
        let hvci = item(&recommend_plan, SECURITY_HVCI_TRADEOFF_TWEAK_ID);

        assert_eq!(hvci.action, PlanAction::Recommend);
        assert_eq!(hvci.backup, BackupRequirement::NotRequired);

        request.hvci_consent = SecurityTradeoffConsent::Granted;
        let apply_plan = build_security_tradeoff_plan(&request);
        let hvci = item(&apply_plan, SECURITY_HVCI_TRADEOFF_TWEAK_ID);

        assert_eq!(hvci.action, PlanAction::Apply);
        assert_eq!(hvci.reboot, RebootPolicy::Required);
        assert_eq!(
            hvci.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_HVCI_ENABLED.to_owned(),
            }
        );
        assert_eq!(hvci.rollback.reboot, RebootPolicy::Required);
        assert!(security_tradeoff_plan_requires_explicit_consent(&apply_plan));
    }

    #[test]
    fn vmp_disable_is_recommended_until_virtualization_stack_is_confirmed_unused() {
        let mut request = SecurityTradeoffPlanRequest::new("plan-vmp");
        request.requested_mode = TweakMode::Competitive;
        request.vmp = SecurityFeatureState::Enabled;
        request.wsl = SecurityFeatureState::Enabled;
        request.virtualization_stack_use = VirtualizationStackUse::Needed;
        request.desired_vmp_state = Some(SecurityFeatureDesiredState::Disabled);
        request.vmp_consent = SecurityTradeoffConsent::Granted;

        let blocked_plan = build_security_tradeoff_plan(&request);
        let vmp = item(&blocked_plan, SECURITY_VMP_TRADEOFF_TWEAK_ID);

        assert_eq!(vmp.action, PlanAction::Recommend);
        assert!(vmp
            .warnings
            .iter()
            .any(|warning| warning.contains("WSL")));

        request.wsl = SecurityFeatureState::Disabled;
        request.virtualization_stack_use = VirtualizationStackUse::NotNeeded;
        let apply_plan = build_security_tradeoff_plan(&request);
        let vmp = item(&apply_plan, SECURITY_VMP_TRADEOFF_TWEAK_ID);

        assert_eq!(vmp.action, PlanAction::Apply);
        assert_eq!(vmp.changes[0].target, TARGET_VMP_FEATURE);
        assert_eq!(vmp.changes[0].desired_value.as_deref(), Some("disabled"));
    }

    #[test]
    fn pending_reboot_blocks_security_tradeoff_apply() {
        let mut request = SecurityTradeoffPlanRequest::new("plan-pending-reboot");
        request.requested_mode = TweakMode::Competitive;
        request.hvci = SecurityFeatureState::Enabled;
        request.desired_hvci_state = Some(SecurityFeatureDesiredState::Disabled);
        request.hvci_consent = SecurityTradeoffConsent::Granted;
        request.pending_reboot = true;

        let plan = build_security_tradeoff_plan(&request);
        let hvci = item(&plan, SECURITY_HVCI_TRADEOFF_TWEAK_ID);

        assert_eq!(hvci.action, PlanAction::Recommend);
        assert!(hvci
            .warnings
            .iter()
            .any(|warning| warning.contains("pending")));
    }
}
