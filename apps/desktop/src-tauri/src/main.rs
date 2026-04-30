mod ipc;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ipc::get_ipc_security_status])
        .run(tauri::generate_context!())
        .expect("failed to run Liiiraa Booster desktop app");
}
