use bollard::{container::ListContainersOptions, models::ContainerSummaryInner, Docker};
use std::collections::HashSet;
use tokio::{
    sync::{broadcast, oneshot},
    task,
};
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub async fn source(
    event_sender: broadcast::Sender<Event>,
    repo_init_receiver: oneshot::Receiver<Vec<String>>,
    client: Docker,
) {
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

        handle_orphaned_containers(&event_sender, repo_init_receiver, &containers).await;

        containers
            .into_iter()
            .flat_map(get_events_by_container)
            .for_each(|event| {
                send_event(event, &event_sender);
            });
    });
}

fn get_events_by_container(container: ContainerSummaryInner) -> Vec<Event> {
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
            container_name,
            event: EventType::Image(image.to_owned()),
        });
    }

    events
}

fn send_event(event: Event, event_sender: &broadcast::Sender<Event>) {
    if let EventType::State(ContainerEvent::Undefined) = &event.event {
        return;
    }

    // TODO refactor send message function with retry and warning if ok > 1
    match event_sender.send(event) {
        Ok(_) => {}
        Err(e) => error!("event could not be send to event_router: {}", e),
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

async fn handle_orphaned_containers(
    event_sender: &broadcast::Sender<Event>,
    repo_init_receiver: oneshot::Receiver<Vec<String>>,
    containers: &[ContainerSummaryInner],
) {
    let docker_container_names: HashSet<String> = containers
        .iter()
        .map(|c| get_container_name(&c).to_owned())
        .collect();

    repo_init_receiver
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !docker_container_names.contains(c))
        .map(|c| Event {
            container_name: c,
            event: EventType::State(ContainerEvent::Prune),
        })
        .for_each(|e| send_event(e, &event_sender));
}

#[cfg(test)]
mod must {
    use super::handle_orphaned_containers;
    use crate::docker::{ContainerEvent, Event, EventType};
    use bollard::models::ContainerSummaryInner;
    use tokio::sync::{broadcast, oneshot};

    fn create_container_summary(name: String) -> ContainerSummaryInner {
        ContainerSummaryInner {
            id: None,
            names: Some(vec![name]),
            image: None,
            image_id: None,
            command: None,
            created: None,
            ports: None,
            size_rw: None,
            size_root_fs: None,
            labels: None,
            state: None,
            status: None,
            host_config: None,
            network_settings: None,
            mounts: None,
        }
    }

    #[tokio::test]
    async fn return_correct_remove_events_for_orphaned_containers() {
        let (repo_init_sender, repo_init_receiver) = oneshot::channel();
        let (mqtt_sender, mut mqtt_receiver) = broadcast::channel(100);

        let container_names: Vec<ContainerSummaryInner> = vec!["first", "second"]
            .into_iter()
            .map(|c| create_container_summary(c.to_owned()))
            .collect();

        if let Err(e) = repo_init_sender.send(vec![String::from("second"), String::from("third")]) {
            panic!("error in test: {:?}", e)
        }

        handle_orphaned_containers(&mqtt_sender, repo_init_receiver, &container_names).await;

        let expected = Event {
            container_name: "third".to_owned(),
            event: EventType::State(ContainerEvent::Prune),
        };
        assert_eq!(expected, mqtt_receiver.recv().await.unwrap());
    }
}
