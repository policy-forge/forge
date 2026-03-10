use std::path::Path;

use forge::trace::formatter::format_trace_table;
use forge::trace::generate_trace_report;
use forge::trace::report::{ArtifactType, ElementType};

// T026: Integration test — generate_trace_report with catalog fixture
#[test]
fn generate_report_from_catalog_fixture() {
    let artifact = Path::new("tests/fixtures/catalog-with-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();

    assert_eq!(report.artifact_type, ArtifactType::Catalog);
    // 2 groups + 4 controls = 6 entries
    assert_eq!(report.entries.len(), 6);
    assert_eq!(report.summary.total_elements, 6);
    assert_eq!(report.summary.mapped_elements, 6);
    assert_eq!(report.summary.unmapped_elements, 0);

    // Verify groups
    assert_eq!(report.entries[0].element_id, "access-control");
    assert_eq!(report.entries[0].element_type, ElementType::Group);
    assert!(report.entries[0].trace.is_some());

    // Verify controls
    assert_eq!(report.entries[1].element_id, "POL-AC-001");
    assert_eq!(report.entries[1].element_type, ElementType::Control);
    assert!(report.entries[1].trace.is_some());
    let meta = report.entries[1].trace.as_ref().unwrap();
    assert_eq!(meta.source_line, 5);
    assert_eq!(meta.source_section, "Access Control");
}

// T027: Integration test — generate_trace_report with compdef fixture
#[test]
fn generate_report_from_compdef_fixture() {
    let artifact = Path::new("tests/fixtures/compdef-with-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();

    assert_eq!(report.artifact_type, ArtifactType::ComponentDefinition);
    assert_eq!(report.entries.len(), 3);

    for entry in &report.entries {
        assert_eq!(entry.element_type, ElementType::ImplementedRequirement);
        assert!(entry.trace.is_some());
    }

    assert_eq!(report.entries[0].element_id, "POL-AC-001");
    assert_eq!(report.entries[1].element_id, "POL-AC-002");
    assert_eq!(report.entries[2].element_id, "POL-DP-001");
}

// T028: Integration test — error cases
#[test]
fn error_missing_artifact_file() {
    let result = generate_trace_report(
        Path::new("nonexistent-artifact.json"),
        Path::new("tests/fixtures/trace-sample-policy.md"),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, forge::ForgeError::FileNotFound { .. }));
}

#[test]
fn error_missing_source_file() {
    let result = generate_trace_report(
        Path::new("tests/fixtures/catalog-with-trace.json"),
        Path::new("nonexistent-source.md"),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, forge::ForgeError::FileNotFound { .. }));
}

#[test]
fn error_invalid_json() {
    // Create a temp file with invalid JSON
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), "not valid json {{{").unwrap();

    let result =
        generate_trace_report(temp.path(), Path::new("tests/fixtures/trace-sample-policy.md"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, forge::ForgeError::Parse(_)));
}

#[test]
fn error_unsupported_type() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), r#"{"profile": {"uuid": "123"}}"#).unwrap();

    let result =
        generate_trace_report(temp.path(), Path::new("tests/fixtures/trace-sample-policy.md"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, forge::ForgeError::TraceUnsupportedArtifact { .. }));
}

// T030: Integration test — file output
#[test]
fn output_to_file() {
    let artifact = Path::new("tests/fixtures/catalog-with-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();
    let table = format_trace_table(&report);

    let temp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp.path(), &table).unwrap();

    let written = std::fs::read_to_string(temp.path()).unwrap();
    assert_eq!(written, table);
}

// T031: Integration test — stdout output (verify format_trace_table returns string)
#[test]
fn output_as_string_for_stdout() {
    let artifact = Path::new("tests/fixtures/catalog-with-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();
    let table = format_trace_table(&report);

    assert!(table.contains("OSCAL Element ID"));
    assert!(table.contains("POL-AC-001"));
    assert!(table.contains("Summary:"));
}

// T036: Integration test — partial trace coverage
#[test]
fn partial_trace_coverage() {
    let artifact = Path::new("tests/fixtures/catalog-partial-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();

    // 1 group + 2 controls = 3 entries
    assert_eq!(report.entries.len(), 3);

    // POL-AC-002 has no trace props
    let unmapped = report.entries.iter().find(|e| e.element_id == "POL-AC-002").unwrap();
    assert!(unmapped.trace.is_none());

    assert_eq!(report.summary.mapped_elements, 2); // group + POL-AC-001
    assert_eq!(report.summary.unmapped_elements, 1);
    assert!(report.summary.coverage_percent < 100.0);

    let table = format_trace_table(&report);
    assert!(table.contains("[unmapped]"));
}

// T039: Integration test — no trace metadata at all
#[test]
fn no_trace_metadata() {
    let artifact = Path::new("tests/fixtures/catalog-no-trace.json");
    let source = Path::new("tests/fixtures/trace-sample-policy.md");

    let report = generate_trace_report(artifact, source).unwrap();

    // 1 group (no props, so unmapped) + 2 controls (no props, so unmapped) = 3 entries
    assert_eq!(report.entries.len(), 3);
    assert_eq!(report.summary.mapped_elements, 0);
    assert_eq!(report.summary.unmapped_elements, 3);
    assert!((report.summary.coverage_percent - 0.0).abs() < f64::EPSILON);

    let table = format_trace_table(&report);
    assert!(table.contains("0.0% coverage"));
}

// T048: Integration test — staleness warning with future-dated source
#[test]
fn staleness_warning_with_newer_source() {
    use std::io::Write;

    // Create a temp source file (its mtime will be "now", which is after the artifact's
    // last-modified of 2026-01-15T10:30:00Z — but only if "now" is after that date)
    let mut temp_source = tempfile::NamedTempFile::new().unwrap();
    writeln!(temp_source, "# Policy\n\nSome content").unwrap();

    // Use the catalog fixture which has last-modified: "2026-01-15T10:30:00Z"
    // Set the temp file's mtime to a far-future date to guarantee staleness
    let future_time = filetime::FileTime::from_unix_time(2_000_000_000, 0); // ~2033
    filetime::set_file_mtime(temp_source.path(), future_time).unwrap();

    let report = generate_trace_report(
        Path::new("tests/fixtures/catalog-with-trace.json"),
        temp_source.path(),
    )
    .unwrap();

    assert!(report.source_stale);

    let table = format_trace_table(&report);
    assert!(table.contains("Warning: Source file may have been modified since conversion"));
}
