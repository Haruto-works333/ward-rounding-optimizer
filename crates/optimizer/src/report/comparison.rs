use domain::{ProblemInput, ScoreBreakdown, Solution};

use crate::{score, validate};

#[derive(Debug, Clone)]
pub struct MethodReport {
    pub nurse_count: usize,
    pub method: &'static str,
    pub score: ScoreBreakdown,
    pub violation_count: usize,
}

pub fn build_method_report(
    nurse_count: usize,
    method: &'static str,
    solution: &Solution,
    problem: &ProblemInput,
) -> MethodReport {
    let violations = validate(solution, problem);
    let score = score(solution, problem);

    MethodReport {
        nurse_count,
        method,
        score,
        violation_count: violations.len(),
    }
}
