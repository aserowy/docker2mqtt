use bollard::models::ContainerSummaryInner;
use tokio::sync::oneshot;
use tracing::error;

use crate::docker::client::DockerMessage;

use super::client::DockerHandle;

pub async fn get(client: &DockerHandle) -> Vec<ContainerSummaryInner> {
    let (response, receiver) = oneshot::channel();
    let message = DockerMessage::GetContainerSummaries { response };

    client.handle(message).await;
    match receiver.await {
        Ok(summary) => summary,
        Err(e) => {
            error!("failed receiving response for get log stream: {}", e);
            vec![]
        }
    }
}

pub async fn get_by_name(client: &DockerHandle, name: &str) -> Option<ContainerSummaryInner> {
    let (response, receiver) = oneshot::channel();
    let message = DockerMessage::GetContainerSummary {
        container_name: name.to_owned(),
        response,
    };

    client.handle(message).await;
    match receiver.await {
        Ok(summary) => summary,
        Err(e) => {
            error!("failed receiving response for get log stream: {}", e);
            None
        }
    }
}

pub fn get_name(container: &ContainerSummaryInner) -> &str {
    let container_names = match &container.names {
        Some(names) => names,
        None => return "",
    };

    let container_name = &container_names[0];
    let (first_char, remainder) = split_first_char_remainder(container_name);

    match first_char {
        "/" => remainder,
        _ => container_name,
    }
}

fn split_first_char_remainder(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}
