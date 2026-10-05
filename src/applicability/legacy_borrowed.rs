//! Non-authorizing legacy applicability facts from complete supplied borrowed originals.
//!
//! This preparation primitive performs no IO and creates no source, lease, lifecycle,
//! currentness or cohort proof. Embedded historical Mapping policy identities remain
//! private data; the maintained legacy validator does not open their hrefs. Callers
//! must retain the genuine originals and resolve the declared complete roster through
//! their own capture owner. This module cannot attest that correspondence or aliases.
//!
//! Every supplied original is precharged, including an unused supplied companion.
//! Logical derived payload, repeated canonical work, target facts, Cartesian pairs,
//! private report copies and the unfiltered control denominator are admitted before
//! their downstream growth. Parser/schema internals are bounded by original bytes
//! and existing native validation, not a total-heap or synchronous-preemption claim.
//! The review profile can conservatively refuse inputs accepted by the file command.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

/// Reuse the actual current helper admission vocabulary without creating authority.
pub(crate) use super::captured::admitted::ApplicabilityCharge;
use super::{MappingValidationState, manifest, model};
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::mapping::inventory::{self, CapturedInventoryError};
use crate::mapping::manifest::SubjectType;
use crate::workspace::preparation::{ProgressUpdate, Stage, WorkControl, WorkError};

/// Plain complete supplied original bytes and their relative-path label base.
///
/// These borrowed fields do not attest roles, path correspondence, physical aliases,
/// capture ownership, currentness or any native proof. The capture factory supplies
/// and retains the actual read set; this evaluator accepts data without issuing authority.
#[derive(Clone, Copy)]
pub(crate) struct LegacyApplicabilityInputs<'a> {
    /// Actual complete applicability manifest original.
    pub(crate) manifest_bytes: &'a [u8],
    /// Base label for declared framework paths; never used for IO by this primitive.
    pub(crate) manifest_dir: &'a Path,
    /// Actual framework artifact original selected by the caller's declared read set.
    pub(crate) framework_bytes: &'a [u8],
    /// Explicit Profile Catalog companion original, without inferred import resolution.
    pub(crate) resolved_catalog_bytes: Option<&'a [u8]>,
    /// Complete borrowed Mapping originals in manifest declaration order.
    pub(crate) mapping_bytes: &'a [&'a [u8]],
}

/// Complete private legacy facts; detached domain data carries no capture authority.
pub(crate) struct LegacyApplicabilityFacts {
    /// Maintained complete report, including private source tuples and all denominator counts.
    pub(crate) report: model::ApplicabilityReport,
    /// Maintained validated decisions and reviewer declarations, without IO fingerprints.
    pub(crate) manifest: manifest::ApplicabilityManifest,
}

/// Preserve native private errors, original admission errors and actual caller work stops.
#[derive(Debug)]
pub(crate) enum LegacyBorrowedError<E> {
    /// Full maintained native domain diagnostic; callers minimize it at their own boundary.
    Domain(ForgeError),
    /// The actual caller's admission failure is returned without conversion or replacement.
    Admission(E),
    /// The actual supplied work control failed or stopped; no substitute control is created.
    Work(WorkError),
    /// A checked/profile limit failed even if the caller accepted the Capacity notification.
    Capacity,
}

/// Discover only validated native declarations from one supplied original, without IO.
///
/// The capture reader uses this exact shared decode before opening declared routes.
/// The result is plain manifest data; it supplies no ownership, role, currentness or
/// cohort proof. Full preparation repeats the same native decode and complete checks.
///
/// # Errors
///
/// Preserve original admission/work failures and the actual post-parse control fence
/// before returning either native declaration data or a maintained domain diagnostic.
pub(crate) fn discover_manifest<E>(
    bytes: &[u8],
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<manifest::ApplicabilityManifest, LegacyBorrowedError<E>> {
    preparse(bytes, admit, control)?;
    let parsed_result = manifest::parse(bytes);
    step(admit, control)?;
    parsed_result.map_err(LegacyBorrowedError::Domain)
}

/// Prepare legacy facts with the same actual admission callback and caller control.
///
/// Mapping originals must appear in complete declared manifest order. The cardinality
/// check establishes only data completeness; paths, physical aliases, ownership and
/// same-owner union proofs remain the capture factory's responsibility. This function
/// deliberately uses legacy embedded policy evidence rather than the stronger current
/// native-source evaluator. The supplied original bytes remain borrowed throughout.
///
/// # Errors
///
/// Preserves maintained manifest, schema, inventory, decision and Mapping domain errors
/// within the profile, plus the first actual caller admission or work-control failure.
/// Additional conservative review-profile bounds are capacity refusals, not approval.
pub(crate) fn prepare<E>(
    inputs: LegacyApplicabilityInputs<'_>,
    filters: model::ReportFilters,
    control: &mut dyn WorkControl,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<LegacyApplicabilityFacts, LegacyBorrowedError<E>> {
    let LegacyApplicabilityInputs {
        manifest_bytes,
        manifest_dir,
        framework_bytes,
        resolved_catalog_bytes,
        mapping_bytes,
    } = inputs;
    step(admit, control)?;
    let filter_extent = filter_bytes(&filters, admit)?;
    work(0, mul(filter_extent, 32, admit)?, 5, admit)?;
    let filter_result = super::validate_filters(&filters);
    step(admit, control)?;
    filter_result.map_err(LegacyBorrowedError::Domain)?;
    if mapping_bytes.len() > 100 {
        return capacity(admit);
    }
    let parsed = discover_manifest(manifest_bytes, control, admit)?;
    if parsed.mapping_collections.len() != mapping_bytes.len() {
        return Err(LegacyBorrowedError::Domain(super::error(
            "borrowed Mapping originals do not cover the complete declared manifest roster",
        )));
    }
    preparse(framework_bytes, admit, control)?;
    if let Some(companion) = resolved_catalog_bytes {
        preparse(companion, admit, control)?;
    }
    let path_extent =
        add(manifest_dir.as_os_str().len(), parsed.framework.artifact.as_os_str().len(), admit)?;
    reserve(add(mul(path_extent, 2, admit)?, 512, admit)?, admit)?;
    if let Some(companion) = &parsed.framework.resolved_catalog {
        let extent = add(manifest_dir.as_os_str().len(), companion.as_os_str().len(), admit)?;
        reserve(add(mul(extent, 2, admit)?, 256, admit)?, admit)?;
    }
    let mut plan = Plan::default();
    let framework_result = inventory::load_captured_admitted(
        manifest_dir,
        "$.framework",
        &parsed.framework,
        framework_bytes,
        resolved_catalog_bytes,
        &mut |value| catalog(value, &mut plan, admit, control),
    );
    let framework = match framework_result {
        // An inventory callback already observed an actual caller failure. Preserve
        // that chronological first cause without issuing a later replacement probe.
        Err(CapturedInventoryError::Admission(error)) => return Err(error),
        Err(CapturedInventoryError::Domain(error)) => {
            step(admit, control)?;
            return Err(LegacyBorrowedError::Domain(super::relabel_mapping_error(error)));
        }
        Ok(resource) => {
            step(admit, control)?;
            resource
        }
    };
    validate_framework_decisions(&parsed, &framework, manifest_bytes.len(), admit, control)?;
    let mut mapping_facts = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut validation_state = MappingValidationState::default();
    reserve(mul(mapping_bytes.len(), 512, admit)?, admit)?;
    for (index, bytes) in mapping_bytes.iter().enumerate() {
        step(admit, control)?;
        preparse(bytes, admit, control)?;
        reserve(128, admit)?;
        let label = format!("$.mapping_collections[{index}]");
        let value_result = super::parse_mapping_value(&label, bytes);
        step(admit, control)?;
        let value = value_result.map_err(LegacyBorrowedError::Domain)?;
        let metrics = tree(&value, 0, admit, control)?;
        reserve(mul(metrics.owned, 5, admit)?, admit)?;
        work(0, mul(metrics.text, 32, admit)?, mul(metrics.nodes, 32, admit)?, admit)?;
        mapping(&value, &mut plan, admit, control)?;
        let mapping_result = super::record_mapping_value(
            &label,
            value,
            &sha256_hex(bytes),
            &framework,
            &mut mapping_facts,
            &mut validation_state,
        );
        step(admit, control)?;
        evidence.push(mapping_result.map_err(LegacyBorrowedError::Domain)?);
    }
    report(&parsed, &framework.inventory, &mapping_facts, &filters, admit, control)?;
    // This is the same complete model builder and reconciliation used by the file wrapper.
    let report_result = super::finish_report(
        &parsed,
        sha256_hex(manifest_bytes),
        framework.evidence,
        &framework.inventory,
        evidence,
        &mapping_facts,
        filters,
    );
    step(admit, control)?;
    let report = report_result.map_err(LegacyBorrowedError::Domain)?;
    Ok(LegacyApplicabilityFacts { report, manifest: parsed })
}

/// Preserve admitted group/decision validation and each native post-phase caller fence.
fn validate_framework_decisions<E>(
    parsed: &manifest::ApplicabilityManifest,
    framework: &inventory::LoadedResource,
    manifest_extent: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    let group_result = super::validate_applicability_group_ids(&framework.inventory);
    step(admit, control)?;
    group_result.map_err(LegacyBorrowedError::Domain)?;
    work(
        parsed.decisions.len(),
        mul(manifest_extent, 32, admit)?,
        mul(parsed.decisions.len(), 64, admit)?,
        admit,
    )?;
    let decision_result = super::validate_decision_references(parsed, framework);
    step(admit, control)?;
    decision_result.map_err(LegacyBorrowedError::Domain)
}

/// Fixed-size complete inventory and Mapping denominators, independent of report selection.
#[derive(Default)]
struct Plan {
    /// Complete eligible subjects in the actual effective Catalog.
    subjects: usize,
    /// Complete controls before filtering or report construction.
    controls: usize,
    /// Complete native maps across all supplied Mapping Collections.
    maps: usize,
    /// Complete Cartesian source-target pair denominator across native maps.
    pairs: usize,
}

/// Fixed-size borrowed-tree measurements; canonical bound includes every field.
#[derive(Default)]
struct Metrics {
    /// Complete node occurrences in this subtree.
    nodes: usize,
    /// Complete primitive and object-key UTF-8 bytes.
    text: usize,
    /// Conservative logical node headers plus primitive/key payload.
    owned: usize,
    /// Conservative canonical encoded bytes, including finite numeric formatting.
    canonical: usize,
    /// Complete recursive object-key sorting reference slots.
    sort_slots: usize,
}

/// Inspect a bounded temporary native tree before downstream typed forms grow.
fn tree<E>(
    value: &Value,
    depth: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<Metrics, LegacyBorrowedError<E>> {
    step(admit, control)?;
    if depth > 64 {
        return capacity(admit);
    }
    work(1, 0, 0, admit)?;
    let mut result = Metrics { nodes: 1, owned: 128, ..Metrics::default() };
    match value {
        Value::Null => result.canonical = 4,
        Value::Bool(_) => result.canonical = 5,
        // The maintained canonical writer formats finite i64/u64/f64. Its
        // fixed-point integral branch is below 1e21; a finite f64 decimal
        // expansion (including the smallest subnormal) needs fewer than 1024
        // bytes. This deliberately exceeds compact JSON's numeric width.
        Value::Number(_) => result.canonical = 1024,
        Value::String(text) => {
            work(0, text.len(), 0, admit)?;
            result.text = text.len();
            result.owned = add(result.owned, text.len(), admit)?;
            result.canonical = add(mul(text.len(), 6, admit)?, 2, admit)?;
        }
        Value::Array(values) => {
            result.canonical = add(2, values.len(), admit)?;
            for child in values {
                merge_metrics(&mut result, &tree(child, depth + 1, admit, control)?, admit)?;
            }
        }
        Value::Object(fields) => {
            result.canonical = add(2, mul(fields.len(), 2, admit)?, admit)?;
            result.sort_slots = fields.len();
            for (key, child) in fields {
                work(0, key.len(), 0, admit)?;
                result.text = add(result.text, key.len(), admit)?;
                result.owned = add(result.owned, key.len(), admit)?;
                result.canonical =
                    add(result.canonical, add(mul(key.len(), 6, admit)?, 2, admit)?, admit)?;
                merge_metrics(&mut result, &tree(child, depth + 1, admit, control)?, admit)?;
            }
        }
    }
    if result.nodes > 100_000 {
        return capacity(admit);
    }
    Ok(result)
}

/// Add complete child metrics with checked arithmetic, never a truncated prefix.
fn merge_metrics<E>(
    parent: &mut Metrics,
    child: &Metrics,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<(), LegacyBorrowedError<E>> {
    parent.nodes = add(parent.nodes, child.nodes, admit)?;
    parent.text = add(parent.text, child.text, admit)?;
    parent.owned = add(parent.owned, child.owned, admit)?;
    parent.canonical = add(parent.canonical, child.canonical, admit)?;
    parent.sort_slots = add(parent.sort_slots, child.sort_slots, admit)?;
    Ok(())
}

/// Admit one genuine effective Catalog expansion before a native inventory producer grows.
///
/// Intended for the maintained `load_captured_admitted` callback, after native schema
/// admission and before inventory construction. The result is admission-only plain
/// preparation; it creates no native, owner, currentness or cohort proof. The caller
/// supplies the same actual review callback/control across all old/new inventories.
/// Per-inventory profile limits apply here; complete union limits remain caller-owned.
///
/// # Errors
///
/// Returns the actual caller admission/work failure or a conservative capacity refusal.
pub(crate) fn admit_legacy_inventory<E>(
    value: &Value,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    catalog(value, &mut Plan::default(), admit, control)
}

/// Charge complete actual effective Catalog inventory expansions before loading.
fn catalog<E>(
    value: &Value,
    plan: &mut Plan,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    step(admit, control)?;
    let root = value.get("catalog").ok_or_else(|| {
        LegacyBorrowedError::Domain(super::error("validated effective inventory is not a Catalog"))
    })?;
    let metrics = tree(value, 0, admit, control)?;
    work(0, mul(metrics.text, 32, admit)?, mul(metrics.nodes, 32, admit)?, admit)?;
    inventory_nodes(root, 0, (0, 0), plan, admit, control)
}

/// Charge actual copied ancestry and repeated canonical buffers by traversal kind.
fn inventory_nodes<E>(
    value: &Value,
    depth: usize,
    ancestry: (usize, usize),
    plan: &mut Plan,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    step(admit, control)?;
    if depth > inventory::MAX_INVENTORY_DEPTH {
        return capacity(admit);
    }
    let (ancestor_slots, ancestor_bytes) = ancestry;
    for key in ["groups", "controls", "parts"] {
        if let Some(rows) = value.get(key).and_then(Value::as_array) {
            for row in rows {
                step(admit, control)?;
                let mut next_slots = ancestor_slots;
                let mut next_bytes = ancestor_bytes;
                let id = row.get("id").and_then(Value::as_str).unwrap_or("");
                if key == "groups" {
                    let copied = add(ancestor_bytes, mul(ancestor_slots, 24, admit)?, admit)?;
                    reserve(
                        add(copied, add(mul(id.len(), 3, admit)?, 192, admit)?, admit)?,
                        admit,
                    )?;
                    if !id.trim().is_empty() {
                        next_slots = add(next_slots, 1, admit)?;
                        next_bytes = add(next_bytes, id.len(), admit)?;
                    }
                } else if key == "controls"
                    || row.get("name").and_then(Value::as_str) == Some("statement")
                {
                    if key == "controls" {
                        plan.controls = add(plan.controls, 1, admit)?;
                        if plan.controls > 10_000 {
                            return capacity(admit);
                        }
                    }
                    plan.subjects = add(plan.subjects, 1, admit)?;
                    if plan.subjects > 100_000 {
                        return capacity(admit);
                    }
                    let subtree = tree(row, 0, admit, control)?;
                    reserve(
                        add(subtree.canonical, mul(subtree.sort_slots, 8, admit)?, admit)?,
                        admit,
                    )?;
                    // IDs, subjects, excerpts, control groups and full native snapshot.
                    // Six ID copies include conservative room; excerpts <=160 Unicode scalars.
                    reserve(add(mul(id.len(), 6, admit)?, 896, admit)?, admit)?;
                    work(
                        0,
                        add(
                            mul(subtree.canonical, 2, admit)?,
                            mul(subtree.text, 32, admit)?,
                            admit,
                        )?,
                        mul(subtree.nodes, 32, admit)?,
                        admit,
                    )?;
                    if key == "controls" {
                        reserve(
                            add(ancestor_bytes, mul(ancestor_slots, 24, admit)?, admit)?,
                            admit,
                        )?;
                    }
                } else if let Some(name) = row.get("name").and_then(Value::as_str) {
                    reserve(add(add(id.len(), name.len(), admit)?, 128, admit)?, admit)?;
                }
                inventory_nodes(row, depth + 1, (next_slots, next_bytes), plan, admit, control)?;
            }
        }
    }
    Ok(())
}

/// Admit complete temporary native decoded forms before calling the maintained parser.
///
/// The lexical scan bounds slot/string payload on valid JSON without admitting syntax.
/// Legacy JSON/parser diagnostics remain those of the maintained downstream parser.
/// Four conservative forms cover manifest/native Value and typed/header logical copies;
/// parser/schema internal allocator behavior is explicitly outside this logical ledger.
fn preparse<E>(
    bytes: &[u8],
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    step(admit, control)?;
    if bytes.len() > 10 * 1024 * 1024 {
        return capacity(admit);
    }
    work(0, mul(bytes.len(), 10, admit)?, 0, admit)?;
    let slots = preparse_slots(bytes, admit, control)?;
    let decoded = add(add(mul(slots, 128, admit)?, bytes.len(), admit)?, 512, admit)?;
    reserve(mul(decoded, 4, admit)?, admit)
}

/// Bound temporary decoded node/payload storage without retaining a decoded tree.
///
/// On valid JSON every Value begins with a container, a quoted string or one
/// primitive token. Object keys are additionally counted as string slots. Raw
/// bytes bound all decoded key/string payload. This scan does not admit syntax,
/// duplicates or authority; the maintained legacy parser still decides those.
fn preparse_slots<E>(
    bytes: &[u8],
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<usize, LegacyBorrowedError<E>> {
    let mut slots = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let mut primitive = false;
    for chunk in bytes.chunks(32 * 1024) {
        step(admit, control)?;
        for &byte in chunk {
            if quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
                continue;
            }
            match byte {
                b'"' => {
                    slots = add(slots, 1, admit)?;
                    quoted = true;
                    primitive = false;
                }
                b'{' | b'[' => {
                    slots = add(slots, 1, admit)?;
                    primitive = false;
                }
                b'}' | b']' | b',' | b':' | b' ' | b'\r' | b'\n' | b'\t' => primitive = false,
                _ if !primitive => {
                    slots = add(slots, 1, admit)?;
                    primitive = true;
                }
                _ => {}
            }
        }
    }
    work(slots, 0, 0, admit)?;
    Ok(slots)
}

/// Send complete repeated work to the caller's single monotonic ledger.
fn work<E>(
    visits: usize,
    byte_work: usize,
    matching_steps: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<(), LegacyBorrowedError<E>> {
    admit(ApplicabilityCharge::Work { visits, byte_work, matching_steps })
        .map_err(LegacyBorrowedError::Admission)
}

/// Reserve logical payload before a downstream typed form/registry is installed.
fn reserve<E>(
    logical_bytes: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<(), LegacyBorrowedError<E>> {
    admit(ApplicabilityCharge::Reserve { logical_bytes }).map_err(LegacyBorrowedError::Admission)
}

/// Preserve caller admission first, then observe the actual immutable work control.
fn step<E>(
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    admit(ApplicabilityCharge::Checkpoint).map_err(LegacyBorrowedError::Admission)?;
    control
        .checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
        .map_err(LegacyBorrowedError::Work)
}

/// Ask the actual caller to latch Capacity; an accepting callback still cannot continue.
fn capacity<T, E>(
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<T, LegacyBorrowedError<E>> {
    match admit(ApplicabilityCharge::Capacity) {
        Err(error) => Err(LegacyBorrowedError::Admission(error)),
        Ok(()) => Err(LegacyBorrowedError::Capacity),
    }
}

/// Add bounded denominators without overflow or silent saturating admission.
fn add<E>(
    left: usize,
    right: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<usize, LegacyBorrowedError<E>> {
    left.checked_add(right).map_or_else(|| capacity(admit), Ok)
}

/// Multiply complete occurrence counts without an overflow/prefix exception.
fn mul<E>(
    left: usize,
    right: usize,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<usize, LegacyBorrowedError<E>> {
    left.checked_mul(right).map_or_else(|| capacity(admit), Ok)
}

/// Admit every native map and complete Cartesian registry before native Mapping growth.
///
/// Optional shape probes do not replace native schema errors. An invalid shape continues
/// to the maintained validator, provided no actual profile/caller stop has occurred.
fn mapping<E>(
    value: &Value,
    plan: &mut Plan,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    let Some(groups) = value
        .get("mapping-collection")
        .and_then(|root| root.get("mappings"))
        .and_then(Value::as_array)
    else {
        return Ok(());
    };
    for group in groups {
        step(admit, control)?;
        let source = group.get("source-resource");
        let href =
            source.and_then(|source| source.get("href")).and_then(Value::as_str).unwrap_or("");
        let Some(maps) = group.get("maps").and_then(Value::as_array) else {
            continue;
        };
        for edge in maps {
            step(admit, control)?;
            plan.maps = add(plan.maps, 1, admit)?;
            if plan.maps > 10_000 {
                return capacity(admit);
            }
            let sources = edge.get("sources").and_then(Value::as_array);
            let targets = edge.get("targets").and_then(Value::as_array);
            let (Some(sources), Some(targets)) = (sources, targets) else {
                continue;
            };
            let pairs = mul(sources.len(), targets.len(), admit)?;
            plan.pairs = add(plan.pairs, pairs, admit)?;
            if plan.pairs > 100_000 {
                return capacity(admit);
            }
            reserve(mul(pairs, 256, admit)?, admit)?;
            reserve(mul(add(sources.len(), targets.len(), admit)?, 256, admit)?, admit)?;
            let mut source_bytes = 0;
            for item in sources {
                let id = item.get("id-ref").and_then(Value::as_str).unwrap_or("");
                source_bytes = add(source_bytes, id.len(), admit)?;
                reserve(add(mul(id.len(), 4, admit)?, 256, admit)?, admit)?;
            }
            let mut target_bytes = 0;
            for item in targets {
                let id = item.get("id-ref").and_then(Value::as_str).unwrap_or("");
                target_bytes = add(target_bytes, id.len(), admit)?;
                reserve(add(mul(id.len(), 3, admit)?, 256, admit)?, admit)?;
            }
            let repeated_ids = add(
                mul(source_bytes, targets.len(), admit)?,
                mul(target_bytes, sources.len(), admit)?,
                admit,
            )?;
            let registry_text = add(repeated_ids, mul(pairs, 64, admit)?, admit)?;
            reserve(registry_text, admit)?;
            work(0, mul(registry_text, 32, admit)?, mul(pairs, 32, admit)?, admit)?;
            // The maintained closure records once per Control target, never once per pair.
            // Reserve every target occurrence, including duplicates later rejected natively.
            reserve(mul(targets.len(), add(href.len(), 192, admit)?, admit)?, admit)?;
            work(0, mul(href.len(), targets.len(), admit)?, targets.len(), admit)?;
        }
    }
    Ok(())
}

/// Admit complete private report fields and denominator controls before the shared builder.
fn report<E>(
    manifest: &manifest::ApplicabilityManifest,
    inventory: &inventory::Inventory,
    mapping_facts: &BTreeMap<String, model::ControlMappingFacts>,
    filters: &model::ReportFilters,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
    control: &mut dyn WorkControl,
) -> Result<(), LegacyBorrowedError<E>> {
    step(admit, control)?;
    reserve(mul(manifest.decisions.len(), 128, admit)?, admit)?;
    for decision in &manifest.decisions {
        step(admit, control)?;
        let mut payload = decision.control_id.len();
        for field in [
            decision.reviewer_key.as_deref(),
            decision.reviewed_at.as_deref(),
            decision.rationale.as_deref(),
            decision.revisit_date.as_deref(),
            decision.note.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            payload = add(payload, field.len(), admit)?;
        }
        reserve(add(mul(payload, 3, admit)?, 512, admit)?, admit)?;
        work(1, mul(payload, 32, admit)?, 32, admit)?;
    }
    for reviewer in &manifest.reviewers {
        step(admit, control)?;
        let payload = add(reviewer.key.len(), reviewer.name.len(), admit)?;
        reserve(add(payload, 128, admit)?, admit)?;
        work(1, payload, 0, admit)?;
    }
    let filter_extent = filter_bytes(filters, admit)?;
    let mut controls = 0;
    for id in inventory.ids_of_type_refs(SubjectType::Control) {
        step(admit, control)?;
        controls = add(controls, 1, admit)?;
        if controls > 10_000 {
            return capacity(admit);
        }
        // BTree IDs, visible-ID registry, full ControlResult and optional queue clones.
        reserve(add(mul(id.len(), 4, admit)?, 1024, admit)?, admit)?;
        // Actual inventory/facts lookup and downstream ID registry comparisons.
        work(1, mul(id.len(), 128, admit)?, 128, admit)?;
        let groups = inventory.groups_for_control(id);
        let mut group_bytes = 0;
        for group in groups {
            group_bytes = add(group_bytes, group.len(), admit)?;
        }
        reserve(add(group_bytes, mul(groups.len(), 24, admit)?, admit)?, admit)?;
        let mut policy_bytes = 0;
        let mut policy_slots = 0;
        if let Some(facts) = mapping_facts.get(id) {
            policy_slots = facts.policy_sources.len();
            for href in &facts.policy_sources {
                policy_bytes = add(policy_bytes, href.len(), admit)?;
            }
        }
        reserve(add(mul(policy_bytes, 3, admit)?, mul(policy_slots, 128, admit)?, admit)?, admit)?;
        let comparisons = add(add(id.len(), group_bytes, admit)?, policy_bytes, admit)?;
        work(
            0,
            mul(add(comparisons, filter_extent, admit)?, 32, admit)?,
            add(64, add(groups.len(), policy_slots, admit)?, admit)?,
            admit,
        )?;
    }
    Ok(())
}

/// Measure all supplied filter string bytes without allocating or applying a selection.
fn filter_bytes<E>(
    filters: &model::ReportFilters,
    admit: &mut impl FnMut(ApplicabilityCharge) -> Result<(), E>,
) -> Result<usize, LegacyBorrowedError<E>> {
    let mut extent = 0;
    for text in [
        filters.group.as_deref(),
        filters.control_prefix.as_deref(),
        filters.reviewer.as_deref(),
        filters.policy_source.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        extent = add(extent, text.len(), admit)?;
    }
    Ok(extent)
}
