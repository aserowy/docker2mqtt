use crate::{configuration::Configuration, sensor::Sensor};

use self::{client::MqttClient, discovery::HassioResult, message::Message};

pub mod client;
mod discovery;
mod message;
mod topic;

pub async fn send_sensor_messages<'a>(
    mqtt_client: &MqttClient,
    sensors: Vec<Sensor<'a>>,
    conf: &Configuration,
) -> () {
    let mut messages: Vec<Message> = sensors
        .into_iter()
        .flat_map(|sensor| get_sensor_messages(sensor, conf))
        .collect();

    messages.sort();
    messages.dedup();

    for message in messages {
        mqtt_client.send_message(message, conf).await;
    }
}

fn get_sensor_messages<'a>(sensor: Sensor<'a>, conf: &Configuration) -> Vec<Message> {
    let mut messages = vec![
        Message {
            topic: topic::availability(&sensor, conf),
            payload: sensor.availability.to_string(),
        },
        Message {
            topic: topic::state(&sensor, conf),
            payload: sensor.state.to_owned(),
        },
    ];

    match get_discovery_message(&sensor, conf) {
        Ok(message) => messages.push(message),
        Err(_) => {}
    }

    messages
}

fn get_discovery_message<'a>(sensor: &Sensor<'a>, conf: &Configuration) -> HassioResult<Message> {
    let topic = match discovery::topic(sensor, conf) {
        Ok(topic) => topic,
        Err(e) => return Err(e),
    };

    let payload = match discovery::payload(sensor, conf) {
        Ok(payload) => payload,
        Err(e) => return Err(e),
    };

    Ok(Message { topic, payload })
}
