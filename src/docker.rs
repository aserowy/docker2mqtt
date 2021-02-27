use bollard::{container::ListContainersOptions, models::ContainerSummaryInner, Docker};
use tracing::{error, instrument};

#[derive(Debug)]
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    #[instrument(level = "debug")]
    pub fn new() -> DockerClient {
        match Docker::connect_with_local_defaults() {
            Ok(client) => DockerClient { client },
            Err(e) => {
                error!("failed to create docker client: {}", e);
                panic!();
            }
        }
    }

    #[instrument(level = "debug")]
    pub async fn get_containers(&self) -> Vec<Container> {
        let filter = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = match self.client.list_containers(filter).await {
            Ok(containers) => containers,
            Err(e) => {
                error!("could not resolve containers: {}", e);
                vec![]
            }
        };

        let mut result = Vec::new();
        for container in containers {
            result.push(Container {
                name: get_container_name(&container).to_owned(),
                id: get_value_or_default(container.id),
                image: get_value_or_default(container.image),
                status: get_value_or_default(container.status),
            });
        }

        result
    }

    #[instrument(level = "debug")]
    pub fn get_stats(&self, container: &Container) -> Stats {
        // let mut stats_reader = match self.client.stats(&container.id, Some(false), Some(true)) {
        //     Ok(rdr) => rdr,
        //     Err(e) => {
        //         error!("could not resolve stats: {}", e);
        //         return Stats::default();
        //     }
        // };

        // let stats = match stats_reader.next() {
        //     Some(nxt) => match nxt {
        //         Ok(stts) => stts,
        //         Err(e) => {
        //             error!("could not resolve stats: {}", e);
        return Stats::default();
        //         }
        //     },
        //     None => return Stats::default(),
        // };

        // Stats::new(stats)
    }
}

#[derive(Debug)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
}

pub struct Stats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Stats {
    // fn new(stats: dockworker::stats::Stats) -> Stats {
    //     let mut cpu_usage = 0.0;
    //     if let Some(usage) = stats.cpu_usage() {
    //         cpu_usage = usage;
    //     }

    //     let mut memory_usage = 0.0;
    //     if let Some(usage) = stats.memory_usage() {
    //         memory_usage = usage;
    //     }

    //     Stats {
    //         cpu_usage,
    //         memory_usage,
    //     }
    // }

    fn default() -> Stats {
        Stats {
            cpu_usage: 0.0,
            memory_usage: 0.0,
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
