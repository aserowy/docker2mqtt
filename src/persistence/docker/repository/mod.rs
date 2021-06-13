mod no_persistence_repository;
mod sled_repository;

use crate::configuration::Configuration;
use tracing::debug;
use self::{no_persistence_repository::NoPersistenceDockerRepository, sled_repository::SledDockerRepository};
use crate::persistence::DATA_DIRECTORY;

pub trait DockerRepository: Send {
    fn list(&self) -> Vec<String>;
    fn add(&mut self, container_name: String);
    fn delete(&mut self, container_name: String);
}

pub fn new(conf: &Configuration) -> Box<dyn DockerRepository> {
    match &conf.docker.persist_state {
        true => {
            debug!("Creating sled repository for docker");
            Box::new(SledDockerRepository::new(DATA_DIRECTORY.to_owned()))
        }
        false => {
            debug!("Creating no persistence repository for docker");
            Box::new(NoPersistenceDockerRepository::new())
        }
    }
}
