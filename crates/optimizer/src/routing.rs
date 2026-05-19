//! Travel-time and earliest-start helpers shared across placement strategies.
//!
//! These are pure-geometry / pure-time queries — they don't mutate the solution. The
//! placement-side functions in `crate::placement` build on top of them.
//!
//! 配置戦略間で共有する travel time と最早開始時刻のヘルパー。これらは純粋な
//! 幾何 / 時間クエリで、解を変更しない。実際の配置は `crate::placement` 側で行う。

use domain::{Minute, ProblemInput, StaffRoute, Task};

/// Earliest time `task` can start if appended after `route`'s temporally-latest visit
/// (or from the depot, if the route is empty). Returns `None` when travel time cannot
/// be computed (e.g. unknown room).
///
/// `route` の時系列上もっとも遅い visit の後ろに `task` を追記したときに開始できる
/// 最早時刻 (route が空ならば depot からの出発時刻)。travel time が計算できない場合
/// (未知の部屋など) は `None`。
pub fn earliest_start_for_append(
    problem: &ProblemInput,
    route: &StaffRoute,
    task: &Task,
) -> Option<Minute> {
    match route.latest_visit_by_time() {
        Some(latest_visit) => {
            let travel_minutes =
                problem.travel_minutes_between_rooms(&latest_visit.room_id, &task.room_id)?;

            Some(latest_visit.end_minute.add_minutes(travel_minutes))
        }
        None => {
            let travel_minutes = problem.travel_minutes_from_depot(&task.room_id)?;
            Some(problem.planning_window.start.add_minutes(travel_minutes))
        }
    }
}

/// Additional travel minutes incurred by appending `task` to `route`.
///
/// `task` を `route` に追記したときに追加で発生する移動時間。
pub fn additional_travel_minutes(
    problem: &ProblemInput,
    route: &StaffRoute,
    task: &Task,
) -> Option<i32> {
    match route.latest_visit_by_time() {
        Some(latest_visit) => {
            problem.travel_minutes_between_rooms(&latest_visit.room_id, &task.room_id)
        }
        None => problem.travel_minutes_from_depot(&task.room_id),
    }
}
