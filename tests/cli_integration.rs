use std::fs;
use std::io::Write;
use std::process::Command;

use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn create_temp_md(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
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

#[test]
fn convert_valid_md_outputs_json() {
    let dir = TempDir::new().unwrap();
    let content = "# Title\n\nSome content.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin().arg("convert").arg(&path).output().expect("Failed to execute process");

    assert!(output.status.success(), "Expected exit code 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    // source_path is a string
    assert!(json["source_path"].is_string(), "source_path should be a string");

    // fingerprint is a 64-char hex string
    let fingerprint = json["fingerprint"].as_str().unwrap();
    assert_eq!(fingerprint.len(), 64, "fingerprint should be 64 chars");
    assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()), "fingerprint should be hex");

    // lines is an array with correct count
    let lines = json["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 3, "Expected 3 lines");
    assert_eq!(lines[0]["number"], 1);
    assert_eq!(lines[0]["text"], "# Title");
    assert_eq!(lines[1]["number"], 2);
    assert_eq!(lines[1]["text"], "");
    assert_eq!(lines[2]["number"], 3);
    assert_eq!(lines[2]["text"], "Some content.");
}

// --- US2 integration tests ---

#[test]
fn convert_pdf_shows_unsupported_format_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.pdf", "fake pdf");

    let output = forge_bin().arg("convert").arg(&path).output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unsupported file format"),
        "stderr should mention unsupported format:\n{stderr}"
    );
    assert!(
        stderr.contains("pandoc") || stderr.contains("markitdown"),
        "stderr should suggest conversion tools:\n{stderr}"
    );
}

// --- US3 integration tests ---

#[test]
fn convert_nonexistent_file_shows_not_found_error() {
    let output = forge_bin()
        .arg("convert")
        .arg("nonexistent.md")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "stderr should indicate file not found:\n{stderr}"
    );
}

#[test]
fn convert_directory_shows_not_a_file_error() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().join("subdir.md");
    fs::create_dir(&dir_path).unwrap();

    let output =
        forge_bin().arg("convert").arg(&dir_path).output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a regular file"),
        "stderr should indicate not a regular file:\n{stderr}"
    );
}

// --- US4 integration tests ---

#[test]
fn convert_oversized_file_shows_size_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.md");
    // Write 11MB
    let content = "x".repeat(11 * 1024 * 1024);
    fs::write(&path, &content).unwrap();

    let output = forge_bin().arg("convert").arg(&path).output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exceeding"), "stderr should mention exceeding limit:\n{stderr}");
    assert!(stderr.contains("max-size"), "stderr should mention --max-size:\n{stderr}");
}

#[test]
fn convert_oversized_file_with_max_size_override_succeeds() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("big.md");
    // Write 11MB
    let content = "x".repeat(11 * 1024 * 1024);
    fs::write(&path, &content).unwrap();

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--max-size")
        .arg("20")
        .output()
        .expect("Failed to execute process");

    assert!(output.status.success(), "Expected exit code 0 with --max-size 20");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}"));
    assert!(json["fingerprint"].is_string());
}

#[test]
fn max_size_flag_is_recognized_by_clap() {
    let output =
        forge_bin().arg("convert").arg("--help").output().expect("Failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("max-size"), "Help should list --max-size flag:\n{stdout}");
}
