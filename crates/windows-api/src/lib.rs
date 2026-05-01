//! Windows implementation adapters for registry, power, services, and devices.

pub mod backup;
pub mod background_work;
pub mod command;
pub mod defender;
pub mod gaming;
pub mod graphics;
pub mod network;
pub mod power;
pub mod scan;
pub mod scheduler;
pub mod security_tradeoff;
pub mod storage;
pub mod windows_lab;
pub mod windows_update;

pub use backup::WindowsRollbackFixture;
pub use background_work::{
    apply_background_service_plan_to_fixture,
    background_services_plan_blocks_searchapp_rename,
    background_work_plan_is_recommendation_only, build_background_services_plan_from_scan,
    build_background_work_plan_from_scan, build_consented_background_services_plan_from_scan,
    verify_background_service_plan_fixture, BackgroundServiceSettingsError,
    BackgroundServiceSettingsErrorReason, BackgroundServiceSettingsSummary,
};
pub use command::{
    FixedWindowsExecutable, StructuredCommandPlan, WindowsArgument, WindowsCommandPlanError,
};
pub use defender::{
    apply_defender_performance_plan_to_fixture, build_defender_performance_plan_from_scan,
    defender_plan_blocks_global_disable, verify_defender_performance_plan_fixture,
    DefenderSettingsError, DefenderSettingsErrorReason, DefenderSettingsSummary,
};
pub use gaming::{
    apply_gaming_capture_plan_to_fixture, verify_gaming_capture_plan_fixture,
    GamingCaptureRegistryError, GamingCaptureRegistryErrorReason,
    GamingCaptureRegistrySummary,
};
pub use graphics::{
    apply_graphics_settings_plan_to_fixture, build_consented_graphics_settings_plan_from_scan,
    build_graphics_settings_plan_from_scan, verify_graphics_settings_plan_fixture,
    GraphicsSettingsError, GraphicsSettingsErrorReason, GraphicsSettingsSummary,
};
pub use network::{
    apply_network_adapter_power_plan_to_fixture, apply_network_advanced_tuning_plan_to_fixture,
    build_consented_network_adapter_power_plan_from_scan,
    build_consented_network_advanced_tuning_plan_from_scan,
    build_network_adapter_power_plan_from_scan, build_network_advanced_tuning_plan_from_scan,
    verify_network_adapter_power_plan_fixture, verify_network_advanced_tuning_plan_fixture,
    NetworkAdapterSettingsError, NetworkAdapterSettingsErrorReason, NetworkAdapterSettingsSummary,
};
pub use power::{
    active_power_scheme_matches, build_liiiraa_powercfg_plan, parse_active_power_scheme_guid,
    LiiiraaPowerCfgApplyPlan, LiiiraaPowerCfgRollbackPlan, LiiiraaPowerPlanApplyRequest,
    PowerPlanApplyError, PowerPlanApplyErrorReason, LIIIRAA_BALANCED_SCHEME_GUID,
    LIIIRAA_COMPETITIVE_SCHEME_GUID, LIIIRAA_PERFORMANCE_SCHEME_GUID,
    WINDOWS_BALANCED_SCHEME_GUID, WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID,
};
pub use scan::{
    parse_system_scan_report, scan_system, BackgroundAppScanItem, CpuScanItem, DefenderScan,
    DirectStorageScan, GpuScanItem, GraphicsAppPreferenceScanItem, GraphicsDwordSettingScan,
    GraphicsSettingsScan, MemoryModuleScanItem, MemoryScan,
    NetworkAdapterAdvancedPropertyScanItem, NetworkAdapterPowerManagementScan,
    NetworkAdapterScanItem, NtfsMetadataScan, OsScan, PhysicalDiskScanItem, PowerPlanScan,
    RebootRequiredScan, ScheduledTaskScanItem, SchedulerRegistryScan, SecurityScan,
    ServiceScanItem, StartupAppScanItem, StorageCleanupCandidateScanItem, StorageCleanupScan,
    StorageScan, StorageSenseScan, StorageTrimScan, StorageVolumeScanItem, SystemScanError,
    SystemScanErrorReason, SystemScanMode, SystemScanReport, WindowsOptionalFeatureScanItem,
    WindowsSystemScanner,
};
pub use scheduler::{
    apply_scheduler_competitive_plan_to_fixture,
    build_consented_scheduler_competitive_plan_from_scan,
    build_scheduler_competitive_plan_from_scan, verify_scheduler_competitive_plan_fixture,
    SchedulerSettingsError, SchedulerSettingsErrorReason, SchedulerSettingsSummary,
};
pub use security_tradeoff::{
    apply_security_tradeoff_plan_to_fixture, build_consented_security_tradeoff_plan_from_scan,
    build_security_tradeoff_plan_from_scan, security_tradeoff_plan_is_conservative,
    verify_security_tradeoff_plan_fixture, SecurityTradeoffSettingsError,
    SecurityTradeoffSettingsErrorReason, SecurityTradeoffSettingsSummary,
};
pub use storage::{
    apply_ntfs_metadata_plan_to_fixture,
    apply_storage_sense_plan_to_fixture, build_consented_storage_sense_plan_from_scan,
    build_consented_ntfs_metadata_plan_from_scan, build_ntfs_metadata_plan_from_scan,
    build_storage_readiness_plan_from_scan, parse_ntfs_8dot3_fsutil_value,
    parse_ntfs_last_access_fsutil_value, verify_ntfs_metadata_plan_fixture,
    verify_storage_sense_plan_fixture, NtfsMetadataSettingsError,
    NtfsMetadataSettingsErrorReason, NtfsMetadataSettingsSummary, StorageSenseRegistryError,
    StorageSenseRegistryErrorReason, StorageSenseRegistrySummary,
};
pub use windows_lab::{
    apply_windows_lab_experiment_plan_to_fixture,
    build_consented_windows_lab_experiment_plan_from_state,
    build_windows_lab_experiment_plan_from_scan,
    verify_windows_lab_experiment_plan_fixture, WindowsLabSettingsError,
    WindowsLabSettingsErrorReason, WindowsLabSettingsSummary,
};
pub use windows_update::{
    apply_windows_update_plan_to_fixture, build_windows_update_plan_from_scan,
    verify_windows_update_plan_fixture, windows_update_plan_blocks_global_disable,
    WindowsUpdateSettingsError, WindowsUpdateSettingsErrorReason, WindowsUpdateSettingsSummary,
};

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

/// Windows API crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "windows-api",
    responsibility: "adapt registry, powercfg, services, scheduled tasks, and adapter access",
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

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "windows-api");
        assert!(info.responsibility.contains("registry"));
        assert!(info.requires_live_windows);
    }
}
