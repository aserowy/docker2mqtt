use std::{collections::HashMap, fmt};

use bollard::{
    container::ListContainersOptions, models::ContainerSummaryInner, system::EventsOptions, Docker,
};
use tokio::{
    sync::{broadcast, mpsc},
    task,
};
use tokio_stream::StreamExt;
use tracing::error;

use crate::configuration::Configuration;

mod client;

pub async fn spin_up(sender: mpsc::Sender<Event>, conf: Configuration) {
    let docker_client = client::new();
    let (event_sender, event_receiver_router) = broadcast::channel(500);
    let event_receiver_stats = event_sender.subscribe();

    initial_event_source(event_sender.clone(), docker_client.clone()).await;
    event_source(event_sender.clone(), docker_client.clone()).await;

    stats_source(
        event_receiver_stats,
        event_sender,
        docker_client.clone(),
        &conf,
    )
    .await;

    event_router(event_receiver_router, sender).await;
}

async fn initial_event_source(event_sender: broadcast::Sender<Event>, client: Docker) -> () {
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
            let event = Event {
                availability: Availability::Online,
                container_name: get_container_name(&container).to_owned(),
                event: EventType::Status(ContainerEvent::Create),
            };

            match event_sender.send(event) {
                Ok(_) => {}
                Err(e) => error!("event could not be send to event_router: {}", e),
            }
        }
    });
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

async fn event_source(event_sender: broadcast::Sender<Event>, client: Docker) -> () {
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
                    container_name: resolve_container_name(&rspns.actor),
                    event: EventType::Status(resolver_container_event(rspns.action)),
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

fn resolve_container_name(actor: &Option<bollard::models::SystemEventsResponseActor>) -> String {
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

fn resolver_container_event(action: Option<String>) -> ContainerEvent {
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

async fn stats_source(
    event_receiver: broadcast::Receiver<Event>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
    conf: &Configuration,
) -> () {
}

async fn event_router(
    mut event_receiver: broadcast::Receiver<Event>,
    sender: mpsc::Sender<Event>,
) -> () {
    task::spawn(async move {
        // TODO handle faulted receive
        while let Ok(event) = event_receiver.recv().await {
            match sender.send(event).await {
                Ok(_) => {}
                Err(e) => error!("event could not be send to mqtt client: {}", e),
            }
        }
    });
}

#[derive(Clone, Debug)]
pub struct Event {
    pub availability: Availability,
    pub container_name: String,
    pub event: EventType,
}

#[derive(Clone, Debug)]
pub enum EventType {
    CpuUsage,
    Image,
    MemoryUsage,
    Status(ContainerEvent),
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum ContainerEvent {
    Undefined,

    // Attach,
    // Commit,
    // Copy,
    Create,
    Destroy,
    // Detach,
    Die,
    // Exec_create,
    // Exec_detach,
    // Exec_start,
    // Exec_die,
    // Export,
    // Health_status,
    Kill,
    // Oom,
    Pause,
    Rename,
    // Resize,
    Restart,
    Start,
    Stop,
    // Top,
    Unpause,
    // Update,
    Prune,
}

#[derive(Clone, Debug)]
pub enum Availability {
    Online,
    Offline,
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
