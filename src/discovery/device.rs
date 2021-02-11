use serde::Serialize;

#[derive(Serialize)]
pub struct Device {
    pub identifiers: Vec<String>,
    pub manufacturer: String,
    pub model: String,
    pub name: String,
}
