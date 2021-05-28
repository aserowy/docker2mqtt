use tokio::sync::mpsc;
use tracing::error;

use crate::Configuration;

use super::{
    client::{MqttClient, MqttLoop},
    message::Message,
};

struct MqttActor {
    client: MqttClient,
    receiver: mpsc::Receiver<MqttMessage>,
}

enum MqttMessage {
    Send(Message),
}

impl MqttActor {
    async fn with(
        receiver: mpsc::Receiver<MqttMessage>,
        conf: &Configuration,
    ) -> (Self, MqttLoop) {
        let (client, keep) = MqttClient::new(conf).await;

        (MqttActor { client, receiver }, keep)
    }

    async fn handle(&mut self, message: MqttMessage) {
        match message {
            MqttMessage::Send(msg) => {
                self.client.send_message(msg).await;
            }
        }
    }
}

async fn run_mqtt_actor(mut actor: MqttActor) {
    while let Some(message) = actor.receiver.recv().await {
        actor.handle(message).await;
    }
}

#[derive(Clone, Debug)]
pub struct MqttHandle {
    sender: mpsc::Sender<MqttMessage>,
}

impl MqttHandle {
    pub async fn with(conf: &Configuration) -> Self {
        let (sender, receiver) = mpsc::channel(10);
        let (actor, keep) = MqttActor::with(receiver, conf).await;

        tokio::spawn(run_mqtt_actor(actor));
        tokio::spawn(keep.start_loop());

        MqttHandle { sender }
    }

    pub async fn send(&self, message: Message) {
        match self.sender.send(MqttMessage::Send(message)).await {
            Ok(_) => {}
            Err(e) => {
                error!("message was not sent: {}", e)
            }
        }
    }
}
