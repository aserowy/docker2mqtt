use std::collections::HashMap;

use bollard::{errors::Error, models::SystemEventsResponse, system::EventsOptions, Docker};
use tokio::{sync::broadcast, task};
use tokio_stream::{Stream, StreamExt};
use tracing::error;

use super::{ContainerEvent, Event, EventType};

mod transition;

pub async fn source(event_sender: broadcast::Sender<Event>, client: Docker) {
    task::spawn(async move {
        let stream = get_event_response_stream(client).filter_map(transition::to_events);

        receive_loop(stream, event_sender).await
    });
}

async fn receive_loop(
    mut stream: impl Stream<Item = Vec<Event>> + Unpin,
    event_sender: broadcast::Sender<Event>,
) {
    while let Some(events) = stream.next().await {
        for event in events.into_iter() {
            match event_sender.send(event) {
                Ok(_) => {}
                Err(e) => error!("event could not be send to event_router: {}", e),
            }
        }
    }
}

fn get_event_response_stream(
    client: Docker,
) -> impl Stream<Item = Result<SystemEventsResponse, Error>> {
    client.events(Some(get_options()))
}

fn get_options() -> EventsOptions<String> {
    let mut query = HashMap::new();
    query.insert("type".to_owned(), vec!["container".to_owned()]);

    EventsOptions::<String> {
        since: None,
        until: None,
        filters: query,
    }
}

#[cfg(test)]
mod must {
    use std::time::Duration;

    use tokio::{sync::broadcast, task};

    use super::super::{Event, EventType};

    #[tokio::test]
    async fn stop_receive_loop_if_stream_closed() {
        let stream = tokio_stream::empty();
        let (event_sender, _) = broadcast::channel(500);

        let result = tokio::time::timeout(
            Duration::from_millis(100),
            super::receive_loop(stream, event_sender),
        );

        if result.await.is_err() {
            panic!("future not closed in time");
        }
    }

    #[tokio::test]
    async fn pass_messages_through_receive_loop() {
        let stream = tokio_stream::iter(vec![
            vec![Event {
                container_name: "test1".to_owned(),
                event: EventType::CpuUsage(1.0),
            }],
            vec![Event {
                container_name: "test2".to_owned(),
                event: EventType::CpuUsage(2.0),
            }],
        ]);

        let (event_sender, mut receiver) = broadcast::channel(500);

        let result = tokio::time::timeout(
            Duration::from_millis(100),
            super::receive_loop(stream, event_sender),
        );

        task::spawn(async move {
            if result.await.is_err() {
                panic!("future not closed in time");
            }
        });

        assert_eq!("test1", receiver.recv().await.unwrap().container_name);
        assert_eq!("test2", receiver.recv().await.unwrap().container_name);
    }
}
