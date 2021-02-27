use std::str;

use bollard::{
    container::{ListContainersOptions, StatsOptions},
    models::ContainerSummaryInner,
    Docker,
};
use futures_util::StreamExt;
use tracing::{error, instrument};

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
            result.push(Container {
                name: get_container_name(&container).to_owned(),
                stats: self.get_stats(&container).await,

                id: get_value_or_default(container.id),
                image: get_value_or_default(container.image),
                status: get_value_or_default(container.status),
            });
        }

        result
    }

    #[instrument(level = "debug")]
    async fn get_stats(&self, container: &ContainerSummaryInner) -> Stats {
        let container_id = match &container.id {
            Some(id) => id,
            None => return Stats::default(),
        };

        let stream = &mut self
            .client
            .stats(container_id, Some(StatsOptions { stream: false }))
            .take(1);

        let mut result = Stats::default();
        if let Some(Ok(stats)) = stream.next().await {
            result = Stats::new(stats);
        }

        result
    }
}

#[derive(Debug)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,

    pub stats: Stats,
}

#[derive(Debug)]
pub struct Stats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Stats {
    fn new(stats: bollard::container::Stats) -> Stats {
        let mut cpu_usage = 0.0;
        if let Some(usage) = calculate_cpu_usage(&stats) {
            cpu_usage = usage;
        }

        let mut memory_usage = 0.0;
        if let Some(usage) = calculate_memory_usage(&stats) {
            memory_usage = usage;
        }

        Stats {
            cpu_usage,
            memory_usage,
        }
    }

    fn default() -> Stats {
        Stats {
            cpu_usage: 0.0,
            memory_usage: 0.0,
        }
    }
}

fn get_container_name(container: &ContainerSummaryInner) -> &str {
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

fn get_value_or_default<T: Default>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => T::default(),
    }
}

fn calculate_cpu_usage(stats: &bollard::container::Stats) -> Option<f64> {
    if let Some(system_cpu_delta) = calculate_system_cpu_delta(stats) {
        Some(
            (calculate_cpu_delta(stats) as f64 / system_cpu_delta as f64)
                * number_cpus(stats) as f64
                * 100.0,
        )
    } else {
        None
    }
}

fn calculate_cpu_delta(stats: &bollard::container::Stats) -> u64 {
    stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
}

fn calculate_system_cpu_delta(stats: &bollard::container::Stats) -> Option<u64> {
    if let (Some(cpu), Some(pre)) = (
        stats.cpu_stats.system_cpu_usage,
        stats.precpu_stats.system_cpu_usage,
    ) {
        Some(cpu - pre)
    } else {
        None
    }
}

fn number_cpus(stats: &bollard::container::Stats) -> u64 {
    if let Some(cpus) = stats.cpu_stats.online_cpus {
        cpus
    } else {
        let empty = &[];
        stats
            .cpu_stats
            .cpu_usage
            .percpu_usage
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(empty)
            .len() as u64
    }
}

fn calculate_memory_usage(stats: &bollard::container::Stats) -> Option<f64> {
    let mut used_memory = 0;
    if let (Some(usage), Some(cached)) = (stats.memory_stats.usage, stats.memory_stats.stats) {
        used_memory = usage - cached.cache;
    }

    if let Some(available_memory) = stats.memory_stats.limit {
        Some((used_memory as f64 / available_memory as f64) * 100.0)
    } else {
        None
    }
}
