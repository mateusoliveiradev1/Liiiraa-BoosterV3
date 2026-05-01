//! Recommendation-only planning for startup and background app reviews.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackPlan, SessionScope,
        TweakCategory, TweakMode, TweakOperationKind, TweakPlan, TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for reviewing startup applications.
pub const BG_STARTUP_REVIEW_TWEAK_ID: &str = "bg.startup.review";
/// Tweak ID for reviewing background app activity.
pub const BG_BACKGROUND_APPS_REVIEW_TWEAK_ID: &str = "bg.background-apps.review";

const STARTUP_TARGET_PREFIX: &str = "startup:";
const BACKGROUND_APP_TARGET_PREFIX: &str = "background-app:";

/// Startup impact reported by Windows or fixture inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupImpact {
    /// Startup entry has high launch impact.
    High,
    /// Startup entry has medium launch impact.
    Medium,
    /// Startup entry has low launch impact.
    Low,
    /// Windows has not measured this entry yet.
    NotMeasured,
    /// Impact could not be determined.
    Unknown,
}

impl StartupImpact {
    /// Returns true when impact is strong enough to recommend review.
    #[must_use]
    pub const fn should_recommend_review(self) -> bool {
        matches!(self, Self::High | Self::Medium)
    }

    fn as_state(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::NotMeasured => "not_measured",
            Self::Unknown => "unknown",
        }
    }
}

/// Observed background app activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundAppActivity {
    /// App appears noisy enough to review.
    High,
    /// App has moderate background activity.
    Moderate,
    /// App activity is low.
    Low,
    /// Activity could not be determined.
    Unknown,
}

impl BackgroundAppActivity {
    /// Returns true when activity is strong enough to recommend review.
    #[must_use]
    pub const fn should_recommend_review(self) -> bool {
        matches!(self, Self::High | Self::Moderate)
    }

    fn as_state(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Moderate => "moderate",
            Self::Low => "low",
            Self::Unknown => "unknown",
        }
    }
}

/// Recommendation safety classification for app startup/background entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppRecommendationClass {
    /// Entry is known noncritical and can be recommended for user review.
    KnownNonCritical,
    /// Entry looks system, security, driver, or hardware critical.
    SystemCritical,
    /// Entry is unknown, so the optimizer should avoid recommending reduction.
    Unknown,
}

impl AppRecommendationClass {
    const fn can_recommend_reduction(self) -> bool {
        matches!(self, Self::KnownNonCritical)
    }
}

/// A startup entry discovered during inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupAppInspection {
    /// Display name from Windows startup inventory.
    pub name: String,
    /// Command line or executable path, when available.
    pub command: Option<String>,
    /// Startup source location.
    pub location: Option<String>,
    /// Owning user, when available.
    pub user: Option<String>,
    /// Whether the entry is enabled.
    pub enabled: Option<bool>,
    /// Startup impact signal.
    pub impact: StartupImpact,
    /// Safety classification for recommendation.
    pub recommendation_class: AppRecommendationClass,
}

impl StartupAppInspection {
    /// Creates an inspection item with conservative defaults.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: None,
            location: None,
            user: None,
            enabled: None,
            impact: StartupImpact::Unknown,
            recommendation_class: AppRecommendationClass::Unknown,
        }
    }

    /// Adds a command line to the inspection.
    #[must_use]
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Adds a startup location to the inspection.
    #[must_use]
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Adds the owning user to the inspection.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Adds enabled state to the inspection.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Adds startup impact to the inspection.
    #[must_use]
    pub const fn with_impact(mut self, impact: StartupImpact) -> Self {
        self.impact = impact;
        self
    }

    /// Adds safety classification to the inspection.
    #[must_use]
    pub const fn with_recommendation_class(
        mut self,
        recommendation_class: AppRecommendationClass,
    ) -> Self {
        self.recommendation_class = recommendation_class;
        self
    }

    fn is_recommendation_candidate(&self) -> bool {
        self.enabled != Some(false)
            && self.impact.should_recommend_review()
            && self.recommendation_class.can_recommend_reduction()
    }
}

/// A background app discovered during inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundAppInspection {
    /// Display name shown to the user.
    pub name: String,
    /// Package, app, or stable inventory identifier.
    pub app_id: String,
    /// Whether background permission appears enabled.
    pub enabled: Option<bool>,
    /// Observed activity signal.
    pub activity: BackgroundAppActivity,
    /// Safety classification for recommendation.
    pub recommendation_class: AppRecommendationClass,
}

impl BackgroundAppInspection {
    /// Creates a background app inspection with conservative defaults.
    #[must_use]
    pub fn new(name: impl Into<String>, app_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            app_id: app_id.into(),
            enabled: None,
            activity: BackgroundAppActivity::Unknown,
            recommendation_class: AppRecommendationClass::Unknown,
        }
    }

    /// Adds enabled state to the inspection.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Adds activity state to the inspection.
    #[must_use]
    pub const fn with_activity(mut self, activity: BackgroundAppActivity) -> Self {
        self.activity = activity;
        self
    }

    /// Adds safety classification to the inspection.
    #[must_use]
    pub const fn with_recommendation_class(
        mut self,
        recommendation_class: AppRecommendationClass,
    ) -> Self {
        self.recommendation_class = recommendation_class;
        self
    }

    fn is_recommendation_candidate(&self) -> bool {
        self.enabled != Some(false)
            && self.activity.should_recommend_review()
            && self.recommendation_class.can_recommend_reduction()
    }
}

/// Request for a T043 background work review plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundWorkPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Startup entries discovered by inspection.
    pub startup_apps: Vec<StartupAppInspection>,
    /// Background apps discovered by inspection.
    pub background_apps: Vec<BackgroundAppInspection>,
}

impl BackgroundWorkPlanRequest {
    /// Creates an empty safe-mode background work review request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            startup_apps: Vec::new(),
            background_apps: Vec::new(),
        }
    }
}

/// Builds a recommendation-only plan for startup/background app inspection.
#[must_use]
pub fn build_background_work_plan(request: &BackgroundWorkPlanRequest) -> TweakPlan {
    let items = vec![startup_review_item(request), background_apps_review_item(request)];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| warning.contains("Recommendation-only"))
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

/// Returns true when the ID belongs to the T043 background work scope.
#[must_use]
pub fn is_background_work_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        BG_STARTUP_REVIEW_TWEAK_ID | BG_BACKGROUND_APPS_REVIEW_TWEAK_ID
    )
}

/// Returns true when a logical target belongs to T043 recommendations.
#[must_use]
pub fn is_background_work_recommendation_target(target: &str) -> bool {
    target.starts_with(STARTUP_TARGET_PREFIX) || target.starts_with(BACKGROUND_APP_TARGET_PREFIX)
}

/// Returns true when a plan keeps T043 work recommendation-only.
#[must_use]
pub fn plan_is_recommendation_only(plan: &TweakPlan) -> bool {
    plan.items
        .iter()
        .filter(|item| is_background_work_tweak_id(&item.tweak_id))
        .all(|item| {
            item.action != PlanAction::Apply
                && item.backup == BackupRequirement::NotRequired
                && item.rollback == RollbackPlan::not_needed()
                && item.changes.iter().all(|change| {
                    change.operation == TweakOperationKind::Manual
                        && change.scope == SessionScope::RecommendationOnly
                        && is_background_work_recommendation_target(&change.target)
                })
        })
}

fn startup_review_item(request: &BackgroundWorkPlanRequest) -> TweakPlanItem {
    let candidates = request
        .startup_apps
        .iter()
        .filter(|app| app.is_recommendation_candidate())
        .collect::<Vec<_>>();

    let mut warnings = recommendation_warnings(request);
    let skipped_critical = request
        .startup_apps
        .iter()
        .filter(|app| app.recommendation_class == AppRecommendationClass::SystemCritical)
        .count();

    if skipped_critical > 0 {
        warnings.push(
            "System, security, driver, or hardware startup entries were left unchanged."
                .to_owned(),
        );
    }

    plan_item(
        BG_STARTUP_REVIEW_TWEAK_ID,
        action_for_candidates(candidates.len()),
        candidates
            .into_iter()
            .map(startup_recommendation_change)
            .collect(),
        warnings,
    )
}

fn background_apps_review_item(request: &BackgroundWorkPlanRequest) -> TweakPlanItem {
    let candidates = request
        .background_apps
        .iter()
        .filter(|app| app.is_recommendation_candidate())
        .collect::<Vec<_>>();

    let mut warnings = recommendation_warnings(request);
    let skipped_critical = request
        .background_apps
        .iter()
        .filter(|app| app.recommendation_class == AppRecommendationClass::SystemCritical)
        .count();

    if skipped_critical > 0 {
        warnings.push("System or security background apps were left unchanged.".to_owned());
    }

    plan_item(
        BG_BACKGROUND_APPS_REVIEW_TWEAK_ID,
        action_for_candidates(candidates.len()),
        candidates
            .into_iter()
            .map(background_app_recommendation_change)
            .collect(),
        warnings,
    )
}

fn recommendation_warnings(request: &BackgroundWorkPlanRequest) -> Vec<String> {
    let mut warnings = vec![concat!(
        "Recommendation-only: startup and background app review does not disable entries ",
        "without a typed backup/apply adapter."
    )
    .to_owned()];

    if request.requested_mode != TweakMode::Safe {
        warnings.push("T043 background work review remains safe-mode only.".to_owned());
    }

    warnings
}

const fn action_for_candidates(candidate_count: usize) -> PlanAction {
    if candidate_count == 0 {
        PlanAction::DetectOnly
    } else {
        PlanAction::Recommend
    }
}

fn plan_item(
    tweak_id: &str,
    action: PlanAction,
    changes: Vec<PlannedChange>,
    warnings: Vec<String>,
) -> TweakPlanItem {
    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: TweakCategory::BackgroundWork,
        action,
        mode: TweakMode::Safe,
        risk: TweakRisk::Low,
        changes,
        backup: BackupRequirement::NotRequired,
        rollback: RollbackPlan::not_needed(),
        reboot: RebootPolicy::None,
        requires_admin: false,
        warnings,
    }
}

fn startup_recommendation_change(app: &StartupAppInspection) -> PlannedChange {
    PlannedChange {
        target: format!("{STARTUP_TARGET_PREFIX}{}", target_slug(&app.name)),
        operation: TweakOperationKind::Manual,
        previous_value: Some(format!(
            "enabled={},impact={}",
            state_from_enabled(app.enabled),
            app.impact.as_state()
        )),
        desired_value: Some("review_disable_startup".to_owned()),
        scope: SessionScope::RecommendationOnly,
    }
}

fn background_app_recommendation_change(app: &BackgroundAppInspection) -> PlannedChange {
    PlannedChange {
        target: format!("{BACKGROUND_APP_TARGET_PREFIX}{}", target_slug(&app.app_id)),
        operation: TweakOperationKind::Manual,
        previous_value: Some(format!(
            "enabled={},activity={}",
            state_from_enabled(app.enabled),
            app.activity.as_state()
        )),
        desired_value: Some("review_background_permission".to_owned()),
        scope: SessionScope::RecommendationOnly,
    }
}

fn state_from_enabled(enabled: Option<bool>) -> &'static str {
    match enabled {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn target_slug(value: &str) -> String {
    let mut slug = String::new();

    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => slug.push(byte as char),
            b'A'..=b'Z' => slug.push((byte + 32) as char),
            b'.' | b'-' | b'_' => slug.push(byte as char),
            _ if !slug.ends_with('-') => slug.push('-'),
            _ => {}
        }
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "unknown".to_owned()
    } else {
        slug.to_owned()
    }
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

    #[test]
    fn startup_review_recommends_only_known_noncritical_high_impact_entries() {
        let mut request = BackgroundWorkPlanRequest::new("plan-background-work");
        request.startup_apps = vec![
            StartupAppInspection::new("Discord")
                .with_enabled(true)
                .with_impact(StartupImpact::High)
                .with_recommendation_class(AppRecommendationClass::KnownNonCritical),
            StartupAppInspection::new("SecurityHealth")
                .with_enabled(true)
                .with_impact(StartupImpact::High)
                .with_recommendation_class(AppRecommendationClass::SystemCritical),
            StartupAppInspection::new("Unknown Helper")
                .with_enabled(true)
                .with_impact(StartupImpact::High),
        ];

        let plan = build_background_work_plan(&request);
        let startup = item(&plan, BG_STARTUP_REVIEW_TWEAK_ID);

        assert_eq!(startup.action, PlanAction::Recommend);
        assert_eq!(startup.changes.len(), 1);
        assert_eq!(startup.changes[0].target, "startup:discord");
        assert_eq!(startup.changes[0].operation, TweakOperationKind::Manual);
        assert_eq!(startup.changes[0].scope, SessionScope::RecommendationOnly);
        assert_eq!(startup.backup, BackupRequirement::NotRequired);
        assert!(plan_is_recommendation_only(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn startup_review_is_detect_only_without_safe_candidates() {
        let mut request = BackgroundWorkPlanRequest::new("plan-startup-clean");
        request.startup_apps = vec![
            StartupAppInspection::new("Discord")
                .with_enabled(false)
                .with_impact(StartupImpact::High)
                .with_recommendation_class(AppRecommendationClass::KnownNonCritical),
            StartupAppInspection::new("SecurityHealth")
                .with_enabled(true)
                .with_impact(StartupImpact::High)
                .with_recommendation_class(AppRecommendationClass::SystemCritical),
        ];

        let plan = build_background_work_plan(&request);
        let startup = item(&plan, BG_STARTUP_REVIEW_TWEAK_ID);

        assert_eq!(startup.action, PlanAction::DetectOnly);
        assert!(startup.changes.is_empty());
        assert!(plan_is_recommendation_only(&plan));
    }

    #[test]
    fn background_app_review_is_prompted_and_recommendation_only() {
        let mut request = BackgroundWorkPlanRequest::new("plan-background-apps");
        request.background_apps = vec![
            BackgroundAppInspection::new("Weather", "Microsoft.BingWeather")
                .with_enabled(true)
                .with_activity(BackgroundAppActivity::High)
                .with_recommendation_class(AppRecommendationClass::KnownNonCritical),
            BackgroundAppInspection::new("Windows Security", "Microsoft.SecHealthUI")
                .with_enabled(true)
                .with_activity(BackgroundAppActivity::High)
                .with_recommendation_class(AppRecommendationClass::SystemCritical),
        ];

        let plan = build_background_work_plan(&request);
        let background = item(&plan, BG_BACKGROUND_APPS_REVIEW_TWEAK_ID);

        assert_eq!(background.action, PlanAction::Recommend);
        assert_eq!(background.changes.len(), 1);
        assert_eq!(
            background.changes[0].target,
            "background-app:microsoft.bingweather"
        );
        assert_eq!(background.rollback, RollbackPlan::not_needed());
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("Recommendation-only")));
        assert!(plan_is_recommendation_only(&plan));
    }
}
