//! CPU platform detection and read-only Intel/AMD planning.

pub use cpu::{
    AmdX3dTopology, CpuCapabilityState, CpuPlatformInspection, CpuThrottleState, CpuTopology,
    CpuVendor, HybridCoreStatus, SmtStatus,
};

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan, TweakPlanItem,
        TweakRisk,
    },
};

/// Tweak ID for vendor, generation, topology, SMT, and cache detection.
pub const CPU_VENDOR_TOPOLOGY_TWEAK_ID: &str = "cpu.detect.vendor-topology";
/// Tweak ID for thermal and power throttling detection.
pub const CPU_THROTTLE_DETECT_TWEAK_ID: &str = "cpu.detect.throttle";
/// Tweak ID for generic CPU chipset driver readiness.
pub const CPU_CHIPSET_DRIVER_DETECT_TWEAK_ID: &str = "cpu.detect.chipset-driver";
/// Tweak ID for Windows Processor Power Management audit.
pub const CPU_PPM_AUDIT_TWEAK_ID: &str = "cpu.power.ppm-audit";
/// Tweak ID for Intel Thread Director readiness.
pub const CPU_INTEL_THREAD_DIRECTOR_DETECT_TWEAK_ID: &str =
    "cpu.intel.thread-director.detect";
/// Tweak ID for Intel APO and DTT readiness.
pub const CPU_INTEL_APO_DETECT_TWEAK_ID: &str = "cpu.intel.apo.detect";
/// Tweak ID for AMD chipset driver readiness.
pub const CPU_AMD_CHIPSET_DRIVER_DETECT_TWEAK_ID: &str =
    "cpu.amd.chipset-driver.detect";
/// Tweak ID for AMD CPPC/preferred cores readiness.
pub const CPU_AMD_CPPC_PREFERRED_CORES_TWEAK_ID: &str =
    "cpu.amd.cppc-preferred-cores";
/// Tweak ID for AMD X3D scheduler readiness.
pub const CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID: &str =
    "cpu.amd.x3d-scheduler.detect";

const INTEL_OFFICIAL_PATH_WARNING: &str = concat!(
    "Use the official Intel APO, DTT, BIOS, and platform package path; do not force ",
    "affinity or disable E-cores."
);
const AMD_OFFICIAL_PATH_WARNING: &str = concat!(
    "Use the official AMD chipset, CPPC, Game Mode, and 3D V-Cache scheduler path before ",
    "considering any workaround."
);
const NO_CPU_MUTATION_WARNING: &str =
    "CPU platform planning is read-only in this task and does not change BIOS, affinity, cores, or security mitigations.";

/// Request used to build the T057 CPU platform plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuPlatformPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Read-only CPU and platform inspection.
    pub inspection: CpuPlatformInspection,
}

impl CpuPlatformPlanRequest {
    /// Creates a conservative CPU platform plan request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>, inspection: CpuPlatformInspection) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            inspection,
        }
    }
}

/// Builds a read-only CPU platform plan for T057.
#[must_use]
pub fn build_cpu_platform_plan(request: &CpuPlatformPlanRequest) -> TweakPlan {
    let mut items = vec![
        vendor_topology_item(&request.inspection),
        throttle_item(&request.inspection),
        chipset_driver_item(&request.inspection),
        ppm_audit_item(&request.inspection),
    ];

    if request.inspection.has_vendor(CpuVendor::Intel) {
        items.push(intel_thread_director_item(&request.inspection));
        items.push(intel_apo_dtt_item(&request.inspection));
    }

    if request.inspection.has_vendor(CpuVendor::Amd) {
        items.push(amd_chipset_driver_item(&request.inspection));
        items.push(amd_cppc_item(&request.inspection));
        items.push(amd_x3d_scheduler_item(&request.inspection));
    }

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

/// Returns true when the ID belongs to the T057 CPU platform scope.
#[must_use]
pub fn is_cpu_platform_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        CPU_VENDOR_TOPOLOGY_TWEAK_ID
            | CPU_THROTTLE_DETECT_TWEAK_ID
            | CPU_CHIPSET_DRIVER_DETECT_TWEAK_ID
            | CPU_PPM_AUDIT_TWEAK_ID
            | CPU_INTEL_THREAD_DIRECTOR_DETECT_TWEAK_ID
            | CPU_INTEL_APO_DETECT_TWEAK_ID
            | CPU_AMD_CHIPSET_DRIVER_DETECT_TWEAK_ID
            | CPU_AMD_CPPC_PREFERRED_CORES_TWEAK_ID
            | CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID
    )
}

/// Returns true when T057 has no default CPU mutation or unsafe operation.
#[must_use]
pub fn cpu_platform_plan_has_no_unsafe_default(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        is_cpu_platform_tweak_id(&item.tweak_id)
            && item.action != PlanAction::Apply
            && item.backup == BackupRequirement::NotRequired
            && item.rollback.kind == RollbackKind::NotNeededReadonly
            && item.changes.iter().all(read_only_or_denial_change)
    })
}

fn read_only_or_denial_change(change: &PlannedChange) -> bool {
    !matches!(
        change.operation,
        TweakOperationKind::Write | TweakOperationKind::Delete
    ) && matches!(
        change.scope,
        SessionScope::RecommendationOnly | SessionScope::Blocked
    )
}

fn vendor_topology_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![NO_CPU_MUTATION_WARNING.to_owned()];
    let action = if inspection.processors.is_empty() {
        warnings.push("No CPU package was present in the scan data.".to_owned());
        PlanAction::Recommend
    } else {
        warnings.extend(inspection.processors.iter().map(processor_summary));
        PlanAction::DetectOnly
    };

    read_only_item(CPU_VENDOR_TOPOLOGY_TWEAK_ID, action, TweakRisk::Low, warnings)
}

fn throttle_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![NO_CPU_MUTATION_WARNING.to_owned()];
    let action = match (
        inspection.thermal_throttling,
        inspection.power_limit_throttling,
    ) {
        (CpuThrottleState::Detected, _) | (_, CpuThrottleState::Detected) => {
            warnings.push(
                "CPU throttling was detected; recommend cooling, power, and benchmark validation before tuning."
                    .to_owned(),
            );
            PlanAction::Recommend
        }
        (CpuThrottleState::Unknown, _) | (_, CpuThrottleState::Unknown) => {
            warnings.push(
                "Thermal or power-limit throttling sensors were unavailable in this scan."
                    .to_owned(),
            );
            PlanAction::DetectOnly
        }
        _ => PlanAction::DetectOnly,
    };

    read_only_item(CPU_THROTTLE_DETECT_TWEAK_ID, action, TweakRisk::Low, warnings)
}

fn chipset_driver_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![NO_CPU_MUTATION_WARNING.to_owned()];
    let state = match inspection.primary_vendor() {
        CpuVendor::Intel => inspection.intel_chipset_driver_state,
        CpuVendor::Amd => inspection.amd_chipset_driver_state,
        CpuVendor::Unknown => CpuCapabilityState::Unknown,
    };
    let action = if state.needs_attention() {
        warnings.push(
            "CPU chipset driver readiness is missing or unknown; recommend the vendor support path."
                .to_owned(),
        );
        PlanAction::Recommend
    } else {
        PlanAction::DetectOnly
    };

    read_only_item(
        CPU_CHIPSET_DRIVER_DETECT_TWEAK_ID,
        action,
        TweakRisk::Low,
        warnings,
    )
}

fn ppm_audit_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![NO_CPU_MUTATION_WARNING.to_owned()];

    if let Some(plan_name) = &inspection.active_power_plan_name {
        warnings.push(format!("Active power plan detected for PPM audit: {plan_name}."));
    } else {
        warnings.push("Active power plan was not available for PPM audit.".to_owned());
    }

    let action = if inspection.ppm_settings_state.is_ready() {
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "Detailed Processor Power Management values are unavailable; keep this audit read-only until power-plan tasks own scoped writes."
                .to_owned(),
        );
        PlanAction::Recommend
    };

    read_only_item(CPU_PPM_AUDIT_TWEAK_ID, action, TweakRisk::Low, warnings)
}

fn intel_thread_director_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![NO_CPU_MUTATION_WARNING.to_owned()];
    let action = if !inspection.has_intel_hybrid_cpu() {
        warnings.push("No Intel hybrid CPU topology was detected.".to_owned());
        PlanAction::DetectOnly
    } else if inspection.windows_11_or_newer() == Some(true) {
        warnings.push(
            "Intel hybrid CPU detected with Windows 11 scheduler readiness.".to_owned(),
        );
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "Intel hybrid CPU detected without confirmed Windows 11 scheduler readiness."
                .to_owned(),
        );
        warnings.push(INTEL_OFFICIAL_PATH_WARNING.to_owned());
        PlanAction::Recommend
    };

    read_only_item(
        CPU_INTEL_THREAD_DIRECTOR_DETECT_TWEAK_ID,
        action,
        TweakRisk::Low,
        warnings,
    )
}

fn intel_apo_dtt_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![
        NO_CPU_MUTATION_WARNING.to_owned(),
        INTEL_OFFICIAL_PATH_WARNING.to_owned(),
    ];
    let action = if inspection.intel_dtt_state.is_ready() && inspection.intel_apo_state.is_ready() {
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "Intel APO or DTT readiness is missing or unknown; recommend BIOS, driver, Windows 11, and supported-game checks."
                .to_owned(),
        );
        PlanAction::Recommend
    };

    read_only_item(CPU_INTEL_APO_DETECT_TWEAK_ID, action, TweakRisk::Low, warnings)
}

fn amd_chipset_driver_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![
        NO_CPU_MUTATION_WARNING.to_owned(),
        AMD_OFFICIAL_PATH_WARNING.to_owned(),
    ];
    let action = if inspection.amd_chipset_driver_state.is_ready() {
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "AMD chipset driver readiness is missing or unknown; recommend official AMD chipset driver repair or update."
                .to_owned(),
        );
        PlanAction::Recommend
    };

    read_only_item(
        CPU_AMD_CHIPSET_DRIVER_DETECT_TWEAK_ID,
        action,
        TweakRisk::Low,
        warnings,
    )
}

fn amd_cppc_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![
        NO_CPU_MUTATION_WARNING.to_owned(),
        AMD_OFFICIAL_PATH_WARNING.to_owned(),
    ];
    let action = if inspection.amd_cppc_state.is_ready() {
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "AMD CPPC/preferred cores readiness is missing or unknown; recommend BIOS and chipset checks."
                .to_owned(),
        );
        PlanAction::Recommend
    };

    read_only_item(
        CPU_AMD_CPPC_PREFERRED_CORES_TWEAK_ID,
        action,
        TweakRisk::Low,
        warnings,
    )
}

fn amd_x3d_scheduler_item(inspection: &CpuPlatformInspection) -> TweakPlanItem {
    let mut warnings = vec![
        NO_CPU_MUTATION_WARNING.to_owned(),
        AMD_OFFICIAL_PATH_WARNING.to_owned(),
    ];
    let action = if !inspection.has_amd_x3d_cpu() {
        warnings.push("No AMD X3D CPU was detected.".to_owned());
        PlanAction::DetectOnly
    } else if !inspection.has_amd_multi_ccd_x3d_cpu() {
        warnings.push(
            "AMD X3D CPU appears single CCD; multi-CCD X3D scheduler checks are informational."
                .to_owned(),
        );
        PlanAction::DetectOnly
    } else if inspection.amd_x3d_scheduler_state.is_ready()
        && inspection.game_mode_state.is_ready()
    {
        PlanAction::DetectOnly
    } else {
        warnings.push(
            "Multi-CCD AMD X3D CPU detected without confirmed AMD PPM/3D V-Cache scheduler and Game Mode readiness."
                .to_owned(),
        );
        PlanAction::Recommend
    };

    read_only_item(
        CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID,
        action,
        TweakRisk::Low,
        warnings,
    )
}

fn read_only_item(
    tweak_id: &str,
    action: PlanAction,
    risk: TweakRisk,
    warnings: Vec<String>,
) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::CpuPlatform,
        action,
        mode: TweakMode::Safe,
        risk,
        changes: Vec::new(),
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

fn processor_summary(processor: &CpuTopology) -> String {
    let physical = processor
        .physical_cores
        .map_or_else(|| "unknown".to_owned(), |count| count.to_string());
    let logical = processor
        .logical_processors
        .map_or_else(|| "unknown".to_owned(), |count| count.to_string());
    let generation = processor
        .generation_hint
        .as_deref()
        .unwrap_or("unknown generation");

    format!(
        "{} CPU detected: {}; {physical} physical cores, {logical} logical processors, {generation}.",
        processor.vendor.as_str(),
        processor.name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    fn amd_x3d_inspection() -> CpuPlatformInspection {
        let processor = CpuTopology::from_scan(
            "AMD Ryzen 9 7950X3D 16-Core Processor",
            Some("AuthenticAMD"),
            Some(16),
            Some(32),
            Some(4_200),
        );
        let mut inspection = CpuPlatformInspection::new(vec![processor]);
        inspection.windows_build_number = Some("26100".to_owned());
        inspection.active_power_plan_name = Some("Balanced".to_owned());
        inspection.amd_chipset_driver_state = CpuCapabilityState::Missing;
        inspection.amd_cppc_state = CpuCapabilityState::Unknown;
        inspection.amd_x3d_scheduler_state = CpuCapabilityState::Missing;
        inspection.game_mode_state = CpuCapabilityState::Unknown;
        inspection.ppm_settings_state = CpuCapabilityState::Unknown;
        inspection
    }

    fn intel_hybrid_inspection() -> CpuPlatformInspection {
        let processor = CpuTopology::from_scan(
            "13th Gen Intel(R) Core(TM) i9-13900K",
            Some("GenuineIntel"),
            Some(24),
            Some(32),
            Some(5_800),
        );
        let mut inspection = CpuPlatformInspection::new(vec![processor]);
        inspection.windows_build_number = Some("26100".to_owned());
        inspection.active_power_plan_name = Some("Balanced".to_owned());
        inspection.intel_chipset_driver_state = CpuCapabilityState::Ready;
        inspection.intel_dtt_state = CpuCapabilityState::Missing;
        inspection.intel_apo_state = CpuCapabilityState::Missing;
        inspection.ppm_settings_state = CpuCapabilityState::Unknown;
        inspection
    }

    #[test]
    fn amd_platform_plan_recommends_official_x3d_path_without_apply() {
        let request = CpuPlatformPlanRequest::new("plan-amd-x3d", amd_x3d_inspection());
        let plan = build_cpu_platform_plan(&request);
        let x3d = item(&plan, CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID);

        assert_eq!(x3d.action, PlanAction::Recommend);
        assert!(x3d
            .warnings
            .iter()
            .any(|warning| warning.contains("official AMD chipset")));
        assert!(!plan.has_apply_items());
        assert!(cpu_platform_plan_has_no_unsafe_default(&plan));
    }

    #[test]
    fn intel_apo_plan_recommends_official_path_without_affinity() {
        let request = CpuPlatformPlanRequest::new(
            "plan-intel-hybrid",
            intel_hybrid_inspection(),
        );
        let plan = build_cpu_platform_plan(&request);
        let apo = item(&plan, CPU_INTEL_APO_DETECT_TWEAK_ID);

        assert_eq!(apo.action, PlanAction::Recommend);
        assert!(apo
            .warnings
            .iter()
            .any(|warning| warning.contains("official Intel APO")));
        assert!(apo.changes.is_empty());
        assert!(cpu_platform_plan_has_no_unsafe_default(&plan));
    }

    #[test]
    fn cpu_platform_scope_rejects_apply_items_as_unsafe_default() {
        let request = CpuPlatformPlanRequest::new("plan-amd-x3d", amd_x3d_inspection());
        let mut plan = build_cpu_platform_plan(&request);
        let topology = plan
            .items
            .iter_mut()
            .find(|item| item.tweak_id == CPU_VENDOR_TOPOLOGY_TWEAK_ID)
            .expect("topology item should exist");
        topology.action = PlanAction::Apply;

        assert!(!cpu_platform_plan_has_no_unsafe_default(&plan));
    }
}
