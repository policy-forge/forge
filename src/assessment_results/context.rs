//! Confined local OSCAL context loading and exact subject inventories.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::manifest::{ArtifactManifest, ContextManifest, EvidenceIndexManifest, SubjectType};
use crate::json_strict::{self, Limits};
use crate::linkage::{EvidenceRecord, EvidenceReference, LinkageIndex};
use crate::validate::{self, OscalModelType};
use crate::{ForgeError, io};

const MAX_SCHEMA_ERRORS: usize = 100;
const MAX_INVENTORY_ITEMS: usize = 100_000;
const MAX_INVENTORY_DEPTH: usize = 64;
const MAX_CONTEXT_STRING_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub kind: &'static str,
    pub href: String,
    pub sha256: String,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
}

#[derive(Debug, Clone)]
pub struct LoadedContext {
    pub assessment_plan: ArtifactIdentity,
    pub ssp: ArtifactIdentity,
    pub profile: ArtifactIdentity,
    pub catalog: ArtifactIdentity,
    pub evidence_index_sha256: Option<String>,
    pub controls: BTreeSet<String>,
    pub statements: BTreeSet<String>,
    pub objectives: BTreeSet<String>,
    pub statement_controls: BTreeMap<String, String>,
    pub objective_controls: BTreeMap<String, String>,
    pub reviewed_controls: BTreeSet<String>,
    pub reviewed_objectives: BTreeSet<String>,
    pub tasks: BTreeSet<String>,
    pub implementation_statements: BTreeSet<String>,
    pub implementation_statement_controls: BTreeMap<String, String>,
    pub scoped_subjects: BTreeMap<SubjectType, BTreeSet<String>>,
    excluded_subjects: BTreeMap<SubjectType, BTreeSet<String>>,
    pub subjects: BTreeMap<SubjectType, BTreeSet<String>>,
    pub include_all_subject_types: BTreeSet<SubjectType>,
    pub evidence: BTreeMap<String, String>,
    pub input_paths: Vec<PathBuf>,
}

impl LoadedContext {
    #[must_use]
    pub fn artifact_identities(&self) -> [&ArtifactIdentity; 4] {
        [&self.assessment_plan, &self.ssp, &self.profile, &self.catalog]
    }

    #[must_use]
    pub fn subject_is_in_scope(&self, subject_type: SubjectType, uuid: &str) -> bool {
        self.subjects.get(&subject_type).is_some_and(|subjects| subjects.contains(uuid))
            && (self
                .scoped_subjects
                .get(&subject_type)
                .is_some_and(|subjects| subjects.contains(uuid))
                || (self.include_all_subject_types.contains(&subject_type)
                    && !self
                        .excluded_subjects
                        .get(&subject_type)
                        .is_some_and(|subjects| subjects.contains(uuid))))
    }
}

struct LoadedArtifact {
    identity: ArtifactIdentity,
    value: Value,
    path: PathBuf,
}

/// Load and cross-check the complete local Assessment Results context.
///
/// # Errors
///
/// Returns an error for unsafe paths, stale identities, invalid OSCAL artifacts,
/// inconsistent import chains, or unsupported/out-of-scope references.
pub fn load(manifest_path: &Path, context: &ContextManifest) -> Result<LoadedContext, ForgeError> {
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let root = manifest_dir.canonicalize().map_err(|cause| {
        error(format!("cannot resolve manifest directory '{}': {cause}", manifest_dir.display()))
    })?;
    load_from_root(&root, context)
}

/// Load and cross-check a context relative to an already resolved scaffold directory.
#[allow(
    clippy::too_many_lines,
    reason = "the context trust-boundary checks are intentionally kept in one auditable sequence"
)]
pub(crate) fn load_from_root(
    root: &Path,
    context: &ContextManifest,
) -> Result<LoadedContext, ForgeError> {
    let assessment_plan = load_artifact(
        root,
        "assessment-plan",
        &context.assessment_plan,
        OscalSchema::AssessmentPlan,
    )?;
    let ssp = load_artifact(root, "system-security-plan", &context.ssp, OscalSchema::Ssp)?;
    let profile = load_artifact(root, "profile", &context.profile, OscalSchema::Profile)?;
    let catalog = load_artifact(root, "catalog", &context.catalog, OscalSchema::Catalog)?;

    cross_check_import(
        &assessment_plan.value,
        &["assessment-plan", "import-ssp", "href"],
        &context.ssp.href,
        "Assessment Plan import-ssp",
    )?;
    cross_check_resolved_href(
        root,
        &assessment_plan.path,
        &context.ssp.href,
        &ssp.path,
        "Assessment Plan import-ssp",
    )?;
    cross_check_import(
        &ssp.value,
        &["system-security-plan", "import-profile", "href"],
        &context.profile.href,
        "SSP import-profile",
    )?;
    cross_check_resolved_href(
        root,
        &ssp.path,
        &context.profile.href,
        &profile.path,
        "SSP import-profile",
    )?;
    let (controls, statements, objectives, statement_controls, objective_controls) =
        inventory_catalog(&catalog.value)?;
    cross_check_resolved_href(
        root,
        &profile.path,
        &context.catalog.href,
        &catalog.path,
        "Profile Catalog import",
    )?;
    let profile_controls =
        resolve_profile_controls(&profile.value, &context.catalog.href, &controls)?;
    let profile_objectives: BTreeSet<_> = objective_controls
        .iter()
        .filter(|(_, control_id)| profile_controls.contains(*control_id))
        .map(|(objective_id, _)| objective_id.clone())
        .collect();
    let assessment_scope =
        inventory_assessment_plan(&assessment_plan.value, &profile_controls, &profile_objectives)?;
    let reviewed_controls = assessment_scope.reviewed_controls;
    let reviewed_objectives = assessment_scope.reviewed_objectives;
    for id in &reviewed_controls {
        if !controls.contains(id) || !profile_controls.contains(id) {
            return Err(error(format!(
                "Assessment Plan reviewed control '{}' is absent from the exact resolved Profile/Catalog scope",
                bounded(id)
            )));
        }
    }
    for id in &reviewed_objectives {
        if !objectives.contains(id) {
            return Err(error(format!(
                "Assessment Plan reviewed objective '{}' is absent from the exact Catalog companion",
                bounded(id)
            )));
        }
    }

    let (mut subjects, implementation_statement_controls) = inventory_ssp(&ssp.value)?;
    for (subject_type, referenced) in &assessment_scope.referenced_subjects {
        let available = subjects.entry(*subject_type).or_default();
        for subject_uuid in referenced {
            if !available.contains(subject_uuid) {
                return Err(error(format!(
                    "Assessment Plan subject {} '{}' is absent from the exact SSP companion",
                    subject_type.as_str(),
                    bounded(subject_uuid)
                )));
            }
        }
    }
    for control_id in implementation_statement_controls.values() {
        if !controls.contains(control_id) {
            return Err(error(format!(
                "SSP implementation control '{}' is absent from the exact Catalog companion",
                bounded(control_id)
            )));
        }
    }
    let implementation_statements = implementation_statement_controls.keys().cloned().collect();

    let (evidence_index_sha256, evidence, evidence_path) = context
        .evidence_index
        .as_ref()
        .map(|index| load_evidence_index(root, index))
        .transpose()?
        .map_or((None, BTreeMap::new(), None), |(hash, identities, path)| {
            (Some(hash), identities, Some(path))
        });

    let mut input_paths = vec![assessment_plan.path, ssp.path, profile.path, catalog.path];
    if let Some(path) = evidence_path {
        input_paths.push(path);
    }

    Ok(LoadedContext {
        assessment_plan: assessment_plan.identity,
        ssp: ssp.identity,
        profile: profile.identity,
        catalog: catalog.identity,
        evidence_index_sha256,
        controls,
        statements,
        objectives,
        statement_controls,
        objective_controls,
        reviewed_controls,
        reviewed_objectives,
        tasks: assessment_scope.tasks,
        implementation_statements,
        implementation_statement_controls,
        scoped_subjects: assessment_scope.explicit_subjects,
        excluded_subjects: assessment_scope.include_all_exclusions,
        subjects,
        include_all_subject_types: assessment_scope.include_all_subject_types,
        evidence,
        input_paths,
    })
}

#[derive(Debug, Clone, Copy)]
enum OscalSchema {
    AssessmentPlan,
    Ssp,
    Profile,
    Catalog,
}

impl OscalSchema {
    const fn root(self) -> &'static str {
        match self {
            Self::AssessmentPlan => "assessment-plan",
            Self::Ssp => "system-security-plan",
            Self::Profile => "profile",
            Self::Catalog => "catalog",
        }
    }
}

fn load_artifact(
    root: &Path,
    kind: &'static str,
    expected: &ArtifactManifest,
    schema: OscalSchema,
) -> Result<LoadedArtifact, ForgeError> {
    let path = resolve_confined_regular_file(root, &expected.artifact, kind)?;
    let bytes = io::read_bounded(&path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("cannot read {kind}: {cause}")))?;
    let sha256 = sha256(&bytes);
    if sha256 != expected.expected_sha256 {
        return Err(error(format!(
            "{kind} SHA-256 mismatch: expected {}, got {sha256}",
            expected.expected_sha256
        )));
    }
    let value = json_strict::parse_value(
        &bytes,
        kind,
        Limits { max_depth: 128, max_string_bytes: MAX_CONTEXT_STRING_BYTES },
    )
    .map_err(|cause| error(format!("{kind}: {cause}")))?;
    validate_schema(kind, &value, schema)?;
    let root_value = value.get(schema.root()).and_then(Value::as_object).ok_or_else(|| {
        error(format!("{kind} must contain exactly one '{}' root", schema.root()))
    })?;
    let root_uuid = required_string(root_value.get("uuid"), &format!("{kind}.uuid"))?;
    Uuid::parse_str(&root_uuid).map_err(|_| error(format!("{kind}.uuid must be a UUID")))?;
    let metadata = root_value
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| error(format!("{kind}.metadata is required")))?;
    let document_version =
        required_string(metadata.get("version"), &format!("{kind}.metadata.version"))?;
    let oscal_version =
        required_string(metadata.get("oscal-version"), &format!("{kind}.metadata.oscal-version"))?;
    if root_uuid != expected.root_uuid
        || document_version != expected.document_version
        || oscal_version != expected.oscal_version
    {
        return Err(error(format!(
            "{kind} identity mismatch: expected root UUID {}, document version '{}', OSCAL version '{}'; got {}, '{}', '{}'",
            expected.root_uuid,
            bounded(&expected.document_version),
            bounded(&expected.oscal_version),
            root_uuid,
            bounded(&document_version),
            bounded(&oscal_version)
        )));
    }
    if !matches!(oscal_version.as_str(), "1.2.0" | "1.2.1" | "1.2.2" | "1.2.3") {
        return Err(error(format!(
            "{kind} declares unsupported OSCAL version '{}'; the pinned compatibility baseline is 1.2.3",
            bounded(&oscal_version)
        )));
    }
    Ok(LoadedArtifact {
        identity: ArtifactIdentity {
            kind,
            href: expected.href.clone(),
            sha256,
            root_uuid,
            document_version,
            oscal_version,
        },
        value,
        path,
    })
}

fn validate_schema(kind: &str, value: &Value, schema: OscalSchema) -> Result<(), ForgeError> {
    let owned_validator;
    let validator = match schema {
        OscalSchema::Catalog => validate::compiled_validator(OscalModelType::Catalog)
            .map_err(|cause| error(format!("Catalog schema compilation failed: {cause}")))?,
        OscalSchema::Profile => validate::compiled_validator(OscalModelType::Profile)
            .map_err(|cause| error(format!("Profile schema compilation failed: {cause}")))?,
        OscalSchema::AssessmentPlan => {
            owned_validator =
                compile_schema(include_str!("../../schemas/oscal_assessment-plan_schema.json"))?;
            &owned_validator
        }
        OscalSchema::Ssp => {
            owned_validator = compile_schema(include_str!("../../schemas/oscal_ssp_schema.json"))?;
            &owned_validator
        }
    };
    let mut errors: Vec<_> = validator
        .iter_errors(value)
        .take(MAX_SCHEMA_ERRORS + 1)
        .map(|schema_error| schema_error.to_string())
        .collect();
    let truncated = errors.len() > MAX_SCHEMA_ERRORS;
    errors.truncate(MAX_SCHEMA_ERRORS);
    if errors.is_empty() {
        return Ok(());
    }
    let mut detail = errors.into_iter().take(10).collect::<Vec<_>>().join("; ");
    if truncated {
        detail.push_str("; additional schema errors omitted at configured bound");
    }
    Err(error(format!("{kind} is not valid against the pinned OSCAL 1.2.3 schema: {detail}")))
}

fn compile_schema(source: &str) -> Result<jsonschema::Validator, ForgeError> {
    let schema: Value = serde_json::from_str(source)
        .map_err(|cause| error(format!("vendored OSCAL schema is invalid JSON: {cause}")))?;
    jsonschema::validator_for(&schema)
        .map_err(|cause| error(format!("vendored OSCAL schema failed to compile: {cause}")))
}

fn cross_check_import(
    value: &Value,
    path: &[&str],
    expected_href: &str,
    label: &str,
) -> Result<(), ForgeError> {
    let mut current = value;
    for key in path {
        current = current.get(*key).ok_or_else(|| error(format!("{label} is missing '{key}'")))?;
    }
    let actual = current.as_str().ok_or_else(|| error(format!("{label} href must be a string")))?;
    if actual != expected_href {
        return Err(error(format!(
            "{label} href '{}' does not match the manifest companion href '{}'",
            bounded(actual),
            bounded(expected_href)
        )));
    }
    Ok(())
}

fn cross_check_resolved_href(
    root: &Path,
    importer: &Path,
    href: &str,
    target: &Path,
    label: &str,
) -> Result<(), ForgeError> {
    let importer_parent =
        importer.parent().ok_or_else(|| error(format!("{label} importer has no parent")))?;
    let resolved = importer_parent.join(href).canonicalize().map_err(|cause| {
        error(format!("{label} href '{}' cannot be resolved: {cause}", bounded(href)))
    })?;
    if !resolved.starts_with(root) || resolved != target {
        return Err(error(format!(
            "{label} href '{}' does not resolve to the exact declared companion artifact",
            bounded(href)
        )));
    }
    Ok(())
}

fn resolve_profile_controls(
    profile: &Value,
    catalog_href: &str,
    catalog_controls: &BTreeSet<String>,
) -> Result<BTreeSet<String>, ForgeError> {
    let imports = profile
        .pointer("/profile/imports")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Profile imports are required"))?;
    if imports.len() != 1 || imports[0].get("href").and_then(Value::as_str) != Some(catalog_href) {
        return Err(error(format!(
            "Assessment Results MVP requires exactly one Profile import for the manifest Catalog href '{}'",
            bounded(catalog_href)
        )));
    }
    let import = &imports[0];
    let mut selected = if import.get("include-all").is_some() {
        catalog_controls.clone()
    } else {
        BTreeSet::new()
    };
    let includes = import.get("include-controls").and_then(Value::as_array);
    if import.get("include-all").is_none() && includes.is_none() {
        return Err(error(
            "Profile import must explicitly select controls with include-all or include-controls",
        ));
    }
    apply_profile_selections(includes, catalog_controls, &mut selected, true)?;
    apply_profile_selections(
        import.get("exclude-controls").and_then(Value::as_array),
        catalog_controls,
        &mut selected,
        false,
    )?;
    Ok(selected)
}

fn apply_profile_selections(
    selections: Option<&Vec<Value>>,
    catalog_controls: &BTreeSet<String>,
    selected: &mut BTreeSet<String>,
    include: bool,
) -> Result<(), ForgeError> {
    for selection in selections.into_iter().flatten() {
        if selection.get("matching").is_some() {
            return Err(error(
                "Assessment Results MVP does not resolve Profile wildcard matching selections",
            ));
        }
        let ids = selection.get("with-ids").and_then(Value::as_array).ok_or_else(|| {
            error("Profile control selections must use explicit non-empty with-ids")
        })?;
        if ids.is_empty() {
            return Err(error("Profile control selection with-ids must not be empty"));
        }
        for id in ids {
            let id = required_string(Some(id), "Profile selected control ID")?;
            if !catalog_controls.contains(&id) {
                return Err(error(format!(
                    "Profile selects control '{}' absent from the exact Catalog companion",
                    bounded(&id)
                )));
            }
            if include {
                selected.insert(id);
            } else {
                selected.remove(&id);
            }
        }
    }
    Ok(())
}

type CatalogInventory = (
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeSet<String>,
    BTreeMap<String, String>,
    BTreeMap<String, String>,
);

fn inventory_catalog(value: &Value) -> Result<CatalogInventory, ForgeError> {
    let root =
        value.get("catalog").ok_or_else(|| error("Catalog root is required for inventory"))?;
    let mut controls = BTreeSet::new();
    let mut statements = BTreeSet::new();
    let mut objectives = BTreeSet::new();
    let mut statement_controls = BTreeMap::new();
    let mut objective_controls = BTreeMap::new();
    inventory_catalog_container(
        root,
        0,
        &mut controls,
        &mut statements,
        &mut objectives,
        &mut statement_controls,
        &mut objective_controls,
    )?;
    Ok((controls, statements, objectives, statement_controls, objective_controls))
}

fn inventory_catalog_container(
    value: &Value,
    depth: usize,
    controls: &mut BTreeSet<String>,
    statements: &mut BTreeSet<String>,
    objectives: &mut BTreeSet<String>,
    statement_controls: &mut BTreeMap<String, String>,
    objective_controls: &mut BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    if let Some(items) = value.get("controls").and_then(Value::as_array) {
        for control in items {
            let control_id = required_string(control.get("id"), "Catalog control")?;
            if !controls.insert(control_id.clone()) {
                return Err(error(format!(
                    "Catalog control '{}' is duplicated",
                    bounded(&control_id)
                )));
            }
            inventory_parts(
                control,
                depth + 1,
                &control_id,
                statements,
                objectives,
                statement_controls,
                objective_controls,
            )?;
            inventory_catalog_container(
                control,
                depth + 1,
                controls,
                statements,
                objectives,
                statement_controls,
                objective_controls,
            )?;
        }
    }
    if let Some(groups) = value.get("groups").and_then(Value::as_array) {
        for group in groups {
            inventory_catalog_container(
                group,
                depth + 1,
                controls,
                statements,
                objectives,
                statement_controls,
                objective_controls,
            )?;
        }
    }
    Ok(())
}

fn inventory_parts(
    value: &Value,
    depth: usize,
    control_id: &str,
    statements: &mut BTreeSet<String>,
    objectives: &mut BTreeSet<String>,
    statement_controls: &mut BTreeMap<String, String>,
    objective_controls: &mut BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    if let Some(parts) = value.get("parts").and_then(Value::as_array) {
        for part in parts {
            match part.get("name").and_then(Value::as_str) {
                Some("statement") => insert_owned_id(
                    part,
                    "Catalog statement",
                    control_id,
                    statements,
                    statement_controls,
                )?,
                Some("objective") => insert_owned_id(
                    part,
                    "Catalog objective",
                    control_id,
                    objectives,
                    objective_controls,
                )?,
                _ => {}
            }
            inventory_parts(
                part,
                depth + 1,
                control_id,
                statements,
                objectives,
                statement_controls,
                objective_controls,
            )?;
        }
    }
    Ok(())
}

fn insert_owned_id(
    value: &Value,
    label: &str,
    control_id: &str,
    inventory: &mut BTreeSet<String>,
    owners: &mut BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    let id = required_string(value.get("id"), label)?;
    if !inventory.insert(id.clone()) {
        return Err(error(format!("{label} '{}' is duplicated", bounded(&id))));
    }
    owners.insert(id, control_id.to_string());
    Ok(())
}

struct AssessmentPlanInventory {
    reviewed_controls: BTreeSet<String>,
    reviewed_objectives: BTreeSet<String>,
    tasks: BTreeSet<String>,
    explicit_subjects: BTreeMap<SubjectType, BTreeSet<String>>,
    include_all_subject_types: BTreeSet<SubjectType>,
    include_all_exclusions: BTreeMap<SubjectType, BTreeSet<String>>,
    referenced_subjects: BTreeMap<SubjectType, BTreeSet<String>>,
}

fn inventory_assessment_plan(
    value: &Value,
    eligible_controls: &BTreeSet<String>,
    eligible_objectives: &BTreeSet<String>,
) -> Result<AssessmentPlanInventory, ForgeError> {
    let root = value
        .get("assessment-plan")
        .ok_or_else(|| error("Assessment Plan root is required for inventory"))?;
    let reviewed = root
        .get("reviewed-controls")
        .ok_or_else(|| error("Assessment Plan reviewed-controls are required"))?;
    let mut controls = BTreeSet::new();
    let selections = reviewed
        .get("control-selections")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Assessment Plan control-selections are required"))?;
    for selection in selections {
        resolve_selection(
            selection,
            "include-controls",
            "exclude-controls",
            "control-id",
            "Assessment Plan reviewed control",
            eligible_controls,
            &mut controls,
        )?;
    }
    if controls.is_empty() {
        return Err(error("Assessment Plan effective reviewed control scope must not be empty"));
    }
    let mut objectives = BTreeSet::new();
    if let Some(selections) = reviewed.get("control-objective-selections").and_then(Value::as_array)
    {
        for selection in selections {
            resolve_selection(
                selection,
                "include-objectives",
                "exclude-objectives",
                "objective-id",
                "Assessment Plan objective",
                eligible_objectives,
                &mut objectives,
            )?;
        }
    }
    let mut tasks = BTreeSet::new();
    if let Some(items) = root.get("tasks").and_then(Value::as_array) {
        inventory_tasks(items, 0, &mut tasks)?;
    }
    let mut explicit_subjects = BTreeMap::new();
    let mut include_all = BTreeSet::new();
    let mut include_all_exclusions: BTreeMap<SubjectType, BTreeSet<String>> = BTreeMap::new();
    let mut referenced_subjects: BTreeMap<SubjectType, BTreeSet<String>> = BTreeMap::new();
    let subject_groups = root
        .get("assessment-subjects")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Assessment Plan assessment-subjects are required"))?;
    for group in subject_groups {
        let subject_type =
            parse_subject_type(group.get("type").and_then(Value::as_str).ok_or_else(|| {
                error("Assessment Plan assessment-subject type must be a string")
            })?)?;
        let included = subject_references(group.get("include-subjects"), subject_type)?;
        let excluded = subject_references(group.get("exclude-subjects"), subject_type)?;
        referenced_subjects
            .entry(subject_type)
            .or_default()
            .extend(included.iter().chain(&excluded).cloned());
        if group.get("include-all").is_some() {
            if include_all.insert(subject_type) {
                include_all_exclusions.insert(subject_type, excluded);
            } else if let Some(existing) = include_all_exclusions.get_mut(&subject_type) {
                existing.retain(|uuid| excluded.contains(uuid));
            }
        } else {
            let selected = included.difference(&excluded).cloned();
            let effective = explicit_subjects.entry(subject_type).or_insert_with(BTreeSet::new);
            effective.extend(selected);
        }
    }
    Ok(AssessmentPlanInventory {
        reviewed_controls: controls,
        reviewed_objectives: objectives,
        tasks,
        explicit_subjects,
        include_all_subject_types: include_all,
        include_all_exclusions,
        referenced_subjects,
    })
}

#[allow(clippy::too_many_arguments)]
fn resolve_selection(
    selection: &Value,
    include_field: &str,
    exclude_field: &str,
    id_field: &str,
    label: &str,
    eligible: &BTreeSet<String>,
    inventory: &mut BTreeSet<String>,
) -> Result<(), ForgeError> {
    let mut selected = if selection.get("include-all").is_some() {
        eligible.clone()
    } else {
        let included = selection.get(include_field).and_then(Value::as_array).ok_or_else(|| {
            error(format!("{label} selection requires include-all or {include_field}"))
        })?;
        let mut selected = BTreeSet::new();
        for item in included {
            let id = required_string(item.get(id_field), label)?;
            if !eligible.contains(&id) {
                return Err(error(format!(
                    "{label} '{}' is absent from the exact resolved Profile/Catalog scope",
                    bounded(&id)
                )));
            }
            if !selected.insert(id.clone()) {
                return Err(error(format!("{label} '{}' is duplicated", bounded(&id))));
            }
        }
        selected
    };
    if let Some(excluded) = selection.get(exclude_field).and_then(Value::as_array) {
        for item in excluded {
            let id = required_string(item.get(id_field), label)?;
            if !eligible.contains(&id) {
                return Err(error(format!(
                    "{label} exclusion '{}' is absent from the exact resolved Profile/Catalog scope",
                    bounded(&id)
                )));
            }
            selected.remove(&id);
        }
    }
    inventory.extend(selected);
    Ok(())
}

fn inventory_tasks(
    items: &[Value],
    depth: usize,
    tasks: &mut BTreeSet<String>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    for task in items {
        insert_id(task, "uuid", "Assessment Plan task", tasks)?;
        if let Some(children) = task.get("tasks").and_then(Value::as_array) {
            inventory_tasks(children, depth + 1, tasks)?;
        }
    }
    Ok(())
}

fn subject_references(
    value: Option<&Value>,
    expected_type: SubjectType,
) -> Result<BTreeSet<String>, ForgeError> {
    let mut subjects = BTreeSet::new();
    for item in value.and_then(Value::as_array).into_iter().flatten() {
        let item_type = parse_subject_type(
            item.get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| error("Assessment Plan subject reference type is required"))?,
        )?;
        if item_type != expected_type {
            return Err(error(
                "Assessment Plan subject reference type does not match its selection group",
            ));
        }
        let subject_uuid =
            required_string(item.get("subject-uuid"), "Assessment Plan subject-uuid")?;
        Uuid::parse_str(&subject_uuid)
            .map_err(|_| error("Assessment Plan subject-uuid must be a UUID"))?;
        if !subjects.insert(subject_uuid) {
            return Err(error("Assessment Plan contains a duplicate subject reference"));
        }
    }
    Ok(subjects)
}

type SspInventory = (BTreeMap<SubjectType, BTreeSet<String>>, BTreeMap<String, String>);

fn inventory_ssp(value: &Value) -> Result<SspInventory, ForgeError> {
    let root = value
        .get("system-security-plan")
        .ok_or_else(|| error("SSP root is required for inventory"))?;
    let mut subjects: BTreeMap<SubjectType, BTreeSet<String>> = BTreeMap::new();
    let paths = [
        ("/system-security-plan/system-implementation/components", SubjectType::Component),
        ("/system-security-plan/system-implementation/inventory-items", SubjectType::InventoryItem),
        ("/system-security-plan/system-implementation/users", SubjectType::User),
        ("/system-security-plan/metadata/locations", SubjectType::Location),
        ("/system-security-plan/metadata/parties", SubjectType::Party),
        ("/system-security-plan/back-matter/resources", SubjectType::Resource),
    ];
    for (pointer, subject_type) in paths {
        if let Some(items) = value.pointer(pointer).and_then(Value::as_array) {
            for item in items {
                let subject_uuid = required_string(item.get("uuid"), "SSP subject UUID")?;
                Uuid::parse_str(&subject_uuid)
                    .map_err(|_| error("SSP subject UUID must be a UUID"))?;
                let inserted =
                    subjects.entry(subject_type).or_default().insert(subject_uuid.clone());
                if !inserted {
                    return Err(error(format!(
                        "SSP contains duplicate {} UUID '{}'",
                        subject_type.as_str(),
                        bounded(&subject_uuid)
                    )));
                }
            }
        }
    }
    let mut implementations = BTreeMap::new();
    if let Some(items) =
        root.pointer("/control-implementation/implemented-requirements").and_then(Value::as_array)
    {
        for item in items {
            let control_id = required_string(item.get("control-id"), "SSP implementation control")?;
            insert_owned_uuid(
                item,
                "SSP implemented requirement",
                &control_id,
                &mut implementations,
            )?;
            if let Some(by_components) = item.get("by-components").and_then(Value::as_array) {
                for entry in by_components {
                    insert_owned_uuid(
                        entry,
                        "SSP by-component implementation",
                        &control_id,
                        &mut implementations,
                    )?;
                }
            }
        }
    }
    Ok((subjects, implementations))
}

fn insert_owned_uuid(
    value: &Value,
    label: &str,
    control_id: &str,
    inventory: &mut BTreeMap<String, String>,
) -> Result<(), ForgeError> {
    let id = required_string(value.get("uuid"), label)?;
    Uuid::parse_str(&id).map_err(|_| error(format!("{label} must be a UUID")))?;
    if inventory.insert(id.clone(), control_id.to_string()).is_some() {
        return Err(error(format!("{label} '{}' is duplicated", bounded(&id))));
    }
    Ok(())
}

fn load_evidence_index(
    root: &Path,
    manifest: &EvidenceIndexManifest,
) -> Result<(String, BTreeMap<String, String>, PathBuf), ForgeError> {
    let path = resolve_confined_regular_file(root, &manifest.artifact, "evidence index")?;
    let bytes = io::read_bounded(&path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("cannot read evidence index: {cause}")))?;
    let hash = sha256(&bytes);
    if hash != manifest.expected_sha256 {
        return Err(error(format!(
            "evidence index SHA-256 mismatch: expected {}, got {hash}",
            manifest.expected_sha256
        )));
    }
    let value = json_strict::parse_value(
        &bytes,
        "PRD 060 evidence index",
        Limits { max_depth: 64, max_string_bytes: MAX_CONTEXT_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))?;
    if value.get("schema_version").and_then(Value::as_str) != Some("forge.linkage-index/1") {
        return Err(error("evidence index must declare schema_version 'forge.linkage-index/1'"));
    }
    let index: LinkageIndex = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid PRD 060 evidence index: {cause}")))?;
    let identities = collect_evidence_identities(&index.evidence)?;
    Ok((hash, identities, path))
}

fn collect_evidence_identities(
    records: &[EvidenceRecord],
) -> Result<BTreeMap<String, String>, ForgeError> {
    if records.len() > MAX_INVENTORY_ITEMS {
        return Err(error("evidence index exceeds the identity inventory limit"));
    }
    let mut identities = BTreeMap::new();
    let mut keys = BTreeSet::new();
    for record in records {
        if record.key.trim().is_empty() {
            return Err(error("evidence index contains an empty evidence key"));
        }
        if !keys.insert(record.key.as_str()) {
            return Err(error(format!(
                "evidence index key '{}' is duplicated",
                bounded(&record.key)
            )));
        }
        let hash = match &record.reference {
            EvidenceReference::Local { approved_sha256, observed_sha256, .. } => {
                observed_sha256.as_ref().unwrap_or(approved_sha256)
            }
            EvidenceReference::Uri { expected_sha256: Some(hash), .. } => hash,
            EvidenceReference::Uri { expected_sha256: None, .. } => continue,
        };
        json_strict::validate_lowercase_sha256("evidence.reference.sha256", hash).map_err(error)?;
        identities.insert(record.key.clone(), hash.clone());
    }
    Ok(identities)
}

fn resolve_confined_regular_file(
    root: &Path,
    relative: &Path,
    label: &str,
) -> Result<PathBuf, ForgeError> {
    if relative.is_absolute()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(error(format!("{label} path must be a confined relative path")));
    }
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(error(format!("{label} path contains an unsafe component")));
        };
        current.push(name);
        let metadata = std::fs::symlink_metadata(&current).map_err(|cause| {
            error(format!("cannot inspect {label} '{}': {cause}", relative.display()))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(error(format!("{label} path must not traverse a symbolic link")));
        }
    }
    let canonical = current.canonicalize().map_err(|cause| {
        error(format!("cannot resolve {label} '{}': {cause}", relative.display()))
    })?;
    if !canonical.starts_with(root) {
        return Err(error(format!("{label} path escapes the manifest directory")));
    }
    let metadata = std::fs::symlink_metadata(&canonical)
        .map_err(|cause| error(format!("cannot inspect {label}: {cause}")))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(error(format!("{label} must be a regular non-symlink file")));
    }
    Ok(canonical)
}

fn insert_id(
    value: &Value,
    key: &str,
    label: &str,
    inventory: &mut BTreeSet<String>,
) -> Result<(), ForgeError> {
    if inventory.len() >= MAX_INVENTORY_ITEMS {
        return Err(error(format!("{label} inventory exceeds {MAX_INVENTORY_ITEMS} entries")));
    }
    let id = required_string(value.get(key), label)?;
    if !inventory.insert(id.clone()) {
        return Err(error(format!("{label} '{}' is duplicated", bounded(&id))));
    }
    Ok(())
}

fn required_string(value: Option<&Value>, path: &str) -> Result<String, ForgeError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| error(format!("{path} must be a non-empty string")))
}

fn parse_subject_type(value: &str) -> Result<SubjectType, ForgeError> {
    match value {
        "component" => Ok(SubjectType::Component),
        "inventory-item" => Ok(SubjectType::InventoryItem),
        "location" => Ok(SubjectType::Location),
        "party" => Ok(SubjectType::Party),
        "user" => Ok(SubjectType::User),
        "resource" => Ok(SubjectType::Resource),
        other => Err(error(format!("unsupported assessment subject type '{}'", bounded(other)))),
    }
}

fn enforce_depth(depth: usize) -> Result<(), ForgeError> {
    if depth > MAX_INVENTORY_DEPTH {
        Err(error(format!("context inventory exceeds nesting depth {MAX_INVENTORY_DEPTH}")))
    } else {
        Ok(())
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bounded(value: &str) -> String {
    json_strict::bounded(value)
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::AssessmentResultsBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidence_identity_collection_rejects_duplicate_keys() {
        let records: Vec<EvidenceRecord> = serde_json::from_value(serde_json::json!([
            evidence_record("e-1", &"a".repeat(64)),
            evidence_record("e-1", &"b".repeat(64))
        ]))
        .expect("records");
        let error = collect_evidence_identities(&records)
            .expect_err("duplicate evidence identity must fail");
        assert!(error.to_string().contains("is duplicated"));
    }

    #[test]
    fn evidence_identity_uses_observed_local_hash_and_verified_uri_hash() {
        let records: Vec<EvidenceRecord> = serde_json::from_value(serde_json::json!([
            evidence_record("local", &"a".repeat(64)),
            {
                "key": "remote",
                "title": "Remote evidence",
                "evidence_type": "ticket",
                "owner": "owner",
                "collected_at": "2026-08-20T00:00:00Z",
                "sensitivity_label": "restricted",
                "source_label": "reviewed reference",
                "freshness": "unverified-uri",
                "reference": {
                    "kind": "uri",
                    "redacted_uri": "https://example.invalid/ticket/1",
                    "expected_sha256": "c".repeat(64)
                }
            },
            {
                "key": "unhashed-remote",
                "title": "Unhashed remote evidence",
                "evidence_type": "ticket",
                "owner": "owner",
                "collected_at": "2026-08-20T00:00:00Z",
                "sensitivity_label": "restricted",
                "source_label": "reviewed reference",
                "freshness": "unverified-uri",
                "reference": {
                    "kind": "uri",
                    "redacted_uri": "https://example.invalid/ticket/2"
                }
            }
        ]))
        .expect("records");
        let identities = collect_evidence_identities(&records).expect("identities");
        assert_eq!(identities["local"], "b".repeat(64));
        assert_eq!(identities["remote"], "c".repeat(64));
        assert!(!identities.contains_key("unhashed-remote"));
    }

    fn evidence_record(key: &str, approved_sha256: &str) -> Value {
        serde_json::json!({
            "key": key,
            "title": "Local evidence",
            "evidence_type": "artifact",
            "owner": "owner",
            "collected_at": "2026-08-20T00:00:00Z",
            "sensitivity_label": "restricted",
            "source_label": "reviewed artifact",
            "freshness": "changed",
            "reference": {
                "kind": "local",
                "root_key": "local",
                "relative_label": "record.bin",
                "approved_sha256": approved_sha256,
                "approved_size": 1,
                "observed_sha256": "b".repeat(64),
                "observed_size": 1
            }
        })
    }

    #[cfg(unix)]
    #[test]
    fn confined_loader_rejects_parent_symlink_escape() {
        let root = tempfile::tempdir().expect("root");
        let outside = tempfile::tempdir().expect("outside");
        std::fs::write(outside.path().join("artifact.json"), "{}").expect("fixture");
        std::os::unix::fs::symlink(outside.path(), root.path().join("linked"))
            .expect("parent symlink");
        let error = resolve_confined_regular_file(
            root.path(),
            Path::new("linked/artifact.json"),
            "fixture",
        )
        .expect_err("parent symlink must fail");
        assert!(error.to_string().contains("symbolic link"));
    }
}
