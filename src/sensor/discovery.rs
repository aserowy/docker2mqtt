use serde::Serialize;

use crate::{docker::Container, Args};

use super::{availability, state, HassioErr, HassioResult, Sensor};

pub fn topic(args: &Args, container: &Container, sensor: &Sensor) -> HassioResult<String> {
    let (_, unique_id) = get_ids(&args.client_id, container, &sensor.to_string());

    let prefix = match args.hass_discovery_prefix.to_owned() {
        Some(value) => value,
        None => return Err(HassioErr::PrefixNotSet),
    };

    Ok(format!(
        "{}/sensor/docker2mqtt/{}/config",
        prefix, unique_id
    ))
}

pub fn payload(args: &Args, container: &Container, sensor: &Sensor) -> HassioResult<String> {
    let (device_name, unique_id) = get_ids(&args.client_id, container, &sensor.to_string());

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let sensor = HassioSensor {
        availability_topic: availability::topic(args, container, sensor),
        device: HassioDevice {
            identifiers,
            manufacturer: "docker2mqtt".to_string(),
            model: "docker".to_string(),
            name: device_name.to_string(),
        },
        name: unique_id.to_string(),
        payload_available: "online".to_string(),
        payload_not_available: "offline".to_string(),
        platform: "mqtt".to_string(),
        state_topic: state::topic(args, container, sensor),
        unique_id,
    };

    Ok(serde_json::to_string(&sensor).unwrap())
}

#[derive(Serialize)]
struct HassioSensor {
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
struct HassioDevice {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub model: String,
    pub name: String,
}

fn get_ids(client_id: &str, container: &Container, sensor: &str) -> (String, String) {
    let device_name = format!("docker_{}_{}", client_id, container.name);
    let unique_id = format!("{}_{}", device_name, sensor);

    (device_name, unique_id)
}
