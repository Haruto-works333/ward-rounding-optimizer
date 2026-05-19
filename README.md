# Role-Constrained Ward Rounding Optimizer

[日本語版はこちら](README.ja.md)

A proof-of-concept optimizer for hospital ward rounding routes with role constraints.

This project models ward rounding as a combinatorial optimization problem.
It assigns room-based tasks to doctors and nurses while considering role requirements, travel time, task duration, task points, staff availability, and synchronization constraints.

This project does not use real patient data or real hospital operation data.

## Current Status

Implemented as a Rust CLI. The following pieces are in place:

- Domain model for rooms, staff, tasks, routes, and solutions
- Fixed synthetic scenario: `Mini Case 001`
- Two baseline methods:
  - `RoomOrderBaseline`
  - `DoctorAccompanyBaseline`
- Greedy optimizer with best-insertion placement
- Constraint validation with categorized violation kinds
- Score calculation with point / penalty breakdown
- Nurse-count sensitivity analysis
- clap-based CLI with subcommands and `--help`
- Regression tests and CLI snapshot tests

Explicitly **not** implemented yet:

- Database persistence
- API server
- Web frontend
- Random scenario generation
- Local search / simulated annealing
- Real medical data integration

## Problem Overview

Each task is assigned to a room and has:

- Required role
- Duration in minutes
- Points
- Priority

Supported task requirements are:

- `DoctorRequired`
- `NurseCapable`
- `NurseOnly`
- `DoctorAndNurseSync`

The main KPI is:

```text
points_per_doctor_minute = earned_points / doctor_active_minutes
```

This measures how efficiently the scarce doctor resource is used.
`doctor_active_minutes` is the on-duty span (from the start of the planning window or the doctor's availability, whichever is later, to the last commitment end) — it includes idle gaps, not just active work.

## Repository Structure

```text
ward-rounding-optimizer/
  apps/
    cli/              # CLI application (clap-based)
      tests/          # CLI snapshot tests

  crates/
    domain/           # Core domain types (no logic, just data)
      geometry.rs     #   Point + Manhattan distance
      ids.rs          #   RoomId, StaffId, TaskId newtypes
      problem.rs      #   ProblemInput
      room.rs / staff.rs / task.rs
      scoring.rs      #   ScoringWeights + ScoreBreakdown
      solution.rs     #   Visit, StaffRoute, Solution
      time.rs         #   Minute, TimeWindow

    optimizer/        # Placement strategies + evaluation
      solver.rs       #   Solver trait + unit-struct strategies
      baseline.rs     #   RoomOrderBaseline
      baseline/
        doctor_accompany.rs
      greedy.rs       #   GreedyOptimizer (best-insertion)
      routing.rs      #   Travel-time / earliest-start helpers
      placement.rs    #   Candidate scoring + solution mutation
      scoring.rs      #   score() function
      validate.rs     #   Constraint validation (Violation)
      report.rs       #   MethodReport
      fixtures.rs     #   Synthetic scenarios (mini_case_001)
```

## Solver API

All placement strategies implement `optimizer::solver::Solver`:

```rust
pub trait Solver {
    fn name(&self) -> &'static str;
    fn solve(&self, problem: &ProblemInput) -> Solution;
}
```

Use `optimizer::solver::all_solvers()` to get every shipped strategy as a `Vec<Box<dyn Solver>>`, or instantiate one directly (`RoomOrderBaseline`, `DoctorAccompanyBaseline`, `GreedyOptimizer`). Free functions like `greedy_optimizer(problem)` available for callers that just want a function reference.

## Requirements

- Rust
- Cargo

If Rust is not installed, install it with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Usage

Show available subcommands:

```bash
cargo run -p cli -- --help
```

Run the mini case (default subcommand if none is given):

```bash
cargo run -p cli -- mini-case
```

Run the mini case with route details and any violations:

```bash
cargo run -p cli -- mini-case --details
```

Run nurse-count sensitivity analysis:

```bash
cargo run -p cli -- sensitivity --nurse-min 1 --nurse-max 4
```

## Example Output

```text
Scenario: mini-case-001
Planning window: 30 min
Doctor count: 1

Nurses | Method                   |  Score | Points | DoctorMin | DoctorTravel | NurseAvgMin |    Pts/DoctorMin | Unassigned | Violations
------------------------------------------------------------------------------------------------------------------------------------
2      | RoomOrderBaseline        |   2340 |   3250 |        30 |            4 |        27.5 |            108.3 |          1 |          0
2      | DoctorAccompanyBaseline  |   -621 |   1900 |        25 |            1 |        19.5 |             76.0 |          3 |          0
2      | GreedyOptimizer          |   2312 |   3250 |        28 |            7 |        27.0 |            116.1 |          1 |          0
```

## Testing

Run all tests:

```bash
cargo test
```

The suite covers optimizer behavior, scoring-weight invariants, and CLI output stability via snapshot tests under `apps/cli/tests/`.

## Future Directions

No fixed roadmap. Plausible next steps include:

- SQLite persistence
- Rust API server
- Web frontend
- Richer scenario generation
- Local search and other optimization methods
- Support for more realistic operational constraints

## License

MIT
