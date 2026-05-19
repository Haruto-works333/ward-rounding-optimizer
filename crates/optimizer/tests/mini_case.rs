use optimizer::{
    baseline::{doctor_accompany_baseline, room_order_baseline},
    fixtures::mini_case_001,
    greedy::greedy_optimizer,
    scoring::score,
    validate::validate,
};

#[test]
fn room_order_baseline_is_valid_for_mini_case_001() {
    let problem = mini_case_001(2);
    let solution = room_order_baseline(&problem);

    let violations = validate(&solution, &problem);
    let score = score(&solution, &problem);

    assert!(violations.is_empty());
    assert_eq!(score.earned_points, 3250);
    assert_eq!(score.unassigned_task_count, 1);
}

#[test]
fn doctor_accompany_baseline_is_valid_but_inefficient_for_mini_case_001() {
    let problem = mini_case_001(2);
    let solution = doctor_accompany_baseline(&problem);

    let violations = validate(&solution, &problem);
    let score = score(&solution, &problem);

    assert!(violations.is_empty());
    assert_eq!(score.earned_points, 1900);
    assert_eq!(score.doctor_active_minutes, 25);
    assert_eq!(score.unassigned_task_count, 3);
}

#[test]
fn greedy_optimizer_improves_points_per_doctor_minute_for_mini_case_001() {
    let problem = mini_case_001(2);

    let room_order = room_order_baseline(&problem);
    let greedy = greedy_optimizer(&problem);

    let room_order_violations = validate(&room_order, &problem);
    let greedy_violations = validate(&greedy, &problem);

    let room_order_score = score(&room_order, &problem);
    let greedy_score = score(&greedy, &problem);

    assert!(room_order_violations.is_empty());
    assert!(greedy_violations.is_empty());

    assert_eq!(greedy_score.earned_points, 3250);
    assert_eq!(greedy_score.unassigned_task_count, 1);

    assert!(
        greedy_score.points_per_doctor_minute > room_order_score.points_per_doctor_minute,
        "greedy points/doctor_min should be better than room-order baseline"
    );
}

#[test]
fn greedy_optimizer_runs_for_nurse_count_1_to_4() {
    for nurse_count in 1..=4 {
        let problem = mini_case_001(nurse_count);
        let solution = greedy_optimizer(&problem);

        let violations = validate(&solution, &problem);
        let score = score(&solution, &problem);

        assert!(
            violations.is_empty(),
            "nurse_count={nurse_count}, violations={violations:?}"
        );

        assert!(
            score.earned_points > 0,
            "nurse_count={nurse_count} should earn some points"
        );
    }
}
