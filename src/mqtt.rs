use std::time::Duration;

use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};

use crate::Args;

pub struct MqttClient {
    client: rumqttc::AsyncClient,
}

impl MqttClient {
    pub async fn new(args: &Args) -> (MqttClient, MqttLoop) {
        let mut options = MqttOptions::new(
            args.client_id.to_owned(),
            args.mqtt_host.to_owned(),
            args.mqtt_port,
        );
        options.set_clean_session(true);
        options.set_connection_timeout(args.mqtt_op_timeout);
        options.set_keep_alive(args.mqtt_keep_alive);
        options.set_pending_throttle(Duration::from_secs(1));

        let (client, eventloop) = AsyncClient::new(options, 100);

        (MqttClient { client }, MqttLoop { eventloop })
    }

    pub async fn send_message(&self, topic: String, payload: String, args: &Args) {
        let tkn = &self
            .client
            .publish(topic, get_qos(args), true, payload)
            .await;

        match tkn {
            Err(e) => {
                panic!("{}", e);
            }
            _ => (),
        }
    }
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
