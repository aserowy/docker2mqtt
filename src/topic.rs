use rs_docker::container::Container;

use crate::sensor::Sensor;

pub fn get_availability_topic(host: &str, container: &Container) -> String {
    format!("{}/availability", get_base_topic(host, container))
}

pub fn get_discovery_topic(host: &str, container: &Container, sensor: &Sensor) -> String {
    format!(
        "homeassistant/sensor/{}/{}/config",
        get_base_topic(host, container),
        sensor
    )
}

pub fn get_state_topic(host: &str, container: &Container, sensor: &Sensor) -> String {
    format!("{}/{}/state", get_base_topic(host, container), sensor)
}

pub fn get_container_name(container: &Container) -> &str {
    let container_name = &container.Names[0];
    let (first_char, remainder) = split_first_char_remainder(container_name);

    match first_char {
        "/" => remainder,
        _ => container_name,
    }
}

fn get_base_topic(host: &str, container: &Container) -> String {
    let container_name = get_container_name(container);

    format!("docker2mqtt/{}/{}", host, container_name)
}

fn split_first_char_remainder(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}
