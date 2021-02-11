use discovery::device::Device;
use discovery::sensor::Sensor;
use rs_docker::container::Container;
use rs_docker::Docker;

mod discovery;

fn main() {
    let host = "hostname";

    let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let containers = match docker.get_containers(false) {
        Ok(containers) => containers,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let sensors: Vec<Sensor> = containers
        .iter()
        .map(|container| map_container_to_sensor(host, "image", container))
        .collect();

    for sensor in sensors {
        println!("{:?}", sensor.to_json());
    }
}

fn map_container_to_sensor(host: &str, sensor: &str, container: &Container) -> Sensor {
    let device_name = &format!("docker_{}", host);

    let mut identifiers = Vec::new();
    identifiers.push(device_name.to_string());

    let container_name = resolve_container_name(container);
    let unique_id = format!("{}_{}_{}", device_name, container_name, sensor);
    let topic_base = format!("docker2mqtt/{}/{}", host, container_name);

    Sensor {
        availability_topic: format!("{}/availability", &topic_base),
        device: Device {
            identifiers,
            manufacturer: "docker2mqtt".to_string(),
            model: "docker".to_string(),
            name: device_name.to_string(),
        },
        icon: "".to_string(),
        name: unique_id.to_string(),
        payload: true,
        payload_available: "".to_string(),
        payload_not_available: "".to_string(),
        platform: "mqtt".to_string(),
        state_topic: format!("{}/{}/state", &topic_base, sensor),
        unique_id,
    }
}

fn resolve_container_name(container: &Container) -> &str {
    let container_name = &container.Names[0];
    let (first_char, remainder) = split_first_char_remainder(container_name);

    match first_char {
        "/" => remainder,
        _ => container_name,
    }
}

fn split_first_char_remainder(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}
