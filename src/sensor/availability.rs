use tracing::instrument;

use crate::docker::Container;

use super::{Availability, SensorType};

#[instrument(level = "debug")]
pub fn get_availability(container: &Container, sensor: &SensorType) -> Availability {
    match sensor {
        SensorType::Image => Availability::Online,
        SensorType::Status => Availability::Online,

        SensorType::CpuUsage => container_availability(container),
        SensorType::MemoryUsage => container_availability(container),
    }
}

fn container_availability(container: &Container) -> Availability {
    if container.status.starts_with("Up") {
        Availability::Online
    } else {
        Availability::Offline
    }
}
