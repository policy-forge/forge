//! Security verification tests for YAML output (WI-27, US4, T016-T019).
//!
//! Verifies YAML output is safe: no type injection, proper character handling,
//! boolean-safe strings, and adversarial input resilience (SEC-1 through SEC-4).

use forge::export::{deserialize_from_yaml, serialize_to_yaml};

// ---------------------------------------------------------------------------
// T016: SEC-1 — No YAML type tags (!! tags)
// ---------------------------------------------------------------------------

#[test]
fn catalog_yaml_has_no_type_tags() {
    let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
    assert!(fixture.exists(), "Test fixture missing: {}", fixture.display());
    let dir = tempfile::TempDir::new().unwrap();
    let yaml_path = dir.path().join("catalog.yaml");
    forge::pipeline::run_catalog_pipeline(
        fixture,
        Some(&yaml_path),
        10 * 1024 * 1024,
        &forge::cli::OutputFormat::Yaml,
        None,
    )
    .expect("Catalog YAML pipeline should succeed");
    let yaml_str = std::fs::read_to_string(&yaml_path).unwrap();
    assert!(
        !yaml_str.contains("!!"),
        "SEC-1: Catalog YAML must not contain type tags (!!). Found in:\n{yaml_str}"
    );
}

#[test]
fn component_yaml_has_no_type_tags() {
    let fixture = std::path::Path::new("tests/fixtures/full_policy.md");
    assert!(fixture.exists(), "Test fixture missing: {}", fixture.display());
    let dir = tempfile::TempDir::new().unwrap();
    let yaml_path = dir.path().join("component.yaml");
    forge::pipeline::run_component_pipeline(
        fixture,
        Some(&yaml_path),
        10 * 1024 * 1024,
        Some("./baselines/nist-800-53.json"),
        &forge::cli::OutputFormat::Yaml,
        None,
    )
    .expect("Component YAML pipeline should succeed");
    let yaml_str = std::fs::read_to_string(&yaml_path).unwrap();
    assert!(
        !yaml_str.contains("!!"),
        "SEC-1: Component YAML must not contain type tags (!!). Found in:\n{yaml_str}"
    );
}

// ---------------------------------------------------------------------------
// T017: SEC-2 — YAML-special characters handled safely
// ---------------------------------------------------------------------------

#[test]
fn yaml_special_characters_produce_valid_parseable_yaml() {
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "Policy: Risk [Assessment] {Framework}",
                "version": "1.0 # not a comment",
                "oscal-version": "1.2.0",
                "last-modified": "2025-01-01T00:00:00Z"
            },
            "groups": [
                {
                    "id": "g1",
                    "title": "Section --- with YAML document markers ...",
                    "controls": [
                        {
                            "id": "c1",
                            "title": "Control with : colon and # hash",
                            "parts": [{
                                "id": "c1-stmt",
                                "name": "statement",
                                "prose": "Values [in] {brackets} and : colons # hashes"
                            }]
                        }
                    ]
                }
            ]
        }
    });

    let yaml = serialize_to_yaml(&model).expect("Serialization should handle special chars");
    let parsed: serde_json::Value =
        deserialize_from_yaml(&yaml).expect("YAML with special chars should parse back");

    assert_eq!(
        model, parsed,
        "Round-trip through YAML with special characters should preserve all values"
    );
}

// ---------------------------------------------------------------------------
// T018: SEC-3 — Boolean-like words remain strings
// ---------------------------------------------------------------------------

#[test]
fn boolean_like_words_remain_strings_in_yaml() {
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "yes",
                "version": "no",
                "oscal-version": "true",
                "last-modified": "false"
            },
            "groups": [
                {
                    "id": "on",
                    "title": "off",
                    "controls": [
                        {
                            "id": "YES",
                            "title": "NO",
                            "parts": [{
                                "id": "TRUE-stmt",
                                "name": "statement",
                                "prose": "FALSE"
                            }]
                        }
                    ]
                }
            ]
        }
    });

    let yaml = serialize_to_yaml(&model).expect("Serialization should succeed");
    let parsed: serde_json::Value =
        deserialize_from_yaml(&yaml).expect("YAML should parse back successfully");

    // Verify boolean-like words are still strings, not coerced to booleans
    assert_eq!(parsed["catalog"]["metadata"]["title"], "yes", "\"yes\" must stay a string");
    assert_eq!(parsed["catalog"]["metadata"]["version"], "no", "\"no\" must stay a string");
    assert_eq!(
        parsed["catalog"]["metadata"]["oscal-version"], "true",
        "\"true\" must stay a string"
    );
    assert_eq!(
        parsed["catalog"]["metadata"]["last-modified"], "false",
        "\"false\" must stay a string"
    );
    assert_eq!(parsed["catalog"]["groups"][0]["id"], "on", "\"on\" must stay a string");
    assert_eq!(parsed["catalog"]["groups"][0]["title"], "off", "\"off\" must stay a string");
    assert_eq!(
        parsed["catalog"]["groups"][0]["controls"][0]["id"], "YES",
        "\"YES\" must stay a string"
    );
    assert_eq!(
        parsed["catalog"]["groups"][0]["controls"][0]["title"], "NO",
        "\"NO\" must stay a string"
    );
}

// ---------------------------------------------------------------------------
// T019: SEC-4 — Adversarial input: embedded directives, multi-line, unicode
// ---------------------------------------------------------------------------

#[test]
fn adversarial_input_parsed_safely() {
    let model = serde_json::json!({
        "catalog": {
            "uuid": "550e8400-e29b-41d4-a716-446655440000",
            "metadata": {
                "title": "Policy with\nnewlines\nand\ttabs",
                "version": "1.0",
                "oscal-version": "1.2.0",
                "last-modified": "2025-01-01T00:00:00Z"
            },
            "groups": [
                {
                    "id": "g1",
                    "title": "%TAG !yaml! tag:yaml.org,2002:",
                    "controls": [
                        {
                            "id": "c1",
                            "title": "!!python/object:__main__.Evil",
                            "parts": [{
                                "id": "c1-stmt",
                                "name": "statement",
                                "prose": "---\n!!str exploit\n..."
                            }]
                        },
                        {
                            "id": "c2",
                            "title": "\u{00e9}\u{00e8}\u{00ea} \u{2603} \u{1f512}",
                            "parts": [{
                                "id": "c2-stmt",
                                "name": "statement",
                                "prose": "Zero-width: \u{200b}\u{200c}\u{200d} and RTL: \u{202e}override"
                            }]
                        }
                    ]
                }
            ]
        }
    });

    let json_str = serde_json::to_string_pretty(&model).unwrap();
    let yaml_str = serialize_to_yaml(&model).expect("Adversarial model should serialize to YAML");

    let json_parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let yaml_parsed: serde_json::Value =
        deserialize_from_yaml(&yaml_str).expect("Adversarial YAML should parse back");

    assert_eq!(
        json_parsed, yaml_parsed,
        "Adversarial input should produce equivalent JSON and YAML Values"
    );
}

#[test]
fn adversarial_yaml_directives_do_not_leak_into_output() {
    let model = serde_json::json!({
        "title": "%YAML 1.2",
        "directive": "%TAG !custom! tag:example.com:",
        "anchor": "&anchor_ref value",
        "alias": "*anchor_ref"
    });

    let yaml = serialize_to_yaml(&model).expect("Should serialize directive-like strings");
    let parsed: serde_json::Value = deserialize_from_yaml(&yaml).expect("Should parse back safely");

    // Values must survive as literal strings, not interpreted as YAML directives
    assert_eq!(parsed["title"], "%YAML 1.2");
    assert_eq!(parsed["directive"], "%TAG !custom! tag:example.com:");
    assert_eq!(parsed["anchor"], "&anchor_ref value");
    assert_eq!(parsed["alias"], "*anchor_ref");
}
