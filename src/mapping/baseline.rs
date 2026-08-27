//! Stable-identity baseline integrity and change-impact analysis.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use super::inventory::Inventory;
use super::manifest::SubjectType;
use super::model::{
    FindingSeverity, ImpactFinding, MappingCollectionEnvelope, MappingItem, MappingReport,
    OscalMap, OscalProp,
};
use crate::validate::{self, OscalModelType};
use crate::{ForgeError, io};

const MAX_BASELINE_FINDINGS: usize = 10_000;

/// Stable code for a baseline mapping provenance change.
pub const CODE_MAPPING_PROVENANCE_CHANGED: &str = "mapping_provenance_changed";
/// Stable code for a resource byte or resolved-catalog change.
pub const CODE_RESOURCE_CHANGED: &str = "resource_changed";
/// Stable code for a removed reviewed map.
pub const CODE_MAP_REMOVED: &str = "map_removed";
/// Stable code for changed map review evidence.
pub const CODE_MAP_REVIEW_EVIDENCE_CHANGED: &str = "map_review_evidence_changed";
/// Stable code for changed relationship metadata or subject sets.
pub const CODE_RELATIONSHIP_CHANGED: &str = "relationship_changed";
/// Stable code for a newly reviewed map.
pub const CODE_MAP_ADDED: &str = "map_added";
/// Stable code for a stale baseline subject reference.
pub const CODE_STALE_REFERENCE: &str = "stale_reference";
/// Stable code for a subject whose type changed.
pub const CODE_SUBJECT_TYPE_CHANGED: &str = "subject_type_changed";
/// Stable code for a content-changed subject.
pub const CODE_SUBJECT_CHANGED: &str = "subject_changed";
/// Stable code for a newly unmapped subject.
pub const CODE_NEW_GAP: &str = "new_gap";
/// Stable code for a changed gap total.
pub const CODE_GAP_CHANGED: &str = "gap_changed";

/// Compare a valid FORGE mapping baseline with current resources and append findings.
///
/// # Errors
///
/// Returns [`ForgeError::MappingBuild`] when the baseline is unreadable, invalid, lacks
/// required stable evidence, or exceeds the bounded finding limit.
pub fn analyze(
    baseline_path: &Path,
    current: &MappingCollectionEnvelope,
    source_inventory: &Inventory,
    target_inventory: &Inventory,
    report: &mut MappingReport,
) -> Result<(), ForgeError> {
    let bytes = super::inventory::read_bounded_file(baseline_path, io::MAX_FILE_SIZE, "baseline")?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| mapping_error(format!("baseline is not valid JSON: {error}")))?;
    let detected = validate::detect_model_type(&value)
        .map_err(|error| mapping_error(format!("baseline: {error}")))?;
    if detected != OscalModelType::Mapping {
        return Err(mapping_error("baseline must be an OSCAL Control Mapping document"));
    }
    super::inventory::validate_schema("baseline", &value, OscalModelType::Mapping)?;
    let declared =
        value.pointer("/mapping-collection/metadata/oscal-version").and_then(Value::as_str);
    if declared != Some(crate::oscal::OSCAL_VERSION) {
        return Err(mapping_error(format!(
            "baseline must declare OSCAL v{}",
            crate::oscal::OSCAL_VERSION
        )));
    }
    let baseline: MappingCollectionEnvelope = serde_json::from_value(value)
        .map_err(|error| mapping_error(format!("baseline structure is unsupported: {error}")))?;
    verify_integrity(&baseline)?;
    ensure_unique_map_uuids(&baseline)?;
    ensure_single_mapping(&baseline, "baseline")?;
    ensure_unique_map_uuids(current)?;
    ensure_single_mapping(current, "current Mapping")?;

    let mut findings = Vec::new();
    compare_resources(&baseline, current, &mut findings)?;
    compare_maps(&baseline, current, source_inventory, target_inventory, &mut findings)?;
    compare_gaps(&baseline, current, &mut findings)?;
    findings.sort();
    report.findings.extend(findings);
    Ok(())
}

fn verify_integrity(baseline: &MappingCollectionEnvelope) -> Result<(), ForgeError> {
    require_prop(&baseline.mapping_collection.metadata.props, "collection-key", "metadata")?;
    for (mapping_index, mapping) in baseline.mapping_collection.mappings.iter().enumerate() {
        require_prop(&mapping.props, "mapping-key", &format!("mappings[{mapping_index}]"))?;
        for (map_index, map) in mapping.maps.iter().enumerate() {
            require_prop(
                &map.props,
                "map-key",
                &format!("mappings[{mapping_index}].maps[{map_index}]"),
            )?;
            for (side, items) in [("sources", &map.sources), ("targets", &map.targets)] {
                for (item_index, item) in items.iter().enumerate() {
                    require_prop(
                        &item.props,
                        "subject-sha256",
                        &format!(
                            "mappings[{mapping_index}].maps[{map_index}].{side}[{item_index}]"
                        ),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn ensure_unique_map_uuids(document: &MappingCollectionEnvelope) -> Result<(), ForgeError> {
    let mut map_uuids = BTreeSet::new();
    for map in document.mapping_collection.mappings.iter().flat_map(|mapping| &mapping.maps) {
        if !map_uuids.insert(map.uuid) {
            return Err(mapping_error(format!(
                "document contains duplicate map UUID '{}'",
                map.uuid
            )));
        }
    }
    Ok(())
}

fn require_prop(props: &[OscalProp], name: &str, path: &str) -> Result<(), ForgeError> {
    require_unique_prop(props, name, path).map(|_| ())
}

fn require_unique_prop<'a>(
    props: &'a [OscalProp],
    name: &str,
    path: &str,
) -> Result<&'a str, ForgeError> {
    unique_prop(props, name, path)?.ok_or_else(|| {
        mapping_error(format!("baseline {path} lacks required FORGE property '{name}'"))
    })
}

fn unique_prop<'a>(
    props: &'a [OscalProp],
    name: &str,
    path: &str,
) -> Result<Option<&'a str>, ForgeError> {
    let mut matching = props.iter().filter(|prop| {
        prop.name == name && prop.ns.as_deref() == Some(super::inventory::FORGE_MAPPING_NS)
    });
    let Some(prop) = matching.next() else {
        return Ok(None);
    };
    if matching.next().is_some() {
        return Err(mapping_error(format!(
            "baseline {path} has duplicate FORGE property '{name}'"
        )));
    }
    Ok(Some(prop.value.as_str()))
}

fn compare_resources(
    baseline: &MappingCollectionEnvelope,
    current: &MappingCollectionEnvelope,
    findings: &mut Vec<ImpactFinding>,
) -> Result<(), ForgeError> {
    let old = &baseline.mapping_collection.mappings[0];
    let new = &current.mapping_collection.mappings[0];
    if old.method != new.method
        || old.matching_rationale != new.matching_rationale
        || old.status != new.status
        || old.mapping_description != new.mapping_description
        || old.confidence_score != new.confidence_score
        || old.coverage != new.coverage
    {
        push_finding(
            findings,
            finding(
                CODE_MAPPING_PROVENANCE_CHANGED,
                "$.mapping".to_string(),
                "mapping-level provenance or reviewer estimate changed".to_string(),
            ),
        )?;
    }
    for (side, old_resource, new_resource) in [
        ("source", &old.source_resource, &new.source_resource),
        ("target", &old.target_resource, &new.target_resource),
    ] {
        let raw_changed = unique_prop(&old_resource.props, "raw-sha256", side)?
            != unique_prop(&new_resource.props, "raw-sha256", side)?;
        let companion_changed = unique_prop(&old_resource.props, "resolved-catalog-sha256", side)?
            != unique_prop(&new_resource.props, "resolved-catalog-sha256", side)?;
        if raw_changed && companion_changed {
            push_finding(
                findings,
                finding(
                    CODE_RESOURCE_CHANGED,
                    format!("$.mapping.{side}"),
                    format!("{side} resource and resolved Catalog bytes changed"),
                ),
            )?;
        } else if raw_changed {
            push_finding(
                findings,
                finding(
                    CODE_RESOURCE_CHANGED,
                    format!("$.mapping.{side}"),
                    format!("{side} resource bytes changed"),
                ),
            )?;
        } else if companion_changed {
            push_finding(
                findings,
                finding(
                    CODE_RESOURCE_CHANGED,
                    format!("$.mapping.{side}"),
                    format!("{side} resolved Catalog bytes changed"),
                ),
            )?;
        }
    }
    Ok(())
}

fn compare_maps(
    baseline: &MappingCollectionEnvelope,
    current: &MappingCollectionEnvelope,
    source_inventory: &Inventory,
    target_inventory: &Inventory,
    findings: &mut Vec<ImpactFinding>,
) -> Result<(), ForgeError> {
    let old_maps = maps_by_uuid(baseline);
    let new_maps = maps_by_uuid(current);
    for (uuid, old) in &old_maps {
        let path = format!("$.mapping.maps[uuid={uuid}]");
        let Some(new) = new_maps.get(uuid) else {
            push_finding(
                findings,
                finding(CODE_MAP_REMOVED, path, "reviewed map was removed".to_string()),
            )?;
            inspect_items(&old.sources, source_inventory, "source", findings)?;
            inspect_items(&old.targets, target_inventory, "target", findings)?;
            continue;
        };
        inspect_items(&old.sources, source_inventory, "source", findings)?;
        inspect_items(&old.targets, target_inventory, "target", findings)?;
        if review_evidence(old)? != review_evidence(new)? {
            push_finding(
                findings,
                finding(
                    CODE_MAP_REVIEW_EVIDENCE_CHANGED,
                    path.clone(),
                    "map reviewer key or review timestamp changed".to_string(),
                ),
            )?;
        }
        if old.relationship != new.relationship
            || old.matching_rationale != new.matching_rationale
            || old.remarks != new.remarks
            || old.confidence_score != new.confidence_score
            || old.coverage != new.coverage
            || old.qualifiers != new.qualifiers
            || subject_keys(&old.sources) != subject_keys(&new.sources)
            || subject_keys(&old.targets) != subject_keys(&new.targets)
        {
            push_finding(
                findings,
                finding(
                    CODE_RELATIONSHIP_CHANGED,
                    path,
                    "relationship, rationale, or subject set changed".to_string(),
                ),
            )?;
        }
    }
    for uuid in new_maps.keys().filter(|uuid| !old_maps.contains_key(uuid.as_str())) {
        push_finding(
            findings,
            finding(
                CODE_MAP_ADDED,
                format!("$.mapping.maps[uuid={uuid}]"),
                "new reviewed map was added".to_string(),
            ),
        )?;
    }
    Ok(())
}

fn inspect_items(
    items: &[MappingItem],
    inventory: &Inventory,
    side: &str,
    findings: &mut Vec<ImpactFinding>,
) -> Result<(), ForgeError> {
    for item in items {
        let path = format!("$.baseline.{side}.{}[{}]", item.subject_type.as_str(), item.id_ref);
        let Some(current_hash) = inventory.fingerprint(item.subject_type, &item.id_ref) else {
            let code = if inventory.type_for_id(&item.id_ref).is_some() {
                CODE_SUBJECT_TYPE_CHANGED
            } else {
                CODE_STALE_REFERENCE
            };
            push_finding(
                findings,
                finding(
                    code,
                    path,
                    format!(
                        "baseline {} '{}' no longer resolves",
                        item.subject_type.as_str(),
                        item.id_ref
                    ),
                ),
            )?;
            continue;
        };
        let old_hash = require_unique_prop(&item.props, "subject-sha256", &path)?;
        if old_hash != current_hash {
            push_finding(
                findings,
                finding_with_fingerprints(
                    CODE_SUBJECT_CHANGED,
                    path,
                    format!(
                        "{} '{}' content fingerprint changed",
                        item.subject_type.as_str(),
                        item.id_ref
                    ),
                    old_hash,
                    current_hash,
                ),
            )?;
        }
    }
    Ok(())
}

fn compare_gaps(
    baseline: &MappingCollectionEnvelope,
    current: &MappingCollectionEnvelope,
    findings: &mut Vec<ImpactFinding>,
) -> Result<(), ForgeError> {
    let Some(old) = baseline.mapping_collection.mappings.first() else { return Ok(()) };
    let Some(new) = current.mapping_collection.mappings.first() else { return Ok(()) };
    for (side, old_gap, new_gap) in [
        ("source", &old.source_gap_summary, &new.source_gap_summary),
        ("target", &old.target_gap_summary, &new.target_gap_summary),
    ] {
        let old_ids = gap_ids(old_gap.as_ref());
        let new_ids = gap_ids(new_gap.as_ref());
        for id in new_ids.difference(&old_ids) {
            push_finding(
                findings,
                finding(
                    CODE_NEW_GAP,
                    format!("$.mapping.{side}-gap-summary[{id}]"),
                    format!("{side} control '{id}' is newly unmapped"),
                ),
            )?;
        }
        if old_ids.len() != new_ids.len() {
            push_finding(
                findings,
                finding(
                    CODE_GAP_CHANGED,
                    format!("$.mapping.{side}-gap-summary"),
                    format!("{side} gap count changed from {} to {}", old_ids.len(), new_ids.len()),
                ),
            )?;
        }
    }
    Ok(())
}

fn push_finding(
    findings: &mut Vec<ImpactFinding>,
    finding: ImpactFinding,
) -> Result<(), ForgeError> {
    if findings.len() >= MAX_BASELINE_FINDINGS {
        return Err(mapping_error(format!(
            "baseline impact exceeds the {MAX_BASELINE_FINDINGS} finding limit"
        )));
    }
    findings.push(finding);
    Ok(())
}

fn maps_by_uuid(document: &MappingCollectionEnvelope) -> BTreeMap<String, &OscalMap> {
    document
        .mapping_collection
        .mappings
        .iter()
        .flat_map(|mapping| mapping.maps.iter())
        .map(|map| (map.uuid.to_string(), map))
        .collect()
}

fn subject_keys(items: &[MappingItem]) -> Vec<(SubjectType, &str)> {
    items.iter().map(|item| (item.subject_type, item.id_ref.as_str())).collect()
}

fn gap_ids(summary: Option<&super::model::GapSummary>) -> BTreeSet<String> {
    summary
        .into_iter()
        .flat_map(|summary| &summary.unmapped_controls)
        .flat_map(|selection| &selection.with_ids)
        .cloned()
        .collect()
}

fn ensure_single_mapping(
    document: &MappingCollectionEnvelope,
    label: &str,
) -> Result<(), ForgeError> {
    let count = document.mapping_collection.mappings.len();
    if count != 1 {
        return Err(mapping_error(format!(
            "{label} must contain exactly one mapping; found {count}"
        )));
    }
    Ok(())
}

fn review_evidence(map: &OscalMap) -> Result<(Option<&str>, Option<&str>), ForgeError> {
    Ok((
        unique_prop(&map.props, "reviewer-key", "map review evidence")?,
        unique_prop(&map.props, "reviewed-at", "map review evidence")?,
    ))
}

fn finding(code: &str, path: String, message: String) -> ImpactFinding {
    ImpactFinding {
        severity: FindingSeverity::Review,
        code: code.to_string(),
        path,
        message,
        old_fingerprint: None,
        new_fingerprint: None,
    }
}

fn finding_with_fingerprints(
    code: &str,
    path: String,
    message: String,
    old_fingerprint: &str,
    new_fingerprint: &str,
) -> ImpactFinding {
    ImpactFinding {
        severity: FindingSeverity::Review,
        code: code.to_string(),
        path,
        message,
        old_fingerprint: Some(old_fingerprint.to_string()),
        new_fingerprint: Some(new_fingerprint.to_string()),
    }
}

fn mapping_error(message: impl Into<String>) -> ForgeError {
    ForgeError::MappingBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_forge_property_is_rejected() {
        let props = vec![
            OscalProp {
                name: "subject-sha256".to_string(),
                ns: Some(super::super::inventory::FORGE_MAPPING_NS.to_string()),
                value: "a".repeat(64),
            },
            OscalProp {
                name: "subject-sha256".to_string(),
                ns: Some(super::super::inventory::FORGE_MAPPING_NS.to_string()),
                value: "b".repeat(64),
            },
        ];

        let error = require_unique_prop(&props, "subject-sha256", "item")
            .expect_err("ambiguous integrity evidence must fail");
        assert!(error.to_string().contains("duplicate FORGE property 'subject-sha256'"));
    }

    #[test]
    fn duplicate_map_uuids_are_rejected_before_comparison() {
        let document: MappingCollectionEnvelope = serde_json::from_value(serde_json::json!({
            "mapping-collection": {
                "uuid": "00000000-0000-4000-8000-000000000000",
                "metadata": {
                    "title": "Collection",
                    "last-modified": "2026-08-26T00:00:00Z",
                    "version": "1",
                    "oscal-version": crate::oscal::OSCAL_VERSION
                },
                "provenance": {
                    "method": "human",
                    "matching-rationale": "semantic",
                    "status": "complete",
                    "mapping-description": "Reviewed mapping."
                },
                "mappings": [{
                    "uuid": "10000000-0000-4000-8000-000000000000",
                    "source-resource": { "type": "catalog", "href": "source.json" },
                    "target-resource": { "type": "catalog", "href": "target.json" },
                    "maps": [
                        {
                            "uuid": "20000000-0000-4000-8000-000000000000",
                            "relationship": "equivalent-to",
                            "sources": [],
                            "targets": []
                        },
                        {
                            "uuid": "20000000-0000-4000-8000-000000000000",
                            "relationship": "equivalent-to",
                            "sources": [],
                            "targets": []
                        }
                    ]
                }]
            }
        }))
        .expect("test mapping artifact deserializes");

        let error = ensure_unique_map_uuids(&document)
            .expect_err("duplicate map UUIDs must not be silently collapsed");
        assert!(error.to_string().contains("document contains duplicate map UUID"));
    }
}
