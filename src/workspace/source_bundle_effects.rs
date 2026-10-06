//! Off-Store exact source export and complete source-restore preparation.
//!
//! The caller owns strict original-body admission, selected API2.3 authority,
//! the absolute preparation deadline, complete pre-capture path admission and
//! final atomic Store retention. This module never writes or accepts a journal
//! intent; new bytes remain inert until complete proposed-closure admission.

use serde_json::{Value, json};

use super::actions;
use super::bundle_effects::{self, PreparedBundle};
use super::contract::{self, ApiMajor, Error, Result};
use super::effects::ArtifactFamily;
use super::index::{INDEX_PATH, Index};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::root::{RestoreTargetPlan, Root, conflict};
use super::services::{self, Snapshot, resource_id};
use super::source_bundles::{self, DecodedSourceBundle};

/// Whole admitted project-file path union, including an absent index target.
const MAX_FILES: usize = 100;
/// One complete conservative old-and-new generation budget, not two captures.
const MAX_CAPTURE_BYTES: usize = 50 * 1024 * 1024;
/// Existing complete safe request/response encoding ceiling.
const MAX_WIRE_BYTES: usize = 1024 * 1024;
/// Existing total text-diff budget across all target rows.
const MAX_DIFF_BYTES: usize = 200_000;

/// Complete private unretained source plan consumed atomically by ROOT's Store.
pub(crate) struct PreparedSourceRestore {
    /// Preallocated nonauthorizing lookup ID retained before receipt confirmation.
    pub(crate) operation_id: String,
    /// Original raw-current snapshot version, without normalization or rewriting.
    pub(crate) snapshot_version: String,
    /// Hash of all current binding/target-version/directory-parent observations.
    pub(crate) observed_batch_version: String,
    /// Hash of the exact complete target/binding/directory intent manifest.
    pub(crate) exact_manifest_sha256: String,
    /// Explicit normalized proposed index; no downgrade or inferred merge.
    pub(crate) proposed_index: Index,
    /// Complete sanitized authorial target rows, with the index target last.
    pub(crate) targets: Vec<Value>,
    /// Complete current/proposed file facts including explicit absence.
    pub(crate) input_bindings: Vec<Value>,
    /// Complete parent-before-child new-directory intentions, never hidden mkdir.
    pub(crate) directories: Vec<Value>,
    /// Structured proposed-closure admission, not freshness or human approval.
    pub(crate) validation: Value,
    /// Complete replacement objects/hashes/removals/planned-path accounting.
    pub(crate) replacement: Value,
    /// Exact distinct admitted current/incoming/base path union, without prefixing.
    pub(crate) consumed_file_count: usize,
    /// Checked opaque private charge; Store additionally charges full reply/preview.
    pub(crate) reserved_private_bytes: usize,
    /// Server-authored sensitivity and external-reader boundary disclosure.
    pub(crate) semantic_summary: String,
    /// Sealed native owned handles, current generations and exact proposed bytes.
    pub(crate) plan: RestoreTargetPlan,
}

/// Return a fixed whole-input limit error without a private path or source excerpt.
fn payload_limit() -> Error {
    Error::new("payload-too-large", "The complete source restore exceeds its input limit.", false)
}

/// Validate every new public field before deriving an internal metadata request.
fn admit_request(request: &Value, schema: &str) -> Result<()> {
    let bytes = contract::encode(request, MAX_WIRE_BYTES, false)?;
    let value = contract::parse(&bytes, MAX_WIRE_BYTES, 64 * 1024)?;
    contract::validate_for(ApiMajor::V2, schema, &value)
}

/// Preserve existing guarded export targets and bindings, replacing only media/bytes.
///
/// The internally authored old request confers no new public permission: the
/// selected-major caller and both public source acknowledgments must admit first.
/// Returned bytes remain uncommitted until the existing single-file receipt flow.
pub(crate) fn prepare_export(
    root: &Root,
    snapshot: &Snapshot,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedBundle> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "PrepareProjectSourceBundleExportRequest")?;
    let path = request["target_path"].as_str().ok_or_else(Error::invalid)?;
    let planned_count = planned_export_paths(snapshot, path)?.len();
    admit_snapshot(snapshot)?;
    let bytes = source_bundles::encode(snapshot, control)?;
    let internal = json!({"target_path":path,"acknowledge_sensitive_metadata":true});
    let mut plan = bundle_effects::prepare_export(root, snapshot, &internal, control)?;
    plan.bytes = bytes;
    plan.consumed_file_count = planned_count;
    plan.artifact_family = Some(ArtifactFamily::SourceBundleJson);
    plan.semantic_summary = "Export the complete explicit resource index and exact registered source bytes. Source content, labels, keys, roles, relative paths, hashes and byte lengths are intentionally included and may be sensitive. No blanket redaction, native validity, current freshness or human authority is inferred. The finite artifact must remain reimportable under the unchanged request cap. The source planning limit reserves the selected output path even when absent, plus index and current paths; a distinct output and present index permit at most 98 registered resources.".into();
    control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
    Ok(plan)
}

/// Reserve the source output even absent before encoding or delegated target I/O.
/// ROOT consumes the same complete source-only planning rule before capture;
/// old metadata effects retain their existing physical-file admission semantics.
fn planned_export_paths(snapshot: &Snapshot, path: &str) -> Result<Vec<String>> {
    actions::output_target(snapshot, path, None)?;
    let mut paths = planned_paths(&snapshot.index, &snapshot.index)?;
    if !paths.contains(&path.to_owned()) {
        if paths.len() >= MAX_FILES {
            return Err(payload_limit());
        }
        paths.push(path.to_owned());
    }
    Ok(paths)
}

/// Choose only explicit index preservation or legacy1-to2 migration; refuse downgrade.
fn proposed_index(snapshot: &Snapshot, supplied: &Index, request: &Value) -> Result<Index> {
    let selected = match request["target_index_schema_version"].as_u64() {
        Some(1) => "forge.workspace/1",
        Some(2) => "forge.workspace/2",
        _ => return Err(Error::invalid()),
    };
    if selected == "forge.workspace/1"
        && (supplied.schema_version != selected
            || (snapshot.index_present && snapshot.index.schema_version != selected))
    {
        return Err(Error::invalid());
    }
    let mut proposed = supplied.clone();
    selected.clone_into(&mut proposed.schema_version);
    Index::parse(&proposed.bytes()?)
}

/// Admit every complete planned path, including absent index, before target work.
/// Runtime must consume the same rule before Snapshot capture; this is defense.
fn planned_paths(current: &Index, proposed: &Index) -> Result<Vec<String>> {
    let mut paths = vec![INDEX_PATH.to_owned()];
    for resource in current.resources.iter().chain(&proposed.resources) {
        if paths.iter().any(|old| old != &resource.path && old.eq_ignore_ascii_case(&resource.path))
        {
            return Err(Error::containment());
        }
        if !paths.contains(&resource.path) {
            if paths.len() >= MAX_FILES {
                return Err(payload_limit());
            }
            paths.push(resource.path.clone());
        }
    }
    Ok(paths)
}

/// Recheck actual snapshot membership/raw index, identity aliases and capture sum.
fn admit_snapshot(snapshot: &Snapshot) -> Result<usize> {
    let expected = snapshot.index.bytes()?;
    let mut bytes = 0_usize;
    let mut identities = Vec::new();
    match (snapshot.index_present, snapshot.captured_index()) {
        (true, Some(captured)) => {
            if Index::parse(&captured.bytes)?.bytes()? != expected {
                return Err(conflict());
            }
            bytes = captured.bytes.len();
            identities.push(captured.identity);
        }
        (false, None) if snapshot.index.resources.is_empty() && snapshot.items.is_empty() => {}
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
        if identities.contains(&item.captured.identity) {
            return Err(Error::containment());
        }
        identities.push(item.captured.identity);
        bytes = bytes.checked_add(item.captured.bytes.len()).ok_or_else(payload_limit)?;
        if bytes > MAX_CAPTURE_BYTES {
            return Err(payload_limit());
        }
    }
    Ok(bytes)
}

/// Compare the caller's inert decode with this admitted Value, preserving exact bytes.
/// Original strict raw parsing belongs to HTTP; no normalized raw-input hash is invented.
fn same_decoded(
    request: &Value,
    decoded: &DecodedSourceBundle,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    let bytes =
        contract::encode(&request["bundle"], source_bundles::MAX_SOURCE_BUNDLE_BYTES, false)?;
    let repeated = source_bundles::decode(&bytes, control)?;
    if repeated.index.bytes()? != decoded.index.bytes()?
        || repeated.index_sha256 != decoded.index_sha256
        || repeated.files.len() != decoded.files.len()
    {
        return Err(Error::invalid().into());
    }
    for (left, right) in repeated.files.iter().zip(&decoded.files) {
        if left.key != right.key
            || left.sha256 != right.sha256
            || left.size_bytes != right.size_bytes
            || left.bytes != right.bytes
        {
            return Err(Error::invalid().into());
        }
    }
    Ok(())
}

/// Count exact old and proposed generations conservatively before plan allocation.
fn admit_generations(
    current_bytes: usize,
    index_bytes: usize,
    decoded: &DecodedSourceBundle,
) -> Result<()> {
    let mut total = current_bytes.checked_add(index_bytes).ok_or_else(payload_limit)?;
    for file in &decoded.files {
        total = total.checked_add(file.bytes.len()).ok_or_else(payload_limit)?;
        if total > MAX_CAPTURE_BYTES {
            return Err(payload_limit());
        }
    }
    if total > MAX_CAPTURE_BYTES {
        return Err(payload_limit());
    }
    Ok(())
}

/// Preserve complete prior registered facts before moving their actual captures.
fn prior_bindings(snapshot: &Snapshot) -> Vec<Value> {
    let mut prior = Vec::new();
    if let Some(captured) = snapshot.captured_index() {
        prior.push(json!({"path":INDEX_PATH,"kind":"index","current":{
            "key":null,"role":null,"resource_id":null,"sha256":captured.sha256,"size":captured.bytes.len()}}));
    }
    for item in &snapshot.items {
        prior.push(json!({"path":item.registration.path,"kind":"resource","current":{
            "key":item.registration.key,"role":item.registration.role,
            "resource_id":resource_id(&item.registration),"sha256":item.captured.sha256,
            "size":item.captured.bytes.len()}}));
    }
    prior
}

/// Bind every current/proposed path; unregistered target bases remain explicitly unlabeled.
fn input_bindings(paths: &[String], prior: &[Value], targets: &[Value]) -> Result<Vec<Value>> {
    let mut bindings = Vec::new();
    for path in paths {
        let target = targets.iter().find(|row| row["path"] == *path);
        if path == INDEX_PATH
            && !prior.iter().any(|row| row["path"] == INDEX_PATH)
            && target.is_some_and(|row| row["status"] == "overwrite")
        {
            return Err(conflict());
        }
        let current = if let Some(old) = prior.iter().find(|row| row["path"] == *path) {
            old["current"].clone()
        } else if let Some(row) = target.filter(|row| row["status"] == "overwrite") {
            json!({"key":null,"role":null,"resource_id":null,
                "sha256":row["base_sha256"],"size":row["base_size"]})
        } else {
            Value::Null
        };
        let proposed = target.map_or(Value::Null, |row| {
            json!({"key":row["key"],"role":row["role"],
            "sha256":row["exact_bytes_sha256"],"size":row["size"]})
        });
        let kind = if path == INDEX_PATH { "index" } else { "resource" };
        let row = json!({"path":path,"kind":kind,"current":current,"proposed":proposed});
        contract::validate_for(ApiMajor::V2, "BatchBinding", &row)?;
        bindings.push(row);
    }
    Ok(bindings)
}

/// Match public bindings against the opaque plan's complete exact-byte observations.
fn plan_binding_views(plan: &RestoreTargetPlan, bindings: &[Value]) -> Result<()> {
    let observed = plan.input_views();
    if observed.len() != bindings.len() {
        return Err(Error::invalid());
    }
    for row in bindings {
        let path = row["path"].as_str().ok_or_else(Error::invalid)?;
        let matching = observed.iter().filter(|item| item["path"] == path).collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(Error::invalid());
        }
        let facts = matching[0];
        if facts["current_sha256"] != row["current"]["sha256"]
            || facts["current_size"] != row["current"]["size"]
            || facts["proposed_sha256"] != row["proposed"]["sha256"]
            || facts["proposed_size"] != row["proposed"]["size"]
        {
            return Err(conflict());
        }
    }
    Ok(())
}

/// Construct closed complete target rows from sealed plan facts, rejecting omissions.
fn target_views(
    plan: &RestoreTargetPlan,
    proposed: &Index,
    exact: &[(String, String, usize)],
) -> Result<Vec<Value>> {
    let views = plan.target_views();
    if views.len() != exact.len() {
        return Err(Error::invalid());
    }
    let mut targets = Vec::new();
    let mut diff_bytes = 0_usize;
    for (view, (path, hash, size)) in views.iter().zip(exact) {
        if view["path"] != *path
            || view["exact_bytes_sha256"] != *hash
            || view["size"].as_u64().and_then(|n| usize::try_from(n).ok()) != Some(*size)
        {
            return Err(Error::invalid());
        }
        let registration = proposed.resources.iter().find(|row| &row.path == path);
        let kind = if path == INDEX_PATH { "index" } else { "resource" };
        if (kind == "index") != registration.is_none() {
            return Err(Error::invalid());
        }
        let row = json!({"path":path,"kind":kind,
            "key":registration.map(|item|&item.key),"role":registration.map(|item|&item.role),
            "status":view["status"],"base_sha256":view["base_sha256"],"base_size":view["base_size"],
            "target_version":view["target_version"],"exact_bytes_sha256":hash,"size":size,
            "diff_text":view["diff_text"],"diff_truncated":view["diff_truncated"],"binary":view["binary"]});
        contract::validate_for(ApiMajor::V2, "BatchTarget", &row)?;
        diff_bytes = diff_bytes
            .checked_add(row["diff_text"].as_str().ok_or_else(Error::invalid)?.len())
            .ok_or_else(payload_limit)?;
        if diff_bytes > MAX_DIFF_BYTES {
            return Err(payload_limit());
        }
        targets.push(row);
    }
    Ok(targets)
}

/// Validate every explicit parent-first directory row rather than hiding later mkdir.
fn directory_views(plan: &RestoreTargetPlan) -> Result<Vec<Value>> {
    let views = plan.directory_views();
    if views.len() > MAX_FILES {
        return Err(payload_limit());
    }
    for (position, row) in views.iter().enumerate() {
        contract::validate_for(ApiMajor::V2, "BatchDirectory", row)?;
        let path = row["path"].as_str().ok_or_else(Error::invalid)?;
        if views[..position].iter().any(|old| old["path"] == path) {
            return Err(Error::invalid());
        }
        if views[position + 1..].iter().any(|later| {
            later["path"]
                .as_str()
                .is_some_and(|later_path| path.starts_with(&format!("{later_path}/")))
        }) {
            return Err(Error::invalid());
        }
    }
    Ok(views)
}

/// Render exact whole-index replacement and removed keys, without deleting old files.
fn replacement(
    previous: Option<&Index>,
    proposed: &Index,
    supplied_hash: &str,
    consumed: usize,
) -> Result<Value> {
    let removed = previous.map_or_else(Vec::new, |old| {
        old.resources
            .iter()
            .filter(|row| !proposed.resources.iter().any(|new| new.key == row.key))
            .map(|row| row.key.clone())
            .collect::<Vec<_>>()
    });
    let value = json!({"previous_index":previous,"proposed_index":proposed,
        "supplied_index_sha256":supplied_hash,"proposed_index_sha256":crate::hashing::sha256_hex(&proposed.bytes()?),
        "removed_resource_keys":removed,"consumed_file_count":consumed});
    contract::validate_for(ApiMajor::V2, "SourceBundleIndexReplacement", &value)?;
    Ok(value)
}

/// Hash checked finite canonical intent objects, never pretending this is raw body provenance.
fn manifest_hash(value: &Value) -> Result<String> {
    Ok(crate::hashing::sha256_hex(&contract::encode(value, MAX_WIRE_BYTES, false)?))
}

/// Move exact decoded resources into native proposed targets, with normalized index last.
fn proposed_targets(
    proposed: &Index,
    decoded: DecodedSourceBundle,
    index_bytes: Vec<u8>,
) -> Result<Vec<(String, Vec<u8>, String)>> {
    if proposed.resources.len() != decoded.files.len() {
        return Err(Error::invalid());
    }
    let mut targets = Vec::new();
    for (registration, file) in proposed.resources.iter().zip(decoded.files) {
        if registration.key != file.key
            || file.size_bytes != file.bytes.len()
            || file.sha256 != crate::hashing::sha256_hex(&file.bytes)
        {
            return Err(Error::invalid());
        }
        targets.push((registration.path.clone(), file.bytes, file.sha256));
    }
    let hash = crate::hashing::sha256_hex(&index_bytes);
    targets.push((INDEX_PATH.into(), index_bytes, hash));
    Ok(targets)
}

/// Prepare complete source restore off Store, retaining no receipt or accepted intent.
///
/// ROOT must capture only current files/present bases after whole-path admission;
/// incoming absent files and new directories are explicit plan intentions. Native
/// plan ownership and proposed closure validation are consumed, not reconstructed.
pub(crate) fn prepare_import(
    root: &Root,
    snapshot: Snapshot,
    decoded: DecodedSourceBundle,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedSourceRestore> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "PrepareProjectSourceBundleImportRequest")?;
    same_decoded(request, &decoded, control)?;
    prepare_import_admitted(root, snapshot, decoded, request, control)
}

/// Share the unchanged native preparation core after each transport's own strict admission.
/// This is not a public codec bypass: only admitted inline or staged producers call it.
fn prepare_import_admitted(
    root: &Root,
    snapshot: Snapshot,
    decoded: DecodedSourceBundle,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedSourceRestore> {
    let proposed = proposed_index(&snapshot, &decoded.index, request)?;
    let paths = planned_paths(&snapshot.index, &proposed)?;
    let old_bytes = admit_snapshot(&snapshot)?;
    let index_bytes = proposed.bytes()?;
    admit_generations(old_bytes, index_bytes.len(), &decoded)?;
    let source_refs = decoded
        .files
        .iter()
        .map(|file| (file.key.as_str(), file.bytes.as_slice()))
        .collect::<Vec<_>>();
    services::validate_proposed_sources(&proposed, &source_refs, control)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let previous = snapshot.index_present.then(|| snapshot.index.clone());
    let projection = replacement(previous.as_ref(), &proposed, &decoded.index_sha256, paths.len())?;
    let snapshot_version = snapshot.version.clone();
    let prior = prior_bindings(&snapshot);
    let targets = proposed_targets(&proposed, decoded, index_bytes)?;
    let exact = targets
        .iter()
        .map(|(path, bytes, hash)| (path.clone(), hash.clone(), bytes.len()))
        .collect::<Vec<_>>();
    let actual_inputs = snapshot.into_captured_inputs();
    let plan = root.plan_restore_targets(targets, actual_inputs, control)?;
    let targets = target_views(&plan, &proposed, &exact)?;
    let directories = directory_views(&plan)?;
    let input_bindings = input_bindings(&paths, &prior, &targets)?;
    plan_binding_views(&plan, &input_bindings)?;
    let operation_id = format!("op_{}", *super::session::random_token()?);
    let target_versions = targets
        .iter()
        .map(|row| json!({"path":row["path"],"target_version":row["target_version"]}))
        .collect::<Vec<_>>();
    let observed_batch_version = manifest_hash(&json!({"snapshot_version":snapshot_version,
        "input_bindings":input_bindings,"target_versions":target_versions,"directories":directories}))?;
    let intent_targets = targets.iter().map(|row|json!({"path":row["path"],
        "kind":row["kind"],"key":row["key"],"role":row["role"],"status":row["status"],
        "base_sha256":row["base_sha256"],"base_size":row["base_size"],"target_version":row["target_version"],
        "sha256":row["exact_bytes_sha256"],"size":row["size"]})).collect::<Vec<_>>();
    let exact_manifest_sha256 = manifest_hash(
        &json!({"schema_version":"forge.workspace-source-restore-manifest/1",
        "operation_id":operation_id,"snapshot_version":snapshot_version,"observed_batch_version":observed_batch_version,
        "proposed_index_sha256":projection["proposed_index_sha256"],"input_bindings":input_bindings,
        "directories":directories,"targets":intent_targets}),
    )?;
    control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
    Ok(PreparedSourceRestore {
        operation_id, snapshot_version, observed_batch_version, exact_manifest_sha256,
        proposed_index: proposed, targets, input_bindings, directories,
        validation: services::validation(true, None), replacement: projection,
        consumed_file_count: paths.len(), reserved_private_bytes: plan.reserved_private_bytes(),
        semantic_summary: "Replace the complete selected resource index and each explicitly listed file with the exact source bundle bytes. Source content and metadata may be sensitive. Removed registrations do not delete their files. Native intrinsic/registered proposed-closure admission is not human approval or universal freshness. Previewed directories and owned rollback/recovery are required; external CLI/editor readers can see mixed whole-file generations. No write or durable accepted intent exists until this separate receipt is confirmed.".into(),
        plan,
    })
}

/// Prepare Bundle4 with the original exact output guard and complete source planning rule.
/// The caller admits selected2.4 and holds the same participating project read lease.
pub(crate) fn prepare_staged_export(
    root: &Root,
    snapshot: &Snapshot,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedBundle> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "PrepareProjectSourceBundleExportRequest")?;
    let path = request["target_path"].as_str().ok_or_else(Error::invalid)?;
    let planned_count = planned_export_paths(snapshot, path)?.len();
    admit_snapshot(snapshot)?;
    let bytes = super::staged_source_bundles::encode(snapshot, control)?;
    let internal = json!({"target_path":path,"acknowledge_sensitive_metadata":true});
    let mut plan = bundle_effects::prepare_export(root, snapshot, &internal, control)?;
    plan.bytes = bytes;
    plan.consumed_file_count = planned_count;
    plan.artifact_family = Some(ArtifactFamily::StagedSourceBundleJson);
    plan.semantic_summary = "Export the complete explicit index and exact registered source bytes as the staged Bundle4 profile. Source content and metadata are intentionally included and may be sensitive. The output path occupies one of the unchanged whole100 planned paths even when absent. This profile has a10MiB raw artifact ceiling but complete shared retention may refuse smaller artifacts. No write exists before the existing export receipt confirmation; manifest/part reads require the exact committed generation. No blanket redaction, native validity, freshness or human authority is inferred.".into();
    control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
    Ok(plan)
}

/// Prepare only a genuine decoded staged artifact under the unchanged native closure/plan core.
/// ROOT passes the exact consumed lease decode; no original raw hash is invented from a Value.
pub(crate) fn prepare_staged_import(
    root: &Root,
    snapshot: Snapshot,
    decoded: DecodedSourceBundle,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<PreparedSourceRestore> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    admit_request(request, "PrepareStagedSourceRestoreRequest")?;
    if decoded.index_sha256 != crate::hashing::sha256_hex(&decoded.index.bytes()?) {
        return Err(Error::invalid().into());
    }
    prepare_import_admitted(root, snapshot, decoded, request, control)
}

/// Authored unexecuted controls using actual bounded fixture files and consumed ports.
#[cfg(test)]
mod tests {
    use super::super::index::{Resource, Role};
    use super::super::preparation::test_support::Recorder;
    use super::super::preparation::{Interruption, NoopControl, WorkError};
    use super::*;

    /// Actual synthetic confined project with complete source/index sentinels.
    struct Fixture {
        /// Owns every actual synthetic file until all byte-preservation assertions.
        project: tempfile::TempDir,
        /// Existing descriptor-anchored Root, never a forged Captured fixture.
        root: Root,
        /// Ordered explicit proposed registrations admitted by the real Index parser.
        index: Index,
    }

    /// Build valid Markdown and optionally present original index in a private project.
    fn fixture(count: usize, present: bool, major_two: bool) -> Fixture {
        let project = tempfile::tempdir().unwrap();
        let mut index = Index::empty();
        if major_two {
            index.schema_version = "forge.workspace/2".into();
        }
        for number in 0..count {
            let path = format!("policy-{number:03}.md");
            std::fs::write(
                project.path().join(&path),
                format!("# Policy {number}\n\n## Rule\nSOURCE PRIVATE SENTINEL {number}\n"),
            )
            .unwrap();
            index.resources.push(Resource {
                key: format!("policy-{number:03}"),
                role: Role::PolicySource,
                path,
            });
        }
        if present {
            std::fs::write(project.path().join(INDEX_PATH), index.bytes().unwrap()).unwrap();
        }
        let root = Root::open(project.path()).unwrap();
        Fixture { project, root, index }
    }

    /// Capture an actual selected-major snapshot using the retained production wrapper.
    fn capture(fixture: &Fixture) -> Snapshot {
        Snapshot::capture_for_api(&fixture.root, ApiMajor::V2).unwrap()
    }

    /// Build admitted source bundle bytes from exact real registered capture.
    fn bundle(fixture: &Fixture) -> Value {
        let bytes = source_bundles::encode(&capture(fixture), &mut NoopControl).unwrap();
        contract::parse(&bytes, source_bundles::MAX_SOURCE_BUNDLE_BYTES, 64 * 1024).unwrap()
    }

    /// Use the real strict source decoder for an inert proposed generation.
    fn decoded(value: &Value) -> DecodedSourceBundle {
        let bytes =
            contract::encode(value, source_bundles::MAX_SOURCE_BUNDLE_BYTES, false).unwrap();
        source_bundles::decode(&bytes, &mut NoopControl).unwrap()
    }

    /// Compose exact explicit confirmation-free preparation fields.
    fn request(bundle: &Value, selected: u64) -> Value {
        json!({"bundle":bundle,"target_index_schema_version":selected,
            "acknowledge_index_replacement":true,"acknowledge_source_content":true,
            "acknowledge_replace_files":true})
    }

    /// Observe every registered byte plus the original index before any preparation.
    fn sentinel(fixture: &Fixture) -> Vec<(String, Vec<u8>)> {
        let mut names =
            fixture.index.resources.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
        if fixture.project.path().join(INDEX_PATH).exists() {
            names.push(INDEX_PATH.into());
        }
        names
            .into_iter()
            .map(|path| {
                let bytes = std::fs::read(fixture.project.path().join(&path)).unwrap();
                (path, bytes)
            })
            .collect()
    }

    /// Check no current input/index was changed by a successful or refused preparation.
    fn unchanged(fixture: &Fixture, before: &[(String, Vec<u8>)]) {
        for (path, bytes) in before {
            assert_eq!(&std::fs::read(fixture.project.path().join(path)).unwrap(), bytes);
        }
    }

    /// Require a fixed safe failure without swallowing controlled interruption.
    fn failure<T>(result: WorkResult<T>, code: &str) {
        let error = match result {
            Err(WorkError::Failed(error)) => error,
            Err(WorkError::Interrupted(_)) => panic!("unexpected interruption"),
            Ok(_) => panic!("unexpected successful preparation"),
        };
        assert_eq!(error.code, code);
        assert!(!error.message.contains("SOURCE PRIVATE SENTINEL"));
    }

    /// Source generation choices use complete union with a reserved index target.
    #[test]
    fn planned_index_absence_and_hundredth_path_are_counted() {
        let initial = fixture(99, true, false);
        let paths = planned_paths(&Index::empty(), &initial.index).unwrap();
        assert_eq!(paths.len(), 100);
        assert_eq!(paths[0], INDEX_PATH);
        let over = fixture(100, true, false);
        assert_eq!(
            planned_paths(&Index::empty(), &over.index).unwrap_err().code,
            "payload-too-large"
        );
    }

    /// Full current-only inputs cannot disappear merely because replacement drops them.
    #[test]
    fn dropped_registration_and_alias_paths_remain_whole_inputs() {
        let current = fixture(99, true, false);
        let mut incoming = Index::empty();
        incoming.resources.push(Resource {
            key: "new-policy".into(),
            role: Role::PolicySource,
            path: "new-policy.md".into(),
        });
        assert_eq!(planned_paths(&current.index, &incoming).unwrap_err().code, "payload-too-large");
        incoming.resources[0].path = "POLICY-000.md".into();
        assert_eq!(
            planned_paths(&current.index, &incoming).unwrap_err().code,
            "resource-containment"
        );
        incoming.resources[0] = current.index.resources[0].clone();
        assert_eq!(planned_paths(&current.index, &incoming).unwrap().len(), 100);
    }

    /// Exact selector preservation/upgrade cannot turn current2 or incoming2 into1.
    #[test]
    fn explicit_selector_preserves_or_upgrades_and_never_downgrades() {
        let legacy = fixture(1, true, false);
        let snapshot = capture(&legacy);
        let selected = request(&bundle(&legacy), 2);
        assert_eq!(
            proposed_index(&snapshot, &legacy.index, &selected).unwrap().schema_version,
            "forge.workspace/2"
        );
        let modern = fixture(1, true, true);
        let wrong = request(&bundle(&modern), 1);
        assert!(proposed_index(&capture(&modern), &modern.index, &wrong).is_err());
        assert!(proposed_index(&capture(&modern), &legacy.index, &wrong).is_err());
    }

    /// A direct internal caller cannot pair admitted request bytes with another decode.
    #[test]
    fn different_inert_decode_is_refused_before_native_plan() {
        let first = fixture(1, true, false);
        let second = fixture(2, true, false);
        let value = request(&bundle(&first), 2);
        assert!(same_decoded(&value, &decoded(&bundle(&second)), &mut NoopControl).is_err());
        assert!(same_decoded(&value, &decoded(&value["bundle"]), &mut NoopControl).is_ok());
    }

    /// A same-path replacement still counts old and new byte generations together.
    #[test]
    fn conservative_generation_sum_has_one_checked_ceiling() {
        let source = fixture(1, true, false);
        let supplied = decoded(&bundle(&source));
        let new_bytes = supplied.files.iter().map(|file| file.bytes.len()).sum::<usize>();
        assert!(admit_generations(MAX_CAPTURE_BYTES - new_bytes - 1, 1, &supplied).is_ok());
        assert_eq!(
            admit_generations(MAX_CAPTURE_BYTES - new_bytes, 1, &supplied).unwrap_err().code,
            "payload-too-large"
        );
        assert_eq!(
            admit_generations(usize::MAX, 1, &supplied).unwrap_err().code,
            "payload-too-large"
        );
    }

    /// Snapshot raw index and membership tampering is not accepted as current authority.
    #[test]
    fn raw_current_index_and_member_capture_relation_is_rechecked() {
        let project = fixture(1, true, false);
        let mut snapshot = capture(&project);
        snapshot.index.label = "Different normalization".into();
        assert_eq!(admit_snapshot(&snapshot).unwrap_err().code, "version-conflict");
        let mut snapshot = capture(&project);
        snapshot.items[0].registration.key = "wrong-key".into();
        assert_eq!(admit_snapshot(&snapshot).unwrap_err().code, "invalid-request");
    }

    /// Public source acknowledgments are not inferred from internally authored metadata fields.
    #[test]
    fn source_acknowledgment_failure_does_not_open_export_destination() {
        let project = fixture(1, true, false);
        let before = sentinel(&project);
        let request = json!({"target_path":"exports/source.json",
            "acknowledge_sensitive_metadata":true,"acknowledge_source_content":false});
        failure(
            prepare_export(&project.root, &capture(&project), &request, &mut NoopControl),
            "invalid-request",
        );
        assert!(!project.project.path().join("exports").exists());
        unchanged(&project, &before);
    }

    /// Preparation preserves typed stop and cannot proceed with destination work.
    #[test]
    fn controlled_preparation_interruption_remains_typed_and_nonwriting() {
        let project = fixture(1, true, false);
        let before = sentinel(&project);
        let value = request(&bundle(&project), 2);
        let mut control = Recorder::at(Stage::PrepareDomain, 1);
        assert!(matches!(
            prepare_import(
                &project.root,
                capture(&project),
                decoded(&value["bundle"]),
                &value,
                &mut control
            ),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        unchanged(&project, &before);
    }

    /// Export grammar does not substitute for proposed native role admission on import.
    #[test]
    fn empty_policy_bytes_refuse_import_before_plan_and_preserve_current_bytes() {
        let source = fixture(1, true, false);
        std::fs::write(source.project.path().join("policy-000.md"), b"").unwrap();
        let before = sentinel(&source);
        let value = request(&bundle(&source), 2);
        failure(
            prepare_import(
                &source.root,
                capture(&source),
                decoded(&value["bundle"]),
                &value,
                &mut NoopControl,
            ),
            "validation-failed",
        );
        unchanged(&source, &before);
    }

    /// Source export uses the existing guarded single-file plan with exact source bytes.
    #[test]
    fn source_export_private_family_and_exact_bytes_preserve_inputs() {
        let project = fixture(1, true, false);
        let before = sentinel(&project);
        let request = json!({"target_path":"source.json","acknowledge_sensitive_metadata":true,"acknowledge_source_content":true});
        let plan =
            prepare_export(&project.root, &capture(&project), &request, &mut NoopControl).unwrap();
        assert!(matches!(plan.artifact_family, Some(ArtifactFamily::SourceBundleJson)));
        assert_eq!(plan.kind, "report-export");
        let value =
            contract::parse(&plan.bytes, source_bundles::MAX_SOURCE_BUNDLE_BYTES, 64 * 1024)
                .unwrap();
        assert_eq!(decoded(&value).files[0].bytes, before[0].1);
        assert!(!project.project.path().join("source.json").exists());
        unchanged(&project, &before);
    }

    /// A reserved absent source output plus raw index admits exactly 98 registrations.
    #[test]
    fn source_export_reserves_absent_output_at_exact_whole_limit() {
        let project = fixture(98, true, false);
        let before = sentinel(&project);
        let request = json!({"target_path":"source.json","acknowledge_sensitive_metadata":true,"acknowledge_source_content":true});
        let plan =
            prepare_export(&project.root, &capture(&project), &request, &mut NoopControl).unwrap();
        assert_eq!(plan.consumed_file_count, 100);
        assert_eq!(plan.external.len(), 99);
        assert!(!project.project.path().join("source.json").exists());
        unchanged(&project, &before);
    }

    /// An over-limit source output directory is refused before delegated target reads.
    #[test]
    fn source_export_whole_overflow_precedes_target_directory_read() {
        let project = fixture(99, true, false);
        std::fs::create_dir(project.project.path().join("source.json")).unwrap();
        let before = sentinel(&project);
        let request = json!({"target_path":"source.json","acknowledge_sensitive_metadata":true,"acknowledge_source_content":true});
        failure(
            prepare_export(&project.root, &capture(&project), &request, &mut NoopControl),
            "payload-too-large",
        );
        assert!(project.project.path().join("source.json").is_dir());
        unchanged(&project, &before);
    }

    /// Native plan preparation on qualified Unix ports remains uncommitted and has full pins.
    #[cfg(unix)]
    #[test]
    fn complete_source_restore_plan_has_preknown_id_and_index_last() {
        let source = fixture(1, true, false);
        let before = sentinel(&source);
        let value = request(&bundle(&source), 2);
        let plan = prepare_import(
            &source.root,
            capture(&source),
            decoded(&value["bundle"]),
            &value,
            &mut NoopControl,
        )
        .unwrap();
        assert!(plan.operation_id.starts_with("op_"));
        assert_eq!(plan.operation_id.len(), 67);
        assert_eq!(plan.proposed_index.schema_version, "forge.workspace/2");
        assert_eq!(plan.targets.len(), 2);
        assert_eq!(plan.targets.last().unwrap()["path"], INDEX_PATH);
        assert_eq!(plan.input_bindings.len(), 2);
        assert_eq!(plan.consumed_file_count, 2);
        assert_eq!(plan.replacement["consumed_file_count"], 2);
        assert_eq!(plan.replacement["proposed_index"]["schema_version"], "forge.workspace/2");
        assert_eq!(plan.exact_manifest_sha256.len(), 64);
        assert_eq!(plan.observed_batch_version.len(), 64);
        assert!(plan.reserved_private_bytes > 0);
        unchanged(&source, &before);
    }

    /// New target bytes are restored from the supplied inert closure, without fake current identity.
    #[cfg(unix)]
    #[test]
    fn absent_resource_and_index_targets_are_explicit_and_nonwriting() {
        let source = fixture(1, true, false);
        let destination = fixture(0, false, false);
        let value = request(&bundle(&source), 2);
        let plan = prepare_import(
            &destination.root,
            capture(&destination),
            decoded(&value["bundle"]),
            &value,
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(plan.targets.len(), 2);
        assert!(plan.targets.iter().all(|row| row["status"] == "create"));
        assert!(plan.input_bindings.iter().all(|row| row["current"].is_null()));
        assert_eq!(plan.replacement["previous_index"], Value::Null);
        assert_eq!(plan.consumed_file_count, 2);
        assert!(!destination.project.path().join(INDEX_PATH).exists());
        assert!(!destination.project.path().join("policy-000.md").exists());
    }

    /// A foreign index appearing after an absent snapshot is not silently adopted as a base.
    #[cfg(unix)]
    #[test]
    fn appeared_index_refuses_prepare_and_preserves_foreign_generation() {
        let source = fixture(1, true, false);
        let destination = fixture(0, false, false);
        let absent = capture(&destination);
        let foreign = Index::empty().bytes().unwrap();
        std::fs::write(destination.project.path().join(INDEX_PATH), &foreign).unwrap();
        let value = request(&bundle(&source), 2);
        failure(
            prepare_import(
                &destination.root,
                absent,
                decoded(&value["bundle"]),
                &value,
                &mut NoopControl,
            ),
            "version-conflict",
        );
        assert_eq!(std::fs::read(destination.project.path().join(INDEX_PATH)).unwrap(), foreign);
        assert!(!destination.project.path().join("policy-000.md").exists());
    }

    /// Existing unregistered destination bytes are a real private base, not a registration claim.
    #[cfg(unix)]
    #[test]
    fn unregistered_target_base_is_disclosed_without_fabricated_registration() {
        let source = fixture(1, true, false);
        let destination = fixture(0, false, false);
        let old = b"# Existing unrelated policy\n\n## Rule\nDo not delete or publish yet.\n";
        std::fs::write(destination.project.path().join("policy-000.md"), old).unwrap();
        let value = request(&bundle(&source), 2);
        let plan = prepare_import(
            &destination.root,
            capture(&destination),
            decoded(&value["bundle"]),
            &value,
            &mut NoopControl,
        )
        .unwrap();
        let row = plan.input_bindings.iter().find(|row| row["path"] == "policy-000.md").unwrap();
        assert_eq!(row["current"]["sha256"], crate::hashing::sha256_hex(old));
        assert!(row["current"]["key"].is_null());
        assert!(row["current"]["role"].is_null());
        assert!(row["current"]["resource_id"].is_null());
        assert_eq!(row["proposed"]["key"], "policy-000");
        assert_eq!(std::fs::read(destination.project.path().join("policy-000.md")).unwrap(), old);
        assert!(!destination.project.path().join(INDEX_PATH).exists());
    }
}
