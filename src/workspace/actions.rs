//! Material requests prepare a single effect; this adapter never writes files.
use super::contract::{self, ApiMajor, Error, Result};
use super::effects::{Reply, Store};
#[cfg(test)]
use super::index::Index;
use super::index::{INDEX_PATH, Resource, Role};
use super::preparation::{NoopControl, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult};
use super::root::{Root, conflict};
use super::services::{Item, Snapshot};
use base64::Engine as _;
use serde_json::{Value, json};

fn validation_error() -> Error {
    Error::new(
        "validation-failed",
        "Review the complete proposed document and its registered dependencies.",
        false,
    )
}
fn text<'a>(request: &'a Value, key: &str) -> Result<&'a str> {
    request[key].as_str().ok_or_else(Error::invalid)
}
#[allow(clippy::needless_pass_by_value)] // Ownership ends at the response envelope.
fn preview_reply(preview: Value) -> Reply {
    Reply {
        value: json!({"validation":super::services::validation(true,None),"preview":preview}),
        schema: "DraftPreviewResponse",
        status: 200,
    }
}
fn operation_reply(value: Value) -> Reply {
    Reply { value, schema: "Operation", status: 202 }
}
/// Reject index/source destinations before any producer reads or previews an output target.
pub(crate) fn output_target(snapshot: &Snapshot, path: &str, allowed: Option<Role>) -> Result<()> {
    if path.eq_ignore_ascii_case(INDEX_PATH)
        || snapshot.items.iter().any(|item| {
            item.registration.path.eq_ignore_ascii_case(path)
                && Some(item.registration.role) != allowed
        })
    {
        return Err(Error::containment());
    }
    super::index::validate_path(path)
}

/// Derive the stable index key for a registered path. The committed index
/// pattern requires a lowercase alphanumeric first and last character
/// (`^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$`), so the candidate is capped to 64
/// characters before both ends are trimmed. A candidate that trims to nothing (a
/// hidden file such as `.env`, or a name that is entirely punctuation) cannot
/// fall back to a constant like `resource`: every such registration would claim
/// the same key and collide. Hashing the portable path keeps those keys distinct,
/// stable, and deterministic across runs.
fn registration_key(path: &str) -> String {
    let stem = path
        .rsplit('/')
        .next()
        .unwrap_or("resource")
        .split('.')
        .next()
        .unwrap_or("resource")
        .to_ascii_lowercase();
    let candidate: String =
        stem.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    // The length cap is applied before the final trim so a trailing separator
    // introduced by the cap does not survive into the key.
    let candidate: String = candidate.chars().take(64).collect();
    let trimmed = candidate.trim_matches('-');
    if trimmed.is_empty() {
        format!("resource-{}", &crate::hashing::sha256_hex(path.as_bytes())[..12])
    } else {
        trimmed.to_owned()
    }
}

/// The registered inputs a prepared manifest write consumes: the manifest being
/// replaced plus every reference the proposed document resolves to. Deriving the
/// consumed set keeps the documented 100-input bound a property of the effect
/// instead of a property of how many unrelated resources the project registers.
fn manifest_inputs<'a>(
    snapshot: &'a Snapshot,
    manifest: &str,
    references: &[&std::path::PathBuf],
) -> Result<Vec<&'a Item>> {
    let mut paths = vec![manifest.to_owned()];
    for reference in references {
        paths.push(super::domain::resolve_reference(manifest, reference)?);
    }
    paths
        .iter()
        .map(|path| {
            snapshot
                .items
                .iter()
                .find(|item| item.registration.path == *path)
                .ok_or_else(validation_error)
        })
        .collect()
}

/// Both artifacts a mapping manifest binds, with the Profile companion each may
/// carry. The domain validation already rejects references that are not
/// registered, so a missing item here can only be a stale prepared request.
fn mapping_references(
    manifest: &crate::mapping::manifest::MappingManifest,
) -> Vec<&std::path::PathBuf> {
    [&manifest.mapping.source, &manifest.mapping.target]
        .into_iter()
        .flat_map(|resource| {
            std::iter::once(&resource.artifact).chain(resource.resolved_catalog.iter())
        })
        .collect()
}

/// Prepare a compatible registration or explicit /1-to-/2 index migration.
///
/// All variants retain the existing index effect/conditional commit and never change
/// domain records. Version/role checks happen before an unregistered path read;
/// migration-only consumes the captured index and preserves authorial resource order.
fn prepare_registration(
    store: &mut Store,
    root: &Root,
    snapshot: &Snapshot,
    request: &Value,
    api_major: ApiMajor,
) -> Result<Reply> {
    contract::validate_for(api_major, "RegisterResourceRequest", request)?;
    let mut index = snapshot.index.clone();
    if request.get("migration").is_some() {
        if !snapshot.index_present || index.schema_version != "forge.workspace/1" {
            return Err(validation_error());
        }
        index.schema_version = "forge.workspace/2".into();
        let bytes = index.bytes()?;
        return Ok(preview_reply(store.preview(
            root,
            snapshot,
            INDEX_PATH,
            "workspace-index-update",
            bytes,
            &[],
        )?));
    }
    if request.get("index_schema_version").is_some() {
        index.schema_version = "forge.workspace/2".into();
    }
    let role: Role =
        serde_json::from_value(request["role"].clone()).map_err(|_| Error::invalid())?;
    if !role.admitted_by(&index.schema_version) {
        return Err(Error::invalid());
    }
    let path = text(request, "path")?;
    let captured = root.read(path, 10 * 1024 * 1024)?;
    if snapshot.items.iter().any(|item| item.captured.identity == captured.identity) {
        return Err(Error::containment());
    }
    let key = request["key"].as_str().map_or_else(|| registration_key(path), str::to_owned);
    let registration = Resource { key, role, path: path.to_owned() };
    if !(super::services::validate_bytes(&registration, &captured.bytes)
        || (role == Role::ApplicabilityReport
            && snapshot.analysis.as_ref().is_some_and(|analysis| {
                serde_json::to_value(analysis).ok()
                    == super::contract::parse(&captured.bytes, 10 * 1024 * 1024, 64 * 1024).ok()
            })))
    {
        return Err(validation_error());
    }
    index.resources.push(registration.clone());
    let bytes = index.bytes()?;
    let mut preview =
        store.preview(root, snapshot, INDEX_PATH, "workspace-index-update", bytes, &[])?;
    // The selected unregistered file remains an exact-byte external dependency.
    store.bind_external(&mut preview, &registration, captured)?;
    Ok(preview_reply(preview))
}

/// Prepare ordinary direct effects without a callback that could relock their Store.
pub(crate) fn prepare(
    store: &mut Store,
    root: &Root,
    snapshot: &mut Snapshot,
    method: &str,
    path: &str,
    request: &Value,
) -> Result<Reply> {
    prepare_for_api(store, root, snapshot, method, path, request, ApiMajor::V1)
}

/// Prepare direct effects for an admitted public major using the private canonical route.
/// The HTTP integration retains the original public path in its replay and idempotency key.
pub(crate) fn prepare_for_api(
    store: &mut Store,
    root: &Root,
    snapshot: &mut Snapshot,
    method: &str,
    path: &str,
    request: &Value,
    api_major: ApiMajor,
) -> Result<Reply> {
    prepare_with_control_for_api(
        store,
        root,
        snapshot,
        method,
        (api_major, path),
        request,
        &mut NoopControl,
    )
    .map_err(WorkError::into_error)
}

/// Preserve the original v1 controlled-preparation entry point and stop semantics.
pub(crate) fn prepare_with_control(
    store: &mut Store,
    root: &Root,
    snapshot: &mut Snapshot,
    method: &str,
    path: &str,
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<Reply> {
    prepare_with_control_for_api(
        store,
        root,
        snapshot,
        method,
        (ApiMajor::V1, path),
        request,
        control,
    )
}

/// Prepare a selected-major request after public admission, with the existing real stop fences.
/// The tuple carries the selected major and private canonical path; it is never a wire alias.
#[allow(clippy::too_many_lines)] // One audited route-to-effect table, unchanged domain branches.
pub(crate) fn prepare_with_control_for_api(
    store: &mut Store,
    root: &Root,
    snapshot: &mut Snapshot,
    method: &str,
    api_path: (ApiMajor, &str),
    request: &Value,
    control: &mut dyn WorkControl,
) -> WorkResult<Reply> {
    let (api_major, path) = api_path;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    if !matches!(
        (api_major, snapshot.index.schema_version.as_str()),
        (ApiMajor::V1, "forge.workspace/1")
            | (ApiMajor::V2, "forge.workspace/1" | "forge.workspace/2")
    ) {
        return Err(Error::invalid().into());
    }
    let reply: WorkResult<Reply> = match (method, path) {
        ("POST", "/api/v1/project/bundle-exports") if api_major == ApiMajor::V2 => {
            let plan = super::bundle_effects::prepare_export(root, snapshot, request, control)?;
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let preview = store.preview_bundle(plan)?;
            let reply = operation_reply(store.completed(
                "export",
                json!({
                    "operation_id":"op_000000000000", "preview":preview,
                    "redaction_summary":{"removed_categories":["source-excerpts"]}
                }),
            )?);
            // Metadata labels/paths are sensitive; charge the entire retained wrapper too.
            store.charge_reply(&reply)?;
            Ok(reply)
        }
        ("POST", "/api/v1/applicability/initializations" | "/api/v1/mapping/initializations") => {
            let target = text(request, "target_path")?;
            output_target(snapshot, target, None)?;
            if root.target(target)?.base.is_some() {
                return Err(conflict().into());
            }
            let mapping = path.contains("/mapping/");
            let bytes = super::domain::initialize(snapshot, request, mapping)?;
            let ids = if mapping {
                vec![text(request, "source_resource_id")?, text(request, "target_resource_id")?]
            } else {
                vec![text(request, "framework_resource_id")?]
            };
            let inputs: Result<Vec<_>> = ids.iter().map(|id| snapshot.item(id)).collect();
            Ok(preview_reply(store.preview(
                root,
                snapshot,
                target,
                if mapping { "mapping-manifest-write" } else { "applicability-manifest-write" },
                bytes,
                &inputs?,
            )?))
        }
        ("POST", "/api/v1/resources/register") => {
            Ok(prepare_registration(store, root, snapshot, request, api_major)?)
        }
        ("POST", "/api/v1/resources/upload") => {
            let path = text(request, "target_path")?;
            output_target(snapshot, path, None)?;
            let role: Role =
                serde_json::from_value(request["role"].clone()).map_err(|_| Error::invalid())?;
            if api_major == ApiMajor::V1 && !role.admitted_by("forge.workspace/1") {
                return Err(Error::invalid().into());
            }
            let encoded = text(request, "content_base64")?;
            if encoded.len() > 13_981_016 {
                return Err(Error::invalid().into());
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(|_| Error::invalid())?;
            if bytes.len() > 10 * 1024 * 1024 {
                return Err(Error::invalid().into());
            }
            let registration = Resource { key: "upload".into(), role, path: path.into() };
            if !super::services::validate_bytes(&registration, &bytes) {
                return Err(validation_error().into());
            }
            Ok(preview_reply(store.preview(root, snapshot, path, "resource-upload", bytes, &[])?))
        }
        ("PUT", "/api/v1/applicability/draft" | "/api/v1/mapping/draft") => {
            let mapping = path.contains("/mapping/");
            let item = if mapping {
                super::domain::mapping_manifest(snapshot)?
            } else {
                super::domain::selected(snapshot, Role::ApplicabilityManifest)?
            };
            let target = item.registration.path.clone();
            if request["observed_version"] != item.metadata["version"] {
                return Err(conflict().into());
            }
            let validation =
                super::domain::validate_draft(snapshot, mapping, &request["manifest"])?;
            if validation["state"] != "valid" {
                return Err(validation_error().into());
            }
            // The draft is validated against the deployed manifest contract, so the
            // encode bound is that contract's byte limit minus the trailing newline
            // the published document carries, capped at the workspace's 10 MiB
            // per-resource capture bound: a larger manifest could never be captured
            // again, so the workspace would fail closed on its own output. The 1 MiB
            // HTTP body limit keeps a request's pretty encoding well under it.
            let mut bytes = super::contract::encode(
                &request["manifest"],
                super::domain::manifest_limit(mapping).min(10 * 1024 * 1024) - 1,
                true,
            )?;
            bytes.push(b'\n');
            let inputs = if mapping {
                let parsed =
                    crate::mapping::manifest::parse(&bytes).map_err(|_| validation_error())?;
                manifest_inputs(snapshot, &target, &mapping_references(&parsed))?
            } else {
                let parsed = crate::applicability::manifest::parse(&bytes)
                    .map_err(|_| validation_error())?;
                let references: Vec<_> = std::iter::once(&parsed.framework.artifact)
                    .chain(parsed.framework.resolved_catalog.iter())
                    .chain(parsed.mapping_collections.iter())
                    .collect();
                manifest_inputs(snapshot, &target, &references)?
            };
            Ok(preview_reply(store.preview(
                root,
                snapshot,
                &target,
                if mapping { "mapping-manifest-write" } else { "applicability-manifest-write" },
                bytes,
                &inputs,
            )?))
        }
        ("POST", "/api/v1/conversions") => {
            let source = snapshot.item(text(request, "source_resource_id")?)?;
            let target = text(request, "target_path")?;
            output_target(snapshot, target, None)?;
            let kind = text(request, "output_kind")?;
            let result = super::domain::convert_with_control(snapshot, source, kind, control)?;
            if !result.secondary_outputs.is_empty() {
                return Err(validation_error().into());
            }
            let count = result.statistics.requirements_extracted;
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let preview = store.preview(
                root,
                snapshot,
                target,
                "policy-conversion",
                result.content.into_bytes(),
                &[source],
            )?;
            let operation = store.completed("conversion",json!({"operation_id":"op_000000000000","products":[{"kind":kind,"statement_count":count}],"validation":super::services::validation(true,None),"preview":preview}))?;
            Ok(operation_reply(operation))
        }
        ("POST", "/api/v1/applicability/analyses") => {
            let analysis = snapshot.analysis.as_ref().ok_or_else(validation_error)?;
            let mut reports = snapshot
                .items
                .iter()
                .filter(|item| item.registration.role == Role::ApplicabilityReport);
            let target = reports
                .next()
                .map_or("applicability-report.json", |item| item.registration.path.as_str());
            if reports.next().is_some() {
                return Err(validation_error().into());
            }
            output_target(snapshot, target, Some(Role::ApplicabilityReport))?;
            let mut bytes = super::contract::encode(analysis, 10 * 1024 * 1024 - 1, true)?;
            bytes.push(b'\n');
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let preview = store.preview(
                root,
                snapshot,
                target,
                "applicability-report-write",
                bytes,
                &snapshot.analysis_inputs()?,
            )?;
            let result = json!({"classification_counts":snapshot.classification_counts()?,"eligible_controls":analysis.counts.total,"stale_inputs":snapshot.items.iter().any(|item|item.metadata["stale"]==true),"report_preview":preview});
            Ok(operation_reply(store.completed("applicability-analysis", result)?))
        }
        ("POST", "/api/v1/mapping/builds") => {
            let built = super::domain::mapping_with_control(snapshot, control)?;
            let manifest_item = super::domain::mapping_manifest(snapshot)?;
            let manifest = crate::mapping::manifest::parse(&manifest_item.captured.bytes)
                .map_err(|_| validation_error())?;
            let target = "mapping-collection.json";
            output_target(snapshot, target, Some(Role::MappingCollection))?;
            if manifest_item.registration.path.eq_ignore_ascii_case(target) {
                return Err(Error::containment().into());
            }
            let inputs = manifest_inputs(
                snapshot,
                &manifest_item.registration.path,
                &mapping_references(&manifest),
            )?;
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let preview = store.preview(
                root,
                snapshot,
                target,
                "mapping-report-write",
                built.artifact_json.into_bytes(),
                &inputs,
            )?;
            let positive = manifest
                .mapping
                .maps
                .iter()
                .filter(|m| {
                    m.relationship != crate::mapping::manifest::Relationship::NoRelationship
                })
                .count();
            let result = json!({"maps_total":manifest.mapping.maps.len(),"positive_relationship_count":positive,"no_relationship_count":manifest.mapping.maps.len()-positive,
                "controls_covered":built.report.target_controls.referenced,"coverage_ratio":built.report.target_controls.ratio,"report_preview":preview});
            Ok(operation_reply(store.completed("mapping-build", result)?))
        }
        ("POST", "/api/v1/exports") => {
            let target = text(request, "target_path")?;
            output_target(snapshot, target, None)?;
            let kind = text(request, "report_kind")?;
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let summary = match kind {
                "applicability-gap" => snapshot.classification_counts()?,
                "mapping-collection" => {
                    let built = super::domain::mapping_with_control(snapshot, control)?;
                    json!({"eligible_controls":built.report.target_controls.eligible,"referenced_controls":built.report.target_controls.referenced})
                }
                "trace" => super::domain::trace_counts(snapshot)?,
                _ => return Err(Error::invalid().into()),
            };
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let bytes = super::reports::render(snapshot, kind, summary)?;
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
            let inputs = super::reports::inputs(snapshot, kind)?;
            control.checkpoint(Stage::PreparePreview, ProgressUpdate::Unchanged)?;
            let preview = store.preview(root, snapshot, target, "report-export", bytes, &inputs)?;
            Ok(operation_reply(store.completed("export",json!({"operation_id":"op_000000000000","preview":preview,"redaction_summary":{"removed_categories":["reviewer-names","absolute-paths","source-excerpts","secrets"]}}))?))
        }
        _ => {
            Err(Error::new("not-found", "The requested effect operation was not found.", false)
                .into())
        }
    };
    let reply = reply?;
    control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    fn catalog(controls: &[&str], uuid: &str) -> Value {
        json!({"catalog":{"uuid":uuid,"metadata":{"title":"Synthetic catalog","last-modified":"2026-09-10T00:00:00Z","version":"1","oscal-version":"1.2.3"},
            "controls":controls.iter().map(|id| json!({"id":id,"title":"Synthetic control"})).collect::<Vec<_>>()}})
    }

    fn write_index(root: &std::path::Path, resources: &[Value]) {
        std::fs::write(
            root.join(INDEX_PATH),
            serde_json::to_vec(&json!({"schema_version":"forge.workspace/1","label":"Example project","resources":resources}))
                .unwrap(),
        )
        .unwrap();
    }

    fn pick(value: &Value, path: &[&str]) -> Value {
        let mut value = value;
        for step in path {
            value = &value[*step];
        }
        value.clone()
    }

    fn commit(store: &mut Store, root: &Root, preview: &Value) {
        let operation = store
            .commit(
                root,
                &json!({"receipt":preview["receipt"]["token"],"observed_version":preview["target_version"],"confirmed":true}),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert_eq!(operation["state"], "succeeded");
    }

    fn registered_version(snapshot: &Snapshot, key: &str) -> Value {
        snapshot.items.iter().find(|item| item.registration.key == key).unwrap().metadata["version"]
            .clone()
    }

    /// A project may register up to 1,000 resources, and the documented bound is
    /// 100 *consumed* inputs per effect. Every effect must stay preparable and
    /// committable when the project registers far more than 100 files.
    #[test]
    #[allow(clippy::too_many_lines)] // One audited prepare-and-commit effect walk.
    fn effects_prepare_and_commit_with_more_than_a_hundred_registrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        std::fs::write(
            path.join("framework.json"),
            serde_json::to_vec(&catalog(
                &["control-a", "framework-a"],
                "22222222-2222-4222-8222-222222222222",
            ))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            path.join("policy.json"),
            serde_json::to_vec(&catalog(&["policy-a"], "11111111-1111-4111-8111-111111111111"))
                .unwrap(),
        )
        .unwrap();
        let mut resources = vec![
            json!({"key":"framework","role":"oscal-catalog-artifact","path":"framework.json"}),
            json!({"key":"policy","role":"oscal-catalog-artifact","path":"policy.json"}),
        ];
        for index in 0..150 {
            let name = format!("source-{index}.md");
            std::fs::write(path.join(&name), "# Example\n\nA human-supplied clause.\n").unwrap();
            resources
                .push(json!({"key":format!("source-{index}"),"role":"policy-source","path":name}));
        }
        write_index(path, &resources);
        let root = Root::open(path).unwrap();
        assert_eq!(root.project_path(), path.canonicalize().unwrap());
        let snapshot = Snapshot::capture(&root).unwrap();
        assert_eq!(snapshot.items.len(), 152);
        let id = |key: &str| {
            super::super::services::resource_id(
                &snapshot
                    .items
                    .iter()
                    .find(|item| item.registration.key == key)
                    .unwrap()
                    .registration,
            )
        };
        let scope = super::super::domain::initialize(
            &snapshot,
            &json!({"target_path":"scope.json","framework_resource_id":id("framework")}),
            false,
        )
        .unwrap();
        std::fs::write(path.join("scope.json"), &scope).unwrap();
        let mapping = super::super::domain::initialize(
            &snapshot,
            &json!({"source_resource_id":id("policy"),"target_resource_id":id("framework"),
                "target_path":"mapping-manifest.json","scope":"control-only",
                "maps":[{"key":"none","relationship":"no-relationship","sources":[{"type":"control","id_ref":"policy-a"}],"targets":[{"type":"control","id_ref":"framework-a"}],"reviewer_key":"reviewer","reviewed_at":"2026-09-10T00:00:00Z","rationale":"Explicit initial review."}],
                "review":{"collection":{"key":"synthetic-map","title":"Synthetic mapping","version":"1","last_modified":"2026-09-10T00:00:00Z"},
                    "reviewers":[{"key":"reviewer","type":"person","name":"Synthetic Reviewer"}],
                    "provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Explicit synthetic review.","reviewer_keys":["reviewer"],"reviewed_at":"2026-09-10T00:00:00Z"}}}),
            true,
        )
        .unwrap();
        std::fs::write(path.join("mapping-manifest.json"), &mapping).unwrap();
        resources.push(json!({"key":"scope","role":"applicability-manifest","path":"scope.json"}));
        resources.push(
            json!({"key":"mapping","role":"mapping-collection","path":"mapping-manifest.json"}),
        );
        write_index(path, &resources);
        assert_eq!(Snapshot::capture(&root).unwrap().items.len(), 154);
        let mut store = Store::default();

        // Each request re-captures the project, exactly as the server does; a
        // committed effect changes the destination identity, not just its bytes.
        let mut snapshot = Snapshot::capture(&root).unwrap();
        let scope_draft = json!({"manifest":serde_json::from_slice::<Value>(&scope).unwrap(),"observed_version":registered_version(&snapshot,"scope")});
        let applicability = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "PUT",
            "/api/v1/applicability/draft",
            &scope_draft,
        )
        .unwrap();
        let preview = pick(&applicability.value, &["preview"]);
        assert_eq!(preview["input_hashes"].as_array().unwrap().len(), 2);
        commit(&mut store, &root, &preview);

        let mut snapshot = Snapshot::capture(&root).unwrap();
        let mapping_draft = json!({"manifest":serde_json::from_slice::<Value>(&mapping).unwrap(),"observed_version":registered_version(&snapshot,"mapping")});
        let mapping_preview = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "PUT",
            "/api/v1/mapping/draft",
            &mapping_draft,
        )
        .unwrap();
        let preview = pick(&mapping_preview.value, &["preview"]);
        assert_eq!(preview["input_hashes"].as_array().unwrap().len(), 3);
        commit(&mut store, &root, &preview);

        let mut snapshot = Snapshot::capture(&root).unwrap();
        let build = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/mapping/builds",
            &json!({}),
        )
        .unwrap();
        let preview = pick(&build.value, &["result", "report_preview"]);
        assert_eq!(preview["input_hashes"].as_array().unwrap().len(), 3);
        commit(&mut store, &root, &preview);

        let mut snapshot = Snapshot::capture(&root).unwrap();
        let export = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/exports",
            &json!({"report_kind":"applicability-gap","target_path":"review.html"}),
        )
        .unwrap();
        let preview = pick(&export.value, &["result", "preview"]);
        assert_eq!(preview["input_hashes"].as_array().unwrap().len(), 4);
        commit(&mut store, &root, &preview);

        assert!(path.join("mapping-collection.json").exists());
        assert!(path.join("review.html").exists());
    }

    /// The index pattern requires an alphanumeric final character, so a derived
    /// key must be re-trimmed after the 64-character cap.
    #[test]
    fn derived_registration_keys_never_end_with_a_hyphen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let name = format!("{}_b.md", "a".repeat(63));
        std::fs::write(path.join(&name), "# Example\n\nA human-supplied clause.\n").unwrap();
        let root = Root::open(path).unwrap();
        let mut snapshot = Snapshot::capture(&root).unwrap();
        let mut store = Store::default();
        let reply = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/resources/register",
            &json!({"role":"policy-source","path":name}),
        )
        .unwrap();
        commit(&mut store, &root, &pick(&reply.value, &["preview"]));
        let index =
            super::super::index::Index::parse(&std::fs::read(path.join(INDEX_PATH)).unwrap())
                .unwrap();
        assert_eq!(index.resources.len(), 1);
        assert_eq!(index.resources[0].key, "a".repeat(63));
        assert_eq!(index.resources[0].key, super::registration_key(&name));
    }

    /// A derived key must satisfy the committed index pattern even for a stem
    /// that leads with a separator or reduces to nothing. A project path cannot
    /// itself lead with `.` or `-` (`root.read` runs `validate_path` before the
    /// key is derived, and the index schema rejects both), so the pathological
    /// inputs are driven through the derivation directly and then round-tripped
    /// through the same closed-index parse the register route runs.
    #[test]
    fn derived_registration_keys_are_pattern_valid_and_distinct() {
        let leading = super::registration_key("-.md");
        let empty = super::registration_key(".env");
        assert_eq!(leading, format!("resource-{}", &crate::hashing::sha256_hex(b"-.md")[..12]));
        assert_eq!(empty, format!("resource-{}", &crate::hashing::sha256_hex(b".env")[..12]));
        assert_ne!(leading, empty, "distinct paths must not share one fallback key");
        for key in [&leading, &empty] {
            assert!(key.len() <= 64, "{key}");
            assert!(key.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric), "{key}");
            assert!(key.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric), "{key}");
        }
        let index = json!({"schema_version":"forge.workspace/1","label":"Example project","resources":[
            {"key":leading,"role":"policy-source","path":"leading.md"},
            {"key":empty,"role":"policy-source","path":"empty.md"}]});
        let index = super::super::index::Index::parse(&serde_json::to_vec(&index).unwrap())
            .expect("both derived keys must register");
        assert_eq!(index.resources.len(), 2);
    }
    /// Build a confined project with complete policy inputs and optional valid inert trace reports.
    fn checkpoint_action_fixture(
        policy_count: usize,
        report_count: usize,
    ) -> (tempfile::TempDir, Root, Snapshot) {
        let directory = tempfile::tempdir().unwrap();
        let mut resources = Vec::new();
        for number in 0..policy_count {
            let path = format!("policy-{number}.md");
            std::fs::write(
                directory.path().join(&path),
                format!("# Policy {number}\n\nAn explicit supplied clause.\n"),
            )
            .unwrap();
            resources
                .push(json!({"key":format!("policy-{number}"),"role":"policy-source","path":path}));
        }
        checkpoint_action_write_index(directory.path(), &resources);
        let root = Root::open(directory.path()).unwrap();
        if report_count != 0 {
            let initial = Snapshot::capture(&root).unwrap();
            let bytes = super::super::reports::render(&initial, "trace", json!({
                "total_elements":0,"asserted_trace_elements":0,"current_source_locations":0,"unresolved_elements":0
            })).unwrap();
            for number in 0..report_count {
                let path = format!("report-{number}.html");
                std::fs::write(directory.path().join(&path), &bytes).unwrap();
                resources.push(
                    json!({"key":format!("report-{number}"),"role":"trace-report","path":path}),
                );
            }
            checkpoint_action_write_index(directory.path(), &resources);
        }
        std::fs::write(directory.path().join("review.html"), b"EXISTING REVIEW SENTINEL\n")
            .unwrap();
        std::fs::write(directory.path().join("keeper.html"), b"EXISTING KEEPER SENTINEL\n")
            .unwrap();
        let snapshot = Snapshot::capture(&root).unwrap();
        (directory, root, snapshot)
    }

    /// Persist the actual normalized closed index instead of assuming a serializer's field ordering.
    fn checkpoint_action_write_index(directory: &std::path::Path, resources: &[Value]) {
        let index = json!({"schema_version":"forge.workspace/1","label":"Checkpoint fixture","resources":resources});
        let index = Index::parse(&serde_json::to_vec(&index).unwrap()).unwrap();
        std::fs::write(directory.join(INDEX_PATH), index.bytes().unwrap()).unwrap();
    }

    /// Record every flat fixture file so failed or prepared work cannot silently add an output or change input bytes.
    fn checkpoint_action_project_bytes(
        directory: &std::path::Path,
    ) -> std::collections::BTreeMap<String, Vec<u8>> {
        std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                assert!(entry.file_type().unwrap().is_file());
                (entry.file_name().into_string().unwrap(), std::fs::read(entry.path()).unwrap())
            })
            .collect()
    }

    /// Prepare a real unrelated mapping-summary receipt whose input roles exclude all policy files.
    fn checkpoint_action_keeper(
        store: &mut Store,
        root: &Root,
        snapshot: &Snapshot,
    ) -> (String, Value) {
        let bytes = super::super::reports::render(
            snapshot,
            "mapping-collection",
            json!({
                "eligible_controls":0,"referenced_controls":0
            }),
        )
        .unwrap();
        let preview =
            store.preview(root, snapshot, "keeper.html", "report-export", bytes, &[]).unwrap();
        (preview["preview_id"].as_str().unwrap().to_owned(), preview)
    }

    /// Test observer of the genuinely consumed control API, with no I/O replacement or production pause hook.
    struct CheckpointActionControl {
        /// Actual producer stage selected for this bounded interruption scenario.
        stop_stage: super::super::preparation::Stage,
        /// One-based matching callback to stop; `usize::MAX` permits ordinary continuation.
        stop_occurrence: usize,
        /// Number of selected stage callbacks actually reached by the producer.
        matching_occurrences: usize,
        /// Ordered stage observations before the sticky stop, without source-bearing metadata.
        events: Vec<std::mem::Discriminant<super::super::preparation::Stage>>,
        /// Internal interruption cause returned at the selected checkpoint.
        reason: super::super::preparation::Interruption,
        /// Sticky latch preventing later callbacks from continuing after interruption.
        stopped: bool,
    }

    impl super::super::preparation::WorkControl for CheckpointActionControl {
        /// Stop on a selected actual checkpoint occurrence and retain the first interruption thereafter.
        fn checkpoint(
            &mut self,
            stage: super::super::preparation::Stage,
            _progress: super::super::preparation::ProgressUpdate,
        ) -> super::super::preparation::WorkResult<()> {
            use super::super::preparation::WorkError;
            if let Some(reason) = self.interruption() {
                return Err(WorkError::Interrupted(reason));
            }
            self.events.push(std::mem::discriminant(&stage));
            if std::mem::discriminant(&stage) == std::mem::discriminant(&self.stop_stage) {
                self.matching_occurrences += 1;
                if self.matching_occurrences == self.stop_occurrence {
                    self.stopped = true;
                    return Err(WorkError::Interrupted(self.interruption().unwrap()));
                }
            }
            Ok(())
        }

        /// Expose the latched internal cause without converting it into a normal validation error.
        fn interruption(&self) -> Option<super::super::preparation::Interruption> {
            self.stopped.then_some(self.reason)
        }
    }

    /// Select an actual stage occurrence; `usize::MAX` yields an ordinary continuing observer for these bounded fixtures.
    fn checkpoint_action_control(
        stage: super::super::preparation::Stage,
        occurrence: usize,
        reason: super::super::preparation::Interruption,
    ) -> CheckpointActionControl {
        CheckpointActionControl {
            stop_stage: stage,
            stop_occurrence: occurrence,
            matching_occurrences: 0,
            events: Vec::new(),
            reason,
            stopped: false,
        }
    }

    /// A trace export binds every one of exactly 100 policy inputs, with full hashes and no project publication.
    #[test]
    fn checkpoint_trace_export_accepts_exactly_one_hundred_complete_inputs() {
        use super::super::preparation::{Interruption, Stage};
        let (directory, root, mut snapshot) = checkpoint_action_fixture(100, 0);
        let before = checkpoint_action_project_bytes(directory.path());
        let mut store = Store::default();
        let mut control = checkpoint_action_control(
            Stage::PreparePreview,
            usize::MAX,
            Interruption::CancelRequested,
        );
        let reply = super::prepare_with_control(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/exports",
            &json!({"report_kind":"trace","target_path":"review.html"}),
            &mut control,
        )
        .unwrap();
        let preview = &reply.value["result"]["preview"];
        let expected: Vec<Value> = snapshot
            .items
            .iter()
            .map(|item| {
                json!({
                    "resource_id":item.metadata["resource_id"],"sha256":item.captured.sha256
                })
            })
            .collect();
        assert_eq!(preview["input_hashes"], json!(expected));
        assert_eq!(preview["input_hashes"].as_array().unwrap().len(), 100);
        assert_eq!(store.get_preview(preview["preview_id"].as_str().unwrap()).unwrap(), *preview);
        assert_eq!(reply.value["state"], "succeeded");
        assert!(!control.stopped);
        assert_eq!(control.matching_occurrences, 1);
        super::super::contract::validate(reply.schema, &reply.value).unwrap();
        assert_eq!(checkpoint_action_project_bytes(directory.path()), before);
    }

    /// The 101st relevant policy input rejects the entire trace export and preserves unrelated receipts and every file.
    #[test]
    fn checkpoint_trace_export_rejects_one_hundred_one_without_a_successful_prefix() {
        use super::super::preparation::{Interruption, Stage, WorkError};
        let (directory, root, mut snapshot) = checkpoint_action_fixture(101, 0);
        let before = checkpoint_action_project_bytes(directory.path());
        assert_eq!(snapshot.items.len(), 101);
        let mut store = Store::default();
        let (keeper_id, keeper) = checkpoint_action_keeper(&mut store, &root, &snapshot);
        let mut control = checkpoint_action_control(
            Stage::PreparePreview,
            usize::MAX,
            Interruption::CancelRequested,
        );
        let outcome = super::prepare_with_control(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/exports",
            &json!({"report_kind":"trace","target_path":"review.html"}),
            &mut control,
        );
        match outcome {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "report-binds-too-many-inputs"),
            Err(WorkError::Interrupted(_)) => {
                panic!("ordinary input-cap rejection must not become interruption")
            }
            Ok(_) => panic!("101 relevant inputs must not become a successful prefix export"),
        }
        assert!(!control.stopped);
        assert_eq!(store.get_preview(&keeper_id).unwrap(), keeper);
        assert_eq!(checkpoint_action_project_bytes(directory.path()), before);
    }

    /// Total registered capture cardinality does not narrow the independently computed trace-input selection.
    #[test]
    fn checkpoint_trace_export_accepts_one_relevant_input_among_one_hundred_one_registrations() {
        use super::super::preparation::{Interruption, Stage};
        let (directory, root, mut snapshot) = checkpoint_action_fixture(1, 100);
        let before = checkpoint_action_project_bytes(directory.path());
        assert_eq!(snapshot.items.len(), 101);
        assert!(snapshot.items.iter().all(|item| item.validation["state"] == "valid"));
        let policy = &snapshot.items[0];
        let expected =
            json!([{"resource_id":policy.metadata["resource_id"],"sha256":policy.captured.sha256}]);
        let mut store = Store::default();
        let mut control = checkpoint_action_control(
            Stage::PreparePreview,
            usize::MAX,
            Interruption::CancelRequested,
        );
        let reply = super::prepare_with_control(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/exports",
            &json!({"report_kind":"trace","target_path":"review.html"}),
            &mut control,
        )
        .unwrap();
        let preview = &reply.value["result"]["preview"];
        assert_eq!(preview["input_hashes"], expected);
        assert_eq!(store.get_preview(preview["preview_id"].as_str().unwrap()).unwrap(), *preview);
        assert_eq!(reply.value["state"], "succeeded");
        super::super::contract::validate(reply.schema, &reply.value).unwrap();
        assert_eq!(checkpoint_action_project_bytes(directory.path()), before);
    }

    /// Stops before the real local preview and after preparation preserve the typed cause and global sentinel receipt.
    #[test]
    fn checkpoint_export_interruption_discards_local_previews_before_global_transfer() {
        use super::super::preparation::{Interruption, Stage, WorkError};
        let (directory, root, mut snapshot) = checkpoint_action_fixture(1, 0);
        let before = checkpoint_action_project_bytes(directory.path());
        for reason in
            [Interruption::CancelRequested, Interruption::Shutdown, Interruption::DeadlineExceeded]
        {
            let expected_reason = std::mem::discriminant(&reason);
            for (stage, occurrence) in [(Stage::PreparePreview, 1), (Stage::RetainPrepared, 1)] {
                let selected_stage = std::mem::discriminant(&stage);
                let mut global = Store::default();
                let (keeper_id, keeper) = checkpoint_action_keeper(&mut global, &root, &snapshot);
                let operation = global.begin("export").unwrap();
                let id = operation["operation_id"].as_str().unwrap();
                assert!(global.running(id).unwrap());
                let mut control = checkpoint_action_control(stage, occurrence, reason);
                let mut local = Store::default();
                let outcome = super::prepare_with_control(
                    &mut local,
                    &root,
                    &mut snapshot,
                    "POST",
                    "/api/v1/exports",
                    &json!({"report_kind":"trace","target_path":"review.html"}),
                    &mut control,
                );
                match outcome {
                    Err(WorkError::Interrupted(actual)) => {
                        assert_eq!(std::mem::discriminant(&actual), expected_reason);
                    }
                    Err(WorkError::Failed(error)) => {
                        panic!("stop was collapsed to ordinary failure: {error}")
                    }
                    Ok(_) => panic!("interrupted export returned a prepared reply"),
                }
                assert!(control.stopped);
                assert_eq!(control.matching_occurrences, occurrence);
                assert_eq!(control.events.last(), Some(&selected_stage));
                drop(local);
                global.finish(id, Err(Error::invalid()), true).unwrap();
                let terminal = global.operation(id).unwrap();
                assert_eq!(terminal["state"], "cancelled");
                assert!(terminal["result"].is_null());
                assert!(terminal["error"].is_null());
                assert!(terminal["progress"].is_null());
                assert_eq!(global.get_preview(&keeper_id).unwrap(), keeper);
                super::super::contract::validate("Operation", &terminal).unwrap();
                assert_eq!(checkpoint_action_project_bytes(directory.path()), before);
            }
        }
    }
    /// Default V1 preparation rejects index2 before unregistered reads; explicit V2 consumes it without an implicit commit.
    #[test]
    fn selected_major_guards_direct_preparation_before_resource_io() {
        let project = tempfile::tempdir().expect("synthetic project");
        let index =
            json!({"schema_version":"forge.workspace/2","label":"Explicit version","resources":[]});
        let original = serde_json::to_vec(&index).expect("fixture index");
        std::fs::write(project.path().join(INDEX_PATH), &original).expect("write index2");
        let root = Root::open(project.path()).expect("confined root");
        let mut snapshot = Snapshot::capture(&root).expect("explicit internal capture");
        let mut store = Store::default();
        let missing = json!({"role":"policy-source","path":"missing.md"});
        let error = super::prepare(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/resources/register",
            &missing,
        )
        .err()
        .expect("V1 must reject index2");
        assert_eq!(error.code, "invalid-request");
        let error = super::prepare_with_control(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/resources/register",
            &missing,
            &mut NoopControl,
        )
        .err()
        .expect("controlled V1 must reject index2")
        .into_error();
        assert_eq!(error.code, "invalid-request");
        std::fs::write(project.path().join("opaque.bin"), [0xff, 0x00]).expect("opaque bytes");
        let reply = super::prepare_for_api(
            &mut store,
            &root,
            &mut snapshot,
            "POST",
            "/api/v1/resources/register",
            &json!({"role":"lifecycle-source","path":"opaque.bin"}),
            ApiMajor::V2,
        )
        .expect("selected major2 canonical admitted registration");
        assert_eq!(reply.value["preview"]["operation_type"], "workspace-index-update");
        assert_eq!(
            std::fs::read(project.path().join(INDEX_PATH)).expect("unchanged index"),
            original
        );
        assert_eq!(
            std::fs::read(project.path().join("opaque.bin")).expect("unchanged source"),
            [0xff, 0x00]
        );
        assert!(!project.path().join("missing.md").exists());
    }
}
