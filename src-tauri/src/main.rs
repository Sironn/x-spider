// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod network;

use tauri::Manager;
use tauri_plugin_window_state::{Builder as WindowStateBuilder, StateFlags};

fn main() {
    tauri::Builder::default()
        .plugin(
            WindowStateBuilder::default()
                .with_state_flags(StateFlags::SIZE)
                .build(),
        )
        .setup(|app| {
            app.get_window("main")
                .expect("main window not found")
                .center()?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
          network::network_fetch,
          network::network_get_system_proxy_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
