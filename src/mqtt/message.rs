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

    if let EventType::State(ContainerEvent::Create) = &event.event {
        for message in get_discovery_messages(&event, conf) {
            messages.push(message)
        }
    }

    if let EventType::State(container_event) = &event.event {
        messages.push(Message {
            topic: topic::availability(&event.container_name, conf),
            payload: availability::get_availability(container_event).to_string(),
        });
    }

    // TODO availability for sensors only between start->stop
    // TODO clean up on destroy

    messages.push(Message {
        topic: topic::state(&event.container_name, &event.event.to_string(), conf),
        payload: payload::get(&event),
    });

    messages
}

fn get_discovery_messages(event: &Event, conf: &Configuration) -> Vec<Message> {
    let sensors = vec![
        EventType::CpuUsage(0.0),
        EventType::Image("".to_owned()),
        EventType::MemoryUsage(0.0),
        EventType::State(ContainerEvent::Create),
    ];

    let container_name = &event.container_name;
    let mut result = vec![];
    for sensor in sensors {
        if let Ok(message) = get_discovery_message(container_name, &sensor.to_string(), conf) {
            result.push(message);
        }
    }

    result
}

fn get_discovery_message(
    container_name: &str,
    event_name: &str,
    conf: &Configuration,
) -> HassioResult<Message> {
    let topic = match discovery::topic(container_name, event_name, conf) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(container_name, event_name, conf) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}
