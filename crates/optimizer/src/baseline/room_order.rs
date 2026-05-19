use domain::{Minute, ProblemInput, Solution, StaffId, StaffRole, Task, TaskRequirement, Visit};

pub fn room_order_baseline(problem: &ProblemInput) -> Solution {
    let mut solution = Solution {
        routes: problem
            .staff
            .iter()
            .map(|staff| domain::StaffRoute::new(staff.id.0.clone()))
            .collect(),
        unassigned_task_ids: Vec::new(),
    };

    let mut tasks = problem.tasks.iter().collect::<Vec<_>>();
    tasks.sort_by(|a, b| {
        a.room_id
            .cmp(&b.room_id)
            .then_with(|| a.priority.cmp(&b.priority))
            .then_with(|| a.id.cmp(&b.id))
    });

    for task in tasks {
        let assigned = match task.requirement {
            TaskRequirement::DoctorRequired => {
                assign_to_best_staff(problem, &mut solution, task, StaffRole::Doctor)
            }
            TaskRequirement::NurseOnly | TaskRequirement::NurseCapable => {
                assign_to_best_staff(problem, &mut solution, task, StaffRole::Nurse)
            }
            TaskRequirement::DoctorAndNurseSync => {
                assign_to_best_doctor_nurse_pair(problem, &mut solution, task)
            }
        };

        if !assigned {
            solution.unassigned_task_ids.push(task.id.clone());
        }
    }

    solution
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
            let start = earliest_start_for_append(problem, route, task)?;
            let end = start.add_minutes(task.duration_minutes);

            if !problem.planning_window.contains(start, end)
                || start < staff.available_from
                || staff.available_to < end
            {
                return None;
            }

            let additional_travel = additional_travel_minutes(problem, route, task)?;

            Some(StaffCandidate {
                staff_id: staff.id.clone(),
                start,
                end,
                additional_travel,
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

    route.visits.push(Visit {
        task_id: task.id.clone(),
        room_id: task.room_id.clone(),
        start_minute: candidate.start,
        end_minute: candidate.end,
    });

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

    let doctor_visit = Visit {
        task_id: task.id.clone(),
        room_id: task.room_id.clone(),
        start_minute: candidate.start,
        end_minute: candidate.end,
    };

    let nurse_visit = doctor_visit.clone();

    let Some(doctor_route) = solution.route_by_staff_id_mut(&candidate.doctor_id) else {
        return false;
    };
    doctor_route.visits.push(doctor_visit);

    let Some(nurse_route) = solution.route_by_staff_id_mut(&candidate.nurse_id) else {
        return false;
    };
    nurse_route.visits.push(nurse_visit);

    true
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

#[derive(Debug, Clone)]
struct StaffCandidate {
    staff_id: StaffId,
    start: Minute,
    end: Minute,
    additional_travel: i32,
}

#[derive(Debug, Clone)]
struct PairCandidate {
    doctor_id: StaffId,
    nurse_id: StaffId,
    start: Minute,
    end: Minute,
    additional_travel: i32,
}
