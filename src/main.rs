use rs_docker::Docker;

mod discovery;
mod lwt;
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

    let sensors: Vec<String> = containers
        .iter()
        .map(|container| discovery::get_discovery_payload(host, container, "image"))
        .collect();

    for sensor in sensors {
        println!("{:?}", sensor);
    }
}
