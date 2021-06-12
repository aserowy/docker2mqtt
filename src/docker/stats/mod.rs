use std::collections::HashMap;

use bollard::Docker;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::events::Event;

mod cpu;
mod handle;
mod memory;
mod stream;

struct StatsActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tasks: HashMap<String, JoinHandle<()>>,
    client: Docker,
}

impl StatsActor {
    fn new(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Event>,
        tasks: HashMap<String, JoinHandle<()>>,
        client: Docker,
    ) -> Self {
        StatsActor {
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
    pub async fn new(receiver: mpsc::Receiver<Event>, client: Docker) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = StatsActor::new(receiver, sender, HashMap::new(), client);

        tokio::spawn(actor.run());

        StatsReactor {
            receiver: actor_receiver,
        }
    }
}
