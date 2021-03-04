use crate::docker::{ContainerEvent, Event, EventType};

pub fn get(event: &Event) -> String {
    match &event.event {
        EventType::CpuUsage(usage) => format!("{:.2}", usage),
        EventType::Image(image) => image.to_owned(),
        EventType::MemoryUsage(usage) => format!("{:.2}", usage),
        EventType::State(event) => get_status_payload(event).to_owned(),
    }
}

fn get_status_payload(event: &ContainerEvent) -> &str {
    match event {
        ContainerEvent::Undefined => "undefined",
        ContainerEvent::Create => "created",
        ContainerEvent::Destroy => "removing",
        ContainerEvent::Die => "dead",
        ContainerEvent::Kill => "exited",
        ContainerEvent::Pause => "paused",
        ContainerEvent::Rename => "running",
        ContainerEvent::Restart => "restarting",
        ContainerEvent::Start => "running",
        ContainerEvent::Stop => "exited",
        ContainerEvent::Unpause => "running",
        ContainerEvent::Prune => "removing",
    }
}
