//! Windows scan and registry-fixture adapters for T044 storage readiness checks.

use std::fmt;

use optimizer_core::{
    storage::{
        build_ntfs_metadata_plan, build_storage_readiness_plan, is_ntfs_metadata_fsutil_target,
        is_ntfs_metadata_tweak_id, is_storage_sense_registry_target,
        ntfs_metadata_plan_has_compatibility_warnings, CleanupCandidate, CleanupCandidateKind,
        CleanupPreview, DirectStorageInspection, NtfsEightDotThreeNameState, NtfsLastAccessState,
        NtfsMetadataCompatibility, NtfsMetadataPlanRequest, StorageControlConsent,
        StorageReadinessPlanRequest, StorageSenseInspection, StorageSenseState, TrimInspection,
        TrimState, STORAGE_NTFS_8DOT3_TWEAK_ID, STORAGE_NTFS_LAST_ACCESS_TWEAK_ID,
        STORAGE_SENSE_CONFIGURE_TWEAK_ID, TARGET_NTFS_DISABLE_8DOT3,
        TARGET_NTFS_DISABLE_LAST_ACCESS,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{
    PhysicalDiskScanItem, StorageCleanupCandidateScanItem, StorageSenseScan, StorageTrimScan,
    SystemScanReport, WindowsRollbackFixture,
};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSenseRegistrySummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Registry targets written or verified.
    pub targets: Vec<String>,
}

impl StorageSenseRegistrySummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Summary for fixture-backed NTFS metadata apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtfsMetadataSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Fsutil-backed NTFS metadata targets written or verified.
    pub targets: Vec<String>,
}

impl NtfsMetadataSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T044 storage readiness plan from a system scan.
#[must_use]
pub fn build_storage_readiness_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let mut request = StorageReadinessPlanRequest::new(plan_id);
    request.cleanup_preview = CleanupPreview {
        candidates: report
            .storage
            .cleanup
            .candidates
            .iter()
            .map(cleanup_candidate_from_scan)
            .collect(),
        excluded_patterns: report.storage.cleanup.excluded_patterns.clone(),
    };
    request.storage_sense = storage_sense_from_scan(&report.storage.storage_sense);
    request.trim = trim_from_scan(&report.storage.trim, &report.storage.physical_disks);
    request.direct_storage = direct_storage_from_scan(report);

    build_storage_readiness_plan(&request)
}

/// Builds a consented T044 Storage Sense plan from a system scan.
#[must_use]
pub fn build_consented_storage_sense_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let mut request = StorageReadinessPlanRequest::new(plan_id);
    request.storage_sense = storage_sense_from_scan(&report.storage.storage_sense);
    request.storage_sense_consent = StorageControlConsent::Granted;

    build_storage_readiness_plan(&request)
}

/// Builds a T054 NTFS metadata plan from read-only fsutil scan data.
#[must_use]
pub fn build_ntfs_metadata_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    build_ntfs_metadata_plan(&ntfs_metadata_request_from_scan(plan_id, report))
}

/// Builds a consented T054 NTFS metadata plan after compatibility review.
#[must_use]
pub fn build_consented_ntfs_metadata_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let mut request = ntfs_metadata_request_from_scan(plan_id, report);
    request.last_access_consent = StorageControlConsent::Granted;
    request.last_access_compatibility = NtfsMetadataCompatibility::LowRisk;
    request.eight_dot_three_consent = StorageControlConsent::Granted;
    request.eight_dot_three_compatibility = NtfsMetadataCompatibility::LowRisk;

    build_ntfs_metadata_plan(&request)
}

/// Applies Storage Sense registry changes to an in-memory Windows fixture.
pub fn apply_storage_sense_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<StorageSenseRegistrySummary, StorageSenseRegistryError> {
    let mut summary = StorageSenseRegistrySummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                StorageSenseRegistryError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            fixture.set_value(change.target.clone(), desired.to_owned());
            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Verifies Storage Sense registry changes against an in-memory fixture.
pub fn verify_storage_sense_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<StorageSenseRegistrySummary, StorageSenseRegistryError> {
    let mut summary = StorageSenseRegistrySummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                StorageSenseRegistryError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(StorageSenseRegistryError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Applies T054 NTFS metadata fsutil changes to an in-memory Windows fixture.
pub fn apply_ntfs_metadata_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<NtfsMetadataSettingsSummary, NtfsMetadataSettingsError> {
    validate_ntfs_metadata_plan(plan)?;

    let mut summary = NtfsMetadataSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_ntfs_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_ntfs_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                NtfsMetadataSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            fixture.set_value(change.target.clone(), desired.to_owned());
            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Verifies T054 NTFS metadata fsutil changes against an in-memory Windows fixture.
pub fn verify_ntfs_metadata_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<NtfsMetadataSettingsSummary, NtfsMetadataSettingsError> {
    validate_ntfs_metadata_plan(plan)?;

    let mut summary = NtfsMetadataSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_ntfs_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_ntfs_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                NtfsMetadataSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(NtfsMetadataSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn cleanup_candidate_from_scan(item: &StorageCleanupCandidateScanItem) -> CleanupCandidate {
    if item.safe_to_preview {
        CleanupCandidate::safe(
            item.target.clone(),
            item.path.clone(),
            cleanup_kind(&item.kind),
            item.reclaimable_bytes,
            item.file_count,
        )
    } else {
        CleanupCandidate::excluded(item.target.clone(), item.path.clone(), cleanup_kind(&item.kind))
    }
}

fn cleanup_kind(value: &str) -> CleanupCandidateKind {
    match normalized(value).as_str() {
        "usertemp" | "user_temp" => CleanupCandidateKind::UserTemp,
        "windowstemp" | "windows_temp" => CleanupCandidateKind::WindowsTemp,
        "shadercache" | "shader_cache" => CleanupCandidateKind::ShaderCache,
        _ => CleanupCandidateKind::Other,
    }
}

fn storage_sense_from_scan(scan: &StorageSenseScan) -> StorageSenseInspection {
    StorageSenseInspection {
        state: match scan.enabled {
            Some(true) => StorageSenseState::Enabled,
            Some(false) => StorageSenseState::Disabled,
            None => StorageSenseState::Unknown,
        },
        cadence_days: scan.cadence_days,
        recycle_bin_days: scan.recycle_bin_cleanup_days,
        downloads_days: scan.downloads_cleanup_days,
    }
}

fn trim_from_scan(scan: &StorageTrimScan, disks: &[PhysicalDiskScanItem]) -> TrimInspection {
    TrimInspection {
        ntfs: TrimState::from_disable_delete_notify(scan.ntfs_disable_delete_notify),
        refs: TrimState::from_disable_delete_notify(scan.refs_disable_delete_notify),
        optimize_volume_available: scan.optimize_volume_available,
        solid_state_media_present: disks.iter().any(disk_is_solid_state),
    }
}

fn direct_storage_from_scan(report: &SystemScanReport) -> DirectStorageInspection {
    let os_supported = report
        .os
        .build_number
        .parse::<u32>()
        .ok()
        .map(|build| build >= 19041)
        .or(report.storage.direct_storage.os_supported);
    let nvme_present = if report.storage.physical_disks.is_empty() {
        report.storage.direct_storage.nvme_present
    } else {
        Some(report.storage.physical_disks.iter().any(disk_is_nvme))
    };
    let game_on_nvme = report
        .storage
        .direct_storage
        .game_volume_bus_type
        .as_deref()
        .map(|bus_type| normalized(bus_type).contains("nvme"));

    DirectStorageInspection {
        os_supported,
        nvme_present,
        game_on_nvme,
        gpu_decompression_supported: report.storage.direct_storage.gpu_decompression_supported,
    }
}

fn ntfs_metadata_request_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> NtfsMetadataPlanRequest {
    let mut request = NtfsMetadataPlanRequest::new(plan_id);
    request.last_access_state = NtfsLastAccessState::from_disable_last_access_value(
        report.storage.ntfs_metadata.disable_last_access,
    );
    request.eight_dot_three_state = NtfsEightDotThreeNameState::from_disable_8dot3_value(
        report.storage.ntfs_metadata.disable_8dot3,
    );
    request
}

/// Parses the fsutil DisableLastAccess value from command output.
#[must_use]
pub fn parse_ntfs_last_access_fsutil_value(output: &str) -> Option<u32> {
    output
        .lines()
        .find(|line| normalized(line).contains("disablelastaccess"))
        .and_then(parse_u32_after_separator)
}

/// Parses the fsutil Disable8dot3 registry value from command output.
#[must_use]
pub fn parse_ntfs_8dot3_fsutil_value(output: &str) -> Option<u32> {
    output
        .lines()
        .find(|line| {
            let normalized = normalized(line);
            normalized.contains("registrystate") || normalized.contains("disable8dot3")
        })
        .and_then(parse_u32_after_separator)
}

fn disk_is_solid_state(disk: &PhysicalDiskScanItem) -> bool {
    disk.media_type
        .as_deref()
        .is_some_and(|media_type| normalized(media_type).contains("ssd"))
        || disk_is_nvme(disk)
}

fn disk_is_nvme(disk: &PhysicalDiskScanItem) -> bool {
    disk.bus_type
        .as_deref()
        .is_some_and(|bus_type| normalized(bus_type).contains("nvme"))
}

fn validate_ntfs_metadata_plan(plan: &TweakPlan) -> Result<(), NtfsMetadataSettingsError> {
    if ntfs_metadata_plan_has_compatibility_warnings(plan) {
        Ok(())
    } else {
        Err(NtfsMetadataSettingsError::missing_compatibility_warning())
    }
}

fn validate_ntfs_tweak_id(tweak_id: &str) -> Result<(), NtfsMetadataSettingsError> {
    if is_ntfs_metadata_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(NtfsMetadataSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_ntfs_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), NtfsMetadataSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(NtfsMetadataSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_ntfs_metadata_fsutil_target(&change.target) {
        return Err(NtfsMetadataSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), StorageSenseRegistryError> {
    if tweak_id == STORAGE_SENSE_CONFIGURE_TWEAK_ID {
        Ok(())
    } else {
        Err(StorageSenseRegistryError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), StorageSenseRegistryError> {
    if change.operation != TweakOperationKind::Write {
        return Err(StorageSenseRegistryError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_storage_sense_registry_target(&change.target) {
        return Err(StorageSenseRegistryError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn parse_u32_after_separator(value: &str) -> Option<u32> {
    let after_separator = value
        .split_once('=')
        .or_else(|| value.split_once(':'))
        .map_or(value, |(_, suffix)| suffix);

    let digits = after_separator
        .chars()
        .skip_while(|character| !character.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

/// Stable failure reason for fixture-backed Storage Sense operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageSenseRegistryErrorReason {
    /// Plan item was not part of the T044 Storage Sense scope.
    UnsupportedTweak,
    /// Plan item targeted an unsupported registry value.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
}

impl StorageSenseRegistryErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-Storage Sense tweak",
            Self::UnsupportedTarget => "Plan targets a registry value outside the T044 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Registry fixture readback did not match the plan",
        }
    }
}

/// Structured error for fixture-backed Storage Sense registry operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSenseRegistryError {
    reason: StorageSenseRegistryErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl StorageSenseRegistryError {
    fn new(
        reason: StorageSenseRegistryErrorReason,
        tweak_id: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            reason,
            tweak_id,
            target,
        }
    }

    fn unsupported_tweak(tweak_id: impl Into<String>) -> Self {
        Self::new(
            StorageSenseRegistryErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            StorageSenseRegistryErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            StorageSenseRegistryErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            StorageSenseRegistryErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            StorageSenseRegistryErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> StorageSenseRegistryErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected registry target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for StorageSenseRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for StorageSenseRegistryError {}

/// Stable failure reason for fixture-backed NTFS metadata operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtfsMetadataSettingsErrorReason {
    /// Plan item was not part of the T054 NTFS metadata scope.
    UnsupportedTweak,
    /// Plan item targeted an fsutil value outside the T054 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired value.
    VerificationFailed,
    /// Plan lost required compatibility warnings.
    MissingCompatibilityWarning,
}

impl NtfsMetadataSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::MissingCompatibilityWarning => "missing_compatibility_warning",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-NTFS metadata tweak",
            Self::UnsupportedTarget => "Plan targets an fsutil value outside the T054 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "NTFS metadata fixture readback did not match the plan",
            Self::MissingCompatibilityWarning => {
                "NTFS metadata plans must include compatibility warnings"
            }
        }
    }
}

/// Structured error for fixture-backed NTFS metadata operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NtfsMetadataSettingsError {
    reason: NtfsMetadataSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl NtfsMetadataSettingsError {
    fn new(
        reason: NtfsMetadataSettingsErrorReason,
        tweak_id: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            reason,
            tweak_id,
            target,
        }
    }

    fn unsupported_tweak(tweak_id: impl Into<String>) -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_compatibility_warning() -> Self {
        Self::new(
            NtfsMetadataSettingsErrorReason::MissingCompatibilityWarning,
            None,
            None,
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> NtfsMetadataSettingsErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected fsutil target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for NtfsMetadataSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for NtfsMetadataSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        storage::{
            storage_plan_has_no_blind_deletes, STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID,
            STORAGE_TEMP_CLEANUP_TWEAK_ID, TARGET_STORAGE_SENSE_CADENCE,
            TARGET_STORAGE_SENSE_ENABLED, TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS,
        },
        tweak_contracts::{PlanAction, RollbackStatus},
    };

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");

    fn item<'a>(
        plan: &'a TweakPlan,
        tweak_id: &str,
    ) -> &'a optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn scan_fixture_builds_storage_readiness_plan() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_storage_readiness_plan_from_scan("plan-t044-fixture", &report);
        let cleanup = item(&plan, STORAGE_TEMP_CLEANUP_TWEAK_ID);
        let direct_storage = item(&plan, STORAGE_DIRECTSTORAGE_CHECK_TWEAK_ID);

        assert_eq!(cleanup.action, PlanAction::Recommend);
        assert!(storage_plan_has_no_blind_deletes(&plan));
        assert_eq!(direct_storage.action, PlanAction::DetectOnly);
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn storage_sense_fixture_applies_verifies_and_rolls_back_values() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_consented_storage_sense_plan_from_scan("plan-storage-sense", &report);
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_STORAGE_SENSE_ENABLED, "0")
            .with_value(TARGET_STORAGE_SENSE_CADENCE, "7")
            .with_value(TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS, "14");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("backup should be captured");
        assert_eq!(backups.len(), 1);

        let applied = apply_storage_sense_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");
        assert_eq!(applied.item_count, 1);
        assert_eq!(fixture.value(TARGET_STORAGE_SENSE_ENABLED), Some("1"));
        assert_eq!(fixture.value(TARGET_STORAGE_SENSE_CADENCE), Some("30"));
        verify_storage_sense_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");

        let item = item(&plan, STORAGE_SENSE_CONFIGURE_TWEAK_ID);
        let rollback_request = RollbackRequest::new(
            STORAGE_SENSE_CONFIGURE_TWEAK_ID,
            backups[0].clone(),
            item.rollback.clone(),
        )
        .expect("rollback request should be valid");
        let rollback = execute_rollback(&mut fixture, &rollback_request)
            .expect("rollback should restore previous registry values");

        assert_eq!(rollback.status, RollbackStatus::Restored);
        assert_eq!(fixture.value(TARGET_STORAGE_SENSE_ENABLED), Some("0"));
        assert_eq!(fixture.value(TARGET_STORAGE_SENSE_CADENCE), Some("7"));
    }

    #[test]
    fn fsutil_fixtures_parse_ntfs_metadata_values() {
        let last_access = r"
DisableLastAccess = 3  (System Managed, Disabled)
";
        let eight_dot_three = r"
The registry state is: 2 (Per volume setting - the default).
Based on the above two settings, 8dot3 name creation is enabled on C:
";
        let behavior_style = "Disable8dot3 = 1";

        assert_eq!(parse_ntfs_last_access_fsutil_value(last_access), Some(3));
        assert_eq!(parse_ntfs_8dot3_fsutil_value(eight_dot_three), Some(2));
        assert_eq!(parse_ntfs_8dot3_fsutil_value(behavior_style), Some(1));
    }

    #[test]
    fn scan_fixture_builds_ntfs_metadata_prompt_plan() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_ntfs_metadata_plan_from_scan("plan-t054-fixture", &report);
        let last_access = item(&plan, STORAGE_NTFS_LAST_ACCESS_TWEAK_ID);
        let eight_dot_three = item(&plan, STORAGE_NTFS_8DOT3_TWEAK_ID);

        assert_eq!(last_access.action, PlanAction::Recommend);
        assert_eq!(eight_dot_three.action, PlanAction::Recommend);
        assert!(ntfs_metadata_plan_has_compatibility_warnings(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn ntfs_metadata_fixture_applies_verifies_and_rolls_back_values() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_consented_ntfs_metadata_plan_from_scan("plan-t054-consented", &report);
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_NTFS_DISABLE_LAST_ACCESS, "enabled:0")
            .with_value(TARGET_NTFS_DISABLE_8DOT3, "enabled_all:0");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("ntfs backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_ntfs_metadata_plan_to_fixture(&mut fixture, &plan)
            .expect("fixture apply should succeed");
        assert_eq!(applied.item_count, 2);
        assert_eq!(fixture.value(TARGET_NTFS_DISABLE_LAST_ACCESS), Some("1"));
        assert_eq!(fixture.value(TARGET_NTFS_DISABLE_8DOT3), Some("1"));

        verify_ntfs_metadata_plan_fixture(&fixture, &plan)
            .expect("fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore ntfs fixture state");
        }

        assert_eq!(
            fixture.value(TARGET_NTFS_DISABLE_LAST_ACCESS),
            Some("enabled:0")
        );
        assert_eq!(fixture.value(TARGET_NTFS_DISABLE_8DOT3), Some("enabled_all:0"));
    }
}
