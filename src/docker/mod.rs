use tokio::sync::{mpsc, oneshot};

use crate::{
    configuration::Configuration,
    docker::{initial::InitReactor, logs::LoggingReactor, stats::StatsReactor},
    events::Event,
    reaktor::{multiplier::Multiplier, reducer::Reducer, relay::Relay},
};

use self::events::EventReactor;

mod client;
mod container;
mod events;
mod initial;
mod logs;
mod stats;

pub async fn task(
    sender: mpsc::Sender<Event>,
    repo_init_receiver: oneshot::Receiver<Vec<String>>,
    conf: &Configuration,
) {
    let docker_client = client::new();

    let init_reactor = InitReactor::with(repo_init_receiver, docker_client.clone()).await;
    let event_reactor = EventReactor::with(docker_client.clone()).await;

    let reducer = Reducer::with(vec![init_reactor.receiver, event_reactor.receiver]).await;
    let multiplier = Multiplier::with(reducer.receiver).await;

    let stats_reactor =
        StatsReactor::with(multiplier.clone().await.receiver, docker_client.clone()).await;

    let logging_reactor =
        LoggingReactor::with(multiplier.clone().await.receiver, docker_client, conf).await;

    let reducer = Reducer::with(vec![
        multiplier.receiver,
        stats_reactor.receiver,
        logging_reactor.receiver,
    ])
    .await;

    Relay::with(reducer.receiver, sender).await;
}
