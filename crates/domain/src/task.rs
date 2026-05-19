use crate::{RoomId, TaskId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
    Low,
}

impl Priority {
    /// Lower rank = higher priority. Explicit mapping decouples sort order
    /// from the enum declaration order, so reordering variants cannot silently
    /// flip optimizer behavior.
    ///
    /// rank が小さいほど高優先度。enum の宣言順に依存しないよう明示的に
    /// マッピングしており、variant の並び替えで optimizer の挙動が静かに
    /// 逆転することを防ぐ。
    pub fn rank(self) -> u8 {
        match self {
            Priority::High => 0,
            Priority::Normal => 1,
            Priority::Low => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskRequirement {
    DoctorRequired,
    NurseCapable,
    NurseOnly,
    DoctorAndNurseSync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: TaskId,
    pub room_id: RoomId,
    pub requirement: TaskRequirement,
    pub duration_minutes: i32,
    pub points: i32,
    pub priority: Priority,
}

impl Task {
    pub fn new(
        id: impl Into<String>,
        room_id: impl Into<String>,
        requirement: TaskRequirement,
        duration_minutes: i32,
        points: i32,
        priority: Priority,
    ) -> Self {
        Self {
            id: TaskId::new(id),
            room_id: RoomId::new(room_id),
            requirement,
            duration_minutes,
            points,
            priority,
        }
    }

    pub fn requires_doctor(&self) -> bool {
        matches!(
            self.requirement,
            TaskRequirement::DoctorRequired | TaskRequirement::DoctorAndNurseSync
        )
    }

    pub fn requires_nurse(&self) -> bool {
        matches!(
            self.requirement,
            TaskRequirement::NurseOnly
                | TaskRequirement::NurseCapable
                | TaskRequirement::DoctorAndNurseSync
        )
    }
}
