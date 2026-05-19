use domain::{Minute, ProblemInput, Solution, StaffId, StaffRole, Task, TaskRequirement, Visit};

pub fn greedy_optimizer(problem: &ProblemInput) -> Solution {
    let mut solution = Solution {
        routes: problem
            .staff
            .iter()
            .map(|staff| domain::StaffRoute::new(staff.id.0.clone()))
            .collect(),
        unassigned_task_ids: Vec::new(),
    };

    let mut sync_tasks = problem
        .tasks
        .iter()
        .filter(|task| task.requirement == TaskRequirement::DoctorAndNurseSync)
        .collect::<Vec<_>>();

    sync_tasks.sort_by(task_value_order);

    for task in sync_tasks {
        if !assign_to_best_doctor_nurse_pair(problem, &mut solution, task) {
            solution.unassigned_task_ids.push(task.id.clone());
        }
    }

    let mut doctor_tasks = problem
        .tasks
        .iter()
        .filter(|task| task.requirement == TaskRequirement::DoctorRequired)
        .collect::<Vec<_>>();

    doctor_tasks.sort_by(task_value_order);

    for task in doctor_tasks {
        if !assign_to_best_staff(problem, &mut solution, task, StaffRole::Doctor) {
            solution.unassigned_task_ids.push(task.id.clone());
        }
    }

    let mut nurse_side_tasks = problem
        .tasks
        .iter()
        .filter(|task| {
            matches!(
                task.requirement,
                TaskRequirement::NurseOnly | TaskRequirement::NurseCapable
            )
        })
        .collect::<Vec<_>>();

    nurse_side_tasks.sort_by(task_value_order);

    for task in nurse_side_tasks {
        if !assign_to_best_staff(problem, &mut solution, task, StaffRole::Nurse) {
            solution.unassigned_task_ids.push(task.id.clone());
        }
    }

    solution
}

fn task_value_order(a: &&Task, b: &&Task) -> std::cmp::Ordering {
    a.priority
        .cmp(&b.priority)
        .then_with(|| b.points.cmp(&a.points))
        .then_with(|| a.duration_minutes.cmp(&b.duration_minutes))
        .then_with(|| a.room_id.cmp(&b.room_id))
        .then_with(|| a.id.cmp(&b.id))
}

fn assign_to_best_staff(
    problem: &ProblemInput,
    solution: &mut Solution,
    task: &Task,
    role: StaffRole,
) -> bool {
    let candidate = problem
        .staff
        .iter()
        .filter(|staff| staff.role == role)
        .filter_map(|staff| {
            let route = solution.route_by_staff_id(&staff.id)?;

            best_insertion_for_route(problem, staff, route, task).map(|insertion| StaffCandidate {
                staff_id: staff.id.clone(),
                end: insertion.end,
                additional_travel: insertion.additional_travel,
                visits_after_insert: insertion.visits_after_insert,
            })
        })
        .min_by(|a, b| {
            a.end
                .cmp(&b.end)
                .then_with(|| a.additional_travel.cmp(&b.additional_travel))
                .then_with(|| a.staff_id.cmp(&b.staff_id))
        });

    let Some(candidate) = candidate else {
        return false;
    };

    let Some(route) = solution.route_by_staff_id_mut(&candidate.staff_id) else {
        return false;
    };

    route.visits = candidate.visits_after_insert;

    true
}

fn assign_to_best_doctor_nurse_pair(
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

fn best_insertion_for_route(
    problem: &ProblemInput,
    staff: &domain::Staff,
    route: &domain::StaffRoute,
    task: &Task,
) -> Option<RouteInsertion> {
    let mut visits = route.visits.clone();
    visits.sort_by_key(|visit| visit.start_minute);

    let mut candidates = Vec::new();

    for insert_index in 0..=visits.len() {
        let previous = insert_index
            .checked_sub(1)
            .and_then(|index| visits.get(index));

        let next = visits.get(insert_index);

        let start = match previous {
            Some(previous_visit) => {
                let travel_minutes =
                    problem.travel_minutes_between_rooms(&previous_visit.room_id, &task.room_id)?;

                previous_visit.end_minute.add_minutes(travel_minutes)
            }
            None => {
                let travel_minutes = problem.travel_minutes_from_depot(&task.room_id)?;
                problem.planning_window.start.add_minutes(travel_minutes)
            }
        };

        let end = start.add_minutes(task.duration_minutes);

        if !problem.planning_window.contains(start, end)
            || start < staff.available_from
            || staff.available_to < end
        {
            continue;
        }

        if let Some(next_visit) = next {
            let travel_to_next =
                problem.travel_minutes_between_rooms(&task.room_id, &next_visit.room_id)?;

            if end.add_minutes(travel_to_next) > next_visit.start_minute {
                continue;
            }
        }

        let added_travel = insertion_travel_delta(problem, previous, task, next)?;

        let mut visits_after_insert = visits.clone();
        visits_after_insert.insert(
            insert_index,
            Visit {
                task_id: task.id.clone(),
                room_id: task.room_id.clone(),
                start_minute: start,
                end_minute: end,
            },
        );

        candidates.push(RouteInsertion {
            end,
            additional_travel: added_travel,
            visits_after_insert,
        });
    }

    candidates.into_iter().min_by(|a, b| {
        a.end
            .cmp(&b.end)
            .then_with(|| a.additional_travel.cmp(&b.additional_travel))
    })
}

fn insertion_travel_delta(
    problem: &ProblemInput,
    previous: Option<&Visit>,
    task: &Task,
    next: Option<&Visit>,
) -> Option<i32> {
    let previous_to_task = match previous {
        Some(previous_visit) => {
            problem.travel_minutes_between_rooms(&previous_visit.room_id, &task.room_id)?
        }
        None => problem.travel_minutes_from_depot(&task.room_id)?,
    };

    let task_to_next = match next {
        Some(next_visit) => {
            problem.travel_minutes_between_rooms(&task.room_id, &next_visit.room_id)?
        }
        None => 0,
    };

    let previous_to_next = match (previous, next) {
        (Some(previous_visit), Some(next_visit)) => {
            problem.travel_minutes_between_rooms(&previous_visit.room_id, &next_visit.room_id)?
        }
        (None, Some(next_visit)) => problem.travel_minutes_from_depot(&next_visit.room_id)?,
        (_, None) => 0,
    };

    Some(previous_to_task + task_to_next - previous_to_next)
}

fn earliest_start_for_append(
    problem: &ProblemInput,
    route: &domain::StaffRoute,
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

fn additional_travel_minutes(
    problem: &ProblemInput,
    route: &domain::StaffRoute,
    task: &Task,
) -> Option<i32> {
    match route.last_visit() {
        Some(last_visit) => {
            problem.travel_minutes_between_rooms(&last_visit.room_id, &task.room_id)
        }
        None => problem.travel_minutes_from_depot(&task.room_id),
    }
}

fn push_visit(
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
struct StaffCandidate {
    staff_id: StaffId,
    end: Minute,
    additional_travel: i32,
    visits_after_insert: Vec<Visit>,
}

#[derive(Debug, Clone)]
struct PairCandidate {
    doctor_id: StaffId,
    nurse_id: StaffId,
    start: Minute,
    end: Minute,
    additional_travel: i32,
}

#[derive(Debug, Clone)]
struct RouteInsertion {
    end: Minute,
    additional_travel: i32,
    visits_after_insert: Vec<Visit>,
}
