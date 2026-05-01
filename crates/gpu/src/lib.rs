//! Vendor-neutral GPU capability and display modeling.

use std::collections::BTreeSet;

/// GPU vendor family used by detection and profile planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuVendor {
    /// NVIDIA GPU or driver stack.
    Nvidia,
    /// AMD Radeon GPU or driver stack.
    Amd,
    /// Intel integrated graphics, Iris Xe, or Arc graphics stack.
    Intel,
    /// Vendor could not be classified from scan data.
    Unknown,
}

impl GpuVendor {
    /// Returns a stable lowercase label for logs and plan messages.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::Intel => "intel",
            Self::Unknown => "unknown",
        }
    }

    /// Returns a user-facing vendor label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Amd => "AMD",
            Self::Intel => "Intel",
            Self::Unknown => "Unknown",
        }
    }

    /// Returns the vendor-facing name for ReBAR/SAM guidance.
    #[must_use]
    pub const fn rebar_sam_label(self) -> &'static str {
        match self {
            Self::Nvidia | Self::Intel | Self::Unknown => "Resizable BAR",
            Self::Amd => "Smart Access Memory/ReBAR",
        }
    }

    /// Returns the vendor-facing frame-generation feature name.
    #[must_use]
    pub const fn frame_generation_label(self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA Frame Generation",
            Self::Amd => "AMD Fluid Motion Frames",
            Self::Intel => "Intel frame generation",
            Self::Unknown => "frame generation",
        }
    }
}

/// Driver age that triggers a normal update recommendation.
pub const GPU_DRIVER_UPDATE_WARNING_DAYS: u16 = 180;
/// Driver age or crash evidence that triggers a clean-driver recommendation.
pub const GPU_DRIVER_CLEAN_INSTALL_WARNING_DAYS: u16 = 365;
/// Refresh rate treated as high-refresh for gaming pipeline checks.
pub const HIGH_REFRESH_RATE_HZ: u16 = 120;
/// Shader cache size that should be inspected instead of blindly cleared.
pub const LARGE_SHADER_CACHE_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// Optimization intent used by vendor-neutral GPU platform policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPlatformIntent {
    /// Latency-first competitive play.
    CompetitiveLatency,
    /// General performance profile.
    Balanced,
    /// Visual quality or single-player profile where generated frames may be acceptable.
    VisualQuality,
    /// Troubleshooting flow for crashes, stutter, or corruption symptoms.
    Troubleshooting,
}

impl GpuPlatformIntent {
    /// Returns true when latency policy should keep generated frames disabled by default.
    #[must_use]
    pub const fn is_competitive(self) -> bool {
        matches!(self, Self::CompetitiveLatency)
    }
}

/// Stable capability check identifier for platform recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuPlatformCheck {
    /// GPU driver version and release age.
    DriverAge,
    /// Active display refresh rate and highest available mode.
    DisplayRefresh,
    /// Variable refresh rate support or current state.
    VariableRefreshRate,
    /// Resizable BAR / Smart Access Memory platform state.
    RebarSam,
    /// Driver or game frame-generation policy.
    FrameGeneration,
    /// Shader cache availability and health.
    ShaderCache,
    /// Clean driver update or reinstall recommendation.
    CleanDriver,
}

/// Vendor-neutral action class for a platform recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPlatformDecision {
    /// State is healthy or already aligned with policy.
    Ready,
    /// More read-only evidence is needed before recommending changes.
    Inspect,
    /// Recommend a user-visible official setting or vendor update path.
    Recommend,
    /// Present as an explicit user choice with tradeoff copy.
    UserChoice,
    /// Keep disabled for the selected intent.
    KeepDisabled,
    /// Capability does not apply to this vendor or machine.
    NotApplicable,
}

/// Driver-age classification for the installed GPU stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuDriverAgeState {
    /// Driver age is within the normal window.
    Fresh,
    /// Driver is old enough to recommend an official update.
    Aging,
    /// Driver is stale or crashy enough to recommend a clean install path.
    Old,
    /// Driver date or version evidence is unavailable.
    Unknown,
    /// No matching vendor adapter was detected.
    NotApplicable,
}

/// Driver maintenance action selected by the platform planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuDriverMaintenanceAction {
    /// No driver maintenance recommendation is needed.
    None,
    /// Capture or expose driver version/date first.
    CaptureVersionAndDate,
    /// Recommend the official vendor update flow.
    UpdateRecommended,
    /// Recommend the vendor clean-install flow.
    CleanInstallRecommended,
}

/// Read-only display pipeline state used by GPU capability checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDisplayPipelineState {
    /// User-visible display name, if the scan exposes it.
    pub display_name: Option<String>,
    /// Active refresh rate in Hz.
    pub active_refresh_hz: Option<u16>,
    /// Highest refresh mode exposed by the current display path.
    pub highest_available_refresh_hz: Option<u16>,
    /// Variable refresh rate support or current state.
    pub vrr_state: GpuCapabilityState,
}

impl GpuDisplayPipelineState {
    /// Creates a display pipeline state with unknown scan details.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            display_name: None,
            active_refresh_hz: None,
            highest_available_refresh_hz: None,
            vrr_state: GpuCapabilityState::Unknown,
        }
    }

    /// Creates a display pipeline state from active refresh and VRR state.
    #[must_use]
    pub fn new(
        active_refresh_hz: Option<u16>,
        highest_available_refresh_hz: Option<u16>,
        vrr_state: GpuCapabilityState,
    ) -> Self {
        Self {
            display_name: None,
            active_refresh_hz,
            highest_available_refresh_hz,
            vrr_state,
        }
    }

    /// Adds a user-facing display name.
    #[must_use]
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}

/// Shader cache status exposed by vendor or OS scan data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuShaderCacheState {
    /// Whether shader caching applies to the detected vendor stack.
    pub availability: GpuCapabilityState,
    /// Whether shader caching is enabled, if exposed.
    pub enabled: Option<bool>,
    /// Current cache size in bytes, if measured.
    pub size_bytes: Option<u64>,
    /// Whether corruption/stutter symptoms point to a troubleshooting clear.
    pub corruption_suspected: bool,
}

impl GpuShaderCacheState {
    /// Creates an unknown shader-cache state for a detected GPU.
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            availability: GpuCapabilityState::Unknown,
            enabled: None,
            size_bytes: None,
            corruption_suspected: false,
        }
    }

    /// Creates an enabled shader-cache state.
    #[must_use]
    pub fn enabled(size_bytes: Option<u64>) -> Self {
        Self {
            availability: GpuCapabilityState::Ready,
            enabled: Some(true),
            size_bytes,
            corruption_suspected: false,
        }
    }

    /// Creates a disabled shader-cache state.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            availability: GpuCapabilityState::Ready,
            enabled: Some(false),
            size_bytes: None,
            corruption_suspected: false,
        }
    }

    /// Flags suspected shader-cache corruption for a troubleshooting plan.
    #[must_use]
    pub fn with_corruption_suspected(mut self) -> Self {
        self.corruption_suspected = true;
        self
    }
}

/// Inputs used to plan vendor-neutral GPU platform capability checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPlatformCheckRequest {
    /// Optimization intent for latency/visual tradeoff policy.
    pub intent: GpuPlatformIntent,
    /// Approximate GPU driver age in days.
    pub driver_age_days: Option<u16>,
    /// Whether recent driver resets/crashes were detected.
    pub recent_driver_crashes: bool,
    /// Display refresh and VRR state.
    pub display: GpuDisplayPipelineState,
    /// Resizable BAR / Smart Access Memory state.
    pub rebar_sam_state: GpuCapabilityState,
    /// Frame generation support state.
    pub frame_generation_state: GpuCapabilityState,
    /// Shader cache state.
    pub shader_cache: GpuShaderCacheState,
}

impl GpuPlatformCheckRequest {
    /// Creates a conservative platform check request.
    #[must_use]
    pub fn new(intent: GpuPlatformIntent) -> Self {
        Self {
            intent,
            driver_age_days: None,
            recent_driver_crashes: false,
            display: GpuDisplayPipelineState::unknown(),
            rebar_sam_state: GpuCapabilityState::Unknown,
            frame_generation_state: GpuCapabilityState::Unknown,
            shader_cache: GpuShaderCacheState::unknown(),
        }
    }

    /// Adds driver age evidence.
    #[must_use]
    pub fn with_driver_age_days(mut self, driver_age_days: u16) -> Self {
        self.driver_age_days = Some(driver_age_days);
        self
    }

    /// Adds recent driver crash evidence.
    #[must_use]
    pub fn with_recent_driver_crashes(mut self, recent_driver_crashes: bool) -> Self {
        self.recent_driver_crashes = recent_driver_crashes;
        self
    }

    /// Adds display pipeline state.
    #[must_use]
    pub fn with_display(mut self, display: GpuDisplayPipelineState) -> Self {
        self.display = display;
        self
    }

    /// Adds ReBAR/SAM state.
    #[must_use]
    pub fn with_rebar_sam_state(mut self, rebar_sam_state: GpuCapabilityState) -> Self {
        self.rebar_sam_state = rebar_sam_state;
        self
    }

    /// Adds frame-generation support state.
    #[must_use]
    pub fn with_frame_generation_state(
        mut self,
        frame_generation_state: GpuCapabilityState,
    ) -> Self {
        self.frame_generation_state = frame_generation_state;
        self
    }

    /// Adds shader-cache state.
    #[must_use]
    pub fn with_shader_cache(mut self, shader_cache: GpuShaderCacheState) -> Self {
        self.shader_cache = shader_cache;
        self
    }
}

/// Driver health check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDriverHealthCheck {
    /// Driver-age classification.
    pub age_state: GpuDriverAgeState,
    /// Driver maintenance action.
    pub action: GpuDriverMaintenanceAction,
    /// Driver age in days when known.
    pub age_days: Option<u16>,
    /// Whether driver crash evidence was used.
    pub recent_driver_crashes: bool,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// Display refresh check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDisplayRefreshCheck {
    /// Active refresh rate in Hz.
    pub active_refresh_hz: Option<u16>,
    /// Highest refresh mode in Hz.
    pub highest_available_refresh_hz: Option<u16>,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// VRR check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuVrrCheck {
    /// VRR capability state.
    pub state: GpuCapabilityState,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// ReBAR/SAM check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRebarSamCheck {
    /// Vendor-specific label for the capability.
    pub label: &'static str,
    /// ReBAR/SAM state.
    pub state: GpuCapabilityState,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// Frame-generation policy selected by the platform planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuFrameGenerationPolicy {
    /// Feature is not available or not applicable.
    Unsupported,
    /// Feature support must be confirmed before it appears in a plan.
    VerifySupport,
    /// Competitive latency profiles keep generated frames disabled by default.
    KeepOffForCompetitive,
    /// Visual/general profiles may offer frame generation with explicit consent.
    OptionalWithConsent,
}

/// Frame-generation check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuFrameGenerationCheck {
    /// Vendor-facing feature label.
    pub label: &'static str,
    /// Frame-generation support state.
    pub state: GpuCapabilityState,
    /// Selected policy.
    pub policy: GpuFrameGenerationPolicy,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Whether explicit consent is required before enabling.
    pub requires_user_consent: bool,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// Shader-cache policy selected by the platform planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuShaderCachePolicy {
    /// Shader cache does not apply.
    NotApplicable,
    /// Keep shader cache enabled.
    KeepEnabled,
    /// Recommend enabling shader cache.
    Enable,
    /// Inspect cache state because scan data is incomplete.
    Inspect,
    /// Inspect a large cache, but do not clear it by default.
    InspectLargeCache,
    /// Clear only as a troubleshooting action with first-run stutter warning.
    ClearForTroubleshooting,
}

/// Shader-cache check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuShaderCacheCheck {
    /// Selected shader-cache policy.
    pub policy: GpuShaderCachePolicy,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Measured cache size in bytes, if known.
    pub size_bytes: Option<u64>,
    /// Whether explicit consent is required before clearing cache data.
    pub requires_user_consent: bool,
    /// Human-readable notes for the UI or audit log.
    pub notes: Vec<String>,
}

/// One user-visible platform recommendation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPlatformRecommendation {
    /// Capability check that produced this recommendation.
    pub check: GpuPlatformCheck,
    /// Planner decision.
    pub decision: GpuPlatformDecision,
    /// Whether explicit consent is required before applying or guiding the action.
    pub requires_user_consent: bool,
    /// Human-readable recommendation.
    pub message: String,
}

impl GpuPlatformRecommendation {
    fn new(
        check: GpuPlatformCheck,
        decision: GpuPlatformDecision,
        requires_user_consent: bool,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check,
            decision,
            requires_user_consent,
            message: message.into(),
        }
    }
}

/// Vendor-neutral GPU platform capability plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPlatformCapabilityPlan {
    /// Vendor being planned.
    pub vendor: GpuVendor,
    /// Matching driver versions from inventory.
    pub driver_versions: Vec<String>,
    /// Driver health and clean-install guidance.
    pub driver: GpuDriverHealthCheck,
    /// Display refresh check.
    pub display_refresh: GpuDisplayRefreshCheck,
    /// Variable refresh rate check.
    pub vrr: GpuVrrCheck,
    /// ReBAR/SAM check.
    pub rebar_sam: GpuRebarSamCheck,
    /// Frame-generation policy check.
    pub frame_generation: GpuFrameGenerationCheck,
    /// Shader-cache state check.
    pub shader_cache: GpuShaderCacheCheck,
    /// User-visible recommendations that need attention.
    pub recommendations: Vec<GpuPlatformRecommendation>,
}

/// Conservative readiness state for GPU driver or API capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuCapabilityState {
    /// Capability or dependency appears present.
    Ready,
    /// Capability or dependency appears absent.
    Missing,
    /// Scan data does not expose this capability.
    Unknown,
    /// Capability does not apply to the detected hardware.
    NotApplicable,
}

impl GpuCapabilityState {
    /// Returns true when a user-visible recommendation is needed.
    #[must_use]
    pub const fn needs_attention(self) -> bool {
        matches!(self, Self::Missing | Self::Unknown)
    }
}

/// Read-only facts for one GPU adapter from OS inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuAdapter {
    /// Adapter display name.
    pub name: String,
    /// Classified vendor.
    pub vendor: GpuVendor,
    /// Driver version as reported by Windows or the vendor tool.
    pub driver_version: Option<String>,
    /// Video processor string when exposed by the OS.
    pub video_processor: Option<String>,
    /// Adapter memory in bytes when exposed.
    pub adapter_ram_bytes: Option<u64>,
    /// Plug-and-play device ID, used for PCI vendor-ID classification.
    pub pnp_device_id: Option<String>,
}

impl GpuAdapter {
    /// Classifies one GPU adapter from read-only scan values.
    #[must_use]
    pub fn from_scan(
        name: impl Into<String>,
        driver_version: Option<&str>,
        video_processor: Option<&str>,
        adapter_ram_bytes: Option<u64>,
        pnp_device_id: Option<&str>,
    ) -> Self {
        let name = name.into();
        let video_processor = video_processor.map(str::to_owned);
        let pnp_device_id = pnp_device_id.map(str::to_owned);
        let vendor = classify_vendor(&name, video_processor.as_deref(), pnp_device_id.as_deref());

        Self {
            name,
            vendor,
            driver_version: driver_version
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
            video_processor,
            adapter_ram_bytes,
            pnp_device_id,
        }
    }

    /// Returns true when the adapter reported a non-empty driver version.
    #[must_use]
    pub fn has_driver_version(&self) -> bool {
        self.driver_version
            .as_deref()
            .is_some_and(|version| !version.trim().is_empty())
    }
}

/// Read-only GPU inventory consumed by vendor-specific detectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuInventory {
    /// GPU adapters detected on the machine.
    pub adapters: Vec<GpuAdapter>,
}

impl GpuInventory {
    /// Creates a GPU inventory from detected adapters.
    #[must_use]
    pub fn new(adapters: Vec<GpuAdapter>) -> Self {
        Self { adapters }
    }

    /// Returns true when any adapter matches the vendor.
    #[must_use]
    pub fn has_vendor(&self, vendor: GpuVendor) -> bool {
        self.adapters
            .iter()
            .any(|adapter| adapter.vendor == vendor)
    }

    /// Returns all adapters for one vendor.
    #[must_use]
    pub fn adapters_for_vendor(&self, vendor: GpuVendor) -> Vec<&GpuAdapter> {
        self.adapters
            .iter()
            .filter(|adapter| adapter.vendor == vendor)
            .collect()
    }

    /// Builds a vendor-specific read-only detection summary.
    #[must_use]
    pub fn vendor_detection(&self, vendor: GpuVendor) -> GpuVendorDetection {
        GpuVendorDetection::from_inventory(vendor, self)
    }
}

/// Read-only summary for one GPU vendor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuVendorDetection {
    /// Vendor being summarized.
    pub vendor: GpuVendor,
    /// Matching adapters.
    pub adapters: Vec<GpuAdapter>,
    /// Unique non-empty driver versions for matching adapters.
    pub driver_versions: Vec<String>,
    /// Number of matching adapters missing a driver version.
    pub missing_driver_versions: usize,
}

impl GpuVendorDetection {
    /// Builds a detection summary from a vendor-neutral inventory.
    #[must_use]
    pub fn from_inventory(vendor: GpuVendor, inventory: &GpuInventory) -> Self {
        let adapters = inventory
            .adapters
            .iter()
            .filter(|adapter| adapter.vendor == vendor)
            .cloned()
            .collect::<Vec<_>>();
        let driver_versions = adapters
            .iter()
            .filter_map(|adapter| adapter.driver_version.as_deref())
            .map(str::trim)
            .filter(|version| !version.is_empty())
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let missing_driver_versions = adapters
            .iter()
            .filter(|adapter| !adapter.has_driver_version())
            .count();

        Self {
            vendor,
            adapters,
            driver_versions,
            missing_driver_versions,
        }
    }

    /// Returns true when at least one matching adapter was detected.
    #[must_use]
    pub fn is_available(&self) -> bool {
        !self.adapters.is_empty()
    }

    /// Returns a conservative driver readiness signal for this vendor.
    #[must_use]
    pub fn driver_state(&self) -> GpuCapabilityState {
        if !self.is_available() {
            GpuCapabilityState::NotApplicable
        } else if self.missing_driver_versions > 0 || self.driver_versions.is_empty() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::Ready
        }
    }
}

/// Builds a vendor-neutral GPU platform capability plan from read-only scan facts.
#[must_use]
pub fn plan_gpu_platform_capabilities(
    detection: &GpuVendorDetection,
    request: &GpuPlatformCheckRequest,
) -> GpuPlatformCapabilityPlan {
    let driver = plan_driver_health(detection, request);
    let display_refresh = plan_display_refresh(detection, &request.display);
    let vrr = plan_vrr(detection, request.display.vrr_state);
    let rebar_sam = plan_rebar_sam(detection, request.rebar_sam_state);
    let frame_generation = plan_frame_generation(
        detection,
        request.intent,
        request.frame_generation_state,
    );
    let shader_cache = plan_shader_cache(detection, &request.shader_cache);
    let recommendations = collect_platform_recommendations(
        detection,
        &driver,
        &display_refresh,
        &vrr,
        &rebar_sam,
        &frame_generation,
        &shader_cache,
    );

    GpuPlatformCapabilityPlan {
        vendor: detection.vendor,
        driver_versions: detection.driver_versions.clone(),
        driver,
        display_refresh,
        vrr,
        rebar_sam,
        frame_generation,
        shader_cache,
        recommendations,
    }
}

fn plan_driver_health(
    detection: &GpuVendorDetection,
    request: &GpuPlatformCheckRequest,
) -> GpuDriverHealthCheck {
    if !detection.is_available() {
        return GpuDriverHealthCheck {
            age_state: GpuDriverAgeState::NotApplicable,
            action: GpuDriverMaintenanceAction::None,
            age_days: None,
            recent_driver_crashes: request.recent_driver_crashes,
            notes: vec![format!(
                "{} GPU platform checks are unavailable because no matching adapter was detected.",
                detection.vendor.display_name()
            )],
        };
    }

    if !matches!(detection.driver_state(), GpuCapabilityState::Ready) {
        return GpuDriverHealthCheck {
            age_state: GpuDriverAgeState::Unknown,
            action: GpuDriverMaintenanceAction::CaptureVersionAndDate,
            age_days: request.driver_age_days,
            recent_driver_crashes: request.recent_driver_crashes,
            notes: vec![format!(
                "{} driver version/date is incomplete; capture it before update or profile planning.",
                detection.vendor.display_name()
            )],
        };
    }

    match (
        request.driver_age_days,
        request.recent_driver_crashes,
    ) {
        (_, true) => GpuDriverHealthCheck {
            age_state: GpuDriverAgeState::Old,
            action: GpuDriverMaintenanceAction::CleanInstallRecommended,
            age_days: request.driver_age_days,
            recent_driver_crashes: true,
            notes: vec![format!(
                "{} driver crashes were detected; recommend the official clean-install path before profile tuning.",
                detection.vendor.display_name()
            )],
        },
        (Some(age_days), false) if age_days >= GPU_DRIVER_CLEAN_INSTALL_WARNING_DAYS => {
            GpuDriverHealthCheck {
                age_state: GpuDriverAgeState::Old,
                action: GpuDriverMaintenanceAction::CleanInstallRecommended,
                age_days: Some(age_days),
                recent_driver_crashes: false,
                notes: vec![format!(
                    "{} driver is {age_days} days old; recommend official update or clean reinstall if symptoms persist.",
                    detection.vendor.display_name()
                )],
            }
        }
        (Some(age_days), false) if age_days >= GPU_DRIVER_UPDATE_WARNING_DAYS => {
            GpuDriverHealthCheck {
                age_state: GpuDriverAgeState::Aging,
                action: GpuDriverMaintenanceAction::UpdateRecommended,
                age_days: Some(age_days),
                recent_driver_crashes: false,
                notes: vec![format!(
                    "{} driver is {age_days} days old; recommend checking the official vendor update flow.",
                    detection.vendor.display_name()
                )],
            }
        }
        (Some(age_days), false) => GpuDriverHealthCheck {
            age_state: GpuDriverAgeState::Fresh,
            action: GpuDriverMaintenanceAction::None,
            age_days: Some(age_days),
            recent_driver_crashes: false,
            notes: vec![format!(
                "{} driver age is inside the normal update window.",
                detection.vendor.display_name()
            )],
        },
        (None, false) => GpuDriverHealthCheck {
            age_state: GpuDriverAgeState::Unknown,
            action: GpuDriverMaintenanceAction::CaptureVersionAndDate,
            age_days: None,
            recent_driver_crashes: false,
            notes: vec![format!(
                "{} driver date is unknown; capture release date before judging age.",
                detection.vendor.display_name()
            )],
        },
    }
}

fn plan_display_refresh(
    detection: &GpuVendorDetection,
    display: &GpuDisplayPipelineState,
) -> GpuDisplayRefreshCheck {
    if !detection.is_available() {
        return GpuDisplayRefreshCheck {
            active_refresh_hz: None,
            highest_available_refresh_hz: None,
            decision: GpuPlatformDecision::NotApplicable,
            notes: vec!["Display refresh checks require a detected GPU adapter.".to_owned()],
        };
    }

    let Some(active_refresh_hz) = display.active_refresh_hz else {
        return GpuDisplayRefreshCheck {
            active_refresh_hz: None,
            highest_available_refresh_hz: display.highest_available_refresh_hz,
            decision: GpuPlatformDecision::Inspect,
            notes: vec![
                "Active display refresh is unknown; read Windows display mode before gaming profile planning."
                    .to_owned(),
            ],
        };
    };

    if display
        .highest_available_refresh_hz
        .is_some_and(|highest| highest > active_refresh_hz)
    {
        return GpuDisplayRefreshCheck {
            active_refresh_hz: Some(active_refresh_hz),
            highest_available_refresh_hz: display.highest_available_refresh_hz,
            decision: GpuPlatformDecision::Recommend,
            notes: vec![format!(
                "Display is running at {active_refresh_hz} Hz while a higher refresh mode is available; recommend selecting the highest official mode."
            )],
        };
    }

    if active_refresh_hz >= HIGH_REFRESH_RATE_HZ {
        GpuDisplayRefreshCheck {
            active_refresh_hz: Some(active_refresh_hz),
            highest_available_refresh_hz: display.highest_available_refresh_hz,
            decision: GpuPlatformDecision::Ready,
            notes: vec![format!(
                "Display refresh is {active_refresh_hz} Hz and ready for high-refresh profile planning."
            )],
        }
    } else {
        GpuDisplayRefreshCheck {
            active_refresh_hz: Some(active_refresh_hz),
            highest_available_refresh_hz: display.highest_available_refresh_hz,
            decision: GpuPlatformDecision::Inspect,
            notes: vec![format!(
                "Display is running at {active_refresh_hz} Hz; verify the monitor, cable, and Windows display mode before assuming a GPU bottleneck."
            )],
        }
    }
}

fn plan_vrr(
    detection: &GpuVendorDetection,
    vrr_state: GpuCapabilityState,
) -> GpuVrrCheck {
    if !detection.is_available() {
        return GpuVrrCheck {
            state: GpuCapabilityState::NotApplicable,
            decision: GpuPlatformDecision::NotApplicable,
            notes: vec!["VRR checks require a detected GPU adapter.".to_owned()],
        };
    }

    match vrr_state {
        GpuCapabilityState::Ready => GpuVrrCheck {
            state: vrr_state,
            decision: GpuPlatformDecision::Ready,
            notes: vec![
                "VRR is confirmed; cap and sync policy can be planned below refresh rate."
                    .to_owned(),
            ],
        },
        GpuCapabilityState::Missing => GpuVrrCheck {
            state: vrr_state,
            decision: GpuPlatformDecision::Inspect,
            notes: vec![
                "VRR is not enabled or not supported; recommend only official display/driver setup steps."
                    .to_owned(),
            ],
        },
        GpuCapabilityState::Unknown => GpuVrrCheck {
            state: vrr_state,
            decision: GpuPlatformDecision::Inspect,
            notes: vec!["VRR state is unknown; detect it before applying cap/sync policy.".to_owned()],
        },
        GpuCapabilityState::NotApplicable => GpuVrrCheck {
            state: vrr_state,
            decision: GpuPlatformDecision::NotApplicable,
            notes: vec!["VRR does not apply to this display path.".to_owned()],
        },
    }
}

fn plan_rebar_sam(
    detection: &GpuVendorDetection,
    rebar_sam_state: GpuCapabilityState,
) -> GpuRebarSamCheck {
    let label = detection.vendor.rebar_sam_label();
    if !detection.is_available() {
        return GpuRebarSamCheck {
            label,
            state: GpuCapabilityState::NotApplicable,
            decision: GpuPlatformDecision::NotApplicable,
            notes: vec![format!("{label} checks require a detected GPU adapter.")],
        };
    }

    match rebar_sam_state {
        GpuCapabilityState::Ready => GpuRebarSamCheck {
            label,
            state: rebar_sam_state,
            decision: GpuPlatformDecision::Ready,
            notes: vec![format!(
                "{label} is detected; keep vendor/game policy and avoid hidden global overrides."
            )],
        },
        GpuCapabilityState::Missing | GpuCapabilityState::Unknown => GpuRebarSamCheck {
            label,
            state: rebar_sam_state,
            decision: GpuPlatformDecision::Recommend,
            notes: vec![format!(
                "{label} is not confirmed; recommend official BIOS, driver, and platform checks without firmware flashing or forced hidden flags."
            )],
        },
        GpuCapabilityState::NotApplicable => GpuRebarSamCheck {
            label,
            state: rebar_sam_state,
            decision: GpuPlatformDecision::NotApplicable,
            notes: vec![format!("{label} does not apply to this machine.")],
        },
    }
}

fn plan_frame_generation(
    detection: &GpuVendorDetection,
    intent: GpuPlatformIntent,
    frame_generation_state: GpuCapabilityState,
) -> GpuFrameGenerationCheck {
    let label = detection.vendor.frame_generation_label();
    if !detection.is_available() {
        return GpuFrameGenerationCheck {
            label,
            state: GpuCapabilityState::NotApplicable,
            policy: GpuFrameGenerationPolicy::Unsupported,
            decision: GpuPlatformDecision::NotApplicable,
            requires_user_consent: false,
            notes: vec![format!("{label} policy requires a detected GPU adapter.")],
        };
    }

    match frame_generation_state {
        GpuCapabilityState::Ready if intent.is_competitive() => {
            GpuFrameGenerationCheck {
                label,
                state: frame_generation_state,
                policy: GpuFrameGenerationPolicy::KeepOffForCompetitive,
                decision: GpuPlatformDecision::KeepDisabled,
                requires_user_consent: false,
                notes: vec![format!(
                    "{label} is available but stays off for competitive latency profiles; benchmark native and generated frames separately."
                )],
            }
        }
        GpuCapabilityState::Ready => GpuFrameGenerationCheck {
            label,
            state: frame_generation_state,
            policy: GpuFrameGenerationPolicy::OptionalWithConsent,
            decision: GpuPlatformDecision::UserChoice,
            requires_user_consent: true,
            notes: vec![format!(
                "{label} may be offered for visual/general profiles only with explicit consent and benchmark labeling."
            )],
        },
        GpuCapabilityState::Unknown => GpuFrameGenerationCheck {
            label,
            state: frame_generation_state,
            policy: GpuFrameGenerationPolicy::VerifySupport,
            decision: GpuPlatformDecision::Inspect,
            requires_user_consent: false,
            notes: vec![format!(
                "{label} support is unknown; verify driver and game support before showing it as an option."
            )],
        },
        GpuCapabilityState::Missing | GpuCapabilityState::NotApplicable => {
            GpuFrameGenerationCheck {
                label,
                state: frame_generation_state,
                policy: GpuFrameGenerationPolicy::Unsupported,
                decision: GpuPlatformDecision::NotApplicable,
                requires_user_consent: false,
                notes: vec![format!("{label} is unavailable for this platform.")],
            }
        }
    }
}

fn plan_shader_cache(
    detection: &GpuVendorDetection,
    shader_cache: &GpuShaderCacheState,
) -> GpuShaderCacheCheck {
    if !detection.is_available()
        || shader_cache.availability == GpuCapabilityState::NotApplicable
    {
        return GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::NotApplicable,
            decision: GpuPlatformDecision::NotApplicable,
            size_bytes: None,
            requires_user_consent: false,
            notes: vec!["Shader cache checks do not apply to this platform.".to_owned()],
        };
    }

    if shader_cache.corruption_suspected {
        return GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::ClearForTroubleshooting,
            decision: GpuPlatformDecision::UserChoice,
            size_bytes: shader_cache.size_bytes,
            requires_user_consent: true,
            notes: vec![
                "Clear shader cache only for corruption/stutter troubleshooting and warn about first-run shader rebuild stutter."
                    .to_owned(),
            ],
        };
    }

    if shader_cache.enabled == Some(false) {
        return GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::Enable,
            decision: GpuPlatformDecision::Recommend,
            size_bytes: shader_cache.size_bytes,
            requires_user_consent: false,
            notes: vec![
                "Shader cache is disabled; recommend enabling the vendor default instead of clearing on every launch."
                    .to_owned(),
            ],
        };
    }

    if shader_cache
        .size_bytes
        .is_some_and(|size| size >= LARGE_SHADER_CACHE_BYTES)
    {
        return GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::InspectLargeCache,
            decision: GpuPlatformDecision::Inspect,
            size_bytes: shader_cache.size_bytes,
            requires_user_consent: false,
            notes: vec![
                "Shader cache is large; inspect size and symptoms, but do not schedule blind cache clears."
                    .to_owned(),
            ],
        };
    }

    if shader_cache.availability == GpuCapabilityState::Unknown || shader_cache.enabled.is_none() {
        GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::Inspect,
            decision: GpuPlatformDecision::Inspect,
            size_bytes: shader_cache.size_bytes,
            requires_user_consent: false,
            notes: vec!["Shader cache state is unknown; inspect before recommending changes.".to_owned()],
        }
    } else {
        GpuShaderCacheCheck {
            policy: GpuShaderCachePolicy::KeepEnabled,
            decision: GpuPlatformDecision::Ready,
            size_bytes: shader_cache.size_bytes,
            requires_user_consent: false,
            notes: vec![
                "Shader cache is enabled; keep it on and avoid routine clear-on-launch behavior."
                    .to_owned(),
            ],
        }
    }
}

fn collect_platform_recommendations(
    detection: &GpuVendorDetection,
    driver: &GpuDriverHealthCheck,
    display_refresh: &GpuDisplayRefreshCheck,
    vrr: &GpuVrrCheck,
    rebar_sam: &GpuRebarSamCheck,
    frame_generation: &GpuFrameGenerationCheck,
    shader_cache: &GpuShaderCacheCheck,
) -> Vec<GpuPlatformRecommendation> {
    let mut recommendations = Vec::new();

    match driver.action {
        GpuDriverMaintenanceAction::CaptureVersionAndDate => {
            recommendations.push(GpuPlatformRecommendation::new(
                GpuPlatformCheck::DriverAge,
                GpuPlatformDecision::Inspect,
                false,
                driver.notes[0].clone(),
            ));
        }
        GpuDriverMaintenanceAction::UpdateRecommended => {
            recommendations.push(GpuPlatformRecommendation::new(
                GpuPlatformCheck::DriverAge,
                GpuPlatformDecision::Recommend,
                false,
                driver.notes[0].clone(),
            ));
        }
        GpuDriverMaintenanceAction::CleanInstallRecommended => {
            recommendations.push(GpuPlatformRecommendation::new(
                GpuPlatformCheck::CleanDriver,
                GpuPlatformDecision::Recommend,
                true,
                driver.notes[0].clone(),
            ));
        }
        GpuDriverMaintenanceAction::None => {}
    }

    if matches!(
        display_refresh.decision,
        GpuPlatformDecision::Inspect | GpuPlatformDecision::Recommend
    ) {
        recommendations.push(GpuPlatformRecommendation::new(
            GpuPlatformCheck::DisplayRefresh,
            display_refresh.decision,
            false,
            display_refresh.notes[0].clone(),
        ));
    }

    if matches!(
        vrr.decision,
        GpuPlatformDecision::Inspect | GpuPlatformDecision::Recommend
    ) {
        recommendations.push(GpuPlatformRecommendation::new(
            GpuPlatformCheck::VariableRefreshRate,
            vrr.decision,
            false,
            vrr.notes[0].clone(),
        ));
    }

    if rebar_sam.decision == GpuPlatformDecision::Recommend {
        recommendations.push(GpuPlatformRecommendation::new(
            GpuPlatformCheck::RebarSam,
            rebar_sam.decision,
            false,
            rebar_sam.notes[0].clone(),
        ));
    }

    if frame_generation.decision != GpuPlatformDecision::NotApplicable
        && frame_generation.decision != GpuPlatformDecision::Ready
    {
        recommendations.push(GpuPlatformRecommendation::new(
            GpuPlatformCheck::FrameGeneration,
            frame_generation.decision,
            frame_generation.requires_user_consent,
            frame_generation.notes[0].clone(),
        ));
    }

    if shader_cache.decision != GpuPlatformDecision::NotApplicable
        && shader_cache.decision != GpuPlatformDecision::Ready
    {
        recommendations.push(GpuPlatformRecommendation::new(
            GpuPlatformCheck::ShaderCache,
            shader_cache.decision,
            shader_cache.requires_user_consent,
            shader_cache.notes[0].clone(),
        ));
    }

    recommendations.sort_by_key(|recommendation| recommendation.check);
    if !detection.is_available() {
        recommendations.clear();
    }
    recommendations
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

/// GPU crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "gpu",
    responsibility: "model vendor-neutral GPU capabilities, display state, VRR, ReBAR, and SAM",
    requires_live_windows: true,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

/// Classifies a GPU vendor from adapter strings and PCI vendor IDs.
#[must_use]
pub fn classify_vendor(
    name: &str,
    video_processor: Option<&str>,
    pnp_device_id: Option<&str>,
) -> GpuVendor {
    let text = format!(
        "{} {} {}",
        name,
        video_processor.unwrap_or_default(),
        pnp_device_id.unwrap_or_default()
    )
    .to_ascii_lowercase();

    if text.contains("ven_10de")
        || text.contains("nvidia")
        || text.contains("geforce")
        || text.contains("quadro")
    {
        GpuVendor::Nvidia
    } else if text.contains("ven_1002")
        || text.contains("advanced micro devices")
        || text.contains("amd")
        || text.contains("radeon")
        || text.contains("ati ")
    {
        GpuVendor::Amd
    } else if text.contains("ven_8086")
        || text.contains("intel")
        || text.contains("iris xe")
        || text.contains("uhd graphics")
        || text.contains("arc a")
    {
        GpuVendor::Intel
    } else {
        GpuVendor::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "gpu");
        assert!(info.responsibility.contains("VRR"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn classifies_gpu_vendors_from_fixture_scan_values() {
        let nvidia = GpuAdapter::from_scan(
            "NVIDIA GeForce RTX 4070",
            Some("32.0.15.6094"),
            Some("NVIDIA GeForce RTX 4070"),
            Some(12 * 1024 * 1024 * 1024),
            Some("PCI\\VEN_10DE&DEV_2786"),
        );
        let amd = GpuAdapter::from_scan(
            "AMD Radeon RX 7800 XT",
            Some("31.0.24002.92"),
            Some("AMD Radeon Graphics Processor"),
            Some(16 * 1024 * 1024 * 1024),
            Some("PCI\\VEN_1002&DEV_747E"),
        );
        let intel = GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            Some(16 * 1024 * 1024 * 1024),
            Some("PCI\\VEN_8086&DEV_56A0"),
        );

        assert_eq!(nvidia.vendor, GpuVendor::Nvidia);
        assert_eq!(amd.vendor, GpuVendor::Amd);
        assert_eq!(intel.vendor, GpuVendor::Intel);
    }

    #[test]
    fn summarizes_vendor_driver_detection() {
        let inventory = GpuInventory::new(vec![
            GpuAdapter::from_scan(
                "NVIDIA GeForce RTX 4070",
                Some("32.0.15.6094"),
                None,
                None,
                Some("PCI\\VEN_10DE&DEV_2786"),
            ),
            GpuAdapter::from_scan(
                "NVIDIA GeForce RTX 4060 Laptop GPU",
                Some("32.0.15.6094"),
                None,
                None,
                Some("PCI\\VEN_10DE&DEV_28E0"),
            ),
            GpuAdapter::from_scan(
                "Microsoft Basic Display Adapter",
                None,
                None,
                None,
                None,
            ),
        ]);

        let nvidia = inventory.vendor_detection(GpuVendor::Nvidia);
        let amd = inventory.vendor_detection(GpuVendor::Amd);

        assert!(nvidia.is_available());
        assert_eq!(nvidia.adapters.len(), 2);
        assert_eq!(nvidia.driver_versions, vec!["32.0.15.6094"]);
        assert_eq!(nvidia.driver_state(), GpuCapabilityState::Ready);
        assert_eq!(amd.driver_state(), GpuCapabilityState::NotApplicable);
    }

    #[test]
    fn unknown_driver_versions_need_attention() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "AMD Radeon RX 6800",
            None,
            None,
            None,
            Some("PCI\\VEN_1002&DEV_73BF"),
        )]);

        let amd = inventory.vendor_detection(GpuVendor::Amd);

        assert_eq!(amd.missing_driver_versions, 1);
        assert_eq!(amd.driver_state(), GpuCapabilityState::Unknown);
        assert!(amd.driver_state().needs_attention());
    }

    #[test]
    fn plans_clean_driver_recommendation_for_old_crashy_nvidia_stack() {
        let detection = nvidia_detection();
        let request = GpuPlatformCheckRequest::new(GpuPlatformIntent::CompetitiveLatency)
            .with_driver_age_days(420)
            .with_recent_driver_crashes(true)
            .with_display(GpuDisplayPipelineState::new(
                Some(144),
                Some(144),
                GpuCapabilityState::Ready,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Ready)
            .with_frame_generation_state(GpuCapabilityState::Ready)
            .with_shader_cache(GpuShaderCacheState::enabled(Some(512 * 1024 * 1024)));

        let plan = plan_gpu_platform_capabilities(&detection, &request);

        assert_eq!(plan.driver.age_state, GpuDriverAgeState::Old);
        assert_eq!(
            plan.driver.action,
            GpuDriverMaintenanceAction::CleanInstallRecommended
        );
        assert_eq!(
            plan.frame_generation.policy,
            GpuFrameGenerationPolicy::KeepOffForCompetitive
        );
        assert_eq!(plan.shader_cache.policy, GpuShaderCachePolicy::KeepEnabled);
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == GpuPlatformCheck::CleanDriver
                && recommendation.requires_user_consent
                && recommendation.message.contains("clean-install")
        }));
    }

    #[test]
    fn catches_display_refresh_and_vrr_setup_gaps() {
        let detection = nvidia_detection();
        let request = GpuPlatformCheckRequest::new(GpuPlatformIntent::Balanced)
            .with_driver_age_days(30)
            .with_display(GpuDisplayPipelineState::new(
                Some(60),
                Some(240),
                GpuCapabilityState::Missing,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Missing)
            .with_frame_generation_state(GpuCapabilityState::Unknown)
            .with_shader_cache(GpuShaderCacheState::unknown());

        let plan = plan_gpu_platform_capabilities(&detection, &request);

        assert_eq!(
            plan.display_refresh.decision,
            GpuPlatformDecision::Recommend
        );
        assert_eq!(plan.vrr.decision, GpuPlatformDecision::Inspect);
        assert_eq!(plan.rebar_sam.label, "Resizable BAR");
        assert_eq!(plan.rebar_sam.decision, GpuPlatformDecision::Recommend);
        assert!(plan
            .rebar_sam
            .notes
            .iter()
            .any(|note| note.contains("without firmware flashing")));
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == GpuPlatformCheck::DisplayRefresh
                && recommendation.message.contains("higher refresh")
        }));
    }

    #[test]
    fn treats_amd_frame_generation_as_visual_user_choice() {
        let detection = amd_detection();
        let request = GpuPlatformCheckRequest::new(GpuPlatformIntent::VisualQuality)
            .with_driver_age_days(45)
            .with_display(GpuDisplayPipelineState::new(
                Some(165),
                Some(165),
                GpuCapabilityState::Ready,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Ready)
            .with_frame_generation_state(GpuCapabilityState::Ready)
            .with_shader_cache(GpuShaderCacheState::enabled(None));

        let plan = plan_gpu_platform_capabilities(&detection, &request);

        assert_eq!(plan.rebar_sam.label, "Smart Access Memory/ReBAR");
        assert_eq!(plan.frame_generation.label, "AMD Fluid Motion Frames");
        assert_eq!(
            plan.frame_generation.policy,
            GpuFrameGenerationPolicy::OptionalWithConsent
        );
        assert!(plan.frame_generation.requires_user_consent);
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == GpuPlatformCheck::FrameGeneration
                && recommendation.requires_user_consent
        }));
    }

    #[test]
    fn shader_cache_clear_is_troubleshooting_only() {
        let detection = nvidia_detection();
        let request = GpuPlatformCheckRequest::new(GpuPlatformIntent::Troubleshooting)
            .with_driver_age_days(40)
            .with_display(GpuDisplayPipelineState::new(
                Some(240),
                Some(240),
                GpuCapabilityState::Ready,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Ready)
            .with_frame_generation_state(GpuCapabilityState::NotApplicable)
            .with_shader_cache(
                GpuShaderCacheState::enabled(Some(2 * 1024 * 1024 * 1024))
                    .with_corruption_suspected(),
            );

        let plan = plan_gpu_platform_capabilities(&detection, &request);

        assert_eq!(
            plan.shader_cache.policy,
            GpuShaderCachePolicy::ClearForTroubleshooting
        );
        assert!(plan.shader_cache.requires_user_consent);
        assert!(plan
            .shader_cache
            .notes
            .iter()
            .any(|note| note.contains("first-run shader rebuild stutter")));
    }

    fn nvidia_detection() -> GpuVendorDetection {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "NVIDIA GeForce RTX 4070",
            Some("32.0.15.6094"),
            None,
            None,
            Some("PCI\\VEN_10DE&DEV_2786"),
        )]);

        inventory.vendor_detection(GpuVendor::Nvidia)
    }

    fn amd_detection() -> GpuVendorDetection {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "AMD Radeon RX 7800 XT",
            Some("31.0.24002.92"),
            None,
            None,
            Some("PCI\\VEN_1002&DEV_747E"),
        )]);

        inventory.vendor_detection(GpuVendor::Amd)
    }
}
