use reaktor::multiplier::Multiplier;
use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;
use crate::persistence::docker::DockerRepositoryHandle;

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
    let multiplier = Multiplier::new(mqtt_receiver).await;

    let docker_repo_handle = DockerRepositoryHandle::new(&conf);

    persistence::init_task(repo_init_sender, docker_repo_handle.clone()).await;
    docker::task(mqtt_sender, repo_init_receiver, &conf).await;
    persistence::state_task(multiplier.clone().await.receiver, docker_repo_handle).await;

    // must be the last task to start event loop
    mqtt::task(multiplier.receiver, &conf).await;
}
