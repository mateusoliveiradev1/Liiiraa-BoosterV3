//! Windows scan adapter for startup/background app review and conditional services.

use std::fmt;

use optimizer_core::{
    background_work::{
        background_services_plan_blocks_system_binary_rename,
        background_services_plan_is_not_safe_default,
        background_services_plan_requires_conditional_evidence, build_background_services_plan,
        build_background_work_plan, is_background_service_mutation_target,
        is_background_service_tweak_id, plan_is_recommendation_only, AppRecommendationClass,
        BackgroundAppActivity, BackgroundAppInspection, BackgroundServiceActivity,
        BackgroundServiceConsent, BackgroundServiceInspection, BackgroundServiceRunState,
        BackgroundServiceStartMode, BackgroundServicesPlanRequest, BackgroundWorkPlanRequest,
        SearchIndexerInspection, StartupImpact, StartupAppInspection, SysMainInspection,
        SysMainMemoryPressure, SysMainStorageProfile,
    },
    tweak_contracts::{PlanAction, TweakOperationKind, TweakPlan},
};

use crate::{
    BackgroundAppScanItem, PhysicalDiskScanItem, ServiceScanItem, StartupAppScanItem,
    SystemScanReport, WindowsRollbackFixture,
};

/// Summary for fixture apply or verify work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundServiceSettingsSummary {
    /// Count of applied or verified plan items.
    pub item_count: usize,
    /// Background service targets written or verified.
    pub targets: Vec<String>,
}

impl BackgroundServiceSettingsSummary {
    fn empty() -> Self {
        Self {
            item_count: 0,
            targets: Vec::new(),
        }
    }
}

/// Builds a T043 recommendation-only background work plan from a system scan.
#[must_use]
pub fn build_background_work_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let mut request = BackgroundWorkPlanRequest::new(plan_id);
    request.startup_apps = report.startup_apps.iter().map(startup_from_scan).collect();
    request.background_apps = report
        .background_apps
        .iter()
        .map(background_app_from_scan)
        .collect();

    build_background_work_plan(&request)
}

/// Builds a T053 conditional background service plan from a system scan.
#[must_use]
pub fn build_background_services_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> TweakPlan {
    let request = background_services_request_from_scan(plan_id, report);

    build_background_services_plan(&request)
}

/// Builds a consented T053 background service plan from scan and live activity evidence.
#[must_use]
pub fn build_consented_background_services_plan_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
    gaming_session_active: bool,
    search_indexer_load_observed: bool,
    sysmain_load_observed: bool,
    sysmain_benchmark_completed: bool,
) -> TweakPlan {
    let mut request = background_services_request_from_scan(plan_id, report);
    request.requested_mode = optimizer_core::tweak_contracts::TweakMode::Lab;
    request.search_indexer_consent = BackgroundServiceConsent::Granted;
    request.gaming_session_active = gaming_session_active;
    request.search_indexer.activity = activity_from_bool(search_indexer_load_observed);
    request.sysmain_consent = BackgroundServiceConsent::Granted;
    request.sysmain.activity = activity_from_bool(sysmain_load_observed);
    request.sysmain.benchmark_completed = sysmain_benchmark_completed;

    build_background_services_plan(&request)
}

/// Returns true when a scan-derived T043 plan contains no automatic apply items.
#[must_use]
pub fn background_work_plan_is_recommendation_only(plan: &TweakPlan) -> bool {
    plan_is_recommendation_only(plan)
}

/// Returns true when a T053 plan contains no SearchApp/system-binary rename path.
#[must_use]
pub fn background_services_plan_blocks_searchapp_rename(plan: &TweakPlan) -> bool {
    background_services_plan_blocks_system_binary_rename(plan)
}

/// Applies T053 fixture service changes after validating conditional service policy.
pub fn apply_background_service_plan_to_fixture(
    fixture: &mut WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<BackgroundServiceSettingsSummary, BackgroundServiceSettingsError> {
    validate_background_services_plan(plan)?;

    let mut summary = BackgroundServiceSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                BackgroundServiceSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            fixture.set_value(change.target.clone(), desired.to_owned());
            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

/// Verifies T053 fixture service changes after validating conditional service policy.
pub fn verify_background_service_plan_fixture(
    fixture: &WindowsRollbackFixture,
    plan: &TweakPlan,
) -> Result<BackgroundServiceSettingsSummary, BackgroundServiceSettingsError> {
    validate_background_services_plan(plan)?;

    let mut summary = BackgroundServiceSettingsSummary::empty();

    for item in plan.items.iter().filter(|item| item.action == PlanAction::Apply) {
        validate_tweak_id(&item.tweak_id)?;
        summary.item_count += 1;

        for change in &item.changes {
            validate_change(&item.tweak_id, change)?;
            let desired = change.desired_value.as_deref().ok_or_else(|| {
                BackgroundServiceSettingsError::missing_desired_value(
                    item.tweak_id.clone(),
                    change.target.clone(),
                )
            })?;

            if fixture.value(&change.target) != Some(desired) {
                return Err(BackgroundServiceSettingsError::verification_failed(
                    item.tweak_id.clone(),
                    change.target.clone(),
                ));
            }

            summary.targets.push(change.target.clone());
        }
    }

    Ok(summary)
}

fn startup_from_scan(app: &StartupAppScanItem) -> StartupAppInspection {
    let mut inspection = StartupAppInspection::new(app.name.clone())
        .with_impact(parse_startup_impact(app.startup_impact.as_deref()))
        .with_recommendation_class(classify_startup_app(app));

    if let Some(command) = &app.command {
        inspection = inspection.with_command(command.clone());
    }

    if let Some(location) = &app.location {
        inspection = inspection.with_location(location.clone());
    }

    if let Some(user) = &app.user {
        inspection = inspection.with_user(user.clone());
    }

    if let Some(enabled) = app.enabled {
        inspection = inspection.with_enabled(enabled);
    }

    inspection
}

fn background_services_request_from_scan(
    plan_id: impl Into<String>,
    report: &SystemScanReport,
) -> BackgroundServicesPlanRequest {
    let mut request = BackgroundServicesPlanRequest::new(plan_id);
    request.search_indexer = SearchIndexerInspection {
        service: service_from_scan(&report.services, "wsearch"),
        activity: BackgroundServiceActivity::Unknown,
    };
    request.sysmain = SysMainInspection {
        service: service_from_scan(&report.services, "sysmain"),
        storage_profile: sysmain_storage_profile(&report.storage.physical_disks),
        memory_pressure: sysmain_memory_pressure(report),
        activity: BackgroundServiceActivity::Unknown,
        benchmark_completed: false,
    };

    request
}

fn service_from_scan(
    services: &[ServiceScanItem],
    service_name: &str,
) -> BackgroundServiceInspection {
    services
        .iter()
        .find(|service| service.name.eq_ignore_ascii_case(service_name))
        .map(|service| {
            BackgroundServiceInspection::present(
                parse_service_run_state(service.state.as_deref()),
                parse_service_start_mode(service.start_mode.as_deref()),
            )
        })
        .unwrap_or_else(BackgroundServiceInspection::missing)
}

fn parse_service_run_state(value: Option<&str>) -> BackgroundServiceRunState {
    match normalized(value).as_deref() {
        Some("running") => BackgroundServiceRunState::Running,
        Some("stopped" | "stop" | "paused") => BackgroundServiceRunState::Stopped,
        _ => BackgroundServiceRunState::Unknown,
    }
}

fn parse_service_start_mode(value: Option<&str>) -> BackgroundServiceStartMode {
    match normalized(value).as_deref() {
        Some("auto" | "automatic" | "automaticdelayedstart" | "delayedauto") => {
            BackgroundServiceStartMode::Automatic
        }
        Some("manual" | "demand" | "demandstart") => BackgroundServiceStartMode::Manual,
        Some("disabled") => BackgroundServiceStartMode::Disabled,
        _ => BackgroundServiceStartMode::Unknown,
    }
}

fn sysmain_storage_profile(disks: &[PhysicalDiskScanItem]) -> SysMainStorageProfile {
    if disks.is_empty() {
        return SysMainStorageProfile::Unknown;
    }

    let solid_state = disks.iter().any(disk_is_solid_state);
    let rotational = disks.iter().any(disk_is_rotational);

    match (solid_state, rotational) {
        (true, true) => SysMainStorageProfile::Mixed,
        (true, false) => SysMainStorageProfile::SsdOnly,
        (false, true) => SysMainStorageProfile::HddOnly,
        (false, false) => SysMainStorageProfile::Unknown,
    }
}

fn disk_is_solid_state(disk: &PhysicalDiskScanItem) -> bool {
    disk.media_type
        .as_deref()
        .is_some_and(|media_type| normalized_text(media_type).contains("ssd"))
        || disk
            .bus_type
            .as_deref()
            .is_some_and(|bus_type| normalized_text(bus_type).contains("nvme"))
}

fn disk_is_rotational(disk: &PhysicalDiskScanItem) -> bool {
    disk.media_type.as_deref().is_some_and(|media_type| {
        let normalized = normalized_text(media_type);
        normalized.contains("hdd")
            || normalized.contains("harddisk")
            || normalized.contains("rotational")
    })
}

fn sysmain_memory_pressure(report: &SystemScanReport) -> SysMainMemoryPressure {
    match (
        report.memory.total_visible_memory_bytes,
        report.memory.free_physical_memory_bytes,
    ) {
        (Some(total), Some(free)) if total > 0 => {
            if free.saturating_mul(100) < total.saturating_mul(15) {
                SysMainMemoryPressure::High
            } else {
                SysMainMemoryPressure::Normal
            }
        }
        _ => SysMainMemoryPressure::Unknown,
    }
}

const fn activity_from_bool(observed: bool) -> BackgroundServiceActivity {
    if observed {
        BackgroundServiceActivity::Observed
    } else {
        BackgroundServiceActivity::NotObserved
    }
}

fn validate_background_services_plan(
    plan: &TweakPlan,
) -> Result<(), BackgroundServiceSettingsError> {
    if !background_services_plan_blocks_system_binary_rename(plan) {
        return Err(BackgroundServiceSettingsError::system_binary_rename_denied());
    }

    if !background_services_plan_is_not_safe_default(plan) {
        return Err(BackgroundServiceSettingsError::safe_default_denied());
    }

    if !background_services_plan_requires_conditional_evidence(plan) {
        return Err(BackgroundServiceSettingsError::conditional_evidence_missing());
    }

    Ok(())
}

fn validate_tweak_id(tweak_id: &str) -> Result<(), BackgroundServiceSettingsError> {
    if is_background_service_tweak_id(tweak_id) {
        Ok(())
    } else {
        Err(BackgroundServiceSettingsError::unsupported_tweak(tweak_id))
    }
}

fn validate_change(
    tweak_id: &str,
    change: &optimizer_core::tweak_contracts::PlannedChange,
) -> Result<(), BackgroundServiceSettingsError> {
    if change.operation != TweakOperationKind::Write {
        return Err(BackgroundServiceSettingsError::unsupported_operation(
            tweak_id,
            change.target.clone(),
        ));
    }

    if !is_background_service_mutation_target(&change.target) {
        return Err(BackgroundServiceSettingsError::unsupported_target(
            tweak_id,
            change.target.clone(),
        ));
    }

    Ok(())
}

fn background_app_from_scan(app: &BackgroundAppScanItem) -> BackgroundAppInspection {
    let name = app.display_name.as_deref().unwrap_or(&app.app_id);
    let mut inspection = BackgroundAppInspection::new(name.to_owned(), app.app_id.clone())
        .with_activity(parse_background_activity(app.activity.as_deref()))
        .with_recommendation_class(classify_background_app(app));

    if let Some(enabled) = background_enabled(app) {
        inspection = inspection.with_enabled(enabled);
    }

    inspection
}

fn background_enabled(app: &BackgroundAppScanItem) -> Option<bool> {
    app.enabled.or_else(|| {
        if app.disabled == Some(true) || app.disabled_by_user == Some(true) {
            Some(false)
        } else {
            None
        }
    })
}

fn parse_startup_impact(value: Option<&str>) -> StartupImpact {
    match normalized(value).as_deref() {
        Some("high") => StartupImpact::High,
        Some("medium") => StartupImpact::Medium,
        Some("low") => StartupImpact::Low,
        Some("notmeasured" | "not_measured" | "none") => StartupImpact::NotMeasured,
        _ => StartupImpact::Unknown,
    }
}

fn parse_background_activity(value: Option<&str>) -> BackgroundAppActivity {
    match normalized(value).as_deref() {
        Some("high") => BackgroundAppActivity::High,
        Some("moderate" | "medium") => BackgroundAppActivity::Moderate,
        Some("low") => BackgroundAppActivity::Low,
        _ => BackgroundAppActivity::Unknown,
    }
}

fn classify_startup_app(app: &StartupAppScanItem) -> AppRecommendationClass {
    let haystack = normalized_join([
        Some(app.name.as_str()),
        app.command.as_deref(),
        app.location.as_deref(),
    ]);

    classify_app_text(&haystack)
}

fn classify_background_app(app: &BackgroundAppScanItem) -> AppRecommendationClass {
    let haystack = normalized_join([
        Some(app.app_id.as_str()),
        app.display_name.as_deref(),
    ]);

    classify_app_text(&haystack)
}

fn classify_app_text(haystack: &str) -> AppRecommendationClass {
    if contains_any(
        haystack,
        &[
            "securityhealth",
            "sechealth",
            "windowsdefender",
            "defender",
            "microsoft.security",
            "driver",
            "realtek",
            "rthd",
            "amdsoftware",
            "nvidia",
        ],
    ) {
        AppRecommendationClass::SystemCritical
    } else if contains_any(
        haystack,
        &[
            "discord",
            "spotify",
            "steam",
            "epicgameslauncher",
            "epicgames",
            "slack",
            "teams",
            "adobe",
            "onedrive",
            "xbox",
            "bingweather",
            "weather",
        ],
    ) {
        AppRecommendationClass::KnownNonCritical
    } else {
        AppRecommendationClass::Unknown
    }
}

fn normalized(value: Option<&str>) -> Option<String> {
    value.map(|text| {
        text.chars()
            .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
            .flat_map(char::to_lowercase)
            .collect()
    })
}

fn normalized_text(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalized_join<const N: usize>(values: [Option<&str>; N]) -> String {
    values
        .into_iter()
        .flatten()
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

/// Stable failure reason for fixture-backed T053 service operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundServiceSettingsErrorReason {
    /// Plan item was not part of the T053 background service scope.
    UnsupportedTweak,
    /// Plan item targeted a setting outside the T053 allowlist.
    UnsupportedTarget,
    /// Plan item requested a non-write operation.
    UnsupportedOperation,
    /// A write operation did not include a desired value.
    MissingDesiredValue,
    /// Fixture readback did not match the desired state.
    VerificationFailed,
    /// Plan attempted a Safe/default service apply.
    SafeDefaultDenied,
    /// Plan apply was missing conditional consent/load/benchmark evidence.
    ConditionalEvidenceMissing,
    /// Plan attempted to rename or delete SearchApp/system binaries.
    SystemBinaryRenameDenied,
}

impl BackgroundServiceSettingsErrorReason {
    /// Returns a stable reason string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "unsupported_tweak",
            Self::UnsupportedTarget => "unsupported_target",
            Self::UnsupportedOperation => "unsupported_operation",
            Self::MissingDesiredValue => "missing_desired_value",
            Self::VerificationFailed => "verification_failed",
            Self::SafeDefaultDenied => "safe_default_denied",
            Self::ConditionalEvidenceMissing => "conditional_evidence_missing",
            Self::SystemBinaryRenameDenied => "system_binary_rename_denied",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedTweak => "Plan contains a non-background-service tweak",
            Self::UnsupportedTarget => "Plan targets a setting outside the T053 allowlist",
            Self::UnsupportedOperation => "Plan contains an unsupported operation",
            Self::MissingDesiredValue => "Plan write is missing a desired value",
            Self::VerificationFailed => "Background service fixture readback did not match",
            Self::SafeDefaultDenied => {
                "Background service tweaks must not apply from Safe/default planning"
            }
            Self::ConditionalEvidenceMissing => {
                "Background service apply requires consent, load, and benchmark evidence"
            }
            Self::SystemBinaryRenameDenied => "SearchApp/system binary rename or delete is denied",
        }
    }
}

/// Structured error for fixture-backed T053 service operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundServiceSettingsError {
    reason: BackgroundServiceSettingsErrorReason,
    tweak_id: Option<String>,
    target: Option<String>,
}

impl BackgroundServiceSettingsError {
    fn new(
        reason: BackgroundServiceSettingsErrorReason,
        tweak_id: Option<String>,
        target: Option<String>,
    ) -> Self {
        Self {
            reason,
            tweak_id,
            target,
        }
    }

    fn unsupported_tweak(tweak_id: impl Into<String>) -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::UnsupportedTweak,
            Some(tweak_id.into()),
            None,
        )
    }

    fn unsupported_target(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::UnsupportedTarget,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn unsupported_operation(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::UnsupportedOperation,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn missing_desired_value(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::MissingDesiredValue,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn verification_failed(tweak_id: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::VerificationFailed,
            Some(tweak_id.into()),
            Some(target.into()),
        )
    }

    fn safe_default_denied() -> Self {
        Self::new(BackgroundServiceSettingsErrorReason::SafeDefaultDenied, None, None)
    }

    fn conditional_evidence_missing() -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::ConditionalEvidenceMissing,
            None,
            None,
        )
    }

    fn system_binary_rename_denied() -> Self {
        Self::new(
            BackgroundServiceSettingsErrorReason::SystemBinaryRenameDenied,
            None,
            None,
        )
    }

    /// Returns the stable failure reason.
    #[must_use]
    pub const fn reason(&self) -> BackgroundServiceSettingsErrorReason {
        self.reason
    }

    /// Returns the affected tweak ID, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns the affected target, when known.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }
}

impl fmt::Display for BackgroundServiceSettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(target) = self.target() {
            write!(formatter, " [{target}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for BackgroundServiceSettingsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        backup::{capture_plan_backups, execute_rollback, RollbackRequest},
        background_work::{
            BG_BACKGROUND_APPS_REVIEW_TWEAK_ID, BG_SEARCH_SYSTEM_FILE_RENAME_TWEAK_ID,
            BG_STARTUP_REVIEW_TWEAK_ID, TARGET_SEARCH_APP_BINARY_RENAME,
            TARGET_SEARCH_INDEXER_SESSION_PAUSE, TARGET_SYSMAIN_START_MODE,
        },
        tweak_contracts::{
            BackupRequirement, PlannedChange, RebootPolicy, RollbackKind, RollbackPlan,
            RollbackStatus, SessionScope, TweakCategory, TweakMode, TweakOperationKind,
            TweakPlanItem, TweakRisk,
        },
    };

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");

    fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a optimizer_core::tweak_contracts::TweakPlanItem {
        plan.items
            .iter()
            .find(|item| item.tweak_id == tweak_id)
            .expect("plan item should exist")
    }

    #[test]
    fn scan_fixture_builds_recommendation_only_background_work_plan() {
        let report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        let plan = build_background_work_plan_from_scan("plan-t043-fixture", &report);
        let startup = item(&plan, BG_STARTUP_REVIEW_TWEAK_ID);
        let background = item(&plan, BG_BACKGROUND_APPS_REVIEW_TWEAK_ID);

        assert_eq!(startup.action, PlanAction::Recommend);
        assert_eq!(startup.changes[0].target, "startup:discord");
        assert_eq!(startup.changes[0].scope, SessionScope::RecommendationOnly);
        assert_eq!(background.action, PlanAction::Recommend);
        assert_eq!(
            background.changes[0].target,
            "background-app:microsoft.bingweather"
        );
        assert!(background_work_plan_is_recommendation_only(&plan));
        assert!(!plan.has_apply_items());
    }

    #[test]
    fn classifier_leaves_security_entries_out_of_recommendations() {
        let report = SystemScanReport {
            startup_apps: vec![StartupAppScanItem {
                name: "SecurityHealth".to_owned(),
                command: Some("C:\\Windows\\System32\\SecurityHealthSystray.exe".to_owned()),
                location: Some("HKLM\\Run".to_owned()),
                user: None,
                enabled: Some(true),
                startup_impact: Some("high".to_owned()),
            }],
            background_apps: Vec::new(),
            ..crate::parse_system_scan_report(FIXTURE).expect("fixture should parse")
        };
        let plan = build_background_work_plan_from_scan("plan-security-skip", &report);
        let startup = item(&plan, BG_STARTUP_REVIEW_TWEAK_ID);

        assert_eq!(startup.action, PlanAction::DetectOnly);
        assert!(startup.changes.is_empty());
        assert!(background_work_plan_is_recommendation_only(&plan));
    }

    fn report_with_services() -> SystemScanReport {
        let mut report = crate::parse_system_scan_report(FIXTURE).expect("fixture should parse");
        report.services.push(ServiceScanItem {
            name: "WSearch".to_owned(),
            display_name: Some("Windows Search".to_owned()),
            state: Some("Running".to_owned()),
            start_mode: Some("Automatic".to_owned()),
        });
        report.services.push(ServiceScanItem {
            name: "SysMain".to_owned(),
            display_name: Some("SysMain".to_owned()),
            state: Some("Running".to_owned()),
            start_mode: Some("Automatic".to_owned()),
        });
        report
    }

    #[test]
    fn scan_fixture_builds_conditional_service_plan_without_safe_apply() {
        let report = report_with_services();
        let plan = build_background_services_plan_from_scan("plan-t053-fixture", &report);

        assert!(!plan.has_apply_items());
        assert!(background_services_plan_blocks_searchapp_rename(&plan));
        assert!(background_services_plan_is_not_safe_default(&plan));
    }

    #[test]
    fn fixture_applies_verifies_and_rolls_back_conditional_services() {
        let report = report_with_services();
        let plan = build_consented_background_services_plan_from_scan(
            "plan-t053-consented",
            &report,
            true,
            true,
            true,
            true,
        );
        let mut fixture = WindowsRollbackFixture::new()
            .with_value(TARGET_SEARCH_INDEXER_SESSION_PAUSE, "running")
            .with_value(TARGET_SYSMAIN_START_MODE, "automatic");

        let backups =
            capture_plan_backups(&plan, &mut fixture).expect("service backups should capture");
        assert_eq!(backups.len(), 2);

        let applied = apply_background_service_plan_to_fixture(&mut fixture, &plan)
            .expect("conditional service fixture apply should succeed");

        assert_eq!(applied.item_count, 2);
        assert_eq!(
            fixture.value(TARGET_SEARCH_INDEXER_SESSION_PAUSE),
            Some("paused_for_gaming_session")
        );
        assert_eq!(fixture.value(TARGET_SYSMAIN_START_MODE), Some("manual"));

        verify_background_service_plan_fixture(&fixture, &plan)
            .expect("conditional service fixture readback should verify");

        for backup in backups {
            let tweak_id = backup.tweak_id.clone();
            let plan_item = item(&plan, &tweak_id);
            let rollback_request =
                RollbackRequest::new(tweak_id, backup, plan_item.rollback.clone())
                    .expect("rollback request should be valid");
            let rollback = execute_rollback(&mut fixture, &rollback_request)
                .expect("rollback should restore service fixture state");
            assert_eq!(rollback.status, RollbackStatus::Restored);
        }

        assert_eq!(fixture.value(TARGET_SEARCH_INDEXER_SESSION_PAUSE), Some("running"));
        assert_eq!(fixture.value(TARGET_SYSMAIN_START_MODE), Some("automatic"));
    }

    #[test]
    fn fixture_rejects_searchapp_system_binary_rename_apply() {
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
                rollback: RollbackPlan {
                    kind: RollbackKind::NotNeededReadonly,
                    steps: Vec::new(),
                    requires_admin: true,
                    reboot: RebootPolicy::None,
                    manual_instructions: Vec::new(),
                },
                reboot: RebootPolicy::None,
                requires_admin: true,
                warnings: Vec::new(),
            }],
            warnings: Vec::new(),
        };
        let mut fixture = WindowsRollbackFixture::new();

        let error = apply_background_service_plan_to_fixture(&mut fixture, &plan)
            .expect_err("SearchApp/system binary rename must be denied");

        assert_eq!(
            error.reason(),
            BackgroundServiceSettingsErrorReason::SystemBinaryRenameDenied
        );
    }
}
