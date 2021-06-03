use tokio::sync::{mpsc, oneshot};

use crate::{
    configuration::Configuration,
    docker::{initial::InitReactor, logs::LoggingReactor},
    events::Event,
    reaktor::{reducer::Reducer, relay::Relay},
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

    /*
    let (stats_sender, stats_receiver) = broadcast::channel(500);
    stats::source(event_streams_stats, stats_sender, docker_client.clone()).await;
        */

    let logging_reactor = LoggingReactor::with(reducer.receiver, docker_client, conf).await;

    Relay::with(logging_reactor.receiver, sender).await;
}
