use std::fmt;

use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        mpsc,
        oneshot
    },
    task,
};
use tracing::error;
use crate::persistence;

mod client;
mod events;
mod initial;
mod stats;

pub async fn spin_up(
    mqtt_sender: mpsc::Sender<Event>,
    repo_receiver: oneshot::Receiver<Option<Vec<String>>>,
    repo_sender: mpsc::Sender<persistence::Event>
) {
    let docker_client = client::new();
    let (event_sender, event_receiver_router) = broadcast::channel(500);
    let event_receiver_stats = event_sender.subscribe();

    initial::source(event_sender.clone(), docker_client.clone()).await;
    events::source(event_sender.clone(), docker_client.clone()).await;
    stats::source(event_receiver_stats, event_sender, docker_client.clone()).await;

    event_router(event_receiver_router, mqtt_sender).await;
}

async fn event_router(mut event_receiver: broadcast::Receiver<Event>, sender: mpsc::Sender<Event>) {
    task::spawn(async move {
        loop {
            let receive = event_receiver.recv().await;
            let event: Event;
            match receive {
                Ok(evnt) => event = evnt,
                Err(RecvError::Closed) => break,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }

            match sender.send(event).await {
                Ok(_) => {}
                Err(e) => error!("event could not be send to mqtt client: {}", e),
            }
        }
    });
}

#[derive(Clone, Debug)]
pub struct Event {
    pub container_name: String,
    pub event: EventType,
}

#[derive(Clone, Debug)]
pub enum EventType {
    CpuUsage(f64),
    Image(String),
    MemoryUsage(f64),
    State(ContainerEvent),
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            EventType::CpuUsage(_) => "cpu_usage",
            EventType::Image(_) => "image",
            EventType::MemoryUsage(_) => "memory_usage",
            EventType::State(_) => "state",
        };

        write!(formatter, "{}", value)
    }
}

#[derive(Clone, Debug)]
pub enum ContainerEvent {
    Undefined,

    Create,
    Destroy,
    Die,
    Kill,
    Pause,
    Rename,
    Restart,
    Start,
    Stop,
    Unpause,
    Prune,
}
