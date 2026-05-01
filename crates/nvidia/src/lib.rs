//! NVIDIA profile backup, planning, apply, and verification.

use gpu::{GpuCapabilityState, GpuInventory, GpuVendor, GpuVendorDetection};

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
}
