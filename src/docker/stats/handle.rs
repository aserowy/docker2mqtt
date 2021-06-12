use std::collections::HashMap;

use bollard::Docker;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::events::{ContainerEvent, Event, EventType};

use super::stream;

pub async fn event(
    event: Event,
    tasks: &mut HashMap<String, JoinHandle<()>>,
    client: &Docker,
    sender: &mpsc::Sender<Event>,
) {
    match &event.event {
        EventType::State(ContainerEvent::Start) => {
            tasks.insert(
                event.container_name.to_owned(),
                stream::start_stats_stream(client.clone(), event.clone(), sender.clone()).await,
            );
        }
        EventType::State(ContainerEvent::Stop) => {
            if let Some(handle) = tasks.remove(&event.container_name) {
                handle.abort();
            }
        }
        EventType::State(ContainerEvent::Die) => {
            if let Some(handle) = tasks.remove(&event.container_name) {
                handle.abort();
            }
        }
        _ => {}
    }
}
