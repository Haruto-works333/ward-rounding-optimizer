[日本語版はこちら](README.ja.md)

# Role-Constrained Ward Rounding Optimizer

A proof-of-concept optimizer for hospital ward rounding routes with role constraints.

This project models ward rounding as a combinatorial optimization problem. It assigns room-based tasks to doctors and nurses while considering role requirements, travel time, task duration, task points, staff availability, and synchronization constraints.

## Current Status

Phase 1 is implemented as a Rust CLI.

The current implementation includes:

- Domain model for rooms, staff, tasks, routes, and solutions
- Fixed synthetic scenario: `Mini Case 001`
- Two baseline methods:
  - `RoomOrderBaseline`
  - `DoctorAccompanyBaseline`
- Greedy optimizer
- Constraint validation
- Score calculation
- Nurse-count sensitivity analysis
- CLI route details output
- Regression tests

This project does not use real patient data or real hospital operation data.

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

## Repository Structure

```text
ward-rounding-optimizer/
  apps/
    cli/              # CLI application

  crates/
    domain/           # Core domain types
    optimizer/        # Validation, scoring, baselines, greedy optimizer
```

## Requirements

- Rust
- Cargo

If Rust is not installed, install it with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Usage

Run the mini case:

```bash
cargo run -p cli -- mini-case
```

Run the mini case with route details:

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

## Phase 1 Scope

Phase 1 focuses on proving the optimization core in a CLI environment.

Implemented:

- Fixed synthetic mini case
- Baseline route generation
- Greedy optimization
- Hard constraint validation
- Score breakdown
- Nurse-count sensitivity analysis

Not included yet:

- Database persistence
- API server
- Web frontend
- Random scenario generation
- Local search
- Simulated annealing
- Real medical data integration

## Roadmap

Planned future phases:

1. Add SQLite persistence
2. Add Rust API server
3. Add web frontend
4. Add richer scenario generation
5. Add local search and other optimization methods
6. Support more realistic operational constraints

## License

MIT