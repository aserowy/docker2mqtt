use std::{borrow::Borrow, process};

use enum_iterator::IntoEnumIterator;
use paho_mqtt as mqtt;
use rs_docker::{container::Container, Docker};

use crate::{discovery, lwt, state};
use crate::{sensor::Sensor, Args};

pub(crate) fn get_messages(
    docker: &Docker,
    container: &Container,
    args: &Args,
) -> Vec<(String, String)> {
    let mut messages: Vec<(String, String)> = Vec::new();
    messages.push((
        lwt::get_availability_topic(&args.client_id, container),
        lwt::get_lwt_payload(container),
    ));

    for sensor in Sensor::into_enum_iter() {
        match args.hass_discovery_prefix.borrow() {
            Some(hass_discovery_prefix) => messages.push((
                discovery::get_discovery_topic(
                    &hass_discovery_prefix,
                    &args.client_id,
                    container,
                    &sensor,
                ),
                discovery::get_discovery_payload(&args.client_id, container, &sensor),
            )),
            None => (),
        }

        messages.push((
            state::get_state_topic(&args.client_id, container, &sensor),
            state::get_state_payload(docker, container, &sensor),
        ));
    }

    messages
}

pub(crate) fn create_client(args: &Args) -> mqtt::AsyncClient {
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(args.mqtt_server_uri.to_owned())
        .client_id(args.client_id.to_owned())
        .finalize();

    let client = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|error| {
        println!("Error creating the client: {:?}", error);
        process::exit(1);
    });

    let conn_opts = mqtt::ConnectOptions::new();

    if let Err(error) = client.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", error);
        process::exit(1);
    }

    client
}

pub(crate) fn send_message(
    client: &mqtt::AsyncClient,
    topic: String,
    payload: String,
    args: &Args,
) -> () {
    let message = mqtt::Message::new(topic, payload, args.mqtt_qos);

    let tok = client.publish(message);
    if let Err(e) = tok.wait() {
        println!("Error sending message: {:?}", e);
    }
}

pub(crate) fn close_client(client: mqtt::AsyncClient) {
    client.disconnect(None).wait().unwrap();
}
