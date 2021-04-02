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
use crate::docker::{ContainerEvent, Event, EventType};

mod no_persistence_repository;
mod sled_repository;

pub trait Repository: Send {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

pub async fn task(
    init_sender: oneshot::Sender<Vec<String>>,
    event_receiver: broadcast::Receiver<Event>,
    conf: &Configuration,
) {
    let repository = create_repository(conf);
    initial(init_sender, repository.as_ref());
    source(event_receiver, repository).await;
}

fn create_repository(conf: &Configuration) -> Box<dyn Repository> {
    match &conf.persistence {
        Some(persistence) => {
            debug!("Creating sled repository");
            Box::new(sled_repository::create(persistence.directory.to_owned()))
        }
        _ => {
            debug!("Creating no persistence repository");
            Box::new(NoPersistenceRepository {})
        }
    }
}

fn initial(init_sender: oneshot::Sender<Vec<String>>, repo: &dyn Repository) {
    if let Err(err) = init_sender.send(repo.list()) {
        error!("error sending initial vector: {:?}", err);
    }
}

async fn source(mut event_receiver: broadcast::Receiver<Event>, mut repo: Box<dyn Repository>) {
    task::spawn(async move {
        loop {
            match event_receiver.recv().await {
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
            ContainerEvent::Create | ContainerEvent::Start => repo.add(event.container_name),
            ContainerEvent::Destroy
            | ContainerEvent::Die
            | ContainerEvent::Kill
            | ContainerEvent::Stop
            | ContainerEvent::Prune => repo.delete(event.container_name),
            _ => {}
        }
    }
}
