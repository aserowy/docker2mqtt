use tokio::sync::mpsc;

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

    let (sender, receiver) = mpsc::channel(100);

    persistence::spin_up(&conf).await;
    docker::spin_up(sender).await;
    mqtt::spin_up(receiver, conf).await;
}
