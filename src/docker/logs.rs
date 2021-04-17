use std::collections::HashMap;

use bollard::{
    container::{LogOutput, LogsOptions},
    Docker,
};
use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tracing::error;

use crate::{
    configuration::Configuration,
    events::{ContainerEvent, Event, EventType},
};

use super::container;

pub async fn source(
    receivers: Vec<broadcast::Receiver<Event>>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
    conf: &Configuration,
) {
    if !conf.docker.stream_logs {
        return;
    }

    let (sender, mut receiver) = broadcast::channel::<Event>(500);
    task::spawn(async move {
        let mut tasks = HashMap::new();
        loop {
            match receiver.recv().await {
                Ok(event) => handle_event(event, &mut tasks, &client, &event_sender).await,
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

async fn handle_event(
    event: Event,
    tasks: &mut HashMap<String, JoinHandle<()>>,
    client: &Docker,
    event_sender: &broadcast::Sender<Event>,
) {
    match &event.event {
        EventType::State(ContainerEvent::Start) => {
            if !is_target_valid(&event, client).await {
                return;
            }

            tasks.insert(
                event.container_name.to_owned(),
                start_logs_stream(client.clone(), event.clone(), event_sender.clone()).await,
            );
        }
        EventType::State(ContainerEvent::Stop) => {
            if !is_target_valid(&event, client).await {
                return;
            }

            stop_logs_stream(tasks, &event);
        }
        EventType::State(ContainerEvent::Die) => {
            if !is_target_valid(&event, client).await {
                return;
            }

            stop_logs_stream(tasks, &event);
        }
        _ => {}
    }
}

async fn is_target_valid(event: &Event, client: &Docker) -> bool {
    let result;
    match container::get_by_name(client, &event.container_name).await {
        Some(c) => result = c,
        None => return false,
    }

    // docker2mqtt should not stream his own logs generating logs streaming his on logs gene..
    if let Some(image) = result.image {
        if image.starts_with("docker2mqtt") {
            return false;
        }
    }

    true
}

async fn start_logs_stream(
    client: Docker,
    event: Event,
    sender: broadcast::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let mut stream = client.logs(
            &event.container_name,
            Some(LogsOptions {
                follow: true,
                stdout: true,
                stderr: false,
                tail: "all".to_owned(),
                ..Default::default()
            }),
        );

        while let Some(result) = stream.next().await {
            match result {
                Ok(logs) => send_log_events(&event, &logs, &sender),
                Err(e) => error!("failed to receive valid stats: {}", e),
            }
        }
    })
}

fn stop_logs_stream(tasks: &mut HashMap<String, task::JoinHandle<()>>, event: &Event) {
    if let Some(handle) = tasks.remove(&event.container_name) {
        handle.abort()
    }
}

fn send_log_events(source: &Event, logs: &LogOutput, sender: &broadcast::Sender<Event>) {
    match sender.send(get_log_event(source, logs)) {
        Ok(_) => {}
        Err(e) => {
            error!("message was not sent: {}", e)
        }
    }
}

fn get_log_event(event: &Event, logs: &LogOutput) -> Event {
    Event {
        container_name: event.container_name.to_owned(),
        event: EventType::Log(format!("{}", logs)),
    }
}
