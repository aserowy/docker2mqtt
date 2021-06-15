use tracing::warn;

use crate::{
    configuration::Configuration,
    events::{ContainerEvent, Event, EventType},
    mqtt::Message,
};

use super::content;

pub fn for_create_event(event: &Event, conf: &Configuration) -> Vec<Message> {
    get_sensors()
        .into_iter()
        .map(|sensor| (get_messages_with_topic(event, &sensor, conf), sensor))
        .flat_map(|(message, sensor)| add_payload_to_message(message, event, &sensor, conf))
        .collect()
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

    content::topic(container_name, event_name, conf)
        .map_err(|e| warn!("could not resolve discovery topic: {:?}", e))
        .map_or(None, |topic| {
            Some(Message {
                topic,
                payload: String::new(),
            })
        })
}

fn add_payload_to_message(
    message: Option<Message>,
    event: &Event,
    sensor: &EventType,
    conf: &Configuration,
) -> Option<Message> {
    let container_name = &event.container_name;
    let event_name = &sensor.to_string();

    content::payload(container_name, event_name, conf)
        .map_err(|e| warn!("could not resolve discovery payload: {:?}", e))
        .map_or(None, |payload| {
            message
                .map(|msg| msg.topic)
                .map(|topic| Message { topic, payload })
        })
}
