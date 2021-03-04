use std::collections::HashMap;

use bollard::{
    container::{Stats, StatsOptions},
    Docker,
};
use tokio::{sync::broadcast, task};
use tokio_stream::StreamExt;
use tracing::error;

use super::{ContainerEvent, Event, EventType};

pub async fn source(
    mut event_receiver: broadcast::Receiver<Event>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
) -> () {
    task::spawn(async move {
        let mut tasks = HashMap::new();
        loop {
            let receive = event_receiver.recv().await;
            let event: Event;
            match receive {
                Ok(evnt) => event = evnt,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }

            match &event.event {
                &EventType::Status(ContainerEvent::Start) => {
                    let client_clone = client.clone();
                    let event_sender_clone = event_sender.clone();
                    let event_clone = event.clone();

                    let task = task::spawn(async move {
                        handle_container_start(client_clone, event_clone, event_sender_clone)
                    });

                    tasks.insert(event.container_name, task);
                }
                &EventType::Status(ContainerEvent::Stop) => {
                    let task = tasks.remove(&event.container_name);

                    match task {
                        Some(handle) => handle.abort(),
                        None => {}
                    }
                }
                _ => {}
            }
        }
    });
}

async fn handle_container_start(client: Docker, event: Event, sender: broadcast::Sender<Event>) {
    let mut stream = client.stats(&event.container_name, Some(StatsOptions { stream: true }));

    loop {
        match stream.next().await {
            Some(Ok(stats)) => send_stat_events(&event, &stats, &sender),
            Some(Err(e)) => error!("failed to receive valid stats: {}", e),
            None => {}
        }
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
            .as_ref()
            .map(|v| v.as_slice())
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
