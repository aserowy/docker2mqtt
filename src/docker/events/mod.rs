use std::collections::HashMap;

use bollard::{errors::Error, models::SystemEventsResponse, system::EventsOptions, Docker};
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};
use tracing::error;

use crate::events::{ContainerEvent, Event, EventType};

mod transition;

struct EventActor {
    sender: mpsc::Sender<Event>,
    client: Docker,
}

impl EventActor {
    fn new(sender: mpsc::Sender<Event>, client: Docker) -> Self {
        EventActor { sender, client }
    }

    async fn handle(&mut self, mut stream: impl Stream<Item = Vec<Event>> + Unpin) {
        while let Some(events) = stream.next().await {
            for event in events.into_iter() {
                if let Err(e) = self.sender.send(event).await {
                    error!("message was not sent: {}", e);
                }
            }
        }
    }

    async fn run(mut self) {
        let stream = get_event_response_stream(&self.client).filter_map(transition::to_events);

        self.handle(stream).await;
    }
}

#[derive(Debug)]
pub struct EventReactor {
    pub receiver: mpsc::Receiver<Event>,
}

impl EventReactor {
    pub async fn new(client: Docker) -> Self {
        let (sender, receiver) = mpsc::channel(50);
        let actor = EventActor::new(sender, client);

        tokio::spawn(actor.run());

        EventReactor { receiver }
    }
}

fn get_event_response_stream(
    client: &Docker,
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
