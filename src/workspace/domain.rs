//! Reuse domain engines over a private, confined input snapshot.
//!
//! Domain engines receive only captured registered files. All path-bearing
//! manifest references are checked before any engine is called. The staging
//! directory is temporary and never publishes a project artifact.

use std::path::Path;

use super::contract::{Error, Result};
use super::index::{Role, validate_path};
use super::preparation::{NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::services::{Item, Snapshot};

fn invalid() -> Error {
    Error::new(
        "validation-failed",
        "The selected domain inputs are missing, ambiguous, invalid, or stale.",
        false,
    )
}

/// The workspace never narrows a registered manifest below the byte bound its
/// own domain parser enforces.
pub(crate) fn manifest_limit(mapping: bool) -> usize {
    let limit = if mapping {
        crate::mapping::manifest::MAX_MANIFEST_BYTES
    } else {
        crate::applicability::manifest::MAX_MANIFEST_BYTES
    };
    usize::try_from(limit).unwrap_or(usize::MAX)
}

pub(crate) fn selected(snapshot: &Snapshot, role: Role) -> Result<&Item> {
    let mut matching = snapshot.items.iter().filter(|item| item.registration.role == role);
    let item = matching.next().ok_or_else(invalid)?;
    if matching.next().is_some() {
        return Err(invalid());
    }
    Ok(item)
}

/// Resolve lexical relative references without allowing escape or OS aliases.
pub(crate) fn resolve_reference(manifest: &str, reference: &Path) -> Result<String> {
    let reference = reference.to_str().ok_or_else(Error::containment)?;
    // Parse the portable spelling before native Path components can normalize
    // Windows backslashes or interior dot segments into apparently safe names.
    if reference.contains('\\') {
        return Err(Error::containment());
    }
    let mut parts: Vec<_> = manifest.split('/').collect();
    parts.pop();
    for segment in reference.split('/') {
        if segment == ".." {
            if parts.pop().is_none() {
                return Err(Error::containment());
            }
        } else {
            validate_path(segment)?;
            parts.push(segment);
        }
    }
    let path = parts.join("/");
    validate_path(&path)?;
    Ok(path)
}

fn registered_reference(snapshot: &Snapshot, manifest: &str, reference: &Path) -> Result<()> {
    let path = resolve_reference(manifest, reference)?;
    if snapshot.items.iter().any(|item| item.registration.path == path) {
        Ok(())
    } else {
        Err(invalid())
    }
}

fn check_resource(
    snapshot: &Snapshot,
    manifest: &str,
    resource: &crate::mapping::manifest::ResourceManifest,
) -> Result<()> {
    registered_reference(snapshot, manifest, &resource.artifact)?;
    if let Some(companion) = &resource.resolved_catalog {
        registered_reference(snapshot, manifest, companion)?;
    }
    Ok(())
}

/// Copy captured registrations for an ordinary query or direct effect without
/// creating operation progress or reacquiring the shared operation-store lock.
pub(crate) fn stage(snapshot: &Snapshot) -> Result<tempfile::TempDir> {
    stage_with_control(snapshot, &mut NoopControl).map_err(WorkError::into_error)
}

/// Copy every captured registration into a private temporary directory, checking
/// before allocation and before/after each file copy. An interrupted copy drops
/// the directory; a write already in progress is not forcibly interrupted.
pub(crate) fn stage_with_control(
    snapshot: &Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<tempfile::TempDir> {
    control.checkpoint(Stage::CopyInputs, ProgressUpdate::Clear)?;
    let stage = tempfile::tempdir().map_err(|_| invalid())?;
    copy_to_stage(snapshot, stage, control)
}

/// Consume temporary-directory ownership while copying captured bytes. Keeping
/// ownership in this production helper ensures every error/interruption drops
/// the partial stage, and the controlled engine receives only a complete stage.
fn copy_to_stage(
    snapshot: &Snapshot,
    stage: tempfile::TempDir,
    control: &mut dyn WorkControl,
) -> WorkResult<tempfile::TempDir> {
    for item in &snapshot.items {
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        let path = stage.path().join(&item.registration.path);
        let parent = path.parent().ok_or_else(invalid)?;
        let copied = (|| {
            std::fs::create_dir_all(parent).map_err(|_| invalid())?;
            std::fs::write(&path, &item.captured.bytes).map_err(|_| invalid())
        })();
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        copied?;
    }
    Ok(stage)
}

/// Analyze a captured snapshot for existing synchronous validation callers.
pub(crate) fn analyze(
    snapshot: &Snapshot,
) -> Result<crate::applicability::model::ApplicabilityReport> {
    analyze_with_control(snapshot, &mut NoopControl).map_err(WorkError::into_error)
}

/// Validate registered references, stage the complete capture and bracket the
/// shared applicability engine. Internal interruptions remain distinct from a
/// normal invalid manifest and cannot become best-effort snapshot invalidity.
pub(crate) fn analyze_with_control(
    snapshot: &Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<crate::applicability::model::ApplicabilityReport> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let manifest = selected(snapshot, Role::ApplicabilityManifest);
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let manifest = manifest?;
    let parsed =
        crate::applicability::manifest::parse(&manifest.captured.bytes).map_err(|_| invalid());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let parsed = parsed?;
    check_resource(snapshot, &manifest.registration.path, &parsed.framework)?;
    for reference in &parsed.mapping_collections {
        registered_reference(snapshot, &manifest.registration.path, reference)?;
    }
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let stage = stage_with_control(snapshot, control)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let prepared = crate::applicability::prepare_analysis(
        &stage.path().join(&manifest.registration.path),
        crate::applicability::model::ReportFilters::default(),
    )
    .map_err(|_| invalid());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    Ok(prepared?.report)
}

pub(crate) fn mapping_manifest(snapshot: &Snapshot) -> Result<&Item> {
    let mut matches = snapshot.items.iter().filter(|item| {
        item.registration.role == Role::MappingCollection
            && crate::mapping::manifest::parse(&item.captured.bytes).is_ok()
    });
    let selected = matches.next().ok_or_else(invalid)?;
    if matches.next().is_some() {
        return Err(invalid());
    }
    Ok(selected)
}

/// Build mappings for current synchronous queries and draft validation.
pub(crate) fn mapping(snapshot: &Snapshot) -> Result<crate::mapping::PreparedBuild> {
    mapping_with_control(snapshot, &mut NoopControl).map_err(WorkError::into_error)
}

/// Build a mapping from checked captured references and a cooperatively copied
/// stage. The existing engine is checked on entry and return, not preempted.
pub(crate) fn mapping_with_control(
    snapshot: &Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<crate::mapping::PreparedBuild> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let item = mapping_manifest(snapshot);
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let item = item?;
    let manifest = crate::mapping::manifest::parse(&item.captured.bytes).map_err(|_| invalid());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let manifest = manifest?;
    check_resource(snapshot, &item.registration.path, &manifest.mapping.source)?;
    check_resource(snapshot, &item.registration.path, &manifest.mapping.target)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let stage = stage_with_control(snapshot, control)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let built = crate::mapping::prepare(&stage.path().join(&item.registration.path), None, false)
        .map_err(|_| invalid());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    Ok(built?)
}

/// The single registered decision manifest for `role`. Absence is setup state:
/// the browser initializes only when the draft endpoint reports `not-found`
/// (HTTP 404), so a missing manifest must not be reported as a generic
/// validation failure. An ambiguous or malformed registration stays a
/// validation failure.
fn registered_manifest(snapshot: &Snapshot, role: Role) -> Result<&Item> {
    let mut matching = snapshot.items.iter().filter(|item| item.registration.role == role);
    let item = matching.next().ok_or_else(not_found_manifest)?;
    if matching.next().is_some() {
        return Err(invalid());
    }
    Ok(item)
}

fn not_found_manifest() -> Error {
    Error::new("not-found", "The decision manifest is not registered in this project.", false)
}

pub(crate) fn draft(snapshot: &Snapshot, mapping: bool) -> Result<serde_json::Value> {
    let item = if mapping {
        // A registered but malformed mapping collection is not setup state; only
        // an unregistered one is.
        if !snapshot.items.iter().any(|item| item.registration.role == Role::MappingCollection) {
            return Err(not_found_manifest());
        }
        mapping_manifest(snapshot)?
    } else {
        registered_manifest(snapshot, Role::ApplicabilityManifest)?
    };
    let manifest =
        super::contract::parse(&item.captured.bytes, manifest_limit(mapping), 64 * 1024)?;
    Ok(serde_json::json!({"version":item.metadata["version"],"manifest":manifest,
        "validation_summary":{"state":item.validation["state"],"error_count":item.validation["error_count"],"warning_count":item.validation["warning_count"]}}))
}

/// Validate a complete proposed draft against the same captured dependencies.
pub(crate) fn validate_draft(
    snapshot: &mut Snapshot,
    mapping_draft: bool,
    value: &serde_json::Value,
) -> Result<serde_json::Value> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid())?;
    if bytes.len() > manifest_limit(mapping_draft) {
        return Err(invalid());
    }
    let item = if mapping_draft {
        mapping_manifest(snapshot)?
    } else {
        selected(snapshot, Role::ApplicabilityManifest)?
    };
    let path = item.registration.path.clone();
    let index = snapshot
        .items
        .iter()
        .position(|item| item.registration.path == path)
        .ok_or_else(invalid)?;
    let old_hash = std::mem::replace(
        &mut snapshot.items[index].captured.sha256,
        crate::hashing::sha256_hex(&bytes),
    );
    let old_bytes = std::mem::replace(&mut snapshot.items[index].captured.bytes, bytes);
    let valid = if mapping_draft { mapping(snapshot).is_ok() } else { analyze(snapshot).is_ok() };
    snapshot.items[index].captured.bytes = old_bytes;
    snapshot.items[index].captured.sha256 = old_hash;
    Ok(super::services::validation(
        valid,
        Some(&super::services::resource_id(&snapshot.items[index].registration)),
    ))
}

pub(crate) fn subjects(snapshot: &Snapshot) -> Result<Vec<serde_json::Value>> {
    Ok(subject_inventory(snapshot)?.rows)
}

/// Before a draft exists, both the resource and its side must be explicitly selected.
pub(crate) fn subjects_for(
    snapshot: &Snapshot,
    resource_id: Option<&str>,
    side: Option<&str>,
) -> Result<Vec<serde_json::Value>> {
    Ok(inventory(snapshot, resource_id, side)?.rows)
}

/// Bounded subject rows plus the resources whose identifiers exceeded the
/// contract's display-label bound. The opaque `subject_id` and `fingerprint`
/// carry the exact identity, so a truncated label loses no evidence.
pub(crate) struct SubjectInventory {
    pub rows: Vec<serde_json::Value>,
    /// `(resource_id, truncated subject count)` per supplying resource.
    pub truncated: Vec<(String, usize)>,
}

/// Compute subject rows for the existing synchronous inventory view.
pub(crate) fn subject_inventory(snapshot: &Snapshot) -> Result<SubjectInventory> {
    subject_inventory_with_control(snapshot, &mut NoopControl).map_err(WorkError::into_error)
}

/// Compute derived subject rows with controlled temporary copies and checks
/// around each shared staged-input load, preserving internal interruption.
pub(crate) fn subject_inventory_with_control(
    snapshot: &Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<SubjectInventory> {
    inventory_with_control(snapshot, None, None, control)
}

/// Serve the existing explicit resource/side inventory selection synchronously.
fn inventory(
    snapshot: &Snapshot,
    resource_id: Option<&str>,
    side: Option<&str>,
) -> Result<SubjectInventory> {
    inventory_with_control(snapshot, resource_id, side, &mut NoopControl)
        .map_err(WorkError::into_error)
}

/// Stage only captured registrations, then compute a bounded subject inventory
/// for either the registered mapping or an explicit existing view selection.
fn inventory_with_control(
    snapshot: &Snapshot,
    resource_id: Option<&str>,
    side: Option<&str>,
    control: &mut dyn WorkControl,
) -> WorkResult<SubjectInventory> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let stage = stage_with_control(snapshot, control)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let mut output = Vec::new();
    let mut truncated = Vec::new();
    let declaration = mapping_manifest(snapshot);
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if let Ok(item) = declaration {
        let manifest = crate::mapping::manifest::parse(&item.captured.bytes).map_err(|_| invalid());
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let manifest = manifest?;
        for (side, resource) in
            [("policy", &manifest.mapping.source), ("framework", &manifest.mapping.target)]
        {
            check_resource(snapshot, &item.registration.path, resource)?;
            let path = resolve_reference(&item.registration.path, &resource.artifact)?;
            let registration = snapshot
                .items
                .iter()
                .find(|item| item.registration.path == path)
                .ok_or_else(invalid)?;
            let manifest_path = stage.path().join(&item.registration.path);
            append_subjects(
                &mut output,
                &mut truncated,
                manifest_path.parent().ok_or_else(invalid)?,
                side,
                resource,
                registration,
                control,
            )?;
        }
    } else {
        // An ambiguous or malformed existing manifest must not be bypassed.
        if snapshot.items.iter().any(|item| item.registration.role == Role::MappingCollection) {
            return Err(invalid().into());
        }
        let item = snapshot.item(resource_id.ok_or_else(invalid)?)?;
        let side =
            side.filter(|side| matches!(*side, "policy" | "framework")).ok_or_else(invalid)?;
        if item.registration.role != Role::OscalCatalogArtifact
            || item.validation["state"] != "valid"
        {
            return Err(invalid().into());
        }
        let resource = crate::mapping::manifest::ResourceManifest {
            resource_type: crate::mapping::manifest::ResourceType::Catalog,
            artifact: std::path::PathBuf::from(&item.registration.path),
            href: item.registration.path.clone(),
            resolved_catalog: None,
            resolved_catalog_attestation: None,
            expected_sha256: Some(item.captured.sha256.clone()),
            expected_resolved_catalog_sha256: None,
            inventory: None,
        };
        append_subjects(&mut output, &mut truncated, stage.path(), side, &resource, item, control)?;
    }
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    output.sort_by(|a, b| a["subject_id"].as_str().cmp(&b["subject_id"].as_str()));
    Ok(SubjectInventory { rows: output, truncated })
}

/// Load one checked staged resource and append bounded subject identities.
/// Checks surround the shared loader and each append; labels remain excluded
/// from operation progress, and parser work is cooperative only on return.
fn append_subjects(
    output: &mut Vec<serde_json::Value>,
    truncated: &mut Vec<(String, usize)>,
    parent: &Path,
    side: &str,
    resource: &crate::mapping::manifest::ResourceManifest,
    item: &Item,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    use crate::mapping::manifest::SubjectType;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let loaded = crate::mapping::inventory::load(parent, side, resource).map_err(|_| invalid());
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let loaded = loaded?;
    for kind in [SubjectType::Control, SubjectType::Statement] {
        if output.len().saturating_add(loaded.inventory.count(kind)) > 10000 {
            return Err(invalid().into());
        }
        for id in loaded.inventory.ids_of_type(kind) {
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let resource_id = super::services::resource_id(&item.registration);
            let (label, label_truncated) =
                super::services::bounded_label(&id, super::services::SUBJECT_LABEL_MAX);
            if label_truncated {
                if let Some((_, count)) =
                    truncated.iter_mut().find(|(existing, _)| existing == &resource_id)
                {
                    *count += 1;
                } else {
                    truncated.push((resource_id.clone(), 1));
                }
            }
            let subject_id =
                super::services::opaque("subj", &[side, &resource_id, kind.as_str(), &id]);
            let value = serde_json::json!({"subject_id":subject_id,"provenance_ref":super::services::opaque("prov", &[&subject_id]),"side":side,"resource_id":resource_id,
                "label":label,"fingerprint":loaded.inventory.fingerprint(kind,&id).ok_or_else(invalid)?,
                "statement_count":usize::from(kind == SubjectType::Statement)});
            super::contract::validate("MappingSubject", &value)?;
            output.push(value);
        }
    }
    Ok(())
}

/// Walk only validated registered OSCAL bytes through the existing trace walker.
/// Source-file props are asserted references: exact portable paths only, never
/// basename matching or evidence that the original source bytes are unchanged.
pub(crate) fn trace_entries(item: &Item) -> Result<Vec<crate::trace::report::TraceEntry>> {
    if item.validation["state"] != "valid" {
        return Err(invalid());
    }
    let value = super::contract::parse(&item.captured.bytes, 10 * 1024 * 1024, 64 * 1024)?;
    let entries = match item.registration.role {
        Role::OscalCatalogArtifact => {
            crate::trace::walker::walk_catalog_elements(&value["catalog"])
        }
        Role::OscalComponentArtifact => {
            crate::trace::walker::walk_compdef_elements(&value["component-definition"])
        }
        _ => return Err(invalid()),
    };
    if entries.len() > 10000 {
        return Err(invalid());
    }
    Ok(entries)
}

pub(crate) fn trace_source<'a>(
    snapshot: &'a Snapshot,
    trace: &crate::trace::report::TraceMetadata,
) -> Option<&'a Item> {
    if validate_path(&trace.source_file).is_err() {
        return None;
    }
    snapshot.items.iter().find(|item| {
        item.registration.role == Role::PolicySource
            && item.registration.path == trace.source_file
            && item.validation["state"] == "valid"
    })
}

pub(crate) fn trace_counts(snapshot: &Snapshot) -> Result<serde_json::Value> {
    let mut total = 0;
    let mut asserted = 0;
    let mut resolved = 0;
    for item in snapshot.items.iter().filter(|item| {
        matches!(item.registration.role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact)
    }) {
        for entry in trace_entries(item)? {
            total += 1;
            if total > 10000 {
                return Err(invalid());
            }
            if let Some(trace) = entry.trace {
                asserted += 1;
                if let Some(source) = trace_source(snapshot, &trace) {
                    let text =
                        std::str::from_utf8(&source.captured.bytes).map_err(|_| invalid())?;
                    if crate::trace::resolver::validate_line_reference(
                        trace.source_line,
                        text.lines().count(),
                    ) {
                        resolved += 1;
                    }
                }
            }
        }
    }
    Ok(
        serde_json::json!({"total_elements":total,"asserted_trace_elements":asserted,"current_source_locations":resolved,"unresolved_elements":total-resolved}),
    )
}

/// Convert one captured policy source using the existing deterministic domain
/// pipeline and suppressed source-bearing logger. Temporary copies and pipeline
/// calls are cooperative boundaries; no parser preemption or publication occurs.
pub(crate) fn convert_with_control(
    snapshot: &Snapshot,
    source: &Item,
    kind: &str,
    control: &mut dyn WorkControl,
) -> WorkResult<crate::pipeline::PipelineOutput> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    if source.registration.role != Role::PolicySource {
        return Err(invalid().into());
    }
    let stage = stage_with_control(snapshot, control)?;
    let input = stage.path().join(&source.registration.path);
    // Domain progress logs contain source labels. The local workspace never
    // forwards them to the process-wide logger, including under --verbose.
    tracing::subscriber::with_default(tracing::subscriber::NoSubscriber::default(), || {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let document =
            crate::pipeline::prepare_document(&input, 10 * 1024 * 1024).map_err(|_| invalid());
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let mut document = document?;
        document.metadata.source_path = std::path::PathBuf::from(&source.registration.path);
        let portable = std::path::Path::new(&source.registration.path);
        let metadata = Some(crate::oscal::metadata::MetadataOptions {
            uuid_override: Some(uuid::Uuid::new_v5(
                &uuid::Uuid::NAMESPACE_URL,
                format!("forge.workspace-conversion/1:{kind}:{}", source.captured.sha256)
                    .as_bytes(),
            )),
            // Reproducible generation epoch, never a review/approval timestamp.
            timestamp_override: Some(chrono::DateTime::UNIX_EPOCH),
        });
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let output = match kind {
            "oscal-catalog" => crate::pipeline::run_catalog_pipeline_with_metadata(
                portable,
                &document,
                crate::types::OutputFormat::Json,
                None,
                metadata,
            ),
            "oscal-component-definition" => crate::pipeline::run_component_pipeline_with_metadata(
                portable,
                &document,
                None,
                crate::types::OutputFormat::Json,
                None,
                metadata,
            ),
            _ => return Err(invalid().into()),
        }
        .map_err(|_| invalid());
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        Ok(output?)
    })
}

/// Initialize a domain manifest from explicit Catalog selections. Inventory and
/// pins are computed facts. Scope decisions remain empty; mapping relationships
/// and review evidence must be explicitly supplied by the caller.
pub(crate) fn initialize(
    snapshot: &Snapshot,
    request: &serde_json::Value,
    mapping: bool,
) -> Result<Vec<u8>> {
    let destination = request["target_path"].as_str().ok_or_else(invalid)?;
    let stage = stage(snapshot)?;
    let resource = |id: &str| -> Result<crate::mapping::manifest::ResourceManifest> {
        let item = snapshot.item(id)?;
        if item.registration.role != Role::OscalCatalogArtifact
            || item.validation["state"] != "valid"
        {
            return Err(invalid());
        }
        let mut resource = crate::mapping::manifest::ResourceManifest {
            resource_type: crate::mapping::manifest::ResourceType::Catalog,
            artifact: std::path::PathBuf::from(&item.registration.path),
            href: item.registration.path.clone(),
            resolved_catalog: None,
            resolved_catalog_attestation: None,
            expected_sha256: Some(item.captured.sha256.clone()),
            expected_resolved_catalog_sha256: None,
            inventory: None,
        };
        let loaded = crate::mapping::inventory::load(stage.path(), "selected resource", &resource)
            .map_err(|_| invalid())?;
        resource.inventory = Some(loaded.snapshot());
        resource.artifact =
            std::path::PathBuf::from(relative_reference(destination, &item.registration.path));
        Ok(resource)
    };
    let value = if mapping {
        let source = request["source_resource_id"].as_str().ok_or_else(invalid)?;
        let target = request["target_resource_id"].as_str().ok_or_else(invalid)?;
        if source == target {
            return Err(invalid());
        }
        serde_json::json!({"schema_version":"forge.mapping-manifest/1","collection":request["review"]["collection"],"reviewers":request["review"]["reviewers"],"provenance":request["review"]["provenance"],
            "mapping":{"key":request["review"]["collection"]["key"],"scope":request["scope"],"source":resource(source)?,"target":resource(target)?,"maps":request["maps"]}})
    } else {
        serde_json::json!({"schema_version":"forge.applicability/1","framework":resource(request["framework_resource_id"].as_str().ok_or_else(invalid)?)?,"reviewers":[],"decisions":[],"mapping_collections":[]})
    };
    let mut bytes = super::contract::encode(&value, manifest_limit(mapping) - 1, true)?;
    bytes.push(b'\n');
    if mapping {
        crate::mapping::manifest::parse(&bytes).map_err(|_| invalid())?;
        let manifest_path = stage.path().join(destination);
        std::fs::create_dir_all(manifest_path.parent().ok_or_else(invalid)?)
            .map_err(|_| invalid())?;
        std::fs::write(&manifest_path, &bytes).map_err(|_| invalid())?;
        crate::mapping::prepare(&manifest_path, None, false).map_err(|_| invalid())?;
    } else {
        crate::applicability::manifest::parse(&bytes).map_err(|_| invalid())?;
    }
    Ok(bytes)
}
fn relative_reference(manifest: &str, resource: &str) -> String {
    let mut parent: Vec<_> = manifest.split('/').collect();
    parent.pop();
    let target: Vec<_> = resource.split('/').collect();
    let common = parent.iter().zip(&target).take_while(|(a, b)| a == b).count();
    std::iter::repeat_n("..", parent.len() - common)
        .chain(target[common..].iter().copied())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_references_cannot_escape_or_change_platform_meaning() {
        assert_eq!(
            resolve_reference("manifests/scope.json", Path::new("../framework.json")).unwrap(),
            "framework.json"
        );
        for path in [
            "../../outside.json",
            "/outside.json",
            "C:/outside.json",
            "../CON.json",
            "../a\\b.json",
            "../a/./b.json",
            "../a//b.json",
        ] {
            assert!(resolve_reference("manifests/scope.json", Path::new(path)).is_err(), "{path}");
        }
    }

    fn catalog_item(id: &str) -> (crate::workspace::index::Resource, Vec<u8>) {
        let catalog = serde_json::json!({"catalog":{"uuid":"11111111-1111-4111-8111-111111111111","metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},"controls":[{"id":id,"title":"Synthetic control"}]}});
        let bytes = serde_json::to_vec(&catalog).unwrap();
        (
            crate::workspace::index::Resource {
                key: "catalog".to_owned(),
                role: crate::workspace::index::Role::OscalCatalogArtifact,
                path: "catalog.json".to_owned(),
            },
            bytes,
        )
    }

    /// Ordinary no-op inventory preserves exact long subject identities while
    /// limiting display labels and emitting valid bounded view rows.
    #[test]
    fn over_long_subject_ids_are_bounded_without_losing_identity() {
        let dir = tempfile::tempdir().unwrap();
        let long_id = "c".repeat(600);
        let (registration, bytes) = catalog_item(&long_id);
        std::fs::write(dir.path().join("catalog.json"), &bytes).unwrap();
        let resource = crate::mapping::manifest::ResourceManifest {
            resource_type: crate::mapping::manifest::ResourceType::Catalog,
            artifact: Path::new("catalog.json").to_path_buf(),
            href: "catalog.json".to_owned(),
            resolved_catalog: None,
            resolved_catalog_attestation: None,
            expected_sha256: None,
            expected_resolved_catalog_sha256: None,
            inventory: None,
        };
        let id = crate::workspace::services::resource_id(&registration);
        let sha256 = crate::hashing::sha256_hex(&bytes);
        let item = Item {
            registration,
            captured: crate::workspace::root::Captured { bytes, identity: (1, 1), sha256 },
            metadata: serde_json::json!({"resource_id": id}),
            validation: crate::workspace::services::validation(true, Some(&id)),
        };
        let mut output = Vec::new();
        let mut truncated = Vec::new();
        append_subjects(
            &mut output,
            &mut truncated,
            dir.path(),
            "policy",
            &resource,
            &item,
            &mut NoopControl,
        )
        .expect("an over-long id must not fail the inventory");
        assert_eq!(output.len(), 1);
        assert_eq!(
            output[0]["label"].as_str().unwrap().chars().count(),
            crate::workspace::services::SUBJECT_LABEL_MAX
        );
        assert_eq!(truncated, vec![(id, 1)]);
        crate::workspace::contract::validate("MappingSubject", &output[0]).unwrap();
        // The opaque anchors still bind the exact, untruncated subject identity.
        assert_ne!(output[0]["subject_id"].as_str().unwrap(), output[0]["label"].as_str().unwrap());
        assert_eq!(
            output[0]["fingerprint"].as_str().unwrap(),
            crate::mapping::inventory::load(dir.path(), "policy", &resource)
                .unwrap()
                .inventory
                .fingerprint(crate::mapping::manifest::SubjectType::Control, &long_id)
                .unwrap()
        );
    }

    /// Setup state: no decision manifest is registered, so the draft endpoint
    /// must report `not-found` (HTTP 404) rather than a validation failure, or
    /// the browser can never tell that it should offer initialization.
    #[test]
    fn missing_decision_manifest_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let snapshot =
            Snapshot::capture(&crate::workspace::root::Root::open(dir.path()).unwrap()).unwrap();
        assert_eq!(draft(&snapshot, false).unwrap_err().code, "not-found");
        assert_eq!(draft(&snapshot, true).unwrap_err().code, "not-found");
    }

    /// An ambiguous registration is not setup state: it remains a validation
    /// failure so no other failure code changes.
    #[test]
    fn ambiguous_decision_manifest_is_not_a_not_found() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("scope-a.json"), "{}\n").unwrap();
        std::fs::write(dir.path().join("scope-b.json"), "{}\n").unwrap();
        std::fs::write(
            dir.path().join(crate::workspace::index::INDEX_PATH),
            serde_json::to_vec(&serde_json::json!({"schema_version":"forge.workspace/1","label":"Example project","resources":[
                {"key":"scope-a","role":"applicability-manifest","path":"scope-a.json"},
                {"key":"scope-b","role":"applicability-manifest","path":"scope-b.json"}]}))
            .unwrap(),
        )
        .unwrap();
        let snapshot =
            Snapshot::capture(&crate::workspace::root::Root::open(dir.path()).unwrap()).unwrap();
        let error = draft(&snapshot, false).unwrap_err();
        assert_eq!(error.code, "validation-failed");
        assert_ne!(error.code, "not-found");
    }

    /// Capture three real synthetic Markdown registrations in order, including
    /// nested and unrelated inputs. Their bytes and identities come from Root.
    fn staged_fixture() -> (tempfile::TempDir, Snapshot) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("inputs")).unwrap();
        let paths = ["first.md", "inputs/source.md", "unrelated.md"];
        let mut resources = Vec::new();
        for (index, path) in paths.into_iter().enumerate() {
            std::fs::write(
                dir.path().join(path),
                "# Synthetic policy\n\nStaff must review proposed changes.\n",
            )
            .unwrap();
            resources.push(serde_json::json!({"key":format!("source-{index}"),"role":"policy-source","path":path}));
        }
        std::fs::write(dir.path().join(crate::workspace::index::INDEX_PATH), serde_json::to_vec(&serde_json::json!({"schema_version":"forge.workspace/1","label":"Synthetic staged inputs","resources":resources})).unwrap()).unwrap();
        let snapshot =
            Snapshot::capture(&crate::workspace::root::Root::open(dir.path()).unwrap()).unwrap();
        (dir, snapshot)
    }

    /// A later unrelated registration is still a staging boundary. Interrupting
    /// before its copy drops the already partially copied private directory.
    #[test]
    fn controlled_staging_drops_partial_owned_directory() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let (dir, snapshot) = staged_fixture();
        let stage = tempfile::tempdir().unwrap();
        let stage_path = stage.path().to_owned();
        let mut control = Recorder::at(Stage::CopyInputs, 5);
        assert!(matches!(
            copy_to_stage(&snapshot, stage, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(!stage_path.exists());
        for item in &snapshot.items {
            assert_eq!(
                std::fs::read(dir.path().join(&item.registration.path)).unwrap(),
                item.captured.bytes
            );
        }
    }

    /// No-op staging preserves all captured bytes; a genuine parent/file collision
    /// remains an ordinary safe failure and drops partial temporary output.
    #[test]
    fn controlled_staging_preserves_success_and_ordinary_io_failure() {
        let (dir, snapshot) = staged_fixture();
        let stage = stage_with_control(&snapshot, &mut NoopControl).unwrap();
        for item in &snapshot.items {
            assert_eq!(
                std::fs::read(stage.path().join(&item.registration.path)).unwrap(),
                item.captured.bytes
            );
        }
        let blocked = tempfile::tempdir().unwrap();
        let blocked_path = blocked.path().to_owned();
        std::fs::write(blocked.path().join("inputs"), "blocking file").unwrap();
        let error = copy_to_stage(&snapshot, blocked, &mut NoopControl).err().unwrap();
        assert!(matches!(error, WorkError::Failed(error) if error.code == "validation-failed"));
        assert!(!blocked_path.exists());
        assert_eq!(
            std::fs::read(dir.path().join("inputs/source.md")).unwrap(),
            snapshot.items[1].captured.bytes
        );
    }

    /// Controlled conversion retains deterministic normal output, and interruption
    /// after the existing input parser returns prevents later pipeline preparation.
    #[test]
    fn controlled_conversion_preserves_output_and_post_parser_stop() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let (dir, snapshot) = staged_fixture();
        let source = &snapshot.items[1];
        let first =
            convert_with_control(&snapshot, source, "oscal-catalog", &mut NoopControl).unwrap();
        let mut observed = Recorder::default();
        let second =
            convert_with_control(&snapshot, source, "oscal-catalog", &mut observed).unwrap();
        assert_eq!(first.content, second.content);
        let value: serde_json::Value = serde_json::from_str(&first.content).unwrap();
        assert!(
            crate::validate::run_full_validation(
                "synthetic controlled conversion",
                &value,
                crate::validate::OscalModelType::Catalog
            )
            .unwrap()
            .is_valid()
        );
        let mut stop = Recorder::at(Stage::PrepareDomain, 3);
        assert!(matches!(
            convert_with_control(&snapshot, source, "oscal-catalog", &mut stop),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(!dir.path().join("converted.json").exists());
    }

    /// Missing domain declarations are ordinary validation failures under no-op
    /// control. Controlled checkpoints remain distinct from those safe failures.
    #[test]
    fn controlled_domain_errors_remain_ordinary_validation_failures() {
        use super::super::preparation::{Interruption, test_support::Recorder};
        let dir = tempfile::tempdir().unwrap();
        let snapshot =
            Snapshot::capture(&crate::workspace::root::Root::open(dir.path()).unwrap()).unwrap();
        assert_eq!(analyze(&snapshot).err().unwrap().code, "validation-failed");
        assert_eq!(mapping(&snapshot).err().unwrap().code, "validation-failed");
        assert_eq!(subject_inventory(&snapshot).err().unwrap().code, "validation-failed");
        let mut stop = Recorder::at(Stage::PrepareDomain, 2);
        assert!(matches!(
            mapping_with_control(&snapshot, &mut stop),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
    }
}
