use tokio::{sync::{broadcast::{error::RecvError, Receiver}, mpsc}, task};
use tracing::{error, instrument};

use crate::{configuration::Configuration, events::Event};

use self::{discovery::HassioReactor, message::Message, sending::MqttReactor};

mod availability;
mod client;
mod discovery;
mod message;
mod payload;
mod sending;
mod topic;

pub async fn task(mut receiver: Receiver<Event>, conf: &Configuration) {
    let conf_for_move = conf.clone();
    let (sender, message_receiver) = mpsc::channel(50);

    task::spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(event) => send_event_messages(&sender, event, &conf_for_move).await,
                Err(RecvError::Closed) => break,
                Err(RecvError::Lagged(m)) => error!("Receiver lagging. Skipped {} messages", m),
            }
        }
    });

    let discovery_reactor = HassioReactor::with();

    MqttReactor::with(message_receiver, &conf).await;
}

#[instrument(level = "debug")]
async fn send_event_messages(mqtt_client: &mpsc::Sender<Message>, event: Event, conf: &Configuration) {
    let messages = message::get_event_messages(event, conf);

    for message in messages.into_iter() {
        if let Err(e) = mqtt_client.send(message).await {
            error!("message was not sent: {}", e);
        }
    }
}
