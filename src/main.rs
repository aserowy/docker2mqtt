use tokio::sync::mpsc;

use crate::configuration::Configuration;

mod configuration;
mod docker;
mod logging;
mod mqtt;

#[tokio::main]
async fn main() {
    let _guards = logging::init();
    let conf = Configuration::new();

    let (sender, receiver) = mpsc::channel(100);

    docker::spin_up(sender, conf.clone()).await;
    mqtt::spin_up(receiver, conf).await;
}
