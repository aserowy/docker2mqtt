use crate::{
    configuration::Configuration,
    events::{Event, EventType},
};

use super::{availability, payload, topic};

#[derive(Debug)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub fn get_event_messages(event: Event, conf: &Configuration) -> Vec<Message> {
    let mut messages = vec![];
    if let EventType::State(container_event) = &event.event {
        messages.push(Message {
            topic: topic::availability(&event.container_name, conf),
            payload: availability::get_availability(container_event).to_string(),
        });
    }

    // TODO availability for sensors only between start->stop

    messages.push(Message {
        topic: topic::state(&event.container_name, &event.event.to_string(), conf),
        payload: payload::get(&event),
    });

    messages
}

