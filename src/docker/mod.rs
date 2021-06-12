use tokio::sync::{mpsc, oneshot};

use crate::{configuration::Configuration, docker::{client::DockerHandle, initial::InitReactor, logs::LoggingReactor, stats::StatsReactor}, events::Event, reaktor::{multiplier::Multiplier, reducer::Reducer, relay::Relay}};

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
    let docker_client = DockerHandle::new().await;

    let init_reactor = InitReactor::new(repo_init_receiver, docker_client.clone()).await;
    let event_reactor = EventReactor::new(docker_client.clone()).await;

    let reducer = Reducer::new(vec![init_reactor.receiver, event_reactor.receiver]).await;
    let multiplier = Multiplier::new(reducer.receiver).await;

    let stats_reactor =
        StatsReactor::new(multiplier.clone().await.receiver, docker_client.clone()).await;

    let logging_reactor =
        LoggingReactor::new(multiplier.clone().await.receiver, docker_client, conf).await;

    let reducer = Reducer::new(vec![
        multiplier.receiver,
        stats_reactor.receiver,
        logging_reactor.receiver,
    ])
    .await;

    Relay::new(reducer.receiver, sender).await;
}
