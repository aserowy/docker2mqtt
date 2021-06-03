use tokio::sync::{mpsc, oneshot};

use crate::{
    configuration::Configuration,
    docker::{initial::InitReactor, logs::LoggingReactor},
    events::Event,
    reaktor::relay::Relay,
};

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

    /* let (init_sender, init_receiver) = broadcast::channel(500);
    let mut event_streams_stats = vec![init_sender.subscribe()];
    let mut event_streams_logs = vec![init_sender.subscribe()];

    initial::source(init_sender, repo_init_receiver, docker_client.clone()).await;

    let (event_sender, event_receiver) = broadcast::channel(500);
    event_streams_stats.push(event_sender.subscribe());
    event_streams_logs.push(event_sender.subscribe());

    events::source(event_sender, docker_client.clone()).await;

    let (stats_sender, stats_receiver) = broadcast::channel(500);
    stats::source(event_streams_stats, stats_sender, docker_client.clone()).await;

        let (logs_sender, logs_receiver) = broadcast::channel(500); */

    let logging_reactor = LoggingReactor::with(init_reactor.receiver, docker_client, conf).await;

    Relay::with(logging_reactor.receiver, sender).await;
}
