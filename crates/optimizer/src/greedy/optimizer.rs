use domain::{Minute, ProblemInput, Solution, StaffRole, Task, TaskRequirement, Visit};

use crate::common::assign_pair_for_sync_task;

pub fn greedy_optimizer(problem: &ProblemInput) -> Solution {
    let mut solution = Solution::with_empty_routes(&problem.staff);

    let mut sync_tasks = problem
        .tasks
        .iter()
        .filter(|task| task.requirement == TaskRequirement::DoctorAndNurseSync)
        .collect::<Vec<_>>();

    sync_tasks.sort_by(task_value_order);

    for task in sync_tasks {
        if !assign_pair_for_sync_task(problem, &mut solution, task) {
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
        .rank()
        .cmp(&b.priority.rank())
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

            best_insertion_for_route(problem, staff, route, task).map(|insertion| {
                InsertionCandidate {
                    staff_id: staff.id.clone(),
                    end: insertion.end,
                    additional_travel: insertion.additional_travel,
                    visits_after_insert: insertion.visits_after_insert,
                }
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

#[derive(Debug, Clone)]
struct InsertionCandidate {
    staff_id: domain::StaffId,
    end: Minute,
    additional_travel: i32,
    visits_after_insert: Vec<Visit>,
}

#[derive(Debug, Clone)]
struct RouteInsertion {
    end: Minute,
    additional_travel: i32,
    visits_after_insert: Vec<Visit>,
}
