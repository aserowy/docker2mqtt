mod repository;

use tokio::{
    sync::{mpsc, oneshot}
};
use tracing::error;
use crate::configuration::Configuration;
use self::repository::DockerRepository;

pub enum DockerDbMessage {
    GetAllDockerContainers {
        respond_to: oneshot::Sender<Vec<String>>,
    },
    DeleteDockerContainer {
        name: String,
    },
    AddDockerContainer {
        name: String,
    },
}

#[derive(Clone)]
pub struct DockerDbHandle {
    sender: mpsc::Sender<DockerDbMessage>,
}

impl DockerDbHandle {
    pub fn new(conf: &Configuration) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = DockerDbActor::new(conf, receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }

    pub async fn handle(&self, message: DockerDbMessage) {
        self.sender
            .send(message)
            .await
            .map_err(|err| error!("Error sending DockerRepositoryMessage: {}", err))
            .ok();
    }
}

struct DockerDbActor {
    repository: Box<dyn DockerRepository>,
    receiver: mpsc::Receiver<DockerDbMessage>,
}

impl DockerDbActor {
    fn new(conf: &Configuration, receiver: mpsc::Receiver<DockerDbMessage>) -> Self {
        Self {
            repository: repository::new(conf),
            receiver,
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message);
        }
    }

    fn handle(&mut self, message: DockerDbMessage) {
        match message {
            DockerDbMessage::GetAllDockerContainers { respond_to } => {
                if let Err(err) = respond_to.send(self.repository.list()) {
                    error!("Error sending docker container list: {:?}", err)
                }
            }
            DockerDbMessage::DeleteDockerContainer { name } => self.repository.delete(name),
            DockerDbMessage::AddDockerContainer { name } => self.repository.add(name),
        }
    }
}
