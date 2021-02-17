use crate::{docker::Container, Args};

use super::{topic, Sensor};

pub fn topic(args: &Args, container: &Container, sensor: &Sensor) -> String {
    match sensor {
        Sensor::CpuUsage => topic::device_availability(&args.client_id, &container.name),
        _ => topic::sensor_availibility(&args.client_id, &container.name, &sensor.to_string()),
    }
}

pub fn payload(container: &Container, sensor: &Sensor) -> String {
    match sensor {
        Sensor::CpuUsage => container_availability(container),
        _ => "online".to_owned(),
    }
}

fn container_availability(container: &Container) -> String {
    if container.status.starts_with("Up") {
        "online".to_owned()
    } else {
        "offline".to_owned()
    }
}
