use rs_docker::Docker;
use rumqttc::{Event, Incoming};

mod container;
mod discovery;
mod lwt;
mod mqtt;
mod sensor;
mod state;
mod topic;

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

    let (client, mut eventloop) = mqtt::create_client(&args).await;

    let docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let messages = get_messages(docker, &args);
    for (topic, payload) in messages {
        println!("Topic: {}, Payload: {:?}", topic, payload);
        mqtt::send_message(&client, topic, payload, &args).await;
    }

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                println!("Topic: {}, Payload: {:?}", p.topic, p.payload)
            }
            Ok(Event::Incoming(i)) => {
                println!("Incoming = {:?}", i);
            }
            Ok(Event::Outgoing(o)) => println!("Outgoing = {:?}", o),
            Err(e) => {
                println!("Error = {:?}", e);
                continue;
            }
        }
    }
}

fn get_messages(mut docker: Docker, args: &Args) -> Vec<(String, String)> {
    let containers = match docker.get_containers(false) {
        Ok(containers) => containers,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let messages: Vec<(String, String)> = containers
        .iter()
        .flat_map(|container| sensor::get_messages(&docker, container, args))
        .collect();

    messages
}
