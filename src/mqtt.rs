use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};

use crate::Args;

pub(crate) async fn create_client(args: &Args) -> (rumqttc::AsyncClient, rumqttc::EventLoop) {
    let mut options = MqttOptions::new(
        args.client_id.to_owned(),
        args.mqtt_host.to_owned(),
        args.mqtt_port,
    );
    options.set_clean_session(true);
    options.set_connection_timeout(args.mqtt_op_timeout);
    options.set_keep_alive(args.mqtt_keep_alive);
    options.set_pending_throttle(Duration::from_secs(1));

    AsyncClient::new(options, 100)
}

pub(crate) async fn send_message(
    client: &rumqttc::AsyncClient,
    topic: String,
    payload: String,
    args: &Args,
) -> () {
    match client.publish(topic, get_qos(args), true, payload).await {
        Err(e) => {
            panic!("{}", e);
        }
        _ => (),
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
