use std::collections::HashMap;

use bollard::{
    errors::Error,
    models::{SystemEventsResponse, SystemEventsResponseActor},
    system::EventsOptions,
    Docker,
};
use tokio::{sync::broadcast, task};
use tokio_stream::StreamExt;
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub async fn source(event_sender: broadcast::Sender<Event>, client: Docker) -> () {
    task::spawn(async move {
        let mut query = HashMap::new();
        query.insert("type".to_owned(), vec!["container".to_owned()]);

        let filter = Some(EventsOptions::<String> {
            since: None,
            until: None,
            filters: query,
        });

        let mut stream = client.events(filter).filter_map(|rslt| get_events(rslt));

        while let Some(events) = stream.next().await {
            for event in events.into_iter() {
                match event_sender.send(event) {
                    Ok(_) => {}
                    Err(e) => error!("event could not be send to event_router: {}", e),
                }
            }
        }
    });
}

fn get_events(result: Result<SystemEventsResponse, Error>) -> Option<Vec<Event>> {
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
        &EventType::Status(ContainerEvent::Undefined) => return None,
        &EventType::Status(ContainerEvent::Create) => messages.push(get_image_event(&response)),
        _ => {}
    }

    messages.push(state_event);

    Some(messages)
}

fn get_state_event(response: &SystemEventsResponse) -> Event {
    let container_event = get_container_event(&response.action);

    Event {
        container_name: get_attribute(&response.actor, "name"),
        event: EventType::Status(container_event),
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
            match attributes.get(attribute) {
                Some(name) => result = name.to_owned(),
                None => {}
            }
        }
    }

    result
}

fn get_container_event(action: &Option<String>) -> ContainerEvent {
    match action.as_deref() {
        // Some("attach") => ContainerEvent::Attach,
        // Some("commit") => ContainerEvent::Commit,
        // Some("copy") => ContainerEvent::Copy,
        Some("create") => ContainerEvent::Create,
        Some("destroy") => ContainerEvent::Destroy,
        // Some("detach") => ContainerEvent::Detach,
        Some("die") => ContainerEvent::Die,
        // Some("exec_create") => ContainerEvent::Exec_create,
        // Some("exec_detach") => ContainerEvent::Exec_detach,
        // Some("exec_start") => ContainerEvent::Exec_start,
        // Some("exec_die") => ContainerEvent::Exec_die,
        // Some("export") => ContainerEvent::Export,
        // Some("health_status") => ContainerEvent::Health_status,
        Some("kill") => ContainerEvent::Kill,
        // Some("oom") => ContainerEvent::Oom,
        Some("pause") => ContainerEvent::Pause,
        Some("rename") => ContainerEvent::Rename,
        // Some("resize") => ContainerEvent::Resize,
        Some("restart") => ContainerEvent::Restart,
        Some("start") => ContainerEvent::Start,
        Some("stop") => ContainerEvent::Stop,
        // Some("top") => ContainerEvent::Top,
        Some("unpause") => ContainerEvent::Unpause,
        // Some("update") => ContainerEvent::Update,
        Some("prune") => ContainerEvent::Prune,
        Some(_) => ContainerEvent::Undefined,
        None => ContainerEvent::Undefined,
    }
}
