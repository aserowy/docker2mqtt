use rs_docker::Docker;

mod discovery;
mod lwt;
mod mqtt;
mod sensor;
mod state;
mod topic;

fn main() {
    let host = "hostname";

    let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let containers = match docker.get_containers(false) {
        Ok(containers) => containers,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let messages: Vec<(String, String)> = containers
        .iter()
        .flat_map(|container| mqtt::get_messages(host, &docker, container))
        .collect();

    for message in messages {
        println!("{:?}", message);
    }
}
