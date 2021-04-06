use tokio::{
    sync::{
        broadcast::{self, error::RecvError},
        oneshot,
    },
    task,
};
use tracing::{debug, error};

use self::no_persistence_repository::NoPersistenceRepository;
use crate::configuration::Configuration;
use crate::events::{ContainerEvent, Event, EventType};

mod no_persistence_repository;
mod sled_repository;

pub trait Repository: Send {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

pub fn create_repository(conf: &Configuration) -> Box<dyn Repository> {
    match &conf.persistence {
        Some(true) => {
            debug!("Creating sled repository");
            Box::new(sled_repository::create("/docker2mqtt/data".to_owned()))
        }
        _ => {
            debug!("Creating no persistence repository");
            Box::new(NoPersistenceRepository {})
        }
    }
}

pub async fn init_task(init_sender: oneshot::Sender<Vec<String>>, repo: &dyn Repository) {
    let list = repo.list();
    task::spawn(async move {
        if let Err(err) = init_sender.send(list) {
            error!("error sending initial vector: {:?}", err);
        }
    });
}

pub async fn state_task(mut receiver: broadcast::Receiver<Event>, mut repo: Box<dyn Repository>) {
    task::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => dispatch_event(event, &mut repo),
                Err(RecvError::Closed) => break,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }
        }
    });
}

fn dispatch_event(event: Event, repo: &mut Box<dyn Repository>) {
    if let EventType::State(container_event) = event.event {
        match container_event {
            ContainerEvent::Create => repo.add(event.container_name),
            ContainerEvent::Destroy => repo.delete(event.container_name),
            _ => {}
        }
    }
}
