use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    docker::client::DockerHandle,
    events::{ContainerEvent, Event, EventType},
};

use super::stream;

pub async fn event(
    event: Event,
    tasks: &mut HashMap<String, JoinHandle<()>>,
    client: &DockerHandle,
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
            tasks
                .remove(&event.container_name)
                .and_then(|handle| Some(handle.abort()));
        }
        EventType::State(ContainerEvent::Die) => {
            tasks
                .remove(&event.container_name)
                .and_then(|handle| Some(handle.abort()));
        }
        _ => {}
    }
}
