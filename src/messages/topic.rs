use crate::docker::Container;

pub fn get_base_topic(client_id: &str, container: &Container) -> String {
    format!("docker2mqtt/{}/{}", client_id, container.name)
}
