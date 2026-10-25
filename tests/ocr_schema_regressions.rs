//! Regression checks for verified OCR schema-boundary defects.
use serde_json::{Value, json};

fn validate_fragment(schema: &str, pointer: &str, value: &Value) -> bool {
    let schema: Value = serde_json::from_str(schema).unwrap();
    let fragment = schema.pointer(pointer).unwrap();
    jsonschema::validator_for(fragment).unwrap().is_valid(value)
}

#[test]
fn reuse_and_evidence_digests_reject_trailing_line_terminators() {
    let schemas = [
        (include_str!("../schemas/forge.authoring-reuse-1.schema.json"), "/$defs/sha256"),
        (
            include_str!("../schemas/forge.poam-evidence-links-1.schema.json"),
            "/properties/linkage/properties/expected_sha256",
        ),
    ];
    for (schema, pointer) in schemas {
        let digest = "a".repeat(64);
        assert!(validate_fragment(schema, pointer, &json!(digest)));
        for suffix in ["\n", "\r", "\u{2028}", "\u{2029}"] {
            assert!(!validate_fragment(schema, pointer, &json!(format!("{digest}{suffix}"))));
        }
    }
}

#[test]
fn reuse_paths_reject_trailing_controls_and_c1_controls() {
    let schema = include_str!("../schemas/forge.authoring-reuse-1.schema.json");
    assert!(validate_fragment(schema, "/$defs/path", &json!("policies/source.md")));
    for path in ["source.md\n", "source.md\r", "source\u{85}.md", "source\u{9f}.md"] {
        assert!(!validate_fragment(schema, "/$defs/path", &json!(path)));
    }
}
