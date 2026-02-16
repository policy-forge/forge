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
    let mut content = String::from("# Large Policy\n\n");
    let padding = "x".repeat(11 * 1024 * 1024 - content.len());
    content.push_str(&padding);
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
fn convert_missing_format_flag_defaults_to_json() {
    // EC-1 / T003: omitted --format → defaults to JSON, succeeds
    let dir = TempDir::new().unwrap();
    let content = "# Title\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Should succeed when --format omitted (defaults to json), stderr: {stderr}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout should be valid JSON: {e}\nOutput: {stdout}"));
    assert!(json["catalog"].is_object(), "Should produce OSCAL catalog JSON");
}

#[test]
fn convert_format_xml_produces_valid_xml() {
    let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
    if !fixture.exists() {
        return;
    }

    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("catalog.xml");

    let output = forge_bin()
        .arg("convert")
        .arg(fixture)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("xml")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "XML output should succeed, stderr: {stderr}");
    assert!(output_path.exists(), "XML output file should be created");

    let xml = std::fs::read_to_string(&output_path).unwrap();
    assert!(xml.contains("<?xml"), "Output must contain XML declaration");
    assert!(xml.contains("<catalog"), "Output must contain <catalog> element");
}

#[test]
fn convert_strategy_component_without_source_profile_succeeds_with_warning() {
    // T014 (S-1): --strategy component without --source-profile → warning about missing
    // source-profile. Empty control-implementations is omitted (skip_serializing_if),
    // so the output passes schema validation (field is optional in OSCAL schema).
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("component")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify warning about missing source-profile on stderr
    assert!(
        stderr.contains("source-profile") && stderr.contains("control-id mapping"),
        "Should warn about missing source-profile on stderr: {stderr}"
    );

    // With skip_serializing_if, empty control-implementations is omitted,
    // making the output valid OSCAL (field is optional).
    assert!(
        output.status.success(),
        "Should succeed: empty control-implementations is omitted from output, stderr: {stderr}"
    );
}

// =============================================================================
// T006–T009, T022–T023: Component pipeline CLI integration tests (WI-18 Phase 2)
// =============================================================================

/// T006 [US1] — Full component pipeline produces valid Component Definition JSON.
/// Covers: AC-1, AC-2, AC-3, AC-5, M-3, M-4, M-5, M-7
#[test]
fn convert_component_strategy_produces_valid_component_definition() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    // Top-level key
    assert!(
        json["component-definition"].is_object(),
        "Should have 'component-definition' top-level key"
    );

    let cd = &json["component-definition"];

    // UUID
    assert!(cd["uuid"].is_string(), "component-definition.uuid should be a string");
    assert!(
        !cd["uuid"].as_str().unwrap().is_empty(),
        "component-definition.uuid should not be empty"
    );

    // Metadata
    let metadata = &cd["metadata"];
    assert_eq!(
        metadata["title"].as_str().unwrap(),
        "Sample Security Policy",
        "metadata.title should match frontmatter"
    );
    assert_eq!(
        metadata["version"].as_str().unwrap(),
        "1.0.0",
        "metadata.version should match frontmatter"
    );
    assert_eq!(
        metadata["oscal-version"].as_str().unwrap(),
        "1.2.0",
        "metadata.oscal-version should be 1.2.0"
    );
    assert!(metadata["last-modified"].is_string(), "metadata.last-modified should be a string");
    assert!(
        !metadata["last-modified"].as_str().unwrap().is_empty(),
        "metadata.last-modified should not be empty"
    );

    // Components
    let components = cd["components"].as_array().expect("components should be an array");
    assert_eq!(components.len(), 1, "Should have exactly 1 component");
    assert_eq!(
        components[0]["type"].as_str().unwrap(),
        "policy",
        "components[0].type should be 'policy'"
    );

    // Control implementations
    let ctrl_impls = components[0]["control-implementations"]
        .as_array()
        .expect("control-implementations should be an array");
    assert_eq!(ctrl_impls.len(), 1, "Should have exactly 1 control-implementation");

    // Implemented requirements (non-empty)
    let impl_reqs = ctrl_impls[0]["implemented-requirements"]
        .as_array()
        .expect("implemented-requirements should be an array");
    assert!(
        !impl_reqs.is_empty(),
        "implemented-requirements should not be empty for full_policy.md"
    );
}

/// T007 [US1] — --format omitted defaults to JSON for component strategy.
/// Covers: T-EC1, EC-1
#[test]
fn convert_component_strategy_format_omitted_defaults_to_json() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("Stdout should be valid JSON when --format omitted: {e}\nOutput: {stdout}")
    });

    assert!(
        json["component-definition"].is_object(),
        "Should have 'component-definition' top-level key when --format omitted"
    );
}

/// T008 [US1] — --output writes Component Definition to file.
/// Covers: AC-6, M-8
#[test]
fn convert_component_strategy_output_to_file() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.json");

    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    // Stdout should be empty (output went to file)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "Stdout should be empty when --output is used, got: {stdout}"
    );

    // Read file and parse as JSON
    let file_content = fs::read_to_string(&output_path)
        .unwrap_or_else(|e| panic!("Failed to read output file: {e}"));
    let json: serde_json::Value = serde_json::from_str(&file_content)
        .unwrap_or_else(|e| panic!("Output file is not valid JSON: {e}"));

    assert!(
        json["component-definition"].is_object(),
        "File should contain 'component-definition' top-level key"
    );
}

/// T009 [US2] — Trace props present in implemented-requirements.
/// Covers: AC-4, M-6, SEC-1
#[test]
fn convert_component_strategy_has_trace_props() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    let impl_reqs = json["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"]
        .as_array()
        .expect("implemented-requirements should be an array");

    assert!(!impl_reqs.is_empty(), "implemented-requirements should not be empty");

    for (i, req) in impl_reqs.iter().enumerate() {
        let props = req["props"]
            .as_array()
            .unwrap_or_else(|| panic!("implemented-requirement[{i}] should have 'props' array"));

        // Collect prop names for assertion messages
        let prop_names: Vec<&str> = props.iter().filter_map(|p| p["name"].as_str()).collect();

        assert!(
            prop_names.contains(&"source-file"),
            "implemented-requirement[{i}] props should contain 'source-file', got: {prop_names:?}"
        );
        assert!(
            prop_names.contains(&"source-section"),
            "implemented-requirement[{i}] props should contain 'source-section', got: {prop_names:?}"
        );
        assert!(
            prop_names.contains(&"source-line"),
            "implemented-requirement[{i}] props should contain 'source-line', got: {prop_names:?}"
        );

        // SEC-1: source-file value is filename-only (no path separators)
        let source_file_prop = props
            .iter()
            .find(|p| p["name"].as_str() == Some("source-file"))
            .expect("source-file prop should exist");
        let source_file_value =
            source_file_prop["value"].as_str().expect("source-file value should be a string");
        assert!(
            !source_file_value.contains('/') && !source_file_value.contains('\\'),
            "SEC-1: source-file should be filename-only (no path separators), got: {source_file_value}"
        );

        // source-line value should be a parseable number > 0
        let source_line_prop = props
            .iter()
            .find(|p| p["name"].as_str() == Some("source-line"))
            .expect("source-line prop should exist");
        let source_line_value =
            source_line_prop["value"].as_str().expect("source-line value should be a string");
        let line_num: u64 = source_line_value.parse().unwrap_or_else(|e| {
            panic!("source-line value '{source_line_value}' should be a number: {e}")
        });
        assert!(line_num > 0, "source-line should be > 0, got: {line_num}");
    }
}

/// T022 [US1] — Zero extractable requirements produces empty implemented-requirements.
/// Covers: EC-2
#[test]
fn convert_component_strategy_zero_requirements_empty_control_implementations() {
    // Zero extractable requirements produces empty implemented-requirements, which
    // fails OSCAL schema validation (minItems: 1). Expect non-zero exit.
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"No Requirements\"\nversion: \"1.0\"\n---\n\n# Section\n\nJust a paragraph with no requirements.\n";
    let path = create_temp_md(&dir, "no_reqs.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Zero extractable requirements produces empty implemented-requirements,
    // which fails OSCAL schema validation (minItems: 1). The CLI should
    // report a schema validation error rather than succeed.
    assert!(
        !output.status.success(),
        "Should fail schema validation with zero requirements, stderr: {stderr}"
    );
    assert!(
        stderr.contains("schema") || stderr.contains("Schema"),
        "Should mention schema validation error on stderr: {stderr}"
    );
}

// =============================================================================
// T015–T016: Source Profile Validation (WI-18 Phase 4, US-4)
// =============================================================================

/// T015 [US4] — Non-existent --source-profile path produces descriptive error and exits non-zero.
/// Covers: T-S2-03, AC-8, SEC-3
#[test]
fn convert_component_strategy_nonexistent_source_profile_errors() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("nonexistent/path/profile.json")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    assert!(
        !output.status.success(),
        "Expected non-zero exit code for non-existent source-profile path"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("source-profile") || stderr.contains("source_profile"),
        "Error should mention source-profile: {stderr}"
    );
    assert!(
        stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("No such file"),
        "Error should indicate file not found: {stderr}"
    );
}

/// T016 [US4] — Directory path as --source-profile produces descriptive error and exits non-zero.
/// Covers: SEC-3
#[test]
fn convert_component_strategy_directory_as_source_profile_errors() {
    let dir = TempDir::new().unwrap();

    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg(dir.path())
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    assert!(
        !output.status.success(),
        "Expected non-zero exit code for directory as source-profile"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("source-profile") || stderr.contains("source_profile"),
        "Error should mention source-profile: {stderr}"
    );
    assert!(
        stderr.contains("not a file")
            || stderr.contains("not a regular file")
            || stderr.contains("is a directory"),
        "Error should indicate path is not a file: {stderr}"
    );
}

/// T023 [US1] — Source profile with no matching control IDs still produces valid output.
/// The pipeline uses `source_profile` as a string reference — it does not parse or validate the file.
/// Covers: EC-3
#[test]
fn convert_component_strategy_no_matching_control_ids() {
    // Create a temp file with a distinctive name to verify source string is reflected in output
    let dir = TempDir::new().unwrap();
    let profile_path = dir.path().join("some-other-baseline.json");
    fs::write(&profile_path, "{}").unwrap();

    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg(&profile_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Stdout is not valid JSON: {e}\nOutput: {stdout}"));

    assert!(
        json["component-definition"].is_object(),
        "Should have 'component-definition' top-level key with any profile string"
    );

    // Verify the source profile path is reflected in the output
    let ctrl_impls = json["component-definition"]["components"][0]["control-implementations"]
        .as_array()
        .expect("control-implementations should be an array");
    assert!(!ctrl_impls.is_empty(), "Should have at least one control-implementation");
    let source = ctrl_impls[0]["source"].as_str().unwrap();
    assert!(
        source.contains("some-other-baseline.json"),
        "source should reflect the provided --source-profile value, got: {source}"
    );
}

// T023 [US5] Exit code integration tests

#[test]
fn exit_code_1_for_file_not_found() {
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
    assert_eq!(output.status.code(), Some(1), "FileNotFound should exit with code 1");
}

#[test]
fn exit_code_2_for_no_structure_detected() {
    let dir = TempDir::new().unwrap();
    let content = "This is just plain text without any headings or structure.\nNo sections here.\n";
    let path = create_temp_md(&dir, "flat.md", content);

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
    assert_eq!(output.status.code(), Some(2), "NoStructureDetected should exit with code 2");
}

#[test]
fn exit_code_3_for_validate_command() {
    let output =
        forge_bin().arg("validate").arg("any.json").output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");
    assert_eq!(output.status.code(), Some(3), "Validation error should exit with code 3");
}

// =========================================================================
// WI-25 Phase 3: User Story 1 — Error message tests (T013, T014)
// =========================================================================

/// T013 [US1] S-3, AC-11, SEC-4: Missing file produces descriptive error, no panic.
#[test]
fn test_error_message_missing_file() {
    let output = forge_bin()
        .arg("convert")
        .arg("absolutely_nonexistent_file.md")
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain descriptive error with file path
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "Error should mention file not found: {stderr}"
    );

    // SEC-4: No internal Rust module paths in error
    assert!(
        !stderr.contains("src::") && !stderr.contains("src/"),
        "Error should not contain internal Rust paths (SEC-4): {stderr}"
    );

    // No panic/backtrace
    assert!(
        !stderr.contains("panicked") && !stderr.contains("RUST_BACKTRACE"),
        "Error should not contain panic output: {stderr}"
    );
}

/// T014 [US1] EC-5, SEC-4: Invalid JSON for validate produces descriptive error.
#[test]
fn test_error_message_invalid_json_for_validate() {
    let dir = TempDir::new().unwrap();
    let path = create_temp_md(&dir, "not_json.json", "This is not JSON content at all.");

    let output =
        forge_bin().arg("validate").arg(&path).output().expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should mention JSON or parse error
    assert!(
        stderr.contains("JSON") || stderr.contains("parse") || stderr.contains("json"),
        "Error should mention JSON parse issue: {stderr}"
    );

    // SEC-4: No internal Rust module paths
    assert!(
        !stderr.contains("src::") && !stderr.contains("src/"),
        "Error should not contain internal Rust paths (SEC-4): {stderr}"
    );
}

// =========================================================================
// WI-25 Phase 5: User Story 3 — CLI Help and Discoverability (T021-T024)
// =========================================================================

/// T021 [US3] M-5, AC-5, EC-3: Help text lists all subcommands.
#[test]
fn test_help_text_lists_all_subcommands() {
    let output = forge_bin().arg("--help").output().expect("Failed to execute process");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success exit code");

    // Must contain both subcommands
    assert!(stdout.contains("convert"), "Help should list 'convert': {stdout}");
    assert!(stdout.contains("validate"), "Help should list 'validate': {stdout}");

    // Must mention verbose/quiet flags
    assert!(
        stdout.contains("verbose") || stdout.contains("-v"),
        "Help should mention verbose flag: {stdout}"
    );
    assert!(
        stdout.contains("quiet") || stdout.contains("-q"),
        "Help should mention quiet flag: {stdout}"
    );
}

/// T022 [US3] M-5: Convert help lists all options.
#[test]
fn test_convert_help_lists_all_options() {
    let output =
        forge_bin().args(["convert", "--help"]).output().expect("Failed to execute process");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success exit code");

    // All convert options
    assert!(stdout.contains("strategy"), "Should list --strategy: {stdout}");
    assert!(stdout.contains("format"), "Should list --format: {stdout}");
    assert!(stdout.contains("output"), "Should list --output: {stdout}");
    assert!(stdout.contains("max-size"), "Should list --max-size: {stdout}");
    assert!(stdout.contains("source-profile"), "Should list --source-profile: {stdout}");
}

/// T023 [US3] M-5: Validate help lists all options.
#[test]
fn test_validate_help_lists_all_options() {
    let output =
        forge_bin().args(["validate", "--help"]).output().expect("Failed to execute process");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success exit code");

    // All validate options
    assert!(stdout.contains("schema-type"), "Should list --schema-type: {stdout}");
    assert!(stdout.contains("format"), "Should list --format: {stdout}");
}

/// T024 [US3]: Version flag outputs version string.
#[test]
fn test_version_flag() {
    let output = forge_bin().arg("--version").output().expect("Failed to execute process");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success exit code");
    assert!(stdout.contains("forge"), "Version output should contain 'forge': {stdout}");
}

// =========================================================================
// WI-25 Phase 6: User Story 4 — Verbose/Quiet Output Control (T029-T031)
// =========================================================================

/// T029 [US4] S-1, AC-8, SEC-7: Verbose flag shows pipeline stage messages on stderr.
#[test]
fn test_verbose_flag_shows_pipeline_stages() {
    let dir = TempDir::new().unwrap();
    let content =
        "---\ntitle: \"Test\"\nversion: \"1.0\"\n---\n\n# Section\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("--verbose")
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success with --verbose, stderr: {stderr}");

    // SEC-7: tracing output should be on stderr, not stdout
    // Verbose mode enables DEBUG level — should show some tracing output
    assert!(!stderr.is_empty(), "Verbose mode should produce tracing output on stderr");

    // Verify tracing output contains expected level indicators from tracing_subscriber
    assert!(
        stderr.contains("DEBUG") || stderr.contains("INFO") || stderr.contains("TRACE"),
        "Verbose mode should include DEBUG/INFO/TRACE level output on stderr: {stderr}"
    );

    // stdout should contain JSON output, not tracing messages
    assert!(stdout.contains("catalog"), "stdout should contain OSCAL JSON, not tracing: {stdout}");
}

/// T030 [US4] S-1, AC-9: Quiet flag suppresses non-essential output.
#[test]
fn test_quiet_flag_suppresses_output() {
    let dir = TempDir::new().unwrap();
    let content =
        "---\ntitle: \"Test\"\nversion: \"1.0\"\n---\n\n# Section\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("--quiet")
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Expected success with --quiet, stderr: {stderr}");

    // Quiet mode sets tracing_subscriber filter to "error" level, which suppresses
    // INFO/WARN/DEBUG prefixes from tracing output (only ERROR-level messages appear).
    assert!(
        !stderr.contains("INFO") && !stderr.contains("WARN") && !stderr.contains("DEBUG"),
        "Quiet mode should suppress INFO/WARN/DEBUG on stderr: {stderr}"
    );

    // stdout should still have JSON output
    assert!(
        stdout.contains("catalog"),
        "stdout should contain OSCAL JSON even in quiet mode: {stdout}"
    );
}

/// T031 [US4] S-1, EC-4: Verbose + quiet conflict produces clear error.
#[test]
fn test_verbose_quiet_conflict_error() {
    let output = forge_bin()
        .arg("--verbose")
        .arg("--quiet")
        .arg("convert")
        .arg("test.md")
        .arg("--strategy")
        .arg("catalog")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code for --verbose --quiet conflict");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // clap should report the conflict
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflict"),
        "Error should mention flag conflict: {stderr}"
    );
}

/// Test that --max-size with an overflow-inducing value produces a validation error, not a panic.
#[test]
fn test_convert_max_size_overflow_produces_error() {
    let output = forge_bin()
        .arg("convert")
        .arg("test.md")
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("json")
        .arg("--max-size")
        .arg("18446744073709551")
        .output()
        .expect("Failed to execute process");

    assert!(!output.status.success(), "Expected non-zero exit code for overflow --max-size");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("too large"), "Should report --max-size value is too large: {stderr}");
}

// =========================================================================
// WI-27: YAML Output — CLI Integration Tests (T006, T007)
// =========================================================================

/// T006 [US1] — `--format yaml` catalog stdout produces valid YAML (no JSON braces).
#[test]
fn convert_catalog_format_yaml_stdout_valid_yaml() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // YAML output should NOT start with JSON brace
    assert!(!stdout.trim_start().starts_with('{'), "YAML output should not start with JSON brace");
    // Should be parseable as YAML
    let value: serde_json::Value =
        serde_yaml::from_str(&stdout).expect("stdout should be valid YAML");
    assert!(value["catalog"].is_object(), "YAML should contain 'catalog' key");
}

/// T006 [US1] — `--format yaml --output` catalog creates parseable YAML file.
#[test]
fn convert_catalog_format_yaml_file_output() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);
    let output_path = dir.path().join("catalog.yaml");

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("yaml")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let file_content = fs::read_to_string(&output_path).expect("Should read YAML file");
    let value: serde_json::Value =
        serde_yaml::from_str(&file_content).expect("File should be valid YAML");
    assert!(value["catalog"].is_object(), "YAML file should contain 'catalog' key");
}

/// T006 [US1] — YAML output contains expected OSCAL catalog keys.
#[test]
fn convert_catalog_format_yaml_contains_oscal_keys() {
    let dir = TempDir::new().unwrap();
    let content = "---\ntitle: \"Test Policy\"\nversion: \"1.0\"\n---\n\n# Access Control\n\n- Users must authenticate.\n";
    let path = create_temp_md(&dir, "policy.md", content);

    let output = forge_bin()
        .arg("convert")
        .arg(&path)
        .arg("--strategy")
        .arg("catalog")
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_yaml::from_str(&stdout).expect("stdout should be valid YAML");

    let catalog = &value["catalog"];
    assert!(catalog["uuid"].is_string(), "Should have catalog.uuid");
    assert!(catalog["metadata"].is_object(), "Should have catalog.metadata");

    // AC-7: All OSCAL required metadata fields present
    let metadata = &catalog["metadata"];
    assert!(
        metadata["title"].is_string() && !metadata["title"].as_str().unwrap().is_empty(),
        "metadata.title should be non-empty string"
    );
    assert!(metadata["last-modified"].is_string(), "metadata.last-modified should be present");
    assert!(metadata["version"].is_string(), "metadata.version should be present");
    assert_eq!(
        metadata["oscal-version"].as_str(),
        Some("1.2.0"),
        "metadata.oscal-version should be 1.2.0"
    );
}

/// T007 [US2] — `--format yaml` component stdout produces valid YAML.
#[test]
fn convert_component_format_yaml_stdout_valid_yaml() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim_start().starts_with('{'), "YAML output should not start with JSON brace");

    let value: serde_json::Value =
        serde_yaml::from_str(&stdout).expect("stdout should be valid YAML");
    assert!(
        value["component-definition"].is_object(),
        "YAML should contain 'component-definition' key"
    );
}

/// T007 [US2] — `--format yaml --output` component creates parseable YAML file.
#[test]
fn convert_component_format_yaml_file_output() {
    let dir = TempDir::new().unwrap();
    let output_path = dir.path().join("component.yaml");

    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("yaml")
        .arg("--output")
        .arg(&output_path)
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let file_content = fs::read_to_string(&output_path).expect("Should read YAML file");
    let value: serde_json::Value =
        serde_yaml::from_str(&file_content).expect("File should be valid YAML");
    assert!(
        value["component-definition"].is_object(),
        "YAML file should contain 'component-definition' key"
    );
}

/// T007 [US2] — YAML component output contains expected keys and metadata.
#[test]
fn convert_component_format_yaml_contains_oscal_keys() {
    let output = forge_bin()
        .arg("convert")
        .arg("tests/fixtures/full_policy.md")
        .arg("--strategy")
        .arg("component")
        .arg("--source-profile")
        .arg("tests/fixtures/sample_profile.json")
        .arg("--format")
        .arg("yaml")
        .output()
        .expect("Failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Expected exit code 0, stderr: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_yaml::from_str(&stdout).expect("stdout should be valid YAML");

    let cd = &value["component-definition"];
    assert!(cd["uuid"].is_string(), "Should have component-definition.uuid");

    // AC-7: All OSCAL required metadata fields present
    let metadata = &cd["metadata"];
    assert!(
        metadata["title"].is_string() && !metadata["title"].as_str().unwrap().is_empty(),
        "metadata.title should be non-empty string"
    );
    assert!(metadata["last-modified"].is_string(), "metadata.last-modified should be present");
    assert!(metadata["version"].is_string(), "metadata.version should be present");
    assert_eq!(
        metadata["oscal-version"].as_str(),
        Some("1.2.0"),
        "metadata.oscal-version should be 1.2.0"
    );
}
