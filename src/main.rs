use tokio::task;
use tokio_stream::StreamExt;

use crate::{configuration::Configuration, docker::DockerClient, mqtt::client::MqttClient};

mod configuration;
mod docker;
mod logging;
mod mqtt;

#[tokio::main]
async fn main() {
    let _guards = logging::init();

    let conf = Configuration::new();
    let (mqtt_client, mqtt_loop) = MqttClient::new(&conf).await;

    task::spawn(async move {
        while let Some(sensors) = DockerClient::new().get_event_stream().next().await {
            mqtt::send_event_messages(&mqtt_client, sensors, &conf).await;
        }
    });

    mqtt_loop.start_loop().await;
}
