use rs_docker::container::Container;

pub fn get_lwt_payload(container: &Container) -> &str {
    if container.Status.starts_with("Up") {
        "online"
    } else {
        "offline"
    }
}
