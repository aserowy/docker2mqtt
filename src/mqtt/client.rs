use core::panic;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use std::time::Duration;
use tracing::{error, instrument, trace};

use crate::configuration::Configuration;

use super::Message;

#[derive(Clone, Debug)]
pub struct MqttClient {
    client: AsyncClient,
    conf: Configuration,
}

impl MqttClient {
    #[instrument]
    pub async fn new(conf: &Configuration) -> (MqttClient, MqttLoop) {
        let mut options = MqttOptions::new(
            conf.mqtt.client_id.to_owned(),
            conf.mqtt.host.to_owned(),
            conf.mqtt.port,
        );
        options
            .set_clean_session(true)
            .set_connection_timeout(conf.mqtt.connection_timeout)
            .set_keep_alive(conf.mqtt.keep_alive)
            .set_pending_throttle(Duration::from_secs(1));

        set_credentials(conf, &mut options);

        let (client, eventloop) = AsyncClient::new(options, 100);
        let configuration = conf.clone();

        (MqttClient { client, conf: configuration, }, MqttLoop { eventloop })
    }

    #[instrument(level = "debug")]
    pub async fn send_message(&self, message: Message) {
        let tkn = &self
            .client
            .publish(message.topic, get_qos(&self.conf), true, message.payload)
            .await;

        if let Err(e) = tkn {
            error!("could not publish to mqtt broker: {}", e);
        }
    }
}

fn set_credentials(conf: &Configuration, options: &mut MqttOptions) {
    let username = match &conf.mqtt.username {
        Some(username) => username,
        None => return,
    };

    let password = match &conf.mqtt.password {
        Some(password) => password,
        None => return,
    };

    options.set_credentials(username, password);
}

pub struct MqttLoop {
    eventloop: EventLoop,
}

impl MqttLoop {
    #[instrument(skip(self))]
    pub async fn start_loop(mut self) {
        loop {
            match self.eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(p))) => {
                    trace!("incoming publish mqtt event: {}, {:?}", p.topic, p.payload)
                }
                Ok(_) => {}
                Err(e) => {
                    error!("could not connect to mqtt broker: {}", e);
                    continue;
                }
            }
        }
    }
}

fn get_qos(conf: &Configuration) -> QoS {
    match conf.mqtt.qos {
        0 => QoS::AtMostOnce,
        1 => QoS::ExactlyOnce,
        2 => QoS::AtLeastOnce,
        _ => {
            error!(
                "mqtt_qos invalid, must be between 0 and 2, but {} is configured",
                conf.mqtt.qos
            );

            panic!();
        }
    }
}
