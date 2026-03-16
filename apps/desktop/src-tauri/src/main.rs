use serde::Serialize;

#[derive(Debug, Serialize)]
struct DesktopRuntimeHealth {
    shell: &'static str,
    secure_store: &'static str,
    command_bus: &'static str,
}

#[tauri::command]
fn desktop_runtime_health() -> DesktopRuntimeHealth {
    DesktopRuntimeHealth {
        shell: "tauri",
        secure_store: "os-keychain",
        command_bus: "local-rust-control-plane",
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![desktop_runtime_health])
        .run(tauri::generate_context!())
        .expect("error while running adaptive operator desktop");
}

fn main() {
    run();
}

