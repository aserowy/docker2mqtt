use serde::Serialize;

use crate::{docker::Container, sensor};

use super::{lwt, state};

pub fn get_discovery_topic(
    hass_discovery_prefix: &str,
    client_id: &str,
    container: &Container,
    sensor: &sensor::Sensor,
) -> String {
    let (_, unique_id) = get_ids(client_id, container, sensor);

    format!(
        "{}/sensor/docker2mqtt/{}/config",
        hass_discovery_prefix, unique_id
    )
}

pub fn get_discovery_payload(
    client_id: &str,
    container: &Container,
    sensor: &sensor::Sensor,
) -> String {
    let (device_name, unique_id) = get_ids(client_id, container, sensor);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let sensor = Sensor {
        availability_topic: lwt::get_availability_topic(client_id, container),
        device: Device {
            identifiers,
            manufacturer: "docker2mqtt".to_string(),
            model: "docker".to_string(),
            name: device_name.to_string(),
        },
        name: sensor.to_string(),
        payload_available: "online".to_string(),
        payload_not_available: "offline".to_string(),
        platform: "mqtt".to_string(),
        state_topic: state::get_state_topic(client_id, container, sensor),
        unique_id,
    };

    serde_json::to_string(&sensor).unwrap()
}

#[derive(Serialize)]
struct Sensor {
    pub availability_topic: String,
    pub device: Device,
    pub name: String,
    pub payload_available: String,
    pub payload_not_available: String,
    pub platform: String,
    pub state_topic: String,
    pub unique_id: String,
}

#[derive(Serialize)]
struct Device {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub model: String,
    pub name: String,
}

fn get_ids(client_id: &str, container: &Container, sensor: &sensor::Sensor) -> (String, String) {
    let device_name = format!("docker_{}_{}", client_id, container.name);

    let unique_id = format!("{}_{}", device_name, sensor);

    (device_name, unique_id)
}
