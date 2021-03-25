use crate::persistence::Repository;
use sled::{open, Db, Error, IVec};
use std::ops::Add;

pub struct SledRepository {
    database: Db
}

pub fn create(directory: String) -> SledRepository {
    SledRepository {
        database: open(directory.add("/docker.db")).unwrap() //TODO Panic okay?
    }
}

impl Repository for SledRepository {
    fn list(&self) -> Vec<String> {
        unimplemented!()
    }

    fn add(&mut self, container_name: String) {
        unimplemented!()
    }

    fn delete(&mut self, container_name: String) {
        unimplemented!()
    }
}
