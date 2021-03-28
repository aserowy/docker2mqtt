use tokio::sync::{broadcast, oneshot};

use crate::configuration::Configuration;

mod configuration;
mod docker;
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

    persistence::spin_up(repo_init_sender, repo_receiver, &conf).await;
    docker::spin_up(mqtt_sender, repo_init_receiver).await;
    mqtt::spin_up(mqtt_receiver, conf).await;
}
