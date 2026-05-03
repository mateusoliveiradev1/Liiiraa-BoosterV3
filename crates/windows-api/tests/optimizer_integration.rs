use std::collections::BTreeMap;

use optimizer_core::{
    backup::{capture_plan_backups, execute_rollback, RollbackRequest},
    background_work::{
        BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID, BG_SYSMAIN_CONDITIONAL_TWEAK_ID,
        TARGET_SEARCH_INDEXER_SESSION_PAUSE, TARGET_SYSMAIN_START_MODE,
    },
    power_plan::{
        DevicePowerClass, LiiiraaPowerPlanProfile, PowerPlanConsent, PowerSourceState,
    },
    storage::{
        TARGET_STORAGE_SENSE_CADENCE, TARGET_STORAGE_SENSE_ENABLED,
        TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS, STORAGE_SENSE_CONFIGURE_TWEAK_ID,
    },
    tweak_contracts::{PlanAction, RollbackStatus, TweakPlan, TweakPlanItem},
};
use windows_api::{
    active_power_scheme_matches, apply_background_service_plan_to_fixture,
    apply_storage_sense_plan_to_fixture, build_consented_background_services_plan_from_scan,
    build_consented_storage_sense_plan_from_scan, build_liiiraa_powercfg_plan,
    parse_system_scan_report, verify_background_service_plan_fixture,
    verify_storage_sense_plan_fixture, FixedWindowsExecutable, ServiceScanItem,
    StructuredCommandPlan, SystemScanReport, WindowsArgument, WindowsRollbackFixture,
    LIIIRAA_PERFORMANCE_SCHEME_GUID, WINDOWS_BALANCED_SCHEME_GUID,
    WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID,
};

const FIXTURE: &str = include_str!("fixtures/system_scan.json");
#[cfg(windows)]
const LIVE_WINDOWS_INTEGRATION_ENV: &str = "LIIIRAA_WINDOWS_API_LIVE";

fn item<'a>(plan: &'a TweakPlan, tweak_id: &str) -> &'a TweakPlanItem {
    plan.items
        .iter()
        .find(|item| item.tweak_id == tweak_id)
        .expect("plan item should exist")
}

#[test]
fn registry_fixture_covers_backup_apply_verify_and_rollback() {
    let report = parse_system_scan_report(FIXTURE).expect("fixture should parse");
    let plan = build_consented_storage_sense_plan_from_scan("it-registry-storage-sense", &report);
    let storage_sense = item(&plan, STORAGE_SENSE_CONFIGURE_TWEAK_ID);
    let mut fixture = WindowsRollbackFixture::new()
        .with_value(TARGET_STORAGE_SENSE_ENABLED, "0")
        .with_value(TARGET_STORAGE_SENSE_CADENCE, "7")
        .with_value(TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS, "14");

    assert_eq!(storage_sense.action, PlanAction::Apply);

    let backups = capture_plan_backups(&plan, &mut fixture)
        .expect("registry backups should be captured before apply");
    assert_eq!(backups.len(), 1);

    let applied = apply_storage_sense_plan_to_fixture(&mut fixture, &plan)
        .expect("registry fixture apply should succeed");
    assert_eq!(applied.item_count, 1);
    assert_eq!(fixture.value(TARGET_STORAGE_SENSE_ENABLED), Some("1"));
    assert_eq!(fixture.value(TARGET_STORAGE_SENSE_CADENCE), Some("30"));

    verify_storage_sense_plan_fixture(&fixture, &plan)
        .expect("registry fixture readback should verify");

    let rollback_request = RollbackRequest::new(
        STORAGE_SENSE_CONFIGURE_TWEAK_ID,
        backups[0].clone(),
        storage_sense.rollback.clone(),
    )
    .expect("registry rollback request should be valid");
    let rollback = execute_rollback(&mut fixture, &rollback_request)
        .expect("registry rollback should restore fixture state");

    assert_eq!(rollback.status, RollbackStatus::Restored);
    assert_eq!(fixture.value(TARGET_STORAGE_SENSE_ENABLED), Some("0"));
    assert_eq!(fixture.value(TARGET_STORAGE_SENSE_CADENCE), Some("7"));
    assert_eq!(
        fixture.value(TARGET_STORAGE_SENSE_RECYCLE_BIN_DAYS),
        Some("14")
    );
}

#[derive(Debug)]
struct PowerCfgMock {
    active_scheme_guid: String,
    schemes: BTreeMap<String, String>,
    settings: BTreeMap<String, u32>,
}

impl PowerCfgMock {
    fn new(active_scheme_guid: &str) -> Self {
        let mut schemes = BTreeMap::new();
        schemes.insert(WINDOWS_BALANCED_SCHEME_GUID.to_owned(), "Balanced".to_owned());
        schemes.insert(
            WINDOWS_HIGH_PERFORMANCE_SCHEME_GUID.to_owned(),
            "High performance".to_owned(),
        );

        Self {
            active_scheme_guid: active_scheme_guid.to_owned(),
            schemes,
            settings: BTreeMap::new(),
        }
    }

    fn run(&mut self, command: &StructuredCommandPlan) -> String {
        assert_eq!(command.executable(), FixedWindowsExecutable::PowerCfg);
        let args = command
            .arguments()
            .iter()
            .map(WindowsArgument::as_str)
            .collect::<Vec<_>>();

        match args.as_slice() {
            ["/duplicatescheme", source, destination] => {
                let source_name = self
                    .schemes
                    .get(*source)
                    .expect("source scheme must exist")
                    .clone();
                self.schemes.insert((*destination).to_owned(), source_name);
                String::new()
            }
            ["/changename", scheme, name] => {
                self.schemes.insert((*scheme).to_owned(), (*name).to_owned());
                String::new()
            }
            ["/setacvalueindex", scheme, subgroup, setting, value]
            | ["/setdcvalueindex", scheme, subgroup, setting, value] => {
                self.settings.insert(
                    format!("{}:{}:{}:{}", args[0], scheme, subgroup, setting),
                    value.parse::<u32>().expect("setting value should be numeric"),
                );
                String::new()
            }
            ["/setactive", scheme] => {
                assert!(
                    self.schemes.contains_key(*scheme),
                    "active scheme must exist before activation"
                );
                self.active_scheme_guid = (*scheme).to_owned();
                String::new()
            }
            ["/getactivescheme"] => {
                let name = self
                    .schemes
                    .get(&self.active_scheme_guid)
                    .expect("active scheme should have a display name");
                format!("Power Scheme GUID: {} ({name})", self.active_scheme_guid)
            }
            ["/delete", scheme] => {
                self.schemes.remove(*scheme);
                String::new()
            }
            other => panic!("unsupported mock powercfg command: {other:?}"),
        }
    }
}

#[test]
fn power_plan_mock_covers_command_apply_verify_and_rollback() {
    let request = windows_api::LiiiraaPowerPlanApplyRequest::new(
        LiiiraaPowerPlanProfile::Performance,
        DevicePowerClass::Desktop,
        PowerSourceState::Ac,
        PowerPlanConsent::NotGranted,
        WINDOWS_BALANCED_SCHEME_GUID,
    );
    let plan = build_liiiraa_powercfg_plan(&request)
        .expect("desktop performance power plan should build");
    let mut mock = PowerCfgMock::new(WINDOWS_BALANCED_SCHEME_GUID);

    for command in &plan.commands {
        mock.run(command);
    }

    let readback = mock.run(&plan.verify_active_scheme);
    assert!(active_power_scheme_matches(&readback, &plan.scheme_guid));
    assert_eq!(plan.scheme_guid, LIIIRAA_PERFORMANCE_SCHEME_GUID);
    assert!(mock.settings.keys().any(|key| {
        key == &format!(
            "/setacvalueindex:{}:SUB_USB:USBSELECTIVE",
            LIIIRAA_PERFORMANCE_SCHEME_GUID
        )
    }));

    for command in &plan.rollback.commands {
        mock.run(command);
    }

    assert_eq!(mock.active_scheme_guid, WINDOWS_BALANCED_SCHEME_GUID);
    assert!(!mock.schemes.contains_key(LIIIRAA_PERFORMANCE_SCHEME_GUID));
}

fn report_with_service_inventory() -> SystemScanReport {
    let mut report = parse_system_scan_report(FIXTURE).expect("fixture should parse");
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
fn services_fixture_covers_conditional_apply_verify_and_rollback() {
    let report = report_with_service_inventory();
    let plan = build_consented_background_services_plan_from_scan(
        "it-services-conditional",
        &report,
        true,
        true,
        true,
        true,
    );
    let search = item(&plan, BG_SEARCH_INDEXER_PAUSE_SESSION_TWEAK_ID);
    let sysmain = item(&plan, BG_SYSMAIN_CONDITIONAL_TWEAK_ID);
    let mut fixture = WindowsRollbackFixture::new()
        .with_value(TARGET_SEARCH_INDEXER_SESSION_PAUSE, "running")
        .with_value(TARGET_SYSMAIN_START_MODE, "automatic");

    assert_eq!(search.action, PlanAction::Apply);
    assert_eq!(sysmain.action, PlanAction::Apply);

    let backups = capture_plan_backups(&plan, &mut fixture)
        .expect("service backups should be captured before apply");
    assert_eq!(backups.len(), 2);

    let applied = apply_background_service_plan_to_fixture(&mut fixture, &plan)
        .expect("service fixture apply should succeed");
    assert_eq!(applied.item_count, 2);
    assert_eq!(
        fixture.value(TARGET_SEARCH_INDEXER_SESSION_PAUSE),
        Some("paused_for_gaming_session")
    );
    assert_eq!(fixture.value(TARGET_SYSMAIN_START_MODE), Some("manual"));

    verify_background_service_plan_fixture(&fixture, &plan)
        .expect("service fixture readback should verify");

    for backup in backups {
        let plan_item = item(&plan, &backup.tweak_id);
        let rollback_request = RollbackRequest::new(
            backup.tweak_id.clone(),
            backup,
            plan_item.rollback.clone(),
        )
        .expect("service rollback request should be valid");
        let rollback = execute_rollback(&mut fixture, &rollback_request)
            .expect("service rollback should restore fixture state");
        assert_eq!(rollback.status, RollbackStatus::Restored);
    }

    assert_eq!(fixture.value(TARGET_SEARCH_INDEXER_SESSION_PAUSE), Some("running"));
    assert_eq!(fixture.value(TARGET_SYSMAIN_START_MODE), Some("automatic"));
}

#[cfg(windows)]
#[test]
fn guarded_live_windows_scan_runs_only_when_enabled() {
    if std::env::var(LIVE_WINDOWS_INTEGRATION_ENV).ok().as_deref() != Some("1") {
        eprintln!(
            "skipping live Windows integration; set {LIVE_WINDOWS_INTEGRATION_ENV}=1 to run"
        );
        return;
    }

    let report = windows_api::scan_system().expect("live read-only scan should complete");

    assert_eq!(report.scan_mode, windows_api::SystemScanMode::ReadOnly);
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.power.source, "powercfg /getactivescheme");
}

#[cfg(not(windows))]
#[test]
fn live_windows_scan_reports_explicit_non_windows_skip() {
    eprintln!("skipping live Windows integration because this host is not Windows");
    assert_ne!(std::env::consts::OS, "windows");
}
