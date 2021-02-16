use enum_iterator::IntoEnumIterator;
use std::borrow::Borrow;

use crate::{
    docker::{Container, DockerClient},
    sensor::Sensor,
    Args,
};

mod discovery;
mod lwt;
mod state;
mod topic;

pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub fn get_messages(
    client: &DockerClient,
    containers: Vec<Container>,
    args: &Args,
) -> Vec<Message> {
    containers
        .iter()
        .flat_map(|container| get_messages_per_container(&client, container, &args))
        .collect()
}

fn get_messages_per_container(
    client: &DockerClient,
    container: &Container,
    args: &Args,
) -> Vec<Message> {
    let mut messages = Vec::new();
    messages.push(Message {
        topic: lwt::get_availability_topic(&args.client_id, container),
        payload: lwt::get_lwt_payload(container),
    });

    for sensor in Sensor::into_enum_iter() {
        match args.hass_discovery_prefix.borrow() {
            Some(hass_discovery_prefix) => messages.push(Message {
                topic: discovery::get_discovery_topic(
                    &hass_discovery_prefix,
                    &args.client_id,
                    container,
                    &sensor,
                ),
                payload: discovery::get_discovery_payload(&args.client_id, container, &sensor),
            }),
            None => (),
        }

        messages.push(Message {
            topic: state::get_state_topic(&args.client_id, container, &sensor),
            payload: state::get_state_payload(client, container, &sensor),
        });
    }

    messages
}
