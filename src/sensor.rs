use std::fmt;
use tracing::instrument;

use crate::docker::{Container, DockerClient};

mod availability;
mod state;

#[derive(Debug)]
pub struct Sensor<'a> {
    pub sensor_type: &'a SensorType,
    pub container: &'a Container,
    pub availability: Availability,
    pub state: String,
}

#[derive(Debug)]
pub enum Availability {
    Online,
    Offline,
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum SensorType {
    CpuUsage,
    Image,
    MemoryUsage,
    Status,
}

impl fmt::Display for SensorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[instrument(level = "debug")]
pub fn get_sensors<'a>(client: &'a DockerClient, container: &'a Container) -> Vec<Sensor<'a>> {
    vec![
        get_sensor(client, container, &SensorType::CpuUsage),
        get_sensor(client, container, &SensorType::Image),
        get_sensor(client, container, &SensorType::MemoryUsage),
        get_sensor(client, container, &SensorType::Status),
    ]
}

fn get_sensor<'a>(
    client: &'a DockerClient,
    container: &'a Container,
    sensor_type: &'a SensorType,
) -> Sensor<'a> {
    Sensor {
        sensor_type,
        container,
        availability: availability::get_availability(container, sensor_type),
        state: state::get_state(client, container, sensor_type),
    }
}
