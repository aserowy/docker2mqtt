pub mod no_persistence_repository;
pub mod sled_repository;

use tokio::{
    sync::{mpsc, oneshot},
    task,
};
use tracing::{debug, error};

use self::no_persistence_repository::NoPersistenceDockerRepository;
use crate::configuration::Configuration;
use crate::events::{ContainerEvent, Event, EventType};

pub trait DockerRepository: Send {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

pub fn create_repository(conf: &Configuration) -> Box<dyn DockerRepository> {
    match &conf.docker.persist_state {
        true => {
            debug!("Creating sled repository for docker");
            Box::new(sled_repository::create(super::DATA_DIRECTORY.to_owned()))
        }
        false => {
            debug!("Creating no persistence repository for docker");
            Box::new(NoPersistenceDockerRepository {})
        }
    }
}

pub async fn init_task(init_sender: oneshot::Sender<Vec<String>>, repo: &dyn DockerRepository) {
    let list = repo.list();
    task::spawn(async move {
        if let Err(err) = init_sender.send(list) {
            error!("error sending initial vector: {:?}", err);
        }
    });
}

pub async fn state_task(mut receiver: mpsc::Receiver<Event>, mut repo: Box<dyn DockerRepository>) {
    task::spawn(async move {
        while let Some(event) = receiver.recv().await {
            dispatch_event(event, &mut repo);
        }
    });
}

fn dispatch_event(event: Event, repo: &mut Box<dyn DockerRepository>) {
    if let EventType::State(container_event) = event.event {
        match container_event {
            ContainerEvent::Create => repo.add(event.container_name),
            ContainerEvent::Destroy => repo.delete(event.container_name),
            _ => {}
        }
    }
}
