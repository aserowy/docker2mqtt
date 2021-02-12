use enum_iterator::IntoEnumIterator;
use rs_docker::{container::Container, Docker};

use crate::{discovery, lwt, state};

use crate::sensor::Sensor;

pub fn get_messages(host: &str, docker: &Docker, container: &Container) -> Vec<(String, String)> {
    let mut messages: Vec<(String, String)> = Vec::new();
    messages.push((
        lwt::get_availability_topic(host, container),
        lwt::get_lwt_payload(container),
    ));

    for sensor in Sensor::into_enum_iter() {
        messages.push((
            discovery::get_discovery_topic(host, container, &sensor),
            discovery::get_discovery_payload(host, container, &sensor),
        ));

        messages.push((
            state::get_state_topic(host, container, &sensor),
            state::get_state_payload(docker, container, &sensor),
        ));
    }

    messages
}
