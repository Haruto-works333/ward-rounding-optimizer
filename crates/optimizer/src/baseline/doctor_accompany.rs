use domain::{
    Minute, ProblemInput, RoomId, Solution, Staff, StaffId, StaffRole, Task, TaskRequirement,
};

use crate::placement::{push_visit, StaffCandidate};
use crate::routing::{additional_travel_minutes, earliest_start_for_append};

pub fn doctor_accompany_baseline(problem: &ProblemInput) -> Solution {
    let mut solution = Solution::with_empty_routes(&problem.staff);

    let Some(doctor) = problem
        .staff
        .iter()
        .find(|staff| staff.role == StaffRole::Doctor)
    else {
        solution.unassigned_task_ids = problem.tasks.iter().map(|task| task.id.clone()).collect();
        return solution;
    };

    let Some(accompanying_nurse) = problem
        .staff
        .iter()
        .find(|staff| staff.role == StaffRole::Nurse)
    else {
        solution.unassigned_task_ids = problem.tasks.iter().map(|task| task.id.clone()).collect();
        return solution;
    };

    let extra_nurses = problem
        .staff
        .iter()
        .filter(|staff| staff.role == StaffRole::Nurse && staff.id != accompanying_nurse.id)
        .collect::<Vec<_>>();

    let mut tasks = problem.tasks.iter().collect::<Vec<_>>();
    tasks.sort_by(|a, b| {
        a.room_id
            .cmp(&b.room_id)
            .then_with(|| a.priority.rank().cmp(&b.priority.rank()))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut pair_available_at = problem.planning_window.start;
    let mut pair_room_id: Option<RoomId> = None;

    for task in tasks {
        let assigned = match task.requirement {
            TaskRequirement::DoctorRequired => assign_doctor_task_with_pair(
                problem,
                &mut solution,
                doctor,
                accompanying_nurse,
                task,
                &mut pair_available_at,
                &mut pair_room_id,
            ),
            TaskRequirement::DoctorAndNurseSync => assign_sync_task_with_pair(
                problem,
                &mut solution,
                doctor,
                accompanying_nurse,
                task,
                &mut pair_available_at,
                &mut pair_room_id,
            ),
            TaskRequirement::NurseOnly | TaskRequirement::NurseCapable => {
                assign_nurse_task_with_pair(
                    problem,
                    &mut solution,
                    doctor,
                    accompanying_nurse,
                    task,
                    &mut pair_available_at,
                    &mut pair_room_id,
                ) || assign_to_extra_nurse(problem, &mut solution, &extra_nurses, task)
            }
        };

        if !assigned {
            solution.unassigned_task_ids.push(task.id.clone());
        }
    }

    solution
}

fn assign_doctor_task_with_pair(
    problem: &ProblemInput,
    solution: &mut Solution,
    doctor: &Staff,
    accompanying_nurse: &Staff,
    task: &Task,
    pair_available_at: &mut Minute,
    pair_room_id: &mut Option<RoomId>,
) -> bool {
    let Some(start) = earliest_pair_start(problem, *pair_available_at, pair_room_id.as_ref(), task)
    else {
        return false;
    };

    let end = start.add_minutes(task.duration_minutes);

    if !pair_can_work(problem, doctor, accompanying_nurse, start, end) {
        return false;
    }

    push_visit(solution, &doctor.id, task, start, end);

    // The accompanying nurse is occupied while following the doctor.
    // 同行看護師は医師に追随している間は拘束されているとみなす。
    mark_active_until(solution, &accompanying_nurse.id, end);

    *pair_available_at = end;
    *pair_room_id = Some(task.room_id.clone());

    true
}

fn assign_sync_task_with_pair(
    problem: &ProblemInput,
    solution: &mut Solution,
    doctor: &Staff,
    accompanying_nurse: &Staff,
    task: &Task,
    pair_available_at: &mut Minute,
    pair_room_id: &mut Option<RoomId>,
) -> bool {
    let Some(start) = earliest_pair_start(problem, *pair_available_at, pair_room_id.as_ref(), task)
    else {
        return false;
    };

    let end = start.add_minutes(task.duration_minutes);

    if !pair_can_work(problem, doctor, accompanying_nurse, start, end) {
        return false;
    }

    push_visit(solution, &doctor.id, task, start, end);
    push_visit(solution, &accompanying_nurse.id, task, start, end);

    *pair_available_at = end;
    *pair_room_id = Some(task.room_id.clone());

    true
}

fn assign_nurse_task_with_pair(
    problem: &ProblemInput,
    solution: &mut Solution,
    doctor: &Staff,
    accompanying_nurse: &Staff,
    task: &Task,
    pair_available_at: &mut Minute,
    pair_room_id: &mut Option<RoomId>,
) -> bool {
    let Some(start) = earliest_pair_start(problem, *pair_available_at, pair_room_id.as_ref(), task)
    else {
        return false;
    };

    let end = start.add_minutes(task.duration_minutes);

    if !pair_can_work(problem, doctor, accompanying_nurse, start, end) {
        return false;
    }

    // The nurse performs the task, while the doctor is occupied by accompanying/waiting.
    // 看護師がタスクを実施する一方、医師は同行/待機で拘束される。
    push_visit(solution, &accompanying_nurse.id, task, start, end);
    mark_active_until(solution, &doctor.id, end);

    *pair_available_at = end;
    *pair_room_id = Some(task.room_id.clone());

    true
}

fn assign_to_extra_nurse(
    problem: &ProblemInput,
    solution: &mut Solution,
    extra_nurses: &[&Staff],
    task: &Task,
) -> bool {
    let candidate = extra_nurses
        .iter()
        .filter_map(|nurse| {
            let route = solution.route_by_staff_id(&nurse.id)?;
            let start = earliest_start_for_append(problem, route, task)?;
            let end = start.add_minutes(task.duration_minutes);

            if !problem.planning_window.contains(start, end)
                || start < nurse.available_from
                || nurse.available_to < end
            {
                return None;
            }

            let additional_travel = additional_travel_minutes(problem, route, task)?;

            Some(StaffCandidate {
                staff_id: nurse.id.clone(),
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

    push_visit(
        solution,
        &candidate.staff_id,
        task,
        candidate.start,
        candidate.end,
    );

    true
}

fn earliest_pair_start(
    problem: &ProblemInput,
    pair_available_at: Minute,
    pair_room_id: Option<&RoomId>,
    task: &Task,
) -> Option<Minute> {
    let travel_minutes = match pair_room_id {
        Some(room_id) => problem.travel_minutes_between_rooms(room_id, &task.room_id)?,
        None => problem.travel_minutes_from_depot(&task.room_id)?,
    };

    Some(pair_available_at.add_minutes(travel_minutes))
}

fn pair_can_work(
    problem: &ProblemInput,
    doctor: &Staff,
    nurse: &Staff,
    start: Minute,
    end: Minute,
) -> bool {
    problem.planning_window.contains(start, end)
        && doctor.available_from <= start
        && end <= doctor.available_to
        && nurse.available_from <= start
        && end <= nurse.available_to
}

fn mark_active_until(solution: &mut Solution, staff_id: &StaffId, end: Minute) {
    let Some(route) = solution.route_by_staff_id_mut(staff_id) else {
        return;
    };

    route.mark_active_until(end);
}
