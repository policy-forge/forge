//! Native JSON/YAML export preservation through the actual Forge CLI.
//!
//! The synthetic fixtures retain complete OSCAL trees, including fields beyond
//! Forge's generation structs. Equality compares every object member and array
//! position; no pruning or generated-model normalization is permitted. The
//! supported declarations are checked against Forge's pinned 1.2.3 validator,
//! not against historical schemas. XML presentation and losslessness are outside
//! this suite's scope. Existing CLI failures use exit 1 for export decoding or
//! unsupported models and exit 3 for schema, semantic or declaration validation.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;
use tempfile::TempDir;

const FIXTURES: [(&str, &str); 2] =
    [("lossless-catalog.json", "catalog"), ("lossless-component.json", "component-definition")];
const SUPPORTED_VERSIONS: [&str; 4] = ["1.2.0", "1.2.1", "1.2.2", "1.2.3"];

/// Locate one immutable, checked-in synthetic native fixture.
fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/export").join(name)
}

/// Read original fixture bytes without rewriting or normalizing the source.
fn fixture_bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).expect("lossless export fixture must exist")
}

/// Decode the complete original fixture as an independent equality oracle.
fn fixture_value(name: &str) -> Value {
    serde_json::from_slice(&fixture_bytes(name)).expect("native fixture must be valid JSON")
}

/// Invoke the Cargo-selected Forge binary with no stdin or external converter.
///
/// `Command::output` drains both captured streams while waiting, including when
/// a complete fixture exceeds a platform's pipe buffer. Execution belongs to the
/// integration runner; fixture preparation itself does not run this helper.
fn run_export(input: &Path, format: &str, destination: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_forge"));
    command.args(["--quiet", "export"]).arg(input).args(["--format", format]);
    if let Some(destination) = destination {
        command.arg("--output").arg(destination);
    }
    command
        .env("RUST_LOG", "error")
        .stdin(Stdio::null())
        .output()
        .expect("actual Forge export binary must execute")
}

/// Require success and compare an entire emitted JSON/YAML tree with its source.
fn assert_exported_value(output: &Output, bytes: &[u8], format: &str, expected: &Value) {
    assert!(
        output.status.success(),
        "export to {format} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = match format {
        "json" => serde_json::from_slice(bytes).expect("exported JSON must decode"),
        "yaml" => serde_yaml::from_slice(bytes).expect("exported YAML must decode as JSON data"),
        _ => panic!("this preservation suite covers only JSON and YAML"),
    };
    assert_eq!(&actual, expected, "export changed the complete native {format} tree");
}

/// Exercise JSON-to-JSON without touching the original source bytes or arrays.
fn assert_lossless_json(name: &str) {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(name);
    let destination = dir.path().join("exported.json");
    let original = fixture_bytes(name);
    std::fs::write(&input, &original).unwrap();
    let expected: Value = serde_json::from_slice(&original).unwrap();
    let output = run_export(&input, "json", Some(&destination));
    assert!(output.stdout.is_empty(), "file export must not publish content on stdout");
    let exported = std::fs::read(&destination).unwrap_or_default();
    assert_exported_value(&output, &exported, "json", &expected);
    assert_eq!(std::fs::read(input).unwrap(), original, "export rewrote source bytes");
}

/// Compare both YAML and its re-exported JSON directly to the complete original.
fn assert_lossless_yaml_round_trip(name: &str, original: &[u8]) {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(name);
    let yaml_path = dir.path().join("intermediate.yaml");
    let json_path = dir.path().join("returned.json");
    std::fs::write(&input, original).unwrap();
    let expected: Value = serde_json::from_slice(original).unwrap();
    let yaml_output = run_export(&input, "yaml", Some(&yaml_path));
    assert!(yaml_output.stdout.is_empty(), "file export leaked content to stdout");
    let yaml_bytes = std::fs::read(&yaml_path).unwrap_or_default();
    assert_exported_value(&yaml_output, &yaml_bytes, "yaml", &expected);
    let json_output = run_export(&yaml_path, "json", Some(&json_path));
    assert!(json_output.stdout.is_empty(), "file export leaked content to stdout");
    let json_bytes = std::fs::read(&json_path).unwrap_or_default();
    assert_exported_value(&json_output, &json_bytes, "json", &expected);
    assert_eq!(std::fs::read(&input).unwrap(), original, "original JSON bytes changed");
    assert_eq!(std::fs::read(&yaml_path).unwrap(), yaml_bytes, "YAML input bytes changed");
}

/// Require a contract failure before either JSON/YAML destination is published.
///
/// Each malformed input is tested against an existing binary sentinel and an
/// absent destination. The exact malformed source also remains unchanged. The
/// explicit expected exit follows the established CLI contract: decoding and
/// unsupported models use 1; schema, semantic and declaration validation use 3.
/// Nonempty stderr and zero stdout distinguish rejection from silent publication.
fn assert_rejected(extension: &str, malformed: &[u8], expected_exit: i32, case: &str) {
    assert_rejected_diagnostic(extension, malformed, expected_exit, case, None);
}

/// Check an optional failure diagnostic for every guarded destination publication.
///
/// A specific diagnostic distinguishes decoded bounds or unsupported-model refusal
/// from a decoder recursion error, schema rejection or unrecognized document root.
fn assert_rejected_diagnostic(
    extension: &str,
    malformed: &[u8],
    expected_exit: i32,
    case: &str,
    expected_diagnostic: Option<&str>,
) {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join(format!("invalid.{extension}"));
    std::fs::write(&input, malformed).unwrap();
    for format in ["json", "yaml"] {
        let existing = dir.path().join(format!("existing.{format}"));
        let missing = dir.path().join(format!("missing.{format}"));
        let sentinel = b"existing destination\0\xff\n";
        std::fs::write(&existing, sentinel).unwrap();
        for destination in [&existing, &missing] {
            let output = run_export(&input, format, Some(destination));
            assert_eq!(
                output.status.code(),
                Some(expected_exit),
                "{case} was not rejected for {format}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty(), "{case} emitted partial content");
            assert!(!output.stderr.is_empty(), "{case} omitted its failure diagnostic");
            if let Some(expected) = expected_diagnostic {
                let diagnostic = String::from_utf8_lossy(&output.stderr);
                assert!(
                    diagnostic.contains(expected),
                    "{case} did not report {expected:?}: {diagnostic}"
                );
            }
            assert_eq!(std::fs::read(&existing).unwrap(), sentinel, "{case} changed destination");
            assert!(!missing.exists(), "{case} created a new output");
            assert_eq!(std::fs::read(&input).unwrap(), malformed, "{case} changed source bytes");
        }
    }
}

/// Prove fixture validity without passing through export's potentially lossy structs.
#[test]
fn native_fixtures_validate_for_supported_declarations_against_pinned_schema() {
    for (name, root) in FIXTURES {
        for version in SUPPORTED_VERSIONS {
            let mut value = fixture_value(name);
            value[root]["metadata"]["oscal-version"] = Value::String(version.into());
            let model = forge::validate::detect_model_type(&value).unwrap();
            let report = forge::validate::run_full_validation(name, &value, model).unwrap();
            assert!(report.is_valid(), "{name} declaration {version}: {:?}", report.errors());
            assert_eq!(report.declared_oscal_version(), Some(version));
            assert_eq!(report.schema_version_used(), "1.2.3");
        }
    }
}

/// Preserve rich Catalog metadata, nested controls, parts and bound resource arrays.
#[test]
fn catalog_json_export_preserves_complete_native_value() {
    assert_lossless_json("lossless-catalog.json");
}

/// Preserve native Component statement UUIDs, capabilities and implementation arrays.
#[test]
fn component_json_export_preserves_complete_native_value() {
    assert_lossless_json("lossless-component.json");
}

/// Preserve every Catalog field through YAML and compare again to the original JSON.
#[test]
fn catalog_yaml_round_trip_preserves_complete_native_value() {
    assert_lossless_yaml_round_trip(
        "lossless-catalog.json",
        &fixture_bytes("lossless-catalog.json"),
    );
}

/// Preserve every Component field through YAML, including ordered native statements.
#[test]
fn component_yaml_round_trip_preserves_complete_native_value() {
    assert_lossless_yaml_round_trip(
        "lossless-component.json",
        &fixture_bytes("lossless-component.json"),
    );
}

/// Preserve all four supported declarations through actual CLI YAML-to-JSON exports.
#[test]
fn supported_declarations_round_trip_without_version_rewrite() {
    for (name, root) in FIXTURES {
        for version in SUPPORTED_VERSIONS {
            let mut value = fixture_value(name);
            value[root]["metadata"]["oscal-version"] = Value::String(version.into());
            let original = serde_json::to_vec_pretty(&value).unwrap();
            assert_lossless_yaml_round_trip(name, &original);
        }
    }
}

/// Require stdout JSON and YAML exports to retain the same entire original tree.
#[test]
fn stdout_exports_preserve_complete_native_values_and_source_bytes() {
    for (name, _) in FIXTURES {
        let original = fixture_bytes(name);
        let expected = fixture_value(name);
        for format in ["json", "yaml"] {
            let output = run_export(&fixture_path(name), format, None);
            assert_exported_value(&output, &output.stdout, format, &expected);
            assert_eq!(fixture_bytes(name), original, "stdout export changed source fixture");
        }
    }
}

/// Reject a nested duplicate property key instead of retaining only its final value.
#[test]
fn nested_duplicate_json_is_rejected_without_publication() {
    let original = String::from_utf8(fixture_bytes("lossless-catalog.json")).unwrap();
    assert!(original.contains("\"value\": \"false\""));
    let duplicate =
        original.replacen("\"value\": \"false\"", "\"value\": \"false\", \"value\": \"true\"", 1);
    assert_rejected("json", duplicate.as_bytes(), 1, "nested duplicate JSON");
}

/// JSON flow mappings are valid YAML: duplicate detection must also hold there.
#[test]
fn nested_duplicate_yaml_flow_mapping_is_rejected_without_publication() {
    let original = String::from_utf8(fixture_bytes("lossless-component.json")).unwrap();
    assert!(original.contains("\"value\": \"false\""));
    let duplicate =
        original.replacen("\"value\": \"false\"", "\"value\": \"false\", \"value\": \"true\"", 1);
    assert_rejected("yaml", duplicate.as_bytes(), 1, "nested duplicate YAML flow mapping");
}

/// Block-style YAML must also reject a duplicate nested mapping member.
#[test]
fn nested_duplicate_yaml_block_mapping_is_rejected_without_publication() {
    let duplicate = br"catalog:
  uuid: 00000001-abcd-4abc-8abc-000000000001
  metadata:
    title: Synthetic duplicate-key fixture
    last-modified: '2026-10-02T10:00:00Z'
    version: '1.0.0'
    oscal-version: '1.2.3'
    props:
      - name: nested-duplicate
        value: first
        value: second
";
    assert_rejected("yaml", duplicate, 1, "nested duplicate YAML block mapping");
}

/// Reject schema-invalid members before projection could silently discard them.
#[test]
fn unknown_native_fields_are_rejected_without_publication() {
    for (name, root) in FIXTURES {
        let mut value = fixture_value(name);
        value[root]["metadata"]["unknown-export-field"] = Value::String("must not vanish".into());
        let malformed = serde_json::to_vec_pretty(&value).unwrap();
        assert_rejected("json", &malformed, 3, "unknown native metadata field");
        let yaml = serde_yaml::to_string(&value).unwrap();
        assert_rejected("yaml", yaml.as_bytes(), 3, "unknown native metadata field in YAML");
    }
}

/// Reject unsupported or noncanonical declarations rather than defaulting their version.
#[test]
fn unsupported_version_declarations_are_rejected_without_publication() {
    for version in ["1.1.0", "1.2.4", "2.0.0", "1.02.3"] {
        let mut value = fixture_value("lossless-catalog.json");
        value["catalog"]["metadata"]["oscal-version"] = Value::String(version.into());
        assert_rejected("json", &serde_json::to_vec(&value).unwrap(), 3, version);
    }
}

/// Syntax-invalid JSON must leave both existing and absent destinations untouched.
#[test]
fn malformed_json_is_rejected_without_publication() {
    assert_rejected("json", br#"{"catalog":{"metadata":{},}}"#, 1, "malformed nested JSON");
}

/// Syntax-invalid YAML must fail before emitting a normalized partial artifact.
#[test]
fn malformed_yaml_is_rejected_without_publication() {
    assert_rejected("yaml", b"catalog:\n  metadata: [unterminated\n", 1, "malformed nested YAML");
}

/// Do not select one YAML document and discard another document from the input.
#[test]
fn multiple_yaml_documents_are_rejected_without_publication() {
    let original = String::from_utf8(fixture_bytes("lossless-catalog.json")).unwrap();
    let multiple = format!("---\n{original}\n---\n{original}");
    assert_rejected("yaml", multiple.as_bytes(), 1, "multiple YAML documents");
}

/// Reject nonfinite YAML values rather than silently converting them into null.
#[test]
fn nonfinite_yaml_scalars_are_rejected_without_publication() {
    let original = String::from_utf8(fixture_bytes("lossless-catalog.json")).unwrap();
    let needle = "\"value\": \"synthetic-development\"";
    assert!(original.contains(needle));
    for scalar in [".nan", ".inf", "-.inf"] {
        // The complete rich native tree remains present. A nonfinite scalar
        // replaces one property value; parser/schema rejection must precede
        // publication, rather than silently retaining a null or lost field.
        let malformed = original.replacen(needle, &format!("\"value\": {scalar}"), 1);
        assert_rejected("yaml", malformed.as_bytes(), 1, scalar);
    }
}

/// Preserve semantic validation of local links on the original, unprojected tree.
#[test]
fn orphaned_resource_reference_is_rejected_without_publication() {
    let mut value = fixture_value("lossless-catalog.json");
    value["catalog"]["metadata"]["links"][0]["href"] =
        Value::String("#ffffffff-abcd-4abc-8abc-000000000001".into());
    assert_rejected(
        "json",
        &serde_json::to_vec(&value).unwrap(),
        3,
        "orphaned metadata resource link",
    );
}

/// Native Component statements require UUIDs even if a generated struct would omit them.
#[test]
fn component_statement_without_uuid_is_rejected_without_publication() {
    let mut value = fixture_value("lossless-component.json");
    value["component-definition"]["components"][0]["control-implementations"][0]
        ["implemented-requirements"][0]["statements"][0]
        .as_object_mut()
        .unwrap()
        .remove("uuid");
    assert_rejected(
        "json",
        &serde_json::to_vec(&value).unwrap(),
        3,
        "missing native statement UUID",
    );
}

/// Refuse recognized Profile, SSP and Mapping object roots in JSON and YAML.
///
/// The exact model diagnostic proves refusal precedes schema/body validation;
/// every case retains both destination guards and the original source bytes.
#[test]
fn other_oscal_models_are_rejected_without_publication() {
    let catalog = fixture_value("lossless-catalog.json");
    for (root, model) in [
        ("profile", "Profile"),
        ("system-security-plan", "System Security Plan"),
        ("mapping-collection", "Control Mapping"),
    ] {
        let mut artifact = serde_json::json!({});
        artifact[root] = serde_json::json!({
            "uuid": "00000003-abcd-4abc-8abc-000000000001",
            "metadata": catalog["catalog"]["metadata"].clone(),
        });
        let diagnostic = format!("Export of OSCAL {model} documents is not yet supported");
        let case = format!("unsupported {model} export");
        let json = serde_json::to_vec(&artifact).unwrap();
        let yaml = serde_yaml::to_string(&artifact).unwrap();
        for (extension, bytes) in [("json", json.as_slice()), ("yaml", yaml.as_bytes())] {
            assert_rejected_diagnostic(extension, bytes, 1, &case, Some(&diagnostic));
        }
    }
}

/// Reject native recursive controls deeper than the configured decoded depth 100.
///
/// A 51-descendant control chain reaches decoded depth 106 while remaining below
/// the JSON/YAML decoders' 128-level recursion limits. JSON, block YAML and JSON
/// flow syntax interpreted as YAML must report the decoded depth bound, rather
/// than decoder recursion or schema errors, before any destination publication.
#[test]
fn decoded_depth_limit_is_rejected_without_publication() {
    let mut artifact = fixture_value("lossless-catalog.json");
    let mut control = serde_json::json!({"id": "depth-leaf", "title": "Synthetic depth leaf"});
    for level in 0..51 {
        control = serde_json::json!({
            "id": format!("depth-{level}"),
            "title": "Synthetic recursive control",
            "controls": [control],
        });
    }
    artifact["catalog"]["controls"] = serde_json::json!([control]);
    let json = serde_json::to_vec(&artifact).unwrap();
    let block_yaml = serde_yaml::to_string(&artifact).unwrap();
    for (extension, bytes, case) in [
        ("json", json.as_slice(), "decoded depth in JSON"),
        ("yaml", json.as_slice(), "decoded depth in YAML flow syntax"),
        ("yaml", block_yaml.as_bytes(), "decoded depth in YAML block syntax"),
    ] {
        assert_rejected_diagnostic(
            extension,
            bytes,
            1,
            case,
            Some("exceeds maximum JSON depth 100"),
        );
    }
}

/// Preserve the largest supported unsigned integer through actual CLI JSON/YAML.
///
/// The port datatype has no schema maximum; complete-tree equality binds the
/// exact integer value rather than using a rounded floating-point oracle.
#[test]
fn unsigned_integer_boundaries_round_trip_without_rounding() {
    for boundary in [0, 9_223_372_036_854_775_807_u64, u64::MAX] {
        let mut value = fixture_value("lossless-component.json");
        let range =
            &mut value["component-definition"]["components"][0]["protocols"][0]["port-ranges"][0];
        range["start"] = Value::from(boundary);
        range["end"] = Value::from(boundary);
        let original = serde_json::to_vec_pretty(&value).unwrap();
        assert_lossless_yaml_round_trip("boundary-component.json", &original);
    }
}

/// Oversized integer tokens and all floating representations fail before output.
///
/// Both formats exercise current native port fields and an unknown nested field;
/// numeric refusal must precede schema errors or silent generation projection.
#[test]
fn unsupported_numeric_representations_are_rejected_without_publication() {
    let value = fixture_value("lossless-component.json");
    let original = serde_json::to_string(&value).unwrap();
    let needle = "\"start\":443";
    assert!(original.contains(needle));
    for number in [
        "18446744073709551616",
        "-9223372036854775809",
        "9007199254740993.0",
        "443.0",
        "443e0",
        "1.0",
        "1e0",
        "0.1",
    ] {
        let malformed = original.replacen(needle, &format!("\"start\":{number}"), 1);
        for extension in ["json", "yaml"] {
            assert_rejected(extension, malformed.as_bytes(), 1, number);
        }
    }
    for number in ["1.0", "1e0", "18446744073709551616"] {
        let malformed = format!("{{\"catalog\":{{\"unknown-numeric\":[{{\"value\":{number}}}]}}}}");
        for extension in ["json", "yaml"] {
            let diagnostic = (number != "18446744073709551616")
                .then_some("only exact i64/u64 integer numeric values");
            assert_rejected_diagnostic(
                extension,
                malformed.as_bytes(),
                1,
                "unknown nested numeric value",
                diagnostic,
            );
        }
    }
}
