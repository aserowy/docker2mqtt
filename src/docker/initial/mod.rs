use bollard::Docker;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::events::{ContainerEvent, Event, EventType};

use super::container;

mod events;

struct InitActor {
    sender: mpsc::Sender<Event>,
    client: Docker,
}

impl InitActor {
    fn with(sender: mpsc::Sender<Event>, client: Docker) -> Self {
        InitActor { sender, client }
    }

    async fn handle(&mut self, container_names: Vec<String>) {
        let containers = container::get(&self.client).await;
        let mut events = vec![];

        container_names
            .iter()
            .filter_map(events::from_orphaned(&containers))
            .for_each(|event| {
                events.push(event);
            });

        containers
            .into_iter()
            .flat_map(events::from_container)
            .for_each(|event| {
                events.push(event);
            });

        for event in events.into_iter() {
            self.send(event).await;
        }
    }

    async fn send(&mut self, event: Event) {
        if let EventType::State(ContainerEvent::Undefined) = &event.event {
            return;
        }

        if let Err(e) = self.sender.send(event).await {
            error!("event could not be send to event_router: {}", e);
        }
    }

    async fn run(mut self, container_names: Vec<String>) {
        self.handle(container_names).await;
    }
}

#[derive(Debug)]
pub struct InitReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl InitReactor {
    pub async fn with(startup: oneshot::Receiver<Vec<String>>, client: Docker) -> Self {
        let (sender, receiver) = mpsc::channel(50);

        tokio::spawn(async move {
            let actor = InitActor::with(sender, client);
            let container_names = startup.await.unwrap_or_default();

            actor.run(container_names).await;
        });

        InitReactor { receiver }
    }
}
