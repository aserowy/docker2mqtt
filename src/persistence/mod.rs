use crate::events::{ContainerEvent, Event, EventType};
use crate::persistence::docker::DockerDbMessage::{
    AddDockerContainer, DeleteDockerContainer, GetAllDockerContainers,
};
use crate::persistence::docker::{DockerDbHandle, DockerDbMessage};
use tokio::sync::{mpsc, oneshot};

pub mod docker;
pub mod logging;

const DATA_DIRECTORY: &str = "/docker2mqtt/data";

pub async fn init_task(init_sender: oneshot::Sender<Vec<String>>, handle: DockerDbHandle) {
    handle.handle(GetAllDockerContainers {
        respond_to: init_sender,
    }).await
}

pub async fn state_task(mut receiver: mpsc::Receiver<Event>, handle: DockerDbHandle) {
    while let Some(event) = receiver.recv().await {
        dispatch_event(event).map(|m| handle.handle(m));
    }
}

fn dispatch_event(event: Event) -> Option<DockerDbMessage> {
    if let EventType::State(container_event) = event.event {
        match container_event {
            ContainerEvent::Create => Option::Some(AddDockerContainer {
                name: event.container_name,
            }),
            ContainerEvent::Destroy => Option::Some(DeleteDockerContainer {
                name: event.container_name,
            }),
            _ => Option::None,
        }
    } else {
        Option::None
    }
}
