use crate::persistence::Repository;

pub struct SledRepository {

}

impl Repository for SledRepository {
    fn add(&mut self, container_name: String) {
        unimplemented!()
    }

    fn delete(&mut self, container_name: String) {
        unimplemented!()
    }
}
