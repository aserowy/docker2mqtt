use bollard::container::Stats;
use tokio::{
    sync::{mpsc, oneshot},
    task::{self, JoinHandle},
};
use tracing::error;

use crate::{
    docker::{
        client::{DockerHandle, DockerMessage},
        stats::{cpu, memory},
    },
    events::{Event, EventType},
};

pub async fn start_stats_stream(
    client: DockerHandle,
    event: Event,
    sender: mpsc::Sender<Event>,
) -> JoinHandle<()> {
    task::spawn(async move {
        let (response, receiver) = oneshot::channel();
        let message = DockerMessage::GetStatsStream {
            container_name: event.container_name.to_owned(),
            response,
        };

        client.handle(message).await;
        match receiver.await {
            Ok(mut stream) => {
                while let Some(result) = stream.recv().await {
                    send_stat_events(&event, &result, &sender).await;
                }
            }
            Err(e) => error!("failed receiving response for get log stream: {}", e),
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
