const { invoke } = globalThis.__TAURI__.core;
const { listen } = globalThis.__TAURI__.event;

/**
 * @typedef {import('@tauri-apps/api/event').UnlistenFn} UnlistenFn
 * A function that unsubscribes from an event.
 */

/**
 * The type of device, classified by its discovered mDNS service.
 * @typedef {'Bluesound'|'Volumio'|'SpotifyConnect'|'QobuzConnect'|'Generic'} DeviceType
 */

/**
 * Represents a specific mDNS service discovered on a device.
 * @typedef {object} DiscoveredService
 * @property {string} serviceType - The mDNS service type (e.g. `_http._tcp.local.`).
 * @property {number} port - The advertised port for the service.
 * @property {DeviceType} deviceType - Classification derived from the service.
 * @property {number} lastSeenMs - Milliseconds elapsed when this service was last observed.
 */

/**
 * Represents a device discovered on the local network.
 * @typedef {object} Device
 * @property {string} name - The advertised name of the device.
 * @property {string} ip - The IP address of the device.
 * @property {number} discoveryTimeMs - Milliseconds elapsed before the first service on this device was discovered.
 * @property {DiscoveredService[]} services - The services discovered on this device.
 */

/**
 * The RAWDOG API for the LAN Scanner plugin.
 * This is attached to `globalThis.__TAURI__.lanScanner` for easy access from the frontend.
 * @typedef {{ 
 *  startScan: typeof startScan,
 *  stopScan: typeof stopScan,
 *  isScanning: typeof isScanning,
 *  getDiscoveredDevices: typeof getDiscoveredDevices,
 *  onNewDevice: typeof onNewDevice,
 *  onScanStopped: typeof onScanStopped,
 *  onScanTick: typeof onScanTick
 * }} LanScannerPlugin
 */

/**
 * Starts the mDNS service discovery scan on the local network.
 * The scan runs for 30 seconds and then stops automatically.
 *
 * @returns {Promise<void>} A promise that resolves when the scan has been initiated.
 * @example
 * await scanner.startScan();
 * console.log("Scan started!");
 */
async function startScan() {
	await invoke("plugin:lan-scanner|start_scan");
}

/**
 * Manually stops the ongoing mDNS service discovery scan.
 *
 * @returns {Promise<void>} A promise that resolves when the scan has been stopped.
 * @example
 * await scanner.stopScan();
 * console.log("Scan stopped!");
 */
async function stopScan() {
	await invoke("plugin:lan-scanner|stop_scan");
}

/**
 * Checks if a scan is currently in progress.
 *
 * @returns {Promise<boolean>} A promise that resolves with `true` if a scan is active, otherwise `false`.
 * @example
 * const scanning = await scanner.isScanning();
 * console.log(`Is scanning: ${scanning}`);
 */
async function isScanning() {
	return await invoke("plugin:lan-scanner|is_scanning");
}

/**
 * Retrieves the list of all devices discovered since the scan started.
 *
 * @returns {Promise<Device[]>} A promise that resolves with an array of discovered devices.
 * @example
 * const devices = await scanner.getDiscoveredDevices();
 * devices.forEach(device => console.log(`Found: ${device.name} at ${device.ip}`));
 */
async function getDiscoveredDevices() {
	return await invoke("plugin:lan-scanner|get_discovered_devices");
}

/**
 * Listens for new devices discovered on the network.
 * The callback will be invoked each time a new device is found.
 *
 * @param {(device: Device) => void} callback - The function to call with the new device information.
 * @returns {Promise<UnlistenFn>} A promise that resolves with a function to unregister the listener.
 * @example
 * const unlisten = await scanner.onNewDevice((device) => {
 *   console.log(`New device: ${device.name} at ${device.ip}`);
 * });
 *
 * // To stop listening:
 * // unlisten();
 */
async function onNewDevice(callback) {
	return await listen("new-device", (event) => {
		callback(event.payload);
	});
}

/**
 * Listens for the scan to stop.
 * The callback is invoked when the scan is stopped, either manually or by the 30-second timeout.
 *
 * @param {() => void} callback - The function to call when the scan stops.
 * @returns {Promise<UnlistenFn>} A promise that resolves with a function to unregister the listener.
 * @example
 * const unlisten = await scanner.onScanStopped(() => {
 *   console.log('The scan has stopped!');
 * });
 *
 * // To stop listening:
 * // unlisten();
 */
async function onScanStopped(callback) {
	return await listen("scan-stopped", () => {
		callback();
	});
}

/**
 * Listens for the scan countdown tick.
 * The callback is invoked every second with the remaining time before the scan automatically stops.
 *
 * @param {(seconds: number) => void} callback - The function to call with the remaining seconds.
 * @returns {Promise<UnlistenFn>} A promise that resolves with a function to unregister the listener.
 * @example
 * const unlisten = await scanner.onScanTick((seconds) => {
 *   console.log(`Scan stopping in ${seconds}s`);
 * });
 *
 * // To stop listening:
 * // unlisten();
 */
async function onScanTick(callback) {
	return await listen("scan-tick", (event) => {
		callback(event.payload);
	});
}

/**
 * @type {LanScannerPlugin}
 */
const __TAURI_PLUGIN_LAN_SCANNER_API__ = {
	startScan,
	stopScan,
	isScanning,
	getDiscoveredDevices,
	onNewDevice,
	onScanStopped,
	onScanTick,
};

globalThis.__TAURI__.lanScanner = __TAURI_PLUGIN_LAN_SCANNER_API__;