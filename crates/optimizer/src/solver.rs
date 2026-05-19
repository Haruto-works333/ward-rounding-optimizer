//! Unified interface for placement strategies.
//!
//! Each baseline and the greedy optimizer implement `Solver`, so the CLI (and any future
//! benchmark harness) can iterate over a list of strategies without knowing the concrete
//! types. Names are pinned per strategy and used as keys in reports and snapshot tests.
//!
//! 配置戦略の統一インタフェース。各ベースラインと greedy が `Solver` を実装するので、
//! CLI や将来のベンチマークは具体型を知らずに戦略リストを回せる。名前は戦略ごとに
//! 固定で、レポートやスナップショットテストでキーとして使う。

use domain::{ProblemInput, Solution};

use crate::baseline::{doctor_accompany_baseline, room_order_baseline};
use crate::greedy::greedy_optimizer;

pub trait Solver {
    /// Stable display name for this strategy. Used as the `method` field in reports.
    ///
    /// この戦略の表示名 (安定)。レポートの `method` フィールドに用いる。
    fn name(&self) -> &'static str;

    /// Run the strategy on `problem` and return the resulting solution.
    ///
    /// `problem` に対して戦略を走らせ、得られた解を返す。
    fn solve(&self, problem: &ProblemInput) -> Solution;
}

pub struct RoomOrderBaseline;

impl Solver for RoomOrderBaseline {
    fn name(&self) -> &'static str {
        "RoomOrderBaseline"
    }

    fn solve(&self, problem: &ProblemInput) -> Solution {
        room_order_baseline(problem)
    }
}

pub struct DoctorAccompanyBaseline;

impl Solver for DoctorAccompanyBaseline {
    fn name(&self) -> &'static str {
        "DoctorAccompanyBaseline"
    }

    fn solve(&self, problem: &ProblemInput) -> Solution {
        doctor_accompany_baseline(problem)
    }
}

pub struct GreedyOptimizer;

impl Solver for GreedyOptimizer {
    fn name(&self) -> &'static str {
        "GreedyOptimizer"
    }

    fn solve(&self, problem: &ProblemInput) -> Solution {
        greedy_optimizer(problem)
    }
}

/// Convenience constructor returning every shipped solver, in the order used by
/// `mini-case` comparison output.
///
/// 同梱の全 Solver を `mini-case` の比較出力と同じ順序で返すヘルパー。
pub fn all_solvers() -> Vec<Box<dyn Solver>> {
    vec![
        Box::new(RoomOrderBaseline),
        Box::new(DoctorAccompanyBaseline),
        Box::new(GreedyOptimizer),
    ]
}
