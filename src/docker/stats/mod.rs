use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::events::Event;

use super::client::DockerHandle;

mod cpu;
mod handle;
mod memory;
mod stream;

struct StatsActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tasks: HashMap<String, JoinHandle<()>>,
    client: DockerHandle,
}

impl StatsActor {
    fn new(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Event>,
        tasks: HashMap<String, JoinHandle<()>>,
        client: DockerHandle,
    ) -> Self {
        Self {
            receiver,
            sender,
            tasks,
            client,
        }
    }

    async fn handle(&mut self, event: Event) {
        handle::event(event, &mut self.tasks, &self.client, &self.sender).await;
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

#[derive(Debug)]
pub struct StatsReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl StatsReactor {
    pub async fn new(receiver: mpsc::Receiver<Event>, client: DockerHandle) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = StatsActor::new(receiver, sender, HashMap::new(), client);

        tokio::spawn(actor.run());

        Self {
            receiver: actor_receiver,
        }
    }
}
