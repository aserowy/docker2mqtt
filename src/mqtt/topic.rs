use tracing::instrument;

use crate::configuration::Configuration;

#[instrument(level = "debug")]
pub fn availability(container_name: &str, conf: &Configuration) -> String {
    device_availability(&conf.mqtt.client_id, container_name)
}

#[instrument(level = "debug")]
pub fn state(container_name: &str, event_name: &str, conf: &Configuration) -> String {
    format!(
        "{}/{}/state",
        base(&conf.mqtt.client_id, container_name),
        event_name
    )
}

fn device_availability(client_id: &str, container_name: &str) -> String {
    format!("{}/lwt", base(client_id, container_name))
}

fn base(client_id: &str, container_name: &str) -> String {
    format!("docker2mqtt/{}/{}", client_id, container_name)
}
