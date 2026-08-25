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
    io::check_file_size(baseline_path, io::MAX_FILE_SIZE)
        .map_err(|error| mapping_error(format!("baseline: {error}")))?;
    let bytes = std::fs::read(baseline_path)
        .map_err(|error| mapping_error(format!("baseline: {error}")))?;
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
    if declared != Some("1.2.3") {
        return Err(mapping_error("baseline must declare OSCAL v1.2.3"));
    }
    let baseline: MappingCollectionEnvelope = serde_json::from_value(value)
        .map_err(|error| mapping_error(format!("baseline structure is unsupported: {error}")))?;
    verify_integrity(&baseline)?;

    let mut findings = Vec::new();
    compare_resources(&baseline, current, &mut findings);
    compare_maps(&baseline, current, source_inventory, target_inventory, &mut findings)?;
    compare_gaps(&baseline, current, &mut findings);
    findings.sort();
    if findings.len() > MAX_BASELINE_FINDINGS {
        return Err(mapping_error(format!(
            "baseline impact exceeds the {MAX_BASELINE_FINDINGS} finding limit"
        )));
    }
    report.findings = findings;
    Ok(())
}

fn verify_integrity(baseline: &MappingCollectionEnvelope) -> Result<(), ForgeError> {
    require_prop(&baseline.mapping_collection.metadata.props, "collection-key", "metadata")?;
    let mut map_uuids = BTreeSet::new();
    for (mapping_index, mapping) in baseline.mapping_collection.mappings.iter().enumerate() {
        require_prop(&mapping.props, "mapping-key", &format!("mappings[{mapping_index}]"))?;
        for (map_index, map) in mapping.maps.iter().enumerate() {
            if !map_uuids.insert(map.uuid) {
                return Err(mapping_error(format!(
                    "baseline contains duplicate map UUID '{}'",
                    map.uuid
                )));
            }
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

fn require_prop(props: &[OscalProp], name: &str, path: &str) -> Result<(), ForgeError> {
    if prop_value(props, name).is_none() {
        Err(mapping_error(format!("baseline {path} lacks required FORGE property '{name}'")))
    } else {
        Ok(())
    }
}

fn compare_resources(
    baseline: &MappingCollectionEnvelope,
    current: &MappingCollectionEnvelope,
    findings: &mut Vec<ImpactFinding>,
) {
    let Some(old) = baseline.mapping_collection.mappings.first() else { return };
    let Some(new) = current.mapping_collection.mappings.first() else { return };
    if old.method != new.method
        || old.matching_rationale != new.matching_rationale
        || old.status != new.status
        || old.mapping_description != new.mapping_description
        || old.confidence_score != new.confidence_score
        || old.coverage != new.coverage
    {
        findings.push(finding(
            "mapping_provenance_changed",
            "$.mapping".to_string(),
            "mapping-level provenance or reviewer estimate changed".to_string(),
        ));
    }
    for (side, old_resource, new_resource) in [
        ("source", &old.source_resource, &new.source_resource),
        ("target", &old.target_resource, &new.target_resource),
    ] {
        let raw_changed = prop_value(&old_resource.props, "raw-sha256")
            != prop_value(&new_resource.props, "raw-sha256");
        let companion_changed = prop_value(&old_resource.props, "resolved-catalog-sha256")
            != prop_value(&new_resource.props, "resolved-catalog-sha256");
        if raw_changed || companion_changed {
            let detail = match (raw_changed, companion_changed) {
                (true, true) => "resource and resolved Catalog bytes changed",
                (true, false) => "resource bytes changed",
                (false, true) => "resolved Catalog bytes changed",
                (false, false) => unreachable!("guard requires a changed resource fingerprint"),
            };
            findings.push(finding(
                "resource_changed",
                format!("$.mapping.{side}"),
                format!("{side} {detail}"),
            ));
        }
    }
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
            findings.push(finding("map_removed", path, "reviewed map was removed".to_string()));
            inspect_items(&old.sources, source_inventory, "source", findings)?;
            inspect_items(&old.targets, target_inventory, "target", findings)?;
            continue;
        };
        inspect_items(&old.sources, source_inventory, "source", findings)?;
        inspect_items(&old.targets, target_inventory, "target", findings)?;
        if review_evidence(old) != review_evidence(new) {
            findings.push(finding(
                "map_review_evidence_changed",
                path.clone(),
                "map reviewer key or review timestamp changed".to_string(),
            ));
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
            findings.push(finding(
                "relationship_changed",
                path,
                "relationship, rationale, or subject set changed".to_string(),
            ));
        }
    }
    for uuid in new_maps.keys().filter(|uuid| !old_maps.contains_key(uuid.as_str())) {
        findings.push(finding(
            "map_added",
            format!("$.mapping.maps[uuid={uuid}]"),
            "new reviewed map was added".to_string(),
        ));
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
                "subject_type_changed"
            } else {
                "stale_reference"
            };
            findings.push(finding(
                code,
                path,
                format!(
                    "baseline {} '{}' no longer resolves",
                    item.subject_type.as_str(),
                    item.id_ref
                ),
            ));
            continue;
        };
        let old_hash = prop_value(&item.props, "subject-sha256")
            .ok_or_else(|| mapping_error("baseline item lacks subject-sha256"))?;
        if old_hash != current_hash {
            findings.push(finding_with_fingerprints(
                "subject_changed",
                path,
                format!(
                    "{} '{}' content fingerprint changed",
                    item.subject_type.as_str(),
                    item.id_ref
                ),
                old_hash,
                current_hash,
            ));
        }
    }
    Ok(())
}

fn compare_gaps(
    baseline: &MappingCollectionEnvelope,
    current: &MappingCollectionEnvelope,
    findings: &mut Vec<ImpactFinding>,
) {
    let Some(old) = baseline.mapping_collection.mappings.first() else { return };
    let Some(new) = current.mapping_collection.mappings.first() else { return };
    for (side, old_gap, new_gap) in [
        ("source", &old.source_gap_summary, &new.source_gap_summary),
        ("target", &old.target_gap_summary, &new.target_gap_summary),
    ] {
        let old_ids = gap_ids(old_gap.as_ref());
        let new_ids = gap_ids(new_gap.as_ref());
        for id in new_ids.difference(&old_ids) {
            findings.push(finding(
                "new_gap",
                format!("$.mapping.{side}-gap-summary[{id}]"),
                format!("{side} control '{id}' is newly unmapped"),
            ));
        }
        if old_ids.len() != new_ids.len() {
            findings.push(finding(
                "gap_changed",
                format!("$.mapping.{side}-gap-summary"),
                format!("{side} gap count changed from {} to {}", old_ids.len(), new_ids.len()),
            ));
        }
    }
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

fn prop_value<'a>(props: &'a [OscalProp], name: &str) -> Option<&'a str> {
    props
        .iter()
        .find(|prop| {
            prop.name == name && prop.ns.as_deref() == Some(super::inventory::FORGE_MAPPING_NS)
        })
        .map(|prop| prop.value.as_str())
}

fn review_evidence(map: &OscalMap) -> (Option<&str>, Option<&str>) {
    (prop_value(&map.props, "reviewer-key"), prop_value(&map.props, "reviewed-at"))
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
