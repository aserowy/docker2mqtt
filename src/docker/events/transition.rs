use bollard::{
    errors::Error,
    models::{SystemEventsResponse, SystemEventsResponseActor},
};
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub fn to_events(result: Result<SystemEventsResponse, Error>) -> Option<Vec<Event>> {
    let response: SystemEventsResponse;
    match result {
        Ok(rspns) => response = rspns,
        Err(error) => {
            error!("could not resolve event from stream: {}", error);
            return None;
        }
    }

    let state_event = get_state_event(&response);
    let mut messages = vec![];

    match &state_event.event {
        EventType::State(ContainerEvent::Undefined) => return None,
        EventType::State(ContainerEvent::Create) => messages.push(get_image_event(&response)),
        _ => {}
    }

    messages.push(state_event);

    Some(messages)
}

fn get_state_event(response: &SystemEventsResponse) -> Event {
    Event {
        container_name: get_attribute(&response.actor, "name"),
        event: EventType::State(get_container_event(&response.action)),
    }
}

fn get_image_event(response: &SystemEventsResponse) -> Event {
    Event {
        container_name: get_attribute(&response.actor, "name"),
        event: EventType::Image(get_attribute(&response.actor, "image")),
    }
}

fn get_attribute(actor: &Option<SystemEventsResponseActor>, attribute: &str) -> String {
    let mut result = "".to_owned();
    if let Some(some_actor) = actor {
        if let Some(attributes) = &some_actor.attributes {
            if let Some(name) = attributes.get(attribute) {
                result = name.to_owned()
            }
        }
    }

    result
}

fn get_container_event(action: &Option<String>) -> ContainerEvent {
    match action.as_deref() {
        Some("create") => ContainerEvent::Create,
        Some("destroy") => ContainerEvent::Destroy,
        Some("die") => ContainerEvent::Die,
        Some("kill") => ContainerEvent::Kill,
        Some("pause") => ContainerEvent::Pause,
        Some("rename") => ContainerEvent::Rename,
        Some("restart") => ContainerEvent::Restart,
        Some("start") => ContainerEvent::Start,
        Some("stop") => ContainerEvent::Stop,
        Some("unpause") => ContainerEvent::Unpause,
        Some("prune") => ContainerEvent::Prune,
        Some(_) => ContainerEvent::Undefined,
        None => ContainerEvent::Undefined,
    }
}
