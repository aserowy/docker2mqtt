use rs_docker::Docker;

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
    mqtt_auto_connect: String,
    mqtt_keep_alive: i32,
    mqtt_op_timeout: i32,
    mqtt_password: Option<String>,
    mqtt_server_uri: String,
    mqtt_tls_mozilla_root_cas: bool,
    mqtt_tls_server_ca_file: Option<String>,
    mqtt_username: Option<String>,
    mqtt_qos: i32,
}

fn main() {
    let args = Args {
        client_id: "testhost".to_owned(),
        hass_discovery_prefix: Option::Some("homeassistant".to_owned()),
        mqtt_auto_connect: "true".to_owned(),
        mqtt_keep_alive: 30,
        mqtt_op_timeout: 20,
        mqtt_password: Option::None,
        mqtt_server_uri: "tcp://localhost:1883".to_owned(),
        mqtt_tls_mozilla_root_cas: false,
        mqtt_tls_server_ca_file: Option::None,
        mqtt_username: Option::None,
        mqtt_qos: 1,
    };

    let client = mqtt::create_client(&args);

    let docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let messages = get_messages(docker, &args);
    for (topic, payload) in messages {
        mqtt::send_message(&client, topic, payload, &args);
    }

    mqtt::close_client(client);
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
        .flat_map(|container| mqtt::get_messages(&docker, container, args))
        .collect();

    messages
}
