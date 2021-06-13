mod repository;

use tokio::{
    sync::{mpsc, oneshot}
};
use tracing::error;
use crate::configuration::Configuration;
use self::repository::DockerRepository;

pub enum DockerRepositoryMessage {
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
pub struct DockerRepositoryHandle {
    sender: mpsc::Sender<DockerRepositoryMessage>,
}

impl DockerRepositoryHandle {
    pub fn new(conf: &Configuration) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = DockerRepositoryActor::new(conf, receiver);
        tokio::spawn(actor.run());
        Self { sender }
    }

    pub async fn handle(&self, message: DockerRepositoryMessage) {
        self.sender
            .send(message)
            .await
            .map_err(|err| error!("Error sending DockerRepositoryMessage: {}", err))
            .ok();
    }
}

struct DockerRepositoryActor {
    repository: Box<dyn DockerRepository>,
    receiver: mpsc::Receiver<DockerRepositoryMessage>,
}

impl DockerRepositoryActor {
    fn new(conf: &Configuration, receiver: mpsc::Receiver<DockerRepositoryMessage>) -> Self {
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

    fn handle(&mut self, message: DockerRepositoryMessage) {
        match message {
            DockerRepositoryMessage::GetAllDockerContainers { respond_to: response } => {
                if let Err(err) = response.send(self.repository.list()) {
                    error!("Error sending docker container list: {:?}", err)
                }
            }
            DockerRepositoryMessage::DeleteDockerContainer { name } => self.repository.delete(name),
            DockerRepositoryMessage::AddDockerContainer { name } => self.repository.add(name),
        }
    }
}
