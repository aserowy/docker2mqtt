use bollard::Docker;
// use tokio::sync::mpsc;
use tracing::{error, instrument};

/* #[derive(Debug)]
struct DockerActor {
    receiver: mpsc::Receiver<DockerMessage>,
}

#[derive(Debug)]
enum DockerMessage {
}

impl DockerActor {
    fn new(receiver: mpsc::Receiver<DockerMessage>) -> Self {
        Self { receiver }
    }
} */

#[instrument(level = "debug")]
pub fn new() -> Docker {
    match Docker::connect_with_unix_defaults() {
        Ok(client) => client,
        Err(e) => {
            error!("failed to create docker client: {}", e);
            panic!();
        }
    }
}
