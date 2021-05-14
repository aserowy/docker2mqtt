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

use crate::events::{ContainerEvent, Event, EventType};

mod cpu;
mod memory;

pub async fn source(
    receivers: Vec<broadcast::Receiver<Event>>,
    event_sender: broadcast::Sender<Event>,
    client: Docker,
) {
    let (sender, mut receiver) = broadcast::channel(500);
    task::spawn(async move {
        let mut tasks = HashMap::new();
        loop {
            match receiver.recv().await {
                Ok(event) => handle_event(event, &mut tasks, &client, &event_sender).await,
                Err(RecvError::Closed) => break,
                Err(e) => {
                    error!("receive failed: {}", e);
                    continue;
                }
            }
        }
    });

    super::join_receivers(receivers, sender).await;
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
            event: EventType::CpuUsage(cpu::usage(&stats.precpu_stats, &stats.cpu_stats)),
        },
        Event {
            container_name: event.container_name.to_owned(),
            event: EventType::MemoryUsage(memory::usage(&stats.memory_stats)),
        },
    ]
}
