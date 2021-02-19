pub struct Configuration {
    pub hassio: Hassio,
    pub mqtt: Mqtt,
}

pub struct Hassio {
    pub discovery_enabled: Option<bool>,
    pub discovery_prefix: Option<String>,
    pub device_prefix: Option<String>,
}

pub struct Mqtt {
    pub client_id: String,
    pub connection_timeout: u64,
    pub host: String,
    pub keep_alive: u16,
    pub password: Option<String>,
    pub port: u16,
    pub qos: i32,
    pub username: Option<String>,
}

impl Configuration {
    pub fn new() -> Configuration {
        todo!()
    }
}

// let conf = Configuration {
//     client_id: "testhost".to_owned(),
//     hassio.discovery_enabled: Option::Some(true),
//     hassio.discovery_prefix: Option::Some("homeassistant".to_owned()),
//     hassio.device_prefix: Option::Some("docker".to_owned()),
//     mqtt_connection_timeout: 20,
//     mqtt_host: "mosquitto".to_owned(),
//     mqtt_keep_alive: 30,
//     mqtt_password: Option::None,
//     mqtt_port: 1883,
//     mqtt_qos: 1,
//     mqtt_username: Option::None,
// };
