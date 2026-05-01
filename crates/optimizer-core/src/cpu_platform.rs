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
/// Tweak ID for denying Intel E-core disable requests.
pub const CPU_INTEL_DISABLE_E_CORES_TWEAK_ID: &str = "cpu.intel.disable-e-cores";
/// Tweak ID for denying global SMT/Hyper-Threading disable requests.
pub const CPU_SMT_DISABLE_TWEAK_ID: &str = "cpu.smt-disable";
/// Tweak ID for denying CPU security mitigation disable requests.
pub const CPU_SECURITY_MITIGATIONS_DISABLE_TWEAK_ID: &str =
    "cpu.security-mitigations-disable";
/// Tweak ID for denying realtime process priority requests.
pub const CPU_REALTIME_PRIORITY_TWEAK_ID: &str = "cpu.priority.realtime-game";
/// Tweak ID for denying forced hard affinity requests.
pub const CPU_HARD_AFFINITY_TWEAK_ID: &str = "cpu.hard-affinity.force";
/// Blocked guardrail ID for automatic CPU overclocking or undervolting.
pub const BLOCKED_CPU_AUTO_OC_GUARDRAIL_ID: &str = "blocked.software-overclock-auto";

/// Logical denial target for Intel E-core disable requests.
pub const TARGET_CPU_DISABLE_E_CORES: &str = "blocked:cpu/intel/e-cores-disable";
/// Logical denial target for global SMT/Hyper-Threading disable requests.
pub const TARGET_CPU_DISABLE_SMT: &str = "blocked:cpu/smt-disable";
/// Logical denial target for CPU security mitigation disable requests.
pub const TARGET_CPU_DISABLE_SECURITY_MITIGATIONS: &str =
    "blocked:cpu/security-mitigations-disable";
/// Logical denial target for realtime game priority requests.
pub const TARGET_CPU_REALTIME_PRIORITY: &str = "blocked:cpu/realtime-priority";
/// Logical denial target for forced hard affinity requests.
pub const TARGET_CPU_HARD_AFFINITY: &str = "blocked:cpu/hard-affinity";
/// Logical denial target for automatic CPU overclocking or undervolting.
pub const TARGET_CPU_AUTO_OC: &str = "blocked:cpu/auto-overclock-undervolt";

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

/// Unsafe CPU action requested by a script, catalog entry, or future shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuGuardrailAction {
    /// Request to disable Intel E-cores as a gaming optimization.
    DisableIntelECores,
    /// Request to disable SMT or Hyper-Threading globally.
    DisableSmt,
    /// Request to disable Spectre, Meltdown, or related CPU security mitigations.
    DisableSecurityMitigations,
    /// Request to force realtime priority for a game process.
    ForceRealtimePriority,
    /// Request to force hard CPU affinity for a game or process.
    ForceHardAffinity,
    /// Request to automatically overclock or undervolt CPU settings.
    AutomaticOverclockOrUndervolt,
}

impl CpuGuardrailAction {
    /// All CPU guardrail actions owned by T058.
    pub const ALL: [Self; 6] = [
        Self::DisableIntelECores,
        Self::DisableSmt,
        Self::DisableSecurityMitigations,
        Self::ForceRealtimePriority,
        Self::ForceHardAffinity,
        Self::AutomaticOverclockOrUndervolt,
    ];

    const fn tweak_id(self) -> &'static str {
        match self {
            Self::DisableIntelECores => CPU_INTEL_DISABLE_E_CORES_TWEAK_ID,
            Self::DisableSmt => CPU_SMT_DISABLE_TWEAK_ID,
            Self::DisableSecurityMitigations => CPU_SECURITY_MITIGATIONS_DISABLE_TWEAK_ID,
            Self::ForceRealtimePriority => CPU_REALTIME_PRIORITY_TWEAK_ID,
            Self::ForceHardAffinity => CPU_HARD_AFFINITY_TWEAK_ID,
            Self::AutomaticOverclockOrUndervolt => BLOCKED_CPU_AUTO_OC_GUARDRAIL_ID,
        }
    }

    const fn denial_target(self) -> &'static str {
        match self {
            Self::DisableIntelECores => TARGET_CPU_DISABLE_E_CORES,
            Self::DisableSmt => TARGET_CPU_DISABLE_SMT,
            Self::DisableSecurityMitigations => TARGET_CPU_DISABLE_SECURITY_MITIGATIONS,
            Self::ForceRealtimePriority => TARGET_CPU_REALTIME_PRIORITY,
            Self::ForceHardAffinity => TARGET_CPU_HARD_AFFINITY,
            Self::AutomaticOverclockOrUndervolt => TARGET_CPU_AUTO_OC,
        }
    }

    const fn desired_value(self) -> &'static str {
        match self {
            Self::DisableIntelECores => "disable_e_cores",
            Self::DisableSmt => "disable_smt_or_hyper_threading",
            Self::DisableSecurityMitigations => "disable_cpu_security_mitigations",
            Self::ForceRealtimePriority => "force_realtime_priority",
            Self::ForceHardAffinity => "force_hard_affinity",
            Self::AutomaticOverclockOrUndervolt => "automatic_overclock_or_undervolt",
        }
    }

    const fn denial_warning(self) -> &'static str {
        match self {
            Self::DisableIntelECores => {
                "Intel E-core disable requests are denied; prefer OS and Thread Director-friendly scheduling."
            }
            Self::DisableSmt => {
                "Global SMT or Hyper-Threading disable requests are denied because regressions are workload-specific."
            }
            Self::DisableSecurityMitigations => {
                "CPU security mitigation disable requests are denied because they reduce platform protection."
            }
            Self::ForceRealtimePriority => {
                "Realtime game priority requests are denied because they can starve system threads and increase stutter."
            }
            Self::ForceHardAffinity => {
                "Forced hard affinity requests are denied unless a future benchmark-only Lab profile owns a session-scoped rollback."
            }
            Self::AutomaticOverclockOrUndervolt => {
                "Automatic CPU overclocking or undervolting is denied; keep XTU, Ryzen Master, PBO, and Curve Optimizer guidance advisory."
            }
        }
    }
}

/// Request used to build T058 CPU guardrail denials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuGuardrailPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Unsafe CPU actions requested by a script, catalog entry, or shortcut.
    pub requested_actions: Vec<CpuGuardrailAction>,
}

impl CpuGuardrailPlanRequest {
    /// Creates a CPU guardrail plan request with no unsafe actions requested yet.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            requested_actions: Vec::new(),
        }
    }

    /// Creates a CPU guardrail plan request with explicit unsafe action candidates.
    #[must_use]
    pub fn with_actions(
        plan_id: impl Into<String>,
        requested_actions: Vec<CpuGuardrailAction>,
    ) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            requested_actions,
        }
    }
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

/// Builds a CPU guardrail plan that denies unsafe CPU mutation requests.
#[must_use]
pub fn build_cpu_guardrail_plan(request: &CpuGuardrailPlanRequest) -> TweakPlan {
    let items = CpuGuardrailAction::ALL
        .iter()
        .copied()
        .map(|action| {
            cpu_guardrail_item(action, request.requested_actions.contains(&action))
        })
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

/// Returns true when the ID belongs to the T058 CPU guardrail scope.
#[must_use]
pub fn is_cpu_guardrail_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        CPU_INTEL_DISABLE_E_CORES_TWEAK_ID
            | CPU_SMT_DISABLE_TWEAK_ID
            | CPU_SECURITY_MITIGATIONS_DISABLE_TWEAK_ID
            | CPU_REALTIME_PRIORITY_TWEAK_ID
            | CPU_HARD_AFFINITY_TWEAK_ID
            | BLOCKED_CPU_AUTO_OC_GUARDRAIL_ID
    )
}

/// Returns true when every unsafe CPU request is denied and no apply action exists.
#[must_use]
pub fn cpu_guardrail_plan_blocks_unsafe_actions(plan: &TweakPlan) -> bool {
    !plan.has_apply_items()
        && plan.items.iter().all(|item| {
            is_cpu_guardrail_tweak_id(&item.tweak_id)
                && item.mode == TweakMode::Blocked
                && item.risk == TweakRisk::Critical
                && item.category == TweakCategory::BlockedGuardrail
                && item.backup == BackupRequirement::NotRequired
                && item.rollback.kind == RollbackKind::NotNeededReadonly
                && matches!(item.action, PlanAction::DetectOnly | PlanAction::Deny)
                && item.changes.iter().all(|change| {
                    change.operation == TweakOperationKind::Deny
                        && change.scope == SessionScope::Blocked
                })
        })
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

fn cpu_guardrail_item(action: CpuGuardrailAction, requested: bool) -> TweakPlanItem {
    let changes = if requested {
        vec![PlannedChange {
            target: action.denial_target().to_owned(),
            operation: TweakOperationKind::Deny,
            previous_value: None,
            desired_value: Some(action.desired_value().to_owned()),
            scope: SessionScope::Blocked,
        }]
    } else {
        Vec::new()
    };
    let warnings = if requested {
        vec![action.denial_warning().to_owned()]
    } else {
        Vec::new()
    };

    TweakPlanItem {
        tweak_id: action.tweak_id().to_owned(),
        category: TweakCategory::BlockedGuardrail,
        action: if requested {
            PlanAction::Deny
        } else {
            PlanAction::DetectOnly
        },
        mode: TweakMode::Blocked,
        risk: TweakRisk::Critical,
        changes,
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
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

    #[test]
    fn unsafe_cpu_tweaks_are_denied_with_blocked_changes() {
        let request = CpuGuardrailPlanRequest::with_actions(
            "plan-cpu-denials",
            CpuGuardrailAction::ALL.to_vec(),
        );

        let plan = build_cpu_guardrail_plan(&request);

        assert!(plan.has_denials());
        assert!(!plan.has_apply_items());
        assert!(cpu_guardrail_plan_blocks_unsafe_actions(&plan));
        assert_eq!(plan.items.len(), CpuGuardrailAction::ALL.len());

        for item in &plan.items {
            assert_eq!(item.action, PlanAction::Deny);
            assert_eq!(item.mode, TweakMode::Blocked);
            assert_eq!(item.risk, TweakRisk::Critical);
            assert_eq!(item.category, TweakCategory::BlockedGuardrail);
            assert_eq!(item.backup, BackupRequirement::NotRequired);
            assert_eq!(item.rollback.kind, RollbackKind::NotNeededReadonly);
            assert_eq!(item.reboot, RebootPolicy::None);
            assert!(!item.requires_admin);
            assert_eq!(item.changes.len(), 1);
            assert_eq!(item.changes[0].operation, TweakOperationKind::Deny);
            assert_eq!(item.changes[0].scope, SessionScope::Blocked);
            assert!(!item.warnings.is_empty());
        }
    }

    #[test]
    fn cpu_guardrail_plan_covers_required_denial_ids() {
        let request = CpuGuardrailPlanRequest::with_actions(
            "plan-cpu-denials",
            vec![
                CpuGuardrailAction::DisableIntelECores,
                CpuGuardrailAction::DisableSmt,
                CpuGuardrailAction::DisableSecurityMitigations,
                CpuGuardrailAction::ForceRealtimePriority,
                CpuGuardrailAction::ForceHardAffinity,
                CpuGuardrailAction::AutomaticOverclockOrUndervolt,
            ],
        );

        let plan = build_cpu_guardrail_plan(&request);
        let denied_ids = plan
            .items
            .iter()
            .filter(|item| item.action == PlanAction::Deny)
            .map(|item| item.tweak_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            denied_ids,
            vec![
                CPU_INTEL_DISABLE_E_CORES_TWEAK_ID,
                CPU_SMT_DISABLE_TWEAK_ID,
                CPU_SECURITY_MITIGATIONS_DISABLE_TWEAK_ID,
                CPU_REALTIME_PRIORITY_TWEAK_ID,
                CPU_HARD_AFFINITY_TWEAK_ID,
                BLOCKED_CPU_AUTO_OC_GUARDRAIL_ID,
            ]
        );
    }

    #[test]
    fn cpu_guardrails_are_idle_until_unsafe_action_is_requested() {
        let request = CpuGuardrailPlanRequest::new("plan-cpu-idle");

        let plan = build_cpu_guardrail_plan(&request);

        assert!(!plan.has_denials());
        assert!(!plan.has_apply_items());
        assert!(cpu_guardrail_plan_blocks_unsafe_actions(&plan));
        assert!(plan
            .items
            .iter()
            .all(|item| item.action == PlanAction::DetectOnly && item.changes.is_empty()));
    }
}
