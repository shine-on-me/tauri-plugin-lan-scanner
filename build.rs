fn main() {
    tauri_plugin::Builder::new(&["start_scan", "stop_scan", "is_scanning", "get_discovered_devices"])
        .global_api_script_path("./api.js")
        .build();
}
