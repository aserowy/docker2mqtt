use tokio::sync::{
    mpsc,
    oneshot
};

use crate::configuration::Configuration;
use self:: {
    no_persistence_repository::NoPersistenceRepository,
    sled_repository::SledRepository
};

mod no_persistence_repository;
mod sled_repository;

pub trait Repository {

}

pub struct Event {
    container_name: String,
    event_type: EventType
}

pub enum EventType {
    Add,
    Remove
}

pub async fn spin_up(
    init_sender: oneshot::Sender<Option<Vec<String>>>,
    receiver: mpsc::Receiver<Event>,
    conf: &Configuration
) {
    let repository = create_repository(conf);
}

fn create_repository(conf: &Configuration) -> Box<dyn Repository> {
    if conf.persistence.is_some() {
        Box::new(SledRepository {

        })
    } else {
        Box::new(NoPersistenceRepository {

        })
    }
}