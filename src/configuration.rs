use core::panic;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Read},
};
use tracing::{error, instrument};

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub hassio: Option<Hassio>,

    pub mqtt: Mqtt,
}

impl Configuration {
    #[instrument]
    pub fn new() -> Configuration {
        let content = read_file(
            "/docker2mqtt/config/",
            vec!["configuration.yaml", "configuration.yml"],
        );

        serde_yaml::from_str(&content).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Hassio {
    pub discovery: bool,

    #[serde(default = "Hassio::default_discovery_prefix")]
    pub discovery_prefix: String,

    #[serde(default = "Hassio::default_device_prefix")]
    pub device_prefix: String,
}

impl Hassio {
    fn default_discovery_prefix() -> String {
        "homeassistant".to_owned()
    }

    fn default_device_prefix() -> String {
        "docker".to_owned()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Mqtt {
    pub client_id: String,
    pub host: String,
    pub port: u16,

    #[serde(default = "Mqtt::default_connection_timeout")]
    pub connection_timeout: u64,

    #[serde(default = "Mqtt::default_keep_alive")]
    pub keep_alive: u16,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(default = "Mqtt::default_qos")]
    pub qos: u8,

    #[serde(default)]
    pub username: Option<String>,
}

impl Mqtt {
    fn default_connection_timeout() -> u64 {
        20
    }

    fn default_keep_alive() -> u16 {
        30
    }

    fn default_qos() -> u8 {
        0
    }
}

fn read_file(path: &str, filename_variants: Vec<&str>) -> String {
    for variant in filename_variants {
        let file = format!("{}{}", path, variant);

        match read_single_file(file) {
            Ok(value) => return value,
            Err(_) => continue,
        }
    }

    error!("Configuration file missing.");
    panic!();
}

fn read_single_file(file: String) -> io::Result<String> {
    match File::open(file) {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            Ok(content)
        }
        Err(e) => Err(e),
    }
}
