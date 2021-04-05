use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub container_name: String,
    pub event: EventType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    CpuUsage(f64),
    Image(String),
    Log(String),
    MemoryUsage(f64),
    State(ContainerEvent),
}

impl fmt::Display for EventType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            EventType::CpuUsage(_) => "cpu_usage",
            EventType::Image(_) => "image",
            EventType::Log(_) => "logs",
            EventType::MemoryUsage(_) => "memory_usage",
            EventType::State(_) => "state",
        };

        write!(formatter, "{}", value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContainerEvent {
    Undefined,

    Create,
    Destroy,
    Die,
    Kill,
    Pause,
    Rename,
    Restart,
    Start,
    Stop,
    Unpause,
    Prune,
}
