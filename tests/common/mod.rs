#![allow(dead_code)]
pub mod fixture_generator;

use std::path::{Path, PathBuf};

use forge::model::{DocumentMetadata, PolicyDocument, PolicyRequirement, PolicySection};

/// Normalize a JSON value for stable snapshot comparison.
///
/// Replaces dynamic fields with stable placeholder values so that
/// identical inputs always produce the same snapshot, regardless of
/// when or where the test is run:
///
/// - UUID-format strings → `"00000000-0000-0000-0000-000000000000"`
/// - `last-modified` string values → `"2026-01-01T00:00:00Z"`
/// - Repo-local and Windows absolute path strings → `"NORMALIZED_PATH"`
///
/// Normalization is applied recursively to all JSON values.
pub fn normalize_for_snapshot(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;

    // UUID pattern: 8-4-4-4-12 hex digits
    static UUID_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
        )
        .expect("UUID regex is valid")
    });

    match value {
        Value::Object(map) => {
            let normalized: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| {
                    if k == "last-modified" {
                        if v.is_string() {
                            (k.clone(), Value::String("2026-01-01T00:00:00Z".to_string()))
                        } else {
                            (k.clone(), normalize_for_snapshot(v))
                        }
                    } else {
                        (k.clone(), normalize_for_snapshot(v))
                    }
                })
                .collect();
            Value::Object(normalized)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_for_snapshot).collect()),
        Value::String(s) => {
            if UUID_RE.is_match(s) {
                Value::String("00000000-0000-0000-0000-000000000000".to_string())
            } else if is_repo_local_path(s) || is_windows_path(s) {
                Value::String("NORMALIZED_PATH".to_string())
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

/// Returns true when a path belongs to this checkout rather than OSCAL content.
fn is_repo_local_path(s: &str) -> bool {
    Path::new(s).starts_with(Path::new(env!("CARGO_MANIFEST_DIR")))
}

/// Returns true if the string looks like a Windows absolute path.
fn is_windows_path(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(
        (chars.next(), chars.next(), chars.next()),
        (Some(c), Some(':'), Some('\\' | '/')) if c.is_ascii_alphabetic()
    ) || s.starts_with(r"\\?\")
        || s.starts_with(r"\\")
}

/// Maximum file size for ingest in tests (10 MB).
pub const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Assert that a required fixture exists, failing the test otherwise.
///
/// The synthetic fixture generator is deterministic (no randomness or time
/// dependence), so a missing fixture is always a genuine defect — never a
/// reason to skip quietly (F0832).
#[track_caller]
pub fn require_fixture(path: &Path) {
    assert!(
        path.exists(),
        "required fixture missing (run the fixture generator?): {}",
        path.display()
    );
}

pub fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
    PolicyRequirement {
        stable_id: None,
        text: text.to_string(),
        source_line,
        nesting_depth: 0,
        atom_index: 0,
        parent_text: None,
        citations: vec![],
        modality: None,
        parameters: vec![],
        parameters_extracted: false,
    }
}

pub fn make_section(title: &str, requirements: Vec<PolicyRequirement>) -> PolicySection {
    PolicySection {
        title: title.to_string(),
        heading_level: 1,
        source_line: 1,
        body_text: None,
        children: vec![],
        requirements,
    }
}

pub fn make_doc(title: &str, sections: Vec<PolicySection>) -> PolicyDocument {
    PolicyDocument {
        id: "test".to_string(),
        metadata: DocumentMetadata {
            title: title.to_string(),
            version: "0.0.0".to_string(),
            author: None,
            date: None,
            source_path: PathBuf::from("test.md"),
            content_hash: None,
        },
        sections,
    }
}

#[cfg(test)]
mod normalization_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_slash_prefixed_oscal_hrefs() {
        let value = normalize_for_snapshot(&json!({"href": "/oscal/cat/1.1.3"}));
        assert_eq!(value["href"], "/oscal/cat/1.1.3");
    }

    #[test]
    fn normalizes_checkout_and_windows_absolute_paths() {
        let checkout = format!("{}/tests/fixture.md", env!("CARGO_MANIFEST_DIR"));
        let value = normalize_for_snapshot(&json!({
            "checkout": checkout,
            "unc": r"\\server\share\fixture.md",
            "verbatim": r"\\?\C:\fixture.md",
        }));
        assert_eq!(value["checkout"], "NORMALIZED_PATH");
        assert_eq!(value["unc"], "NORMALIZED_PATH");
        assert_eq!(value["verbatim"], "NORMALIZED_PATH");
    }
}
