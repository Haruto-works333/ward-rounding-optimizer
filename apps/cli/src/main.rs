use clap::{Parser, Subcommand};
use domain::{ProblemInput, Solution};
use optimizer::{
    fixtures::mini_case_001,
    report::{build_method_report, MethodReport},
    solver::{all_solvers, GreedyOptimizer, Solver},
    validate::Violation,
};

#[derive(Parser, Debug)]
#[command(name = "cli", about = "Ward rounding optimizer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the mini-case-001 scenario across all methods.
    MiniCase {
        /// Show per-route visit details and any constraint violations.
        #[arg(long)]
        details: bool,
    },
    /// Run the greedy optimizer over a range of nurse counts.
    Sensitivity {
        /// Minimum nurse count (inclusive).
        #[arg(long, default_value_t = 1)]
        nurse_min: usize,
        /// Maximum nurse count (inclusive).
        #[arg(long, default_value_t = 4)]
        nurse_max: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::MiniCase { details: false }) {
        Command::MiniCase { details } => run_mini_case(details),
        Command::Sensitivity {
            nurse_min,
            nurse_max,
        } => run_sensitivity(nurse_min, nurse_max),
    }
}

fn run_mini_case(show_details: bool) {
    let problem = mini_case_001(2);
    let solvers = all_solvers();

    let runs = solvers
        .iter()
        .map(|solver| {
            let name = solver.name();
            let solution = solver.solve(&problem);
            let report = build_method_report(name, &solution, &problem);
            (name, solution, report)
        })
        .collect::<Vec<_>>();

    let rows = runs
        .iter()
        .map(|(_, _, report)| report_to_row(report))
        .collect::<Vec<_>>();

    println!("Scenario: mini-case-001");
    println!(
        "Planning window: {} min",
        problem.planning_window.duration_minutes()
    );
    println!("Doctor count: {}", problem.doctors().count());
    println!();

    print_rows(&rows);

    if show_details {
        for (name, solution, report) in &runs {
            print_solution_details(name, solution, &problem);
            print_violations(&report.violations);
        }
    }
}

fn run_sensitivity(nurse_min: usize, nurse_max: usize) {
    let solver = GreedyOptimizer;
    let mut rows = Vec::new();

    for nurse_count in nurse_min..=nurse_max {
        let problem = mini_case_001(nurse_count);
        let solution = solver.solve(&problem);
        let report = build_method_report(solver.name(), &solution, &problem);

        rows.push(report_to_row(&report));
    }

    let sample_problem = mini_case_001(nurse_min);

    println!("Scenario: mini-case-001");
    println!(
        "Planning window: {} min",
        sample_problem.planning_window.duration_minutes()
    );
    println!("Doctor count: {}", sample_problem.doctors().count());
    println!();

    print_rows(&rows);
}

fn report_to_row(report: &MethodReport) -> ReportRow {
    let score = &report.score;

    ReportRow {
        nurse_count: report.nurse_count,
        method: report.method,
        total_score: score.total_score,
        points: score.earned_points,
        doctor_min: score.doctor_active_minutes,
        doctor_travel: score.doctor_travel_minutes,
        nurse_avg_min: score.nurse_avg_active_minutes,
        points_per_doctor_minute: score.points_per_doctor_minute,
        unassigned: score.unassigned_task_count,
        violations: report.violation_count(),
    }
}

fn print_rows(rows: &[ReportRow]) {
    println!(
        "{:<6} | {:<24} | {:>6} | {:>6} | {:>9} | {:>12} | {:>11} | {:>16} | {:>10} | {:>10}",
        "Nurses",
        "Method",
        "Score",
        "Points",
        "DoctorMin",
        "DoctorTravel",
        "NurseAvgMin",
        "Pts/DoctorMin",
        "Unassigned",
        "Violations",
    );

    println!("{}", "-".repeat(132));

    for row in rows {
        println!(
            "{:<6} | {:<24} | {:>6} | {:>6} | {:>9} | {:>12} | {:>11.1} | {:>16.1} | {:>10} | {:>10}",
            row.nurse_count,
            row.method,
            row.total_score,
            row.points,
            row.doctor_min,
            row.doctor_travel,
            row.nurse_avg_min,
            row.points_per_doctor_minute,
            row.unassigned,
            row.violations,
        );
    }
}

fn print_solution_details(method: &str, solution: &Solution, problem: &ProblemInput) {
    println!();
    println!("== {method} routes ==");

    for route in &solution.routes {
        let Some(staff) = problem
            .staff
            .iter()
            .find(|staff| staff.id == route.staff_id)
        else {
            println!("- Unknown staff {:?}", route.staff_id);
            continue;
        };

        println!();
        println!("- {} ({:?})", staff.name, staff.role);

        let mut visits = route.visits.clone();
        visits.sort_by_key(|visit| visit.start_minute);

        if visits.is_empty() {
            match route.forced_active_until {
                Some(until) => {
                    println!("  no visits, but active until {} min", until.value());
                }
                None => {
                    println!("  no visits");
                }
            }
            continue;
        }

        for visit in visits {
            let Some(task) = problem.task_by_id(&visit.task_id) else {
                println!(
                    "  {}-{} {:?} unknown task",
                    visit.start_minute.value(),
                    visit.end_minute.value(),
                    visit.task_id
                );
                continue;
            };

            let room_name = problem
                .room_by_id(&visit.room_id)
                .map_or("unknown-room", |room| room.name.as_str());

            println!(
                "  {:>2}-{:>2} | {:<3} | {:<4} | {:?} | {} pts",
                visit.start_minute.value(),
                visit.end_minute.value(),
                task.id.0,
                room_name,
                task.requirement,
                task.points
            );
        }

        if let Some(until) = route.forced_active_until {
            println!("  forced active until: {} min", until.value());
        }
    }

    if solution.unassigned_task_ids.is_empty() {
        println!();
        println!("Unassigned: none");
        return;
    }

    println!();
    println!("Unassigned:");

    for task_id in &solution.unassigned_task_ids {
        let Some(task) = problem.task_by_id(task_id) else {
            println!("  {task_id:?}");
            continue;
        };

        let room_name = problem
            .room_by_id(&task.room_id)
            .map_or("unknown-room", |room| room.name.as_str());

        println!(
            "  {:<3} | {:<4} | {:?} | {:?} | {} pts",
            task.id.0, room_name, task.requirement, task.priority, task.points
        );
    }
}

fn print_violations(violations: &[Violation]) {
    if violations.is_empty() {
        return;
    }

    println!();
    println!("Violations:");

    for violation in violations {
        println!("  [{:?}] {}", violation.kind, violation.message);
    }
}

#[derive(Debug, Clone)]
struct ReportRow {
    nurse_count: usize,
    method: &'static str,
    total_score: i32,
    points: i32,
    doctor_min: i32,
    doctor_travel: i32,
    nurse_avg_min: f64,
    points_per_doctor_minute: f64,
    unassigned: usize,
    violations: usize,
}
