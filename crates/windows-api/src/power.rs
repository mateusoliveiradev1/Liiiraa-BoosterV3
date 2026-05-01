//! Windows `powercfg` plans for Liiiraa Boost power profiles.

use std::fmt;

use optimizer_core::power_plan::{
    evaluate_power_plan_policy, DevicePowerClass, LiiiraaPowerPlanProfile, PowerPlanConsent,
    PowerPlanPolicyAction, PowerPlanPolicyDecision, PowerPlanPolicyRequest, PowerSourceState,
};

use crate::{StructuredCommandPlan, WindowsCommandPlanError};

/// Built-in Windows Balanced power scheme GUID.
pub const WINDOWS_BALANCED_SCHEME_GUID: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";

/// Built-in Windows High Performance power scheme GUID.
pub const WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID: &str =
    "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";

/// Liiiraa-owned Balanced scheme GUID.
pub const LIIIRAA_BALANCED_SCHEME_GUID: &str = "2f5f87ba-3c75-4b5c-9bb8-118e6e11b001";

/// Liiiraa-owned Performance scheme GUID.
pub const LIIIRAA_PERFORMANCE_SCHEME_GUID: &str =
    "2f5f87ba-3c75-4b5c-9bb8-118e6e11b002";

/// Liiiraa-owned Competitive scheme GUID.
pub const LIIIRAA_COMPETITIVE_SCHEME_GUID: &str =
    "2f5f87ba-3c75-4b5c-9bb8-118e6e11b003";

const SUB_USB: &str = "SUB_USB";
const USB_SELECTIVE_SUSPEND: &str = "USBSELECTIVE";
const SUB_PCI_EXPRESS: &str = "SUB_PCIEXPRESS";
const PCI_EXPRESS_LINK_STATE: &str = "ASPM";
const SUB_DISK: &str = "SUB_DISK";
const DISK_IDLE_TIMEOUT: &str = "DISKIDLE";
const SUB_PROCESSOR: &str = "SUB_PROCESSOR";
const PROCESSOR_MINIMUM_STATE: &str = "PROCTHROTTLEMIN";
const PROCESSOR_MAXIMUM_STATE: &str = "PROCTHROTTLEMAX";

/// Request for a reversible Liiiraa power-plan command sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiiiraaPowerPlanApplyRequest {
    /// Requested Liiiraa profile.
    pub profile: LiiiraaPowerPlanProfile,
    /// Desktop/laptop classification from scan data.
    pub device_class: DevicePowerClass,
    /// Current AC/battery state from scan data.
    pub power_source: PowerSourceState,
    /// Whether the user accepted disclosed power tradeoffs.
    pub consent: PowerPlanConsent,
    /// Active scheme GUID captured before apply.
    pub previous_active_scheme_guid: String,
}

impl LiiiraaPowerPlanApplyRequest {
    /// Creates a request using the previously active scheme captured by scan/backup.
    #[must_use]
    pub fn new(
        profile: LiiiraaPowerPlanProfile,
        device_class: DevicePowerClass,
        power_source: PowerSourceState,
        consent: PowerPlanConsent,
        previous_active_scheme_guid: impl Into<String>,
    ) -> Self {
        Self {
            profile,
            device_class,
            power_source,
            consent,
            previous_active_scheme_guid: previous_active_scheme_guid.into(),
        }
    }
}

/// Reversible command plan for applying one Liiiraa power profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiiiraaPowerCfgApplyPlan {
    /// Requested profile.
    pub profile: LiiiraaPowerPlanProfile,
    /// Deterministic Liiiraa scheme GUID created by the plan.
    pub scheme_guid: String,
    /// Windows display name assigned to the scheme.
    pub display_name: String,
    /// Policy decision that allowed this plan to be built.
    pub policy: PowerPlanPolicyDecision,
    /// Ordered `powercfg` commands for duplicate, settings, and activation.
    pub commands: Vec<StructuredCommandPlan>,
    /// Readback command used to verify the active scheme.
    pub verify_active_scheme: StructuredCommandPlan,
    /// Rollback command sequence restoring the old active plan and deleting the created scheme.
    pub rollback: LiiiraaPowerCfgRollbackPlan,
}

/// Rollback command sequence for an applied Liiiraa power profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiiiraaPowerCfgRollbackPlan {
    /// Active scheme GUID captured before apply.
    pub previous_active_scheme_guid: String,
    /// Optimizer-created scheme GUID to remove after active scheme restore.
    pub created_scheme_guid: String,
    /// Ordered rollback commands.
    pub commands: Vec<StructuredCommandPlan>,
}

/// Builds a typed `powercfg` command plan for a Liiiraa power profile.
pub fn build_liiiraa_powercfg_plan(
    request: &LiiiraaPowerPlanApplyRequest,
) -> Result<LiiiraaPowerCfgApplyPlan, PowerPlanApplyError> {
    validate_power_scheme_guid(&request.previous_active_scheme_guid)?;

    let policy = evaluate_power_plan_policy(PowerPlanPolicyRequest::new(
        request.profile,
        request.device_class,
        request.power_source,
        request.consent,
    ));

    match policy.action {
        PowerPlanPolicyAction::Apply => {}
        PowerPlanPolicyAction::RequireConsent => {
            return Err(PowerPlanApplyError::consent_required(request.profile));
        }
        PowerPlanPolicyAction::Deny => {
            return Err(PowerPlanApplyError::policy_denied(request.profile));
        }
    }

    let scheme_guid = profile_scheme_guid(request.profile);
    let display_name = request.profile.display_name();
    let mut commands = vec![
        StructuredCommandPlan::powercfg_duplicate_scheme(
            profile_base_scheme_guid(request.profile, &request.previous_active_scheme_guid),
            scheme_guid,
        )?,
        StructuredCommandPlan::powercfg_change_scheme_name(scheme_guid, display_name)?,
    ];

    for setting in profile_settings(request.profile, request.device_class) {
        commands.push(setting.to_command(scheme_guid)?);
    }

    commands.push(StructuredCommandPlan::powercfg_activate_scheme(scheme_guid)?);

    let rollback = LiiiraaPowerCfgRollbackPlan {
        previous_active_scheme_guid: request.previous_active_scheme_guid.clone(),
        created_scheme_guid: scheme_guid.to_owned(),
        commands: vec![
            StructuredCommandPlan::powercfg_activate_scheme(
                &request.previous_active_scheme_guid,
            )?,
            StructuredCommandPlan::powercfg_delete_scheme(scheme_guid)?,
        ],
    };

    Ok(LiiiraaPowerCfgApplyPlan {
        profile: request.profile,
        scheme_guid: scheme_guid.to_owned(),
        display_name: display_name.to_owned(),
        policy,
        commands,
        verify_active_scheme: StructuredCommandPlan::powercfg_query_active_scheme()?,
        rollback,
    })
}

/// Parses the active scheme GUID from `powercfg /getactivescheme` output.
#[must_use]
pub fn parse_active_power_scheme_guid(output: &str) -> Option<String> {
    output
        .split(|character: char| !(character.is_ascii_hexdigit() || character == '-'))
        .find(|candidate| is_power_scheme_guid(candidate))
        .map(|candidate| candidate.to_ascii_lowercase())
}

/// Returns true when readback output proves the expected active scheme.
#[must_use]
pub fn active_power_scheme_matches(output: &str, expected_scheme_guid: &str) -> bool {
    parse_active_power_scheme_guid(output).is_some_and(|actual| {
        actual.eq_ignore_ascii_case(expected_scheme_guid)
    })
}

fn profile_base_scheme_guid<'a>(
    profile: LiiiraaPowerPlanProfile,
    previous_active_scheme_guid: &'a str,
) -> &'a str {
    match profile {
        LiiiraaPowerPlanProfile::Balanced => previous_active_scheme_guid,
        LiiiraaPowerPlanProfile::Performance | LiiiraaPowerPlanProfile::Competitive => {
            WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID
        }
    }
}

fn profile_scheme_guid(profile: LiiiraaPowerPlanProfile) -> &'static str {
    match profile {
        LiiiraaPowerPlanProfile::Balanced => LIIIRAA_BALANCED_SCHEME_GUID,
        LiiiraaPowerPlanProfile::Performance => LIIIRAA_PERFORMANCE_SCHEME_GUID,
        LiiiraaPowerPlanProfile::Competitive => LIIIRAA_COMPETITIVE_SCHEME_GUID,
    }
}

fn profile_settings(
    profile: LiiiraaPowerPlanProfile,
    device_class: DevicePowerClass,
) -> Vec<PowerCfgSetting> {
    match profile {
        LiiiraaPowerPlanProfile::Balanced => Vec::new(),
        LiiiraaPowerPlanProfile::Performance => performance_settings(device_class),
        LiiiraaPowerPlanProfile::Competitive => competitive_settings(),
    }
}

fn performance_settings(device_class: DevicePowerClass) -> Vec<PowerCfgSetting> {
    let mut settings = vec![
        PowerCfgSetting::ac(SUB_USB, USB_SELECTIVE_SUSPEND, 0),
        PowerCfgSetting::ac(SUB_PCI_EXPRESS, PCI_EXPRESS_LINK_STATE, 0),
        PowerCfgSetting::ac(SUB_DISK, DISK_IDLE_TIMEOUT, 0),
        PowerCfgSetting::ac(SUB_PROCESSOR, PROCESSOR_MINIMUM_STATE, 5),
        PowerCfgSetting::ac(SUB_PROCESSOR, PROCESSOR_MAXIMUM_STATE, 100),
    ];

    if device_class == DevicePowerClass::Laptop {
        settings.extend([
            PowerCfgSetting::dc(SUB_USB, USB_SELECTIVE_SUSPEND, 1),
            PowerCfgSetting::dc(SUB_PCI_EXPRESS, PCI_EXPRESS_LINK_STATE, 1),
            PowerCfgSetting::dc(SUB_DISK, DISK_IDLE_TIMEOUT, 900),
            PowerCfgSetting::dc(SUB_PROCESSOR, PROCESSOR_MINIMUM_STATE, 5),
            PowerCfgSetting::dc(SUB_PROCESSOR, PROCESSOR_MAXIMUM_STATE, 100),
        ]);
    }

    settings
}

fn competitive_settings() -> Vec<PowerCfgSetting> {
    vec![
        PowerCfgSetting::ac(SUB_USB, USB_SELECTIVE_SUSPEND, 0),
        PowerCfgSetting::ac(SUB_PCI_EXPRESS, PCI_EXPRESS_LINK_STATE, 0),
        PowerCfgSetting::ac(SUB_DISK, DISK_IDLE_TIMEOUT, 0),
        PowerCfgSetting::ac(SUB_PROCESSOR, PROCESSOR_MINIMUM_STATE, 100),
        PowerCfgSetting::ac(SUB_PROCESSOR, PROCESSOR_MAXIMUM_STATE, 100),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PowerCfgSettingPower {
    Ac,
    Dc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PowerCfgSetting {
    power: PowerCfgSettingPower,
    subgroup: &'static str,
    setting: &'static str,
    value: u32,
}

impl PowerCfgSetting {
    const fn ac(subgroup: &'static str, setting: &'static str, value: u32) -> Self {
        Self {
            power: PowerCfgSettingPower::Ac,
            subgroup,
            setting,
            value,
        }
    }

    const fn dc(subgroup: &'static str, setting: &'static str, value: u32) -> Self {
        Self {
            power: PowerCfgSettingPower::Dc,
            subgroup,
            setting,
            value,
        }
    }

    fn to_command(
        self,
        scheme_guid: &str,
    ) -> Result<StructuredCommandPlan, WindowsCommandPlanError> {
        match self.power {
            PowerCfgSettingPower::Ac => StructuredCommandPlan::powercfg_set_ac_value_index(
                scheme_guid,
                self.subgroup,
                self.setting,
                self.value,
            ),
            PowerCfgSettingPower::Dc => StructuredCommandPlan::powercfg_set_dc_value_index(
                scheme_guid,
                self.subgroup,
                self.setting,
                self.value,
            ),
        }
    }
}

fn validate_power_scheme_guid(scheme_guid: &str) -> Result<(), PowerPlanApplyError> {
    if is_power_scheme_guid(scheme_guid) {
        Ok(())
    } else {
        Err(PowerPlanApplyError::invalid_guid(scheme_guid))
    }
}

fn is_power_scheme_guid(value: &str) -> bool {
    let bytes = value.as_bytes();

    bytes.len() == 36
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| match index {
                8 | 13 | 18 | 23 => *byte == b'-',
                _ => byte.is_ascii_hexdigit(),
            })
}

/// Reason a Liiiraa power plan could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPlanApplyErrorReason {
    /// Previous active scheme GUID failed validation.
    InvalidSchemeGuid,
    /// The request needs explicit user consent before command planning.
    ConsentRequired,
    /// The policy denies the request for the current local state.
    PolicyDenied,
    /// A structured `powercfg` command failed validation.
    CommandPlanFailed,
}

impl PowerPlanApplyErrorReason {
    /// Returns the stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidSchemeGuid => "invalid_scheme_guid",
            Self::ConsentRequired => "consent_required",
            Self::PolicyDenied => "policy_denied",
            Self::CommandPlanFailed => "command_plan_failed",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidSchemeGuid => "Power scheme GUID failed validation",
            Self::ConsentRequired => "Power plan request requires explicit consent",
            Self::PolicyDenied => "Power plan request is denied by local policy",
            Self::CommandPlanFailed => "Powercfg command plan failed validation",
        }
    }
}

/// Structured error from Liiiraa power-plan command planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerPlanApplyError {
    reason: PowerPlanApplyErrorReason,
    profile: Option<LiiiraaPowerPlanProfile>,
    detail: Option<String>,
}

impl PowerPlanApplyError {
    fn new(
        reason: PowerPlanApplyErrorReason,
        profile: Option<LiiiraaPowerPlanProfile>,
        detail: Option<String>,
    ) -> Self {
        Self {
            reason,
            profile,
            detail,
        }
    }

    fn invalid_guid(scheme_guid: &str) -> Self {
        Self::new(
            PowerPlanApplyErrorReason::InvalidSchemeGuid,
            None,
            Some(scheme_guid.to_owned()),
        )
    }

    fn consent_required(profile: LiiiraaPowerPlanProfile) -> Self {
        Self::new(
            PowerPlanApplyErrorReason::ConsentRequired,
            Some(profile),
            None,
        )
    }

    fn policy_denied(profile: LiiiraaPowerPlanProfile) -> Self {
        Self::new(PowerPlanApplyErrorReason::PolicyDenied, Some(profile), None)
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> PowerPlanApplyErrorReason {
        self.reason
    }

    /// Returns the profile associated with the failure, when known.
    #[must_use]
    pub const fn profile(&self) -> Option<LiiiraaPowerPlanProfile> {
        self.profile
    }

    /// Returns extra detail, when available.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl From<WindowsCommandPlanError> for PowerPlanApplyError {
    fn from(error: WindowsCommandPlanError) -> Self {
        Self::new(
            PowerPlanApplyErrorReason::CommandPlanFailed,
            None,
            Some(error.to_string()),
        )
    }
}

impl fmt::Display for PowerPlanApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(profile) = self.profile {
            write!(formatter, " ({})", profile.tweak_id())?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, " [{detail}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for PowerPlanApplyError {}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::WindowsArgument;

    #[derive(Debug)]
    struct PowerCfgMock {
        active_scheme_guid: String,
        schemes: BTreeMap<String, String>,
        settings: BTreeMap<String, u32>,
    }

    impl PowerCfgMock {
        fn new(active_scheme_guid: &str) -> Self {
            let mut schemes = BTreeMap::new();
            schemes.insert(
                WINDOWS_BALANCED_SCHEME_GUID.to_owned(),
                "Balanced".to_owned(),
            );
            schemes.insert(
                WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID.to_owned(),
                "High performance".to_owned(),
            );

            Self {
                active_scheme_guid: active_scheme_guid.to_owned(),
                schemes,
                settings: BTreeMap::new(),
            }
        }

        fn run(&mut self, command: &StructuredCommandPlan) -> String {
            let args = command
                .arguments()
                .iter()
                .map(WindowsArgument::as_str)
                .collect::<Vec<_>>();

            match args.as_slice() {
                ["/duplicatescheme", source, destination] => {
                    let source_name = self
                        .schemes
                        .get(*source)
                        .expect("source scheme must exist")
                        .clone();
                    self.schemes.insert((*destination).to_owned(), source_name);
                    String::new()
                }
                ["/changename", scheme, name] => {
                    self.schemes.insert((*scheme).to_owned(), (*name).to_owned());
                    String::new()
                }
                ["/setacvalueindex", scheme, subgroup, setting, value]
                | ["/setdcvalueindex", scheme, subgroup, setting, value] => {
                    self.settings.insert(
                        format!("{}:{}:{}:{}", args[0], scheme, subgroup, setting),
                        value.parse::<u32>().expect("setting value should be numeric"),
                    );
                    String::new()
                }
                ["/setactive", scheme] => {
                    assert!(
                        self.schemes.contains_key(*scheme),
                        "active scheme must exist"
                    );
                    self.active_scheme_guid = (*scheme).to_owned();
                    String::new()
                }
                ["/getactivescheme"] => {
                    let name = self
                        .schemes
                        .get(&self.active_scheme_guid)
                        .expect("active scheme should have a name");
                    format!(
                        "Power Scheme GUID: {} ({name})",
                        self.active_scheme_guid
                    )
                }
                ["/delete", scheme] => {
                    self.schemes.remove(*scheme);
                    String::new()
                }
                other => panic!("unsupported mock powercfg command: {other:?}"),
            }
        }
    }

    #[test]
    fn desktop_performance_plan_applies_verifies_and_rolls_back_with_mock_powercfg() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Performance,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::NotGranted,
            WINDOWS_BALANCED_SCHEME_GUID,
        );
        let plan = build_liiiraa_powercfg_plan(&request)
            .expect("desktop performance plan should build");
        let mut mock = PowerCfgMock::new(WINDOWS_BALANCED_SCHEME_GUID);

        for command in &plan.commands {
            mock.run(command);
        }

        let readback = mock.run(&plan.verify_active_scheme);
        assert!(active_power_scheme_matches(&readback, &plan.scheme_guid));
        assert_eq!(
            mock.settings
                .get(&format!(
                    "/setacvalueindex:{}:{}:{}",
                    plan.scheme_guid, SUB_USB, USB_SELECTIVE_SUSPEND
                ))
                .copied(),
            Some(0)
        );

        for command in &plan.rollback.commands {
            mock.run(command);
        }

        assert_eq!(mock.active_scheme_guid, WINDOWS_BALANCED_SCHEME_GUID);
        assert!(!mock.schemes.contains_key(&plan.scheme_guid));
    }

    #[test]
    fn laptop_performance_requires_consent_before_command_planning() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Performance,
            DevicePowerClass::Laptop,
            PowerSourceState::Battery,
            PowerPlanConsent::NotGranted,
            WINDOWS_BALANCED_SCHEME_GUID,
        );

        let error = build_liiiraa_powercfg_plan(&request)
            .expect_err("laptop performance should require consent");

        assert_eq!(error.reason(), PowerPlanApplyErrorReason::ConsentRequired);
        assert_eq!(error.profile(), Some(LiiiraaPowerPlanProfile::Performance));
    }

    #[test]
    fn laptop_performance_with_consent_keeps_gentler_dc_values() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Performance,
            DevicePowerClass::Laptop,
            PowerSourceState::Battery,
            PowerPlanConsent::Granted,
            WINDOWS_BALANCED_SCHEME_GUID,
        );
        let plan = build_liiiraa_powercfg_plan(&request)
            .expect("consented laptop performance plan should build");

        assert!(plan
            .commands
            .iter()
            .any(|command| command.arguments()[0].as_str() == "/setdcvalueindex"
                && command.arguments()[2].as_str() == SUB_PCI_EXPRESS
                && command.arguments()[3].as_str() == PCI_EXPRESS_LINK_STATE
                && command.arguments()[4].as_str() == "1"));
        assert!(plan
            .policy
            .warnings
            .iter()
            .any(|warning| warning.contains("Battery defaults stay gentler")));
    }

    #[test]
    fn competitive_profile_is_ac_only_and_rollback_capable() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Competitive,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::Granted,
            WINDOWS_BALANCED_SCHEME_GUID,
        );
        let plan = build_liiiraa_powercfg_plan(&request)
            .expect("consented competitive plan should build on AC");

        assert_eq!(plan.scheme_guid, LIIIRAA_COMPETITIVE_SCHEME_GUID);
        assert!(plan
            .commands
            .iter()
            .all(|command| command.arguments()[0].as_str() != "/setdcvalueindex"));
        assert_eq!(
            plan.rollback.previous_active_scheme_guid,
            WINDOWS_BALANCED_SCHEME_GUID
        );
        assert_eq!(
            plan.rollback.created_scheme_guid,
            LIIIRAA_COMPETITIVE_SCHEME_GUID
        );
    }

    #[test]
    fn competitive_profile_is_denied_on_battery() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Competitive,
            DevicePowerClass::Laptop,
            PowerSourceState::Battery,
            PowerPlanConsent::Granted,
            WINDOWS_BALANCED_SCHEME_GUID,
        );

        let error =
            build_liiiraa_powercfg_plan(&request).expect_err("battery competitive is denied");

        assert_eq!(error.reason(), PowerPlanApplyErrorReason::PolicyDenied);
        assert_eq!(error.profile(), Some(LiiiraaPowerPlanProfile::Competitive));
    }

    #[test]
    fn balanced_profile_duplicates_previous_active_scheme() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Balanced,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::NotGranted,
            WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID,
        );
        let plan =
            build_liiiraa_powercfg_plan(&request).expect("balanced plan should build");

        assert_eq!(plan.scheme_guid, LIIIRAA_BALANCED_SCHEME_GUID);
        assert_eq!(plan.commands[0].arguments()[0].as_str(), "/duplicatescheme");
        assert_eq!(
            plan.commands[0].arguments()[1].as_str(),
            WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID
        );
        assert_eq!(
            plan.rollback.previous_active_scheme_guid,
            WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID
        );
    }

    #[test]
    fn parses_powercfg_active_scheme_output() {
        let output = "Power Scheme GUID: 381B4222-F694-41F0-9685-FF5BB260DF2E (Balanced)";

        assert_eq!(
            parse_active_power_scheme_guid(output).as_deref(),
            Some(WINDOWS_BALANCED_SCHEME_GUID)
        );
        assert!(active_power_scheme_matches(
            output,
            WINDOWS_BALANCED_SCHEME_GUID
        ));
    }

    #[test]
    fn rejects_invalid_previous_active_scheme_guid() {
        let request = LiiiraaPowerPlanApplyRequest::new(
            LiiiraaPowerPlanProfile::Balanced,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::NotGranted,
            "not-a-guid",
        );

        let error = build_liiiraa_powercfg_plan(&request)
            .expect_err("previous active scheme must be a GUID");

        assert_eq!(error.reason(), PowerPlanApplyErrorReason::InvalidSchemeGuid);
    }
}
