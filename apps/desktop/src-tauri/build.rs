fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&[
                "check_signed_update",
                "get_ipc_security_status",
                "get_live_resource_snapshot",
                "get_updater_configuration",
                "run_read_only_system_scan",
            ])),
    )
    .expect("failed to build Tauri application manifest");
}
