use crate::{
    docker::{Container, DockerClient},
    Args,
};

use super::{topic, Sensor};

pub fn topic(args: &Args, container: &Container, sensor: &Sensor) -> String {
    topic::state(&args.client_id, &container.name, &sensor.to_string())
}

pub fn payload(docker_client: &mut DockerClient, container: &Container, sensor: &Sensor) -> String {
    match sensor {
        Sensor::CpuUsage => get_cpu_usage_payload(docker_client, container),
        Sensor::Image => container.image.to_owned(),
        Sensor::Status => get_status_payload(container),
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

fn get_cpu_usage_payload(docker_client: &mut DockerClient, container: &Container) -> String {
    docker_client.get_stats(container).cpu_usage.to_string()
}
