use crate::{Minute, RoomId, Staff, StaffId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Visit {
    pub task_id: TaskId,
    pub room_id: RoomId,
    pub start_minute: Minute,
    pub end_minute: Minute,
}

impl Visit {
    pub fn new(
        task_id: impl Into<String>,
        room_id: impl Into<String>,
        start_minute: Minute,
        end_minute: Minute,
    ) -> Self {
        Self {
            task_id: TaskId::new(task_id),
            room_id: RoomId::new(room_id),
            start_minute,
            end_minute,
        }
    }

    pub fn duration_minutes(&self) -> i32 {
        self.end_minute.value() - self.start_minute.value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaffRoute {
    pub staff_id: StaffId,
    pub visits: Vec<Visit>,

    /// A route may keep a staff member occupied even when no task Visit is recorded.
    /// This is mainly used for the doctor-accompany baseline, where the doctor waits
    /// while the nurse performs nurse-side tasks.
    ///
    /// タスクの Visit が記録されていなくても、スタッフを拘束し続ける場合に使う。
    /// 主に doctor-accompany ベースラインで、看護師が看護師側タスクを行う間に
    /// 医師が待機しているケースを表現する。
    pub forced_active_until: Option<Minute>,
}

impl StaffRoute {
    pub fn new(staff_id: StaffId) -> Self {
        Self {
            staff_id,
            visits: Vec::new(),
            forced_active_until: None,
        }
    }

    pub fn last_visit(&self) -> Option<&Visit> {
        self.visits.last()
    }

    pub fn mark_active_until(&mut self, minute: Minute) {
        self.forced_active_until = Some(match self.forced_active_until {
            Some(current) => current.max(minute),
            None => minute,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    pub routes: Vec<StaffRoute>,
    pub unassigned_task_ids: Vec<TaskId>,
}

impl Solution {
    pub fn new(routes: Vec<StaffRoute>, unassigned_task_ids: Vec<TaskId>) -> Self {
        Self {
            routes,
            unassigned_task_ids,
        }
    }

    pub fn with_empty_routes(staff: &[Staff]) -> Self {
        Self {
            routes: staff
                .iter()
                .map(|s| StaffRoute::new(s.id.clone()))
                .collect(),
            unassigned_task_ids: Vec::new(),
        }
    }

    pub fn route_by_staff_id(&self, staff_id: &StaffId) -> Option<&StaffRoute> {
        self.routes.iter().find(|route| &route.staff_id == staff_id)
    }

    pub fn route_by_staff_id_mut(&mut self, staff_id: &StaffId) -> Option<&mut StaffRoute> {
        self.routes
            .iter_mut()
            .find(|route| &route.staff_id == staff_id)
    }
}
