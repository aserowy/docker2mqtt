use std::collections::HashMap;

use bollard::{
    container::{ListContainersOptions, LogsOptions, Stats, StatsOptions},
    models::{ContainerSummaryInner, SystemEventsResponse},
    system::EventsOptions,
    Docker,
};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};
use tokio_stream::StreamExt;
use tracing::{error, warn};

#[derive(Debug)]
struct DockerActor {
    docker: Docker,
    receiver: mpsc::Receiver<DockerMessage>,
}

#[derive(Debug)]
pub enum DockerMessage {
    GetEventStream {
        response: oneshot::Sender<mpsc::Receiver<SystemEventsResponse>>,
    },
    GetContainerSummaries {
        response: oneshot::Sender<Vec<ContainerSummaryInner>>,
    },
    GetContainerSummary {
        container_name: String,
        response: oneshot::Sender<Option<ContainerSummaryInner>>,
    },
    GetLogStream {
        container_name: String,
        response: oneshot::Sender<mpsc::Receiver<String>>,
    },
    GetStatsStream {
        container_name: String,
        response: oneshot::Sender<mpsc::Receiver<Stats>>,
    },
}

impl DockerActor {
    fn new(receiver: mpsc::Receiver<DockerMessage>) -> Self {
        Self {
            docker: Docker::connect_with_unix_defaults().unwrap(),
            receiver,
        }
    }

    async fn handle(&mut self, message: DockerMessage) {
        match message {
            DockerMessage::GetEventStream { response } => {
                handle_get_event_stream(self.docker.clone(), response).await
            }
            DockerMessage::GetContainerSummaries { response } => {
                handle_get_container_summaries(self.docker.clone(), response).await
            }
            DockerMessage::GetContainerSummary {
                container_name,
                response,
            } => handle_get_container_summary(self.docker.clone(), container_name, response).await,
            DockerMessage::GetLogStream {
                container_name,
                response,
            } => handle_get_log_stream(self.docker.clone(), container_name, response).await,
            DockerMessage::GetStatsStream {
                container_name,
                response,
            } => handle_get_stats_stream(self.docker.clone(), container_name, response).await,
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

async fn handle_get_event_stream(
    client: Docker,
    response: oneshot::Sender<mpsc::Receiver<SystemEventsResponse>>,
) {
    let (sender, receiver) = mpsc::channel(50);

    task::spawn(async move {
        let mut stream = client.events(Some(get_event_options()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    if let Err(e) = sender.send(event).await {
                        error!("message was not sent: {}", e);
                    }
                }
                Err(e) => warn!("failed to receive event: {}", e),
            }
        }
    });

    if response.send(receiver).is_err() {
        error!("receiver dropped");
    }
}

fn get_event_options() -> EventsOptions<String> {
    let mut query = HashMap::new();
    query.insert("type".to_owned(), vec!["container".to_owned()]);

    EventsOptions::<String> {
        since: None,
        until: None,
        filters: query,
    }
}

async fn handle_get_container_summaries(
    client: Docker,
    response: oneshot::Sender<Vec<ContainerSummaryInner>>,
) {
    let filter = Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    match client.list_containers(filter).await {
        Ok(containers) => {
            if response.send(containers).is_err() {
                error!("receiver dropped");
            }
        }
        Err(e) => {
            error!("could not resolve containers: {}", e);
        }
    }
}

async fn handle_get_container_summary(
    client: Docker,
    container_name: String,
    response: oneshot::Sender<Option<ContainerSummaryInner>>,
) {
    let mut name_filter = HashMap::new();
    name_filter.insert("name".to_owned(), vec![container_name.to_owned()]);

    let filter = Some(ListContainersOptions::<String> {
        all: true,
        filters: name_filter,
        ..Default::default()
    });

    match client.list_containers(filter).await {
        Ok(mut containers) => {
            if response.send(containers.pop()).is_err() {
                error!("receiver dropped");
            }
        }
        Err(e) => {
            error!("could not resolve containers: {}", e);
        }
    }
}

async fn handle_get_log_stream(
    client: Docker,
    container_name: String,
    response: oneshot::Sender<mpsc::Receiver<String>>,
) {
    let (sender, receiver) = mpsc::channel(50);

    task::spawn(async move {
        let mut stream = client.logs(&container_name, Some(get_log_options()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(logs) => {
                    if let Err(e) = sender.send(format!("{}", logs)).await {
                        error!("message was not sent: {}", e);
                    }
                }
                Err(e) => warn!("failed to receive valid logs: {}", e),
            }
        }
    });

    if response.send(receiver).is_err() {
        error!("receiver dropped");
    }
}

async fn handle_get_stats_stream(
    client: Docker,
    container_name: String,
    response: oneshot::Sender<mpsc::Receiver<Stats>>,
) {
    let (sender, receiver) = mpsc::channel(50);

    task::spawn(async move {
        let mut stream = client.stats(&container_name, Some(StatsOptions { stream: true }));

        while let Some(result) = stream.next().await {
            match result {
                Ok(stats) => {
                    if let Err(e) = sender.send(stats).await {
                        error!("failed to send valid stats: {}", e);
                    }
                }
                Err(e) => warn!("failed to receive valid stats: {}", e),
            }
        }
    });

    if response.send(receiver).is_err() {
        error!("receiver dropped");
    }
}

fn get_log_options() -> LogsOptions<String> {
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

#[derive(Debug, Clone)]
pub struct DockerHandle {
    sender: mpsc::Sender<DockerMessage>,
}

impl DockerHandle {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = DockerActor::new(receiver);

        tokio::spawn(actor.run());

        Self { sender }
    }

    pub async fn handle(&self, message: DockerMessage) {
        if let Err(e) = self.sender.send(message).await {
            error!("message was not sent: {}", e);
        }
    }
}
