use std::{
    fs::File,
    io::{self, Error, Read},
};

use serde::Deserialize;
use tracing::instrument;

#[derive(Clone, Debug, Deserialize)]
pub struct Configuration {
    #[serde(default)]
    pub docker: Docker,

    #[serde(default)]
    pub hassio: Option<Hassio>,

    #[serde(default)]
    pub logging: Logging,

    pub mqtt: Mqtt,
}

impl Configuration {
    #[instrument]
    pub fn new() -> Self {
        let content = read_file(
            "/docker2mqtt/config/",
            vec!["configuration.yaml", "configuration.yml"],
        );

        serde_yaml::from_str(&content).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Docker {
    #[serde(default)]
    pub persist_state: bool,

    #[serde(default = "Docker::default_stream_logs")]
    pub stream_logs: bool,

    #[serde(default)]
    pub stream_logs_container: Vec<String>,

    #[serde(default)]
    pub stream_logs_filter: Vec<String>,
}

impl Default for Docker {
    fn default() -> Self {
        Docker {
            persist_state: false,
            stream_logs: true,
            stream_logs_container: vec![],
            stream_logs_filter: vec![],
        }
    }
}

impl Docker {
    fn default_stream_logs() -> bool {
        true
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
pub struct Logging {
    #[serde(default = "Logging::default_level")]
    pub level: String,
}

impl Default for Logging {
    fn default() -> Self {
        Logging {
            level: Logging::default_level(),
        }
    }
}

impl Logging {
    fn default_level() -> String {
        "INFO".to_owned()
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
    let mut error: Option<Error> = None;
    for variant in filename_variants {
        let file = format!("{}{}", path, variant);

        match read_single_file(file) {
            Ok(value) => return value,
            Err(e) => error = Some(e),
        }
    }

    panic!("Configuration file missing: {:?}", error);
}

fn read_single_file(file: String) -> io::Result<String> {
    let mut file = File::open(file)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    Ok(content)
}

#[cfg(test)]
mod must {
    #[test]
    fn parse_defaults_for_minimal_config() {
        // arrange
        let buffer = "
mqtt:
  client_id: qwert
  host: yuio
  port: 1234";

        // act
        let mut config: super::Configuration = serde_yaml::from_str(buffer).unwrap();

        // assert
        assert!(config.hassio.is_none());

        assert_eq!(config.docker.persist_state, false);
        assert_eq!(config.docker.stream_logs, true);
        assert_eq!(config.docker.stream_logs_container.pop(), None);
        assert_eq!(config.docker.stream_logs_filter.pop(), None);

        assert_eq!("INFO", config.logging.level);

        assert_eq!(20, config.mqtt.connection_timeout);
        assert_eq!(30, config.mqtt.keep_alive);
        assert_eq!(0, config.mqtt.qos);
    }

    #[test]
    fn parse_given_values_for_config() {
        // arrange
        let buffer = "
docker:
  persist_state: true
  stream_logs_container:
    - docker2mqtt
    - borg
  stream_logs_filter:
    - test
    - test02

mqtt:
  client_id: qwert
  host: yuio
  port: 1234";

        // act
        let mut config: super::Configuration = serde_yaml::from_str(buffer).unwrap();

        // assert
        assert_eq!(config.docker.persist_state, true);
        assert_eq!(config.docker.stream_logs, true);

        assert_eq!(
            config.docker.stream_logs_container.pop(),
            Some("borg".to_owned())
        );
        assert_eq!(
            config.docker.stream_logs_container.pop(),
            Some("docker2mqtt".to_owned())
        );
        assert_eq!(config.docker.stream_logs_container.pop(), None);

        assert_eq!(
            config.docker.stream_logs_filter.pop(),
            Some("test02".to_owned())
        );
        assert_eq!(
            config.docker.stream_logs_filter.pop(),
            Some("test".to_owned())
        );
        assert_eq!(config.docker.stream_logs_filter.pop(), None);
    }
}
