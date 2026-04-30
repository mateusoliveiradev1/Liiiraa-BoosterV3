fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&[
                "get_ipc_security_status",
                "run_read_only_system_scan",
            ])),
    )
    .expect("failed to build Tauri application manifest");
}
