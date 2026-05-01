//! NVIDIA profile backup, planning, apply, and verification.

use gpu::{
    plan_gpu_platform_capabilities, GpuCapabilityState, GpuInventory,
    GpuPlatformCapabilityPlan, GpuPlatformCheckRequest, GpuVendor,
    GpuVendorDetection,
};
use pubg::{PubgRuntimeState, PUBG_EXECUTABLE_NAME};
use std::{collections::BTreeSet, fmt};

/// Tweak ID for the required NVIDIA profile backup action.
pub const NVIDIA_PROFILE_BACKUP_TWEAK_ID: &str = "nvidia.backup.profiles";

/// Tweak ID for the conservative global NVIDIA performance profile.
pub const NVIDIA_GLOBAL_PROFILE_TWEAK_ID: &str = "nvidia.global.profile";

/// Tweak ID for the PUBG competitive NVIDIA application profile.
pub const NVIDIA_PUBG_PROFILE_TWEAK_ID: &str = "nvidia.pubg.profile";
/// Tweak ID for NVIDIA clean driver update guidance.
pub const NVIDIA_DRIVER_UPDATE_CLEAN_TWEAK_ID: &str = "nvidia.driver.update-clean";
/// Tweak ID for NVIDIA Resizable BAR detection.
pub const NVIDIA_REBAR_DETECT_TWEAK_ID: &str = "nvidia.rebar.detect";
/// Tweak ID for NVIDIA frame-generation competitive policy.
pub const NVIDIA_FRAMEGEN_POLICY_TWEAK_ID: &str = "nvidia.framegen.competitive-policy";
/// Tweak ID for NVIDIA shader-cache state inspection.
pub const NVIDIA_SHADER_CACHE_TWEAK_ID: &str = "nvidia.shader-cache.size";

/// Tweak ID for restoring backed-up NVIDIA profile state.
pub const NVIDIA_PROFILE_ROLLBACK_TWEAK_ID: &str = "nvidia.profile.rollback";

/// Driver profile name owned by Liiiraa for conservative global performance defaults.
pub const LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME: &str =
    "Liiiraa Boost - Global Performance";

/// Driver profile name owned by Liiiraa for PUBG competitive settings.
pub const LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME: &str =
    "Liiiraa Boost - PUBG Competitive";

const APPROVED_GLOBAL_PROFILE_SETTINGS: &[(
    &str,
    &str,
    &str,
    NvidiaProfileSettingVisibility,
)] = &[
    (
        "max-frame-rate",
        "Max Frame Rate",
        "Off",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
    (
        "low-latency-mode",
        "Low Latency Mode",
        "Off",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
    (
        "power-management-mode",
        "Power management mode",
        "Normal",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
    (
        "shader-cache",
        "Shader Cache",
        "On",
        NvidiaProfileSettingVisibility::Documented,
    ),
    (
        "texture-filtering-quality",
        "Texture filtering - Quality",
        "Quality",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
    (
        "threaded-optimization",
        "Threaded optimization",
        "Auto",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
    (
        "vertical-sync",
        "Vertical sync",
        "Use the 3D application setting",
        NvidiaProfileSettingVisibility::UserVisible,
    ),
];

const REQUIRED_PUBG_COMPETITIVE_SETTING_IDS: &[&str] = &[
    "max-frame-rate",
    "low-latency-mode",
    "monitor-technology",
    "power-management-mode",
    "preferred-refresh-rate",
    "shader-cache",
    "texture-filtering-quality",
    "vertical-sync",
];

/// Read-only NVIDIA GPU, driver, and profile-tool detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaDriverDetection {
    /// NVIDIA adapters and driver versions detected from inventory.
    pub vendor: GpuVendorDetection,
    /// NVAPI or Driver Settings API readiness.
    pub profile_api_state: GpuCapabilityState,
    /// NVIDIA Profile Inspector compatibility/import availability.
    pub profile_inspector_state: GpuCapabilityState,
}

impl NvidiaDriverDetection {
    /// Builds a conservative NVIDIA detection summary from GPU inventory.
    #[must_use]
    pub fn from_inventory(inventory: &GpuInventory) -> Self {
        let vendor = inventory.vendor_detection(GpuVendor::Nvidia);
        let capability_state = if vendor.is_available() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::NotApplicable
        };

        Self {
            vendor,
            profile_api_state: capability_state,
            profile_inspector_state: capability_state,
        }
    }

    /// Returns true when an NVIDIA adapter was detected.
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

/// Builds the NVIDIA view of the shared GPU platform capability plan.
#[must_use]
pub fn plan_nvidia_platform_capabilities(
    detection: &NvidiaDriverDetection,
    request: &GpuPlatformCheckRequest,
) -> GpuPlatformCapabilityPlan {
    plan_gpu_platform_capabilities(&detection.vendor, request)
}

/// Reflex and driver Low Latency Mode decision for PUBG.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvidiaPubgReflexPolicy {
    /// Prefer PUBG's in-game NVIDIA Reflex path and keep driver Low Latency Mode off.
    PreferInGameReflex,
    /// Use driver Low Latency Mode On when Reflex support cannot be confirmed.
    DriverLowLatencyOn,
}

impl NvidiaPubgReflexPolicy {
    /// Returns the planned driver Low Latency Mode value.
    #[must_use]
    pub const fn driver_low_latency_value(self) -> &'static str {
        match self {
            Self::PreferInGameReflex => "Off",
            Self::DriverLowLatencyOn => "On",
        }
    }

    /// Returns a user-visible policy note.
    #[must_use]
    pub const fn note(self) -> &'static str {
        match self {
            Self::PreferInGameReflex => {
                "Prefer PUBG in-game NVIDIA Reflex + Boost; keep driver Low Latency Mode Off to avoid stacking latency controls."
            }
            Self::DriverLowLatencyOn => {
                "Use driver Low Latency Mode On for the non-Reflex path; Ultra remains blocked as a default."
            }
        }
    }
}

/// G-SYNC/VRR and FPS cap policy for the PUBG profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvidiaPubgVrrPolicy {
    /// VRR is enabled and the driver profile owns an FPS cap below refresh rate.
    VrrCap {
        /// Active display refresh rate.
        refresh_rate_hz: u16,
        /// Recommended cap below refresh rate.
        cap_fps: u16,
    },
    /// VRR is unavailable or unknown, so sync and cap stay application-controlled.
    ApplicationControlled,
}

impl NvidiaPubgVrrPolicy {
    /// Returns the planned Max Frame Rate setting.
    #[must_use]
    pub fn max_frame_rate_value(self) -> String {
        match self {
            Self::VrrCap { cap_fps, .. } => format!("{cap_fps} FPS"),
            Self::ApplicationControlled => "Off".to_owned(),
        }
    }

    /// Returns the planned monitor technology setting.
    #[must_use]
    pub const fn monitor_technology_value(self) -> &'static str {
        match self {
            Self::VrrCap { .. } => "G-SYNC Compatible",
            Self::ApplicationControlled => "Use global setting",
        }
    }

    /// Returns the planned Vertical sync setting.
    #[must_use]
    pub const fn vertical_sync_value(self) -> &'static str {
        match self {
            Self::VrrCap { .. } => "On",
            Self::ApplicationControlled => "Use the 3D application setting",
        }
    }

    /// Returns a user-visible policy note.
    #[must_use]
    pub fn note(self) -> String {
        match self {
            Self::VrrCap {
                refresh_rate_hz,
                cap_fps,
            } => format!(
                "Use G-SYNC/VRR with NVIDIA V-SYNC On and cap PUBG to {cap_fps} FPS below {refresh_rate_hz} Hz."
            ),
            Self::ApplicationControlled => {
                "Keep FPS cap and sync application-controlled until VRR support and refresh rate are confirmed."
                    .to_owned()
            }
        }
    }
}

/// Resizable BAR policy for the PUBG profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvidiaPubgRebarPolicy {
    /// ReBAR is detected; keep NVIDIA's official per-title driver policy.
    OfficialDriverPolicy,
    /// ReBAR appears disabled or unknown; recommend official BIOS/VBIOS/driver checks only.
    RecommendOfficialEnablement,
    /// ReBAR does not apply to this system.
    NotApplicable,
}

impl NvidiaPubgRebarPolicy {
    /// Builds a ReBAR policy from read-only capability state.
    #[must_use]
    pub const fn from_capability(state: GpuCapabilityState) -> Self {
        match state {
            GpuCapabilityState::Ready => Self::OfficialDriverPolicy,
            GpuCapabilityState::Missing | GpuCapabilityState::Unknown => {
                Self::RecommendOfficialEnablement
            }
            GpuCapabilityState::NotApplicable => Self::NotApplicable,
        }
    }

    /// Returns a user-visible policy note.
    #[must_use]
    pub const fn note(self) -> &'static str {
        match self {
            Self::OfficialDriverPolicy => {
                "Resizable BAR is detected; keep NVIDIA's official per-title policy and avoid hidden override bits."
            }
            Self::RecommendOfficialEnablement => {
                "Resizable BAR is not confirmed; recommend official BIOS/VBIOS/driver checks, not firmware flashing or hidden overrides."
            }
            Self::NotApplicable => {
                "Resizable BAR does not apply to this machine, so no ReBAR setting is written."
            }
        }
    }
}

/// Inputs used to build the PUBG competitive NVIDIA profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaPubgCompetitiveProfileRequest {
    /// Primary display refresh rate when known.
    pub display_refresh_hz: Option<u16>,
    /// Whether G-SYNC/VRR is currently enabled for the target display path.
    pub vrr_enabled: bool,
    /// Whether PUBG's in-game NVIDIA Reflex path is supported and should be preferred.
    pub pubg_reflex_supported: bool,
    /// Read-only Resizable BAR state from GPU/platform detection.
    pub rebar_state: GpuCapabilityState,
    /// PUBG/BattlEye runtime state used to defer profile mutation.
    pub runtime_state: PubgRuntimeState,
}

impl NvidiaPubgCompetitiveProfileRequest {
    /// Creates a PUBG competitive NVIDIA profile request.
    #[must_use]
    pub fn new(
        display_refresh_hz: Option<u16>,
        vrr_enabled: bool,
        pubg_reflex_supported: bool,
        rebar_state: GpuCapabilityState,
        runtime_state: PubgRuntimeState,
    ) -> Self {
        Self {
            display_refresh_hz,
            vrr_enabled,
            pubg_reflex_supported,
            rebar_state,
            runtime_state,
        }
    }
}

/// Dry-run plan for the PUBG competitive NVIDIA profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaPubgCompetitiveProfilePlan {
    /// Driver application profile to create or update.
    pub profile: NvidiaProfile,
    /// FPS cap chosen for VRR, when applicable.
    pub fps_cap: Option<u16>,
    /// Reflex and driver Low Latency Mode policy.
    pub reflex_policy: NvidiaPubgReflexPolicy,
    /// G-SYNC/VRR and sync policy.
    pub vrr_policy: NvidiaPubgVrrPolicy,
    /// Resizable BAR policy.
    pub rebar_policy: NvidiaPubgRebarPolicy,
    /// Manual actions or explanations shown alongside the profile.
    pub manual_actions: Vec<String>,
}

/// Backing integration used to read NVIDIA driver profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvidiaProfileBridgeKind {
    /// Official NVIDIA NVAPI Driver Settings API path.
    NvapiDriverSettings,
    /// NVIDIA Profile Inspector import/export compatibility path.
    NvidiaProfileInspectorCompatibility(NpiCompatibilityValidation),
}

impl NvidiaProfileBridgeKind {
    fn stable_key(&self) -> &'static str {
        match self {
            Self::NvapiDriverSettings => "nvapi-driver-settings",
            Self::NvidiaProfileInspectorCompatibility(_) => "npi-compatibility",
        }
    }
}

/// Evidence that an NPI compatibility bridge is safe to use for backup/readback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpiCompatibilityValidation {
    /// NPI tool version used by the compatibility bridge.
    pub tool_version: String,
    /// NVIDIA driver version that completed the import/export round trip.
    pub driver_version: String,
    /// Whether a structured export/import/readback round trip was verified.
    pub import_export_roundtrip: bool,
}

impl NpiCompatibilityValidation {
    /// Creates a validation record for a checked NPI compatibility path.
    #[must_use]
    pub fn validated(tool_version: impl Into<String>, driver_version: impl Into<String>) -> Self {
        Self {
            tool_version: tool_version.into(),
            driver_version: driver_version.into(),
            import_export_roundtrip: true,
        }
    }

    /// Creates a validation record for an NPI path that has not passed round-trip checks.
    #[must_use]
    pub fn unvalidated(tool_version: impl Into<String>, driver_version: impl Into<String>) -> Self {
        Self {
            tool_version: tool_version.into(),
            driver_version: driver_version.into(),
            import_export_roundtrip: false,
        }
    }

    fn is_validated_for(&self, detection: &NvidiaDriverDetection) -> bool {
        self.import_export_roundtrip
            && !self.tool_version.trim().is_empty()
            && detection
                .vendor
                .driver_versions
                .iter()
                .any(|version| version.as_str() == self.driver_version.trim())
    }
}

/// NVIDIA driver profile scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NvidiaProfileScope {
    /// Driver global profile.
    Global,
    /// Per-application driver profile.
    Application,
}

impl NvidiaProfileScope {
    fn stable_key(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Application => "application",
        }
    }
}

/// Visibility/risk class for a profile setting captured from readback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NvidiaProfileSettingVisibility {
    /// Officially documented setting.
    Documented,
    /// User-visible driver control panel or NVIDIA App setting.
    UserVisible,
    /// Hidden setting captured only to make rollback exact.
    Hidden,
}

impl NvidiaProfileSettingVisibility {
    fn stable_key(self) -> &'static str {
        match self {
            Self::Documented => "documented",
            Self::UserVisible => "user-visible",
            Self::Hidden => "hidden",
        }
    }
}

/// One NVIDIA profile setting value read from the driver profile store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaProfileSetting {
    /// Stable setting identifier from NVAPI/Driver Settings or validated NPI mapping.
    pub id: String,
    /// Human-readable setting name.
    pub name: String,
    /// Current driver value captured for rollback.
    pub value: String,
    /// Whether the setting is documented, user-visible, or hidden.
    pub visibility: NvidiaProfileSettingVisibility,
}

impl NvidiaProfileSetting {
    /// Creates a structured profile setting readback value.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        value: impl Into<String>,
        visibility: NvidiaProfileSettingVisibility,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            value: value.into(),
            visibility,
        }
    }
}

/// One NVIDIA global or application profile captured by readback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaProfile {
    /// Profile name as reported by the driver profile API.
    pub name: String,
    /// Profile scope.
    pub scope: NvidiaProfileScope,
    /// Executables associated with an application profile.
    pub applications: Vec<String>,
    /// Profile setting values captured for rollback.
    pub settings: Vec<NvidiaProfileSetting>,
}

impl NvidiaProfile {
    /// Creates a global NVIDIA profile readback record.
    #[must_use]
    pub fn global(name: impl Into<String>, settings: Vec<NvidiaProfileSetting>) -> Self {
        Self {
            name: name.into(),
            scope: NvidiaProfileScope::Global,
            applications: Vec::new(),
            settings,
        }
    }

    /// Creates an application NVIDIA profile readback record.
    #[must_use]
    pub fn application(
        name: impl Into<String>,
        applications: Vec<String>,
        settings: Vec<NvidiaProfileSetting>,
    ) -> Self {
        Self {
            name: name.into(),
            scope: NvidiaProfileScope::Application,
            applications,
            settings,
        }
    }

    /// Returns true when this profile has driver settings worth backing up.
    #[must_use]
    pub fn is_customized(&self) -> bool {
        !self.settings.is_empty()
    }
}

/// Request flags for selecting which readback profiles become a rollback backup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NvidiaProfileBackupRequest {
    /// Capture the global profile.
    pub include_global_profile: bool,
    /// Capture per-application profiles.
    pub include_application_profiles: bool,
    /// Capture only profiles that have explicit settings.
    pub customized_only: bool,
}

impl NvidiaProfileBackupRequest {
    /// Captures global and application profiles before a profile mutation.
    #[must_use]
    pub const fn all_profiles_before_mutation() -> Self {
        Self {
            include_global_profile: true,
            include_application_profiles: true,
            customized_only: false,
        }
    }

    /// Captures customized global and application profiles before a profile mutation.
    #[must_use]
    pub const fn customized_profiles_before_mutation() -> Self {
        Self {
            include_global_profile: true,
            include_application_profiles: true,
            customized_only: true,
        }
    }

    fn includes(self, profile: &NvidiaProfile) -> bool {
        let scope_matches = match profile.scope {
            NvidiaProfileScope::Global => self.include_global_profile,
            NvidiaProfileScope::Application => self.include_application_profiles,
        };

        scope_matches && (!self.customized_only || profile.is_customized())
    }
}

/// Structured profile readback snapshot from one bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaProfileSnapshot {
    /// Bridge used to read the profile store.
    pub bridge_kind: NvidiaProfileBridgeKind,
    /// NVIDIA driver versions present when the snapshot was captured.
    pub driver_versions: Vec<String>,
    /// Captured profile readback records.
    pub profiles: Vec<NvidiaProfile>,
}

/// Backup payload metadata for exact NVIDIA profile rollback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaProfileBackup {
    /// Tweak ID that produced the backup.
    pub tweak_id: &'static str,
    /// Captured profile snapshot.
    pub snapshot: NvidiaProfileSnapshot,
    /// Stable non-cryptographic fingerprint for integrity checks in tests/audit logs.
    pub fingerprint: String,
    /// Rollback strategy attached to this backup shape.
    pub rollback_kind: &'static str,
}

/// Result of restoring NVIDIA profiles from a captured rollback backup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaProfileRollbackResult {
    /// Tweak ID that produced this rollback mutation.
    pub tweak_id: &'static str,
    /// Fingerprint of the backup used for rollback.
    pub backup_fingerprint: String,
    /// Profiles restored from the backup payload.
    pub restored_profiles: Vec<NvidiaProfile>,
    /// Liiiraa-owned profiles deleted because they did not exist in the backup.
    pub deleted_profiles: Vec<NvidiaProfile>,
    /// Profile store snapshot observed after rollback verification.
    pub verified_snapshot: NvidiaProfileSnapshot,
}

/// Failure while reading or backing up NVIDIA driver profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NvidiaProfileError {
    /// NVIDIA hardware was not detected.
    NoNvidiaAdapter,
    /// The requested profile bridge is not ready on this machine.
    ProfileBridgeUnavailable {
        /// Requested bridge kind.
        kind: NvidiaProfileBridgeKind,
        /// Detection state that blocked the bridge.
        state: GpuCapabilityState,
    },
    /// NPI compatibility was requested without a validated round trip.
    UnvalidatedProfileInspector {
        /// Validation failure reason.
        reason: String,
    },
    /// The bridge failed while reading profiles.
    BridgeReadFailed {
        /// Error message from the bridge implementation.
        message: String,
    },
    /// The bridge failed while writing a profile.
    ProfileWriteFailed {
        /// Error message from the bridge implementation.
        message: String,
    },
    /// The bridge failed while deleting a created profile during rollback.
    ProfileDeleteFailed {
        /// Error message from the bridge implementation.
        message: String,
    },
    /// Profile readback returned an invalid shape.
    InvalidProfile {
        /// Profile name when available.
        profile: String,
        /// Validation failure reason.
        reason: String,
    },
    /// A requested global profile setting is outside the conservative allowlist.
    UnsafeGlobalProfileSetting {
        /// Setting ID that failed validation.
        setting_id: String,
        /// Validation failure reason.
        reason: String,
    },
    /// A requested PUBG profile setting violates competitive profile policy.
    UnsafePubgProfileSetting {
        /// Setting ID that failed validation.
        setting_id: String,
        /// Validation failure reason.
        reason: String,
    },
    /// The expected Liiiraa global profile was not visible during readback.
    GlobalProfileReadbackMissing {
        /// Expected global profile name.
        profile: String,
    },
    /// The expected Liiiraa PUBG profile was not visible during readback.
    PubgProfileReadbackMissing {
        /// Expected application profile name.
        profile: String,
    },
    /// PUBG or BattlEye is running, so profile mutation must be deferred.
    PubgOrBattleyeRunning {
        /// Blocking process names.
        processes: Vec<String>,
    },
    /// The backup request filtered out all readback profiles.
    NoProfilesSelected,
    /// The backup fingerprint did not match the profile payload.
    BackupIntegrityMismatch {
        /// Fingerprint stored with the backup.
        expected: String,
        /// Fingerprint calculated from the backup payload.
        actual: String,
    },
    /// The rollback bridge cannot safely restore this backup shape.
    RollbackBridgeMismatch {
        /// Bridge key stored in the backup.
        backup_bridge: String,
        /// Bridge key selected for rollback.
        rollback_bridge: String,
    },
    /// Post-rollback readback did not match the expected restored profile state.
    RollbackVerificationFailed {
        /// Profile that failed verification.
        profile: String,
        /// Verification failure reason.
        reason: String,
    },
}

impl fmt::Display for NvidiaProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoNvidiaAdapter => formatter.write_str("no NVIDIA adapter was detected"),
            Self::ProfileBridgeUnavailable { kind, state } => {
                write!(formatter, "{kind:?} is not ready for profile readback: {state:?}")
            }
            Self::UnvalidatedProfileInspector { reason } => {
                write!(formatter, "NPI compatibility is not validated: {reason}")
            }
            Self::BridgeReadFailed { message } => {
                write!(formatter, "NVIDIA profile bridge read failed: {message}")
            }
            Self::ProfileWriteFailed { message } => {
                write!(formatter, "NVIDIA profile bridge write failed: {message}")
            }
            Self::ProfileDeleteFailed { message } => {
                write!(formatter, "NVIDIA profile bridge delete failed: {message}")
            }
            Self::InvalidProfile { profile, reason } => {
                write!(formatter, "invalid NVIDIA profile {profile:?}: {reason}")
            }
            Self::UnsafeGlobalProfileSetting { setting_id, reason } => {
                write!(
                    formatter,
                    "unsafe global NVIDIA profile setting {setting_id:?}: {reason}"
                )
            }
            Self::UnsafePubgProfileSetting { setting_id, reason } => {
                write!(
                    formatter,
                    "unsafe PUBG NVIDIA profile setting {setting_id:?}: {reason}"
                )
            }
            Self::GlobalProfileReadbackMissing { profile } => {
                write!(
                    formatter,
                    "global NVIDIA profile {profile:?} was not found during readback"
                )
            }
            Self::PubgProfileReadbackMissing { profile } => {
                write!(
                    formatter,
                    "PUBG NVIDIA profile {profile:?} was not found during readback"
                )
            }
            Self::PubgOrBattleyeRunning { processes } => {
                write!(
                    formatter,
                    "PUBG or BattlEye is running; defer NVIDIA profile mutation until these processes close: {}",
                    processes.join(", ")
                )
            }
            Self::NoProfilesSelected => {
                formatter.write_str("backup request did not select any NVIDIA profiles")
            }
            Self::BackupIntegrityMismatch { expected, actual } => {
                write!(
                    formatter,
                    "NVIDIA profile backup fingerprint mismatch: expected {expected}, got {actual}"
                )
            }
            Self::RollbackBridgeMismatch {
                backup_bridge,
                rollback_bridge,
            } => {
                write!(
                    formatter,
                    "NVIDIA profile backup bridge {backup_bridge:?} cannot be restored through {rollback_bridge:?}"
                )
            }
            Self::RollbackVerificationFailed { profile, reason } => {
                write!(
                    formatter,
                    "NVIDIA profile rollback verification failed for {profile:?}: {reason}"
                )
            }
        }
    }
}

impl std::error::Error for NvidiaProfileError {}

/// Read-only bridge for NVIDIA profile backup/readback.
pub trait NvidiaProfileBridge {
    /// Returns the bridge implementation kind.
    fn kind(&self) -> NvidiaProfileBridgeKind;

    /// Reads the current driver profile store into structured profile records.
    fn read_profiles(&self) -> Result<Vec<NvidiaProfile>, NvidiaProfileError>;
}

/// Bridge for applying NVIDIA profile mutations after backup.
pub trait NvidiaProfileWriteBridge: NvidiaProfileBridge {
    /// Writes or replaces a global NVIDIA profile.
    fn write_global_profile(
        &mut self,
        profile: NvidiaProfile,
    ) -> Result<(), NvidiaProfileError>;

    /// Writes or replaces an application NVIDIA profile.
    fn write_application_profile(
        &mut self,
        profile: NvidiaProfile,
    ) -> Result<(), NvidiaProfileError>;
}

/// Bridge for restoring NVIDIA profile backups and removing created Liiiraa profiles.
pub trait NvidiaProfileRollbackBridge: NvidiaProfileWriteBridge {
    /// Deletes a global or application profile that did not exist in the backup.
    fn delete_profile(&mut self, profile: NvidiaProfile) -> Result<(), NvidiaProfileError>;
}

/// Result of applying and verifying the conservative Liiiraa global profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaGlobalProfileApplyResult {
    /// Tweak ID that produced this profile mutation.
    pub tweak_id: &'static str,
    /// Profile requested for application.
    pub requested_profile: NvidiaProfile,
    /// Rollback backup captured before mutation.
    pub backup: NvidiaProfileBackup,
    /// Profile as observed through post-apply readback.
    pub verified_profile: NvidiaProfile,
}

/// Result of applying and verifying the PUBG competitive NVIDIA profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaPubgProfileApplyResult {
    /// Tweak ID that produced this profile mutation.
    pub tweak_id: &'static str,
    /// Profile plan requested for application.
    pub requested_plan: NvidiaPubgCompetitiveProfilePlan,
    /// Rollback backup captured before mutation.
    pub backup: NvidiaProfileBackup,
    /// Profile as observed through post-apply readback.
    pub verified_profile: NvidiaProfile,
}

/// Reads NVIDIA profiles through a validated bridge without creating a backup payload.
///
/// This is the verification/readback half used after profile apply steps.
pub fn readback_profiles<B: NvidiaProfileBridge>(
    detection: &NvidiaDriverDetection,
    bridge: &B,
) -> Result<NvidiaProfileSnapshot, NvidiaProfileError> {
    let bridge_kind = bridge.kind();
    validate_bridge(detection, &bridge_kind)?;

    let profiles = canonicalize_profiles(bridge.read_profiles()?);
    for profile in &profiles {
        validate_profile(profile)?;
    }

    Ok(NvidiaProfileSnapshot {
        bridge_kind,
        driver_versions: detection.vendor.driver_versions.clone(),
        profiles,
    })
}

/// Captures NVIDIA profile readback as a rollback backup before mutation.
pub fn backup_profiles<B: NvidiaProfileBridge>(
    detection: &NvidiaDriverDetection,
    bridge: &B,
    request: NvidiaProfileBackupRequest,
) -> Result<NvidiaProfileBackup, NvidiaProfileError> {
    let mut snapshot = readback_profiles(detection, bridge)?;
    snapshot.profiles.retain(|profile| request.includes(profile));

    if snapshot.profiles.is_empty() {
        return Err(NvidiaProfileError::NoProfilesSelected);
    }

    let fingerprint = stable_profile_fingerprint(&snapshot);

    Ok(NvidiaProfileBackup {
        tweak_id: NVIDIA_PROFILE_BACKUP_TWEAK_ID,
        snapshot,
        fingerprint,
        rollback_kind: "restore-profile-export",
    })
}

/// Returns the approved conservative settings for the global Liiiraa profile.
#[must_use]
pub fn approved_global_performance_settings() -> Vec<NvidiaProfileSetting> {
    APPROVED_GLOBAL_PROFILE_SETTINGS
        .iter()
        .map(|setting| NvidiaProfileSetting {
            id: setting.0.to_owned(),
            name: setting.1.to_owned(),
            value: setting.2.to_owned(),
            visibility: setting.3,
        })
        .collect()
}

/// Builds the conservative global NVIDIA profile owned by Liiiraa.
#[must_use]
pub fn global_performance_profile() -> NvidiaProfile {
    NvidiaProfile::global(
        LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME,
        approved_global_performance_settings(),
    )
}

/// Returns the recommended VRR cap below refresh rate for competitive latency.
#[must_use]
pub const fn recommended_vrr_frame_cap(refresh_rate_hz: u16) -> u16 {
    if refresh_rate_hz >= 100 {
        refresh_rate_hz - 3
    } else if refresh_rate_hz > 2 {
        refresh_rate_hz - 2
    } else {
        1
    }
}

/// Builds a dry-run PUBG competitive NVIDIA profile plan.
pub fn plan_pubg_competitive_profile(
    request: &NvidiaPubgCompetitiveProfileRequest,
) -> Result<NvidiaPubgCompetitiveProfilePlan, NvidiaProfileError> {
    let reflex_policy = if request.pubg_reflex_supported {
        NvidiaPubgReflexPolicy::PreferInGameReflex
    } else {
        NvidiaPubgReflexPolicy::DriverLowLatencyOn
    };

    let vrr_policy = request
        .display_refresh_hz
        .filter(|_| request.vrr_enabled)
        .map(|refresh_rate_hz| NvidiaPubgVrrPolicy::VrrCap {
            refresh_rate_hz,
            cap_fps: recommended_vrr_frame_cap(refresh_rate_hz),
        })
        .unwrap_or(NvidiaPubgVrrPolicy::ApplicationControlled);
    let rebar_policy = NvidiaPubgRebarPolicy::from_capability(request.rebar_state);
    let fps_cap = match vrr_policy {
        NvidiaPubgVrrPolicy::VrrCap { cap_fps, .. } => Some(cap_fps),
        NvidiaPubgVrrPolicy::ApplicationControlled => None,
    };
    let profile = pubg_competitive_profile_from_policy(reflex_policy, vrr_policy);
    validate_pubg_competitive_profile(&profile)?;

    Ok(NvidiaPubgCompetitiveProfilePlan {
        profile,
        fps_cap,
        reflex_policy,
        vrr_policy,
        rebar_policy,
        manual_actions: vec![
            reflex_policy.note().to_owned(),
            vrr_policy.note(),
            rebar_policy.note().to_owned(),
            "Do not import hidden ReBAR compatibility bits or bulk NVIDIA Profile Inspector dumps."
                .to_owned(),
        ],
    })
}

/// Builds the PUBG competitive NVIDIA application profile for `TslGame.exe`.
pub fn pubg_competitive_profile(
    request: &NvidiaPubgCompetitiveProfileRequest,
) -> Result<NvidiaProfile, NvidiaProfileError> {
    Ok(plan_pubg_competitive_profile(request)?.profile)
}

fn pubg_competitive_profile_from_policy(
    reflex_policy: NvidiaPubgReflexPolicy,
    vrr_policy: NvidiaPubgVrrPolicy,
) -> NvidiaProfile {
    NvidiaProfile::application(
        LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME,
        vec![PUBG_EXECUTABLE_NAME.to_owned()],
        vec![
            NvidiaProfileSetting::new(
                "max-frame-rate",
                "Max Frame Rate",
                vrr_policy.max_frame_rate_value(),
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "low-latency-mode",
                "Low Latency Mode",
                reflex_policy.driver_low_latency_value(),
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "monitor-technology",
                "Monitor Technology",
                vrr_policy.monitor_technology_value(),
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "power-management-mode",
                "Power management mode",
                "Prefer maximum performance",
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "preferred-refresh-rate",
                "Preferred refresh rate",
                "Highest available",
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "shader-cache",
                "Shader Cache",
                "On",
                NvidiaProfileSettingVisibility::Documented,
            ),
            NvidiaProfileSetting::new(
                "texture-filtering-quality",
                "Texture filtering - Quality",
                "High performance",
                NvidiaProfileSettingVisibility::UserVisible,
            ),
            NvidiaProfileSetting::new(
                "vertical-sync",
                "Vertical sync",
                vrr_policy.vertical_sync_value(),
                NvidiaProfileSettingVisibility::UserVisible,
            ),
        ],
    )
}

/// Validates that a global profile contains only approved conservative settings.
pub fn validate_global_performance_profile(
    profile: &NvidiaProfile,
) -> Result<(), NvidiaProfileError> {
    validate_profile(profile)?;

    if profile.name != LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: format!(
                "expected global profile name {LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME:?}"
            ),
        });
    }

    if profile.scope != NvidiaProfileScope::Global {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: "Liiiraa global performance profile must use global scope".to_owned(),
        });
    }

    let mut seen_setting_ids = BTreeSet::new();
    for setting in &profile.settings {
        if !seen_setting_ids.insert(setting.id.as_str()) {
            return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
                setting_id: setting.id.clone(),
                reason: "duplicate settings make readback and rollback ambiguous".to_owned(),
            });
        }

        validate_global_performance_setting(setting)?;
    }

    for expected_setting in approved_global_performance_settings() {
        let expected_setting_id = expected_setting.id.as_str();
        if !profile
            .settings
            .iter()
            .any(|setting| setting.id == expected_setting_id)
        {
            return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
                setting_id: expected_setting.id,
                reason: "required conservative setting is missing".to_owned(),
            });
        }
    }

    Ok(())
}

/// Validates that the PUBG profile is scoped, visible, and policy-approved.
pub fn validate_pubg_competitive_profile(
    profile: &NvidiaProfile,
) -> Result<(), NvidiaProfileError> {
    validate_profile(profile)?;

    if profile.name != LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: format!(
                "expected PUBG profile name {LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME:?}"
            ),
        });
    }

    if profile.scope != NvidiaProfileScope::Application {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: "PUBG competitive profile must use application scope".to_owned(),
        });
    }

    if !profile
        .applications
        .iter()
        .any(|application| application.eq_ignore_ascii_case(PUBG_EXECUTABLE_NAME))
    {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: format!("PUBG profile must target {PUBG_EXECUTABLE_NAME}"),
        });
    }

    let mut seen_setting_ids = BTreeSet::new();
    for setting in &profile.settings {
        if !seen_setting_ids.insert(setting.id.as_str()) {
            return Err(NvidiaProfileError::UnsafePubgProfileSetting {
                setting_id: setting.id.clone(),
                reason: "duplicate settings make readback and rollback ambiguous".to_owned(),
            });
        }

        validate_pubg_competitive_setting(setting)?;
    }

    for expected_setting_id in REQUIRED_PUBG_COMPETITIVE_SETTING_IDS {
        if !profile
            .settings
            .iter()
            .any(|setting| setting.id == *expected_setting_id)
        {
            return Err(NvidiaProfileError::UnsafePubgProfileSetting {
                setting_id: (*expected_setting_id).to_owned(),
                reason: "required competitive setting is missing".to_owned(),
            });
        }
    }

    Ok(())
}

/// Verifies that post-apply profile readback contains the exact Liiiraa global profile.
pub fn verify_global_performance_profile_readback(
    snapshot: &NvidiaProfileSnapshot,
) -> Result<NvidiaProfile, NvidiaProfileError> {
    let profile = snapshot
        .profiles
        .iter()
        .find(|profile| {
            profile.scope == NvidiaProfileScope::Global
                && profile.name == LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
        })
        .ok_or_else(|| NvidiaProfileError::GlobalProfileReadbackMissing {
            profile: LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME.to_owned(),
        })?;

    validate_global_performance_profile(profile)?;

    Ok(profile.clone())
}

/// Verifies that post-apply profile readback contains the exact Liiiraa PUBG profile.
pub fn verify_pubg_competitive_profile_readback(
    snapshot: &NvidiaProfileSnapshot,
) -> Result<NvidiaProfile, NvidiaProfileError> {
    let profile = snapshot
        .profiles
        .iter()
        .find(|profile| {
            profile.scope == NvidiaProfileScope::Application
                && profile.name == LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME
                && profile
                    .applications
                    .iter()
                    .any(|application| application.eq_ignore_ascii_case(PUBG_EXECUTABLE_NAME))
        })
        .ok_or_else(|| NvidiaProfileError::PubgProfileReadbackMissing {
            profile: LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME.to_owned(),
        })?;

    validate_pubg_competitive_profile(profile)?;

    Ok(profile.clone())
}

/// Backs up current profiles, applies the global profile, and verifies readback.
pub fn apply_global_performance_profile<B: NvidiaProfileWriteBridge>(
    detection: &NvidiaDriverDetection,
    bridge: &mut B,
) -> Result<NvidiaGlobalProfileApplyResult, NvidiaProfileError> {
    let backup = backup_profiles(
        detection,
        &*bridge,
        NvidiaProfileBackupRequest::all_profiles_before_mutation(),
    )?;
    let requested_profile = global_performance_profile();
    validate_global_performance_profile(&requested_profile)?;

    bridge.write_global_profile(requested_profile.clone())?;

    let snapshot = readback_profiles(detection, &*bridge)?;
    let verified_profile = verify_global_performance_profile_readback(&snapshot)?;

    Ok(NvidiaGlobalProfileApplyResult {
        tweak_id: NVIDIA_GLOBAL_PROFILE_TWEAK_ID,
        requested_profile,
        backup,
        verified_profile,
    })
}

/// Backs up current profiles, applies the PUBG profile, and verifies readback.
pub fn apply_pubg_competitive_profile<B: NvidiaProfileWriteBridge>(
    detection: &NvidiaDriverDetection,
    bridge: &mut B,
    request: &NvidiaPubgCompetitiveProfileRequest,
) -> Result<NvidiaPubgProfileApplyResult, NvidiaProfileError> {
    let blocking_processes = request.runtime_state.blocking_profile_mutation_processes();
    if !blocking_processes.is_empty() {
        return Err(NvidiaProfileError::PubgOrBattleyeRunning {
            processes: blocking_processes,
        });
    }

    let backup = backup_profiles(
        detection,
        &*bridge,
        NvidiaProfileBackupRequest::all_profiles_before_mutation(),
    )?;
    let requested_plan = plan_pubg_competitive_profile(request)?;

    bridge.write_application_profile(requested_plan.profile.clone())?;

    let snapshot = readback_profiles(detection, &*bridge)?;
    let verified_profile = verify_pubg_competitive_profile_readback(&snapshot)?;

    Ok(NvidiaPubgProfileApplyResult {
        tweak_id: NVIDIA_PUBG_PROFILE_TWEAK_ID,
        requested_plan,
        backup,
        verified_profile,
    })
}

/// Restores NVIDIA profiles from a rollback backup and verifies post-rollback readback.
pub fn rollback_profiles_from_backup<B: NvidiaProfileRollbackBridge>(
    detection: &NvidiaDriverDetection,
    bridge: &mut B,
    backup: &NvidiaProfileBackup,
) -> Result<NvidiaProfileRollbackResult, NvidiaProfileError> {
    validate_profile_backup(backup)?;

    let rollback_bridge = bridge.kind();
    validate_bridge(detection, &rollback_bridge)?;
    if rollback_bridge.stable_key() != backup.snapshot.bridge_kind.stable_key() {
        return Err(NvidiaProfileError::RollbackBridgeMismatch {
            backup_bridge: backup.snapshot.bridge_kind.stable_key().to_owned(),
            rollback_bridge: rollback_bridge.stable_key().to_owned(),
        });
    }

    let restored_profiles = backup.snapshot.profiles.clone();
    for profile in &restored_profiles {
        match profile.scope {
            NvidiaProfileScope::Global => bridge.write_global_profile(profile.clone())?,
            NvidiaProfileScope::Application => bridge.write_application_profile(profile.clone())?,
        }
    }

    let snapshot_after_restore = readback_profiles(detection, &*bridge)?;
    let mut deleted_profiles = Vec::new();
    for profile in owned_liiiraa_profiles_missing_from_backup(&backup.snapshot) {
        if snapshot_after_restore
            .profiles
            .iter()
            .any(|current| same_profile_identity(current, &profile))
        {
            bridge.delete_profile(profile.clone())?;
            deleted_profiles.push(profile);
        }
    }

    let verified_snapshot = readback_profiles(detection, &*bridge)?;
    verify_rollback_snapshot(&backup.snapshot, &verified_snapshot)?;

    Ok(NvidiaProfileRollbackResult {
        tweak_id: NVIDIA_PROFILE_ROLLBACK_TWEAK_ID,
        backup_fingerprint: backup.fingerprint.clone(),
        restored_profiles,
        deleted_profiles,
        verified_snapshot,
    })
}

/// Produces a stable non-cryptographic fingerprint for one profile snapshot.
#[must_use]
pub fn stable_profile_fingerprint(snapshot: &NvidiaProfileSnapshot) -> String {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    hash_text(&mut hash, snapshot.bridge_kind.stable_key(), FNV_PRIME);
    for version in &snapshot.driver_versions {
        hash_text(&mut hash, version, FNV_PRIME);
    }

    for profile in &snapshot.profiles {
        hash_text(&mut hash, profile.scope.stable_key(), FNV_PRIME);
        hash_text(&mut hash, &profile.name, FNV_PRIME);
        for application in &profile.applications {
            hash_text(&mut hash, application, FNV_PRIME);
        }
        for setting in &profile.settings {
            hash_text(&mut hash, &setting.id, FNV_PRIME);
            hash_text(&mut hash, &setting.name, FNV_PRIME);
            hash_text(&mut hash, &setting.value, FNV_PRIME);
            hash_text(&mut hash, setting.visibility.stable_key(), FNV_PRIME);
        }
    }

    format!("{hash:016x}")
}

fn validate_profile_backup(backup: &NvidiaProfileBackup) -> Result<(), NvidiaProfileError> {
    if backup.snapshot.profiles.is_empty() {
        return Err(NvidiaProfileError::NoProfilesSelected);
    }

    for profile in &backup.snapshot.profiles {
        validate_profile(profile)?;
    }

    let actual = stable_profile_fingerprint(&backup.snapshot);
    if actual != backup.fingerprint {
        return Err(NvidiaProfileError::BackupIntegrityMismatch {
            expected: backup.fingerprint.clone(),
            actual,
        });
    }

    Ok(())
}

fn hash_text(hash: &mut u64, value: &str, prime: u64) {
    for byte in value.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(prime);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(prime);
}

fn validate_bridge(
    detection: &NvidiaDriverDetection,
    bridge_kind: &NvidiaProfileBridgeKind,
) -> Result<(), NvidiaProfileError> {
    if !detection.is_available() {
        return Err(NvidiaProfileError::NoNvidiaAdapter);
    }

    match bridge_kind {
        NvidiaProfileBridgeKind::NvapiDriverSettings => {
            if detection.profile_api_state == GpuCapabilityState::Ready {
                Ok(())
            } else {
                Err(NvidiaProfileError::ProfileBridgeUnavailable {
                    kind: bridge_kind.clone(),
                    state: detection.profile_api_state,
                })
            }
        }
        NvidiaProfileBridgeKind::NvidiaProfileInspectorCompatibility(validation) => {
            if detection.profile_inspector_state != GpuCapabilityState::Ready {
                return Err(NvidiaProfileError::ProfileBridgeUnavailable {
                    kind: bridge_kind.clone(),
                    state: detection.profile_inspector_state,
                });
            }

            if validation.is_validated_for(detection) {
                Ok(())
            } else {
                Err(NvidiaProfileError::UnvalidatedProfileInspector {
                    reason: "import/export round trip must match the detected driver version"
                        .to_owned(),
                })
            }
        }
    }
}

fn canonicalize_profiles(profiles: Vec<NvidiaProfile>) -> Vec<NvidiaProfile> {
    let mut profiles = profiles
        .into_iter()
        .map(|mut profile| {
            profile.name = profile.name.trim().to_owned();
            profile.applications = normalized_strings(profile.applications);
            profile.settings = canonicalize_settings(profile.settings);
            profile
        })
        .collect::<Vec<_>>();

    profiles.sort_by(|left, right| {
        left.scope
            .cmp(&right.scope)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.applications.cmp(&right.applications))
    });

    profiles
}

fn canonicalize_settings(settings: Vec<NvidiaProfileSetting>) -> Vec<NvidiaProfileSetting> {
    let mut settings = settings
        .into_iter()
        .map(|mut setting| {
            setting.id = setting.id.trim().to_owned();
            setting.name = setting.name.trim().to_owned();
            setting.value = setting.value.trim().to_owned();
            setting
        })
        .collect::<Vec<_>>();

    settings.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.value.cmp(&right.value))
            .then_with(|| left.visibility.cmp(&right.visibility))
    });

    settings
}

fn normalized_strings(values: Vec<String>) -> Vec<String> {
    let mut values = values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn owned_liiiraa_profiles_missing_from_backup(
    snapshot: &NvidiaProfileSnapshot,
) -> Vec<NvidiaProfile> {
    liiiraa_owned_profile_identities()
        .into_iter()
        .filter(|profile| {
            !snapshot
                .profiles
                .iter()
                .any(|backup_profile| same_profile_identity(backup_profile, profile))
        })
        .collect()
}

fn liiiraa_owned_profile_identities() -> Vec<NvidiaProfile> {
    vec![
        NvidiaProfile::global(LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME, Vec::new()),
        NvidiaProfile::application(
            LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME,
            vec![PUBG_EXECUTABLE_NAME.to_owned()],
            Vec::new(),
        ),
    ]
}

fn same_profile_identity(left: &NvidiaProfile, right: &NvidiaProfile) -> bool {
    left.scope == right.scope && left.name == right.name
}

fn profile_identity_label(profile: &NvidiaProfile) -> String {
    match profile.scope {
        NvidiaProfileScope::Global => format!("global:{}", profile.name),
        NvidiaProfileScope::Application => {
            format!("application:{}:{}", profile.name, profile.applications.join(","))
        }
    }
}

fn verify_rollback_snapshot(
    expected: &NvidiaProfileSnapshot,
    actual: &NvidiaProfileSnapshot,
) -> Result<(), NvidiaProfileError> {
    for expected_profile in &expected.profiles {
        let Some(actual_profile) = actual
            .profiles
            .iter()
            .find(|profile| same_profile_identity(profile, expected_profile))
        else {
            return Err(NvidiaProfileError::RollbackVerificationFailed {
                profile: profile_identity_label(expected_profile),
                reason: "restored profile was not present in readback".to_owned(),
            });
        };

        if actual_profile != expected_profile {
            return Err(NvidiaProfileError::RollbackVerificationFailed {
                profile: profile_identity_label(expected_profile),
                reason: "restored profile values did not match the backup".to_owned(),
            });
        }
    }

    for deleted_profile in owned_liiiraa_profiles_missing_from_backup(expected) {
        if actual
            .profiles
            .iter()
            .any(|profile| same_profile_identity(profile, &deleted_profile))
        {
            return Err(NvidiaProfileError::RollbackVerificationFailed {
                profile: profile_identity_label(&deleted_profile),
                reason: "created Liiiraa profile still exists after rollback".to_owned(),
            });
        }
    }

    Ok(())
}

fn validate_profile(profile: &NvidiaProfile) -> Result<(), NvidiaProfileError> {
    if profile.name.is_empty() {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: String::new(),
            reason: "profile name is required".to_owned(),
        });
    }

    match profile.scope {
        NvidiaProfileScope::Global if !profile.applications.is_empty() => {
            return Err(NvidiaProfileError::InvalidProfile {
                profile: profile.name.clone(),
                reason: "global profiles cannot have executable bindings".to_owned(),
            });
        }
        NvidiaProfileScope::Application if profile.applications.is_empty() => {
            return Err(NvidiaProfileError::InvalidProfile {
                profile: profile.name.clone(),
                reason: "application profiles require at least one executable".to_owned(),
            });
        }
        _ => {}
    }

    if let Some(setting) = profile
        .settings
        .iter()
        .find(|setting| setting.id.is_empty() || setting.value.is_empty())
    {
        return Err(NvidiaProfileError::InvalidProfile {
            profile: profile.name.clone(),
            reason: format!("setting {:?} requires id and value", &setting.name),
        });
    }

    Ok(())
}

fn validate_global_performance_setting(
    setting: &NvidiaProfileSetting,
) -> Result<(), NvidiaProfileError> {
    if setting.visibility == NvidiaProfileSettingVisibility::Hidden {
        return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
            setting_id: setting.id.clone(),
            reason: "hidden or undocumented settings are Lab-only, not global defaults"
                .to_owned(),
        });
    }

    let Some((expected_value, expected_visibility)) = approved_global_setting(setting.id.as_str())
    else {
        return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
            setting_id: setting.id.clone(),
            reason: "setting is not part of the conservative global allowlist".to_owned(),
        });
    };

    if setting.value != expected_value {
        return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
            setting_id: setting.id.clone(),
            reason: format!("expected conservative value {expected_value:?}"),
        });
    }

    if setting.visibility != expected_visibility {
        return Err(NvidiaProfileError::UnsafeGlobalProfileSetting {
            setting_id: setting.id.clone(),
            reason: format!(
                "expected {} visibility",
                expected_visibility.stable_key()
            ),
        });
    }

    Ok(())
}

fn validate_pubg_competitive_setting(
    setting: &NvidiaProfileSetting,
) -> Result<(), NvidiaProfileError> {
    if setting.visibility == NvidiaProfileSettingVisibility::Hidden {
        return Err(NvidiaProfileError::UnsafePubgProfileSetting {
            setting_id: setting.id.clone(),
            reason: "hidden or undocumented settings are Lab-only, not PUBG competitive defaults"
                .to_owned(),
        });
    }

    let valid = match setting.id.as_str() {
        "max-frame-rate" => valid_pubg_fps_cap(setting.value.as_str()),
        "low-latency-mode" => matches!(setting.value.as_str(), "Off" | "On"),
        "monitor-technology" => {
            matches!(
                setting.value.as_str(),
                "G-SYNC Compatible" | "Use global setting"
            )
        }
        "power-management-mode" => setting.value == "Prefer maximum performance",
        "preferred-refresh-rate" => setting.value == "Highest available",
        "shader-cache" => setting.value == "On",
        "texture-filtering-quality" => setting.value == "High performance",
        "vertical-sync" => {
            matches!(
                setting.value.as_str(),
                "On" | "Use the 3D application setting"
            )
        }
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(NvidiaProfileError::UnsafePubgProfileSetting {
            setting_id: setting.id.clone(),
            reason: "setting is not part of the approved PUBG competitive policy".to_owned(),
        })
    }
}

fn valid_pubg_fps_cap(value: &str) -> bool {
    if value == "Off" {
        return true;
    }

    value
        .strip_suffix(" FPS")
        .and_then(|cap| cap.parse::<u16>().ok())
        .is_some_and(|cap| (30..=1000).contains(&cap))
}

fn approved_global_setting(
    setting_id: &str,
) -> Option<(&'static str, NvidiaProfileSettingVisibility)> {
    APPROVED_GLOBAL_PROFILE_SETTINGS
        .iter()
        .find(|setting| setting.0 == setting_id)
        .map(|setting| (setting.2, setting.3))
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

/// NVIDIA crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "nvidia",
    responsibility: "manage NVIDIA profile backup, compatibility import, apply, verify, and rollback",
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

    #[derive(Debug, Clone)]
    struct FixtureProfileBridge {
        kind: NvidiaProfileBridgeKind,
        profiles: Vec<NvidiaProfile>,
        writes: Vec<NvidiaProfile>,
        deletes: Vec<NvidiaProfile>,
        tampered_write: Option<(String, String)>,
    }

    impl FixtureProfileBridge {
        fn new(kind: NvidiaProfileBridgeKind, profiles: Vec<NvidiaProfile>) -> Self {
            Self {
                kind,
                profiles,
                writes: Vec::new(),
                deletes: Vec::new(),
                tampered_write: None,
            }
        }

        fn with_tampered_write(
            kind: NvidiaProfileBridgeKind,
            profiles: Vec<NvidiaProfile>,
            setting_id: &str,
            value: &str,
        ) -> Self {
            Self {
                kind,
                profiles,
                writes: Vec::new(),
                deletes: Vec::new(),
                tampered_write: Some((setting_id.to_owned(), value.to_owned())),
            }
        }
    }

    impl NvidiaProfileBridge for FixtureProfileBridge {
        fn kind(&self) -> NvidiaProfileBridgeKind {
            self.kind.clone()
        }

        fn read_profiles(&self) -> Result<Vec<NvidiaProfile>, NvidiaProfileError> {
            Ok(self.profiles.clone())
        }
    }

    impl NvidiaProfileWriteBridge for FixtureProfileBridge {
        fn write_global_profile(
            &mut self,
            mut profile: NvidiaProfile,
        ) -> Result<(), NvidiaProfileError> {
            validate_profile(&profile)?;
            if profile.scope != NvidiaProfileScope::Global {
                return Err(NvidiaProfileError::InvalidProfile {
                    profile: profile.name,
                    reason: "write_global_profile requires a global profile".to_owned(),
                });
            }

            self.writes.push(profile.clone());

            if let Some((setting_id, value)) = &self.tampered_write {
                if let Some(setting) = profile
                    .settings
                    .iter_mut()
                    .find(|setting| setting.id == setting_id.as_str())
                {
                    setting.value = value.clone();
                }
            }

            self.profiles.retain(|existing| {
                existing.scope != NvidiaProfileScope::Global
                    || existing.name != profile.name.as_str()
            });
            self.profiles.push(profile);

            Ok(())
        }

        fn write_application_profile(
            &mut self,
            mut profile: NvidiaProfile,
        ) -> Result<(), NvidiaProfileError> {
            validate_profile(&profile)?;
            if profile.scope != NvidiaProfileScope::Application {
                return Err(NvidiaProfileError::InvalidProfile {
                    profile: profile.name,
                    reason: "write_application_profile requires an application profile"
                        .to_owned(),
                });
            }

            self.writes.push(profile.clone());

            if let Some((setting_id, value)) = &self.tampered_write {
                if let Some(setting) = profile
                    .settings
                    .iter_mut()
                    .find(|setting| setting.id == setting_id.as_str())
                {
                    setting.value = value.clone();
                }
            }

            self.profiles.retain(|existing| {
                existing.scope != NvidiaProfileScope::Application
                    || existing.name != profile.name.as_str()
            });
            self.profiles.push(profile);

            Ok(())
        }
    }

    impl NvidiaProfileRollbackBridge for FixtureProfileBridge {
        fn delete_profile(&mut self, profile: NvidiaProfile) -> Result<(), NvidiaProfileError> {
            validate_profile(&profile)?;
            self.deletes.push(profile.clone());
            self.profiles
                .retain(|existing| !same_profile_identity(existing, &profile));

            Ok(())
        }
    }

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "nvidia");
        assert!(info.responsibility.contains("rollback"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn detects_nvidia_adapters_and_driver_versions() {
        let inventory = GpuInventory::new(vec![
            GpuAdapter::from_scan(
                "NVIDIA GeForce RTX 4070",
                Some("32.0.15.6094"),
                None,
                None,
                Some("PCI\\VEN_10DE&DEV_2786"),
            ),
            GpuAdapter::from_scan(
                "AMD Radeon RX 7800 XT",
                Some("31.0.24002.92"),
                None,
                None,
                Some("PCI\\VEN_1002&DEV_747E"),
            ),
        ]);

        let detection = NvidiaDriverDetection::from_inventory(&inventory);

        assert!(detection.is_available());
        assert!(detection.has_driver_version());
        assert_eq!(detection.vendor.adapters.len(), 1);
        assert_eq!(detection.vendor.driver_versions, vec!["32.0.15.6094"]);
        assert_eq!(detection.profile_api_state, GpuCapabilityState::Unknown);
    }

    #[test]
    fn marks_nvidia_capabilities_not_applicable_without_nvidia_gpu() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_56A0"),
        )]);

        let detection = NvidiaDriverDetection::from_inventory(&inventory);

        assert!(!detection.is_available());
        assert_eq!(detection.profile_api_state, GpuCapabilityState::NotApplicable);
        assert_eq!(
            detection.profile_inspector_state,
            GpuCapabilityState::NotApplicable
        );
    }

    #[test]
    fn nvidia_platform_plan_uses_shared_capability_policy() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let request = gpu::GpuPlatformCheckRequest::new(gpu::GpuPlatformIntent::CompetitiveLatency)
            .with_driver_age_days(30)
            .with_display(gpu::GpuDisplayPipelineState::new(
                Some(240),
                Some(240),
                GpuCapabilityState::Ready,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Ready)
            .with_frame_generation_state(GpuCapabilityState::Ready)
            .with_shader_cache(gpu::GpuShaderCacheState::enabled(None));

        let plan = plan_nvidia_platform_capabilities(&detection, &request);

        assert_eq!(plan.vendor, GpuVendor::Nvidia);
        assert_eq!(plan.rebar_sam.label, "Resizable BAR");
        assert_eq!(
            plan.frame_generation.policy,
            gpu::GpuFrameGenerationPolicy::KeepOffForCompetitive
        );
        assert_eq!(
            plan.frame_generation.decision,
            gpu::GpuPlatformDecision::KeepDisabled
        );
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == gpu::GpuPlatformCheck::FrameGeneration
        }));
    }

    #[test]
    fn profile_backup_captures_nvapi_readback_with_stable_fingerprint() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                pubg_profile_with_reversed_settings(),
                NvidiaProfile::global(
                    "Base Profile",
                    vec![NvidiaProfileSetting::new(
                        "power-management-mode",
                        "Power management mode",
                        "Prefer maximum performance",
                        NvidiaProfileSettingVisibility::UserVisible,
                    )],
                ),
            ],
        );

        let backup = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect("NVAPI readback should produce a backup");

        assert_eq!(backup.tweak_id, NVIDIA_PROFILE_BACKUP_TWEAK_ID);
        assert_eq!(backup.rollback_kind, "restore-profile-export");
        assert_eq!(backup.snapshot.profiles.len(), 2);
        assert_eq!(backup.snapshot.profiles[0].scope, NvidiaProfileScope::Global);
        assert_eq!(
            backup.snapshot.profiles[1].applications,
            vec!["TslGame.exe".to_owned()]
        );
        assert_eq!(
            backup.snapshot.profiles[1].settings[0].id,
            "low-latency-mode"
        );
        assert_eq!(backup.fingerprint.len(), 16);

        let reordered_bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                NvidiaProfile::global(
                    "Base Profile",
                    vec![NvidiaProfileSetting::new(
                        "power-management-mode",
                        "Power management mode",
                        "Prefer maximum performance",
                        NvidiaProfileSettingVisibility::UserVisible,
                    )],
                ),
                pubg_profile_with_reversed_settings(),
            ],
        );
        let reordered_backup = backup_profiles(
            &detection,
            &reordered_bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect("canonical readback order should still back up");

        assert_eq!(backup.fingerprint, reordered_backup.fingerprint);
    }

    #[test]
    fn profile_backup_allows_validated_npi_compatibility_path() {
        let detection = ready_detection(GpuCapabilityState::Missing, GpuCapabilityState::Ready);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvidiaProfileInspectorCompatibility(
                NpiCompatibilityValidation::validated("2.4.0.4", "32.0.15.6094"),
            ),
            vec![pubg_profile_with_reversed_settings()],
        );

        let backup = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect("validated NPI compatibility should be accepted");

        assert_eq!(
            backup.snapshot.bridge_kind.stable_key(),
            "npi-compatibility"
        );
        assert_eq!(backup.snapshot.driver_versions, vec!["32.0.15.6094"]);
    }

    #[test]
    fn profile_backup_rejects_unvalidated_npi_export() {
        let detection = ready_detection(GpuCapabilityState::Missing, GpuCapabilityState::Ready);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvidiaProfileInspectorCompatibility(
                NpiCompatibilityValidation::unvalidated("2.4.0.4", "32.0.15.6094"),
            ),
            vec![pubg_profile_with_reversed_settings()],
        );

        let error = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect_err("unvalidated NPI exports must be blocked");

        assert!(matches!(
            error,
            NvidiaProfileError::UnvalidatedProfileInspector { .. }
        ));
    }

    #[test]
    fn profile_backup_requires_ready_nvapi_bridge() {
        let detection = ready_detection(GpuCapabilityState::Unknown, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![pubg_profile_with_reversed_settings()],
        );

        let error = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect_err("unknown NVAPI state should not back up profiles");

        assert_eq!(
            error,
            NvidiaProfileError::ProfileBridgeUnavailable {
                kind: NvidiaProfileBridgeKind::NvapiDriverSettings,
                state: GpuCapabilityState::Unknown,
            }
        );
    }

    #[test]
    fn profile_backup_filters_to_customized_profiles_when_requested() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                NvidiaProfile::application(
                    "Empty app profile",
                    vec!["Empty.exe".to_owned()],
                    Vec::new(),
                ),
                pubg_profile_with_reversed_settings(),
            ],
        );

        let backup = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::customized_profiles_before_mutation(),
        )
        .expect("customized profile should remain selected");

        assert_eq!(backup.snapshot.profiles.len(), 1);
        assert_eq!(backup.snapshot.profiles[0].name, "PUBG Competitive");
    }

    #[test]
    fn profile_backup_validates_application_profile_executables() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::application(
                "PUBG Competitive",
                Vec::new(),
                vec![NvidiaProfileSetting::new(
                    "low-latency-mode",
                    "Low Latency Mode",
                    "On",
                    NvidiaProfileSettingVisibility::UserVisible,
                )],
            )],
        );

        let error = readback_profiles(&detection, &bridge)
            .expect_err("application profiles require executable bindings");

        assert!(matches!(
            error,
            NvidiaProfileError::InvalidProfile { profile, .. } if profile == "PUBG Competitive"
        ));
    }

    #[test]
    fn global_performance_profile_uses_only_conservative_settings() {
        let profile = global_performance_profile();

        validate_global_performance_profile(&profile)
            .expect("global profile should pass its conservative allowlist");

        assert_eq!(profile.name, LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME);
        assert_eq!(profile.scope, NvidiaProfileScope::Global);
        assert!(profile.applications.is_empty());
        assert!(profile.settings.iter().all(|setting| {
            setting.visibility != NvidiaProfileSettingVisibility::Hidden
                && setting.value != "Prefer maximum performance"
                && setting.value != "Ultra"
        }));
        assert!(profile
            .settings
            .iter()
            .any(|setting| setting.id == "shader-cache" && setting.value == "On"));
        assert!(profile
            .settings
            .iter()
            .any(|setting| setting.id == "max-frame-rate" && setting.value == "Off"));
    }

    #[test]
    fn global_profile_apply_backs_up_writes_and_verifies_readback() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let mut bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                NvidiaProfile::global(
                    "Base Profile",
                    vec![NvidiaProfileSetting::new(
                        "power-management-mode",
                        "Power management mode",
                        "Prefer maximum performance",
                        NvidiaProfileSettingVisibility::UserVisible,
                    )],
                ),
                pubg_profile_with_reversed_settings(),
            ],
        );

        let result = apply_global_performance_profile(&detection, &mut bridge)
            .expect("global profile should apply and verify through readback");

        assert_eq!(result.tweak_id, NVIDIA_GLOBAL_PROFILE_TWEAK_ID);
        assert_eq!(
            result.requested_profile.name,
            LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
        );
        assert_eq!(result.backup.tweak_id, NVIDIA_PROFILE_BACKUP_TWEAK_ID);
        assert_eq!(result.backup.snapshot.profiles.len(), 2);
        assert_eq!(bridge.writes.len(), 1);
        assert_eq!(
            result.verified_profile.name,
            LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
        );
        assert_eq!(
            result.verified_profile.settings,
            canonicalize_settings(approved_global_performance_settings())
        );
    }

    #[test]
    fn global_profile_readback_rejects_tampered_settings() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let mut bridge = FixtureProfileBridge::with_tampered_write(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::global("Base Profile", Vec::new())],
            "low-latency-mode",
            "Ultra",
        );

        let error = apply_global_performance_profile(&detection, &mut bridge)
            .expect_err("readback must catch settings changed outside the allowlist");

        assert!(matches!(
            error,
            NvidiaProfileError::UnsafeGlobalProfileSetting { setting_id, .. }
                if setting_id == "low-latency-mode"
        ));
    }

    #[test]
    fn global_profile_readback_requires_the_named_liiiraa_profile() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::global("Base Profile", Vec::new())],
        );
        let snapshot = readback_profiles(&detection, &bridge)
            .expect("fixture readback should be valid without the Liiiraa profile");

        let error = verify_global_performance_profile_readback(&snapshot)
            .expect_err("missing Liiiraa global profile should fail verification");

        assert!(matches!(
            error,
            NvidiaProfileError::GlobalProfileReadbackMissing { profile }
                if profile == LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
        ));
    }

    #[test]
    fn pubg_competitive_profile_uses_vrr_reflex_and_rebar_policy() {
        let request = NvidiaPubgCompetitiveProfileRequest::new(
            Some(240),
            true,
            true,
            GpuCapabilityState::Ready,
            PubgRuntimeState::no_processes(),
        );

        let plan = plan_pubg_competitive_profile(&request)
            .expect("PUBG competitive profile should plan from read-only state");

        assert_eq!(plan.profile.name, LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME);
        assert_eq!(plan.profile.scope, NvidiaProfileScope::Application);
        assert_eq!(
            plan.profile.applications,
            vec![PUBG_EXECUTABLE_NAME.to_owned()]
        );
        assert_eq!(plan.fps_cap, Some(237));
        assert_eq!(
            plan.reflex_policy,
            NvidiaPubgReflexPolicy::PreferInGameReflex
        );
        assert_eq!(
            plan.vrr_policy,
            NvidiaPubgVrrPolicy::VrrCap {
                refresh_rate_hz: 240,
                cap_fps: 237,
            }
        );
        assert_eq!(plan.rebar_policy, NvidiaPubgRebarPolicy::OfficialDriverPolicy);
        assert!(plan
            .manual_actions
            .iter()
            .any(|action| action.contains("Reflex + Boost")));

        let low_latency = setting(&plan.profile, "low-latency-mode");
        let max_frame_rate = setting(&plan.profile, "max-frame-rate");
        let v_sync = setting(&plan.profile, "vertical-sync");

        assert_eq!(low_latency.value, "Off");
        assert_eq!(max_frame_rate.value, "237 FPS");
        assert_eq!(v_sync.value, "On");
        assert!(plan
            .profile
            .settings
            .iter()
            .all(|setting| setting.visibility != NvidiaProfileSettingVisibility::Hidden));
    }

    #[test]
    fn pubg_competitive_profile_uses_driver_llm_without_reflex_or_vrr() {
        let request = NvidiaPubgCompetitiveProfileRequest::new(
            None,
            false,
            false,
            GpuCapabilityState::Missing,
            PubgRuntimeState::no_processes(),
        );

        let plan = plan_pubg_competitive_profile(&request)
            .expect("non-VRR non-Reflex path should still produce a safe profile");

        assert_eq!(
            plan.reflex_policy,
            NvidiaPubgReflexPolicy::DriverLowLatencyOn
        );
        assert_eq!(plan.vrr_policy, NvidiaPubgVrrPolicy::ApplicationControlled);
        assert_eq!(
            plan.rebar_policy,
            NvidiaPubgRebarPolicy::RecommendOfficialEnablement
        );
        assert_eq!(setting(&plan.profile, "low-latency-mode").value, "On");
        assert_eq!(setting(&plan.profile, "max-frame-rate").value, "Off");
        assert_eq!(
            setting(&plan.profile, "vertical-sync").value,
            "Use the 3D application setting"
        );
    }

    #[test]
    fn pubg_profile_apply_backs_up_writes_and_verifies_readback() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let mut bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                NvidiaProfile::global("Base Profile", Vec::new()),
                pubg_profile_with_reversed_settings(),
            ],
        );
        let request = NvidiaPubgCompetitiveProfileRequest::new(
            Some(165),
            true,
            true,
            GpuCapabilityState::Ready,
            PubgRuntimeState::no_processes(),
        );

        let result = apply_pubg_competitive_profile(&detection, &mut bridge, &request)
            .expect("PUBG profile should apply and verify through readback");

        assert_eq!(result.tweak_id, NVIDIA_PUBG_PROFILE_TWEAK_ID);
        assert_eq!(
            result.requested_plan.profile.name,
            LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME
        );
        assert_eq!(result.backup.tweak_id, NVIDIA_PROFILE_BACKUP_TWEAK_ID);
        assert_eq!(result.backup.snapshot.profiles.len(), 2);
        assert_eq!(bridge.writes.len(), 1);
        assert_eq!(
            result.verified_profile.name,
            LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME
        );
        assert_eq!(
            setting(&result.verified_profile, "max-frame-rate").value,
            "162 FPS"
        );
    }

    #[test]
    fn pubg_profile_apply_defers_while_pubg_or_battleye_runs() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let mut bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::global("Base Profile", Vec::new())],
        );
        let request = NvidiaPubgCompetitiveProfileRequest::new(
            Some(144),
            true,
            true,
            GpuCapabilityState::Ready,
            PubgRuntimeState::from_process_names(["BEService.exe"]),
        );

        let error = apply_pubg_competitive_profile(&detection, &mut bridge, &request)
            .expect_err("live BattlEye should block profile mutation");

        assert!(matches!(
            error,
            NvidiaProfileError::PubgOrBattleyeRunning { processes }
                if processes == vec!["BEService.exe".to_owned()]
        ));
        assert!(bridge.writes.is_empty());
    }

    #[test]
    fn profile_rollback_restores_backup_and_deletes_created_liiiraa_profiles() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let mut bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::global(
                "Base Profile",
                vec![NvidiaProfileSetting::new(
                    "power-management-mode",
                    "Power management mode",
                    "Prefer maximum performance",
                    NvidiaProfileSettingVisibility::UserVisible,
                )],
            )],
        );

        let apply = apply_global_performance_profile(&detection, &mut bridge)
            .expect("apply should create a Liiiraa global profile");

        assert!(bridge.profiles.iter().any(|profile| {
            profile.name == LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
                && profile.scope == NvidiaProfileScope::Global
        }));

        let rollback = rollback_profiles_from_backup(&detection, &mut bridge, &apply.backup)
            .expect("rollback should restore the captured profile backup");

        assert_eq!(rollback.tweak_id, NVIDIA_PROFILE_ROLLBACK_TWEAK_ID);
        assert_eq!(rollback.restored_profiles.len(), 1);
        assert_eq!(rollback.deleted_profiles.len(), 1);
        assert_eq!(
            rollback.deleted_profiles[0].name,
            LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
        );
        assert!(!bridge.profiles.iter().any(|profile| {
            profile.name == LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME
                && profile.scope == NvidiaProfileScope::Global
        }));
        assert!(rollback
            .verified_snapshot
            .profiles
            .iter()
            .any(|profile| profile.name == "Base Profile"));
    }

    #[test]
    fn profile_rollback_restores_existing_liiiraa_profile_values() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let original_profile = NvidiaProfile::application(
            LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME,
            vec![PUBG_EXECUTABLE_NAME.to_owned()],
            vec![
                NvidiaProfileSetting::new(
                    "max-frame-rate",
                    "Max Frame Rate",
                    "Off",
                    NvidiaProfileSettingVisibility::UserVisible,
                ),
                NvidiaProfileSetting::new(
                    "low-latency-mode",
                    "Low Latency Mode",
                    "Off",
                    NvidiaProfileSettingVisibility::UserVisible,
                ),
            ],
        );
        let mut bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![
                NvidiaProfile::global("Base Profile", Vec::new()),
                original_profile.clone(),
            ],
        );
        let backup = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect("backup should capture the existing Liiiraa PUBG profile");
        let request = NvidiaPubgCompetitiveProfileRequest::new(
            Some(165),
            true,
            true,
            GpuCapabilityState::Ready,
            PubgRuntimeState::no_processes(),
        );

        apply_pubg_competitive_profile(&detection, &mut bridge, &request)
            .expect("apply should replace the Liiiraa PUBG profile");
        assert_eq!(
            setting(
                bridge
                    .profiles
                    .iter()
                    .find(|profile| profile.name == LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME)
                    .expect("Liiiraa profile should exist after apply"),
                "max-frame-rate"
            )
            .value,
            "162 FPS"
        );

        let rollback = rollback_profiles_from_backup(&detection, &mut bridge, &backup)
            .expect("rollback should restore backed-up profile settings");
        let restored = rollback
            .verified_snapshot
            .profiles
            .iter()
            .find(|profile| profile.name == LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME)
            .expect("restored Liiiraa profile should remain present");

        assert!(rollback.deleted_profiles.is_empty());
        assert_eq!(setting(restored, "max-frame-rate").value, "Off");
        assert_eq!(setting(restored, "low-latency-mode").value, "Off");
        assert_eq!(setting(&original_profile, "max-frame-rate").value, "Off");
    }

    #[test]
    fn profile_rollback_rejects_tampered_backup_payload() {
        let detection = ready_detection(GpuCapabilityState::Ready, GpuCapabilityState::Missing);
        let bridge = FixtureProfileBridge::new(
            NvidiaProfileBridgeKind::NvapiDriverSettings,
            vec![NvidiaProfile::global("Base Profile", Vec::new())],
        );
        let mut backup = backup_profiles(
            &detection,
            &bridge,
            NvidiaProfileBackupRequest::all_profiles_before_mutation(),
        )
        .expect("backup should capture");
        backup.fingerprint = "0000000000000000".to_owned();
        let mut rollback_bridge = bridge.clone();

        let error = rollback_profiles_from_backup(&detection, &mut rollback_bridge, &backup)
            .expect_err("tampered backups must not restore");

        assert!(matches!(
            error,
            NvidiaProfileError::BackupIntegrityMismatch { .. }
        ));
    }

    #[test]
    fn pubg_profile_validation_rejects_hidden_rebar_override() {
        let mut profile = pubg_competitive_profile(&NvidiaPubgCompetitiveProfileRequest::new(
            Some(240),
            true,
            true,
            GpuCapabilityState::Ready,
            PubgRuntimeState::no_processes(),
        ))
        .expect("base PUBG profile should be valid");
        profile.settings.push(NvidiaProfileSetting::new(
            "rebar-hidden-override",
            "Resizable BAR hidden override",
            "Forced",
            NvidiaProfileSettingVisibility::Hidden,
        ));

        let error = validate_pubg_competitive_profile(&profile)
            .expect_err("hidden ReBAR settings must be Lab-only");

        assert!(matches!(
            error,
            NvidiaProfileError::UnsafePubgProfileSetting { setting_id, .. }
                if setting_id == "rebar-hidden-override"
        ));
    }

    #[test]
    fn catalog_fixture_matches_global_profile_contract() {
        let fixture = include_str!("../tests/fixtures/nvidia_global_performance_profile.catalog.json");

        assert!(fixture.contains(NVIDIA_GLOBAL_PROFILE_TWEAK_ID));
        assert!(fixture.contains(LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME));
        assert!(fixture.contains("\"requiresBackup\": true"));
        assert!(fixture.contains("\"verify\": \"readback\""));
        assert!(!fixture.contains("\"visibility\": \"hidden\""));

        for setting in approved_global_performance_settings() {
            assert!(fixture.contains(setting.id.as_str()));
            assert!(fixture.contains(setting.value.as_str()));
        }
    }

    #[test]
    fn catalog_fixture_matches_pubg_profile_contract() {
        let fixture =
            include_str!("../tests/fixtures/nvidia_pubg_competitive_profile.catalog.json");

        assert!(fixture.contains(NVIDIA_PUBG_PROFILE_TWEAK_ID));
        assert!(fixture.contains(LIIIRAA_PUBG_COMPETITIVE_PROFILE_NAME));
        assert!(fixture.contains(PUBG_EXECUTABLE_NAME));
        assert!(fixture.contains("\"mode\": \"competitive\""));
        assert!(fixture.contains("\"requiresBackup\": true"));
        assert!(fixture.contains("\"verify\": \"readback\""));
        assert!(fixture.contains("\"gameClosedRequired\": true"));
        assert!(fixture.contains("Reflex"));
        assert!(fixture.contains("G-SYNC"));
        assert!(fixture.contains("Resizable BAR"));
        assert!(!fixture.contains("\"visibility\": \"hidden\""));

        for setting_id in REQUIRED_PUBG_COMPETITIVE_SETTING_IDS {
            assert!(fixture.contains(*setting_id));
        }
    }

    fn setting<'a>(profile: &'a NvidiaProfile, setting_id: &str) -> &'a NvidiaProfileSetting {
        profile
            .settings
            .iter()
            .find(|setting| setting.id == setting_id)
            .expect("profile setting should exist")
    }

    fn ready_detection(
        profile_api_state: GpuCapabilityState,
        profile_inspector_state: GpuCapabilityState,
    ) -> NvidiaDriverDetection {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "NVIDIA GeForce RTX 4070",
            Some("32.0.15.6094"),
            None,
            None,
            Some("PCI\\VEN_10DE&DEV_2786"),
        )]);
        let mut detection = NvidiaDriverDetection::from_inventory(&inventory);
        detection.profile_api_state = profile_api_state;
        detection.profile_inspector_state = profile_inspector_state;
        detection
    }

    fn pubg_profile_with_reversed_settings() -> NvidiaProfile {
        NvidiaProfile::application(
            "PUBG Competitive",
            vec!["TslGame.exe".to_owned(), "TslGame.exe".to_owned()],
            vec![
                NvidiaProfileSetting::new(
                    "power-management-mode",
                    "Power management mode",
                    "Prefer maximum performance",
                    NvidiaProfileSettingVisibility::UserVisible,
                ),
                NvidiaProfileSetting::new(
                    "low-latency-mode",
                    "Low Latency Mode",
                    "On",
                    NvidiaProfileSettingVisibility::UserVisible,
                ),
            ],
        )
    }
}
