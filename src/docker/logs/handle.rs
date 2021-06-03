use std::collections::HashMap;

use bollard::Docker;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    configuration::Configuration,
    events::{ContainerEvent, Event, EventType},
};

use super::{stream, validate};

pub async fn event(
    event: Event,
    tasks: &mut HashMap<String, JoinHandle<()>>,
    client: &Docker,
    event_sender: &mpsc::Sender<Event>,
    conf: &Configuration,
) {
    match &event.event {
        EventType::State(ContainerEvent::Start) => {
            if !validate::target(&event, client, conf).await {
                return;
            }

            tasks.insert(
                event.container_name.to_owned(),
                stream::start(client.clone(), event.clone(), event_sender.clone()).await,
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
