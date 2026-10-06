//! Genuine complete Catalog inventory admission from the same held /2 originals.
//!
//! The maintained pure captured loader computes canonical Control/Statement fingerprints,
//! ancestry, excerpts and ambiguity checks. This caller supplies complete pre-growth
//! operand/retained-data admission; an inventory is ordinary data, never native approval.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::capture_v2::{Admission, AdmittedValue, HeldInputsV2, Observation};
use super::declarations_v2::{DeclarationAdmission, ProjectDeclarationV2};
use super::native_work_v2::{self as work, MemberFacts};
use super::{Role, policy_failure};
use crate::mapping::inventory::{self, CapturedInventoryError, LoadedResource};
use crate::mapping::manifest::{ResourceManifest, ResourceType};
use crate::workspace::preparation::{Stage, WorkControl, WorkResult};

/// Source-backed complete owned inventory/canonical scratch upper envelope.
#[derive(Default)]
struct Budget {
    /// Complete actual row/canonical/ancestry traversal descriptors before construction.
    work: usize,
    /// Every actual retained registry/private string occurrence, including duplicate attempts.
    logical: usize,
    /// Maximum full canonical subject plus sorting scratch, reserved through native return.
    canonical: usize,
}

/// Prepare complete actual Catalog inventories, retaining hidden/invalid observations as None.
/// Profile effective inventory is not inferred; its explicit domain companion remains OPEN.
pub(super) fn prepare<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    project: &ProjectDeclarationV2,
    members: &[Option<AdmittedValue<MemberFacts>>],
) -> WorkResult<AdmittedValue<Vec<Option<AdmittedValue<LoadedResource>>>>> {
    let mut admission = held.admission();
    let entry = std::mem::size_of::<Option<AdmittedValue<LoadedResource>>>();
    let slots = work::multiply(project.resources.len(), entry, &mut admission)?;
    let slots = work::add(
        slots,
        std::mem::size_of::<Vec<Option<AdmittedValue<LoadedResource>>>>(),
        &mut admission,
    )?;
    admission.retain(slots, |admission| {
        let mut result = Vec::with_capacity(project.resources.len());
        for (index, policy) in project.resources.iter().enumerate() {
            admission.charge(1)?;
            let facts = members.get(index).and_then(Option::as_ref).map(AdmittedValue::value);
            let mut width = policy.expected_sha256.len();
            for identity in facts
                .and_then(|facts| facts.identity.as_ref())
                .into_iter()
                .chain(policy.native_identity.as_ref())
            {
                for text in [
                    &identity.model,
                    &identity.root_id,
                    &identity.document_version,
                    &identity.oscal_version,
                ] {
                    width = work::add(width, text.len(), admission)?;
                }
            }
            admission.charge(1 + work::byte_work(width))?;
            let loaded = match facts {
                Some(facts)
                    if policy.role == Role::OscalCatalogArtifact
                        && facts.identity.as_ref() == policy.native_identity.as_ref() =>
                {
                    match (
                        facts.value.as_ref(),
                        held.original(&policy.key)?.ok_or_else(policy_failure)?,
                    ) {
                        (Some(value), Observation::Present { bytes, raw_sha256 })
                            if raw_sha256 == policy.expected_sha256 =>
                        {
                            Some(prepare_catalog(held, policy, value, bytes, admission)?)
                        }
                        _ => None,
                    }
                }
                _ => None,
            };
            result.push(loaded);
        }
        Ok(result)
    })
}

/// Use the genuine shared loader after reserving its full source-backed owned payload.
fn prepare_catalog<C: WorkControl + ?Sized>(
    held: &HeldInputsV2<'_, C>,
    policy: &super::ResourcePolicy,
    value: &Value,
    raw: &[u8],
    admission: &mut Admission<'_, C>,
) -> WorkResult<AdmittedValue<LoadedResource>> {
    work::admit_native_schema(Role::OscalCatalogArtifact, admission)?;
    let measured = budget(value, admission)?;
    let key_bytes = work::multiply(policy.key.len(), 4, admission)?;
    let logical = work::add(
        work::add(measured.logical, measured.canonical, admission)?,
        work::add(key_bytes, 4096, admission)?,
        admission,
    )?;
    let (tree_upper, _) = work::raw_bound(raw, 10 * 1024 * 1024, admission)?;
    let outer = work::add(
        measured.work,
        work::add(tree_upper, work::byte_work(raw.len()), admission)?,
        admission,
    )?;
    let mut callback_admission = held.admission();
    admission.retain(logical, |admission| {
        let resource = ResourceManifest {
            resource_type: ResourceType::Catalog,
            artifact: PathBuf::from(&policy.key),
            href: policy.key.clone(),
            resolved_catalog: None,
            resolved_catalog_attestation: None,
            expected_sha256: Some(policy.expected_sha256.clone()),
            expected_resolved_catalog_sha256: None,
            inventory: None,
        };
        work::run(admission, outer, Stage::SnapshotMapping, |_| {
            let result = inventory::load_captured_admitted(
                Path::new(""),
                "captured MCP Catalog",
                &resource,
                raw,
                None,
                &mut |actual| {
                    let complete = budget(actual, &mut callback_admission)?;
                    work::run(
                        &mut callback_admission,
                        complete.work,
                        Stage::SnapshotMapping,
                        |_| Ok(()),
                    )
                },
            );
            match result {
                Ok(loaded) => Ok(loaded),
                Err(CapturedInventoryError::Admission(error)) => Err(error),
                Err(CapturedInventoryError::Domain(_)) => Err(policy_failure()),
            }
        })
    })
}

/// Measure all complete native construction/string multiplicities before inventory growth.
fn budget<C: WorkControl + ?Sized>(
    value: &Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<Budget> {
    let complete = work::operand(value, admission)?;
    let Some(root) = value.get("catalog") else {
        return Err(policy_failure());
    };
    let mut result = Budget {
        work: work::multiply(complete.work, 2, admission)?,
        logical: work::multiply(complete.strings, 2, admission)?,
        canonical: 0,
    };
    catalog(root, 0, 0, &mut result, admission)?;
    Ok(result)
}

/// Inspect every Catalog group/control occurrence with complete repeated ancestor payload.
fn catalog<C: WorkControl + ?Sized>(
    value: &Value,
    depth: usize,
    ancestors: usize,
    result: &mut Budget,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    admission.charge(1)?;
    if depth > inventory::MAX_INVENTORY_DEPTH {
        return Err(policy_failure());
    }
    for group in array(value, "groups")? {
        admission.charge(1)?;
        let id = group.get("id").and_then(Value::as_str).filter(|id| !id.trim().is_empty());
        let width = id.map_or(0, str::len);
        // parent_groups.to_vec plus the actual group_path row/vector and two ID registries.
        let parent = work::add(ancestors, work::add(width, 640, admission)?, admission)?;
        result.logical = work::add(
            result.logical,
            work::add(parent, work::multiply(width, 2, admission)?, admission)?,
            admission,
        )?;
        result.work = work::add(result.work, 1 + work::byte_work(parent), admission)?;
        let string_row = work::add(width, std::mem::size_of::<String>(), admission)?;
        let inherited =
            if id.is_some() { work::add(ancestors, string_row, admission)? } else { ancestors };
        catalog(group, depth + 1, inherited, result, admission)?;
    }
    for control in array(value, "controls")? {
        subject(control, result, admission)?;
        let id = control.get("id").and_then(Value::as_str).ok_or_else(policy_failure)?;
        let copied = work::add(ancestors, work::add(id.len(), 128, admission)?, admission)?;
        result.logical = work::add(result.logical, copied, admission)?;
        result.work = work::add(result.work, 1 + work::byte_work(copied), admission)?;
        parts(control, depth + 1, result, admission)?;
        controls(control, depth + 1, ancestors, result, admission)?;
    }
    Ok(())
}

/// Nested native controls retain the exact same group ancestry, not an invented control path.
fn controls<C: WorkControl + ?Sized>(
    value: &Value,
    depth: usize,
    ancestors: usize,
    result: &mut Budget,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    admission.charge(1)?;
    if depth > inventory::MAX_INVENTORY_DEPTH {
        return Err(policy_failure());
    }
    for control in array(value, "controls")? {
        subject(control, result, admission)?;
        let id = control.get("id").and_then(Value::as_str).ok_or_else(policy_failure)?;
        let copied = work::add(ancestors, work::add(id.len(), 128, admission)?, admission)?;
        result.logical = work::add(result.logical, copied, admission)?;
        result.work = work::add(result.work, 1 + work::byte_work(copied), admission)?;
        parts(control, depth + 1, result, admission)?;
        controls(control, depth + 1, ancestors, result, admission)?;
    }
    Ok(())
}

/// Preserve every statement/ineligible part row and complete recursively nested part occurrence.
fn parts<C: WorkControl + ?Sized>(
    value: &Value,
    depth: usize,
    result: &mut Budget,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    admission.charge(1)?;
    if depth > inventory::MAX_INVENTORY_DEPTH {
        return Err(policy_failure());
    }
    for part in array(value, "parts")? {
        admission.charge(1)?;
        if part.get("name").and_then(Value::as_str) == Some("statement") {
            subject(part, result, admission)?;
        } else if let (Some(id), Some(name)) =
            (part.get("id").and_then(Value::as_str), part.get("name").and_then(Value::as_str))
        {
            let width = work::add(id.len(), name.len(), admission)?;
            result.logical =
                work::add(result.logical, work::add(width, 256, admission)?, admission)?;
            result.work = work::add(result.work, 1 + work::byte_work(width), admission)?;
        }
        parts(part, depth + 1, result, admission)?;
    }
    Ok(())
}

/// Include each complete repeated canonical subtree, hash, ID registry and 160-char excerpt.
fn subject<C: WorkControl + ?Sized>(
    value: &Value,
    result: &mut Budget,
    admission: &mut Admission<'_, C>,
) -> WorkResult<()> {
    let complete = work::operand(value, admission)?;
    let id = value.get("id").and_then(Value::as_str).ok_or_else(policy_failure)?;
    let ids = work::multiply(id.len(), 4, admission)?;
    let retained = work::add(ids, 1280, admission)?;
    result.logical = work::add(result.logical, retained, admission)?;
    // Every complete key/value can serialize to <=6*UTF8 bytes plus <=64 bytes per node.
    // Sorting vectors borrow actual keys and need no new private key string copies.
    let nodes = work::multiply(complete.nodes, 64, admission)?;
    let strings = work::multiply(complete.strings, 6, admission)?;
    let sorting = work::multiply(complete.fields, std::mem::size_of::<&str>(), admission)?;
    let canonical = work::add(work::add(nodes, strings, admission)?, sorting, admission)?;
    result.canonical = result.canonical.max(canonical);
    let repeated = work::multiply(complete.work, 2, admission)?;
    result.work = work::add(
        result.work,
        work::add(repeated, 6 + work::byte_work(retained) + work::byte_work(canonical), admission)?,
        admission,
    )?;
    Ok(())
}

/// Optional native arrays preserve wrong-type refusal rather than silently skipping input.
fn array<'a>(value: &'a Value, key: &str) -> WorkResult<&'a [Value]> {
    match value.get(key) {
        None => Ok(&[]),
        Some(Value::Array(values)) => Ok(values),
        _ => Err(policy_failure()),
    }
}
