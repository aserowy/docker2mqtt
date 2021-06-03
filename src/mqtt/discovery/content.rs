use tracing::instrument;

use crate::configuration::{Configuration, Hassio};

pub type HassioResult<T> = Result<T, HassioErr>;

#[derive(Debug)]
pub enum HassioErr {
    DiscoveryDisabled,
}

#[instrument(level = "debug")]
pub fn topic(container_name: &str, event_name: &str, conf: &Configuration) -> HassioResult<String> {
    let hassio = get_hassio(conf)?;
    let unique_id = super::payload::get_unique_id(conf, hassio, container_name, event_name);

    Ok(format!(
        "{}/sensor/docker2mqtt/{}/config",
        hassio.discovery_prefix, unique_id
    ))
}

#[instrument(level = "debug")]
pub fn payload(
    container_name: &str,
    event_name: &str,
    conf: &Configuration,
) -> HassioResult<String> {
    let hassio = get_hassio(conf)?;

    Ok(super::payload::create(
        container_name,
        event_name,
        conf,
        hassio,
    ))
}

// TODO: move check into message creation
fn get_hassio(conf: &Configuration) -> HassioResult<&Hassio> {
    match &conf.hassio {
        Some(hassio) => match hassio {
            Hassio {
                discovery: false, ..
            } => Err(HassioErr::DiscoveryDisabled),
            _ => Ok(hassio),
        },
        None => Err(HassioErr::DiscoveryDisabled),
    }
}
