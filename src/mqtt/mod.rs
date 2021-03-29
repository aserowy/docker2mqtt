use tokio::{sync::broadcast::Receiver, task};
use tracing::{error, instrument};

use crate::{configuration::Configuration, docker::Event};

use self::client::MqttClient;
use tokio::sync::broadcast::error::RecvError;

mod availability;
mod client;
mod discovery;
mod message;
mod payload;
mod topic;

pub async fn task(mut receiver: Receiver<Event>, conf: Configuration) {
    let (mqtt_client, mqtt_loop) = MqttClient::new(&conf).await;

    task::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => send_event_messages(&mqtt_client, event, &conf).await,
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(m)) => error!("Receiver lagging. Skipped {} messages", m),
            }
        }
    });

    mqtt_loop.start_loop().await;
}

#[instrument(level = "debug")]
async fn send_event_messages(mqtt_client: &MqttClient, event: Event, conf: &Configuration) {
    let messages = message::get_event_messages(event, conf);

    for message in messages.into_iter() {
        mqtt_client.send_message(message, conf).await;
    }
}
