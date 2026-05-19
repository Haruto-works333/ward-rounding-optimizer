use domain::{ProblemInput, Solution};
use optimizer::{
    baseline::{doctor_accompany_baseline, room_order_baseline},
    fixtures::mini_case_001,
    greedy::greedy_optimizer,
    report::build_method_report,
};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    match args.get(1).map(String::as_str) {
        Some("mini-case") | None => run_mini_case(&args),
        Some("sensitivity") => run_sensitivity(&args),
        Some(command) => {
            eprintln!("unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn run_mini_case(args: &[String]) {
    let problem = mini_case_001(2);
    let show_details = has_flag(args, "--details");

    let solutions = vec![
        ("RoomOrderBaseline", room_order_baseline(&problem)),
        (
            "DoctorAccompanyBaseline",
            doctor_accompany_baseline(&problem),
        ),
        ("GreedyOptimizer", greedy_optimizer(&problem)),
    ];

    let rows = solutions
        .iter()
        .map(|(method, solution)| build_row(method, solution, &problem))
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
        for (method, solution) in &solutions {
            print_solution_details(method, solution, &problem);
        }
    }
}

fn run_sensitivity(args: &[String]) {
    let nurse_min = parse_usize_arg(args, "--nurse-min").unwrap_or(1);
    let nurse_max = parse_usize_arg(args, "--nurse-max").unwrap_or(4);

    let mut rows = Vec::new();

    for nurse_count in nurse_min..=nurse_max {
        let problem = mini_case_001(nurse_count);
        let solution = greedy_optimizer(&problem);

        rows.push(build_row("GreedyOptimizer", &solution, &problem));
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

fn build_row(
    method: &'static str,
    solution: &Solution,
    problem: &ProblemInput,
) -> ReportRow {
    let report = build_method_report(method, solution, problem);
    let score = report.score;

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
        violations: report.violation_count,
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
                .map(|room| room.name.as_str())
                .unwrap_or("unknown-room");

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
            println!("  {:?}", task_id);
            continue;
        };

        let room_name = problem
            .room_by_id(&task.room_id)
            .map(|room| room.name.as_str())
            .unwrap_or("unknown-room");

        println!(
            "  {:<3} | {:<4} | {:?} | {:?} | {} pts",
            task.id.0, room_name, task.requirement, task.priority, task.points
        );
    }
}

fn parse_usize_arg(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find(|window| window[0] == name)
        .and_then(|window| window[1].parse::<usize>().ok())
}

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  cargo run -p cli -- mini-case");
    eprintln!("  cargo run -p cli -- mini-case --details");
    eprintln!("  cargo run -p cli -- sensitivity --nurse-min 1 --nurse-max 4");
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
