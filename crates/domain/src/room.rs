use crate::{Point, RoomId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub location: Point,
}

impl Room {
    pub fn new(id: impl Into<String>, name: impl Into<String>, location: Point) -> Self {
        Self {
            id: RoomId::new(id),
            name: name.into(),
            location,
        }
    }
}
