use crate::{Minute, Room, RoomId, Staff, StaffRole, Task, TaskId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoringWeights {
    pub doctor_work_minute_penalty: i32,
    pub doctor_travel_minute_penalty: i32,
    pub nurse_work_minute_penalty: i32,
    pub nurse_travel_minute_penalty: i32,
    pub unassigned_high_priority_penalty: i32,
    pub unassigned_normal_priority_penalty: i32,
    pub unassigned_low_priority_penalty: i32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            doctor_work_minute_penalty: 20,
            doctor_travel_minute_penalty: 10,
            nurse_work_minute_penalty: 3,
            nurse_travel_minute_penalty: 2,
            unassigned_high_priority_penalty: 1000,
            unassigned_normal_priority_penalty: 300,
            unassigned_low_priority_penalty: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemInput {
    pub rooms: Vec<Room>,
    pub tasks: Vec<Task>,
    pub staff: Vec<Staff>,
    pub depot: Point,
    pub planning_window: TimeWindow,
    pub scoring_weights: ScoringWeights,
}

impl ProblemInput {
    pub fn room_by_id(&self, room_id: &RoomId) -> Option<&Room> {
        self.rooms.iter().find(|room| &room.id == room_id)
    }

    pub fn task_by_id(&self, task_id: &TaskId) -> Option<&Task> {
        self.tasks.iter().find(|task| &task.id == task_id)
    }

    pub fn doctors(&self) -> impl Iterator<Item = &Staff> {
        self.staff
            .iter()
            .filter(|staff| staff.role == StaffRole::Doctor)
    }

    pub fn nurses(&self) -> impl Iterator<Item = &Staff> {
        self.staff
            .iter()
            .filter(|staff| staff.role == StaffRole::Nurse)
    }

    pub fn travel_minutes_between_rooms(
        &self,
        from_room_id: &RoomId,
        to_room_id: &RoomId,
    ) -> Option<i32> {
        let from = self.room_by_id(from_room_id)?;
        let to = self.room_by_id(to_room_id)?;

        Some((from.x - to.x).abs() + (from.y - to.y).abs())
    }

    pub fn travel_minutes_from_depot(&self, to_room_id: &RoomId) -> Option<i32> {
        let to = self.room_by_id(to_room_id)?;

        Some((self.depot.x - to.x).abs() + (self.depot.y - to.y).abs())
    }
}
