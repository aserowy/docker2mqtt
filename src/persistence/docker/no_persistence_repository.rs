use super::DockerRepository;

#[derive(Debug)]
pub struct NoPersistenceDockerRepository {}

impl NoPersistenceDockerRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl DockerRepository for NoPersistenceDockerRepository {
    fn list(&self) -> Vec<String> {
        Vec::new()
    }

    fn add(&mut self, _: String) {}

    fn delete(&mut self, _: String) {}
}
