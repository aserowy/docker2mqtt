use rs_docker::container::Container;

pub fn get_lwt_payload(container: &Container) -> String {
    if container.Status.starts_with("Up") {
        "online".to_string()
    } else {
        "offline".to_string()
    }
}
