use tokio::{
    sync::mpsc,
    task,
};
use tracing::{error, instrument};

use crate::{
    configuration::Configuration,
    events::Event,
    reaktor::{multiplier::Multiplier, reducer::Reducer},
};

use self::{discovery::HassioReactor, message::Message, sending::MqttReactor};

mod availability;
mod client;
mod discovery;
mod message;
mod payload;
mod sending;
mod topic;

pub async fn task(receiver: mpsc::Receiver<Event>, conf: &Configuration) {
    let multiplier = Multiplier::with(receiver).await;

    let conf_for_move = conf.clone();
    let mut multiplier_for_move = multiplier.clone().await;
    let (sender, message_receiver) = mpsc::channel(50);
    task::spawn(async move {
        loop {
            if let Some(event) = multiplier_for_move.receiver.recv().await {
                send_event_messages(&sender, event, &conf_for_move).await;
            }
        }
    });

    let discovery_reactor = HassioReactor::with(multiplier.receiver, conf).await;
    let reducer = Reducer::with(vec![discovery_reactor.receiver, message_receiver]).await;

    MqttReactor::with(reducer.receiver, &conf).await;
}

#[instrument(level = "debug")]
async fn send_event_messages(
    mqtt_client: &mpsc::Sender<Message>,
    event: Event,
    conf: &Configuration,
) {
    let messages = message::get_event_messages(event, conf);

    for message in messages.into_iter() {
        if let Err(e) = mqtt_client.send(message).await {
            error!("message was not sent: {}", e);
        }
    }
}
