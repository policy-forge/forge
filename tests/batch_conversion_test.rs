use std::path::PathBuf;
use std::process::Command;

use tempfile::TempDir;

/// Helper: create a valid Markdown policy file that the pipeline can convert.
fn create_policy_file(dir: &std::path::Path, name: &str, title: &str) -> PathBuf {
    let path = dir.join(name);
    let content = format!(
        r#"---
title: {title}
version: "1.0.0"
---

# {title}

## Access Control

- All users must authenticate before accessing the system.
- Access privileges must be reviewed quarterly.
"#
    );
    std::fs::write(&path, content).unwrap();
    path
}

/// Helper: create an invalid (structureless) Markdown file.
fn create_invalid_file(dir: &std::path::Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, "Just plain text without any structure.\n").unwrap();
    path
}

/// Helper: run forge convert with given args and return (exit code, stdout, stderr).
fn run_forge(args: &[&str]) -> (i32, String, String) {
    let output =
        Command::new(env!("CARGO_BIN_EXE_forge")).args(args).output().expect("Failed to run forge");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

// T015: Batch of 3 valid files → 3 output files
#[test]
fn batch_three_valid_files_produces_three_outputs() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "policy1.md", "Policy One");
    let f2 = create_policy_file(input_dir.path(), "policy2.md", "Policy Two");
    let f3 = create_policy_file(input_dir.path(), "policy3.md", "Policy Three");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        f3.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_eq!(code, 0, "Expected exit 0, stderr: {stderr}");
    assert!(output_dir.path().join("policy1.json").exists());
    assert!(output_dir.path().join("policy2.json").exists());
    assert!(output_dir.path().join("policy3.json").exists());
}

// T016: Single-file backward compatibility
#[test]
fn single_file_backward_compatibility() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "policy.md", "My Policy");
    let out_file = output_dir.path().join("policy.json");

    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out_file.to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "Single file should succeed, stderr: {stderr}");
    assert!(out_file.exists(), "Output file should exist");
}

// T017: Filename collision → _2 suffix
#[test]
fn filename_collision_produces_suffix() {
    let dir1 = TempDir::new().unwrap();
    let dir2 = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(dir1.path(), "policy.md", "Policy A");
    let f2 = create_policy_file(dir2.path(), "policy.md", "Policy B");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_eq!(code, 0, "Expected exit 0, stderr: {stderr}");
    assert!(output_dir.path().join("policy.json").exists(), "First file should be policy.json");
    assert!(
        output_dir.path().join("policy_2.json").exists(),
        "Second file should be policy_2.json"
    );
}

// T018: --output is a file (not dir) with multiple inputs → error
#[test]
fn output_is_file_with_multiple_inputs_errors() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "a.md", "A");
    let f2 = create_policy_file(input_dir.path(), "b.md", "B");

    // Create a regular file as --output
    let out_file = output_dir.path().join("output.json");
    std::fs::write(&out_file, "{}").unwrap();

    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out_file.to_str().unwrap(),
    ]);

    assert_ne!(code, 0, "Should fail when --output is a file");
    assert!(
        stderr.contains("directory") || stderr.contains("not a file"),
        "Error should mention directory requirement, got: {stderr}"
    );
}

// T019: Zero input files → descriptive error
#[test]
fn zero_input_files_errors() {
    let (code, _stdout, stderr) = run_forge(&["convert", "--strategy", "catalog"]);

    assert_ne!(code, 0, "Should fail with zero inputs");
    // clap will report the error about missing required argument
    assert!(
        stderr.contains("required") || stderr.contains("error"),
        "Should report error about missing input, got: {stderr}"
    );
}

// T020: --output dir does not exist → auto-created
#[test]
fn output_dir_auto_created() {
    let input_dir = TempDir::new().unwrap();
    let base_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "a.md", "A");
    let f2 = create_policy_file(input_dir.path(), "b.md", "B");

    let new_out_dir = base_dir.path().join("new_output_dir");
    assert!(!new_out_dir.exists());

    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        new_out_dir.to_str().unwrap(),
    ]);

    assert_eq!(code, 0, "Should succeed and create dir, stderr: {stderr}");
    assert!(new_out_dir.exists(), "Output dir should be auto-created");
    assert!(new_out_dir.join("a.json").exists());
    assert!(new_out_dir.join("b.json").exists());
}

// T021: Batch without --output → files in current directory
#[test]
fn batch_without_output_writes_to_current_dir() {
    let input_dir = TempDir::new().unwrap();
    let work_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "alpha.md", "Alpha");
    let f2 = create_policy_file(input_dir.path(), "beta.md", "Beta");

    let output = Command::new(env!("CARGO_BIN_EXE_forge"))
        .current_dir(work_dir.path())
        .args([
            "convert",
            f1.to_str().unwrap(),
            f2.to_str().unwrap(),
            "--strategy",
            "catalog",
            "--format",
            "json",
        ])
        .output()
        .expect("Failed to run forge");

    let code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(code, 0, "Should succeed, stderr: {stderr}");
    assert!(work_dir.path().join("alpha.json").exists(), "alpha.json should be in current dir");
    assert!(work_dir.path().join("beta.json").exists(), "beta.json should be in current dir");
}

// ============================================================
// Phase 4: US2 — Aggregated Status Output
// ============================================================

// T029: Mixed batch (2 valid + 1 invalid) → aggregated status
#[test]
fn mixed_batch_shows_success_and_failure_counts() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "good1.md", "Good One");
    let f2 = create_policy_file(input_dir.path(), "good2.md", "Good Two");
    let f3 = create_invalid_file(input_dir.path(), "bad.md");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        f3.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_ne!(code, 0, "Should fail when any file fails");
    assert!(
        stderr.contains("2 succeeded") && stderr.contains("1 failed"),
        "Should show 2 succeeded, 1 failed. Got: {stderr}"
    );
    // Verify per-file lines in stderr
    assert!(stderr.contains("bad.md"), "Should mention failed file");
}

// T030: All-success batch → aggregated status on stderr
#[test]
fn all_success_batch_shows_summary_on_stderr() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "a.md", "Alpha");
    let f2 = create_policy_file(input_dir.path(), "b.md", "Beta");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_eq!(code, 0, "All success should exit 0, stderr: {stderr}");
    assert!(
        stderr.contains("2 succeeded") && stderr.contains("0 failed"),
        "Should show all succeeded. Got: {stderr}"
    );
}

// T031: Batch with any failure → non-zero exit code
#[test]
fn batch_with_failure_nonzero_exit() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "good.md", "Good");
    let f2 = create_invalid_file(input_dir.path(), "bad.md");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, _stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_ne!(code, 0, "Should have non-zero exit on any failure");
}

// T032: All-failure batch → all failures, exit code non-zero
#[test]
fn all_failure_batch_nonzero_exit() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_invalid_file(input_dir.path(), "bad1.md");
    let f2 = create_invalid_file(input_dir.path(), "bad2.md");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_ne!(code, 0, "All-failure batch should exit non-zero");
    assert!(
        stderr.contains("0 succeeded") && stderr.contains("2 failed"),
        "Should show all failed. Got: {stderr}"
    );
}

// T033: Aggregated status is on stderr (not stdout), OSCAL output only in files
#[test]
fn status_on_stderr_not_stdout() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "a.md", "A");
    let f2 = create_policy_file(input_dir.path(), "b.md", "B");

    let out = output_dir.path().to_str().unwrap();
    let (code, stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
    ]);

    assert_eq!(code, 0, "Should succeed, stderr: {stderr}");
    // Status should be on stderr
    assert!(
        stderr.contains("Batch conversion complete"),
        "Summary should be on stderr. Got: {stderr}"
    );
    // stdout should be empty (OSCAL output is in files only)
    assert!(stdout.is_empty(), "stdout should be empty for batch mode. Got: {stdout}");
}

// ============================================================
// Phase 5: US3 — Parallel Processing
// ============================================================

// T038: --jobs 1 → sequential processing
#[test]
fn jobs_one_sequential_processing() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "a.md", "A");
    let f2 = create_policy_file(input_dir.path(), "b.md", "B");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
        "--jobs",
        "1",
    ]);

    assert_eq!(code, 0, "Should succeed with --jobs 1, stderr: {stderr}");
    assert!(output_dir.path().join("a.json").exists());
    assert!(output_dir.path().join("b.json").exists());
}

// T040: Error isolation under parallelism
#[test]
fn error_isolation_under_parallelism() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let f1 = create_policy_file(input_dir.path(), "good.md", "Good Policy");
    let f2 = create_invalid_file(input_dir.path(), "bad.md");
    let f3 = create_policy_file(input_dir.path(), "also_good.md", "Also Good");

    let out = output_dir.path().to_str().unwrap();
    let (code, _stdout, stderr) = run_forge(&[
        "convert",
        f1.to_str().unwrap(),
        f2.to_str().unwrap(),
        f3.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        out,
        "--jobs",
        "4",
    ]);

    // Should have non-zero exit (one failure)
    assert_ne!(code, 0, "Should fail when any file fails");
    // Good files should still produce output
    assert!(
        output_dir.path().join("good.json").exists(),
        "good.json should exist despite bad.md failing"
    );
    assert!(
        output_dir.path().join("also_good.json").exists(),
        "also_good.json should exist despite bad.md failing"
    );
    assert!(
        stderr.contains("2 succeeded") && stderr.contains("1 failed"),
        "Should show 2 succeeded, 1 failed. Got: {stderr}"
    );
}

// ============================================================
// Phase 6: US4 — Glob Pattern Input
// ============================================================

// T045: Simulated glob expansion (5 files → 5 outputs)
#[test]
fn five_files_simulating_glob_expansion() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let files: Vec<PathBuf> = (1..=5)
        .map(|i| {
            create_policy_file(input_dir.path(), &format!("policy{i}.md"), &format!("Policy {i}"))
        })
        .collect();

    let mut args: Vec<&str> = vec!["convert"];
    let file_strs: Vec<String> = files.iter().map(|f| f.to_str().unwrap().to_string()).collect();
    for f in &file_strs {
        args.push(f);
    }
    args.extend([
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        output_dir.path().to_str().unwrap(),
    ]);

    let (code, _stdout, stderr) = run_forge(&args);
    assert_eq!(code, 0, "Should succeed with 5 files, stderr: {stderr}");

    for i in 1..=5 {
        assert!(
            output_dir.path().join(format!("policy{i}.json")).exists(),
            "policy{i}.json should exist"
        );
    }
}

// T046: >100 file warning
#[test]
fn large_batch_warning_over_100_files() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    let files: Vec<PathBuf> = (1..=101)
        .map(|i| {
            create_policy_file(input_dir.path(), &format!("p{i:03}.md"), &format!("Policy {i}"))
        })
        .collect();

    let mut args: Vec<&str> = vec!["convert"];
    let file_strs: Vec<String> = files.iter().map(|f| f.to_str().unwrap().to_string()).collect();
    for f in &file_strs {
        args.push(f);
    }
    args.extend([
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        output_dir.path().to_str().unwrap(),
        "-v",
    ]);

    let (code, _stdout, stderr) = run_forge(&args);
    // With -v (verbose), the warning should be visible
    assert_eq!(code, 0, "Should succeed with 101 files, stderr: {stderr}");
    assert!(
        stderr.contains("Large batch") || stderr.contains("101 files"),
        "Should warn about large batch. Got stderr (first 500 chars): {}",
        &stderr[..stderr.len().min(500)]
    );
}
