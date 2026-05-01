//! Policy rules for Liiiraa Boost Windows power plans.

/// Liiiraa-owned Windows power plan profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiiiraaPowerPlanProfile {
    /// Conservative plan cloned from the Windows Balanced scheme.
    Balanced,
    /// Performance-oriented plan with desktop-safe defaults and gentler laptop values.
    Performance,
    /// Explicit opt-in competitive AC profile.
    Competitive,
}

impl LiiiraaPowerPlanProfile {
    /// Returns the stable tweak ID for this power plan profile.
    #[must_use]
    pub const fn tweak_id(self) -> &'static str {
        match self {
            Self::Balanced => "power.plan.liiiraa-balanced",
            Self::Performance => "power.plan.liiiraa-performance",
            Self::Competitive => "power.plan.liiiraa-competitive",
        }
    }

    /// Returns the Windows display name used for the duplicated scheme.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Balanced => "Liiiraa Boost - Balanced",
            Self::Performance => "Liiiraa Boost - Performance",
            Self::Competitive => "Liiiraa Boost - Competitive",
        }
    }

    /// Returns whether this profile is eligible for default desktop optimization.
    #[must_use]
    pub const fn default_enabled_on_desktop(self) -> bool {
        matches!(self, Self::Balanced | Self::Performance)
    }
}

/// Device class used by power-plan policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePowerClass {
    /// A desktop or always-plugged system without a battery profile concern.
    Desktop,
    /// A laptop, handheld PC, or other battery-backed device.
    Laptop,
}

/// Current power-source state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSourceState {
    /// The device is connected to AC power.
    Ac,
    /// The device is currently on battery.
    Battery,
    /// The source is unknown or unavailable from scan data.
    Unknown,
}

/// User consent supplied for a performance tradeoff profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPlanConsent {
    /// The user has not accepted a competitive power, battery, or heat tradeoff.
    NotGranted,
    /// The user explicitly accepted the tradeoff for this request.
    Granted,
}

impl PowerPlanConsent {
    /// Returns true when explicit consent was granted.
    #[must_use]
    pub const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Mode classification selected by power-plan policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPlanMode {
    /// Default-eligible safe behavior.
    Safe,
    /// Explicit opt-in behavior with disclosed power, heat, or comfort tradeoffs.
    Competitive,
}

impl PowerPlanMode {
    /// Returns the stable mode string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Competitive => "competitive",
        }
    }
}

/// Policy outcome for a power-plan request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerPlanPolicyAction {
    /// The request may proceed to backup, apply, verify, and rollback planning.
    Apply,
    /// The request is valid, but needs explicit user consent before apply.
    RequireConsent,
    /// The request is blocked for the current local state.
    Deny,
}

/// Input used to classify a Liiiraa power-plan apply request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerPlanPolicyRequest {
    /// Requested Liiiraa profile.
    pub profile: LiiiraaPowerPlanProfile,
    /// Desktop/laptop classification from scan data.
    pub device_class: DevicePowerClass,
    /// Current AC/battery state from scan data.
    pub power_source: PowerSourceState,
    /// Whether the user accepted the disclosed tradeoff.
    pub consent: PowerPlanConsent,
}

impl PowerPlanPolicyRequest {
    /// Creates a policy request.
    #[must_use]
    pub const fn new(
        profile: LiiiraaPowerPlanProfile,
        device_class: DevicePowerClass,
        power_source: PowerSourceState,
        consent: PowerPlanConsent,
    ) -> Self {
        Self {
            profile,
            device_class,
            power_source,
            consent,
        }
    }
}

/// Decision returned by power-plan policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerPlanPolicyDecision {
    /// Whether apply may proceed, must wait for consent, or is denied.
    pub action: PowerPlanPolicyAction,
    /// Effective safety mode for the requested profile on this device.
    pub mode: PowerPlanMode,
    /// Whether the profile can appear in the default optimize flow.
    pub default_enabled: bool,
    /// Warnings or disclosure copy to show before apply.
    pub warnings: Vec<String>,
}

impl PowerPlanPolicyDecision {
    /// Returns true when command planning may proceed.
    #[must_use]
    pub const fn can_apply(&self) -> bool {
        matches!(self.action, PowerPlanPolicyAction::Apply)
    }
}

/// Evaluates desktop/laptop, battery, and opt-in rules for a power plan profile.
#[must_use]
pub fn evaluate_power_plan_policy(
    request: PowerPlanPolicyRequest,
) -> PowerPlanPolicyDecision {
    match request.profile {
        LiiiraaPowerPlanProfile::Balanced => balanced_decision(request),
        LiiiraaPowerPlanProfile::Performance => performance_decision(request),
        LiiiraaPowerPlanProfile::Competitive => competitive_decision(request),
    }
}

fn balanced_decision(request: PowerPlanPolicyRequest) -> PowerPlanPolicyDecision {
    let mut warnings = Vec::new();

    if request.device_class == DevicePowerClass::Laptop {
        warnings.push(
            "Laptop detected; Balanced keeps battery-friendly defaults in the Liiiraa plan."
                .to_owned(),
        );
    }

    PowerPlanPolicyDecision {
        action: PowerPlanPolicyAction::Apply,
        mode: PowerPlanMode::Safe,
        default_enabled: true,
        warnings,
    }
}

fn performance_decision(request: PowerPlanPolicyRequest) -> PowerPlanPolicyDecision {
    let laptop = request.device_class == DevicePowerClass::Laptop;
    let mut warnings = Vec::new();

    if laptop {
        warnings.push(
            "Laptop Performance mode can increase heat, fan noise, and battery drain."
                .to_owned(),
        );
        warnings.push(
            "Battery defaults stay gentler unless the user explicitly accepts the tradeoff."
                .to_owned(),
        );
    }

    let action = if laptop && !request.consent.is_granted() {
        PowerPlanPolicyAction::RequireConsent
    } else {
        PowerPlanPolicyAction::Apply
    };

    PowerPlanPolicyDecision {
        action,
        mode: if laptop {
            PowerPlanMode::Competitive
        } else {
            PowerPlanMode::Safe
        },
        default_enabled: !laptop,
        warnings,
    }
}

fn competitive_decision(request: PowerPlanPolicyRequest) -> PowerPlanPolicyDecision {
    let mut warnings = vec![
        "Competitive power mode uses aggressive AC-only values and requires explicit consent."
            .to_owned(),
    ];

    if request.device_class == DevicePowerClass::Laptop {
        warnings.push(
            "Laptop detected; expect higher heat, fan noise, and faster battery drain."
                .to_owned(),
        );
    }

    let action = if request.power_source == PowerSourceState::Battery {
        warnings.push("Competitive power mode is blocked while the device is on battery.".to_owned());
        PowerPlanPolicyAction::Deny
    } else if !request.consent.is_granted() {
        PowerPlanPolicyAction::RequireConsent
    } else {
        PowerPlanPolicyAction::Apply
    };

    PowerPlanPolicyDecision {
        action,
        mode: PowerPlanMode::Competitive,
        default_enabled: false,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(
        profile: LiiiraaPowerPlanProfile,
        device_class: DevicePowerClass,
        power_source: PowerSourceState,
        consent: PowerPlanConsent,
    ) -> PowerPlanPolicyRequest {
        PowerPlanPolicyRequest::new(profile, device_class, power_source, consent)
    }

    #[test]
    fn desktop_performance_is_safe_default_apply() {
        let decision = evaluate_power_plan_policy(request(
            LiiiraaPowerPlanProfile::Performance,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::NotGranted,
        ));

        assert_eq!(decision.action, PowerPlanPolicyAction::Apply);
        assert_eq!(decision.mode, PowerPlanMode::Safe);
        assert!(decision.default_enabled);
        assert!(decision.warnings.is_empty());
    }

    #[test]
    fn laptop_performance_requires_consent_and_is_not_default() {
        let decision = evaluate_power_plan_policy(request(
            LiiiraaPowerPlanProfile::Performance,
            DevicePowerClass::Laptop,
            PowerSourceState::Battery,
            PowerPlanConsent::NotGranted,
        ));

        assert_eq!(decision.action, PowerPlanPolicyAction::RequireConsent);
        assert_eq!(decision.mode, PowerPlanMode::Competitive);
        assert!(!decision.default_enabled);
        assert!(decision
            .warnings
            .iter()
            .any(|warning| warning.contains("battery drain")));
    }

    #[test]
    fn competitive_requires_consent_and_blocks_battery() {
        let needs_consent = evaluate_power_plan_policy(request(
            LiiiraaPowerPlanProfile::Competitive,
            DevicePowerClass::Desktop,
            PowerSourceState::Ac,
            PowerPlanConsent::NotGranted,
        ));
        let battery = evaluate_power_plan_policy(request(
            LiiiraaPowerPlanProfile::Competitive,
            DevicePowerClass::Laptop,
            PowerSourceState::Battery,
            PowerPlanConsent::Granted,
        ));

        assert_eq!(needs_consent.action, PowerPlanPolicyAction::RequireConsent);
        assert_eq!(battery.action, PowerPlanPolicyAction::Deny);
        assert!(battery
            .warnings
            .iter()
            .any(|warning| warning.contains("blocked while the device is on battery")));
    }
}
