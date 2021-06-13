use crate::persistence::logging::{LoggingRepository, UnixTimestamp};

#[derive(Debug)]
pub struct NoPersistenceLoggingRepository {}

impl NoPersistenceLoggingRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl LoggingRepository for NoPersistenceLoggingRepository {
    fn set_last_logging_time(&mut self, _: UnixTimestamp) {}
    fn get_last_logging_time(&self) -> Option<UnixTimestamp> {
        Option::None
    }
}
