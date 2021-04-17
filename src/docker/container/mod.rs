use bollard::models::ContainerSummaryInner;

pub fn get_name(container: &ContainerSummaryInner) -> &str {
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

