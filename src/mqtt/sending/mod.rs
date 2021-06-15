use tokio::sync::mpsc;

use crate::Configuration;

use super::{
    client::{MqttClient, MqttLoop},
    Message,
};

struct MqttActor {
    client: MqttClient,
    receiver: mpsc::Receiver<Message>,
}

impl MqttActor {
    async fn new(receiver: mpsc::Receiver<Message>, conf: &Configuration) -> (Self, MqttLoop) {
        let (client, keep) = MqttClient::new(conf).await;

        (Self { client, receiver }, keep)
    }

    async fn handle(&mut self, message: Message) {
        self.client.send_message(message).await;
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

#[derive(Debug)]
pub struct MqttReactor {}

impl MqttReactor {
    pub async fn new(receiver: mpsc::Receiver<Message>, conf: &Configuration) -> Self {
        let (actor, keep) = MqttActor::new(receiver, conf).await;

        tokio::spawn(actor.run());
        tokio::spawn(keep.start_loop());

        Self {}
    }
}
