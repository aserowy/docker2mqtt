use std::collections::HashMap;

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

use crate::events::{ContainerEvent, Event, EventType};

mod cpu;
mod memory;

struct StatsActor {
    receiver: mpsc::Receiver<Event>,
    sender: mpsc::Sender<Event>,
    tasks: HashMap<String, JoinHandle<()>>,
    client: Docker,
}

impl StatsActor {
    fn with(
        receiver: mpsc::Receiver<Event>,
        sender: mpsc::Sender<Event>,
        tasks: HashMap<String, JoinHandle<()>>,
        client: Docker,
    ) -> Self {
        StatsActor {
            receiver,
            sender,
            tasks,
            client,
        }
    }

    async fn handle(&mut self, event: Event) {
        match &event.event {
            EventType::State(ContainerEvent::Start) => {
                self.tasks.insert(
                    event.container_name.to_owned(),
                    start_stats_stream(self.client.clone(), event.clone(), self.sender.clone())
                        .await,
                );
            }
            EventType::State(ContainerEvent::Stop) => {
                stop_stats_stream(&mut self.tasks, &event);
            }
            EventType::State(ContainerEvent::Die) => {
                stop_stats_stream(&mut self.tasks, &event);
            }
            _ => {}
        }
    }

    async fn run(mut self) {
        while let Some(message) = self.receiver.recv().await {
            self.handle(message).await;
        }
    }
}

#[derive(Debug)]
pub struct StatsReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl StatsReactor {
    pub async fn with(receiver: mpsc::Receiver<Event>, client: Docker) -> Self {
        let (sender, actor_receiver) = mpsc::channel(50);
        let actor = StatsActor::with(receiver, sender, HashMap::new(), client);

        tokio::spawn(actor.run());

        StatsReactor {
            receiver: actor_receiver,
        }
    }
}

async fn start_stats_stream(
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

fn stop_stats_stream(tasks: &mut HashMap<String, task::JoinHandle<()>>, event: &Event) {
    if let Some(handle) = tasks.remove(&event.container_name) {
        handle.abort()
    }
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
