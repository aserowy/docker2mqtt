use crate::docker::Container;

use super::{Availability, SensorType};

pub fn get_availability(container: &Container, sensor: &SensorType) -> Availability {
    match sensor {
        SensorType::Image => Availability::Online,
        SensorType::Status => Availability::Online,
        _ => container_availability(container),
    }
}

fn container_availability(container: &Container) -> Availability {
    if container.status.starts_with("Up") {
        Availability::Online
    } else {
        Availability::Offline
    }
}
