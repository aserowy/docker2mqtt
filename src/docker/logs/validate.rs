use bollard::{container::LogOutput, Docker};
use lazy_static::lazy_static;
use regex::Regex;
use tracing::warn;

use crate::{configuration::Configuration, docker::container, events::Event};

pub async fn target(event: &Event, client: &Docker, conf: &Configuration) -> bool {
    let container;
    match container::get_by_name(client, &event.container_name).await {
        Some(c) => container = c,
        None => return false,
    }

    // docker2mqtt should not stream his own logs generating logs streaming his on logs gene..
    if let Some(image) = &container.image {
        if image.contains("docker2mqtt") {
            return false;
        }
    }

    let container_name = container::get_name(&container);
    if conf
        .docker
        .stream_logs_container
        .iter()
        .any(|name| name.eq_ignore_ascii_case(container_name))
    {
        return true;
    }

    false
}

pub fn log(logs: &LogOutput) -> bool {
    lazy_static! {
        static ref LOG_VALIDATORS: Vec<Regex> = get_log_validation_regexes();
    }

    let log = format!("{}", logs);
    for rgx in LOG_VALIDATORS.iter() {
        if rgx.is_match(&log) {
            return true;
        }
    }
    false
}

fn get_log_validation_regexes() -> Vec<Regex> {
    let conf = Configuration::new();
    let mut validators = vec![];
    for rgx in conf.docker.stream_logs_filter.iter() {
        match Regex::new(rgx) {
            Ok(regex) => validators.push(regex),
            Err(e) => warn!("creating log validator (regex) failed: {}", e),
        }
    }

    validators
}
