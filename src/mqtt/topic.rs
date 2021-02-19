use crate::{
    sensor::{Sensor, SensorType},
    Args,
};

pub fn availability(sensor: &Sensor, args: &Args) -> String {
    let container_name = &sensor.container.name;
    let sensor_name = &sensor.sensor_type.to_string();

    match sensor.sensor_type {
        &SensorType::CpuUsage => device_availability(&args.client_id, container_name),
        _ => sensor_availibility(&args.client_id, container_name, sensor_name),
    }
}

pub fn state(sensor: &Sensor, args: &Args) -> String {
    let container_name = &sensor.container.name;
    let sensor_name = &sensor.sensor_type.to_string();

    format!(
        "{}/{}/state",
        base(&args.client_id, container_name),
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