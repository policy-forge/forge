//! Exact Catalog classification and PRD 055 Mapping Collection dependency traversal.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::manifest::{FrameworkResource, FrameworkRole, ImpactManifest, MappingDependency};
use super::model::{
    ChangeClass, ChangeSummary, FindingPriority, IdentityMigrationEvidence, ImpactFilters,
    ImpactFinding, ImpactReport, REPORT_SCHEMA_VERSION, ReasonCode, RequiredAction,
    SubjectFingerprint,
};
use crate::mapping::inventory::{Inventory, LoadedResource};
use crate::mapping::manifest::{Relationship, ResourceManifest, SubjectType};
use crate::mapping::model::{MappingCollectionEnvelope, MappingItem, OscalMap, OscalProp};
use crate::{ForgeError, io, json_strict::bounded, validate};

const MAX_FINDINGS: usize = 100_000;
const MAX_FILTER_BYTES: usize = 4 * 1024;
const MAPPING_JSON_LIMITS: crate::json_strict::Limits = crate::json_strict::Limits {
    max_depth: 64,
    max_string_bytes: crate::mapping::manifest::MAX_STRING_BYTES,
};

#[derive(Debug)]
struct MappingReference {
    artifact_id: String,
    mapping_id: String,
    map_id: String,
    policy_resource_identity: String,
    policy_resource_label: String,
}

#[derive(Debug, Default)]
struct MappingPortfolio {
    references: BTreeMap<String, Vec<MappingReference>>,
    applicability_facts: BTreeMap<String, crate::applicability::model::ControlMappingFacts>,
    target_collection_sha256s: BTreeSet<String>,
}

#[derive(Debug)]
struct FindingContext {
    identity: String,
    dependency_path: Vec<String>,
    affected_artifact_id: Option<String>,
    dependency_id: Option<String>,
    policy_resource_identity: Option<String>,
    prior_gap_classification: Option<String>,
    prior_decision_state: Option<crate::applicability::manifest::DecisionState>,
    owner: Option<String>,
    policy_sources: Vec<String>,
}

impl FindingContext {
    fn subject(subject_id: &str) -> Self {
        Self {
            identity: "subject".to_string(),
            dependency_path: vec![format!("control:{subject_id}")],
            affected_artifact_id: None,
            dependency_id: None,
            policy_resource_identity: None,
            prior_gap_classification: None,
            prior_decision_state: None,
            owner: None,
            policy_sources: Vec::new(),
        }
    }
}

/// Run a complete framework impact analysis.
///
/// # Errors
///
/// Returns [`ForgeError::FrameworkImpact`] if resource evidence, schema validation, mapping
/// baseline integrity, or configured bounds prevent a complete result.
pub fn analyze(
    manifest_dir: &Path,
    manifest: &ImpactManifest,
    filters: ImpactFilters,
) -> Result<(ImpactReport, Vec<PathBuf>), ForgeError> {
    validate_filters(&filters)?;
    let old = load_framework_resource(manifest_dir, "$.old", &manifest.old)?;
    let new = load_framework_resource(manifest_dir, "$.new", &manifest.new)?;
    if filters.group.is_some() {
        validate_group_ids("$.old", &old.inventory)?;
        validate_group_ids("$.new", &new.inventory)?;
    }
    let mut input_paths = vec![old.path.clone(), new.path.clone()];
    for resource in [&manifest.old, &manifest.new] {
        if let Some(companion) = &resource.resolved_catalog {
            input_paths.push(manifest_dir.join(companion));
        }
    }
    let portfolio = load_mapping_references(
        manifest_dir,
        &manifest.mapping_collections,
        &old,
        &mut input_paths,
    )?;
    let applicability = manifest
        .applicability_manifest
        .as_ref()
        .map(|path| {
            load_applicability(&manifest_dir.join(path), &old, &portfolio, &mut input_paths)
        })
        .transpose()?;
    let successor_map = manifest
        .successor_map
        .as_ref()
        .map(|path| load_successor_map(&manifest_dir.join(path), &mut input_paths))
        .transpose()?;
    let changes = classify(&old.inventory, &new.inventory, successor_map.as_ref())?;
    let mut findings =
        build_findings(&changes, &portfolio.references, applicability.as_ref(), &old, &new)?;
    add_metadata_finding(&mut findings, &changes, &old, &new)?;
    attach_framework_groups(&mut findings, &old.inventory, &new.inventory);
    findings.sort_by(|left, right| {
        (
            left.priority,
            left.subject_id.as_str(),
            left.reason_code,
            left.dependency_id.as_deref(),
            left.finding_id.as_str(),
        )
            .cmp(&(
                right.priority,
                right.subject_id.as_str(),
                right.reason_code,
                right.dependency_id.as_deref(),
                right.finding_id.as_str(),
            ))
    });
    let mut summary = summarize(&changes, &findings);
    summary.old_controls = old.inventory.count(SubjectType::Control);
    summary.new_controls = new.inventory.count(SubjectType::Control);
    let mut report = ImpactReport {
        schema_version: REPORT_SCHEMA_VERSION,
        status: "complete",
        old: old.evidence,
        new: new.evidence,
        summary,
        filters,
        matched_findings: findings.len(),
        changes,
        findings,
        filtered_out_findings: Vec::new(),
        prior_only_dispositions: Vec::new(),
    };
    if let (Some(prior_report), Some(disposition_file)) =
        (&manifest.prior_report, &manifest.disposition_file)
    {
        apply_dispositions(
            &manifest_dir.join(prior_report),
            &manifest_dir.join(disposition_file),
            &mut report,
            &mut input_paths,
        )?;
    }
    update_disposition_summary(&mut report);
    apply_filters(&mut report);
    Ok((report, input_paths))
}

fn validate_filters(filters: &ImpactFilters) -> Result<(), ForgeError> {
    for (name, value) in [
        ("--group", filters.group.as_deref()),
        ("--policy-source", filters.policy_source.as_deref()),
        ("--owner", filters.owner.as_deref()),
    ] {
        let Some(value) = value else { continue };
        if value.is_empty() || value.trim() != value {
            return Err(impact_error(format!(
                "{name} must be non-empty without leading or trailing whitespace"
            )));
        }
        if value.len() > MAX_FILTER_BYTES {
            return Err(impact_error(format!("{name} exceeds the {MAX_FILTER_BYTES} byte limit")));
        }
        if value.chars().any(char::is_control) {
            return Err(impact_error(format!("{name} must not contain control characters")));
        }
    }
    if let Some(policy_source) = &filters.policy_source {
        crate::applicability::manifest::validate_report_href("--policy-source", policy_source)
            .map_err(|error| impact_error(strip_applicability_error_prefix(&error.to_string())))?;
    }
    Ok(())
}

fn validate_group_ids(label: &str, inventory: &Inventory) -> Result<(), ForgeError> {
    if let Some(group_id) = inventory.ambiguous_group_ids().iter().next() {
        return Err(impact_error(format!(
            "{label} contains duplicate group id '{}'; framework group filtering would be ambiguous",
            bounded(group_id)
        )));
    }
    Ok(())
}

fn attach_framework_groups(findings: &mut [ImpactFinding], old: &Inventory, new: &Inventory) {
    for finding in findings {
        let mut groups = BTreeSet::new();
        for subject in finding.old_subjects.iter().chain(&finding.new_subjects) {
            groups.extend(old.groups_for_control(&subject.id).iter().cloned());
            groups.extend(new.groups_for_control(&subject.id).iter().cloned());
        }
        finding.framework_groups = groups.into_iter().collect();
    }
}

fn apply_filters(report: &mut ImpactReport) {
    if report.filters.is_empty() {
        report.matched_findings = report.findings.len();
        return;
    }
    let (visible, hidden) = std::mem::take(&mut report.findings)
        .into_iter()
        .partition(|finding| report.filters.matches(finding));
    report.findings = visible;
    report.filtered_out_findings = hidden;
    report.matched_findings = report.findings.len();
}

fn add_metadata_finding(
    findings: &mut Vec<ImpactFinding>,
    changes: &[super::model::ControlChange],
    old: &LoadedResource,
    new: &LoadedResource,
) -> Result<(), ForgeError> {
    if old.evidence.raw_sha256 == new.evidence.raw_sha256
        || changes.iter().any(|change| change.change_class != ChangeClass::Unchanged)
    {
        return Ok(());
    }
    let metadata_change = super::model::ControlChange {
        subject_id: "$resource".to_string(),
        change_class: ChangeClass::Unchanged,
        old_sha256: None,
        new_sha256: None,
        old_subjects: Vec::new(),
        new_subjects: Vec::new(),
        migration: None,
    };
    push_finding(
        findings,
        finding(
            old,
            new,
            &metadata_change,
            "$resource",
            FindingPriority::Informational,
            ReasonCode::ResourceMetadataChanged,
            RequiredAction::ReviewResourceMetadata,
            FindingContext {
                identity: "catalog-metadata".to_string(),
                dependency_path: vec!["catalog-metadata".to_string()],
                affected_artifact_id: None,
                dependency_id: None,
                policy_resource_identity: None,
                prior_gap_classification: None,
                prior_decision_state: None,
                owner: None,
                policy_sources: Vec::new(),
            },
        ),
    )
}

fn apply_dispositions(
    prior_report_path: &Path,
    disposition_path: &Path,
    report: &mut ImpactReport,
    input_paths: &mut Vec<PathBuf>,
) -> Result<(), ForgeError> {
    crate::io::regular_file_metadata(prior_report_path, "$.prior_report").map_err(impact_error)?;
    io::check_file_size(prior_report_path, super::disposition::MAX_PRIOR_REPORT_BYTES)
        .map_err(|error| impact_error(format!("$.prior_report: {error}")))?;
    let prior_bytes = std::fs::read(prior_report_path)
        .map_err(|error| impact_error(format!("$.prior_report: {error}")))?;
    let dispositions = super::disposition::load(disposition_path)?;
    if sha256(&prior_bytes) != dispositions.prior_report_sha256 {
        return Err(impact_error(
            "$.disposition_file prior_report_sha256 does not match $.prior_report",
        ));
    }
    let prior = super::disposition::parse_strict_value(&prior_bytes, "$.prior_report")?;
    validate_prior_report(&prior, report)?;
    let prior_finding_ids = prior["findings"]
        .as_array()
        .ok_or_else(|| impact_error("$.prior_report.findings must be an array"))?
        .iter()
        .map(|finding| {
            finding["finding_id"]
                .as_str()
                .ok_or_else(|| impact_error("$.prior_report contains a finding without an ID"))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    if prior_finding_ids.len() != prior["findings"].as_array().map_or(0, Vec::len) {
        return Err(impact_error("$.prior_report contains duplicate finding IDs"));
    }
    let current_finding_indexes: BTreeMap<_, _> = report
        .findings
        .iter()
        .enumerate()
        .map(|(index, finding)| (finding.finding_id.clone(), index))
        .collect();
    for disposition in dispositions.dispositions {
        if !prior_finding_ids.contains(disposition.finding_id.as_str()) {
            return Err(impact_error(format!(
                "$.disposition_file references finding '{}' absent from $.prior_report",
                bounded(&disposition.finding_id)
            )));
        }
        if let Some(index) = current_finding_indexes.get(&disposition.finding_id) {
            report.findings[*index].disposition = Some(disposition);
        } else {
            report.prior_only_dispositions.push(disposition);
        }
    }
    report.prior_only_dispositions.sort_by(|left, right| left.finding_id.cmp(&right.finding_id));
    input_paths.push(prior_report_path.to_path_buf());
    input_paths.push(disposition_path.to_path_buf());
    Ok(())
}

fn validate_prior_report(prior: &Value, report: &ImpactReport) -> Result<(), ForgeError> {
    if prior["schema_version"] != REPORT_SCHEMA_VERSION || prior["status"] != "complete" {
        return Err(impact_error(
            "$.prior_report must be a complete forge.framework-impact-report/1 report",
        ));
    }
    let old = serde_json::to_value(&report.old)
        .map_err(|error| impact_error(format!("old evidence serialization failed: {error}")))?;
    let new = serde_json::to_value(&report.new)
        .map_err(|error| impact_error(format!("new evidence serialization failed: {error}")))?;
    if prior["old"] != old || prior["new"] != new {
        return Err(impact_error(
            "$.prior_report old/new resource evidence does not match the current analysis",
        ));
    }
    Ok(())
}

fn update_disposition_summary(report: &mut ImpactReport) {
    report.summary.dispositioned_resolved = 0;
    report.summary.dispositioned_accepted_risk = 0;
    report.summary.dispositioned_still_open = 0;
    report.summary.undispositioned = 0;
    for finding in &report.findings {
        match finding.disposition.as_ref().map(|value| value.status) {
            Some(super::disposition::DispositionStatus::Resolved) => {
                report.summary.dispositioned_resolved += 1;
            }
            Some(super::disposition::DispositionStatus::AcceptedRisk) => {
                report.summary.dispositioned_accepted_risk += 1;
            }
            Some(super::disposition::DispositionStatus::StillOpen) => {
                report.summary.dispositioned_still_open += 1;
            }
            None => report.summary.undispositioned += 1,
        }
    }
}

fn load_framework_resource(
    manifest_dir: &Path,
    path_label: &str,
    resource: &FrameworkResource,
) -> Result<LoadedResource, ForgeError> {
    let path = manifest_dir.join(&resource.artifact);
    crate::io::regular_file_metadata(&path, path_label).map_err(impact_error)?;
    if let Some(companion) = &resource.resolved_catalog {
        crate::io::regular_file_metadata(
            &manifest_dir.join(companion),
            &format!("{path_label}.resolved_catalog"),
        )
        .map_err(impact_error)?;
    }
    let descriptor = ResourceManifest {
        resource_type: resource.resource_type,
        artifact: resource.artifact.clone(),
        href: path
            .file_name()
            .map_or_else(|| "catalog.json".to_string(), |name| name.to_string_lossy().into_owned()),
        resolved_catalog: resource.resolved_catalog.clone(),
        resolved_catalog_attestation: resource.resolved_catalog_attestation,
        expected_sha256: Some(resource.expected_sha256.clone()),
        expected_resolved_catalog_sha256: resource.expected_resolved_catalog_sha256.clone(),
        inventory: None,
    };
    let loaded = crate::mapping::inventory::load(manifest_dir, path_label, &descriptor)
        .map_err(|error| impact_error(strip_error_prefix(&error.to_string())))?;
    if loaded.evidence.resolved_catalog_sha256 != resource.expected_resolved_catalog_sha256 {
        return Err(impact_error(format!(
            "{path_label}.expected_resolved_catalog_sha256 does not match the supplied Profile companion"
        )));
    }
    for (field, declared, actual) in [
        ("root_uuid", resource.root_uuid.as_str(), loaded.evidence.root_uuid.as_str()),
        (
            "document_version",
            resource.document_version.as_str(),
            loaded.evidence.document_version.as_str(),
        ),
        ("oscal_version", resource.oscal_version.as_str(), loaded.evidence.oscal_version.as_str()),
    ] {
        if declared != actual {
            return Err(impact_error(format!(
                "{path_label}.{field} mismatch: expected '{}', got '{}'",
                bounded(declared),
                bounded(actual)
            )));
        }
    }
    Ok(loaded)
}

fn load_successor_map(
    path: &Path,
    input_paths: &mut Vec<PathBuf>,
) -> Result<crate::migration::successor::SuccessorMap, ForgeError> {
    let successor_map = crate::migration::successor::load(path)
        .map_err(|error| impact_error(strip_migration_error_prefix(&error.to_string())))?;
    input_paths.push(path.to_path_buf());
    Ok(successor_map)
}

fn classify(
    old: &Inventory,
    new: &Inventory,
    successor_map: Option<&crate::migration::successor::SuccessorMap>,
) -> Result<Vec<super::model::ControlChange>, ForgeError> {
    let old_ids = old.ids_of_type(SubjectType::Control);
    let new_ids = new.ids_of_type(SubjectType::Control);
    let stable_ids: BTreeSet<_> = old_ids.intersection(&new_ids).cloned().collect();
    let mut migrated_old = BTreeSet::new();
    let mut migrated_new = BTreeSet::new();
    let mut changes = Vec::new();
    if let Some(successor_map) = successor_map {
        for relationship in &successor_map.relationships {
            validate_migration_ids(
                relationship,
                &old_ids,
                &new_ids,
                &stable_ids,
                &mut migrated_old,
                &mut migrated_new,
            )?;
            let old_subjects = subject_fingerprints(old, &relationship.old_ids);
            let new_subjects = subject_fingerprints(new, &relationship.new_ids);
            changes.push(super::model::ControlChange {
                subject_id: format!(
                    "{}=>{}",
                    relationship.old_ids.join(","),
                    relationship.new_ids.join(",")
                ),
                change_class: ChangeClass::IdentityMigrated,
                old_sha256: single_sha256(&old_subjects),
                new_sha256: single_sha256(&new_subjects),
                old_subjects,
                new_subjects,
                migration: Some(IdentityMigrationEvidence {
                    relationship: relationship.relationship,
                    approved_by: relationship.approved_by.clone(),
                    approved_at: relationship.approved_at.clone(),
                    rationale: relationship.rationale.clone(),
                }),
            });
        }
    }
    changes.extend(
        old_ids
            .union(&new_ids)
            .filter(|id| !migrated_old.contains(*id) && !migrated_new.contains(*id))
            .map(|id| {
                let old_hash = old.fingerprint(SubjectType::Control, id).map(str::to_string);
                let new_hash = new.fingerprint(SubjectType::Control, id).map(str::to_string);
                let change_class = match (&old_hash, &new_hash) {
                    (None, Some(_)) => ChangeClass::Added,
                    (Some(_), None) => ChangeClass::Removed,
                    (Some(old), Some(new)) if old != new => ChangeClass::ContentChanged,
                    (Some(_), Some(_)) => ChangeClass::Unchanged,
                    (None, None) => unreachable!("union IDs must occur in an inventory"),
                };
                super::model::ControlChange {
                    subject_id: id.clone(),
                    change_class,
                    old_sha256: old_hash.clone(),
                    new_sha256: new_hash.clone(),
                    old_subjects: old_hash
                        .map(|sha256| vec![SubjectFingerprint { id: id.clone(), sha256 }])
                        .unwrap_or_default(),
                    new_subjects: new_hash
                        .map(|sha256| vec![SubjectFingerprint { id: id.clone(), sha256 }])
                        .unwrap_or_default(),
                    migration: None,
                }
            }),
    );
    changes.sort_by(|left, right| left.subject_id.cmp(&right.subject_id));
    Ok(changes)
}

fn validate_migration_ids(
    relationship: &crate::migration::successor::SuccessorRelationship,
    old_ids: &BTreeSet<String>,
    new_ids: &BTreeSet<String>,
    stable_ids: &BTreeSet<String>,
    migrated_old: &mut BTreeSet<String>,
    migrated_new: &mut BTreeSet<String>,
) -> Result<(), ForgeError> {
    for id in &relationship.old_ids {
        if !old_ids.contains(id) {
            return Err(impact_error(format!(
                "$.successor_map references old control '{}' absent from the old framework",
                bounded(id)
            )));
        }
        if stable_ids.contains(id) || !migrated_old.insert(id.clone()) {
            return Err(impact_error(format!(
                "$.successor_map old control '{}' is already reconciled",
                bounded(id)
            )));
        }
    }
    for id in &relationship.new_ids {
        if !new_ids.contains(id) {
            return Err(impact_error(format!(
                "$.successor_map references new control '{}' absent from the new framework",
                bounded(id)
            )));
        }
        if stable_ids.contains(id) || !migrated_new.insert(id.clone()) {
            return Err(impact_error(format!(
                "$.successor_map new control '{}' is already reconciled",
                bounded(id)
            )));
        }
    }
    Ok(())
}

fn subject_fingerprints(inventory: &Inventory, ids: &[String]) -> Vec<SubjectFingerprint> {
    ids.iter()
        .filter_map(|id| {
            inventory
                .fingerprint(SubjectType::Control, id)
                .map(|sha256| SubjectFingerprint { id: id.clone(), sha256: sha256.to_string() })
        })
        .collect()
}

fn single_sha256(subjects: &[SubjectFingerprint]) -> Option<String> {
    (subjects.len() == 1).then(|| subjects[0].sha256.clone())
}

fn load_applicability(
    path: &Path,
    old: &LoadedResource,
    portfolio: &MappingPortfolio,
    input_paths: &mut Vec<PathBuf>,
) -> Result<crate::applicability::PreparedAnalysis, ForgeError> {
    crate::io::regular_file_metadata(path, "$.applicability_manifest").map_err(impact_error)?;
    let prepared = crate::applicability::prepare_analysis(
        path,
        crate::applicability::model::ReportFilters::default(),
    )
    .map_err(|error| impact_error(strip_applicability_error_prefix(&error.to_string())))?;
    if !same_resource_identity(&prepared.report.framework, &old.evidence) {
        return Err(impact_error(
            "$.applicability_manifest references a different old framework baseline",
        ));
    }
    let applicability_mapping_hashes: BTreeSet<_> = prepared
        .report
        .mapping_collections
        .iter()
        .map(|collection| collection.raw_sha256.clone())
        .collect();
    if applicability_mapping_hashes != portfolio.target_collection_sha256s {
        return Err(impact_error(
            "$.applicability_manifest Mapping Collection portfolio does not exactly match framework-role target inputs",
        ));
    }
    if prepared.report.controls.len() != old.inventory.count(SubjectType::Control)
        || prepared.report.matched_controls != prepared.report.counts.total
    {
        return Err(impact_error(
            "$.applicability_manifest did not produce a complete unfiltered old-baseline inventory",
        ));
    }
    for control in &prepared.report.controls {
        let facts = portfolio.applicability_facts.get(&control.control_id);
        let (positive_count, no_relationship_count, policy_sources) = facts.map_or_else(
            || (0, 0, BTreeSet::new()),
            |facts| {
                (facts.positive_count, facts.no_relationship_count, facts.policy_sources.clone())
            },
        );
        if control.positive_mapping_count != positive_count
            || control.no_relationship_count != no_relationship_count
            || control.policy_sources.iter().cloned().collect::<BTreeSet<_>>() != policy_sources
        {
            return Err(impact_error(format!(
                "$.applicability_manifest control '{}' conflicts with the supplied Mapping Collection portfolio",
                bounded(&control.control_id)
            )));
        }
    }
    for input in &prepared.input_paths {
        crate::io::regular_file_metadata(input, "$.applicability_manifest dependency")
            .map_err(impact_error)?;
    }
    input_paths.extend(prepared.input_paths.iter().cloned());
    Ok(prepared)
}

fn same_resource_identity(
    left: &crate::mapping::inventory::ResourceEvidence,
    right: &crate::mapping::inventory::ResourceEvidence,
) -> bool {
    left.resource_type == right.resource_type
        && left.raw_sha256 == right.raw_sha256
        && left.root_uuid == right.root_uuid
        && left.document_version == right.document_version
        && left.oscal_version == right.oscal_version
        && left.resolved_catalog_sha256 == right.resolved_catalog_sha256
}

fn load_mapping_references(
    manifest_dir: &Path,
    dependencies: &[MappingDependency],
    old: &LoadedResource,
    input_paths: &mut Vec<PathBuf>,
) -> Result<MappingPortfolio, ForgeError> {
    let mut portfolio = MappingPortfolio::default();
    let mut mapping_paths: Vec<PathBuf> = Vec::new();
    let mut collection_ids = BTreeSet::new();
    for (index, dependency) in dependencies.iter().enumerate() {
        let label = format!("$.mapping_collections[{index}]");
        let path = manifest_dir.join(&dependency.artifact);
        crate::io::regular_file_metadata(&path, &label).map_err(impact_error)?;
        for previous in &mapping_paths {
            if crate::mapping::paths_alias(&path, previous)
                .map_err(|error| impact_error(strip_error_prefix(&error.to_string())))?
            {
                return Err(impact_error(format!(
                    "{label}.artifact aliases another Mapping Collection input"
                )));
            }
        }
        io::check_file_size(&path, io::MAX_FILE_SIZE)
            .map_err(|error| impact_error(format!("{label}.artifact: {error}")))?;
        let bytes = std::fs::read(&path)
            .map_err(|error| impact_error(format!("{label}.artifact: {error}")))?;
        let raw_sha256 = sha256(&bytes);
        let value = parse_mapping_value(&bytes, &label)?;
        validate_mapping(&label, &value)?;
        let collection: MappingCollectionEnvelope = serde_json::from_value(value)
            .map_err(|error| impact_error(format!("{label}.artifact is unsupported: {error}")))?;
        if !collection_ids.insert(collection.mapping_collection.uuid) {
            return Err(impact_error(format!(
                "{label}.artifact duplicates Mapping Collection identity '{}'",
                collection.mapping_collection.uuid
            )));
        }
        inventory_mapping(
            &label,
            &collection,
            dependency.framework_role,
            old,
            &mut portfolio.references,
            &mut portfolio.applicability_facts,
        )?;
        if dependency.framework_role == FrameworkRole::Target {
            portfolio.target_collection_sha256s.insert(raw_sha256);
        }
        input_paths.push(path.clone());
        mapping_paths.push(path);
    }
    for entries in portfolio.references.values_mut() {
        entries.sort_by(|left, right| {
            (
                left.artifact_id.as_str(),
                left.mapping_id.as_str(),
                left.map_id.as_str(),
                left.policy_resource_identity.as_str(),
            )
                .cmp(&(
                    right.artifact_id.as_str(),
                    right.mapping_id.as_str(),
                    right.map_id.as_str(),
                    right.policy_resource_identity.as_str(),
                ))
        });
    }
    Ok(portfolio)
}

fn parse_mapping_value(bytes: &[u8], label: &str) -> Result<Value, ForgeError> {
    crate::json_strict::parse_value(bytes, &format!("{label}.artifact"), MAPPING_JSON_LIMITS)
        .map_err(impact_error)
}

fn validate_mapping(label: &str, value: &Value) -> Result<(), ForgeError> {
    let detected = validate::detect_model_type(value)
        .map_err(|error| impact_error(format!("{label}.artifact: {error}")))?;
    if detected != crate::OscalModelType::Mapping {
        return Err(impact_error(format!("{label}.artifact must be an OSCAL Control Mapping")));
    }
    let validator = validate::compiled_validator(crate::OscalModelType::Mapping)
        .map_err(|error| impact_error(format!("Mapping schema compilation failed: {error}")))?;
    let errors: Vec<_> =
        validator.iter_errors(value).take(11).map(|error| error.to_string()).collect();
    let version = validate::version::inspect_oscal_version(value, crate::OscalModelType::Mapping);
    if !errors.is_empty() || version.error.is_some() {
        let mut details = errors.into_iter().take(10).collect::<Vec<_>>();
        if let Some(error) = version.error {
            details.push(error.message);
        }
        return Err(impact_error(format!(
            "{label}.artifact is not a valid Mapping: {}",
            details.join("; ")
        )));
    }
    Ok(())
}

fn inventory_mapping(
    label: &str,
    collection: &MappingCollectionEnvelope,
    role: FrameworkRole,
    old: &LoadedResource,
    references: &mut BTreeMap<String, Vec<MappingReference>>,
    applicability_facts: &mut BTreeMap<String, crate::applicability::model::ControlMappingFacts>,
) -> Result<(), ForgeError> {
    require_forge_prop(&collection.mapping_collection.metadata.props, "collection-key", label)?;
    let mut mapping_ids = BTreeSet::new();
    let mut map_ids = BTreeSet::new();
    for (mapping_index, mapping) in collection.mapping_collection.mappings.iter().enumerate() {
        let mapping_path = format!("{label}.mappings[{mapping_index}]");
        if !mapping_ids.insert(mapping.uuid) {
            return Err(impact_error(format!(
                "{mapping_path} duplicates mapping identity '{}'",
                mapping.uuid
            )));
        }
        require_forge_prop(&mapping.props, "mapping-key", &mapping_path)?;
        let (framework, policy, framework_items): (_, _, fn(&OscalMap) -> &Vec<MappingItem>) =
            match role {
                FrameworkRole::Source => {
                    (&mapping.source_resource, &mapping.target_resource, |map: &OscalMap| {
                        &map.sources
                    })
                }
                FrameworkRole::Target => {
                    (&mapping.target_resource, &mapping.source_resource, |map: &OscalMap| {
                        &map.targets
                    })
                }
            };
        crate::applicability::manifest::validate_report_href(
            &format!("{mapping_path}.policy-resource.href"),
            &policy.href,
        )
        .map_err(|error| impact_error(strip_applicability_error_prefix(&error.to_string())))?;
        verify_old_resource(&mapping_path, framework, old)?;
        let policy_identity = require_forge_prop(&policy.props, "root-uuid", &mapping_path)?;
        for (map_index, map) in mapping.maps.iter().enumerate() {
            let map_path = format!("{mapping_path}.maps[{map_index}]");
            if !map_ids.insert(map.uuid) {
                return Err(impact_error(format!(
                    "{map_path} duplicates map identity '{}'",
                    map.uuid
                )));
            }
            require_forge_prop(&map.props, "map-key", &map_path)?;
            let mut subjects = BTreeSet::new();
            for (item_index, item) in framework_items(map).iter().enumerate() {
                let item_path = format!("{map_path}.items[{item_index}]");
                if !subjects.insert((item.subject_type, item.id_ref.as_str())) {
                    return Err(impact_error(format!(
                        "{item_path} duplicates {} '{}'",
                        item.subject_type.as_str(),
                        bounded(&item.id_ref)
                    )));
                }
                let recorded = require_forge_prop(&item.props, "subject-sha256", &item_path)?;
                let actual = old
                    .inventory
                    .fingerprint(item.subject_type, &item.id_ref)
                    .ok_or_else(|| {
                        impact_error(format!(
                            "{item_path} references {} '{}' absent from the declared old baseline",
                            item.subject_type.as_str(),
                            bounded(&item.id_ref)
                        ))
                    })?;
                if recorded != actual {
                    return Err(impact_error(format!(
                        "{item_path} subject fingerprint does not match the declared old baseline"
                    )));
                }
                if item.subject_type == SubjectType::Control {
                    references.entry(item.id_ref.clone()).or_default().push(MappingReference {
                        artifact_id: collection.mapping_collection.uuid.to_string(),
                        mapping_id: mapping.uuid.to_string(),
                        map_id: map.uuid.to_string(),
                        policy_resource_identity: policy_identity.clone(),
                        policy_resource_label: policy.href.clone(),
                    });
                    if role == FrameworkRole::Target {
                        let facts = applicability_facts.entry(item.id_ref.clone()).or_default();
                        if map.relationship == Relationship::NoRelationship {
                            facts.no_relationship_count += 1;
                        } else {
                            facts.positive_count += 1;
                        }
                        facts.policy_sources.insert(policy.href.clone());
                    }
                }
            }
        }
    }
    Ok(())
}

fn verify_old_resource(
    path: &str,
    reference: &crate::mapping::model::MappingResourceReference,
    old: &LoadedResource,
) -> Result<(), ForgeError> {
    if reference.resource_type != old.evidence.resource_type {
        return Err(impact_error(format!(
            "{path} framework resource type does not match the declared old baseline"
        )));
    }
    for (name, expected) in [
        ("raw-sha256", old.evidence.raw_sha256.as_str()),
        ("root-uuid", old.evidence.root_uuid.as_str()),
        ("document-version", old.evidence.document_version.as_str()),
        ("oscal-version", old.evidence.oscal_version.as_str()),
    ] {
        let actual = require_forge_prop(&reference.props, name, path)?;
        if actual != expected {
            return Err(impact_error(format!(
                "{path} references a different old baseline ({name} mismatch)"
            )));
        }
    }
    let resolved = reference
        .props
        .iter()
        .filter(|prop| {
            prop.name == "resolved-catalog-sha256"
                && prop.ns.as_deref() == Some(crate::mapping::inventory::FORGE_MAPPING_NS)
        })
        .map(|prop| prop.value.as_str())
        .collect::<Vec<_>>();
    match old.evidence.resolved_catalog_sha256.as_deref() {
        Some(expected) if resolved.as_slice() == [expected] => {}
        None if resolved.is_empty() => {}
        _ => {
            return Err(impact_error(format!(
                "{path} resolved Catalog fingerprint does not match the declared old baseline"
            )));
        }
    }
    Ok(())
}

fn require_forge_prop(props: &[OscalProp], name: &str, path: &str) -> Result<String, ForgeError> {
    let mut matches = props.iter().filter(|prop| {
        prop.name == name && prop.ns.as_deref() == Some(crate::mapping::inventory::FORGE_MAPPING_NS)
    });
    let value = matches
        .next()
        .ok_or_else(|| impact_error(format!("{path} lacks required FORGE property '{name}'")))?;
    if matches.next().is_some() {
        return Err(impact_error(format!(
            "{path} contains ambiguous duplicate FORGE property '{name}'"
        )));
    }
    Ok(value.value.clone())
}

fn build_findings(
    changes: &[super::model::ControlChange],
    references: &BTreeMap<String, Vec<MappingReference>>,
    applicability: Option<&crate::applicability::PreparedAnalysis>,
    old: &LoadedResource,
    new: &LoadedResource,
) -> Result<Vec<ImpactFinding>, ForgeError> {
    let mut findings = Vec::new();
    for change in changes {
        let base = match change.change_class {
            ChangeClass::Added => Some((
                FindingPriority::ReviewRequired,
                ReasonCode::ControlAdded,
                RequiredAction::ReviewApplicability,
            )),
            ChangeClass::Removed => Some((
                FindingPriority::Informational,
                ReasonCode::ControlRemoved,
                RequiredAction::ReviewFrameworkRemoval,
            )),
            ChangeClass::ContentChanged => Some((
                FindingPriority::Informational,
                ReasonCode::ControlContentChanged,
                RequiredAction::ReviewControlChange,
            )),
            ChangeClass::IdentityMigrated => Some((
                FindingPriority::ReviewRequired,
                ReasonCode::IdentityMigrationDeclared,
                RequiredAction::ReviewIdentityMigration,
            )),
            ChangeClass::Unchanged => None,
        };
        if let Some((priority, reason, action)) = base {
            let context = if change.change_class == ChangeClass::IdentityMigrated {
                FindingContext {
                    identity: migration_identity(change),
                    dependency_path: vec![format!("identity-migration:{}", change.subject_id)],
                    affected_artifact_id: None,
                    dependency_id: Some(format!("migration:{}", change.subject_id)),
                    policy_resource_identity: None,
                    prior_gap_classification: None,
                    prior_decision_state: None,
                    owner: change.migration.as_ref().map(|migration| migration.approved_by.clone()),
                    policy_sources: Vec::new(),
                }
            } else {
                FindingContext::subject(&change.subject_id)
            };
            push_finding(
                &mut findings,
                finding(old, new, change, &change.subject_id, priority, reason, action, context),
            )?;
        }
        for subject in &change.old_subjects {
            add_mapping_findings(
                &mut findings,
                change,
                &subject.id,
                references.get(&subject.id).map_or(&[], Vec::as_slice),
                old,
                new,
            )?;
            add_applicability_finding(&mut findings, change, &subject.id, applicability, old, new)?;
        }
    }
    Ok(findings)
}

fn add_mapping_findings(
    findings: &mut Vec<ImpactFinding>,
    change: &super::model::ControlChange,
    subject_id: &str,
    dependencies: &[MappingReference],
    old: &LoadedResource,
    new: &LoadedResource,
) -> Result<(), ForgeError> {
    let classification = match change.change_class {
        ChangeClass::Removed => Some((
            FindingPriority::Blocking,
            ReasonCode::MappingReferenceRemoved,
            RequiredAction::RepairOrApproveMapping,
        )),
        ChangeClass::ContentChanged => Some((
            FindingPriority::ReviewRequired,
            ReasonCode::MappingSubjectChanged,
            RequiredAction::ReapproveMappingRationale,
        )),
        ChangeClass::IdentityMigrated => Some((
            FindingPriority::ReviewRequired,
            ReasonCode::MappingSubjectMigrated,
            RequiredAction::ReapproveMappingRationale,
        )),
        ChangeClass::Added | ChangeClass::Unchanged => None,
    };
    let Some((priority, reason, action)) = classification else { return Ok(()) };
    for dependency in dependencies {
        push_finding(
            findings,
            finding(
                old,
                new,
                change,
                subject_id,
                priority,
                reason,
                action,
                FindingContext {
                    identity: dependency_identity(
                        change,
                        format!(
                            "{}:{}:{}:{}",
                            dependency.artifact_id,
                            dependency.mapping_id,
                            dependency.map_id,
                            dependency.policy_resource_identity
                        ),
                    ),
                    dependency_path: vec![
                        format!("control:{subject_id}"),
                        format!("mapping-collection:{}", dependency.artifact_id),
                        format!("mapping:{}", dependency.mapping_id),
                        format!("map:{}", dependency.map_id),
                        format!("policy-resource:{}", dependency.policy_resource_identity),
                    ],
                    affected_artifact_id: Some(dependency.artifact_id.clone()),
                    dependency_id: Some(dependency.map_id.clone()),
                    policy_resource_identity: Some(dependency.policy_resource_identity.clone()),
                    prior_gap_classification: None,
                    prior_decision_state: None,
                    owner: None,
                    policy_sources: vec![dependency.policy_resource_label.clone()],
                },
            ),
        )?;
    }
    Ok(())
}

fn add_applicability_finding(
    findings: &mut Vec<ImpactFinding>,
    change: &super::model::ControlChange,
    subject_id: &str,
    applicability: Option<&crate::applicability::PreparedAnalysis>,
    old: &LoadedResource,
    new: &LoadedResource,
) -> Result<(), ForgeError> {
    let Some(prepared) = applicability else { return Ok(()) };
    if !matches!(
        change.change_class,
        ChangeClass::Removed | ChangeClass::ContentChanged | ChangeClass::IdentityMigrated
    ) {
        return Ok(());
    }
    let control = prepared
        .report
        .controls
        .iter()
        .find(|control| control.control_id == subject_id)
        .ok_or_else(|| {
            impact_error(format!(
                "applicability analysis omitted old-baseline control '{}'",
                bounded(subject_id)
            ))
        })?;
    let reason = match change.change_class {
        ChangeClass::Removed => ReasonCode::ApplicabilityDecisionRemoved,
        ChangeClass::ContentChanged => ReasonCode::ApplicabilityDecisionChanged,
        ChangeClass::IdentityMigrated => ReasonCode::ApplicabilityDecisionMigrated,
        ChangeClass::Added | ChangeClass::Unchanged => {
            unreachable!("guard restricts applicability impact classes")
        }
    };
    let classification = control.classification.as_str().to_string();
    let decision_state = prepared
        .manifest
        .decisions
        .iter()
        .find(|decision| decision.control_id == subject_id)
        .map_or(crate::applicability::manifest::DecisionState::UnderReview, |decision| {
            decision.state
        });
    let dependency_id = format!("applicability:{subject_id}");
    push_finding(
        findings,
        finding(
            old,
            new,
            change,
            subject_id,
            FindingPriority::ReviewRequired,
            reason,
            RequiredAction::ReviewApplicabilityDecision,
            FindingContext {
                identity: dependency_identity(change, format!("{dependency_id}:{classification}")),
                dependency_path: vec![
                    format!("control:{subject_id}"),
                    format!("applicability-manifest:{}", prepared.report.manifest_sha256),
                    dependency_id.clone(),
                    format!("gap-state:{classification}"),
                ],
                affected_artifact_id: Some(prepared.report.manifest_sha256.clone()),
                dependency_id: Some(dependency_id),
                policy_resource_identity: None,
                prior_gap_classification: Some(classification),
                prior_decision_state: Some(decision_state),
                owner: control.reviewer_key.clone(),
                policy_sources: control.policy_sources.clone(),
            },
        ),
    )
}

fn push_finding(
    findings: &mut Vec<ImpactFinding>,
    finding: ImpactFinding,
) -> Result<(), ForgeError> {
    ensure_finding_slot(findings.len())?;
    findings.push(finding);
    Ok(())
}

fn ensure_finding_slot(current: usize) -> Result<(), ForgeError> {
    if current >= MAX_FINDINGS {
        return Err(impact_error(format!("analysis exceeds the {MAX_FINDINGS} finding limit")));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn finding(
    old: &LoadedResource,
    new: &LoadedResource,
    change: &super::model::ControlChange,
    subject_id: &str,
    priority: FindingPriority,
    reason_code: ReasonCode,
    required_action: RequiredAction,
    context: FindingContext,
) -> ImpactFinding {
    let seed = format!(
        "{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}\0{}",
        REPORT_SCHEMA_VERSION,
        old.evidence.raw_sha256,
        old.evidence.resolved_catalog_sha256.as_deref().unwrap_or(""),
        new.evidence.raw_sha256,
        new.evidence.resolved_catalog_sha256.as_deref().unwrap_or(""),
        subject_id,
        change.change_class.as_str(),
        reason_code.as_str(),
        context.identity
    );
    ImpactFinding {
        finding_id: Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, seed.as_bytes()).to_string(),
        priority,
        reason_code,
        required_action,
        subject_id: subject_id.to_string(),
        change_class: change.change_class,
        old_sha256: change.old_sha256.clone(),
        new_sha256: change.new_sha256.clone(),
        old_subjects: change.old_subjects.clone(),
        new_subjects: change.new_subjects.clone(),
        migration: change.migration.clone(),
        dependency_path: context.dependency_path,
        framework_groups: Vec::new(),
        affected_artifact_id: context.affected_artifact_id,
        dependency_id: context.dependency_id,
        policy_resource_identity: context.policy_resource_identity,
        prior_gap_classification: context.prior_gap_classification,
        prior_decision_state: context.prior_decision_state,
        owner: context.owner,
        policy_sources: context.policy_sources,
        disposition: None,
    }
}

fn migration_identity(change: &super::model::ControlChange) -> String {
    let Some(migration) = &change.migration else { return "not-migrated".to_string() };
    sha256(
        format!(
            "{}\0{}\0{}\0{}\0{}",
            change.subject_id,
            migration.relationship.as_str(),
            migration.approved_by,
            migration.approved_at,
            migration.rationale
        )
        .as_bytes(),
    )
}

fn dependency_identity(change: &super::model::ControlChange, base: String) -> String {
    if change.change_class == ChangeClass::IdentityMigrated {
        format!("{base}:{}", migration_identity(change))
    } else {
        base
    }
}

fn summarize(changes: &[super::model::ControlChange], findings: &[ImpactFinding]) -> ChangeSummary {
    let mut summary = ChangeSummary::default();
    for change in changes {
        match change.change_class {
            ChangeClass::Added => summary.added += 1,
            ChangeClass::Removed => summary.removed += 1,
            ChangeClass::ContentChanged => summary.content_changed += 1,
            ChangeClass::IdentityMigrated => summary.identity_migrated += 1,
            ChangeClass::Unchanged => summary.unchanged += 1,
        }
    }
    summary.findings = findings.len();
    for finding in findings {
        match finding.priority {
            FindingPriority::Blocking => summary.blocking += 1,
            FindingPriority::ReviewRequired => summary.review_required += 1,
            FindingPriority::Informational => summary.informational += 1,
        }
    }
    summary
}

fn strip_error_prefix(message: &str) -> String {
    message.strip_prefix("Control Mapping build error: ").unwrap_or(message).to_string()
}

fn strip_applicability_error_prefix(message: &str) -> String {
    message.strip_prefix("Applicability analysis error: ").unwrap_or(message).to_string()
}

fn strip_migration_error_prefix(message: &str) -> String {
    message.strip_prefix("Migration analysis error: ").unwrap_or(message).to_string()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn impact_error(message: impl Into<String>) -> ForgeError {
    ForgeError::FrameworkImpact(message.into())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{MAX_FINDINGS, classify, ensure_finding_slot, parse_mapping_value};
    use crate::mapping::inventory::Inventory;
    use crate::mapping::manifest::{ResourceManifest, ResourceType, SubjectType};
    use crate::migration::successor::{RelationshipType, SuccessorMap, SuccessorRelationship};

    #[test]
    fn finding_limit_is_checked_before_the_next_allocation() {
        assert!(ensure_finding_slot(MAX_FINDINGS - 1).is_ok());
        let error = ensure_finding_slot(MAX_FINDINGS).unwrap_err();
        assert!(error.to_string().contains("100000 finding limit"));
    }

    #[test]
    fn mapping_json_is_duplicate_key_safe_and_bounded() {
        let duplicate = parse_mapping_value(
            br#"{"mapping-collection":{},"mapping-collection":{}}"#,
            "$.mapping_collections[0]",
        )
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate object key 'mapping-collection'"));

        let oversized = format!(r#"{{"value":"{}"}}"#, "x".repeat(64 * 1024 + 1));
        let bounded =
            parse_mapping_value(oversized.as_bytes(), "$.mapping_collections[0]").unwrap_err();
        assert!(bounded.to_string().contains("maximum string length 65536 bytes"));
    }

    #[test]
    fn stable_ids_cannot_be_redeclared_as_migration_endpoints() {
        let old = inventory(&[("stable", "same"), ("old-only", "old")]);
        let new = inventory(&[("stable", "same"), ("new-only", "new")]);

        let old_stable = successor_map(vec![relationship(
            RelationshipType::Successor,
            &["stable"],
            &["new-only"],
        )]);
        let error = classify(&old, &new, Some(&old_stable)).unwrap_err();
        assert!(error.to_string().contains("old control 'stable' is already reconciled"));

        let new_stable = successor_map(vec![relationship(
            RelationshipType::Successor,
            &["old-only"],
            &["stable"],
        )]);
        let error = classify(&old, &new, Some(&new_stable)).unwrap_err();
        assert!(error.to_string().contains("new control 'stable' is already reconciled"));
    }

    #[test]
    fn migration_endpoints_cannot_be_consumed_more_than_once() {
        let old = inventory(&[("old-a", "a"), ("old-b", "b")]);
        let new = inventory(&[("new-a", "a"), ("new-b", "b")]);

        let reused_old = successor_map(vec![
            relationship(RelationshipType::Successor, &["old-a"], &["new-a"]),
            relationship(RelationshipType::Successor, &["old-a"], &["new-b"]),
        ]);
        let error = classify(&old, &new, Some(&reused_old)).unwrap_err();
        assert!(error.to_string().contains("old control 'old-a' is already reconciled"));

        let reused_new = successor_map(vec![
            relationship(RelationshipType::Successor, &["old-a"], &["new-a"]),
            relationship(RelationshipType::Successor, &["old-b"], &["new-a"]),
        ]);
        let error = classify(&old, &new, Some(&reused_new)).unwrap_err();
        assert!(error.to_string().contains("new control 'new-a' is already reconciled"));
    }

    #[test]
    fn split_and_merge_hashes_and_classification_are_reconciled_exactly_once() {
        let old = inventory(&[
            ("stable", "same"),
            ("split-old", "split"),
            ("merge-old-a", "merge a"),
            ("merge-old-b", "merge b"),
            ("removed", "removed"),
        ]);
        let new = inventory(&[
            ("stable", "same"),
            ("split-new-a", "split a"),
            ("split-new-b", "split b"),
            ("merge-new", "merge"),
            ("added", "added"),
        ]);
        let migrations = successor_map(vec![
            relationship(RelationshipType::Split, &["split-old"], &["split-new-a", "split-new-b"]),
            relationship(RelationshipType::Merge, &["merge-old-a", "merge-old-b"], &["merge-new"]),
        ]);

        let changes = classify(&old, &new, Some(&migrations)).unwrap();
        let split = changes
            .iter()
            .find(|change| {
                change
                    .migration
                    .as_ref()
                    .is_some_and(|migration| migration.relationship == RelationshipType::Split)
            })
            .expect("split change");
        assert!(split.old_sha256.is_some());
        assert!(split.new_sha256.is_none());
        let merge = changes
            .iter()
            .find(|change| {
                change
                    .migration
                    .as_ref()
                    .is_some_and(|migration| migration.relationship == RelationshipType::Merge)
            })
            .expect("merge change");
        assert!(merge.old_sha256.is_none());
        assert!(merge.new_sha256.is_some());

        let mut old_occurrences = BTreeMap::new();
        let mut new_occurrences = BTreeMap::new();
        for change in &changes {
            for subject in &change.old_subjects {
                *old_occurrences.entry(subject.id.clone()).or_insert(0) += 1;
            }
            for subject in &change.new_subjects {
                *new_occurrences.entry(subject.id.clone()).or_insert(0) += 1;
            }
        }
        assert_eq!(
            old_occurrences,
            old.ids_of_type(SubjectType::Control).into_iter().map(|id| (id, 1)).collect()
        );
        assert_eq!(
            new_occurrences,
            new.ids_of_type(SubjectType::Control).into_iter().map(|id| (id, 1)).collect()
        );
    }

    fn inventory(controls: &[(&str, &str)]) -> Inventory {
        let directory = tempfile::tempdir().expect("temporary inventory directory");
        let artifact = directory.path().join("catalog.json");
        let catalog = json!({
            "catalog": {
                "uuid": "77777777-7777-4777-8777-777777777777",
                "metadata": {
                    "title": "Classification fixture",
                    "last-modified": "2026-08-26T12:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "groups": [{
                    "id": "fixture-group",
                    "title": "Fixture group",
                    "controls": controls.iter().map(|(id, prose)| json!({
                        "id": id,
                        "title": format!("Control {id}"),
                        "parts": [{
                            "id": format!("{id}_smt"),
                            "name": "statement",
                            "prose": prose
                        }]
                    })).collect::<Vec<_>>()
                }]
            }
        });
        std::fs::write(&artifact, serde_json::to_vec(&catalog).unwrap()).unwrap();
        crate::mapping::inventory::load(
            directory.path(),
            "$.fixture",
            &ResourceManifest {
                resource_type: ResourceType::Catalog,
                artifact: "catalog.json".into(),
                href: "catalog.json".to_string(),
                resolved_catalog: None,
                resolved_catalog_attestation: None,
                expected_sha256: None,
                expected_resolved_catalog_sha256: None,
                inventory: None,
            },
        )
        .unwrap()
        .inventory
    }

    fn successor_map(relationships: Vec<SuccessorRelationship>) -> SuccessorMap {
        SuccessorMap {
            schema_version: crate::migration::successor::SUCCESSOR_MAP_SCHEMA_VERSION.to_string(),
            relationships,
        }
    }

    fn relationship(
        relationship: RelationshipType,
        old_ids: &[&str],
        new_ids: &[&str],
    ) -> SuccessorRelationship {
        SuccessorRelationship {
            relationship,
            old_ids: old_ids.iter().map(|id| (*id).to_string()).collect(),
            new_ids: new_ids.iter().map(|id| (*id).to_string()).collect(),
            approved_by: "reviewer".to_string(),
            approved_at: "2026-08-26T12:00:00Z".to_string(),
            rationale: "Reviewed migration.".to_string(),
        }
    }
}
