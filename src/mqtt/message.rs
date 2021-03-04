use tracing::debug;

use crate::{
    configuration::Configuration,
    docker::{ContainerEvent, Event, EventType},
};

use super::{
    availability,
    discovery::{self, HassioResult},
    payload, topic,
};

#[derive(Debug)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub fn get_event_messages(event: Event, conf: &Configuration) -> Vec<Message> {
    let mut messages = vec![];

    if let EventType::Status(ContainerEvent::Create) = &event.event {
        match get_discovery_message(&event, conf) {
            Ok(message) => messages.push(message),
            Err(e) => debug!("discovery messages not generated: {:?}", e),
        }
    }

    if let EventType::Status(container_event) = &event.event {
        messages.push(Message {
            topic: topic::availability(&event, conf),
            payload: availability::get_availability(container_event).to_string(),
        });
    }

    messages.push(Message {
        topic: topic::state(&event, conf),
        payload: payload::get(&event),
    });

    messages
}

fn get_discovery_message(event: &Event, conf: &Configuration) -> HassioResult<Message> {
    let topic = match discovery::topic(event, conf) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(event, conf) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}
