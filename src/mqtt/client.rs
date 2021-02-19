use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};

use crate::configuration::Configuration;

use super::message::Message;

pub struct MqttClient {
    client: AsyncClient,
}

impl MqttClient {
    pub async fn new(conf: &Configuration) -> (MqttClient, MqttLoop) {
        let mut options = MqttOptions::new(
            conf.client_id.to_owned(),
            conf.mqtt_host.to_owned(),
            conf.mqtt_port,
        );
        options
            .set_clean_session(true)
            .set_connection_timeout(conf.mqtt_connection_timeout)
            .set_keep_alive(conf.mqtt_keep_alive)
            .set_pending_throttle(Duration::from_secs(1));

        set_credentials(conf, &mut options);

        let (client, eventloop) = AsyncClient::new(options, 100);

        (MqttClient { client }, MqttLoop { eventloop })
    }

    pub async fn send_message(&self, message: Message, conf: &Configuration) {
        let tkn = &self
            .client
            .publish(message.topic, get_qos(conf), true, message.payload)
            .await;

        match tkn {
            Err(e) => {
                panic!("{}", e);
            }
            _ => (),
        }
    }
}

fn set_credentials(conf: &Configuration, options: &mut MqttOptions) -> () {
    let username = match &conf.mqtt_username {
        Some(username) => username,
        None => return,
    };

    let password = match &conf.mqtt_password {
        Some(password) => password,
        None => return,
    };

    options.set_credentials(username, password);
}

pub struct MqttLoop {
    eventloop: EventLoop,
}

impl MqttLoop {
    pub async fn start_loop(mut self) {
        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    println!("Topic: {}, Payload: {:?}", p.topic, p.payload)
                }
                Ok(Event::Incoming(i)) => {
                    println!("Incoming = {:?}", i);
                }
                Ok(Event::Outgoing(o)) => println!("Outgoing = {:?}", o),
                Err(e) => {
                    println!("Error = {:?}", e);
                    continue;
                }
            }
        }
    }
}

fn get_qos(conf: &Configuration) -> QoS {
    match conf.mqtt_qos {
        0 => QoS::AtMostOnce,
        1 => QoS::ExactlyOnce,
        2 => QoS::AtLeastOnce,
        _ => panic!(
            "mqtt_qos invalid, must be between 0 and 2, but {} is configured",
            conf.mqtt_qos
        ),
    }
}
