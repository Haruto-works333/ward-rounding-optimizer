use std::process::Command;

/// Run the compiled CLI binary with the given args and return stdout as a string.
/// Fails the test if the process exits non-zero.
///
/// コンパイル済みの CLI バイナリを与えた引数で実行し、stdout を文字列で返す。
/// 終了コードが 0 以外ならテスト失敗。
fn run_cli(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(args)
        .output()
        .expect("failed to spawn cli binary");

    assert!(
        output.status.success(),
        "cli exited with {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout).expect("cli stdout is not valid utf-8")
}

#[test]
fn mini_case_output_matches_snapshot() {
    let actual = run_cli(&["mini-case"]);
    let expected = include_str!("snapshots/mini_case.txt");

    assert_eq!(actual, expected);
}

#[test]
fn sensitivity_output_matches_snapshot() {
    let actual = run_cli(&["sensitivity", "--nurse-min", "1", "--nurse-max", "4"]);
    let expected = include_str!("snapshots/sensitivity.txt");

    assert_eq!(actual, expected);
}

#[test]
fn no_subcommand_defaults_to_mini_case() {
    let with_subcommand = run_cli(&["mini-case"]);
    let without_subcommand = run_cli(&[]);

    assert_eq!(with_subcommand, without_subcommand);
}

#[test]
fn details_flag_extends_output() {
    let plain = run_cli(&["mini-case"]);
    let detailed = run_cli(&["mini-case", "--details"]);

    assert!(detailed.starts_with(&plain));
    assert!(detailed.len() > plain.len());
    assert!(detailed.contains("== RoomOrderBaseline routes =="));
}
