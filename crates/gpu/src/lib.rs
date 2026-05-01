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
}
