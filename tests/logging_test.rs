//! Integration tests for structured JSON logging (--log-format flag).
//!
//! These tests run the compiled binary as a subprocess to verify that CLI
//! argument parsing around `--log-format` works end-to-end.

use std::process::Command;

/// Helper: path to the built binary. `cargo test` compiles it for us.
fn ccchat_bin() -> String {
    // `cargo test` places the binary under target/debug/
    let mut path = std::env::current_exe()
        .expect("cannot determine test exe path")
        .parent()
        .expect("no parent dir")
        .parent()
        .expect("no grandparent dir")
        .to_path_buf();
    path.push("ccchat");
    path.to_string_lossy().to_string()
}

#[test]
fn log_format_json_with_help_exits_zero() {
    // --help causes immediate exit(0). If --log-format json is not a valid
    // flag the binary would print an error and exit(2).
    let output = Command::new(ccchat_bin())
        .args(["--log-format", "json", "--help"])
        .output()
        .expect("failed to run ccchat");

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // --help output should mention the flag itself
    assert!(
        stdout.contains("--log-format"),
        "help text should mention --log-format:\n{stdout}"
    );
}

#[test]
fn log_format_text_with_help_exits_zero() {
    let output = Command::new(ccchat_bin())
        .args(["--log-format", "text", "--help"])
        .output()
        .expect("failed to run ccchat");

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn log_format_env_var_with_help_exits_zero() {
    // CCCHAT_LOG_FORMAT env var should be accepted just like the CLI flag.
    let output = Command::new(ccchat_bin())
        .env("CCCHAT_LOG_FORMAT", "json")
        .args(["--help"])
        .output()
        .expect("failed to run ccchat");

    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CCCHAT_LOG_FORMAT"),
        "help text should mention CCCHAT_LOG_FORMAT env var:\n{stdout}"
    );
}

#[test]
fn help_describes_log_format_default() {
    let output = Command::new(ccchat_bin())
        .args(["--help"])
        .output()
        .expect("failed to run ccchat");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The default value "text" should appear in the help output
    assert!(
        stdout.contains("text"),
        "help text should show default value 'text':\n{stdout}"
    );
}

#[test]
fn missing_account_fails() {
    // Without --account (and no CCCHAT_ACCOUNT env), the binary should fail.
    // This validates that argument parsing is actually enforced.
    let output = Command::new(ccchat_bin())
        .env_remove("CCCHAT_ACCOUNT")
        .output()
        .expect("failed to run ccchat");

    assert!(
        !output.status.success(),
        "expected non-zero exit when --account is missing"
    );
}
