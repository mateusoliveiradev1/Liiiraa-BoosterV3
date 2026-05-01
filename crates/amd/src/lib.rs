//! AMD GPU profile planning and safe Adrenalin guidance.

use gpu::{GpuCapabilityState, GpuInventory, GpuVendor, GpuVendorDetection};

/// Read-only AMD Radeon GPU and driver detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdDriverDetection {
    /// AMD adapters and driver versions detected from inventory.
    pub vendor: GpuVendorDetection,
    /// AMD Software: Adrenalin profile-management readiness.
    pub adrenalin_state: GpuCapabilityState,
}

impl AmdDriverDetection {
    /// Builds a conservative AMD detection summary from GPU inventory.
    #[must_use]
    pub fn from_inventory(inventory: &GpuInventory) -> Self {
        let vendor = inventory.vendor_detection(GpuVendor::Amd);
        let adrenalin_state = if vendor.is_available() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::NotApplicable
        };

        Self {
            vendor,
            adrenalin_state,
        }
    }

    /// Returns true when an AMD adapter was detected.
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
    }
}
