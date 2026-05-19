#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RoomId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StaffId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(pub String);

impl RoomId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl StaffId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl TaskId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
