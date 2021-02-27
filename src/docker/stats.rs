use bollard::{container::StatsOptions, models::ContainerSummaryInner, Docker};
use futures_util::StreamExt;
use tracing::instrument;

#[instrument(level = "debug")]
pub async fn get_stats(client: &Docker, container: &ContainerSummaryInner) -> Stats {
    let container_id = match &container.id {
        Some(id) => id,
        None => return Stats::default(),
    };

    let stream = &mut client
        .stats(container_id, Some(StatsOptions { stream: false }))
        .take(1);

    let mut result = Stats::default();
    if let Some(Ok(stats)) = stream.next().await {
        result = Stats::new(stats);
    }

    result
}

#[derive(Debug)]
pub struct Stats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Stats {
    pub fn new(stats: bollard::container::Stats) -> Stats {
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

    pub fn default() -> Stats {
        Stats {
            cpu_usage: 0.0,
            memory_usage: 0.0,
        }
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
