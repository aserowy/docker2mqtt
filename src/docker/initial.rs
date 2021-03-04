use bollard::{container::ListContainersOptions, models::ContainerSummaryInner, Docker};
use tokio::{sync::broadcast, task};
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub async fn source(event_sender: broadcast::Sender<Event>, client: Docker) -> () {
    task::spawn(async move {
        let filter = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = match client.list_containers(filter).await {
            Ok(containers) => containers,
            Err(e) => {
                error!("could not resolve containers: {}", e);
                vec![]
            }
        };

        for container in containers {
            let container_name = get_container_name(&container).to_owned();

            let mut events = vec![
                Event {
                    container_name: container_name.to_owned(),
                    event: EventType::State(ContainerEvent::Create),
                },
                Event {
                    container_name: container_name.to_owned(),
                    event: EventType::State(get_state(&container)),
                },
            ];

            if let Some(image) = &container.image {
                events.push(Event {
                    container_name: container_name.to_owned(),
                    event: EventType::Image(image.to_owned()),
                });
            }

            for event in events.into_iter() {
                if let &EventType::State(ContainerEvent::Undefined) = &event.event {
                    continue;
                }

                // TODO refactor into function with retry and warning on count > 1
                match event_sender.send(event) {
                    Ok(_) => {}
                    Err(e) => error!("event could not be send to event_router: {}", e),
                }
            }
        }
    });
}

fn get_state(container: &ContainerSummaryInner) -> ContainerEvent {
    match container.state.as_deref() {
        Some("created") => ContainerEvent::Create,
        Some("restarting") => ContainerEvent::Restart,
        Some("running") => ContainerEvent::Start,
        Some("removing") => ContainerEvent::Prune,
        Some("paused") => ContainerEvent::Pause,
        Some("exited") => ContainerEvent::Stop,
        Some("dead") => ContainerEvent::Die,
        Some(_) => ContainerEvent::Undefined,
        None => ContainerEvent::Undefined,
    }
}

fn get_container_name(container: &ContainerSummaryInner) -> &str {
    let container_names = match &container.names {
        Some(names) => names,
        None => return "",
    };

    let container_name = &container_names[0];
    let (first_char, remainder) = split_first_char_remainder(container_name);

    match first_char {
        "/" => remainder,
        _ => container_name,
    }
}

fn split_first_char_remainder(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}
