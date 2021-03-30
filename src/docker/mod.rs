use std::fmt;

use futures::future::join_all;
use tokio::sync::{
    broadcast::{self, error::RecvError},
    oneshot,
};
use tracing::error;

mod client;
mod events;
mod initial;
mod stats;

pub async fn task(
    sender: broadcast::Sender<Event>,
    repo_init_receiver: oneshot::Receiver<Vec<String>>,
) {
    let docker_client = client::new();

    let (init_sender, init_receiver) = broadcast::channel(500);
    let mut stats_receivers = vec![init_sender.subscribe()];
    initial::source(init_sender, repo_init_receiver, docker_client.clone()).await;

    let (event_sender, event_receiver) = broadcast::channel(500);
    stats_receivers.push(event_sender.subscribe());
    events::source(event_sender, docker_client.clone()).await;

    let (stats_sender, stats_receiver) = broadcast::channel(500);
    stats::source(stats_receivers, stats_sender, docker_client.clone()).await;

    join_receivers(vec![init_receiver, event_receiver, stats_receiver], sender).await;
}

async fn join_receivers(
    receivers: Vec<broadcast::Receiver<Event>>,
    sender: broadcast::Sender<Event>,
) {
    let mut handles = vec![];
    for receiver in receivers {
        let sender_clone = sender.clone();
        handles.push(handle_receiver(receiver, sender_clone));
    }
    join_all(handles).await;
}

async fn handle_receiver(
    mut receiver: broadcast::Receiver<Event>,
    sender: broadcast::Sender<Event>,
) {
    loop {
        let receive = receiver.recv().await;
        let event: Event;
        match receive {
            Ok(evnt) => event = evnt,
            Err(RecvError::Closed) => break,
            Err(e) => {
                error!("receive failed: {}", e);
                continue;
            }
        }

        match sender.send(event) {
            Ok(_) => {}
            Err(e) => {
                error!("message was not sent: {}", e)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub container_name: String,
    pub event: EventType,
}

#[derive(Clone, Debug)]
pub enum EventType {
    CpuUsage(f64),
    Image(String),
    MemoryUsage(f64),
    State(ContainerEvent),
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            EventType::CpuUsage(_) => "cpu_usage",
            EventType::Image(_) => "image",
            EventType::MemoryUsage(_) => "memory_usage",
            EventType::State(_) => "state",
        };

        write!(formatter, "{}", value)
    }
}

#[derive(Clone, Debug)]
pub enum ContainerEvent {
    Undefined,

    Create,
    Destroy,
    Die,
    Kill,
    Pause,
    Rename,
    Restart,
    Start,
    Stop,
    Unpause,
    Prune,
}

#[cfg(test)]
mod must {
    use std::time::Duration;

    use tokio::{sync::broadcast, task};

    use super::{Event, EventType};

    #[tokio::test]
    async fn stop_join_receviers_if_all_channels_closed() {
        // arrange
        let (sender1, receiver1) = broadcast::channel(100);
        let (sender2, receiver2) = broadcast::channel(100);

        let (sender3, _) = broadcast::channel(100);

        // act
        let timeout = tokio::time::timeout(
            Duration::from_millis(100),
            super::join_receivers(vec![receiver1, receiver2], sender3),
        );

        drop(sender1);
        drop(sender2);

        // assert
        if timeout.await.is_err() {
            panic!("future not closed in time");
        }
    }

    #[tokio::test]
    async fn send_messages_from_all_receivers_while_one_is_closed() {
        // arrange
        let (sender1, receiver1) = broadcast::channel(100);
        let (sender2, receiver2) = broadcast::channel(100);
        let (sender3, receiver3) = broadcast::channel(100);

        let (sender4, mut receiver4) = broadcast::channel(100);

        drop(sender1);

        // act
        let timeout = tokio::time::timeout(
            Duration::from_millis(100),
            super::join_receivers(vec![receiver1, receiver2, receiver3], sender4),
        );

        task::spawn(async move {
            if timeout.await.is_err() {
                panic!("future not closed in time");
            }
        });

        sender2
            .send(Event {
                container_name: "container1".to_owned(),
                event: EventType::CpuUsage(0.0),
            })
            .unwrap();

        sender3
            .send(Event {
                container_name: "container2".to_owned(),
                event: EventType::CpuUsage(1.0),
            })
            .unwrap();

        // assert
        assert_eq!("container1", receiver4.recv().await.unwrap().container_name);
        assert_eq!("container2", receiver4.recv().await.unwrap().container_name);
        assert!(receiver4.try_recv().is_err());
    }
}
