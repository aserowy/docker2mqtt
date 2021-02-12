use crate::{sensor::Sensor, topic};

use rs_docker::{container::Container, Docker};

pub fn get_state_topic(host: &str, container: &Container, sensor: &Sensor) -> String {
    format!(
        "{}/{}/state",
        topic::get_base_topic(host, container),
        sensor
    )
}

pub fn get_state_payload(docker: &Docker, container: &Container, sensor: &Sensor) -> String {
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
