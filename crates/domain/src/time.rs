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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: Minute,
    pub end: Minute,
}

impl TimeWindow {
    pub fn new(start: Minute, end: Minute) -> Self {
        Self { start, end }
    }

    pub fn duration_minutes(&self) -> i32 {
        self.end.value() - self.start.value()
    }

    pub fn contains(&self, start: Minute, end: Minute) -> bool {
        self.start <= start && end <= self.end
    }
}
