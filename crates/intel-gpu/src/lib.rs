//! Intel graphics detection and recommendation planning.

use gpu::{GpuCapabilityState, GpuInventory, GpuVendor, GpuVendorDetection};

/// Read-only Intel graphics and driver detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelGraphicsDetection {
    /// Intel adapters and driver versions detected from inventory.
    pub vendor: GpuVendorDetection,
    /// PresentMon GPU Busy metric readiness for Intel-friendly benchmarking.
    pub presentmon_gpu_busy_state: GpuCapabilityState,
}

impl IntelGraphicsDetection {
    /// Builds a conservative Intel graphics detection summary from GPU inventory.
    #[must_use]
    pub fn from_inventory(inventory: &GpuInventory) -> Self {
        let vendor = inventory.vendor_detection(GpuVendor::Intel);
        let presentmon_gpu_busy_state = if vendor.is_available() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::NotApplicable
        };

        Self {
            vendor,
            presentmon_gpu_busy_state,
        }
    }

    /// Returns true when an Intel graphics adapter was detected.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.vendor.is_available()
    }
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

/// Intel GPU crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "intel-gpu",
    responsibility: "detect Intel graphics capabilities and produce PresentMon-friendly guidance",
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

        assert_eq!(info.name, "intel-gpu");
        assert!(info.responsibility.contains("Intel"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn detects_intel_adapters_and_driver_versions() {
        let inventory = GpuInventory::new(vec![
            GpuAdapter::from_scan(
                "Intel(R) Arc(TM) A770 Graphics",
                Some("31.0.101.5590"),
                None,
                None,
                Some("PCI\\VEN_8086&DEV_56A0"),
            ),
            GpuAdapter::from_scan(
                "AMD Radeon RX 7800 XT",
                Some("31.0.24002.92"),
                None,
                None,
                Some("PCI\\VEN_1002&DEV_747E"),
            ),
        ]);

        let detection = IntelGraphicsDetection::from_inventory(&inventory);

        assert!(detection.is_available());
        assert_eq!(detection.vendor.adapters.len(), 1);
        assert_eq!(detection.vendor.driver_versions, vec!["31.0.101.5590"]);
        assert_eq!(
            detection.presentmon_gpu_busy_state,
            GpuCapabilityState::Unknown
        );
    }
}
