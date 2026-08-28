//! Stable-identity Assessment Results revision impact analysis.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use uuid::Uuid;

use super::context::LoadedContext;
use super::manifest::{ConclusionType, SubjectType};
use super::model::{FORGE_ASSESSMENT_RESULTS_NS, ObjectSnapshot, UUID_SEED_VERSION};
use super::report::{AssessmentResultsReport, BaselineFinding};
use crate::ForgeError;
use crate::json_strict::{self, Limits};

pub const CODE_OBJECT_ADDED: &str = "object-added";
pub const CODE_OBJECT_REMOVED: &str = "object-removed";
pub const CODE_CONTENT_CHANGED: &str = "content-changed";
pub const CODE_RATIONALE_CHANGED: &str = "rationale-changed";
pub const CODE_STATUS_CHANGED: &str = "status-changed";
pub const CODE_STALE_SUBJECT: &str = "stale-subject";
pub const CODE_UPSTREAM_CHANGED: &str = "upstream-fingerprint-changed";

const MAX_BASELINE_BYTES: u64 = 50 * 1024 * 1024;

/// Compare a prior FORGE Assessment Results artifact to the current reviewed build.
///
/// # Errors
///
/// Returns an error when the baseline is oversized, invalid, not a FORGE-produced
/// schema-valid artifact, or cannot be compared by stable identity.
pub fn analyze(
    baseline_bytes: &[u8],
    current_snapshots: &BTreeMap<(ConclusionType, String), ObjectSnapshot>,
    context: &LoadedContext,
    report: &mut AssessmentResultsReport,
) -> Result<(), ForgeError> {
    if baseline_bytes.len() as u64 > MAX_BASELINE_BYTES {
        return Err(error(format!("baseline exceeds the {MAX_BASELINE_BYTES} byte limit")));
    }
    let baseline = json_strict::parse_value(
        baseline_bytes,
        "Assessment Results baseline",
        Limits { max_depth: 128, max_string_bytes: 1024 * 1024 },
    )
    .map_err(|cause| error(cause.to_string()))?;
    super::validate_completed_json(&baseline)?;
    let prior_snapshots = extract_snapshots(&baseline)?;

    let all_keys: BTreeSet<_> =
        prior_snapshots.keys().chain(current_snapshots.keys()).cloned().collect();
    for (object_type, key) in all_keys {
        match (
            prior_snapshots.get(&(object_type, key.clone())),
            current_snapshots.get(&(object_type, key.clone())),
        ) {
            (None, Some(current)) => push(
                report,
                CODE_OBJECT_ADDED,
                Some(object_type),
                &key,
                None,
                Some(current.uuid.clone()),
            ),
            (Some(prior), None) => push(
                report,
                CODE_OBJECT_REMOVED,
                Some(object_type),
                &key,
                Some(prior.uuid.clone()),
                None,
            ),
            (Some(prior), Some(current)) => {
                if prior.uuid != current.uuid {
                    return Err(error(format!(
                        "baseline stable identity for {} '{}' resolves to a different UUID",
                        object_type.as_str(),
                        bounded(&key)
                    )));
                }
                if prior.content_sha256 != current.content_sha256 {
                    push(
                        report,
                        CODE_CONTENT_CHANGED,
                        Some(object_type),
                        &key,
                        Some(prior.content_sha256.clone()),
                        Some(current.content_sha256.clone()),
                    );
                }
                if prior.rationale_sha256 != current.rationale_sha256 {
                    push(
                        report,
                        CODE_RATIONALE_CHANGED,
                        Some(object_type),
                        &key,
                        Some(prior.rationale_sha256.clone()),
                        Some(current.rationale_sha256.clone()),
                    );
                }
                if prior.status != current.status {
                    push(
                        report,
                        CODE_STATUS_CHANGED,
                        Some(object_type),
                        &key,
                        prior.status.clone(),
                        current.status.clone(),
                    );
                }
            }
            (None, None) => {}
        }
    }

    compare_upstream(&baseline, context, report)?;
    find_stale_references(&baseline, context, report)?;
    Ok(())
}

fn extract_snapshots(
    baseline: &Value,
) -> Result<BTreeMap<(ConclusionType, String), ObjectSnapshot>, ForgeError> {
    let results = baseline
        .pointer("/assessment-results/results")
        .and_then(Value::as_array)
        .ok_or_else(|| error("baseline results array is required"))?;
    let mut snapshots = BTreeMap::new();
    for result in results {
        for (field, object_type) in [
            ("observations", ConclusionType::Observation),
            ("findings", ConclusionType::Finding),
            ("risks", ConclusionType::Risk),
        ] {
            for object in result.get(field).and_then(Value::as_array).into_iter().flatten() {
                let key = required_prop(object, "stable-key")?;
                let uuid = required_string(object.get("uuid"), "baseline conclusion UUID")?;
                Uuid::parse_str(&uuid).map_err(|_| error("baseline conclusion UUID is invalid"))?;
                let content_sha256 = required_prop(object, "content-sha256")?;
                let rationale_sha256 = required_prop(object, "rationale-sha256")?;
                json_strict::validate_lowercase_sha256("baseline content-sha256", &content_sha256)
                    .map_err(error)?;
                json_strict::validate_lowercase_sha256(
                    "baseline rationale-sha256",
                    &rationale_sha256,
                )
                .map_err(error)?;
                let status = match object_type {
                    ConclusionType::Observation => None,
                    ConclusionType::Finding => {
                        let state = required_string(
                            object.pointer("/target/status/state"),
                            "baseline finding state",
                        )?;
                        let reason = object
                            .pointer("/target/status/reason")
                            .and_then(Value::as_str)
                            .unwrap_or("none");
                        Some(format!("{state}:{reason}"))
                    }
                    ConclusionType::Risk => {
                        Some(required_string(object.get("status"), "baseline risk status")?)
                    }
                };
                if snapshots
                    .insert(
                        (object_type, key.clone()),
                        ObjectSnapshot { uuid, content_sha256, rationale_sha256, status },
                    )
                    .is_some()
                {
                    return Err(error(format!(
                        "baseline duplicates {} stable key '{}'",
                        object_type.as_str(),
                        bounded(&key)
                    )));
                }
            }
        }
    }
    Ok(snapshots)
}

fn compare_upstream(
    baseline: &Value,
    context: &LoadedContext,
    report: &mut AssessmentResultsReport,
) -> Result<(), ForgeError> {
    let metadata = baseline
        .pointer("/assessment-results/metadata")
        .ok_or_else(|| error("baseline metadata is required"))?;
    let prior_values = props_named(metadata, "context-sha256");
    let mut prior = BTreeMap::new();
    for value in prior_values {
        let (kind, hash) = value
            .split_once(':')
            .ok_or_else(|| error("baseline context-sha256 property is malformed"))?;
        json_strict::validate_lowercase_sha256("baseline context hash", hash).map_err(error)?;
        if prior.insert(kind.to_string(), hash.to_string()).is_some() {
            return Err(error(format!(
                "baseline duplicates context fingerprint for '{}'",
                bounded(kind)
            )));
        }
    }
    for identity in context.artifact_identities() {
        let Some(previous) = prior.get(identity.kind) else {
            return Err(error(format!(
                "baseline is missing FORGE context fingerprint for '{}'",
                identity.kind
            )));
        };
        if previous != &identity.sha256 {
            push(
                report,
                CODE_UPSTREAM_CHANGED,
                None,
                identity.kind,
                Some(previous.clone()),
                Some(identity.sha256.clone()),
            );
        }
    }
    let prior_evidence_hashes = props_named(metadata, "evidence-index-sha256");
    let prior_evidence_hash = match prior_evidence_hashes.as_slice() {
        [] => None,
        [hash] => {
            json_strict::validate_lowercase_sha256("baseline evidence index hash", hash)
                .map_err(error)?;
            Some((*hash).to_string())
        }
        _ => return Err(error("baseline duplicates the evidence index fingerprint")),
    };
    if prior_evidence_hash.as_ref() != context.evidence_index_sha256.as_ref() {
        push(
            report,
            CODE_UPSTREAM_CHANGED,
            None,
            "evidence-index",
            prior_evidence_hash,
            context.evidence_index_sha256.clone(),
        );
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "one bounded traversal keeps all supported stale-reference categories visible"
)]
fn find_stale_references(
    baseline: &Value,
    context: &LoadedContext,
    report: &mut AssessmentResultsReport,
) -> Result<(), ForgeError> {
    let results = baseline
        .pointer("/assessment-results/results")
        .and_then(Value::as_array)
        .ok_or_else(|| error("baseline results array is required"))?;
    let mut stale = BTreeSet::new();
    for result in results {
        if let Some(selections) =
            result.pointer("/reviewed-controls/control-selections").and_then(Value::as_array)
        {
            for control in selections
                .iter()
                .filter_map(|selection| selection.get("include-controls"))
                .filter_map(Value::as_array)
                .flatten()
            {
                if let Some(id) = control.get("control-id").and_then(Value::as_str)
                    && (!context.controls.contains(id) || !context.reviewed_controls.contains(id))
                {
                    stale.insert(format!("control:{id}"));
                }
            }
        }
        if let Some(selections) = result
            .pointer("/reviewed-controls/control-objective-selections")
            .and_then(Value::as_array)
        {
            for objective in selections
                .iter()
                .filter_map(|selection| selection.get("include-objectives"))
                .filter_map(Value::as_array)
                .flatten()
            {
                if let Some(id) = objective.get("objective-id").and_then(Value::as_str)
                    && (!context.objectives.contains(id)
                        || !context.reviewed_objectives.contains(id))
                {
                    stale.insert(format!("objective:{id}"));
                }
            }
        }
        for observation in
            result.get("observations").and_then(Value::as_array).into_iter().flatten()
        {
            for subject in
                observation.get("subjects").and_then(Value::as_array).into_iter().flatten()
            {
                let Some(kind) = subject.get("type").and_then(Value::as_str) else {
                    continue;
                };
                let Some(uuid) = subject.get("subject-uuid").and_then(Value::as_str) else {
                    continue;
                };
                if let Ok(subject_type) = parse_subject_type(kind)
                    && !context.subject_is_in_scope(subject_type, uuid)
                {
                    stale.insert(format!("subject:{kind}:{uuid}"));
                }
            }
            for origin in observation.get("origins").and_then(Value::as_array).into_iter().flatten()
            {
                for task in
                    origin.get("related-tasks").and_then(Value::as_array).into_iter().flatten()
                {
                    if let Some(uuid) = task.get("task-uuid").and_then(Value::as_str)
                        && !context.tasks.contains(uuid)
                    {
                        stale.insert(format!("task:{uuid}"));
                    }
                }
            }
            for evidence in
                observation.get("relevant-evidence").and_then(Value::as_array).into_iter().flatten()
            {
                if let Ok(key) = required_prop(evidence, "stable-key")
                    && !context.evidence.contains_key(&key)
                {
                    stale.insert(format!("evidence:{key}"));
                }
            }
        }
        for finding in result.get("findings").and_then(Value::as_array).into_iter().flatten() {
            if let (Some(kind), Some(id)) = (
                finding.pointer("/target/type").and_then(Value::as_str),
                finding.pointer("/target/target-id").and_then(Value::as_str),
            ) {
                let current = match kind {
                    "statement-id" => context.statements.contains(id),
                    "objective-id" => {
                        context.objectives.contains(id) && context.reviewed_objectives.contains(id)
                    }
                    _ => false,
                };
                if !current {
                    stale.insert(format!("finding-target:{kind}:{id}"));
                }
            }
            if let Some(uuid) = finding.get("implementation-statement-uuid").and_then(Value::as_str)
                && !context.implementation_statements.contains(uuid)
            {
                stale.insert(format!("implementation:{uuid}"));
            }
        }
    }
    for key in stale {
        push(report, CODE_STALE_SUBJECT, None, &key, None, None);
    }
    Ok(())
}

fn props_named<'a>(value: &'a Value, name: &str) -> Vec<&'a str> {
    value
        .get("props")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|prop| {
            prop.get("ns").and_then(Value::as_str) == Some(FORGE_ASSESSMENT_RESULTS_NS)
                && prop.get("name").and_then(Value::as_str) == Some(name)
        })
        .filter_map(|prop| prop.get("value").and_then(Value::as_str))
        .collect()
}

fn required_prop(value: &Value, name: &str) -> Result<String, ForgeError> {
    let values = props_named(value, name);
    match values.as_slice() {
        [value] if !value.trim().is_empty() => Ok((*value).to_string()),
        [] => Err(error(format!("baseline object is missing FORGE property '{name}'"))),
        _ => Err(error(format!("baseline object duplicates FORGE property '{name}'"))),
    }
}

fn required_string(value: Option<&Value>, label: &str) -> Result<String, ForgeError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| error(format!("{label} must be a non-empty string")))
}

fn parse_subject_type(value: &str) -> Result<SubjectType, ForgeError> {
    match value {
        "component" => Ok(SubjectType::Component),
        "inventory-item" => Ok(SubjectType::InventoryItem),
        "location" => Ok(SubjectType::Location),
        "party" => Ok(SubjectType::Party),
        "user" => Ok(SubjectType::User),
        "resource" => Ok(SubjectType::Resource),
        _ => Err(error("baseline contains an unsupported subject type")),
    }
}

fn push(
    report: &mut AssessmentResultsReport,
    code: &str,
    object_type: Option<ConclusionType>,
    key: &str,
    old_fingerprint: Option<String>,
    new_fingerprint: Option<String>,
) {
    let kind = object_type.map_or("context", ConclusionType::as_str);
    let mut seed = Vec::new();
    for value in [UUID_SEED_VERSION, "baseline-finding", code, kind, key] {
        seed.extend_from_slice(&(value.len() as u64).to_be_bytes());
        seed.extend_from_slice(value.as_bytes());
    }
    let id = Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &seed).to_string();
    report.findings.push(BaselineFinding {
        id,
        code: code.to_string(),
        object_type,
        key: key.to_string(),
        old_fingerprint,
        new_fingerprint,
    });
}

fn bounded(value: &str) -> String {
    json_strict::bounded(value)
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::AssessmentResultsBuild(message.into())
}
