//! Planning for Windows graphics settings, VRR, HAGS, and app GPU preference.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for PUBG Windows Graphics high-performance GPU preference.
pub const GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID: &str = "game.graphics.preference.pubg";
/// Tweak ID for Windows 11 optimizations for windowed games.
pub const GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID: &str = "game.windowed.optimizations";
/// Tweak ID for variable refresh rate detection and planning.
pub const GAME_VRR_DETECT_PLAN_TWEAK_ID: &str = "game.vrr.detect-plan";
/// Tweak ID for HAGS benchmark-only planning.
pub const GAME_HAGS_BENCHMARK_TWEAK_ID: &str = "game.hags.benchmark";

/// Logical HAGS registry target.
pub const TARGET_HAGS_MODE: &str =
    "registry:hklm/system/currentcontrolset/control/graphicsdrivers/hwschmode";
/// Logical default graphics setting target for windowed game optimizations.
pub const TARGET_WINDOWED_OPTIMIZATIONS: &str =
    "registry:hkcu/software/microsoft/directx/graphicssettings/swapeffectupgradeenable";
/// Logical default graphics setting target for variable refresh rate.
pub const TARGET_VARIABLE_REFRESH_RATE: &str =
    "registry:hkcu/software/microsoft/directx/graphicssettings/variablerefreshrate";
/// Logical prefix for per-app Windows Graphics preferences.
pub const TARGET_GRAPHICS_PREFERENCE_PREFIX: &str =
    "registry:hkcu/software/microsoft/directx/usergpupreferences/";

/// Desired Windows Graphics preference payload for high-performance GPU.
pub const DESIRED_GRAPHICS_PREFERENCE_HIGH_PERFORMANCE: &str = "GpuPreference=2;";
/// Desired enabled value for windowed game optimizations.
pub const DESIRED_WINDOWED_OPTIMIZATIONS_ENABLED: u32 = 1;
/// Desired enabled value for variable refresh rate.
pub const DESIRED_VARIABLE_REFRESH_RATE_ENABLED: u32 = 1;
/// HAGS enabled DWORD value used by Windows Graphics settings.
pub const HAGS_ENABLED_VALUE: u32 = 2;
/// HAGS disabled DWORD value used by Windows Graphics settings.
pub const HAGS_DISABLED_VALUE: u32 = 1;

const GRAPHICS_BENCHMARK_WARNING: &str = concat!(
    "Baseline benchmark is required before applying graphics setting changes; compare ",
    "frametime stability before and after instead of assuming universal FPS gains."
);

/// Current DWORD state for a graphics setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsRegistryDwordState {
    /// The registry value exists and was read.
    Value(u32),
    /// The registry value is absent.
    Missing,
    /// The scan could not prove the value.
    Unknown,
}

impl GraphicsRegistryDwordState {
    /// Converts an optional registry DWORD into a conservative state.
    #[must_use]
    pub const fn from_option(value: Option<u32>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Unknown,
        }
    }

    const fn is_unknown(self) -> bool {
        matches!(self, Self::Unknown)
    }

    fn matches_desired(self, desired: u32) -> bool {
        matches!(self, Self::Value(value) if value == desired)
    }

    fn previous_value(self) -> Option<String> {
        match self {
            Self::Value(value) => Some(value.to_string()),
            Self::Missing | Self::Unknown => None,
        }
    }

    const fn rollback_kind(self) -> RollbackKind {
        match self {
            Self::Value(_) => RollbackKind::ExactValue,
            Self::Missing => RollbackKind::DeleteCreatedValue,
            Self::Unknown => RollbackKind::NotNeededReadonly,
        }
    }
}

/// Current per-app Windows Graphics preference state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphicsPreferenceState {
    /// The per-app value exists and was read.
    Value(String),
    /// No per-app value exists yet.
    Missing,
    /// The scan could not prove the value.
    Unknown,
}

impl GraphicsPreferenceState {
    /// Converts an optional preference payload into a conservative state.
    #[must_use]
    pub fn from_option(value: Option<String>) -> Self {
        match value {
            Some(value) => Self::Value(value),
            None => Self::Unknown,
        }
    }

    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    fn is_high_performance(&self) -> bool {
        matches!(self, Self::Value(value) if value.contains("GpuPreference=2"))
    }

    fn previous_value(&self) -> Option<String> {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Missing | Self::Unknown => None,
        }
    }

    const fn rollback_kind(&self) -> RollbackKind {
        match self {
            Self::Value(_) => RollbackKind::ExactValue,
            Self::Missing => RollbackKind::DeleteCreatedValue,
            Self::Unknown => RollbackKind::NotNeededReadonly,
        }
    }
}

/// Explicit consent state for prompt-only graphics controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsControlConsent {
    /// The user has not accepted this graphics control.
    NotGranted,
    /// The user explicitly accepted this graphics control.
    Granted,
}

impl GraphicsControlConsent {
    fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// HAGS state the user wants to benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HagsBenchmarkTarget {
    /// Keep the current HAGS state and only detect.
    KeepCurrent,
    /// Enable HAGS for a before/after benchmark.
    Enable,
    /// Disable HAGS for a before/after benchmark.
    Disable,
}

impl HagsBenchmarkTarget {
    const fn desired_value(self) -> Option<u32> {
        match self {
            Self::KeepCurrent => None,
            Self::Enable => Some(HAGS_ENABLED_VALUE),
            Self::Disable => Some(HAGS_DISABLED_VALUE),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::KeepCurrent => "keep current HAGS state",
            Self::Enable => "enable HAGS",
            Self::Disable => "disable HAGS",
        }
    }
}

/// Request used to build the T052 graphics settings plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsSettingsPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// PUBG executable path, when detected.
    pub pubg_executable_path: Option<String>,
    /// Current PUBG per-app graphics preference.
    pub pubg_graphics_preference: GraphicsPreferenceState,
    /// Whether Windows exposes a high-performance GPU preference choice.
    pub high_performance_gpu_available: Option<bool>,
    /// Current default windowed game optimization state.
    pub windowed_optimizations: GraphicsRegistryDwordState,
    /// Whether the OS appears to support windowed game optimizations.
    pub windowed_optimizations_supported: Option<bool>,
    /// Current default VRR state.
    pub variable_refresh_rate: GraphicsRegistryDwordState,
    /// Whether the display pipeline appears to support VRR.
    pub variable_refresh_rate_supported: Option<bool>,
    /// Current HAGS state.
    pub hags: GraphicsRegistryDwordState,
    /// Whether HAGS appears available on this GPU and driver.
    pub hags_supported: Option<bool>,
    /// Consent for windowed optimizations.
    pub windowed_optimizations_consent: GraphicsControlConsent,
    /// Consent for VRR.
    pub variable_refresh_rate_consent: GraphicsControlConsent,
    /// Consent for HAGS.
    pub hags_consent: GraphicsControlConsent,
    /// Desired HAGS benchmark target.
    pub hags_target: HagsBenchmarkTarget,
    /// Whether a baseline benchmark exists before applying graphics changes.
    pub baseline_benchmark_captured: bool,
}

impl GraphicsSettingsPlanRequest {
    /// Creates a conservative graphics settings request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            pubg_executable_path: None,
            pubg_graphics_preference: GraphicsPreferenceState::Unknown,
            high_performance_gpu_available: None,
            windowed_optimizations: GraphicsRegistryDwordState::Unknown,
            windowed_optimizations_supported: None,
            variable_refresh_rate: GraphicsRegistryDwordState::Unknown,
            variable_refresh_rate_supported: None,
            hags: GraphicsRegistryDwordState::Unknown,
            hags_supported: None,
            windowed_optimizations_consent: GraphicsControlConsent::NotGranted,
            variable_refresh_rate_consent: GraphicsControlConsent::NotGranted,
            hags_consent: GraphicsControlConsent::NotGranted,
            hags_target: HagsBenchmarkTarget::KeepCurrent,
            baseline_benchmark_captured: false,
        }
    }
}

/// Builds a dry-run plan for T052 graphics settings.
#[must_use]
pub fn build_graphics_settings_plan(request: &GraphicsSettingsPlanRequest) -> TweakPlan {
    let items = vec![
        pubg_graphics_preference_item(request),
        windowed_optimizations_item(request),
        variable_refresh_rate_item(request),
        hags_benchmark_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("graphics")
                || warning.contains("Graphics")
                || warning.contains("HAGS")
                || warning.contains("VRR")
                || warning.contains("windowed")
                || warning.contains("benchmark")
                || warning.contains("PUBG")
        })
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

/// Returns a stable logical target for a per-app Windows Graphics preference.
#[must_use]
pub fn graphics_preference_target(executable_path: &str) -> String {
    let mut normalized = String::new();

    for byte in executable_path.bytes() {
        let next = if byte.is_ascii_alphanumeric() {
            byte.to_ascii_lowercase() as char
        } else if matches!(byte, b'.' | b'-' | b'_') {
            byte as char
        } else {
            '-'
        };

        if normalized.len() < 96 {
            normalized.push(next);
        }
    }

    format!("{TARGET_GRAPHICS_PREFERENCE_PREFIX}{normalized}")
}

/// Returns true when the ID belongs to the T052 graphics settings scope.
#[must_use]
pub fn is_graphics_settings_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID
            | GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID
            | GAME_VRR_DETECT_PLAN_TWEAK_ID
            | GAME_HAGS_BENCHMARK_TWEAK_ID
    )
}

/// Returns true when the target belongs to the T052 graphics settings scope.
#[must_use]
pub fn is_graphics_settings_registry_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_HAGS_MODE | TARGET_WINDOWED_OPTIMIZATIONS | TARGET_VARIABLE_REFRESH_RATE
    ) || target.starts_with(TARGET_GRAPHICS_PREFERENCE_PREFIX)
}

/// Returns true when a tweak ID is paired with its exact graphics setting target.
#[must_use]
pub fn graphics_tweak_targets_setting(tweak_id: &str, target: &str) -> bool {
    matches!(
        (tweak_id, target),
        (GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID, TARGET_WINDOWED_OPTIMIZATIONS)
            | (GAME_VRR_DETECT_PLAN_TWEAK_ID, TARGET_VARIABLE_REFRESH_RATE)
            | (GAME_HAGS_BENCHMARK_TWEAK_ID, TARGET_HAGS_MODE)
    ) || (tweak_id == GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID
        && target.starts_with(TARGET_GRAPHICS_PREFERENCE_PREFIX))
}

/// Returns true when HAGS is not applied by Safe/default planning.
#[must_use]
pub fn graphics_plan_has_no_safe_hags_apply(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        item.tweak_id != GAME_HAGS_BENCHMARK_TWEAK_ID
            || item.action != PlanAction::Apply
            || plan.requested_mode != TweakMode::Safe
    })
}

/// Returns true when any HAGS apply is Competitive, consented, and benchmark-framed.
#[must_use]
pub fn graphics_hags_apply_requires_consent_and_benchmark(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if item.tweak_id != GAME_HAGS_BENCHMARK_TWEAK_ID || item.action != PlanAction::Apply {
            return true;
        }

        item.mode == TweakMode::Competitive
            && item.risk == TweakRisk::Medium
            && item.reboot == RebootPolicy::Required
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("explicit consent"))
            && item
                .warnings
                .iter()
                .any(|warning| warning.contains("Baseline benchmark"))
    })
}

fn pubg_graphics_preference_item(request: &GraphicsSettingsPlanRequest) -> TweakPlanItem {
    let target = request
        .pubg_executable_path
        .as_deref()
        .map(graphics_preference_target);
    let mut warnings = vec![
        "PUBG graphics preference uses Windows per-app Graphics settings only.".to_owned(),
        "No game files, memory, BattlEye files, or anti-cheat processes are modified.".to_owned(),
    ];

    let action = if request.pubg_executable_path.is_none() {
        warnings.push("PUBG executable path is unknown; detect install before planning GPU preference.".to_owned());
        PlanAction::DetectOnly
    } else if request.high_performance_gpu_available == Some(false) {
        warnings.push("Windows did not expose a high-performance GPU preference option.".to_owned());
        PlanAction::DetectOnly
    } else if request.pubg_graphics_preference.is_unknown() {
        warnings.push("Current PUBG graphics preference is unknown; inspect before apply.".to_owned());
        PlanAction::DetectOnly
    } else if request.pubg_graphics_preference.is_high_performance() {
        warnings.push("PUBG already has the high-performance GPU preference.".to_owned());
        PlanAction::DetectOnly
    } else if request.high_performance_gpu_available == Some(true) {
        PlanAction::Apply
    } else {
        warnings.push("High-performance GPU preference support is unknown; keep this as a recommendation.".to_owned());
        PlanAction::Recommend
    };

    let changes = if matches!(action, PlanAction::Apply | PlanAction::Recommend) {
        target
            .into_iter()
            .map(|target| {
                write_change(
                    &target,
                    request.pubg_graphics_preference.previous_value(),
                    DESIRED_GRAPHICS_PREFERENCE_HIGH_PERFORMANCE,
                    SessionScope::ProfileScoped,
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let rollback_kind = request.pubg_graphics_preference.rollback_kind();

    plan_item(GraphicsPlanItemInput {
        tweak_id: GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID,
        action,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes,
        rollback_kind,
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    })
}

fn windowed_optimizations_item(request: &GraphicsSettingsPlanRequest) -> TweakPlanItem {
    let input = ToggleItemInput {
        tweak_id: GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID,
        target: TARGET_WINDOWED_OPTIMIZATIONS,
        current: request.windowed_optimizations,
        desired: DESIRED_WINDOWED_OPTIMIZATIONS_ENABLED,
        supported: request.windowed_optimizations_supported,
        consent: request.windowed_optimizations_consent,
        summary: "Optimizations for windowed games",
        prompt_warning: "Windowed optimizations are prompt-only because per-game compatibility can vary.",
        unsupported_warning: "Optimizations for windowed games are not available on this Windows build.",
        unknown_warning: "Current windowed optimization state is unknown; inspect before apply.",
    };

    safe_toggle_item(input, request)
}

fn variable_refresh_rate_item(request: &GraphicsSettingsPlanRequest) -> TweakPlanItem {
    let input = ToggleItemInput {
        tweak_id: GAME_VRR_DETECT_PLAN_TWEAK_ID,
        target: TARGET_VARIABLE_REFRESH_RATE,
        current: request.variable_refresh_rate,
        desired: DESIRED_VARIABLE_REFRESH_RATE_ENABLED,
        supported: request.variable_refresh_rate_supported,
        consent: request.variable_refresh_rate_consent,
        summary: "Variable refresh rate",
        prompt_warning: "VRR is prompt-only and should be recommended only with a compatible display path.",
        unsupported_warning: "VRR support was not detected for the active display path.",
        unknown_warning: "Current VRR state is unknown; inspect display settings before apply.",
    };

    safe_toggle_item(input, request)
}

#[derive(Clone, Copy)]
struct ToggleItemInput<'a> {
    tweak_id: &'a str,
    target: &'a str,
    current: GraphicsRegistryDwordState,
    desired: u32,
    supported: Option<bool>,
    consent: GraphicsControlConsent,
    summary: &'a str,
    prompt_warning: &'a str,
    unsupported_warning: &'a str,
    unknown_warning: &'a str,
}

fn safe_toggle_item(
    input: ToggleItemInput<'_>,
    request: &GraphicsSettingsPlanRequest,
) -> TweakPlanItem {
    let mut warnings = vec![
        input.prompt_warning.to_owned(),
        GRAPHICS_BENCHMARK_WARNING.to_owned(),
        "No game files, memory, or anti-cheat services are modified.".to_owned(),
    ];

    let action = safe_toggle_action(input, &mut warnings);
    if request.requested_mode != TweakMode::Safe {
        warnings.push(format!(
            "{} remains a Safe prompt even when a higher mode is requested.",
            input.summary
        ));
    }

    let changes = if matches!(action, PlanAction::Apply | PlanAction::Recommend)
        && input.supported == Some(true)
        && !input.current.is_unknown()
        && !input.current.matches_desired(input.desired)
    {
        vec![write_change(
            input.target,
            input.current.previous_value(),
            &input.desired.to_string(),
            SessionScope::Persistent,
        )]
    } else {
        Vec::new()
    };

    plan_item(GraphicsPlanItemInput {
        tweak_id: input.tweak_id,
        action,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes,
        rollback_kind: input.current.rollback_kind(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    })
}

fn safe_toggle_action(input: ToggleItemInput<'_>, warnings: &mut Vec<String>) -> PlanAction {
    if input.supported == Some(false) {
        warnings.push(input.unsupported_warning.to_owned());
        return PlanAction::DetectOnly;
    }

    if input.supported.is_none() {
        warnings.push(format!(
            "{} support is unknown; keep this as a recommendation.",
            input.summary
        ));
        return PlanAction::Recommend;
    }

    if input.current.is_unknown() {
        warnings.push(input.unknown_warning.to_owned());
        return PlanAction::DetectOnly;
    }

    if input.current.matches_desired(input.desired) {
        warnings.push(format!("{} already matches the desired state.", input.summary));
        return PlanAction::DetectOnly;
    }

    if input.consent.is_granted() {
        PlanAction::Apply
    } else {
        warnings.push(format!("{} consent has not been granted.", input.summary));
        PlanAction::Recommend
    }
}

fn hags_benchmark_item(request: &GraphicsSettingsPlanRequest) -> TweakPlanItem {
    let desired = request.hags_target.desired_value();
    let mut warnings = vec![
        format!(
            "HAGS benchmark path requires explicit consent to {}.",
            request.hags_target.label()
        ),
        GRAPHICS_BENCHMARK_WARNING.to_owned(),
        "HAGS changes require a reboot or sign-out boundary before verification.".to_owned(),
        "HAGS changes affect global Windows GPU scheduling state, not a per-game profile.".to_owned(),
        "No game files, memory, BattlEye files, or anti-cheat processes are modified.".to_owned(),
    ];

    let action = match desired {
        None => {
            warnings.push("No HAGS target was selected; detect only.".to_owned());
            PlanAction::DetectOnly
        }
        Some(desired) => hags_action(request, desired, &mut warnings),
    };
    let changes = match desired {
        Some(desired)
            if matches!(action, PlanAction::Apply | PlanAction::Recommend)
                && request.hags_supported == Some(true)
                && !request.hags.is_unknown()
                && !request.hags.matches_desired(desired) =>
        {
            vec![write_change(
                TARGET_HAGS_MODE,
                request.hags.previous_value(),
                &desired.to_string(),
                SessionScope::Persistent,
            )]
        }
        _ => Vec::new(),
    };

    plan_item(GraphicsPlanItemInput {
        tweak_id: GAME_HAGS_BENCHMARK_TWEAK_ID,
        action,
        mode: TweakMode::Competitive,
        risk: TweakRisk::Medium,
        changes,
        rollback_kind: request.hags.rollback_kind(),
        reboot: RebootPolicy::Required,
        requires_admin: true,
        warnings,
    })
}

fn hags_action(
    request: &GraphicsSettingsPlanRequest,
    desired: u32,
    warnings: &mut Vec<String>,
) -> PlanAction {
    if request.hags_supported == Some(false) {
        warnings.push("HAGS is not exposed by this GPU, driver, or Windows build.".to_owned());
        return PlanAction::DetectOnly;
    }

    if request.hags_supported.is_none() {
        warnings.push("HAGS support is unknown; keep this as a recommendation.".to_owned());
        return PlanAction::Recommend;
    }

    if request.hags.is_unknown() {
        warnings.push("Current HAGS state is unknown; inspect and back it up before apply.".to_owned());
        return PlanAction::DetectOnly;
    }

    if request.hags.matches_desired(desired) {
        warnings.push("HAGS already matches the selected benchmark target.".to_owned());
        return PlanAction::DetectOnly;
    }

    if request.requested_mode == TweakMode::Safe {
        warnings.push("HAGS is Competitive and stays off in Safe/default planning.".to_owned());
        return PlanAction::Recommend;
    }

    if !request.hags_consent.is_granted() {
        warnings.push("HAGS consent has not been granted.".to_owned());
        return PlanAction::Recommend;
    }

    if !request.baseline_benchmark_captured {
        warnings.push("Capture a baseline benchmark before applying HAGS.".to_owned());
        return PlanAction::Recommend;
    }

    PlanAction::Apply
}

struct GraphicsPlanItemInput<'a> {
    tweak_id: &'a str,
    action: PlanAction,
    mode: TweakMode,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    rollback_kind: RollbackKind,
    reboot: RebootPolicy,
    requires_admin: bool,
    warnings: Vec<String>,
}

fn plan_item(input: GraphicsPlanItemInput<'_>) -> TweakPlanItem {
    let backup = backup_requirement(input.action, input.rollback_kind, &input.changes);
    let rollback = rollback_plan(
        input.action,
        input.rollback_kind,
        input.reboot,
        input.requires_admin,
        &input.changes,
    );

    TweakPlanItem {
        tweak_id: input.tweak_id.to_owned(),
        category: TweakCategory::WindowsGaming,
        action: input.action,
        mode: input.mode,
        risk: input.risk,
        changes: input.changes,
        backup,
        rollback,
        reboot: input.reboot,
        requires_admin: input.requires_admin,
        warnings: input.warnings,
    }
}

fn backup_requirement(
    action: PlanAction,
    rollback_kind: RollbackKind,
    changes: &[PlannedChange],
) -> BackupRequirement {
    if action == PlanAction::Apply && rollback_kind.needs_backup_before_apply() {
        BackupRequirement::Required {
            kind: rollback_kind,
            target: changes
                .first()
                .map_or_else(String::new, |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(
    action: PlanAction,
    rollback_kind: RollbackKind,
    reboot: RebootPolicy,
    requires_admin: bool,
    changes: &[PlannedChange],
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: rollback_kind,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous Windows graphics setting.".to_owned(),
                target: change.target.clone(),
                operation: if rollback_kind == RollbackKind::DeleteCreatedValue {
                    TweakOperationKind::Delete
                } else {
                    TweakOperationKind::Write
                },
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin,
        reboot,
        manual_instructions: Vec::new(),
    }
}

fn write_change(
    target: &str,
    previous_value: Option<String>,
    desired_value: &str,
    scope: SessionScope,
) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value,
        desired_value: Some(desired_value.to_owned()),
        scope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GraphicsSettingsPlanRequest {
        let mut request = GraphicsSettingsPlanRequest::new("plan-graphics");
        request.pubg_executable_path = Some("C:/Games/PUBG/TslGame.exe".to_owned());
        request.pubg_graphics_preference = GraphicsPreferenceState::Missing;
        request.high_performance_gpu_available = Some(true);
        request.windowed_optimizations = GraphicsRegistryDwordState::Value(0);
        request.windowed_optimizations_supported = Some(true);
        request.variable_refresh_rate = GraphicsRegistryDwordState::Value(0);
        request.variable_refresh_rate_supported = Some(true);
        request.hags = GraphicsRegistryDwordState::Value(HAGS_DISABLED_VALUE);
        request.hags_supported = Some(true);
        request.hags_target = HagsBenchmarkTarget::Enable;
        request
    }

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn safe_plan_sets_pubg_high_performance_when_supported() {
        let request = request();
        let plan = build_graphics_settings_plan(&request);
        let pubg = item(&plan, GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID);

        assert_eq!(pubg.action, PlanAction::Apply);
        assert_eq!(
            pubg.backup,
            BackupRequirement::Required {
                kind: RollbackKind::DeleteCreatedValue,
                target: graphics_preference_target("C:/Games/PUBG/TslGame.exe"),
            }
        );
        assert_eq!(
            pubg.changes[0].desired_value.as_deref(),
            Some(DESIRED_GRAPHICS_PREFERENCE_HIGH_PERFORMANCE)
        );
        assert_eq!(pubg.rollback.kind, RollbackKind::DeleteCreatedValue);
        assert!(pubg.changes[0]
            .target
            .starts_with(TARGET_GRAPHICS_PREFERENCE_PREFIX));
    }

    #[test]
    fn windowed_and_vrr_are_prompted_until_consented() {
        let mut request = request();
        let prompt_plan = build_graphics_settings_plan(&request);

        assert_eq!(
            item(&prompt_plan, GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID).action,
            PlanAction::Recommend
        );
        assert_eq!(
            item(&prompt_plan, GAME_VRR_DETECT_PLAN_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.windowed_optimizations_consent = GraphicsControlConsent::Granted;
        request.variable_refresh_rate_consent = GraphicsControlConsent::Granted;
        let apply_plan = build_graphics_settings_plan(&request);

        assert_eq!(
            item(&apply_plan, GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID).action,
            PlanAction::Apply
        );
        assert_eq!(
            item(&apply_plan, GAME_VRR_DETECT_PLAN_TWEAK_ID).action,
            PlanAction::Apply
        );
    }

    #[test]
    fn hags_never_applies_from_safe_or_without_benchmark() {
        let mut request = request();
        request.hags_consent = GraphicsControlConsent::Granted;
        request.baseline_benchmark_captured = true;
        let safe_plan = build_graphics_settings_plan(&request);

        assert_eq!(
            item(&safe_plan, GAME_HAGS_BENCHMARK_TWEAK_ID).action,
            PlanAction::Recommend
        );
        assert!(graphics_plan_has_no_safe_hags_apply(&safe_plan));

        request.requested_mode = TweakMode::Competitive;
        request.baseline_benchmark_captured = false;
        let no_baseline_plan = build_graphics_settings_plan(&request);
        assert_eq!(
            item(&no_baseline_plan, GAME_HAGS_BENCHMARK_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.baseline_benchmark_captured = true;
        let apply_plan = build_graphics_settings_plan(&request);
        let hags = item(&apply_plan, GAME_HAGS_BENCHMARK_TWEAK_ID);

        assert_eq!(hags.action, PlanAction::Apply);
        assert_eq!(hags.reboot, RebootPolicy::Required);
        assert_eq!(hags.requires_admin, true);
        assert_eq!(hags.changes[0].desired_value.as_deref(), Some("2"));
        assert!(graphics_hags_apply_requires_consent_and_benchmark(&apply_plan));
    }

    #[test]
    fn unsupported_or_unknown_graphics_states_do_not_apply() {
        let mut request = request();
        request.high_performance_gpu_available = Some(false);
        request.windowed_optimizations = GraphicsRegistryDwordState::Unknown;
        request.variable_refresh_rate_supported = Some(false);
        request.hags_supported = Some(false);
        request.requested_mode = TweakMode::Competitive;
        request.hags_consent = GraphicsControlConsent::Granted;
        request.baseline_benchmark_captured = true;

        let plan = build_graphics_settings_plan(&request);

        assert_eq!(
            item(&plan, GAME_GRAPHICS_PREFERENCE_PUBG_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, GAME_WINDOWED_OPTIMIZATIONS_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, GAME_VRR_DETECT_PLAN_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert_eq!(
            item(&plan, GAME_HAGS_BENCHMARK_TWEAK_ID).action,
            PlanAction::DetectOnly
        );
        assert!(!plan.has_apply_items());
    }
}
