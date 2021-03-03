use std::collections::HashMap;

use bollard::{models::SystemEventsResponseActor, system::EventsOptions, Docker};
use tokio::{sync::broadcast, task};
use tokio_stream::StreamExt;
use tracing::error;

use super::{Availability, ContainerEvent, Event, EventType};

pub async fn source(event_sender: broadcast::Sender<Event>, client: Docker) -> () {
    task::spawn(async move {
        let mut query = HashMap::new();
        query.insert("type".to_owned(), vec!["container".to_owned()]);

        let filter = Some(EventsOptions::<String> {
            since: None,
            until: None,
            filters: query,
        });

        let mut stream = client
            .events(filter)
            .filter_map(|rslt| match rslt {
                Ok(rspns) => Some(Event {
                    availability: Availability::Online,
                    container_name: get_container_name(&rspns.actor),
                    event: EventType::Status(get_container_event(rspns.action)),
                }),
                Err(error) => {
                    error!("could not resolve event from stream: {}", error);
                    None
                }
            })
            .filter(|evnt| match evnt.event {
                EventType::Status(ContainerEvent::Undefined) => false,
                EventType::Status(_) => true,
                _ => true,
            });

        while let Some(event) = stream.next().await {
            match event_sender.send(event) {
                Ok(_) => {}
                Err(e) => error!("event could not be send to event_router: {}", e),
            }
        }
    });
}

fn get_container_name(actor: &Option<SystemEventsResponseActor>) -> String {
    let mut container_name = "".to_owned();
    if let Some(some_actor) = actor {
        if let Some(attributes) = &some_actor.attributes {
            match attributes.get("name") {
                Some(name) => container_name = name.to_owned(),
                None => {}
            }
        }
    }

    container_name
}

fn get_container_event(action: Option<String>) -> ContainerEvent {
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
