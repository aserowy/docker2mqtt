use enum_iterator::IntoEnumIterator;
use std::fmt;

#[derive(Debug, IntoEnumIterator)]
pub enum Sensor {
    Image,
    Status,
}

impl fmt::Display for Sensor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
