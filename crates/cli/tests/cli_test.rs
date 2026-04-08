use std::process::Command;

/// Helper to run the aletheia CLI and capture output.
fn run_cli(args: &[&str]) -> (String, String, i32) {
    let output = Command::new(env!("CARGO_BIN_EXE_aletheia"))
        .args(args)
        .output()
        .expect("failed to run aletheia");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

#[test]
fn cli_help_exits_zero() {
    let (stdout, _, code) = run_cli(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("aletheia"));
}

#[test]
fn cli_status_without_daemon() {
    let (stdout, _, _) = run_cli(&["status"]);
    assert!(stdout.contains("STOPPED") || stdout.contains("Daemon"));
}

#[test]
fn cli_doctor_runs() {
    let (stdout, _, code) = run_cli(&["doctor"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("Doctor") || stdout.contains("Plugin"));
}

#[test]
fn cli_init_with_existing_config() {
    // Should detect existing config and not overwrite
    let (stdout, _, code) = run_cli(&["init"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("already exists") || stdout.contains("initialized"));
}

#[test]
fn cli_recall_without_daemon() {
    let (stdout, _, _) = run_cli(&["recall", "--query", "test"]);
    assert!(stdout.contains("not running") || stdout.contains("Daemon"));
}
