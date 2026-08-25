//! Local OSCAL resource validation and deterministic subject inventories.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::manifest::{ResourceInventorySnapshot, ResourceManifest, ResourceType, SubjectType};
use crate::validate::{self, OscalModelType};
use crate::{ForgeError, io};

/// Namespace used for FORGE Control Mapping extension properties.
pub const FORGE_MAPPING_NS: &str = "https://policy-forge.github.io/ns/control-mapping";
/// Maximum eligible subjects in one effective resource.
pub const MAX_INVENTORY_SUBJECTS: usize = 100_000;
/// Maximum control/part nesting depth inventoried.
pub const MAX_INVENTORY_DEPTH: usize = 64;
/// Maximum schema/version errors collected for one mapping input.
pub const MAX_SCHEMA_ERRORS: usize = 100;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResourceEvidence {
    pub resource_type: ResourceType,
    pub href: String,
    pub raw_sha256: String,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_catalog_sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    subjects: BTreeMap<SubjectType, BTreeMap<String, String>>,
    ids: BTreeMap<String, SubjectType>,
    ineligible_parts: BTreeMap<String, String>,
    excerpts: BTreeMap<SubjectType, BTreeMap<String, String>>,
    control_groups: BTreeMap<String, Vec<String>>,
    group_ids: BTreeSet<String>,
    ambiguous_group_ids: BTreeSet<String>,
}

impl Inventory {
    #[must_use]
    pub fn contains(&self, subject_type: SubjectType, id: &str) -> bool {
        self.subjects.get(&subject_type).is_some_and(|subjects| subjects.contains_key(id))
    }

    #[must_use]
    pub fn type_for_id(&self, id: &str) -> Option<SubjectType> {
        self.ids.get(id).copied()
    }

    #[must_use]
    pub fn ineligible_part_name(&self, id: &str) -> Option<&str> {
        self.ineligible_parts.get(id).map(String::as_str)
    }

    #[must_use]
    pub fn fingerprint(&self, subject_type: SubjectType, id: &str) -> Option<&str> {
        self.subjects.get(&subject_type).and_then(|subjects| subjects.get(id)).map(String::as_str)
    }

    #[must_use]
    pub fn excerpt(&self, subject_type: SubjectType, id: &str) -> Option<&str> {
        self.excerpts.get(&subject_type).and_then(|excerpts| excerpts.get(id)).map(String::as_str)
    }

    #[must_use]
    pub fn ids_of_type(&self, subject_type: SubjectType) -> BTreeSet<String> {
        self.subjects
            .get(&subject_type)
            .into_iter()
            .flat_map(|subjects| subjects.keys())
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn count(&self, subject_type: SubjectType) -> usize {
        self.subjects.get(&subject_type).map_or(0, BTreeMap::len)
    }

    /// Return the containing group hierarchy for an eligible control.
    #[must_use]
    pub fn groups_for_control(&self, id: &str) -> &[String] {
        self.control_groups.get(id).map_or(&[], Vec::as_slice)
    }

    /// Return group IDs that occur more than once in the effective Catalog.
    #[must_use]
    pub fn ambiguous_group_ids(&self) -> &BTreeSet<String> {
        &self.ambiguous_group_ids
    }
}

impl LoadedResource {
    #[must_use]
    pub fn snapshot(&self) -> ResourceInventorySnapshot {
        ResourceInventorySnapshot {
            root_uuid: self.evidence.root_uuid.clone(),
            document_version: self.evidence.document_version.clone(),
            oscal_version: self.evidence.oscal_version.clone(),
            control_ids: self.inventory.ids_of_type(SubjectType::Control).into_iter().collect(),
            statement_ids: self.inventory.ids_of_type(SubjectType::Statement).into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadedResource {
    pub path: PathBuf,
    pub evidence: ResourceEvidence,
    pub inventory: Inventory,
}

/// Load, schema-validate, fingerprint, and inventory one manifest resource.
///
/// # Errors
///
/// Returns [`ForgeError::MappingBuild`] for unsafe input size, invalid JSON/schema/type,
/// evidence mismatch, duplicate subjects, or an exceeded inventory bound.
pub fn load(
    manifest_dir: &Path,
    path_label: &str,
    resource: &ResourceManifest,
) -> Result<LoadedResource, ForgeError> {
    let artifact_path = manifest_dir.join(&resource.artifact);
    let bytes = read_bounded_json(&artifact_path, path_label)?;
    let raw_sha256 = sha256(&bytes);
    if let Some(expected) = &resource.expected_sha256
        && expected != &raw_sha256
    {
        return Err(mapping_error(format!(
            "{path_label}.expected_sha256 mismatch: expected {expected}, got {raw_sha256}"
        )));
    }
    let json: Value = serde_json::from_slice(&bytes).map_err(|error| {
        mapping_error(format!("{path_label}.artifact is not valid JSON: {error}"))
    })?;
    let expected_model = match resource.resource_type {
        ResourceType::Catalog => OscalModelType::Catalog,
        ResourceType::Profile => OscalModelType::Profile,
    };
    let detected = validate::detect_model_type(&json)
        .map_err(|error| mapping_error(format!("{path_label}.artifact: {error}")))?;
    if detected != expected_model {
        return Err(mapping_error(format!(
            "{path_label}.type declares '{}' but artifact root is '{}'",
            resource.resource_type.as_str(),
            detected.as_str()
        )));
    }
    validate_schema(path_label, &json, expected_model)?;
    let mut evidence = extract_evidence(path_label, resource, &json, raw_sha256)?;

    let inventory = if resource.resource_type == ResourceType::Profile {
        let companion = resource.resolved_catalog.as_ref().ok_or_else(|| {
            mapping_error(format!("{path_label}.resolved_catalog is required for a Profile"))
        })?;
        let companion_path = manifest_dir.join(companion);
        let companion_bytes = read_bounded_json(&companion_path, path_label)?;
        evidence.resolved_catalog_sha256 = Some(sha256(&companion_bytes));
        let companion_json: Value = serde_json::from_slice(&companion_bytes).map_err(|error| {
            mapping_error(format!("{path_label}.resolved_catalog is not valid JSON: {error}"))
        })?;
        let detected = validate::detect_model_type(&companion_json)
            .map_err(|error| mapping_error(format!("{path_label}.resolved_catalog: {error}")))?;
        if detected != OscalModelType::Catalog {
            return Err(mapping_error(format!(
                "{path_label}.resolved_catalog must contain a Catalog root"
            )));
        }
        validate_schema(path_label, &companion_json, OscalModelType::Catalog)?;
        inventory_catalog(path_label, &companion_json)?
    } else {
        inventory_catalog(path_label, &json)?
    };

    if let Some(expected) = &resource.inventory {
        let actual = ResourceInventorySnapshot {
            root_uuid: evidence.root_uuid.clone(),
            document_version: evidence.document_version.clone(),
            oscal_version: evidence.oscal_version.clone(),
            control_ids: inventory.ids_of_type(SubjectType::Control).into_iter().collect(),
            statement_ids: inventory.ids_of_type(SubjectType::Statement).into_iter().collect(),
        };
        if expected != &actual {
            return Err(mapping_error(format!(
                "{path_label}.inventory no longer matches the supplied resource; regenerate or review the manifest"
            )));
        }
    }
    Ok(LoadedResource { path: artifact_path, evidence, inventory })
}

pub(crate) fn validate_schema(
    path_label: &str,
    json: &Value,
    model: OscalModelType,
) -> Result<(), ForgeError> {
    let validator = validate::compiled_validator(model).map_err(|error| {
        mapping_error(format!("{path_label} schema compilation failed: {error}"))
    })?;
    let mut errors: Vec<_> = validator
        .iter_errors(json)
        .take(MAX_SCHEMA_ERRORS + 1)
        .map(|error| error.to_string())
        .collect();
    let version = validate::version::inspect_oscal_version(json, model);
    if let Some(error) = version.error {
        errors.push(error.message);
    }
    if !errors.is_empty() {
        let truncated = errors.len() > MAX_SCHEMA_ERRORS;
        errors.truncate(MAX_SCHEMA_ERRORS);
        let mut detail = errors.into_iter().take(10).collect::<Vec<_>>().join("; ");
        if truncated {
            detail.push_str("; additional schema errors omitted at configured bound");
        }
        return Err(mapping_error(format!(
            "{path_label} is not a valid {}: {detail}",
            model.as_str()
        )));
    }
    Ok(())
}

fn extract_evidence(
    path_label: &str,
    resource: &ResourceManifest,
    json: &Value,
    raw_sha256: String,
) -> Result<ResourceEvidence, ForgeError> {
    let root = json.get(resource.resource_type.as_str()).ok_or_else(|| {
        mapping_error(format!("{path_label} is missing the declared resource root"))
    })?;
    let root_uuid = required_string(root, "uuid", &format!("{path_label}.root.uuid"))?;
    Uuid::parse_str(&root_uuid)
        .map_err(|_| mapping_error(format!("{path_label}.root.uuid is not a valid UUID")))?;
    let metadata = root
        .get("metadata")
        .ok_or_else(|| mapping_error(format!("{path_label}.root.metadata is required")))?;
    let document_version =
        required_string(metadata, "version", &format!("{path_label}.metadata.version"))?;
    let oscal_version = required_string(
        metadata,
        "oscal-version",
        &format!("{path_label}.metadata.oscal-version"),
    )?;
    Ok(ResourceEvidence {
        resource_type: resource.resource_type,
        href: resource.href.clone(),
        raw_sha256,
        root_uuid,
        document_version,
        oscal_version,
        resolved_catalog_sha256: None,
    })
}

fn required_string(value: &Value, key: &str, path: &str) -> Result<String, ForgeError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| mapping_error(format!("{path} must be a non-empty string")))
}

fn inventory_catalog(path_label: &str, json: &Value) -> Result<Inventory, ForgeError> {
    let catalog = json.get("catalog").ok_or_else(|| {
        mapping_error(format!("{path_label} effective inventory is not a Catalog"))
    })?;
    let mut inventory = Inventory {
        subjects: BTreeMap::new(),
        ids: BTreeMap::new(),
        ineligible_parts: BTreeMap::new(),
        excerpts: BTreeMap::new(),
        control_groups: BTreeMap::new(),
        group_ids: BTreeSet::new(),
        ambiguous_group_ids: BTreeSet::new(),
    };
    if let Some(controls) = catalog.get("controls").and_then(Value::as_array) {
        inventory_controls(path_label, controls, 0, &[], &mut inventory)?;
    }
    if let Some(groups) = catalog.get("groups").and_then(Value::as_array) {
        inventory_groups(path_label, groups, 0, &[], &mut inventory)?;
    }
    Ok(inventory)
}

fn inventory_groups(
    path_label: &str,
    groups: &[Value],
    depth: usize,
    parent_groups: &[String],
    inventory: &mut Inventory,
) -> Result<(), ForgeError> {
    enforce_depth(path_label, depth)?;
    for group in groups {
        let mut group_path = parent_groups.to_vec();
        if let Some(group_id) =
            group.get("id").and_then(Value::as_str).filter(|id| !id.trim().is_empty())
        {
            if !inventory.group_ids.insert(group_id.to_string()) {
                inventory.ambiguous_group_ids.insert(group_id.to_string());
            }
            group_path.push(group_id.to_string());
        }
        if let Some(controls) = group.get("controls").and_then(Value::as_array) {
            inventory_controls(path_label, controls, depth + 1, &group_path, inventory)?;
        }
        if let Some(children) = group.get("groups").and_then(Value::as_array) {
            inventory_groups(path_label, children, depth + 1, &group_path, inventory)?;
        }
    }
    Ok(())
}

fn inventory_controls(
    path_label: &str,
    controls: &[Value],
    depth: usize,
    groups: &[String],
    inventory: &mut Inventory,
) -> Result<(), ForgeError> {
    enforce_depth(path_label, depth)?;
    for control in controls {
        let control_id = insert_subject(path_label, control, SubjectType::Control, inventory)?;
        inventory.control_groups.insert(control_id.to_string(), groups.to_vec());
        if let Some(parts) = control.get("parts").and_then(Value::as_array) {
            inventory_parts(path_label, parts, depth + 1, inventory)?;
        }
        if let Some(children) = control.get("controls").and_then(Value::as_array) {
            inventory_controls(path_label, children, depth + 1, groups, inventory)?;
        }
    }
    Ok(())
}

fn inventory_parts(
    path_label: &str,
    parts: &[Value],
    depth: usize,
    inventory: &mut Inventory,
) -> Result<(), ForgeError> {
    enforce_depth(path_label, depth)?;
    for part in parts {
        if part.get("name").and_then(Value::as_str) == Some("statement") {
            insert_subject(path_label, part, SubjectType::Statement, inventory)?;
        } else if let (Some(id), Some(name)) =
            (part.get("id").and_then(Value::as_str), part.get("name").and_then(Value::as_str))
        {
            inventory.ineligible_parts.insert(id.to_string(), name.to_string());
        }
        if let Some(children) = part.get("parts").and_then(Value::as_array) {
            inventory_parts(path_label, children, depth + 1, inventory)?;
        }
    }
    Ok(())
}

fn insert_subject<'a>(
    path_label: &str,
    value: &'a Value,
    subject_type: SubjectType,
    inventory: &mut Inventory,
) -> Result<&'a str, ForgeError> {
    if inventory.ids.len() >= MAX_INVENTORY_SUBJECTS {
        return Err(mapping_error(format!(
            "{path_label} exceeds the {MAX_INVENTORY_SUBJECTS} subject inventory limit"
        )));
    }
    let id =
        value.get("id").and_then(Value::as_str).filter(|id| !id.trim().is_empty()).ok_or_else(
            || {
                mapping_error(format!(
                    "{path_label} contains an eligible {} without a non-empty id",
                    subject_type.as_str()
                ))
            },
        )?;
    if let Some(existing) = inventory.ids.insert(id.to_string(), subject_type) {
        let kind = if existing == subject_type { "duplicate" } else { "type-ambiguous" };
        return Err(mapping_error(format!(
            "{path_label} contains {kind} eligible id '{}'",
            bounded(id)
        )));
    }
    let fingerprint = canonical_subject_sha256(value)?;
    let excerpt = value
        .get(if subject_type == SubjectType::Control { "title" } else { "prose" })
        .and_then(Value::as_str)
        .unwrap_or("")
        .chars()
        .take(160)
        .collect();
    inventory.subjects.entry(subject_type).or_default().insert(id.to_string(), fingerprint);
    inventory.excerpts.entry(subject_type).or_default().insert(id.to_string(), excerpt);
    Ok(id)
}

fn canonical_subject_sha256(value: &Value) -> Result<String, ForgeError> {
    let mut canonical = value.clone();
    strip_forge_fingerprint_props(&mut canonical);
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| mapping_error(format!("subject fingerprint failed: {error}")))?;
    Ok(sha256(&bytes))
}

fn strip_forge_fingerprint_props(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if let Some(Value::Array(props)) = object.get_mut("props") {
                props.retain(|prop| {
                    !(prop.get("ns").and_then(Value::as_str) == Some(FORGE_MAPPING_NS)
                        && prop.get("name").and_then(Value::as_str) == Some("subject-sha256"))
                });
            }
            for child in object.values_mut() {
                strip_forge_fingerprint_props(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                strip_forge_fingerprint_props(child);
            }
        }
        _ => {}
    }
}

fn enforce_depth(path_label: &str, depth: usize) -> Result<(), ForgeError> {
    if depth > MAX_INVENTORY_DEPTH {
        Err(mapping_error(format!(
            "{path_label} exceeds maximum subject nesting depth {MAX_INVENTORY_DEPTH}"
        )))
    } else {
        Ok(())
    }
}

fn read_bounded_json(path: &Path, label: &str) -> Result<Vec<u8>, ForgeError> {
    io::check_file_size(path, io::MAX_FILE_SIZE)
        .map_err(|error| mapping_error(format!("{label}: {error}")))?;
    std::fs::read(path).map_err(|error| mapping_error(format!("{label}: {error}")))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

fn mapping_error(message: impl Into<String>) -> ForgeError {
    ForgeError::MappingBuild(message.into())
}
