use crate::{
    configuration::Configuration,
    sensor::{Sensor, SensorType},
};

pub fn availability(sensor: &Sensor, conf: &Configuration) -> String {
    let container_name = &sensor.container.name;
    let sensor_name = &sensor.sensor_type.to_string();

    match sensor.sensor_type {
        &SensorType::CpuUsage => device_availability(&conf.mqtt.client_id, container_name),
        _ => sensor_availibility(&conf.mqtt.client_id, container_name, sensor_name),
    }
}

pub fn state(sensor: &Sensor, conf: &Configuration) -> String {
    let container_name = &sensor.container.name;
    let sensor_name = &sensor.sensor_type.to_string();

    format!(
        "{}/{}/state",
        base(&conf.mqtt.client_id, container_name),
        sensor_name
    )
}

fn device_availability(client_id: &str, container: &str) -> String {
    format!("{}/lwt", base(client_id, container))
}

fn sensor_availibility(client_id: &str, container: &str, sensor: &str) -> String {
    format!("{}/{}/lwt", base(client_id, container), sensor)
}

fn base(client_id: &str, container: &str) -> String {
    format!("docker2mqtt/{}/{}", client_id, container)
}
