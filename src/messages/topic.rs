use rs_docker::container::Container;

use crate::container;

pub fn get_base_topic(client_id: &str, container: &Container) -> String {
    format!(
        "docker2mqtt/{}/{}",
        client_id,
        container::get_container_name(container)
    )
}
