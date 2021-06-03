use std::collections::HashSet;

use bollard::models::ContainerSummaryInner;

use crate::{
    docker::container,
    events::{ContainerEvent, Event, EventType},
};

pub fn from_container(container: ContainerSummaryInner) -> Vec<Event> {
    let container_name = container::get_name(&container).to_owned();

    let mut events = vec![
        Event {
            container_name: container_name.to_owned(),
            event: EventType::State(ContainerEvent::Create),
        },
        Event {
            container_name: container_name.to_owned(),
            event: EventType::State(get_state(&container)),
        },
    ];

    if let Some(image) = &container.image {
        events.push(Event {
            container_name,
            event: EventType::Image(image.to_owned()),
        });
    }

    events
}

fn get_state(container: &ContainerSummaryInner) -> ContainerEvent {
    match container.state.as_deref() {
        Some("created") => ContainerEvent::Create,
        Some("restarting") => ContainerEvent::Restart,
        Some("running") => ContainerEvent::Start,
        Some("removing") => ContainerEvent::Prune,
        Some("paused") => ContainerEvent::Pause,
        Some("exited") => ContainerEvent::Stop,
        Some("dead") => ContainerEvent::Die,
        Some(_) => ContainerEvent::Undefined,
        None => ContainerEvent::Undefined,
    }
}

pub fn from_orphaned(
    containers: &[ContainerSummaryInner],
) -> Box<dyn FnMut(&String) -> Option<Event>> {
    let docker_container_names: HashSet<String> = containers
        .iter()
        .map(|summary| container::get_name(&summary).to_owned())
        .collect();

    return Box::new(move |cn| {
        if docker_container_names.contains(cn) {
            return None;
        }

        Some(Event {
            container_name: cn.to_owned(),
            event: EventType::State(ContainerEvent::Destroy),
        })
    });
}

#[cfg(test)]
mod must {
    use bollard::models::ContainerSummaryInner;

    use crate::events::{ContainerEvent, Event, EventType};

    fn create_container_summary(name: String) -> ContainerSummaryInner {
        ContainerSummaryInner {
            names: Some(vec![name]),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn return_correct_remove_events_for_orphaned_containers() {
        let containers: Vec<ContainerSummaryInner> = vec!["first", "second"]
            .into_iter()
            .map(|c| create_container_summary(c.to_owned()))
            .collect();

        let filter = super::from_orphaned(&containers);
        let orphaned: Vec<Event> = vec![String::from("second"), String::from("third")]
            .iter()
            .filter_map(filter)
            .collect();

        let expected = Event {
            container_name: "third".to_owned(),
            event: EventType::State(ContainerEvent::Destroy),
        };
        assert_eq!(vec![expected], orphaned);
    }
}
