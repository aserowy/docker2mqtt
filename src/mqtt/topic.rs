use tracing::instrument;

use crate::{configuration::Configuration, docker::Event};

#[instrument(level = "debug")]
pub fn availability(event: &Event, conf: &Configuration) -> String {
    device_availability(&conf.mqtt.client_id, &event.container_name)
}

#[instrument(level = "debug")]
pub fn state(event: &Event, conf: &Configuration) -> String {
    let container_name = &event.container_name;
    let event_name = &event.event.to_string();

    format!(
        "{}/{}/state",
        base(&conf.mqtt.client_id, container_name),
        event_name
    )
}

fn device_availability(client_id: &str, container: &str) -> String {
    format!("{}/lwt", base(client_id, container))
}

fn base(client_id: &str, container: &str) -> String {
    format!("docker2mqtt/{}/{}", client_id, container)
}
