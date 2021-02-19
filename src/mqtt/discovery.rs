use serde::Serialize;

use crate::{
    configuration::{Configuration, Hassio},
    sensor::Sensor,
};

use super::topic;

pub type HassioResult<T> = Result<T, HassioErr>;

pub enum HassioErr {
    DiscoveryDisabled,
}

pub fn topic<'a>(sensor: &Sensor<'a>, conf: &Configuration) -> HassioResult<String> {
    let hassio = match &conf.hassio {
        Some(hassio) => match hassio {
            Hassio {
                discovery: false, ..
            } => return Err(HassioErr::DiscoveryDisabled),
            _ => hassio,
        },
        None => return Err(HassioErr::DiscoveryDisabled),
    };

    let (_, unique_id) = get_ids(conf, hassio, sensor);

    Ok(format!(
        "{}/sensor/docker2mqtt/{}/config",
        hassio.discovery_prefix, unique_id
    ))
}

pub fn payload<'a>(sensor: &Sensor<'a>, conf: &Configuration) -> HassioResult<String> {
    let hassio = match &conf.hassio {
        Some(hassio) => match hassio {
            Hassio {
                discovery: false, ..
            } => return Err(HassioErr::DiscoveryDisabled),
            _ => hassio,
        },
        None => return Err(HassioErr::DiscoveryDisabled),
    };

    let (device_name, unique_id) = get_ids(conf, hassio, sensor);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let sensor = HassioSensor {
        availability_topic: topic::availability(sensor, conf),
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
        state_topic: topic::state(sensor, conf),
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

fn get_ids(conf: &Configuration, hassio: &Hassio, sensor: &Sensor) -> (String, String) {
    let container_name = &sensor.container.name;
    let sensor_name = sensor.sensor_type.to_string();

    let device_name = format!(
        "{}_{}_{}",
        hassio.device_prefix, conf.mqtt.client_id, container_name
    );

    let unique_id = format!("{}_{}", device_name, sensor_name);

    (device_name, unique_id)
}
