use tokio::sync::mpsc;
use tracing::error;

use crate::{events::Event, Configuration};

use super::Message;

mod message;
mod payload;

struct MappingActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Message>,
    conf: Configuration,
}

impl MappingActor {
    fn new(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Message>,
        conf: Configuration,
    ) -> Self {
        MappingActor {
            receiver,
            sender,
            conf,
        }
    }

    async fn handle(&mut self, event: Event) {
        for message in message::map(event, &self.conf).into_iter() {
            if let Err(e) = self.sender.send(message).await {
                error!("message was not sent: {}", e);
            }
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

#[derive(Debug)]
pub struct MappingReactor {
    pub receiver: mpsc::Receiver<Message>,
}

impl MappingReactor {
    pub async fn new(receiver: mpsc::Receiver<Event>, conf: &Configuration) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = MappingActor::new(receiver, sender, conf.clone());

        tokio::spawn(actor.run());

        MappingReactor {
            receiver: actor_receiver,
        }
    }
}
