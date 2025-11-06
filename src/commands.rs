use super::models::{Device, DeviceType};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tauri::{command, AppHandle, Emitter, Manager, Runtime, State};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const BLUESOUND_SERVICE_TYPE: &str = "_musc._tcp.local.";
const VOLUMIO_SERVICE_TYPE: &str = "_http._tcp.local.";
const SPOTIFY_CONNECT_SERVICE_TYPE: &str = "_spotify-connect._tcp.local.";
const QOBUZ_CONNECT_SERVICE_TYPE: &str = "_qobuz-connect._tcp.local.";

/// Holds the state for the mDNS scanning service.
///
/// This struct is managed by Tauri and provides shared access to the mDNS daemon,
/// the scanning status, a map of discovered devices, and the handle for the scan timeout task.
#[derive(Default)]
pub struct MdnsState {
    /// The mDNS service daemon instance.
    pub daemon: Arc<Mutex<Option<ServiceDaemon>>>,
    /// A flag indicating whether a scan is currently in progress.
    pub scanning: Arc<Mutex<bool>>,
    /// A map of discovered devices, keyed by their IP address.
    pub devices: Arc<Mutex<HashMap<String, Device>>>,
    /// The handle for the asynchronous task that stops the scan after a timeout.
    pub timeout_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Resolves the `DeviceType` from the mDNS service type domain and fullname.
fn resolve_device_type(ty_domain: &str, fullname: &str) -> Option<DeviceType> {
    match ty_domain {
        BLUESOUND_SERVICE_TYPE => Some(DeviceType::Bluesound),
        VOLUMIO_SERVICE_TYPE if fullname.to_lowercase().contains("volumio") => {
            Some(DeviceType::Volumio)
        }
        SPOTIFY_CONNECT_SERVICE_TYPE => Some(DeviceType::SpotifyConnect),
        QOBUZ_CONNECT_SERVICE_TYPE => Some(DeviceType::QobuzConnect),
        _ => Some(DeviceType::Generic),
    }
}

/// Handles a resolved mDNS service, updating the device list and emitting an event.
async fn handle_resolved_service<R: Runtime>(
    info: Box<mdns_sd::ResolvedService>,
    app_handle: AppHandle<R>,
    seen_services: Arc<Mutex<HashSet<String>>>,
    devices: Arc<Mutex<HashMap<String, Device>>>,
    service_type: &str,
    scan_start_time: Instant,
) {
    log::debug!(
        "Addresses for {}: {:?}",
        info.get_fullname(),
        info.get_addresses()
    );
    let ip_option = info.get_addresses().iter().find_map(|addr| {
        match addr.to_ip_addr() {
            IpAddr::V4(ipv4_addr) if !ipv4_addr.is_link_local() => Some(IpAddr::V4(ipv4_addr)),
            _ => None,
        }
    });

    let Some(ip) = ip_option else { return };

    let ip_string = ip.to_string();
    let service_key = format!("{ip_string}|{service_type}");
    if !seen_services.lock().await.insert(service_key) {
        return;
    }

    let Some(device_type) = resolve_device_type(service_type, info.get_fullname()) else {
        return;
    };

    let name = info
        .get_fullname()
        .split('.')
        .next()
        .unwrap_or("")
        .to_string();
    let port = info.get_port();
    let elapsed_ms = scan_start_time.elapsed().as_millis();

    log::info!(
        "{} ({}:{}) {} ({}ms)",
        name,
        ip_string,
        port,
        service_type,
        elapsed_ms
    );

    let mut devices_guard = devices.lock().await;
    let device_entry = devices_guard
        .entry(ip_string.clone())
        .or_insert_with(|| Device {
            name: name.clone(),
            ip: ip_string.clone(),
            discovery_time_ms: elapsed_ms,
            services: Vec::new(),
        });

    if elapsed_ms < device_entry.discovery_time_ms {
        device_entry.discovery_time_ms = elapsed_ms;
    }
    device_entry.name = name.clone();

    device_entry.add_or_update_service(&service_type, port, device_type.clone(), elapsed_ms);

    let device_payload = device_entry.clone();
    drop(devices_guard);

    if let Err(e) = app_handle.emit("new-device", &device_payload) {
        log::error!("Failed to emit new-device event: {}", e);
    }
}

/// Processes events from a specific mDNS service receiver.
async fn process_service_receiver<R: Runtime>(
    receiver: mdns_sd::Receiver<ServiceEvent>,
    app_handle: AppHandle<R>,
    seen_services: Arc<Mutex<HashSet<String>>>,
    devices: Arc<Mutex<HashMap<String, Device>>>,
    service_type: String,
    scan_start_time: Instant,
) {
    while let Ok(event) = receiver.recv_async().await {
        if let ServiceEvent::ServiceResolved(info) = event {
            handle_resolved_service(
                info,
                app_handle.clone(),
                seen_services.clone(),
                devices.clone(),
                &service_type,
                scan_start_time,
            )
            .await;
        }
    }
    log::info!("Receiver for {} disconnected.", service_type);
}

/// Starts the LAN scan for mDNS services.
///
/// This command initializes the mDNS daemon, browses for a predefined set of services,
/// and spawns a timeout task to automatically stop the scan after 30 seconds.
#[command]
pub async fn start_scan<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, MdnsState>,
) -> Result<(), String> {
    log::info!("`start_scan` command called");
    let mut scanning_guard = state.scanning.lock().await;
    if *scanning_guard {
        log::info!("Scan is already in progress.");
        return Ok(());
    }
    *scanning_guard = true;
    drop(scanning_guard);

    // Abort any existing timeout task to prevent multiple stop calls
    if let Some(task) = state.timeout_task.lock().await.take() {
        task.abort();
    }

    log::info!("Starting LAN scan");

    state.devices.lock().await.clear();

    let mdns_result = ServiceDaemon::new();

    let mdns = match mdns_result {
        Ok(daemon) => daemon,
        Err(e) => {
            log::error!("Failed to create mDNS daemon: {}", e);
            *state.scanning.lock().await = false;
            return Err(format!("Failed to create mDNS daemon: {}", e));
        }
    };

    {
        let mut daemon_guard = state.daemon.lock().await;
        *daemon_guard = Some(mdns.clone());
    }

    let scan_start_time = Instant::now();
    let services_to_browse = vec![
        BLUESOUND_SERVICE_TYPE.to_string(),
        VOLUMIO_SERVICE_TYPE.to_string(),
        SPOTIFY_CONNECT_SERVICE_TYPE.to_string(),
        QOBUZ_CONNECT_SERVICE_TYPE.to_string(),
    ];

    let seen_services = Arc::new(Mutex::new(HashSet::new()));
    let state_devices = state.devices.clone();

    for service_type in services_to_browse {
        log::debug!("Browsing for service type: {}", service_type);
        let receiver = match mdns.browse(&service_type) {
            Ok(rec) => rec,
            Err(e) => {
                log::error!("Failed to browse for service '{}': {}", service_type, e);
                continue;
            }
        };

        tokio::spawn(process_service_receiver(
            receiver,
            app.clone(),
            seen_services.clone(),
            state_devices.clone(),
            service_type,
            scan_start_time,
        ));
    }

    let app_clone = app.clone();
    let timeout_task = tokio::spawn(async move {
        const SCAN_DURATION_SECS: u64 = 30;
        for i in 0..SCAN_DURATION_SECS {
            let seconds_left = SCAN_DURATION_SECS - i;
            log::info!("Scan stopping in {} seconds...", seconds_left);
            if let Err(e) = app_clone.emit("scan-tick", seconds_left) {
                log::warn!("Failed to emit scan-tick event: {}", e);
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        log::info!("Scan timeout reached. Stopping scan automatically.");
        let state_from_app = app_clone.state::<MdnsState>();
        if let Err(e) = stop_scan(app_clone.clone(), state_from_app).await {
            log::error!("Failed to stop scan automatically: {}", e);
        }
    });

    *state.timeout_task.lock().await = Some(timeout_task);

    Ok(())
}

/// Stops the LAN scan.
///
/// This command shuts down the mDNS daemon and aborts the scan timeout task.
#[command]
pub async fn stop_scan<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, MdnsState>,
) -> Result<(), String> {
    log::info!("Stopping LAN scan");
    let mut scanning_guard = state.scanning.lock().await;
    if !*scanning_guard {
        log::info!("Scan is not running.");
        return Ok(());
    }
    *scanning_guard = false;
    drop(scanning_guard);

    // Abort the timeout task as it's no longer needed
    if let Some(task) = state.timeout_task.lock().await.take() {
        task.abort();
    }

    if let Some(mdns) = state.daemon.lock().await.take() {
        if let Err(e) = mdns.shutdown() {
            log::error!("Failed to shutdown mDNS daemon: {}", e);
            return Err(format!("Failed to shutdown mDNS daemon: {}", e));
        }
        log::info!("mDNS daemon shut down.");
        if let Err(e) = app.emit("scan-stopped", ()) {
            log::error!("Failed to emit scan-stopped event: {}", e);
        }
    }
    Ok(())
}

/// Checks if a scan is currently in progress.
#[command]
pub async fn is_scanning(state: State<'_, MdnsState>) -> Result<bool, String> {
    Ok(*state.scanning.lock().await)
}

/// Returns the list of discovered devices.
#[command]
pub async fn get_discovered_devices(state: State<'_, MdnsState>) -> Result<Vec<Device>, String> {
    let devices_guard = state.devices.lock().await;
    Ok(devices_guard.values().cloned().collect())
}
