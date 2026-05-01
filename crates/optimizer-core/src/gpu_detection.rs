//! Read-only GPU vendor and driver detection planning.

pub use gpu::{GpuAdapter, GpuCapabilityState, GpuInventory, GpuVendor, GpuVendorDetection};

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, RebootPolicy, RollbackKind, RollbackPlan, TweakCategory,
        TweakMode, TweakPlan, TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for NVIDIA GPU and driver detection.
pub const NVIDIA_DETECT_TWEAK_ID: &str = "nvidia.detect";
/// Tweak ID for AMD Radeon GPU and driver detection.
pub const AMD_DETECT_TWEAK_ID: &str = "amd.detect";
/// Tweak ID for Intel graphics and driver detection.
pub const INTEL_DETECT_TWEAK_ID: &str = "intel.detect";

const NO_GPU_MUTATION_WARNING: &str = concat!(
    "GPU detection is read-only and does not change driver profiles, firmware, voltage, ",
    "game files, memory, or anti-cheat services."
);

/// Request used to build the T060 GPU detection plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDetectionPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Read-only GPU inventory.
    pub inventory: GpuInventory,
}

impl GpuDetectionPlanRequest {
    /// Creates a conservative GPU detection request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>, inventory: GpuInventory) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            inventory,
        }
    }
}

/// Builds a read-only GPU vendor and driver detection plan for T060.
#[must_use]
pub fn build_gpu_detection_plan(request: &GpuDetectionPlanRequest) -> TweakPlan {
    let items = vec![
        vendor_detection_item(
            NVIDIA_DETECT_TWEAK_ID,
            TweakCategory::Nvidia,
            request.inventory.vendor_detection(GpuVendor::Nvidia),
        ),
        vendor_detection_item(
            AMD_DETECT_TWEAK_ID,
            TweakCategory::Amd,
            request.inventory.vendor_detection(GpuVendor::Amd),
        ),
        vendor_detection_item(
            INTEL_DETECT_TWEAK_ID,
            TweakCategory::IntelGraphics,
            request.inventory.vendor_detection(GpuVendor::Intel),
        ),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
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

/// Returns true when the ID belongs to the T060 GPU detection scope.
#[must_use]
pub fn is_gpu_detection_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        NVIDIA_DETECT_TWEAK_ID | AMD_DETECT_TWEAK_ID | INTEL_DETECT_TWEAK_ID
    )
}

/// Returns true when T060 contains no profile, registry, firmware, or game mutation.
#[must_use]
pub fn gpu_detection_plan_is_read_only(plan: &TweakPlan) -> bool {
    !plan.has_apply_items()
        && plan.items.iter().all(|item| {
            is_gpu_detection_tweak_id(&item.tweak_id)
                && item.mode == TweakMode::Safe
                && item.risk == TweakRisk::Low
                && item.backup == BackupRequirement::NotRequired
                && item.rollback.kind == RollbackKind::NotNeededReadonly
                && item.reboot == RebootPolicy::None
                && !item.requires_admin
                && item.changes.is_empty()
        })
}

fn vendor_detection_item(
    tweak_id: &str,
    category: TweakCategory,
    detection: GpuVendorDetection,
) -> TweakPlanItem {
    let mut warnings = vec![NO_GPU_MUTATION_WARNING.to_owned()];
    let vendor_name = detection.vendor.display_name();
    let action = if !detection.is_available() {
        warnings.push(format!(
            "No {vendor_name} GPU was detected; mark {vendor_name} optimization modules unavailable."
        ));
        PlanAction::DetectOnly
    } else if detection.driver_state().needs_attention() {
        warnings.push(format!(
            "{vendor_name} GPU detected, but one or more driver versions were missing from scan data; recommend official vendor driver inspection."
        ));
        PlanAction::Recommend
    } else {
        warnings.push(format!(
            "{vendor_name} GPU detected with driver version(s): {}.",
            detection.driver_versions.join(", ")
        ));
        PlanAction::DetectOnly
    };

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category,
        action,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes: Vec::new(),
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mixed_inventory() -> GpuInventory {
        GpuInventory::new(vec![
            GpuAdapter::from_scan(
                "NVIDIA GeForce RTX 4070",
                Some("32.0.15.6094"),
                None,
                Some(12 * 1024 * 1024 * 1024),
                Some("PCI\\VEN_10DE&DEV_2786"),
            ),
            GpuAdapter::from_scan(
                "AMD Radeon RX 7800 XT",
                Some("31.0.24002.92"),
                None,
                Some(16 * 1024 * 1024 * 1024),
                Some("PCI\\VEN_1002&DEV_747E"),
            ),
            GpuAdapter::from_scan(
                "Intel(R) Arc(TM) A770 Graphics",
                Some("31.0.101.5590"),
                None,
                Some(16 * 1024 * 1024 * 1024),
                Some("PCI\\VEN_8086&DEV_56A0"),
            ),
        ])
    }

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn gpu_detection_plan_reports_all_vendors_without_apply() {
        let request = GpuDetectionPlanRequest::new("plan-gpu-detect", mixed_inventory());
        let plan = build_gpu_detection_plan(&request);

        assert_eq!(plan.items.len(), 3);
        assert_eq!(item(&plan, NVIDIA_DETECT_TWEAK_ID).action, PlanAction::DetectOnly);
        assert_eq!(item(&plan, AMD_DETECT_TWEAK_ID).action, PlanAction::DetectOnly);
        assert_eq!(item(&plan, INTEL_DETECT_TWEAK_ID).action, PlanAction::DetectOnly);
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("32.0.15.6094")));
        assert!(gpu_detection_plan_is_read_only(&plan));
    }

    #[test]
    fn missing_vendor_is_marked_unavailable() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "AMD Radeon RX 6800",
            Some("31.0.24002.92"),
            None,
            None,
            Some("PCI\\VEN_1002&DEV_73BF"),
        )]);
        let request = GpuDetectionPlanRequest::new("plan-no-nvidia", inventory);
        let plan = build_gpu_detection_plan(&request);
        let nvidia = item(&plan, NVIDIA_DETECT_TWEAK_ID);

        assert_eq!(nvidia.category, TweakCategory::Nvidia);
        assert_eq!(nvidia.action, PlanAction::DetectOnly);
        assert!(nvidia
            .warnings
            .iter()
            .any(|warning| warning.contains("No NVIDIA GPU")));
        assert!(gpu_detection_plan_is_read_only(&plan));
    }

    #[test]
    fn missing_driver_version_recommends_official_driver_inspection() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) UHD Graphics",
            None,
            None,
            None,
            Some("PCI\\VEN_8086&DEV_9A60"),
        )]);
        let request = GpuDetectionPlanRequest::new("plan-missing-driver", inventory);
        let plan = build_gpu_detection_plan(&request);
        let intel = item(&plan, INTEL_DETECT_TWEAK_ID);

        assert_eq!(intel.action, PlanAction::Recommend);
        assert!(intel
            .warnings
            .iter()
            .any(|warning| warning.contains("official vendor driver")));
        assert!(gpu_detection_plan_is_read_only(&plan));
    }

    #[test]
    fn read_only_guard_rejects_accidental_apply() {
        let request = GpuDetectionPlanRequest::new("plan-gpu-detect", mixed_inventory());
        let mut plan = build_gpu_detection_plan(&request);
        item_mut(&mut plan, NVIDIA_DETECT_TWEAK_ID).action = PlanAction::Apply;

        assert!(!gpu_detection_plan_is_read_only(&plan));
    }

    fn item_mut<'a>(plan: &'a mut TweakPlan, tweak_id: &str) -> &'a mut TweakPlanItem {
        plan.items
            .iter_mut()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }
}
