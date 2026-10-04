//! Black-box evidence-overlay contracts using real rich native originals and explicit linkage.
//!
//! These synthetic controls require the Root-owned CLI and isolated overlay producer. They do
//! not establish consumer interoperability, evidence authority, or runtime coverage until run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use forge::validate::{OscalModelType, run_full_validation};
use serde_json::{Value, json};

/// Existing digest-only fixture helpers; no new dependency or fixture authority.
mod common;

/// Actual originals and an explicit manifest; retained temporary-directory ownership cleans up.
struct Fixture {
    /// Keeps every actual original and output alive for the complete control.
    _directory: tempfile::TempDir,
    /// Canonical actual root, independent of the command's working directory.
    root: PathBuf,
}

impl Fixture {
    /// Copy immutable rich native templates and declare six independently observable evidence rows.
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary overlay root");
        let root = directory.path().canonicalize().expect("actual root");
        std::fs::create_dir(root.join("native")).expect("native directory");
        std::fs::write(
            root.join("native/catalog.json"),
            include_bytes!("fixtures/export/lossless-catalog.json"),
        )
        .expect("copy rich Catalog");
        std::fs::write(
            root.join("native/component.json"),
            include_bytes!("fixtures/export/lossless-component.json"),
        )
        .expect("copy rich Component Definition");
        let evidence = fixture_evidence_rows(&root);
        let manifest = json!({
            "schema_version":"forge.linkage/1",
            "project":{
                "key":"overlay-test", "title":"PRIVATE-PROJECT-TITLE",
                "expiring_window_days":30, "max_evidence_bytes":1_048_576,
                "approved_uri_schemes":[]
            },
            "reviewers":[{"key":"reviewer", "name":"PRIVATE-REVIEWER-NAME"}],
            "requirement_resources":[{
                "key":"catalog-main", "type":"catalog", "artifact":"native/catalog.json",
                "href":"native/catalog.json", "expected_sha256":common::sha256_file(&root.join("native/catalog.json"))
            }],
            "implementation_resource":{
                "key":"component-main", "type":"component-definition", "artifact":"native/component.json",
                "href":"native/component.json", "expected_sha256":common::sha256_file(&root.join("native/component.json"))
            },
            "evidence_roots":[{"key":"local", "path":"native"}],
            "evidence":evidence,
            "links":[{
                "key":"many-to-many",
                "requirements":[
                    {"resource_key":"catalog-main", "type":"control", "id_ref":"EX-1"},
                    {"resource_key":"catalog-main", "type":"statement", "id_ref":"EX-1_smt"}
                ],
                "implementations":[
                    {"type":"implemented-requirement", "id_ref":"000000d4-abcd-4abc-8abc-000000000001"},
                    {"type":"statement", "id_ref":"000000d7-abcd-4abc-8abc-000000000001"}
                ],
                "evidence_keys":["ev-current", "ev-uri"], "evidence_required":true,
                "responsible_role":"PRIVATE-ROLE", "implementation_status":"partial",
                "review":{
                    "reviewer_key":"reviewer", "reviewed_at":"2026-10-03T12:00:00Z",
                    "rationale":"PRIVATE-RATIONALE"
                },
                "impact_finding_ids":["PRIVATE-IMPACT"], "policy_version_keys":["PRIVATE-POLICY"]
            }]
        });
        write_json(&root.join("links.json"), &manifest);
        let fixture = Self { _directory: directory, root };
        // Admission canaries validate genuine native fixtures before any intended refusal.
        assert_native(
            &read_json(&fixture.root.join("native/catalog.json")),
            OscalModelType::Catalog,
        );
        assert_native(
            &read_json(&fixture.root.join("native/component.json")),
            OscalModelType::ComponentDefinition,
        );
        fixture
    }

    /// Invoke the proposed four-required-flag public CLI with an explicit actual manifest.
    fn overlay(&self, target: &str, as_of: &str, relative: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_forge"))
            .current_dir(&self.root)
            .args(["linkage", "overlay", "--manifest"])
            .arg(self.root.join("links.json"))
            .args(["--target-resource", target, "--as-of", as_of, "--output", relative])
            .output()
            .expect("run overlay CLI")
    }

    /// Read the actual declaration for a bounded, explicit mutation followed by a real write.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn manifest(&self) -> Value {
        read_json(&self.root.join("links.json"))
    }

    /// Replace only the caller-authored declaration bytes, without touching any native original.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn save_manifest(&self, value: &Value) {
        write_json(&self.root.join("links.json"), value);
    }

    /// Explicitly approve the current actual native bytes after a deliberate source mutation.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn repin(&self, relative: &str) {
        let mut manifest = self.manifest();
        let digest = common::sha256_file(&self.root.join(relative));
        match relative {
            "native/catalog.json" => {
                manifest["requirement_resources"][0]["expected_sha256"] = json!(digest);
            }
            "native/component.json" => {
                manifest["implementation_resource"]["expected_sha256"] = json!(digest);
            }
            _ => panic!("only explicitly known native originals can be repinned"),
        }
        self.save_manifest(&manifest);
    }

    /// Require a complete new file and no stdout from a supported publisher before decoding it.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn published(&self, target: &str, relative: &str) -> Value {
        let output = self.overlay(target, "2026-10-04", relative);
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, [] as [u8; 0]);
        read_json(&self.root.join(relative))
    }

    /// Require invalid exit two, no stdout, and no newly created destination.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn refused(&self, target: &str, as_of: &str, relative: &str) {
        assert!(!self.root.join(relative).exists(), "refusal destination must initially be absent");
        let output = self.overlay(target, as_of, relative);
        assert_eq!(output.status.code(), Some(2), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!self.root.join(relative).exists());
    }
}

/// Create the same six explicit evidence approvals and actual bounded originals on every platform.
fn fixture_evidence_rows(root: &Path) -> Vec<Value> {
    let mut evidence = Vec::new();
    for (key, name, valid_through, approved, actual) in [
        ("ev-current", "current.bin", "2027-12-31", b"current".as_slice(), b"current".as_slice()),
        (
            "ev-expiring",
            "expiring.bin",
            "2026-10-05",
            b"expiring".as_slice(),
            b"expiring".as_slice(),
        ),
        ("ev-expired", "expired.bin", "2026-10-03", b"expired".as_slice(), b"expired".as_slice()),
        (
            "ev-changed",
            "changed.bin",
            "2027-12-31",
            b"approved".as_slice(),
            b"mutated-original".as_slice(),
        ),
    ] {
        std::fs::write(root.join("native").join(name), actual).expect("actual evidence");
        evidence.push(local_evidence(key, name, valid_through, approved));
    }
    let mut unavailable = local_evidence("ev-unavailable", "absent.json", "2027-12-31", b"missing");
    // The recorded approval is u64; an unavailable record is not constrained by observed size.
    unavailable["location"]["expected_size"] = json!(u64::MAX);
    evidence.push(unavailable);
    evidence.push(json!({
        "key":"ev-uri", "title":"PRIVATE-EVIDENCE-TITLE", "evidence_type":"test-record",
        "owner":"PRIVATE-OWNER", "collected_at":"2026-10-02T00:00:00Z",
        "sensitivity_label":"restricted", "source_label":"PRIVATE-SOURCE-LABEL",
        "location":{
            "kind":"uri", "unverified":true,
            "uri":"https://user:password@example.invalid/ticket/1?token=SECRET-TOKEN#private"
        }
    }));
    evidence
}

/// Author a local approval with explicit sensitive metadata and exact recorded digest and size.
fn local_evidence(key: &str, path: &str, valid_through: &str, approved: &[u8]) -> Value {
    json!({
        "key":key, "title":"PRIVATE-EVIDENCE-TITLE", "evidence_type":"test-record",
        "owner":"PRIVATE-OWNER", "collected_at":"2026-10-02T00:00:00Z",
        "valid_through":valid_through, "sensitivity_label":"restricted", "source_label":"PRIVATE-SOURCE-LABEL",
        "location":{
            "kind":"local", "root_key":"local", "path":path,
            "expected_sha256":common::sha256_hex(approved), "expected_size":approved.len()
        }
    })
}

/// Serialize caller-authored fixture JSON; this helper never derives approvals or native content.
fn write_json(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).expect("serialize explicit JSON"))
        .expect("write explicit JSON");
}

/// Decode a complete actual file for value equality and native schema checks.
fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("read complete actual file"))
        .expect("decode complete actual file")
}

/// Consume the existing official native full validator, retaining its separate validity result.
fn assert_native(value: &Value, model: OscalModelType) {
    let report = run_full_validation("overlay-native-control.json", value, model)
        .expect("official native validator admitted input type");
    assert!(report.is_valid(), "native artifact must pass official full validation");
}

/// Require an exact object key set, rather than treating extra decoded fields as harmless.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_keys(value: &Value, expected: &[&str]) {
    let mut actual: Vec<_> =
        value.as_object().expect("closed object").keys().map(String::as_str).collect();
    let mut required = expected.to_vec();
    actual.sort_unstable();
    required.sort_unstable();
    assert_eq!(actual, required);
}

/// Recover the exact generated suffix, validate both closed profile layers, and prove full removal equality.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn generated(original: &Value, overlay: &Value, root_label: &str) -> Vec<(String, Value)> {
    let model = match root_label {
        "catalog" => OscalModelType::Catalog,
        "component-definition" => OscalModelType::ComponentDefinition,
        _ => panic!("only the supported native pair is exercised"),
    };
    assert_native(overlay, model);
    let old = original[root_label]["back-matter"]["resources"]
        .as_array()
        .expect("rich originals have resources");
    let resources = overlay[root_label]["back-matter"]["resources"]
        .as_array()
        .expect("complete overlay resources");
    assert_eq!(resources.len(), old.len() + 8);
    assert_eq!(&resources[..old.len()], old.as_slice());
    let mut restored = overlay.clone();
    restored[root_label]["back-matter"]["resources"]
        .as_array_mut()
        .expect("resource array")
        .truncate(old.len());
    assert_eq!(
        &restored, original,
        "remove exactly the suffix; preserve the entire decoded original"
    );
    let resource_schema: Value = serde_json::from_str(include_str!(
        "../schemas/forge.linkage-overlay-resource-1.schema.json"
    ))
    .expect("frozen closed resource schema");
    let record_schema: Value =
        serde_json::from_str(include_str!("../schemas/forge.linkage-overlay-record-1.schema.json"))
            .expect("frozen closed record schema");
    let resource_validator =
        jsonschema::validator_for(&resource_schema).expect("local resource schema");
    let record_validator = jsonschema::validator_for(&record_schema).expect("local record schema");
    resources[old.len()..]
        .iter()
        .map(|resource| {
            assert!(
                resource_validator.is_valid(resource),
                "new resource has the exact narrow wrapper"
            );
            let record_text =
                resource["props"][1]["value"].as_str().expect("compact record property");
            assert!(!record_text.contains('\n') && !record_text.contains('\r'));
            let record: Value =
                serde_json::from_str(record_text).expect("complete embedded record JSON");
            assert!(record_validator.is_valid(&record), "decoded record has the closed shape");
            assert_eq!(
                uuid::Uuid::parse_str(resource["uuid"].as_str().expect("resource UUID"))
                    .expect("native UUID")
                    .get_version_num(),
                5
            );
            (resource["uuid"].as_str().expect("resource UUID").to_owned(), record)
        })
        .collect()
}

/// Project stable record identity independently of mutable generation hashes and authoring order.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn identities(records: &[(String, Value)]) -> Vec<(String, String)> {
    records
        .iter()
        .map(|(id, value)| {
            let key = match value["kind"].as_str().expect("record kind") {
                "provenance" => "provenance",
                "link" => value["link_key"].as_str().expect("link key"),
                "evidence" => value["evidence_key"].as_str().expect("evidence key"),
                _ => panic!("closed record kind"),
            };
            (key.to_owned(), id.clone())
        })
        .collect()
}

/// Check the same complete original source identities, exact raw pins and source versions beside either target.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_complete_source_provenance(
    fixture: &Fixture,
    original: &Value,
    original_bytes: &[u8],
    provenance: &Value,
    target: &str,
    label: &str,
) {
    assert_eq!(provenance["link_count"], 1);
    assert_eq!(provenance["evidence_count"], 6);
    assert_eq!(provenance["authority"], "association-only");
    assert_eq!(provenance["as_of"], "2026-10-04");
    assert_eq!(
        provenance["manifest_sha256"],
        common::sha256_file(&fixture.root.join("links.json"))
    );
    assert_eq!(
        provenance["target"],
        json!({
            "resource_key":target,
            "side":if label == "catalog" { "requirement" } else { "implementation" },
            "model":label, "root_uuid":original[label]["uuid"],
            "raw_sha256":common::sha256_hex(original_bytes),
            "document_version":original[label]["metadata"]["version"],
            "oscal_version":original[label]["metadata"]["oscal-version"]
        })
    );
    assert_eq!(provenance["sources"].as_array().expect("whole native closure").len(), 2);
    for (key, path) in
        [("catalog-main", "native/catalog.json"), ("component-main", "native/component.json")]
    {
        let source_row = provenance["sources"]
            .as_array()
            .expect("source rows")
            .iter()
            .find(|row| row["resource_key"] == key)
            .expect("each whole-closure source");
        assert_eq!(source_row["raw_sha256"], common::sha256_file(&fixture.root.join(path)));
        assert!(source_row["resolved_catalog_sha256"].is_null());
        let native = read_json(&fixture.root.join(path));
        let source_label = if key == "catalog-main" { "catalog" } else { "component-definition" };
        assert_eq!(source_row["root_uuid"], native[source_label]["uuid"]);
        assert_eq!(source_row["document_version"], native[source_label]["metadata"]["version"]);
        assert_eq!(source_row["oscal_version"], native[source_label]["metadata"]["oscal-version"]);
    }
}

/// Both real native models preserve complete decoded originals and one full many-to-many association.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_both_models_preserve_complete_native_values_and_many_to_many() {
    let fixture = Fixture::new();
    for (target, label, source, destination) in [
        ("catalog-main", "catalog", "native/catalog.json", "native/catalog-overlay.json"),
        (
            "component-main",
            "component-definition",
            "native/component.json",
            "native/component-overlay.json",
        ),
    ] {
        let original_bytes = std::fs::read(fixture.root.join(source)).expect("original raw bytes");
        let original: Value = serde_json::from_slice(&original_bytes).expect("rich native value");
        let records = generated(&original, &fixture.published(target, destination), label);
        assert_eq!(records[0].1["kind"], "provenance");
        assert_eq!(records[1].1["kind"], "link");
        assert_eq!(records.iter().filter(|(_, row)| row["kind"] == "link").count(), 1);
        let provenance = &records[0].1;
        assert_complete_source_provenance(
            &fixture,
            &original,
            &original_bytes,
            provenance,
            target,
            label,
        );
        let link = &records[1].1;
        assert_eq!(link["requirements"].as_array().expect("requirements").len(), 2);
        assert_eq!(link["implementations"].as_array().expect("implementations").len(), 2);
        assert_eq!(link["requirements"][0]["resource_key"], "catalog-main");
        assert_eq!(link["implementations"][0]["resource_key"], "component-main");
        for (side, key, kinds_and_ids) in [
            ("requirements", "catalog-main", [("control", "EX-1"), ("statement", "EX-1_smt")]),
            (
                "implementations",
                "component-main",
                [
                    ("implemented-requirement", "000000d4-abcd-4abc-8abc-000000000001"),
                    ("statement", "000000d7-abcd-4abc-8abc-000000000001"),
                ],
            ),
        ] {
            for (kind, id) in kinds_and_ids {
                let subject = link[side]
                    .as_array()
                    .expect("complete subject side")
                    .iter()
                    .find(|row| row["type"] == kind && row["id"] == id)
                    .expect("exact selected subject");
                assert_eq!(subject["resource_key"], key);
            }
        }
        assert_eq!(link["evidence"].as_array().expect("evidence associations").len(), 2);
        assert_eq!(
            link["recorded_review"],
            json!({"reviewer_key":"reviewer", "reviewed_at":"2026-10-03T12:00:00Z"})
        );
        for pair in link["evidence"].as_array().expect("evidence associations") {
            let record = records
                .iter()
                .find(|(_, row)| row["evidence_key"] == pair["key"])
                .expect("association joins a generated evidence resource");
            assert_eq!(pair["resource_uuid"], record.0);
        }
        assert_eq!(
            std::fs::read(fixture.root.join(source)).expect("unchanged raw original"),
            original_bytes
        );
    }
}

/// Complete fresh states preserve nullable observations and redact only newly generated sensitive fields.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_complete_fresh_states_and_original_sensitive_exception_are_explicit() {
    let fixture = Fixture::new();
    let original = read_json(&fixture.root.join("native/catalog.json"));
    let output = fixture.published("catalog-main", "native/states.json");
    let records = generated(&original, &output, "catalog");
    for (key, state) in [
        ("ev-current", "current"),
        ("ev-expiring", "expiring"),
        ("ev-expired", "expired"),
        ("ev-changed", "changed"),
        ("ev-unavailable", "unavailable"),
        ("ev-uri", "unverified-uri"),
    ] {
        let row = &records
            .iter()
            .find(|(_, row)| row["evidence_key"] == key)
            .expect("all declared evidence")
            .1;
        assert_eq!(row["freshness"], state);
        let reference = &row["reference"];
        if key == "ev-uri" {
            assert_eq!(reference, &json!({"kind":"uri", "expected_sha256":null}));
            assert!(row["recorded_valid_through"].is_null());
        } else if key == "ev-unavailable" {
            assert!(reference["observed_sha256"].is_null());
            assert!(reference["observed_size"].is_null());
            assert_eq!(reference["approved_size"], json!(u64::MAX));
        } else {
            assert!(reference["observed_sha256"].is_string());
            assert!(reference["observed_size"].is_u64());
            if key == "ev-changed" {
                assert_ne!(reference["approved_sha256"], reference["observed_sha256"]);
            } else {
                assert_eq!(reference["approved_sha256"], reference["observed_sha256"]);
                assert_eq!(reference["approved_size"], reference["observed_size"]);
            }
        }
    }
    let generated_text = serde_json::to_string(&records).expect("bounded generated records");
    for sensitive in [
        "PRIVATE-OWNER",
        "PRIVATE-RATIONALE",
        "PRIVATE-ROLE",
        "PRIVATE-IMPACT",
        "PRIVATE-POLICY",
        "PRIVATE-EVIDENCE-TITLE",
        "PRIVATE-SOURCE-LABEL",
        "PRIVATE-PROJECT-TITLE",
        "PRIVATE-REVIEWER-NAME",
        "SECRET-TOKEN",
        "password",
        "https://user:",
        "current.bin",
        "native/catalog.json",
        "mutated-original",
    ] {
        assert!(!generated_text.contains(sensitive), "new payload leaked {sensitive}");
    }
    assert!(!generated_text.contains(fixture.root.to_str().expect("fixture UTF-8 path")));
    // Original rich resource content is deliberately preserved; the whole artifact is not sanitized.
    assert!(
        original["catalog"]["back-matter"]["resources"]
            .as_array()
            .expect("original resources")
            .iter()
            .any(|resource| resource.get("base64").is_some())
    );
    assert_keys(&records[1].1["recorded_review"], &["reviewer_key", "reviewed_at"]);
}

/// Identical held input bytes are deterministic across roots; reordered declarations keep stable IDs only.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_determinism_and_generation_identity_are_separate() {
    let first = Fixture::new();
    let second = Fixture::new();
    let original = read_json(&first.root.join("native/catalog.json"));
    let first_value = first.published("catalog-main", "native/first.json");
    let second_value = second.published("catalog-main", "native/second.json");
    assert_eq!(first_value, second_value);
    assert_eq!(
        std::fs::read(first.root.join("native/first.json")).expect("first bytes"),
        std::fs::read(second.root.join("native/second.json")).expect("second bytes")
    );
    let before = generated(&original, &first_value, "catalog");
    let mut declaration = first.manifest();
    declaration["evidence"].as_array_mut().expect("declared evidence").reverse();
    declaration["links"][0]["requirements"].as_array_mut().expect("requirements").reverse();
    declaration["links"][0]["implementations"].as_array_mut().expect("implementations").reverse();
    declaration["links"][0]["evidence_keys"].as_array_mut().expect("evidence keys").reverse();
    first.save_manifest(&declaration);
    let after =
        generated(&original, &first.published("catalog-main", "native/reordered.json"), "catalog");
    assert_eq!(identities(&before), identities(&after));
    assert_ne!(before[0].1["manifest_sha256"], after[0].1["manifest_sha256"]);
    assert_eq!(
        &before[1..],
        &after[1..],
        "authoring order changes only the recorded manifest generation"
    );
}

/// A schema-valid string property containing any complete parsed UUID spelling collides with stable output.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_full_original_uuid_collision_scan_is_not_limited_to_uuid_fields() {
    let fixture = Fixture::new();
    let original = read_json(&fixture.root.join("native/catalog.json"));
    let records = generated(
        &original,
        &fixture.published("catalog-main", "native/id-canary.json"),
        "catalog",
    );
    let generated_uuid = uuid::Uuid::parse_str(&records[0].0).expect("generated UUID");
    let mut with_property = original;
    with_property["catalog"]["metadata"]["props"].as_array_mut().expect("rich metadata properties")
        .push(json!({"name":"overlay-collision-canary", "value":"11111111-2222-4333-8444-555555555555"}));
    let canary_index =
        with_property["catalog"]["metadata"]["props"].as_array().expect("properties").len() - 1;
    for (index, spelling) in [
        generated_uuid.hyphenated().to_string(),
        generated_uuid.hyphenated().to_string().to_ascii_uppercase(),
        generated_uuid.simple().to_string(),
        generated_uuid.braced().to_string(),
        generated_uuid.urn().to_string(),
    ]
    .into_iter()
    .enumerate()
    {
        with_property["catalog"]["metadata"]["props"][canary_index]["value"] =
            json!("11111111-2222-4333-8444-555555555555");
        write_json(&fixture.root.join("native/catalog.json"), &with_property);
        fixture.repin("native/catalog.json");
        fixture.published("catalog-main", &format!("native/property-canary-{index}.json"));
        with_property["catalog"]["metadata"]["props"][canary_index]["value"] = json!(spelling);
        assert_native(&with_property, OscalModelType::Catalog);
        write_json(&fixture.root.join("native/catalog.json"), &with_property);
        fixture.repin("native/catalog.json");
        fixture.refused("catalog-main", "2026-10-04", &format!("native/collision-{index}.json"));
    }
}

/// Selecting a Catalog still binds current Component originals; exact repinning restores valid preparation.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_whole_native_closure_rejects_unselected_raw_pin_drift() {
    let fixture = Fixture::new();
    fixture.published("catalog-main", "native/closure-positive.json");
    let original = std::fs::read(fixture.root.join("native/component.json"))
        .expect("unselected actual component");
    let mut changed = original.clone();
    changed.push(b'\n');
    std::fs::write(fixture.root.join("native/component.json"), &changed)
        .expect("same native value new raw generation");
    assert_native(
        &read_json(&fixture.root.join("native/component.json")),
        OscalModelType::ComponentDefinition,
    );
    fixture.refused("catalog-main", "2026-10-04", "native/unselected-drift.json");
    fixture.repin("native/component.json");
    fixture.published("catalog-main", "native/closure-reapproved.json");
    assert_eq!(
        std::fs::read(fixture.root.join("native/component.json"))
            .expect("actual mutated bytes preserved"),
        changed
    );
}

/// Output is confined to the selected target's existing parent and cannot replace or alias held inputs.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_same_parent_no_overwrite_and_unavailable_ancestor_collisions() {
    let fixture = Fixture::new();
    fixture.published("catalog-main", "native/placement-positive.json");
    std::fs::create_dir(fixture.root.join("other")).expect("existing unrelated output parent");
    fixture.refused("catalog-main", "2026-10-04", "other/catalog-overlay.json");
    let existing = fixture.root.join("native/existing.json");
    std::fs::write(&existing, b"existing-output-sentinel").expect("existing destination");
    let output = fixture.overlay("catalog-main", "2026-10-04", "native/existing.json");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_eq!(std::fs::read(&existing).expect("existing sentinel"), b"existing-output-sentinel");
    for relative in ["native/catalog.json", "native/component.json", "links.json"] {
        let path = fixture.root.join(relative);
        let before = std::fs::read(&path).expect("held input sentinel");
        let output = fixture.overlay("catalog-main", "2026-10-04", relative);
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert_eq!(std::fs::read(&path).expect("unchanged held input"), before);
    }
    // Preserve the lowercase extension so suffix rejection cannot preempt this case vector.
    let case_alias = "native/CATALOG.json";
    let catalog_before =
        std::fs::read(fixture.root.join("native/catalog.json")).expect("catalog bytes");
    let output = fixture.overlay("catalog-main", "2026-10-04", case_alias);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert_eq!(
        std::fs::read(fixture.root.join("native/catalog.json")).expect("catalog preserved"),
        catalog_before
    );
    // Existing-file publication may preempt the alias check on a case-insensitive volume.
    assert!(!fixture.root.join("native/absent.json").exists());
    fixture.refused("catalog-main", "2026-10-04", "native/absent.json");
    let mut manifest = fixture.manifest();
    manifest["evidence"].as_array_mut().expect("evidence declarations").push(local_evidence(
        "ev-ancestor",
        "blocked.json/child.json",
        "2027-12-31",
        b"absent-child",
    ));
    fixture.save_manifest(&manifest);
    fixture.published("catalog-main", "native/ancestor-positive.json");
    fixture.refused("catalog-main", "2026-10-04", "native/blocked.json");
}

/// Required CLI flags, actual target membership, calendar admission and closed declarations refuse atomically.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn overlay_closed_cli_and_manifest_admission_refusals_follow_valid_canary() {
    let fixture = Fixture::new();
    fixture.published("catalog-main", "native/admission-positive.json");
    fixture.refused("not-a-declared-target", "2026-10-04", "native/wrong-target.json");
    fixture.refused("catalog-main", "2026-02-30", "native/wrong-calendar.json");
    for missing in ["--manifest", "--target-resource", "--as-of", "--output"] {
        let manifest = fixture.root.join("links.json");
        let all = [
            ("--manifest", manifest.to_str().expect("fixture UTF-8 manifest")),
            ("--target-resource", "catalog-main"),
            ("--as-of", "2026-10-04"),
            ("--output", "native/missing-flag.json"),
        ];
        let mut command = Command::new(env!("CARGO_BIN_EXE_forge"));
        command.current_dir(&fixture.root).args(["linkage", "overlay"]);
        for (flag, value) in all {
            if flag != missing {
                command.args([flag, value]);
            }
        }
        let output = command.output().expect("missing required flag");
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(output.stdout, [] as [u8; 0]);
        assert!(!fixture.root.join("native/missing-flag.json").exists());
    }
    let mut unknown = fixture.manifest();
    unknown["unrecognized_authority"] = json!(true);
    fixture.save_manifest(&unknown);
    fixture.refused("catalog-main", "2026-10-04", "native/unknown-field.json");
    unknown.as_object_mut().expect("manifest object").remove("unrecognized_authority");
    fixture.save_manifest(&unknown);
    fixture.published("catalog-main", "native/closed-reapproved.json");
    let raw = std::fs::read_to_string(fixture.root.join("links.json"))
        .expect("actual valid manifest bytes");
    let duplicate = format!("{{\"schema_version\":\"forge.linkage/1\",{}", &raw[1..]);
    std::fs::write(fixture.root.join("links.json"), duplicate)
        .expect("genuine duplicate key spelling");
    fixture.refused("catalog-main", "2026-10-04", "native/duplicate-key.json");
}

/// Unsupported publishers fail closed for an otherwise explicit real native/source fixture, without stdout.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[test]
fn overlay_unsupported_platform_does_not_publish_or_emit_stdout() {
    let fixture = Fixture::new();
    let output = fixture.overlay("catalog-main", "2026-10-04", "native/unsupported.json");
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, [] as [u8; 0]);
    assert!(!fixture.root.join("native/unsupported.json").exists());
    #[cfg(target_os = "windows")]
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("cannot publish a new complete evidence overlay"),
        "Windows must reach publication refusal after real source admission"
    );
    assert_eq!(
        std::fs::read(fixture.root.join("native/catalog.json")).expect("original preserved"),
        include_bytes!("fixtures/export/lossless-catalog.json")
    );
    assert_eq!(
        std::fs::read(fixture.root.join("native/component.json")).expect("original preserved"),
        include_bytes!("fixtures/export/lossless-component.json")
    );
}
