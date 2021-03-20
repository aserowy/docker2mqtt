use std::collections::HashMap;

use bollard::{
    container::{Stats, StatsOptions},
    Docker,
};
use tokio::{
    sync::broadcast::{self, error::RecvError},
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub async fn source(
    mut event_receiver: broadcast::Receiver<Event>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
) {
    task::spawn(async move {
        let mut tasks = HashMap::new();
        loop {
            let receive = event_receiver.recv().await;
            match receive {
                Ok(event) => handle_event(event, &mut tasks, &client, &event_sender).await,
                Err(RecvError::Closed) => break,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }
        }
    });
}

async fn handle_event(
    event: Event,
    tasks: &mut HashMap<String, JoinHandle<()>>,
    client: &Docker,
    event_sender: &broadcast::Sender<Event>,
) {
    match &event.event {
        EventType::State(ContainerEvent::Start) => {
            tasks.insert(
                event.container_name.to_owned(),
                start_stats_stream(client.clone(), event.clone(), event_sender.clone()).await,
            );
        }
        EventType::State(ContainerEvent::Stop) => {
            stop_stats_stream(tasks, &event);
        }
        EventType::State(ContainerEvent::Die) => {
            stop_stats_stream(tasks, &event);
        }
        _ => {}
    }
}

async fn start_stats_stream(
    client: Docker,
    event: Event,
    sender: broadcast::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let mut stream = client.stats(&event.container_name, Some(StatsOptions { stream: true }));
        while let Some(result) = stream.next().await {
            match result {
                Ok(stats) => send_stat_events(&event, &stats, &sender),
                Err(e) => error!("failed to receive valid stats: {}", e),
            }
        }
    })
}

fn stop_stats_stream(tasks: &mut HashMap<String, task::JoinHandle<()>>, event: &Event) {
    if let Some(handle) = tasks.remove(&event.container_name) {
        handle.abort()
    }
}

fn send_stat_events(source: &Event, stats: &Stats, sender: &broadcast::Sender<Event>) {
    for event in get_stat_events(source, stats).into_iter() {
        match sender.send(event) {
            Ok(_) => {}
            Err(e) => {
                error!("message was not sent: {}", e)
            }
        }
    }
}

fn get_stat_events(event: &Event, stats: &Stats) -> Vec<Event> {
    vec![
        Event {
            container_name: event.container_name.to_owned(),
            event: EventType::CpuUsage(calculate_cpu_usage(stats)),
        },
        Event {
            container_name: event.container_name.to_owned(),
            event: EventType::MemoryUsage(calculate_memory_usage(stats)),
        },
    ]
}

fn calculate_cpu_usage(stats: &Stats) -> f64 {
    if let Some(system_cpu_delta) = calculate_system_cpu_delta(stats) {
        (calculate_cpu_delta(stats) as f64 / system_cpu_delta as f64)
            * number_cpus(stats) as f64
            * 100.0
    } else {
        0.0
    }
}

fn calculate_cpu_delta(stats: &Stats) -> u64 {
    stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage
}

fn calculate_system_cpu_delta(stats: &Stats) -> Option<u64> {
    if let (Some(cpu), Some(pre)) = (
        stats.cpu_stats.system_cpu_usage,
        stats.precpu_stats.system_cpu_usage,
    ) {
        Some(cpu - pre)
    } else {
        None
    }
}

fn number_cpus(stats: &Stats) -> u64 {
    if let Some(cpus) = stats.cpu_stats.online_cpus {
        cpus
    } else {
        let empty = &[];
        stats
            .cpu_stats
            .cpu_usage
            .percpu_usage
            .as_deref()
            .unwrap_or(empty)
            .len() as u64
    }
}

fn calculate_memory_usage(stats: &Stats) -> f64 {
    let mut used_memory = 0;
    if let (Some(usage), Some(cached)) = (stats.memory_stats.usage, stats.memory_stats.stats) {
        used_memory = usage - cached.cache;
    }

    if let Some(available_memory) = stats.memory_stats.limit {
        (used_memory as f64 / available_memory as f64) * 100.0
    } else {
        0.0
    }
}
