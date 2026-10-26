//! Add-only reviewed-risk export into one existing `forge.poam/1` workflow plan.
//!
//! The 2026-10-25 owner disposition for F10 (PRD 063 S-3) approves export into an
//! existing plan as add-only: new `planned` items with new stable identities are
//! appended, no existing item, milestone, history, terminal or superseding record
//! changes, and the M-13 baseline reports the new items as added. Rebinding an
//! existing plan to a different Assessment Results generation is refused. A
//! reviewed risk already referenced by an existing item is refused as a whole input.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Map, Value};

use super::manifest::{self, SourceKind};
use super::risk_authoring::{self, PreparedRiskAuthoring};
use super::workflow::WorkflowManifest;
use super::{identity, source, workflow, workflow_model};
use crate::ForgeError;
use crate::evidence_capture::{CaptureRole, CaptureSession};
use crate::json_strict::{self, Limits};

/// Prepare one complete add-only plan from an existing plan and a reviewed-risk request.
///
/// The request has the unchanged `forge.poam-risk-authoring/1` profile. Its workflow
/// carries only the new items, the existing plan's exact source, roles and parties,
/// and caller-supplied document metadata under the same plan key. The output is the
/// existing plan's original values with that document metadata and the new items
/// appended; no owner, date, actor, history or identity is inferred.
/// # Errors
/// Refuses invalid paths, a non-workflow or empty existing plan, source mismatch or
/// staleness, changed roles/parties, colliding identities, already-referenced risks,
/// any existing-content change, S-3 union violations or changed originals.
pub(crate) fn prepare(
    plan_path: &Path,
    authoring_relative: &Path,
    as_of: &str,
    output_relative: Option<&Path>,
) -> Result<PreparedRiskAuthoring, ForgeError> {
    workflow::validate_authoring_date(as_of)
        .map_err(|_| error("as-of must be a canonical actual full date"))?;
    let (root, plan_relative) = risk_authoring::declaration_root(plan_path)
        .map_err(|_| error("existing plan path must be a normalized portable JSON filename"))?;
    risk_authoring::validate_json_relative(authoring_relative, false)?;
    if let Some(output) = output_relative {
        risk_authoring::validate_json_relative(output, true)?;
    }
    let mut capture = CaptureSession::new(&root)
        .map_err(|_| error("risk authoring root is unsafe or unavailable"))?;
    if let Some(output) = output_relative {
        capture
            .reserve_output(output)
            .map_err(|_| error("risk authoring output namespace is invalid"))?;
        risk_authoring::preflight_destination(&capture, output)?;
    }
    let plan = capture
        .required(&plan_relative, CaptureRole::WorkflowDeclaration, manifest::MAX_MANIFEST_BYTES)
        .map_err(|_| error("existing plan original is unsafe or unavailable"))?;
    let existing = Existing::parse(plan.bytes())?;
    let authoring = capture
        .required(
            authoring_relative,
            CaptureRole::RiskAuthoringRequest,
            manifest::MAX_MANIFEST_BYTES,
        )
        .map_err(|_| error("risk authoring original is unsafe or unavailable"))?;
    let request = risk_authoring::parse_request(authoring.bytes())?;
    let addition = workflow::parse(&risk_authoring::encode(&request["workflow"], false)?)
        .map_err(|_| error("risk authoring does not satisfy the current workflow rules"))?;
    check_binding(&existing, &request["workflow"], &addition)?;
    check_identities(&existing.typed, &addition)?;
    check_risks_unreferenced(&existing.typed, &addition)?;
    let merged = merge(&existing.value, &request["workflow"])?;
    let mut ledger = Map::new();
    ledger.insert("workflow".to_string(), merged.clone());
    ledger.insert("reviewed_risks".to_string(), request["reviewed_risks"].clone());
    let accounting = risk_authoring::preflight(&Value::Object(ledger), &mut capture)?;
    let combined = workflow::parse(&risk_authoring::encode(&merged, false)?)
        .map_err(|_| error("add-only plan does not satisfy the current workflow rules"))?;
    workflow::validate_successor(&existing.typed, &combined)
        .map_err(|_| error("add-only export would change existing plan content"))?;
    let actual_path = capture.root().join(&plan_relative);
    let sources = source::load_with_capture(&actual_path, &existing.typed.source, &mut capture)
        .map_err(|_| error(STALE_SOURCE))?;
    for item in &existing.typed.items {
        sources.validate_selection(&item.source_refs).map_err(|_| error(STALE_SOURCE))?;
    }
    risk_authoring::validate_union(&request, &addition, &sources)?;
    let native = workflow_model::render(&combined)
        .map_err(|_| error("add-only plan does not satisfy the current native projection"))?;
    if native.len() > accounting.native_upper {
        return Err(error("risk native projection exceeded its conservative admission"));
    }
    drop(native);
    verify_add_only(&existing.value, &merged)?;
    let bytes = risk_authoring::encode(&merged, true)?;
    if bytes.len() != accounting.workflow_bytes {
        return Err(error("risk authoring encoding differs from its complete admission"));
    }
    risk_authoring::verify_encoding(&bytes, &merged)?;
    risk_authoring::finish(bytes, capture)
}

/// Fixed refusal for a stale existing source or a requested rebinding.
const STALE_SOURCE: &str = "existing plan source originals are stale or invalid; add-only export never rebinds a plan to another Assessment Results generation";

/// Existing plan original values plus their typed full-workflow admission.
struct Existing {
    /// Strict duplicate-free original JSON values, preserved verbatim in output.
    value: Value,
    /// Current full workflow admission of the same bytes, including D064 records.
    typed: WorkflowManifest,
}

impl Existing {
    /// Admit only a valid nonempty workflow; an empty scaffold keeps its own mode.
    fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        let value = json_strict::parse_value(
            bytes,
            "existing POA&M plan",
            Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
        )
        .map_err(|_| error("existing plan is not bounded duplicate-free JSON"))?;
        let typed = workflow::parse(bytes).map_err(|_| {
            error("existing plan must be a valid nonempty forge.poam/1 workflow; use --scaffold for an empty scaffold")
        })?;
        Ok(Self { value, typed })
    }
}

/// Require the same plan key, exact source tuple and exact role/party declarations.
fn check_binding(
    existing: &Existing,
    workflow: &Value,
    addition: &WorkflowManifest,
) -> Result<(), ForgeError> {
    if addition.document.key != existing.typed.document.key {
        return Err(error("risk authoring plan key differs from the existing plan"));
    }
    if !risk_authoring::same_source(&addition.source, &existing.typed.source) {
        return Err(error(
            "risk authoring source differs from the existing plan; add-only export never rebinds a plan to another Assessment Results generation",
        ));
    }
    if workflow["roles"] != existing.value["roles"]
        || workflow["parties"] != existing.value["parties"]
    {
        return Err(error("add-only export requires the existing plan's exact roles and parties"));
    }
    Ok(())
}

/// Require new item keys and new item/milestone UUIDs disjoint from every existing identity.
fn check_identities(
    existing: &WorkflowManifest,
    addition: &WorkflowManifest,
) -> Result<(), ForgeError> {
    let keys: BTreeSet<_> = existing.items.iter().map(|item| item.key.as_str()).collect();
    if addition.items.iter().any(|item| keys.contains(item.key.as_str())) {
        return Err(error("new item keys must not reuse an existing plan item key"));
    }
    let used = uuids(existing);
    if uuids(addition).iter().any(|value| used.contains(value)) {
        return Err(error("new item identities collide with existing plan identities"));
    }
    Ok(())
}

/// Collect every stable item and milestone identity from the frozen key protocol.
fn uuids(plan: &WorkflowManifest) -> BTreeSet<uuid::Uuid> {
    let key = plan.document.key.as_str();
    let mut values = BTreeSet::new();
    for item in &plan.items {
        values.insert(identity::item(key, &item.key));
        for step in &item.milestones {
            values.insert(identity::milestone(key, &item.key, &step.key));
        }
    }
    values
}

/// Refuse any new risk selection already referenced by an existing item.
///
/// Matching uses the source domain plus either the stable key or the native UUID,
/// so neither identifier alone can hide a repeat. Whether such a repeat could be a
/// legitimate re-export is an open owner question; it stays refused.
fn check_risks_unreferenced(
    existing: &WorkflowManifest,
    addition: &WorkflowManifest,
) -> Result<(), ForgeError> {
    let referenced: Vec<_> = existing
        .items
        .iter()
        .flat_map(|item| &item.source_refs)
        .filter(|reference| reference.kind == SourceKind::Risk)
        .collect();
    let repeated = addition
        .items
        .iter()
        .flat_map(|item| &item.source_refs)
        .any(|new| referenced.iter().any(|old| old.key == new.key || old.uuid == new.uuid));
    if repeated {
        return Err(error("a reviewed risk is already referenced by an existing plan item"));
    }
    Ok(())
}

/// Append the new item values to the existing original values and take the request document.
fn merge(existing: &Value, workflow: &Value) -> Result<Value, ForgeError> {
    let mut merged = existing.clone();
    merged["document"] = workflow["document"].clone();
    let additions = workflow["items"]
        .as_array()
        .ok_or_else(|| error("risk authoring expected a closed array"))?;
    merged["items"]
        .as_array_mut()
        .ok_or_else(|| error("existing plan expected a closed item array"))?
        .extend(additions.iter().cloned());
    Ok(merged)
}

/// Final oracle: every existing value except document metadata is retained exactly.
///
/// Existing items, with their milestones, history, terminal assertions and
/// supersession links, form an exact prefix of the output items.
fn verify_add_only(existing: &Value, merged: &Value) -> Result<(), ForgeError> {
    let unchanged = ["schema_version", "source", "roles", "parties"]
        .iter()
        .all(|field| existing.get(*field) == merged.get(*field));
    let before = existing["items"].as_array();
    let after = merged["items"].as_array();
    let prefix = before
        .zip(after)
        .is_some_and(|(before, after)| after.len() > before.len() && after.starts_with(before));
    if !unchanged || !prefix {
        return Err(error("add-only output differs from the existing plan content"));
    }
    Ok(())
}

/// Emit only a fixed typed invalid reason, never author prose or private native errors.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}

/// Isolated add-only predicates; end-to-end capture is covered by the CLI tests.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Minimal existing values with one terminal-shaped item for prefix comparisons.
    fn existing() -> Value {
        json!({"schema_version":"forge.poam/1","document":{"key":"plan"},"source":{"a":1},
            "roles":[{"id":"owner"}],"parties":[{"key":"caller"}],
            "items":[{"key":"old","state":"completed-asserted"}]})
    }

    /// Appending keeps the existing prefix and takes only the request document.
    #[test]
    fn merge_appends_new_items_after_unchanged_existing_values() {
        let workflow = json!({"document":{"key":"plan","version":"2"},"items":[{"key":"new"}]});
        let merged = merge(&existing(), &workflow).unwrap();
        assert_eq!(merged["document"], workflow["document"]);
        assert_eq!(merged["items"][0], existing()["items"][0]);
        assert_eq!(merged["items"][1], json!({"key":"new"}));
        verify_add_only(&existing(), &merged).unwrap();
    }

    /// Any changed existing field, rewritten item or absent addition is refused.
    #[test]
    fn add_only_oracle_refuses_changes_to_existing_content() {
        let mut changed =
            merge(&existing(), &json!({"document":{},"items":[{"key":"new"}]})).unwrap();
        verify_add_only(&existing(), &changed).unwrap();
        changed["items"][0]["state"] = json!("planned");
        assert!(verify_add_only(&existing(), &changed).is_err());
        for field in ["source", "roles", "parties", "schema_version"] {
            let mut changed =
                merge(&existing(), &json!({"document":{},"items":[{"key":"new"}]})).unwrap();
            changed[field] = json!("different");
            assert!(verify_add_only(&existing(), &changed).is_err(), "{field}");
        }
        assert!(verify_add_only(&existing(), &existing()).is_err());
    }
}
