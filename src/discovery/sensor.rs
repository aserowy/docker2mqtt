use serde::Serialize;

use super::device::Device;

#[derive(Serialize)]
pub struct Sensor {
    pub availability_topic: String,
    pub device: Device,
    pub icon: String,
    pub name: String,

    #[serde(skip_serializing)]
    pub payload: bool,

    pub payload_available: String,
    pub payload_not_available: String,
    pub platform: String,
    pub state_topic: String,
    pub unique_id: String,
}

impl Sensor {
    pub fn to_json(mut self) -> String {
        self.payload_available = match self.payload {
            true => String::from("ON"),
            false => String::from("OFF"),
        };

        self.payload_not_available = match self.payload {
            true => String::from("OFF"),
            false => String::from("ON"),
        };

        serde_json::to_string(&self).unwrap()
    }
}

// {
//     "unit_of_measurement": "%",
//     "icon": "mdi:memory",
//     "availability_topic": "iotlink/workgroup/desktop-uvakaql/lwt",
//     "state_topic": "iotlink/workgroup/desktop-uvakaql/windows-monitor/stats/memory/usage",
//     "name": "DESKTOP-UVAKAQL Memory Usage",
//     "unique_id": "desktop-uvakaql_memory_usage",
//     "payload_available": "ON",
//     "payload_not_available": "OFF",
//     "device": {
//       "identifiers": [
//         "DESKTOP-UVAKAQL_Memory"
//       ],
//       "name": "DESKTOP-UVAKAQL Memory",
//       "model": "WORKGROUP",
//       "manufacturer": "IOTLink 2.2.2.0"
//     },
//     "platform": "mqtt"
//   }
