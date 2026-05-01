//! Intel graphics recommendation planning for read-only GPU and benchmark guidance.

pub use intel_gpu::{
    IntelAdapterFamilyDetection, IntelGraphicsDetection, IntelGraphicsFamily,
    IntelGraphicsRecommendation, IntelGraphicsRecommendationAction, INTEL_DETECT_TWEAK_ID,
    INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID,
};

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan, TweakPlanItem,
        TweakRisk,
    },
};

/// Logical target for Intel graphics driver review recommendations.
pub const TARGET_INTEL_DRIVER_REVIEW: &str = "intel:graphics/official-driver-review";
/// Logical target for PresentMon GPU Busy benchmark capture.
pub const TARGET_PRESENTMON_GPU_BUSY_METRIC: &str = "benchmark:presentmon/gpu_busy";

/// Request used to build the T065 Intel graphics recommendation plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntelGraphicsPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Read-only Intel graphics detection.
    pub detection: IntelGraphicsDetection,
}

impl IntelGraphicsPlanRequest {
    /// Creates a conservative Intel graphics plan request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>, detection: IntelGraphicsDetection) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            detection,
        }
    }
}

/// Builds a safe Intel graphics recommendation plan for T065.
#[must_use]
pub fn build_intel_graphics_recommendation_plan(
    request: &IntelGraphicsPlanRequest,
) -> TweakPlan {
    let items = request
        .detection
        .safe_recommendations()
        .iter()
        .map(recommendation_item)
        .collect::<Vec<_>>();
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

/// Returns true when the ID belongs to the T065 Intel graphics scope.
#[must_use]
pub fn is_intel_graphics_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        INTEL_DETECT_TWEAK_ID | INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID
    )
}

/// Returns true when T065 contains only read-only or manual recommendation work.
#[must_use]
pub fn intel_graphics_plan_is_recommendation_only(plan: &TweakPlan) -> bool {
    !plan.has_apply_items()
        && plan.items.iter().all(|item| {
            is_intel_graphics_tweak_id(&item.tweak_id)
                && item.category == TweakCategory::IntelGraphics
                && item.mode == TweakMode::Safe
                && item.risk == TweakRisk::Low
                && item.backup == BackupRequirement::NotRequired
                && item.rollback.kind == RollbackKind::NotNeededReadonly
                && item.reboot == RebootPolicy::None
                && !item.requires_admin
                && matches!(item.action, PlanAction::DetectOnly | PlanAction::Recommend)
                && item.changes.iter().all(recommendation_change_is_safe)
        })
}

fn recommendation_item(recommendation: &IntelGraphicsRecommendation) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: recommendation.tweak_id.to_owned(),
        category: TweakCategory::IntelGraphics,
        action: plan_action(recommendation.action),
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes: recommendation_changes(recommendation),
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings: recommendation.notes.clone(),
    }
}

const fn plan_action(action: IntelGraphicsRecommendationAction) -> PlanAction {
    match action {
        IntelGraphicsRecommendationAction::DetectOnly
        | IntelGraphicsRecommendationAction::Unavailable => PlanAction::DetectOnly,
        IntelGraphicsRecommendationAction::Recommend => PlanAction::Recommend,
    }
}

fn recommendation_changes(recommendation: &IntelGraphicsRecommendation) -> Vec<PlannedChange> {
    if recommendation.action != IntelGraphicsRecommendationAction::Recommend {
        return Vec::new();
    }

    let (target, operation, desired_value) = match recommendation.tweak_id {
        INTEL_DETECT_TWEAK_ID => (
            TARGET_INTEL_DRIVER_REVIEW,
            TweakOperationKind::Manual,
            "review_official_intel_driver_and_pubg_guidance",
        ),
        INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID => (
            TARGET_PRESENTMON_GPU_BUSY_METRIC,
            TweakOperationKind::Read,
            "capture_gpu_busy_when_available",
        ),
        _ => return Vec::new(),
    };

    vec![PlannedChange {
        target: target.to_owned(),
        operation,
        previous_value: None,
        desired_value: Some(desired_value.to_owned()),
        scope: SessionScope::RecommendationOnly,
    }]
}

fn recommendation_change_is_safe(change: &PlannedChange) -> bool {
    matches!(
        change.operation,
        TweakOperationKind::Read | TweakOperationKind::Manual
    ) && change.scope == SessionScope::RecommendationOnly
        && change.previous_value.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpu::{GpuAdapter, GpuCapabilityState, GpuInventory};

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn intel_graphics_plan_recommends_arc_driver_review_and_gpu_busy_metrics() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_56A0"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory)
            .with_presentmon_gpu_busy_state(GpuCapabilityState::Ready);
        let request = IntelGraphicsPlanRequest::new("plan-intel-graphics", detection);

        let plan = build_intel_graphics_recommendation_plan(&request);
        let detect = item(&plan, INTEL_DETECT_TWEAK_ID);
        let gpu_busy = item(&plan, INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID);

        assert_eq!(plan.items.len(), 2);
        assert_eq!(detect.action, PlanAction::Recommend);
        assert_eq!(detect.changes[0].target, TARGET_INTEL_DRIVER_REVIEW);
        assert_eq!(detect.changes[0].operation, TweakOperationKind::Manual);
        assert!(detect
            .warnings
            .iter()
            .any(|warning| warning.contains("official Intel")));
        assert_eq!(gpu_busy.action, PlanAction::Recommend);
        assert_eq!(gpu_busy.changes[0].target, TARGET_PRESENTMON_GPU_BUSY_METRIC);
        assert_eq!(gpu_busy.changes[0].operation, TweakOperationKind::Read);
        assert!(!plan.has_apply_items());
        assert!(intel_graphics_plan_is_recommendation_only(&plan));
    }

    #[test]
    fn missing_presentmon_support_is_recommended_but_never_applied() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Iris(R) Xe Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_9A49"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory)
            .with_presentmon_gpu_busy_state(GpuCapabilityState::Missing);
        let request = IntelGraphicsPlanRequest::new("plan-intel-presentmon", detection);

        let plan = build_intel_graphics_recommendation_plan(&request);
        let detect = item(&plan, INTEL_DETECT_TWEAK_ID);
        let gpu_busy = item(&plan, INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID);

        assert_eq!(detect.action, PlanAction::DetectOnly);
        assert!(detect.changes.is_empty());
        assert_eq!(gpu_busy.action, PlanAction::Recommend);
        assert_eq!(
            gpu_busy.changes[0].desired_value.as_deref(),
            Some("capture_gpu_busy_when_available")
        );
        assert!(intel_graphics_plan_is_recommendation_only(&plan));
    }

    #[test]
    fn non_intel_inventory_keeps_scope_detect_only() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "AMD Radeon RX 7800 XT",
            Some("31.0.24002.92"),
            None,
            None,
            Some("PCI\\VEN_1002&DEV_747E"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory);
        let request = IntelGraphicsPlanRequest::new("plan-no-intel", detection);

        let plan = build_intel_graphics_recommendation_plan(&request);

        assert!(plan
            .items
            .iter()
            .all(|item| item.action == PlanAction::DetectOnly && item.changes.is_empty()));
        assert!(intel_graphics_plan_is_recommendation_only(&plan));
    }

    #[test]
    fn recommendation_guard_rejects_accidental_apply() {
        let inventory = GpuInventory::new(vec![GpuAdapter::from_scan(
            "Intel(R) Arc(TM) A770 Graphics",
            Some("31.0.101.5590"),
            None,
            None,
            Some("PCI\\VEN_8086&DEV_56A0"),
        )]);
        let detection = IntelGraphicsDetection::from_inventory(&inventory);
        let request = IntelGraphicsPlanRequest::new("plan-intel-graphics", detection);
        let mut plan = build_intel_graphics_recommendation_plan(&request);
        item_mut(&mut plan, INTEL_PRESENTMON_GPU_BUSY_TWEAK_ID).action = PlanAction::Apply;

        assert!(!intel_graphics_plan_is_recommendation_only(&plan));
    }

    fn item_mut<'a>(plan: &'a mut TweakPlan, tweak_id: &str) -> &'a mut TweakPlanItem {
        plan.items
            .iter_mut()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }
}
