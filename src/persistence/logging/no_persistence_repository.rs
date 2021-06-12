use super::{LoggingRepository, UnixTimestamp};

pub struct NoPersistenceLoggingRepository {}

impl LoggingRepository for NoPersistenceLoggingRepository {
    fn set_last_logging_time(&mut self, _: UnixTimestamp) {}
    fn get_last_logging_time(&self) -> Option<UnixTimestamp> {
        Option::None
    }
}

