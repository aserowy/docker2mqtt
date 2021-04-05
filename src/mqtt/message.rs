use tracing::warn;

use crate::configuration::Configuration;
use crate::events::{ContainerEvent, Event, EventType};

use super::{availability, discovery, payload, topic};

#[derive(Debug)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub fn get_event_messages(event: Event, conf: &Configuration) -> Vec<Message> {
    let mut messages = vec![];

    for message in get_discovery(&event, conf) {
        messages.push(message)
    }

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

fn get_discovery(event: &Event, conf: &Configuration) -> Vec<Message> {
    let sensors = vec![
        EventType::CpuUsage(0.0),
        EventType::Image("".to_owned()),
        EventType::Log("".to_owned()),
        EventType::MemoryUsage(0.0),
        EventType::State(ContainerEvent::Create),
    ];

    let mut result = vec![];

    sensors
        .iter()
        .filter_map(|sensor| get_discovery_message(event, sensor, conf))
        .for_each(|message| result.push(message));

    result
}

fn get_discovery_message(
    event: &Event,
    sensor: &EventType,
    conf: &Configuration,
) -> Option<Message> {
    let container_name = &event.container_name;
    let event_name = &sensor.to_string();

    let topic = match discovery::topic(container_name, event_name, conf) {
        Ok(topic) => topic,
        Err(e) => {
            warn!("could not resolve discovery topic: {:?}", e);
            return None;
        }
    };

    let payload = match discovery::payload(container_name, event_name, conf) {
        Ok(payload) => payload,
        Err(e) => {
            warn!("could not resolve discovery topic: {:?}", e);
            return None;
        }
    };

    match &event.event {
        EventType::State(ContainerEvent::Create) => Some(Message { topic, payload }),
        EventType::State(ContainerEvent::Destroy) => Some(Message {
            topic,
            payload: "".to_owned(),
        }),
        _ => None,
    }
}
