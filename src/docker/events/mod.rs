use bollard::models::SystemEventsResponse;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::{
    docker::client::DockerMessage,
    events::{ContainerEvent, Event, EventType},
};

use super::client::DockerHandle;

mod transition;

struct EventActor {
    sender: mpsc::Sender<Event>,
    client: DockerHandle,
}

impl EventActor {
    fn new(sender: mpsc::Sender<Event>, client: DockerHandle) -> Self {
        Self { sender, client }
    }

    async fn handle(&mut self, response: SystemEventsResponse) {
        if let Some(events) = transition::to_events(response) {
            for event in events.into_iter() {
                if let Err(e) = self.sender.send(event).await {
                    error!("message was not sent: {}", e);
                }
            }
        }
    }

    async fn run(mut self) {
        let (response, receiver) = oneshot::channel();
        let message = DockerMessage::GetEventStream { response };

        self.client.handle(message).await;
        match receiver.await {
            Ok(mut stream) => {
                while let Some(result) = stream.recv().await {
                    self.handle(result).await;
                }
            }
            Err(e) => error!("failed receiving response for get log stream: {}", e),
        }
    }
}

#[derive(Debug)]
pub struct EventReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl EventReactor {
    pub async fn new(client: DockerHandle) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = EventActor::new(sender, client);

        tokio::spawn(actor.run());

        Self { receiver }
    }
}
