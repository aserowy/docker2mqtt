use std::cmp::Ordering;

#[derive(Eq)]
pub struct Message {
    pub topic: String,
    pub payload: String,
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.topic.cmp(&other.topic)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.topic == other.topic
    }
}
