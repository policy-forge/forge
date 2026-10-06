//! Subprocess startup controls exercise the actual binary rather than library test-thread parsing.

use std::process::{Command, Output};

/// Start the real CLI without an inherited Rust thread-stack override.
fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(arguments)
        .env_remove("RUST_MIN_STACK")
        .output()
        .expect("the actual Forge binary must start")
}

/// A platform exception or Rust panic must never satisfy a descriptive CLI result.
fn assert_no_panic(output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("has overflowed its stack") && !stderr.contains("panicked at"),
        "the CLI must reach a controlled result: {stderr}"
    );
}

/// Both early root-parser exits must successfully start the actual binary.
#[test]
fn root_help_and_version_start_the_actual_binary() {
    let help = run(&["--help"]);
    assert_no_panic(&help);
    assert!(help.status.success(), "root help failed: {help:?}");
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("Usage:") && text.contains("convert") && text.contains("review"));

    let version = run(&["--version"]);
    assert_no_panic(&version);
    assert!(version.status.success(), "version failed: {version:?}");
    let text = String::from_utf8_lossy(&version.stdout);
    assert!(text.contains(env!("CARGO_PKG_VERSION")));
}

/// Each maintained review help must survive binary startup and preserve its authority disclosure.
#[test]
fn review_operation_help_starts_the_actual_binary() {
    for operation in ["init", "respond", "merge", "status", "export-html", "export-notifications"] {
        let output = run(&["review", operation, "--help"]);
        assert_no_panic(&output);
        assert!(output.status.success(), "{operation} help failed: {output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(text.contains("not authenticated or signed"));
        assert!(text.contains("grants no domain approval"));
    }
}

/// A missing required input must retain clap's controlled parser error instead of a startup crash.
#[test]
fn incomplete_review_command_retains_parser_exit_and_diagnostic() {
    let output = run(&["review", "init"]);
    assert_no_panic(&output);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") && stderr.contains("Usage:"), "{stderr}");
}
