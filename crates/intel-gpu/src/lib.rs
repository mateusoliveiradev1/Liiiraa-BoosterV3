//! Intel graphics detection and recommendation planning.

use gpu::{
    plan_gpu_platform_capabilities, GpuAdapter, GpuCapabilityState, GpuInventory,
    GpuPlatformCapabilityPlan, GpuPlatformCheckRequest, GpuVendor,
    GpuVendorDetection,
};

/// Tweak ID for Intel graphics and driver detection.
pub const INTEL_DETECT_TWEAK_ID: &str = "intel.detect";
/// Tweak ID for PresentMon GPU Busy metric planning.
pub const INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID: &str = "intel.presentmon.gpubusy";

const NO_INTEL_GPU_MUTATION_WARNING: &str = concat!(
    "Intel graphics planning is read-only and does not change driver profiles, ",
    "firmware, voltage, game files, memory, or anti-cheat services."
);
const INTEL_OFFICIAL_DRIVER_GUIDANCE: &str = concat!(
    "Recommend the official Intel graphics driver or Intel Driver & Support Assistant ",
    "path when driver data is missing, stale, or tied to a known game issue."
);
const PRESENTMON_GPU_BUSY_GUIDANCE: &str = concat!(
    "Use PresentMon GPU Busy metrics when capture support is available, and label the ",
    "metric unavailable when the capture cannot expose it."
);

/// Intel graphics adapter family inferred from read-only inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntelGraphicsFamily {
    /// Intel Arc discrete or integrated Arc graphics.
    Arc,
    /// Intel Iris Xe graphics.
    IrisXe,
    /// Intel UHD or older HD integrated graphics.
    Uhd,
    /// Generic Intel integrated graphics where the exact family is unclear.
    Integrated,
    /// Intel family could not be inferred from scan text.
    Unknown,
}

impl IntelGraphicsFamily {
    /// Returns a user-facing family label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Arc => "Intel Arc",
            Self::IrisXe => "Intel Iris Xe",
            Self::Uhd => "Intel UHD/HD Graphics",
            Self::Integrated => "Intel integrated graphics",
            Self::Unknown => "Unknown Intel graphics",
        }
    }

    /// Returns true when the family is Arc, where driver freshness matters most.
    #[must_use]
    pub const fn is_arc(self) -> bool {
        matches!(self, Self::Arc)
    }
}

/// Intel adapter family summary used by recommendation planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelAdapterFamilyDetection {
    /// Adapter display name.
    pub name: String,
    /// Inferred Intel graphics family.
    pub family: IntelGraphicsFamily,
    /// Driver version reported by the OS or vendor scan.
    pub driver_version: Option<String>,
}

impl IntelAdapterFamilyDetection {
    fn from_adapter(adapter: &GpuAdapter) -> Self {
        Self {
            name: adapter.name.clone(),
            family: classify_intel_graphics_family(
                &adapter.name,
                adapter.video_processor.as_deref(),
            ),
            driver_version: adapter.driver_version.clone(),
        }
    }
}

/// Action selected for one Intel graphics recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelGraphicsRecommendationAction {
    /// The detector only needs to report facts.
    DetectOnly,
    /// The UI should show a safe recommendation, with no automatic mutation.
    Recommend,
    /// The recommendation does not apply on this machine.
    Unavailable,
}

/// Recommendation emitted by the Intel graphics planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelGraphicsRecommendation {
    /// Stable tweak ID associated with the recommendation.
    pub tweak_id: &'static str,
    /// User-facing title for the recommendation.
    pub title: &'static str,
    /// Capability state that informed the action.
    pub capability: GpuCapabilityState,
    /// Recommendation action.
    pub action: IntelGraphicsRecommendationAction,
    /// User-visible notes explaining safety and next steps.
    pub notes: Vec<String>,
}

impl IntelGraphicsRecommendation {
    fn new(
        tweak_id: &'static str,
        title: &'static str,
        capability: GpuCapabilityState,
        action: IntelGraphicsRecommendationAction,
        notes: Vec<String>,
    ) -> Self {
        Self {
            tweak_id,
            title,
            capability,
            action,
            notes,
        }
    }
}

/// Read-only Intel graphics and driver detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelGraphicsDetection {
    /// Intel adapters and driver versions detected from inventory.
    pub vendor: GpuVendorDetection,
    /// Family summary for each detected Intel adapter.
    pub adapters: Vec<IntelAdapterFamilyDetection>,
    /// Driver-version readiness for detected Intel adapters.
    pub driver_state: GpuCapabilityState,
    /// PresentMon GPU Busy metric readiness for Intel-friendly benchmarking.
    pub presentmon_gpu_busy_state: GpuCapabilityState,
    /// Whether PUBG-specific Intel graphics issue review is resolved.
    pub pubg_known_issue_state: GpuCapabilityState,
}

impl IntelGraphicsDetection {
    /// Builds a conservative Intel graphics detection summary from GPU inventory.
    #[must_use]
    pub fn from_inventory(inventory: &GpuInventory) -> Self {
        let vendor = inventory.vendor_detection(GpuVendor::Intel);
        let adapters = vendor
            .adapters
            .iter()
            .map(IntelAdapterFamilyDetection::from_adapter)
            .collect::<Vec<_>>();
        let presentmon_gpu_busy_state = if vendor.is_available() {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::NotApplicable
        };
        let pubg_known_issue_state = if !vendor.is_available() {
            GpuCapabilityState::NotApplicable
        } else if adapters.iter().any(|adapter| adapter.family.is_arc()) {
            GpuCapabilityState::Unknown
        } else {
            GpuCapabilityState::Ready
        };
        let driver_state = vendor.driver_state();

        Self {
            vendor,
            adapters,
            driver_state,
            presentmon_gpu_busy_state,
            pubg_known_issue_state,
        }
    }

    /// Overrides PresentMon GPU Busy readiness with capture-probe evidence.
    #[must_use]
    pub const fn with_presentmon_gpu_busy_state(mut self, state: GpuCapabilityState) -> Self {
        self.presentmon_gpu_busy_state = state;
        self
    }

    /// Overrides PUBG known-issue review state with vendor/game evidence.
    #[must_use]
    pub const fn with_pubg_known_issue_state(mut self, state: GpuCapabilityState) -> Self {
        self.pubg_known_issue_state = state;
        self
    }

    /// Returns true when an Intel graphics adapter was detected.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.vendor.is_available()
    }

    /// Returns true when an Intel Arc adapter was detected.
    #[must_use]
    pub fn has_arc_adapter(&self) -> bool {
        self.adapters
            .iter()
            .any(|adapter| adapter.family.is_arc())
    }

    /// Returns true when Intel driver versions are ready for recommendation planning.
    #[must_use]
    pub const fn has_driver_version(&self) -> bool {
        matches!(self.driver_state, GpuCapabilityState::Ready)
    }

    /// Builds safe Intel graphics recommendations from read-only state.
    #[must_use]
    pub fn safe_recommendations(&self) -> Vec<IntelGraphicsRecommendation> {
        vec![
            self.driver_and_pubg_recommendation(),
            self.presentmon_gpu_busy_recommendation(),
        ]
    }

    fn driver_and_pubg_recommendation(&self) -> IntelGraphicsRecommendation {
        let mut notes = vec![NO_INTEL_GPU_MUTATION_WARNING.to_owned()];

        if !self.is_available() {
            notes.push("No Intel graphics adapter was detected.".to_owned());
            return IntelGraphicsRecommendation::new(
                INTEL_DETECT_TWEAK_ID,
                "Intel graphics detection",
                GpuCapabilityState::NotApplicable,
                IntelGraphicsRecommendationAction::Unavailable,
                notes,
            );
        }

        notes.extend(self.adapters.iter().map(adapter_summary));

        if self.driver_state.needs_attention() {
            notes.push(INTEL_OFFICIAL_DRIVER_GUIDANCE.to_owned());
            return IntelGraphicsRecommendation::new(
                INTEL_DETECT_TWEAK_ID,
                "Intel graphics detection",
                self.driver_state,
                IntelGraphicsRecommendationAction::Recommend,
                notes,
            );
        }

        if self.pubg_known_issue_state.needs_attention() || self.has_arc_adapter() {
            notes.push(
                "Intel Arc was detected; review official Intel and PUBG driver guidance before claiming a PUBG-specific improvement."
                    .to_owned(),
            );
            notes.push(INTEL_OFFICIAL_DRIVER_GUIDANCE.to_owned());
            return IntelGraphicsRecommendation::new(
                INTEL_DETECT_TWEAK_ID,
                "Intel graphics detection",
                self.pubg_known_issue_state,
                IntelGraphicsRecommendationAction::Recommend,
                notes,
            );
        }

        IntelGraphicsRecommendation::new(
            INTEL_DETECT_TWEAK_ID,
            "Intel graphics detection",
            self.driver_state,
            IntelGraphicsRecommendationAction::DetectOnly,
            notes,
        )
    }

    fn presentmon_gpu_busy_recommendation(&self) -> IntelGraphicsRecommendation {
        let mut notes = vec![
            NO_INTEL_GPU_MUTATION_WARNING.to_owned(),
            PRESENTMON_GPU_BUSY_GUIDANCE.to_owned(),
        ];

        if !self.is_available() {
            notes.push("No Intel graphics adapter was detected for GPU Busy planning.".to_owned());
            return IntelGraphicsRecommendation::new(
                INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID,
                "PresentMon GPU Busy metrics",
                GpuCapabilityState::NotApplicable,
                IntelGraphicsRecommendationAction::Unavailable,
                notes,
            );
        }

        let action = match self.presentmon_gpu_busy_state {
            GpuCapabilityState::Ready => {
                notes.push(
                    "PresentMon GPU Busy is available; include gpu_busy metrics in benchmark summaries."
                        .to_owned(),
                );
                IntelGraphicsRecommendationAction::Recommend
            }
            GpuCapabilityState::Missing => {
                notes.push(
                    "PresentMon GPU Busy is missing; benchmark summaries must not infer GPU-bound status from this metric."
                        .to_owned(),
                );
                IntelGraphicsRecommendationAction::Recommend
            }
            GpuCapabilityState::Unknown => {
                notes.push(
                    "PresentMon GPU Busy support is unknown; mark the metric unavailable until capture confirms it."
                        .to_owned(),
                );
                IntelGraphicsRecommendationAction::Recommend
            }
            GpuCapabilityState::NotApplicable => IntelGraphicsRecommendationAction::Unavailable,
        };

        IntelGraphicsRecommendation::new(
            INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID,
            "PresentMon GPU Busy metrics",
            self.presentmon_gpu_busy_state,
            action,
            notes,
        )
    }
}

/// Builds the Intel graphics view of the shared GPU platform capability plan.
#[must_use]
pub fn plan_intel_platform_capabilities(
    detection: &IntelGraphicsDetection,
    request: &GpuPlatformCheckRequest,
) -> GpuPlatformCapabilityPlan {
    plan_gpu_platform_capabilities(&detection.vendor, request)
}

/// Classifies an Intel graphics family from adapter strings.
#[must_use]
pub fn classify_intel_graphics_family(
    name: &str,
    video_processor: Option<&str>,
) -> IntelGraphicsFamily {
    let text = format!("{} {}", name, video_processor.unwrap_or_default()).to_ascii_lowercase();

    if text.contains("arc") {
        IntelGraphicsFamily::Arc
    } else if text.contains("iris xe") || text.contains("iris(r) xe") {
        IntelGraphicsFamily::IrisXe
    } else if text.contains("uhd graphics") || text.contains("hd graphics") {
        IntelGraphicsFamily::Uhd
    } else if text.contains("intel") {
        IntelGraphicsFamily::Integrated
    } else {
        IntelGraphicsFamily::Unknown
    }
}

fn adapter_summary(adapter: &IntelAdapterFamilyDetection) -> String {
    let driver = adapter.driver_version.as_deref().unwrap_or("unknown driver");

    format!(
        "{} adapter detected: {}; driver {driver}.",
        adapter.family.display_name(),
        adapter.name
    )
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
        assert!(detection.has_arc_adapter());
        assert_eq!(detection.vendor.adapters.len(), 1);
        assert_eq!(detection.adapters[0].family, IntelGraphicsFamily::Arc);
        assert_eq!(detection.vendor.driver_versions, vec!["31.0.101.5590"]);
        assert_eq!(detection.driver_state, GpuCapabilityState::Ready);
        assert_eq!(
            detection.presentmon_gpu_busy_state,
            GpuCapabilityState::Unknown
        );
        assert_eq!(detection.pubg_known_issue_state, GpuCapabilityState::Unknown);
    }

    #[test]
    fn intel_platform_plan_uses_shared_read_only_policy() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_56A0"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory);
        let request = gpu::GpuPlatformCheckRequest::new(gpu::GpuPlatformIntent::Balanced)
            .with_driver_age_days(220)
            .with_display(gpu::GpuDisplayPipelineState::new(
                Some(144),
                Some(144),
                GpuCapabilityState::Unknown,
            ))
            .with_rebar_sam_state(GpuCapabilityState::Unknown)
            .with_frame_generation_state(GpuCapabilityState::NotApplicable)
            .with_shader_cache(gpu::GpuShaderCacheState::unknown());

        let plan = plan_intel_platform_capabilities(&detection, &request);

        assert_eq!(plan.vendor, GpuVendor::Intel);
        assert_eq!(plan.driver.action, gpu::GpuDriverMaintenanceAction::UpdateRecommended);
        assert_eq!(plan.rebar_sam.label, "Resizable BAR");
        assert_eq!(
            plan.frame_generation.policy,
            gpu::GpuFrameGenerationPolicy::Unsupported
        );
        assert!(plan.recommendations.iter().any(|recommendation| {
            recommendation.check == gpu::GpuPlatformCheck::DriverAge
        }));
    }

    #[test]
    fn arc_recommendations_stay_safe_and_presentmon_friendly() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_56A0"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory)
            .with_presentmon_gpu_busy_state(GpuCapabilityState::Ready);
        let recommendations = detection.safe_recommendations();
        let driver = recommendation(&recommendations, INTEL_DETECT_TWEAK_ID);
        let gpu_busy = recommendation(&recommendations, INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID);

        assert_eq!(
            driver.action,
            IntelGraphicsRecommendationAction::Recommend
        );
        assert!(driver
            .notes
            .iter()
            .any(|note| note.contains("official Intel")));
        assert_eq!(
            gpu_busy.action,
            IntelGraphicsRecommendationAction::Recommend
        );
        assert_eq!(gpu_busy.capability, GpuCapabilityState::Ready);
        assert!(gpu_busy
            .notes
            .iter()
            .any(|note| note.contains("gpu_busy metrics")));
    }

    #[test]
    fn non_intel_inventory_marks_recommendations_unavailable() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "NVIDIA GeForce RTX 4070",
            Some("32.0.15.6094"),
            None,
            None,
            Some("PCI\\VEN_10DE&DEV_2786"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory);
        let recommendations = detection.safe_recommendations();

        assert!(!detection.is_available());
        assert_eq!(
            detection.presentmon_gpu_busy_state,
            GpuCapabilityState::NotApplicable
        );
        assert!(recommendations.iter().all(|recommendation| {
            recommendation.action == IntelGraphicsRecommendationAction::Unavailable
        }));
    }

    #[test]
    fn classifies_common_intel_graphics_families() {
        assert_eq!(
            classify_intel_graphics_family("Intel(R) Arc(TM) A770 Graphics", None),
            IntelGraphicsFamily::Arc
        );
        assert_eq!(
            classify_intel_graphics_family("Intel(R) Iris(R) Xe Graphics", None),
            IntelGraphicsFamily::IrisXe
        );
        assert_eq!(
            classify_intel_graphics_family("Intel(R) UHD Graphics 770", None),
            IntelGraphicsFamily::Uhd
        );
    }

    fn recommendation<'a>(
        recommendations: &'a [IntelGraphicsRecommendation],
        tweak_id: &str,
    ) -> &'a IntelGraphicsRecommendation {
        recommendations
            .iter()
            .find(|recommendation| recommendation.tweak_id == tweak_id)
            .expect("recommendation should exist")
    }
}
