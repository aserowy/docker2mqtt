use std::collections::HashMap;

use bollard::Docker;

use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::{self},
};
use tracing::error;

use crate::{configuration::Configuration, events::Event};

mod handle;
mod stream;
mod validate;

pub async fn source(
    receivers: Vec<broadcast::Receiver<Event>>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
    conf: &Configuration,
) {
    if !conf.docker.stream_logs {
        return;
    }

    let conf = conf.clone();
    let (sender, mut receiver) = broadcast::channel::<Event>(500);
    task::spawn(async move {
        let mut tasks = HashMap::new();
        loop {
            match receiver.recv().await {
                Ok(event) => handle::event(event, &mut tasks, &client, &event_sender, &conf).await,
                Err(RecvError::Closed) => break,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }
        }
    });

    super::join_receivers(receivers, sender).await;
}
