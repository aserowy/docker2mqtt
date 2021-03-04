use tokio::{sync::mpsc::Receiver, task};
use tracing::{debug, instrument};

use crate::{
    configuration::Configuration,
    docker::{ContainerEvent, Event, EventType},
};

use self::{client::MqttClient, discovery::HassioResult, message::Message};

mod client;
mod discovery;
mod message;
mod topic;

pub async fn spin_up(mut receiver: Receiver<Event>, conf: Configuration) {
    let (mqtt_client, mqtt_loop) = MqttClient::new(&conf).await;

    task::spawn(async move {
        while let Some(event) = receiver.recv().await {
            send_event_messages(&mqtt_client, event, &conf).await;
        }
    });

    mqtt_loop.start_loop().await;
}

#[instrument(level = "debug")]
async fn send_event_messages(mqtt_client: &MqttClient, event: Event, conf: &Configuration) -> () {
    for message in get_event_messages(event, conf).into_iter() {
        mqtt_client.send_message(message, conf).await;
    }
}

fn get_event_messages(event: Event, conf: &Configuration) -> Vec<Message> {
    let mut messages = vec![];

    if let EventType::Status(ContainerEvent::Create) = event.event {
        match get_discovery_message(&event, conf) {
            Ok(message) => messages.push(message),
            Err(e) => debug!("discovery messages not generated: {:?}", e),
        }
    }

    messages.push(Message {
        topic: topic::availability(&event, conf),
        payload: event.availability.to_string(),
    });

    messages.push(Message {
        topic: topic::state(&event, conf),
        payload: get_payload(&event),
    });

    messages
}

fn get_discovery_message(event: &Event, conf: &Configuration) -> HassioResult<Message> {
    let topic = match discovery::topic(event, conf) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(event, conf) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}

fn get_payload(event: &Event) -> String {
    match &event.event {
        EventType::CpuUsage(usage) => format!("{:.2}", usage),
        EventType::Image(image) => image.to_owned(),
        EventType::MemoryUsage(usage) => format!("{:.2}", usage),
        EventType::Status(event) => get_status_payload(event).to_owned(),
    }
}

fn get_status_payload(event: &ContainerEvent) -> &str {
    match event {
        ContainerEvent::Undefined => "undefined",
        ContainerEvent::Create => "created",
        ContainerEvent::Destroy => "removing",
        ContainerEvent::Die => "dead",
        ContainerEvent::Kill => "removing",
        ContainerEvent::Pause => "paused",
        ContainerEvent::Rename => "running",
        ContainerEvent::Restart => "restarting",
        ContainerEvent::Start => "running",
        ContainerEvent::Stop => "exited",
        ContainerEvent::Unpause => "running",
        ContainerEvent::Prune => "removing",
    }
}
