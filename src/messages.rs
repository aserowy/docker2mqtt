use enum_iterator::IntoEnumIterator;
use std::borrow::Borrow;

use rs_docker::{container::Container, Docker};

use crate::{sensor::Sensor, Args};

mod discovery;
mod lwt;
mod state;
mod topic;

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
