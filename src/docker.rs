use bollard::{container::ListContainersOptions, Docker};
use tracing::{error, instrument};

use self::container::Container;

pub mod container;
mod stats;

#[derive(Debug)]
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    #[instrument(level = "debug")]
    pub fn new() -> DockerClient {
        match Docker::connect_with_unix_defaults() {
            Ok(client) => DockerClient { client },
            Err(e) => {
                error!("failed to create docker client: {}", e);
                panic!();
            }
        }
    }

    #[instrument(level = "debug")]
    pub async fn get_containers(&self) -> Vec<Container> {
        let filter = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = match self.client.list_containers(filter).await {
            Ok(containers) => containers,
            Err(e) => {
                error!("could not resolve containers: {}", e);
                vec![]
            }
        };

        let mut result = Vec::new();
        for container in containers {
            result.push(Container::new(&self.client, container).await);
        }

        result
    }
}
