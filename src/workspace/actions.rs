//! Material requests prepare a single effect; this adapter never writes files.
use super::contract::{Error, Result};
use super::effects::{Reply, Store};
use super::index::{INDEX_PATH, Index, Resource, Role};
use super::root::{Root, conflict};
use super::services::Snapshot;
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
                    key.trim_matches('-').chars().take(64).collect()
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
            let mut bytes = super::contract::encode(&request["manifest"], 1024 * 1024 - 1, true)?;
            bytes.push(b'\n');
            let inputs: Vec<_> = snapshot.items.iter().collect();
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
            let inputs: Vec<_> = snapshot.items.iter().collect();
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
            let inputs: Vec<_> = snapshot.items.iter().collect();
            let preview = store.preview(root, snapshot, target, "report-export", bytes, &inputs)?;
            Ok(operation_reply(store.completed("export",json!({"operation_id":"op_000000000000","preview":preview,"redaction_summary":{"removed_categories":["reviewer-names","absolute-paths","source-excerpts","secrets"]}}))?))
        }
        _ => Err(Error::new("not-found", "The requested effect operation was not found.", false)),
    }
}
