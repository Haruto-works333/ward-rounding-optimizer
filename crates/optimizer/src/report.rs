use domain::{ProblemInput, ScoreBreakdown, Solution};

use crate::scoring::score;
use crate::validate::{validate, Violation};

#[derive(Debug, Clone)]
pub struct MethodReport {
    pub nurse_count: usize,
    pub method: &'static str,
    pub score: ScoreBreakdown,
    pub violations: Vec<Violation>,
}

impl MethodReport {
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

pub fn build_method_report(
    method: &'static str,
    solution: &Solution,
    problem: &ProblemInput,
) -> MethodReport {
    let violations = validate(solution, problem);
    let score = score(solution, problem);

    MethodReport {
        nurse_count: problem.nurses().count(),
        method,
        score,
        violations,
    }
}
