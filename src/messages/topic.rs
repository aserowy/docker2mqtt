use rs_docker::container::Container;

use crate::container;

pub fn get_base_topic(host: &str, container: &Container) -> String {
    format!(
        "docker2mqtt/{}/{}",
        host,
        container::get_container_name(container)
    )
}
