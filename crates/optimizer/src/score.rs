use std::collections::HashSet;

use domain::{Priority, ProblemInput, ScoreBreakdown, Solution, StaffRole, TaskId};

pub fn score(solution: &Solution, problem: &ProblemInput) -> ScoreBreakdown {
    let mut earned_task_ids = HashSet::<TaskId>::new();

    let mut doctor_work_minutes = 0;
    let mut doctor_travel_minutes = 0;
    let mut nurse_work_minutes = 0;
    let mut nurse_travel_minutes = 0;

    let mut doctor_active_minutes = 0;
    let mut nurse_total_active_minutes = 0;
    let mut nurse_count = 0;

    for route in &solution.routes {
        let Some(staff) = problem
            .staff
            .iter()
            .find(|staff| staff.id == route.staff_id)
        else {
            continue;
        };

        let mut sorted_visits = route.visits.clone();
        sorted_visits.sort_by_key(|visit| visit.start_minute);

        let work_minutes = sorted_visits
            .iter()
            .map(|visit| visit.duration_minutes())
            .sum::<i32>();

        let travel_minutes = route_travel_minutes(&sorted_visits, problem);

        for visit in &sorted_visits {
            earned_task_ids.insert(visit.task_id.clone());
        }

        let active_minutes = route_active_minutes(route, &sorted_visits, problem);

        match staff.role {
            StaffRole::Doctor => {
                doctor_work_minutes += work_minutes;
                doctor_travel_minutes += travel_minutes;
                doctor_active_minutes += active_minutes;
            }
            StaffRole::Nurse => {
                nurse_count += 1;
                nurse_work_minutes += work_minutes;
                nurse_travel_minutes += travel_minutes;
                nurse_total_active_minutes += active_minutes;
            }
        }
    }

    let earned_points = earned_task_ids
        .iter()
        .filter_map(|task_id| problem.task_by_id(task_id))
        .map(|task| task.points)
        .sum::<i32>();

    let unassigned_high_priority_count = solution
        .unassigned_task_ids
        .iter()
        .filter_map(|task_id| problem.task_by_id(task_id))
        .filter(|task| task.priority == Priority::High)
        .count();

    let unassigned_normal_priority_count = solution
        .unassigned_task_ids
        .iter()
        .filter_map(|task_id| problem.task_by_id(task_id))
        .filter(|task| task.priority == Priority::Normal)
        .count();

    let unassigned_low_priority_count = solution
        .unassigned_task_ids
        .iter()
        .filter_map(|task_id| problem.task_by_id(task_id))
        .filter(|task| task.priority == Priority::Low)
        .count();

    let weights = problem.scoring_weights;

    let total_score = earned_points
        - doctor_work_minutes * weights.doctor_work_minute_penalty
        - doctor_travel_minutes * weights.doctor_travel_minute_penalty
        - nurse_work_minutes * weights.nurse_work_minute_penalty
        - nurse_travel_minutes * weights.nurse_travel_minute_penalty
        - unassigned_high_priority_count as i32 * weights.unassigned_high_priority_penalty
        - unassigned_normal_priority_count as i32 * weights.unassigned_normal_priority_penalty
        - unassigned_low_priority_count as i32 * weights.unassigned_low_priority_penalty;

    let nurse_avg_active_minutes = if nurse_count == 0 {
        0.0
    } else {
        nurse_total_active_minutes as f64 / nurse_count as f64
    };

    let points_per_doctor_minute = if doctor_active_minutes == 0 {
        0.0
    } else {
        earned_points as f64 / doctor_active_minutes as f64
    };

    ScoreBreakdown {
        total_score,
        earned_points,

        doctor_work_minutes,
        doctor_travel_minutes,
        nurse_work_minutes,
        nurse_travel_minutes,

        doctor_active_minutes,
        nurse_total_active_minutes,
        nurse_avg_active_minutes,

        points_per_doctor_minute,

        unassigned_task_count: solution.unassigned_task_ids.len(),
        unassigned_high_priority_count,
        unassigned_normal_priority_count,
        unassigned_low_priority_count,
    }
}

fn route_travel_minutes(visits: &[domain::Visit], problem: &ProblemInput) -> i32 {
    let Some(first) = visits.first() else {
        return 0;
    };

    let mut total = problem
        .travel_minutes_from_depot(&first.room_id)
        .unwrap_or_default();

    for pair in visits.windows(2) {
        let previous = &pair[0];
        let next = &pair[1];

        total += problem
            .travel_minutes_between_rooms(&previous.room_id, &next.room_id)
            .unwrap_or_default();
    }

    total
}

fn route_active_minutes(
    route: &domain::StaffRoute,
    sorted_visits: &[domain::Visit],
    problem: &ProblemInput,
) -> i32 {
    let last_visit_end = sorted_visits.last().map(|visit| visit.end_minute);

    let active_until = match (last_visit_end, route.forced_active_until) {
        (Some(visit_end), Some(forced_end)) => visit_end.max(forced_end),
        (Some(visit_end), None) => visit_end,
        (None, Some(forced_end)) => forced_end,
        (None, None) => return 0,
    };

    active_until.value() - problem.planning_window.start.value()
}
