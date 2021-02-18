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
    pub cpu_usage: u64,
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

    pub fn get_containers(&mut self) -> Vec<Container> {
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

    pub fn get_stats(&mut self, container: &Container) -> Stats {
        match self.client.stats(&container.id, Some(false), Some(true)) {
            Ok(stats) => {
                for stat in stats {
                    println!("{:#?}", stat.unwrap());
                }
            }
            Err(_) => (), //Stats { cpu_usage: 0 },
        }

        Stats {
            cpu_usage: 0, //stats.,
        }
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
