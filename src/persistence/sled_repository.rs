use sled::{open, Db};
use std::ops::Add;
use tracing::error;

use crate::persistence::Repository;
use std::str::from_utf8;

pub struct SledRepository {
    database: Db,
}

pub fn create(directory: String) -> SledRepository {
    SledRepository {
        database: open(directory.add("/docker.db")).unwrap(), //TODO Panic okay?
    }
}

impl Repository for SledRepository {
    fn list(&self) -> Vec<String> {
        let mut entries = Vec::new();
        for entry_result in self.database.iter() {
            match entry_result {
                Ok(entry) => convert_to_string(entry.1.as_ref())
                    .into_iter()
                    .for_each(|v| entries.push(v)),
                Err(err) => error!("error receiving entry from repository: {}", err),
            }
        }
        entries
    }

    fn add(&mut self, container_name: String) {
        let result = self
            .database
            .insert(container_name.as_bytes(), container_name.as_bytes());
        if let Err(e) = result {
            error!("error saving string: {}", e)
        }
    }

    fn delete(&mut self, container_name: String) {
        let result = self.database.remove(container_name.as_bytes());
        if let Err(e) = result {
            error!("error deleting string: {}", e)
        }
    }
}

fn convert_to_string(bytes: &[u8]) -> Option<String> {
    match from_utf8(bytes) {
        Ok(r) => Option::Some(r.to_owned()),
        Err(err) => {
            error!("error converting bytes to String: {}", err);
            Option::None
        }
    }
}
