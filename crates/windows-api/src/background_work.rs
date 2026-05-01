//! Windows scan adapter for recommendation-only startup/background app review.

use optimizer_core::{
    background_work::{
        build_background_work_plan, plan_is_recommendation_only, AppRecommendationClass,
        BackgroundAppActivity, BackgroundAppInspection, BackgroundWorkPlanRequest, StartupImpact,
        StartupAppInspection,
    },
    tweak_contracts::TweakPlan,
};

use crate::{BackgroundAppScanItem, StartupAppScanItem, SystemScanReport};

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

/// Returns true when a scan-derived T043 plan contains no automatic apply items.
#[must_use]
pub fn background_work_plan_is_recommendation_only(plan: &TweakPlan) -> bool {
    plan_is_recommendation_only(plan)
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

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::{
        background_work::{BG_BACKGROUND_APPS_REVIEW_TWEAK_ID, BG_STARTUP_REVIEW_TWEAK_ID},
        tweak_contracts::{PlanAction, SessionScope},
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
}
