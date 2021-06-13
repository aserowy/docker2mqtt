use reaktor::multiplier::Multiplier;
use tokio::sync::{mpsc, oneshot};

use crate::configuration::Configuration;
use crate::persistence::docker::DockerDbHandle;

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

    let (db_init_sender, db_init_receiver) = oneshot::channel();
    let (mqtt_sender, mqtt_receiver) = mpsc::channel(100);
    let multiplier = Multiplier::new(mqtt_receiver).await;

    let docker_db_handle = DockerDbHandle::new(&conf);

    persistence::init_task(db_init_sender, docker_db_handle.clone()).await;
    docker::task(mqtt_sender, db_init_receiver, &conf).await;
    persistence::state_task(multiplier.clone().await.receiver, docker_db_handle).await;

    // must be the last task to start event loop
    mqtt::task(multiplier.receiver, &conf).await;
}
