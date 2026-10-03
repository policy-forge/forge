//! Exact metadata export and index-replacement plans, before Store retention.
//!
//! Import alone grants explicit confined reads of incoming portable paths. The
//! registered-only comparison remains unchanged. A plan confers no domain
//! approval, source hydration or multi-file transaction authority.

use serde_json::{Value, json};

use super::actions::output_target;
use super::bundles::{self, Bundle};
use super::contract::{self, ApiMajor, Error, Result};
use super::effects::ArtifactFamily;
use super::index::{INDEX_PATH, Index, MAX_INDEX_BYTES, Resource, Role};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::root::{Captured, Root, Target, conflict};
use super::services::{Snapshot, resource_id, validate_bytes};

/// Complete physical-file limit for these effects, including a present index.
const MAX_FILES: usize = 100;
/// Existing complete capture ceiling, shared by current and incoming files.
const MAX_CAPTURE_BYTES: usize = 50 * 1024 * 1024;
/// Existing per-file limit; incoming declarations do not increase it.
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;

/// Complete unretained proposal consumed atomically by the root-owned Store seam.
pub(crate) struct PreparedBundle {
    /// One already-opened conditional target; import always uses the index path.
    pub(crate) target: Target,
    /// Existing closed effect type, without new Operation or preview enum values.
    pub(crate) kind: &'static str,
    /// Exact bytes to publish only after receipt confirmation.
    pub(crate) bytes: Vec<u8>,
    /// Existing captured version, without changing its raw-index hash algorithm.
    pub(crate) snapshot_version: String,
    /// Complete ordered resource-id/hash pins, independently bounded to 100.
    pub(crate) input_hashes: Vec<Value>,
    /// Private distinct-file captures including the raw index when present.
    pub(crate) external: Vec<(String, Captured)>,
    /// Truthful server-authored summary; authored metadata may be sensitive.
    pub(crate) semantic_summary: String,
    /// Private media family; index replacement is not a downloadable export.
    pub(crate) artifact_family: Option<ArtifactFamily>,
    /// Complete import membership projection, consumed before the plan is moved.
    pub(crate) replacement: Option<Value>,
    /// Actual distinct physical-file union, including any existing output base.
    pub(crate) consumed_file_count: usize,
}

/// Ordered private captures and public pins assembled without mutating a Store.
#[derive(Default)]
struct Bindings {
    /// First-observation file order; repeated exact paths retain one byte buffer.
    files: Vec<(String, Captured)>,
    /// Public incoming pins followed by current-only pins; raw index is private.
    pins: Vec<Value>,
    /// Checked distinct retained capture bytes, never two independent budgets.
    bytes: usize,
}

impl Bindings {
    /// Reserve a possible new file before reading it and reject portable aliases.
    fn before_read(&self, path: &str) -> Result<usize> {
        if self.files.iter().any(|(old, _)| old != path && old.eq_ignore_ascii_case(path)) {
            return Err(Error::containment());
        }
        if !self.files.iter().any(|(old, _)| old == path) && self.files.len() >= MAX_FILES {
            return Err(too_many_inputs());
        }
        MAX_CAPTURE_BYTES.checked_sub(self.bytes).ok_or_else(payload_limit)
    }

    /// Bind exact physical identity/hash/length before cloning a distinct capture.
    fn insert(&mut self, path: &str, captured: &Captured) -> Result<()> {
        self.before_read(path)?;
        if let Some((old_path, old)) = self
            .files
            .iter()
            .find(|(old_path, old)| old_path == path || old.identity == captured.identity)
        {
            if old_path != path {
                return Err(Error::containment());
            }
            return same_capture(old, captured);
        }
        let bytes = self.bytes.checked_add(captured.bytes.len()).ok_or_else(payload_limit)?;
        if bytes > MAX_CAPTURE_BYTES {
            return Err(payload_limit());
        }
        self.files.push((path.to_owned(), captured.clone()));
        self.bytes = bytes;
        Ok(())
    }

    /// Reuse only an exact portable path; aliases never receive another authority.
    fn captured(&self, path: &str) -> Option<&Captured> {
        self.files.iter().find(|(old, _)| old == path).map(|(_, captured)| captured)
    }

    /// Add complete stable resource pins without hiding changed or duplicate IDs.
    fn pin(&mut self, registration: &Resource, captured: &Captured) -> Result<()> {
        let id = resource_id(registration);
        if let Some(old) = self.pins.iter().find(|old| old["resource_id"] == id) {
            if old["sha256"] != captured.sha256 {
                return Err(conflict());
            }
            return Ok(());
        }
        if self.pins.len() >= MAX_FILES {
            return Err(too_many_inputs());
        }
        self.pins.push(json!({"resource_id":id,"sha256":captured.sha256}));
        Ok(())
    }
}

/// Preserve the existing safe whole-effect rejection rather than a prefix result.
fn too_many_inputs() -> Error {
    Error::new(
        "invalid-request",
        "The prepared effect consumes more than 100 inputs. Reduce the inputs this effect binds.",
        false,
    )
}

/// Reject the complete private union without disclosing which file exceeded it.
fn payload_limit() -> Error {
    Error::new("payload-too-large", "The complete bundle capture exceeds the input limit.", false)
}

/// Return the existing generic intrinsic-admission failure without private prose.
fn validation_error() -> Error {
    Error::new(
        "validation-failed",
        "Review the complete proposed document and its registered dependencies.",
        false,
    )
}

/// Compare retained instance and exact-byte facts, including length and content.
fn same_capture(left: &Captured, right: &Captured) -> Result<()> {
    if left.identity != right.identity || left.sha256 != right.sha256 || left.bytes != right.bytes {
        return Err(conflict());
    }
    Ok(())
}

/// Validate snapshot/raw-index consistency and bind every current physical file.
fn current_bindings(snapshot: &Snapshot) -> Result<Bindings> {
    let mut bindings = Bindings::default();
    let index_bytes = snapshot.index.bytes()?;
    match (snapshot.index_present, snapshot.captured_index()) {
        (true, Some(captured)) => {
            if Index::parse(&captured.bytes)?.bytes()? != index_bytes {
                return Err(conflict());
            }
            bindings.insert(INDEX_PATH, captured)?;
        }
        (false, None) if snapshot.items.is_empty() && snapshot.index.resources.is_empty() => {}
        _ => return Err(Error::invalid()),
    }
    if snapshot.index.resources.len() != snapshot.items.len() {
        return Err(Error::invalid());
    }
    for (registration, item) in snapshot.index.resources.iter().zip(&snapshot.items) {
        if registration.key != item.registration.key
            || registration.role != item.registration.role
            || registration.path != item.registration.path
        {
            return Err(Error::invalid());
        }
        bindings.insert(&registration.path, &item.captured)?;
    }
    Ok(bindings)
}

/// Revalidate a direct Value request using the selected v2 closed wire definition.
fn admit_request(request: &Value, schema: &str) -> Result<()> {
    let bytes = contract::encode(request, 1024 * 1024, false)?;
    let value = contract::parse(&bytes, 1024 * 1024, 64 * 1024)?;
    contract::validate_for(ApiMajor::V2, schema, &value)
}

/// Build exact metadata bytes without source content or an unguarded destination.
///
/// This returns no receipt/Operation and writes nothing. Root runtime supplies
/// selected-major admission, cooperative control and final atomic retention.
pub(crate) fn prepare_export(
    root: &Root,
    snapshot: &Snapshot,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedBundle> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "ProjectBundleExportRequest")?;
    let path = request["target_path"].as_str().ok_or_else(Error::invalid)?;
    output_target(snapshot, path, None)?;
    let mut bindings = current_bindings(snapshot)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let remaining = MAX_CAPTURE_BYTES.checked_sub(bindings.bytes).ok_or_else(payload_limit)?;
    let target = root.target_with_limit(path, remaining.min(MAX_RESOURCE_BYTES))?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if let Some(base) = &target.base {
        bindings.insert(path, base)?;
    }
    for item in &snapshot.items {
        bindings.pin(&item.registration, &item.captured)?;
    }
    let bytes = bundles::encode_metadata_for_api(snapshot, ApiMajor::V2)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    Ok(PreparedBundle {
        target,
        kind: "report-export",
        bytes,
        snapshot_version: snapshot.version.clone(),
        input_hashes: bindings.pins,
        consumed_file_count: bindings.files.len(),
        external: bindings.files,
        semantic_summary: "Export the complete registered index with authored labels, keys, roles, relative paths, hashes and byte lengths. This metadata may be sensitive. Source excerpts are excluded; no secret-free or domain-approval claim is made.".into(),
        artifact_family: Some(ArtifactFamily::MetadataJson),
        replacement: None,
    })
}

/// Choose only the explicitly requested index version and refuse every downgrade.
fn proposed_index(snapshot: &Snapshot, bundle: &Bundle, request: &Value) -> Result<Index> {
    let version = match request["target_index_schema_version"].as_u64() {
        Some(1) => "forge.workspace/1",
        Some(2) => "forge.workspace/2",
        _ => return Err(Error::invalid()),
    };
    if version == "forge.workspace/1"
        && (bundle.index.schema_version != version
            || (snapshot.index_present && snapshot.index.schema_version != version))
    {
        return Err(Error::invalid());
    }
    let mut proposed = bundle.index.clone();
    proposed.schema_version = version.into();
    Index::parse(&proposed.bytes()?)
}

/// Reject an impossible complete path union before any incoming file is opened.
/// Exact shared paths deduplicate; portable aliases remain invalid rather than
/// receiving physical-count credit. Captured identity checks still follow.
fn admit_capture_paths(snapshot: &Snapshot, bundle: &Bundle) -> Result<()> {
    let mut paths = Vec::<&str>::new();
    if snapshot.index_present {
        paths.push(INDEX_PATH);
    }
    for registration in snapshot.index.resources.iter().chain(&bundle.index.resources) {
        let path = registration.path.as_str();
        if paths.iter().any(|old| *old != path && old.eq_ignore_ascii_case(path)) {
            return Err(Error::containment());
        }
        if !paths.contains(&path) {
            if paths.len() >= MAX_FILES {
                return Err(too_many_inputs());
            }
            paths.push(path);
        }
    }
    Ok(())
}

/// Preserve intrinsic registration admission and its exact applicability-report fallback.
/// The fallback compares current captured analysis; other limited roles gain no freshness proof.
fn admitted_resource(snapshot: &Snapshot, registration: &Resource, captured: &Captured) -> bool {
    validate_bytes(registration, &captured.bytes)
        || (registration.role == Role::ApplicabilityReport
            && snapshot.analysis.as_ref().is_some_and(|analysis| {
                serde_json::to_value(analysis).ok()
                    == contract::parse(&captured.bytes, MAX_RESOURCE_BYTES, 64 * 1024).ok()
            }))
}

/// Read every new explicit input and preserve typed interruption around admission.
fn incoming_bindings(
    root: &Root,
    snapshot: &Snapshot,
    bundle: &Bundle,
    bindings: &mut Bindings,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    for (registration, pin) in bundle.index.resources.iter().zip(&bundle.pins) {
        let remaining = bindings.before_read(&registration.path)?;
        control.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
        let captured = match bindings.captured(&registration.path) {
            Some(existing) => existing.clone(),
            None => root.read(&registration.path, remaining.min(MAX_RESOURCE_BYTES))?,
        };
        control.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
        if captured.sha256 != pin.sha256 || captured.bytes.len() != pin.size_bytes {
            return Err(validation_error().into());
        }
        control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
        let valid = admitted_resource(snapshot, registration, &captured);
        control.checkpoint(Stage::ValidateResource, ProgressUpdate::Unchanged)?;
        if !valid {
            return Err(validation_error().into());
        }
        bindings.insert(&registration.path, &captured)?;
        bindings.pin(registration, &captured)?;
    }
    for item in &snapshot.items {
        bindings.pin(&item.registration, &item.captured)?;
    }
    Ok(())
}

/// Bind exact index absence/presence and physical instance before replacing it.
fn index_target(root: &Root, snapshot: &Snapshot) -> Result<Target> {
    let limit = snapshot.captured_index().map_or(MAX_INDEX_BYTES, |captured| captured.bytes.len());
    let target = root.target_with_limit(INDEX_PATH, limit)?;
    match (snapshot.captured_index(), target.base.as_ref()) {
        (None, None) if !snapshot.index_present => {}
        (Some(old), Some(base)) if snapshot.index_present => same_capture(old, base)?,
        _ => return Err(conflict()),
    }
    Ok(target)
}

/// Produce complete old/new membership and ordered removals, not a truncated diff.
fn replacement(
    snapshot: &Snapshot,
    bundle: &Bundle,
    proposed: &Index,
    consumed: usize,
) -> Result<Value> {
    let previous = if snapshot.index_present {
        serde_json::to_value(&snapshot.index).map_err(|_| Error::invalid())?
    } else {
        Value::Null
    };
    let removed = snapshot
        .index
        .resources
        .iter()
        .filter(|old| !proposed.resources.iter().any(|new| new.key == old.key))
        .map(|old| old.key.clone())
        .collect::<Vec<_>>();
    Ok(json!({"previous_index":previous,"proposed_index":proposed,
        "supplied_index_sha256":bundle.index_sha256,
        "proposed_index_sha256":crate::hashing::sha256_hex(&proposed.bytes()?),
        "removed_resource_keys":removed,"consumed_file_count":consumed}))
}

/// Prepare an explicit exact index replacement without retaining or publishing it.
///
/// Incoming source files remain unchanged. The complete private/public bindings
/// and replacement projection are atomically admitted by the root-owned Store;
/// confirmation still rechecks all inputs and publishes only this one index.
pub(crate) fn prepare_import(
    root: &Root,
    snapshot: &Snapshot,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedBundle> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "ProjectBundleImportRequest")?;
    let bundle = bundles::decode_bundle_for_api(&request["bundle"], ApiMajor::V2)?;
    let proposed = proposed_index(snapshot, &bundle, request)?;
    admit_capture_paths(snapshot, &bundle)?;
    let mut bindings = current_bindings(snapshot)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let target = index_target(root, snapshot)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    incoming_bindings(root, snapshot, &bundle, &mut bindings, control)?;
    let bytes = proposed.bytes()?;
    let projection = replacement(snapshot, &bundle, &proposed, bindings.files.len())?;
    control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
    Ok(PreparedBundle {
        target,
        kind: "workspace-index-update",
        bytes,
        snapshot_version: snapshot.version.clone(),
        input_hashes: bindings.pins,
        consumed_file_count: bindings.files.len(),
        external: bindings.files,
        semantic_summary: "Replace the complete ordered resource index and project label with the explicit selected metadata bundle. Removed registrations become unregistered. Files are neither hydrated nor deleted; domain decisions and freshness remain in their own contracts.".into(),
        artifact_family: None,
        replacement: Some(projection),
    })
}

#[cfg(test)]
mod tests {
    use super::super::preparation::test_support::Recorder;
    use super::super::preparation::{Interruption, NoopControl, WorkError};
    use super::*;

    /// Actual bounded files, explicit optional index and captured production view.
    struct Fixture {
        /// Keeps every synthetic input alive for sentinel preservation assertions.
        project: tempfile::TempDir,
        /// Descriptor-confined root used by the real capture and producer paths.
        root: Root,
        /// Complete current registration capture, absent when the index is absent.
        snapshot: Snapshot,
        /// Candidate incoming index, including files not yet registered.
        incoming: Index,
    }

    /// Create valid Markdown inputs and an optional closed authorial index.
    fn fixture(count: usize, present: bool, version: &str) -> Fixture {
        let project = tempfile::tempdir().expect("bounded synthetic project");
        let mut incoming = Index::empty();
        incoming.schema_version = version.into();
        incoming.label = "Sensitive authored label".into();
        for n in 0..count {
            let path = format!("policy-{n:04}.md");
            std::fs::write(
                project.path().join(&path),
                format!("# Policy {n}\n\n## Rules\nPRIVATE SOURCE SENTINEL {n}\n"),
            )
            .expect("write actual Markdown input");
            incoming.resources.push(Resource {
                key: format!("policy-{n:04}"),
                role: Role::PolicySource,
                path,
            });
        }
        if present {
            std::fs::write(project.path().join(INDEX_PATH), incoming.bytes().unwrap())
                .expect("write explicit index");
        }
        let root = Root::open(project.path()).expect("confined root");
        let snapshot = Snapshot::capture_for_api(&root, ApiMajor::V2).expect("actual capture");
        Fixture { project, root, snapshot, incoming }
    }

    /// Build exact complete metadata pins from the real confined fixture files.
    fn bundle(fixture: &Fixture, index: &Index) -> Value {
        let pins = index.resources.iter().map(|registration| {
            let captured = fixture.root.read(&registration.path, MAX_RESOURCE_BYTES).unwrap();
            json!({"key":registration.key,"sha256":captured.sha256,"size_bytes":captured.bytes.len()})
        }).collect::<Vec<_>>();
        json!({"schema_version":if index.schema_version == "forge.workspace/1" {
            "forge.workspace-index-bundle/1"
        } else { "forge.workspace-index-bundle/2" },
            "content_profile":"index-and-hashes", "index":index,
            "index_sha256":crate::hashing::sha256_hex(&index.bytes().unwrap()), "pins":pins})
    }

    /// Construct the exact numeric-selector import wire shape, with explicit acknowledgment.
    fn import_request(bundle: &Value, target: u64) -> Value {
        json!({"bundle":bundle,"target_index_schema_version":target,"acknowledge_index_replacement":true})
    }

    /// Construct the closed sensitivity-acknowledged metadata export request.
    fn export_request(path: &str) -> Value {
        json!({"target_path":path,"acknowledge_sensitive_metadata":true})
    }

    /// Assert a safe typed failure, excluding successful or interrupted preparation.
    fn expect_failure(result: WorkResult<PreparedBundle>, code: &str) {
        let error = match result {
            Err(WorkError::Failed(error)) => error,
            Err(WorkError::Interrupted(_)) => panic!("unexpected interruption"),
            Ok(_) => panic!("unexpected retained plan"),
        };
        assert_eq!(error.code, code);
        assert!(!error.message.contains("PRIVATE SOURCE SENTINEL"));
    }

    /// Observe exact source/index bytes for preparation's no-write assertions.
    fn source_bytes(fixture: &Fixture) -> Vec<(String, Vec<u8>)> {
        fixture
            .incoming
            .resources
            .iter()
            .map(|resource| {
                (
                    resource.path.clone(),
                    std::fs::read(fixture.project.path().join(&resource.path)).unwrap(),
                )
            })
            .chain(fixture.snapshot.index_present.then(|| {
                (INDEX_PATH.into(), std::fs::read(fixture.project.path().join(INDEX_PATH)).unwrap())
            }))
            .collect()
    }

    /// Check every previously observed input remains byte-exact after a plan or refusal.
    fn unchanged(fixture: &Fixture, before: &[(String, Vec<u8>)]) {
        for (path, bytes) in before {
            assert_eq!(&std::fs::read(fixture.project.path().join(path)).unwrap(), bytes);
        }
    }

    /// Capture a real framework/scope analysis and an unregistered serialized report.
    fn applicability_fixture() -> Fixture {
        let project = tempfile::tempdir().unwrap();
        let catalog = json!({"catalog":{"uuid":"11111111-1111-4111-8111-111111111111",
            "metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},
            "controls":[{"id":"control-a","title":"Synthetic control"}]}});
        std::fs::write(
            project.path().join("framework.json"),
            serde_json::to_vec(&catalog).unwrap(),
        )
        .unwrap();
        let mut incoming = Index::empty();
        incoming.resources.push(Resource {
            key: "framework".into(),
            role: Role::OscalCatalogArtifact,
            path: "framework.json".into(),
        });
        std::fs::write(project.path().join(INDEX_PATH), incoming.bytes().unwrap()).unwrap();
        let root = Root::open(project.path()).unwrap();
        let initial = Snapshot::capture_for_api(&root, ApiMajor::V2).unwrap();
        let scope = super::super::domain::initialize(&initial,
            &json!({"target_path":"scope.json","framework_resource_id":resource_id(&incoming.resources[0])}),false).unwrap();
        std::fs::write(project.path().join("scope.json"), scope).unwrap();
        incoming.resources.push(Resource {
            key: "scope".into(),
            role: Role::ApplicabilityManifest,
            path: "scope.json".into(),
        });
        std::fs::write(project.path().join(INDEX_PATH), incoming.bytes().unwrap()).unwrap();
        let snapshot = Snapshot::capture_for_api(&root, ApiMajor::V2).unwrap();
        let bytes =
            serde_json::to_vec(snapshot.analysis.as_ref().expect("actual current analysis"))
                .unwrap();
        std::fs::write(project.path().join("report.json"), bytes).unwrap();
        incoming.resources.push(Resource {
            key: "report".into(),
            role: Role::ApplicabilityReport,
            path: "report.json".into(),
        });
        Fixture { project, root, snapshot, incoming }
    }

    /// Build five real bounded opaque captures leaving less than 8KiB union room.
    fn near_capture_limit_fixture() -> Fixture {
        use std::io::Write as _;
        let project = tempfile::tempdir().unwrap();
        let mut incoming = Index::empty();
        incoming.schema_version = "forge.workspace/2".into();
        for n in 0..5 {
            let path = format!("opaque-{n}.bin");
            let mut file = std::fs::File::create(project.path().join(&path)).unwrap();
            let mut remaining = MAX_RESOURCE_BYTES - 1024;
            let block = [b'x'; 4096];
            while remaining != 0 {
                let size = remaining.min(block.len());
                file.write_all(&block[..size]).unwrap();
                remaining -= size;
            }
            incoming.resources.push(Resource {
                key: format!("opaque-{n}"),
                role: Role::LifecycleSource,
                path,
            });
        }
        std::fs::write(project.path().join(INDEX_PATH), incoming.bytes().unwrap()).unwrap();
        let root = Root::open(project.path()).unwrap();
        let snapshot = Snapshot::capture_for_api(&root, ApiMajor::V2).unwrap();
        Fixture { project, root, snapshot, incoming }
    }

    /// Exact export contains sensitive metadata and hashes, while source bytes remain excluded.
    #[test]
    fn export_binds_complete_metadata_without_writing_or_hydrating() {
        let fixture = fixture(2, true, "forge.workspace/1");
        let before = source_bytes(&fixture);
        let plan = prepare_export(
            &fixture.root,
            &fixture.snapshot,
            &export_request("bundle.json"),
            &mut NoopControl,
        )
        .unwrap();
        let value: Value = serde_json::from_slice(&plan.bytes).unwrap();
        assert_eq!(value["index"]["label"], "Sensitive authored label");
        assert_eq!(value["pins"].as_array().unwrap().len(), 2);
        assert_eq!(plan.input_hashes.len(), 2);
        assert_eq!(plan.consumed_file_count, 3);
        assert_eq!(plan.external.len(), 3);
        assert!(matches!(plan.artifact_family, Some(ArtifactFamily::MetadataJson)));
        assert!(plan.replacement.is_none());
        assert!(!std::str::from_utf8(&plan.bytes).unwrap().contains("PRIVATE SOURCE SENTINEL"));
        assert!(plan.semantic_summary.contains("may be sensitive"));
        assert!(!fixture.project.path().join("bundle.json").exists());
        unchanged(&fixture, &before);
    }

    /// Sensitivity acknowledgment never grants index or registered-source overwrite authority.
    #[test]
    fn export_preserves_reserved_and_registered_destination_guard() {
        let fixture = fixture(1, true, "forge.workspace/1");
        let before = source_bytes(&fixture);
        for path in [INDEX_PATH, "FORGE.WORKSPACE.JSON", "policy-0000.md", "POLICY-0000.md"] {
            expect_failure(
                prepare_export(
                    &fixture.root,
                    &fixture.snapshot,
                    &export_request(path),
                    &mut NoopControl,
                ),
                "resource-containment",
            );
        }
        unchanged(&fixture, &before);
    }

    /// Complete replacement preserves order and separates supplied and migrated normalized hashes.
    #[test]
    fn import_explicit_migration_projects_complete_membership_and_removed_order() {
        let fixture = fixture(3, true, "forge.workspace/1");
        let before = source_bytes(&fixture);
        let mut incoming = fixture.incoming.clone();
        incoming.resources = vec![incoming.resources[2].clone()];
        let request = import_request(&bundle(&fixture, &incoming), 2);
        let plan =
            prepare_import(&fixture.root, &fixture.snapshot, &request, &mut NoopControl).unwrap();
        let replacement = plan.replacement.as_ref().unwrap();
        assert_eq!(replacement["previous_index"], serde_json::to_value(&fixture.incoming).unwrap());
        assert_eq!(
            replacement["proposed_index"]["resources"],
            request["bundle"]["index"]["resources"]
        );
        assert_eq!(replacement["proposed_index"]["schema_version"], "forge.workspace/2");
        assert_eq!(replacement["removed_resource_keys"], json!(["policy-0000", "policy-0001"]));
        assert_eq!(replacement["supplied_index_sha256"], request["bundle"]["index_sha256"]);
        assert_eq!(replacement["proposed_index_sha256"], crate::hashing::sha256_hex(&plan.bytes));
        assert_ne!(replacement["supplied_index_sha256"], replacement["proposed_index_sha256"]);
        assert_eq!(replacement["consumed_file_count"], 4);
        assert_eq!(plan.input_hashes.len(), 3);
        assert_eq!(plan.input_hashes[0]["resource_id"], resource_id(&incoming.resources[0]));
        assert!(plan.artifact_family.is_none());
        unchanged(&fixture, &before);
    }

    /// A current index2 cannot be downgraded even by a structurally valid bundle1.
    #[test]
    fn current_or_supplied_index2_downgrade_is_refused_before_incoming_reads() {
        let fixture = fixture(0, true, "forge.workspace/2");
        let mut incoming = Index::empty();
        incoming.resources.push(Resource {
            key: "missing".into(),
            role: Role::PolicySource,
            path: "missing.md".into(),
        });
        let value = json!({"schema_version":"forge.workspace-index-bundle/1","content_profile":"index-and-hashes",
            "index":incoming,"index_sha256":crate::hashing::sha256_hex(&incoming.bytes().unwrap()),
            "pins":[{"key":"missing","sha256":"a".repeat(64),"size_bytes":1}]});
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&value, 1),
                &mut NoopControl,
            ),
            "invalid-request",
        );
        let value = bundle(&fixture, &fixture.incoming);
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&value, 1),
                &mut NoopControl,
            ),
            "invalid-request",
        );
    }

    /// Hash/length agreement and intrinsic role admission are independent mandatory checks.
    #[test]
    fn import_rejects_wrong_pin_facts_and_wrong_declared_native_role() {
        let fixture = fixture(1, false, "forge.workspace/1");
        let before = source_bytes(&fixture);
        let original = bundle(&fixture, &fixture.incoming);
        for field in ["sha256", "size_bytes"] {
            let mut value = original.clone();
            value["pins"][0][field] =
                if field == "sha256" { json!("a".repeat(64)) } else { json!(1) };
            expect_failure(
                prepare_import(
                    &fixture.root,
                    &fixture.snapshot,
                    &import_request(&value, 1),
                    &mut NoopControl,
                ),
                "validation-failed",
            );
        }
        let mut index = fixture.incoming.clone();
        index.resources[0].role = Role::OscalCatalogArtifact;
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&bundle(&fixture, &index), 1),
                &mut NoopControl,
            ),
            "validation-failed",
        );
        unchanged(&fixture, &before);
        assert!(!fixture.project.path().join(INDEX_PATH).exists());
    }

    /// An absent index and explicit empty bundle has a real zero-file replacement projection.
    #[test]
    fn absent_empty_index_projects_null_previous_and_zero_consumed_files() {
        let fixture = fixture(0, false, "forge.workspace/1");
        let plan = prepare_import(
            &fixture.root,
            &fixture.snapshot,
            &import_request(&bundle(&fixture, &fixture.incoming), 1),
            &mut NoopControl,
        )
        .unwrap();
        assert!(plan.target.base.is_none());
        assert_eq!(plan.consumed_file_count, 0);
        assert!(plan.external.is_empty());
        assert!(plan.input_hashes.is_empty());
        let replacement = plan.replacement.unwrap();
        assert!(replacement["previous_index"].is_null());
        assert_eq!(replacement["removed_resource_keys"], json!([]));
        assert_eq!(replacement["consumed_file_count"], 0);
        assert!(!fixture.project.path().join(INDEX_PATH).exists());
    }

    /// The current raw index and a distinct overwrite base each count toward the whole cap.
    #[test]
    fn present_index_and_existing_export_base_are_not_hidden_input_allowances() {
        let current = fixture(99, true, "forge.workspace/1");
        let plan = prepare_export(
            &current.root,
            &current.snapshot,
            &export_request("bundle.json"),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(plan.consumed_file_count, 100);
        std::fs::write(
            current.project.path().join("bundle.json"),
            b"existing destination sentinel",
        )
        .unwrap();
        expect_failure(
            prepare_export(
                &current.root,
                &current.snapshot,
                &export_request("bundle.json"),
                &mut NoopControl,
            ),
            "invalid-request",
        );
        assert_eq!(
            std::fs::read(current.project.path().join("bundle.json")).unwrap(),
            b"existing destination sentinel"
        );
        let fixture = fixture(100, true, "forge.workspace/1");
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&bundle(&fixture, &fixture.incoming), 1),
                &mut NoopControl,
            ),
            "invalid-request",
        );
    }

    /// A full hundred incoming files fits only an absent index and no other consumed file.
    #[test]
    fn whole_incoming_hundred_is_admitted_without_a_prefix() {
        let current = fixture(100, false, "forge.workspace/1");
        let request = import_request(&bundle(&current, &current.incoming), 1);
        let plan =
            prepare_import(&current.root, &current.snapshot, &request, &mut NoopControl).unwrap();
        assert_eq!(plan.consumed_file_count, 100);
        assert_eq!(plan.external.len(), 100);
        assert_eq!(plan.input_hashes.len(), 100);
        assert_eq!(
            plan.replacement.as_ref().unwrap()["proposed_index"]["resources"]
                .as_array()
                .unwrap()
                .len(),
            100
        );
        assert!(!current.project.path().join(INDEX_PATH).exists());
        let overflow = fixture(101, false, "forge.workspace/1");
        let mut control = Recorder::default();
        expect_failure(
            prepare_import(
                &overflow.root,
                &overflow.snapshot,
                &import_request(&bundle(&overflow, &overflow.incoming), 1),
                &mut control,
            ),
            "invalid-request",
        );
        assert!(!control.events.iter().any(|(stage, _)| *stage == Stage::CaptureResource));
        assert!(!overflow.project.path().join(INDEX_PATH).exists());
    }

    /// A renamed registration retains one physical capture and both honest public IDs.
    #[test]
    fn same_path_replacement_deduplicates_physical_files_but_preserves_public_binding() {
        let fixture = fixture(1, true, "forge.workspace/1");
        let mut incoming = fixture.incoming.clone();
        incoming.resources[0].key = "renamed".into();
        let plan = prepare_import(
            &fixture.root,
            &fixture.snapshot,
            &import_request(&bundle(&fixture, &incoming), 1),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(plan.consumed_file_count, 2);
        assert_eq!(plan.external.len(), 2);
        assert_eq!(plan.input_hashes.len(), 2);
        assert_eq!(plan.input_hashes[0]["resource_id"], resource_id(&incoming.resources[0]));
        assert_eq!(
            plan.input_hashes[1]["resource_id"],
            resource_id(&fixture.incoming.resources[0])
        );
        assert_eq!(plan.replacement.unwrap()["removed_resource_keys"], json!(["policy-0000"]));
    }

    /// Equal raw index bytes cannot conceal replacement of the captured file instance.
    #[test]
    fn changed_raw_index_instance_refuses_preparation_without_changing_cursor_version() {
        let fixture = fixture(1, true, "forge.workspace/1");
        let bytes = std::fs::read(fixture.project.path().join(INDEX_PATH)).unwrap();
        let replacement = fixture.project.path().join("replacement.json");
        std::fs::write(&replacement, &bytes).unwrap();
        std::fs::remove_file(fixture.project.path().join(INDEX_PATH)).unwrap();
        std::fs::rename(replacement, fixture.project.path().join(INDEX_PATH)).unwrap();
        let current = Snapshot::capture_for_api(&fixture.root, ApiMajor::V2).unwrap();
        assert_eq!(current.version, fixture.snapshot.version);
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&bundle(&fixture, &fixture.incoming), 1),
                &mut NoopControl,
            ),
            "version-conflict",
        );
        assert_eq!(std::fs::read(fixture.project.path().join(INDEX_PATH)).unwrap(), bytes);
    }

    /// Portable aliases across current/incoming indexes fail before opening the new spelling.
    #[test]
    fn incoming_alias_and_interruption_preserve_typed_failure_and_original_files() {
        let fixture = fixture(1, true, "forge.workspace/1");
        let before = source_bytes(&fixture);
        let mut value = bundle(&fixture, &fixture.incoming);
        value["index"]["resources"][0]["path"] = json!("POLICY-0000.md");
        let index: Index = serde_json::from_value(value["index"].clone()).unwrap();
        value["index_sha256"] = json!(crate::hashing::sha256_hex(&index.bytes().unwrap()));
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&value, 1),
                &mut NoopControl,
            ),
            "resource-containment",
        );
        let mut control = Recorder::at(Stage::CaptureResource, 1);
        let result = prepare_import(
            &fixture.root,
            &fixture.snapshot,
            &import_request(&bundle(&fixture, &fixture.incoming), 1),
            &mut control,
        );
        assert!(matches!(result, Err(WorkError::Interrupted(Interruption::CancelRequested))));
        assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        unchanged(&fixture, &before);
    }

    /// Closed direct-value admission retains acknowledgment and unknown-field refusal.
    #[test]
    fn closed_requests_refuse_extra_fields_false_acknowledgment_and_string_selector() {
        let fixture = fixture(0, true, "forge.workspace/1");
        let mut request = export_request("bundle.json");
        request["private"] = json!("PRIVATE SOURCE SENTINEL");
        expect_failure(
            prepare_export(&fixture.root, &fixture.snapshot, &request, &mut NoopControl),
            "invalid-request",
        );
        let mut request = import_request(&bundle(&fixture, &fixture.incoming), 1);
        request["acknowledge_index_replacement"] = json!(false);
        expect_failure(
            prepare_import(&fixture.root, &fixture.snapshot, &request, &mut NoopControl),
            "invalid-request",
        );
        request["acknowledge_index_replacement"] = json!(true);
        request["target_index_schema_version"] = json!("forge.workspace/1");
        expect_failure(
            prepare_import(&fixture.root, &fixture.snapshot, &request, &mut NoopControl),
            "invalid-request",
        );
    }

    /// The captured analysis report is admitted without extending that exception to other roles.
    #[test]
    fn applicability_report_uses_existing_admission_and_exact_current_analysis_relation() {
        let fixture = applicability_fixture();
        let original = source_bytes(&fixture);
        let report = fixture.incoming.resources.last().unwrap();
        let captured = fixture.root.read(&report.path, MAX_RESOURCE_BYTES).unwrap();
        assert!(admitted_resource(&fixture.snapshot, report, &captured));
        let plan = prepare_import(
            &fixture.root,
            &fixture.snapshot,
            &import_request(&bundle(&fixture, &fixture.incoming), 1),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(plan.consumed_file_count, 4);
        assert_eq!(plan.input_hashes.len(), 3);
        let mut invalid: Value = serde_json::from_slice(&captured.bytes).unwrap();
        invalid["unexpected_private_field"] = json!("PRIVATE SOURCE SENTINEL");
        std::fs::write(
            fixture.project.path().join("report.json"),
            serde_json::to_vec(&invalid).unwrap(),
        )
        .unwrap();
        expect_failure(
            prepare_import(
                &fixture.root,
                &fixture.snapshot,
                &import_request(&bundle(&fixture, &fixture.incoming), 1),
                &mut NoopControl,
            ),
            "validation-failed",
        );
        let other = Resource {
            key: report.key.clone(),
            role: Role::FrameworkImpactReport,
            path: report.path.clone(),
        };
        assert!(!admitted_resource(&fixture.snapshot, &other, &captured));
        let preserved =
            original.into_iter().filter(|(path, _)| path != "report.json").collect::<Vec<_>>();
        unchanged(&fixture, &preserved);
    }

    /// Destination capture refuses the remaining whole-union limit with the platform's preserved typed error.
    #[test]
    fn existing_export_destination_cannot_exceed_remaining_capture_budget() {
        let fixture = near_capture_limit_fixture();
        let bindings = current_bindings(&fixture.snapshot).unwrap();
        assert!(MAX_CAPTURE_BYTES - bindings.bytes < 8192);
        let path = fixture.project.path().join("bundle.json");
        std::fs::write(&path, vec![b'y'; 8192]).unwrap();
        let code = if cfg!(windows) { "resource-containment" } else { "payload-too-large" };
        expect_failure(
            prepare_export(
                &fixture.root,
                &fixture.snapshot,
                &export_request("bundle.json"),
                &mut NoopControl,
            ),
            code,
        );
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 8192);
        assert_eq!(std::fs::read(path).unwrap(), vec![b'y'; 8192]);
    }
}
