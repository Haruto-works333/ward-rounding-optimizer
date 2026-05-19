use std::collections::{HashMap, HashSet};

use domain::{ProblemInput, Solution, StaffId, StaffRole, TaskId, TaskRequirement, Visit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationKind {
    UnknownStaff,
    UnknownTask,
    UnknownRoom,
    InvalidVisitTime,
    DurationMismatch,
    StaffAvailabilityExceeded,
    StaffVisitOverlap,
    InsufficientTravelTime,
    DuplicateAssignment,
    RoleMismatch,
    SyncRequirementViolation,
    AssignmentStateMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub message: String,
}

impl Violation {
    pub fn new(kind: ViolationKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct AssignedVisit {
    staff_id: StaffId,
    role: StaffRole,
    visit: Visit,
}

#[must_use]
pub fn validate(solution: &Solution, problem: &ProblemInput) -> Vec<Violation> {
    let mut violations = Vec::new();

    let staff_by_id = problem
        .staff
        .iter()
        .map(|staff| (staff.id.clone(), staff))
        .collect::<HashMap<_, _>>();

    let task_by_id = problem
        .tasks
        .iter()
        .map(|task| (task.id.clone(), task))
        .collect::<HashMap<_, _>>();

    let room_ids = problem
        .rooms
        .iter()
        .map(|room| room.id.clone())
        .collect::<HashSet<_>>();

    let mut unassigned_task_ids = HashSet::new();

    for task_id in &solution.unassigned_task_ids {
        if !task_by_id.contains_key(task_id) {
            violations.push(Violation::new(
                ViolationKind::UnknownTask,
                format!("unknown unassigned task: {task_id:?}"),
            ));
        }

        if !unassigned_task_ids.insert(task_id.clone()) {
            violations.push(Violation::new(
                ViolationKind::AssignmentStateMismatch,
                format!("duplicated unassigned task id: {task_id:?}"),
            ));
        }
    }

    let mut assigned_by_task_id: HashMap<TaskId, Vec<AssignedVisit>> = HashMap::new();

    for route in &solution.routes {
        let Some(staff) = staff_by_id.get(&route.staff_id) else {
            violations.push(Violation::new(
                ViolationKind::UnknownStaff,
                format!("unknown staff in route: {:?}", route.staff_id),
            ));
            continue;
        };

        for visit in &route.visits {
            let Some(task) = task_by_id.get(&visit.task_id) else {
                violations.push(Violation::new(
                    ViolationKind::UnknownTask,
                    format!("unknown task in visit: {:?}", visit.task_id),
                ));
                continue;
            };

            if !room_ids.contains(&visit.room_id) {
                violations.push(Violation::new(
                    ViolationKind::UnknownRoom,
                    format!("unknown room in visit: {:?}", visit.room_id),
                ));
            }

            if visit.room_id != task.room_id {
                violations.push(Violation::new(
                    ViolationKind::AssignmentStateMismatch,
                    format!(
                        "visit room does not match task room: task={:?}, visit_room={:?}, task_room={:?}",
                        visit.task_id, visit.room_id, task.room_id
                    ),
                ));
            }

            if visit.start_minute >= visit.end_minute {
                violations.push(Violation::new(
                    ViolationKind::InvalidVisitTime,
                    format!("invalid visit time: {visit:?}"),
                ));
            }

            if visit.duration_minutes() != task.duration_minutes {
                violations.push(Violation::new(
                    ViolationKind::DurationMismatch,
                    format!(
                        "duration mismatch: task={:?}, expected={}, actual={}",
                        visit.task_id,
                        task.duration_minutes,
                        visit.duration_minutes()
                    ),
                ));
            }

            if !problem
                .planning_window
                .contains(visit.start_minute, visit.end_minute)
                || visit.start_minute < staff.available_from
                || staff.available_to < visit.end_minute
            {
                violations.push(Violation::new(
                    ViolationKind::StaffAvailabilityExceeded,
                    format!(
                        "staff availability exceeded: staff={:?}, visit={:?}",
                        route.staff_id, visit
                    ),
                ));
            }

            if !is_role_allowed(staff.role, task.requirement) {
                violations.push(Violation::new(
                    ViolationKind::RoleMismatch,
                    format!(
                        "role mismatch: staff={:?}, role={:?}, task={:?}, requirement={:?}",
                        route.staff_id, staff.role, visit.task_id, task.requirement
                    ),
                ));
            }

            assigned_by_task_id
                .entry(visit.task_id.clone())
                .or_default()
                .push(AssignedVisit {
                    staff_id: route.staff_id.clone(),
                    role: staff.role,
                    visit: visit.clone(),
                });
        }

        validate_route_timing(
            route.visits.as_slice(),
            problem,
            &route.staff_id,
            &mut violations,
        );
    }

    validate_assignment_state(
        &task_by_id,
        &assigned_by_task_id,
        &unassigned_task_ids,
        &mut violations,
    );

    violations
}

fn is_role_allowed(role: StaffRole, requirement: TaskRequirement) -> bool {
    match requirement {
        TaskRequirement::DoctorRequired => role == StaffRole::Doctor,
        TaskRequirement::NurseCapable => matches!(role, StaffRole::Doctor | StaffRole::Nurse),
        TaskRequirement::NurseOnly => role == StaffRole::Nurse,
        TaskRequirement::DoctorAndNurseSync => matches!(role, StaffRole::Doctor | StaffRole::Nurse),
    }
}

fn validate_route_timing(
    visits: &[Visit],
    problem: &ProblemInput,
    staff_id: &StaffId,
    violations: &mut Vec<Violation>,
) {
    if visits.is_empty() {
        return;
    }

    let mut visits = visits.to_vec();
    visits.sort_by_key(|visit| visit.start_minute);

    let first = &visits[0];

    if let Some(travel_minutes) = problem.travel_minutes_from_depot(&first.room_id) {
        let earliest_start = problem.planning_window.start.add_minutes(travel_minutes);

        if earliest_start > first.start_minute {
            violations.push(Violation::new(
                ViolationKind::InsufficientTravelTime,
                format!(
                    "insufficient travel time from depot: staff={staff_id:?}, visit={first:?}, required_start_at_least={earliest_start:?}"
                ),
            ));
        }
    }

    for pair in visits.windows(2) {
        let previous = &pair[0];
        let next = &pair[1];

        if next.start_minute < previous.end_minute {
            violations.push(Violation::new(
                ViolationKind::StaffVisitOverlap,
                format!(
                    "staff visit overlap: staff={staff_id:?}, previous={previous:?}, next={next:?}"
                ),
            ));
            continue;
        }

        let Some(travel_minutes) =
            problem.travel_minutes_between_rooms(&previous.room_id, &next.room_id)
        else {
            continue;
        };

        let earliest_next_start = previous.end_minute.add_minutes(travel_minutes);

        if earliest_next_start > next.start_minute {
            violations.push(Violation::new(
                ViolationKind::InsufficientTravelTime,
                format!(
                    "insufficient travel time: staff={staff_id:?}, previous={previous:?}, next={next:?}, required_start_at_least={earliest_next_start:?}"
                ),
            ));
        }
    }
}

fn validate_assignment_state(
    task_by_id: &HashMap<TaskId, &domain::Task>,
    assigned_by_task_id: &HashMap<TaskId, Vec<AssignedVisit>>,
    unassigned_task_ids: &HashSet<TaskId>,
    violations: &mut Vec<Violation>,
) {
    for (task_id, task) in task_by_id {
        let assigned_visits = assigned_by_task_id.get(task_id);
        let is_assigned = assigned_visits.is_some_and(|visits| !visits.is_empty());
        let is_unassigned = unassigned_task_ids.contains(task_id);

        if is_assigned && is_unassigned {
            violations.push(Violation::new(
                ViolationKind::AssignmentStateMismatch,
                format!("task is both assigned and unassigned: {task_id:?}"),
            ));
        }

        if !is_assigned && !is_unassigned {
            violations.push(Violation::new(
                ViolationKind::AssignmentStateMismatch,
                format!("task is neither assigned nor unassigned: {task_id:?}"),
            ));
        }

        let Some(assigned_visits) = assigned_visits else {
            continue;
        };

        match task.requirement {
            TaskRequirement::DoctorAndNurseSync => {
                validate_sync_task(task_id, assigned_visits, violations);
            }
            _ => {
                if assigned_visits.len() > 1 {
                    violations.push(Violation::new(
                        ViolationKind::DuplicateAssignment,
                        format!(
                            "non-sync task assigned multiple times: task={:?}, count={}",
                            task_id,
                            assigned_visits.len()
                        ),
                    ));
                }
            }
        }
    }
}

fn validate_sync_task(
    task_id: &TaskId,
    assigned_visits: &[AssignedVisit],
    violations: &mut Vec<Violation>,
) {
    let doctor_visits = assigned_visits
        .iter()
        .filter(|assigned| assigned.role == StaffRole::Doctor)
        .collect::<Vec<_>>();

    let nurse_visits = assigned_visits
        .iter()
        .filter(|assigned| assigned.role == StaffRole::Nurse)
        .collect::<Vec<_>>();

    if doctor_visits.len() > 1 || nurse_visits.len() > 1 {
        // Duplicate has precedence — the missing-side case (count == 0) is implicit in the counts.
        // 重複側を優先して報告する。片側欠落 (count == 0) の事象はメッセージ内の counts に表れる。
        violations.push(Violation::new(
            ViolationKind::DuplicateAssignment,
            format!(
                "sync task has duplicate role assignments: task={:?}, doctors={}, nurses={}",
                task_id,
                doctor_visits.len(),
                nurse_visits.len()
            ),
        ));
        return;
    }

    if doctor_visits.len() != 1 || nurse_visits.len() != 1 {
        violations.push(Violation::new(
            ViolationKind::SyncRequirementViolation,
            format!(
                "sync task must have exactly one doctor and one nurse: task={:?}, doctors={}, nurses={}",
                task_id,
                doctor_visits.len(),
                nurse_visits.len()
            ),
        ));
        return;
    }

    let doctor = doctor_visits[0];
    let nurse = nurse_visits[0];

    if doctor.visit.room_id != nurse.visit.room_id
        || doctor.visit.start_minute != nurse.visit.start_minute
        || doctor.visit.end_minute != nurse.visit.end_minute
    {
        violations.push(Violation::new(
            ViolationKind::SyncRequirementViolation,
            format!(
                "sync task visit mismatch: task={:?}, doctor_staff={:?}, nurse_staff={:?}",
                task_id, doctor.staff_id, nurse.staff_id
            ),
        ));
    }
}
