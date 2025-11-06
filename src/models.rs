use serde::Serialize;

/// Represents a device discovered on the local network.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// The advertised name of the device.
    pub name: String,
    /// The IP address of the device.
    pub ip: String,
    /// The time in milliseconds from the start of the scan until the first service on this device was discovered.
    pub discovery_time_ms: u128,
    /// A list of mDNS services discovered on this device.
    pub services: Vec<DiscoveredService>,
}

impl Device {
    /// Adds a new service to the device or updates an existing one.
    pub fn add_or_update_service(
        &mut self,
        service_type: &str,
        port: u16,
        device_type: DeviceType,
        elapsed_ms: u128,
    ) {
        if let Some(service) = self
            .services
            .iter_mut()
            .find(|s| s.service_type == service_type)
        {
            service.port = port;
            service.device_type = device_type;
            service.last_seen_ms = elapsed_ms;
        } else {
            self.services.push(DiscoveredService {
                service_type: service_type.to_string(),
                port,
                device_type,
                last_seen_ms: elapsed_ms,
            });
        }
    }
}

/// Represents a specific mDNS service discovered on a device.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredService {
    /// The mDNS service type (e.g., `_http._tcp.local.`).
    pub service_type: String,
    /// The advertised port for the service.
    pub port: u16,
    /// The classification of the device based on the service type.
    pub device_type: DeviceType,
    /// The time in milliseconds from the start of the scan when this service was last observed.
    pub last_seen_ms: u128,
}

/// The type of device, classified by its discovered mDNS service.
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum DeviceType {
    /// A Bluesound device.
    Bluesound,
    /// A Volumio device.
    Volumio,
    /// A device with Spotify Connect.
    SpotifyConnect,
    /// A device with Qobuz Connect.
    QobuzConnect,
    /// A generic or unrecognized device.
    Generic,
}
