use bollard::models::{SystemEventsResponse, SystemEventsResponseActor};

use super::{ContainerEvent, Event, EventType};

pub fn to_events(response: SystemEventsResponse) -> Option<Vec<Event>> {
    let state_event = get_state_event(&response);

    let mut messages = vec![];
    match &state_event.event {
        EventType::State(ContainerEvent::Undefined) => return None,
        EventType::State(ContainerEvent::Create) => messages.push(get_image_event(&response)),
        _ => {}
    }

    messages.push(state_event);

    Some(messages)
}

fn get_state_event(response: &SystemEventsResponse) -> Event {
    Event {
        container_name: get_attribute(&response.actor, "name"),
        event: EventType::State(get_container_event(&response.action)),
    }
}

fn get_image_event(response: &SystemEventsResponse) -> Event {
    Event {
        container_name: get_attribute(&response.actor, "name"),
        event: EventType::Image(get_attribute(&response.actor, "image")),
    }
}

fn get_attribute(actor: &Option<SystemEventsResponseActor>, attribute: &str) -> String {
    actor
        .as_ref()
        .map_or(None, |response| response.attributes.as_ref())
        .map_or(None, |attributes| attributes.get(attribute))
        .map_or("".to_owned(), |name| name.to_owned())
}

fn get_container_event(action: &Option<String>) -> ContainerEvent {
    match action.as_deref() {
        Some("create") => ContainerEvent::Create,
        Some("destroy") => ContainerEvent::Destroy,
        Some("die") => ContainerEvent::Die,
        Some("kill") => ContainerEvent::Kill,
        Some("pause") => ContainerEvent::Pause,
        Some("rename") => ContainerEvent::Rename,
        Some("restart") => ContainerEvent::Restart,
        Some("start") => ContainerEvent::Start,
        Some("stop") => ContainerEvent::Stop,
        Some("unpause") => ContainerEvent::Unpause,
        Some("prune") => ContainerEvent::Prune,
        Some(_) => ContainerEvent::Undefined,
        None => ContainerEvent::Undefined,
    }
}

#[cfg(test)]
mod must {
    use bollard::models::SystemEventsResponse;

    #[test]
    fn return_none_if_action_is_undefined() {
        // arrange
        let response = SystemEventsResponse {
            ..Default::default()
        };

        // act
        let events = super::to_events(response);

        // assert
        assert!(events.is_none());
    }

    #[test]
    fn return_none_if_action_is_random_string() {
        // arrange
        let response = create_response("random");

        // act
        let events = super::to_events(response);

        // assert
        assert!(events.is_none());
    }

    #[test]
    fn return_messages_on_defined_actions() {
        // arrange
        let actions = vec![
            ("create", 2),
            ("destroy", 1),
            ("die", 1),
            ("kill", 1),
            ("pause", 1),
            ("rename", 1),
            ("restart", 1),
            ("start", 1),
            ("stop", 1),
            ("unpause", 1),
            ("prune", 1),
        ];

        let mut responses = vec![];
        for (action, result_count) in actions {
            responses.push((create_response(action), result_count));
        }

        // act && assert
        for (response, result_count) in responses {
            assert_eq!(
                result_count,
                super::to_events(response).unwrap().iter().count()
            );
        }
    }

    fn create_response(action: &str) -> SystemEventsResponse {
        SystemEventsResponse {
            action: Some(action.to_owned()),
            ..Default::default()
        }
    }
}
