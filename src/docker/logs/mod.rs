use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{events::Event, Configuration};

use super::client::DockerHandle;

mod handle;
mod stream;
mod validate;

struct LoggingActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tasks: HashMap<String, JoinHandle<()>>,
    client: DockerHandle,
    conf: Configuration,
}

impl LoggingActor {
    fn new(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Event>,
        tasks: HashMap<String, JoinHandle<()>>,
        client: DockerHandle,
        conf: Configuration,
    ) -> Self {
        LoggingActor {
            receiver,
            sender,
            tasks,
            client,
            conf,
        }
    }

    async fn handle(&mut self, event: Event) {
        handle::event(
            event,
            &mut self.tasks,
            &self.client,
            &self.sender,
            &self.conf,
        )
        .await;
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

#[derive(Debug)]
pub struct LoggingReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl LoggingReactor {
    pub async fn new(
        receiver: mpsc::Receiver<Event>,
        client: DockerHandle,
        conf: &Configuration,
    ) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = LoggingActor::new(receiver, sender, HashMap::new(), client, conf.clone());

        if conf.docker.stream_logs {
            tokio::spawn(actor.run());
        }

        LoggingReactor {
            receiver: actor_receiver,
        }
    }
}
