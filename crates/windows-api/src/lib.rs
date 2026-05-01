//! Windows implementation adapters for registry, power, services, and devices.

pub mod backup;
pub mod background_work;
pub mod command;
pub mod gaming;
pub mod power;
pub mod scan;

pub use backup::WindowsRollbackFixture;
pub use background_work::{
    background_work_plan_is_recommendation_only, build_background_work_plan_from_scan,
};
pub use command::{
    FixedWindowsExecutable, StructuredCommandPlan, WindowsArgument, WindowsCommandPlanError,
};
pub use gaming::{
    apply_gaming_capture_plan_to_fixture, verify_gaming_capture_plan_fixture,
    GamingCaptureRegistryError, GamingCaptureRegistryErrorReason,
    GamingCaptureRegistrySummary,
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
    GpuScanItem, MemoryModuleScanItem, MemoryScan, NetworkAdapterScanItem, OsScan,
    PhysicalDiskScanItem, PowerPlanScan, RebootRequiredScan, ScheduledTaskScanItem,
    SecurityScan, ServiceScanItem, StartupAppScanItem, StorageScan, StorageVolumeScanItem,
    SystemScanError, SystemScanErrorReason, SystemScanMode, SystemScanReport,
    WindowsSystemScanner,
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
