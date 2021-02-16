use rs_docker::Docker;

use crate::mqtt::MqttClient;

mod container;
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

    let client = MqttClient::new(&args).await;

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

    let messages: Vec<(String, String)> = containers
        .iter()
        .flat_map(|container| messages::get_messages(&docker, container, &args))
        .collect();

    for (topic, payload) in messages {
        println!("Topic: {}, Payload: {:?}", topic, payload);
        client.send_message(topic, payload, &args).await;
    }

    client.start_loop().await;
}
