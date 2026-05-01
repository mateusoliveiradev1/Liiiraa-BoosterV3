//! Recommendation-only planning for startup and background app reviews.

use crate::{
    catalog::SUPPORTED_CATALOG_SCHEMA_VERSION,
    tweak_contracts::{
        BackupRequirement, PlanAction, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
        RollbackStep, SessionScope, TweakCategory, TweakMode, TweakOperationKind, TweakPlan,
        TweakPlanItem, TweakRisk,
    },
};

/// Tweak ID for reviewing startup applications.
pub const BG_STARTUP_REVIEW_TWEAK_ID: &str = "bg.startup.review";
/// Tweak ID for reviewing background app activity.
pub const BG_BACKGROUND_APPS_REVIEW_TWEAK_ID: &str = "bg.background-apps.review";
/// Tweak ID for pausing or reducing Search indexing during a gaming session.
pub const BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID: &str =
    "bg.search.indexer.pause-session";
/// Tweak ID for denying SearchApp/system binary rename or delete requests.
pub const BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID: &str = "bg.search.system-file-rename";
/// Blocked guardrail ID used by the V1 matrix for Windows binary rename/delete attempts.
pub const BLOCKED_SYSTEM_FILE_RENAME_GUARDRAIL_ID: &str = "blocked.system-file-rename";
/// Tweak ID for conditional SysMain planning.
pub const BG_SYSMAIN_CONDITIONAL_TWEAK_ID: &str = "bg.sysmain.conditional";

const STARTUP_TARGET_PREFIX: &str = "startup:";
const BACKGROUND_APP_TARGET_PREFIX: &str = "background-app:";
/// Logical target for session-scoped Search indexing pause/reduction.
pub const TARGET_SEARCH_INDEXER_SESSION_PAUSE: &str = "service:wsearch/session-pause";
/// Logical target for Lab-only SysMain startup-mode changes.
pub const TARGET_SYSMAIN_START_MODE: &str = "service:sysmain/start-mode";
/// Denial target for SearchApp/system binary rename or delete attempts.
pub const TARGET_SEARCH_APP_BINARY_RENAME: &str =
    "blocked:system-binary/searchapp.exe/rename-or-delete";

const SEARCH_INDEXER_DESIRED_STATE: &str = "paused_for_gaming_session";
const SYSMAIN_DESIRED_START_MODE: &str = "manual";

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

/// Current service run state from the Windows service inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundServiceRunState {
    /// Service is currently running.
    Running,
    /// Service is present but stopped.
    Stopped,
    /// Current service state could not be proven.
    Unknown,
}

impl BackgroundServiceRunState {
    const fn as_state(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Unknown => "unknown",
        }
    }
}

/// Current service startup mode from the Windows service inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundServiceStartMode {
    /// Service starts automatically.
    Automatic,
    /// Service starts on demand.
    Manual,
    /// Service is disabled.
    Disabled,
    /// Startup mode could not be proven.
    Unknown,
}

impl BackgroundServiceStartMode {
    const fn as_state(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
            Self::Disabled => "disabled",
            Self::Unknown => "unknown",
        }
    }

    const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    const fn is_known(self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Minimal service posture needed by the background service planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackgroundServiceInspection {
    /// Whether the service exists in the scan.
    pub present: bool,
    /// Current run state.
    pub run_state: BackgroundServiceRunState,
    /// Current startup mode.
    pub start_mode: BackgroundServiceStartMode,
}

impl BackgroundServiceInspection {
    /// Creates an inspection for a present service.
    #[must_use]
    pub const fn present(
        run_state: BackgroundServiceRunState,
        start_mode: BackgroundServiceStartMode,
    ) -> Self {
        Self {
            present: true,
            run_state,
            start_mode,
        }
    }

    /// Creates an inspection for a service missing from inventory.
    #[must_use]
    pub const fn missing() -> Self {
        Self {
            present: false,
            run_state: BackgroundServiceRunState::Unknown,
            start_mode: BackgroundServiceStartMode::Unknown,
        }
    }

    fn can_pause_session(self) -> bool {
        self.present
            && self.run_state == BackgroundServiceRunState::Running
            && !self.start_mode.is_disabled()
    }

    fn can_change_start_mode(self) -> bool {
        self.present && self.start_mode.is_known() && !self.start_mode.is_disabled()
    }

    fn previous_value(self) -> String {
        format!(
            "present={},state={},start_mode={}",
            self.present,
            self.run_state.as_state(),
            self.start_mode.as_state()
        )
    }
}

/// Observed service activity signal used to keep background-service changes conditional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundServiceActivity {
    /// Meaningful service activity/load was observed while gaming or benchmarking.
    Observed,
    /// The service was inspected and no meaningful load was observed.
    NotObserved,
    /// The scan did not include a load/activity signal.
    Unknown,
}

impl BackgroundServiceActivity {
    const fn is_observed(self) -> bool {
        matches!(self, Self::Observed)
    }

    const fn as_state(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::NotObserved => "not_observed",
            Self::Unknown => "unknown",
        }
    }
}

/// Explicit user consent for conditional background-service controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundServiceConsent {
    /// The user has not accepted the conditional service control.
    NotGranted,
    /// The user explicitly accepted the conditional service control.
    Granted,
}

impl BackgroundServiceConsent {
    const fn is_granted(self) -> bool {
        matches!(self, Self::Granted)
    }
}

/// Search indexing service inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchIndexerInspection {
    /// WSearch service posture.
    pub service: BackgroundServiceInspection,
    /// Whether Search indexing load was observed during gameplay.
    pub activity: BackgroundServiceActivity,
}

impl SearchIndexerInspection {
    /// Creates an unknown Search indexing inspection.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            service: BackgroundServiceInspection::missing(),
            activity: BackgroundServiceActivity::Unknown,
        }
    }
}

/// Storage profile used by the SysMain conditional planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysMainStorageProfile {
    /// Only rotational/HDD media was detected.
    HddOnly,
    /// Both solid-state and rotational media were detected.
    Mixed,
    /// Only solid-state media was detected.
    SsdOnly,
    /// Storage media profile could not be proven.
    Unknown,
}

impl SysMainStorageProfile {
    const fn supports_lab_reduction(self) -> bool {
        matches!(self, Self::Mixed | Self::SsdOnly)
    }

    const fn as_state(self) -> &'static str {
        match self {
            Self::HddOnly => "hdd_only",
            Self::Mixed => "mixed",
            Self::SsdOnly => "ssd_only",
            Self::Unknown => "unknown",
        }
    }
}

/// RAM pressure signal used by the SysMain conditional planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysMainMemoryPressure {
    /// RAM pressure is high enough that changing SysMain is risky.
    High,
    /// RAM pressure is not currently high.
    Normal,
    /// RAM pressure could not be proven.
    Unknown,
}

impl SysMainMemoryPressure {
    const fn supports_lab_reduction(self) -> bool {
        matches!(self, Self::Normal)
    }

    const fn as_state(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Normal => "normal",
            Self::Unknown => "unknown",
        }
    }
}

/// SysMain service and prerequisite inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysMainInspection {
    /// SysMain service posture.
    pub service: BackgroundServiceInspection,
    /// HDD/SSD profile.
    pub storage_profile: SysMainStorageProfile,
    /// RAM pressure profile.
    pub memory_pressure: SysMainMemoryPressure,
    /// Whether SysMain load was observed.
    pub activity: BackgroundServiceActivity,
    /// Whether a baseline benchmark exists for this Lab change.
    pub benchmark_completed: bool,
}

impl SysMainInspection {
    /// Creates an unknown SysMain inspection.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            service: BackgroundServiceInspection::missing(),
            storage_profile: SysMainStorageProfile::Unknown,
            memory_pressure: SysMainMemoryPressure::Unknown,
            activity: BackgroundServiceActivity::Unknown,
            benchmark_completed: false,
        }
    }

    fn supports_lab_reduction(self) -> bool {
        self.service.can_change_start_mode()
            && self.storage_profile.supports_lab_reduction()
            && self.memory_pressure.supports_lab_reduction()
            && self.activity.is_observed()
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

/// Request for T053 Search indexing and SysMain conditional service planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundServicesPlanRequest {
    /// Stable plan ID supplied by the caller.
    pub plan_id: String,
    /// Highest mode requested by the user.
    pub requested_mode: TweakMode,
    /// Search indexing inspection.
    pub search_indexer: SearchIndexerInspection,
    /// Consent for session-scoped Search indexing pause/reduction.
    pub search_indexer_consent: BackgroundServiceConsent,
    /// Whether a gaming session is currently active.
    pub gaming_session_active: bool,
    /// Whether a SearchApp or system binary rename/delete path was requested.
    pub search_system_file_rename_requested: bool,
    /// SysMain inspection and prerequisites.
    pub sysmain: SysMainInspection,
    /// Consent for SysMain Lab planning.
    pub sysmain_consent: BackgroundServiceConsent,
}

impl BackgroundServicesPlanRequest {
    /// Creates a conservative T053 request.
    #[must_use]
    pub fn new(plan_id: impl Into<String>) -> Self {
        Self {
            plan_id: plan_id.into(),
            requested_mode: TweakMode::Safe,
            search_indexer: SearchIndexerInspection::unknown(),
            search_indexer_consent: BackgroundServiceConsent::NotGranted,
            gaming_session_active: false,
            search_system_file_rename_requested: false,
            sysmain: SysMainInspection::unknown(),
            sysmain_consent: BackgroundServiceConsent::NotGranted,
        }
    }
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

/// Builds a dry-run plan for T053 conditional Search indexing and SysMain controls.
#[must_use]
pub fn build_background_services_plan(request: &BackgroundServicesPlanRequest) -> TweakPlan {
    let items = vec![
        search_indexer_pause_item(request),
        search_system_file_rename_guardrail_item(request),
        sysmain_conditional_item(request),
    ];
    let warnings = items
        .iter()
        .flat_map(|item| item.warnings.iter())
        .filter(|warning| {
            warning.contains("Search")
                || warning.contains("SysMain")
                || warning.contains("service")
                || warning.contains("binary")
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

/// Returns true when the ID belongs to the T043 background work scope.
#[must_use]
pub fn is_background_work_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        BG_STARTUP_REVIEW_TWEAK_ID | BG_BACKGROUND_APPS_REVIEW_TWEAK_ID
    )
}

/// Returns true when the ID belongs to T053 background service planning.
#[must_use]
pub fn is_background_service_tweak_id(tweak_id: &str) -> bool {
    matches!(
        tweak_id,
        BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID
            | BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID
            | BLOCKED_SYSTEM_FILE_RENAME_GUARDRAIL_ID
            | BG_SYSMAIN_CONDITIONAL_TWEAK_ID
    )
}

/// Returns true when a target is an allowlisted T053 mutable service target.
#[must_use]
pub fn is_background_service_mutation_target(target: &str) -> bool {
    matches!(
        target,
        TARGET_SEARCH_INDEXER_SESSION_PAUSE | TARGET_SYSMAIN_START_MODE
    )
}

/// Returns true when a plan has no path to rename or delete SearchApp/system binaries.
#[must_use]
pub fn background_services_plan_blocks_system_binary_rename(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        let is_rename_guardrail = matches!(
            item.tweak_id.as_str(),
            BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID | BLOCKED_SYSTEM_FILE_RENAME_GUARDRAIL_ID
        );
        let applies_system_binary_rename = item.action == PlanAction::Apply
            && item.changes.iter().any(|change| {
                is_system_binary_rename_target(&change.target)
                    || change
                        .desired_value
                        .as_deref()
                        .is_some_and(is_system_binary_rename_value)
            });

        !applies_system_binary_rename && (!is_rename_guardrail || item.action != PlanAction::Apply)
    })
}

/// Returns true when no T053 apply item can run from Safe/default planning.
#[must_use]
pub fn background_services_plan_is_not_safe_default(plan: &TweakPlan) -> bool {
    plan.requested_mode != TweakMode::Safe
        || plan.items.iter().all(|item| item.action != PlanAction::Apply)
}

/// Returns true when T053 apply items carry the required mode and condition warnings.
#[must_use]
pub fn background_services_plan_requires_conditional_evidence(plan: &TweakPlan) -> bool {
    plan.items.iter().all(|item| {
        if item.action != PlanAction::Apply {
            return true;
        }

        match item.tweak_id.as_str() {
            BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID => {
                item.mode == TweakMode::Competitive
                    && item.risk == TweakRisk::Medium
                    && item
                        .warnings
                        .iter()
                        .any(|warning| warning.contains("explicit consent"))
                    && item
                        .warnings
                        .iter()
                        .any(|warning| warning.contains("Active Search indexing load"))
                    && item.changes.iter().all(|change| {
                        change.scope == SessionScope::SessionOnly
                            && change.target == TARGET_SEARCH_INDEXER_SESSION_PAUSE
                    })
            }
            BG_SYSMAIN_CONDITIONAL_TWEAK_ID => {
                item.mode == TweakMode::Lab
                    && item
                        .warnings
                        .iter()
                        .any(|warning| warning.contains("HDD/SSD, RAM, load, and benchmark"))
                    && item.changes.iter().all(|change| {
                        change.scope == SessionScope::Persistent
                            && change.target == TARGET_SYSMAIN_START_MODE
                    })
            }
            _ => false,
        }
    })
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

fn search_indexer_pause_item(request: &BackgroundServicesPlanRequest) -> TweakPlanItem {
    let changes = if request.search_indexer.service.can_pause_session()
        && request.search_indexer.activity.is_observed()
    {
        vec![PlannedChange {
            target: TARGET_SEARCH_INDEXER_SESSION_PAUSE.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: Some(request.search_indexer.service.previous_value()),
            desired_value: Some(SEARCH_INDEXER_DESIRED_STATE.to_owned()),
            scope: SessionScope::SessionOnly,
        }]
    } else {
        Vec::new()
    };
    let action = search_indexer_action(request, changes.is_empty());
    let warnings = search_indexer_warnings(request);

    background_service_plan_item(
        BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID,
        action,
        TweakMode::Competitive,
        TweakRisk::Medium,
        changes,
        true,
        warnings,
        RollbackKind::ExactValue,
    )
}

fn search_system_file_rename_guardrail_item(
    request: &BackgroundServicesPlanRequest,
) -> TweakPlanItem {
    let mut warnings = Vec::new();
    if request.search_system_file_rename_requested {
        warnings.push(
            "SearchApp.exe and other Windows system binary rename/delete requests are denied."
                .to_owned(),
        );
    }

    let changes = if request.search_system_file_rename_requested {
        vec![PlannedChange {
            target: TARGET_SEARCH_APP_BINARY_RENAME.to_owned(),
            operation: TweakOperationKind::Deny,
            previous_value: None,
            desired_value: Some("rename_or_delete".to_owned()),
            scope: SessionScope::Blocked,
        }]
    } else {
        Vec::new()
    };

    background_service_plan_item(
        BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID,
        if request.search_system_file_rename_requested {
            PlanAction::Deny
        } else {
            PlanAction::DetectOnly
        },
        TweakMode::Blocked,
        TweakRisk::Critical,
        changes,
        false,
        warnings,
        RollbackKind::NotNeededReadonly,
    )
}

fn sysmain_conditional_item(request: &BackgroundServicesPlanRequest) -> TweakPlanItem {
    let changes = if request.sysmain.supports_lab_reduction() {
        vec![PlannedChange {
            target: TARGET_SYSMAIN_START_MODE.to_owned(),
            operation: TweakOperationKind::Write,
            previous_value: Some(request.sysmain.service.previous_value()),
            desired_value: Some(SYSMAIN_DESIRED_START_MODE.to_owned()),
            scope: SessionScope::Persistent,
        }]
    } else {
        Vec::new()
    };
    let action = sysmain_action(request, changes.is_empty());
    let warnings = sysmain_warnings(request);

    background_service_plan_item(
        BG_SYSMAIN_CONDITIONAL_TWEAK_ID,
        action,
        TweakMode::Lab,
        TweakRisk::Medium,
        changes,
        true,
        warnings,
        RollbackKind::ExactValue,
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

fn search_indexer_action(
    request: &BackgroundServicesPlanRequest,
    no_changes: bool,
) -> PlanAction {
    if no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode == TweakMode::Safe
        || !request.search_indexer_consent.is_granted()
        || !request.gaming_session_active
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn sysmain_action(request: &BackgroundServicesPlanRequest, no_changes: bool) -> PlanAction {
    if no_changes {
        return PlanAction::DetectOnly;
    }

    if request.requested_mode != TweakMode::Lab
        || !request.sysmain_consent.is_granted()
        || !request.sysmain.benchmark_completed
    {
        PlanAction::Recommend
    } else {
        PlanAction::Apply
    }
}

fn search_indexer_warnings(request: &BackgroundServicesPlanRequest) -> Vec<String> {
    let mut warnings = vec![
        "Search indexing pause is Competitive, session-scoped, and requires explicit consent."
            .to_owned(),
        "Only WSearch service session state is planned; SearchApp.exe and system binaries are never renamed or deleted."
            .to_owned(),
    ];

    if request.requested_mode == TweakMode::Safe {
        warnings.push("Search indexing pause stays off in Safe mode.".to_owned());
    }

    if !request.gaming_session_active {
        warnings.push("A gaming session must be active before pausing Search indexing.".to_owned());
    }

    if !request.search_indexer_consent.is_granted() {
        warnings.push("Search indexing pause consent has not been granted.".to_owned());
    }

    match request.search_indexer.activity {
        BackgroundServiceActivity::Observed => {
            warnings.push("Active Search indexing load was observed.".to_owned());
        }
        BackgroundServiceActivity::NotObserved => {
            warnings.push("No active Search indexing load was observed.".to_owned());
        }
        BackgroundServiceActivity::Unknown => {
            warnings.push(
                "Search indexing activity is unknown; observe load before planning a pause."
                    .to_owned(),
            );
        }
    }

    if !request.search_indexer.service.present {
        warnings.push("WSearch service was not present in the scan.".to_owned());
    } else if request.search_indexer.service.start_mode.is_disabled() {
        warnings.push("WSearch service is disabled; do not preserve service-disable tweaks.".to_owned());
    }

    warnings
}

fn sysmain_warnings(request: &BackgroundServicesPlanRequest) -> Vec<String> {
    let mut warnings = vec![
        "SysMain is Lab-only and requires HDD/SSD, RAM, load, and benchmark evidence before apply."
            .to_owned(),
        "SysMain changes are service-startup changes with backup and rollback; broad service-disable packs are not allowed."
            .to_owned(),
    ];

    if request.requested_mode != TweakMode::Lab {
        warnings.push("SysMain conditional planning stays off outside Lab mode.".to_owned());
    }

    if !request.sysmain_consent.is_granted() {
        warnings.push("SysMain Lab consent has not been granted.".to_owned());
    }

    if !request.sysmain.benchmark_completed {
        warnings.push("Complete a before/after benchmark before applying SysMain changes.".to_owned());
    }

    if !request.sysmain.service.present {
        warnings.push("SysMain service was not present in the scan.".to_owned());
    } else if request.sysmain.service.start_mode.is_disabled() {
        warnings.push("SysMain is already disabled; Liiiraa will not preserve broad service-disable tweaks.".to_owned());
    }

    if !request.sysmain.storage_profile.supports_lab_reduction() {
        warnings.push(format!(
            "SysMain storage profile is {}; require SSD/mixed-media evidence before reduction.",
            request.sysmain.storage_profile.as_state()
        ));
    }

    if !request.sysmain.memory_pressure.supports_lab_reduction() {
        warnings.push(format!(
            "SysMain RAM pressure is {}; avoid reducing SysMain without stable memory headroom.",
            request.sysmain.memory_pressure.as_state()
        ));
    }

    if !request.sysmain.activity.is_observed() {
        warnings.push(format!(
            "SysMain load is {}; observe service load before planning a Lab change.",
            request.sysmain.activity.as_state()
        ));
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

fn background_service_plan_item(
    tweak_id: &str,
    action: PlanAction,
    mode: TweakMode,
    risk: TweakRisk,
    changes: Vec<PlannedChange>,
    requires_admin: bool,
    warnings: Vec<String>,
    rollback_kind: RollbackKind,
) -> TweakPlanItem {
    let backup = background_service_backup_requirement(action, rollback_kind, &changes);
    let rollback = background_service_rollback_plan(action, rollback_kind, &changes, requires_admin);

    TweakPlanItem {
        tweak_id: tweak_id.to_owned(),
        category: if mode == TweakMode::Blocked {
            TweakCategory::BlockedGuardrail
        } else {
            TweakCategory::BackgroundWork
        },
        action,
        mode,
        risk,
        changes,
        backup,
        rollback,
        reboot: RebootPolicy::None,
        requires_admin,
        warnings,
    }
}

fn background_service_backup_requirement(
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

fn background_service_rollback_plan(
    action: PlanAction,
    rollback_kind: RollbackKind,
    changes: &[PlannedChange],
    requires_admin: bool,
) -> RollbackPlan {
    if action != PlanAction::Apply || changes.is_empty() {
        return RollbackPlan::not_needed();
    }

    RollbackPlan {
        kind: rollback_kind,
        steps: changes
            .iter()
            .map(|change| RollbackStep {
                summary: "Restore previous background service state.".to_owned(),
                target: change.target.clone(),
                operation: TweakOperationKind::Write,
                expected_state: change.previous_value.clone(),
            })
            .collect(),
        requires_admin,
        reboot: RebootPolicy::None,
        manual_instructions: Vec::new(),
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

fn is_system_binary_rename_target(target: &str) -> bool {
    let normalized = target.to_ascii_lowercase();
    normalized == TARGET_SEARCH_APP_BINARY_RENAME
        || (normalized.contains("searchapp") && normalized.contains("rename"))
        || (normalized.contains("searchapp") && normalized.contains("delete"))
        || (normalized.contains("system-binary") && normalized.contains("rename"))
        || (normalized.contains("system-binary") && normalized.contains("delete"))
}

fn is_system_binary_rename_value(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("searchapp")
        || normalized.contains("rename")
        || normalized.contains("delete")
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

    fn service() -> BackgroundServiceInspection {
        BackgroundServiceInspection::present(
            BackgroundServiceRunState::Running,
            BackgroundServiceStartMode::Automatic,
        )
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

    #[test]
    fn search_indexing_pause_requires_competitive_consent_and_observed_load() {
        let mut request = BackgroundServicesPlanRequest::new("plan-search-indexing");
        request.search_indexer = SearchIndexerInspection {
            service: service(),
            activity: BackgroundServiceActivity::Observed,
        };
        request.search_indexer_consent = BackgroundServiceConsent::Granted;
        request.gaming_session_active = true;

        let safe_plan = build_background_services_plan(&request);
        let safe_search = item(&safe_plan, BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID);

        assert_eq!(safe_search.action, PlanAction::Recommend);
        assert_eq!(safe_search.backup, BackupRequirement::NotRequired);
        assert!(!safe_plan.has_apply_items());
        assert!(background_services_plan_is_not_safe_default(&safe_plan));

        request.requested_mode = TweakMode::Competitive;
        let apply_plan = build_background_services_plan(&request);
        let search = item(&apply_plan, BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID);

        assert_eq!(search.action, PlanAction::Apply);
        assert_eq!(search.mode, TweakMode::Competitive);
        assert_eq!(search.changes[0].target, TARGET_SEARCH_INDEXER_SESSION_PAUSE);
        assert_eq!(search.changes[0].scope, SessionScope::SessionOnly);
        assert_eq!(
            search.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_SEARCH_INDEXER_SESSION_PAUSE.to_owned(),
            }
        );
        assert!(background_services_plan_requires_conditional_evidence(
            &apply_plan
        ));
    }

    #[test]
    fn search_app_system_binary_rename_is_blocked() {
        let mut request = BackgroundServicesPlanRequest::new("plan-searchapp-deny");
        request.search_system_file_rename_requested = true;

        let plan = build_background_services_plan(&request);
        let guardrail = item(&plan, BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID);

        assert_eq!(guardrail.action, PlanAction::Deny);
        assert_eq!(guardrail.mode, TweakMode::Blocked);
        assert_eq!(guardrail.risk, TweakRisk::Critical);
        assert_eq!(guardrail.category, TweakCategory::BlockedGuardrail);
        assert_eq!(guardrail.changes[0].target, TARGET_SEARCH_APP_BINARY_RENAME);
        assert_eq!(guardrail.changes[0].operation, TweakOperationKind::Deny);
        assert!(background_services_plan_blocks_system_binary_rename(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn sysmain_lab_change_requires_analysis_consent_and_benchmark() {
        let mut request = BackgroundServicesPlanRequest::new("plan-sysmain");
        request.requested_mode = TweakMode::Lab;
        request.sysmain = SysMainInspection {
            service: service(),
            storage_profile: SysMainStorageProfile::SsdOnly,
            memory_pressure: SysMainMemoryPressure::Normal,
            activity: BackgroundServiceActivity::Observed,
            benchmark_completed: false,
        };
        request.sysmain_consent = BackgroundServiceConsent::Granted;

        let recommend_plan = build_background_services_plan(&request);
        let sysmain = item(&recommend_plan, BG_SYSMAIN_CONDITIONAL_TWEAK_ID);

        assert_eq!(sysmain.action, PlanAction::Recommend);
        assert_eq!(sysmain.mode, TweakMode::Lab);
        assert_eq!(sysmain.backup, BackupRequirement::NotRequired);

        request.sysmain.benchmark_completed = true;
        let apply_plan = build_background_services_plan(&request);
        let sysmain = item(&apply_plan, BG_SYSMAIN_CONDITIONAL_TWEAK_ID);

        assert_eq!(sysmain.action, PlanAction::Apply);
        assert_eq!(sysmain.changes[0].target, TARGET_SYSMAIN_START_MODE);
        assert_eq!(sysmain.changes[0].desired_value.as_deref(), Some("manual"));
        assert_eq!(
            sysmain.backup,
            BackupRequirement::Required {
                kind: RollbackKind::ExactValue,
                target: TARGET_SYSMAIN_START_MODE.to_owned(),
            }
        );
        assert!(background_services_plan_requires_conditional_evidence(
            &apply_plan
        ));
    }

    #[test]
    fn malicious_apply_path_cannot_rename_searchapp() {
        let plan = TweakPlan {
            id: "plan-malicious-searchapp".to_owned(),
            requested_mode: TweakMode::Competitive,
            catalog_schema_version: "1".to_owned(),
            items: vec![TweakPlanItem {
                tweak_id: BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID.to_owned(),
                category: TweakCategory::BlockedGuardrail,
                action: PlanAction::Apply,
                mode: TweakMode::Blocked,
                risk: TweakRisk::Critical,
                changes: vec![PlannedChange {
                    target: TARGET_SEARCH_APP_BINARY_RENAME.to_owned(),
                    operation: TweakOperationKind::Write,
                    previous_value: None,
                    desired_value: Some("rename SearchApp.exe".to_owned()),
                    scope: SessionScope::Blocked,
                }],
                backup: BackupRequirement::NotRequired,
                rollback: RollbackPlan::not_needed(),
                reboot: RebootPolicy::None,
                requires_admin: true,
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        };

        assert!(!background_services_plan_blocks_system_binary_rename(&plan));
    }
}
