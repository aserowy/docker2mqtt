pub mod no_persistence_repository;
pub mod sled_repository;

use tokio::{
    sync::{mpsc, oneshot}
};
use tracing::{debug, error};

use self::{
    no_persistence_repository::NoPersistenceDockerRepository, sled_repository::SledDockerRepository,
};
use crate::configuration::Configuration;

pub enum DockerRepositoryMessage {
    GetAllDockerContainers {
        response: oneshot::Sender<Vec<String>>,
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
            .map_err(|err| error!("Error sending DockerRepositoryMessage: {}", err));
    }
}

pub trait DockerRepository: Send {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

struct DockerRepositoryActor {
    repository: Box<dyn DockerRepository>,
    receiver: mpsc::Receiver<DockerRepositoryMessage>,
}

impl DockerRepositoryActor {
    fn new(conf: &Configuration, receiver: mpsc::Receiver<DockerRepositoryMessage>) -> Self {
        Self {
            repository: create_repository(conf),
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
            DockerRepositoryMessage::GetAllDockerContainers { response } => {
                if let Err(err) = response.send(self.repository.list()) {
                    error!("Error sending docker container list: {:?}", err)
                }
            }
            DockerRepositoryMessage::DeleteDockerContainer { name } => self.repository.delete(name),
            DockerRepositoryMessage::AddDockerContainer { name } => self.repository.add(name),
        }
    }
}

pub fn create_repository(conf: &Configuration) -> Box<dyn DockerRepository> {
    match &conf.docker.persist_state {
        true => {
            debug!("Creating sled repository for docker");
            Box::new(SledDockerRepository::new(super::DATA_DIRECTORY.to_owned()))
        }
        false => {
            debug!("Creating no persistence repository for docker");
            Box::new(NoPersistenceDockerRepository::new())
        }
    }
}
