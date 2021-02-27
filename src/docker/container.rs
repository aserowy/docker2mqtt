use bollard::{models::ContainerSummaryInner, Docker};

use super::stats::{self, Stats};

#[derive(Debug)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,

    pub stats: Stats,
}

impl Container {
    pub async fn new(client: &Docker, container: ContainerSummaryInner) -> Container {
        Container {
            name: get_container_name(&container).to_owned(),
            stats: stats::get_stats(client, &container).await,

            id: get_value_or_default(container.id),
            image: get_value_or_default(container.image),
            status: get_value_or_default(container.status),
        }
    }
}

fn get_container_name(container: &ContainerSummaryInner) -> &str {
    let container_names = match &container.names {
        Some(names) => names,
        None => return "",
    };

    let container_name = &container_names[0];
    let (first_char, remainder) = split_first_char_remainder(container_name);

    match first_char {
        "/" => remainder,
        _ => container_name,
    }
}

fn split_first_char_remainder(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}

fn get_value_or_default<T: Default>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => T::default(),
    }
}
