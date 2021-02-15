use rs_docker::{container::Container, Docker};

use crate::sensor::Sensor;

use super::topic;

pub fn get_state_topic(client_id: &str, container: &Container, sensor: &Sensor) -> String {
    format!(
        "{}/{}/state",
        topic::get_base_topic(client_id, container),
        sensor
    )
}

pub fn get_state_payload(_docker: &Docker, container: &Container, sensor: &Sensor) -> String {
    match sensor {
        Sensor::Image => container.Image.to_string(),
        Sensor::Status => get_container_status(container),
    }
}

fn get_container_status(container: &Container) -> String {
    match container.Status.chars().next() {
        Some('U') => "running".to_string(),
        Some('P') => "paused".to_string(),
        Some(_) => "unknown".to_string(),
        None => "stopped".to_string(),
    }
}
