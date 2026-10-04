//! Recorded applicability and mapping queries over actual approved captured closure.
//!
//! The maintained captured-domain helper contributes
//! current domain equality and native relationships, never approval authority.
//! Every selected domain original and every native dependency obtains its own
//! genuine recorded-current disclosure proof. Hrefs never authorize a read.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::Value;

use super::super::disclosure::{CapturedQueryScope, CapturedResource};
use super::super::{Role, resolve_record_path};
use super::requirements::{self, RequirementSet};
use super::{
    Citation, Page, Paged, QueryError, QueryResult, Reason, TraceRow, checkpoint, citation,
    page_result, page_start, retain, safe_token,
};
use crate::applicability::captured::{
    self, MappingInput, NativeInput, PreparedApplicability, ResolvedInput,
};
use crate::applicability::manifest::DecisionState;
use crate::applicability::model::{ClassificationCounts, GapClassification};
use crate::mapping::manifest::{ResourceType, SubjectType};
use crate::workspace::preparation::{WorkControl, WorkError, WorkResult};

/// Complete native domain ceiling; no filtered prefix admits an oversized report.
const MAX_CONTROLS: usize = 10_000;
/// Exact maintained mapping-collection denominator for one domain closure.
const MAX_MAPPINGS: usize = 100;

/// One actual current classification with independently approved source citations.
#[derive(Serialize)]
pub(super) struct ApplicabilityRow {
    /// Exact eligible framework control ID.
    control_id: String,
    /// Actual native recorded enum, or null for a genuinely omitted decision.
    decision_state: Option<DecisionState>,
    /// Fixed actual explicit-record or implicit-under-review qualification.
    decision_source: &'static str,
    /// Maintained classification, without copied inference rules.
    classification: GapClassification,
    /// Actual independently approved Catalog/source relation.
    citations: Vec<Citation>,
}

/// Minimized native gap classification, not a compliance finding.
#[derive(Serialize)]
struct GapRow {
    /// Exact eligible framework control ID.
    control_id: String,
    /// Actual maintained native classification.
    classification: GapClassification,
    /// Actual same-capture source/native citation.
    citations: Vec<Citation>,
}

/// Complete native totals retained separately from any page or subject selection.
#[derive(Serialize)]
pub(super) struct GapData {
    /// Exact full captured scope and canonical query generation.
    generation: String,
    /// Complete current maintained classification counts.
    summary: ClassificationCounts,
    /// Entire eligible matching denominator, before paging.
    matched: usize,
    /// Exact retained row count, at most fifty.
    emitted: usize,
    /// Same-generation next ordinal or explicit null.
    next_cursor: Option<String>,
    /// Individually admitted complete current rows.
    rows: Vec<GapRow>,
}

/// Future actual F20 companion projection, unavailable without its own real seam.
#[derive(Serialize)]
pub(super) struct EvidenceMetadata {
    /// Actual relationship identifier, never synthesized from a native href.
    relationship_id: String,
    /// Closed actual implementation/evidence kind.
    kind: &'static str,
    /// Actual current/changed/missing-local or uri-unverified qualification.
    freshness: &'static str,
    /// Actual captured provenance without evidence bytes or private filenames.
    citations: Vec<Citation>,
}

/// Private full domain proof plus exact resolved control-citation locator.
struct DomainPlan<'a> {
    /// Root's maintained semantics over genuine captured originals only.
    prepared: PreparedApplicability<'a>,
    /// Actual approved Catalog key, including an explicit resolved Profile binding.
    catalog_key: &'a str,
    /// Exact declared, independently approved native Mapping keys.
    mapping_keys: BTreeSet<&'a str>,
    /// Explicit framework Profile binding; no hash-only resolved discovery.
    profile_catalog: Option<(&'a str, &'a str)>,
}

/// Borrowed native values in canonical complete control order.
type ControlSet<'a> = BTreeMap<&'a str, (Option<DecisionState>, GapClassification)>;

/// Prepare a complete current domain, then project only the requested full page.
pub(super) fn applicability(
    scope: &CapturedQueryScope,
    generation: String,
    key: &str,
    subject: Option<&str>,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<Paged<ApplicabilityRow>> {
    let plan = select_domain(scope, key, control)?;
    let requirements = requirements::collect(scope, Some(plan.catalog_key), control)?;
    let controls = control_set(scope, &plan, &requirements, control)?;
    let matched = controls.keys().filter(|id| subject.is_none_or(|wanted| **id == wanted)).count();
    let start = page_start(&generation, page, matched)?;
    let mut rows = Vec::new();
    let mut charge = 0;
    for (id, (decision, classification)) in controls
        .iter()
        .filter(|(id, _)| subject.is_none_or(|wanted| **id == wanted))
        .skip(start)
        .take(page.limit)
    {
        checkpoint(control)?;
        let entry = requirements.get(&(plan.catalog_key, *id)).ok_or(Reason::IncompleteClosure)?;
        retain(
            &mut rows,
            ApplicabilityRow {
                control_id: (*id).to_owned(),
                decision_state: *decision,
                decision_source: if decision.is_some() {
                    "explicit-record"
                } else {
                    "implicit-under-review"
                },
                classification: *classification,
                citations: vec![citation(entry, control)?],
            },
            &mut charge,
        )?;
    }
    Ok(page_result(generation, matched, start, rows))
}

/// Retain the full native summary while emitting at most one admitted page.
pub(super) fn gaps(
    scope: &CapturedQueryScope,
    generation: String,
    key: &str,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<GapData> {
    let plan = select_domain(scope, key, control)?;
    let requirements = requirements::collect(scope, Some(plan.catalog_key), control)?;
    let controls = control_set(scope, &plan, &requirements, control)?;
    let matched = controls.len();
    let start = page_start(&generation, page, matched)?;
    let mut rows = Vec::new();
    let mut charge = 0;
    for (id, (_, classification)) in controls.iter().skip(start).take(page.limit) {
        checkpoint(control)?;
        let entry = requirements.get(&(plan.catalog_key, *id)).ok_or(Reason::IncompleteClosure)?;
        retain(
            &mut rows,
            GapRow {
                control_id: (*id).to_owned(),
                classification: *classification,
                citations: vec![citation(entry, control)?],
            },
            &mut charge,
        )?;
    }
    let page = page_result(generation, matched, start, rows);
    Ok(GapData {
        generation: page.generation,
        summary: plan.prepared.counts().clone(),
        matched: page.matched,
        emitted: page.emitted,
        next_cursor: page.next_cursor,
        rows: page.rows,
    })
}

/// Prove every full native control/source tuple before a filter or page can hide it.
fn control_set<'a>(
    scope: &CapturedQueryScope,
    plan: &'a DomainPlan<'_>,
    requirements: &RequirementSet<'_>,
    control: &mut dyn WorkControl,
) -> QueryResult<ControlSet<'a>> {
    let mut rows = BTreeMap::new();
    for row in plan.prepared.control_rows() {
        checkpoint(control)?;
        if rows.len() >= MAX_CONTROLS
            || !safe_token(row.control_id())
            || !requirements.contains_key(&(plan.catalog_key, row.control_id()))
            || rows.contains_key(row.control_id())
        {
            return Err(Reason::IncompleteClosure.into());
        }
        scope.charge_relationships(1)?;
        rows.insert(row.control_id(), (row.decision_state(), row.classification()));
    }
    if rows.len() != plan.prepared.counts().total || rows.len() != requirements.len() {
        return Err(Reason::IncompleteClosure.into());
    }
    Ok(rows)
}

/// Require exactly one full current report match among independently approved manifests.
///
/// A stored raw manifest hash is a candidate filter only. Ordinary invalid
/// candidates do not become current reports; interruption and shared-accounting
/// failures propagate unchanged, and a second full match refuses the whole query.
fn select_domain<'a>(
    scope: &'a CapturedQueryScope,
    key: &str,
    control: &mut dyn WorkControl,
) -> QueryResult<DomainPlan<'a>> {
    let selected = domain_candidate(scope, key, control)?;
    match selected.role() {
        Role::ApplicabilityManifest => manifest_plan(scope, selected, control),
        Role::ApplicabilityReport => {
            let hint = selected
                .json()
                .and_then(|value| value.get("manifest_sha256"))
                .and_then(Value::as_str)
                .filter(|value| valid_hash(value))
                .ok_or(Reason::InvalidArtifact)?;
            let mut matched = None;
            for candidate_key in scope.visible_keys() {
                checkpoint(control)?;
                let candidate = scope.resource(candidate_key).ok_or(Reason::IncompleteClosure)?;
                if candidate.role() != Role::ApplicabilityManifest || candidate.raw_sha256() != hint
                {
                    continue;
                }
                scope.charge_relationships(1)?;
                let candidate = match domain_candidate(scope, candidate_key, control) {
                    Ok(candidate) => candidate,
                    Err(QueryError::Unavailable(_)) => continue,
                    Err(error) => return Err(error),
                };
                let plan = match manifest_plan(scope, candidate, control) {
                    Ok(plan) => plan,
                    Err(QueryError::Unavailable(_)) => continue,
                    Err(error) => return Err(error),
                };
                let compared = run_domain(scope, |charge| {
                    plan.prepared.validate_report(selected.bytes(), charge, control)
                });
                match compared {
                    Ok(()) if matched.is_none() => matched = Some(plan),
                    Ok(()) => return Err(Reason::IncompleteClosure.into()),
                    Err(QueryError::Unavailable(_)) => {}
                    Err(error) => return Err(error),
                }
            }
            matched.ok_or_else(|| Reason::IncompleteClosure.into())
        }
        _ => Err(Reason::UnsupportedRole.into()),
    }
}

/// Select an actual approved recorded source original, never a generated substitute.
fn domain_candidate<'a>(
    scope: &'a CapturedQueryScope,
    key: &str,
    control: &mut dyn WorkControl,
) -> QueryResult<CapturedResource<'a>> {
    checkpoint(control)?;
    if !scope.visible_keys().any(|visible| visible == key) {
        return Err(Reason::VisibilityRefused.into());
    }
    let candidate =
        scope.domain_source_candidate(key, control)?.ok_or(Reason::ApprovalUnavailable)?;
    let resource = candidate.resource();
    if !matches!(resource.role(), Role::ApplicabilityManifest | Role::ApplicabilityReport) {
        return Err(Reason::UnsupportedRole.into());
    }
    if !resource.pin_matches() || resource.native_identity().is_some() {
        return Err(Reason::InvalidArtifact.into());
    }
    Ok(resource)
}

/// Build only a charged borrowed roster from already-declared actual originals.
///
/// The maintained helper reparses the whole manifest and validates every native
/// identity, asserted subject fingerprint, current report field and classification.
/// This path admission never authorizes I/O or grants its dependencies approval.
fn manifest_plan<'a>(
    scope: &'a CapturedQueryScope,
    manifest: CapturedResource<'a>,
    control: &mut dyn WorkControl,
) -> QueryResult<DomainPlan<'a>> {
    checkpoint(control)?;
    let value = manifest.json().ok_or(Reason::InvalidArtifact)?;
    let framework = value.get("framework").ok_or(Reason::InvalidArtifact)?;
    let artifact = path_field(framework, "artifact")?;
    let native = approved_at(scope, manifest.path(), artifact, control)?;
    let (kind, catalog_key, resolved) = match native.role() {
        Role::OscalCatalogArtifact => (ResourceType::Catalog, native.key(), None),
        Role::OscalProfileArtifact => {
            let relative = path_field(framework, "resolved_catalog")?;
            let catalog = approved_at(scope, manifest.path(), relative, control)?;
            if catalog.role() != Role::OscalCatalogArtifact {
                return Err(Reason::IncompleteClosure.into());
            }
            (ResourceType::Profile, catalog.key(), Some(catalog))
        }
        _ => return Err(Reason::UnsupportedRole.into()),
    };
    if framework.get("type").and_then(Value::as_str) != Some(kind.as_str()) {
        return Err(Reason::InvalidArtifact.into());
    }
    let declared = value
        .get("mapping_collections")
        .and_then(Value::as_array)
        .ok_or(Reason::InvalidArtifact)?;
    if declared.len() > MAX_MAPPINGS {
        return Err(Reason::InvalidArtifact.into());
    }
    let mut mappings = Vec::new();
    let mut mapping_keys = BTreeSet::new();
    for path in declared {
        checkpoint(control)?;
        let relative = path.as_str().ok_or(Reason::InvalidArtifact)?;
        let mapping = approved_at(scope, manifest.path(), relative, control)?;
        if mapping.role() != Role::MappingCollection || mapping_keys.contains(mapping.key()) {
            return Err(Reason::IncompleteClosure.into());
        }
        scope.charge_relationships(1)?;
        mapping_keys.insert(mapping.key());
        mappings.push(MappingInput { key: mapping.key(), bytes: mapping.bytes() });
    }
    let mut natives = Vec::new();
    for key in scope.visible_keys() {
        checkpoint(control)?;
        let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
        let resource_type = match resource.role() {
            Role::OscalCatalogArtifact => ResourceType::Catalog,
            Role::OscalProfileArtifact => ResourceType::Profile,
            _ => continue,
        };
        // This first profile supplies the explicit framework Profile companion.
        // Other Profiles have no inferred hash/href companion and must be refused
        // by the maintained helper if a mapping actually needs that unresolved pair.
        let approved = scope.approved(key, control)?.ok_or(Reason::Unapproved)?;
        let resource = approved.resource();
        super::validate_identity(resource.native_identity().ok_or(Reason::InvalidArtifact)?)?;
        scope.charge_relationships(1)?;
        natives.push(NativeInput {
            key,
            resource_type,
            bytes: resource.bytes(),
            resolved_catalog: if key == native.key() {
                resolved.map(|catalog| ResolvedInput { key: catalog.key(), bytes: catalog.bytes() })
            } else {
                None
            },
        });
    }
    let prepared = run_domain(scope, |charge| {
        captured::prepare(manifest.bytes(), native.key(), &natives, &mappings, charge, control)
    })?;
    if prepared.manifest_sha256() != manifest.raw_sha256() {
        return Err(Reason::InvalidArtifact.into());
    }
    Ok(DomainPlan {
        prepared,
        catalog_key,
        mapping_keys,
        profile_catalog: resolved.map(|catalog| (native.key(), catalog.key())),
    })
}

/// Resolve a validated private manifest-relative path solely into the captured roster.
fn approved_at<'a>(
    scope: &'a CapturedQueryScope,
    manifest_path: &str,
    original_relative: &str,
    control: &mut dyn WorkControl,
) -> QueryResult<CapturedResource<'a>> {
    let path =
        resolve_record_path(manifest_path, original_relative).ok_or(Reason::IncompleteClosure)?;
    let mut selected = None;
    for key in scope.visible_keys() {
        checkpoint(control)?;
        let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
        if resource.path() != path {
            continue;
        }
        if selected.is_some() {
            return Err(Reason::IncompleteClosure.into());
        }
        let approved = scope.approved(key, control)?.ok_or(Reason::Unapproved)?;
        selected = Some(approved.resource());
    }
    selected.ok_or_else(|| Reason::IncompleteClosure.into())
}

/// Borrow an exact actual path field before any composition or parent transform.
fn path_field<'a>(value: &'a Value, field: &str) -> QueryResult<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| Reason::InvalidArtifact.into())
}

/// Preserve sticky interruption and accounting failures while minimizing domain errors.
pub(super) fn run_domain<T>(
    scope: &CapturedQueryScope,
    work: impl FnOnce(&mut dyn FnMut(usize) -> WorkResult<()>) -> WorkResult<T>,
) -> QueryResult<T> {
    let mut ledger_error = None;
    let result = {
        let mut charge = |count| {
            let result = scope.charge_relationships(count);
            if ledger_error.is_none() {
                if let Err(error) = &result {
                    ledger_error = Some(match error {
                        WorkError::Failed(error) => WorkError::Failed(error.clone()),
                        WorkError::Interrupted(reason) => WorkError::Interrupted(*reason),
                    });
                }
            }
            result
        };
        work(&mut charge)
    };
    if let Some(error) = ledger_error {
        return Err(QueryError::Work(error));
    }
    match result {
        Ok(value) => Ok(value),
        Err(error @ WorkError::Interrupted(_)) => Err(QueryError::Work(error)),
        Err(WorkError::Failed(_)) => Err(Reason::InvalidArtifact.into()),
    }
}

/// Admit the exact raw lower-case SHA-256 hint; it never proves a current report.
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Complete native trace locator, borrowing admitted helper facts rather than DTO prose.
struct MappingLocator<'a> {
    /// Exact original Mapping key.
    collection_key: &'a str,
    /// Exact native UUID spelling, never typed UUID reserialization.
    map_id: &'a str,
    /// Actual independently approved left native key after explicit resolution.
    source_key: &'a str,
    /// Actual current left control ID.
    source_id: &'a str,
    /// Actual independently approved right native key after explicit resolution.
    target_key: &'a str,
    /// Actual current right control ID.
    target_id: &'a str,
}

/// Stable borrowed relation identity across repeated references and domain manifests.
type MappingKey<'a> = (&'a str, &'a str, &'a str, &'a str, &'a str, &'a str);

/// Admit the complete visible Mapping universe before emitting any trace page.
///
/// Statement subjects with no actual requirement/source-span port are refused;
/// no inherited control citation is substituted. Unbound Mapping collections
/// cannot disappear from a direct-trace prefix advertised as complete.
#[allow(clippy::too_many_lines)] // Complete native relationship proof and projection remain one auditable bounded traversal.
pub(super) fn trace(
    scope: &CapturedQueryScope,
    generation: String,
    artifact_key: &str,
    control_id: &str,
    page: &Page,
    control: &mut dyn WorkControl,
) -> QueryResult<Paged<TraceRow>> {
    if !scope.visible_keys().any(|key| key == artifact_key) {
        return Err(Reason::VisibilityRefused.into());
    }
    let selected = scope.resource(artifact_key).ok_or(Reason::IncompleteClosure)?;
    if !matches!(selected.role(), Role::OscalCatalogArtifact | Role::OscalComponentArtifact) {
        return Err(Reason::UnsupportedRole.into());
    }
    let requirements = requirements::collect(scope, None, control)?;
    let mut plans = Vec::new();
    let mut covered = BTreeSet::new();
    let has_mappings = scope.visible_keys().any(|key| {
        scope.resource(key).is_some_and(|resource| resource.role() == Role::MappingCollection)
    });
    if has_mappings {
        for key in scope.visible_keys() {
            checkpoint(control)?;
            let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
            if resource.role() != Role::ApplicabilityManifest {
                continue;
            }
            let plan = manifest_plan(scope, domain_candidate(scope, key, control)?, control)?;
            for mapping in &plan.mapping_keys {
                if !covered.contains(mapping) {
                    scope.charge_relationships(1)?;
                }
                covered.insert(*mapping);
            }
            scope.charge_relationships(1)?;
            plans.push(plan);
        }
        for key in scope.visible_keys() {
            checkpoint(control)?;
            let resource = scope.resource(key).ok_or(Reason::IncompleteClosure)?;
            if resource.role() == Role::MappingCollection && !covered.contains(key) {
                return Err(Reason::IncompleteClosure.into());
            }
        }
    }
    let mut relations: BTreeMap<MappingKey<'_>, MappingLocator<'_>> = BTreeMap::new();
    for plan in &plans {
        // The helper returns the entire admitted native pair roster for this ID.
        // Both exact endpoints are checked before a selected-artifact filter.
        for relation in plan.prepared.relations(control_id) {
            checkpoint(control)?;
            if relation.source_subject_type() != SubjectType::Control
                || relation.target_subject_type() != SubjectType::Control
            {
                return Err(Reason::SourceSpanUnavailable.into());
            }
            let source_key = citation_key(plan, relation.source_key());
            let target_key = citation_key(plan, relation.target_key());
            if !plan.mapping_keys.contains(relation.collection_key())
                || !safe_token(relation.source_id())
                || !safe_token(relation.target_id())
                || relation.map_id().len() > 45
                || uuid::Uuid::parse_str(relation.map_id()).is_err()
                || !requirements.contains_key(&(source_key, relation.source_id()))
                || !requirements.contains_key(&(target_key, relation.target_id()))
            {
                return Err(Reason::IncompleteClosure.into());
            }
            // Exact native identity/fingerprint relation was proved by the helper;
            // selection here is not a hash-only resource lookup or href resolution.
            if (source_key != artifact_key || relation.source_id() != control_id)
                && (target_key != artifact_key || relation.target_id() != control_id)
            {
                continue;
            }
            let key = (
                relation.collection_key(),
                relation.map_id(),
                source_key,
                relation.source_id(),
                target_key,
                relation.target_id(),
            );
            if relations.contains_key(&key) {
                continue;
            }
            scope.charge_relationships(1)?;
            relations.insert(
                key,
                MappingLocator {
                    collection_key: relation.collection_key(),
                    map_id: relation.map_id(),
                    source_key,
                    source_id: relation.source_id(),
                    target_key,
                    target_id: relation.target_id(),
                },
            );
        }
    }
    let mut direct_keys = Vec::new();
    for entry in requirements
        .values()
        .filter(|entry| entry.artifact_key == artifact_key && entry.control_id == control_id)
    {
        checkpoint(control)?;
        scope.charge_relationships(1)?;
        direct_keys.push((entry.artifact_key, entry.id));
    }
    let matched =
        direct_keys.len().checked_add(relations.len()).ok_or(Reason::OutputBoundExceeded)?;
    let start = page_start(&generation, page, matched)?;
    let mut rows = Vec::new();
    let mut charge = 0;
    for (ordinal, (key, id)) in direct_keys.iter().enumerate() {
        if ordinal < start || rows.len() == page.limit {
            continue;
        }
        let entry = requirements.get(&(*key, *id)).ok_or(Reason::IncompleteClosure)?;
        retain(
            &mut rows,
            TraceRow {
                control_id: control_id.to_owned(),
                requirement_id: entry.id.to_owned(),
                mapping_id: None,
                relation: "exact-recorded-control",
                citations: vec![citation(entry, control)?],
                evidence_metadata: None,
            },
            &mut charge,
        )?;
    }
    for (ordinal, relation) in relations.values().enumerate() {
        let ordinal = direct_keys.len() + ordinal;
        if ordinal < start || rows.len() == page.limit {
            continue;
        }
        let source = requirements
            .get(&(relation.source_key, relation.source_id))
            .ok_or(Reason::IncompleteClosure)?;
        let target = requirements
            .get(&(relation.target_key, relation.target_id))
            .ok_or(Reason::IncompleteClosure)?;
        retain(
            &mut rows,
            TraceRow {
                control_id: control_id.to_owned(),
                requirement_id: control_id.to_owned(),
                mapping_id: Some(relation.map_id.to_owned()),
                relation: "recorded-mapping",
                citations: vec![
                    citation(source, control)?,
                    citation(target, control)?,
                    mapping_citation(scope, relation, control)?,
                ],
                evidence_metadata: None,
            },
            &mut charge,
        )?;
    }
    Ok(page_result(generation, matched, start, rows))
}

/// Apply only the explicit Profile/resolved-Catalog pair retained in this plan.
fn citation_key<'a>(plan: &'a DomainPlan<'_>, key: &'a str) -> &'a str {
    match plan.profile_catalog {
        Some((profile, catalog)) if profile == key => catalog,
        _ => key,
    }
}

/// Cite the actual native Mapping row while preserving its original UUID spelling.
fn mapping_citation(
    scope: &CapturedQueryScope,
    relation: &MappingLocator<'_>,
    control: &mut dyn WorkControl,
) -> QueryResult<Citation> {
    let approved = scope.approved(relation.collection_key, control)?.ok_or(Reason::Unapproved)?;
    let resource = approved.resource();
    let tree = resource.native_value().ok_or(Reason::InvalidArtifact)?;
    let root = tree.get("mapping-collection").ok_or(Reason::InvalidArtifact)?;
    let mappings = root.get("mappings").and_then(Value::as_array).ok_or(Reason::InvalidArtifact)?;
    let mut selected = None;
    for mapping in mappings {
        checkpoint(control)?;
        let maps = mapping.get("maps").and_then(Value::as_array).ok_or(Reason::InvalidArtifact)?;
        for node in maps {
            checkpoint(control)?;
            if node.get("uuid").and_then(Value::as_str) == Some(relation.map_id) {
                if selected.is_some() {
                    return Err(Reason::IncompleteClosure.into());
                }
                selected = Some(node);
            }
        }
    }
    let node = selected.ok_or(Reason::IncompleteClosure)?;
    let identity = resource.native_identity().ok_or(Reason::InvalidArtifact)?;
    super::validate_identity(identity)?;
    Ok(Citation {
        artifact_key: resource.key().to_owned(),
        artifact_identity: Some(identity.clone()),
        artifact_raw_sha256: resource.raw_sha256().to_owned(),
        citation_label: resource.citation_label().to_owned(),
        native_pointer: requirements::pointer_for_node(tree, node, control)?,
        source_key: None,
        source_raw_sha256: None,
        source_span: None,
        source_state: "native-only",
        artifact_role: Role::MappingCollection,
    })
}
