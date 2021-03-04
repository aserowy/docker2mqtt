use serde::Serialize;

use crate::{
    configuration::{Configuration, Hassio},
    mqtt::availability::Availability,
};

use super::topic;

#[derive(Serialize)]
pub struct HassioEvent {
    pub availability_topic: String,
    pub device: HassioDevice,
    pub name: String,
    pub payload_available: String,
    pub payload_not_available: String,
    pub platform: String,
    pub state_topic: String,
    pub unique_id: String,
}

#[derive(Serialize)]
pub struct HassioDevice {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub model: String,
    pub name: String,
}

pub fn create(
    container_name: &str,
    event_name: &str,
    conf: &Configuration,
    hassio: &Hassio,
) -> String {
    let device_name = get_device_name(conf, hassio, container_name);
    let unique_id = get_unique_id(conf, hassio, container_name, event_name);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let event = HassioEvent {
        availability_topic: topic::availability(container_name, conf),
        device: HassioDevice {
            identifiers,
            manufacturer: "docker2mqtt".to_string(),
            model: "docker".to_string(),
            name: device_name.to_string(),
        },
        name: unique_id.to_string(),
        payload_available: Availability::Online.to_string(),
        payload_not_available: Availability::Offline.to_string(),
        platform: "mqtt".to_string(),
        state_topic: topic::state(container_name, event_name, conf),
        unique_id,
    };

    serde_json::to_string(&event).unwrap()
}

fn get_device_name(conf: &Configuration, hassio: &Hassio, container_name: &str) -> String {
    format!(
        "{}_{}_{}",
        hassio.device_prefix, conf.mqtt.client_id, container_name
    )
}

pub fn get_unique_id(
    conf: &Configuration,
    hassio: &Hassio,
    container_name: &str,
    event_name: &str,
) -> String {
    format!(
        "{}_{}",
        get_device_name(conf, hassio, container_name),
        event_name
    )
}
