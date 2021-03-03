use std::fmt;

use tokio::{
    sync::{broadcast, mpsc},
    task,
};
use tracing::error;

mod client;
mod events;
mod initial;
mod stats;

pub async fn spin_up(sender: mpsc::Sender<Event>) {
    let docker_client = client::new();
    let (event_sender, event_receiver_router) = broadcast::channel(500);
    let event_receiver_stats = event_sender.subscribe();

    initial::source(event_sender.clone(), docker_client.clone()).await;
    events::source(event_sender.clone(), docker_client.clone()).await;

    stats::source(event_receiver_stats, event_sender, docker_client.clone()).await;

    event_router(event_receiver_router, sender).await;
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
    CpuUsage(Option<f64>),
    Image(String),
    MemoryUsage(Option<f64>),
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
