use std::collections::HashMap;

use bollard::Docker;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{events::Event, Configuration};

mod handle;
mod stream;
mod validate;

struct LoggingActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tasks: HashMap<String, JoinHandle<()>>,
    client: Docker,
    conf: Configuration,
}

impl LoggingActor {
    fn with(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Event>,
        tasks: HashMap<String, JoinHandle<()>>,
        client: Docker,
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
    pub async fn with(
        receiver: mpsc::Receiver<Event>,
        client: Docker,
        conf: &Configuration,
    ) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = LoggingActor::with(receiver, sender, HashMap::new(), client, conf.clone());

        if conf.docker.stream_logs {
            tokio::spawn(actor.run());
        }

        LoggingReactor {
            receiver: actor_receiver,
        }
    }
}
