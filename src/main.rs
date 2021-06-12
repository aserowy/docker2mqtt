use reaktor::multiplier::Multiplier;
use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;

mod configuration;
mod docker;
mod events;
mod logging;
mod mqtt;
mod persistence;
mod reaktor;

#[tokio::main]
async fn main() {
    let conf = Configuration::new();
    let _guards = logging::init(&conf);

    let (repo_init_sender, repo_init_receiver) = oneshot::channel();
    let (mqtt_sender, mqtt_receiver) = mpsc::channel(100);
    let multiplier = Multiplier::with(mqtt_receiver).await;

    let repo = persistence::docker::create_repository(&conf);

    persistence::docker::init_task(repo_init_sender, &*repo).await;
    docker::task(mqtt_sender, repo_init_receiver, &conf).await;
    persistence::docker::state_task(multiplier.clone().await.receiver, repo).await;

    // must be the last task to start event loop
    mqtt::task(multiplier.receiver, &conf).await;
}
