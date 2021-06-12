use sled::{Db, IVec};
use std::ops::Add;
use tracing::error;

use crate::persistence::logging::{LoggingRepository, UnixTimestamp};
use std::convert::TryInto;

pub struct SledLoggingRepository {
    database: Db,
}

pub fn create(directory: String) -> SledLoggingRepository {
    SledLoggingRepository {
        database: sled::open(directory.add("/logging.db")).unwrap(),
    }
}

impl LoggingRepository for SledLoggingRepository {

    fn set_last_logging_time(&mut self, time: UnixTimestamp) {
        let result = self
            .database
            .insert("last_logging_time", &time.time.to_be_bytes());
        if let Err(e) = result {
            error!("error saving string: {}", e)
        }
    }

    fn get_last_logging_time(&self) -> Option<UnixTimestamp> {
        match self.database.get("last_logging_time") {
            Ok(val) => val.and_then(|v| read_from_ivec(v)),
            Err(err) => {
                error!("error receiving entry from repository: {}", err);
                Option::None
            }
        }
    }
}

fn read_from_ivec(ivec: IVec) -> Option<UnixTimestamp> {
     match ivec.as_ref().try_into().map(|v: [u8; 8]| UnixTimestamp{time: i64::from_be_bytes(v)}) {
         Ok(val) => Option::Some(val),
         Err(err) => {
             error!("error converting byte array to UnixTimestamp: {}", err);
             Option::None
         }
     }
}
