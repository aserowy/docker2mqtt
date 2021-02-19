use tokio::{task, time};

use crate::{
    configuration::Configuration, docker::DockerClient, mqtt::client::MqttClient, sensor::Sensor,
};

mod configuration;
mod docker;
mod mqtt;
mod sensor;

#[tokio::main]
async fn main() {
    let conf = Configuration::new();
    let (mqtt_client, mqtt_loop) = MqttClient::new(&conf).await;

    task::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(15));

        loop {
            let docker_client = DockerClient::new();
            let containers = docker_client.get_containers();

            let sensors: Vec<Sensor> = containers
                .iter()
                .flat_map(|container| sensor::get_sensors(&docker_client, container))
                .collect();

            mqtt::send_sensor_messages(&mqtt_client, sensors, &conf).await;

            interval.tick().await;
        }
    });

    mqtt_loop.start_loop().await;
}
