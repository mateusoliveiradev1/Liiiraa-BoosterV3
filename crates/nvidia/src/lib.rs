//! NVIDIA profile backup, planning, apply, and verification.

use gpu::{GpuCapabilityState, GpuInventory, GpuVendor, GpuVendorDetection};
use std::{collections::BTreeSet, fmt};

/// Tweak ID for the required NVIDIA profile backup action.
pub const NVIDIA_PROFILE_BACKUP_TWEAK_ID: &str = "nvidia.backup.profiles";

/// Tweak ID for the conservative global NVIDIA performance profile.
pub const NVIDIA_GLOBAL_PROFILE_TWEAK_ID: &str = "nvidia.global.profile";

/// Driver profile name owned by Liiiraa for conservative global performance defaults.
pub const LIIIRAA_GLOBAL_PERFORMANCE_PROFILE_NAME: &str =
    "Liiiraa Boost - Global Performance";

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
    /// The expected Liiiraa global profile was not visible during readback.
    GlobalProfileReadbackMissing {
        /// Expected global profile name.
        profile: String,
    },
    /// The backup request filtered out all readback profiles.
    NoProfilesSelected,
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
            Self::InvalidProfile { profile, reason } => {
                write!(formatter, "invalid NVIDIA profile {profile:?}: {reason}")
            }
            Self::UnsafeGlobalProfileSetting { setting_id, reason } => {
                write!(
                    formatter,
                    "unsafe global NVIDIA profile setting {setting_id:?}: {reason}"
                )
            }
            Self::GlobalProfileReadbackMissing { profile } => {
                write!(
                    formatter,
                    "global NVIDIA profile {profile:?} was not found during readback"
                )
            }
            Self::NoProfilesSelected => {
                formatter.write_str("backup request did not select any NVIDIA profiles")
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
        tampered_write: Option<(String, String)>,
    }

    impl FixtureProfileBridge {
        fn new(kind: NvidiaProfileBridgeKind, profiles: Vec<NvidiaProfile>) -> Self {
            Self {
                kind,
                profiles,
                writes: Vec::new(),
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
