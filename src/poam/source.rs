//! Exact, offline Assessment Results capture and source selection.
//!
//! The foundation accepts the current Forge AR representation, preserves all
//! finding/risk objects, and reports identity without judging remediation need.
//! The supported producer profile requires its four exact AP/SSP/Profile/Catalog
//! back-matter receipts; receipt consistency conveys no authenticity or approval.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::Value;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use super::manifest::{SourceKind, SourceManifest, SourceReference};
use super::report::{INVENTORY_SCHEMA_VERSION, SourceInventory, SourceObject};
use crate::ForgeError;
use crate::assessment_results::context::{self, LoadedContext};
use crate::assessment_results::manifest::{ArtifactManifest, SubjectType};
use crate::assessment_results::model::FORGE_ASSESSMENT_RESULTS_NS;
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};

const MAX_AGGREGATE_BYTES: usize = 100 * 1024 * 1024;
const MAX_SOURCE_OBJECTS: usize = 10_000;
const MAX_RESULTS: usize = 1_000;
const MAX_REFERENCES: usize = 1_000;
const MAX_PROPERTIES: usize = 64;
const MAX_GRAPH_EDGES: usize = 100_000;
const MAX_STRING_BYTES: usize = 64 * 1024;

static AR_VALIDATOR: OnceLock<Result<jsonschema::Validator, String>> = OnceLock::new();

/// Original confined file generation retained independently of parsed source objects.
#[derive(Debug)]
struct CapturedInput {
    /// Normalized local path under the qualified bundle root.
    relative: PathBuf,
    /// Complete original captured bytes used by exact revalidation.
    bytes: Vec<u8>,
    /// File identity tuple from the confined single-link reader.
    identity: (u64, u64),
}

/// Captured source and companions plus a content-minimizing selected inventory.
///
/// The five inputs retain their original file identities and bytes for revalidation.
/// Successful construction establishes the supported source profile and internal
/// references; it does not authenticate the producer, accept assessment assertions,
/// approve a remediation plan or determine which source objects require work.
#[derive(Debug)]
pub struct PreparedSource {
    /// Qualified bundle root used only for original-input revalidation.
    root: PathBuf,
    /// All five actual captured input generations, without invented sources.
    inputs: Vec<CapturedInput>,
    /// Complete selected-result identities after supported source qualification.
    inventory: SourceInventory,
}

impl PreparedSource {
    /// Borrow complete paths of the actual held five-source originals for sibling output preflight.
    pub(super) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.inputs.iter().map(|input| self.root.join(&input.relative))
    }

    /// The complete finding/risk inventory of the explicitly selected result.
    ///
    /// Includes satisfied findings and closed risks without eligibility filtering.
    /// Rows are sorted by source kind and exact stable key. State and optional digest
    /// properties remain explicit source declarations; computed object hashes bind
    /// the captured content. Assessment prose, actor identities and paths are omitted.
    #[must_use]
    pub fn inventory(&self) -> &SourceInventory {
        &self.inventory
    }

    /// Reject auxiliary originals that alias any held actual source identity.
    ///
    /// This read-only identity gate exposes no bytes or detached capture constructor.
    /// # Errors
    /// Refuses any authoring/native identity equal to a held source generation.
    pub(super) fn reject_input_aliases(&self, identities: &[(u64, u64)]) -> Result<(), ForgeError> {
        if self.inputs.iter().any(|input| identities.contains(&input.identity)) {
            return Err(error("auxiliary original aliases a held source input"));
        }
        Ok(())
    }

    /// Reopen confined inputs and require both the original identity and bytes.
    ///
    /// A same-byte replacement with a different captured identity is rejected.
    /// The check covers the inputs at revalidation time and does not assert that
    /// they cannot change afterward or that their assessment content was reviewed.
    ///
    /// # Errors
    /// Returns an error when any original input changes, disappears, or becomes unsafe.
    pub fn verify_inputs(&self) -> Result<(), ForgeError> {
        for captured in &self.inputs {
            let (bytes, identity) = crate::linkage::read_confined_local_file(
                &self.root,
                &captured.relative,
                crate::io::MAX_FILE_SIZE,
            )
            .map_err(|cause| error(format!("source input revalidation failed: {cause}")))?;
            if identity != captured.identity || bytes != captured.bytes {
                return Err(error("source input identity or bytes changed after capture"));
            }
        }
        Ok(())
    }
}

/// Capture and validate five local inputs and an explicitly identified AR result.
///
/// Each input is capped at the shared file bound; their total is capped at
/// 100 MiB before snapshot construction. At most 1,000 results and 10,000 total
/// conclusions are accepted. Neither network access nor process execution occurs.
/// The manifest path supplies the local bundle directory; the manifest file itself
/// is not read here. Directly constructed [`SourceManifest`] values receive the same
/// declaration checks as parsed values. AR 1.2.3 is checked against the pinned 1.2.3
/// schema, while exact companion version declarations may be 1.2.0–1.2.3.
///
/// Every result is checked for coherent references, including unselected results.
/// Selection requires the explicit result UUID and stable key, never its position.
/// Four metadata context hashes and four native companion receipts must agree with
/// the captured AP/SSP/Profile/Catalog and their import chain. These consistency
/// checks provide no producer authenticity, source-review approval or workflow authority.
///
/// # Errors
/// Returns an error for unsafe paths, stale identities, unsupported source
/// structures, inconsistent references, or an absent/mismatched selected result.
pub fn load(manifest_path: &Path, source: &SourceManifest) -> Result<PreparedSource, ForgeError> {
    super::manifest::validate_source(source)?;
    let parent = manifest_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let root = parent
        .canonicalize()
        .map_err(|cause| error(format!("cannot resolve source manifest directory: {cause}")))?;
    let declarations = [
        ("assessment-results", &source.assessment_results),
        ("assessment-plan", &source.context.assessment_plan),
        ("system-security-plan", &source.context.ssp),
        ("profile", &source.context.profile),
        ("catalog", &source.context.catalog),
    ];
    let inputs = capture_inputs(&root, &declarations)?;
    let captured: BTreeMap<_, _> =
        inputs.iter().map(|input| (input.relative.clone(), input.bytes.clone())).collect();
    let loaded_context = context::load_captured(&source.context, &captured)
        .map_err(|cause| error(format!("captured source context is invalid: {cause}")))?;
    let ar = parse_artifact(&inputs[0].bytes, "Assessment Results source")?;
    validate_ar_schema(&ar)?;
    let document =
        ar.get("assessment-results").ok_or_else(|| error("Assessment Results root is required"))?;
    validate_artifact_identity(document, &source.assessment_results)?;
    validate_import(source, document)?;
    validate_context_pins(document, &loaded_context)?;
    validate_resource_pins(document, source, &loaded_context)?;
    validate_supported_structure(&ar)?;
    validate_graph_edge_bound(document)?;
    let inventory = inventory_results(document, source, &loaded_context)?;
    let prepared = PreparedSource { root, inputs, inventory };
    // Catch changes during context/schema/inventory validation before exposing a result.
    prepared.verify_inputs()?;
    Ok(prepared)
}

/// Capture the five unique pinned local artifacts within the whole source byte budget.
fn capture_inputs(
    root: &Path,
    declarations: &[(&str, &ArtifactManifest); 5],
) -> Result<Vec<CapturedInput>, ForgeError> {
    let mut inputs = Vec::with_capacity(5);
    let mut paths = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut total = 0_usize;
    for (kind, expected) in declarations {
        if !paths.insert(&expected.artifact) {
            return Err(error("source artifacts must have distinct confined paths"));
        }
        let remaining = u64::try_from(MAX_AGGREGATE_BYTES.saturating_sub(total))
            .map_err(|_| error("source aggregate input size conversion failed"))?;
        let (bytes, identity) = crate::linkage::read_confined_local_file(
            root,
            &expected.artifact,
            crate::io::MAX_FILE_SIZE.min(remaining),
        )
        .map_err(|cause| error(format!("cannot capture {kind} source input: {cause}")))?;
        if !identities.insert(identity) {
            return Err(error("source artifacts must have distinct file identities"));
        }
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| error("source aggregate input size overflowed"))?;
        if total > MAX_AGGREGATE_BYTES {
            return Err(error("source artifacts exceed the 100 MiB aggregate input bound"));
        }
        if sha256_hex(&bytes) != expected.expected_sha256 {
            return Err(error(format!("{kind} source input does not match the exact SHA-256 pin")));
        }
        inputs.push(CapturedInput { relative: expected.artifact.clone(), bytes, identity });
    }
    Ok(inputs)
}

/// Strict-parse captured artifact JSON without duplicate keys or unbounded strings.
fn parse_artifact(bytes: &[u8], label: &str) -> Result<Value, ForgeError> {
    json_strict::parse_value(
        bytes,
        label,
        Limits { max_depth: 128, max_string_bytes: MAX_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))
}

/// Validate the captured Assessment Results against the pinned offline native schema.
fn validate_ar_schema(value: &Value) -> Result<(), ForgeError> {
    let validator = AR_VALIDATOR
        .get_or_init(|| {
            let schema: Value = serde_json::from_str(include_str!(
                "../../schemas/oscal_assessment-results_schema.json"
            ))
            .map_err(|cause| format!("vendored AR schema is invalid JSON: {cause}"))?;
            jsonschema::validator_for(&schema)
                .map_err(|cause| format!("vendored AR schema failed to compile: {cause}"))
        })
        .as_ref()
        .map_err(|cause| error(cause.clone()))?;
    if validator.is_valid(value) {
        Ok(())
    } else {
        // Schema error displays can contain assessment prose; expose locations only.
        let locations: Vec<_> = validator
            .iter_errors(value)
            .take(10)
            .map(|cause| json_strict::bounded(&cause.instance_path().to_string()))
            .collect();
        Err(error(format!(
            "source AR fails the pinned OSCAL 1.2.3 schema at: {}",
            locations.join(", ")
        )))
    }
}

/// Compare each captured root UUID and version to its exact declared pin.
fn validate_artifact_identity(
    document: &Value,
    expected: &ArtifactManifest,
) -> Result<(), ForgeError> {
    let uuid = required_uuid(document.get("uuid"), "AR document UUID")?;
    let metadata = document.get("metadata").ok_or_else(|| error("AR metadata is required"))?;
    let version = required_string(metadata.get("version"), "AR document version")?;
    let oscal_version = required_string(metadata.get("oscal-version"), "AR OSCAL version")?;
    if uuid != expected.root_uuid
        || version != expected.document_version
        || oscal_version != expected.oscal_version
    {
        return Err(error(
            "AR root UUID, document version or OSCAL version does not match the exact manifest pin",
        ));
    }
    if oscal_version != "1.2.3" {
        return Err(error("the POA&M foundation supports Forge AR OSCAL version 1.2.3 only"));
    }
    Ok(())
}

/// Resolve the complete five-artifact import chain against the declared local companions.
fn validate_import(source: &SourceManifest, document: &Value) -> Result<(), ForgeError> {
    let href = required_string(document.pointer("/import-ap/href"), "AR import-ap href")?;
    if href != source.context.assessment_plan.href {
        return Err(error(
            "AR import-ap href differs from the exact Assessment Plan companion href",
        ));
    }
    let parent = source.assessment_results.artifact.parent().unwrap_or_else(|| Path::new(""));
    let resolved = normalized_href_path(parent, href)?;
    if resolved != source.context.assessment_plan.artifact {
        return Err(error("AR import-ap resolves to a different confined Assessment Plan path"));
    }
    Ok(())
}

/// Resolve a normalized import href relative to its containing artifact directory.
fn normalized_href_path(parent: &Path, href: &str) -> Result<PathBuf, ForgeError> {
    let mut relative = parent.to_path_buf();
    for segment in href.split('/') {
        match segment {
            "." => {}
            "" | ".." => return Err(error("source import href is not a normalized descendant")),
            segment if segment.contains(['\\', ':', '?', '#', '%']) => {
                return Err(error(
                    "source import href contains unsupported URI or platform aliases",
                ));
            }
            segment => relative.push(segment),
        }
    }
    Ok(relative)
}

/// Require complete metadata hashes for the four captured assessment companions.
fn validate_context_pins(document: &Value, context: &LoadedContext) -> Result<(), ForgeError> {
    let metadata = document.get("metadata").ok_or_else(|| error("AR metadata is required"))?;
    let properties = property_values(metadata, &["trust-boundary", "context-sha256"])?;
    let mut pins = BTreeMap::new();
    for value in properties.get("context-sha256").into_iter().flatten() {
        let (kind, hash) = value
            .split_once(':')
            .ok_or_else(|| error("AR context-sha256 property is malformed"))?;
        json_strict::validate_lowercase_sha256("AR context-sha256", hash).map_err(error)?;
        if pins.insert(kind, hash).is_some() {
            return Err(error("AR duplicates a context-sha256 kind"));
        }
    }
    if pins.len() != 4 {
        return Err(error("AR must declare exactly four AP/SSP/Profile/Catalog context pins"));
    }
    for identity in context.artifact_identities() {
        if pins.get(identity.kind) != Some(&identity.sha256.as_str()) {
            return Err(error(format!(
                "AR {} context pin differs from captured companion bytes",
                identity.kind
            )));
        }
    }
    Ok(())
}

/// Require unique back-matter context receipts bound to exact captured companions.
/// Resolve each receipt from the AR directory; companion import hrefs have their
/// own importer-relative bases and are verified by the captured context loader.
fn validate_resource_pins(
    document: &Value,
    source: &SourceManifest,
    context: &LoadedContext,
) -> Result<(), ForgeError> {
    let back_matter = document.get("back-matter").ok_or_else(|| {
        error("supported Forge AR source requires all four context receipts in back-matter")
    })?;
    let declarations: BTreeMap<_, _> = [
        ("assessment-plan", &source.context.assessment_plan),
        ("system-security-plan", &source.context.ssp),
        ("profile", &source.context.profile),
        ("catalog", &source.context.catalog),
    ]
    .into_iter()
    .collect();
    let identities: BTreeMap<_, _> = context
        .artifact_identities()
        .into_iter()
        .map(|identity| (identity.kind, identity))
        .collect();
    let parent = source.assessment_results.artifact.parent().unwrap_or_else(|| Path::new(""));
    for resource in
        required_array(back_matter.get("resources"), "AR context resources", MAX_REFERENCES)?
    {
        let properties = property_values(
            resource,
            &["context-kind", "root-uuid", "document-version", "oscal-version"],
        )?;
        let kind = one_property(&properties, "context-kind")?
            .ok_or_else(|| error("AR context resource kind is missing"))?;
        let identity =
            identities.get(kind).ok_or_else(|| error("unsupported AR context resource kind"))?;
        if one_property(&properties, "root-uuid")? != Some(identity.root_uuid.as_str())
            || one_property(&properties, "document-version")?
                != Some(identity.document_version.as_str())
            || one_property(&properties, "oscal-version")? != Some(identity.oscal_version.as_str())
        {
            return Err(error(
                "AR context resource identity differs from captured companion identity",
            ));
        }
        let links =
            required_array(resource.get("rlinks"), "AR context resource links", MAX_REFERENCES)?;
        if links.len() != 1 {
            return Err(error("AR context resource requires one exact companion link"));
        }
        let href = required_string(links[0].get("href"), "AR context resource href")?;
        let declaration =
            declarations.get(kind).ok_or_else(|| error("unknown AR context resource kind"))?;
        if normalized_href_path(parent, href)? != declaration.artifact {
            return Err(error(
                "AR context resource link resolves to a different confined companion",
            ));
        }
        let hashes =
            required_array(links[0].get("hashes"), "AR context resource hashes", MAX_REFERENCES)?;
        if hashes.len() != 1
            || hashes[0].get("algorithm").and_then(Value::as_str) != Some("SHA-256")
            || hashes[0].get("value").and_then(Value::as_str) != Some(identity.sha256.as_str())
        {
            return Err(error("AR context resource SHA-256 differs from captured companion bytes"));
        }
    }
    Ok(())
}

/// Compare every selected reference against the complete captured inventory.
///
/// At most 10,000 references are allowed. Each unique kind/key pair must also match
/// the selected result UUID, original object UUID and computed canonical SHA-256.
/// Matching proves source identity only; it does not select an eligible remediation
/// item, validate workflow state or accept the producer's declared content digest.
/// An empty reference list is valid for an unselected foundation scaffold.
///
/// # Errors
/// Returns an error for duplicate selections or any differing result, kind, key,
/// UUID or computed canonical object digest. Declared source digests give no credit.
pub fn validate_selection(
    prepared: &PreparedSource,
    references: &[SourceReference],
) -> Result<(), ForgeError> {
    if references.len() > MAX_SOURCE_OBJECTS {
        return Err(error("source selection exceeds the 10,000 object bound"));
    }
    let inventory = prepared.inventory();
    let mut selected = BTreeSet::new();
    let indexed: BTreeMap<_, _> = inventory
        .objects
        .iter()
        .map(|object| ((object.kind, object.key.as_str()), object))
        .collect();
    for reference in references {
        json_strict::validate_lowercase_sha256(
            "source reference SHA-256",
            &reference.expected_sha256,
        )
        .map_err(error)?;
        if !selected.insert((reference.kind, reference.key.as_str())) {
            return Err(error("source selection duplicates a kind/key reference"));
        }
        let object = indexed
            .get(&(reference.kind, reference.key.as_str()))
            .ok_or_else(|| error("source selection references an absent kind/key"))?;
        if reference.result_uuid != inventory.result_uuid
            || reference.result_uuid != object.result_uuid
            || reference.uuid != object.uuid
            || reference.expected_sha256 != object.sha256
        {
            return Err(error(
                "source selection result/UUID/computed object digest differs from captured source",
            ));
        }
    }
    Ok(())
}

/// Bound the whole native reference graph before expanding selected adjacency.
fn validate_graph_edge_bound(value: &Value) -> Result<(), ForgeError> {
    /// Count nested reference arrays while enforcing the aggregate graph-edge ceiling.
    fn count(value: &Value, total: &mut usize) -> Result<(), ForgeError> {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    let length = if matches!(
                        key.as_str(),
                        "related-observations"
                            | "related-risks"
                            | "related-tasks"
                            | "actors"
                            | "subjects"
                            | "include-controls"
                            | "include-objectives"
                    ) {
                        child.as_array().map_or(0, Vec::len)
                    } else {
                        usize::from(matches!(
                            key.as_str(),
                            "target" | "implementation-statement-uuid"
                        ))
                    };
                    *total = total
                        .checked_add(length)
                        .ok_or_else(|| error("AR source graph edge count overflowed"))?;
                    if *total > MAX_GRAPH_EDGES {
                        return Err(error(
                            "AR source graph exceeds the 100,000 reference-edge aggregate bound",
                        ));
                    }
                    count(child, total)?;
                }
            }
            Value::Array(values) => {
                for child in values {
                    count(child, total)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    count(value, &mut 0)
}

/// Validate all result identities and select the exact UUID and stable-key pair.
fn inventory_results(
    document: &Value,
    source: &SourceManifest,
    context: &LoadedContext,
) -> Result<SourceInventory, ForgeError> {
    let results = required_array(document.get("results"), "AR results", MAX_RESULTS)?;
    let mut uuids = BTreeSet::new();
    collect_object_uuids(document, &mut uuids)?;
    let mut result_keys = BTreeSet::new();
    let mut stable_keys = BTreeSet::new();
    let mut selected = None;
    let mut object_count = 0_usize;
    for result in results {
        let uuid = required_uuid(result.get("uuid"), "AR result UUID")?;
        let key = required_property(result, "stable-key", &["stable-key"])?;
        if !result_keys.insert(key) {
            return Err(error("AR duplicates a result stable key"));
        }
        for (field, kind) in
            [("observations", "observation"), ("findings", "finding"), ("risks", "risk")]
        {
            for object in optional_array(result.get(field), "AR conclusions", MAX_SOURCE_OBJECTS)? {
                object_count += 1;
                if object_count > MAX_SOURCE_OBJECTS {
                    return Err(error("AR conclusions exceed the 10,000 object aggregate bound"));
                }
                let key = required_property(object, "stable-key", CONCLUSION_PROPERTIES)?;
                if !stable_keys.insert((kind, key)) {
                    return Err(error("AR duplicates a conclusion kind/stable-key across results"));
                }
            }
        }
        // Every result must have coherent source references, including unselected results.
        let objects = inventory_one_result(result, context)?;
        if uuid == source.result.uuid {
            if key != source.result.key {
                return Err(error("explicit AR result UUID resolves to a different stable key"));
            }
            selected = Some(objects);
        } else if key == source.result.key {
            return Err(error("explicit AR result stable key resolves to a different UUID"));
        }
    }
    let mut objects =
        selected.ok_or_else(|| error("explicit AR result UUID and stable key are absent"))?;
    objects.sort_by(|left, right| (&left.kind, &left.key).cmp(&(&right.kind, &right.key)));
    Ok(SourceInventory {
        schema_version: INVENTORY_SCHEMA_VERSION,
        validation_scope: "source-integrity-only",
        workflow_validated: false,
        source_sha256: source.assessment_results.expected_sha256.clone(),
        result_uuid: source.result.uuid.clone(),
        result_key: source.result.key.clone(),
        objects,
    })
}

const CONCLUSION_PROPERTIES: &[&str] = &[
    "stable-key",
    "content-sha256",
    "rationale-sha256",
    "assessor-key",
    "assessment-start",
    "assessment-end",
    "assessment-method",
    "rationale",
    "reviewer-declared-severity",
    "reviewer-declared-confidence",
];

/// Inventory every selected-result finding and risk without eligibility filtering.
fn inventory_one_result(
    result: &Value,
    context: &LoadedContext,
) -> Result<Vec<SourceObject>, ForgeError> {
    let result_uuid = required_uuid(result.get("uuid"), "AR result UUID")?;
    let scope = result_scope(result, context)?;
    let observations =
        optional_array(result.get("observations"), "AR observations", MAX_SOURCE_OBJECTS)?;
    let findings = optional_array(result.get("findings"), "AR findings", MAX_SOURCE_OBJECTS)?;
    let risks = optional_array(result.get("risks"), "AR risks", MAX_SOURCE_OBJECTS)?;
    let observation_uuids: BTreeSet<_> = observations
        .iter()
        .map(|value| required_uuid(value.get("uuid"), "AR observation UUID"))
        .collect::<Result<_, _>>()?;
    let risk_uuids: BTreeSet<_> = risks
        .iter()
        .map(|value| required_uuid(value.get("uuid"), "AR risk UUID"))
        .collect::<Result<_, _>>()?;
    for observation in observations {
        validate_observation(observation, context)?;
    }
    let mut objects = Vec::with_capacity(findings.len() + risks.len());
    let mut risk_controls: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for finding in findings {
        let control = finding_control(finding, context, &scope)?;
        let related_observations = optional_array(
            finding.get("related-observations"),
            "related observations",
            MAX_REFERENCES,
        )?;
        validate_uuid_references(related_observations, "observation-uuid", &observation_uuids)?;
        let related_risks =
            optional_array(finding.get("related-risks"), "related risks", MAX_REFERENCES)?;
        validate_uuid_references(related_risks, "risk-uuid", &risk_uuids)?;
        for related in related_risks {
            let risk_uuid = required_uuid(related.get("risk-uuid"), "related risk UUID")?;
            risk_controls.entry(risk_uuid).or_default().insert(control.clone());
        }
        let state = required_string(finding.pointer("/target/status/state"), "finding state")?;
        if !matches!(state, "satisfied" | "not-satisfied") {
            return Err(error("unsupported Forge AR finding state"));
        }
        objects.push(source_object(
            finding,
            SourceKind::Finding,
            result_uuid,
            state,
            vec![control],
        )?);
    }
    for risk in risks {
        let state = required_string(risk.get("status"), "risk status")?;
        if !matches!(
            state,
            "open"
                | "investigating"
                | "remediating"
                | "deviation-requested"
                | "deviation-approved"
                | "closed"
        ) {
            return Err(error("unsupported Forge AR risk status"));
        }
        let uuid = required_uuid(risk.get("uuid"), "risk UUID")?;
        let controls = risk_controls.get(uuid).into_iter().flatten().cloned().collect();
        objects.push(source_object(risk, SourceKind::Risk, result_uuid, state, controls)?);
    }
    Ok(objects)
}

#[derive(Debug)]
/// Explicit result control/objective sets validated against captured companion context.
struct ResultScope {
    /// Complete selected control identifiers from the native result.
    controls: BTreeSet<String>,
    /// Complete selected assessment objective identifiers from the native result.
    objectives: BTreeSet<String>,
}

/// Resolve the declared result controls and objectives against the captured native context.
fn result_scope(result: &Value, context: &LoadedContext) -> Result<ResultScope, ForgeError> {
    let reviewed =
        result.get("reviewed-controls").ok_or_else(|| error("AR reviewed-controls is required"))?;
    let controls = selected_ids(reviewed, "control-selections", "include-controls", "control-id")?;
    let objectives = selected_ids(
        reviewed,
        "control-objective-selections",
        "include-objectives",
        "objective-id",
    )?;
    if controls.is_empty() {
        return Err(error("supported Forge AR result requires an explicit nonempty control scope"));
    }
    if !controls.is_subset(&context.controls)
        || !controls.is_subset(&context.reviewed_controls)
        || !objectives.is_subset(&context.objectives)
        || !objectives.is_subset(&context.reviewed_objectives)
    {
        return Err(error("AR result scope differs from the captured AP/Profile/Catalog scope"));
    }
    Ok(ResultScope { controls, objectives })
}

/// Require a bounded unique set of native control or objective references.
fn selected_ids(
    parent: &Value,
    field: &str,
    selection: &str,
    id_field: &str,
) -> Result<BTreeSet<String>, ForgeError> {
    let mut values = BTreeSet::new();
    for group in optional_array(parent.get(field), "AR control selections", MAX_REFERENCES)? {
        closed_object(group, &[selection], "AR control selection")?;
        for reference in
            required_array(group.get(selection), "AR included references", MAX_SOURCE_OBJECTS)?
        {
            closed_object(reference, &[id_field], "AR included reference")?;
            let id = required_string(reference.get(id_field), "AR included ID")?;
            if !values.insert(id.to_string()) {
                return Err(error("AR reviewed-controls duplicates an explicit selected ID"));
            }
            if values.len() > MAX_SOURCE_OBJECTS {
                return Err(error(
                    "AR explicit result scope exceeds the 10,000 ID aggregate bound",
                ));
            }
        }
    }
    Ok(values)
}

/// Resolve a finding target to the captured result scope without inferring applicability.
fn finding_control(
    finding: &Value,
    context: &LoadedContext,
    scope: &ResultScope,
) -> Result<String, ForgeError> {
    let kind = required_string(finding.pointer("/target/type"), "finding target type")?;
    let id = required_string(finding.pointer("/target/target-id"), "finding target ID")?;
    let control = match kind {
        "statement-id" => {
            context.statement_controls.get(id).filter(|control| scope.controls.contains(*control))
        }
        "objective-id" => {
            context.objective_controls.get(id).filter(|_| scope.objectives.contains(id))
        }
        _ => return Err(error("unsupported Forge AR finding target type")),
    }
    .ok_or_else(|| error("finding target is absent from the exact result/Catalog/AP scope"))?;
    if let Some(value) = finding.get("implementation-statement-uuid") {
        let implementation = required_uuid(Some(value), "finding implementation statement UUID")?;
        if context.implementation_statement_controls.get(implementation) != Some(control) {
            return Err(error(
                "finding SSP implementation statement does not match its exact target control",
            ));
        }
    }
    Ok(control.clone())
}

/// Validate the supported observation references against captured assessment subjects.
fn validate_observation(observation: &Value, context: &LoadedContext) -> Result<(), ForgeError> {
    for subject in
        required_array(observation.get("subjects"), "AR observation subjects", MAX_REFERENCES)?
    {
        let uuid = required_uuid(subject.get("subject-uuid"), "observation subject UUID")?;
        let kind = match required_string(subject.get("type"), "observation subject type")? {
            "component" => SubjectType::Component,
            "inventory-item" => SubjectType::InventoryItem,
            "location" => SubjectType::Location,
            "party" => SubjectType::Party,
            "user" => SubjectType::User,
            "resource" => SubjectType::Resource,
            _ => return Err(error("unsupported Forge AR observation subject type")),
        };
        if !context.subject_is_in_scope(kind, uuid) {
            return Err(error("AR observation subject is absent from exact captured SSP/AP scope"));
        }
    }
    if !optional_array(
        observation.get("relevant-evidence"),
        "AR relevant evidence",
        MAX_REFERENCES,
    )?
    .is_empty()
    {
        return Err(error(
            "AR evidence references require an evidence-index context unsupported by this foundation",
        ));
    }
    for origin in
        required_array(observation.get("origins"), "AR observation origins", MAX_REFERENCES)?
    {
        for task in optional_array(origin.get("related-tasks"), "AR origin tasks", MAX_REFERENCES)?
        {
            let uuid = required_uuid(task.get("task-uuid"), "AR related task UUID")?;
            if !context.tasks.contains(uuid) {
                return Err(error("AR observation references an absent captured AP task"));
            }
        }
    }
    Ok(())
}

/// Require every bounded UUID edge to identify an admitted source object.
fn validate_uuid_references(
    references: &[Value],
    field: &str,
    available: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    let mut seen = BTreeSet::new();
    for reference in references {
        closed_object(reference, &[field], "AR conclusion reference")?;
        let uuid = required_uuid(reference.get(field), "AR related conclusion UUID")?;
        if !available.contains(uuid) || !seen.insert(uuid) {
            return Err(error("AR conclusion reference is absent from its result or duplicated"));
        }
    }
    Ok(())
}

/// Preserve original state and compute the canonical source digest for one inventory row.
fn source_object(
    object: &Value,
    kind: SourceKind,
    result_uuid: &str,
    state: &str,
    control_ids: Vec<String>,
) -> Result<SourceObject, ForgeError> {
    let properties = property_values(object, CONCLUSION_PROPERTIES)?;
    Ok(SourceObject {
        kind,
        key: one_property(&properties, "stable-key")?
            .ok_or_else(|| error("AR conclusion requires one Forge stable-key"))?
            .to_string(),
        uuid: required_uuid(object.get("uuid"), "AR conclusion UUID")?.to_string(),
        result_uuid: result_uuid.to_string(),
        sha256: canonical_object_sha256(object)?,
        state: state.to_string(),
        control_ids,
        declared_content_sha256: optional_digest(&properties, "content-sha256")?,
        declared_rationale_sha256: optional_digest(&properties, "rationale-sha256")?,
    })
}

/// Retain a declared lowercase digest only when its explicit type and syntax are valid.
fn optional_digest(
    properties: &BTreeMap<&str, Vec<&str>>,
    name: &str,
) -> Result<Option<String>, ForgeError> {
    one_property(properties, name)?
        .map(|digest| {
            json_strict::validate_lowercase_sha256("AR declared digest", digest).map_err(error)?;
            Ok(digest.to_string())
        })
        .transpose()
}

/// Canonical identity preserves array order and recursively sorts object keys.
fn canonical_object_sha256(value: &Value) -> Result<String, ForgeError> {
    let mut writer = CanonicalHash { digest: Sha256::new(), bytes: 0 };
    write_canonical(value, &mut writer)?;
    Ok(crate::hashing::lower_hex(&writer.digest.finalize()))
}

/// Streaming canonical digest with a bounded logical serialized byte count.
struct CanonicalHash {
    /// Incremental canonical SHA-256 state; original file hashes remain separate.
    digest: Sha256,
    /// Logical bytes already hashed against the canonical object ceiling.
    bytes: u64,
}

impl std::io::Write for CanonicalHash {
    /// Reject overbound canonical bytes before updating the streaming digest.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let length = u64::try_from(bytes.len())
            .map_err(|_| std::io::Error::other("canonical source size overflow"))?;
        if length > crate::io::MAX_FILE_SIZE.saturating_sub(self.bytes) {
            return Err(std::io::Error::other(
                "canonical source object exceeds the file byte bound",
            ));
        }
        self.digest.update(bytes);
        self.bytes += length;
        Ok(bytes.len())
    }

    /// Complete the in-memory writer interface without an external output operation.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Hash recursively sorted object keys while preserving array order and scalar values.
fn write_canonical(value: &Value, writer: &mut CanonicalHash) -> Result<(), ForgeError> {
    use std::io::Write as _;
    let write = |writer: &mut CanonicalHash, bytes: &[u8]| {
        writer.write_all(bytes).map_err(|cause| {
            error(format!("canonical source object serialization failed: {cause}"))
        })
    };
    match value {
        Value::Object(map) => {
            write(writer, b"{")?;
            let ordered: BTreeMap<_, _> = map.iter().collect();
            for (index, (key, value)) in ordered.into_iter().enumerate() {
                if index != 0 {
                    write(writer, b",")?;
                }
                serde_json::to_writer(&mut *writer, key).map_err(|cause| {
                    error(format!("canonical source key serialization failed: {cause}"))
                })?;
                write(writer, b":")?;
                write_canonical(value, writer)?;
            }
            write(writer, b"}")
        }
        Value::Array(values) => {
            write(writer, b"[")?;
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    write(writer, b",")?;
                }
                write_canonical(value, writer)?;
            }
            write(writer, b"]")
        }
        value => serde_json::to_writer(writer, value).map_err(|cause| {
            error(format!("canonical source value serialization failed: {cause}"))
        }),
    }
}

/// Collect unique canonical UUIDs for the complete supported native object family.
fn collect_object_uuids<'a>(
    value: &'a Value,
    seen: &mut BTreeSet<&'a str>,
) -> Result<(), ForgeError> {
    match value {
        Value::Object(map) => {
            if let Some(uuid) = map.get("uuid") {
                let uuid = required_uuid(Some(uuid), "AR object UUID")?;
                if !seen.insert(uuid) {
                    return Err(error("AR duplicates an object UUID"));
                }
            }
            for child in map.values() {
                collect_object_uuids(child, seen)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_object_uuids(child, seen)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Read only the supported namespaced properties without silently dropping extensions.
fn property_values<'a>(
    object: &'a Value,
    allowed: &[&str],
) -> Result<BTreeMap<&'a str, Vec<&'a str>>, ForgeError> {
    let mut properties: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for property in optional_array(object.get("props"), "AR properties", MAX_PROPERTIES)? {
        closed_object(property, &["name", "ns", "value"], "Forge AR property")?;
        if property.get("ns").and_then(Value::as_str) != Some(FORGE_ASSESSMENT_RESULTS_NS) {
            return Err(error(
                "unsupported AR property namespace; foundation accepts the Forge AR source profile only",
            ));
        }
        let name = required_string(property.get("name"), "AR property name")?;
        if !allowed.contains(&name) {
            return Err(error("unsupported Forge AR source property"));
        }
        let value = required_string(property.get("value"), "AR property value")?;
        let entries = properties.entry(name).or_default();
        if !entries.is_empty() && name != "context-sha256" {
            return Err(error("AR duplicates a Forge source property"));
        }
        entries.push(value);
    }
    Ok(properties)
}

/// Require exactly one property value for an explicit source assertion.
fn one_property<'a>(
    properties: &BTreeMap<&str, Vec<&'a str>>,
    name: &str,
) -> Result<Option<&'a str>, ForgeError> {
    match properties.get(name).map_or(&[][..], Vec::as_slice) {
        [] => Ok(None),
        [value] => Ok(Some(*value)),
        _ => Err(error("AR duplicates a required identity property")),
    }
}

/// Read a mandatory supported property rather than infer a missing assertion.
fn required_property<'a>(
    object: &'a Value,
    name: &str,
    allowed: &[&str],
) -> Result<&'a str, ForgeError> {
    let properties = property_values(object, allowed)?;
    one_property(&properties, name)?
        .ok_or_else(|| error("AR source is missing a required Forge identity property"))
}

/// Require a nonempty source string within the 64 KiB byte bound.
fn required_string<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, ForgeError> {
    let value = value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| error(format!("{label} must be a nonempty string")))?;
    if value.len() > MAX_STRING_BYTES {
        return Err(error(format!("{label} exceeds the 64 KiB string bound")));
    }
    Ok(value)
}

/// Require one canonical native UUID from a mandatory source field.
fn required_uuid<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, ForgeError> {
    let value = required_string(value, label)?;
    let parsed = Uuid::parse_str(value).map_err(|_| error(format!("{label} is invalid")))?;
    if parsed.hyphenated().to_string() != value {
        return Err(error(format!("{label} must use lowercase hyphenated UUID spelling")));
    }
    Ok(value)
}

/// Require a bounded native array before constructing source references.
fn required_array<'a>(
    value: Option<&'a Value>,
    label: &str,
    maximum: usize,
) -> Result<&'a [Value], ForgeError> {
    let values = value
        .and_then(Value::as_array)
        .ok_or_else(|| error(format!("{label} must be an array")))?;
    if values.len() > maximum {
        return Err(error(format!("{label} exceeds the {maximum} item bound")));
    }
    Ok(values)
}

/// Admit a missing optional array or require the complete bounded supplied array.
fn optional_array<'a>(
    value: Option<&'a Value>,
    label: &str,
    maximum: usize,
) -> Result<&'a [Value], ForgeError> {
    match value {
        None => Ok(&[]),
        Some(value) => required_array(Some(value), label, maximum),
    }
}

/// Refuse unsupported native fields rather than discard them from source qualification.
fn closed_object(value: &Value, allowed: &[&str], label: &str) -> Result<(), ForgeError> {
    let object = value.as_object().ok_or_else(|| error(format!("{label} must be an object")))?;
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(error(format!(
            "{label} contains a native source extension unsupported by this foundation"
        )));
    }
    Ok(())
}

/// Check the complete supported Forge Assessment Results subset before inventory.
fn validate_supported_structure(value: &Value) -> Result<(), ForgeError> {
    closed_object(value, &["assessment-results"], "AR envelope")?;
    let document = value.get("assessment-results").ok_or_else(|| error("AR root is required"))?;
    closed_object(
        document,
        &["uuid", "metadata", "import-ap", "results", "back-matter"],
        "Forge AR document",
    )?;
    let metadata = document.get("metadata").ok_or_else(|| error("AR metadata is required"))?;
    closed_object(
        metadata,
        &["title", "last-modified", "version", "oscal-version", "props", "roles", "parties"],
        "Forge AR metadata",
    )?;
    property_values(metadata, &["trust-boundary", "context-sha256"])?;
    let mut role_ids = BTreeSet::new();
    for role in optional_array(metadata.get("roles"), "AR roles", MAX_REFERENCES)? {
        closed_object(role, &["id", "title"], "Forge AR role")?;
        if !role_ids.insert(required_string(role.get("id"), "AR role ID")?) {
            return Err(error("AR duplicates a role ID"));
        }
    }
    let mut party_uuids = BTreeSet::new();
    let mut party_keys = BTreeSet::new();
    for party in optional_array(metadata.get("parties"), "AR parties", MAX_REFERENCES)? {
        closed_object(party, &["uuid", "type", "name", "props"], "Forge AR party")?;
        party_uuids.insert(required_uuid(party.get("uuid"), "AR party UUID")?);
        if !party_keys.insert(required_property(party, "stable-key", &["stable-key"])?) {
            return Err(error("AR duplicates a party stable key"));
        }
    }
    let import = document.get("import-ap").ok_or_else(|| error("AR import-ap is required"))?;
    closed_object(import, &["href"], "Forge AR import-ap")?;
    for result in required_array(document.get("results"), "AR results", MAX_RESULTS)? {
        validate_result_structure(result, &role_ids, &party_uuids)?;
    }
    let back_matter = document.get("back-matter").ok_or_else(|| {
        error("supported Forge AR source requires all four context receipts in back-matter")
    })?;
    validate_back_matter(back_matter)?;
    Ok(())
}

/// Check every supported result assembly and its complete source-object families.
fn validate_result_structure(
    result: &Value,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    closed_object(
        result,
        &[
            "uuid",
            "title",
            "description",
            "start",
            "end",
            "props",
            "reviewed-controls",
            "observations",
            "findings",
            "risks",
        ],
        "Forge AR result",
    )?;
    property_values(result, &["stable-key"])?;
    closed_object(
        result.get("reviewed-controls").ok_or_else(|| error("AR reviewed-controls is required"))?,
        &["control-selections", "control-objective-selections"],
        "Forge AR reviewed-controls",
    )?;
    for observation in
        optional_array(result.get("observations"), "AR observations", MAX_SOURCE_OBJECTS)?
    {
        validate_observation_structure(observation, roles, parties)?;
    }
    for finding in optional_array(result.get("findings"), "AR findings", MAX_SOURCE_OBJECTS)? {
        closed_object(
            finding,
            &[
                "uuid",
                "title",
                "description",
                "props",
                "origins",
                "target",
                "implementation-statement-uuid",
                "related-observations",
                "related-risks",
            ],
            "Forge AR finding",
        )?;
        validate_conclusion_properties(finding, false)?;
        validate_origins(finding, roles, parties)?;
        reject_conclusion_tasks(finding)?;
        let target = finding.get("target").ok_or_else(|| error("AR finding target is required"))?;
        closed_object(target, &["type", "target-id", "status"], "Forge AR finding target")?;
        closed_object(
            target.get("status").ok_or_else(|| error("AR finding target status is required"))?,
            &["state", "reason"],
            "Forge AR finding target status",
        )?;
        if let Some(reason) = target.pointer("/status/reason") {
            if !matches!(
                required_string(Some(reason), "AR finding reason")?,
                "pass" | "fail" | "other"
            ) {
                return Err(error("unsupported Forge AR finding reason"));
            }
        }
    }
    for risk in optional_array(result.get("risks"), "AR risks", MAX_SOURCE_OBJECTS)? {
        closed_object(
            risk,
            &["uuid", "title", "description", "statement", "props", "status", "origins"],
            "Forge AR risk",
        )?;
        validate_conclusion_properties(risk, true)?;
        validate_origins(risk, roles, parties)?;
        reject_conclusion_tasks(risk)?;
    }
    Ok(())
}

/// Check supported observation assemblies without accepting workflow authority.
fn validate_observation_structure(
    observation: &Value,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    closed_object(
        observation,
        &[
            "uuid",
            "title",
            "description",
            "props",
            "methods",
            "origins",
            "subjects",
            "relevant-evidence",
            "collected",
        ],
        "Forge AR observation",
    )?;
    validate_conclusion_properties(observation, false)?;
    validate_origins(observation, roles, parties)?;
    for subject in
        required_array(observation.get("subjects"), "AR observation subjects", MAX_REFERENCES)?
    {
        closed_object(subject, &["subject-uuid", "type"], "Forge AR observation subject")?;
    }
    let methods =
        required_array(observation.get("methods"), "AR observation methods", MAX_REFERENCES)?;
    for method in methods {
        if !matches!(
            required_string(Some(method), "AR observation method")?,
            "EXAMINE" | "INTERVIEW" | "TEST" | "UNKNOWN"
        ) {
            return Err(error("unsupported Forge AR observation method"));
        }
    }
    Ok(())
}

/// Require the supported finding or risk declaration profile.
fn validate_conclusion_properties(object: &Value, risk: bool) -> Result<(), ForgeError> {
    let allowed = if risk { CONCLUSION_PROPERTIES } else { &CONCLUSION_PROPERTIES[..8] };
    let properties = property_values(object, allowed)?;
    optional_digest(&properties, "content-sha256")?;
    optional_digest(&properties, "rationale-sha256")?;
    Ok(())
}

/// Validate supported origin references without authenticating their actors.
fn validate_origins(
    object: &Value,
    roles: &BTreeSet<&str>,
    parties: &BTreeSet<&str>,
) -> Result<(), ForgeError> {
    for origin in required_array(object.get("origins"), "AR origins", MAX_REFERENCES)? {
        closed_object(origin, &["actors", "related-tasks"], "Forge AR origin")?;
        for actor in required_array(origin.get("actors"), "AR origin actors", MAX_REFERENCES)? {
            closed_object(actor, &["type", "actor-uuid", "role-id"], "Forge AR origin actor")?;
            let uuid = required_uuid(actor.get("actor-uuid"), "AR actor UUID")?;
            let role = required_string(actor.get("role-id"), "AR actor role ID")?;
            if actor.get("type").and_then(Value::as_str) != Some("party")
                || !parties.contains(uuid)
                || !roles.contains(role)
            {
                return Err(error(
                    "AR actor does not reference an exact local metadata party and role",
                ));
            }
        }
        for task in optional_array(origin.get("related-tasks"), "AR related tasks", MAX_REFERENCES)?
        {
            closed_object(task, &["task-uuid"], "Forge AR related task")?;
        }
    }
    Ok(())
}

/// Refuse task extensions whose remediation workflow is outside this foundation.
fn reject_conclusion_tasks(object: &Value) -> Result<(), ForgeError> {
    for origin in required_array(object.get("origins"), "AR conclusion origins", MAX_REFERENCES)? {
        if !optional_array(origin.get("related-tasks"), "AR conclusion tasks", MAX_REFERENCES)?
            .is_empty()
        {
            return Err(error(
                "finding/risk task extensions are unsupported by the Forge AR source profile",
            ));
        }
    }
    Ok(())
}

/// Check complete supported context receipts and their declared native identities.
fn validate_back_matter(back_matter: &Value) -> Result<(), ForgeError> {
    closed_object(back_matter, &["resources"], "Forge AR back matter")?;
    let resources =
        required_array(back_matter.get("resources"), "AR back matter resources", MAX_REFERENCES)?;
    if resources.len() != 4 {
        return Err(error(
            "supported Forge AR without evidence index requires exactly four context resources",
        ));
    }
    let mut kinds = BTreeSet::new();
    for resource in resources {
        closed_object(
            resource,
            &["uuid", "title", "props", "rlinks"],
            "Forge AR context resource",
        )?;
        let properties = property_values(
            resource,
            &["context-kind", "root-uuid", "document-version", "oscal-version"],
        )?;
        let kind = one_property(&properties, "context-kind")?
            .ok_or_else(|| error("AR context resource kind is required"))?;
        if !matches!(kind, "assessment-plan" | "system-security-plan" | "profile" | "catalog")
            || !kinds.insert(kind)
        {
            return Err(error("AR context resource kind is unsupported or duplicated"));
        }
        let links =
            required_array(resource.get("rlinks"), "AR context resource links", MAX_REFERENCES)?;
        if links.len() != 1 {
            return Err(error("Forge AR context resource requires one exact companion link"));
        }
        for link in links {
            closed_object(link, &["href", "hashes"], "Forge AR context resource link")?;
            let hashes =
                required_array(link.get("hashes"), "AR context resource hashes", MAX_REFERENCES)?;
            if hashes.len() != 1 {
                return Err(error("Forge AR context resource link requires one exact SHA-256"));
            }
            for hash in hashes {
                closed_object(hash, &["algorithm", "value"], "Forge AR context resource hash")?;
                if hash.get("algorithm").and_then(Value::as_str) != Some("SHA-256") {
                    return Err(error("unsupported AR context resource hash algorithm"));
                }
                json_strict::validate_lowercase_sha256(
                    "AR context resource hash",
                    required_string(hash.get("value"), "AR context resource hash value")?,
                )
                .map_err(error)?;
            }
        }
    }
    Ok(())
}

/// Construct the typed foundation failure without granting source or workflow approval.
fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

/// Private current source inventory qualified from actual shared captures.
///
/// The caller must retain the owning `CaptureSession` and finalized `CaptureProof`.
/// This type has no public constructor, original-byte clone or artifact renderer.
pub(super) struct CapturedSource {
    /// Complete explicitly selected result inventory after the unchanged native checks.
    inventory: SourceInventory,
}

impl CapturedSource {
    /// Borrow the complete selected inventory without transferring current-proof authority.
    pub(super) fn inventory(&self) -> &SourceInventory {
        &self.inventory
    }

    /// Apply the existing exact source-reference predicates to this privately qualified inventory.
    ///
    /// This does not infer eligibility or evidence sufficiency. The private caller
    /// separately retains and rechecks every original through its `CaptureProof`.
    /// # Errors
    /// Refuses duplicate, absent or differing result/kind/key/UUID/computed-hash tuples.
    pub(super) fn validate_selection(
        &self,
        references: &[SourceReference],
    ) -> Result<(), ForgeError> {
        if references.len() > MAX_SOURCE_OBJECTS {
            return Err(error("source selection exceeds the 10,000 object bound"));
        }
        let inventory = self.inventory();
        let mut selected = BTreeSet::new();
        let indexed: BTreeMap<_, _> = inventory
            .objects
            .iter()
            .map(|object| ((object.kind, object.key.as_str()), object))
            .collect();
        for reference in references {
            json_strict::validate_lowercase_sha256(
                "source reference SHA-256",
                &reference.expected_sha256,
            )
            .map_err(error)?;
            if !selected.insert((reference.kind, reference.key.as_str())) {
                return Err(error("source selection duplicates a kind/key reference"));
            }
            let object = indexed
                .get(&(reference.kind, reference.key.as_str()))
                .ok_or_else(|| error("source selection references an absent kind/key"))?;
            if reference.result_uuid != inventory.result_uuid
                || reference.result_uuid != object.result_uuid
                || reference.uuid != object.uuid
                || reference.expected_sha256 != object.sha256
            {
                return Err(error(
                    "source selection result/UUID/computed object digest differs from captured source",
                ));
            }
        }
        Ok(())
    }
}

/// Qualify the same five-source profile once, retaining only its selected inventory for existing callers.
///
/// The shared private epoch loader consumes the original validation sequence.
/// Decoded native/context values are dropped here; the owning capture proof,
/// rather than this inventory, retains and rechecks original generations.
/// Existing public `load` and `PreparedSource` remain a separate unchanged path.
/// # Errors
/// Refuses the unchanged declaration, alias, pin, native, context, graph and budget predicates.
pub(super) fn load_with_capture(
    manifest_path: &Path,
    source: &SourceManifest,
    capture: &mut crate::evidence_capture::CaptureSession,
) -> Result<CapturedSource, ForgeError> {
    let prepared = load_epoch_source_with_capture(manifest_path, source, capture)?;
    Ok(CapturedSource { inventory: prepared.inventory })
}

/// Actual decoded native tree and context qualified together in the caller's single capture pool.
///
/// Fields and construction are private. This holder borrows the exact declaration
/// used by capture; it exposes no original bytes, detached proof or Debug prose.
/// The caller's complete `CaptureProof` remains the only original freshness check.
pub(super) struct CapturedEpochSource<'a> {
    /// Complete original decoded native envelope, including every unselected result.
    original: Value,
    /// Actual once-loaded AP/SSP/Profile/Catalog context from held original leases.
    context: LoadedContext,
    /// Complete explicitly selected inventory after all-result source qualification.
    inventory: SourceInventory,
    /// Borrowed exact five-source declaration used by this construction.
    declaration: &'a SourceManifest,
}

impl CapturedEpochSource<'_> {
    /// Borrow the complete original decoded envelope without generation or output authority.
    pub(super) fn original(&self) -> &Value {
        &self.original
    }

    /// Borrow the actual captured context for the maintained typed next-result constructor.
    pub(super) fn context(&self) -> &LoadedContext {
        &self.context
    }

    /// Borrow the selected inventory; all other epochs were qualified by the same loader.
    pub(super) fn inventory(&self) -> &SourceInventory {
        &self.inventory
    }
}

/// Retain the actual native/context products of the existing shared five-source validation sequence.
///
/// No raw original is cloned, reopened or recaptured. The declaration reference
/// is borrowed rather than copied, and every former loader predicate retains its
/// order. The caller owns the session/proof and charges derived growth separately.
/// # Errors
/// Refuses unsafe declarations, aliases, stale raw pins, schema/profile/context
/// mismatches, shared budget overflow and incoherent complete graph or selection.
pub(super) fn load_epoch_source_with_capture<'a>(
    manifest_path: &Path,
    source: &'a SourceManifest,
    capture: &mut crate::evidence_capture::CaptureSession,
) -> Result<CapturedEpochSource<'a>, ForgeError> {
    use crate::evidence_capture::CaptureRole;

    super::manifest::validate_source(source)?;
    if manifest_path.parent() != Some(capture.root()) {
        return Err(error("shared source root differs from the original plan directory"));
    }
    let declarations = [
        (CaptureRole::AssessmentResults, &source.assessment_results),
        (CaptureRole::AssessmentPlan, &source.context.assessment_plan),
        (CaptureRole::SystemSecurityPlan, &source.context.ssp),
        (CaptureRole::Profile, &source.context.profile),
        (CaptureRole::Catalog, &source.context.catalog),
    ];
    let mut leases = Vec::with_capacity(5);
    let mut paths = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for (role, expected) in declarations {
        if !paths.insert(&expected.artifact) {
            return Err(error("source artifacts must have distinct confined paths"));
        }
        let lease = capture.required(&expected.artifact, role, crate::io::MAX_FILE_SIZE)?;
        if !identities.insert(lease.identity()) {
            return Err(error("source artifacts must have distinct file identities"));
        }
        if sha256_hex(lease.bytes()) != expected.expected_sha256 {
            return Err(error("shared source input differs from its exact raw SHA-256 pin"));
        }
        leases.push(lease);
    }
    let ar = parse_artifact(leases[0].bytes(), "Assessment Results source")?;
    validate_ar_schema(&ar)?;
    let document =
        ar.get("assessment-results").ok_or_else(|| error("Assessment Results root is required"))?;
    validate_supported_structure(&ar)?;
    validate_graph_edge_bound(document)?;
    charge_shared_source_graph(document, capture)?;
    let captured: BTreeMap<_, _> = [
        &source.assessment_results,
        &source.context.assessment_plan,
        &source.context.ssp,
        &source.context.profile,
        &source.context.catalog,
    ]
    .into_iter()
    .zip(leases.iter())
    .map(|(expected, lease)| (expected.artifact.clone(), lease.bytes()))
    .collect();
    let loaded_context = context::load_captured_refs(&source.context, &captured, capture)?;
    validate_artifact_identity(document, &source.assessment_results)?;
    validate_import(source, document)?;
    validate_context_pins(document, &loaded_context)?;
    validate_resource_pins(document, source, &loaded_context)?;
    let inventory = inventory_results(document, source, &loaded_context)?;
    Ok(CapturedEpochSource {
        original: ar,
        context: loaded_context,
        inventory,
        declaration: source,
    })
}

/// Qualify a generated complete epoch graph against the actual held context and declaration.
///
/// This is derived schema/profile/reference validation, never raw-generation
/// admission. It does not feed an invented Value through capture lease checks.
/// The private producer must separately admit growth, preserve the old tree and
/// metadata, and retain/recheck the complete original `CaptureProof` before writes.
/// # Errors
/// Refuses root-identity, native/schema/profile/graph/import/context/receipt or
/// all-result selection mismatches and shared before-adjacency budget overflow.
pub(super) fn validate_derived_epoch_graph(
    source: &CapturedEpochSource<'_>,
    derived: &Value,
    capture: &mut crate::evidence_capture::CaptureSession,
) -> Result<(), ForgeError> {
    validate_ar_schema(derived)?;
    let document = derived
        .get("assessment-results")
        .ok_or_else(|| error("Assessment Results root is required"))?;
    if document.get("uuid") != source.original.pointer("/assessment-results/uuid") {
        return Err(error("derived epoch graph changes the captured native root identity"));
    }
    validate_supported_structure(derived)?;
    validate_graph_edge_bound(document)?;
    charge_shared_source_graph(document, capture)?;
    validate_import(source.declaration, document)?;
    validate_context_pins(document, &source.context)?;
    validate_resource_pins(document, source.declaration, &source.context)?;
    inventory_results(document, source.declaration, &source.context)?;
    Ok(())
}

/// Forward the maintained canonical whole-object digest without creating source freshness authority.
/// # Errors
/// Refuses the same unsupported canonical JSON representation as the existing source writer.
pub(super) fn canonical_epoch_object_sha256(value: &Value) -> Result<String, ForgeError> {
    canonical_object_sha256(value)
}

/// Charge complete AR object records and the existing native reference edge families before adjacency growth.
fn charge_shared_source_graph(
    value: &Value,
    capture: &mut crate::evidence_capture::CaptureSession,
) -> Result<(), ForgeError> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let count = if matches!(
                    key.as_str(),
                    "results"
                        | "observations"
                        | "findings"
                        | "risks"
                        | "related-observations"
                        | "related-risks"
                        | "related-tasks"
                        | "actors"
                        | "subjects"
                        | "include-controls"
                        | "include-objectives"
                ) {
                    child.as_array().map_or(0, Vec::len)
                } else {
                    usize::from(matches!(key.as_str(), "target" | "implementation-statement-uuid"))
                };
                capture.relationships(count)?;
                charge_shared_source_graph(child, capture)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                charge_shared_source_graph(child, capture)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const RESULT: &str = "11111111-1111-4111-8111-111111111111";
    const FINDING: &str = "22222222-2222-4222-8222-222222222222";
    const RISK: &str = "33333333-3333-4333-8333-333333333333";

    /// Construct a synthetic namespaced property for strict source-profile controls.
    fn property(name: &str, value: &str) -> Value {
        json!({"name": name, "ns": FORGE_ASSESSMENT_RESULTS_NS, "value": value})
    }

    /// Build a detached inventory fixture for exact tuple-selection controls.
    fn prepared() -> PreparedSource {
        PreparedSource {
            root: PathBuf::new(),
            inputs: Vec::new(),
            inventory: SourceInventory {
                schema_version: INVENTORY_SCHEMA_VERSION,
                validation_scope: "source-integrity-only",
                workflow_validated: false,
                source_sha256: "a".repeat(64),
                result_uuid: RESULT.to_string(),
                result_key: "result-1".to_string(),
                objects: vec![SourceObject {
                    kind: SourceKind::Finding,
                    key: "finding-1".to_string(),
                    uuid: FINDING.to_string(),
                    result_uuid: RESULT.to_string(),
                    sha256: "b".repeat(64),
                    state: "satisfied".to_string(),
                    control_ids: vec!["AC-1".to_string()],
                    declared_content_sha256: Some("c".repeat(64)),
                    declared_rationale_sha256: Some("d".repeat(64)),
                }],
            },
        }
    }

    /// Construct one complete source-selection tuple from a fixture inventory row.
    fn reference() -> SourceReference {
        SourceReference {
            kind: SourceKind::Finding,
            key: "finding-1".to_string(),
            uuid: FINDING.to_string(),
            result_uuid: RESULT.to_string(),
            expected_sha256: "b".repeat(64),
        }
    }

    /// Verify that canonical digest sorts nested keys and preserves arrays.
    #[test]
    fn canonical_digest_sorts_nested_keys_and_preserves_arrays() {
        let first = parse_artifact(br#"{"z":[{"b":2,"a":1},3],"a":true}"#, "test").unwrap();
        let reordered = parse_artifact(br#"{"a":true,"z":[{"a":1,"b":2},3]}"#, "test").unwrap();
        assert_eq!(
            canonical_object_sha256(&first).unwrap(),
            canonical_object_sha256(&reordered).unwrap()
        );
        let changed = json!({"a": true, "z": [3, {"a": 1, "b": 2}]});
        assert_ne!(
            canonical_object_sha256(&first).unwrap(),
            canonical_object_sha256(&changed).unwrap()
        );
        assert_eq!(
            canonical_object_sha256(&first).unwrap(),
            sha256_hex(br#"{"a":true,"z":[{"a":1,"b":2},3]}"#)
        );
    }

    /// Verify that computed digest changes when prose changes despite same declared hash.
    #[test]
    fn computed_digest_changes_when_prose_changes_despite_same_declared_hash() {
        let mut object = json!({"description": "original prose", "props": [property("content-sha256", &"a".repeat(64))]});
        let before = canonical_object_sha256(&object).unwrap();
        object["description"] = json!("changed prose");
        assert_ne!(before, canonical_object_sha256(&object).unwrap());
        let properties = property_values(&object, CONCLUSION_PROPERTIES).unwrap();
        assert_eq!(optional_digest(&properties, "content-sha256").unwrap(), Some("a".repeat(64)));
    }

    /// Verify that strict source parser rejects duplicate keys trailing values and large strings.
    #[test]
    fn strict_source_parser_rejects_duplicate_keys_trailing_values_and_large_strings() {
        assert!(parse_artifact(br#"{"uuid":"a","uuid":"b"}"#, "test").is_err());
        assert!(parse_artifact(b"{} {}", "test").is_err());
        let bytes =
            serde_json::to_vec(&json!({"description": "x".repeat(MAX_STRING_BYTES + 1)})).unwrap();
        assert!(parse_artifact(&bytes, "test").is_err());
    }

    /// Verify that property profile rejects foreign unknown duplicate and extended properties.
    #[test]
    fn property_profile_rejects_foreign_unknown_duplicate_and_extended_properties() {
        let good = json!({"props": [property("stable-key", "finding-1")]});
        assert_eq!(
            required_property(&good, "stable-key", CONCLUSION_PROPERTIES).unwrap(),
            "finding-1"
        );
        let mut foreign = good.clone();
        foreign["props"][0]["ns"] = json!("https://example.invalid/ns");
        assert!(property_values(&foreign, CONCLUSION_PROPERTIES).is_err());
        let unknown = json!({"props": [property("unsupported", "value")]});
        assert!(property_values(&unknown, CONCLUSION_PROPERTIES).is_err());
        let duplicate =
            json!({"props": [property("stable-key", "one"), property("stable-key", "two")]});
        assert!(property_values(&duplicate, CONCLUSION_PROPERTIES).is_err());
        let mut extended = good;
        extended["props"][0]["remarks"] = json!("native extension");
        assert!(property_values(&extended, CONCLUSION_PROPERTIES).is_err());
    }

    /// Verify that optional declared digests are explicitly typed and lowercase.
    #[test]
    fn optional_declared_digests_are_explicitly_typed_and_lowercase() {
        let absent = BTreeMap::new();
        assert_eq!(optional_digest(&absent, "content-sha256").unwrap(), None);
        let upper = "A".repeat(64);
        let properties = BTreeMap::from([("content-sha256", vec![upper.as_str()])]);
        assert!(optional_digest(&properties, "content-sha256").is_err());
        let short = BTreeMap::from([("rationale-sha256", vec!["abcd"])]);
        assert!(optional_digest(&short, "rationale-sha256").is_err());
    }

    /// Verify that uuid spelling and global object duplicates fail closed.
    #[test]
    fn uuid_spelling_and_global_object_duplicates_fail_closed() {
        assert!(required_uuid(Some(&json!(RESULT)), "test").is_ok());
        assert!(required_uuid(Some(&json!("11111111111141118111111111111111")), "test").is_err());
        assert!(
            required_uuid(Some(&json!("ABCDEFAB-1111-4111-8111-111111111111")), "test").is_err()
        );
        let duplicate = json!({"uuid": RESULT, "results": [{"uuid": RESULT}]});
        assert!(collect_object_uuids(&duplicate, &mut BTreeSet::new()).is_err());
        let distinct = json!({"uuid": RESULT, "results": [{"uuid": FINDING}]});
        assert!(collect_object_uuids(&distinct, &mut BTreeSet::new()).is_ok());
    }

    /// Verify that import href resolution is relative and rejects aliases.
    #[test]
    fn import_href_resolution_is_relative_and_rejects_aliases() {
        assert_eq!(
            normalized_href_path(Path::new("nested"), "./assessment-plan.json").unwrap(),
            Path::new("nested/assessment-plan.json")
        );
        for href in [
            "/assessment-plan.json",
            "../assessment-plan.json",
            "a//assessment-plan.json",
            "a%2fb.json",
            "a\\b.json",
            "C:input.json",
            "input.json?query",
            "input.json#fragment",
        ] {
            assert!(normalized_href_path(Path::new("nested"), href).is_err(), "{href}");
        }
    }

    /// Verify that selections require the entire computed identity tuple.
    #[test]
    fn selections_require_the_entire_computed_identity_tuple() {
        let prepared = prepared();
        let reference = reference();
        assert!(validate_selection(&prepared, std::slice::from_ref(&reference)).is_ok());
        for field in 0..5 {
            let mut altered = reference.clone();
            match field {
                0 => altered.kind = SourceKind::Risk,
                1 => altered.key = "another-finding".to_string(),
                2 => altered.uuid = RISK.to_string(),
                3 => altered.result_uuid = RISK.to_string(),
                _ => altered.expected_sha256 = "c".repeat(64), // Declared content hash must never qualify.
            }
            assert!(validate_selection(&prepared, &[altered]).is_err());
        }
        assert!(validate_selection(&prepared, &[reference.clone(), reference]).is_err());
    }

    /// Verify that finding and risk task extensions fail explicitly.
    #[test]
    fn finding_and_risk_task_extensions_fail_explicitly() {
        let unsupported =
            json!({"origins": [{"actors": [], "related-tasks": [{"task-uuid": RISK}]}]});
        assert!(reject_conclusion_tasks(&unsupported).is_err());
        assert!(reject_conclusion_tasks(&json!({"origins": [{"actors": []}]})).is_ok());
    }

    /// Verify that closed native structures reject extensions that official schema may allow.
    #[test]
    fn closed_native_structures_reject_extensions_that_official_schema_may_allow() {
        assert!(
            closed_object(
                &json!({"href": "ap.json", "remarks": "native extension"}),
                &["href"],
                "import"
            )
            .is_err()
        );
        assert!(closed_object(&json!({"href": "ap.json"}), &["href"], "import").is_ok());
    }

    /// Verify that revalidation rejects byte mutation identity replacement and deletion.
    #[test]
    fn revalidation_rejects_byte_mutation_identity_replacement_and_deletion() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let file = root.join("source.json");
        std::fs::write(&file, b"original").unwrap();
        let (bytes, identity) =
            crate::linkage::read_confined_local_file(&root, Path::new("source.json"), 100).unwrap();
        let mut prepared = prepared();
        prepared.root = root;
        prepared.inputs =
            vec![CapturedInput { relative: PathBuf::from("source.json"), bytes, identity }];
        assert!(prepared.verify_inputs().is_ok());
        std::fs::write(&file, b"modified").unwrap();
        assert!(prepared.verify_inputs().is_err());
        std::fs::write(&file, b"original").unwrap();
        assert!(prepared.verify_inputs().is_ok());
        // Keep the old inode alive to exclude immediate inode reuse on replacement.
        std::fs::rename(&file, prepared.root.join("preserved-original.json")).unwrap();
        std::fs::write(&file, b"original").unwrap();
        assert!(prepared.verify_inputs().is_err());
        std::fs::remove_file(&file).unwrap();
        assert!(prepared.verify_inputs().is_err());
    }

    /// Verify that graph reference bound is aggregate before adjacency expansion.
    #[test]
    fn graph_reference_bound_is_aggregate_before_adjacency_expansion() {
        let one = json!({"risk-uuid": RISK});
        let at_bound = json!({"findings": [{"related-risks": vec![one.clone(); MAX_GRAPH_EDGES]}]});
        assert!(validate_graph_edge_bound(&at_bound).is_ok());
        let overflow = json!({"findings": [{"related-risks": vec![one; MAX_GRAPH_EDGES]}, {"target": {"type": "statement-id"}}]});
        assert!(validate_graph_edge_bound(&overflow).is_err());
    }

    /// Verify that revalidation rejects new symlink and hardlink aliases.
    #[cfg(unix)]
    #[test]
    fn revalidation_rejects_new_symlink_and_hardlink_aliases() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let file = root.join("source.json");
        std::fs::write(&file, b"original").unwrap();
        let (bytes, identity) =
            crate::linkage::read_confined_local_file(&root, Path::new("source.json"), 100).unwrap();
        let mut prepared = prepared();
        prepared.root = root;
        prepared.inputs =
            vec![CapturedInput { relative: PathBuf::from("source.json"), bytes, identity }];
        let alias = prepared.root.join("alias.json");
        std::fs::hard_link(&file, &alias).unwrap();
        assert!(prepared.verify_inputs().is_err());
        std::fs::remove_file(&alias).unwrap();
        std::fs::rename(&file, &alias).unwrap();
        symlink(&alias, &file).unwrap();
        assert!(prepared.verify_inputs().is_err());
    }
    /// Construct a synthetic context resource with the supplied local href and identity.
    fn context_receipt(kind: &str) -> Value {
        json!({
            "uuid": RISK,
            "title": "Context receipt",
            "props": [property("context-kind", kind), property("root-uuid", RESULT),
                property("document-version", "1.0.0"), property("oscal-version", "1.2.3")],
            "rlinks": [{"href": "companion.json", "hashes": [{"algorithm": "SHA-256", "value": "a".repeat(64)}]}]
        })
    }

    /// Verify that supported profile requires complete unique context receipts.
    #[test]
    fn supported_profile_requires_complete_unique_context_receipts() {
        let missing = json!({"assessment-results": {
            "uuid": RESULT, "metadata": {"props": [], "roles": [], "parties": []},
            "import-ap": {"href": "ap.json"}, "results": []
        }});
        let failure = validate_supported_structure(&missing).unwrap_err().to_string();
        assert!(failure.contains("requires all four context receipts"));
        let complete = json!({"resources": [context_receipt("assessment-plan"),
            context_receipt("system-security-plan"), context_receipt("profile"), context_receipt("catalog")]});
        assert!(validate_back_matter(&complete).is_ok());
        let partial = json!({"resources": [context_receipt("assessment-plan")]});
        assert!(validate_back_matter(&partial).is_err());
        let duplicate = json!({"resources": [context_receipt("assessment-plan"),
            context_receipt("system-security-plan"), context_receipt("profile"), context_receipt("profile")]});
        assert!(validate_back_matter(&duplicate).is_err());
    }
}
