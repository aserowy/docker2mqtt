use rs_docker::Docker;

pub struct DockerClient {
    client: Docker,
}

pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
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
                name: get_container_name(&container).to_owned(),
                image: container.Image.to_owned(),
                status: container.Status.to_owned(),
            });
        }

        result
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
