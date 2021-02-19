use tokio::{task, time};

use crate::{docker::DockerClient, mqtt::client::MqttClient, sensor::Sensor};

mod docker;
mod mqtt;
mod sensor;

pub struct Args {
    client_id: String,
    hassio_discovery_enabled: Option<bool>,
    hassio_discovery_prefix: Option<String>,
    hassio_device_prefix: Option<String>,
    mqtt_host: String,
    mqtt_keep_alive: u16,
    mqtt_op_timeout: u64,
    // mqtt_password: Option<String>,
    mqtt_port: u16,
    // mqtt_tls_mozilla_root_cas: bool,
    // mqtt_tls_server_ca_file: Option<String>,
    // mqtt_username: Option<String>,
    mqtt_qos: i32,
}

#[tokio::main]
async fn main() {
    let args = Args {
        client_id: "testhost".to_owned(),
        hassio_discovery_enabled: Option::Some(true),
        hassio_discovery_prefix: Option::Some("homeassistant".to_owned()),
        hassio_device_prefix: Option::Some("docker".to_owned()),
        mqtt_host: "mosquitto".to_owned(),
        mqtt_keep_alive: 30,
        mqtt_op_timeout: 20,
        // mqtt_password: Option::None,
        // mqtt_tls_mozilla_root_cas: false,
        mqtt_port: 1883,
        // mqtt_tls_server_ca_file: Option::None,
        // mqtt_username: Option::None,
        mqtt_qos: 1,
    };

    let (mqtt_client, mqtt_loop) = MqttClient::new(&args).await;

    task::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(15));

        loop {
            let docker_client = DockerClient::new();
            let containers = docker_client.get_containers();

            let sensors: Vec<Sensor> = containers
                .iter()
                .flat_map(|container| sensor::get_sensors(&docker_client, container))
                .collect();

            mqtt::send_messages_for(&mqtt_client, sensors, &args).await;

            interval.tick().await;
        }
    });

    mqtt_loop.start_loop().await;
}
