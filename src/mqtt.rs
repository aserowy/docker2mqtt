use crate::{sensor::Sensor, Args};

use self::{client::MqttClient, discovery::HassioResult, message::Message};

pub mod client;
mod discovery;
mod message;
mod topic;

pub async fn send_messages_for<'a>(
    mqtt_client: &MqttClient,
    sensors: Vec<Sensor<'a>>,
    args: &Args,
) -> () {
    let mut messages: Vec<Message> = sensors
        .into_iter()
        .flat_map(|sensor| get_sensor_messages(sensor, args))
        .collect();

    messages.sort();
    messages.dedup();

    for message in messages {
        mqtt_client.send_message(message, args).await;
    }
}

fn get_sensor_messages<'a>(sensor: Sensor<'a>, args: &Args) -> Vec<Message> {
    let mut messages = vec![
        Message {
            topic: topic::availability(&sensor, args),
            payload: sensor.availability.to_string(),
        },
        Message {
            topic: topic::state(&sensor, args),
            payload: sensor.state.to_owned(),
        },
    ];

    match get_discovery_message(&sensor, args) {
        Ok(message) => messages.push(message),
        Err(_) => {}
    }

    messages
}

fn get_discovery_message<'a>(sensor: &Sensor<'a>, args: &Args) -> HassioResult<Message> {
    let topic = match discovery::topic(sensor, args) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(sensor, args) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}
