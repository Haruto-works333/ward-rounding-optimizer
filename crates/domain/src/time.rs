#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minute(pub i32);

impl Minute {
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    pub fn value(self) -> i32 {
        self.0
    }

    pub fn add_minutes(self, minutes: i32) -> Self {
        Self(self.0 + minutes)
    }
}
