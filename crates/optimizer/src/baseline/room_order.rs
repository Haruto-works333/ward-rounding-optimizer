use domain::{ProblemInput, Solution, StaffRole, Task, TaskRequirement, Visit};

use crate::common::{
    additional_travel_minutes, assign_pair_for_sync_task, earliest_start_for_append, StaffCandidate,
};

pub fn room_order_baseline(problem: &ProblemInput) -> Solution {
    let mut solution = Solution::with_empty_routes(&problem.staff);

    let mut tasks = problem.tasks.iter().collect::<Vec<_>>();
    tasks.sort_by(|a, b| {
        a.room_id
            .cmp(&b.room_id)
            .then_with(|| a.priority.rank().cmp(&b.priority.rank()))
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
                assign_pair_for_sync_task(problem, &mut solution, task)
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
