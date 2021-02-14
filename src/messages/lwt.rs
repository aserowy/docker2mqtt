use rs_docker::container::Container;

use super::topic;

pub fn get_availability_topic(host: &str, container: &Container) -> String {
    format!("{}/availability", topic::get_base_topic(host, container))
}

pub fn get_lwt_payload(container: &Container) -> String {
    if container.Status.starts_with("Up") {
        "online".to_string()
    } else {
        "offline".to_string()
    }
}
