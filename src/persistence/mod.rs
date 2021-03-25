use tokio::sync::{
    mpsc,
    oneshot
};

use crate::configuration::Configuration;
use self:: {
    no_persistence_repository::NoPersistenceRepository
};

mod no_persistence_repository;
mod sled_repository;

pub trait Repository {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

pub struct Event {
    pub container_name: String,
    pub event_type: EventType
}

pub enum EventType {
    Add,
    Remove
}

pub async fn spin_up(
    init_sender: oneshot::Sender<Vec<String>>,
    receiver: mpsc::Receiver<Event>,
    conf: &Configuration
) {
    let repository = create_repository(conf);


}

fn create_repository(conf: &Configuration) -> Box<dyn Repository> {
    match &conf.persistence {
        Some(persistence) => {
            Box::new(sled_repository::create(persistence.directory.to_owned()))
        }
        _ => {
            Box::new(NoPersistenceRepository {

            })
        }
    }
}