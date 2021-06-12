use tokio::{
    sync::{mpsc, oneshot},
    task::{self, JoinHandle},
};
use tracing::error;

use crate::{
    docker::{
        client::{DockerHandle, DockerMessage},
        logs::validate,
    },
    events::{Event, EventType},
};

pub async fn start(
    client: DockerHandle,
    event: Event,
    sender: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let (response, receiver) = oneshot::channel();
        let message = DockerMessage::GetLogStream {
            container_name: event.container_name.to_owned(),
            response,
        };

        client.handle(message).await;
        match receiver.await {
            Ok(mut stream) => {
                while let Some(result) = stream.recv().await {
                    if validate::log(&result) {
                        send_log_events(&event, &result, &sender).await;
                    }
                }
            }
            Err(e) => error!("failed receiving response for get log stream: {}", e),
        }
    })
}

async fn send_log_events(source: &Event, logs: &str, sender: &mpsc::Sender<Event>) {
    let event = get_log_event(source, logs);
    if let Err(e) = sender.send(event).await {
        error!("message was not sent: {}", e);
    }
}

fn get_log_event(event: &Event, logs: &str) -> Event {
    Event {
        container_name: event.container_name.to_owned(),
        event: EventType::Log(format!("{}", logs)),
    }
}
