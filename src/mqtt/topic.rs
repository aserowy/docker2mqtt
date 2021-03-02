use tracing::instrument;

use crate::{
    configuration::Configuration,
    docker::{Event, EventType},
};

#[instrument(level = "debug")]
pub fn availability(event: &Event, conf: &Configuration) -> String {
    let container_name = &event.container_id;
    let event_name = &event.event_type.to_string();

    match &event.event_type {
        &EventType::Image => event_availibility(&conf.mqtt.client_id, container_name, event_name),
        &EventType::Status => event_availibility(&conf.mqtt.client_id, container_name, event_name),

        &EventType::CpuUsage => device_availability(&conf.mqtt.client_id, container_name),
        &EventType::MemoryUsage => device_availability(&conf.mqtt.client_id, container_name),
    }
}

#[instrument(level = "debug")]
pub fn state(event: &Event, conf: &Configuration) -> String {
    let container_name = &event.container_id;
    let event_name = &event.event_type.to_string();

    format!(
        "{}/{}/state",
        base(&conf.mqtt.client_id, container_name),
        event_name
    )
}

fn device_availability(client_id: &str, container: &str) -> String {
    format!("{}/lwt", base(client_id, container))
}

fn event_availibility(client_id: &str, container: &str, event: &str) -> String {
    format!("{}/{}/lwt", base(client_id, container), event)
}

fn base(client_id: &str, container: &str) -> String {
    format!("docker2mqtt/{}/{}", client_id, container)
}
