use crate::persistence::Repository;

pub struct NoPersistenceRepository {

}

impl Repository for NoPersistenceRepository {
    fn add(&mut self, container_name: String) {}

    fn delete(&mut self, container_name: String) {}
}