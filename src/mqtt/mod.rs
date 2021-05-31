use tokio::sync::mpsc;

use crate::{
    configuration::Configuration,
    events::Event,
    mqtt::mapping::MappingReactor,
    reaktor::{multiplier::Multiplier, reducer::Reducer},
};

use self::{discovery::HassioReactor, sending::MqttReactor};

mod availability;
mod client;
mod discovery;
mod mapping;
mod sending;
mod topic;

#[derive(Debug)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

pub async fn task(receiver: mpsc::Receiver<Event>, conf: &Configuration) {
    let multiplier = Multiplier::with(receiver).await;

    let mapping_reactor = MappingReactor::with(multiplier.clone().await.receiver, conf).await;
    let discovery_reactor = HassioReactor::with(multiplier.receiver, conf).await;

    let reducer = Reducer::with(vec![mapping_reactor.receiver, discovery_reactor.receiver]).await;

    MqttReactor::with(reducer.receiver, &conf).await;
}
