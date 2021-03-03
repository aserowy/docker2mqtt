use std::{collections::HashMap, fmt};

use bollard::{system::EventsOptions, Docker};
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
    let (event_sender, event_receiver) = broadcast::channel(500);
    let event_receiver_stats = event_sender.subscribe();

    initial_event_source(event_sender.clone(), docker_client.clone(), &conf).await;
    event_source(event_sender.clone(), docker_client.clone()).await;

    stats_source(
        event_receiver_stats,
        event_sender,
        docker_client.clone(),
        &conf,
    )
    .await;

    event_router(event_receiver, sender).await;
}

async fn initial_event_source(
    event_sender: broadcast::Sender<Event>,
    client: Docker,
    conf: &Configuration,
) -> () {
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

        let mut stream = client.events(filter).filter_map(|rslt| match rslt {
            Ok(rspns) => Some(Event {
                availability: Availability::Online,
                container_name: resolve_container_name(&rspns.actor),
                event: EventType::Status(resolver_container_event(rspns.action)),
            }),
            Err(error) => {
                error!("could not resolve event from stream: {}", error);
                None
            }
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
    todo!()
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
    Attach,
    Commit,
    Copy,
    Create,
    Destroy,
    Detach,
    Die,
    Exec_create,
    Exec_detach,
    Exec_start,
    Exec_die,
    Export,
    Health_status,
    Kill,
    Oom,
    Pause,
    Rename,
    Resize,
    Restart,
    Start,
    Stop,
    Top,
    Unpause,
    Update,
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
