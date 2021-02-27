use tracing::instrument;

use crate::docker::{Container, DockerClient};

use super::SensorType;

#[instrument(level = "debug")]
pub fn get_state(client: &DockerClient, container: &Container, sensor: &SensorType) -> String {
    match sensor {
        SensorType::CpuUsage => get_cpu_usage_payload(client, container),
        SensorType::Image => container.image.to_owned(),
        SensorType::MemoryUsage => get_memory_usage_payload(client, container),
        SensorType::Status => get_status_payload(container),
    }
}

fn get_cpu_usage_payload(client: &DockerClient, container: &Container) -> String {
    format!("{:.2}", client.get_stats(container).cpu_usage)
}

fn get_memory_usage_payload(client: &DockerClient, container: &Container) -> String {
    format!("{:.2}", client.get_stats(container).memory_usage)
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
