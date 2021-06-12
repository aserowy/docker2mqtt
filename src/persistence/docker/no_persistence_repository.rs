use super::DockerRepository;

pub struct NoPersistenceDockerRepository {}

impl DockerRepository for NoPersistenceDockerRepository {
    fn list(&self) -> Vec<String> {
        Vec::new()
    }

    fn add(&mut self, _: String) {}

    fn delete(&mut self, _: String) {}
}
