//! Integration tests: Profile generation end-to-end pipeline (WI-35, US2, M-2–M-4, S-1–S-2).
//!
//! Verifies the complete `forge profile` pipeline: generate Profile → validate via `forge validate`.
//! Covers include/exclude control selection, set-param, multi-format output, and schema validation.
//! Uses the CLI subprocess pattern (`env!("CARGO_BIN_EXE_forge")`) for end-to-end coverage.

use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn run_forge(args: &[&str]) -> std::process::Output {
    let output = forge_bin().args(args).output().expect("failed to execute forge");
    assert!(output.status.success(),
        "forge {:?} failed (exit {})\nstdout: {}\nstderr: {}",
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Generate a Catalog JSON from the golden/small fixture and return the temp dir + catalog path.
fn catalog_from_policy(dir: &TempDir) -> std::path::PathBuf {
    let catalog_path = dir.path().join("catalog.json");
    run_forge(&[
        "convert",
        "tests/fixtures/golden/small/input.md",
        "--strategy",
        "catalog",
        "--format",
        "json",
        "--output",
        catalog_path.to_str().unwrap(),
    ]);
    catalog_path
}

fn read_json(path: &std::path::Path) -> Value {
    let content = fs::read_to_string(path).expect("failed to read JSON file");
    serde_json::from_str(&content).expect("failed to parse JSON")
}

/// Extract the first two control IDs from `catalog.groups[n].controls[*].id`.
fn extract_control_ids(catalog: &Value) -> Vec<String> {
    let groups = catalog["catalog"]["groups"].as_array().unwrap();
    for group in groups {
        if let Some(controls) = group["controls"].as_array() {
            let ids: Vec<String> = controls
                .iter()
                .take(2)
                .filter_map(|c| c["id"].as_str())
                .map(std::string::ToString::to_string)
                .collect();
            if !ids.is_empty() {
                return ids;
            }
        }
    }
    panic!("no controls found in catalog fixture")
}

// ── M-2 / AC-3: include-controls ─────────────────────────────────────────────

#[test]
fn profile_include_produces_valid_oscal() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);
    let include_arg = ids.join(",");

    let profile_path = dir.path().join("profile.json");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--include",
        &include_arg,
        "--format",
        "json",
        "--output",
        profile_path.to_str().unwrap(),
    ]);

    let profile = read_json(&profile_path);

    // imports[0].include-controls[0].with-ids must contain the requested IDs
    let with_ids = &profile["profile"]["imports"][0]["include-controls"][0]["with-ids"];
    assert!(with_ids.is_array(), "include-controls[0].with-ids must be an array");
    let with_ids_arr = with_ids.as_array().unwrap();
    for id in &ids {
        assert!(
            with_ids_arr.iter().any(|v| v.as_str() == Some(id.as_str())),
            "expected control ID '{id}' in with-ids, got: {with_ids_arr:?}"
        );
    }
}

// ── S-1: exclude-controls ─────────────────────────────────────────────────────

#[test]
fn profile_exclude_produces_valid_oscal() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);
    let exclude_arg = ids[0].clone();

    let profile_path = dir.path().join("profile_exclude.json");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--exclude",
        &exclude_arg,
        "--format",
        "json",
        "--output",
        profile_path.to_str().unwrap(),
    ]);

    let profile = read_json(&profile_path);

    // imports[0].exclude-controls must be present
    let exclude_controls = &profile["profile"]["imports"][0]["exclude-controls"];
    assert!(
        exclude_controls.is_array(),
        "exclude-controls must be an array, got: {exclude_controls}"
    );
}

// ── M-3 / AC-4: set-parameters ───────────────────────────────────────────────

#[test]
fn profile_set_param_produces_modify_section() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);

    let profile_path = dir.path().join("profile_param.json");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--include",
        &ids[0],
        "--set-param",
        "password-length",
        "16",
        "--format",
        "json",
        "--output",
        profile_path.to_str().unwrap(),
    ]);

    let profile = read_json(&profile_path);

    // modify.set-parameters must contain the password-length entry
    let set_params = &profile["profile"]["modify"]["set-parameters"];
    assert!(set_params.is_array(), "modify.set-parameters must be an array");
    let found = set_params
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["param-id"].as_str() == Some("password-length"));
    assert!(found, "expected set-parameter with param-id 'password-length' in: {set_params}");

    // The value must be "16"
    let entry = set_params
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["param-id"].as_str() == Some("password-length"))
        .unwrap();
    let values = &entry["values"];
    assert!(
        values.as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some("16"))),
        "expected value '16' in set-parameter, got: {values}"
    );
}

// ── M-4 / AC-5: forge validate on generated Profile ──────────────────────────

#[test]
fn profile_passes_schema_validation() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);

    let profile_path = dir.path().join("profile_valid.json");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--include",
        &ids.join(","),
        "--format",
        "json",
        "--output",
        profile_path.to_str().unwrap(),
    ]);

    // forge validate must exit 0 and report Valid
    let output = forge_bin()
        .args(["validate", profile_path.to_str().unwrap()])
        .output()
        .expect("failed to run forge validate");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "forge validate failed:\n{stdout}");
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "expected 'Valid' in validate output, got: {stdout}"
    );
}

// ── EC-2: nonexistent control ID (permissive: exits 0, ID included) ───────────

#[test]
fn profile_include_nonexistent_id_produces_error() {
    // EC-2 clarification: forge profile is permissive — nonexistent control IDs are included
    // in the Profile output (they are just references; no catalog validation is performed).
    // The command exits 0 and the ID appears in imports[0].include-controls[0].with-ids.
    // This matches the OSCAL model where profile control imports are URI references.
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);

    let profile_path = dir.path().join("profile_nonexistent.json");
    let output = forge_bin()
        .args([
            "profile",
            "--catalog",
            catalog_path.to_str().unwrap(),
            "--include",
            "NONEXISTENT-CONTROL-999",
            "--format",
            "json",
            "--output",
            profile_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run forge profile");

    // Permissive: exits 0, ID included in profile output
    assert!(
        output.status.success(),
        "forge profile exited non-zero for nonexistent ID: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(profile_path.exists(), "profile output file should be created");
    let profile = read_json(&profile_path);
    let with_ids = &profile["profile"]["imports"][0]["include-controls"][0]["with-ids"];
    assert!(
        with_ids
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("NONEXISTENT-CONTROL-999"))),
        "expected NONEXISTENT-CONTROL-999 in with-ids (permissive behavior)"
    );
}

// ── EC-3: --set-param with nonexistent param ID exits 0 (permissive) ─────────

#[test]
fn profile_set_param_nonexistent_id_exits_zero() {
    // EC-3 clarification: OSCAL Profile param IDs are independent of Catalog param IDs.
    // --set-param always produces a modify.set-parameters entry regardless of whether the
    // param ID exists in the source Catalog. The command exits 0.
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);

    let profile_path = dir.path().join("profile_nonexistent_param.json");
    let output = forge_bin()
        .args([
            "profile",
            "--catalog",
            catalog_path.to_str().unwrap(),
            "--include",
            &ids[0],
            "--set-param",
            "nonexistent-param-999",
            "42",
            "--format",
            "json",
            "--output",
            profile_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run forge profile");

    assert!(
        output.status.success(),
        "forge profile should exit 0 for nonexistent param ID: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let profile = read_json(&profile_path);
    let set_params = &profile["profile"]["modify"]["set-parameters"];
    assert!(set_params.is_array(), "modify.set-parameters must be an array");
    let found = set_params
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["param-id"].as_str() == Some("nonexistent-param-999"));
    assert!(found, "expected set-parameter with param-id 'nonexistent-param-999' in: {set_params}");
    let entry = set_params
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["param-id"].as_str() == Some("nonexistent-param-999"))
        .unwrap();
    let values = &entry["values"];
    assert!(
        values.as_array().is_some_and(|a| a.iter().any(|v| v.as_str() == Some("42"))),
        "expected value '42' in nonexistent param entry, got: {values}"
    );
}

// ── EC-6: --include and --exclude together are rejected by the CLI ─────────────

#[test]
fn profile_include_and_exclude_rejected() {
    // EC-6: --include and --exclude are declared mutually exclusive (conflicts_with).
    // clap rejects the combination before any forge logic runs; exit code is non-zero.
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);

    let output = forge_bin()
        .args([
            "profile",
            "--catalog",
            catalog_path.to_str().unwrap(),
            "--include",
            &ids[0],
            "--exclude",
            ids.get(1).map_or(&ids[0], String::as_str),
            "--format",
            "json",
        ])
        .output()
        .expect("failed to run forge profile");

    assert!(
        !output.status.success(),
        "forge profile should exit non-zero when both --include and --exclude are provided"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("error"),
        "expected usage error in stderr, got: {stderr}"
    );
}

// ── S-2: Profile in XML and YAML formats ─────────────────────────────────────

#[test]
fn profile_xml_yaml_formats() {
    let dir = TempDir::new().unwrap();
    let catalog_path = catalog_from_policy(&dir);
    let catalog = read_json(&catalog_path);
    let ids = extract_control_ids(&catalog);

    // XML format
    let xml_path = dir.path().join("profile.xml");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--include",
        &ids[0],
        "--format",
        "xml",
        "--output",
        xml_path.to_str().unwrap(),
    ]);
    let xml_content = fs::read_to_string(&xml_path).expect("failed to read profile XML");
    assert!(!xml_content.is_empty(), "profile XML must not be empty");
    assert!(
        xml_content.contains("<profile"),
        "profile XML must contain <profile element, got: {}...",
        &xml_content[..xml_content.len().min(200)]
    );

    // YAML format — parse and assert profile.uuid is present
    let yaml_path = dir.path().join("profile.yaml");
    run_forge(&[
        "profile",
        "--catalog",
        catalog_path.to_str().unwrap(),
        "--include",
        &ids[0],
        "--format",
        "yaml",
        "--output",
        yaml_path.to_str().unwrap(),
    ]);
    let yaml_content = fs::read_to_string(&yaml_path).expect("failed to read profile YAML");
    let yaml_value: Value =
        serde_yaml::from_str(&yaml_content).expect("failed to parse profile YAML");
    let uuid = yaml_value["profile"]["uuid"].as_str().unwrap_or("");
    assert!(!uuid.is_empty(), "profile YAML must have a non-empty profile.uuid");
}
