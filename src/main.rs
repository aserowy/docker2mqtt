use tokio::sync::{broadcast, oneshot};

use crate::configuration::Configuration;

mod configuration;
mod docker;
mod events;
mod logging;
mod mqtt;
mod persistence;

#[tokio::main]
async fn main() {
    let conf = Configuration::new();
    let _guards = logging::init(&conf);

    let (repo_init_sender, repo_init_receiver) = oneshot::channel();
    let (mqtt_sender, mqtt_receiver) = broadcast::channel(100);
    let repo_receiver = mqtt_sender.subscribe();

    let repo = persistence::create_repository(&conf);

    persistence::init_task(repo_init_sender, &*repo).await;
    docker::task(mqtt_sender, repo_init_receiver, &conf).await;
    persistence::state_task(repo_receiver, repo).await;

    // must be the last task to start event loop
    mqtt::task(mqtt_receiver, &conf).await;
}
