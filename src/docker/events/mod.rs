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

#[cfg(test)]
mod must {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use tokio::{sync::broadcast, task};
    use tokio_stream::StreamExt;

    use super::super::{Event, EventType};

    #[test]
    fn filter_events_for_type_container_only() {
        // act
        let options = super::get_options();
        let mut filters = options.filters.into_iter();

        // assert
        assert_eq!(
            Some(("type".to_owned(), vec!["container".to_owned()])),
            filters.next()
        );

        assert_eq!(None, filters.next());
    }

    #[tokio::test]
    async fn stop_receive_loop_if_stream_closed() {
        // arrange
        let stream = tokio_stream::empty();
        let (event_sender, _) = broadcast::channel(500);

        // act
        let timeout = tokio::time::timeout(
            Duration::from_millis(100),
            super::receive_loop(stream, event_sender),
        );

        // assert
        if timeout.await.is_err() {
            panic!("future not closed in time");
        }
    }

    #[tokio::test]
    async fn pass_messages_through_receive_loop() {
        // arrange
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

        // act
        let timeout = tokio::time::timeout(
            Duration::from_millis(100),
            super::receive_loop(stream, event_sender),
        );

        task::spawn(async move {
            if timeout.await.is_err() {
                panic!("future not closed in time");
            }
        });

        // assert
        assert_eq!("test1", receiver.recv().await.unwrap().container_name);
        assert_eq!("test2", receiver.recv().await.unwrap().container_name);
    }

    #[tokio::test]
    async fn not_stop_sending_while_getting_errors() {
        // arrange
        let counter = Arc::new(Mutex::new(0));
        let counter_moved = counter.clone();

        let stream = tokio_stream::iter(vec![
            vec![Event {
                container_name: "test1".to_owned(),
                event: EventType::CpuUsage(1.0),
            }],
            vec![Event {
                container_name: "test2".to_owned(),
                event: EventType::CpuUsage(2.0),
            }],
        ])
        .map(|evnts| {
            let mut data = counter_moved.lock().unwrap();
            *data += 1;

            evnts
        });

        let (event_sender, receiver) = broadcast::channel(500);
        drop(receiver); // droping receiver enforces err while sending to channel

        // act
        let timeout = tokio::time::timeout(
            Duration::from_millis(100),
            super::receive_loop(stream, event_sender),
        );

        if timeout.await.is_err() {
            panic!("future not closed in time");
        }

        // assert
        assert_eq!(2, *counter.lock().unwrap());
    }
}
