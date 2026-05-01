//! Windows scan and fixture adapters for T057 CPU platform planning.

use std::fmt;

use optimizer_core::{
    cpu_platform::{
        build_cpu_platform_plan, cpu_platform_plan_has_no_unsafe_default,
        is_cpu_platform_tweak_id, CpuCapabilityState, CpuPlatformInspection, CpuPlatformPlanRequest,
        CpuThrottleState, CpuTopology, CpuVendor,
    },
    tweak_contracts::{PlanAction, TweakPlan},
};

use crate::{ServiceScanItem, SystemScanReport, WindowsRollbackFixture};

/// Summary for CPU fixture verification work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuPlatformSettingsSummary {
    /// Count of CPU plan items verified.
    pub item_count: usize,
    /// Tweak IDs verified as read-only or recommendation-only.
    pub tweak_ids: Vec<String>,
}

impl CpuPlatformSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            tweak_ids: Vec::new(),
        }
    }
}

/// Builds a T057 CPU platform plan from read-only scan data.
#[must_use]
pub fn build_cpu_platform_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let inspection = cpu_platform_inspection_from_scan(report);
    build_cpu_platform_plan(&CpuPlatformPlanRequest::new(plan_id, inspection))
}

/// Verifies the T057 CPU platform plan against an in-memory Windows fixture.
///
/// CPU planning for T057 is intentionally read-only, so this verification proves
/// that the plan contains no default apply items or write targets.
pub fn verify_cpu_platform_plan_fixture(
    _fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<CpuPlatformSettingsSummary, CpuPlatformSettingsError> {
    if !cpu_platform_plan_has_no_unsafe_default(plan) {
        return Err(CpuPlatformSettingsError::unsafe_default());
    }

    let mut summary = CpuPlatformSettingsSummary::empty();

    for item in &plan.items {
        if !is_cpu_platform_tweak_id(&item.tweak_id) {
            return Err(CpuPlatformSettingsError::unsupported_tweak(&item.tweak_id));
        }

        if item.action == PlanAction::Apply {
            return Err(CpuPlatformSettingsError::unsafe_default());
        }

        summary.item_count += 1;
        summary.tweak_ids.push(item.tweak_id.clone());
    }

    Ok(summary)
}

fn cpu_platform_inspection_from_scan(report: &SystemScanReport) -> CpuPlatformInspection {
    let processors = report
        .cpus
        .iter()
        .map(|cpu| {
            CpuTopology::from_scan(
                &cpu.name,
                cpu.manufacturer.as_deref(),
                cpu.physical_cores,
                cpu.logical_processors,
                cpu.max_clock_mhz,
            )
        })
        .collect::<Vec<_>>();
    let mut inspection = CpuPlatformInspection::new(processors);
    let has_intel = inspection.has_vendor(CpuVendor::Intel);
    let has_amd = inspection.has_vendor(CpuVendor::Amd);

    inspection.windows_build_number = Some(report.os.build_number.clone());
    inspection.active_power_plan_name = report.power.active_scheme_name.clone();
    inspection.intel_chipset_driver_state = service_readiness(
        has_intel,
        &report.services,
        &[
            "intel chipset",
            "intel dynamic tuning",
            "intel innovation platform",
            "ipfsvc",
        ],
    );
    inspection.intel_dtt_state = service_readiness(
        has_intel,
        &report.services,
        &["dynamic tuning", "dtt", "innovation platform framework", "ipfsvc"],
    );
    inspection.intel_apo_state = service_readiness(
        has_intel,
        &report.services,
        &["application optimization", "intel apo", "apo"],
    );
    inspection.amd_chipset_driver_state = service_readiness(
        has_amd,
        &report.services,
        &["amd chipset", "amd processor", "amdppm", "amd psp"],
    );
    inspection.amd_cppc_state = service_readiness(
        has_amd,
        &report.services,
        &["cppc", "preferred core", "amdppm", "amd processor"],
    );
    inspection.amd_x3d_scheduler_state = service_readiness(
        has_amd && inspection.has_amd_multi_ccd_x3d_cpu(),
        &report.services,
        &["3d v-cache", "amd3dvcache", "amd ppm provisioning", "amdppm"],
    );
    inspection.game_mode_state = if inspection.has_amd_multi_ccd_x3d_cpu() {
        CpuCapabilityState::Unknown
    } else {
        CpuCapabilityState::NotApplicable
    };
    inspection.ppm_settings_state = if report.power.active_scheme_guid.is_some() {
        CpuCapabilityState::Unknown
    } else {
        CpuCapabilityState::Missing
    };
    inspection.thermal_throttling = CpuThrottleState::Unknown;
    inspection.power_limit_throttling = CpuThrottleState::Unknown;
    inspection
}

fn service_readiness(
    applicable: bool,
    services: &[ServiceScanItem],
    needles: &[&str],
) -> CpuCapabilityState {
    if !applicable {
        return CpuCapabilityState::NotApplicable;
    }

    if services.iter().any(|service| service_matches(service, needles)) {
        CpuCapabilityState::Ready
    } else {
        CpuCapabilityState::Missing
    }
}

fn service_matches(service: &ServiceScanItem, needles: &[&str]) -> bool {
    let mut haystack = normalized(&service.name);

    if let Some(display_name) = &service.display_name {
        haystack.push_str(&normalized(display_name));
    }

    needles
        .iter()
        .map(|needle| normalized(needle))
        .any(|needle| haystack.contains(&needle))
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Stable failure reason for fixture-backed CPU platform verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuPlatformSettingsErrorReason {
    /// CPU plan tried to apply or write during default planning.
    UnsafeDefault,
    /// A plan item was outside the T057 CPU scope.
    UnsupportedTweak,
}

impl CpuPlatformSettingsErrorReason {
    /// Returns a stable string representation for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsafeDefault => "unsafe_default",
            Self::UnsupportedTweak => "unsupported_tweak",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsafeDefault => "CPU platform plan must not apply unsafe defaults",
            Self::UnsupportedTweak => "CPU platform plan contains an unsupported tweak",
        }
    }
}

/// Structured error for fixture-backed CPU platform verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpuPlatformSettingsError {
    reason: CpuPlatformSettingsErrorReason,
    tweak_id: Option<String>,
}

impl CpuPlatformSettingsError {
    fn unsafe_default() -> Self {
        Self {
            reason: CpuPlatformSettingsErrorReason::UnsafeDefault,
            tweak_id: None,
        }
    }

    fn unsupported_tweak(tweak_id: &str) -> Self {
        Self {
            reason: CpuPlatformSettingsErrorReason::UnsupportedTweak,
            tweak_id: Some(tweak_id.to_owned()),
        }
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> CpuPlatformSettingsErrorReason {
        self.reason
    }

    /// Returns the associated tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }
}

impl fmt::Display for CpuPlatformSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        Ok(())
    }
}

impl std::error::Error for CpuPlatformSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::cpu_platform::{
        CPU_AMD_CHIPSET_DRIVER_DETECT_TWEAK_ID, CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID,
        CPU_PPM_AUDIT_TWEAK_ID, CPU_VENDOR_TOPOLOGY_TWEAK_ID,
    };

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");

    fn item<'a>(
        plan: &'a TweakPlan,
        tweak_id: &str,
    ) -> &'a optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn scan_fixture_builds_cpu_platform_plan_without_unsafe_default() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_cpu_platform_plan_from_scan("plan-t057-fixture", &report);
        let fixture = WindowsRollbackFixture::new();
        let summary = verify_cpu_platform_plan_fixture(&fixture, &plan)
            .expect("CPU fixture should verify read-only defaults");

        assert!(summary.item_count >= 6);
        assert!(!plan.has_apply_items());
        assert_eq!(
            item(&plan, CPU_VENDOR_TOPOLOGY_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, CPU_AMD_CHIPSET_DRIVER_DETECT_TWEAK_ID).action,
            PlanAction::Recommend
        );
        assert_eq!(
            item(&plan, CPU_AMD_X3D_SCHEDULER_DETECT_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, CPU_PPM_AUDIT_TWEAK_ID).action,
            PlanAction::Recommend
        );
    }

    #[test]
    fn fixture_rejects_cpu_platform_apply_item() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let mut plan = build_cpu_platform_plan_from_scan("plan-t057-unsafe", &report);
        item_mut(&mut plan, CPU_VENDOR_TOPOLOGY_TWEAK_ID).action = PlanAction::Apply;
        let fixture = WindowsRollbackFixture::new();
        let error = verify_cpu_platform_plan_fixture(&fixture, &plan)
            .expect_err("CPU apply item must be rejected");

        assert_eq!(error.reason(), CpuPlatformSettingsErrorReason::UnsafeDefault);
    }

    fn item_mut<'a>(
        plan: &'a mut TweakPlan,
        tweak_id: &str,
    ) -> &'a mut optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter_mut()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }
}
