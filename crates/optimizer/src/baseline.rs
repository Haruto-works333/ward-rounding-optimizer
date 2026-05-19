//! Baseline placement strategies.
//!
//! Each baseline is a deterministic, append-only routing rule used as a reference point
//! for the greedy optimizer. They prioritize predictability over score — the goal is to
//! see how much value the greedy approach actually adds.
//!
//! ベースライン配置戦略。各ベースラインは決定論的な追記専用ルーティングルールで、
//! greedy optimizer の参照点として使う。スコアより予測可能性を優先し、greedy が
//! どれだけ価値を加えているかを比較するのが目的。

mod doctor_accompany;

pub use doctor_accompany::doctor_accompany_baseline;

use domain::{ProblemInput, Solution, StaffRole, TaskRequirement};

use crate::placement::{assign_pair_for_sync_task, assign_via_append_to_best_staff};

/// Assign tasks in room-id order, picking the best-fitting staff per task.
///
/// 病室 ID 順にタスクを処理し、各タスクごとに最適なスタッフを選んで割り当てる。
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
                assign_via_append_to_best_staff(problem, &mut solution, task, StaffRole::Doctor)
            }
            TaskRequirement::NurseOnly | TaskRequirement::NurseCapable => {
                assign_via_append_to_best_staff(problem, &mut solution, task, StaffRole::Nurse)
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
