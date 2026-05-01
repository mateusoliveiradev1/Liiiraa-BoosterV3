//! AMD GPU profile planning and safe Adrenalin guidance.

use gpu::{
    plan_gpu_platform_capabilities, GpuCapabilityState, GpuInventory,
    GpuPlatformCapabilityPlan, GpuPlatformCheckRequest, GpuVendor,
    GpuVendorDetection,
};

/// Tweak ID for AMD Radeon GPU and driver detection.
pub const AMD_DETECT_TWEAK_ID: &str = "amd.detect";
/// Tweak ID for AMD HYPR-RX profile planning.
pub const AMD_HYPR_RX_TWEAK_ID: &str = "amd.hypr-rx.plan";
/// Tweak ID for validating AMD profile feature combinations.
pub const AMD_PROFILE_CONFLICT_VALIDATOR_TWEAK_ID: &str =
    "amd.profile.conflict-validator";
/// Tweak ID for AMD Anti-Lag planning.
pub const AMD_ANTI_LAG_TWEAK_ID: &str = "amd.anti-lag";
/// Tweak ID for AMD Anti-Lag 2 support gating.
pub const AMD_ANTI_LAG2_TWEAK_ID: &str = "amd.anti-lag2.supported-only";
/// Tweak ID for Radeon Boost planning.
pub const AMD_RADEON_BOOST_TWEAK_ID: &str = "amd.radeon-boost";
/// Tweak ID for Radeon Chill planning.
pub const AMD_CHILL_TWEAK_ID: &str = "amd.chill";
/// Tweak ID for Frame Rate Target Control planning.
pub const AMD_FRTC_TWEAK_ID: &str = "amd.frtc.frame-cap";
/// Tweak ID for Enhanced Sync planning.
pub const AMD_ENHANCED_SYNC_TWEAK_ID: &str = "amd.enhanced-sync";
/// Tweak ID for FreeSync and VRR policy planning.
pub const AMD_FREESYNC_TWEAK_ID: &str = "amd.freesync.vrr.profile";
/// Tweak ID for Smart Access Memory detection.
pub const AMD_SAM_DETECT_TWEAK_ID: &str = "amd.sam.detect";
/// Tweak ID for Smart Access Memory official enablement guidance.
pub const AMD_SAM_ENABLE_GUIDE_TWEAK_ID: &str = "amd.sam.enable-guide";
/// Tweak ID for AMD Fluid Motion Frames policy.
pub const AMD_AFMF_TWEAK_ID: &str = "amd.afmf.framegen-policy";
/// Tweak ID for Radeon Image Sharpening and Radeon Super Resolution planning.
pub const AMD_RIS_RSR_TWEAK_ID: &str = "amd.ris-rsr.profile";
/// Tweak ID for AMD clean driver update guidance.
pub const AMD_DRIVER_UPDATE_CLEAN_TWEAK_ID: &str = "amd.driver.update-clean";

/// User-facing AMD profile name owned by Liiiraa.
pub const LIIIRAA_AMD_PROFILE_NAME: &str = "Liiiraa Boost - Radeon Profile";
/// User-facing AMD PUBG competitive profile name owned by Liiiraa.
pub const LIIIRAA_AMD_PUBG_COMPETITIVE_PROFILE_NAME: &str =
    "Liiiraa Boost - PUBG Radeon Competitive";

const COMPETITIVE_LATENCY_WARNING: &str = concat!(
    "Competitive AMD profiles prioritize latency and visibility over generated frames, ",
    "dynamic resolution, or broad HYPR-RX bundles."
);
const AMD_MANUAL_APPLY_NOTE: &str = concat!(
    "Plan only: apply through AMD Software: Adrenalin or a validated supported API, ",
    "with profile backup/readback when mutation support is available."
);
const SAM_OFFICIAL_GUIDANCE: &str = concat!(
    "Use only official SAM/ReBAR BIOS, driver, and chipset guidance; do not flash ",
    "firmware or force hidden profile flags."
);

/// Read-only AMD Radeon GPU and driver detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdDriverDetection {
    /// AMD adapters and driver versions detected from inventory.
    pub vendor: GpuVendorDetection,
    /// AMD Software: Adrenalin profile-management readiness.
    pub adrenalin_state: GpuCapabilityState,
    /// Per-game profile support exposed by the detected driver stack.
    pub per_game_profile_state: GpuCapabilityState,
}

impl AmdDriverDetection {
    /// Builds a conservative AMD detection summary from GPU inventory.
    #[must_use]
    pub fn from_inventory(inventory: &GpuInventory) -> Self {
        let vendor = inventory.vendor_detection(GpuVendor::Amd);
        let capability_state = if vendor.is_available() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::NotApplicable
        };

        Self {
            vendor,
            adrenalin_state: capability_state,
            per_game_profile_state: capability_state,
        }
    }

    /// Returns true when an AMD adapter was detected.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.vendor.is_available()
    }

    /// Returns true when detection has enough driver data for safe profile planning.
    #[must_use]
    pub fn has_driver_version(&self) -> bool {
        matches!(self.vendor.driver_state(), GpuCapabilityState::Ready)
    }
}

/// Builds the AMD view of the shared GPU platform capability plan.
#[must_use]
pub fn plan_amd_platform_capabilities(
    detection: &AmdDriverDetection,
    request: &GpuPlatformCheckRequest,
) -> GpuPlatformCapabilityPlan {
    plan_gpu_platform_capabilities(&detection.vendor, request)
}

/// AMD Radeon features covered by the V1 planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AmdProfileFeature {
    /// AMD HYPR-RX preset.
    HyprRx,
    /// AMD Anti-Lag.
    AntiLag,
    /// AMD Anti-Lag 2, where the game integration supports it.
    AntiLag2,
    /// Radeon Boost dynamic resolution.
    RadeonBoost,
    /// Radeon Chill frame pacing and power limiting.
    Chill,
    /// Frame Rate Target Control.
    Frtc,
    /// Enhanced Sync.
    EnhancedSync,
    /// FreeSync or compatible VRR.
    FreeSync,
    /// AMD Fluid Motion Frames.
    Afmf,
    /// Radeon Image Sharpening.
    Ris,
    /// Radeon Super Resolution.
    Rsr,
    /// Smart Access Memory or Resizable BAR.
    SamReBar,
}

impl AmdProfileFeature {
    /// Returns the tweak ID associated with this feature.
    #[must_use]
    pub const fn tweak_id(self) -> &'static str {
        match self {
            Self::HyprRx => AMD_HYPR_RX_TWEAK_ID,
            Self::AntiLag => AMD_ANTI_LAG_TWEAK_ID,
            Self::AntiLag2 => AMD_ANTI_LAG2_TWEAK_ID,
            Self::RadeonBoost => AMD_RADEON_BOOST_TWEAK_ID,
            Self::Chill => AMD_CHILL_TWEAK_ID,
            Self::Frtc => AMD_FRTC_TWEAK_ID,
            Self::EnhancedSync => AMD_ENHANCED_SYNC_TWEAK_ID,
            Self::FreeSync => AMD_FREESYNC_TWEAK_ID,
            Self::Afmf => AMD_AFMF_TWEAK_ID,
            Self::Ris | Self::Rsr => AMD_RIS_RSR_TWEAK_ID,
            Self::SamReBar => AMD_SAM_DETECT_TWEAK_ID,
        }
    }

    /// Returns a user-facing feature label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::HyprRx => "HYPR-RX",
            Self::AntiLag => "Anti-Lag",
            Self::AntiLag2 => "Anti-Lag 2",
            Self::RadeonBoost => "Radeon Boost",
            Self::Chill => "Radeon Chill",
            Self::Frtc => "Frame Rate Target Control",
            Self::EnhancedSync => "Enhanced Sync",
            Self::FreeSync => "FreeSync/VRR",
            Self::Afmf => "AMD Fluid Motion Frames",
            Self::Ris => "Radeon Image Sharpening",
            Self::Rsr => "Radeon Super Resolution",
            Self::SamReBar => "Smart Access Memory/ReBAR",
        }
    }
}

/// Optimization intent used to resolve feature tradeoffs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmdProfileIntent {
    /// Latency-first competitive profile.
    CompetitiveLatency,
    /// PUBG-specific competitive profile with anti-cheat-safe guidance.
    PubgCompetitive,
    /// General balanced game profile.
    Balanced,
    /// Visual or single-player profile where frame generation/upscaling can be acceptable.
    VisualQuality,
    /// Thermal or power-limited profile.
    ThermalPower,
}

impl AmdProfileIntent {
    /// Returns true when latency and visibility must win over image generation.
    #[must_use]
    pub const fn is_competitive(self) -> bool {
        matches!(self, Self::CompetitiveLatency | Self::PubgCompetitive)
    }

    /// Returns true when the target is PUBG.
    #[must_use]
    pub const fn is_pubg(self) -> bool {
        matches!(self, Self::PubgCompetitive)
    }
}

/// Decision selected for one AMD profile feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmdProfileDecision {
    /// Plan to enable or recommend enabling the feature.
    Enable,
    /// Plan to keep the feature disabled.
    Disable,
    /// Present as an explicit user choice.
    UserChoice,
    /// Recommend a manual official setup path, with no driver mutation.
    ManualOnly,
    /// Capability is not supported or not applicable.
    NotSupported,
    /// Deny unsafe or unsupported requested behavior.
    Deny,
}

/// Severity for a feature-combination validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AmdConflictSeverity {
    /// Informational note that should be shown with the plan.
    Info,
    /// Combination needs user confirmation or benchmark framing.
    Warning,
    /// Combination must not be applied by the optimizer.
    Blocker,
}

/// One feature-combination validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdProfileConflict {
    /// Severity of the finding.
    pub severity: AmdConflictSeverity,
    /// Features involved in the finding.
    pub features: Vec<AmdProfileFeature>,
    /// Human-readable validation message.
    pub message: String,
}

impl AmdProfileConflict {
    fn new(
        severity: AmdConflictSeverity,
        features: Vec<AmdProfileFeature>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            features,
            message: message.into(),
        }
    }
}

/// Capability state for AMD profile features.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdFeatureSupport {
    /// HYPR-RX availability.
    pub hypr_rx: GpuCapabilityState,
    /// Anti-Lag availability.
    pub anti_lag: GpuCapabilityState,
    /// Anti-Lag 2 game-integrated support.
    pub anti_lag2: GpuCapabilityState,
    /// Radeon Boost availability.
    pub radeon_boost: GpuCapabilityState,
    /// Radeon Chill availability.
    pub chill: GpuCapabilityState,
    /// Frame Rate Target Control availability.
    pub frtc: GpuCapabilityState,
    /// Enhanced Sync availability.
    pub enhanced_sync: GpuCapabilityState,
    /// FreeSync/VRR active display support.
    pub freesync: GpuCapabilityState,
    /// AMD Fluid Motion Frames availability.
    pub afmf: GpuCapabilityState,
    /// Radeon Image Sharpening availability.
    pub ris: GpuCapabilityState,
    /// Radeon Super Resolution availability.
    pub rsr: GpuCapabilityState,
    /// Smart Access Memory / Resizable BAR state.
    pub sam_rebar: GpuCapabilityState,
    /// AMD per-game profile support.
    pub per_game_profile: GpuCapabilityState,
}

impl AmdFeatureSupport {
    /// Returns a detected-system default with AMD-specific capabilities unknown.
    #[must_use]
    pub fn unknown_for_detected_amd() -> Self {
        Self {
            hypr_rx: GpuCapabilityState::Unknown,
            anti_lag: GpuCapabilityState::Unknown,
            anti_lag2: GpuCapabilityState::Unknown,
            radeon_boost: GpuCapabilityState::Unknown,
            chill: GpuCapabilityState::Unknown,
            frtc: GpuCapabilityState::Unknown,
            enhanced_sync: GpuCapabilityState::Unknown,
            freesync: GpuCapabilityState::Unknown,
            afmf: GpuCapabilityState::Unknown,
            ris: GpuCapabilityState::Unknown,
            rsr: GpuCapabilityState::Unknown,
            sam_rebar: GpuCapabilityState::Unknown,
            per_game_profile: GpuCapabilityState::Unknown,
        }
    }

    /// Returns a default where AMD profile features do not apply.
    #[must_use]
    pub fn not_applicable() -> Self {
        Self {
            hypr_rx: GpuCapabilityState::NotApplicable,
            anti_lag: GpuCapabilityState::NotApplicable,
            anti_lag2: GpuCapabilityState::NotApplicable,
            radeon_boost: GpuCapabilityState::NotApplicable,
            chill: GpuCapabilityState::NotApplicable,
            frtc: GpuCapabilityState::NotApplicable,
            enhanced_sync: GpuCapabilityState::NotApplicable,
            freesync: GpuCapabilityState::NotApplicable,
            afmf: GpuCapabilityState::NotApplicable,
            ris: GpuCapabilityState::NotApplicable,
            rsr: GpuCapabilityState::NotApplicable,
            sam_rebar: GpuCapabilityState::NotApplicable,
            per_game_profile: GpuCapabilityState::NotApplicable,
        }
    }

    /// Builds conservative support defaults from read-only AMD detection.
    #[must_use]
    pub fn from_detection(detection: &AmdDriverDetection) -> Self {
        if detection.is_available() {
            let mut support = Self::unknown_for_detected_amd();
            support.per_game_profile = detection.per_game_profile_state;
            support
        } else {
            Self::not_applicable()
        }
    }

    fn state_for(&self, feature: AmdProfileFeature) -> GpuCapabilityState {
        match feature {
            AmdProfileFeature::HyprRx => self.hypr_rx,
            AmdProfileFeature::AntiLag => self.anti_lag,
            AmdProfileFeature::AntiLag2 => self.anti_lag2,
            AmdProfileFeature::RadeonBoost => self.radeon_boost,
            AmdProfileFeature::Chill => self.chill,
            AmdProfileFeature::Frtc => self.frtc,
            AmdProfileFeature::EnhancedSync => self.enhanced_sync,
            AmdProfileFeature::FreeSync => self.freesync,
            AmdProfileFeature::Afmf => self.afmf,
            AmdProfileFeature::Ris => self.ris,
            AmdProfileFeature::Rsr => self.rsr,
            AmdProfileFeature::SamReBar => self.sam_rebar,
        }
    }
}

/// Inputs used to build an AMD Radeon profile plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdProfilePlannerRequest {
    /// Profile intent used for feature policy.
    pub intent: AmdProfileIntent,
    /// Game or app name shown in the profile plan.
    pub target_name: String,
    /// Executable associated with the profile, when known.
    pub executable_name: Option<String>,
    /// Primary display refresh rate when known.
    pub display_refresh_hz: Option<u16>,
    /// Whether a GPU-bound condition was detected.
    pub gpu_limited: GpuCapabilityState,
    /// Whether thermal or power limiting was detected or requested.
    pub thermal_or_power_limited: bool,
    /// Feature support gathered from AMD Software, display, and platform probes.
    pub support: AmdFeatureSupport,
    /// Explicit consent for visual-quality tradeoffs such as sharpening/upscaling.
    pub visual_tradeoff_consent: bool,
    /// Explicit consent for frame generation.
    pub frame_generation_consent: bool,
    /// Explicit consent for dynamic resolution features.
    pub dynamic_resolution_consent: bool,
    /// Explicit consent to evaluate Enhanced Sync as a tearing/latency alternative.
    pub enhanced_sync_consent: bool,
}

impl AmdProfilePlannerRequest {
    /// Creates a conservative AMD profile planner request.
    #[must_use]
    pub fn new(
        intent: AmdProfileIntent,
        target_name: impl Into<String>,
        support: AmdFeatureSupport,
    ) -> Self {
        Self {
            intent,
            target_name: target_name.into(),
            executable_name: None,
            display_refresh_hz: None,
            gpu_limited: GpuCapabilityState::Unknown,
            thermal_or_power_limited: false,
            support,
            visual_tradeoff_consent: false,
            frame_generation_consent: false,
            dynamic_resolution_consent: false,
            enhanced_sync_consent: false,
        }
    }

    /// Adds the executable associated with this profile.
    #[must_use]
    pub fn with_executable(mut self, executable_name: impl Into<String>) -> Self {
        self.executable_name = Some(executable_name.into());
        self
    }

    /// Adds display state used for FreeSync/FRTC policy.
    #[must_use]
    pub const fn with_display_refresh(mut self, display_refresh_hz: u16) -> Self {
        self.display_refresh_hz = Some(display_refresh_hz);
        self
    }

    /// Adds whether the workload is currently GPU-limited.
    #[must_use]
    pub const fn with_gpu_limited(mut self, gpu_limited: GpuCapabilityState) -> Self {
        self.gpu_limited = gpu_limited;
        self
    }

    /// Adds thermal or power limiting evidence.
    #[must_use]
    pub const fn with_thermal_or_power_limited(mut self, thermal_or_power_limited: bool) -> Self {
        self.thermal_or_power_limited = thermal_or_power_limited;
        self
    }

    /// Adds user consent flags for non-default AMD tradeoffs.
    #[must_use]
    pub const fn with_tradeoff_consent(
        mut self,
        visual_tradeoff: bool,
        frame_generation: bool,
        dynamic_resolution: bool,
        enhanced_sync: bool,
    ) -> Self {
        self.visual_tradeoff_consent = visual_tradeoff;
        self.frame_generation_consent = frame_generation;
        self.dynamic_resolution_consent = dynamic_resolution;
        self.enhanced_sync_consent = enhanced_sync;
        self
    }
}

/// One AMD feature recommendation in a dry-run profile plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdProfileRecommendation {
    /// Feature being planned.
    pub feature: AmdProfileFeature,
    /// Stable tweak ID associated with the feature.
    pub tweak_id: &'static str,
    /// Capability state that informed the decision.
    pub capability: GpuCapabilityState,
    /// Planned decision.
    pub decision: AmdProfileDecision,
    /// Desired value or setting summary, when applicable.
    pub desired_state: Option<String>,
    /// Whether explicit user consent is required before apply.
    pub requires_user_consent: bool,
    /// User-visible notes explaining tradeoffs and guardrails.
    pub notes: Vec<String>,
}

impl AmdProfileRecommendation {
    fn new(
        feature: AmdProfileFeature,
        capability: GpuCapabilityState,
        decision: AmdProfileDecision,
        desired_state: Option<String>,
        requires_user_consent: bool,
        notes: Vec<String>,
    ) -> Self {
        Self {
            feature,
            tweak_id: feature.tweak_id(),
            capability,
            decision,
            desired_state,
            requires_user_consent,
            notes,
        }
    }

    fn unsupported(feature: AmdProfileFeature, capability: GpuCapabilityState) -> Self {
        Self::new(
            feature,
            capability,
            AmdProfileDecision::NotSupported,
            None,
            false,
            vec![format!(
                "{} is {capability:?}; keep it unavailable until AMD capability detection proves support.",
                feature.display_name()
            )],
        )
    }
}

/// Dry-run AMD Radeon profile plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdProfilePlan {
    /// Profile name owned by Liiiraa.
    pub profile_name: String,
    /// Target game or app.
    pub target_name: String,
    /// Target executable, when known.
    pub executable_name: Option<String>,
    /// Intent used to resolve tradeoffs.
    pub intent: AmdProfileIntent,
    /// Feature recommendations.
    pub recommendations: Vec<AmdProfileRecommendation>,
    /// Conflict validation findings.
    pub conflicts: Vec<AmdProfileConflict>,
    /// Manual actions or explanations shown alongside the plan.
    pub manual_actions: Vec<String>,
}

impl AmdProfilePlan {
    /// Returns the recommendation for one feature.
    #[must_use]
    pub fn recommendation(
        &self,
        feature: AmdProfileFeature,
    ) -> Option<&AmdProfileRecommendation> {
        self.recommendations
            .iter()
            .find(|recommendation| recommendation.feature == feature)
    }

    /// Returns true when conflict validation found a blocker.
    #[must_use]
    pub fn has_blocking_conflicts(&self) -> bool {
        self.conflicts
            .iter()
            .any(|conflict| conflict.severity == AmdConflictSeverity::Blocker)
    }
}

/// Feature selection used by the conflict validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdProfileFeatureSelection {
    /// HYPR-RX selected.
    pub hypr_rx: bool,
    /// Anti-Lag selected.
    pub anti_lag: bool,
    /// Anti-Lag 2 selected.
    pub anti_lag2: bool,
    /// Radeon Boost selected.
    pub radeon_boost: bool,
    /// Radeon Chill selected.
    pub chill: bool,
    /// FRTC selected.
    pub frtc: bool,
    /// Enhanced Sync selected.
    pub enhanced_sync: bool,
    /// FreeSync/VRR selected.
    pub freesync: bool,
    /// AFMF selected.
    pub afmf: bool,
    /// RIS selected.
    pub ris: bool,
    /// RSR selected.
    pub rsr: bool,
}

impl AmdProfileFeatureSelection {
    /// Creates an empty feature selection.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            hypr_rx: false,
            anti_lag: false,
            anti_lag2: false,
            radeon_boost: false,
            chill: false,
            frtc: false,
            enhanced_sync: false,
            freesync: false,
            afmf: false,
            ris: false,
            rsr: false,
        }
    }
}

/// Context for AMD feature-combination validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AmdProfileValidationContext {
    /// Profile intent.
    pub intent: AmdProfileIntent,
    /// Anti-Lag 2 game-integrated support state.
    pub anti_lag2_support: GpuCapabilityState,
    /// Explicit consent for visual-quality tradeoffs.
    pub visual_tradeoff_consent: bool,
    /// Explicit consent for frame generation.
    pub frame_generation_consent: bool,
    /// Explicit consent for dynamic resolution.
    pub dynamic_resolution_consent: bool,
}

impl AmdProfileValidationContext {
    fn from_request(request: &AmdProfilePlannerRequest) -> Self {
        Self {
            intent: request.intent,
            anti_lag2_support: request.support.anti_lag2,
            visual_tradeoff_consent: request.visual_tradeoff_consent,
            frame_generation_consent: request.frame_generation_consent,
            dynamic_resolution_consent: request.dynamic_resolution_consent,
        }
    }
}

/// Builds a dry-run AMD Radeon profile plan from capability state.
#[must_use]
pub fn plan_amd_profile(request: &AmdProfilePlannerRequest) -> AmdProfilePlan {
    let mut recommendations = vec![
        hypr_rx_recommendation(request),
        anti_lag_recommendation(request),
        anti_lag2_recommendation(request),
        radeon_boost_recommendation(request),
        chill_recommendation(request),
        frtc_recommendation(request),
        enhanced_sync_recommendation(request),
        freesync_recommendation(request),
        afmf_recommendation(request),
        ris_recommendation(request),
        rsr_recommendation(request),
        sam_rebar_recommendation(request),
    ];

    recommendations.sort_by_key(|recommendation| recommendation.feature);

    let selection = selection_from_recommendations(&recommendations);
    let mut conflicts = validate_amd_profile_feature_selection(
        &selection,
        AmdProfileValidationContext::from_request(request),
    );

    if request.support.per_game_profile.needs_attention() {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![AmdProfileFeature::HyprRx],
            "AMD per-game profile support is not confirmed; keep this as guidance until Adrenalin profile APIs are available.",
        ));
    }

    let mut manual_actions = vec![AMD_MANUAL_APPLY_NOTE.to_owned()];
    if request.intent.is_competitive() {
        manual_actions.push(COMPETITIVE_LATENCY_WARNING.to_owned());
    }
    manual_actions.push(SAM_OFFICIAL_GUIDANCE.to_owned());

    AmdProfilePlan {
        profile_name: if request.intent.is_pubg() {
            LIIIRAA_AMD_PUBG_COMPETITIVE_PROFILE_NAME.to_owned()
        } else {
            LIIIRAA_AMD_PROFILE_NAME.to_owned()
        },
        target_name: request.target_name.clone(),
        executable_name: request.executable_name.clone(),
        intent: request.intent,
        recommendations,
        conflicts,
        manual_actions,
    }
}

/// Validates AMD feature combinations before any profile apply path can use them.
#[must_use]
pub fn validate_amd_profile_feature_selection(
    selection: &AmdProfileFeatureSelection,
    context: AmdProfileValidationContext,
) -> Vec<AmdProfileConflict> {
    let mut conflicts = Vec::new();

    if selection.anti_lag && selection.anti_lag2 {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![AmdProfileFeature::AntiLag, AmdProfileFeature::AntiLag2],
            "Do not stack driver Anti-Lag with game-integrated Anti-Lag 2 without a benchmarked per-title policy.",
        ));
    }

    if selection.anti_lag2 && context.anti_lag2_support != GpuCapabilityState::Ready {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Blocker,
            vec![AmdProfileFeature::AntiLag2],
            "Anti-Lag 2 must be game-integrated and supported; injection-like or fake support is denied.",
        ));
    }

    if selection.afmf
        && context.intent.is_competitive()
        && !context.frame_generation_consent
    {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Blocker,
            vec![AmdProfileFeature::Afmf],
            "AFMF/frame generation is not a default competitive latency setting.",
        ));
    }

    if selection.radeon_boost
        && context.intent.is_competitive()
        && !context.dynamic_resolution_consent
    {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Blocker,
            vec![AmdProfileFeature::RadeonBoost],
            "Radeon Boost changes resolution dynamically and is not default for visibility-critical competitive play.",
        ));
    }

    if selection.rsr
        && context.intent.is_competitive()
        && !context.visual_tradeoff_consent
    {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![AmdProfileFeature::Rsr],
            "RSR can affect image clarity; require explicit visual-tradeoff consent for competitive profiles.",
        ));
    }

    if selection.chill && selection.frtc {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![AmdProfileFeature::Chill, AmdProfileFeature::Frtc],
            "Use either Chill or FRTC as the active limiter unless a benchmark proves the combination.",
        ));
    }

    if selection.enhanced_sync && selection.freesync && selection.frtc {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![
                AmdProfileFeature::EnhancedSync,
                AmdProfileFeature::FreeSync,
                AmdProfileFeature::Frtc,
            ],
            "Enhanced Sync plus FreeSync plus a frame cap must be benchmarked for stutter before use.",
        ));
    }

    if selection.hypr_rx
        && (selection.radeon_boost
            || selection.chill
            || selection.afmf
            || selection.rsr
            || selection.frtc)
    {
        conflicts.push(AmdProfileConflict::new(
            AmdConflictSeverity::Warning,
            vec![AmdProfileFeature::HyprRx],
            "HYPR-RX is a bundle; avoid silently stacking it with manual overrides without confirming the resulting Adrenalin toggles.",
        ));
    }

    conflicts
}

/// Computes a conservative cap below refresh rate for VRR latency consistency.
#[must_use]
pub const fn amd_vrr_frame_cap(refresh_rate_hz: u16) -> u16 {
    if refresh_rate_hz <= 60 {
        refresh_rate_hz.saturating_sub(2)
    } else {
        refresh_rate_hz.saturating_sub(3)
    }
}

fn hypr_rx_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::HyprRx;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.intent.is_competitive() {
        return AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::UserChoice,
            Some("Manual review".to_owned()),
            true,
            vec![
                "HYPR-RX may include AFMF, Boost, Anti-Lag, or RSR depending on driver version."
                    .to_owned(),
                "For competitive profiles, review the exact toggles instead of enabling the bundle blindly."
                    .to_owned(),
            ],
        );
    }

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::Enable,
        Some("Recommend HYPR-RX when the user accepts bundled tradeoffs".to_owned()),
        true,
        vec!["Use AMD's official preset only when Adrenalin reports support.".to_owned()],
    )
}

fn anti_lag_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::AntiLag;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    let should_enable = request.intent.is_competitive()
        && matches!(
            request.gpu_limited,
            GpuCapabilityState::Ready | GpuCapabilityState::Unknown
        );

    if should_enable {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Enable,
            Some("On".to_owned()),
            true,
            vec![
                "Use Anti-Lag for GPU-limited competitive paths where AMD reports support."
                    .to_owned(),
                "Benchmark frametime consistency before keeping it enabled globally.".to_owned(),
            ],
        )
    } else {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::UserChoice,
            Some("Benchmark first".to_owned()),
            true,
            vec![
                "Anti-Lag is most useful when GPU-bound; keep it optional without bottleneck evidence."
                    .to_owned(),
            ],
        )
    }
}

fn anti_lag2_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::AntiLag2;
    let capability = request.support.state_for(feature);

    match capability {
        GpuCapabilityState::Ready => AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::ManualOnly,
            Some("Use in-game supported integration".to_owned()),
            false,
            vec![
                "Anti-Lag 2 is game-integrated/support-dependent; prefer the in-game path where available."
                    .to_owned(),
                "Never inject, shim, or fake Anti-Lag 2 support.".to_owned(),
            ],
        ),
        GpuCapabilityState::Missing | GpuCapabilityState::Unknown => {
            AmdProfileRecommendation::new(
                feature,
                capability,
                AmdProfileDecision::Disable,
                Some("Unavailable for this game".to_owned()),
                false,
                vec![
                    "Anti-Lag 2 is not confirmed for this target; keep it unavailable and do not emulate support."
                        .to_owned(),
                ],
            )
        }
        GpuCapabilityState::NotApplicable => {
            AmdProfileRecommendation::unsupported(feature, capability)
        }
    }
}

fn radeon_boost_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::RadeonBoost;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.intent.is_competitive() && !request.dynamic_resolution_consent {
        return AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Disable,
            Some("Off".to_owned()),
            false,
            vec![
                "Dynamic resolution is not default for visibility-critical competitive profiles."
                    .to_owned(),
            ],
        );
    }

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::UserChoice,
        Some("Optional with dynamic-resolution consent".to_owned()),
        true,
        vec!["Explain visibility and image-stability tradeoffs before enabling Radeon Boost.".to_owned()],
    )
}

fn chill_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::Chill;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.thermal_or_power_limited || request.intent == AmdProfileIntent::ThermalPower {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Enable,
            Some("Thermal/power cap range".to_owned()),
            true,
            vec!["Use Chill for thermal or power goals, not as a max-FPS competitive default.".to_owned()],
        )
    } else {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Disable,
            Some("Off".to_owned()),
            false,
            vec!["Keep Chill off for max-FPS planning unless thermal or power limits are the goal.".to_owned()],
        )
    }
}

fn frtc_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::Frtc;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    let cap = request.display_refresh_hz.map(amd_vrr_frame_cap);
    let desired_state = cap
        .map(|cap| format!("{cap} FPS"))
        .unwrap_or_else(|| "Benchmark-derived cap".to_owned());

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::Enable,
        Some(desired_state),
        true,
        vec![
            "Use one frame limiter for VRR, thermals, or latency consistency; avoid stacking multiple caps blindly."
                .to_owned(),
        ],
    )
}

fn enhanced_sync_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::EnhancedSync;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.enhanced_sync_consent {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::UserChoice,
            Some("Optional".to_owned()),
            true,
            vec![
                "Enhanced Sync can reduce tearing but may introduce stutter; benchmark before keeping it."
                    .to_owned(),
            ],
        )
    } else {
        AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Disable,
            Some("Off".to_owned()),
            false,
            vec!["Keep Enhanced Sync optional until the user accepts the tearing/stutter tradeoff.".to_owned()],
        )
    }
}

fn freesync_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::FreeSync;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    let desired_state = request.display_refresh_hz.map_or_else(
        || "On with benchmarked cap".to_owned(),
        |refresh| format!("On with cap below {refresh} Hz"),
    );

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::Enable,
        Some(desired_state),
        false,
        vec!["Use FreeSync/VRR only when the display path reports support and the cap policy is clear.".to_owned()],
    )
}

fn afmf_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::Afmf;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.intent.is_competitive() && !request.frame_generation_consent {
        return AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Disable,
            Some("Off".to_owned()),
            false,
            vec!["AFMF/frame generation is optional for visual profiles and not a competitive latency default.".to_owned()],
        );
    }

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::UserChoice,
        Some("Optional with frame-generation consent".to_owned()),
        true,
        vec![
            "Separate native frames from generated frames in benchmark summaries where tooling allows."
                .to_owned(),
        ],
    )
}

fn ris_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::Ris;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    AmdProfileRecommendation::new(
        feature,
        capability,
        if request.visual_tradeoff_consent {
            AmdProfileDecision::UserChoice
        } else {
            AmdProfileDecision::Disable
        },
        Some("Optional sharpening".to_owned()),
        !request.visual_tradeoff_consent,
        vec!["RIS is a visual preference/tradeoff, not a guaranteed performance tweak.".to_owned()],
    )
}

fn rsr_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::Rsr;
    let capability = request.support.state_for(feature);
    if capability != GpuCapabilityState::Ready {
        return AmdProfileRecommendation::unsupported(feature, capability);
    }

    if request.intent.is_competitive() && !request.visual_tradeoff_consent {
        return AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::Disable,
            Some("Off".to_owned()),
            false,
            vec!["RSR can change image clarity and should stay off for competitive profiles without consent.".to_owned()],
        );
    }

    AmdProfileRecommendation::new(
        feature,
        capability,
        AmdProfileDecision::UserChoice,
        Some("Optional upscaling".to_owned()),
        true,
        vec!["Use RSR only after the user accepts resolution and sharpening tradeoffs.".to_owned()],
    )
}

fn sam_rebar_recommendation(request: &AmdProfilePlannerRequest) -> AmdProfileRecommendation {
    let feature = AmdProfileFeature::SamReBar;
    let capability = request.support.state_for(feature);

    match capability {
        GpuCapabilityState::Ready => AmdProfileRecommendation::new(
            feature,
            capability,
            AmdProfileDecision::ManualOnly,
            Some("Detected enabled".to_owned()),
            false,
            vec![SAM_OFFICIAL_GUIDANCE.to_owned()],
        ),
        GpuCapabilityState::Missing | GpuCapabilityState::Unknown => {
            AmdProfileRecommendation::new(
                feature,
                capability,
                AmdProfileDecision::ManualOnly,
                Some("Recommend official enablement check".to_owned()),
                false,
                vec![
                    "SAM/ReBAR is not confirmed; recommend official motherboard BIOS, chipset, and Radeon driver checks."
                        .to_owned(),
                    SAM_OFFICIAL_GUIDANCE.to_owned(),
                ],
            )
        }
        GpuCapabilityState::NotApplicable => {
            AmdProfileRecommendation::unsupported(feature, capability)
        }
    }
}

fn selection_from_recommendations(
    recommendations: &[AmdProfileRecommendation],
) -> AmdProfileFeatureSelection {
    let mut selection = AmdProfileFeatureSelection::empty();

    for recommendation in recommendations {
        let selected = matches!(recommendation.decision, AmdProfileDecision::Enable);

        match recommendation.feature {
            AmdProfileFeature::HyprRx => selection.hypr_rx = selected,
            AmdProfileFeature::AntiLag => selection.anti_lag = selected,
            AmdProfileFeature::AntiLag2 => selection.anti_lag2 = selected,
            AmdProfileFeature::RadeonBoost => selection.radeon_boost = selected,
            AmdProfileFeature::Chill => selection.chill = selected,
            AmdProfileFeature::Frtc => selection.frtc = selected,
            AmdProfileFeature::EnhancedSync => selection.enhanced_sync = selected,
            AmdProfileFeature::FreeSync => selection.freesync = selected,
            AmdProfileFeature::Afmf => selection.afmf = selected,
            AmdProfileFeature::Ris => selection.ris = selected,
            AmdProfileFeature::Rsr => selection.rsr = selected,
            AmdProfileFeature::SamReBar => {}
        }
    }

    selection
}

/// Static metadata describing this workspace crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrateInfo {
    /// Cargo package name.
    pub name: &'static str,
    /// Design-level responsibility owned by the crate.
    pub responsibility: &'static str,
    /// Whether the crate eventually needs live Windows state for full coverage.
    pub requires_live_windows: bool,
}

/// AMD crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "amd",
    responsibility: "plan AMD Adrenalin, SAM, Anti-Lag, HYPR-RX, and related Radeon guidance",
    requires_live_windows: true,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpu::GpuAdapter;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "amd");
        assert!(info.responsibility.contains("Radeon"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn detects_amd_adapters_and_driver_versions() {
        let inventory = GpuInventory::new(vec![
            GpuAdapter::from_scan(
                "AMD Radeon RX 7800 XT",
                Some("31.0.24002.92"),
                None,
                None,
                Some("PCI\\VEN_1002&DEV_747E"),
            ),
            GpuAdapter::from_scan(
                "NVIDIA GeForce RTX 4070",
                Some("32.0.15.6094"),
                None,
                None,
                Some("PCI\\VEN_10DE&DEV_2786"),
            ),
        ]);

        let detection = AmdDriverDetection::from_inventory(&inventory);

        assert!(detection.is_available());
        assert_eq!(detection.vendor.adapters.len(), 1);
        assert_eq!(detection.vendor.driver_versions, vec!["31.0.24002.92"]);
        assert_eq!(detection.adrenalin_state, GpuCapabilityState::Unknown);
        assert_eq!(
            detection.per_game_profile_state,
            GpuCapabilityState::Unknown
        );
    }

    #[test]
    fn amd_platform_plan_uses_sam_and_afmf_policy_labels() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "AMD Radeon RX 7800 XT",
            Some("31.0.24002.92"),
            None,
            None,
            Some("PCI\\VEN_1002&DEV_747E"),
        )]);
        let detection = AmdDriverDetection::from_inventory(&inventory);
        let request = gpu::GpuPlatformCheckRequest::new(gpu::GpuPlatformIntent::VisualQuality)
            .with_driver_age_days(20)
            .with_display(gpu::GpuDisplayPipelineState::new(
                Some(165),
                Some(165),
                GpuCapabilityState::Ready,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Missing)
            .with_frame_generation_state(GpuCapabilityState::Ready)
            .with_shader_cache(gpu::GpuShaderCacheState::enabled(None));

        let plan = plan_amd_platform_capabilities(&detection, &request);

        assert_eq!(plan.vendor, GpuVendor::Amd);
        assert_eq!(plan.rebar_sam.label, "Smart Access Memory/ReBAR");
        assert_eq!(plan.rebar_sam.decision, gpu::GpuPlatformDecision::Recommend);
        assert_eq!(plan.frame_generation.label, "AMD Fluid Motion Frames");
        assert_eq!(
            plan.frame_generation.policy,
            gpu::GpuFrameGenerationPolicy::OptionalWithConsent
        );
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == gpu::GpuPlatformCheck::RebarSam
        }));
    }

    #[test]
    fn competitive_pubg_plan_prioritizes_latency_over_generated_frames() {
        let request = AmdProfilePlannerRequest::new(
            AmdProfileIntent::PubgCompetitive,
            "PUBG",
            ready_support(),
        )
        .with_executable("TslGame.exe")
        .with_display_refresh(240)
        .with_gpu_limited(GpuCapabilityState::Ready);

        let plan = plan_amd_profile(&request);

        assert_eq!(
            plan.profile_name,
            LIIIRAA_AMD_PUBG_COMPETITIVE_PROFILE_NAME
        );
        assert_eq!(plan.executable_name.as_deref(), Some("TslGame.exe"));
        assert!(!plan.has_blocking_conflicts());
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::AntiLag).decision,
            AmdProfileDecision::Enable
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::AntiLag2).decision,
            AmdProfileDecision::ManualOnly
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::Frtc).desired_state,
            Some("237 FPS".to_owned())
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::FreeSync).decision,
            AmdProfileDecision::Enable
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::Afmf).decision,
            AmdProfileDecision::Disable
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::RadeonBoost).decision,
            AmdProfileDecision::Disable
        );
        assert!(plan
            .manual_actions
            .iter()
            .any(|action| action.contains("Competitive AMD profiles")));
    }

    #[test]
    fn visual_profile_allows_afmf_and_image_scaling_with_consent() {
        let request = AmdProfilePlannerRequest::new(
            AmdProfileIntent::VisualQuality,
            "Single-player benchmark",
            ready_support(),
        )
        .with_display_refresh(144)
        .with_tradeoff_consent(true, true, true, true);

        let plan = plan_amd_profile(&request);

        assert_eq!(plan.profile_name, LIIIRAA_AMD_PROFILE_NAME);
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::HyprRx).decision,
            AmdProfileDecision::Enable
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::Afmf).decision,
            AmdProfileDecision::UserChoice
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::Ris).decision,
            AmdProfileDecision::UserChoice
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::Rsr).decision,
            AmdProfileDecision::UserChoice
        );
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::SamReBar).desired_state,
            Some("Detected enabled".to_owned())
        );
    }

    #[test]
    fn unsupported_capabilities_stay_unavailable() {
        let support = AmdFeatureSupport::not_applicable();
        let request =
            AmdProfilePlannerRequest::new(AmdProfileIntent::Balanced, "Unknown game", support);

        let plan = plan_amd_profile(&request);

        assert!(plan.recommendations.iter().all(|recommendation| {
            recommendation.decision == AmdProfileDecision::NotSupported
                || recommendation.feature == AmdProfileFeature::SamReBar
        }));
        assert_eq!(
            recommendation(&plan, AmdProfileFeature::AntiLag).capability,
            GpuCapabilityState::NotApplicable
        );
    }

    #[test]
    fn conflict_validator_blocks_fake_anti_lag2_and_competitive_afmf() {
        let mut selection = AmdProfileFeatureSelection::empty();
        selection.anti_lag = true;
        selection.anti_lag2 = true;
        selection.afmf = true;
        selection.radeon_boost = true;

        let conflicts = validate_amd_profile_feature_selection(
            &selection,
            AmdProfileValidationContext {
                intent: AmdProfileIntent::PubgCompetitive,
                anti_lag2_support: GpuCapabilityState::Missing,
                visual_tradeoff_consent: false,
                frame_generation_consent: false,
                dynamic_resolution_consent: false,
            },
        );

        assert!(conflicts.iter().any(|conflict| {
            conflict.severity == AmdConflictSeverity::Blocker
                && conflict.features == vec![AmdProfileFeature::AntiLag2]
        }));
        assert!(conflicts.iter().any(|conflict| {
            conflict.severity == AmdConflictSeverity::Blocker
                && conflict.features == vec![AmdProfileFeature::Afmf]
        }));
        assert!(conflicts.iter().any(|conflict| {
            conflict.severity == AmdConflictSeverity::Blocker
                && conflict.features == vec![AmdProfileFeature::RadeonBoost]
        }));
    }

    #[test]
    fn chill_and_frtc_combination_is_warned_not_blocked() {
        let mut selection = AmdProfileFeatureSelection::empty();
        selection.chill = true;
        selection.frtc = true;

        let conflicts = validate_amd_profile_feature_selection(
            &selection,
            AmdProfileValidationContext {
                intent: AmdProfileIntent::ThermalPower,
                anti_lag2_support: GpuCapabilityState::NotApplicable,
                visual_tradeoff_consent: false,
                frame_generation_consent: false,
                dynamic_resolution_consent: false,
            },
        );

        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].severity, AmdConflictSeverity::Warning);
        assert_eq!(
            conflicts[0].features,
            vec![AmdProfileFeature::Chill, AmdProfileFeature::Frtc]
        );
    }

    fn ready_support() -> AmdFeatureSupport {
        AmdFeatureSupport {
            hypr_rx: GpuCapabilityState::Ready,
            anti_lag: GpuCapabilityState::Ready,
            anti_lag2: GpuCapabilityState::Ready,
            radeon_boost: GpuCapabilityState::Ready,
            chill: GpuCapabilityState::Ready,
            frtc: GpuCapabilityState::Ready,
            enhanced_sync: GpuCapabilityState::Ready,
            freesync: GpuCapabilityState::Ready,
            afmf: GpuCapabilityState::Ready,
            ris: GpuCapabilityState::Ready,
            rsr: GpuCapabilityState::Ready,
            sam_rebar: GpuCapabilityState::Ready,
            per_game_profile: GpuCapabilityState::Ready,
        }
    }

    fn recommendation(
        plan: &AmdProfilePlan,
        feature: AmdProfileFeature,
    ) -> &AmdProfileRecommendation {
        plan.recommendation(feature)
            .expect("feature recommendation should exist")
    }
}
