use bollard::{models::ContainerSummaryInner, Docker};
use std::collections::HashSet;
use tokio::{
    sync::{broadcast, oneshot},
    task,
};
use tracing::error;

use crate::events::{ContainerEvent, Event, EventType};

use super::container;

pub async fn source(
    event_sender: broadcast::Sender<Event>,
    repo_init_receiver: oneshot::Receiver<Vec<String>>,
    client: Docker,
) {
    task::spawn(async move {
        let containers = container::get(&client).await;

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
    let container_name = container::get_name(&container).to_owned();

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
        .map(|c| container::get_name(&c).to_owned())
        .collect();

    repo_init_receiver
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|c| !docker_container_names.contains(c))
        .map(|c| Event {
            container_name: c,
            event: EventType::State(ContainerEvent::Destroy),
        })
        .for_each(|e| send_event(e, &event_sender));
}

#[cfg(test)]
mod must {
    use bollard::models::ContainerSummaryInner;
    use tokio::sync::{broadcast, oneshot};

    use crate::events::{ContainerEvent, Event, EventType};

    use super::handle_orphaned_containers;

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
