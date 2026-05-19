use crate::RoomId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub x: i32,
    pub y: i32,
}

impl Room {
    pub fn new(id: impl Into<String>, name: impl Into<String>, x: i32, y: i32) -> Self {
        Self {
            id: RoomId::new(id),
            name: name.into(),
            x,
            y,
        }
    }
}
