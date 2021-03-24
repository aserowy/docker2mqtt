use tokio::sync::{
    mpsc,
    oneshot
};

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
    let (repo_sender, repo_receiver) = mpsc::channel(100);
    let (mqtt_sender, mqtt_receiver) = mpsc::channel(100);

    persistence::spin_up(repo_init_sender, repo_receiver, &conf).await;
    docker::spin_up(mqtt_sender, repo_init_receiver, repo_sender).await;
    mqtt::spin_up(mqtt_receiver, conf).await;
}
