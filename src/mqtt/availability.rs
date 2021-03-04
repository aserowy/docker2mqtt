use std::fmt;

use crate::docker::ContainerEvent;

#[derive(Clone, Debug)]
pub enum Availability {
    Online,
    Offline,
}

impl fmt::Display for Availability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_availability(container_event: &ContainerEvent) -> Availability {
    match container_event {
        ContainerEvent::Undefined => Availability::Offline,
        ContainerEvent::Create => Availability::Online,
        ContainerEvent::Destroy => Availability::Offline,
        ContainerEvent::Die => Availability::Online,
        ContainerEvent::Kill => Availability::Online,
        ContainerEvent::Pause => Availability::Online,
        ContainerEvent::Rename => Availability::Online,
        ContainerEvent::Restart => Availability::Online,
        ContainerEvent::Start => Availability::Online,
        ContainerEvent::Stop => Availability::Online,
        ContainerEvent::Unpause => Availability::Online,
        ContainerEvent::Prune => Availability::Offline,
    }
}
