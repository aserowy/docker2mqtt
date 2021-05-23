use bollard::{
    container::{LogOutput, LogsOptions},
    Docker,
};
use tokio::{
    sync::broadcast,
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tracing::{error, warn};

use crate::{
    docker::logs::validate,
    events::{Event, EventType},
};

pub async fn start(
    client: Docker,
    event: Event,
    sender: broadcast::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let mut stream = client.logs(&event.container_name, Some(get_options()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(logs) if validate::log(&logs) => {
                    send_log_events(&event, &logs, &sender);
                }
                Ok(_) => {}
                Err(e) => warn!("failed to receive valid stats: {}", e),
            }
        }
    })
}

fn get_options() -> LogsOptions<String> {
    LogsOptions::<String> {
        follow: true,
        stderr: true,
        stdout: true,
        // TODO persist time of last received logs and since then on startup
        tail: 0.to_string(),
        timestamps: true,
        ..Default::default()
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
