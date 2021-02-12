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

    let sensors: Vec<discovery::Sensor> = containers
        .iter()
        .map(|container| discovery::map_container_to_sensor_discovery(host, "image", container))
        .collect();

    for sensor in sensors {
        println!("{:?}", sensor.to_json());
    }
}
