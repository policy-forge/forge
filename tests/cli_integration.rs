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

// T011: Updated to expect OSCAL Catalog JSON structure with required --strategy/--format flags
#[test]
fn convert_valid_md_outputs_json() {
    let dir = TempDir::new().unwrap();
    let content = "# Title\n\nSome content.\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    // Expect OSCAL Catalog JSON structure
    assert!(json["catalog"].is_object(), "Should have 'catalog' object");
    assert!(json["catalog"]["metadata"].is_object(), "Should have 'metadata'");
    assert!(json["catalog"]["uuid"].is_string(), "Should have 'uuid'");
}

// T008 [US2] CLI integration tests for pipeline output

#[test]
fn convert_stdout_outputs_valid_oscal_json() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    assert!(json["catalog"].is_object(), "Should have 'catalog' key (AC-3)");
}

#[test]
fn convert_file_output_creates_oscal_json() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);
    let output_path = dir.path().join("output.json");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    // Verify file was created with valid OSCAL JSON (AC-4)
    let file_content = fs::read_to_string(&output_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&file_content)
        .unwrap_or_else(|e| panic!("File is not valid JSON: {e}"));
    assert!(json["catalog"].is_object(), "File should contain OSCAL catalog");

    // SEC-2: Verify output file has default permissions (not elevated) on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&output_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        // File should not be executable and should not be world-writable,
        // regardless of the process umask (which affects group-writable bits).
        assert_eq!(
            mode & (0o111 | 0o002),
            0,
            "File permissions should not be executable or world-writable, got: {mode:o}"
        );
    }
}

#[test]
fn convert_overwrite_output_file() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);
    let output_path = dir.path().join("output.json");

    // Write garbage content to the output path first
    fs::write(&output_path, "this is not json").unwrap();
    let garbage_content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(garbage_content, "this is not json");

    // Run pipeline (EC-7: should overwrite the garbage content)
    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");
    assert!(output.status.success(), "Pipeline should succeed and overwrite existing file");

    // File should now contain valid OSCAL JSON, not garbage
    let overwritten_content = fs::read_to_string(&output_path).unwrap();
    assert_ne!(overwritten_content, garbage_content, "File should be overwritten");
    let json: serde_json::Value = serde_json::from_str(&overwritten_content)
        .expect("Overwritten file should contain valid JSON");
    assert!(json["catalog"].is_object(), "Overwritten file should contain OSCAL catalog");
}

// T011: Updated existing tests with --strategy catalog --format json flags

#[test]
fn convert_pdf_shows_unsupported_format_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.pdf", "fake pdf");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

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

#[test]
fn convert_nonexistent_file_shows_not_found_error() {
    let output = forge_bin()
        .arg("convert")
        .arg("nonexistent.md")
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
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

    let output = forge_bin()
        .arg("convert")
        .arg(&dir_path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a regular file"),
        "stderr should indicate not a regular file:\n{stderr}"
    );
}

#[test]
fn convert_oversized_file_shows_size_error() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.md");
    let content = "x".repeat(11 * 1024 * 1024);
    fs::write(&path, &content).unwrap();

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exceeding"), "stderr should mention exceeding limit:\n{stderr}");
    assert!(stderr.contains("max-size"), "stderr should mention --max-size:\n{stderr}");
}

#[test]
fn convert_oversized_file_with_max_size_override_succeeds() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("big.md");
    let content = "x".repeat(11 * 1024 * 1024);
    fs::write(&path, &content).unwrap();

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .arg("--max-size")
        .arg("20")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0 with --max-size 20, stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}"));
    assert!(json["catalog"].is_object(), "Should have catalog object");
}

#[test]
fn max_size_flag_is_recognized_by_clap() {
    let output =
        forge_bin().arg("convert").arg("--help").output().expect("Failed to execute process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("max-size"), "Help should list --max-size flag:\n{stdout}");
}

// T016 [US4] CLI edge case tests

#[test]
fn convert_empty_file_shows_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "empty.md", "");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    // EC-2: empty file should produce non-zero exit and descriptive error
    assert!(!output.status.success(), "Expected non-zero exit code for empty file");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty") || stderr.contains("Empty") || stderr.contains("no content"),
        "stderr should mention empty file, got: {stderr}"
    );
}

#[test]
fn convert_output_nonexistent_parent_dir_shows_error() {
    let dir = TempDir::new().unwrap();
    let content = "# Title\n\n- Requirement.\n";
    let path = create_temp_md(&dir, "policy.md", content);
    let bad_output = dir.path().join("nonexistent_subdir").join("output.json");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&bad_output)
        .output()
        .expect("Failed to execute process");

    // EC-3: non-existent output parent dir → error
    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("does not exist") || stderr.contains("invalid"),
        "stderr should mention invalid path:\n{stderr}"
    );
}

#[test]
fn convert_missing_strategy_flag_shows_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.md", "# Title\n");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    // EC-4: omitted --strategy → clap error
    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--strategy") || stderr.contains("required"),
        "stderr should indicate --strategy is required:\n{stderr}"
    );
}

#[test]
fn convert_missing_format_flag_shows_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.md", "# Title\n");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .output()
        .expect("Failed to execute process");

    // EC-5: omitted --format → clap error
    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--format") || stderr.contains("required"),
        "stderr should indicate --format is required:\n{stderr}"
    );
}

#[test]
fn convert_format_xml_shows_rejection_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.md", "# Title\n");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("xml")
        .output()
        .expect("Failed to execute process");

    // W-2: --format xml → descriptive rejection (deferred to WI-26)
    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("json") && stderr.contains("supported"),
        "stderr should mention only json is supported:\n{stderr}"
    );
}

#[test]
fn convert_strategy_component_shows_rejection_error() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "policy.md", "# Title\n");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("component")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    // S-3: --strategy component → descriptive rejection
    assert!(!output.status.success(), "Expected non-zero exit code");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("catalog") && stderr.contains("supported"),
        "stderr should mention only catalog is supported:\n{stderr}"
    );
}
