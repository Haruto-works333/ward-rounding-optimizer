use crate::{Minute, StaffId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffRole {
    Doctor,
    Nurse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staff {
    pub id: StaffId,
    pub name: String,
    pub role: StaffRole,
    pub available_from: Minute,
    pub available_to: Minute,
}

impl Staff {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: StaffRole,
        available_from: Minute,
        available_to: Minute,
    ) -> Self {
        Self {
            id: StaffId::new(id),
            name: name.into(),
            role,
            available_from,
            available_to,
        }
    }

    pub fn is_doctor(&self) -> bool {
        self.role == StaffRole::Doctor
    }

    pub fn is_nurse(&self) -> bool {
        self.role == StaffRole::Nurse
    }
}
