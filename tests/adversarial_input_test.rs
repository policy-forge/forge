use std::process::Command;

/// Select the actual compiled CLI executable for subprocess checks.
fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

/// Invoke the real catalog conversion route with complete required arguments.
fn run_convert(path: &str) -> std::process::Output {
    forge_bin()
        .args(["convert", path, "--strategy", "catalog", "--format", "json"])
        .output()
        .expect("Failed to execute process")
}

// --- T031: Adversarial input integration tests ---

/// An empty source must reach the descriptive controlled CLI error.
#[test]
fn empty_file_exits_nonzero_with_descriptive_error() {
    let output = run_convert("tests/fixtures/adversarial/empty.md");

    // Must not panic (exited cleanly)
    assert!(output.status.code().is_some(), "Process should exit cleanly, not via signal");
    assert!(!output.status.success(), "Expected non-zero exit code for empty file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty") || stderr.contains("Empty"),
        "stderr should mention empty file, got: {stderr}"
    );
}

/// A binary extension must reach the controlled unsupported-format error.
#[test]
fn binary_file_exits_nonzero_with_descriptive_error() {
    let output = run_convert("tests/fixtures/adversarial/binary.bin");

    assert!(output.status.code().is_some(), "Process should exit cleanly, not via signal");
    assert!(!output.status.success(), "Expected non-zero exit code for binary file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // .bin extension triggers UnsupportedFormat before binary detection
    assert!(
        stderr.contains("binary") || stderr.contains("Unsupported"),
        "stderr should mention binary or unsupported format, got: {stderr}"
    );
}

/// Binary Markdown content must reach the controlled ingestion error.
#[test]
fn null_bytes_file_exits_nonzero_with_descriptive_error() {
    let output = run_convert("tests/fixtures/adversarial/null_bytes.md");

    assert!(output.status.code().is_some(), "Process should exit cleanly, not via signal");
    assert!(!output.status.success(), "Expected non-zero exit code for null bytes file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // .md extension passes extension check; null bytes detected as binary content
    assert!(
        stderr.contains("binary") || stderr.contains("Binary"),
        "stderr should mention binary content, got: {stderr}"
    );
}

/// Whitespace-only input must reach the controlled missing-structure result.
#[test]
fn whitespace_only_file_exits_nonzero_with_descriptive_error() {
    let output = run_convert("tests/fixtures/adversarial/whitespace_only.md");

    assert!(output.status.code().is_some(), "Process should exit cleanly, not via signal");
    assert!(!output.status.success(), "Expected non-zero exit code for whitespace-only file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Whitespace-only passes ingest (has bytes, not binary, valid UTF-8) but
    // pipeline finds no headings and no clauses → NoStructureDetected
    assert!(
        stderr.contains("empty") || stderr.contains("structure") || stderr.contains("No policy"),
        "stderr should mention empty or no structure, got: {stderr}"
    );
}

/// Long input without newlines must reach a controlled result, never a startup crash.
#[test]
fn no_newlines_long_line_does_not_panic() {
    let output = run_convert("tests/fixtures/adversarial/no_newlines.md");

    // Primary check: no panic (process exits cleanly with an exit code)
    assert!(
        output.status.code().is_some(),
        "Process should exit cleanly (no signal/panic), got: {:?}",
        output.status
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("has overflowed its stack") && !stderr.contains("panicked at"),
        "Process must reach a controlled CLI result, got: {stderr}"
    );
    // This file has "# Policy" so it has structure; it should succeed
    // or produce a descriptive error — either is acceptable
    if !output.status.success() {
        assert!(!stderr.is_empty(), "If non-zero exit, stderr should contain a descriptive error");
    }
}

// --- T032: Large file test ---

/// An oversized actual source must reach the descriptive size-limit refusal.
#[test]
fn large_file_exceeds_default_limit() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("large.md");
    // Write 11MB of content
    let content = "# Large Policy\n".to_string() + &"x".repeat(11 * 1_048_576);
    std::fs::write(&path, &content).unwrap();

    let output = forge_bin()
        .args(["convert", path.to_str().unwrap(), "--strategy", "catalog", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.code().is_some(), "Process should exit cleanly, not via signal");
    assert!(!output.status.success(), "Expected non-zero exit code for oversized file");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("exceeding"), "Expected size limit error, got: {stderr}");
}
