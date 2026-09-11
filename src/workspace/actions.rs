//! Material requests prepare a single effect; this adapter never writes files.
use super::contract::{Error, Result};
use super::effects::{Reply, Store};
use super::index::{INDEX_PATH, Index, Resource, Role};
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
fn output_target(snapshot: &Snapshot, path: &str, allowed: Option<Role>) -> Result<()> {
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

#[allow(clippy::too_many_lines)] // One audited route-to-effect table.
pub(crate) fn prepare(
    store: &mut Store,
    root: &Root,
    snapshot: &mut Snapshot,
    method: &str,
    path: &str,
    request: &Value,
) -> Result<Reply> {
    match (method, path) {
        ("POST", "/api/v1/applicability/initializations" | "/api/v1/mapping/initializations") => {
            let target = text(request, "target_path")?;
            output_target(snapshot, target, None)?;
            if root.target(target)?.base.is_some() {
                return Err(conflict());
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
            let path = text(request, "path")?;
            let role: Role =
                serde_json::from_value(request["role"].clone()).map_err(|_| Error::invalid())?;
            let captured = root.read(path, 10 * 1024 * 1024)?;
            if snapshot.items.iter().any(|item| item.captured.identity == captured.identity) {
                return Err(Error::containment());
            }
            let key = request["key"].as_str().map_or_else(
                || {
                    let stem = path
                        .rsplit('/')
                        .next()
                        .unwrap_or("resource")
                        .split('.')
                        .next()
                        .unwrap_or("resource")
                        .to_ascii_lowercase();
                    let key = stem
                        .chars()
                        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                        .collect::<String>();
                    // The index pattern requires an alphanumeric last character,
                    // so the length cap is applied before the final trim.
                    let key: String = key.chars().take(64).collect();
                    key.trim_end_matches('-').to_owned()
                },
                str::to_owned,
            );
            let registration = Resource { key, role, path: path.to_owned() };
            if !(super::services::validate_bytes(&registration, &captured.bytes)
                || (role == Role::ApplicabilityReport
                    && snapshot.analysis.as_ref().is_some_and(|analysis| {
                        serde_json::to_value(analysis).ok()
                            == super::contract::parse(&captured.bytes, 10 * 1024 * 1024, 64 * 1024)
                                .ok()
                    })))
            {
                return Err(validation_error());
            }
            let mut index = snapshot.index.clone();
            index.resources.push(registration.clone());
            let bytes = index.bytes()?;
            Index::parse(&bytes)?;
            let mut preview =
                store.preview(root, snapshot, INDEX_PATH, "workspace-index-update", bytes, &[])?;
            // The selected unregistered file is separately bound as an extra
            // dependency by the preview store, before any index can be committed.
            store.bind_external(&mut preview, &registration, captured)?;
            Ok(preview_reply(preview))
        }
        ("POST", "/api/v1/resources/upload") => {
            let path = text(request, "target_path")?;
            output_target(snapshot, path, None)?;
            let role =
                serde_json::from_value(request["role"].clone()).map_err(|_| Error::invalid())?;
            let encoded = text(request, "content_base64")?;
            if encoded.len() > 13_981_016 {
                return Err(Error::invalid());
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .map_err(|_| Error::invalid())?;
            if bytes.len() > 10 * 1024 * 1024 {
                return Err(Error::invalid());
            }
            let registration = Resource { key: "upload".into(), role, path: path.into() };
            if !super::services::validate_bytes(&registration, &bytes) {
                return Err(validation_error());
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
                return Err(conflict());
            }
            let validation =
                super::domain::validate_draft(snapshot, mapping, &request["manifest"])?;
            if validation["state"] != "valid" {
                return Err(validation_error());
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
            let result = super::domain::convert(snapshot, source, kind)?;
            if !result.secondary_outputs.is_empty() {
                return Err(validation_error());
            }
            let count = result.statistics.requirements_extracted;
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
                return Err(validation_error());
            }
            output_target(snapshot, target, Some(Role::ApplicabilityReport))?;
            let mut bytes = super::contract::encode(analysis, 10 * 1024 * 1024 - 1, true)?;
            bytes.push(b'\n');
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
            let built = super::domain::mapping(snapshot)?;
            let manifest_item = super::domain::mapping_manifest(snapshot)?;
            let manifest = crate::mapping::manifest::parse(&manifest_item.captured.bytes)
                .map_err(|_| validation_error())?;
            let target = "mapping-collection.json";
            output_target(snapshot, target, Some(Role::MappingCollection))?;
            if manifest_item.registration.path.eq_ignore_ascii_case(target) {
                return Err(Error::containment());
            }
            let inputs = manifest_inputs(
                snapshot,
                &manifest_item.registration.path,
                &mapping_references(&manifest),
            )?;
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
            let summary = match kind {
                "applicability-gap" => snapshot.classification_counts()?,
                "mapping-collection" => {
                    let built = super::domain::mapping(snapshot)?;
                    json!({"eligible_controls":built.report.target_controls.eligible,"referenced_controls":built.report.target_controls.referenced})
                }
                "trace" => super::domain::trace_counts(snapshot)?,
                _ => return Err(Error::invalid()),
            };
            let bytes = super::reports::render(snapshot, kind, summary)?;
            let inputs = super::reports::inputs(snapshot, kind)?;
            let preview = store.preview(root, snapshot, target, "report-export", bytes, &inputs)?;
            Ok(operation_reply(store.completed("export",json!({"operation_id":"op_000000000000","preview":preview,"redaction_summary":{"removed_categories":["reviewer-names","absolute-paths","source-excerpts","secrets"]}}))?))
        }
        _ => Err(Error::new("not-found", "The requested effect operation was not found.", false)),
    }
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
    }
}
