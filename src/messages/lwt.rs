use crate::docker::Container;

use super::topic;

pub fn get_availability_topic(client_id: &str, container: &Container) -> String {
    format!(
        "{}/availability",
        topic::get_base_topic(client_id, container)
    )
}

pub fn get_lwt_payload(container: &Container) -> String {
    if container.status.starts_with("Up") {
        "online".to_string()
    } else {
        "offline".to_string()
    }
}
