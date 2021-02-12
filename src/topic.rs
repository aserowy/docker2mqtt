use rs_docker::container::Container;

pub fn resolve_base_topic(host: &str, container: &Container) -> String {
    let container_name = resolve_container_name(container);

    format!("docker2mqtt/{}/{}", host, container_name)
}

pub fn resolve_container_name(container: &Container) -> &str {
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
