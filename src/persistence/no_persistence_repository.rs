use crate::persistence::Repository;

pub struct NoPersistenceRepository {

}

impl Repository for NoPersistenceRepository {
    fn list(&self) -> Vec<String> {
        Vec::new()
    }

    fn add(&mut self, _: String) {}

    fn delete(&mut self, _: String) {}

}