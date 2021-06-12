use bollard::{
    container::{Stats, StatsOptions},
    Docker,
};
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tracing::error;

use crate::{docker::stats::{cpu, memory}, events::{Event, EventType}};

pub async fn start_stats_stream(
    client: Docker,
    event: Event,
    sender: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let mut stream = client.stats(&event.container_name, Some(StatsOptions { stream: true }));
        while let Some(result) = stream.next().await {
            match result {
                Ok(stats) => send_stat_events(&event, &stats, &sender).await,
                Err(e) => error!("failed to receive valid stats: {}", e),
            }
        }
    })
}

async fn send_stat_events(source: &Event, stats: &Stats, sender: &mpsc::Sender<Event>) {
    for event in get_stat_events(source, stats).into_iter() {
        if let Err(e) = sender.send(event).await {
            error!("message was not sent: {}", e);
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
