pub mod no_persistence_repository;
pub mod sled_repository;

use self::{
    no_persistence_repository::NoPersistenceLoggingRepository,
    sled_repository::SledLoggingRepository,
};
use super::UnixTimestamp;
use crate::configuration::Configuration;
use crate::persistence::DATA_DIRECTORY;
use tracing::debug;

pub trait LoggingRepository: Send {
    fn set_last_logging_time(&mut self, time: UnixTimestamp);
    fn get_last_logging_time(&self) -> Option<UnixTimestamp>;
}

pub fn new(conf: &Configuration) -> Box<dyn LoggingRepository> {
    match &conf.docker.stream_logs {
        true => {
            debug!("Creating sled repository for logging");
            Box::new(SledLoggingRepository::new(DATA_DIRECTORY.to_owned()))
        }
        false => {
            debug!("Creating no persistence repository for logging");
            Box::new(NoPersistenceLoggingRepository::new())
        }
    }
}
