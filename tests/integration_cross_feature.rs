//! Integration tests: Cross-feature verification of WI-33/WI-34 enrichment (WI-35, US3–US4, M-5, AC-6–AC-8).
//!
//! Verifies that normative/advisory `prop` annotations (WI-33) and `param` elements (WI-34)
//! produced during Phase 2 enrichment passes are:
//!   1. Present in catalog JSON output (T024, T027)
//!   2. Preserved after JSON→XML→JSON round-trip (T025, T028)
//!   3. Preserved after JSON→YAML→JSON round-trip (T026, T029)
//!   4. Correctly assigned per atomized clause when a compound sentence is split (T045)
//!
//! `MIXED_POLICY` fixture design:
//! - "must"  → modality:normative  (WI-33 normative pattern)
//! - "should" → modality:advisory  (WI-33 advisory pattern)
//! - "within 90 days" → time-window param (WI-34 pattern)
//! - "must enforce MFA and should notify" → atomizer split at "and should" (WI-33 split pattern)
//!
//! Uses the CLI subprocess pattern (`env!("CARGO_BIN_EXE_forge")`) for end-to-end coverage.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// ── MIXED_POLICY fixture ──────────────────────────────────────────────────────
//
// 4 bullets, bullet 4 is a compound "must … and should …" sentence.
// Expected controls after atomizer:
//   Bullet 1: 1 control — normative
//   Bullet 2: 1 control — normative + time-window param ("within 90 days")
//   Bullet 3: 1 control — advisory
//   Bullet 4: 2 controls — normative ("must enforce MFA") + advisory ("should notify …")
// Total: 5 controls from 4 input bullets.

const MIXED_POLICY: &str = r#"---
title: "Mixed Modality Policy"
version: "1.0.0"
author: "Security Team"
date: "2026-02-19"
---

# Security Policy

- Systems must enforce multi-factor authentication for all privileged accounts.
- Passwords must be changed within 90 days of expiration notification.
- Administrators should review access logs weekly to detect anomalies.
- Systems must enforce MFA and should notify administrators of policy violations.
"#;

// ── Test helpers ──────────────────────────────────────────────────────────────

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_forge(args: &[&str]) -> std::process::Output {
    let mut child = forge_bin()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute forge");
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if child.try_wait().expect("failed to poll forge").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("failed to kill timed-out forge");
            let output =
                child.wait_with_output().expect("failed to collect timed-out forge output");
            panic!(
                "forge {:?} timed out after 120s\nstdout: {}\nstderr: {}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().expect("failed to collect forge output");
    assert!(
        output.status.success(),
        "forge {:?} failed (exit {})\nstdout: {}\nstderr: {}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn read_json(path: &std::path::Path) -> Value {
    let content = fs::read_to_string(path).expect("failed to read JSON file");
    serde_json::from_str(&content).expect("failed to parse JSON")
}

/// Write `MIXED_POLICY` to a temp file and return its path.
fn mixed_policy_file(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("mixed_policy.md");
    fs::write(&path, MIXED_POLICY).expect("failed to write MIXED_POLICY fixture");
    path
}

/// Convert `MIXED_POLICY` to a catalog JSON and return the path.
fn catalog_from_mixed_policy(dir: &TempDir) -> PathBuf {
    let policy_path = mixed_policy_file(dir);
    let catalog_path = dir.path().join("catalog.json");
    run_forge(&[
        "convert",
        policy_path.to_str().unwrap(),
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        catalog_path.to_str().unwrap(),
    ]);
    catalog_path
}

/// Collect `prop[name=modality]` values keyed by their owning control ID.
fn collect_modality_props(catalog: &Value) -> BTreeMap<String, Vec<(String, String)>> {
    let mut result = BTreeMap::new();
    if let Some(groups) = catalog["catalog"]["groups"].as_array() {
        for group in groups {
            if let Some(controls) = group["controls"].as_array() {
                collect_modality_from_controls(controls, &mut result);
            }
        }
    }
    result
}

fn collect_modality_from_controls(
    controls: &[Value],
    out: &mut BTreeMap<String, Vec<(String, String)>>,
) {
    for control in controls {
        let control_id =
            control["id"].as_str().expect("every compared control must have an id").to_string();
        let props = out.entry(control_id).or_default();
        if let Some(control_props) = control["props"].as_array() {
            for prop in control_props {
                if prop["name"].as_str() == Some("modality") {
                    let value = prop["value"]
                        .as_str()
                        .expect("modality prop must have a string value")
                        .to_string();
                    props.push(("modality".to_string(), value));
                }
            }
        }
        props.sort();
        if let Some(sub) = control["controls"].as_array() {
            collect_modality_from_controls(sub, out);
        }
    }
}

/// Collect parameters keyed by their owning control ID.
fn collect_params(catalog: &Value) -> BTreeMap<String, Vec<(String, Vec<String>)>> {
    let mut result = BTreeMap::new();
    if let Some(groups) = catalog["catalog"]["groups"].as_array() {
        for group in groups {
            if let Some(controls) = group["controls"].as_array() {
                collect_params_from_controls(controls, &mut result);
            }
        }
    }
    result
}

fn collect_params_from_controls(
    controls: &[Value],
    out: &mut BTreeMap<String, Vec<(String, Vec<String>)>>,
) {
    for control in controls {
        let control_id =
            control["id"].as_str().expect("every compared control must have an id").to_string();
        let params_out = out.entry(control_id).or_default();
        if let Some(params) = control["params"].as_array() {
            for param in params {
                let param_id = param["id"]
                    .as_str()
                    .expect("every compared parameter must have an id")
                    .to_string();
                let mut values: Vec<String> = param["values"]
                    .as_array()
                    .expect("every compared parameter must have values")
                    .iter()
                    .map(|value| {
                        value.as_str().expect("parameter values must be strings").to_string()
                    })
                    .collect();
                values.sort();
                params_out.push((param_id, values));
            }
        }
        params_out.sort();
        if let Some(sub) = control["controls"].as_array() {
            collect_params_from_controls(sub, out);
        }
    }
}

/// Count all controls across all groups recursively.
fn count_controls(catalog: &Value) -> usize {
    fn count(controls: &[Value]) -> usize {
        controls
            .iter()
            .map(|control| 1 + control["controls"].as_array().map_or(0, |children| count(children)))
            .sum()
    }
    catalog["catalog"]["groups"].as_array().map_or(0, |groups| {
        groups
            .iter()
            .map(|group| group["controls"].as_array().map_or(0, |controls| count(controls)))
            .sum()
    })
}

fn assert_catalog_round_trip_preserves<T>(
    intermediate_format: &str,
    value_name: &str,
    collect: impl Fn(&Value) -> T,
    is_present: impl Fn(&T) -> bool,
) where
    T: std::fmt::Debug + PartialEq,
{
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_mixed_policy(&dir);
    let original = collect(&read_json(&catalog_path));
    assert!(
        is_present(&original),
        "original catalog must have at least one {value_name} for this test to be meaningful"
    );

    let intermediate_path = dir.path().join(format!("catalog.{intermediate_format}"));
    run_forge(&[
        "export",
        catalog_path.to_str().unwrap(),
        "--format",
        intermediate_format,
        "--output",
        intermediate_path.to_str().unwrap(),
    ]);
    let roundtripped_path = dir.path().join("catalog_rt.json");
    run_forge(&[
        "export",
        intermediate_path.to_str().unwrap(),
        "--format",
        "json",
        "--output",
        roundtripped_path.to_str().unwrap(),
    ]);

    let roundtripped = collect(&read_json(&roundtripped_path));
    assert_eq!(
        original, roundtripped,
        "{value_name} must be identical after JSON→{intermediate_format}→JSON\n\
         original:     {original:?}\n\
         round-tripped: {roundtripped:?}"
    );
}

// ── M-5 / AC-6: normative and advisory props in JSON ────────────────────────

#[test]
fn normative_props_present_in_json() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_mixed_policy(&dir);
    let catalog = read_json(&catalog_path);

    let modalities = collect_modality_props(&catalog);

    assert!(
        modalities.values().flatten().any(|(_, value)| value == "normative"),
        "expected at least one control with modality=normative, got: {modalities:?}"
    );
    assert!(
        modalities.values().flatten().any(|(_, value)| value == "advisory"),
        "expected at least one control with modality=advisory, got: {modalities:?}"
    );
}

// ── M-5 / AC-8: normative/advisory props survive JSON→XML→JSON round-trip ───

#[test]
fn normative_props_survive_xml_round_trip() {
    assert_catalog_round_trip_preserves("xml", "modality prop", collect_modality_props, |values| {
        !values.is_empty()
    });
}

// ── M-5 / AC-8: normative/advisory props survive JSON→YAML→JSON round-trip ──

#[test]
fn normative_props_survive_yaml_round_trip() {
    assert_catalog_round_trip_preserves(
        "yaml",
        "modality prop",
        collect_modality_props,
        |values| !values.is_empty(),
    );
}

// ── EC-4: atomized compound sentence — normative + advisory, each gets own prop

#[test]
fn atomized_normative_advisory_each_gets_correct_prop() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_mixed_policy(&dir);
    let catalog = read_json(&catalog_path);

    // The MIXED_POLICY fixture has 4 input bullets; bullet 4 is a compound sentence
    // "Systems must enforce MFA and should notify administrators of policy violations."
    // The atomizer splits on "and should" → 2 controls from 1 bullet.
    // Total expected controls: 4 bullets + 1 extra from split = 5.
    let total_controls = count_controls(&catalog);
    assert_eq!(
        total_controls, 5,
        "expected exactly 5 controls (atomizer should split compound bullet 4 into 2); got: {total_controls}"
    );

    // EACH atomized half of the compound bullet must carry its own correct
    // modality prop — global presence alone would pass even if both halves
    // were misattributed (F0844).
    let mfa = find_control_by_text(&catalog, "must enforce MFA")
        .expect("atomized 'must enforce MFA' control should exist");
    let notify = find_control_by_text(&catalog, "should notify administrators")
        .expect("atomized 'should notify administrators' control should exist");
    assert_eq!(
        control_modality(mfa),
        Some("normative"),
        "'must enforce MFA' half must be normative; control: {mfa}"
    );
    assert_eq!(
        control_modality(notify),
        Some("advisory"),
        "'should notify administrators' half must be advisory; control: {notify}"
    );

    // Secondary guard: at least one of each across the catalog.
    let modalities = collect_modality_props(&catalog);
    assert!(
        modalities.values().flatten().any(|(_, value)| value == "normative"),
        "expected at least one normative control from compound sentence; modalities: {modalities:?}"
    );
    assert!(
        modalities.values().flatten().any(|(_, value)| value == "advisory"),
        "expected at least one advisory control from compound sentence; modalities: {modalities:?}"
    );
}

/// Find the first catalog control whose title or part prose contains `needle`.
fn find_control_by_text<'a>(catalog: &'a Value, needle: &str) -> Option<&'a Value> {
    fn scan_controls<'a>(controls: &'a [Value], needle: &str) -> Option<&'a Value> {
        for control in controls {
            let title = control["title"].as_str().unwrap_or_default();
            // Search only statement prose: guidance parts copy the entire
            // source subsection, so they would match every needle.
            let prose = control["parts"]
                .as_array()
                .map(|parts| {
                    parts
                        .iter()
                        .filter(|p| p["name"].as_str() == Some("statement"))
                        .filter_map(|p| p["prose"].as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .unwrap_or_default();
            if title.contains(needle) || prose.contains(needle) {
                return Some(control);
            }
            if let Some(nested) = control["controls"].as_array()
                && let Some(found) = scan_controls(nested, needle)
            {
                return Some(found);
            }
        }
        None
    }
    catalog["catalog"]["groups"]
        .as_array()?
        .iter()
        .filter_map(|g| g["controls"].as_array())
        .find_map(|controls| scan_controls(controls, needle))
}

/// Read a control's `prop[name=modality].value`, if present.
fn control_modality(control: &Value) -> Option<&str> {
    control["props"].as_array()?.iter().find_map(|prop| {
        (prop["name"].as_str() == Some("modality")).then(|| prop["value"].as_str())?
    })
}

// ── M-5 / AC-7: param elements present in JSON ──────────────────────────────

#[test]
fn param_elements_present_in_json() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_mixed_policy(&dir);
    let catalog = read_json(&catalog_path);

    let params = collect_params(&catalog);
    assert!(
        !params.is_empty(),
        "expected at least one control with params (from 'within 90 days' in fixture), got 0"
    );

    // At least one param must have both id and values
    let has_id_and_values = params.iter().any(|(id, values)| !id.is_empty() && !values.is_empty());
    assert!(
        has_id_and_values,
        "expected at least one param entry with non-empty id and values; params: {params:?}"
    );
}

// ── M-5 / AC-8: param elements survive JSON→XML→JSON round-trip ─────────────

#[test]
fn param_elements_survive_xml_round_trip() {
    assert_catalog_round_trip_preserves("xml", "parameter", collect_params, |values| {
        !values.is_empty()
    });
}

// ── M-5 / AC-8: param elements survive JSON→YAML→JSON round-trip ────────────

#[test]
fn param_elements_survive_yaml_round_trip() {
    assert_catalog_round_trip_preserves("yaml", "parameter", collect_params, |values| {
        !values.is_empty()
    });
}
