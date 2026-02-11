use std::process::Command;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

#[test]
fn help_shows_convert_and_validate() {
    let output = forge_bin().arg("--help").output().expect("Failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Expected success, got: {stdout}");
    assert!(stdout.contains("convert"), "Help should list 'convert' subcommand:\n{stdout}");
    assert!(stdout.contains("validate"), "Help should list 'validate' subcommand:\n{stdout}");
}

#[test]
fn no_args_shows_help() {
    let output = forge_bin().output().expect("Failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    // With arg_required_else_help, clap prints help and exits with code 2
    assert!(
        combined.contains("convert") || combined.contains("Usage"),
        "No-args output should show help or usage:\n{combined}"
    );
    assert_eq!(output.status.code(), Some(2), "Expected exit code 2 for no-args help display");
}

#[test]
fn convert_without_args_shows_error() {
    let output = forge_bin().arg("convert").output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected failure for 'convert' without arguments");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("INPUT") || stderr.contains("Usage"),
        "Error should indicate required arguments:\n{stderr}"
    );

    // Exit code 2 per clap convention for argument errors
    assert_eq!(output.status.code(), Some(2), "Expected exit code 2 for argument error");
}

#[test]
fn unknown_subcommand_shows_error() {
    let output = forge_bin().arg("unknown-command").output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected failure for unknown subcommand");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("convert") || stderr.contains("validate") || stderr.contains("subcommand"),
        "Error should list available subcommands:\n{stderr}"
    );

    assert_eq!(output.status.code(), Some(2), "Expected exit code 2 for unknown subcommand");
}
