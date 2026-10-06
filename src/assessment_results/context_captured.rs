//! Borrowed, sealed-original OSCAL context loading for read-only evidence inspection.
//!
//! Legacy loaders are unchanged. This path shares the original/relationship ledger
//! and adds a conservative 10 MiB escaped context-metadata admission ceiling.
//! Parsed native Values remain subject to their existing separate input limits.

use super::{
    ArtifactIdentity, ArtifactManifest, AssessmentPlanInventory, BTreeMap, BTreeSet, Component,
    ContextManifest, ForgeError, Limits, LoadedArtifact, LoadedContext, MAX_CONTEXT_STRING_BYTES,
    MAX_INVENTORY_ITEMS, OscalSchema, Path, PathBuf, Serialize, SspInventory, SubjectType, Uuid,
    Value, cross_check_import, enforce_depth, error, io, json_strict, parse_subject_type,
    sha256_hex, validate_schema,
};
use crate::evidence_capture::{CaptureRole, CaptureSession, ProjectionBudget};

/// Admit native context relationships and copied metadata before their collections grow.
struct ContextBudget<'a> {
    /// Complete preparation ledger borrowed from both inspection lanes.
    capture: &'a mut CaptureSession,
    /// Conservative local metadata ceiling; it is not another raw-byte allowance.
    projection: ProjectionBudget,
}

impl ContextBudget<'_> {
    /// Charge even repeated, filtered or subsequently rejected borrowed relationship facts.
    fn row<T: Serialize>(&mut self, row: &T) -> Result<(), ForgeError> {
        self.capture.relationships(1)?;
        self.projection.admit_row(row)
    }
}

/// Inventory Catalog collections without changing their exact legacy native semantics.
#[derive(Default)]
struct CapturedCatalog {
    /// Every declared control ID, retained once after duplicate refusal.
    controls: BTreeSet<String>,
    /// Every supported statement part ID.
    statements: BTreeSet<String>,
    /// Every supported objective part ID.
    objectives: BTreeSet<String>,
    /// Exact statement-to-control relations.
    statement_controls: BTreeMap<String, String>,
    /// Exact objective-to-control relations.
    objective_controls: BTreeMap<String, String>,
}

/// Load the four exact native companions using already held originals and no filesystem IO.
///
/// The supplied slices must be the actual sealed session allocations. The caller
/// retains and rechecks the complete shared proof after analysis and before publication.
/// # Errors
///
/// Returns an error for detached slices, missing roles, invalid native schemas,
/// inconsistent imports, scope defects or either shared/metadata admission bound.
pub(super) fn load(
    context: &ContextManifest,
    captured: &BTreeMap<PathBuf, &[u8]>,
    capture: &mut CaptureSession,
) -> Result<LoadedContext, ForgeError> {
    super::super::manifest::validate_context(context)?;
    if context.evidence_index.is_some() {
        return Err(error("captured POA&M context must not substitute an evidence index"));
    }
    let mut budget = ContextBudget { capture, projection: ProjectionBudget::new() };
    let assessment_plan = artifact(
        &context.assessment_plan,
        OscalSchema::AssessmentPlan,
        CaptureRole::AssessmentPlan,
        captured,
        &mut budget,
    )?;
    let ssp = artifact(
        &context.ssp,
        OscalSchema::Ssp,
        CaptureRole::SystemSecurityPlan,
        captured,
        &mut budget,
    )?;
    let profile = artifact(
        &context.profile,
        OscalSchema::Profile,
        CaptureRole::Profile,
        captured,
        &mut budget,
    )?;
    let catalog = artifact(
        &context.catalog,
        OscalSchema::Catalog,
        CaptureRole::Catalog,
        captured,
        &mut budget,
    )?;
    check_imports(context, &assessment_plan, &ssp, &mut budget)?;
    assemble(assessment_plan, ssp, profile, catalog, context, &mut budget)
}

/// Parse one sealed borrowed generation with the legacy schema, hash and identity predicates.
fn artifact(
    expected: &ArtifactManifest,
    schema: OscalSchema,
    role: CaptureRole,
    captured: &BTreeMap<PathBuf, &[u8]>,
    budget: &mut ContextBudget<'_>,
) -> Result<LoadedArtifact, ForgeError> {
    let supplied = captured
        .get(&expected.artifact)
        .ok_or_else(|| error("captured context companion is missing"))?;
    let bytes = budget.capture.captured(&expected.artifact, role)?;
    if bytes.len() != supplied.len() || bytes.as_ptr() != supplied.as_ptr() {
        return Err(error("context slice is not the exact held original allocation"));
    }
    if u64::try_from(bytes.len()).map_err(|_| error("context length conversion failed"))?
        > io::MAX_FILE_SIZE
    {
        return Err(error("captured context companion exceeds the native file bound"));
    }
    let kind = schema.root();
    let sha256 = sha256_hex(bytes);
    if sha256 != expected.expected_sha256 {
        return Err(error(format!("{kind} SHA-256 mismatch")));
    }
    let value = json_strict::parse_value(
        bytes,
        kind,
        Limits { max_depth: 128, max_string_bytes: MAX_CONTEXT_STRING_BYTES },
    )
    .map_err(|cause| error(format!("{kind}: {cause}")))?;
    validate_schema(kind, &value, schema)?;
    let root_value = value
        .get(kind)
        .and_then(Value::as_object)
        .ok_or_else(|| error(format!("{kind} must contain its exact native root")))?;
    let root_uuid = string_ref(root_value.get("uuid"), "native root UUID")?;
    Uuid::parse_str(root_uuid).map_err(|_| error("native root UUID is invalid"))?;
    let metadata = root_value
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| error("native metadata is required"))?;
    let document_version = string_ref(metadata.get("version"), "native document version")?;
    let oscal_version = string_ref(metadata.get("oscal-version"), "native OSCAL version")?;
    if root_uuid != expected.root_uuid
        || document_version != expected.document_version
        || oscal_version != expected.oscal_version
    {
        return Err(error("captured native companion identity mismatch"));
    }
    if !matches!(oscal_version, "1.2.0" | "1.2.1" | "1.2.2" | "1.2.3") {
        return Err(error("captured native companion uses an unsupported OSCAL version"));
    }
    budget.row(&(kind, &expected.href, &sha256, root_uuid, document_version, oscal_version))?;
    let identity = ArtifactIdentity {
        kind,
        href: expected.href.clone(),
        sha256,
        root_uuid: root_uuid.to_owned(),
        document_version: document_version.to_owned(),
        oscal_version: oscal_version.to_owned(),
    };
    // The absolute path is a private join of actual qualified root and held relative path.
    let path = budget.capture.root().join(&expected.artifact);
    Ok(LoadedArtifact { identity, value, path })
}

/// Verify exact declared imports and importer-relative destinations without reopening a path.
fn check_imports(
    context: &ContextManifest,
    assessment_plan: &LoadedArtifact,
    ssp: &LoadedArtifact,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    budget.row(&(
        "assessment-plan-import",
        assessment_plan.value.pointer("/assessment-plan/import-ssp/href"),
        &context.ssp.href,
    ))?;
    cross_check_import(
        &assessment_plan.value,
        &["assessment-plan", "import-ssp", "href"],
        &context.ssp.href,
        "Assessment Plan import-ssp",
    )?;
    relative_import(&context.assessment_plan.artifact, &context.ssp.href, &context.ssp.artifact)?;
    budget.row(&(
        "ssp-import",
        ssp.value.pointer("/system-security-plan/import-profile/href"),
        &context.profile.href,
    ))?;
    cross_check_import(
        &ssp.value,
        &["system-security-plan", "import-profile", "href"],
        &context.profile.href,
        "SSP import-profile",
    )?;
    relative_import(&context.ssp.artifact, &context.profile.href, &context.profile.artifact)?;
    relative_import(&context.profile.artifact, &context.catalog.href, &context.catalog.artifact)?;
    // Profile's exact one-import predicate is consumed by profile_controls below.
    Ok(())
}

/// Join only normal or dot importer-relative components to an already qualified exact target.
fn relative_import(importer: &Path, href: &str, target: &Path) -> Result<(), ForgeError> {
    let parent = importer.parent().ok_or_else(|| error("context importer has no parent"))?;
    let joined = parent.join(href);
    let mut normalized = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {}
            _ => return Err(error("context import is not a confined relative href")),
        }
    }
    if normalized != target {
        return Err(error("context import does not identify its exact declared companion"));
    }
    Ok(())
}

/// Build the same native `LoadedContext` while accounting all intermediate relationship copies.
fn assemble(
    assessment_plan: LoadedArtifact,
    ssp: LoadedArtifact,
    profile: LoadedArtifact,
    catalog: LoadedArtifact,
    context: &ContextManifest,
    budget: &mut ContextBudget<'_>,
) -> Result<LoadedContext, ForgeError> {
    let mut inventory = CapturedCatalog::default();
    let catalog_root = catalog
        .value
        .get("catalog")
        .ok_or_else(|| error("Catalog root is required for inventory"))?;
    catalog_container(catalog_root, 0, &mut inventory, budget)?;
    let selected =
        profile_controls(&profile.value, &context.catalog.href, &inventory.controls, budget)?;
    let mut selected_objectives = BTreeSet::new();
    for (objective, control) in &inventory.objective_controls {
        budget.row(&(objective, control))?;
        if selected.contains(control) {
            insert(&mut selected_objectives, objective, "profile objective", budget)?;
        }
    }
    let scope = plan_inventory(&assessment_plan.value, &selected, &selected_objectives, budget)?;
    for id in &scope.reviewed_controls {
        budget.row(id)?;
        if !inventory.controls.contains(id) || !selected.contains(id) {
            return Err(error("Assessment Plan reviewed control is outside exact Profile/Catalog"));
        }
    }
    for id in &scope.reviewed_objectives {
        budget.row(id)?;
        if !inventory.objectives.contains(id) {
            return Err(error("Assessment Plan objective is absent from exact Catalog"));
        }
    }
    let (mut subjects, implementations) = ssp_inventory(&ssp.value, budget)?;
    for (kind, references) in &scope.referenced_subjects {
        budget.row(&("subject-scope", kind.as_str()))?;
        let available = subjects.entry(*kind).or_default();
        for id in references {
            budget.row(&(kind.as_str(), id))?;
            if !available.contains(id) {
                return Err(error("Assessment Plan subject is absent from exact SSP"));
            }
        }
    }
    let mut implementation_statements = BTreeSet::new();
    for (id, control) in &implementations {
        budget.row(&(id, control))?;
        if !inventory.controls.contains(control) {
            return Err(error("SSP implementation control is absent from exact Catalog"));
        }
        insert(&mut implementation_statements, id, "implementation statement", budget)?;
    }
    budget.row(&(
        "input-paths",
        &context.assessment_plan.artifact,
        &context.ssp.artifact,
        &context.profile.artifact,
        &context.catalog.artifact,
    ))?;
    Ok(LoadedContext {
        assessment_plan: assessment_plan.identity,
        ssp: ssp.identity,
        profile: profile.identity,
        catalog: catalog.identity,
        evidence_index_sha256: None,
        controls: inventory.controls,
        statements: inventory.statements,
        objectives: inventory.objectives,
        statement_controls: inventory.statement_controls,
        objective_controls: inventory.objective_controls,
        reviewed_controls: scope.reviewed_controls,
        reviewed_objectives: scope.reviewed_objectives,
        tasks: scope.tasks,
        implementation_statements,
        implementation_statement_controls: implementations,
        scoped_subjects: scope.explicit_subjects,
        excluded_subjects: scope.include_all_exclusions,
        subjects,
        include_all_subject_types: scope.include_all_subject_types,
        evidence: BTreeMap::new(),
        input_paths: vec![assessment_plan.path, ssp.path, profile.path, catalog.path],
    })
}

/// Traverse the legacy Catalog group/control structure, charging each declaration even if invalid.
fn catalog_container(
    value: &Value,
    depth: usize,
    inventory: &mut CapturedCatalog,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    if let Some(controls) = value.get("controls").and_then(Value::as_array) {
        for control in controls {
            let id = string_ref(control.get("id"), "Catalog control")?;
            insert(&mut inventory.controls, id, "Catalog control", budget)?;
            catalog_parts(control, depth + 1, id, inventory, budget)?;
            catalog_container(control, depth + 1, inventory, budget)?;
        }
    }
    if let Some(groups) = value.get("groups").and_then(Value::as_array) {
        for group in groups {
            budget.row(&("catalog-group", depth))?;
            catalog_container(group, depth + 1, inventory, budget)?;
        }
    }
    Ok(())
}

/// Retain only statement/objective parts while charging every repeated supported ID before growth.
fn catalog_parts(
    value: &Value,
    depth: usize,
    control: &str,
    inventory: &mut CapturedCatalog,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    if let Some(parts) = value.get("parts").and_then(Value::as_array) {
        for part in parts {
            match part.get("name").and_then(Value::as_str) {
                Some("statement") => part_identity(
                    part,
                    control,
                    &mut inventory.statements,
                    &mut inventory.statement_controls,
                    budget,
                )?,
                Some("objective") => part_identity(
                    part,
                    control,
                    &mut inventory.objectives,
                    &mut inventory.objective_controls,
                    budget,
                )?,
                _ => budget.row(&("filtered-catalog-part", depth))?,
            }
            catalog_parts(part, depth + 1, control, inventory, budget)?;
        }
    }
    Ok(())
}

/// Charge both a supported part's set copy and owner mapping before either insertion.
fn part_identity(
    value: &Value,
    control: &str,
    ids: &mut BTreeSet<String>,
    owners: &mut BTreeMap<String, String>,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    let id = string_ref(value.get("id"), "Catalog part")?;
    insert(ids, id, "Catalog part", budget)?;
    budget.row(&(id, control))?;
    owners.insert(id.to_owned(), control.to_owned());
    Ok(())
}

/// Resolve the same one-import explicit Profile scope with every include/exclude counted.
fn profile_controls(
    profile: &Value,
    catalog_href: &str,
    controls: &BTreeSet<String>,
    budget: &mut ContextBudget<'_>,
) -> Result<BTreeSet<String>, ForgeError> {
    let imports = profile
        .pointer("/profile/imports")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Profile imports are required"))?;
    if imports.len() != 1 || imports[0].get("href").and_then(Value::as_str) != Some(catalog_href) {
        return Err(error("captured context requires exactly one matching Profile import"));
    }
    budget.row(&catalog_href)?;
    let import = &imports[0];
    let mut selected = if import.get("include-all").is_some() {
        copy_set(controls, budget)?
    } else {
        BTreeSet::new()
    };
    let includes = import.get("include-controls").and_then(Value::as_array);
    if import.get("include-all").is_none() && includes.is_none() {
        return Err(error("Profile import must explicitly select controls"));
    }
    profile_selections(includes, controls, &mut selected, true, budget)?;
    profile_selections(
        import.get("exclude-controls").and_then(Value::as_array),
        controls,
        &mut selected,
        false,
        budget,
    )?;
    Ok(selected)
}

/// Charge every explicit Profile selection before filtering or retaining its borrowed ID.
fn profile_selections(
    selections: Option<&Vec<Value>>,
    eligible: &BTreeSet<String>,
    selected: &mut BTreeSet<String>,
    include: bool,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    for selection in selections.into_iter().flatten() {
        budget.row(&("profile-selection", include))?;
        if selection.get("matching").is_some() {
            return Err(error("captured context does not resolve Profile wildcard matching"));
        }
        let ids = selection
            .get("with-ids")
            .and_then(Value::as_array)
            .filter(|ids| !ids.is_empty())
            .ok_or_else(|| error("Profile selections require non-empty explicit with-ids"))?;
        for value in ids {
            let id = string_ref(Some(value), "Profile selected control ID")?;
            budget.row(&id)?;
            if !eligible.contains(id) {
                return Err(error("Profile selects a control absent from exact Catalog"));
            }
            if include {
                selected.insert(id.to_owned());
            } else {
                selected.remove(id);
            }
        }
    }
    Ok(())
}

/// Resolve the same native Assessment Plan scope without allocating uncharged clone sets.
fn plan_inventory(
    value: &Value,
    eligible_controls: &BTreeSet<String>,
    eligible_objectives: &BTreeSet<String>,
    budget: &mut ContextBudget<'_>,
) -> Result<AssessmentPlanInventory, ForgeError> {
    let root =
        value.get("assessment-plan").ok_or_else(|| error("Assessment Plan root is required"))?;
    let reviewed = root
        .get("reviewed-controls")
        .ok_or_else(|| error("Assessment Plan reviewed-controls are required"))?;
    let controls = plan_selections(
        reviewed.get("control-selections"),
        ("include-controls", "exclude-controls", "control-id"),
        eligible_controls,
        true,
        budget,
    )?;
    if controls.is_empty() {
        return Err(error("Assessment Plan effective reviewed control scope must not be empty"));
    }
    let objectives = plan_selections(
        reviewed.get("control-objective-selections"),
        ("include-objectives", "exclude-objectives", "objective-id"),
        eligible_objectives,
        false,
        budget,
    )?;
    let mut tasks = BTreeSet::new();
    if let Some(items) = root.get("tasks").and_then(Value::as_array) {
        plan_tasks(items, 0, &mut tasks, budget)?;
    }
    let mut inventory = AssessmentPlanInventory {
        reviewed_controls: controls,
        reviewed_objectives: objectives,
        tasks,
        explicit_subjects: BTreeMap::new(),
        include_all_subject_types: BTreeSet::new(),
        include_all_exclusions: BTreeMap::new(),
        referenced_subjects: BTreeMap::new(),
    };
    let groups = root
        .get("assessment-subjects")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Assessment Plan assessment-subjects are required"))?;
    for group in groups {
        plan_subject_group(group, &mut inventory, budget)?;
    }
    Ok(inventory)
}

/// Charge explicit selection rows and include-all expansions before merging their temporary sets.
fn plan_selections(
    value: Option<&Value>,
    fields: (&str, &str, &str),
    eligible: &BTreeSet<String>,
    required: bool,
    budget: &mut ContextBudget<'_>,
) -> Result<BTreeSet<String>, ForgeError> {
    let selections = value.and_then(Value::as_array);
    if required && selections.is_none() {
        return Err(error("Assessment Plan control-selections are required"));
    }
    let mut inventory = BTreeSet::new();
    for selection in selections.into_iter().flatten() {
        budget.row(&("plan-selection", fields))?;
        let mut selected = if selection.get("include-all").is_some() {
            copy_set(eligible, budget)?
        } else {
            let included = selection.get(fields.0).and_then(Value::as_array).ok_or_else(|| {
                error("Assessment Plan selection requires include-all or include rows")
            })?;
            let mut selected = BTreeSet::new();
            for item in included {
                let id = string_ref(item.get(fields.2), "Assessment Plan included ID")?;
                budget.row(&id)?;
                if !eligible.contains(id) {
                    return Err(error(
                        "Assessment Plan selects an ID outside exact Profile/Catalog",
                    ));
                }
                if !selected.insert(id.to_owned()) {
                    return Err(error("Assessment Plan selection contains duplicate IDs"));
                }
            }
            selected
        };
        if let Some(excluded) = selection.get(fields.1).and_then(Value::as_array) {
            for item in excluded {
                let id = string_ref(item.get(fields.2), "Assessment Plan excluded ID")?;
                budget.row(&id)?;
                if !eligible.contains(id) {
                    return Err(error(
                        "Assessment Plan excludes an ID outside exact Profile/Catalog",
                    ));
                }
                selected.remove(id);
            }
        }
        // The merge moves Strings; count each complete relation even if the union already has it.
        for id in &selected {
            budget.row(id)?;
        }
        inventory.extend(selected);
    }
    Ok(inventory)
}

/// Preserve recursive task UUID validation and count every declaration before insertion.
fn plan_tasks(
    items: &[Value],
    depth: usize,
    tasks: &mut BTreeSet<String>,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    enforce_depth(depth)?;
    for task in items {
        let id = string_ref(task.get("uuid"), "Assessment Plan task")?;
        insert(tasks, id, "Assessment Plan task", budget)?;
        if let Some(children) = task.get("tasks").and_then(Value::as_array) {
            plan_tasks(children, depth + 1, tasks, budget)?;
        }
    }
    Ok(())
}

/// Validate all subject group references, including exclusions and duplicate group declarations.
fn plan_subject_group(
    group: &Value,
    inventory: &mut AssessmentPlanInventory,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    let kind = parse_subject_type(string_ref(group.get("type"), "Assessment Plan subject type")?)?;
    budget.row(&("plan-subject-group", kind.as_str()))?;
    let included = plan_subject_refs(group.get("include-subjects"), kind, budget)?;
    let excluded = plan_subject_refs(group.get("exclude-subjects"), kind, budget)?;
    // Admit both full reference streams before a BTreeSet union deduplicates them.
    for id in included.iter().chain(&excluded) {
        budget.row(&(kind.as_str(), id))?;
    }
    inventory
        .referenced_subjects
        .entry(kind)
        .or_default()
        .extend(included.iter().chain(&excluded).cloned());
    if group.get("include-all").is_some() {
        if inventory.include_all_subject_types.contains(&kind) {
            if let Some(existing) = inventory.include_all_exclusions.get_mut(&kind) {
                for id in existing.iter() {
                    budget.row(&(kind.as_str(), id))?;
                }
                existing.retain(|id| excluded.contains(id));
            }
        } else {
            budget.row(&kind.as_str())?;
            inventory.include_all_subject_types.insert(kind);
            budget.row(&("subject-exclusions", kind.as_str()))?;
            inventory.include_all_exclusions.insert(kind, excluded);
        }
    } else {
        let effective = inventory.explicit_subjects.entry(kind).or_default();
        for id in &included {
            budget.row(&(kind.as_str(), id))?;
            if !excluded.contains(id) {
                effective.insert(id.clone());
            }
        }
    }
    Ok(())
}

/// Preserve exact subject types and UUID spellings while charging every include/exclude row.
fn plan_subject_refs(
    value: Option<&Value>,
    kind: SubjectType,
    budget: &mut ContextBudget<'_>,
) -> Result<BTreeSet<String>, ForgeError> {
    let mut result = BTreeSet::new();
    for item in value.and_then(Value::as_array).into_iter().flatten() {
        let declared = parse_subject_type(string_ref(item.get("type"), "subject reference type")?)?;
        let id = string_ref(item.get("subject-uuid"), "subject reference UUID")?;
        budget.row(&(declared.as_str(), id))?;
        if declared != kind {
            return Err(error("Assessment Plan subject type does not match its group"));
        }
        Uuid::parse_str(id).map_err(|_| error("Assessment Plan subject-uuid must be a UUID"))?;
        if !result.insert(id.to_owned()) {
            return Err(error("Assessment Plan contains a duplicate subject reference"));
        }
    }
    Ok(result)
}

/// Inventory the same six SSP subject collections and exact implementation UUID/control pairs.
fn ssp_inventory(
    value: &Value,
    budget: &mut ContextBudget<'_>,
) -> Result<SspInventory, ForgeError> {
    let root = value.get("system-security-plan").ok_or_else(|| error("SSP root is required"))?;
    let mut subjects = BTreeMap::new();
    let paths = [
        ("/system-security-plan/system-implementation/components", SubjectType::Component),
        ("/system-security-plan/system-implementation/inventory-items", SubjectType::InventoryItem),
        ("/system-security-plan/system-implementation/users", SubjectType::User),
        ("/system-security-plan/metadata/locations", SubjectType::Location),
        ("/system-security-plan/metadata/parties", SubjectType::Party),
        ("/system-security-plan/back-matter/resources", SubjectType::Resource),
    ];
    for (pointer, kind) in paths {
        if let Some(items) = value.pointer(pointer).and_then(Value::as_array) {
            for item in items {
                let id = string_ref(item.get("uuid"), "SSP subject UUID")?;
                Uuid::parse_str(id).map_err(|_| error("SSP subject UUID is invalid"))?;
                budget.row(&(kind.as_str(), id))?;
                if !subjects.entry(kind).or_insert_with(BTreeSet::new).insert(id.to_owned()) {
                    return Err(error("SSP contains a duplicate subject UUID"));
                }
            }
        }
    }
    let mut implementations = BTreeMap::new();
    if let Some(items) =
        root.pointer("/control-implementation/implemented-requirements").and_then(Value::as_array)
    {
        for item in items {
            let control = string_ref(item.get("control-id"), "SSP implementation control")?;
            implementation(item, control, &mut implementations, budget)?;
            if let Some(entries) = item.get("by-components").and_then(Value::as_array) {
                for entry in entries {
                    implementation(entry, control, &mut implementations, budget)?;
                }
            }
        }
    }
    Ok((subjects, implementations))
}

/// Admit one exact implementation owner tuple before copying either key or control ID.
fn implementation(
    value: &Value,
    control: &str,
    inventory: &mut BTreeMap<String, String>,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    let id = string_ref(value.get("uuid"), "SSP implementation UUID")?;
    Uuid::parse_str(id).map_err(|_| error("SSP implementation UUID is invalid"))?;
    budget.row(&(id, control))?;
    if inventory.insert(id.to_owned(), control.to_owned()).is_some() {
        return Err(error("SSP implementation UUID is duplicated"));
    }
    Ok(())
}

/// Admit an entire borrowed eligible set before cloning it for an include-all selection.
fn copy_set(
    source: &BTreeSet<String>,
    budget: &mut ContextBudget<'_>,
) -> Result<BTreeSet<String>, ForgeError> {
    for id in source {
        budget.row(id)?;
    }
    Ok(source.clone())
}

/// Admit a borrowed ID before copying it into a bounded legacy-equivalent native set.
fn insert(
    inventory: &mut BTreeSet<String>,
    id: &str,
    label: &str,
    budget: &mut ContextBudget<'_>,
) -> Result<(), ForgeError> {
    budget.row(&id)?;
    if inventory.len() >= MAX_INVENTORY_ITEMS {
        return Err(error("captured native inventory exceeds its existing item bound"));
    }
    if !inventory.insert(id.to_owned()) {
        return Err(error(format!("{label} contains a duplicate ID")));
    }
    Ok(())
}

/// Borrow a required native scalar without allocating before relationship admission.
fn string_ref<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, ForgeError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| error(format!("{label} must be a non-empty string")))
}

/// Proposed Unix-native controls; the portable seam remains typed by `CaptureSession`.
#[cfg(all(test, unix))]
mod tests {
    use super::super::{
        inventory_assessment_plan, inventory_catalog, inventory_ssp, load_from_root,
    };
    use super::*;
    use crate::evidence_capture::CaptureLease;
    use crate::oscal::{
        CatalogEnvelope, ProfileRoot, SelectionMode, SspComponentInput, build_assessment_plan,
        build_profile, build_ssp_skeleton,
    };
    use chrono::{TimeZone as _, Utc};
    use serde_json::json;
    use tempfile::TempDir;

    /// Actual owned native files and declared identities, with no detached capture authority.
    struct Fixture {
        /// Keeps the real confined root alive for the entire proof lifetime.
        directory: TempDir,
        /// Exact four original companion declarations derived from written native bytes.
        context: ContextManifest,
    }

    /// Construct genuine native companions using existing builders and a minimal Catalog.
    fn fixture() -> Fixture {
        let directory = TempDir::new().expect("owned fixture root");
        let catalog_value = json!({"catalog": {
            "uuid": "11111111-1111-4111-8111-111111111111",
            "metadata": {"title":"Fixture", "last-modified":"2026-01-01T00:00:00Z",
                "version":"1.0.0", "oscal-version":"1.2.3"},
            "controls":[{"id":"AC-1", "title":"Control", "parts":[
                {"id":"AC-1_smt", "name":"statement", "prose":"Fixture statement"},
                {"id":"AC-1_obj", "name":"objective", "prose":"Fixture objective"}
            ]}]
        }});
        let catalog: CatalogEnvelope =
            serde_json::from_value(catalog_value.clone()).expect("Catalog");
        let profile = build_profile(
            "catalog.json",
            vec!["AC-1".to_owned()],
            SelectionMode::Include,
            &[],
            Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        )
        .expect("Profile");
        let profile_value = serde_json::to_value(ProfileRoot { profile }).expect("Profile JSON");
        let components = [SspComponentInput {
            title: "Fixture service".to_owned(),
            description: "Fixture component".to_owned(),
            component_type: crate::oscal::ssp::ComponentType::Software,
        }];
        let ssp =
            build_ssp_skeleton("Fixture", "1.0.0", &catalog.catalog, &components, "profile.json")
                .expect("SSP");
        let ssp_value = serde_json::to_value(ssp).expect("SSP JSON");
        let mut plan_value = serde_json::to_value(
            build_assessment_plan(&["AC-1".to_owned()], "ssp.json", "Fixture").expect("AP"),
        )
        .expect("AP JSON");
        let component_uuid = ssp_value
            .pointer("/system-security-plan/system-implementation/components/0/uuid")
            .and_then(Value::as_str)
            .expect("generated SSP component UUID");
        plan_value["assessment-plan"]["assessment-subjects"] = json!([{
            "type": "component",
            "include-subjects": [{"subject-uuid": component_uuid, "type": "component"}]
        }]);
        let context = ContextManifest {
            assessment_plan: write_artifact(
                directory.path(),
                "ap.json",
                "assessment-plan",
                &plan_value,
            ),
            ssp: write_artifact(directory.path(), "ssp.json", "system-security-plan", &ssp_value),
            profile: write_artifact(directory.path(), "profile.json", "profile", &profile_value),
            catalog: write_artifact(directory.path(), "catalog.json", "catalog", &catalog_value),
            evidence_index: None,
        };
        Fixture { directory, context }
    }

    /// Write one real original and derive every manifest pin from those exact native bytes.
    fn write_artifact(
        root: &Path,
        path: &str,
        native_root: &str,
        value: &Value,
    ) -> ArtifactManifest {
        let bytes = serde_json::to_vec(value).expect("exact native bytes");
        std::fs::write(root.join(path), &bytes).expect("native fixture file");
        let native = &value[native_root];
        ArtifactManifest {
            artifact: path.into(),
            href: path.to_owned(),
            expected_sha256: sha256_hex(&bytes),
            root_uuid: native["uuid"].as_str().unwrap().to_owned(),
            document_version: native["metadata"]["version"].as_str().unwrap().to_owned(),
            oscal_version: native["metadata"]["oscal-version"].as_str().unwrap().to_owned(),
        }
    }

    /// Capture the four real originals once with their exact distinct model roles.
    fn leases(fixture: &Fixture, capture: &mut CaptureSession) -> Vec<CaptureLease> {
        [
            (&fixture.context.assessment_plan, CaptureRole::AssessmentPlan),
            (&fixture.context.ssp, CaptureRole::SystemSecurityPlan),
            (&fixture.context.profile, CaptureRole::Profile),
            (&fixture.context.catalog, CaptureRole::Catalog),
        ]
        .into_iter()
        .map(|(item, role)| {
            capture
                .required(&item.artifact, role, io::MAX_FILE_SIZE)
                .expect("actual native capture")
        })
        .collect()
    }

    /// Compare every `LoadedContext` field, including empty scope maps and private exclusions.
    fn projection(context: &LoadedContext) -> Value {
        json!({"identities":context.artifact_identities(),
            "evidence_index_sha256":context.evidence_index_sha256,
            "controls":context.controls, "statements":context.statements,
            "objectives":context.objectives, "statement_controls":context.statement_controls,
            "objective_controls":context.objective_controls,
            "reviewed_controls":context.reviewed_controls,
            "reviewed_objectives":context.reviewed_objectives, "tasks":context.tasks,
            "implementation_statements":context.implementation_statements,
            "implementation_statement_controls":context.implementation_statement_controls,
            "scoped_subjects":context.scoped_subjects, "excluded_subjects":context.excluded_subjects,
            "subjects":context.subjects, "include_all_subject_types":context.include_all_subject_types,
            "evidence":context.evidence, "input_paths":context.input_paths})
    }

    /// Exercise the real sealed loader and whole original proof, comparing complete legacy context.
    #[test]
    fn actual_originals_match_legacy_context_and_recheck_full_bytes() {
        let fixture = fixture();
        let mut capture = CaptureSession::new(
            &fixture.directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .expect("qualified root");
        let held = leases(&fixture, &mut capture);
        let captured =
            held.iter().map(|lease| (lease.path().to_path_buf(), lease.bytes())).collect();
        let loaded = load(&fixture.context, &captured, &mut capture).expect("sealed context");
        let legacy =
            load_from_root(capture.root(), &fixture.context).expect("legacy native context");
        assert_eq!(projection(&loaded), projection(&legacy));
        let proof = capture.finish();
        proof.verify_inputs().expect("complete originals unchanged");
        let bytes = std::fs::read(fixture.directory.path().join("catalog.json")).unwrap();
        let mut changed = bytes;
        changed.push(b' ');
        std::fs::write(fixture.directory.path().join("catalog.json"), changed).unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// Byte-equal detached buffers cannot impersonate the actual sealed original allocation.
    #[test]
    fn detached_map_and_wrong_role_fail_before_native_parsing() {
        let fixture = fixture();
        let mut capture = CaptureSession::new(
            &fixture.directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        let held = leases(&fixture, &mut capture);
        let detached: BTreeMap<_, _> =
            held.iter().map(|lease| (lease.path().to_path_buf(), lease.bytes().to_vec())).collect();
        let supplied =
            detached.iter().map(|(path, bytes)| (path.clone(), bytes.as_slice())).collect();
        assert!(load(&fixture.context, &supplied, &mut capture).is_err());
        assert!(
            capture
                .captured(&fixture.context.catalog.artifact, CaptureRole::AssessmentPlan)
                .is_err()
        );
        assert!(capture.captured(Path::new("missing.json"), CaptureRole::Catalog).is_err());
    }

    /// Exact importer-relative joins accept dot spelling but refuse wrong bases and escapes.
    #[test]
    fn nested_imports_use_actual_importer_base_without_filesystem_resolution() {
        assert!(
            relative_import(
                Path::new("nested/ap.json"),
                "./ssp.json",
                Path::new("nested/ssp.json")
            )
            .is_ok()
        );
        assert!(
            relative_import(Path::new("nested/ap.json"), "ssp.json", Path::new("ssp.json"))
                .is_err()
        );
        assert!(
            relative_import(Path::new("nested/ap.json"), "../ssp.json", Path::new("ssp.json"))
                .is_err()
        );
    }

    /// Include-all copies consume the existing complete ledger before another native clone.
    #[test]
    fn shared_relationship_exhaustion_refuses_include_all_without_reset() {
        let directory = TempDir::new().unwrap();
        let mut capture = CaptureSession::new(
            &directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        capture.relationships(crate::evidence_capture::MAX_RELATIONSHIPS - 1).unwrap();
        let mut budget =
            ContextBudget { capture: &mut capture, projection: ProjectionBudget::new() };
        let eligible = BTreeSet::from(["AC-1".to_owned(), "AC-2".to_owned()]);
        assert!(copy_set(&eligible, &mut budget).is_err());
        assert_eq!(eligible.len(), 2);
    }

    /// Duplicate and excluded references consume admission even when the effective union is small.
    #[test]
    fn filtered_profile_references_are_not_deduplicated_out_of_budget() {
        let directory = TempDir::new().unwrap();
        let mut capture = CaptureSession::new(
            &directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        capture.relationships(crate::evidence_capture::MAX_RELATIONSHIPS - 2).unwrap();
        let mut budget =
            ContextBudget { capture: &mut capture, projection: ProjectionBudget::new() };
        let selections = vec![json!({"with-ids":["AC-1","AC-1"]})];
        let eligible = BTreeSet::from(["AC-1".to_owned()]);
        let mut selected = eligible.clone();
        assert!(
            profile_selections(Some(&selections), &eligible, &mut selected, false, &mut budget)
                .is_err()
        );
        assert!(selected.is_empty());
    }

    /// Escaped metadata admission fails before copying a new ID into a retained native collection.
    #[test]
    fn escaped_metadata_cap_refuses_before_set_growth() {
        let directory = TempDir::new().unwrap();
        let mut capture = CaptureSession::new(
            &directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        let mut budget =
            ContextBudget { capture: &mut capture, projection: ProjectionBudget::new() };
        let long = "\\\"".repeat(100_000);
        for _ in 0..18 {
            budget.row(&long).expect("conservative metadata capacity");
        }
        let mut result = BTreeSet::new();
        let too_large = "\\\"".repeat(1_000_000);
        assert!(insert(&mut result, &too_large, "fixture", &mut budget).is_err());
        assert!(result.is_empty());
    }

    /// Budget-aware supported Catalog inventory remains exactly the legacy tuple and duplicate rule.
    #[test]
    fn catalog_inventory_preserves_owner_tuples_and_duplicate_refusal() {
        let directory = TempDir::new().unwrap();
        let mut capture = CaptureSession::new(
            &directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        let mut budget =
            ContextBudget { capture: &mut capture, projection: ProjectionBudget::new() };
        let value = json!({"catalog":{"groups":[{"controls":[{"id":"AC-1","parts":[
            {"id":"s","name":"statement"},{"id":"o","name":"objective"}]}]}]}});
        let expected = inventory_catalog(&value).unwrap();
        let mut actual = CapturedCatalog::default();
        catalog_container(&value["catalog"], 0, &mut actual, &mut budget).unwrap();
        assert_eq!(
            (
                actual.controls,
                actual.statements,
                actual.objectives,
                actual.statement_controls,
                actual.objective_controls
            ),
            expected
        );
        let duplicate = json!({"controls":[{"id":"AC-1"},{"id":"AC-1"}]});
        assert!(
            catalog_container(&duplicate, 0, &mut CapturedCatalog::default(), &mut budget).is_err()
        );
    }

    /// Subject include-all/exclusion intersections preserve the complete native scope semantics.
    #[test]
    fn plan_and_ssp_inventory_preserve_native_scope_and_identity_pairs() {
        let directory = TempDir::new().unwrap();
        let mut capture = CaptureSession::new(
            &directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .unwrap();
        let mut budget =
            ContextBudget { capture: &mut capture, projection: ProjectionBudget::new() };
        let id = "11111111-1111-4111-8111-111111111111";
        let plan = json!({"assessment-plan":{"reviewed-controls":{
            "control-selections":[{"include-all":{}}]},"assessment-subjects":[
            {"type":"component","include-all":{},"exclude-subjects":[{"type":"component","subject-uuid":id}]},
            {"type":"component","include-all":{}}]}});
        let eligible = BTreeSet::from(["AC-1".to_owned()]);
        let before = inventory_assessment_plan(&plan, &eligible, &BTreeSet::new()).unwrap();
        let after = plan_inventory(&plan, &eligible, &BTreeSet::new(), &mut budget).unwrap();
        assert_eq!(before.reviewed_controls, after.reviewed_controls);
        assert_eq!(before.include_all_exclusions, after.include_all_exclusions);
        assert_eq!(before.referenced_subjects, after.referenced_subjects);
        let ssp = json!({"system-security-plan":{"system-implementation":{
            "components":[{"uuid":id}]},"control-implementation":{
            "implemented-requirements":[{"uuid":id,"control-id":"AC-1"}]}}});
        assert_eq!(inventory_ssp(&ssp).unwrap(), ssp_inventory(&ssp, &mut budget).unwrap());
    }

    /// Admit nested tasks from real native originals, reject duplicate UUIDs and recheck AP bytes.
    #[test]
    fn actual_captured_nested_tasks_match_legacy_and_reject_duplicates() {
        let mut valid = fixture();
        let mut plan: Value = serde_json::from_slice(
            &std::fs::read(valid.directory.path().join("ap.json")).expect("original AP bytes"),
        )
        .expect("generated AP JSON");
        let parent = "22222222-2222-4222-8222-222222222222";
        let child = "33333333-3333-4333-8333-333333333333";
        let grandchild = "44444444-4444-4444-8444-444444444444";
        let sibling = "55555555-5555-4555-8555-555555555555";
        plan["assessment-plan"]["tasks"] = json!([
            {
                "uuid": parent, "type": "milestone", "title": "Parent task",
                "tasks": [{
                    "uuid": child, "type": "action", "title": "Child task",
                    "tasks": [{
                        "uuid": grandchild, "type": "action", "title": "Grandchild task"
                    }]
                }]
            },
            {"uuid": sibling, "type": "action", "title": "Sibling task"}
        ]);
        validate_schema("assessment-plan", &plan, OscalSchema::AssessmentPlan)
            .expect("complete nested AP remains valid against the vendored native schema");
        valid.context.assessment_plan =
            write_artifact(valid.directory.path(), "ap.json", "assessment-plan", &plan);
        let mut capture = CaptureSession::new(
            &valid.directory.path().canonicalize().expect("normalized owned fixture root"),
        )
        .expect("qualified native root");
        let held = leases(&valid, &mut capture);
        let captured =
            held.iter().map(|lease| (lease.path().to_path_buf(), lease.bytes())).collect();
        let loaded = load(&valid.context, &captured, &mut capture)
            .expect("actual four-original context admits the nested native tasks");
        let legacy = load_from_root(capture.root(), &valid.context)
            .expect("same native originals admit through the legacy context loader");
        let expected = BTreeSet::from([
            parent.to_owned(),
            child.to_owned(),
            grandchild.to_owned(),
            sibling.to_owned(),
        ]);
        assert_eq!(loaded.tasks, expected);
        assert_eq!(loaded.tasks, legacy.tasks);
        assert_eq!(projection(&loaded), projection(&legacy));
        let proof = capture.finish();
        assert_eq!(proof.captured_original_generations(), 4);
        proof.verify_inputs().expect("all four task-bearing original generations unchanged");
        let mut changed = std::fs::read(valid.directory.path().join("ap.json"))
            .expect("actual held AP still readable");
        changed.push(b' ');
        std::fs::write(valid.directory.path().join("ap.json"), changed)
            .expect("mutate the held AP generation after valid admission");
        assert!(proof.verify_inputs().is_err());

        let mut duplicate = fixture();
        let mut duplicate_plan: Value = serde_json::from_slice(
            &std::fs::read(duplicate.directory.path().join("ap.json"))
                .expect("second fixture actual AP bytes"),
        )
        .expect("second fixture generated AP JSON");
        duplicate_plan["assessment-plan"]["tasks"] = plan["assessment-plan"]["tasks"].clone();
        duplicate_plan["assessment-plan"]["tasks"][0]["tasks"][0]["tasks"][0]["uuid"] =
            json!(parent);
        validate_schema("assessment-plan", &duplicate_plan, OscalSchema::AssessmentPlan)
            .expect("duplicate task UUID shape is schema-valid before native inventory refusal");
        duplicate.context.assessment_plan = write_artifact(
            duplicate.directory.path(),
            "ap.json",
            "assessment-plan",
            &duplicate_plan,
        );
        let mut capture = CaptureSession::new(
            &duplicate.directory.path().canonicalize().expect("normalized duplicate fixture root"),
        )
        .expect("qualified duplicate root");
        let held = leases(&duplicate, &mut capture);
        let captured =
            held.iter().map(|lease| (lease.path().to_path_buf(), lease.bytes())).collect();
        let Err(refused) = load(&duplicate.context, &captured, &mut capture) else {
            panic!("recursive duplicate task UUID must be refused after native schema admission");
        };
        assert!(refused.to_string().contains("Assessment Plan task contains a duplicate ID"));
        let proof = capture.finish();
        assert_eq!(proof.captured_original_generations(), 4);
        proof
            .verify_inputs()
            .expect("duplicate refusal does not modify any actual native original");
    }
}
