use tokio::sync::mpsc;
use tracing::error;

use crate::{
    events::{ContainerEvent, Event, EventType},
    Configuration,
};

use super::message::Message;

mod content;
mod message;
mod payload;

struct HassioActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Message>,
    conf: Configuration,
}

impl HassioActor {
    async fn with(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Message>,
        conf: Configuration,
    ) -> Self {
        HassioActor {
            receiver,
            sender,
            conf,
        }
    }

    async fn handle(&mut self, event: Event) {
        match &event.event {
            EventType::State(ContainerEvent::Create) => {
                self.send(message::for_create_event(&event, &self.conf))
                    .await
            }
            EventType::State(ContainerEvent::Destroy) => {
                self.send(message::for_destroy_event(&event, &self.conf))
                    .await
            }
            _ => {}
        }
    }

    async fn send(&mut self, messages: Vec<Message>) {
        for message in messages.into_iter() {
            if let Err(e) = self.sender.send(message).await {
                error!("message was not sent: {}", e);
            }
        }
    }
}

async fn run_actor(mut actor: HassioActor) {
    while let Some(message) = actor.receiver.recv().await {
        actor.handle(message).await;
    }
}

#[derive(Debug)]
pub struct HassioReactor {
    receiver: mpsc::Receiver<Message>,
}

impl HassioReactor {
    pub async fn with(receiver: mpsc::Receiver<Event>, conf: &Configuration) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = HassioActor::with(receiver, sender, conf.clone()).await;

        tokio::spawn(run_actor(actor));

        HassioReactor {
            receiver: actor_receiver,
        }
    }
}
