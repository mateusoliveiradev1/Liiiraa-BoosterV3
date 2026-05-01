//! CPU topology and platform optimization planning.

/// CPU vendor family used by platform planning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    /// Intel CPU or platform.
    Intel,
    /// AMD CPU or platform.
    Amd,
    /// Vendor could not be classified from scan data.
    Unknown,
}

impl CpuVendor {
    /// Returns a stable lowercase label for logs and plan messages.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intel => "intel",
            Self::Amd => "amd",
            Self::Unknown => "unknown",
        }
    }
}

/// Whether a CPU appears to expose heterogeneous performance and efficiency cores.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HybridCoreStatus {
    /// Model naming suggests P-core/E-core scheduling may apply.
    HybridLikely,
    /// Model naming suggests a homogeneous core topology.
    HomogeneousLikely,
    /// The scan did not expose enough detail to classify topology.
    Unknown,
}

/// Simultaneous multithreading or Hyper-Threading status inferred from counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtStatus {
    /// Logical processor count exceeds physical core count.
    Enabled,
    /// Logical and physical counts match, or the CPU does not expose SMT.
    DisabledOrUnavailable,
    /// Counts were missing or inconsistent.
    Unknown,
}

/// AMD 3D V-Cache topology hint inferred from model naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmdX3dTopology {
    /// No X3D model hint was found.
    NotX3d,
    /// X3D was found and the known model is usually single CCD.
    SingleCcdLikely,
    /// X3D was found and the known model is usually multi CCD.
    MultiCcdLikely,
    /// X3D was found, but the CCD layout could not be inferred.
    UnknownX3d,
}

/// Readiness state for an optional CPU platform capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuCapabilityState {
    /// Capability or dependency appears present.
    Ready,
    /// Capability or dependency appears absent.
    Missing,
    /// Scan data does not expose this capability.
    Unknown,
    /// Capability does not apply to the detected platform.
    NotApplicable,
}

impl CpuCapabilityState {
    /// Returns true when the capability is present.
    #[must_use]
    pub const fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Returns true when the capability needs a user-visible recommendation.
    #[must_use]
    pub const fn needs_attention(self) -> bool {
        matches!(self, Self::Missing | Self::Unknown)
    }
}

/// Throttling signal exposed by scan data or sensors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuThrottleState {
    /// Thermal or power-limit throttling was detected.
    Detected,
    /// Scan data did not detect throttling.
    NotDetected,
    /// Sensors or counters were unavailable.
    Unknown,
}

/// Read-only topology facts for one CPU package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuTopology {
    /// Processor display name.
    pub name: String,
    /// Manufacturer string reported by Windows or firmware.
    pub manufacturer: Option<String>,
    /// Classified vendor.
    pub vendor: CpuVendor,
    /// Physical core count when exposed.
    pub physical_cores: Option<u32>,
    /// Logical processor count when exposed.
    pub logical_processors: Option<u32>,
    /// Maximum clock in MHz when exposed.
    pub max_clock_mhz: Option<u32>,
    /// Human-readable generation hint derived from model naming.
    pub generation_hint: Option<String>,
    /// Hybrid P-core/E-core topology hint.
    pub hybrid_status: HybridCoreStatus,
    /// SMT or Hyper-Threading status inferred from core counts.
    pub smt_status: SmtStatus,
    /// AMD X3D and CCD topology hint.
    pub amd_x3d_topology: AmdX3dTopology,
}

impl CpuTopology {
    /// Classifies one CPU package from read-only scan values.
    #[must_use]
    pub fn from_scan(
        name: impl Into<String>,
        manufacturer: Option<&str>,
        physical_cores: Option<u32>,
        logical_processors: Option<u32>,
        max_clock_mhz: Option<u32>,
    ) -> Self {
        let name = name.into();
        let manufacturer = manufacturer.map(str::to_owned);
        let vendor = classify_vendor(&name, manufacturer.as_deref());

        Self {
            generation_hint: generation_hint(vendor, &name),
            hybrid_status: hybrid_status(vendor, &name),
            smt_status: smt_status(physical_cores, logical_processors),
            amd_x3d_topology: amd_x3d_topology(vendor, &name),
            name,
            manufacturer,
            vendor,
            physical_cores,
            logical_processors,
            max_clock_mhz,
        }
    }

    /// Returns true when this CPU appears to be an Intel hybrid CPU.
    #[must_use]
    pub const fn is_intel_hybrid(&self) -> bool {
        matches!(self.vendor, CpuVendor::Intel)
            && matches!(self.hybrid_status, HybridCoreStatus::HybridLikely)
    }

    /// Returns true when this CPU appears to be a multi-CCD AMD X3D CPU.
    #[must_use]
    pub const fn is_amd_multi_ccd_x3d(&self) -> bool {
        matches!(self.vendor, CpuVendor::Amd)
            && matches!(self.amd_x3d_topology, AmdX3dTopology::MultiCcdLikely)
    }

    /// Returns true when this CPU carries any AMD X3D model hint.
    #[must_use]
    pub const fn is_amd_x3d(&self) -> bool {
        matches!(self.vendor, CpuVendor::Amd)
            && !matches!(self.amd_x3d_topology, AmdX3dTopology::NotX3d)
    }
}

/// Read-only platform state consumed by the CPU planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuPlatformInspection {
    /// CPU packages detected on the machine.
    pub processors: Vec<CpuTopology>,
    /// Windows build number used for Windows 11 scheduler readiness checks.
    pub windows_build_number: Option<String>,
    /// Active power plan name used for PPM audit context.
    pub active_power_plan_name: Option<String>,
    /// Intel chipset or platform driver readiness.
    pub intel_chipset_driver_state: CpuCapabilityState,
    /// Intel Dynamic Tuning Technology readiness.
    pub intel_dtt_state: CpuCapabilityState,
    /// Intel Application Optimization readiness.
    pub intel_apo_state: CpuCapabilityState,
    /// AMD chipset driver readiness.
    pub amd_chipset_driver_state: CpuCapabilityState,
    /// AMD CPPC/preferred cores readiness.
    pub amd_cppc_state: CpuCapabilityState,
    /// AMD X3D scheduler component readiness.
    pub amd_x3d_scheduler_state: CpuCapabilityState,
    /// Windows Game Mode readiness for X3D scheduling.
    pub game_mode_state: CpuCapabilityState,
    /// Thermal throttling signal.
    pub thermal_throttling: CpuThrottleState,
    /// Power-limit throttling signal.
    pub power_limit_throttling: CpuThrottleState,
    /// Whether detailed Windows processor power-management settings were available.
    pub ppm_settings_state: CpuCapabilityState,
}

impl CpuPlatformInspection {
    /// Creates a conservative CPU platform inspection with unknown optional signals.
    #[must_use]
    pub fn new(processors: Vec<CpuTopology>) -> Self {
        Self {
            processors,
            windows_build_number: None,
            active_power_plan_name: None,
            intel_chipset_driver_state: CpuCapabilityState::Unknown,
            intel_dtt_state: CpuCapabilityState::Unknown,
            intel_apo_state: CpuCapabilityState::Unknown,
            amd_chipset_driver_state: CpuCapabilityState::Unknown,
            amd_cppc_state: CpuCapabilityState::Unknown,
            amd_x3d_scheduler_state: CpuCapabilityState::Unknown,
            game_mode_state: CpuCapabilityState::Unknown,
            thermal_throttling: CpuThrottleState::Unknown,
            power_limit_throttling: CpuThrottleState::Unknown,
            ppm_settings_state: CpuCapabilityState::Unknown,
        }
    }

    /// Returns the first known vendor from the CPU package list.
    #[must_use]
    pub fn primary_vendor(&self) -> CpuVendor {
        self.processors
            .iter()
            .find_map(|processor| {
                if processor.vendor == CpuVendor::Unknown {
                    None
                } else {
                    Some(processor.vendor)
                }
            })
            .unwrap_or(CpuVendor::Unknown)
    }

    /// Returns true when any CPU package matches the vendor.
    #[must_use]
    pub fn has_vendor(&self, vendor: CpuVendor) -> bool {
        self.processors
            .iter()
            .any(|processor| processor.vendor == vendor)
    }

    /// Returns true when any CPU package appears to use Intel hybrid topology.
    #[must_use]
    pub fn has_intel_hybrid_cpu(&self) -> bool {
        self.processors.iter().any(CpuTopology::is_intel_hybrid)
    }

    /// Returns true when any CPU package appears to be an AMD X3D CPU.
    #[must_use]
    pub fn has_amd_x3d_cpu(&self) -> bool {
        self.processors.iter().any(CpuTopology::is_amd_x3d)
    }

    /// Returns true when any CPU package appears to be a multi-CCD AMD X3D CPU.
    #[must_use]
    pub fn has_amd_multi_ccd_x3d_cpu(&self) -> bool {
        self.processors.iter().any(CpuTopology::is_amd_multi_ccd_x3d)
    }

    /// Returns true when the OS build is new enough for Windows 11 scheduler features.
    #[must_use]
    pub fn windows_11_or_newer(&self) -> Option<bool> {
        self.windows_build_number
            .as_deref()
            .and_then(windows_build_is_windows_11_or_newer)
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

/// CPU crate metadata used by workspace smoke tests.
pub const CRATE_INFO: CrateInfo = CrateInfo {
    name: "cpu",
    responsibility: "detect CPU topology and plan Intel or AMD platform recommendations",
    requires_live_windows: true,
};

/// Returns this crate's scaffold metadata.
#[must_use]
pub const fn crate_info() -> CrateInfo {
    CRATE_INFO
}

/// Classifies a CPU vendor from processor name and manufacturer strings.
#[must_use]
pub fn classify_vendor(name: &str, manufacturer: Option<&str>) -> CpuVendor {
    let text = format!("{} {}", name, manufacturer.unwrap_or_default()).to_ascii_lowercase();

    if text.contains("genuineintel") || text.contains("intel") {
        CpuVendor::Intel
    } else if text.contains("authenticamd") || text.contains("amd") || text.contains("ryzen") {
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    }
}

/// Returns whether a Windows build number is Windows 11 or newer.
#[must_use]
pub fn windows_build_is_windows_11_or_newer(build_number: &str) -> Option<bool> {
    build_number
        .trim()
        .parse::<u32>()
        .ok()
        .map(|build| build >= 22_000)
}

fn generation_hint(vendor: CpuVendor, name: &str) -> Option<String> {
    match vendor {
        CpuVendor::Intel => intel_generation_hint(name),
        CpuVendor::Amd => amd_generation_hint(name),
        CpuVendor::Unknown => None,
    }
}

fn intel_generation_hint(name: &str) -> Option<String> {
    let normalized = name.to_ascii_lowercase();

    if normalized.contains("core ultra") {
        return Some("Intel Core Ultra".to_owned());
    }

    if let Some(generation) = explicit_intel_generation(&normalized) {
        return Some(format!("Intel Core {generation}th Gen"));
    }

    let digits = ["i3-", "i5-", "i7-", "i9-"]
        .iter()
        .find_map(|marker| digits_after_marker(&normalized, marker))?;

    intel_generation_from_model_digits(&digits)
        .map(|generation| format!("Intel Core {generation}th Gen"))
}

fn explicit_intel_generation(value: &str) -> Option<u8> {
    (8_u8..=14)
        .rev()
        .find(|generation| value.contains(&format!("{generation}th gen")))
}

fn intel_generation_from_model_digits(digits: &str) -> Option<u8> {
    if digits.len() >= 5 {
        return digits.get(..2)?.parse::<u8>().ok();
    }

    if digits.len() == 4 {
        if let Some(first_two) = digits.get(..2).and_then(|value| value.parse::<u8>().ok()) {
            if (10..=14).contains(&first_two) {
                return Some(first_two);
            }
        }

        return digits.get(..1)?.parse::<u8>().ok();
    }

    None
}

fn amd_generation_hint(name: &str) -> Option<String> {
    let digits = first_digit_run_at_least(name, 4)?;
    let generation = digits.chars().next()?;

    if generation.is_ascii_digit() {
        Some(format!("AMD Ryzen {generation}000 Series"))
    } else {
        None
    }
}

fn hybrid_status(vendor: CpuVendor, name: &str) -> HybridCoreStatus {
    if vendor != CpuVendor::Intel {
        return HybridCoreStatus::HomogeneousLikely;
    }

    let normalized = name.to_ascii_lowercase();
    if normalized.contains("core ultra") {
        return HybridCoreStatus::HybridLikely;
    }

    let Some(digits) = ["i3-", "i5-", "i7-", "i9-"]
        .iter()
        .find_map(|marker| digits_after_marker(&normalized, marker))
    else {
        return HybridCoreStatus::Unknown;
    };

    match intel_generation_from_model_digits(&digits) {
        Some(generation) if generation >= 12 => HybridCoreStatus::HybridLikely,
        Some(_) => HybridCoreStatus::HomogeneousLikely,
        None => HybridCoreStatus::Unknown,
    }
}

fn smt_status(physical_cores: Option<u32>, logical_processors: Option<u32>) -> SmtStatus {
    match (physical_cores, logical_processors) {
        (Some(physical), Some(logical)) if logical > physical => SmtStatus::Enabled,
        (Some(physical), Some(logical)) if logical == physical => SmtStatus::DisabledOrUnavailable,
        _ => SmtStatus::Unknown,
    }
}

fn amd_x3d_topology(vendor: CpuVendor, name: &str) -> AmdX3dTopology {
    if vendor != CpuVendor::Amd {
        return AmdX3dTopology::NotX3d;
    }

    let normalized = normalized_ascii(name);
    if !normalized.contains("x3d") {
        return AmdX3dTopology::NotX3d;
    }

    if ["7900x3d", "7950x3d", "9900x3d", "9950x3d"]
        .iter()
        .any(|model| normalized.contains(model))
    {
        AmdX3dTopology::MultiCcdLikely
    } else if ["5600x3d", "5700x3d", "5800x3d", "7800x3d", "9800x3d"]
        .iter()
        .any(|model| normalized.contains(model))
    {
        AmdX3dTopology::SingleCcdLikely
    } else {
        AmdX3dTopology::UnknownX3d
    }
}

fn digits_after_marker(value: &str, marker: &str) -> Option<String> {
    let start = value.find(marker)? + marker.len();
    let digits = value[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    (!digits.is_empty()).then_some(digits)
}

fn first_digit_run_at_least(value: &str, min_len: usize) -> Option<String> {
    let mut current = String::new();

    for character in value.chars() {
        if character.is_ascii_digit() {
            current.push(character);
        } else if current.len() >= min_len {
            return Some(current);
        } else {
            current.clear();
        }
    }

    (current.len() >= min_len).then_some(current)
}

fn normalized_ascii(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_identity() {
        let info = crate_info();

        assert_eq!(info.name, "cpu");
        assert!(info.responsibility.contains("topology"));
        assert!(info.requires_live_windows);
    }

    #[test]
    fn classifies_amd_x3d_topology_and_smt() {
        let topology = CpuTopology::from_scan(
            "AMD Ryzen 9 7950X3D 16-Core Processor",
            Some("AuthenticAMD"),
            Some(16),
            Some(32),
            Some(4_200),
        );

        assert_eq!(topology.vendor, CpuVendor::Amd);
        assert_eq!(
            topology.generation_hint.as_deref(),
            Some("AMD Ryzen 7000 Series")
        );
        assert_eq!(topology.smt_status, SmtStatus::Enabled);
        assert_eq!(topology.amd_x3d_topology, AmdX3dTopology::MultiCcdLikely);
        assert!(topology.is_amd_multi_ccd_x3d());
    }

    #[test]
    fn classifies_intel_hybrid_generation() {
        let topology = CpuTopology::from_scan(
            "13th Gen Intel(R) Core(TM) i7-13700K",
            Some("GenuineIntel"),
            Some(16),
            Some(24),
            Some(5_300),
        );

        assert_eq!(topology.vendor, CpuVendor::Intel);
        assert_eq!(topology.generation_hint.as_deref(), Some("Intel Core 13th Gen"));
        assert_eq!(topology.hybrid_status, HybridCoreStatus::HybridLikely);
        assert!(topology.is_intel_hybrid());
    }

    #[test]
    fn reports_windows_11_build_readiness() {
        assert_eq!(windows_build_is_windows_11_or_newer("26100"), Some(true));
        assert_eq!(windows_build_is_windows_11_or_newer("19045"), Some(false));
        assert_eq!(windows_build_is_windows_11_or_newer("unknown"), None);
    }
}
