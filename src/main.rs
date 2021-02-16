use docker::DockerClient;

use crate::mqtt::MqttClient;

mod docker;
mod messages;
mod mqtt;
mod sensor;

pub struct Args {
    client_id: String,
    hass_discovery_prefix: Option<String>,
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
        hass_discovery_prefix: Option::Some("homeassistant".to_owned()),
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

    let mqtt_client = MqttClient::new(&args).await;
    let mut docker_client = DockerClient::new();

    let containers = docker_client.get_containers();
    let messages = messages::get_messages(&docker_client, containers, &args);
    for message in messages {
        println!("Topic: {}, Payload: {:?}", message.topic, message.payload);
        mqtt_client
            .send_message(message.topic, message.payload, &args)
            .await;
    }

    mqtt_client.start_loop().await;
}
