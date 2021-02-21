use dockworker::{container::ContainerFilters, Docker};

pub struct DockerClient {
    client: Docker,
}

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
    fn new(stats: dockworker::stats::Stats) -> Stats {
        let mut cpu_usage = 0.0;
        if let Some(usage) = stats.cpu_usage() {
            cpu_usage = usage;
        }

        let mut memory_usage = 0.0;
        if let Some(usage) = stats.memory_usage() {
            memory_usage = usage;
        }

        Stats {
            cpu_usage,
            memory_usage,
        }
    }

    fn default() -> Stats {
        Stats {
            cpu_usage: 0.0,
            memory_usage: 0.0,
        }
    }
}

impl DockerClient {
    pub fn new() -> DockerClient {
        match Docker::connect_with_defaults() {
            Ok(client) => DockerClient { client },
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn get_containers(&self) -> Vec<Container> {
        let filter = ContainerFilters::new();

        let containers = match self.client.list_containers(Some(true), None, None, filter) {
            Ok(containers) => containers,
            Err(e) => {
                panic!("{}", e);
            }
        };

        let mut result = Vec::new();
        for container in containers {
            result.push(Container {
                id: container.Id.to_owned(),
                name: get_container_name(&container).to_owned(),
                image: container.Image.to_owned(),
                status: container.Status.to_owned(),
            });
        }

        result
    }

    pub fn get_stats(&self, container: &Container) -> Stats {
        let mut stats_reader = match self.client.stats(&container.id, Some(false), Some(true)) {
            Ok(rdr) => rdr,
            Err(_) => return Stats::default(),
        };

        let stats = match stats_reader.next() {
            Some(nxt) => match nxt {
                Ok(stts) => stts,
                Err(_) => return Stats::default(),
            },
            None => return Stats::default(),
        };

        Stats::new(stats)
    }
}

fn get_container_name(container: &dockworker::container::Container) -> &str {
    let container_name = &container.Names[0];
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
