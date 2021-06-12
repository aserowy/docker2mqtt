mod no_persistence_repository;
mod sled_repository;

use self::{
    no_persistence_repository::NoPersistenceLoggingRepository,
    sled_repository::SledLoggingRepository,
};
use crate::configuration::Configuration;
use tracing::debug;

pub struct UnixTimestamp {
    pub time: i64,
}

pub trait LoggingRepository: Send {
    fn set_last_logging_time(&mut self, time: UnixTimestamp);
    fn get_last_logging_time(&self) -> Option<UnixTimestamp>;
}

pub fn create_repository(conf: &Configuration) -> Box<dyn LoggingRepository> {
    match &conf.docker.stream_logs {
        true => {
            debug!("Creating sled repository for logging");
            Box::new(SledLoggingRepository::new(super::DATA_DIRECTORY.to_owned()))
        }
        false => {
            debug!("Creating no persistence repository for logging");
            Box::new(NoPersistenceLoggingRepository::new())
        }
    }
}
