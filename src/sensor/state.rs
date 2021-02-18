use crate::docker::{Container, DockerClient};

use super::SensorType;

pub fn get_state(client: &DockerClient, container: &Container, sensor: &SensorType) -> String {
    match sensor {
        SensorType::CpuUsage => get_cpu_usage_payload(client, container),
        SensorType::Image => container.image.to_owned(),
        SensorType::Status => get_status_payload(container),
    }
}

fn get_status_payload(container: &Container) -> String {
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

fn get_cpu_usage_payload(client: &DockerClient, container: &Container) -> String {
    client.get_stats(container).cpu_usage.to_string()
}
