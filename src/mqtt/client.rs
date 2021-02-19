use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};

use crate::Args;

use super::message::Message;

pub struct MqttClient {
    client: AsyncClient,
}

impl MqttClient {
    pub async fn new(args: &Args) -> (MqttClient, MqttLoop) {
        let mut options = MqttOptions::new(
            args.client_id.to_owned(),
            args.mqtt_host.to_owned(),
            args.mqtt_port,
        );
        options
            .set_clean_session(true)
            .set_connection_timeout(args.mqtt_op_timeout)
            .set_keep_alive(args.mqtt_keep_alive)
            .set_pending_throttle(Duration::from_secs(1));

        set_credentials(args, &mut options);

        let (client, eventloop) = AsyncClient::new(options, 100);

        (MqttClient { client }, MqttLoop { eventloop })
    }

    pub async fn send_message(&self, message: Message, args: &Args) {
        let tkn = &self
            .client
            .publish(message.topic, get_qos(args), true, message.payload)
            .await;

        match tkn {
            Err(e) => {
                panic!("{}", e);
            }
            _ => (),
        }
    }
}

fn set_credentials(args: &Args, options: &mut MqttOptions) -> () {
    let username = match &args.mqtt_username {
        Some(username) => username,
        None => return,
    };

    let password = match &args.mqtt_password {
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

fn get_qos(args: &Args) -> QoS {
    match args.mqtt_qos {
        0 => QoS::AtMostOnce,
        1 => QoS::ExactlyOnce,
        2 => QoS::AtLeastOnce,
        _ => panic!(
            "mqtt_qos invalid, must be between 0 and 2, but {} is configured",
            args.mqtt_qos
        ),
    }
}