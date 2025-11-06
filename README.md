# Tauri Plugin LAN Scanner

A Tauri plugin for discovering devices on the local network using mDNS.

This plugin provides a simple API to scan for devices advertising specific services (like `_http._tcp.local.`) and reports them back to your Tauri frontend in real-time. It's built with RAWDOG principles: pure Rust on the backend, and a zero-build, JSDoc-annotated JavaScript API for the frontend.

## Install

Add the following to your `src-tauri/Cargo.toml` file under the `[dependencies]` section:

```toml
[dependencies]
tauri-plugin-lan-scanner = { git = "https://github.com/shine-on-me/tauri-plugin-lan-scanner", branch = "master" }
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

## Types

We get full type-safety on our plain JavaScript API without any build steps. Here's how.

We leverage the TypeScript language server to check our `.js` files directly. By enabling `"checkJs": true` in a `deno.jsonc`, we command the LSP to analyze our code and treat JSDoc comments as authoritative type annotations.

To get direct access to this plugin's types without any installs, we use a web-standard [Import Map](https://html.spec.whatwg.org/multipage/webappapis.html#import-maps). This lets us map a clean module specifier directly to the raw `api.js` file on GitHub. That's right—no `npm install`, no `node_modules`, just a URL.

If you're still on Node.js, you can try a similar setup. They're finally adding [import maps](https://nodejs.org/api/module.html#import-maps) and some [subpath imports](https://nodejs.org/docs/v22.19.0/api/packages.html#subpath-imports) thing to `package.json` to escape the `node_modules` disaster. Good luck to you.

```jsonc
// Example from a deno.jsonc
{
  "compilerOptions": {
    "checkJs": true,
  },
  "imports": {
    // just a raw URL, that's it
    "@lan-scanner/": "https://raw.githubusercontent.com/shine-on-me/tauri-plugin-lan-scanner/master/"
  }
}
```

### JSDoc Type Imports

With the import map configured, you can access the plugin from the global scope and pull in types for anything you need using a JSDoc `import()`.

```javascript
// In your frontend code
const { lanScanner: scanner } = globalThis.__TAURI__;

/** @type {Map<string, import("@lan-scanner/api.js").Device>} */
const discoveredDevices = new Map();

// Now you have full type-safety on `scanner` and `discoveredDevices`.
// No build step. No node_modules. Pure power.
```

This approach allows us to write standard JavaScript, get all the benefits of a first-class type system, and ship that exact same code to the browser. No builds, no complexity. Just raw power.

## Contributing

This project is proudly RAWDOG compliant. Our philosophy is simple: no over-engineering. Contributions that adhere to the following principles are welcome:

-   **Zero Build Steps for Frontend:** The frontend API (`api.js`) must remain pure JavaScript with JSDoc annotations. No TypeScript, no bundlers, no compilers.
-   **Web Standards Supremacy:** If a native platform feature can do the job, we use it.

Feel free to open an issue or submit a pull request!