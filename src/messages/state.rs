use crate::{
    docker::{Container, DockerClient},
    sensor::Sensor,
};

use super::topic;

pub fn get_state_topic(client_id: &str, container: &Container, sensor: &Sensor) -> String {
    format!(
        "{}/{}/state",
        topic::get_base_topic(client_id, container),
        sensor
    )
}

pub fn get_state_payload(_client: &DockerClient, container: &Container, sensor: &Sensor) -> String {
    match sensor {
        Sensor::Image => container.image.to_string(),
        Sensor::Status => get_container_status(container),
    }
}

fn get_container_status(container: &Container) -> String {
    let mut result = "unknown".to_string();
    if container.status.contains("Paused") {
        result = "paused".to_string();
    }

    if container.status.contains("Up") {
        result = "running".to_string();
    }

    if container.status.contains("Restarting") {
        result = "restarting".to_string();
    }

    if container.status.contains("Exited") {
        result = "stopped".to_string();
    }

    result
}
