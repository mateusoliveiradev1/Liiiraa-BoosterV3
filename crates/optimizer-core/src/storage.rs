//! Safe planning for Storage Sense, cleanup preview, TRIM, DirectStorage, and NTFS metadata.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for previewing safe temporary cleanup candidates.
pub const STORAGE_TEMP_CLEANUP_TWEAK_ID: &str = "storage.temp.cleanup";
/// Tweak ID for configuring Storage Sense after prompt and backup.
pub const STORAGE_SENSE_CONFIGURE_TWEAK_ID: &str = "storage.sense.configure";
/// Tweak ID for verifying Windows TRIM/Optimize Drives readiness.
pub const STORAGE_TRIM_VERIFY_TWEAK_ID: &str = "storage.trim.verify";
/// Tweak ID for checking DirectStorage readiness.
pub const STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID: &str = "storage.directstorage.check";
/// Tweak ID for denying unsupported NVMe driver registry hacks.
pub const STORAGE_NVME_DRIVER_HACK_TWEAK_ID: &str = "storage.nvme.driver-hack";
/// Tweak ID for disabling NTFS last-access timestamp updates after compatibility review.
pub const STORAGE_NTFS_LAST_ACCESS_TWEAK_ID: &str = "storage.ntfs.last-access";
/// Tweak ID for disabling NTFS 8.3 name creation for future files after compatibility review.
pub const STORAGE_NTFS_8DOT3_TWEAK_ID: &str = "storage.ntfs.8dot3";

/// HKCU Storage Sense global enable value.
pub const TARGET_STORAGE_SENSE_ENABLED: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/storagesense/parameters/storagepolicy/01";
/// HKCU Storage Sense cadence value.
pub const TARGET_STORAGE_SENSE_CADENCE: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/storagesense/parameters/storagepolicy/2048";
/// HKCU Storage Sense recycle-bin age value.
pub const TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/storagesense/parameters/storagepolicy/256";
/// HKCU Storage Sense downloads cleanup age value.
pub const TARGET_STORAGE_SENSE_DOWNLOADS_DAYS: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/storagesense/parameters/storagepolicy/512";
/// NTFS DisableDeleteNotify state exposed by fsutil.
pub const TARGET_TRIM_NTFS_DISABLE_DELETE_NOTIFY: &str =
    "fsutil:behavior/disabledeletenotify/ntfs";
/// ReFS DisableDeleteNotify state exposed by fsutil.
pub const TARGET_TRIM_REFS_DISABLE_DELETE_NOTIFY: &str =
    "fsutil:behavior/disabledeletenotify/refs";
/// Windows-supported Optimize-Volume path.
pub const TARGET_OPTIMIZE_VOLUME_SUPPORTED: &str = "optimize-volume:supported";
/// DirectStorage readiness target.
pub const TARGET_DIRECTSTORAGE_READINESS: &str = "directstorage:readiness";
/// Unsupported storage driver hack denial target.
pub const TARGET_UNSUPPORTED_NVME_DRIVER_HACK: &str = "blocked:storage/nvme-driver-hack";
/// NTFS DisableLastAccess state exposed by fsutil.
pub const TARGET_NTFS_DISABLE_LAST_ACCESS: &str = "fsutil:behavior/disablelastaccess";
/// NTFS Disable8dot3 state exposed by fsutil.
pub const TARGET_NTFS_DISABLE_8DOT3: &str = "fsutil:8dot3name/disable8dot3";

const CLEANUP_TARGET_PREFIX: &str = "cleanup-preview:";
const DESIRED_DISABLE_LAST_ACCESS_VALUE: &str = "1";
const DESIRED_DISABLE_8DOT3_VALUE: &str = "1";

/// Explicit user consent for prompt-only storage controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageControlConsent {
    /// The user has not accepted the prompted storage control.
    NotGranted,
    /// The user explicitly accepted the prompted storage control.
    Granted,
}

impl StorageControlConsent {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Storage Sense state discovered from Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageSenseState {
    /// Storage Sense appears enabled.
    Enabled,
    /// Storage Sense appears disabled.
    Disabled,
    /// Windows did not expose Storage Sense state.
    Unknown,
}

impl StorageSenseState {
    const fn as_previous_value(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unknown => "unknown",
        }
    }
}

/// Desired Storage Sense configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSensePreference {
    /// Desired global enabled state.
    pub enabled: bool,
    /// Desired cadence in days.
    pub cadence_days: u32,
    /// Desired recycle-bin cleanup threshold in days.
    pub recycle_bin_days: u32,
    /// Desired downloads cleanup threshold in days, when the user opts into it.
    pub downloads_days: Option<u32>,
}

impl StorageSensePreference {
    /// Creates a conservative Storage Sense preference.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            enabled: true,
            cadence_days: 30,
            recycle_bin_days: 30,
            downloads_days: None,
        }
    }
}

/// Current Storage Sense inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageSenseInspection {
    /// Current Storage Sense state.
    pub state: StorageSenseState,
    /// Current cadence in days, when known.
    pub cadence_days: Option<u32>,
    /// Current recycle-bin cleanup threshold in days, when known.
    pub recycle_bin_days: Option<u32>,
    /// Current downloads cleanup threshold in days, when known.
    pub downloads_days: Option<u32>,
}

impl StorageSenseInspection {
    /// Creates an unknown Storage Sense inspection.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            state: StorageSenseState::Unknown,
            cadence_days: None,
            recycle_bin_days: None,
            downloads_days: None,
        }
    }

    fn matches_preference(self, preference: StorageSensePreference) -> bool {
        let state_matches = match preference.enabled {
            true => self.state == StorageSenseState::Enabled,
            false => self.state == StorageSenseState::Disabled,
        };

        let downloads_matches = preference
            .downloads_days
            .map_or(true, |days| self.downloads_days == Some(days));

        state_matches
            && self.cadence_days == Some(preference.cadence_days)
            && self.recycle_bin_days == Some(preference.recycle_bin_days)
            && downloads_matches
    }
}

/// Type of previewed cleanup location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupCandidateKind {
    /// User temp directory.
    UserTemp,
    /// Windows temp directory.
    WindowsTemp,
    /// GPU or DirectX shader cache.
    ShaderCache,
    /// Another safe temp-like location.
    Other,
}

impl CleanupCandidateKind {
    const fn as_state(self) -> &'static str {
        match self {
            Self::UserTemp => "user_temp",
            Self::WindowsTemp => "windows_temp",
            Self::ShaderCache => "shader_cache",
            Self::Other => "other",
        }
    }
}

/// One cleanup candidate produced by a read-only preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupCandidate {
    /// Logical target key used by plan previews.
    pub target: String,
    /// Human-facing path or location label.
    pub path: String,
    /// Candidate kind.
    pub kind: CleanupCandidateKind,
    /// Estimated reclaimable bytes.
    pub reclaimable_bytes: u64,
    /// Estimated file count.
    pub file_count: u32,
    /// Whether the candidate is within the safe cleanup allowlist.
    pub safe_to_preview: bool,
}

impl CleanupCandidate {
    /// Creates a safe cleanup candidate preview.
    #[must_use]
    pub fn safe(
        target: impl Into<String>,
        path: impl Into<String>,
        kind: CleanupCandidateKind,
        reclaimable_bytes: u64,
        file_count: u32,
    ) -> Self {
        Self {
            target: target.into(),
            path: path.into(),
            kind,
            reclaimable_bytes,
            file_count,
            safe_to_preview: true,
        }
    }

    /// Creates a cleanup candidate that must be excluded.
    #[must_use]
    pub fn excluded(
        target: impl Into<String>,
        path: impl Into<String>,
        kind: CleanupCandidateKind,
    ) -> Self {
        Self {
            target: target.into(),
            path: path.into(),
            kind,
            reclaimable_bytes: 0,
            file_count: 0,
            safe_to_preview: false,
        }
    }
}

/// Read-only cleanup preview from Windows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupPreview {
    /// Candidate locations and sizes.
    pub candidates: Vec<CleanupCandidate>,
    /// Explicit exclusions that keep cleanup from becoming blind deletion.
    pub excluded_patterns: Vec<String>,
}

impl CleanupPreview {
    /// Creates an empty cleanup preview.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            candidates: Vec::new(),
            excluded_patterns: Vec::new(),
        }
    }

    fn safe_candidates(&self) -> impl Iterator<Item = &CleanupCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.safe_to_preview && candidate.reclaimable_bytes > 0)
    }
}

/// TRIM state discovered by Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrimState {
    /// Delete notification is enabled.
    Enabled,
    /// Delete notification is disabled.
    Disabled,
    /// Windows did not expose the state.
    Unknown,
}

impl TrimState {
    /// Converts `fsutil DisableDeleteNotify` values to TRIM state.
    #[must_use]
    pub const fn from_disable_delete_notify(value: Option<u32>) -> Self {
        match value {
            Some(0) => Self::Enabled,
            Some(_) => Self::Disabled,
            None => Self::Unknown,
        }
    }

    const fn as_state(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unknown => "unknown",
        }
    }
}

/// Windows TRIM and Optimize-Volume inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimInspection {
    /// NTFS TRIM state.
    pub ntfs: TrimState,
    /// ReFS TRIM state.
    pub refs: TrimState,
    /// Whether the Windows Optimize-Volume command is available.
    pub optimize_volume_available: bool,
    /// Whether SSD or NVMe media is present.
    pub solid_state_media_present: bool,
}

impl TrimInspection {
    /// Creates an unknown TRIM inspection.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            ntfs: TrimState::Unknown,
            refs: TrimState::Unknown,
            optimize_volume_available: false,
            solid_state_media_present: false,
        }
    }
}

/// DirectStorage readiness inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectStorageInspection {
    /// Whether the OS build supports DirectStorage.
    pub os_supported: Option<bool>,
    /// Whether an NVMe device is present.
    pub nvme_present: Option<bool>,
    /// Whether the game is on an NVMe-backed volume.
    pub game_on_nvme: Option<bool>,
    /// Whether GPU decompression support is known.
    pub gpu_decompression_supported: Option<bool>,
}

impl DirectStorageInspection {
    /// Creates an unknown DirectStorage inspection.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            os_supported: None,
            nvme_present: None,
            game_on_nvme: None,
            gpu_decompression_supported: None,
        }
    }

    fn is_ready(self) -> bool {
        self.os_supported == Some(true)
            && self.nvme_present == Some(true)
            && self.game_on_nvme != Some(false)
            && self.gpu_decompression_supported != Some(false)
    }

    fn has_known_blocker(self) -> bool {
        self.os_supported == Some(false)
            || self.nvme_present == Some(false)
            || self.game_on_nvme == Some(false)
            || self.gpu_decompression_supported == Some(false)
    }
}

/// Compatibility risk found before applying NTFS metadata tweaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtfsMetadataCompatibility {
    /// No known workload depends on the legacy behavior.
    LowRisk,
    /// Compatibility has not been proven yet.
    Unknown,
    /// A backup, compliance, installer, or legacy application dependency is known.
    KnownDependency,
}

impl NtfsMetadataCompatibility {
    const fn is_low_risk(self) -> bool {
        matches!(self, Self::LowRisk)
    }

    const fn has_known_dependency(self) -> bool {
        matches!(self, Self::KnownDependency)
    }
}

/// NTFS last-access update state from `fsutil behavior query DisableLastAccess`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtfsLastAccessState {
    /// Last-access timestamp updates are enabled.
    Enabled {
        /// Raw fsutil value.
        raw_value: u32,
    },
    /// Last-access timestamp updates are disabled.
    Disabled {
        /// Raw fsutil value.
        raw_value: u32,
    },
    /// The fsutil output could not be parsed.
    Unknown,
}

impl NtfsLastAccessState {
    /// Converts fsutil DisableLastAccess values to state.
    #[must_use]
    pub const fn from_disable_last_access_value(value: Option<u32>) -> Self {
        match value {
            Some(0) => Self::Enabled { raw_value: 0 },
            Some(2) => Self::Enabled { raw_value: 2 },
            Some(1) => Self::Disabled { raw_value: 1 },
            Some(3) => Self::Disabled { raw_value: 3 },
            _ => Self::Unknown,
        }
    }

    const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled { .. })
    }

    const fn raw_value(self) -> Option<u32> {
        match self {
            Self::Enabled { raw_value } | Self::Disabled { raw_value } => Some(raw_value),
            Self::Unknown => None,
        }
    }

    fn as_state(self) -> String {
        match self {
            Self::Enabled { raw_value } => format!("enabled:{raw_value}"),
            Self::Disabled { raw_value } => format!("disabled:{raw_value}"),
            Self::Unknown => "unknown".to_owned(),
        }
    }
}

/// NTFS 8.3 name-creation state from `fsutil 8dot3name query`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtfsEightDotThreeNameState {
    /// 8.3 name creation is enabled for all volumes.
    EnabledAll,
    /// 8.3 name creation is disabled for all volumes.
    DisabledAll,
    /// 8.3 behavior is controlled per volume.
    PerVolume,
    /// 8.3 creation is disabled except on the system volume.
    DisabledExceptSystemVolume,
    /// The fsutil output could not be parsed.
    Unknown,
}

impl NtfsEightDotThreeNameState {
    /// Converts fsutil Disable8dot3 registry values to state.
    #[must_use]
    pub const fn from_disable_8dot3_value(value: Option<u32>) -> Self {
        match value {
            Some(0) => Self::EnabledAll,
            Some(1) => Self::DisabledAll,
            Some(2) => Self::PerVolume,
            Some(3) => Self::DisabledExceptSystemVolume,
            _ => Self::Unknown,
        }
    }

    const fn is_disabled_for_new_files(self) -> bool {
        matches!(self, Self::DisabledAll | Self::DisabledExceptSystemVolume)
    }

    const fn raw_value(self) -> Option<u32> {
        match self {
            Self::EnabledAll => Some(0),
            Self::DisabledAll => Some(1),
            Self::PerVolume => Some(2),
            Self::DisabledExceptSystemVolume => Some(3),
            Self::Unknown => None,
        }
    }

    const fn as_state(self) -> &'static str {
        match self {
            Self::EnabledAll => "enabled_all:0",
            Self::DisabledAll => "disabled_all:1",
            Self::PerVolume => "per_volume:2",
            Self::DisabledExceptSystemVolume => "disabled_except_system_volume:3",
            Self::Unknown => "unknown",
        }
    }
}

/// Request used to build the T044 storage readiness plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageReadinessPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Read-only cleanup preview.
    pub cleanup_preview: CleanupPreview,
    /// Explicit consent for Storage Sense configuration.
    pub storage_sense_consent: StorageControlConsent,
    /// Current Storage Sense state.
    pub storage_sense: StorageSenseInspection,
    /// Desired Storage Sense state.
    pub desired_storage_sense: StorageSensePreference,
    /// Current TRIM state.
    pub trim: TrimInspection,
    /// Current DirectStorage readiness.
    pub direct_storage: DirectStorageInspection,
    /// Whether an unsupported NVMe driver hack was requested.
    pub unsupported_nvme_driver_hack_requested: bool,
}

impl StorageReadinessPlanRequest {
    /// Creates a conservative storage readiness request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            cleanup_preview: CleanupPreview::empty(),
            storage_sense_consent: StorageControlConsent::NotGranted,
            storage_sense: StorageSenseInspection::unknown(),
            desired_storage_sense: StorageSensePreference::conservative(),
            trim: TrimInspection::unknown(),
            direct_storage: DirectStorageInspection::unknown(),
            unsupported_nvme_driver_hack_requested: false,
        }
    }
}

/// Request used to build the T054 NTFS metadata plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtfsMetadataPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Current NTFS last-access update state.
    pub last_access_state: NtfsLastAccessState,
    /// Explicit consent for changing last-access behavior.
    pub last_access_consent: StorageControlConsent,
    /// Compatibility risk for backup/compliance tools that depend on last-access timestamps.
    pub last_access_compatibility: NtfsMetadataCompatibility,
    /// Current NTFS 8.3 name-creation state.
    pub eight_dot_three_state: NtfsEightDotThreeNameState,
    /// Explicit consent for changing 8.3 name creation.
    pub eight_dot_three_consent: StorageControlConsent,
    /// Compatibility risk for legacy installers or applications that depend on 8.3 names.
    pub eight_dot_three_compatibility: NtfsMetadataCompatibility,
}

impl NtfsMetadataPlanRequest {
    /// Creates a conservative NTFS metadata request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            last_access_state: NtfsLastAccessState::Unknown,
            last_access_consent: StorageControlConsent::NotGranted,
            last_access_compatibility: NtfsMetadataCompatibility::Unknown,
            eight_dot_three_state: NtfsEightDotThreeNameState::Unknown,
            eight_dot_three_consent: StorageControlConsent::NotGranted,
            eight_dot_three_compatibility: NtfsMetadataCompatibility::Unknown,
        }
    }
}

/// Builds a dry-run plan for T044 storage readiness checks.
#[must_use]
pub fn build_storage_readiness_plan(request: &StorageReadinessPlanRequest) -> TweakPlan {
    let items = vec![
        cleanup_preview_item(request),
        storage_sense_item(request),
        trim_verify_item(request),
        direct_storage_item(request),
        nvme_driver_hack_guardrail_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Preview-only")
                || warning.contains("Do not force SSD defrag")
                || warning.contains("denied")
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

/// Builds a dry-run plan for T054 NTFS last-access and 8.3 metadata behavior.
#[must_use]
pub fn build_ntfs_metadata_plan(request: &NtfsMetadataPlanRequest) -> TweakPlan {
    let items = vec![
        ntfs_last_access_item(request),
        ntfs_eight_dot_three_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("NTFS")
                || warning.contains("8.3")
                || warning.contains("backup")
                || warning.contains("compatibility")
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

/// Returns true when the ID belongs to the T044 storage scope.
#[must_use]
pub fn is_storage_readiness_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        STORAGE_TEMP_CLEANUP_TWEAK_ID
            | STORAGE_SENSE_CONFIGURE_TWEAK_ID
            | STORAGE_TRIM_VERIFY_TWEAK_ID
            | STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID
            | STORAGE_NVME_DRIVER_HACK_TWEAK_ID
    )
}

/// Returns true when the ID belongs to the T054 NTFS metadata scope.
#[must_use]
pub fn is_ntfs_metadata_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        STORAGE_NTFS_LAST_ACCESS_TWEAK_ID | STORAGE_NTFS_8DOT3_TWEAK_ID
    )
}

/// Returns true when the target is a typed Storage Sense registry target.
#[must_use]
pub fn is_storage_sense_registry_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_STORAGE_SENSE_ENABLED
            | TARGET_STORAGE_SENSE_CADENCE
            | TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS
            | TARGET_STORAGE_SENSE_DOWNLOADS_DAYS
    )
}

/// Returns true when the target is an allowlisted fsutil NTFS metadata target.
#[must_use]
pub fn is_ntfs_metadata_fsutil_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_NTFS_DISABLE_LAST_ACCESS | TARGET_NTFS_DISABLE_8DOT3
    )
}

/// Returns true when NTFS metadata items include compatibility warnings.
#[must_use]
pub fn ntfs_metadata_plan_has_compatibility_warnings(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if !is_ntfs_metadata_tweak_id(&item.tweak_id) {
            return true;
        }

        item.warnings.iter().any(|warning| {
            warning.contains("compatibility")
                || warning.contains("backup")
                || warning.contains("legacy")
                || warning.contains("compliance")
        })
    })
}

/// Returns true when a plan avoids blind cleanup deletion.
#[must_use]
pub fn storage_plan_has_no_blind_deletes(plan: &TweakPlan) -> bool {
    plan.items
        .iter()
        .filter(|item| item.tweak_id == STORAGE_TEMP_CLEANUP_TWEAK_ID)
        .flat_map(|item| item.changes.iter())
        .all(|change| {
            change.operation != TweakOperationKind::Delete
                && change.scope == SessionScope::RecommendationOnly
                && change.target.starts_with(CLEANUP_TARGET_PREFIX)
        })
}

fn ntfs_last_access_item(request: &NtfsMetadataPlanRequest) -> TweakPlanItem {
    let mut warnings = vec![
        "NTFS last-access changes require compatibility review for backup, audit, and compliance tools."
            .to_owned(),
    ];
    let action = if request.last_access_state.raw_value().is_none() {
        warnings.push("Current NTFS last-access fsutil state is unknown; rescan before apply.".to_owned());
        PlanAction::DetectOnly
    } else if request.last_access_state.is_disabled() {
        warnings.push("NTFS last-access updates are already disabled.".to_owned());
        PlanAction::DetectOnly
    } else if request.last_access_compatibility.has_known_dependency() {
        warnings.push("Known backup/compliance dependency blocks automatic last-access changes.".to_owned());
        PlanAction::Recommend
    } else if request.last_access_consent.is_granted()
        && request.last_access_compatibility.is_low_risk()
    {
        PlanAction::Apply
    } else {
        warnings.push(
            "Disabling last-access updates is prompt-only and needs explicit user consent."
                .to_owned(),
        );

        if !request.last_access_compatibility.is_low_risk() {
            warnings.push("Last-access compatibility is not proven low risk yet.".to_owned());
        }

        PlanAction::Recommend
    };

    let state = request.last_access_state.as_state();
    let changes = if matches!(action, PlanAction::Apply | PlanAction::Recommend)
        && !request.last_access_compatibility.has_known_dependency()
    {
        vec![write_change(
            TARGET_NTFS_DISABLE_LAST_ACCESS,
            state,
            DESIRED_DISABLE_LAST_ACCESS_VALUE,
        )]
    } else {
        vec![read_change(TARGET_NTFS_DISABLE_LAST_ACCESS, &state)]
    };

    ntfs_plan_item(
        STORAGE_NTFS_LAST_ACCESS_TWEAK_ID,
        action,
        TweakMode::Safe,
        changes,
        warnings,
    )
}

fn ntfs_eight_dot_three_item(request: &NtfsMetadataPlanRequest) -> TweakPlanItem {
    let mut warnings = vec![
        "NTFS 8.3 name creation changes can affect legacy installers and older applications."
            .to_owned(),
        "Disable 8.3 name creation only for future files after compatibility review.".to_owned(),
    ];
    let action = if request.eight_dot_three_state.raw_value().is_none() {
        warnings.push("Current NTFS 8.3 fsutil state is unknown; rescan before apply.".to_owned());
        PlanAction::DetectOnly
    } else if request.eight_dot_three_state.is_disabled_for_new_files() {
        warnings.push("NTFS 8.3 name creation is already disabled for future files.".to_owned());
        PlanAction::DetectOnly
    } else if request.eight_dot_three_compatibility.has_known_dependency() {
        warnings.push("Known legacy short-name dependency blocks automatic 8.3 changes.".to_owned());
        PlanAction::Recommend
    } else if request.eight_dot_three_consent.is_granted()
        && request.eight_dot_three_compatibility.is_low_risk()
    {
        PlanAction::Apply
    } else {
        warnings.push(
            "Disabling NTFS 8.3 name creation is prompt-only and needs explicit user consent."
                .to_owned(),
        );

        if !request.eight_dot_three_compatibility.is_low_risk() {
            warnings.push("8.3 legacy compatibility is not proven low risk yet.".to_owned());
        }

        PlanAction::Recommend
    };
    let mode = if request.requested_mode == TweakMode::Competitive {
        TweakMode::Competitive
    } else {
        TweakMode::Safe
    };
    let state = request.eight_dot_three_state.as_state();
    let changes = if matches!(action, PlanAction::Apply | PlanAction::Recommend)
        && !request.eight_dot_three_compatibility.has_known_dependency()
    {
        vec![write_change(
            TARGET_NTFS_DISABLE_8DOT3,
            state,
            DESIRED_DISABLE_8DOT3_VALUE,
        )]
    } else {
        vec![read_change(TARGET_NTFS_DISABLE_8DOT3, state)]
    };

    ntfs_plan_item(
        STORAGE_NTFS_8DOT3_TWEAK_ID,
        action,
        mode,
        changes,
        warnings,
    )
}

fn ntfs_plan_item(
    tweak_id: &str,
    action: PlanAction,
    mode: TweakMode,
    changes: Vec<PlannedChange>,
    warnings: Vec<String>,
) -> TweakPlanItem {
    let backup = if action == PlanAction::Apply {
        BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: changes
                .first()
                .map_or_else(|| tweak_id.to_owned(), |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    };
    let rollback = if action == PlanAction::Apply {
        exact_value_rollback("Restore previous NTFS fsutil value.", &changes, true)
    } else {
        RollbackPlan::not_needed()
    };

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::Storage,
        action,
        mode,
        risk: TweakRisk::Medium,
        changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin: action == PlanAction::Apply,
        warnings,
    }
}

fn cleanup_preview_item(request: &StorageReadinessPlanRequest) -> TweakPlanItem {
    let safe_candidates = request.cleanup_preview.safe_candidates().collect::<Vec<_>>();
    let mut warnings = vec![concat!(
        "Preview-only cleanup: game content, downloads, documents, profiles, ",
        "and in-use files stay excluded."
    )
    .to_owned()];

    if request
        .cleanup_preview
        .candidates
        .iter()
        .any(|candidate| !candidate.safe_to_preview)
    {
        warnings.push("Unsafe cleanup candidates were excluded from the plan.".to_owned());
    }

    let changes = safe_candidates
        .into_iter()
        .map(cleanup_preview_change)
        .collect::<Vec<_>>();
    let action = if changes.is_empty() {
        PlanAction::DetectOnly
    } else {
        PlanAction::Recommend
    };

    plan_item(
        STORAGE_TEMP_CLEANUP_TWEAK_ID,
        action,
        TweakRisk::Low,
        changes,
        BackupRequirement::NotRequired,
        RollbackPlan::not_needed(),
        false,
        warnings,
    )
}

fn storage_sense_item(request: &StorageReadinessPlanRequest) -> TweakPlanItem {
    let changes = storage_sense_changes(request);
    let already_configured = request
        .storage_sense
        .matches_preference(request.desired_storage_sense);
    let mut warnings = Vec::new();
    let action = if already_configured {
        PlanAction::DetectOnly
    } else if request.storage_sense_consent.is_granted() {
        PlanAction::Apply
    } else {
        warnings.push("Storage Sense configuration is prompt-only and requires backup.".to_owned());
        PlanAction::Recommend
    };

    let backup = if action == PlanAction::Apply {
        BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: TARGET_STORAGE_SENSE_ENABLED.to_owned(),
        }
    } else {
        BackupRequirement::NotRequired
    };

    let rollback = if action == PlanAction::Apply {
        exact_value_rollback(
            "Restore previous Storage Sense setting.",
            &changes,
            false,
        )
    } else {
        RollbackPlan::not_needed()
    };

    plan_item(
        STORAGE_SENSE_CONFIGURE_TWEAK_ID,
        action,
        TweakRisk::Low,
        if already_configured { Vec::new() } else { changes },
        backup,
        rollback,
        false,
        warnings,
    )
}

fn trim_verify_item(request: &StorageReadinessPlanRequest) -> TweakPlanItem {
    let mut warnings = vec!["Do not force SSD defrag; use Windows-supported optimize paths only.".to_owned()];
    let needs_review = request.trim.solid_state_media_present
        && (request.trim.ntfs == TrimState::Disabled
            || request.trim.refs == TrimState::Disabled
            || !request.trim.optimize_volume_available);

    if !request.trim.solid_state_media_present {
        warnings.push("No SSD/NVMe media was detected for TRIM optimization.".to_owned());
    }

    let changes = vec![
        read_change(
            TARGET_TRIM_NTFS_DISABLE_DELETE_NOTIFY,
            request.trim.ntfs.as_state(),
        ),
        read_change(
            TARGET_TRIM_REFS_DISABLE_DELETE_NOTIFY,
            request.trim.refs.as_state(),
        ),
        manual_change(
            TARGET_OPTIMIZE_VOLUME_SUPPORTED,
            if request.trim.optimize_volume_available {
                "optimize_volume_available"
            } else {
                "optimize_volume_unavailable"
            },
        ),
    ];

    plan_item(
        STORAGE_TRIM_VERIFY_TWEAK_ID,
        if needs_review {
            PlanAction::Recommend
        } else {
            PlanAction::DetectOnly
        },
        TweakRisk::Low,
        changes,
        BackupRequirement::NotRequired,
        RollbackPlan::not_needed(),
        false,
        warnings,
    )
}

fn direct_storage_item(request: &StorageReadinessPlanRequest) -> TweakPlanItem {
    let mut warnings = Vec::new();
    let action = if request.direct_storage.has_known_blocker() {
        warnings.push("DirectStorage readiness has a known blocker on this system.".to_owned());
        PlanAction::Recommend
    } else {
        PlanAction::DetectOnly
    };

    if !request.direct_storage.is_ready() && !request.direct_storage.has_known_blocker() {
        warnings.push("DirectStorage readiness has unknown GPU or game-location signals.".to_owned());
    }

    plan_item(
        STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID,
        action,
        TweakRisk::Low,
        vec![read_change(
            TARGET_DIRECTSTORAGE_READINESS,
            direct_storage_state(request.direct_storage),
        )],
        BackupRequirement::NotRequired,
        RollbackPlan::not_needed(),
        false,
        warnings,
    )
}

fn nvme_driver_hack_guardrail_item(request: &StorageReadinessPlanRequest) -> TweakPlanItem {
    if request.unsupported_nvme_driver_hack_requested {
        plan_item(
            STORAGE_NVME_DRIVER_HACK_TWEAK_ID,
            PlanAction::Deny,
            TweakRisk::Critical,
            vec![PlannedChange {
                target: TARGET_UNSUPPORTED_NVME_DRIVER_HACK.to_owned(),
                operation: TweakOperationKind::Deny,
                previous_value: None,
                desired_value: None,
                scope: SessionScope::Blocked,
            }],
            BackupRequirement::NotRequired,
            RollbackPlan::not_needed(),
            true,
            vec!["Unsupported NVMe/server-driver registry hacks are denied.".to_owned()],
        )
    } else {
        plan_item(
            STORAGE_NVME_DRIVER_HACK_TWEAK_ID,
            PlanAction::DetectOnly,
            TweakRisk::Critical,
            Vec::new(),
            BackupRequirement::NotRequired,
            RollbackPlan::not_needed(),
            false,
            Vec::new(),
        )
    }
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    backup: BackupRequirement,
    rollback: RollbackPlan,
    requires_admin: bool,
    warnings: Vec<String>,
) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: if tweak_id == STORAGE_NVME_DRIVER_HACK_TWEAK_ID {
            TweakCategory::BlockedGuardrail
        } else {
            TweakCategory::Storage
        },
        action,
        mode: if tweak_id == STORAGE_NVME_DRIVER_HACK_TWEAK_ID {
            TweakMode::Blocked
        } else {
            TweakMode::Safe
        },
        risk,
        changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin,
        warnings,
    }
}

fn cleanup_preview_change(candidate: &CleanupCandidate) -> PlannedChange {
    PlannedChange {
        target: format!("{CLEANUP_TARGET_PREFIX}{}", target_slug(&candidate.target)),
        operation: TweakOperationKind::Manual,
        previous_value: Some(format!(
            "kind={},bytes={},files={},path={}",
            candidate.kind.as_state(),
            candidate.reclaimable_bytes,
            candidate.file_count,
            candidate.path
        )),
        desired_value: Some("review_safe_temp_cleanup".to_owned()),
        scope: SessionScope::RecommendationOnly,
    }
}

fn storage_sense_changes(request: &StorageReadinessPlanRequest) -> Vec<PlannedChange> {
    let preference = request.desired_storage_sense;
    let inspection = request.storage_sense;
    let mut changes = vec![
        write_change(
            TARGET_STORAGE_SENSE_ENABLED,
            inspection.state.as_previous_value(),
            if preference.enabled { "1" } else { "0" },
        ),
        write_change(
            TARGET_STORAGE_SENSE_CADENCE,
            optional_u32_state(inspection.cadence_days),
            preference.cadence_days.to_string(),
        ),
        write_change(
            TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS,
            optional_u32_state(inspection.recycle_bin_days),
            preference.recycle_bin_days.to_string(),
        ),
    ];

    if let Some(downloads_days) = preference.downloads_days {
        changes.push(write_change(
            TARGET_STORAGE_SENSE_DOWNLOADS_DAYS,
            optional_u32_state(inspection.downloads_days),
            downloads_days.to_string(),
        ));
    }

    changes
}

fn write_change(
    target: &str,
    previous_value: impl Into<String>,
    desired_value: impl Into<String>,
) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value: Some(previous_value.into()),
        desired_value: Some(desired_value.into()),
        scope: SessionScope::Persistent,
    }
}

fn read_change(target: &str, value: &str) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Read,
        previous_value: Some(value.to_owned()),
        desired_value: None,
        scope: SessionScope::RecommendationOnly,
    }
}

fn manual_change(target: &str, value: &str) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Manual,
        previous_value: Some(value.to_owned()),
        desired_value: Some("use_windows_supported_optimize".to_owned()),
        scope: SessionScope::RecommendationOnly,
    }
}

fn exact_value_rollback(
    summary: &str,
    changes: &[PlannedChange],
    requires_admin: bool,
) -> RollbackPlan {
    RollbackPlan {
        kind: RollbackKind::ExactValue,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: summary.to_owned(),
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

fn optional_u32_state(value: Option<u32>) -> String {
    value.map_or_else(|| "unknown".to_owned(), |value| value.to_string())
}

fn direct_storage_state(inspection: DirectStorageInspection) -> &'static str {
    if inspection.is_ready() {
        "ready"
    } else if inspection.has_known_blocker() {
        "blocked"
    } else {
        "unknown"
    }
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
        "unknown".to_owned()
    } else {
        slug.to_owned()
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
    fn ntfs_metadata_is_prompt_only_until_compatibility_and_consent() {
        let mut request = NtfsMetadataPlanRequest::new("plan-ntfs-prompt");
        request.last_access_state = NtfsLastAccessState::from_disable_last_access_value(Some(0));
        request.eight_dot_three_state =
            NtfsEightDotThreeNameState::from_disable_8dot3_value(Some(0));

        let plan = build_ntfs_metadata_plan(&request);
        let last_access = item(&plan, STORAGE_NTFS_LAST_ACCESS_TWEAK_ID);
        let eight_dot_three = item(&plan, STORAGE_NTFS_8DOT3_TWEAK_ID);

        assert_eq!(last_access.action, PlanAction::Recommend);
        assert_eq!(eight_dot_three.action, PlanAction::Recommend);
        assert_eq!(last_access.backup, BackupRequirement::NotRequired);
        assert!(ntfs_metadata_plan_has_compatibility_warnings(&plan));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("compatibility")));
    }

    #[test]
    fn consented_ntfs_metadata_plan_is_backup_and_rollback_capable() {
        let mut request = NtfsMetadataPlanRequest::new("plan-ntfs-apply");
        request.last_access_state = NtfsLastAccessState::from_disable_last_access_value(Some(2));
        request.last_access_consent = StorageControlConsent::Granted;
        request.last_access_compatibility = NtfsMetadataCompatibility::LowRisk;
        request.eight_dot_three_state =
            NtfsEightDotThreeNameState::from_disable_8dot3_value(Some(0));
        request.eight_dot_three_consent = StorageControlConsent::Granted;
        request.eight_dot_three_compatibility = NtfsMetadataCompatibility::LowRisk;

        let plan = build_ntfs_metadata_plan(&request);
        let last_access = item(&plan, STORAGE_NTFS_LAST_ACCESS_TWEAK_ID);
        let eight_dot_three = item(&plan, STORAGE_NTFS_8DOT3_TWEAK_ID);

        assert_eq!(last_access.action, PlanAction::Apply);
        assert_eq!(eight_dot_three.action, PlanAction::Apply);
        assert_eq!(
            last_access.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_NTFS_DISABLE_LAST_ACCESS.to_owned(),
            }
        );
        assert_eq!(last_access.rollback.steps.len(), 1);
        assert_eq!(eight_dot_three.rollback.steps.len(), 1);
        assert!(last_access.requires_admin);
        assert!(eight_dot_three
            .changes
            .iter()
            .all(|change| is_ntfs_metadata_fsutil_target(&change.target)));
    }

    #[test]
    fn ntfs_metadata_blocks_known_legacy_dependencies_from_apply() {
        let mut request = NtfsMetadataPlanRequest::new("plan-ntfs-legacy");
        request.last_access_state = NtfsLastAccessState::from_disable_last_access_value(Some(0));
        request.last_access_consent = StorageControlConsent::Granted;
        request.last_access_compatibility = NtfsMetadataCompatibility::KnownDependency;
        request.eight_dot_three_state =
            NtfsEightDotThreeNameState::from_disable_8dot3_value(Some(2));
        request.eight_dot_three_consent = StorageControlConsent::Granted;
        request.eight_dot_three_compatibility = NtfsMetadataCompatibility::KnownDependency;

        let plan = build_ntfs_metadata_plan(&request);
        let last_access = item(&plan, STORAGE_NTFS_LAST_ACCESS_TWEAK_ID);
        let eight_dot_three = item(&plan, STORAGE_NTFS_8DOT3_TWEAK_ID);

        assert_eq!(last_access.action, PlanAction::Recommend);
        assert_eq!(eight_dot_three.action, PlanAction::Recommend);
        assert!(!plan.has_apply_items());
        assert!(last_access
            .warnings
            .iter()
            .any(|warning| warning.contains("blocks automatic")));
        assert_eq!(eight_dot_three.changes[0].operation, TweakOperationKind::Read);
    }

    #[test]
    fn cleanup_preview_recommends_without_blind_delete_actions() {
        let mut request = StorageReadinessPlanRequest::new("plan-storage");
        request.cleanup_preview = CleanupPreview {
            candidates: vec![
                CleanupCandidate::safe(
                    "user-temp",
                    "C:\\Users\\Liiiraa\\AppData\\Local\\Temp",
                    CleanupCandidateKind::UserTemp,
                    128 * 1024 * 1024,
                    42,
                ),
                CleanupCandidate::excluded(
                    "downloads",
                    "C:\\Users\\Liiiraa\\Downloads",
                    CleanupCandidateKind::Other,
                ),
            ],
            excluded_patterns: vec!["Downloads".to_owned(), "game install folders".to_owned()],
        };

        let plan = build_storage_readiness_plan(&request);
        let cleanup = item(&plan, STORAGE_TEMP_CLEANUP_TWEAK_ID);

        assert_eq!(cleanup.action, PlanAction::Recommend);
        assert_eq!(cleanup.changes.len(), 1);
        assert_eq!(cleanup.changes[0].operation, TweakOperationKind::Manual);
        assert_eq!(cleanup.changes[0].scope, SessionScope::RecommendationOnly);
        assert!(storage_plan_has_no_blind_deletes(&plan));
    }

    #[test]
    fn storage_sense_requires_prompt_before_apply_and_backup() {
        let mut request = StorageReadinessPlanRequest::new("plan-storage-sense");
        request.storage_sense = StorageSenseInspection {
            state: StorageSenseState::Disabled,
            cadence_days: Some(7),
            recycle_bin_days: Some(14),
            downloads_days: None,
        };

        let prompted = build_storage_readiness_plan(&request);
        let prompted_item = item(&prompted, STORAGE_SENSE_CONFIGURE_TWEAK_ID);

        assert_eq!(prompted_item.action, PlanAction::Recommend);
        assert_eq!(prompted_item.backup, BackupRequirement::NotRequired);

        request.storage_sense_consent = StorageControlConsent::Granted;
        let consented = build_storage_readiness_plan(&request);
        let consented_item = item(&consented, STORAGE_SENSE_CONFIGURE_TWEAK_ID);

        assert_eq!(consented_item.action, PlanAction::Apply);
        assert_eq!(consented_item.backup, BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: TARGET_STORAGE_SENSE_ENABLED.to_owned(),
        });
        assert_eq!(consented_item.rollback.steps.len(), 3);
        assert!(consented_item
            .changes
            .iter()
            .all(|change| is_storage_sense_registry_target(&change.target)));
    }

    #[test]
    fn trim_verify_recommends_windows_supported_optimize_only() {
        let mut request = StorageReadinessPlanRequest::new("plan-trim");
        request.trim = TrimInspection {
            ntfs: TrimState::Disabled,
            refs: TrimState::Unknown,
            optimize_volume_available: true,
            solid_state_media_present: true,
        };

        let plan = build_storage_readiness_plan(&request);
        let trim = item(&plan, STORAGE_TRIM_VERIFY_TWEAK_ID);

        assert_eq!(trim.action, PlanAction::Recommend);
        assert!(trim.changes.iter().all(|change| {
            change.operation == TweakOperationKind::Read
                || change.operation == TweakOperationKind::Manual
        }));
        assert!(trim
            .warnings
            .iter()
            .any(|warning| warning.contains("Do not force SSD defrag")));
    }

    #[test]
    fn directstorage_check_is_read_only_and_reports_blockers() {
        let mut request = StorageReadinessPlanRequest::new("plan-directstorage");
        request.direct_storage = DirectStorageInspection {
            os_supported: Some(true),
            nvme_present: Some(false),
            game_on_nvme: None,
            gpu_decompression_supported: None,
        };

        let plan = build_storage_readiness_plan(&request);
        let direct_storage = item(&plan, STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID);

        assert_eq!(direct_storage.action, PlanAction::Recommend);
        assert_eq!(direct_storage.changes[0].operation, TweakOperationKind::Read);
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn unsupported_nvme_driver_hack_is_denied() {
        let mut request = StorageReadinessPlanRequest::new("plan-nvme-guardrail");
        request.unsupported_nvme_driver_hack_requested = true;

        let plan = build_storage_readiness_plan(&request);
        let guardrail = item(&plan, STORAGE_NVME_DRIVER_HACK_TWEAK_ID);

        assert_eq!(guardrail.action, PlanAction::Deny);
        assert_eq!(guardrail.mode, TweakMode::Blocked);
        assert_eq!(guardrail.changes[0].operation, TweakOperationKind::Deny);
        assert!(plan.has_denials());
    }
}
