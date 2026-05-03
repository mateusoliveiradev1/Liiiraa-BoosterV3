mod ipc;
mod updater;

fn main() {
    tauri::Builder::default()
        .setup(updater::setup)
        .invoke_handler(tauri::generate_handler![
            ipc::get_ipc_security_status,
            ipc::get_live_resource_snapshot,
            ipc::run_read_only_system_scan,
            updater::check_signed_update,
            updater::get_updater_configuration,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Liiiraa Booster desktop app");
}
