//! A Tauri plugin for discovering devices on the local network using mDNS.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod models;

/// Initializes the LAN scanner plugin.
///
/// This function creates and configures the Tauri plugin, setting up the necessary state
/// and registering the invoke handlers for the frontend API.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("lan-scanner")
        .invoke_handler(tauri::generate_handler![
            commands::start_scan,
            commands::stop_scan,
            commands::is_scanning,
            commands::get_discovered_devices
        ])
        .setup(|app, _api| {
            log::info!("lan-scanner plugin initialized");
            app.manage(commands::MdnsState::default());
            Ok(())
        })
        .build()
}
