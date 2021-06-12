use std::collections::HashMap;

use bollard::{container::ListContainersOptions, models::ContainerSummaryInner};
use tracing::error;

use super::client::DockerHandle;

pub async fn get(client: &DockerHandle) -> Vec<ContainerSummaryInner> {
    let filter = Some(ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    });

    match client.list_containers(filter).await {
        Ok(containers) => containers,
        Err(e) => {
            error!("could not resolve containers: {}", e);
            vec![]
        }
    }
}

pub async fn get_by_name(client: &DockerHandle, name: &str) -> Option<ContainerSummaryInner> {
    let mut name_filter = HashMap::new();
    name_filter.insert("name".to_owned(), vec![name.to_owned()]);

    let filter = Some(ListContainersOptions::<String> {
        all: true,
        filters: name_filter,
        ..Default::default()
    });

    match client.list_containers(filter).await {
        Ok(mut containers) => containers.pop(),
        Err(e) => {
            error!("could not resolve containers: {}", e);
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
