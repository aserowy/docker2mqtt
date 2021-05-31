use tracing::warn;

use crate::{configuration::Configuration, events::{ContainerEvent, Event, EventType}, mqtt::Message};

pub fn for_create_event(event: &Event, conf: &Configuration) -> Vec<Message> {
    let mut result = vec![];
    for sensor in get_sensors().into_iter() {
        if let Some(message) = get_messages_with_topic(event, &sensor, conf) {
            if let Some(with_payload) = add_payload_to_message(message, event, &sensor, conf) {
                result.push(with_payload);
            }
        }
    }

    result
}

pub fn for_destroy_event(event: &Event, conf: &Configuration) -> Vec<Message> {
    get_sensors()
        .iter()
        .filter_map(|sensor| get_messages_with_topic(event, sensor, conf))
        .collect()
}

fn get_sensors() -> Vec<EventType> {
    vec![
        EventType::CpuUsage(0.0),
        EventType::Image("".to_owned()),
        EventType::Log("".to_owned()),
        EventType::MemoryUsage(0.0),
        EventType::State(ContainerEvent::Create),
    ]
}

fn get_messages_with_topic(
    event: &Event,
    sensor: &EventType,
    conf: &Configuration,
) -> Option<Message> {
    let container_name = &event.container_name;
    let event_name = &sensor.to_string();

    let topic = match super::content::topic(container_name, event_name, conf) {
        Ok(topic) => topic,
        Err(e) => {
            warn!("could not resolve discovery topic: {:?}", e);
            return None;
        }
    };

    Some(Message {
        topic,
        payload: String::new(),
    })
}

fn add_payload_to_message(
    message: Message,
    event: &Event,
    sensor: &EventType,
    conf: &Configuration,
) -> Option<Message> {
    let container_name = &event.container_name;
    let event_name = &sensor.to_string();

    let payload = match super::content::payload(container_name, event_name, conf) {
        Ok(payload) => payload,
        Err(e) => {
            warn!("could not resolve discovery payload: {:?}", e);
            return None;
        }
    };

    Some(Message {
        topic: message.topic,
        payload,
    })
}
