pub fn device_availability(client_id: &str, container: &str) -> String {
    format!("{}/lwt", base(client_id, container))
}

pub fn sensor_availibility(client_id: &str, container: &str, sensor: &str) -> String {
    format!("{}/{}/lwt", base(client_id, container), sensor)
}

pub fn state(client_id: &str, container: &str, sensor: &str) -> String {
    format!("{}/{}/state", base(client_id, container), sensor)
}

fn base(client_id: &str, container: &str) -> String {
    format!("docker2mqtt/{}/{}", client_id, container)
}
