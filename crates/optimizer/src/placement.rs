//! Task-to-staff placement: candidate generation, scoring, and mutation of the solution.
//!
//! Where `crate::routing` answers "when could this task fit?", this module answers
//! "given a fit, who gets it, and how does the solution change?". Both append-only
//! single-staff placement (used by the baselines) and pair placement for sync tasks
//! live here.
//!
//! タスクからスタッフへの配置: 候補生成・スコアリング・解の更新。`crate::routing` が
//! 「このタスクはいつ収まるか?」に答えるのに対し、こちらは「誰に割り当てて、解が
//! どう変わるか?」に答える。ベースライン用の追記専用単独配置と、Sync タスク用の
//! ペア配置の両方をここに置く。

use domain::{Minute, ProblemInput, Solution, StaffId, StaffRole, Task, Visit};

use crate::routing::{additional_travel_minutes, earliest_start_for_append};

/// Append `task` to `staff_id`'s route as a new visit at `[start, end]`.
/// Silently no-ops if the staff id has no matching route.
///
/// `staff_id` の経路の末尾に `[start, end]` で `task` を追加する。
/// 該当する経路がない場合は黙って何もしない。
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

/// Assign `task` to the role-matching staff member whose **appended** visit ends earliest.
///
/// Tie-break: lower `additional_travel`, then lexicographic staff id. Returns `false` if
/// no candidate fits the planning window and staff availability. This is the append-only
/// counterpart used by the baselines; the greedy optimizer uses an insertion variant
/// because it processes tasks out of chronological order. See
/// `crate::greedy::assign_via_insertion_to_best_staff`.
///
/// `task` を、ロール一致するスタッフのうち **末尾追記** したときに終了が最早の
/// 経路へ割り当てる。タイブレークは `additional_travel` の小さい方、次にスタッフ ID 辞書順。
/// planning window とスタッフの可用域に収まる候補が無ければ `false`。
/// ベースライン用の追記専用版。greedy は時系列順でなくタスクを処理するため、
/// 挿入版 (`crate::greedy::assign_via_insertion_to_best_staff`) を使う。
pub fn assign_via_append_to_best_staff(
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

    push_visit(solution, &candidate.staff_id, task, candidate.start, candidate.end);

    true
}

/// Assign a `DoctorAndNurseSync` task to the best (doctor, nurse) pair.
///
/// **Append-only.** The earliest-start is computed via [`earliest_start_for_append`], which
/// places the new visit after each candidate's temporally-latest visit. Mid-route insertion
/// is not attempted, because finding a slot that simultaneously fits both routes requires
/// a more involved search. Callers that need the resulting solution to be optimal should
/// schedule sync tasks before any other task type (as `greedy_optimizer` does), so both
/// routes are empty and append == insert.
///
/// **追記専用。** 最早開始時刻は [`earliest_start_for_append`] で求めており、各候補の
/// 時系列上もっとも遅い visit の後ろに新しい visit を置く。両ルートに同時に収まる
/// 隙間を探す挿入は実装していない (探索コストが大きいため)。呼び出し側で最適性を
/// 担保したい場合は、`greedy_optimizer` のように Sync タスクを他種別より先に割り当て、
/// 両ルートが空の状態で呼ぶこと (空ならば append == insert)。
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
