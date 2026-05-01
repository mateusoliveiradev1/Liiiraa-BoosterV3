use windows_api::{parse_system_scan_report, scan_system, SystemScanMode};

const FIXTURE: &str = include_str!("fixtures/system_scan.json");

#[test]
fn fixture_covers_t040_inventory_sections() {
    let report = parse_system_scan_report(FIXTURE).expect("fixture should parse");

    assert_eq!(report.scan_mode, SystemScanMode::ReadOnly);
    assert!(report.covers_t040_inventory());
    assert_eq!(
        report.power.active_scheme_guid.as_deref(),
        Some("381b4222-f694-41f0-9685-ff5bb260df2e")
    );
    assert_eq!(report.security.hvci.enabled, Some(1));
    assert!(report
        .security
        .optional_features
        .iter()
        .any(|feature| feature.name == "VirtualMachinePlatform"
            && feature.install_state == Some(1)));
    assert_eq!(report.storage.storage_sense.enabled, Some(false));
    assert_eq!(report.storage.trim.ntfs_disable_delete_notify, Some(0));
    assert_eq!(report.storage.direct_storage.nvme_present, Some(true));
}

#[test]
fn fixture_keeps_scan_read_only_and_complete() {
    let report = parse_system_scan_report(FIXTURE).expect("fixture should parse");

    assert_eq!(report.power.source, "powercfg /getactivescheme");
    assert_eq!(report.storage.cleanup.candidates.len(), 2);
    assert!(!report.reboot_required.is_reboot_required());
    assert!(report.collection_errors.is_empty());
}

#[cfg(windows)]
#[test]
#[ignore = "manual read-only dry run against the local Windows host"]
fn live_system_scan_dry_run() {
    let report = scan_system().expect("live read-only scan should complete");

    assert_eq!(report.scan_mode, SystemScanMode::ReadOnly);
    assert_eq!(report.schema_version, 1);
    assert!(!report.os.caption.is_empty());
    assert!(!report.cpus.is_empty());
    assert_eq!(report.power.source, "powercfg /getactivescheme");
}
