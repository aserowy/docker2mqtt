use bollard::{container::LogsOptions, Docker};
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
    GetLogStream {
        container_name: String,
        response: oneshot::Sender<mpsc::Receiver<String>>,
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
            DockerMessage::GetLogStream {
                container_name,
                response,
            } => handle_get_log_stream(self.docker.clone(), container_name, response).await,
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
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
        let mut stream = client.logs(&container_name, Some(get_options()));

        while let Some(result) = stream.next().await {
            match result {
                Ok(logs) => {
                    if let Err(e) = sender.send(format!("{}", logs)).await {
                        error!("message was not sent: {}", e);
                    }
                }
                Ok(_) => {}
                Err(e) => warn!("failed to receive valid logs: {}", e),
            }
        }
    });

    if let Err(_) = response.send(receiver) {
        error!("receiver dropped");
    }
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

#[derive(Debug, Clone)]
pub struct DockerHandle {
    sender: mpsc::Sender<DockerMessage>,
}

impl DockerHandle {
    pub async fn new() -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = DockerActor::new(receiver);

        tokio::spawn(actor.run());

        DockerHandle { sender }
    }

    pub async fn handle(&self, message: DockerMessage) {
        if let Err(e) = self.sender.send(message).await {
            error!("message was not sent: {}", e);
        }
    }
}
