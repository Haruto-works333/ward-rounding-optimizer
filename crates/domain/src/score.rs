#[derive(Debug, Clone, PartialEq)]
pub struct ScoreBreakdown {
    pub total_score: i32,

    pub earned_points: i32,

    pub doctor_work_minutes: i32,
    pub doctor_travel_minutes: i32,
    pub nurse_work_minutes: i32,
    pub nurse_travel_minutes: i32,

    pub doctor_active_minutes: i32,
    pub nurse_total_active_minutes: i32,
    pub nurse_avg_active_minutes: f64,

    pub points_per_doctor_minute: f64,

    pub unassigned_task_count: usize,
    pub unassigned_high_priority_count: usize,
    pub unassigned_normal_priority_count: usize,
    pub unassigned_low_priority_count: usize,
}

impl ScoreBreakdown {
    pub fn zero() -> Self {
        Self {
            total_score: 0,
            earned_points: 0,

            doctor_work_minutes: 0,
            doctor_travel_minutes: 0,
            nurse_work_minutes: 0,
            nurse_travel_minutes: 0,

            doctor_active_minutes: 0,
            nurse_total_active_minutes: 0,
            nurse_avg_active_minutes: 0.0,

            points_per_doctor_minute: 0.0,

            unassigned_task_count: 0,
            unassigned_high_priority_count: 0,
            unassigned_normal_priority_count: 0,
            unassigned_low_priority_count: 0,
        }
    }
}
