use sled::Db;
use std::{ops::Add, str};
use tracing::error;

use super::DockerRepository;

#[derive(Debug)]
pub struct SledDockerRepository {
    database: Db,
}

impl SledDockerRepository {
    pub fn new(directory: String) -> Self {
        Self {
            database: sled::open(directory.add("/docker.db")).unwrap(),
        }
    }
}

impl DockerRepository for SledDockerRepository {
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
    match str::from_utf8(bytes) {
        Ok(r) => Option::Some(r.to_owned()),
        Err(err) => {
            error!("error converting bytes to String: {}", err);
            Option::None
        }
    }
}
