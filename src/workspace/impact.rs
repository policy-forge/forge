//! Complete read-only framework-impact inspection from one confined registered capture.
//!
//! Private staging contains only the admitted dependency closure. The existing engine proves
//! native identities, Profile companions, mapping/applicability freshness and prior-report pairs.
//! Cooperative fences do not preempt engine loops, parsers or system calls. No report, effect,
//! disposition or authoritative project write is published; assertions remain locally declared.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};

use crate::framework::manifest::{FrameworkResource, ImpactManifest};
use crate::framework::model::{ChangeClass, ImpactFilters, ImpactFinding, ImpactReport};
use crate::mapping::manifest::ResourceType;

use super::contract::{Error, Result};
use super::index::Role;
use super::inspection::{self, Query};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::services::{Item, Snapshot};

/// An exact admitted manifest and the unique captured registrations its engine may read.
struct Comparison<'a> {
    /// Selected captured authoring manifest; its registration is the comparison identity.
    item: &'a Item,
    /// Intrinsically parsed manifest retaining original relative references and raw pins.
    manifest: ImpactManifest,
    /// Complete registration-order indices; repeated legitimate edges share one captured item.
    closure: BTreeSet<usize>,
}

/// Private staging failures use a fixed safe message and never leak filesystem diagnostics.
fn internal() -> Error {
    Error::new("internal-error", "The captured inspection could not be completed.", false)
}

/// Compact complete inventory state; only the requested page becomes JSON rows.
struct InventoryRow<'a> {
    /// Exact captured authorial comparison registration.
    item: &'a Item,
    /// Intrinsic declared evidence, absent only when the raw authoring manifest is invalid.
    manifest: Option<ImpactManifest>,
    /// Structural/closure availability without current analysis or readiness authority.
    availability: &'static str,
    /// Fixed allowlisted redacted reason, with no parser text or private path.
    diagnostic: Option<&'static str>,
}

/// Check the same runtime-owned budget without publishing operation progress or resetting it.
fn boundary(control: &mut dyn WorkControl) -> WorkResult<()> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)
}

/// Select one exact registered comparison ID; no path query or filesystem discovery is allowed.
fn selected<'a>(snapshot: &'a Snapshot, id: &str) -> Result<&'a Item> {
    snapshot
        .items
        .iter()
        .find(|item| {
            item.registration.role == Role::FrameworkImpactManifest
                && item.metadata["resource_id"].as_str() == Some(id)
        })
        .ok_or_else(|| Error::new("not-found", "The selected comparison is not registered.", false))
}

/// Parse captured native OSCAL bytes under the existing semantic/schema validator.
/// This is independent of the legacy capture's globally selected applicability label.
fn native(item: &Item, kind: crate::validate::OscalModelType) -> Result<()> {
    let value = super::contract::parse(&item.captured.bytes, 10 * 1024 * 1024, 64 * 1024)
        .map_err(|_| inspection::domain_failure())?;
    if !crate::validate::run_full_validation("captured inspection", &value, kind)
        .is_ok_and(|report| report.is_valid())
    {
        return Err(inspection::domain_failure());
    }
    Ok(())
}

/// Resolve and add a role-admitted exact registration, never opening its declared path.
fn add<'a>(
    snapshot: &'a Snapshot,
    closure: &mut BTreeSet<usize>,
    base: &str,
    relative: &Path,
    roles: &[Role],
) -> Result<&'a Item> {
    let item = inspection::reference(snapshot, base, relative, roles)?;
    let index = snapshot
        .items
        .iter()
        .position(|candidate| std::ptr::eq(candidate, item))
        .ok_or_else(inspection::domain_failure)?;
    closure.insert(index);
    Ok(item)
}

/// Admit a Catalog/Profile plus its exact role-admitted Catalog companion before any engine read.
fn framework(
    snapshot: &Snapshot,
    closure: &mut BTreeSet<usize>,
    base: &str,
    resource: &FrameworkResource,
) -> Result<()> {
    let (role, kind) = match resource.resource_type {
        ResourceType::Catalog => {
            (Role::OscalCatalogArtifact, crate::validate::OscalModelType::Catalog)
        }
        ResourceType::Profile => {
            (Role::OscalProfileArtifact, crate::validate::OscalModelType::Profile)
        }
    };
    native(add(snapshot, closure, base, &resource.artifact, &[role])?, kind)?;
    if let Some(companion) = &resource.resolved_catalog {
        native(
            add(snapshot, closure, base, companion, &[Role::OscalCatalogArtifact])?,
            crate::validate::OscalModelType::Catalog,
        )?;
    }
    Ok(())
}

/// Admit the raw applicability author's transitive framework and native mapping declarations.
/// The engine subsequently proves its exact old baseline, raw hashes, decisions and rerun counts.
fn applicability(
    snapshot: &Snapshot,
    closure: &mut BTreeSet<usize>,
    base: &str,
    relative: &Path,
) -> Result<()> {
    let item = add(snapshot, closure, base, relative, &[Role::ApplicabilityManifest])?;
    let manifest = crate::applicability::manifest::parse(&item.captured.bytes)
        .map_err(|_| inspection::domain_failure())?;
    let (role, kind) = match manifest.framework.resource_type {
        ResourceType::Catalog => {
            (Role::OscalCatalogArtifact, crate::validate::OscalModelType::Catalog)
        }
        ResourceType::Profile => {
            (Role::OscalProfileArtifact, crate::validate::OscalModelType::Profile)
        }
    };
    let base = &item.registration.path;
    native(add(snapshot, closure, base, &manifest.framework.artifact, &[role])?, kind)?;
    if let Some(companion) = &manifest.framework.resolved_catalog {
        native(
            add(snapshot, closure, base, companion, &[Role::OscalCatalogArtifact])?,
            crate::validate::OscalModelType::Catalog,
        )?;
    }
    for path in &manifest.mapping_collections {
        native(
            add(snapshot, closure, base, path, &[Role::MappingCollection])?,
            crate::validate::OscalModelType::Mapping,
        )?;
    }
    Ok(())
}

/// Preflight the complete captured closure, with typed fences outside ordinary domain failures.
/// No global metadata validity bit substitutes for exact selected dependency admission.
fn admit<'a>(
    snapshot: &'a Snapshot,
    item: &'a Item,
    control: &mut dyn WorkControl,
) -> WorkResult<Comparison<'a>> {
    boundary(control)?;
    let manifest = crate::framework::manifest::parse(&item.captured.bytes)
        .map_err(|_| inspection::domain_failure());
    boundary(control)?;
    let manifest = manifest?;
    let mut closure = BTreeSet::new();
    let index = snapshot
        .items
        .iter()
        .position(|candidate| std::ptr::eq(candidate, item))
        .ok_or_else(inspection::domain_failure)?;
    closure.insert(index);
    let base = &item.registration.path;
    for resource in [&manifest.old, &manifest.new] {
        boundary(control)?;
        let result = framework(snapshot, &mut closure, base, resource);
        boundary(control)?;
        result?;
    }
    for dependency in &manifest.mapping_collections {
        boundary(control)?;
        let result =
            add(snapshot, &mut closure, base, &dependency.artifact, &[Role::MappingCollection])
                .and_then(|item| native(item, crate::validate::OscalModelType::Mapping));
        boundary(control)?;
        result?;
    }
    if let Some(path) = &manifest.applicability_manifest {
        boundary(control)?;
        let result = applicability(snapshot, &mut closure, base, path);
        boundary(control)?;
        result?;
    }
    if let Some(path) = &manifest.successor_map {
        boundary(control)?;
        let item = add(snapshot, &mut closure, base, path, &[Role::SuccessorMap])?;
        let result = crate::migration::parse_successor(&item.captured.bytes)
            .map_err(|_| inspection::domain_failure());
        boundary(control)?;
        result?;
    }
    if let (Some(prior), Some(dispositions)) = (&manifest.prior_report, &manifest.disposition_file)
    {
        boundary(control)?;
        let prior = add(snapshot, &mut closure, base, prior, &[Role::FrameworkImpactReport])?;
        let result = crate::framework::analysis::admit_prior_report(&prior.captured.bytes)
            .map_err(|_| inspection::domain_failure());
        boundary(control)?;
        result?;
        let file =
            add(snapshot, &mut closure, base, dispositions, &[Role::FrameworkImpactDispositions])?;
        let result = crate::framework::disposition::parse(&file.captured.bytes)
            .map_err(|_| inspection::domain_failure());
        boundary(control)?;
        result?;
    }
    Ok(Comparison { item, manifest, closure })
}

/// Copy only admitted captured bytes into an owned private directory with per-file fences.
/// Resource paths were admitted by the explicit index and contain no parent/alias components.
fn stage(
    snapshot: &Snapshot,
    comparison: &Comparison<'_>,
    control: &mut dyn WorkControl,
) -> WorkResult<tempfile::TempDir> {
    boundary(control)?;
    let directory = tempfile::tempdir().map_err(|_| internal())?;
    for index in &comparison.closure {
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        let item = &snapshot.items[*index];
        let path = directory.path().join(&item.registration.path);
        let parent = path.parent().ok_or_else(internal)?;
        std::fs::create_dir_all(parent).map_err(|_| internal())?;
        std::fs::write(&path, &item.captured.bytes).map_err(|_| internal())?;
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
    }
    Ok(directory)
}

/// Reject whole unfiltered collection excess before any detail or filtered page can be encoded.
/// Explicit counts keep this fence executable without constructing oversized synthetic vectors.
fn check_cardinality(changes: usize, emitted: usize, hidden: usize, prior: usize) -> Result<()> {
    if changes > 100_000
        || prior > 100_000
        || emitted.checked_add(hidden).is_none_or(|total| total > 100_000)
    {
        return Err(Error::new(
            "payload-too-large",
            "The complete inspection exceeds its supported bound.",
            false,
        ));
    }
    Ok(())
}

/// Compute the unchanged domain report and reconcile every returned input to the staged closure.
/// The engine's contained reads operate only on owned captured copies, never the project Root.
fn compute(
    snapshot: &Snapshot,
    comparison: &Comparison<'_>,
    filters: ImpactFilters,
    control: &mut dyn WorkControl,
) -> WorkResult<ImpactReport> {
    let directory = stage(snapshot, comparison, control)?;
    let manifest_path = directory.path().join(&comparison.item.registration.path);
    let base = manifest_path.parent().ok_or_else(internal)?;
    boundary(control)?;
    let analyzed = crate::framework::analysis::analyze(base, &comparison.manifest, filters)
        .map_err(|_| inspection::domain_failure());
    boundary(control)?;
    let (report, paths) = analyzed?;
    check_cardinality(
        report.changes.len(),
        report.findings.len(),
        report.filtered_out_findings.len(),
        report.prior_only_dispositions.len(),
    )?;
    let admitted = comparison
        .closure
        .iter()
        .map(|index| {
            directory
                .path()
                .join(&snapshot.items[*index].registration.path)
                .canonicalize()
                .map_err(|_| internal())
        })
        .collect::<Result<BTreeSet<_>>>()?;
    for path in paths {
        let observed = path.canonicalize().map_err(|_| inspection::domain_failure())?;
        if !admitted.contains(&observed) {
            return Err(inspection::domain_failure().into());
        }
    }
    boundary(control)?;
    Ok(report)
}

/// Project a declared or proven resource identity, omitting hrefs and file content.
fn fingerprint(snapshot: &Snapshot, item: &Item, resource: &FrameworkResource) -> Result<Value> {
    let role = match resource.resource_type {
        ResourceType::Catalog => Role::OscalCatalogArtifact,
        ResourceType::Profile => Role::OscalProfileArtifact,
    };
    let artifact =
        inspection::reference(snapshot, &item.registration.path, &resource.artifact, &[role])?;
    let companion = resource
        .resolved_catalog
        .as_ref()
        .map(|path| {
            inspection::reference(
                snapshot,
                &item.registration.path,
                path,
                &[Role::OscalCatalogArtifact],
            )
        })
        .transpose()?;
    Ok(
        json!({"resource_id":artifact.metadata["resource_id"],"resource_type":resource.resource_type,
        "raw_sha256":resource.expected_sha256,"root_uuid":resource.root_uuid,
        "document_version":resource.document_version,"oscal_version":resource.oscal_version,
        "resolved_catalog_sha256":resource.expected_resolved_catalog_sha256,
        "resolved_catalog_resource_id":companion.map(|item| item.metadata["resource_id"].clone())}),
    )
}

/// Bind subpage findings to the unfiltered comparison detail, independently of page filters.
fn comparison_version(snapshot: &Snapshot, id: &str) -> Result<String> {
    let empty = Query::new(&[], &[])?;
    Ok(inspection::version(snapshot, "impact-comparison", id, &empty))
}

/// Preserve the domain's exact five AND filters and reject unsupported enum or token spellings.
fn filters(query: &Query<'_>) -> Result<ImpactFilters> {
    let decision_state = query
        .optional("decision_state")
        .map(|value| match value {
            "applicable" => Ok(crate::applicability::manifest::DecisionState::Applicable),
            "not-applicable" => Ok(crate::applicability::manifest::DecisionState::NotApplicable),
            "deferred" => Ok(crate::applicability::manifest::DecisionState::Deferred),
            "under-review" => Ok(crate::applicability::manifest::DecisionState::UnderReview),
            _ => Err(Error::invalid()),
        })
        .transpose()?;
    let priority = query
        .optional("priority")
        .map(|value| match value {
            "blocking" => Ok(crate::framework::model::FindingPriority::Blocking),
            "review-required" => Ok(crate::framework::model::FindingPriority::ReviewRequired),
            "informational" => Ok(crate::framework::model::FindingPriority::Informational),
            _ => Err(Error::invalid()),
        })
        .transpose()?;
    for name in ["group", "owner", "policy_source"] {
        if query.optional(name).is_some_and(|value| {
            value.len() > 4096 || value.trim() != value || value.chars().any(char::is_control)
        }) {
            return Err(Error::invalid());
        }
    }
    Ok(ImpactFilters {
        group: query.optional("group").map(str::to_owned),
        decision_state,
        policy_source: query.optional("policy_source").map(str::to_owned),
        priority,
        owner: query.optional("owner").map(str::to_owned),
    })
}

/// Parse the closed optional change-class filter before selecting any rows.
fn change_filter(query: &Query<'_>) -> Result<Option<ChangeClass>> {
    query
        .optional("change_class")
        .map(|value| match value {
            "added" => Ok(ChangeClass::Added),
            "removed" => Ok(ChangeClass::Removed),
            "content-changed" => Ok(ChangeClass::ContentChanged),
            "identity-migrated" => Ok(ChangeClass::IdentityMigrated),
            "unchanged" => Ok(ChangeClass::Unchanged),
            _ => Err(Error::invalid()),
        })
        .transpose()
}

/// Remove rationale from a migration assertion without authenticating its supplied actor.
fn migration(value: Option<&crate::framework::model::IdentityMigrationEvidence>) -> Value {
    value.map_or(Value::Null, |value| {
        json!({"relationship":value.relationship,
        "approved_by":value.approved_by,"approved_at":value.approved_at})
    })
}

/// Remove rationale from a pair-validated historical disposition without granting risk approval.
fn disposition(value: &crate::framework::disposition::DispositionRecord) -> Value {
    json!({"finding_id":value.finding_id,"status":value.status,
        "decided_by":value.decided_by,"decided_at":value.decided_at})
}

/// Serialize the required closed nullable change shape without default policy/framework prose.
fn change(value: &crate::framework::model::ControlChange) -> Value {
    json!({"subject_id":value.subject_id,"change_class":value.change_class,
        "old_sha256":value.old_sha256,"new_sha256":value.new_sha256,
        "old_subjects":value.old_subjects,"new_subjects":value.new_subjects,
        "migration":migration(value.migration.as_ref())})
}

/// Serialize one exact-pair finding while omitting dependency paths and all rationale fields.
fn finding(value: &ImpactFinding, id: &str, version: &str) -> Value {
    json!({"finding_id":value.finding_id,"comparison_id":id,"comparison_version":version,
        "priority":value.priority,"reason_code":value.reason_code,"required_action":value.required_action,
        "subject_id":value.subject_id,"change_class":value.change_class,
        "old_sha256":value.old_sha256,"new_sha256":value.new_sha256,
        "old_subjects":value.old_subjects,"new_subjects":value.new_subjects,
        "migration":migration(value.migration.as_ref()),"framework_groups":value.framework_groups,
        "affected_artifact_id":value.affected_artifact_id,"dependency_id":value.dependency_id,
        "policy_resource_identity":value.policy_resource_identity,
        "prior_gap_classification":value.prior_gap_classification,"prior_decision_state":value.prior_decision_state,
        "owner":value.owner,"policy_sources":value.policy_sources,
        "disposition":value.disposition.as_ref().map(disposition)})
}

/// Count disposition states across emitted and hidden findings while preserving full detection totals.
/// Native report summary's disposition fields remain separately labelled as complete-filtered scope.
fn full_summary(report: &ImpactReport) -> crate::framework::model::ChangeSummary {
    let mut summary = report.summary.clone();
    summary.dispositioned_resolved = 0;
    summary.dispositioned_accepted_risk = 0;
    summary.dispositioned_still_open = 0;
    summary.undispositioned = 0;
    for finding in report.findings.iter().chain(&report.filtered_out_findings) {
        match finding.disposition.as_ref().map(|value| value.status) {
            Some(crate::framework::disposition::DispositionStatus::Resolved) => {
                summary.dispositioned_resolved += 1;
            }
            Some(crate::framework::disposition::DispositionStatus::AcceptedRisk) => {
                summary.dispositioned_accepted_risk += 1;
            }
            Some(crate::framework::disposition::DispositionStatus::StillOpen) => {
                summary.dispositioned_still_open += 1;
            }
            None => summary.undispositioned += 1,
        }
    }
    summary
}

/// Decorate an impact page with captured and exact-comparison context before bounded publication.
fn decorate(snapshot: &Snapshot, mut page: Value, id: &str, counts: Value) -> Result<Value> {
    page["snapshot_version"] = json!(snapshot.version);
    page["comparison_id"] = json!(id);
    page["comparison_version"] = json!(comparison_version(snapshot, id)?);
    page["availability"] = json!("available");
    page["counts"] = counts;
    Ok(page)
}

/// List complete registered manifest availability without claiming a current comparison result.
/// Absent/legacy/empty index states remain distinct from invalid or unavailable declared inputs.
pub(crate) fn comparisons(
    snapshot: &Snapshot,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    let query = Query::new(raw, &["page_size", "cursor"])?;
    inspection::validate_snapshot(snapshot, control)?;
    let candidates = snapshot
        .items
        .iter()
        .filter(|item| item.registration.role == Role::FrameworkImpactManifest)
        .collect::<Vec<_>>();
    let mut rows = Vec::with_capacity(candidates.len());
    let mut unavailable = 0;
    for item in &candidates {
        boundary(control)?;
        let parsed = crate::framework::manifest::parse(&item.captured.bytes);
        boundary(control)?;
        let (manifest, availability, diagnostic) = match parsed {
            Err(_) => (None, "invalid", Some("invalid-impact-manifest")),
            Ok(manifest) => match admit(snapshot, item, control) {
                Err(super::preparation::WorkError::Interrupted(reason)) => {
                    return Err(super::preparation::WorkError::Interrupted(reason));
                }
                Err(super::preparation::WorkError::Failed(_)) => {
                    (Some(manifest), "unavailable", Some("impact-input-unavailable"))
                }
                Ok(_) => (Some(manifest), "valid", None),
            },
        };
        if availability != "valid" {
            unavailable += 1;
        }
        rows.push(InventoryRow { item, manifest, availability, diagnostic });
    }
    let version = inspection::version(snapshot, "impact-comparisons", "", &query);
    let mut value = inspection::page(rows.len(), &version, &query, |index| {
        let row = &rows[index];
        let old = row
            .manifest
            .as_ref()
            .and_then(|manifest| fingerprint(snapshot, row.item, &manifest.old).ok());
        let new = row
            .manifest
            .as_ref()
            .and_then(|manifest| fingerprint(snapshot, row.item, &manifest.new).ok());
        Ok(
            json!({"comparison_id":row.item.metadata["resource_id"],"resource_id":row.item.metadata["resource_id"],
            "manifest_sha256":row.item.captured.sha256,"old":old,"new":new,"availability":row.availability,
            "diagnostic_code":row.diagnostic,"freshness":"not-computed"}),
        )
    })?;
    value["snapshot_version"] = json!(snapshot.version);
    let availability = inspection::family_availability(snapshot, candidates.len());
    value["availability"] = json!(if unavailable > 0 && availability == "available" {
        "needs-attention"
    } else {
        availability
    });
    value["counts"] =
        json!({"registered_comparisons":candidates.len(),"unavailable_comparisons":unavailable});
    inspection::finish(value, control)
}

/// Compute one complete current comparison from its captured registered dependency closure.
/// Prior admission is labelled limited-structural even after the engine verifies raw hash/pair.
pub(crate) fn detail(
    snapshot: &Snapshot,
    id: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    inspection::require_index2(snapshot)?;
    let query = Query::new(raw, &[])?;
    inspection::validate_snapshot(snapshot, control)?;
    let comparison = admit(snapshot, selected(snapshot, id)?, control)?;
    let report = compute(snapshot, &comparison, ImpactFilters::default(), control)?;
    let base = &comparison.item.registration.path;
    let prior = comparison
        .manifest
        .prior_report
        .as_ref()
        .map(|path| inspection::reference(snapshot, base, path, &[Role::FrameworkImpactReport]))
        .transpose()?;
    let value = json!({"resource_version":inspection::version(snapshot,"impact-comparison",id,&query),
        "snapshot_version":snapshot.version,"comparison_id":id,"resource_id":id,"freshness":"captured-current",
        "old":fingerprint(snapshot,comparison.item,&comparison.manifest.old)?,
        "new":fingerprint(snapshot,comparison.item,&comparison.manifest.new)?,"summary":full_summary(&report),
        "provenance":inspection::provenance(snapshot,&comparison.closure.iter().copied().collect::<Vec<_>>())?,
        "prior_report_sha256":prior.map(|item|item.captured.sha256.clone()),
        "prior_report_admission":prior.map(|_|"limited-structural"),
        "trust_boundary":"review dispositions and migration assertions are declared locally; FORGE does not authenticate reviewers or approve risk"});
    inspection::finish(value, control)
}

/// Page the complete change inventory with an exact class filter and captured comparison binding.
pub(crate) fn changes(
    snapshot: &Snapshot,
    id: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    inspection::require_index2(snapshot)?;
    let query = Query::new(raw, &["change_class", "page_size", "cursor"])?;
    inspection::validate_snapshot(snapshot, control)?;
    let filter = change_filter(&query)?;
    let comparison = admit(snapshot, selected(snapshot, id)?, control)?;
    let report = compute(snapshot, &comparison, ImpactFilters::default(), control)?;
    let matching = report
        .changes
        .iter()
        .filter(|value| filter.is_none_or(|class| value.change_class == class))
        .collect::<Vec<_>>();
    let version = inspection::version(snapshot, "impact-changes", id, &query);
    let page =
        inspection::page(matching.len(), &version, &query, |index| Ok(change(matching[index])))?;
    let value = decorate(
        snapshot,
        page,
        id,
        json!({"total_changes":report.changes.len(),"matching_changes":matching.len()}),
    )?;
    inspection::finish(value, control)
}

/// Page exact AND-filtered findings while retaining complete detection and disposition denominators.
/// A hidden blocking finding never disappears from `full_summary` or changes the domain gate.
pub(crate) fn findings(
    snapshot: &Snapshot,
    id: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    inspection::require_index2(snapshot)?;
    let query = Query::new(
        raw,
        &["group", "decision_state", "policy_source", "priority", "owner", "page_size", "cursor"],
    )?;
    inspection::validate_snapshot(snapshot, control)?;
    let filters = filters(&query)?;
    let comparison = admit(snapshot, selected(snapshot, id)?, control)?;
    let report = compute(snapshot, &comparison, filters, control)?;
    let version = inspection::version(snapshot, "impact-findings", id, &query);
    let pair = comparison_version(snapshot, id)?;
    let page = inspection::page(report.findings.len(), &version, &query, |index| {
        Ok(finding(&report.findings[index], id, &pair))
    })?;
    let mut value = decorate(
        snapshot,
        page,
        id,
        json!({"total_findings":report.summary.findings,"matching_findings":report.findings.len()}),
    )?;
    value["full_summary"] = json!(full_summary(&report));
    value["emitted_dispositions"] = json!({"resolved":report.summary.dispositioned_resolved,
        "accepted_risk":report.summary.dispositioned_accepted_risk,
        "still_open":report.summary.dispositioned_still_open,"undispositioned":report.summary.undispositioned});
    inspection::finish(value, control)
}

/// Page prior-only historical assertions after the unchanged engine's exact report hash/pair check.
/// Current matched dispositions remain attached to findings; prior-only rows do not resolve them.
pub(crate) fn prior_dispositions(
    snapshot: &Snapshot,
    id: &str,
    raw: &[(String, String)],
    control: &mut dyn WorkControl,
) -> WorkResult<Value> {
    inspection::require_index2(snapshot)?;
    let query = Query::new(raw, &["page_size", "cursor"])?;
    inspection::validate_snapshot(snapshot, control)?;
    let comparison = admit(snapshot, selected(snapshot, id)?, control)?;
    let report = compute(snapshot, &comparison, ImpactFilters::default(), control)?;
    let version = inspection::version(snapshot, "impact-prior-dispositions", id, &query);
    let page = inspection::page(report.prior_only_dispositions.len(), &version, &query, |index| {
        Ok(disposition(&report.prior_only_dispositions[index]))
    })?;
    let value = decorate(
        snapshot,
        page,
        id,
        json!({"total_prior_dispositions":report.prior_only_dispositions.len()}),
    )?;
    inspection::finish(value, control)
}

#[cfg(test)]
mod tests {
    use super::super::preparation::{Interruption, NoopControl, WorkError};
    use super::super::root::Root;
    use super::*;

    /// Owned synthetic files plus a real confined capture, with no external source authority.
    struct Fixture {
        /// Temporary authoring root retained for sentinel/mutation checks.
        directory: tempfile::TempDir,
        /// Actual captured index and bytes; private Snapshot fields are never fabricated.
        snapshot: Snapshot,
        /// Stable registered comparison identity from actual capture metadata.
        id: String,
        /// Exact authoring manifest for deliberate controlled fixture successors.
        manifest: Value,
    }

    /// Write synthetic fixture JSON and return the hash of the actual emitted bytes.
    fn write(path: &Path, value: &Value) -> String {
        let bytes = serde_json::to_vec_pretty(value).unwrap();
        std::fs::write(path, &bytes).unwrap();
        crate::hashing::sha256_hex(&bytes)
    }

    /// Construct native Catalog fixtures matching the exercised existing PRD057 test builder.
    /// Prose is deliberately private; inspection output must contain only hashes and typed facts.
    fn catalog(version: &str, controls: &[(&str, &str)]) -> Value {
        json!({"catalog":{"uuid":"77777777-7777-4777-8777-777777777777",
            "metadata":{"title":"Synthetic framework","last-modified":"2026-08-25T12:00:00Z",
                "version":version,"oscal-version":"1.2.3"},
            "groups":[{"id":"group-1","title":"Synthetic controls","controls":controls.iter().map(|(id, prose)|
                json!({"id":id,"title":format!("Control {id}"),"parts":[{"id":format!("{id}_smt"),"name":"statement","prose":prose}]})).collect::<Vec<_>>()}]}})
    }

    /// Capture an explicit index2 with one comparison, its full pair and an unrelated registration.
    fn fixture() -> Fixture {
        let directory = tempfile::tempdir().unwrap();
        let old = write(
            &directory.path().join("old.json"),
            &catalog(
                "1.0.0",
                &[
                    ("unchanged", "SECRET same"),
                    ("changed", "SECRET old"),
                    ("removed", "SECRET removed"),
                ],
            ),
        );
        let new = write(
            &directory.path().join("new.json"),
            &catalog(
                "2.0.0",
                &[
                    ("unchanged", "SECRET same"),
                    ("changed", "SECRET revised"),
                    ("added", "SECRET added"),
                ],
            ),
        );
        let resource = |path: &str, hash: &str, version: &str| {
            json!({"type":"catalog","artifact":path,
            "expected_sha256":hash,"root_uuid":"77777777-7777-4777-8777-777777777777",
            "document_version":version,"oscal_version":"1.2.3"})
        };
        let manifest = json!({"schema_version":"forge.framework-impact/1",
            "old":resource("old.json",&old,"1.0.0"),"new":resource("new.json",&new,"2.0.0"),"mapping_collections":[]});
        write(&directory.path().join("impact.json"), &manifest);
        std::fs::write(directory.path().join("unrelated.bin"), b"SECRET unrelated").unwrap();
        write(
            &directory.path().join("forge.workspace.json"),
            &json!({"schema_version":"forge.workspace/2","label":"Synthetic",
            "resources":[{"key":"comparison","role":"framework-impact-manifest","path":"impact.json"},
                {"key":"old","role":"oscal-catalog-artifact","path":"old.json"},
                {"key":"new","role":"oscal-catalog-artifact","path":"new.json"},
                {"key":"unrelated","role":"lifecycle-source","path":"unrelated.bin"}]}),
        );
        let snapshot = Snapshot::capture(&Root::open(directory.path()).unwrap()).unwrap();
        let id = snapshot.items[0].metadata["resource_id"].as_str().unwrap().to_owned();
        Fixture { directory, snapshot, id, manifest }
    }

    /// Install only owned synthetic authoring changes and take a fresh actual Root capture.
    fn recapture(fixture: &mut Fixture) {
        write(&fixture.directory.path().join("impact.json"), &fixture.manifest);
        fixture.snapshot =
            Snapshot::capture(&Root::open(fixture.directory.path()).unwrap()).unwrap();
    }

    /// Exact decoded query tokens matching ordinary HTTP/client requests.
    fn query(values: &[(&str, &str)]) -> Vec<(String, String)> {
        values.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect()
    }

    /// Sticky component cancellation without worker threads, clocks or production pause hooks.
    struct Stopped;

    impl WorkControl for Stopped {
        /// Preserve typed cancellation at the first boundary before any private staging.
        fn checkpoint(&mut self, _: Stage, _: ProgressUpdate) -> WorkResult<()> {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        }

        /// Preserve the same internal stop reason for final runtime mapping.
        fn interruption(&self) -> Option<Interruption> {
            Some(Interruption::CancelRequested)
        }
    }

    /// Exact pair computation retains full raw detection and omits prose/paths/approval authority.
    #[test]
    fn computed_pair_projects_complete_counts_and_captured_context() {
        let fixture = fixture();
        let result = detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_eq!(result["summary"]["old_controls"], 3);
        assert_eq!(result["summary"]["new_controls"], 3);
        assert_eq!(result["summary"]["added"], 1);
        assert_eq!(result["summary"]["removed"], 1);
        assert_eq!(result["summary"]["content_changed"], 1);
        assert_eq!(result["summary"]["unchanged"], 1);
        assert_eq!(result["freshness"], "captured-current");
        assert_eq!(result["snapshot_version"], fixture.snapshot.version);
        assert_eq!(result["provenance"].as_array().unwrap().len(), 3);
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains("SECRET"));
        assert!(!serialized.contains("href"));
        assert!(!serialized.contains("unrelated.bin"));
        assert!(result["prior_report_admission"].is_null());
    }

    /// Engine reads remain captured-only after original files change; only admitted bytes are staged.
    #[test]
    fn private_stage_excludes_unrelated_input_and_ignores_original_drift() {
        let fixture = fixture();
        let before = detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        let comparison =
            admit(&fixture.snapshot, &fixture.snapshot.items[0], &mut NoopControl).unwrap();
        let staged = stage(&fixture.snapshot, &comparison, &mut NoopControl).unwrap();
        assert!(!staged.path().join("unrelated.bin").exists());
        for index in &comparison.closure {
            let item = &fixture.snapshot.items[*index];
            assert_eq!(
                std::fs::read(staged.path().join(&item.registration.path)).unwrap(),
                item.captured.bytes
            );
        }
        let stage_path = staged.path().to_owned();
        drop(staged);
        assert!(!stage_path.exists());
        std::fs::write(fixture.directory.path().join("old.json"), b"CHANGED AFTER CAPTURE")
            .unwrap();
        std::fs::write(fixture.directory.path().join("new.json"), b"CHANGED AFTER CAPTURE")
            .unwrap();
        assert_eq!(detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap(), before);
        assert_eq!(
            std::fs::read(fixture.directory.path().join("old.json")).unwrap(),
            b"CHANGED AFTER CAPTURE"
        );
    }

    /// An unregistered supplied path is never opened or converted to partial current state.
    #[test]
    fn incomplete_or_wrong_native_closure_fails_without_changing_sentinel() {
        let mut fixture = fixture();
        let sentinel = fixture.directory.path().join("unregistered.json");
        std::fs::write(&sentinel, b"SECRET sentinel").unwrap();
        fixture.manifest["mapping_collections"] =
            json!([{"artifact":"unregistered.json","framework_role":"source"}]);
        recapture(&mut fixture);
        assert!(matches!(detail(&fixture.snapshot,&fixture.id,&[],&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed"));
        assert_eq!(std::fs::read(sentinel).unwrap(), b"SECRET sentinel");
        let inventory = comparisons(&fixture.snapshot, &[], &mut NoopControl).unwrap();
        assert_eq!(inventory["availability"], "needs-attention");
        assert_eq!(inventory["page"]["items"][0]["availability"], "unavailable");
        assert_eq!(inventory["page"]["items"][0]["freshness"], "not-computed");
        assert_eq!(inventory["counts"]["unavailable_comparisons"], 1);
    }

    /// Filtered zero rows retain full detection gates and exact comparison context; no hidden credit.
    #[test]
    fn filters_preserve_full_summary_and_never_rebind_foreign_cursors() {
        let fixture = fixture();
        let complete = detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        let filtered = findings(
            &fixture.snapshot,
            &fixture.id,
            &query(&[("owner", "nobody")]),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(filtered["full_summary"], complete["summary"]);
        assert_eq!(filtered["comparison_version"], complete["resource_version"]);
        assert_eq!(filtered["snapshot_version"], complete["snapshot_version"]);
        assert_eq!(filtered["page"]["items"], json!([]));
        assert_eq!(filtered["counts"]["matching_findings"], 0);
        assert_eq!(
            filtered["emitted_dispositions"],
            json!({"resolved":0,"accepted_risk":0,"still_open":0,"undispositioned":0})
        );
        let first = changes(
            &fixture.snapshot,
            &fixture.id,
            &query(&[("page_size", "1")]),
            &mut NoopControl,
        )
        .unwrap();
        let cursor = first["page"]["next_cursor"].as_str().unwrap();
        assert!(
            matches!(changes(&fixture.snapshot,&fixture.id,&query(&[("page_size","1"),("change_class","removed"),("cursor",cursor)]),&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "version-conflict")
        );
        assert!(
            matches!(findings(&fixture.snapshot,&fixture.id,&query(&[("group","unknown-group")]),&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed")
        );
    }

    /// Typed stops and invalid captured hashes cannot degrade to invalid rows or partial report success.
    #[test]
    fn capture_tamper_and_interruption_reject_all_selected_publication() {
        let mut fixture = fixture();
        assert!(matches!(
            comparisons(&fixture.snapshot, &[], &mut Stopped),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert!(matches!(
            detail(&fixture.snapshot, &fixture.id, &[], &mut Stopped),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        fixture.snapshot.items[1].captured.bytes.push(b' ');
        assert!(matches!(detail(&fixture.snapshot,&fixture.id,&[],&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed"));
        assert!(matches!(comparisons(&fixture.snapshot,&[],&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed"));
    }
    /// Derive pair evidence from the actual engine and add one synthetic historical finding ID.
    /// The added minimal ID deliberately exercises limited prior admission, not complete report
    /// validation, an authentic historical computation, reviewer judgment or risk acceptance.
    fn add_prior(fixture: &mut Fixture) {
        let comparison =
            admit(&fixture.snapshot, &fixture.snapshot.items[0], &mut NoopControl).unwrap();
        let report =
            compute(&fixture.snapshot, &comparison, ImpactFilters::default(), &mut NoopControl)
                .unwrap();
        let matched = report.findings[0].finding_id.clone();
        let prior_only = "11111111-1111-4111-8111-111111111111";
        assert!(report.findings.iter().all(|finding| finding.finding_id != prior_only));
        let mut prior = serde_json::to_value(&report).unwrap();
        prior["findings"].as_array_mut().unwrap().push(json!({"finding_id":prior_only}));
        let hash = write(&fixture.directory.path().join("prior.json"), &prior);
        write(
            &fixture.directory.path().join("dispositions.json"),
            &json!({
            "schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":hash,
            "dispositions":[{"finding_id":matched,"status":"resolved","decided_by":"declared-reviewer",
                "decided_at":"2026-08-25T15:00:00Z","rationale":"SECRET rationale"},
                {"finding_id":prior_only,"status":"accepted-risk","decided_by":"declared-reviewer",
                "decided_at":"2026-08-25T15:00:00Z","rationale":"SECRET prior-only rationale"}]}),
        );
        fixture.manifest["prior_report"] = json!("prior.json");
        fixture.manifest["disposition_file"] = json!("dispositions.json");
        let mut index = serde_json::to_value(&fixture.snapshot.index).unwrap();
        index["resources"].as_array_mut().unwrap().extend([
            json!({"key":"prior","role":"framework-impact-report","path":"prior.json"}),
            json!({"key":"dispositions","role":"framework-impact-dispositions","path":"dispositions.json"})]);
        write(&fixture.directory.path().join("forge.workspace.json"), &index);
        recapture(fixture);
    }

    /// Exact prior binding preserves historical prior-only rows separately from current dispositions.
    #[test]
    fn prior_only_history_and_raw_pair_hash_cannot_be_conflated() {
        let mut fixture = fixture();
        add_prior(&mut fixture);
        let detail = detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_eq!(detail["prior_report_admission"], "limited-structural");
        assert_eq!(detail["summary"]["dispositioned_resolved"], 1);
        let history =
            prior_dispositions(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_eq!(history["counts"]["total_prior_dispositions"], 1);
        assert_eq!(
            history["page"]["items"][0]["finding_id"],
            "11111111-1111-4111-8111-111111111111"
        );
        assert_eq!(history["comparison_version"], detail["resource_version"]);
        assert!(!serde_json::to_string(&history).unwrap().contains("rationale"));
        let path = fixture.directory.path().join("dispositions.json");
        let mut changed: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        changed["prior_report_sha256"] = json!("0".repeat(64));
        write(&path, &changed);
        recapture(&mut fixture);
        assert!(matches!(super::detail(&fixture.snapshot,&fixture.id,&[],&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed"));
    }

    /// Direct captured-structure limits are separate from native admission and cannot silently narrow.
    #[test]
    fn capture_bound_and_alias_guards_are_explicit_before_selected_engine() {
        let mut fixture = fixture();
        fixture.snapshot.index.resources.resize(1001, fixture.snapshot.index.resources[0].clone());
        assert!(matches!(inspection::validate_snapshot(&fixture.snapshot,&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "payload-too-large"));
        fixture.snapshot.index.resources.truncate(4);
        fixture.snapshot.items[1].captured.bytes = vec![b'x'; 10 * 1024 * 1024];
        fixture.snapshot.items[1].captured.sha256 =
            crate::hashing::sha256_hex(&fixture.snapshot.items[1].captured.bytes);
        assert!(inspection::validate_snapshot(&fixture.snapshot, &mut NoopControl).is_ok());
        fixture.snapshot.items[1].captured.bytes.push(b'x');
        assert!(matches!(inspection::validate_snapshot(&fixture.snapshot,&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "payload-too-large"));
        let mut original = self::fixture();
        original.snapshot.items[2].captured.identity = original.snapshot.items[1].captured.identity;
        assert!(matches!(inspection::validate_snapshot(&original.snapshot,&mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "resource-containment"));
    }

    /// Exact whole counts include hidden findings and checked overflow, regardless of selected page.
    #[test]
    fn full_cardinality_guard_cannot_be_bypassed_by_filters() {
        assert!(check_cardinality(100_000, 50_000, 50_000, 100_000).is_ok());
        for (changes, emitted, hidden, prior) in
            [(100_001, 0, 0, 0), (0, 1, 100_000, 0), (0, 0, 0, 100_001), (0, usize::MAX, 1, 0)]
        {
            assert_eq!(
                check_cardinality(changes, emitted, hidden, prior).unwrap_err().code,
                "payload-too-large"
            );
        }
    }

    /// Rebind one synthetic admitted prior ID without normalizing its exact raw spelling.
    /// The report header/pair originate in the engine fixture, while this inserted history
    /// record deliberately exercises only existing limited prior-report admission.
    fn rebind_prior_only_id(fixture: &mut Fixture, finding_id: &str) {
        let prior_path = fixture.directory.path().join("prior.json");
        let mut prior: Value =
            serde_json::from_slice(&std::fs::read(&prior_path).unwrap()).unwrap();
        let canonical = "11111111-1111-4111-8111-111111111111";
        let historical = prior["findings"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|item| item["finding_id"] == canonical)
            .unwrap();
        historical["finding_id"] = json!(finding_id);
        let hash = write(&prior_path, &prior);
        let disposition_path = fixture.directory.path().join("dispositions.json");
        let mut dispositions: Value =
            serde_json::from_slice(&std::fs::read(&disposition_path).unwrap()).unwrap();
        let declaration = dispositions["dispositions"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|item| item["finding_id"] == canonical)
            .unwrap();
        declaration["finding_id"] = json!(finding_id);
        dispositions["prior_report_sha256"] = json!(hash);
        write(&disposition_path, &dispositions);
        recapture(fixture);
    }

    /// Every native-admitted prior-only UUID spelling survives exact projection and the wire schema.
    /// Invalid prefix casing remains rejected by the existing native parser; these synthetic
    /// historical records do not establish authentic prior computation or review authority.
    #[test]
    fn prior_only_uuid_spellings_remain_exact_and_schema_compatible() {
        for raw in [
            "F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4",
            "F9168C5ECEB24faaB6BF329BF39FA1E4",
            "{F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4}",
            "urn:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4",
        ] {
            let mut fixture = fixture();
            add_prior(&mut fixture);
            rebind_prior_only_id(&mut fixture, raw);
            let history =
                prior_dispositions(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
            assert_eq!(history["counts"]["total_prior_dispositions"], 1);
            assert_eq!(history["page"]["items"][0]["finding_id"], raw);
            assert_eq!(history["page"]["items"][0]["status"], "accepted-risk");
            assert!(!serde_json::to_string(&history).unwrap().contains("rationale"));
            crate::workspace::contract::validate_for(
                crate::workspace::contract::ApiMajor::V2,
                "FrameworkImpactPriorDispositionPage",
                &history,
            )
            .unwrap();
        }
        let mut invalid = fixture();
        add_prior(&mut invalid);
        rebind_prior_only_id(&mut invalid, "URN:uuid:F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4");
        assert!(matches!(prior_dispositions(&invalid.snapshot, &invalid.id,
            &[], &mut NoopControl), Err(WorkError::Failed(error))
            if error.code == "validation-failed"));
    }

    /// Native policy-to-framework Mapping authoring; the existing builder creates all UUIDs,
    /// inventories and exact resource/subject hashes. Synthetic reviewer evidence is not approval.
    fn finding_mapping_manifest() -> Value {
        json!({"schema_version":"forge.mapping-manifest/1",
            "collection":{"key":"s3-finding-collection","title":"SECRET mapping title",
                "version":"1.0.0","last_modified":"2026-08-25T12:00:00Z"},
            "reviewers":[{"key":"mapping-reviewer","type":"person","name":"SECRET person"}],
            "provenance":{"method":"human","matching_rationale":"semantic","status":"complete",
                "mapping_description":"SECRET description","reviewer_keys":["mapping-reviewer"],
                "reviewed_at":"2026-08-25T12:00:00Z"},
            "mapping":{"key":"s3-finding-mapping","scope":"control-only",
                "source":{"type":"catalog","artifact":"policy.json","href":"policy.json"},
                "target":{"type":"catalog","artifact":"old.json","href":"old.json"},
                "maps":(["changed","removed"].iter().map(|id|json!({
                    "key":format!("s3-{id}-edge"),"relationship":"intersects-with",
                    "sources":[{"type":"control","id_ref":"policy-1"}],
                    "targets":[{"type":"control","id_ref":id}],
                    "reviewer_key":"mapping-reviewer","reviewed_at":"2026-08-25T12:00:00Z",
                    "rationale":"SECRET reviewed mapping rationale"})).collect::<Vec<_>>())}})
    }

    /// Extend the captured Catalog pair with a real built Mapping and pinned applicability
    /// scaffold. The same target-role Mapping is declared in both native portfolios, so the
    /// shared engine must prove their raw hashes, counts, decisions and policy-source agreement.
    fn finding_fixture() -> Fixture {
        let mut fixture = fixture();
        let mut policy = catalog("1.0.0", &[("policy-1", "SECRET policy prose")]);
        policy["catalog"]["uuid"] = json!("88888888-8888-4888-8888-888888888888");
        write(&fixture.directory.path().join("policy.json"), &policy);
        let mapping_path = fixture.directory.path().join("mapping-manifest.json");
        write(&mapping_path, &finding_mapping_manifest());
        let built = crate::mapping::prepare(&mapping_path, None, false).unwrap();
        std::fs::write(fixture.directory.path().join("mapping.json"), built.artifact_json).unwrap();
        let applicability_path = fixture.directory.path().join("applicability.json");
        crate::applicability::execute_init(
            &fixture.directory.path().join("old.json"),
            None,
            Some(&applicability_path),
        )
        .unwrap();
        let mut applicability: Value =
            serde_json::from_slice(&std::fs::read(&applicability_path).unwrap()).unwrap();
        applicability["mapping_collections"] = json!(["mapping.json"]);
        applicability["reviewers"] = json!([
            {"key":"scope-reviewer","type":"person","name":"SECRET scope person"},
            {"key":"changed-reviewer","type":"person","name":"SECRET changed person"}]);
        applicability["decisions"] = json!([
            {"control_id":"removed","state":"applicable","reviewer_key":"scope-reviewer",
                "reviewed_at":"2026-08-25T12:00:00Z","rationale":"SECRET applicability rationale"},
            {"control_id":"changed","state":"applicable","reviewer_key":"changed-reviewer",
                "reviewed_at":"2026-08-25T12:00:00Z","note":"SECRET private note"}]);
        write(&applicability_path, &applicability);
        fixture.manifest["applicability_manifest"] = json!("applicability.json");
        fixture.manifest["mapping_collections"] =
            json!([{"artifact":"mapping.json","framework_role":"target"}]);
        let mut index = serde_json::to_value(&fixture.snapshot.index).unwrap();
        index["resources"].as_array_mut().unwrap().extend([
            json!({"key":"policy","role":"oscal-catalog-artifact","path":"policy.json"}),
            json!({"key":"finding-mapping","role":"mapping-collection","path":"mapping.json"}),
            json!({"key":"finding-applicability","role":"applicability-manifest","path":"applicability.json"})]);
        write(&fixture.directory.path().join("forge.workspace.json"), &index);
        recapture(&mut fixture);
        fixture
    }

    /// Bind three local dispositions to actual current native finding IDs and the exact prior
    /// report bytes. No synthetic finding ID, inferred reviewer approval or wire fixture is used.
    fn attach_current_finding_dispositions(fixture: &mut Fixture) -> Vec<Value> {
        let comparison =
            admit(&fixture.snapshot, &fixture.snapshot.items[0], &mut NoopControl).unwrap();
        let report =
            compute(&fixture.snapshot, &comparison, ImpactFilters::default(), &mut NoopControl)
                .unwrap();
        let blocking = report
            .findings
            .iter()
            .find(|finding| finding.priority == crate::framework::model::FindingPriority::Blocking)
            .unwrap();
        let changed = report
            .findings
            .iter()
            .find(|finding| finding.owner.as_deref() == Some("changed-reviewer"))
            .unwrap();
        let removed = report
            .findings
            .iter()
            .find(|finding| finding.owner.as_deref() == Some("scope-reviewer"))
            .unwrap();
        let expected: Vec<Value> = [
            (&blocking.finding_id, "resolved"),
            (&changed.finding_id, "accepted-risk"),
            (&removed.finding_id, "still-open"),
        ]
        .into_iter()
        .map(|(id, status)| {
            json!({"finding_id":id,"status":status,
            "decided_by":"declared-reviewer","decided_at":"2026-08-25T15:00:00Z"})
        })
        .collect();
        let prior_hash = write(
            &fixture.directory.path().join("finding-prior.json"),
            &serde_json::to_value(&report).unwrap(),
        );
        let mut records = expected.clone();
        for value in &mut records {
            value["rationale"] = json!("SECRET current disposition rationale");
        }
        write(
            &fixture.directory.path().join("finding-dispositions.json"),
            &json!({"schema_version":"forge.framework-impact-dispositions/1",
                "prior_report_sha256":prior_hash,"dispositions":records}),
        );
        fixture.manifest["prior_report"] = json!("finding-prior.json");
        fixture.manifest["disposition_file"] = json!("finding-dispositions.json");
        let mut index = serde_json::to_value(&fixture.snapshot.index).unwrap();
        index["resources"].as_array_mut().unwrap().extend([
            json!({"key":"finding-prior","role":"framework-impact-report","path":"finding-prior.json"}),
            json!({"key":"finding-dispositions","role":"framework-impact-dispositions","path":"finding-dispositions.json"})]);
        write(&fixture.directory.path().join("forge.workspace.json"), &index);
        recapture(fixture);
        expected
    }

    /// Check the full nonempty captured finding envelope and actual selected-major wire schema;
    /// private domain paths/prose/rationale stay absent while declared relative source labels remain.
    fn assert_current_finding_wire(fixture: &Fixture, value: &Value) {
        assert!(value["counts"]["total_findings"].as_u64().unwrap() > 0);
        assert_eq!(value["comparison_id"], fixture.id);
        assert_eq!(value["snapshot_version"], fixture.snapshot.version);
        assert_eq!(value["availability"], "available");
        let serialized = serde_json::to_string(value).unwrap();
        for private in [
            "SECRET",
            "\"rationale\":",
            "\"dependency_path\":",
            "mapping-manifest.json",
            "unrelated.bin",
        ] {
            assert!(!serialized.contains(private), "unexpected private marker: {private}");
        }
        assert!(!serialized.contains(fixture.directory.path().to_str().unwrap()));
        crate::workspace::contract::validate_for(
            crate::workspace::contract::ApiMajor::V2,
            "FrameworkImpactFindingPage",
            value,
        )
        .unwrap();
    }

    /// Require the same fixed domain failure before detail or findings can publish a partial pair.
    fn assert_invalid_finding_read(fixture: &Fixture) {
        for read in [
            detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl),
            findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl),
        ] {
            let Err(WorkError::Failed(error)) = read else {
                panic!("incomplete or stale finding closure must refuse complete publication");
            };
            assert_eq!(error.code, "validation-failed");
            assert_eq!(
                error.message,
                "The captured inputs cannot produce this complete inspection."
            );
            assert!(error.resource_version.is_none());
        }
    }

    /// Nonempty native applicability/mapping findings project exact states, owners, source labels,
    /// hashes and dependency IDs with complete detection counts, without private review evidence.
    #[test]
    fn captured_native_applicability_findings_are_nonempty_and_redacted() {
        let fixture = finding_fixture();
        let complete = findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_current_finding_wire(&fixture, &complete);
        let items = complete["page"]["items"].as_array().unwrap();
        assert_eq!(complete["page"]["total_matching"], items.len());
        assert_eq!(complete["counts"]["matching_findings"], items.len());
        assert_eq!(complete["counts"]["total_findings"], items.len());
        let row = items.iter().find(|row| row["owner"] == "scope-reviewer").unwrap();
        assert_eq!(row["reason_code"], "applicability_decision_removed");
        assert_eq!(row["required_action"], "review-applicability-decision");
        assert_eq!(row["subject_id"], "removed");
        assert_eq!(row["priority"], "review-required");
        assert_eq!(row["prior_decision_state"], "applicable");
        assert_eq!(row["prior_gap_classification"], "applicable-mapped");
        assert_eq!(row["policy_sources"], json!(["policy.json"]));
        assert_eq!(row["framework_groups"], json!(["group-1"]));
        assert_eq!(row["dependency_id"], "applicability:removed");
        assert!(row["affected_artifact_id"].as_str().is_some_and(|id| id.len() == 64));
        assert!(row["old_sha256"].as_str().is_some_and(|hash| hash.len() == 64));
        assert!(row["new_sha256"].is_null());
        assert!(row["disposition"].is_null());
        let detail = detail(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_eq!(complete["full_summary"], detail["summary"]);
        assert_eq!(complete["comparison_version"], detail["resource_version"]);
        assert!(detail["summary"]["blocking"].as_u64().unwrap() > 0);
        // The Mapping embeds policy identity; the native engine reads only the five declared closure files.
        assert_eq!(detail["provenance"].as_array().unwrap().len(), 5);
    }

    /// Each of five native filter dimensions retains a real applicability row; their conjunction
    /// is one row, and a contradictory priority hides it without removing complete blocking facts.
    #[test]
    fn five_finding_filters_use_nonempty_native_and_semantics() {
        let fixture = finding_fixture();
        let complete = findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        let predicates = [
            ("group", "group-1"),
            ("decision_state", "applicable"),
            ("policy_source", "policy.json"),
            ("priority", "review-required"),
            ("owner", "scope-reviewer"),
        ];
        for predicate in predicates {
            let selected =
                findings(&fixture.snapshot, &fixture.id, &query(&[predicate]), &mut NoopControl)
                    .unwrap();
            assert_current_finding_wire(&fixture, &selected);
            let rows = selected["page"]["items"].as_array().unwrap();
            assert!(rows.iter().any(|row| row["owner"] == "scope-reviewer"));
            assert!(rows.iter().all(|row| match predicate.0 {
                "group" =>
                    row["framework_groups"].as_array().unwrap().contains(&json!(predicate.1)),
                "policy_source" =>
                    row["policy_sources"].as_array().unwrap().contains(&json!(predicate.1)),
                "decision_state" => row["prior_decision_state"] == predicate.1,
                "priority" => row["priority"] == predicate.1,
                "owner" => row["owner"] == predicate.1,
                _ => unreachable!("fixed five-filter case inventory"),
            }));
            assert_eq!(selected["counts"]["matching_findings"], rows.len());
            assert_eq!(selected["full_summary"], complete["full_summary"]);
        }
        for mismatch in [
            ("decision_state", "not-applicable"),
            ("policy_source", "missing-policy.json"),
            ("owner", "nobody"),
        ] {
            let empty =
                findings(&fixture.snapshot, &fixture.id, &query(&[mismatch]), &mut NoopControl)
                    .unwrap();
            assert_eq!(empty["page"]["items"], json!([]));
            assert_eq!(empty["full_summary"], complete["full_summary"]);
        }
        assert!(matches!(findings(&fixture.snapshot, &fixture.id,
            &query(&[("group", "unknown-group")]), &mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "validation-failed"));
        let selected =
            findings(&fixture.snapshot, &fixture.id, &query(&predicates), &mut NoopControl)
                .unwrap();
        assert_eq!(selected["counts"]["matching_findings"], 1);
        assert_eq!(selected["page"]["items"].as_array().unwrap().len(), 1);
        assert_eq!(selected["page"]["items"][0]["owner"], "scope-reviewer");
        let mut contradiction = predicates;
        contradiction[3] = ("priority", "blocking");
        let hidden =
            findings(&fixture.snapshot, &fixture.id, &query(&contradiction), &mut NoopControl)
                .unwrap();
        assert_eq!(hidden["page"]["items"], json!([]));
        assert_eq!(hidden["counts"]["matching_findings"], 0);
        assert_eq!(hidden["counts"]["total_findings"], complete["counts"]["total_findings"]);
        assert_eq!(hidden["full_summary"], complete["full_summary"]);
        assert!(hidden["full_summary"]["blocking"].as_u64().unwrap() > 0);
    }

    /// Exact native current IDs receive all three declared disposition states; owner filtering
    /// changes only emitted disposition counts and never erases hidden blockers or full states.
    #[test]
    fn current_dispositions_and_hidden_blockers_keep_separate_denominators() {
        let mut fixture = finding_fixture();
        let expected = attach_current_finding_dispositions(&mut fixture);
        let complete = findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_current_finding_wire(&fixture, &complete);
        for record in expected {
            let row = complete["page"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["finding_id"] == record["finding_id"])
                .unwrap();
            assert_eq!(row["disposition"], record);
        }
        for field in
            ["dispositioned_resolved", "dispositioned_accepted_risk", "dispositioned_still_open"]
        {
            assert_eq!(complete["full_summary"][field], 1);
        }
        assert_eq!(complete["emitted_dispositions"]["resolved"], 1);
        assert_eq!(complete["emitted_dispositions"]["accepted_risk"], 1);
        assert_eq!(complete["emitted_dispositions"]["still_open"], 1);
        let selected = findings(
            &fixture.snapshot,
            &fixture.id,
            &query(&[("owner", "scope-reviewer")]),
            &mut NoopControl,
        )
        .unwrap();
        assert_current_finding_wire(&fixture, &selected);
        assert_eq!(selected["counts"]["matching_findings"], 1);
        assert_eq!(selected["full_summary"], complete["full_summary"]);
        assert_eq!(selected["counts"]["total_findings"], complete["counts"]["total_findings"]);
        assert_eq!(
            selected["emitted_dispositions"],
            json!({"resolved":0,"accepted_risk":0,"still_open":1,"undispositioned":0})
        );
        assert!(selected["full_summary"]["blocking"].as_u64().unwrap() > 0);
        assert!(
            selected["page"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["priority"] != "blocking")
        );
        let prior =
            prior_dispositions(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        assert_eq!(prior["page"]["items"], json!([]));
        assert_eq!(prior["counts"]["total_prior_dispositions"], 0);
    }

    /// A real nonempty findings cursor cannot cross its selected comparison, exact filter tuple
    /// or a freshly captured registered byte set, even when both pairs share the same frameworks.
    #[test]
    fn native_finding_cursor_refuses_foreign_pair_filter_and_capture() {
        let mut fixture = finding_fixture();
        write(&fixture.directory.path().join("second-impact.json"), &fixture.manifest);
        let mut index = serde_json::to_value(&fixture.snapshot.index).unwrap();
        index["resources"].as_array_mut().unwrap().push(
            json!({"key":"second-comparison","role":"framework-impact-manifest","path":"second-impact.json"}));
        write(&fixture.directory.path().join("forge.workspace.json"), &index);
        recapture(&mut fixture);
        let second = fixture
            .snapshot
            .items
            .iter()
            .find(|item| item.registration.key == "second-comparison")
            .unwrap()
            .metadata["resource_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let first = findings(
            &fixture.snapshot,
            &fixture.id,
            &query(&[("page_size", "1")]),
            &mut NoopControl,
        )
        .unwrap();
        let cursor = first["page"]["next_cursor"].as_str().unwrap().to_owned();
        let page = query(&[("page_size", "1"), ("cursor", &cursor)]);
        for rejected in [
            findings(&fixture.snapshot, &second, &page, &mut NoopControl),
            findings(
                &fixture.snapshot,
                &fixture.id,
                &query(&[("page_size", "1"), ("owner", "scope-reviewer"), ("cursor", &cursor)]),
                &mut NoopControl,
            ),
        ] {
            assert!(matches!(rejected, Err(WorkError::Failed(error))
                if error.code == "version-conflict"));
        }
        let mut fresh = serde_json::from_slice::<Value>(
            &std::fs::read(fixture.directory.path().join("applicability.json")).unwrap(),
        )
        .unwrap();
        fresh["decisions"][0]["note"] = json!("SECRET new captured note");
        write(&fixture.directory.path().join("applicability.json"), &fresh);
        recapture(&mut fixture);
        assert_ne!(first["snapshot_version"], fixture.snapshot.version);
        assert!(matches!(findings(&fixture.snapshot, &fixture.id, &page, &mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "version-conflict"));
        let restarted = findings(
            &fixture.snapshot,
            &fixture.id,
            &query(&[("page_size", "1")]),
            &mut NoopControl,
        )
        .unwrap();
        assert_eq!(restarted["page"]["items"].as_array().unwrap().len(), 1);
        assert_ne!(first["resource_version"], restarted["resource_version"]);
    }

    /// Native applicability authoring and its transitive Mapping must both be registered;
    /// existing unregistered files cannot repair a missing captured closure or be rewritten.
    #[test]
    fn missing_registered_applicability_or_transitive_mapping_refuses_findings() {
        for missing in ["finding-applicability", "finding-mapping"] {
            let mut fixture = finding_fixture();
            let applicability_path = fixture.directory.path().join("applicability.json");
            let mapping_path = fixture.directory.path().join("mapping.json");
            let applicability_before = std::fs::read(&applicability_path).unwrap();
            let mapping_before = std::fs::read(&mapping_path).unwrap();
            if missing == "finding-mapping" {
                fixture.manifest["mapping_collections"] = json!([]);
            }
            let mut index = serde_json::to_value(&fixture.snapshot.index).unwrap();
            index["resources"]
                .as_array_mut()
                .unwrap()
                .retain(|resource| resource["key"] != missing);
            write(&fixture.directory.path().join("forge.workspace.json"), &index);
            recapture(&mut fixture);
            assert_invalid_finding_read(&fixture);
            assert_eq!(std::fs::read(&applicability_path).unwrap(), applicability_before);
            assert_eq!(std::fs::read(&mapping_path).unwrap(), mapping_before);
        }
    }

    /// Selected raw admission remains distinct from capture labels: valid captured bytes survive
    /// original drift, but a fresh baseline with old Mapping/applicability pins refuses analysis.
    #[test]
    fn captured_finding_freshness_is_not_a_metadata_validity_or_current_file_alias() {
        let mut fixture = finding_fixture();
        let before = findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap();
        let applicability = fixture
            .snapshot
            .items
            .iter_mut()
            .find(|item| item.registration.role == Role::ApplicabilityManifest)
            .unwrap();
        applicability.metadata["validation_state"] = json!("invalid");
        assert_eq!(
            findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap(),
            before
        );
        let old_path = fixture.directory.path().join("old.json");
        let mut changed: Value =
            serde_json::from_slice(&std::fs::read(&old_path).unwrap()).unwrap();
        changed["catalog"]["groups"][0]["controls"][1]["parts"][0]["prose"] =
            json!("SECRET newer old-baseline statement");
        let changed_hash = write(&old_path, &changed);
        assert_eq!(
            findings(&fixture.snapshot, &fixture.id, &[], &mut NoopControl).unwrap(),
            before
        );
        fixture.manifest["old"]["expected_sha256"] = json!(changed_hash);
        recapture(&mut fixture);
        assert_ne!(before["snapshot_version"], fixture.snapshot.version);
        assert_invalid_finding_read(&fixture);
        assert_eq!(std::fs::read(&old_path).unwrap(), serde_json::to_vec_pretty(&changed).unwrap());
    }
}
