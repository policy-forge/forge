//! Golden-file regression test for SSP template output.
//!
//! Verifies that `build_ssp_skeleton` produces valid OSCAL SSP JSON
//! with control-implementation populated from catalog controls and
//! inventory items populated from component definitions.
//!
//! UUIDs are normalized to numbered placeholders (`<UUID:N>`) for
//! stable comparison across runs.

use std::path::Path;
use std::sync::LazyLock;

use forge::oscal::catalog::OscalMetadata;
use forge::oscal::{OscalCatalog, OscalControl, OscalGroup, SspComponentInput, build_ssp_skeleton};
use regex::Regex;
use serde_json::Value;

/// Path to the SSP golden fixture used by this regression test.
///
/// By default, the test compares generated output against this file.
/// To intentionally refresh the fixture after expected template changes, run:
///
/// `UPDATE_GOLDEN_FILES=1 cargo test --test ssp_template_test -- --nocapture`
///
/// Review and commit the updated fixture after regeneration.
const FIXTURE_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ssp_template_golden.json");

/// Pre-compiled UUID regex (RFC 4122 format).
static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
        .expect("UUID regex is valid")
});

// ─── Normalization ───────────────────────────────────────────────────────

/// Fixed reference timestamp for normalization.
const NORMALIZED_TIMESTAMP: &str = "2026-01-01T00:00:00Z";

/// Recursively normalize all UUID-format strings and last-modified timestamps
/// in a JSON value for stable comparison.
fn normalize_for_golden(value: &mut Value, uuid_counter: &mut usize) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if key == "last-modified" {
                    *v = Value::String(NORMALIZED_TIMESTAMP.to_string());
                } else {
                    normalize_for_golden(v, uuid_counter);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                normalize_for_golden(item, uuid_counter);
            }
        }
        Value::String(s) if UUID_RE.is_match(s) => {
            *value = Value::String(format!("<UUID:{uuid_counter}>"));
            *uuid_counter += 1;
        }
        _ => {}
    }
}

// ─── Fixtures ────────────────────────────────────────────────────────────

/// Build a minimal catalog with 2 controls: 1 top-level + 1 in a group.
fn build_minimal_catalog() -> OscalCatalog {
    let metadata = OscalMetadata {
        title: "Test Security Policy".to_string(),
        last_modified: "2026-01-01T00:00:00Z".to_string(),
        version: "1.0.0".to_string(),
        oscal_version: "1.2.0".to_string(),
    };

    OscalCatalog {
        uuid: uuid::Uuid::new_v4().to_string(),
        metadata,
        controls: vec![OscalControl {
            id: "POL-AC-001".to_string(),
            uuid: String::new(),
            title: "Access Control Policy".to_string(),
            params: vec![],
            props: vec![],
            links: vec![],
            parts: vec![],
        }],
        groups: vec![OscalGroup {
            id: "data-protection".to_string(),
            title: "Data Protection".to_string(),
            props: vec![],
            links: vec![],
            controls: vec![OscalControl {
                id: "POL-DP-001".to_string(),
                uuid: String::new(),
                title: "Data Encryption".to_string(),
                params: vec![],
                props: vec![],
                links: vec![],
                parts: vec![],
            }],
            groups: vec![],
        }],
        back_matter: None,
    }
}

/// Build minimal component definitions: 1 software + 1 service.
fn build_minimal_components() -> Vec<SspComponentInput> {
    vec![
        SspComponentInput {
            title: "Web Application Firewall".to_string(),
            description: "Filters incoming HTTP traffic".to_string(),
            component_type: "software".to_string(),
        },
        SspComponentInput {
            title: "Database Server".to_string(),
            description: "Primary PostgreSQL instance".to_string(),
            component_type: "software".to_string(),
        },
    ]
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[test]
fn ssp_template_golden() {
    let catalog = build_minimal_catalog();
    let components = build_minimal_components();

    let envelope = build_ssp_skeleton(
        "Test Security Policy",
        "1.0.0",
        &catalog,
        &components,
        "./test-profile.json",
    )
    .expect("SSP skeleton build failed");

    let json_str = serde_json::to_string_pretty(&envelope).expect("SSP serialization failed");

    let mut parsed: Value = serde_json::from_str(&json_str).expect("SSP JSON parse failed");

    // Normalize UUIDs to numbered placeholders for stable comparison
    let mut counter = 0;
    normalize_for_golden(&mut parsed, &mut counter);

    let normalized =
        serde_json::to_string_pretty(&parsed).expect("Normalized JSON serialization failed");

    // Load or write golden file
    let fixture_path = Path::new(FIXTURE_PATH);
    // Set UPDATE_GOLDEN_FILES=1 to regenerate this golden fixture intentionally.
    if std::env::var("UPDATE_GOLDEN_FILES").is_ok() {
        std::fs::create_dir_all(fixture_path.parent().unwrap())
            .expect("Failed to create fixtures dir");
        std::fs::write(fixture_path, &normalized).expect("Failed to write golden file");
        eprintln!("Written golden file: {}", fixture_path.display());
        return;
    }

    let expected = std::fs::read_to_string(fixture_path)
        .expect("Golden file not found — run with UPDATE_GOLDEN_FILES=1 to generate");

    // Re-parse expected for consistent whitespace
    let expected_parsed: Value =
        serde_json::from_str(&expected).expect("Golden file JSON parse failed");
    let expected_normalized = serde_json::to_string_pretty(&expected_parsed)
        .expect("Golden file re-serialization failed");

    if normalized != expected_normalized {
        // Print diff for debugging
        let norm_lines: Vec<&str> = normalized.lines().collect();
        let exp_lines: Vec<&str> = expected_normalized.lines().collect();
        let max_len = norm_lines.len().max(exp_lines.len());
        let mut diffs = 0_usize;
        for i in 0..max_len {
            let a = norm_lines.get(i).copied().unwrap_or("<missing>");
            let b = exp_lines.get(i).copied().unwrap_or("<missing>");
            if a != b {
                eprintln!("Diff at line {}: expected '{}' got '{}'", i + 1, b, a);
                diffs += 1;
                if diffs > 20 {
                    eprintln!("... ({} more diffs)", max_len - i - 1);
                    break;
                }
            }
        }
        panic!("SSP template output differs from golden file ({diffs} diff lines)");
    }
}

/// Verify that the SSP output contains required structural elements.
#[test]
fn ssp_template_has_required_sections() {
    let catalog = build_minimal_catalog();
    let components = build_minimal_components();

    let envelope = build_ssp_skeleton(
        "Test Security Policy",
        "1.0.0",
        &catalog,
        &components,
        "./test-profile.json",
    )
    .expect("SSP skeleton build failed");

    let ssp = &envelope.system_security_plan;

    // Verifies root key presence indirectly (envelope serialization)
    assert!(!ssp.uuid.is_empty(), "SSP must have a UUID");
    assert!(
        ssp.metadata.title.contains("Test Security Policy"),
        "Metadata title should reference policy"
    );
    assert_eq!(ssp.metadata.oscal_version, forge::oscal::OSCAL_VERSION);

    // System-implementation has components and users
    assert!(!ssp.system_implementation.components.is_empty(), "Must have inventory components");
    assert!(!ssp.system_implementation.users.is_empty(), "Must have placeholder users");

    // Control-implementation is populated
    let ci = ssp.control_implementation.as_ref().expect("Must have control-implementation");
    assert_eq!(
        ci.implemented_requirements.len(),
        2,
        "Must have 2 implemented requirements (one per control)"
    );

    // Each implemented requirement has by-component entries
    for req in &ci.implemented_requirements {
        assert!(
            !req.by_components.is_empty(),
            "Control {} must have by-component entries",
            req.control_id
        );
        for bc in &req.by_components {
            assert_eq!(
                bc.implementation_status.state, "planned",
                "Implementation status should be 'planned'"
            );
            assert!(
                bc.description.contains("TODO"),
                "By-component description must have TODO marker"
            );
        }
    }

    // System-id has TODO markers
    let sys_id = ssp.system_id.as_ref().expect("Must have system-id");
    assert!(sys_id.id.contains("TODO"), "System ID must have TODO marker");
}

/// Verify that the SSP references the correct control IDs from the catalog.
#[test]
fn ssp_control_ids_match_catalog() {
    let catalog = build_minimal_catalog();
    let components = build_minimal_components();

    let envelope = build_ssp_skeleton(
        "Test Security Policy",
        "1.0.0",
        &catalog,
        &components,
        "./test-profile.json",
    )
    .expect("SSP skeleton build failed");

    let ci = envelope
        .system_security_plan
        .control_implementation
        .as_ref()
        .expect("Must have control-implementation");

    let control_ids: Vec<&str> =
        ci.implemented_requirements.iter().map(|r| r.control_id.as_str()).collect();

    assert!(control_ids.contains(&"POL-AC-001"), "Must include top-level control");
    assert!(control_ids.contains(&"POL-DP-001"), "Must include group control");
}
