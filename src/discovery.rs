use rs_docker::container::Container;
use serde::Serialize;

use crate::{container, lwt, sensor, state};

pub fn get_discovery_topic(
    hass_discovery_prefix: &str,
    host: &str,
    container: &Container,
    sensor: &sensor::Sensor,
) -> String {
    format!(
        "{}/sensor/docker2mqtt/{}/config",
        hass_discovery_prefix,
        get_unique_id(host, container, sensor)
    )
}

pub fn get_discovery_payload(host: &str, container: &Container, sensor: &sensor::Sensor) -> String {
    let device_name = &get_device_name(host);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let unique_id = get_unique_id(host, container, sensor);

    let sensor = Sensor {
        availability_topic: lwt::get_availability_topic(host, container),
        device: Device {
            identifiers,
            manufacturer: "docker2mqtt".to_string(),
            model: "docker".to_string(),
            name: device_name.to_string(),
        },
        icon: "".to_string(),
        name: unique_id.to_string(),
        payload_available: "online".to_string(),
        payload_not_available: "offline".to_string(),
        platform: "mqtt".to_string(),
        state_topic: state::get_state_topic(host, container, sensor),
        unique_id,
    };

    serde_json::to_string(&sensor).unwrap()
}

fn get_unique_id(host: &str, container: &Container, sensor: &sensor::Sensor) -> String {
    format!(
        "{}_{}_{}",
        get_device_name(host),
        container::get_container_name(container),
        sensor
    )
}

fn get_device_name(host: &str) -> String {
    format!("docker_{}", host)
}

#[derive(Serialize)]
struct Sensor {
    pub availability_topic: String,
    pub device: Device,
    pub icon: String,
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
