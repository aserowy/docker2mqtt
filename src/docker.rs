use std::{collections::HashMap, fmt};

use bollard::{system::EventsOptions, Docker};
use tokio_stream::{Stream, StreamExt};
use tracing::{error, instrument};

#[derive(Debug)]
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    #[instrument(level = "debug")]
    pub fn new() -> DockerClient {
        match Docker::connect_with_unix_defaults() {
            Ok(client) => DockerClient { client },
            Err(e) => {
                error!("failed to create docker client: {}", e);
                panic!();
            }
        }
    }

    pub fn get_event_stream(&self) -> impl Stream<Item = Event> {
        let filter = Some(EventsOptions::<String> {
            since: None,
            until: None,
            filters: HashMap::new(),
        });

        self.client.events(filter).filter_map(|rslt| match rslt {
            Ok(rspns) => Some(Event {
                event_type: EventType::Status,
                container_id: resolve_container_id(&rspns.actor),
                availability: Availability::Online,
                payload: rspns.action,
            }),
            Err(error) => {
                error!("could not resolve event from stream: {}", error);
                None
            }
        })
    }
}

fn resolve_container_id(actor: &Option<bollard::models::SystemEventsResponseActor>) -> String {
    let mut container_id = "".to_owned();
    if let Some(some_actor) = actor {
        match &some_actor.id {
            Some(id) => container_id = id.to_owned(),
            None => {}
        }
    }

    container_id
}

#[derive(Debug)]
pub enum Availability {
    Online,
    Offline,
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum EventType {
    CpuUsage,
    Image,
    MemoryUsage,
    Status,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Event {
    pub event_type: EventType,
    pub container_id: String,
    pub availability: Availability,
    pub payload: Option<String>,
}
