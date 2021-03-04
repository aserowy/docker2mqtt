use tokio::{sync::mpsc::Receiver, task};
use tracing::instrument;

use crate::{configuration::Configuration, docker::Event};

use self::client::MqttClient;

mod availability;
mod client;
mod discovery;
mod message;
mod payload;
mod topic;

pub async fn spin_up(mut receiver: Receiver<Event>, conf: Configuration) {
    let (mqtt_client, mqtt_loop) = MqttClient::new(&conf).await;

    task::spawn(async move {
        while let Some(event) = receiver.recv().await {
            send_event_messages(&mqtt_client, event, &conf).await;
        }
    });

    mqtt_loop.start_loop().await;
}

#[instrument(level = "debug")]
async fn send_event_messages(mqtt_client: &MqttClient, event: Event, conf: &Configuration) -> () {
    let messages = message::get_event_messages(event, conf);

    for message in messages.into_iter() {
        mqtt_client.send_message(message, conf).await;
    }
}
