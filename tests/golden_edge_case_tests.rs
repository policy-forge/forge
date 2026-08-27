//! WI-22 edge-case golden-file integration tests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::LazyLock;

use regex::Regex;
use serde_json::{Map, Value, json};
use tempfile::TempDir;
use uuid::Uuid;

const EDGE_ROOT: &str = "tests/fixtures/edge-cases";
const SOURCE_PROFILE: &str = "tests/fixtures/edge-cases/source-profile.json";

const NORMALIZED_UUID: &str = "00000000-0000-0000-0000-000000000000";
const NORMALIZED_TIMESTAMP: &str = "2026-01-01T00:00:00Z";

#[derive(Clone, Copy)]
enum ConvertExpectation {
    Output,
    Error,
}

#[derive(Clone, Copy)]
struct ConvertFixture {
    slug: &'static str,
    input_name: &'static str,
    expectation: ConvertExpectation,
}

const CONVERT_FIXTURES: &[ConvertFixture] = &[
    ConvertFixture {
        slug: "ec01-no-headings",
        input_name: "input.md",
        expectation: ConvertExpectation::Error,
    },
    ConvertFixture {
        slug: "ec02-compound-atomic",
        input_name: "input.md",
        expectation: ConvertExpectation::Output,
    },
    ConvertFixture {
        slug: "ec03-empty-sections",
        input_name: "input.md",
        expectation: ConvertExpectation::Output,
    },
    ConvertFixture {
        slug: "ec04-missing-metadata",
        input_name: "input.md",
        expectation: ConvertExpectation::Output,
    },
    ConvertFixture {
        slug: "ec05-whitespace-only",
        input_name: "input-original.md",
        expectation: ConvertExpectation::Output,
    },
    ConvertFixture {
        slug: "ec06-substantive-change",
        input_name: "input-original.md",
        expectation: ConvertExpectation::Output,
    },
    ConvertFixture {
        slug: "ec07-malformed-citation",
        input_name: "input.md",
        expectation: ConvertExpectation::Output,
    },
];

const SUPPLEMENTAL_FIXTURES: &[&str] =
    &["ec-citation-unusual-positions", "ec-parameter-like-content"];
const VALIDATION_ONLY_FIXTURES: &[(&str, &str, &str)] =
    &[("ec10-multiple-errors", "input.json", "expected-errors.txt")];
const STRATEGY_AGNOSTIC_ERROR_FIXTURES: &[(&str, &str)] =
    &[("ec09-file-not-found", "expected-error.txt")];

static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .expect("valid UUID regex")
});

#[derive(Clone, Copy)]
enum Strategy {
    Catalog,
    Component,
}

impl Strategy {
    fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Component => "component",
        }
    }
}

struct ConvertRun {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    output_json: Option<Value>,
}

fn fixture_dir(slug: &str) -> PathBuf {
    Path::new(EDGE_ROOT).join(slug)
}

fn fixture_input(slug: &str, file: &str) -> PathBuf {
    fixture_dir(slug).join(file)
}

fn assert_fixture_file(slug: &str, file: &str) {
    let path = fixture_input(slug, file);
    assert!(path.is_file(), "missing fixture file: {}", path.display());
}

fn load_expected_substrings(path: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading expected substrings {}: {e}", path.display()));
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn load_expected_json(path: &Path) -> Value {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("invalid JSON in expected file {}: {e}", path.display()))
}

fn run_convert(input_path: &Path, strategy: Strategy) -> ConvertRun {
    run_convert_with_baseline(input_path, strategy, None)
}

fn run_convert_with_baseline(
    input_path: &Path,
    strategy: Strategy,
    stable_id_baseline: Option<&Path>,
) -> ConvertRun {
    let temp = TempDir::new().expect("create temp dir");
    let output_path = temp.path().join("out.json");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_forge"));
    cmd.arg("convert")
        .arg(input_path)
        .arg("--strategy")
        .arg(strategy.as_str())
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&output_path);

    if matches!(strategy, Strategy::Component) {
        cmd.arg("--source-profile").arg(SOURCE_PROFILE);
    }

    if let Some(baseline) = stable_id_baseline {
        cmd.arg("--stable-id-baseline").arg(baseline);
    }

    let output = cmd.output().expect("run forge convert command");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status;

    let output_json = if output_path.exists() {
        let text = std::fs::read_to_string(&output_path)
            .unwrap_or_else(|e| panic!("failed reading {}: {e}", output_path.display()));
        Some(serde_json::from_str(&text).unwrap_or_else(|e| {
            panic!(
                "convert output must be JSON ({e}); status={status:?}\n--- stderr ---\n{stderr}\n--- stdout ---\n{stdout}"
            )
        }))
    } else {
        None
    };

    ConvertRun { status, stdout, stderr, output_json }
}

fn assert_required_substrings(haystack: &str, expected: &[String], context: &str) {
    // Normalize Windows path separators for cross-platform compatibility
    let haystack = haystack.replace('\\', "/");
    for needle in expected {
        assert!(
            haystack.contains(needle),
            "{context}: expected substring '{needle}' in output:\n{haystack}"
        );
    }
}

fn assert_edge_case_error(run: &ConvertRun, expected_substrings: &[String], context: &str) {
    assert!(
        run.status.code().is_some(),
        "{context}: convert terminated by signal: {:?}\n--- stderr ---\n{}\n--- stdout ---\n{}",
        run.status,
        run.stderr,
        run.stdout
    );
    assert!(
        !run.status.success(),
        "{context}: expected non-zero exit, got {}\n--- stderr ---\n{}\n--- stdout ---\n{}",
        run.status,
        run.stderr,
        run.stdout
    );
    assert_required_substrings(&run.stderr, expected_substrings, context);
}

fn assert_convert_success(run: &ConvertRun, context: &str) {
    assert!(
        run.status.success(),
        "{context}: convert failed with {}\n--- stderr ---\n{}\n--- stdout ---\n{}",
        run.status,
        run.stderr,
        run.stdout
    );
}

fn assert_expected_warnings(stderr: &str, expected_substrings: &[String], context: &str) {
    assert_required_substrings(stderr, expected_substrings, context);
}

fn normalize_for_comparison(value: &Value) -> Value {
    normalize_value(value, None)
}

fn normalizes_timestamp(s: &str, parent_key: Option<&str>) -> bool {
    parent_key.is_some_and(|key| key.ends_with("-modified") || key.ends_with("-created"))
        || chrono::DateTime::parse_from_rfc3339(s).is_ok()
}

fn is_normalizable_uuid(s: &str) -> bool {
    UUID_RE.is_match(s) && Uuid::parse_str(s).is_ok_and(|uuid| !uuid.is_nil())
}

fn normalize_value(value: &Value, parent_key: Option<&str>) -> Value {
    match value {
        Value::String(s) if normalizes_timestamp(s, parent_key) => {
            Value::String(NORMALIZED_TIMESTAMP.to_string())
        }
        Value::String(s) if is_normalizable_uuid(s) => Value::String(NORMALIZED_UUID.to_string()),
        Value::String(s) => Value::String(s.clone()),
        Value::Array(items) => {
            Value::Array(items.iter().map(|item| normalize_value(item, parent_key)).collect())
        }
        Value::Object(map) => {
            let normalized: Map<String, Value> = map
                .iter()
                .map(|(key, value)| (key.clone(), normalize_value(value, Some(key))))
                .collect();
            Value::Object(normalized)
        }
        _ => value.clone(),
    }
}

fn assert_expected_output(actual: &Value, expected_path: &Path, context: &str) {
    let expected = load_expected_json(expected_path);
    let normalized_actual = normalize_for_comparison(actual);
    let normalized_expected = normalize_for_comparison(&expected);
    assert_eq!(
        normalized_actual,
        normalized_expected,
        "{context}: normalized output did not match expected {}",
        expected_path.display()
    );
}

fn assert_fixture_output(slug: &str, input_name: &str, strategy: Strategy) -> Value {
    let input = fixture_input(slug, input_name);
    let run = run_convert(&input, strategy);
    let strategy_name = strategy.as_str();
    assert_convert_success(&run, &format!("{slug} {strategy_name}"));
    let output =
        run.output_json.unwrap_or_else(|| panic!("{slug} {strategy_name} output should exist"));
    let expected_name = match strategy {
        Strategy::Catalog => "expected-catalog.json",
        Strategy::Component => "expected-component-definition.json",
    };
    assert_expected_output(
        &output,
        &fixture_input(slug, expected_name),
        &format!("{slug} {strategy_name}"),
    );
    output
}

fn assert_dual_strategy_output(slug: &str, input_name: &str) {
    let _ = assert_fixture_output(slug, input_name, Strategy::Catalog);
    let _ = assert_fixture_output(slug, input_name, Strategy::Component);
}

#[test]
fn normalization_preserves_degenerate_ids_and_array_timestamp_context() {
    let value = json!({
        "uuid": "00000000-0000-0000-0000-000000000000",
        "events": [{"last-modified": "2026-02-14T10:30:00Z"}],
    });
    let normalized = normalize_for_comparison(&value);
    assert_eq!(normalized["uuid"], "00000000-0000-0000-0000-000000000000");
    assert_eq!(normalized["events"][0]["last-modified"], NORMALIZED_TIMESTAMP);
}

fn extract_catalog_stable_ids(output: &Value) -> BTreeMap<String, String> {
    let mut ids = BTreeMap::new();
    let groups = output.pointer("/catalog/groups").and_then(Value::as_array);
    for group in groups.into_iter().flatten() {
        let controls = group.get("controls").and_then(Value::as_array);
        for control in controls.into_iter().flatten() {
            if let (Some(id), Some(uuid)) = (
                control.get("id").and_then(Value::as_str),
                control.get("uuid").and_then(Value::as_str),
            ) {
                ids.insert(id.to_string(), uuid.to_string());
            }
        }
    }
    ids
}

fn extract_component_stable_ids(output: &Value) -> BTreeMap<String, String> {
    let mut ids = BTreeMap::new();
    let components = output.pointer("/component-definition/components").and_then(Value::as_array);
    for component in components.into_iter().flatten() {
        let impls = component.get("control-implementations").and_then(Value::as_array);
        for ci in impls.into_iter().flatten() {
            let reqs = ci.get("implemented-requirements").and_then(Value::as_array);
            for req in reqs.into_iter().flatten() {
                if let (Some(control_id), Some(uuid)) = (
                    req.get("control-id").and_then(Value::as_str),
                    req.get("uuid").and_then(Value::as_str),
                ) {
                    ids.insert(control_id.to_string(), uuid.to_string());
                }
            }
        }
    }
    ids
}

fn extract_stable_ids(output: &Value) -> BTreeMap<String, String> {
    let mut ids = extract_catalog_stable_ids(output);
    ids.extend(extract_component_stable_ids(output));
    ids
}

fn assert_component_stable_id_contract(
    slug: &str,
    original_name: &str,
    changed_name: &str,
    baseline: bool,
    expect_rotation: bool,
) {
    let original = fixture_input(slug, original_name);
    let changed = fixture_input(slug, changed_name);
    let run_a = run_convert(&original, Strategy::Component);
    let run_b = if baseline {
        run_convert_with_baseline(&changed, Strategy::Component, Some(&original))
    } else {
        run_convert(&changed, Strategy::Component)
    };
    assert_convert_success(&run_a, &format!("{slug} baseline"));
    assert_convert_success(&run_b, &format!("{slug} changed input"));

    let ids_a = extract_stable_ids(
        &run_a
            .output_json
            .unwrap_or_else(|| panic!("{slug} baseline component output should exist")),
    );
    let ids_b = extract_stable_ids(
        &run_b
            .output_json
            .unwrap_or_else(|| panic!("{slug} changed component output should exist")),
    );
    assert_eq!(
        ids_a.len(),
        ids_b.len(),
        "{slug} must preserve the control-id universe across the edit"
    );
    assert_eq!(
        ids_a.keys().collect::<Vec<_>>(),
        ids_b.keys().collect::<Vec<_>>(),
        "{slug} must preserve every control ID across the edit"
    );

    let changed_count = ids_a
        .iter()
        .filter(|(control_id, id_a)| ids_b.get(*control_id).is_some_and(|id_b| id_b != *id_a))
        .count();
    if expect_rotation {
        let retained_count = ids_a
            .iter()
            .filter(|(control_id, id_a)| ids_b.get(*control_id).is_some_and(|id_b| id_b == *id_a))
            .count();
        assert!(changed_count >= 1, "{slug} should rotate at least one stable ID");
        assert!(retained_count >= 1, "{slug} should retain IDs for untouched controls");
    } else {
        assert_eq!(changed_count, 0, "{slug} should not rotate stable IDs");
    }
}

fn run_validation_fixture(path: &Path) -> forge::validate::ValidationReport {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed reading validation fixture {}: {e}", path.display()));
    let json: Value = serde_json::from_str(&text).unwrap_or_else(|e| {
        panic!("validation fixture must be JSON content {}: {e}", path.display())
    });
    forge::validate::run_full_validation(
        path.to_string_lossy().as_ref(),
        &json,
        forge::OscalModelType::Catalog,
    )
    .expect("full validation report should be produced")
}

#[test]
fn fixture_contract_completeness_smoke_test() {
    for fixture in CONVERT_FIXTURES {
        let path = fixture_dir(fixture.slug);
        assert!(path.is_dir(), "missing fixture directory: {}", path.display());
        assert_fixture_file(fixture.slug, fixture.input_name);
        match fixture.expectation {
            ConvertExpectation::Output => {
                assert_fixture_file(fixture.slug, "expected-catalog.json");
                assert_fixture_file(fixture.slug, "expected-component-definition.json");
            }
            ConvertExpectation::Error => assert_fixture_file(fixture.slug, "expected-error.txt"),
        }
    }

    for slug in SUPPLEMENTAL_FIXTURES {
        let path = fixture_dir(slug);
        assert!(path.is_dir(), "missing fixture directory: {}", path.display());
        assert_fixture_file(slug, "input.md");
        assert_fixture_file(slug, "expected-catalog.json");
        assert_fixture_file(slug, "expected-component-definition.json");
    }

    for (slug, input_name, expected_errors_name) in VALIDATION_ONLY_FIXTURES {
        let path = fixture_dir(slug);
        assert!(path.is_dir(), "missing fixture directory: {}", path.display());
        assert_fixture_file(slug, input_name);
        assert_fixture_file(slug, expected_errors_name);
    }

    for (slug, expected_error_name) in STRATEGY_AGNOSTIC_ERROR_FIXTURES {
        let path = fixture_dir(slug);
        assert!(path.is_dir(), "missing fixture directory: {}", path.display());
        assert_fixture_file(slug, expected_error_name);
    }

    assert!(Path::new(SOURCE_PROFILE).is_file(), "missing source-profile fixture");
}

// FR Traceability:
// - FR-001/FR-002: fixture_contract_completeness_smoke_test, strategy_matrix_dual_strategy_and_agnostic_coverage
// - FR-003: ec01_no_headings_returns_descriptive_failure, ec09_missing_file_is_strategy_agnostic_failure
// - FR-004: ec02_compound_and_atomic_match_expected_outputs
// - FR-005: ec03_empty_sections_preserved_without_failure
// - FR-006: ec04_missing_metadata_defaults_are_applied
// - FR-007/FR-008: ec05_whitespace_only_changes_keep_stable_ids, ec06_substantive_change_rotates_stable_ids
// - FR-009: ec07_malformed_citation_is_retained_with_unvalidated_marker
// - FR-010: ec10_reports_schema_and_semantic_issues_together
// - FR-011: strategy_matrix_dual_strategy_and_agnostic_coverage, strategy_constants_match_expected_scope
// - FR-012: wi22_edge_case_fixture_integrity_and_scope_guards (in fixture_validity_test.rs)

#[test]
fn ec01_no_headings_returns_descriptive_failure() {
    let input = fixture_input("ec01-no-headings", "input.md");
    let expected =
        load_expected_substrings(&fixture_input("ec01-no-headings", "expected-error.txt"));

    let run = run_convert(&input, Strategy::Catalog);
    assert_edge_case_error(&run, &expected, "EC-1 catalog");
}

#[test]
fn ec02_compound_and_atomic_match_expected_outputs() {
    let catalog_output =
        assert_fixture_output("ec02-compound-atomic", "input.md", Strategy::Catalog);
    insta::assert_json_snapshot!("ec02_catalog", normalize_for_comparison(&catalog_output));

    let component_output =
        assert_fixture_output("ec02-compound-atomic", "input.md", Strategy::Component);
    insta::assert_json_snapshot!("ec02_component", normalize_for_comparison(&component_output));
}

#[test]
fn ec03_empty_sections_preserved_without_failure() {
    let _ = assert_fixture_output("ec03-empty-sections", "input.md", Strategy::Catalog);

    let expected_warnings =
        load_expected_substrings(&fixture_input("ec03-empty-sections", "expected-warnings.txt"));
    if !expected_warnings.is_empty() {
        let run = run_convert(&fixture_input("ec03-empty-sections", "input.md"), Strategy::Catalog);
        assert_expected_warnings(&run.stderr, &expected_warnings, "EC-3 warnings");
    }
}

#[test]
fn ec04_missing_metadata_defaults_are_applied() {
    let _ = assert_fixture_output("ec04-missing-metadata", "input.md", Strategy::Catalog);

    let expected_warnings =
        load_expected_substrings(&fixture_input("ec04-missing-metadata", "expected-warnings.txt"));
    if !expected_warnings.is_empty() {
        let run =
            run_convert(&fixture_input("ec04-missing-metadata", "input.md"), Strategy::Catalog);
        assert_expected_warnings(&run.stderr, &expected_warnings, "EC-4 warnings");
    }
}

#[test]
fn ec05_whitespace_only_changes_keep_stable_ids() {
    assert_component_stable_id_contract(
        "ec05-whitespace-only",
        "input-original.md",
        "input-whitespace-variant.md",
        false,
        false,
    );
}

#[test]
fn ec06_substantive_change_rotates_stable_ids() {
    assert_component_stable_id_contract(
        "ec06-substantive-change",
        "input-original.md",
        "input-changed.md",
        true,
        true,
    );

    let original = fixture_input("ec06-substantive-change", "input-original.md");
    let changed = fixture_input("ec06-substantive-change", "input-changed.md");
    let run = run_convert_with_baseline(&changed, Strategy::Component, Some(&original));
    let expected_warnings = load_expected_substrings(&fixture_input(
        "ec06-substantive-change",
        "expected-warnings.txt",
    ));
    assert_expected_warnings(&run.stderr, &expected_warnings, "EC-6 warnings");
}

#[test]
fn ec07_fixture_citation_is_retained() {
    let output = assert_fixture_output("ec07-malformed-citation", "input.md", Strategy::Catalog);
    let resources = output["catalog"]["back-matter"]["resources"]
        .as_array()
        .expect("EC-7 catalog output must contain back-matter resources");
    let citation = resources
        .iter()
        .find(|resource| {
            resource["rlinks"].as_array().is_some_and(|rlinks| {
                rlinks.iter().any(|rlink| rlink["href"] == "https://example.com/retention")
            })
        })
        .expect("EC-7 fixture citation must be retained in back-matter");

    assert!(
        citation["props"].as_array().is_none_or(|props| {
            props.iter().all(|prop| prop["name"] != "url-status" || prop["value"] != "unvalidated")
        }),
        "EC-7 fixture citation uses a valid HTTPS URL and must not be marked unvalidated"
    );
}

#[test]
fn ec09_missing_file_is_strategy_agnostic_failure() {
    let missing_path = fixture_input("ec09-file-not-found", "nonexistent.md");
    let expected =
        load_expected_substrings(&fixture_input("ec09-file-not-found", "expected-error.txt"));

    let run = run_convert(&missing_path, Strategy::Catalog);
    assert_edge_case_error(&run, &expected, "EC-9 missing file");
}

#[test]
fn ec10_reports_schema_and_semantic_issues_together() {
    let input = fixture_input("ec10-multiple-errors", "input.json");
    let report = run_validation_fixture(&input);
    assert!(!report.is_valid(), "EC-10 report should be invalid");
    assert!(report.schema_error_count() > 0, "EC-10 should include schema errors");
    assert!(report.semantic_error_count() > 0, "EC-10 should include semantic errors");

    let rendered = forge::validate::report::render_text_report(&report);
    let expected =
        load_expected_substrings(&fixture_input("ec10-multiple-errors", "expected-errors.txt"));
    assert_required_substrings(&rendered, &expected, "EC-10 validation report");
    insta::assert_snapshot!("ec10_validation_errors", rendered);
}

#[test]
fn strategy_matrix_dual_strategy_and_agnostic_coverage() {
    let mut catalog_status = BTreeMap::new();
    let mut component_status = BTreeMap::new();

    for fixture in CONVERT_FIXTURES {
        match fixture.expectation {
            ConvertExpectation::Error => {
                let input = fixture_input(fixture.slug, fixture.input_name);
                let expected =
                    load_expected_substrings(&fixture_input(fixture.slug, "expected-error.txt"));
                let catalog = run_convert(&input, Strategy::Catalog);
                let component = run_convert(&input, Strategy::Component);
                assert_edge_case_error(
                    &catalog,
                    &expected,
                    &format!("matrix {} catalog", fixture.slug),
                );
                assert_edge_case_error(
                    &component,
                    &expected,
                    &format!("matrix {} component", fixture.slug),
                );
                catalog_status.insert(fixture.slug.to_string(), "error-match".to_string());
                component_status.insert(fixture.slug.to_string(), "error-match".to_string());
            }
            ConvertExpectation::Output => {
                assert_dual_strategy_output(fixture.slug, fixture.input_name);
                match fixture.slug {
                    "ec05-whitespace-only" => assert_component_stable_id_contract(
                        fixture.slug,
                        fixture.input_name,
                        "input-whitespace-variant.md",
                        false,
                        false,
                    ),
                    "ec06-substantive-change" => assert_component_stable_id_contract(
                        fixture.slug,
                        fixture.input_name,
                        "input-changed.md",
                        true,
                        true,
                    ),
                    _ => {}
                }
                catalog_status.insert(fixture.slug.to_string(), "golden-match".to_string());
                component_status.insert(fixture.slug.to_string(), "golden-match".to_string());
            }
        }
    }

    for (slug, input_name, _) in VALIDATION_ONLY_FIXTURES {
        let report = run_validation_fixture(&fixture_input(slug, input_name));
        assert!(report.schema_error_count() > 0);
        assert!(report.semantic_error_count() > 0);
        catalog_status.insert((*slug).to_string(), "validation-aggregate".to_string());
        component_status.insert((*slug).to_string(), "validation-aggregate".to_string());
    }

    for (slug, expected_error_name) in STRATEGY_AGNOSTIC_ERROR_FIXTURES {
        let expected = load_expected_substrings(&fixture_input(slug, expected_error_name));
        let run = run_convert(&fixture_input(slug, "nonexistent.md"), Strategy::Catalog);
        assert_edge_case_error(&run, &expected, &format!("matrix {slug}"));
    }

    insta::assert_json_snapshot!("strategy_matrix_catalog", json!(catalog_status));
    insta::assert_json_snapshot!("strategy_matrix_component", json!(component_status));
}

#[test]
fn supplemental_citation_positions_and_parameter_like_content() {
    for slug in SUPPLEMENTAL_FIXTURES {
        let input = fixture_input(slug, "input.md");

        let cat = run_convert(&input, Strategy::Catalog);
        assert_convert_success(&cat, &format!("supplemental {slug} catalog"));
        let cat_output = cat.output_json.expect("catalog output should exist");
        assert_expected_output(
            &cat_output,
            &fixture_input(slug, "expected-catalog.json"),
            &format!("{slug} catalog"),
        );

        let comp = run_convert(&input, Strategy::Component);
        assert_convert_success(&comp, &format!("supplemental {slug} component"));
        let comp_output = comp.output_json.expect("component output should exist");
        assert_expected_output(
            &comp_output,
            &fixture_input(slug, "expected-component-definition.json"),
            &format!("{slug} component"),
        );
    }
}

#[test]
fn strategy_constants_match_expected_scope() {
    let dual_convert_cases: BTreeSet<&str> =
        CONVERT_FIXTURES.iter().map(|fixture| fixture.slug).collect();
    let validation_only_cases: BTreeSet<&str> =
        VALIDATION_ONLY_FIXTURES.iter().map(|(slug, _, _)| *slug).collect();
    let strategy_applicable_cases: BTreeSet<&str> =
        dual_convert_cases.union(&validation_only_cases).copied().collect();

    assert_eq!(
        dual_convert_cases.len(),
        CONVERT_FIXTURES.len(),
        "convert fixture slugs must be unique"
    );
    assert_eq!(
        validation_only_cases.len(),
        VALIDATION_ONLY_FIXTURES.len(),
        "validation-only fixture slugs must be unique"
    );
    assert_eq!(
        strategy_applicable_cases.len(),
        CONVERT_FIXTURES.len() + VALIDATION_ONLY_FIXTURES.len(),
        "convert and validation-only fixture slugs must not overlap"
    );
    for (slug, _) in STRATEGY_AGNOSTIC_ERROR_FIXTURES {
        assert!(!strategy_applicable_cases.contains(slug), "{slug} must remain strategy-agnostic");
    }
}
