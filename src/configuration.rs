pub struct Configuration {
    pub client_id: String,
    pub hassio_discovery_enabled: Option<bool>,
    pub hassio_discovery_prefix: Option<String>,
    pub hassio_device_prefix: Option<String>,
    pub mqtt_connection_timeout: u64,
    pub mqtt_host: String,
    pub mqtt_keep_alive: u16,
    pub mqtt_password: Option<String>,
    pub mqtt_port: u16,
    pub mqtt_qos: i32,
    pub mqtt_username: Option<String>,
}
