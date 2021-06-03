use bollard::{
    container::{LogOutput, LogsOptions},
    Docker,
};
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tracing::{error, warn};

use crate::{
    docker::logs::validate,
    events::{Event, EventType},
};

pub async fn start(client: Docker, event: Event, sender: mpsc::Sender<Event>) -> JoinHandle<()> {
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

async fn send_log_events(source: &Event, logs: &LogOutput, sender: &mpsc::Sender<Event>) {
    let event = get_log_event(source, logs);
    if let Err(e) = sender.send(event).await {
        error!("message was not sent: {}", e);
    }
}

fn get_log_event(event: &Event, logs: &LogOutput) -> Event {
    Event {
        container_name: event.container_name.to_owned(),
        event: EventType::Log(format!("{}", logs)),
    }
}
