//! Complete native-issued /2 requirement/search tuples over the genuine held owner.
//!
//! The private native ID/control-ID domain is <=4096 UTF-8 bytes with original spelling;
//! /1's public safe-token128 rule is unchanged. Native-valid wider IDs are never omitted
//! to fit public query DTOs. The whole universe matches intent or the whole gate refuses.

use serde_json::Value;

use super::capture_v2::{Admission, AdmittedValue};
use super::declarations_v2::{DeclarationAdmission, SearchSourceTuple, SourceSpan};
use super::native_sources_v2::NativeSourcesV2;
use super::native_work_v2::{self as work};
use super::queries::{QueryError, QueryResult};
use super::requirement_walk::{self as walk, native_string, unique_trace};
use super::{Role, policy_failure};
use crate::mapping::manifest::SubjectType;
use crate::workspace::preparation::{Stage, WorkControl, WorkError, WorkResult};

/// Exact closed native input ID width; no public DTO clipping is permitted.
const NATIVE_ID_BYTES: usize = 4096;
/// Complete actual pointer ceiling, unchanged from the shared native traversal.
const POINTER_BYTES: usize = 4096;
/// Exact current source size domain, including complete UTF-8 and line-scan operands.
const SOURCE_BYTES: usize = 10 * 1024 * 1024;

/// Scalar whole-set retained payload descriptor before the first output row grows.
#[derive(Default)]
struct TupleBudget {
    /// Complete actual eligible requirement occurrences, including duplicates later refused.
    rows: usize,
    /// Complete actual copied tuple/private identity strings plus every bounded pointer.
    logical: usize,
    /// Whether every native/source/span predicate holds; this is private ordinary planning data.
    valid: bool,
}

/// Prepare the complete visible Catalog+Component universe before any intent/page/filter.
/// None is ordinary complete-family refusal; actual first owner stops remain Work errors.
pub(super) fn complete<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
) -> WorkResult<Option<AdmittedValue<Vec<SearchSourceTuple>>>> {
    let mut budget = TupleBudget {
        valid: true,
        logical: std::mem::size_of::<Vec<SearchSourceTuple>>(),
        ..TupleBudget::default()
    };
    walk_visible(sources, &mut |index, node, _pointer| {
        let Some(width) = row_width(sources, index, node)? else {
            budget.valid = false;
            return Ok(());
        };
        let mut admission = sources.admission();
        budget.rows = work::add(budget.rows, 1, &mut admission)?;
        budget.logical = work::add(budget.logical, width, &mut admission)?;
        Ok(())
    })?;
    let mut admission = sources.admission();
    let ordinary = if budget.valid { Ok(()) } else { Err(policy_failure()) };
    let valid = admission.phase(ordinary, Stage::PrepareDomain);
    if let Err(error) = valid {
        // Ordinary planning refusal is identified before retention; original sticky stop
        // remains distinguishable by the same final owner fence, never a supplied verdict.
        work::fence(&mut admission, Stage::PrepareDomain)?;
        match error {
            WorkError::Failed(_) => return Ok(None),
            error @ WorkError::Interrupted(_) => return Err(error),
        }
    }
    let retained = admission.retain(budget.logical, |admission| {
        let mut tuples = Vec::with_capacity(budget.rows);
        walk_visible(sources, &mut |index, node, pointer| {
            admission.charge(1)?;
            tuples.push(row(sources, index, node, pointer)?);
            Ok(())
        })?;
        let work = sorting_work(&tuples, admission)?;
        work::run(admission, work, Stage::PrepareDomain, |_| {
            tuples.sort_unstable_by(|left, right| {
                (&left.artifact_key, &left.native_id).cmp(&(&right.artifact_key, &right.native_id))
            });
            if tuples.windows(2).any(|pair| {
                pair[0].artifact_key == pair[1].artifact_key
                    && pair[0].native_id == pair[1].native_id
            }) {
                return Err(policy_failure());
            }
            Ok(())
        })?;
        compare_inventory(sources, &tuples)?;
        Ok(tuples)
    })?;
    Ok(Some(retained))
}

/// Walk the exact complete selected role universe with one bounded actual cursor scratch.
fn walk_visible<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    visit: &mut impl FnMut(usize, &Value, &str) -> WorkResult<()>,
) -> WorkResult<()> {
    let mut admission = sources.admission();
    let scratch = POINTER_BYTES + std::mem::size_of::<String>() + 64;
    admission
        .retain(scratch, |admission| {
            let mut path = String::with_capacity(POINTER_BYTES);
            for key in &sources.profile().visible_resource_keys {
                admission.charge(1)?;
                let index = sources.index(key)?.ok_or_else(policy_failure)?;
                let role = sources.policies()[index].role;
                if !matches!(role, Role::OscalCatalogArtifact | Role::OscalComponentArtifact) {
                    continue;
                }
                let approved = sources.approved(key)?.ok_or_else(policy_failure)?;
                let tree = approved.resource().native_value().ok_or_else(policy_failure)?;
                let mut walker = sources.admission();
                let mut charger = sources.admission();
                let mut charge = |amount| charger.charge(amount).map_err(QueryError::from);
                path.clear();
                let field = if role == Role::OscalCatalogArtifact {
                    "catalog"
                } else {
                    "component-definition"
                };
                domain(walk::append_located(&mut path, field, &mut charge))?;
                let root = tree.get(field).ok_or_else(policy_failure)?;
                let mut located = |node: &Value, pointer: &str, _: &mut dyn WorkControl| {
                    visit(index, node, pointer).map_err(QueryError::from)
                };
                let result = match role {
                    Role::OscalCatalogArtifact => walk::located_catalog_nodes(
                        root,
                        0,
                        1,
                        &mut path,
                        &mut located,
                        &mut charge,
                        &mut walker,
                    ),
                    Role::OscalComponentArtifact => walk::located_component_nodes(
                        root,
                        &mut path,
                        &mut located,
                        &mut charge,
                        &mut walker,
                    ),
                    _ => return Err(policy_failure()),
                };
                let result = domain(result);
                admission.phase(result, Stage::PrepareDomain)?;
            }
            Ok(())
        })
        .map(|_| ())
}

/// Borrow exact native IDs after genuine full native validation; never safe-token-filter them.
fn identifiers(node: &Value, role: Role) -> WorkResult<(&str, &str)> {
    let id = domain(native_string(
        node,
        if role == Role::OscalCatalogArtifact { "id" } else { "uuid" },
    ))?;
    let control = if role == Role::OscalCatalogArtifact {
        id
    } else {
        domain(native_string(node, "control-id"))?
    };
    if id.len() > NATIVE_ID_BYTES || control.len() > NATIVE_ID_BYTES {
        return Err(policy_failure());
    }
    Ok((id, control))
}

/// Complete actual trace metadata remains an ordinary scratch value under a retained ticket.
fn trace<C: WorkControl + ?Sized>(
    node: &Value,
    admission: &mut Admission<'_, C>,
) -> WorkResult<AdmittedValue<crate::trace::report::TraceMetadata>> {
    let measured = work::operand(node, admission)?;
    let strings = work::multiply(measured.strings, 3, admission)?;
    let logical = work::add(strings, 512, admission)?;
    let passes = work::multiply(measured.work, 4, admission)?;
    admission.retain(logical, |admission| {
        work::run(admission, passes, Stage::PrepareDomain, |_| {
            domain(unique_trace(node))?;
            crate::trace::extractor::extract_trace_metadata(node).ok_or_else(policy_failure)
        })
    })
}

/// Compute complete tuple copied-string/row width before the complete Vec is constructed.
fn row_width<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    index: usize,
    node: &Value,
) -> WorkResult<Option<usize>> {
    let mut admission = sources.admission();
    let result = (|| {
        admission.charge(1)?;
        let policy = &sources.policies()[index];
        let (id, control) = identifiers(node, policy.role)?;
        let trace = trace(node, &mut admission)?;
        let source = sources
            .declared_source(index, &trace.value().source_file)?
            .ok_or_else(policy_failure)?;
        let raw = source.bytes()?;
        let span = source_span(
            raw,
            trace.value().source_line.ok_or_else(policy_failure)?,
            &mut admission,
        )?;
        if span.0 >= span.1 {
            return Err(policy_failure());
        }
        let identity = sources
            .facts(index)
            .and_then(|facts| facts.identity.as_ref())
            .ok_or_else(policy_failure)?;
        let lifecycle = policy.lifecycle_key.as_deref().ok_or_else(policy_failure)?;
        let mut logical = std::mem::size_of::<SearchSourceTuple>() + POINTER_BYTES;
        for text in [
            policy.key.as_str(),
            identity.model.as_str(),
            identity.root_id.as_str(),
            identity.document_version.as_str(),
            identity.oscal_version.as_str(),
            policy.expected_sha256.as_str(),
            id,
            control,
            source.key(),
            source.raw_sha256()?,
            lifecycle,
        ] {
            logical = work::add(logical, text.len(), &mut admission)?;
        }
        Ok(logical)
    })();
    let result = admission.phase(result, Stage::PrepareDomain);
    // Preserve a real original owner stop before ordinary row invalidity can become None.
    work::fence(&mut admission, Stage::PrepareDomain)?;
    match result {
        Ok(width) => Ok(Some(width)),
        Err(WorkError::Failed(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Build exactly one actual privately admitted native/source tuple under whole-set retention.
fn row<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    index: usize,
    node: &Value,
    pointer: &str,
) -> WorkResult<SearchSourceTuple> {
    let mut admission = sources.admission();
    let policy = &sources.policies()[index];
    let (id, control) = identifiers(node, policy.role)?;
    let trace = trace(node, &mut admission)?;
    let source =
        sources.declared_source(index, &trace.value().source_file)?.ok_or_else(policy_failure)?;
    let (start, end) = source_span(
        source.bytes()?,
        trace.value().source_line.ok_or_else(policy_failure)?,
        &mut admission,
    )?;
    if start >= end {
        return Err(policy_failure());
    }
    let identity = sources
        .facts(index)
        .and_then(|facts| facts.identity.as_ref())
        .ok_or_else(policy_failure)?;
    let lifecycle = policy.lifecycle_key.as_ref().ok_or_else(policy_failure)?;
    let copy = row_width(sources, index, node)?.ok_or_else(policy_failure)?;
    work::run(&mut admission, 1 + work::byte_work(copy), Stage::PrepareDomain, |_| {
        Ok(SearchSourceTuple {
            artifact_key: policy.key.clone(),
            artifact_role: policy.role,
            artifact_identity: identity.clone(),
            artifact_raw_sha256: policy.expected_sha256.clone(),
            native_id: id.to_owned(),
            control_id: control.to_owned(),
            native_pointer: pointer.to_owned(),
            source_key: source.key().to_owned(),
            source_raw_sha256: source.raw_sha256()?.to_owned(),
            source_span: SourceSpan {
                start_byte: u64::try_from(start).map_err(|_| policy_failure())?,
                end_byte: u64::try_from(end).map_err(|_| policy_failure())?,
            },
            lifecycle_key: lifecycle.clone(),
        })
    })
}

/// Complete UTF-8 and source line operands are admitted before either pass; no normalized copy.
fn source_span<C: WorkControl + ?Sized>(
    raw: &[u8],
    line: usize,
    admission: &mut Admission<'_, C>,
) -> WorkResult<(usize, usize)> {
    let passes = work::multiply(2, work::byte_work(raw.len()), admission)?;
    let amount = work::add(1, passes, admission)?;
    work::run(admission, amount, Stage::PrepareDomain, |_| {
        if raw.len() > SOURCE_BYTES {
            return Err(policy_failure());
        }
        let text = std::str::from_utf8(raw).map_err(|_| policy_failure())?;
        walk::line_span(text, line).ok_or_else(policy_failure)
    })
}

/// Preserve the full Control inventory denominator, admitting all actual comparison strings.
fn compare_inventory<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    tuples: &[SearchSourceTuple],
) -> WorkResult<()> {
    let mut admission = sources.admission();
    let result = (|| {
        for key in &sources.profile().visible_resource_keys {
            let index = sources.index(key)?.ok_or_else(policy_failure)?;
            if sources.policies()[index].role != Role::OscalCatalogArtifact {
                continue;
            }
            let loaded = sources.inventory(index).ok_or_else(policy_failure)?;
            let mut width = 0;
            for tuple in tuples {
                admission.charge(1)?;
                let keys = work::add(key.len(), tuple.artifact_key.len(), &mut admission)?;
                let operands = work::add(keys, tuple.native_id.len(), &mut admission)?;
                width = work::add(width, operands, &mut admission)?;
            }
            for id in loaded.inventory.ids_of_type_refs(SubjectType::Control) {
                admission.charge(1)?;
                width = work::add(width, id.len(), &mut admission)?;
            }
            let rows = work::add(
                tuples.len(),
                loaded.inventory.count(SubjectType::Control),
                &mut admission,
            )?;
            let amount = work::add(rows, work::byte_work(width), &mut admission)?;
            work::run(&mut admission, amount, Stage::PrepareDomain, |_| {
                let controls = loaded.inventory.ids_of_type_refs(SubjectType::Control);
                let rows = tuples
                    .iter()
                    .filter(|tuple| tuple.artifact_key == *key)
                    .map(|tuple| tuple.native_id.as_str());
                if controls.ne(rows) {
                    return Err(policy_failure());
                }
                Ok(())
            })?;
        }
        Ok(())
    })();
    admission.phase(result, Stage::PrepareDomain)
}

/// Compare the entire authored intent roster bijectively, allowing its original ordering.
/// Native rows are canonical key/native-ID sorted; no authored row is clipped or selected.
pub(super) fn matches_intent<C: WorkControl + ?Sized>(
    sources: &NativeSourcesV2<'_, C>,
    tuples: &[SearchSourceTuple],
) -> WorkResult<bool> {
    let mut admission = sources.admission();
    let Some(intent) = sources.build_intent() else {
        return Err(policy_failure());
    };
    let levels = usize::try_from(usize::BITS - tuples.len().max(1).leading_zeros())
        .map_err(|_| admission.capacity())?
        + 1;
    let probes = work::multiply(intent.search_sources.len(), levels, &mut admission)?;
    let mut widths = 0;
    for tuple in intent.search_sources.iter().chain(tuples) {
        admission.charge(1)?;
        let width = tuple_width(tuple, &mut admission)?;
        widths = work::add(widths, width, &mut admission)?;
    }
    let repetitions = work::add(probes, intent.search_sources.len(), &mut admission)?;
    let repeated = work::multiply(work::byte_work(widths), repetitions, &mut admission)?;
    let amount = work::add(probes, repeated, &mut admission)?;
    work::run(&mut admission, amount, Stage::PrepareDomain, |_| {
        Ok(intent.search_sources.len() == tuples.len()
            && intent.search_sources.iter().all(|expected| {
                tuples
                    .binary_search_by(|actual| {
                        (&actual.artifact_key, &actual.native_id)
                            .cmp(&(&expected.artifact_key, &expected.native_id))
                    })
                    .is_ok_and(|index| tuples[index] == *expected)
            }))
    })
}

/// Complete actual tuple equality string operands, including original spelling/private versions.
fn tuple_width<C: WorkControl + ?Sized>(
    tuple: &SearchSourceTuple,
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let mut result = 0;
    for text in [
        &tuple.artifact_key,
        &tuple.artifact_identity.model,
        &tuple.artifact_identity.root_id,
        &tuple.artifact_identity.document_version,
        &tuple.artifact_identity.oscal_version,
        &tuple.artifact_raw_sha256,
        &tuple.native_id,
        &tuple.control_id,
        &tuple.native_pointer,
        &tuple.source_key,
        &tuple.source_raw_sha256,
        &tuple.lifecycle_key,
    ] {
        result = work::add(result, text.len(), admission)?;
    }
    Ok(result)
}

/// Admit every sort/duplicate comparison and complete key/ID string operand before sorting.
fn sorting_work<C: WorkControl + ?Sized>(
    tuples: &[SearchSourceTuple],
    admission: &mut Admission<'_, C>,
) -> WorkResult<usize> {
    let levels = usize::try_from(usize::BITS - tuples.len().max(1).leading_zeros())
        .map_err(|_| admission.capacity())?;
    let comparisons = work::multiply(tuples.len(), levels + 2, admission)?;
    let mut widths = 0;
    for tuple in tuples {
        admission.charge(1)?;
        let width = work::add(tuple.artifact_key.len(), tuple.native_id.len(), admission)?;
        widths = work::add(widths, width, admission)?;
    }
    let repeated = work::multiply(work::byte_work(widths), comparisons, admission)?;
    work::add(comparisons, repeated, admission)
}

/// Ordinary query-domain refusal is redacted; actual same-owner typed Work is preserved.
fn domain<T>(result: QueryResult<T>) -> WorkResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(QueryError::Work(error)) => Err(error),
        Err(QueryError::Unavailable(_)) => Err(policy_failure()),
    }
}
