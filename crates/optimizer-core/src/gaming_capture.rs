//! Safe planning for Windows Game DVR, capture, Game Bar, and focus controls.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for disabling background Game DVR and capture recording.
pub const GAME_CAPTURE_BACKGROUND_TWEAK_ID: &str = "game.capture.background.off";
/// Tweak ID for warning about color-sensitive present paths.
pub const GAME_CAPTURE_COLOR_WARNING_TWEAK_ID: &str = "game.capture.color-pipeline-warning";
/// Tweak ID for optional Game Bar overlay reduction.
pub const GAME_BAR_OVERLAY_TWEAK_ID: &str = "game.bar.overlay.optional";
/// Tweak ID for session-scoped focus mode during gaming.
pub const GAME_NOTIFICATIONS_FOCUS_TWEAK_ID: &str = "game.notifications.focus";

/// HKCU GameConfigStore GameDVR state.
pub const TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED: &str =
    "registry:hkcu/system/gameconfigstore/gamedvr_enabled";
/// HKCU GameDVR app capture state.
pub const TARGET_GAME_DVR_APP_CAPTURE_ENABLED: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/gamedvr/appcaptureenabled";
/// HKCU GameDVR historical/background capture state.
pub const TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/gamedvr/historicalcaptureenabled";
/// Optional HKLM policy for allowing GameDVR.
pub const TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR: &str =
    "registry:hklm/software/policies/microsoft/windows/gamedvr/allowgamedvr";
/// HKCU Game Bar Nexus overlay launch state.
pub const TARGET_GAME_BAR_USE_NEXUS_FOR_GAME_BAR_ENABLED: &str =
    "registry:hkcu/software/microsoft/gamebar/usenexusforgamebarenabled";
/// HKCU Game Bar startup panel state.
pub const TARGET_GAME_BAR_SHOW_STARTUP_PANEL: &str =
    "registry:hkcu/software/microsoft/gamebar/showstartuppanel";
/// HKCU notifications DND state used for session-scoped gaming focus.
pub const TARGET_NOTIFICATIONS_DND_ENABLED: &str =
    "registry:hkcu/software/microsoft/windows/currentversion/notifications/settings/noc_global_setting_dnd";

/// Whether a Windows gaming feature appears to be used by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamingFeatureUse {
    /// Scan or user preference says the feature is used.
    Used,
    /// Scan or user preference says the feature is not used.
    NotUsed,
    /// Usage is unknown and should be treated as prompt-only.
    Unknown,
}

/// Explicit consent state for prompt-only controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamingControlConsent {
    /// The user has not accepted this optional control.
    NotGranted,
    /// The user explicitly accepted this optional control.
    Granted,
}

impl GamingControlConsent {
    /// Returns true when consent was granted.
    #[must_use]
    pub const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Display/color pipeline state relevant to GameDVR and overlay changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPipelineState {
    /// No known HDR, ICC, or exclusive-fullscreen color workflow risk.
    Standard,
    /// HDR, ICC, exclusive fullscreen, or similar color-sensitive workflow detected.
    Sensitive,
}

impl ColorPipelineState {
    /// Returns true when GameDVR or overlay changes need a color warning.
    #[must_use]
    pub const fn needs_warning(self) -> bool {
        matches!(self, Self::Sensitive)
    }
}

/// Request used to build the safe gaming capture plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamingCapturePlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Whether Windows capture/recording is used.
    pub windows_capture_use: GamingFeatureUse,
    /// Whether Game Bar widgets/overlay are used.
    pub game_bar_use: GamingFeatureUse,
    /// Explicit consent for disabling the Game Bar overlay.
    pub game_bar_overlay_consent: GamingControlConsent,
    /// Explicit consent for session-scoped focus mode.
    pub focus_assist_consent: GamingControlConsent,
    /// Display/color pipeline state.
    pub color_pipeline: ColorPipelineState,
    /// Whether to include the machine-wide GameDVR policy value.
    pub include_machine_policy: bool,
}

impl GamingCapturePlanRequest {
    /// Creates a safe default request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            windows_capture_use: GamingFeatureUse::Unknown,
            game_bar_use: GamingFeatureUse::Unknown,
            game_bar_overlay_consent: GamingControlConsent::NotGranted,
            focus_assist_consent: GamingControlConsent::NotGranted,
            color_pipeline: ColorPipelineState::Standard,
            include_machine_policy: false,
        }
    }
}

/// Builds a dry-run plan for T042 safe gaming capture controls.
#[must_use]
pub fn build_gaming_capture_plan(request: &GamingCapturePlanRequest) -> TweakPlan {
    let mut items = vec![
        background_capture_item(request),
        color_pipeline_warning_item(request),
        game_bar_overlay_item(request),
        focus_assist_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| warning.contains("HDR") || warning.contains("ICC"))
        .cloned()
        .collect();

    if request.requested_mode == TweakMode::Safe {
        TweakPlan {
            id: request.plan_id.clone(),
            requested_mode: request.requested_mode,
            catalog_schema_version: SUPPORTED_CATALOG_SCHEMA_VERSION.to_owned(),
            items,
            warnings,
        }
    } else {
        for item in &mut items {
            item.warnings
                .push("T042 gaming capture controls are safe-mode controls.".to_owned());
        }

        TweakPlan {
            id: request.plan_id.clone(),
            requested_mode: request.requested_mode,
            catalog_schema_version: SUPPORTED_CATALOG_SCHEMA_VERSION.to_owned(),
            items,
            warnings,
        }
    }
}

/// Returns true when the ID belongs to a T042 gaming capture control.
#[must_use]
pub fn is_gaming_capture_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        GAME_CAPTURE_BACKGROUND_TWEAK_ID
            | GAME_CAPTURE_COLOR_WARNING_TWEAK_ID
            | GAME_BAR_OVERLAY_TWEAK_ID
            | GAME_NOTIFICATIONS_FOCUS_TWEAK_ID
    )
}

/// Returns true when the target is one of the typed T042 registry targets.
#[must_use]
pub fn is_gaming_capture_registry_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED
            | TARGET_GAME_DVR_APP_CAPTURE_ENABLED
            | TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED
            | TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR
            | TARGET_GAME_BAR_USE_NEXUS_FOR_GAME_BAR_ENABLED
            | TARGET_GAME_BAR_SHOW_STARTUP_PANEL
            | TARGET_NOTIFICATIONS_DND_ENABLED
    )
}

fn background_capture_item(request: &GamingCapturePlanRequest) -> TweakPlanItem {
    let mut warnings = color_warnings(request);
    let action = match request.windows_capture_use {
        GamingFeatureUse::NotUsed => PlanAction::Apply,
        GamingFeatureUse::Used => {
            warnings.push(
                "Windows capture or recording appears to be used; keep this as a user choice."
                    .to_owned(),
            );
            PlanAction::Recommend
        }
        GamingFeatureUse::Unknown => {
            warnings.push(
                "Windows capture usage is unknown; ask before disabling capture features."
                    .to_owned(),
            );
            PlanAction::Recommend
        }
    };
    let changes = background_capture_changes(request.include_machine_policy);

    plan_item(
        GAME_CAPTURE_BACKGROUND_TWEAK_ID,
        action,
        TweakRisk::Low,
        changes,
        request.include_machine_policy,
        warnings,
    )
}

fn color_pipeline_warning_item(request: &GamingCapturePlanRequest) -> TweakPlanItem {
    let warnings = if request.color_pipeline.needs_warning() {
        color_warnings(request)
    } else {
        Vec::new()
    };

    TweakPlanItem {
        tweak_id: GAME_CAPTURE_COLOR_WARNING_TWEAK_ID.to_owned(),
        category: TweakCategory::WindowsGaming,
        action: PlanAction::DetectOnly,
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

fn game_bar_overlay_item(request: &GamingCapturePlanRequest) -> TweakPlanItem {
    let mut warnings = color_warnings(request);
    let can_apply = request.game_bar_overlay_consent.is_granted()
        && request.game_bar_use == GamingFeatureUse::NotUsed
        && request.windows_capture_use != GamingFeatureUse::Used;

    let action = if can_apply {
        PlanAction::Apply
    } else {
        if !request.game_bar_overlay_consent.is_granted() {
            warnings.push("Game Bar overlay disable is prompt-only.".to_owned());
        }

        if request.game_bar_use == GamingFeatureUse::Used {
            warnings.push(
                "Game Bar widgets appear to be used; do not disable the overlay automatically."
                    .to_owned(),
            );
        }

        if request.windows_capture_use == GamingFeatureUse::Used {
            warnings.push(
                "Windows capture appears to be used; keep Game Bar overlay controls optional."
                    .to_owned(),
            );
        }

        PlanAction::Recommend
    };

    plan_item(
        GAME_BAR_OVERLAY_TWEAK_ID,
        action,
        TweakRisk::Low,
        vec![
            write_change(
                TARGET_GAME_BAR_USE_NEXUS_FOR_GAME_BAR_ENABLED,
                "0",
                SessionScope::Persistent,
            ),
            write_change(TARGET_GAME_BAR_SHOW_STARTUP_PANEL, "0", SessionScope::Persistent),
        ],
        false,
        warnings,
    )
}

fn focus_assist_item(request: &GamingCapturePlanRequest) -> TweakPlanItem {
    let mut warnings = Vec::new();
    let action = if request.focus_assist_consent.is_granted() {
        PlanAction::Apply
    } else {
        warnings.push("Gaming focus mode is session-scoped and prompt-only.".to_owned());
        PlanAction::Recommend
    };

    plan_item(
        GAME_NOTIFICATIONS_FOCUS_TWEAK_ID,
        action,
        TweakRisk::Low,
        vec![write_change(
            TARGET_NOTIFICATIONS_DND_ENABLED,
            "1",
            SessionScope::SessionOnly,
        )],
        false,
        warnings,
    )
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    requires_admin: bool,
    warnings: Vec<String>,
) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::WindowsGaming,
        action,
        mode: TweakMode::Safe,
        risk,
        backup: backup_requirement(action, &changes),
        rollback: rollback_plan(&changes, requires_admin),
        changes,
        reboot: RebootPolicy::None,
        requires_admin,
        warnings,
    }
}

fn backup_requirement(action: PlanAction, changes: &[PlannedChange]) -> BackupRequirement {
    if action == PlanAction::Apply {
        BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: changes
                .first()
                .map_or_else(String::new, |change| change.target.clone()),
        }
    } else {
        BackupRequirement::NotRequired
    }
}

fn rollback_plan(changes: &[PlannedChange], requires_admin: bool) -> RollbackPlan {
    if changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: RollbackKind::ExactValue,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous gaming capture setting.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: None,
            })
            .collect(),
        requires_admin,
        reboot: RebootPolicy::None,
        manual_instructions: Vec::new(),
    }
}

fn background_capture_changes(include_machine_policy: bool) -> Vec<PlannedChange> {
    let mut changes = vec![
        write_change(
            TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED,
            "0",
            SessionScope::Persistent,
        ),
        write_change(
            TARGET_GAME_DVR_APP_CAPTURE_ENABLED,
            "0",
            SessionScope::Persistent,
        ),
        write_change(
            TARGET_GAME_DVR_HISTORICAL_CAPTURE_ENABLED,
            "0",
            SessionScope::Persistent,
        ),
    ];

    if include_machine_policy {
        changes.push(write_change(
            TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR,
            "0",
            SessionScope::Persistent,
        ));
    }

    changes
}

fn write_change(target: &str, value: &str, scope: SessionScope) -> PlannedChange {
    PlannedChange {
        target: target.to_owned(),
        operation: TweakOperationKind::Write,
        previous_value: None,
        desired_value: Some(value.to_owned()),
        scope,
    }
}

fn color_warnings(request: &GamingCapturePlanRequest) -> Vec<String> {
    if request.color_pipeline.needs_warning() {
        vec![concat!(
            "HDR, ICC, or exclusive fullscreen color workflow detected; warn before ",
            "GameDVR, capture, or overlay changes."
        )
        .to_owned()]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GamingCapturePlanRequest {
        GamingCapturePlanRequest::new("plan-gaming-capture")
    }

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("tweak item should exist")
    }

    #[test]
    fn safe_plan_disables_background_capture_when_capture_is_unused() {
        let mut request = request();
        request.windows_capture_use = GamingFeatureUse::NotUsed;
        request.include_machine_policy = true;
        let plan = build_gaming_capture_plan(&request);
        let item = item(&plan, GAME_CAPTURE_BACKGROUND_TWEAK_ID);

        assert_eq!(item.action, PlanAction::Apply);
        assert_eq!(item.backup, BackupRequirement::Required {
            kind: RollbackKind::ExactValue,
            target: TARGET_GAME_CONFIG_STORE_GAME_DVR_ENABLED.to_owned(),
        });
        assert_eq!(item.changes.len(), 4);
        assert!(item.requires_admin);
        assert!(item
            .changes
            .iter()
            .any(|change| change.target == TARGET_GAME_DVR_POLICY_ALLOW_GAME_DVR));
        assert!(item
            .changes
            .iter()
            .all(|change| change.desired_value.as_deref() == Some("0")));
    }

    #[test]
    fn capture_usage_and_color_pipeline_keep_change_prompted_with_warning() {
        let mut request = request();
        request.windows_capture_use = GamingFeatureUse::Used;
        request.color_pipeline = ColorPipelineState::Sensitive;
        let plan = build_gaming_capture_plan(&request);
        let capture = item(&plan, GAME_CAPTURE_BACKGROUND_TWEAK_ID);
        let warning = item(&plan, GAME_CAPTURE_COLOR_WARNING_TWEAK_ID);

        assert_eq!(capture.action, PlanAction::Recommend);
        assert_eq!(capture.backup, BackupRequirement::NotRequired);
        assert!(warning
            .warnings
            .iter()
            .any(|message| message.contains("HDR, ICC")));
        assert!(plan
            .warnings
            .iter()
            .any(|message| message.contains("exclusive fullscreen")));
    }

    #[test]
    fn game_bar_overlay_requires_prompt_and_never_removes_xbox_packages() {
        let mut request = request();
        request.windows_capture_use = GamingFeatureUse::NotUsed;
        request.game_bar_use = GamingFeatureUse::NotUsed;
        let plan_without_consent = build_gaming_capture_plan(&request);

        assert_eq!(
            item(&plan_without_consent, GAME_BAR_OVERLAY_TWEAK_ID).action,
            PlanAction::Recommend
        );

        request.game_bar_overlay_consent = GamingControlConsent::Granted;
        let plan_with_consent = build_gaming_capture_plan(&request);
        let overlay = item(&plan_with_consent, GAME_BAR_OVERLAY_TWEAK_ID);

        assert_eq!(overlay.action, PlanAction::Apply);
        assert!(overlay
            .changes
            .iter()
            .all(|change| !change.target.to_ascii_lowercase().contains("xbox")));
    }

    #[test]
    fn focus_assist_is_session_scoped_and_rollback_capable_when_consented() {
        let mut request = request();
        request.focus_assist_consent = GamingControlConsent::Granted;
        let plan = build_gaming_capture_plan(&request);
        let focus = item(&plan, GAME_NOTIFICATIONS_FOCUS_TWEAK_ID);

        assert_eq!(focus.action, PlanAction::Apply);
        assert_eq!(focus.changes.len(), 1);
        assert_eq!(focus.changes[0].scope, SessionScope::SessionOnly);
        assert_eq!(
            focus.changes[0].target,
            TARGET_NOTIFICATIONS_DND_ENABLED.to_owned()
        );
        assert_eq!(focus.rollback.kind, RollbackKind::ExactValue);
        assert_eq!(focus.rollback.steps.len(), 1);
    }
}
