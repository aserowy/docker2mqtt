use std::fmt;

use crate::{
    docker::{Container, DockerClient},
    mqtt::message::Message,
    Args,
};

pub mod availability;
pub mod discovery;
pub mod state;
mod topic;

#[derive(Debug)]
pub enum Sensor {
    CpuUsage,
    Image,
    Status,
}

impl fmt::Display for Sensor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type HassioResult<T> = Result<T, HassioErr>;

pub enum HassioErr {
    PrefixNotSet,
}

pub fn get_messages(
    args: &Args,
    docker_client: &mut DockerClient,
    container: &Container,
) -> Vec<Message> {
    let messages = vec![
        // get_sensor_messages(args, docker_client, container, Sensor::CpuUsage),
        get_sensor_messages(args, docker_client, container, Sensor::Image),
        get_sensor_messages(args, docker_client, container, Sensor::Status),
    ];

    messages.into_iter().flat_map(|vector| vector).collect()
}

fn get_sensor_messages(
    args: &Args,
    docker_client: &mut DockerClient,
    container: &Container,
    sensor: Sensor,
) -> Vec<Message> {
    let mut messages = vec![
        Message {
            topic: availability::topic(args, container, &sensor),
            payload: availability::payload(container, &sensor),
        },
        Message {
            topic: state::topic(args, container, &sensor),
            payload: state::payload(docker_client, container, &sensor),
        },
    ];

    match get_discovery_message(args, container, &sensor) {
        Ok(message) => messages.push(message),
        Err(_) => {}
    }

    messages
}

fn get_discovery_message(
    args: &Args,
    container: &Container,
    sensor: &Sensor,
) -> HassioResult<Message> {
    let topic = match discovery::topic(args, container, sensor) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(args, container, sensor) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}
