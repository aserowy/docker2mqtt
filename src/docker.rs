use rs_docker::{container::HostConfig, Docker};

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
        match Docker::connect("unix:///var/run/docker.sock") {
            Ok(client) => DockerClient { client },
            Err(e) => {
                panic!("{}", e);
            }
        }
    }

    pub fn get_containers(&mut self) -> Vec<Container> {
        let containers = match self.client.get_containers(true) {
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
        let wrapper = rs_docker::container::Container {
            Id: container.id.to_owned(),
            Status: container.status.to_owned(),

            Image: "".to_owned(),
            Command: "".to_owned(),
            Created: 0,
            Names: vec![],
            Ports: vec![],
            SizeRw: None,
            SizeRootFs: 0,
            Labels: None,
            HostConfig: HostConfig {
                NetworkMode: "".to_owned(),
            },
        };

        match self.client.get_stats(&wrapper) {
            Ok(stats) => Stats {
                cpu_usage: stats.cpu_stats.cpu_usage.total_usage,
            },
            Err(_) => Stats { cpu_usage: 0 },
        }
    }
}

fn get_container_name(container: &rs_docker::container::Container) -> &str {
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
