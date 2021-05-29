use tokio::sync::mpsc;

use crate::Configuration;

use super::{
    client::{MqttClient, MqttLoop},
    message::Message,
};

struct MqttActor {
    client: MqttClient,
    receiver: mpsc::Receiver<Message>,
}

impl MqttActor {
    async fn with(receiver: mpsc::Receiver<Message>, conf: &Configuration) -> (Self, MqttLoop) {
        let (client, keep) = MqttClient::new(conf).await;

        (MqttActor { client, receiver }, keep)
    }

    async fn handle(&mut self, message: Message) {
        self.client.send_message(message).await;
    }
}

async fn run_mqtt_actor(mut actor: MqttActor) {
    while let Some(message) = actor.receiver.recv().await {
        actor.handle(message).await;
    }
}

#[derive(Clone, Debug)]
pub struct MqttReactor {}

impl MqttReactor {
    pub async fn with(receiver: mpsc::Receiver<Message>, conf: &Configuration) -> Self {
        let (actor, keep) = MqttActor::with(receiver, conf).await;

        tokio::spawn(run_mqtt_actor(actor));
        tokio::spawn(keep.start_loop());

        MqttReactor {}
    }
}
