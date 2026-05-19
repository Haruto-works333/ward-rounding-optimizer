use domain::{Minute, ProblemInput, Solution, StaffId, StaffRole, StaffRoute, Task, Visit};

pub fn earliest_start_for_append(
    problem: &ProblemInput,
    route: &StaffRoute,
    task: &Task,
) -> Option<Minute> {
    match route.last_visit() {
        Some(last_visit) => {
            let travel_minutes =
                problem.travel_minutes_between_rooms(&last_visit.room_id, &task.room_id)?;

            Some(last_visit.end_minute.add_minutes(travel_minutes))
        }
        None => {
            let travel_minutes = problem.travel_minutes_from_depot(&task.room_id)?;
            Some(problem.planning_window.start.add_minutes(travel_minutes))
        }
    }
}

pub fn additional_travel_minutes(
    problem: &ProblemInput,
    route: &StaffRoute,
    task: &Task,
) -> Option<i32> {
    match route.last_visit() {
        Some(last_visit) => {
            problem.travel_minutes_between_rooms(&last_visit.room_id, &task.room_id)
        }
        None => problem.travel_minutes_from_depot(&task.room_id),
    }
}

pub fn push_visit(
    solution: &mut Solution,
    staff_id: &StaffId,
    task: &Task,
    start: Minute,
    end: Minute,
) {
    let Some(route) = solution.route_by_staff_id_mut(staff_id) else {
        return;
    };

    route.visits.push(Visit {
        task_id: task.id.clone(),
        room_id: task.room_id.clone(),
        start_minute: start,
        end_minute: end,
    });
}

#[derive(Debug, Clone)]
pub struct StaffCandidate {
    pub staff_id: StaffId,
    pub start: Minute,
    pub end: Minute,
    pub additional_travel: i32,
}

#[derive(Debug, Clone)]
pub struct PairCandidate {
    pub doctor_id: StaffId,
    pub nurse_id: StaffId,
    pub start: Minute,
    pub end: Minute,
    pub additional_travel: i32,
}

pub fn assign_pair_for_sync_task(
    problem: &ProblemInput,
    solution: &mut Solution,
    task: &Task,
) -> bool {
    let mut candidates = Vec::new();

    for doctor in problem
        .staff
        .iter()
        .filter(|staff| staff.role == StaffRole::Doctor)
    {
        for nurse in problem
            .staff
            .iter()
            .filter(|staff| staff.role == StaffRole::Nurse)
        {
            let Some(doctor_route) = solution.route_by_staff_id(&doctor.id) else {
                continue;
            };

            let Some(nurse_route) = solution.route_by_staff_id(&nurse.id) else {
                continue;
            };

            let Some(doctor_start) = earliest_start_for_append(problem, doctor_route, task) else {
                continue;
            };

            let Some(nurse_start) = earliest_start_for_append(problem, nurse_route, task) else {
                continue;
            };

            let start = doctor_start.max(nurse_start);
            let end = start.add_minutes(task.duration_minutes);

            if !problem.planning_window.contains(start, end)
                || start < doctor.available_from
                || doctor.available_to < end
                || start < nurse.available_from
                || nurse.available_to < end
            {
                continue;
            }

            let Some(doctor_travel) = additional_travel_minutes(problem, doctor_route, task) else {
                continue;
            };

            let Some(nurse_travel) = additional_travel_minutes(problem, nurse_route, task) else {
                continue;
            };

            candidates.push(PairCandidate {
                doctor_id: doctor.id.clone(),
                nurse_id: nurse.id.clone(),
                start,
                end,
                additional_travel: doctor_travel + nurse_travel,
            });
        }
    }

    let Some(candidate) = candidates.into_iter().min_by(|a, b| {
        a.end
            .cmp(&b.end)
            .then_with(|| a.additional_travel.cmp(&b.additional_travel))
            .then_with(|| a.doctor_id.cmp(&b.doctor_id))
            .then_with(|| a.nurse_id.cmp(&b.nurse_id))
    }) else {
        return false;
    };

    push_visit(
        solution,
        &candidate.doctor_id,
        task,
        candidate.start,
        candidate.end,
    );

    push_visit(
        solution,
        &candidate.nurse_id,
        task,
        candidate.start,
        candidate.end,
    );

    true
}
