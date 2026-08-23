use std::fs;
use std::process::Command;

use serde_json::{Value, json};
use tempfile::TempDir;

fn forge_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forge"))
}

fn catalog(uuid: &str, last_modified: &str, prose: &str) -> Value {
    json!({
        "catalog": {
            "uuid": uuid,
            "metadata": {
                "title": "Test Policy",
                "last-modified": last_modified,
                "version": "1.0",
                "oscal-version": "1.2.0"
            },
            "groups": [{
                "id": "access-control",
                "title": "Access Control",
                "controls": [{
                    "id": "POL-AC-001",
                    "uuid": "stable-control-uuid",
                    "title": "Authentication",
                    "parts": [{"name": "statement", "prose": prose}]
                }]
            }]
        }
    })
}

fn write_pair(dir: &TempDir, committed: &Value, generated: &Value) -> (String, String) {
    let committed_path = dir.path().join("committed.json");
    let generated_path = dir.path().join("generated.json");
    fs::write(&committed_path, serde_json::to_vec_pretty(committed).unwrap()).unwrap();
    fs::write(&generated_path, serde_json::to_vec_pretty(generated).unwrap()).unwrap();
    (committed_path.to_string_lossy().into_owned(), generated_path.to_string_lossy().into_owned())
}

#[test]
fn drift_json_reports_clean_for_only_volatile_metadata_changes() {
    let dir = TempDir::new().unwrap();
    let committed = catalog(
        "11111111-1111-4111-8111-111111111111",
        "2026-01-01T00:00:00Z",
        "Users must authenticate.",
    );
    let generated = catalog(
        "22222222-2222-4222-8222-222222222222",
        "2026-08-23T12:34:56Z",
        "Users must authenticate.",
    );
    let (committed_path, generated_path) = write_pair(&dir, &committed, &generated);

    let output = forge_bin()
        .args(["drift", &committed_path, &generated_path, "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "clean");
    assert_eq!(result["artifact_type"], "catalog");
    assert_eq!(result["comparison_contract"], 1);
    assert!(output.stderr.is_empty());
}

#[test]
fn drift_json_exits_one_without_disclosing_changed_policy_content() {
    let dir = TempDir::new().unwrap();
    let committed = catalog(
        "11111111-1111-4111-8111-111111111111",
        "2026-01-01T00:00:00Z",
        "Original confidential control text",
    );
    let generated = catalog(
        "22222222-2222-4222-8222-222222222222",
        "2026-08-23T12:34:56Z",
        "Changed confidential control text",
    );
    let (committed_path, generated_path) = write_pair(&dir, &committed, &generated);

    let output = forge_bin()
        .args(["drift", &committed_path, &generated_path, "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "drift");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!combined.contains("Original confidential"));
    assert!(!combined.contains("Changed confidential"));
}

#[test]
fn drift_rejects_mismatched_models_with_exit_two() {
    let dir = TempDir::new().unwrap();
    let committed = catalog(
        "11111111-1111-4111-8111-111111111111",
        "2026-01-01T00:00:00Z",
        "Users must authenticate.",
    );
    let generated = json!({
        "component-definition": {
            "uuid": "22222222-2222-4222-8222-222222222222",
            "metadata": {
                "title": "Component",
                "last-modified": "2026-08-23T12:34:56Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            },
            "components": []
        }
    });
    let (committed_path, generated_path) = write_pair(&dir, &committed, &generated);

    let output = forge_bin()
        .args(["drift", &committed_path, &generated_path, "--format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Artifact type mismatch"));
    assert!(output.stdout.is_empty());
}

#[test]
fn drift_read_errors_do_not_disclose_absolute_runner_paths() {
    let dir = TempDir::new().unwrap();
    let generated = catalog(
        "22222222-2222-4222-8222-222222222222",
        "2026-08-23T12:34:56Z",
        "Users must authenticate.",
    );
    let generated_path = dir.path().join("generated.json");
    fs::write(&generated_path, serde_json::to_vec_pretty(&generated).unwrap()).unwrap();
    let missing_path = dir.path().join("sensitive-absolute-runner-path.json");

    let output = forge_bin()
        .arg("drift")
        .arg(&missing_path)
        .arg(&generated_path)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let absolute_root = dir.path().display().to_string();
    assert!(stderr.contains("unable to inspect committed artifact"), "{stderr}");
    assert!(!stderr.contains(&absolute_root), "{stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn drift_model_errors_do_not_disclose_absolute_runner_paths() {
    let dir = TempDir::new().unwrap();
    let committed_path = dir.path().join("committed-profile.json");
    let generated_path = dir.path().join("generated-catalog.json");
    fs::write(
        &committed_path,
        serde_json::to_vec_pretty(&json!({"profile": {"uuid": "profile-uuid"}})).unwrap(),
    )
    .unwrap();
    fs::write(
        &generated_path,
        serde_json::to_vec_pretty(&catalog(
            "22222222-2222-4222-8222-222222222222",
            "2026-08-23T12:34:56Z",
            "Users must authenticate.",
        ))
        .unwrap(),
    )
    .unwrap();

    let output =
        forge_bin().arg("drift").arg(&committed_path).arg(&generated_path).output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let absolute_root = dir.path().display().to_string();
    assert!(stderr.contains("committed artifact uses unsupported Profile"), "{stderr}");
    assert!(!stderr.contains(&absolute_root), "{stderr}");
    assert!(output.stdout.is_empty());
}
