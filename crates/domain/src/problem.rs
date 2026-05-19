use crate::{Point, Room, RoomId, ScoringWeights, Staff, StaffRole, Task, TaskId, TimeWindow};

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

        Some(from.location.manhattan(to.location))
    }

    pub fn travel_minutes_from_depot(&self, to_room_id: &RoomId) -> Option<i32> {
        let to = self.room_by_id(to_room_id)?;

        Some(self.depot.manhattan(to.location))
    }
}
