use serde::Serialize;
use tracing::instrument;

use crate::{
    configuration::{Configuration, Hassio},
    docker::{Availability, Event},
};

use super::topic;

pub type HassioResult<T> = Result<T, HassioErr>;

#[derive(Debug)]
pub enum HassioErr {
    DiscoveryDisabled,
}

#[instrument(level = "debug")]
pub fn topic(event: &Event, conf: &Configuration) -> HassioResult<String> {
    let hassio = match get_hassio(conf) {
        Ok(hassio) => hassio,
        Err(e) => return Err(e),
    };

    let (_, unique_id) = get_ids(conf, hassio, event);

    Ok(format!(
        "{}/event/docker2mqtt/{}/config",
        hassio.discovery_prefix, unique_id
    ))
}

#[instrument(level = "debug")]
pub fn payload(event: &Event, conf: &Configuration) -> HassioResult<String> {
    let hassio = match get_hassio(conf) {
        Ok(hassio) => hassio,
        Err(e) => return Err(e),
    };

    let (device_name, unique_id) = get_ids(conf, hassio, event);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let event = HassioEvent {
        availability_topic: topic::availability(event, conf),
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
        state_topic: topic::state(event, conf),
        unique_id,
    };

    Ok(serde_json::to_string(&event).unwrap())
}

#[derive(Serialize)]
struct HassioEvent {
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

fn get_hassio(conf: &Configuration) -> HassioResult<&Hassio> {
    match &conf.hassio {
        Some(hassio) => match hassio {
            Hassio {
                discovery: false, ..
            } => return Err(HassioErr::DiscoveryDisabled),
            _ => Ok(hassio),
        },
        None => return Err(HassioErr::DiscoveryDisabled),
    }
}

fn get_ids(conf: &Configuration, hassio: &Hassio, event: &Event) -> (String, String) {
    let container_name = &event.container_name;
    let event_name = event.event.to_string();

    let device_name = format!(
        "{}_{}_{}",
        hassio.device_prefix, conf.mqtt.client_id, container_name
    );

    let unique_id = format!("{}_{}", device_name, event_name);

    (device_name, unique_id)
}
