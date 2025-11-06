# Tauri Plugin LAN Scanner

A Tauri plugin for discovering devices on the local network using mDNS.

This plugin provides a simple API to scan for devices advertising specific services (like `_http._tcp.local.`) and reports them back to your Tauri frontend in real-time. It's built with RAWDOG principles: pure Rust on the backend, and a zero-build, JSDoc-annotated JavaScript API for the frontend.

## Install

Add the following to your `src-tauri/Cargo.toml` file under the `[dependencies]` section:

```toml
[dependencies]
tauri-plugin-lan-scanner = { git = "https://github.com/shine-on-me/tauri-plugin-lan-scanner", branch = "main" }
```

## Usage

First, you need to register the plugin with Tauri in your `main.rs` file:

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_lan_scanner::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Next, you can access the plugin's API from your frontend JavaScript:

```javascript
// In your frontend code
// The plugin is injected automatically, so you can access it globally.
const { lanScanner: scanner } = globalThis.__TAURI__;

// Kick off a scan. It runs for 30 seconds and then stops on its own.
await scanner.startScan();

// You can also stop it manually whenever you want.
await scanner.stopScan();

// Not sure if a scan is running? Just ask.
const isRunning = await scanner.isScanning();
console.log(`Are we scanning? ${isRunning}`);

// See what devices have been found so far.
const devices = await scanner.getDiscoveredDevices();
console.log("Discovered devices:", devices);

// Listen for new devices as they pop up on the network.
const unlistenDevice = await scanner.onNewDevice((device) => {
  console.log(`New device found: ${device.name} at ${device.ip}`);
});

// Get a heads-up every second before the scan automatically stops.
const unlistenTick = await scanner.onScanTick((seconds) => {
  console.log(`Scan stopping in ${seconds}s`);
});

// Know exactly when the scan has finished.
const unlistenStop = await scanner.onScanStopped(() => {
  console.log("The scan has officially stopped!");
  // It's good practice to clean up your listeners when you're done.
  unlistenDevice();
  unlistenTick();
  unlistenStop();
});
```

## API

The frontend API is exposed via `globalThis.__TAURI__.lanScanner` and provides a clean, promise-based interface.

### `startScan(): Promise<void>`

Starts the mDNS service discovery scan on the local network. The scan runs for 30 seconds by default and then stops automatically.

### `stopScan(): Promise<void>`

Manually stops the ongoing mDNS service discovery scan.

### `isScanning(): Promise<boolean>`

Checks if a scan is currently in progress.

### `getDiscoveredDevices(): Promise<Device[]>`

Retrieves the list of all devices discovered since the scan started.

### `onNewDevice((device: Device) => void): Promise<UnlistenFn>`

Listens for new devices discovered on the network. The callback will be invoked each time a new device is found.

### `onScanStopped(() => void): Promise<UnlistenFn>`

Listens for the scan to stop. The callback is invoked when the scan is stopped, either manually or by the 30-second timeout.

### `onScanTick((seconds: number) => void): Promise<UnlistenFn>`

Listens for the scan countdown tick. The callback is invoked every second with the remaining time before the scan automatically stops.

## Contributing

This project is proudly RAWDOG compliant. Our philosophy is simple: no over-engineering. Contributions that adhere to the following principles are welcome:

-   **Zero Build Steps for Frontend:** The frontend API (`api.js`) must remain pure JavaScript with JSDoc annotations. No TypeScript, no bundlers, no compilers.
-   **Lean Rust Core:** The Rust code should be clean, efficient, and well-documented.
-   **Web Standards Supremacy:** If a native platform feature can do the job, we use it.

Feel free to open an issue or submit a pull request!