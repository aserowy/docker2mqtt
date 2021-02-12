use crate::topic;

use rs_docker::container::Container;
use serde::Serialize;

#[derive(Serialize)]
pub struct Sensor {
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
pub struct Device {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub model: String,
    pub name: String,
}

pub fn get_discovery_payload(host: &str, container: &Container, sensor: &str) -> String {
    let device_name = &format!("docker_{}", host);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let container_name = topic::get_container_name(container);
    let unique_id = format!("{}_{}_{}", device_name, container_name, sensor);

    let sensor = Sensor {
        availability_topic: topic::get_availability_topic(host, container),
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
        state_topic: topic::get_state_topic(host, container, sensor),
        unique_id,
    };

    serde_json::to_string(&sensor).unwrap()
}
