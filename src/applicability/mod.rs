//! Human-reviewed framework applicability and policy-gap analysis.

pub mod manifest;
pub mod model;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli::{ApplicabilityFailOn, ApplicabilityReportFormat, ApplicabilityStateFilter};
use crate::mapping::inventory::{self, FORGE_MAPPING_NS, LoadedResource, ResourceEvidence};
use crate::mapping::manifest::{ResourceManifest, ResourceType, SubjectType};
use crate::mapping::model::{
    MappingCollectionEnvelope, MappingResourceReference, OscalProp, stable_uuid,
};
use crate::{ForgeError, OscalModelType, io, validate};

type SourceSubjectKey = (String, SubjectType, String);
type RelationshipKey = (String, SubjectType, String, SubjectType, String);

#[derive(Default)]
struct MappingValidationState {
    collection_uuids: BTreeSet<uuid::Uuid>,
    mapping_uuids: BTreeSet<uuid::Uuid>,
    map_uuids: BTreeSet<uuid::Uuid>,
    source_resources_by_href: BTreeMap<String, ResourceEvidence>,
    source_subject_fingerprints: BTreeMap<SourceSubjectKey, String>,
    relationship_polarities: BTreeMap<RelationshipKey, bool>,
}

/// Create an undecided, deterministic applicability manifest scaffold.
///
/// # Errors
///
/// Returns an error for invalid resources, a missing Profile companion, unsafe aliases, or a
/// serialization/write failure.
pub fn execute_init(
    framework_path: &Path,
    resolved_catalog: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let (framework, loaded) = scaffold_framework(framework_path, resolved_catalog, output)?;
    validate_applicability_group_ids(&loaded.inventory)?;
    let mut inputs = vec![framework_path.to_path_buf()];
    inputs.extend(resolved_catalog.map(Path::to_path_buf));
    validate_destination(&inputs, output)?;
    let scaffold = manifest::ApplicabilityManifest {
        schema_version: manifest::MANIFEST_SCHEMA_VERSION.to_string(),
        framework,
        reviewers: Vec::new(),
        decisions: Vec::new(),
        mapping_collections: Vec::new(),
    };
    let mut rendered = serde_json::to_string_pretty(&scaffold)
        .map_err(|cause| error(format!("manifest scaffold serialization failed: {cause}")))?;
    rendered.push('\n');
    crate::cli::output::write_output(&rendered, output)
}

/// Analyze a validated applicability manifest and emit a deterministic report.
///
/// # Errors
///
/// Returns an error before output is written for invalid manifests, resources, decisions,
/// Mapping Collections, stale references, resource mismatches, unsafe aliases, or serialization.
pub fn execute_analyze(
    manifest_path: &Path,
    format: &ApplicabilityReportFormat,
    output: Option<&Path>,
    fail_on: &ApplicabilityFailOn,
    as_of: Option<&str>,
    filters: model::ReportFilters,
) -> Result<bool, ForgeError> {
    validate_filters(&filters)?;
    let as_of = parse_gate_date(fail_on, as_of)?;
    io::check_file_size(manifest_path, manifest::MAX_MANIFEST_BYTES)
        .map_err(|cause| error(format!("manifest: {cause}")))?;
    let manifest_bytes = std::fs::read(manifest_path).map_err(|cause| {
        error(format!("cannot read manifest '{}': {cause}", manifest_path.display()))
    })?;
    let parsed = manifest::parse(&manifest_bytes)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let framework = inventory::load(manifest_dir, "$.framework", &parsed.framework)
        .map_err(relabel_mapping_error)?;
    validate_applicability_group_ids(&framework.inventory)?;
    validate_decision_references(&parsed, &framework)?;

    let mut mapping_facts = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut validation_state = MappingValidationState::default();
    let mut inputs = vec![manifest_path.to_path_buf(), framework.path.clone()];
    if let Some(companion) = &parsed.framework.resolved_catalog {
        inputs.push(manifest_dir.join(companion));
    }
    for (index, relative_path) in parsed.mapping_collections.iter().enumerate() {
        let path = manifest_dir.join(relative_path);
        inputs.push(path.clone());
        let loaded =
            load_mapping(&path, index, &framework, &mut mapping_facts, &mut validation_state)?;
        evidence.push(loaded);
    }
    evidence.sort_by(|left, right| left.uuid.cmp(&right.uuid));
    validate_destination(&inputs, output)?;

    let report = model::build_report(
        &parsed,
        sha256(&manifest_bytes),
        framework.evidence,
        &framework.inventory,
        evidence,
        &mapping_facts,
        filters,
    );
    validate_classification_counts(&report.counts)?;
    let rendered = render_report(&report, format)?;
    crate::cli::output::write_output(&rendered, output)?;
    Ok(review_required(&report, &parsed, fail_on, as_of))
}

/// Convert the CLI filter vocabulary into the report model vocabulary.
#[must_use]
pub const fn classification_filter(filter: &ApplicabilityStateFilter) -> model::GapClassification {
    match filter {
        ApplicabilityStateFilter::ApplicableMapped => model::GapClassification::ApplicableMapped,
        ApplicabilityStateFilter::ApplicableReviewedNoRelationship => {
            model::GapClassification::ApplicableReviewedNoRelationship
        }
        ApplicabilityStateFilter::ApplicableUnmapped => {
            model::GapClassification::ApplicableUnmapped
        }
        ApplicabilityStateFilter::NotApplicable => model::GapClassification::NotApplicable,
        ApplicabilityStateFilter::Deferred => model::GapClassification::Deferred,
        ApplicabilityStateFilter::UnderReview => model::GapClassification::UnderReview,
    }
}

fn validate_decision_references(
    parsed: &manifest::ApplicabilityManifest,
    framework: &LoadedResource,
) -> Result<(), ForgeError> {
    for (index, decision) in parsed.decisions.iter().enumerate() {
        if !framework.inventory.contains(SubjectType::Control, &decision.control_id) {
            let detail = if framework.inventory.type_for_id(&decision.control_id).is_some() {
                "resolves to another eligible subject type"
            } else {
                "does not resolve in the current framework inventory"
            };
            return Err(error(format!(
                "$.decisions[{index}].control_id '{}' {detail}",
                bounded(&decision.control_id)
            )));
        }
    }
    Ok(())
}

fn load_mapping(
    path: &Path,
    index: usize,
    framework: &LoadedResource,
    mapping_facts: &mut BTreeMap<String, model::ControlMappingFacts>,
    validation_state: &mut MappingValidationState,
) -> Result<model::MappingEvidence, ForgeError> {
    let label = format!("$.mapping_collections[{index}]");
    io::check_file_size(path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("{label}: {cause}")))?;
    let bytes =
        std::fs::read(path).map_err(|cause| error(format!("{label} cannot be read: {cause}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|cause| error(format!("{label} is not valid JSON: {cause}")))?;
    let detected =
        validate::detect_model_type(&value).map_err(|cause| error(format!("{label}: {cause}")))?;
    if detected != OscalModelType::Mapping {
        return Err(error(format!("{label} must contain an OSCAL Mapping Collection")));
    }
    inventory::validate_schema(&label, &value, OscalModelType::Mapping)
        .map_err(relabel_mapping_error)?;
    let collection: MappingCollectionEnvelope = serde_json::from_value(value)
        .map_err(|cause| error(format!("{label} structure is unsupported: {cause}")))?;
    if collection.mapping_collection.metadata.oscal_version != "1.2.3" {
        return Err(error(format!("{label} must declare OSCAL v1.2.3")));
    }
    let collection_key = require_single_prop(
        &format!("{label}.mapping-collection.metadata"),
        &collection.mapping_collection.metadata.props,
        "collection-key",
    )?;
    validate_stable_uuid(
        &format!("{label}.mapping-collection"),
        "collection",
        collection_key,
        collection.mapping_collection.uuid,
    )?;
    if !validation_state.collection_uuids.insert(collection.mapping_collection.uuid) {
        return Err(error(format!(
            "{label} duplicates Mapping Collection UUID '{}'",
            collection.mapping_collection.uuid
        )));
    }
    let reviewed_at = require_single_prop(
        &format!("{label}.mapping-collection.provenance"),
        &collection.mapping_collection.provenance.props,
        "reviewed-at",
    )?
    .to_string();
    chrono::DateTime::parse_from_rfc3339(&reviewed_at).map_err(|_| {
        error(format!(
            "{label}.mapping-collection.provenance FORGE 'reviewed-at' must be an RFC 3339 timestamp"
        ))
    })?;
    let (reviewers, party_uuids) = mapping_reviewer_evidence(&label, &collection)?;
    let mut source_resources = BTreeMap::new();
    for (mapping_index, mapping) in collection.mapping_collection.mappings.iter().enumerate() {
        let mapping_label = format!("{label}.mapping-collection.mappings[{mapping_index}]");
        let mapping_key = require_single_prop(&mapping_label, &mapping.props, "mapping-key")?;
        validate_stable_uuid(&mapping_label, "mapping", mapping_key, mapping.uuid)?;
        if !validation_state.mapping_uuids.insert(mapping.uuid) {
            return Err(error(format!(
                "{mapping_label} duplicates mapping UUID '{}'",
                mapping.uuid
            )));
        }
        manifest::validate_report_href(
            &format!("{mapping_label}.target-resource.href"),
            &mapping.target_resource.href,
        )?;
        let source_resource = source_resource_evidence(&mapping_label, &mapping.source_resource)?;
        record_source_resource(&mapping_label, &source_resource, validation_state)?;
        source_resources
            .entry(source_resource.href.clone())
            .or_insert_with(|| source_resource.clone());
        validate_framework_reference(&mapping_label, &mapping.target_resource, framework)?;
        validate_mapping_edges(
            &mapping_label,
            mapping,
            &source_resource,
            framework,
            &party_uuids,
            mapping_facts,
            validation_state,
        )?;
    }
    Ok(model::MappingEvidence {
        uuid: collection.mapping_collection.uuid.to_string(),
        raw_sha256: sha256(&bytes),
        version: collection.mapping_collection.metadata.version,
        oscal_version: collection.mapping_collection.metadata.oscal_version,
        reviewed_at,
        reviewers,
        source_resources: source_resources.into_values().collect(),
    })
}

fn mapping_reviewer_evidence(
    label: &str,
    collection: &MappingCollectionEnvelope,
) -> Result<(Vec<model::MappingReviewerEvidence>, BTreeSet<uuid::Uuid>), ForgeError> {
    let parties: BTreeMap<_, _> = collection
        .mapping_collection
        .metadata
        .parties
        .iter()
        .map(|party| (party.uuid, party))
        .collect();
    if parties.len() != collection.mapping_collection.metadata.parties.len() {
        return Err(error(format!(
            "{label}.mapping-collection.metadata.parties contains duplicate UUIDs"
        )));
    }
    let mut reviewer_uuids = BTreeSet::new();
    for responsible in &collection.mapping_collection.provenance.responsible_parties {
        if responsible.role_id == "mapping-reviewer" {
            reviewer_uuids.extend(responsible.party_uuids.iter().copied());
        }
    }
    if reviewer_uuids.is_empty() {
        return Err(error(format!(
            "{label}.mapping-collection.provenance must identify at least one mapping-reviewer"
        )));
    }
    let party_uuids = parties.keys().copied().collect();
    let reviewers = reviewer_uuids
        .into_iter()
        .map(|uuid| -> Result<model::MappingReviewerEvidence, ForgeError> {
            let party = parties.get(&uuid).ok_or_else(|| {
                error(format!(
                    "{label}.mapping-collection.provenance references unknown reviewer UUID '{uuid}'"
                ))
            })?;
            Ok(model::MappingReviewerEvidence {
                uuid: uuid.to_string(),
                reviewer_type: party.party_type,
                name: party.name.clone(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((reviewers, party_uuids))
}

fn validate_mapping_edges(
    mapping_label: &str,
    mapping: &crate::mapping::model::OscalMapping,
    source_resource: &ResourceEvidence,
    framework: &LoadedResource,
    party_uuids: &BTreeSet<uuid::Uuid>,
    mapping_facts: &mut BTreeMap<String, model::ControlMappingFacts>,
    validation_state: &mut MappingValidationState,
) -> Result<(), ForgeError> {
    for (map_index, edge) in mapping.maps.iter().enumerate() {
        let edge_label = format!("{mapping_label}.maps[{map_index}]");
        let map_key = require_single_prop(&edge_label, &edge.props, "map-key")?;
        validate_stable_uuid(&edge_label, "map", map_key, edge.uuid)?;
        if !validation_state.map_uuids.insert(edge.uuid) {
            return Err(error(format!("{edge_label} duplicates map UUID '{}'", edge.uuid)));
        }
        let reviewer_key = require_single_prop(&edge_label, &edge.props, "reviewer-key")?;
        if reviewer_key.trim().is_empty() {
            return Err(error(format!("{edge_label} FORGE 'reviewer-key' must not be empty")));
        }
        let reviewer_uuid = stable_uuid("party", reviewer_key);
        if !party_uuids.contains(&reviewer_uuid) {
            return Err(error(format!(
                "{edge_label} FORGE 'reviewer-key' references undeclared reviewer '{reviewer_key}'"
            )));
        }
        let reviewed_at = require_single_prop(&edge_label, &edge.props, "reviewed-at")?;
        chrono::DateTime::parse_from_rfc3339(reviewed_at).map_err(|_| {
            error(format!("{edge_label} FORGE 'reviewed-at' must be an RFC 3339 timestamp"))
        })?;
        if edge.remarks.trim().is_empty() {
            return Err(error(format!("{edge_label}.remarks must preserve review rationale")));
        }
        let source_subjects =
            validate_source_subjects(&edge_label, edge, source_resource, validation_state)?;
        let positive = edge.relationship != crate::mapping::manifest::Relationship::NoRelationship;
        let mut target_subjects = BTreeSet::new();
        for (target_index, target) in edge.targets.iter().enumerate() {
            let target_label = format!("{edge_label}.targets[{target_index}]");
            if !target_subjects.insert((target.subject_type, target.id_ref.as_str())) {
                return Err(error(format!(
                    "{target_label} duplicates {} '{}' within the map",
                    target.subject_type.as_str(),
                    bounded(&target.id_ref)
                )));
            }
            if !framework.inventory.contains(target.subject_type, &target.id_ref) {
                return Err(error(format!(
                    "{target_label} contains stale {} reference '{}'",
                    target.subject_type.as_str(),
                    bounded(&target.id_ref)
                )));
            }
            let expected_fingerprint = framework
                .inventory
                .fingerprint(target.subject_type, &target.id_ref)
                .expect("inventory membership checked above");
            let actual_fingerprint =
                require_single_prop(&target_label, &target.props, "subject-sha256")?;
            if actual_fingerprint != expected_fingerprint {
                return Err(error(format!(
                    "{target_label} FORGE 'subject-sha256' is stale for {} '{}'",
                    target.subject_type.as_str(),
                    bounded(&target.id_ref)
                )));
            }
            for (source_type, source_id) in &source_subjects {
                let relationship_key = (
                    source_resource.raw_sha256.clone(),
                    *source_type,
                    source_id.clone(),
                    target.subject_type,
                    target.id_ref.clone(),
                );
                if validation_state
                    .relationship_polarities
                    .get(&relationship_key)
                    .is_some_and(|existing| *existing != positive)
                {
                    return Err(error(format!(
                        "{target_label} contradicts another reviewed relationship between {} '{}' and {} '{}'",
                        source_type.as_str(),
                        bounded(source_id),
                        target.subject_type.as_str(),
                        bounded(&target.id_ref)
                    )));
                }
                validation_state.relationship_polarities.insert(relationship_key, positive);
            }
            if target.subject_type == SubjectType::Control {
                let facts = mapping_facts.entry(target.id_ref.clone()).or_default();
                if edge.relationship == crate::mapping::manifest::Relationship::NoRelationship {
                    facts.no_relationship_count += 1;
                } else {
                    facts.positive_count += 1;
                }
                facts.policy_sources.insert(mapping.source_resource.href.clone());
            }
        }
    }
    Ok(())
}

fn validate_source_subjects(
    edge_label: &str,
    edge: &crate::mapping::model::OscalMap,
    source_resource: &ResourceEvidence,
    validation_state: &mut MappingValidationState,
) -> Result<Vec<(SubjectType, String)>, ForgeError> {
    if edge.sources.is_empty() {
        return Err(error(format!("{edge_label}.sources must not be empty")));
    }
    let mut seen = BTreeSet::new();
    let mut subjects = Vec::with_capacity(edge.sources.len());
    for (source_index, source) in edge.sources.iter().enumerate() {
        let source_label = format!("{edge_label}.sources[{source_index}]");
        if !seen.insert((source.subject_type, source.id_ref.as_str())) {
            return Err(error(format!(
                "{source_label} duplicates {} '{}' within the map",
                source.subject_type.as_str(),
                bounded(&source.id_ref)
            )));
        }
        let fingerprint = require_single_prop(&source_label, &source.props, "subject-sha256")?;
        validate_sha256(&format!("{source_label} FORGE 'subject-sha256'"), fingerprint)?;
        let subject_key =
            (source_resource.raw_sha256.clone(), source.subject_type, source.id_ref.clone());
        if validation_state
            .source_subject_fingerprints
            .get(&subject_key)
            .is_some_and(|existing| existing != fingerprint)
        {
            return Err(error(format!(
                "{source_label} has a conflicting subject fingerprint for {} '{}'",
                source.subject_type.as_str(),
                bounded(&source.id_ref)
            )));
        }
        validation_state.source_subject_fingerprints.insert(subject_key, fingerprint.to_string());
        subjects.push((source.subject_type, source.id_ref.clone()));
    }
    Ok(subjects)
}

fn source_resource_evidence(
    path: &str,
    reference: &MappingResourceReference,
) -> Result<ResourceEvidence, ForgeError> {
    let resource_path = format!("{path}.source-resource");
    manifest::validate_report_href(&format!("{resource_path}.href"), &reference.href)?;
    let raw_sha256 = require_single_prop(&resource_path, &reference.props, "raw-sha256")?;
    validate_sha256(&format!("{resource_path} FORGE 'raw-sha256'"), raw_sha256)?;
    let root_uuid = require_single_prop(&resource_path, &reference.props, "root-uuid")?;
    uuid::Uuid::parse_str(root_uuid)
        .map_err(|_| error(format!("{resource_path} FORGE 'root-uuid' must be a valid UUID")))?;
    let document_version =
        require_single_prop(&resource_path, &reference.props, "document-version")?;
    if document_version.trim().is_empty() {
        return Err(error(format!("{resource_path} FORGE 'document-version' must not be empty")));
    }
    let oscal_version = require_single_prop(&resource_path, &reference.props, "oscal-version")?;
    if oscal_version != "1.2.3" {
        return Err(error(format!(
            "{resource_path} FORGE 'oscal-version' must identify OSCAL v1.2.3"
        )));
    }
    let resolved_catalog_sha256 = match reference.resource_type {
        ResourceType::Profile => {
            let fingerprint =
                require_single_prop(&resource_path, &reference.props, "resolved-catalog-sha256")?;
            validate_sha256(
                &format!("{resource_path} FORGE 'resolved-catalog-sha256'"),
                fingerprint,
            )?;
            Some(fingerprint.to_string())
        }
        ResourceType::Catalog => {
            if optional_single_prop(&resource_path, &reference.props, "resolved-catalog-sha256")?
                .is_some()
            {
                return Err(error(format!(
                    "{resource_path} unexpectedly identifies a resolved Catalog companion"
                )));
            }
            None
        }
    };
    Ok(ResourceEvidence {
        resource_type: reference.resource_type,
        href: reference.href.clone(),
        raw_sha256: raw_sha256.to_string(),
        root_uuid: root_uuid.to_string(),
        document_version: document_version.to_string(),
        oscal_version: oscal_version.to_string(),
        resolved_catalog_sha256,
    })
}

fn record_source_resource(
    path: &str,
    source: &ResourceEvidence,
    validation_state: &mut MappingValidationState,
) -> Result<(), ForgeError> {
    if validation_state
        .source_resources_by_href
        .get(&source.href)
        .is_some_and(|existing| existing != source)
    {
        return Err(error(format!(
            "{path}.source-resource contradicts another resource fingerprint for href '{}'",
            bounded(&source.href)
        )));
    }
    validation_state.source_resources_by_href.insert(source.href.clone(), source.clone());
    Ok(())
}

fn validate_stable_uuid(
    path: &str,
    kind: &str,
    key: &str,
    actual: uuid::Uuid,
) -> Result<(), ForgeError> {
    if key.trim().is_empty() {
        return Err(error(format!("{path} FORGE '{kind}-key' must not be empty")));
    }
    let expected = stable_uuid(kind, key);
    if actual != expected {
        return Err(error(format!(
            "{path}.uuid does not match the deterministic FORGE {kind} identity for key '{}'",
            bounded(key)
        )));
    }
    Ok(())
}

fn validate_sha256(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(error(format!("{path} must be 64 lowercase hexadecimal characters")));
    }
    Ok(())
}

fn validate_framework_reference(
    path: &str,
    reference: &MappingResourceReference,
    framework: &LoadedResource,
) -> Result<(), ForgeError> {
    if reference.resource_type != framework.evidence.resource_type {
        return Err(error(format!("{path}.target-resource type does not match the framework")));
    }
    for (name, expected) in [
        ("raw-sha256", framework.evidence.raw_sha256.as_str()),
        ("root-uuid", framework.evidence.root_uuid.as_str()),
        ("document-version", framework.evidence.document_version.as_str()),
        ("oscal-version", framework.evidence.oscal_version.as_str()),
    ] {
        require_matching_prop(path, &reference.props, name, expected)?;
    }
    match framework.evidence.resolved_catalog_sha256.as_deref() {
        Some(expected) => {
            require_matching_prop(path, &reference.props, "resolved-catalog-sha256", expected)?;
        }
        None if reference.props.iter().any(|prop| {
            prop.ns.as_deref() == Some(FORGE_MAPPING_NS) && prop.name == "resolved-catalog-sha256"
        }) =>
        {
            return Err(error(format!(
                "{path}.target-resource unexpectedly identifies a resolved Catalog companion"
            )));
        }
        None => {}
    }
    Ok(())
}

fn require_matching_prop(
    path: &str,
    props: &[OscalProp],
    name: &str,
    expected: &str,
) -> Result<(), ForgeError> {
    let actual = require_single_prop(&format!("{path}.target-resource"), props, name)?;
    if actual != expected {
        return Err(error(format!(
            "{path}.target-resource '{name}' does not match the applicability framework"
        )));
    }
    Ok(())
}

fn require_single_prop<'a>(
    path: &str,
    props: &'a [OscalProp],
    name: &str,
) -> Result<&'a str, ForgeError> {
    let values: Vec<_> = props
        .iter()
        .filter_map(|prop| {
            (prop.ns.as_deref() == Some(FORGE_MAPPING_NS) && prop.name == name)
                .then_some(prop.value.as_str())
        })
        .collect();
    if values.len() != 1 {
        return Err(error(format!("{path} must contain exactly one FORGE '{name}' property")));
    }
    Ok(values[0])
}

fn optional_single_prop<'a>(
    path: &str,
    props: &'a [OscalProp],
    name: &str,
) -> Result<Option<&'a str>, ForgeError> {
    let mut values = props.iter().filter_map(|prop| {
        (prop.ns.as_deref() == Some(FORGE_MAPPING_NS) && prop.name == name)
            .then_some(prop.value.as_str())
    });
    let first = values.next();
    if values.next().is_some() {
        return Err(error(format!("{path} must contain at most one FORGE '{name}' property")));
    }
    Ok(first)
}

fn render_report(
    report: &model::ApplicabilityReport,
    format: &ApplicabilityReportFormat,
) -> Result<String, ForgeError> {
    match format {
        ApplicabilityReportFormat::Json => {
            let mut rendered = serde_json::to_string_pretty(report)
                .map_err(|cause| error(format!("report serialization failed: {cause}")))?;
            rendered.push('\n');
            Ok(rendered)
        }
        ApplicabilityReportFormat::Text => Ok(render_text_report(report)),
        ApplicabilityReportFormat::Html => Ok(render_html_report(report)),
    }
}

fn render_text_report(report: &model::ApplicabilityReport) -> String {
    let mut output = String::new();
    output.push_str("FORGE framework applicability and policy-gap report\n");
    let _ = writeln!(output, "schema: {}", report.schema_version);
    let _ = writeln!(output, "manifest-sha256: {}", report.manifest_sha256);
    append_text_provenance(&mut output, report);
    append_text_counts(&mut output, report);
    append_text_controls(&mut output, report);
    append_text_queue(&mut output, report);
    output
}

fn append_text_provenance(output: &mut String, report: &model::ApplicabilityReport) {
    let _ = writeln!(
        output,
        "framework: type={} href={} raw-sha256={} root-uuid={} document-version={} oscal-version={}",
        report.framework.resource_type.as_str(),
        escape(&report.framework.href),
        report.framework.raw_sha256,
        report.framework.root_uuid,
        escape(&report.framework.document_version),
        escape(&report.framework.oscal_version),
    );
    if let Some(hash) = &report.framework.resolved_catalog_sha256 {
        let _ = writeln!(output, "framework resolved-catalog-sha256: {hash}");
    }
    let _ = writeln!(output, "reviewers: {}", report.reviewers.len());
    for reviewer in &report.reviewers {
        let _ = writeln!(
            output,
            "- key={} type={:?} name={}",
            escape(&reviewer.key),
            reviewer.party_type,
            escape(&reviewer.name)
        );
    }
    let _ = writeln!(output, "mapping collections: {}", report.mapping_collections.len());
    for mapping in &report.mapping_collections {
        let _ = writeln!(
            output,
            "- uuid={} raw-sha256={} version={} oscal-version={} reviewed-at={}",
            mapping.uuid,
            mapping.raw_sha256,
            escape(&mapping.version),
            escape(&mapping.oscal_version),
            escape(&mapping.reviewed_at)
        );
        for source in &mapping.source_resources {
            let _ = writeln!(
                output,
                "  source type={} href={} raw-sha256={} root-uuid={} document-version={} oscal-version={}",
                source.resource_type.as_str(),
                escape(&source.href),
                source.raw_sha256,
                source.root_uuid,
                escape(&source.document_version),
                escape(&source.oscal_version)
            );
            if let Some(hash) = &source.resolved_catalog_sha256 {
                let _ = writeln!(output, "  source resolved-catalog-sha256: {hash}");
            }
        }
        for reviewer in &mapping.reviewers {
            let _ = writeln!(
                output,
                "  reviewer uuid={} type={} name={}",
                reviewer.uuid,
                reviewer_type(reviewer.reviewer_type),
                escape(&reviewer.name)
            );
        }
    }
}

fn append_text_counts(output: &mut String, report: &model::ApplicabilityReport) {
    let counts = &report.counts;
    let _ = writeln!(output, "controls total: {}", counts.total);
    for (label, count) in [
        ("applicable-mapped", counts.applicable_mapped),
        ("applicable-reviewed-no-relationship", counts.applicable_reviewed_no_relationship),
        ("applicable-unmapped", counts.applicable_unmapped),
        ("not-applicable", counts.not_applicable),
        ("deferred", counts.deferred),
        ("under-review", counts.under_review),
    ] {
        let _ = writeln!(output, "{label}: {count}");
    }
    let _ = writeln!(output, "matched controls: {}", report.matched_controls);
}

fn append_text_controls(output: &mut String, report: &model::ApplicabilityReport) {
    output.push_str("control classifications:\n");
    for control in &report.controls {
        let _ = writeln!(
            output,
            "- {} {} positive-maps={} no-relationship-maps={}",
            escape(&control.control_id),
            control.classification.as_str(),
            control.positive_mapping_count,
            control.no_relationship_count,
        );
        if !control.groups.is_empty() {
            let _ = writeln!(output, "  groups: {}", control.groups.join(", "));
        }
        if !control.policy_sources.is_empty() {
            let _ = writeln!(output, "  policy-sources: {}", control.policy_sources.join(", "));
        }
        if let Some(reviewer) = &control.reviewer_key {
            let _ = writeln!(output, "  reviewer-key: {}", escape(reviewer));
        }
        if let Some(reviewed_at) = &control.reviewed_at {
            let _ = writeln!(output, "  reviewed-at: {}", escape(reviewed_at));
        }
        if let Some(rationale) = &control.rationale {
            let _ = writeln!(output, "  rationale: {}", escape(rationale));
        }
        if let Some(revisit_date) = &control.revisit_date {
            let _ = writeln!(output, "  revisit-date: {}", escape(revisit_date));
        }
        if let Some(note) = &control.note {
            let _ = writeln!(output, "  note: {}", escape(note));
        }
    }
}

fn append_text_queue(output: &mut String, report: &model::ApplicabilityReport) {
    let _ = writeln!(output, "review queue: {}", report.review_queue.len());
    for item in &report.review_queue {
        let _ = writeln!(
            output,
            "- {} reason={} owner={} revisit-date={}",
            escape(&item.control_id),
            item.reason_code,
            item.owner.as_deref().map_or_else(|| "unassigned".to_string(), escape),
            item.revisit_date.as_deref().map_or_else(|| "none".to_string(), escape)
        );
    }
}

fn render_html_report(report: &model::ApplicabilityReport) -> String {
    let mut output = String::from(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>FORGE applicability and policy-gap report</title>\n\
         <style>body{font-family:system-ui,sans-serif;max-width:1200px;margin:2rem auto;padding:0 1rem;color:#17202a}table{border-collapse:collapse;width:100%;margin:1rem 0}th,td{border:1px solid #ccd1d1;padding:.45rem;text-align:left;vertical-align:top}th{background:#f2f4f4}code{overflow-wrap:anywhere}.counts{display:grid;grid-template-columns:repeat(auto-fit,minmax(13rem,1fr));gap:.5rem}.count{border:1px solid #ccd1d1;padding:.75rem}</style>\n\
         </head><body>\n<h1>FORGE framework applicability and policy-gap report</h1>\n",
    );
    let _ = writeln!(
        output,
        "<p>Schema <code>{}</code>; manifest SHA-256 <code>{}</code>.</p>",
        report.schema_version, report.manifest_sha256
    );
    let _ = writeln!(
        output,
        "<p>Framework: {} <code>{}</code>, root UUID <code>{}</code>, version <code>{}</code>, OSCAL <code>{}</code>, SHA-256 <code>{}</code>.</p>",
        report.framework.resource_type.as_str(),
        escape_html(&report.framework.href),
        report.framework.root_uuid,
        escape_html(&report.framework.document_version),
        escape_html(&report.framework.oscal_version),
        report.framework.raw_sha256
    );
    append_html_provenance(&mut output, report);
    append_html_counts(&mut output, report);
    append_html_controls(&mut output, report);
    append_html_queue(&mut output, report);
    output.push_str("</body></html>\n");
    output
}

fn append_html_provenance(output: &mut String, report: &model::ApplicabilityReport) {
    output.push_str("<section><h2>Review provenance</h2><ul>\n");
    for reviewer in &report.reviewers {
        let _ = writeln!(
            output,
            "<li>Applicability reviewer <code>{}</code> ({}, {})</li>",
            escape_html(&reviewer.key),
            reviewer_type(reviewer.party_type),
            escape_html(&reviewer.name)
        );
    }
    for mapping in &report.mapping_collections {
        let _ = writeln!(
            output,
            "<li>Mapping Collection <code>{}</code>, version <code>{}</code>, OSCAL <code>{}</code>, SHA-256 <code>{}</code>, reviewed <code>{}</code></li>",
            mapping.uuid,
            escape_html(&mapping.version),
            escape_html(&mapping.oscal_version),
            mapping.raw_sha256,
            escape_html(&mapping.reviewed_at)
        );
        for source in &mapping.source_resources {
            let _ = writeln!(
                output,
                "<li>Policy source {} <code>{}</code>, root UUID <code>{}</code>, version <code>{}</code>, OSCAL <code>{}</code>, SHA-256 <code>{}</code></li>",
                source.resource_type.as_str(),
                escape_html(&source.href),
                source.root_uuid,
                escape_html(&source.document_version),
                escape_html(&source.oscal_version),
                source.raw_sha256
            );
            if let Some(hash) = &source.resolved_catalog_sha256 {
                let _ = writeln!(
                    output,
                    "<li>Policy source resolved Catalog SHA-256 <code>{hash}</code></li>"
                );
            }
        }
        for reviewer in &mapping.reviewers {
            let _ = writeln!(
                output,
                "<li>Mapping reviewer <code>{}</code> ({}, {})</li>",
                reviewer.uuid,
                reviewer_type(reviewer.reviewer_type),
                escape_html(&reviewer.name)
            );
        }
    }
    output.push_str("</ul></section>\n");
}

fn append_html_counts(output: &mut String, report: &model::ApplicabilityReport) {
    output.push_str("<section><h2>Inventory totals</h2><div class=\"counts\">\n");
    for (label, count) in [
        ("Total controls", report.counts.total),
        ("Applicable mapped", report.counts.applicable_mapped),
        (
            "Applicable reviewed with no relationship",
            report.counts.applicable_reviewed_no_relationship,
        ),
        ("Applicable unmapped", report.counts.applicable_unmapped),
        ("Not applicable", report.counts.not_applicable),
        ("Deferred", report.counts.deferred),
        ("Under review", report.counts.under_review),
    ] {
        let _ = writeln!(
            output,
            "<div class=\"count\"><strong>{}</strong><br>{count}</div>",
            escape_html(label)
        );
    }
    let _ = writeln!(
        output,
        "</div><p>Displayed controls: {}. Filters never change inventory totals.</p></section>",
        report.matched_controls
    );
}

fn append_html_controls(output: &mut String, report: &model::ApplicabilityReport) {
    output.push_str("<section><h2>Control classifications</h2><table><thead><tr><th>Control</th><th>Groups</th><th>Classification</th><th>Reviewer</th><th>Reviewed</th><th>Rationale</th><th>Revisit</th><th>Note</th><th>Positive maps</th><th>No-relationship maps</th><th>Policy sources</th></tr></thead><tbody>\n");
    for control in &report.controls {
        let _ = writeln!(
            output,
            "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape_html(&control.control_id),
            escape_html(&control.groups.join(", ")),
            control.classification.as_str(),
            control.reviewer_key.as_deref().map(escape_html).unwrap_or_default(),
            control.reviewed_at.as_deref().map(escape_html).unwrap_or_default(),
            control.rationale.as_deref().map(escape_html).unwrap_or_default(),
            control.revisit_date.as_deref().map(escape_html).unwrap_or_default(),
            control.note.as_deref().map(escape_html).unwrap_or_default(),
            control.positive_mapping_count,
            control.no_relationship_count,
            escape_html(&control.policy_sources.join(", "))
        );
    }
    output.push_str("</tbody></table></section>\n");
}

fn append_html_queue(output: &mut String, report: &model::ApplicabilityReport) {
    output.push_str("<section><h2>Review queue</h2><table><thead><tr><th>Control</th><th>Reason code</th><th>Owner</th><th>Revisit</th></tr></thead><tbody>\n");
    for item in &report.review_queue {
        let _ = writeln!(
            output,
            "<tr><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
            escape_html(&item.control_id),
            item.reason_code,
            item.owner.as_deref().map(escape_html).unwrap_or_default(),
            item.revisit_date.as_deref().map(escape_html).unwrap_or_default()
        );
    }
    output.push_str("</tbody></table></section>\n");
}

fn review_required(
    report: &model::ApplicabilityReport,
    manifest: &manifest::ApplicabilityManifest,
    fail_on: &ApplicabilityFailOn,
    as_of: Option<chrono::NaiveDate>,
) -> bool {
    match fail_on {
        ApplicabilityFailOn::Never => false,
        ApplicabilityFailOn::ApplicableUnmapped => report.counts.applicable_unmapped > 0,
        ApplicabilityFailOn::AnyReviewAction => {
            report.counts.applicable_unmapped > 0
                || report.counts.applicable_reviewed_no_relationship > 0
                || report.counts.deferred > 0
                || report.counts.under_review > 0
        }
        ApplicabilityFailOn::OverdueDeferred => {
            let as_of = as_of.expect("overdue gate date validated before analysis");
            manifest.decisions.iter().any(|decision| {
                decision.state == manifest::DecisionState::Deferred
                    && decision.revisit_date.as_deref().is_some_and(|date| {
                        chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                            .is_ok_and(|revisit| revisit < as_of)
                    })
            })
        }
    }
}

fn parse_gate_date(
    fail_on: &ApplicabilityFailOn,
    as_of: Option<&str>,
) -> Result<Option<chrono::NaiveDate>, ForgeError> {
    match (fail_on, as_of) {
        (ApplicabilityFailOn::OverdueDeferred, Some(value)) => {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map(Some)
                .map_err(|_| error("--as-of must use YYYY-MM-DD"))
        }
        (ApplicabilityFailOn::OverdueDeferred, None) => {
            Err(error("--as-of is required with --fail-on overdue-deferred"))
        }
        (_, Some(_)) => Err(error("--as-of is only valid with --fail-on overdue-deferred")),
        (_, None) => Ok(None),
    }
}

fn validate_filters(filters: &model::ReportFilters) -> Result<(), ForgeError> {
    for (flag, value) in [
        ("--group", filters.group.as_deref()),
        ("--control-prefix", filters.control_prefix.as_deref()),
        ("--reviewer", filters.reviewer.as_deref()),
        ("--policy-source", filters.policy_source.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(error(format!("{flag} must not be empty")));
            }
            if value.trim() != value {
                return Err(error(format!(
                    "{flag} must not contain leading or trailing whitespace"
                )));
            }
        }
    }
    Ok(())
}

const fn classification_total(counts: &model::ClassificationCounts) -> usize {
    counts.applicable_mapped
        + counts.applicable_reviewed_no_relationship
        + counts.applicable_unmapped
        + counts.not_applicable
        + counts.deferred
        + counts.under_review
}

fn validate_classification_counts(counts: &model::ClassificationCounts) -> Result<(), ForgeError> {
    let classified = classification_total(counts);
    if classified != counts.total {
        return Err(error(format!(
            "internal classification reconciliation failed: {classified} classified controls for inventory total {}",
            counts.total
        )));
    }
    Ok(())
}

fn validate_applicability_group_ids(inventory: &inventory::Inventory) -> Result<(), ForgeError> {
    if let Some(group_id) = inventory.ambiguous_group_ids().iter().next() {
        return Err(error(format!(
            "framework contains duplicate group id '{}'; applicability group filters would be ambiguous",
            bounded(group_id)
        )));
    }
    Ok(())
}

fn scaffold_framework(
    path: &Path,
    resolved_catalog: Option<&Path>,
    output: Option<&Path>,
) -> Result<(ResourceManifest, LoadedResource), ForgeError> {
    io::check_file_size(path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("framework: {cause}")))?;
    let bytes =
        std::fs::read(path).map_err(|cause| error(format!("framework cannot be read: {cause}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|cause| error(format!("framework is not valid JSON: {cause}")))?;
    let resource_type = match validate::detect_model_type(&value)
        .map_err(|cause| error(format!("framework: {cause}")))?
    {
        OscalModelType::Catalog => ResourceType::Catalog,
        OscalModelType::Profile => ResourceType::Profile,
        other => {
            return Err(error(format!(
                "framework uses unsupported '{}' model; expected Catalog or Profile",
                other.as_str()
            )));
        }
    };
    if resource_type == ResourceType::Profile && resolved_catalog.is_none() {
        return Err(error("--resolved-catalog is required when the framework is a Profile"));
    }
    let temporary = ResourceManifest {
        resource_type,
        artifact: path.to_path_buf(),
        href: safe_file_label(path),
        resolved_catalog: resolved_catalog.map(Path::to_path_buf),
        resolved_catalog_attestation: resolved_catalog.map(|_| true),
        expected_sha256: None,
        inventory: None,
    };
    let loaded = inventory::load(Path::new("."), "$.framework", &temporary)
        .map_err(relabel_mapping_error)?;
    let framework = ResourceManifest {
        resource_type,
        artifact: manifest_relative_path(path, output)?,
        href: safe_file_label(path),
        resolved_catalog: resolved_catalog
            .map(|companion| manifest_relative_path(companion, output))
            .transpose()?,
        resolved_catalog_attestation: resolved_catalog.map(|_| false),
        expected_sha256: Some(loaded.evidence.raw_sha256.clone()),
        inventory: Some(loaded.snapshot()),
    };
    Ok((framework, loaded))
}

fn manifest_relative_path(path: &Path, output: Option<&Path>) -> Result<PathBuf, ForgeError> {
    let target = path.canonicalize().map_err(|cause| {
        error(format!("cannot resolve framework resource '{}': {cause}", path.display()))
    })?;
    let manifest_dir_path = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !manifest_dir_path.is_dir() {
        return Err(error(format!(
            "output directory '{}' does not exist",
            manifest_dir_path.display()
        )));
    }
    let manifest_dir = manifest_dir_path
        .canonicalize()
        .map_err(|cause| error(format!("cannot resolve manifest directory: {cause}")))?;
    Ok(relative_path(&manifest_dir, &target).unwrap_or(target))
}

fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let common = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return None;
    }
    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &target_components[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

fn validate_destination(inputs: &[PathBuf], output: Option<&Path>) -> Result<(), ForgeError> {
    let Some(output) = output else { return Ok(()) };
    for input in inputs {
        if crate::mapping::paths_alias(output, input).map_err(relabel_mapping_error)? {
            return Err(error(format!(
                "destination '{}' aliases an applicability input",
                output.display()
            )));
        }
    }
    Ok(())
}

fn safe_file_label(path: &Path) -> String {
    path.file_name().and_then(|name| name.to_str()).unwrap_or("framework.json").to_string()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn relabel_mapping_error(cause: ForgeError) -> ForgeError {
    match cause {
        ForgeError::MappingBuild(message) => error(message),
        other => error(other.to_string()),
    }
}

fn escape(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

const fn reviewer_type(value: crate::mapping::manifest::ReviewerType) -> &'static str {
    match value {
        crate::mapping::manifest::ReviewerType::Person => "person",
        crate::mapping::manifest::ReviewerType::Organization => "organization",
    }
}

fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::ApplicabilityAnalysis(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_reconciliation_is_a_runtime_invariant() {
        let counts = model::ClassificationCounts {
            total: 2,
            applicable_mapped: 1,
            ..model::ClassificationCounts::default()
        };
        let cause = validate_classification_counts(&counts).expect_err("mismatch must fail");
        assert!(cause.to_string().contains("classification reconciliation failed"));
    }
}
