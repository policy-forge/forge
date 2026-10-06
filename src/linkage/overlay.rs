//! Derive a complete association overlay from one actual confined linkage capture.
//!
//! Only newly added resources use the closed metadata profile. Original native
//! content, including pre-existing embedded content, remains sensitive and exact.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::ForgeError;
use crate::cli::{OutputFormat, export};
use crate::evidence_capture::{CaptureProof, CaptureRole, CaptureSession, ProjectionBudget};

use super::{EvidenceRecord, EvidenceReference, LinkRecord, LinkageIndex, ResourceEvidence};

/// Versioned generated metadata profile, separate from the native linkage schema.
const PROFILE: &str = "forge.linkage-overlay/1";
/// Exact property namespace; generated values confer association metadata only.
const PROPERTY_NS: &str = "https://policy-forge.github.io/ns/linkage-overlay/1";
/// Separate resource identity namespace, never reusing native linkage ID domains.
const RESOURCE_SEED: &str = "forge.linkage-overlay/1:resource";
/// Whole encoded native output cap, including escaping and the final newline.
const MAX_OUTPUT: usize = 50 * 1024 * 1024;
/// Complete compact property bound; cumulative admission uses the shared budget.
const MAX_RECORD: usize = crate::evidence_capture::MAX_PROJECTION_BYTES;

/// Complete private output plus its actual original-generation authority.
pub(crate) struct PreparedOverlay {
    /// Whole bounded output retained only after validation and preservation.
    bytes: Vec<u8>,
    /// Actual proof moved from the sole capture; no detached constructor exists.
    proof: CaptureProof,
    /// Qualified root used by the caller's existing no-replacement publisher.
    root: PathBuf,
}

impl PreparedOverlay {
    /// Borrow only the complete schema-validated derived artifact.
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Borrow the actual qualified publication root without inventing a new base.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Recheck every original, ancestor, directory and actual absence generation.
    /// # Errors
    /// Returns a fixed linkage refusal when any actual generation is no longer safe.
    pub(crate) fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.proof.verify_inputs().map_err(|_| error("overlay original generations changed"))
    }

    /// Iterate actual held input observations for the caller's final collision check.
    pub(crate) fn input_paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.proof.input_paths()
    }
}

/// Selected original declaration and freshly revalidated native source evidence.
struct Target<'a> {
    /// Exact declared source side, which also domains stable UUIDs.
    side: &'static str,
    /// Native model key admitted for this target, not any other supported source.
    model: &'static str,
    /// Actual root-relative declared artifact, never a sanitized href substitute.
    relative: PathBuf,
    /// Source identities produced from original captured bytes by native preparation.
    source: &'a ResourceEvidence,
}

/// Exact newly generated resource without optional native prose/content fields.
#[derive(Serialize)]
struct ResourceView<'a> {
    /// Canonical lowercase v5 resource identity.
    uuid: &'a str,
    /// Fixed profile then compact record properties, in deterministic order.
    props: [PropertyView<'a>; 2],
}

/// One closed native property wrapper; no UUID, remarks or arbitrary fields.
#[derive(Serialize)]
struct PropertyView<'a> {
    /// Literal `NCName` from the closed property profile.
    name: &'static str,
    /// Literal absolute URI shared by these exact generated properties.
    ns: &'static str,
    /// Literal profile or complete compact JSON, never pretty physical newlines.
    value: &'a str,
}

/// Borrowed target identity, preserving original source UUID spelling and versions.
#[derive(Serialize)]
struct TargetView<'a> {
    /// Exact declared stable source key.
    resource_key: &'a str,
    /// Requirement or implementation role, without new workspace authority.
    side: &'static str,
    /// Catalog or Component Definition native model.
    model: &'static str,
    /// Exact actual original root UUID spelling.
    root_uuid: &'a str,
    /// Original complete file bytes, distinct from subject fingerprints.
    raw_sha256: &'a str,
    /// Original explicit native document version.
    document_version: &'a str,
    /// Original declared OSCAL version admitted by the existing validator.
    oscal_version: &'a str,
}

/// Complete source provenance row without source href or filesystem paths.
#[derive(Serialize)]
struct SourceView<'a> {
    /// Exact side of the native source declaration.
    side: &'static str,
    /// Exact declared source key.
    resource_key: &'a str,
    /// Existing supported source model; it does not broaden overlay targets.
    model: &'a str,
    /// Exact native root identity from actual captured original bytes.
    root_uuid: &'a str,
    /// Complete original-byte hash.
    raw_sha256: &'a str,
    /// Exact native document version.
    document_version: &'a str,
    /// Exact native OSCAL version.
    oscal_version: &'a str,
    /// Required nullable Profile companion hash; no omitted unknown field.
    resolved_catalog_sha256: Option<&'a str>,
}

/// One complete profile provenance record, with no assessment or owner inference.
#[derive(Serialize)]
struct ProvenanceView<'a> {
    /// Closed record discriminator.
    kind: &'static str,
    /// Exact generated profile version.
    schema_version: &'static str,
    /// Actual declared project identity.
    project_key: &'a str,
    /// Explicit admitted CLI date, never the wall clock.
    as_of: NaiveDate,
    /// Original once-captured manifest hash.
    manifest_sha256: &'a str,
    /// Exact selected original target generation.
    target: TargetView<'a>,
    /// Complete ordered native source identities, without sanitized path guessing.
    sources: Vec<SourceView<'a>>,
    /// Complete freshly produced link denominator.
    link_count: usize,
    /// Complete freshly produced evidence denominator, including unused records.
    evidence_count: usize,
    /// Fixed limit on interpretation of these generated records.
    authority: &'static str,
}

/// Exact link-to-evidence association; source key and generated UUID stay paired.
#[derive(Serialize)]
struct EvidencePairView<'a> {
    /// Actual declared evidence key.
    key: &'a str,
    /// Canonical separately generated resource identity.
    resource_uuid: String,
}

/// Explicit recorded review metadata, not a new approval or reviewer assignment.
#[derive(Serialize)]
struct ReviewView<'a> {
    /// Existing explicit native reviewer key, without human name or rationale.
    reviewer_key: &'a str,
    /// Existing recorded review timestamp, copied without normalization.
    reviewed_at: &'a str,
}

/// Complete many-to-many link record borrowing already admitted subject arrays.
#[derive(Serialize)]
struct LinkView<'a> {
    /// Closed record discriminator.
    kind: &'static str,
    /// The single generated provenance resource.
    provenance_uuid: &'a str,
    /// Exact stable link key.
    link_key: &'a str,
    /// Existing native linkage UUID, with its original namespace unchanged.
    link_id: &'a str,
    /// Complete exact requirement tuples; never a Cartesian expansion.
    requirements: &'a [super::SubjectRecord],
    /// Complete exact implementation tuples.
    implementations: &'a [super::SubjectRecord],
    /// Complete exact evidence membership with separately generated references.
    evidence: Vec<EvidencePairView<'a>>,
    /// Explicit author declaration, not inferred evidence sufficiency.
    evidence_required: bool,
    /// Explicit recorded implementation status, never computed from freshness.
    recorded_implementation_status: super::manifest::ImplementationStatus,
    /// Original explicit review fields, not new approval.
    recorded_review: ReviewView<'a>,
}

/// Closed local or URI metadata; no path, URI value, content or excerpt is emitted.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ReferenceView<'a> {
    /// Actual local byte observation and the explicit original approved tuple.
    Local {
        /// Declared full original evidence hash.
        approved_sha256: &'a str,
        /// Declared u64 size, which can differ from the actual bounded observation.
        approved_size: u64,
        /// Actual original byte hash or explicit absence.
        observed_sha256: Option<&'a str>,
        /// Actual bounded byte size or explicit absence.
        observed_size: Option<u64>,
    },
    /// URI metadata only; it is never fetched or converted into local authority.
    Uri {
        /// Required nullable explicit expected hash; the URI itself is omitted.
        expected_sha256: Option<&'a str>,
    },
}

/// Complete evidence record preserving actual native freshness and null facts.
#[derive(Serialize)]
struct EvidenceView<'a> {
    /// Closed record discriminator.
    kind: &'static str,
    /// Single generated provenance resource identity.
    provenance_uuid: &'a str,
    /// Exact explicit evidence key.
    evidence_key: &'a str,
    /// Existing native observation state, never an assessment conclusion.
    freshness: &'a super::EvidenceFreshness,
    /// Exact native date serialization or explicit null, not new date authority.
    recorded_valid_through: Option<NaiveDate>,
    /// Closed minimized local or URI observation metadata.
    reference: ReferenceView<'a>,
}

/// Precise container-presence bookkeeping for removing only the generated suffix.
struct AppendShape {
    /// Whether the original model root had a back-matter member.
    had_back_matter: bool,
    /// Whether original back-matter had its resources array.
    had_resources: bool,
    /// Exact original resource prefix length, independent of UUID selection.
    original_count: usize,
    /// Exact complete generated suffix length.
    generated_count: usize,
}

/// Consumed closed generated schemas, compiled once for the actual preparation.
struct ProfileValidators {
    /// Closed decoded record schema; all references are local definitions.
    record: jsonschema::Validator,
    /// Exact two-property resource schema, separate from original native resources.
    resource: jsonschema::Validator,
}

/// Prepare one complete schema-valid, preserved target from actual captured originals.
///
/// The capture is shared once; selection and unchanged fresh preparation separately
/// bounded-parse the same held manifest. Output stays beside the selected source.
/// # Errors
/// Refuses invalid, unsafe, changed, unsupported, ambiguous, colliding or overbound
/// inputs before returning complete bytes. Publication belongs to the caller.
pub(crate) fn prepare(
    manifest_path: &Path,
    target_key: &str,
    as_of: &str,
    output_relative: &Path,
) -> Result<PreparedOverlay, ForgeError> {
    let as_of = canonical_date(as_of)?;
    if manifest_path.as_os_str().is_empty()
        || !super::has_normalized_path_spelling(manifest_path)
        || manifest_path.components().any(|part| {
            matches!(part, std::path::Component::CurDir | std::path::Component::ParentDir)
        })
    {
        return Err(error("overlay manifest needs its original normalized path spelling"));
    }
    let parent =
        manifest_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let root = std::path::absolute(parent).map_err(|_| error("overlay root is unavailable"))?;
    let manifest_relative = Path::new(
        manifest_path.file_name().ok_or_else(|| error("overlay manifest needs a filename"))?,
    );
    let mut capture = CaptureSession::new(&root).map_err(|_| error("overlay root is unsafe"))?;
    preflight_output(capture.root(), output_relative)?;
    capture
        .reserve_output(output_relative)
        .map_err(|_| error("overlay output namespace is invalid"))?;
    let lease = capture
        .required(
            manifest_relative,
            CaptureRole::LinkageManifest,
            super::manifest::MAX_MANIFEST_BYTES,
        )
        .map_err(|_| error("overlay manifest could not be captured"))?;
    let manifest =
        super::manifest::parse(lease.bytes()).map_err(|_| error("overlay manifest is invalid"))?;
    let manifest_sha = crate::hashing::sha256_hex(lease.bytes());
    let mut projection = ProjectionBudget::new();
    let fresh = super::fresh::prepare_fresh(
        manifest_relative,
        &manifest_sha,
        as_of,
        &mut capture,
        &mut projection,
    )
    .map_err(|_| error("overlay complete linkage inputs are invalid"))?;
    let target = select_target(&manifest, fresh.index(), target_key)?;
    if output_relative.parent().unwrap_or(Path::new(""))
        != target.relative.parent().unwrap_or(Path::new(""))
    {
        return Err(error("overlay output must share the original target directory"));
    }
    let role = if target.model == "catalog" {
        CaptureRole::Catalog
    } else {
        CaptureRole::ComponentDefinition
    };
    let original = capture
        .captured(&target.relative, role)
        .map_err(|_| error("overlay target is not its actual held original"))?;
    let (mut derived, model) = decode_native(original)?;
    check_target(&derived, &target, original)?;
    export::validate_oscal_json_value(&derived, model)
        .map_err(|_| error("overlay original native validation failed"))?;
    let mut original_ids = BTreeSet::new();
    collect_original_uuids(&derived, &mut capture, &mut original_ids)?;
    let validators = profile_validators()?;
    let generated = generated_resources(
        fresh.index(),
        &target,
        &mut capture,
        &mut projection,
        &original_ids,
        &validators,
    )?;
    let shape = append_resources(&mut derived, target.model, generated)?;
    let bytes = encode_bounded(&derived, MAX_OUTPUT, true)?;
    let (mut reparsed, parsed_model) = decode_native(&bytes)?;
    if parsed_model != model || reparsed != derived {
        return Err(error("overlay complete serialization changed its expected tree"));
    }
    export::validate_oscal_json_value(&reparsed, parsed_model)
        .map_err(|_| error("overlay derived native validation failed"))?;
    let original = capture
        .captured(&target.relative, role)
        .map_err(|_| error("overlay original target is no longer held"))?;
    let (original_value, original_model) = decode_native(original)?;
    if original_model != model {
        return Err(error("overlay original target model changed"));
    }
    remove_generated(&mut reparsed, target.model, &shape)?;
    if reparsed != original_value {
        return Err(error("overlay full original tree preservation failed"));
    }
    let root = capture.root().to_path_buf();
    let proof = capture.finish();
    proof.verify_inputs().map_err(|_| error("overlay originals changed during preparation"))?;
    Ok(PreparedOverlay { bytes, proof, root })
}

/// Admit one literal canonical calendar date without signed/expanded years or clock fallback.
fn canonical_date(value: &str) -> Result<NaiveDate, ForgeError> {
    if value.len() != 10
        || value
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(i, b)| if matches!(i, 4 | 7) { *b != b'-' } else { !b.is_ascii_digit() })
    {
        return Err(error("overlay as-of must be a canonical calendar date"));
    }
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| error("overlay as-of is invalid"))?;
    if date.format("%Y-%m-%d").to_string() != value {
        return Err(error("overlay as-of is not canonical"));
    }
    Ok(date)
}

/// Qualify an existing output parent and an absent portable destination before captures.
fn preflight_output(root: &Path, relative: &Path) -> Result<(), ForgeError> {
    let text = relative.to_str().ok_or_else(|| error("overlay output must be UTF8"))?;
    crate::authoring::output::validate_relative(text)
        .map_err(|_| error("overlay output is not portable"))?;
    if relative.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
        return Err(error("overlay output must have the exact JSON suffix"));
    }
    let parent = root.join(relative.parent().unwrap_or(Path::new("")));
    super::fresh::qualify_root(&parent)
        .map_err(|_| error("overlay output parent is unsafe or unavailable"))?;
    match std::fs::symlink_metadata(root.join(relative)) {
        Err(cause) if cause.kind() == io::ErrorKind::NotFound => Ok(()),
        _ => Err(error("overlay output already exists or cannot be qualified")),
    }
}

/// Select exactly one declared source key and its actual native identity without href guessing.
fn select_target<'a>(
    manifest: &super::manifest::LinkageManifest,
    index: &'a LinkageIndex,
    key: &str,
) -> Result<Target<'a>, ForgeError> {
    let requirements: Vec<_> =
        manifest.requirement_resources.iter().filter(|r| r.key == key).collect();
    let implementation = manifest.implementation_resource.key == key;
    if requirements.len() + usize::from(implementation) != 1 {
        return Err(error("overlay target key is absent or ambiguous"));
    }
    let target = if let Some(resource) = requirements.first() {
        if resource.resource_type != crate::mapping::manifest::ResourceType::Catalog {
            return Err(error("overlay requirement target must be a declared Catalog"));
        }
        let mut matching = index.provenance.requirement_resources.iter().filter(|r| r.key == key);
        let source =
            matching.next().ok_or_else(|| error("overlay target has no fresh source identity"))?;
        if matching.next().is_some()
            || source.resource_type != "catalog"
            || source.raw_sha256 != resource.expected_sha256
        {
            return Err(error("overlay target source declaration is inconsistent"));
        }
        Target {
            side: "requirement",
            model: "catalog",
            relative: resource.artifact.clone(),
            source,
        }
    } else {
        let resource = &manifest.implementation_resource;
        let source = &index.provenance.implementation_resource;
        if resource.resource_type
            != super::manifest::ImplementationResourceType::ComponentDefinition
            || source.key != key
            || source.resource_type != "component-definition"
            || source.raw_sha256 != resource.expected_sha256
        {
            return Err(error(
                "overlay implementation target must be its declared Component Definition",
            ));
        }
        Target {
            side: "implementation",
            model: "component-definition",
            relative: resource.artifact.clone(),
            source,
        }
    };
    let participates = index.links.iter().any(|link| {
        let subjects =
            if target.side == "requirement" { &link.requirements } else { &link.implementations };
        subjects.iter().any(|s| s.resource_key == key && s.side == target.side)
    });
    if !participates {
        return Err(error("overlay target has no exact linked subject tuple"));
    }
    Ok(target)
}

/// Decode complete native JSON using the unchanged exporter integer-only predicate.
fn decode_native(bytes: &[u8]) -> Result<(Value, crate::validate::OscalModelType), ForgeError> {
    if bytes.len() > MAX_OUTPUT {
        return Err(error("overlay complete native bytes exceed their bound"));
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| error("overlay native bytes are not UTF8"))?;
    export::export_json_value(text, OutputFormat::Json)
        .map_err(|_| error("overlay native JSON is invalid or unsupported"))
}

/// Match actual selected original bytes/model/root/version against freshly consumed provenance.
fn check_target(value: &Value, target: &Target<'_>, original: &[u8]) -> Result<(), ForgeError> {
    let root = value
        .get(target.model)
        .and_then(Value::as_object)
        .ok_or_else(|| error("overlay selected target model differs"))?;
    let metadata = root
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| error("overlay selected target metadata is missing"))?;
    if root.get("uuid").and_then(Value::as_str) != Some(target.source.root_uuid.as_str())
        || metadata.get("version").and_then(Value::as_str)
            != Some(target.source.document_version.as_str())
        || metadata.get("oscal-version").and_then(Value::as_str)
            != Some(target.source.oscal_version.as_str())
        || crate::hashing::sha256_hex(original) != target.source.raw_sha256
    {
        return Err(error("overlay actual target identity is inconsistent"));
    }
    Ok(())
}

/// Count every complete original UUID-valued string before full-tree set insertion.
fn collect_original_uuids(
    value: &Value,
    capture: &mut CaptureSession,
    found: &mut BTreeSet<Uuid>,
) -> Result<(), ForgeError> {
    match value {
        Value::String(text) if matches!(text.len(), 32 | 36 | 38 | 45) => {
            if let Ok(id) = Uuid::try_parse(text) {
                capture
                    .relationships(1)
                    .map_err(|_| error("overlay complete UUID occurrence bound exceeded"))?;
                found.insert(id);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_original_uuids(child, capture, found)?;
            }
        }
        Value::Object(values) => {
            for child in values.values() {
                collect_original_uuids(child, capture, found)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Derive a resource UUID from exact byte-framed stable keys, never generation or path fields.
fn resource_id(
    project: &str,
    target: &Target<'_>,
    kind: &str,
    key: &str,
) -> Result<Uuid, ForgeError> {
    let root = Uuid::try_parse(&target.source.root_uuid)
        .map_err(|_| error("overlay target root UUID is invalid"))?
        .to_string();
    let namespace = Uuid::new_v5(&Uuid::NAMESPACE_URL, RESOURCE_SEED.as_bytes());
    Ok(Uuid::new_v5(
        &namespace,
        &super::stable_bytes(&[
            PROFILE,
            project,
            target.side,
            &target.source.key,
            target.model,
            &root,
            kind,
            key,
        ]),
    ))
}

/// Admit every separately generated resource identity before UUID-map or resource growth.
fn admit_id(
    id: Uuid,
    original: &BTreeSet<Uuid>,
    generated: &mut BTreeSet<Uuid>,
    capture: &mut CaptureSession,
) -> Result<(), ForgeError> {
    capture.relationships(1).map_err(|_| error("overlay generated resource bound exceeded"))?;
    if original.contains(&id) || !generated.insert(id) {
        return Err(error("overlay resource UUID collision"));
    }
    Ok(())
}

/// Borrow one complete source row without copying paths or unbounded author metadata.
fn source_view<'a>(source: &'a ResourceEvidence, side: &'static str) -> SourceView<'a> {
    SourceView {
        side,
        resource_key: &source.key,
        model: &source.resource_type,
        root_uuid: &source.root_uuid,
        raw_sha256: &source.raw_sha256,
        document_version: &source.document_version,
        oscal_version: &source.oscal_version,
        resolved_catalog_sha256: source.resolved_catalog_sha256.as_deref(),
    }
}

/// Charge complete borrowed source metadata before each corresponding row is retained.
fn source_rows<'a>(
    index: &'a LinkageIndex,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<Vec<SourceView<'a>>, ForgeError> {
    let mut rows = Vec::new();
    for source in &index.provenance.requirement_resources {
        let row = source_view(source, "requirement");
        capture.relationships(1).map_err(|_| error("overlay source projection bound exceeded"))?;
        projection
            .admit_row(&row)
            .map_err(|_| error("overlay source metadata exceeds its bound"))?;
        rows.push(row);
    }
    rows.sort_by(|a, b| a.resource_key.cmp(b.resource_key));
    let row = source_view(&index.provenance.implementation_resource, "implementation");
    capture.relationships(1).map_err(|_| error("overlay source projection bound exceeded"))?;
    projection.admit_row(&row).map_err(|_| error("overlay source metadata exceeds its bound"))?;
    rows.push(row);
    Ok(rows)
}

/// Produce one complete compact property and escaped resource only after shared admission.
fn encode_resource<T: Serialize>(
    id: Uuid,
    record: &T,
    projection: &mut ProjectionBudget,
    validators: &ProfileValidators,
) -> Result<Value, ForgeError> {
    projection
        .admit_row(record)
        .map_err(|_| error("overlay record metadata exceeds its shared bound"))?;
    let compact = String::from_utf8(encode_bounded(record, MAX_RECORD, false)?)
        .map_err(|_| error("overlay record encoding is invalid"))?;
    if compact.contains(['\r', '\n']) {
        return Err(error("overlay record contains a physical line break"));
    }
    let decoded = strict_generated(compact.as_bytes())?;
    if !validators.record.is_valid(&decoded) {
        return Err(error("overlay generated record violates its closed profile"));
    }
    let identity = id.to_string();
    let resource = ResourceView {
        uuid: &identity,
        props: [
            PropertyView { name: "profile", ns: PROPERTY_NS, value: PROFILE },
            PropertyView { name: "record", ns: PROPERTY_NS, value: &compact },
        ],
    };
    projection
        .admit_row(&resource)
        .map_err(|_| error("overlay escaped resource exceeds its shared bound"))?;
    let bytes = encode_bounded(&resource, MAX_RECORD, false)?;
    let encoded = strict_generated(&bytes)?;
    validate_encoded_resource(&encoded, &identity, &compact, &decoded, validators)?;
    Ok(encoded)
}

/// Consume both closed schemas and match every allowed native field to its actual expected record.
fn validate_encoded_resource(
    resource: &Value,
    identity: &str,
    compact: &str,
    expected_record: &Value,
    validators: &ProfileValidators,
) -> Result<(), ForgeError> {
    if !validators.resource.is_valid(resource) || !validators.record.is_valid(expected_record) {
        return Err(error("overlay generated resource violates its closed profile"));
    }
    let props = resource
        .get("props")
        .and_then(Value::as_array)
        .ok_or_else(|| error("overlay generated native properties are invalid"))?;
    if resource.get("uuid").and_then(Value::as_str) != Some(identity)
        || props.len() != 2
        || props[0].get("name").and_then(Value::as_str) != Some("profile")
        || props[0].get("ns").and_then(Value::as_str) != Some(PROPERTY_NS)
        || props[0].get("value").and_then(Value::as_str) != Some(PROFILE)
        || props[1].get("name").and_then(Value::as_str) != Some("record")
        || props[1].get("ns").and_then(Value::as_str) != Some(PROPERTY_NS)
        || props[1].get("value").and_then(Value::as_str) != Some(compact)
        || strict_generated(compact.as_bytes())? != *expected_record
    {
        return Err(error(
            "overlay generated native fields differ from their complete expected record",
        ));
    }
    Ok(())
}

/// Borrow exact local observation pairs or URI metadata without revealing locations/content.
fn evidence_view<'a>(
    record: &'a EvidenceRecord,
    provenance: &'a str,
) -> Result<EvidenceView<'a>, ForgeError> {
    use super::EvidenceFreshness;
    let reference = match &record.reference {
        EvidenceReference::Local {
            approved_sha256,
            approved_size,
            observed_sha256,
            observed_size,
            ..
        } => {
            if observed_sha256.is_some() != observed_size.is_some()
                || (record.freshness == EvidenceFreshness::Unavailable) != observed_sha256.is_none()
                || record.freshness == EvidenceFreshness::UnverifiedUri
            {
                return Err(error("overlay local observation fields are inconsistent"));
            }
            let equal = observed_sha256.as_deref() == Some(approved_sha256.as_str())
                && *observed_size == Some(*approved_size);
            if matches!(
                record.freshness,
                EvidenceFreshness::Current
                    | EvidenceFreshness::Expiring
                    | EvidenceFreshness::Expired
            ) && !equal
                || record.freshness == EvidenceFreshness::Changed && equal
            {
                return Err(error("overlay local observation state is inconsistent"));
            }
            ReferenceView::Local {
                approved_sha256,
                approved_size: *approved_size,
                observed_sha256: observed_sha256.as_deref(),
                observed_size: *observed_size,
            }
        }
        EvidenceReference::Uri { expected_sha256, .. } => {
            if record.freshness != EvidenceFreshness::UnverifiedUri {
                return Err(error("overlay URI observation state is inconsistent"));
            }
            ReferenceView::Uri { expected_sha256: expected_sha256.as_deref() }
        }
    };
    Ok(EvidenceView {
        kind: "evidence",
        provenance_uuid: provenance,
        evidence_key: &record.key,
        freshness: &record.freshness,
        recorded_valid_through: record.valid_through,
        reference,
    })
}

/// Retain complete evidence references and charge every repeated subject edge before projection.
fn link_view<'a>(
    link: &'a LinkRecord,
    provenance: &'a str,
    ids: &BTreeMap<&str, Uuid>,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<LinkView<'a>, ForgeError> {
    capture
        .relationships(link.requirements.len())
        .and_then(|()| capture.relationships(link.implementations.len()))
        .map_err(|_| error("overlay complete subject tuple bound exceeded"))?;
    let mut evidence = Vec::new();
    for key in &link.evidence_keys {
        let id = ids
            .get(key.as_str())
            .ok_or_else(|| error("overlay complete evidence membership is inconsistent"))?;
        capture
            .relationships(1)
            .map_err(|_| error("overlay complete evidence association bound exceeded"))?;
        let row = EvidencePairView { key, resource_uuid: id.to_string() };
        projection
            .admit_row(&row)
            .map_err(|_| error("overlay evidence association metadata exceeds its bound"))?;
        evidence.push(row);
    }
    evidence.sort_by(|a, b| a.key.cmp(b.key));
    Ok(LinkView {
        kind: "link",
        provenance_uuid: provenance,
        link_key: &link.key,
        link_id: &link.link_id,
        requirements: &link.requirements,
        implementations: &link.implementations,
        evidence,
        evidence_required: link.evidence_required,
        recorded_implementation_status: link.implementation_status,
        recorded_review: ReviewView {
            reviewer_key: &link.reviewer_key,
            reviewed_at: &link.reviewed_at,
        },
    })
}

/// Generate the entire source-bound project set without filtered or partial-prefix admission.
fn generated_resources(
    index: &LinkageIndex,
    target: &Target<'_>,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
    original: &BTreeSet<Uuid>,
    validators: &ProfileValidators,
) -> Result<Vec<Value>, ForgeError> {
    let count = index
        .links
        .len()
        .checked_add(index.evidence.len())
        .and_then(|n| n.checked_add(1))
        .filter(|n| *n <= 20_001)
        .ok_or_else(|| error("overlay complete resource cardinality is invalid"))?;
    let mut generated_ids = BTreeSet::new();
    let provenance_id = resource_id(&index.project_key, target, "provenance", "provenance")?;
    admit_id(provenance_id, original, &mut generated_ids, capture)?;
    let mut evidence_ids = BTreeMap::new();
    for record in &index.evidence {
        let id = resource_id(&index.project_key, target, "evidence", &record.key)?;
        admit_id(id, original, &mut generated_ids, capture)?;
        if evidence_ids.insert(record.key.as_str(), id).is_some() {
            return Err(error("overlay evidence keys are ambiguous"));
        }
    }
    let mut links = BTreeMap::new();
    for link in &index.links {
        let id = resource_id(&index.project_key, target, "link", &link.key)?;
        admit_id(id, original, &mut generated_ids, capture)?;
        if links.insert(link.key.as_str(), (link, id)).is_some() {
            return Err(error("overlay link keys are ambiguous"));
        }
    }
    let provenance = provenance_id.to_string();
    let record = ProvenanceView {
        kind: "provenance",
        schema_version: PROFILE,
        project_key: &index.project_key,
        as_of: index.as_of,
        manifest_sha256: &index.provenance.manifest_sha256,
        target: TargetView {
            resource_key: &target.source.key,
            side: target.side,
            model: target.model,
            root_uuid: &target.source.root_uuid,
            raw_sha256: &target.source.raw_sha256,
            document_version: &target.source.document_version,
            oscal_version: &target.source.oscal_version,
        },
        sources: source_rows(index, capture, projection)?,
        link_count: index.links.len(),
        evidence_count: index.evidence.len(),
        authority: "association-only",
    };
    let mut resources = Vec::new();
    resources.push(encode_resource(provenance_id, &record, projection, validators)?);
    for (_, (link, id)) in links {
        let record = link_view(link, &provenance, &evidence_ids, capture, projection)?;
        resources.push(encode_resource(id, &record, projection, validators)?);
    }
    let evidence_by_key: BTreeMap<_, _> =
        index.evidence.iter().map(|record| (record.key.as_str(), record)).collect();
    if evidence_by_key.len() != index.evidence.len() {
        return Err(error("overlay complete evidence keys are inconsistent"));
    }
    for (key, id) in evidence_ids {
        let record = evidence_view(evidence_by_key[key], &provenance)?;
        resources.push(encode_resource(id, &record, projection, validators)?);
    }
    if resources.len() != count || generated_ids.len() != count {
        return Err(error("overlay complete resource count is inconsistent"));
    }
    Ok(resources)
}

/// Append only a complete admitted generated suffix and retain exact old container presence.
fn append_resources(
    value: &mut Value,
    model: &str,
    generated: Vec<Value>,
) -> Result<AppendShape, ForgeError> {
    let root = value
        .get_mut(model)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| error("overlay model root is invalid"))?;
    let had_back_matter = root.contains_key("back-matter");
    let back = root
        .entry("back-matter")
        .or_insert_with(|| Value::Object(serde_json::Map::new()))
        .as_object_mut()
        .ok_or_else(|| error("overlay original back-matter is invalid"))?;
    let had_resources = back.contains_key("resources");
    let resources = back
        .entry("resources")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| error("overlay original resources are invalid"))?;
    let original_count = resources.len();
    let generated_count = generated.len();
    original_count
        .checked_add(generated_count)
        .ok_or_else(|| error("overlay resource length overflows"))?;
    resources.extend(generated);
    Ok(AppendShape { had_back_matter, had_resources, original_count, generated_count })
}

/// Remove exactly the expected suffix, restoring only containers that did not originally exist.
fn remove_generated(value: &mut Value, model: &str, shape: &AppendShape) -> Result<(), ForgeError> {
    let root = value
        .get_mut(model)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| error("overlay round-trip model is invalid"))?;
    let back = root
        .get_mut("back-matter")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| error("overlay round-trip back-matter is invalid"))?;
    let resources = back
        .get_mut("resources")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| error("overlay round-trip resources are invalid"))?;
    if resources.len()
        != shape
            .original_count
            .checked_add(shape.generated_count)
            .ok_or_else(|| error("overlay round-trip count overflows"))?
    {
        return Err(error("overlay round-trip suffix count differs"));
    }
    resources.truncate(shape.original_count);
    if !shape.had_resources {
        back.remove("resources");
    }
    if !shape.had_back_matter {
        if !back.is_empty() {
            return Err(error("overlay new container has unexpected members"));
        }
        root.remove("back-matter");
    }
    Ok(())
}

/// Bounded private byte sink that rejects before each append, not after a full allocation.
struct LimitedWriter {
    /// Sole complete output owner; no partial result is returned on a serializer fault.
    bytes: Vec<u8>,
    /// Actual encoded-byte ceiling, including pretty output and its final newline.
    limit: usize,
}

impl io::Write for LimitedWriter {
    /// Check exact byte arithmetic before every private buffer growth.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other("overlay encoded bound"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    /// Flush a private memory sink without performing external IO.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encode a complete borrowed view or native tree through a consumed capped writer.
fn encode_bounded<T: Serialize>(
    value: &T,
    limit: usize,
    pretty: bool,
) -> Result<Vec<u8>, ForgeError> {
    let mut writer = LimitedWriter { bytes: Vec::new(), limit };
    if pretty {
        serde_json::to_writer_pretty(&mut writer, value)
    } else {
        serde_json::to_writer(&mut writer, value)
    }
    .map_err(|_| error("overlay complete encoded output exceeds its bound"))?;
    if pretty {
        writer
            .write_all(b"\n")
            .map_err(|_| error("overlay complete encoded output exceeds its bound"))?;
    }
    Ok(writer.bytes)
}

/// Strict-decode a complete bounded generated wrapper or compact property record.
fn strict_generated(bytes: &[u8]) -> Result<Value, ForgeError> {
    if bytes.len() > MAX_RECORD {
        return Err(error("overlay generated JSON exceeds its bound"));
    }
    crate::json_strict::parse_value(
        bytes,
        "generated overlay",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: MAX_RECORD },
    )
    .map_err(|_| error("overlay generated JSON is invalid"))
}

/// Compile the exact shipped local schemas without any external schema reference.
fn profile_validators() -> Result<ProfileValidators, ForgeError> {
    let record = strict_generated(include_bytes!(
        "../../schemas/forge.linkage-overlay-record-1.schema.json"
    ))?;
    let resource = strict_generated(include_bytes!(
        "../../schemas/forge.linkage-overlay-resource-1.schema.json"
    ))?;
    let record = jsonschema::validator_for(&record)
        .map_err(|_| error("overlay record schema is invalid"))?;
    let resource = jsonschema::validator_for(&resource)
        .map_err(|_| error("overlay resource schema is invalid"))?;
    Ok(ProfileValidators { record, resource })
}

/// Emit fixed stage text without paths, author prose, raw errors or source contents.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::Linkage(reason.to_string())
}

/// Proposed native-domain and bounded projection controls; runtime results are separately bound.
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Owned synthetic rich native files, declaration and current evidence generation.
    struct Fixture {
        /// Retain the actual private directory for every capture proof lifetime.
        _dir: tempfile::TempDir,
        /// Actual canonical test root, separate from publication-relative paths.
        root: PathBuf,
        /// Original complete declaration consumed by the producer.
        manifest: PathBuf,
        /// Actual local evidence file used by native freshness observation.
        evidence: PathBuf,
    }

    /// Write complete fixture JSON without running a CLI or fabricating a capture proof.
    fn write_json(path: &Path, value: &Value) {
        std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }

    /// Read one complete synthetic JSON fixture for controlled declaration changes.
    fn read_json(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    /// Create actual rich native originals and an explicit two-by-two reviewed association.
    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("native")).unwrap();
        std::fs::create_dir(root.join("evidence")).unwrap();
        let catalog = include_bytes!("../../tests/fixtures/export/lossless-catalog.json");
        let component = include_bytes!("../../tests/fixtures/export/lossless-component.json");
        std::fs::write(root.join("native/catalog.json"), catalog).unwrap();
        std::fs::write(root.join("native/component.json"), component).unwrap();
        let evidence = root.join("evidence/record.bin");
        std::fs::write(&evidence, b"synthetic private body\n").unwrap();
        let manifest = root.join("links.json");
        write_json(
            &manifest,
            &json!({
                "schema_version":"forge.linkage/1",
                "project":{"key":"overlay-test","title":"Synthetic overlay project","expiring_window_days":30,"max_evidence_bytes":1_048_576,"approved_uri_schemes":[]},
                "reviewers":[{"key":"reviewer","name":"Synthetic Reviewer"}],
                "requirement_resources":[{"key":"catalog-main","type":"catalog","artifact":"native/catalog.json","href":"native/catalog.json","expected_sha256":crate::hashing::sha256_hex(catalog)}],
                "implementation_resource":{"key":"component-main","type":"component-definition","artifact":"native/component.json","href":"native/component.json","expected_sha256":crate::hashing::sha256_hex(component)},
                "evidence_roots":[{"key":"local","path":"evidence"}],
                "evidence":[{"key":"record","title":"Sensitive fixture label","evidence_type":"synthetic-record","owner":"explicit-owner","collected_at":"2026-10-01T12:00:00Z","valid_through":"2026-12-31","sensitivity_label":"restricted","source_label":"private source label","location":{"kind":"local","root_key":"local","path":"record.bin","expected_sha256":crate::hashing::sha256_hex(b"synthetic private body\n"),"expected_size":23}}],
                "links":[{"key":"many-to-many","requirements":[{"resource_key":"catalog-main","type":"control","id_ref":"EX-1"},{"resource_key":"catalog-main","type":"statement","id_ref":"EX-1_smt"}],"implementations":[{"type":"implemented-requirement","id_ref":"000000d4-abcd-4abc-8abc-000000000001"},{"type":"statement","id_ref":"000000d7-abcd-4abc-8abc-000000000001"}],"evidence_keys":["record"],"evidence_required":true,"responsible_role":"explicit-owner","implementation_status":"partial","review":{"reviewer_key":"reviewer","reviewed_at":"2026-10-03T12:00:00Z","rationale":"Private rationale with newline\nand tab\tcontent."}}]
            }),
        );
        Fixture { _dir: dir, root, manifest, evidence }
    }

    /// Prepare through the actual capture path with explicit deterministic date and destination.
    fn prepared(fixture: &Fixture, key: &str, output: &str) -> PreparedOverlay {
        prepare(&fixture.manifest, key, "2026-10-04", Path::new(output)).unwrap()
    }

    /// Borrow the original complete prefix length without making empty containers equivalent.
    fn original_count(value: &Value, model: &str) -> usize {
        value[model]
            .get("back-matter")
            .and_then(|v| v.get("resources"))
            .and_then(Value::as_array)
            .map_or(0, Vec::len)
    }

    /// Decode only newly generated embedded records after the untouched original prefix.
    fn records(prepared: &PreparedOverlay, original: &Value, model: &str) -> Vec<Value> {
        let derived: Value = serde_json::from_slice(prepared.bytes()).unwrap();
        derived[model]["back-matter"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .skip(original_count(original, model))
            .map(|resource| {
                strict_generated(resource["props"][1]["value"].as_str().unwrap().as_bytes())
                    .unwrap()
            })
            .collect()
    }

    /// Make an independent explicit native source identity for pure UUID framing controls.
    fn source() -> ResourceEvidence {
        ResourceEvidence {
            key: "catalog-main".into(),
            resource_type: "catalog".into(),
            href: "private.json".into(),
            raw_sha256: "a".repeat(64),
            root_uuid: "11111111-1111-4111-8111-111111111111".into(),
            document_version: "1".into(),
            oscal_version: "1.2.3".into(),
            resolved_catalog_sha256: None,
        }
    }

    /// Construct only an internal identity view; it never creates actual preparation authority.
    fn target(source: &ResourceEvidence) -> Target<'_> {
        Target { side: "requirement", model: "catalog", relative: "private.json".into(), source }
    }

    /// Complete native models retain every original tree field and the exact many-to-many tuples.
    #[test]
    fn rich_catalog_and_component_preserve_complete_trees_and_native_schemas() {
        let fixture = fixture();
        for (key, model, path, output) in [
            ("catalog-main", "catalog", "native/catalog.json", "native/catalog-overlay.json"),
            (
                "component-main",
                "component-definition",
                "native/component.json",
                "native/component-overlay.json",
            ),
        ] {
            let original = read_json(&fixture.root.join(path));
            let prepared = prepared(&fixture, key, output);
            let mut derived: Value = serde_json::from_slice(prepared.bytes()).unwrap();
            let generated = records(&prepared, &original, model);
            assert_eq!(generated.len(), 3);
            assert_eq!(generated[0]["target"]["resource_key"], key);
            assert_eq!(generated[0]["sources"].as_array().unwrap().len(), 2);
            assert_eq!(generated[1]["requirements"].as_array().unwrap().len(), 2);
            assert_eq!(generated[1]["implementations"].as_array().unwrap().len(), 2);
            assert_eq!(generated[1]["evidence"].as_array().unwrap().len(), 1);
            assert_eq!(generated[2]["freshness"], "current");
            let shape = AppendShape {
                had_back_matter: original[model].get("back-matter").is_some(),
                had_resources: original[model]
                    .get("back-matter")
                    .and_then(|v| v.get("resources"))
                    .is_some(),
                original_count: original_count(&original, model),
                generated_count: 3,
            };
            remove_generated(&mut derived, model, &shape).unwrap();
            assert_eq!(derived, original);
            assert!(prepared.verify_inputs().is_ok());
            assert!(!fixture.root.join(output).exists());
        }
    }

    /// Unused evidence remains complete while new records omit private paths, labels, rationale and bytes.
    #[test]
    fn complete_evidence_denominator_and_minimized_generated_metadata_are_preserved() {
        let fixture = fixture();
        let mut manifest = read_json(&fixture.manifest);
        let mut unused = manifest["evidence"][0].clone();
        unused["key"] = json!("unused-uri");
        unused["location"] = json!({"kind":"uri","uri":"https://user:secret@example.test/private?q=token","unverified":true});
        manifest["evidence"].as_array_mut().unwrap().push(unused);
        write_json(&fixture.manifest, &manifest);
        let original = read_json(&fixture.root.join("native/catalog.json"));
        let prepared = prepared(&fixture, "catalog-main", "native/out.json");
        let generated = records(&prepared, &original, "catalog");
        assert_eq!(generated.len(), 4);
        assert_eq!(generated[0]["evidence_count"], 2);
        assert_eq!(generated[3]["freshness"], "unverified-uri");
        assert!(generated[3]["reference"]["expected_sha256"].is_null());
        let text = serde_json::to_string(&generated).unwrap();
        for private in [
            "native/catalog.json",
            "record.bin",
            "synthetic private body",
            "Sensitive fixture label",
            "Private rationale",
            "explicit-owner",
            "secret",
            "q=token",
        ] {
            assert!(!text.contains(private), "unexpected generated metadata: {private}");
        }
    }

    /// Actual changed/expired/expiring/absent local generations remain distinct and do not renumber UUIDs.
    #[test]
    fn actual_freshness_and_absence_states_keep_stable_resource_identity() {
        let fixture = fixture();
        let initial = prepared(&fixture, "catalog-main", "native/out.json");
        let original = read_json(&fixture.root.join("native/catalog.json"));
        let initial_value: Value = serde_json::from_slice(initial.bytes()).unwrap();
        let offset = original_count(&original, "catalog");
        let id = initial_value["catalog"]["back-matter"]["resources"][offset + 2]["uuid"].clone();
        drop(initial);
        for (date, state) in [("2026-12-02", "expiring"), ("2027-01-01", "expired")] {
            let current =
                prepare(&fixture.manifest, "catalog-main", date, Path::new("native/out.json"))
                    .unwrap();
            let value: Value = serde_json::from_slice(current.bytes()).unwrap();
            assert_eq!(value["catalog"]["back-matter"]["resources"][offset + 2]["uuid"], id);
            assert_eq!(records(&current, &original, "catalog")[2]["freshness"], state);
        }
        std::fs::write(&fixture.evidence, b"changed private bytes!!").unwrap();
        let changed = prepared(&fixture, "catalog-main", "native/out.json");
        let record = &records(&changed, &original, "catalog")[2];
        assert_eq!(record["freshness"], "changed");
        assert_ne!(record["reference"]["approved_sha256"], record["reference"]["observed_sha256"]);
        drop(changed);
        std::fs::remove_file(&fixture.evidence).unwrap();
        let absent = prepared(&fixture, "catalog-main", "native/out.json");
        let record = &records(&absent, &original, "catalog")[2];
        assert_eq!(record["freshness"], "unavailable");
        assert!(record["reference"]["observed_sha256"].is_null());
        assert!(record["reference"]["observed_size"].is_null());
        std::fs::write(&fixture.evidence, b"synthetic private body\n").unwrap();
        assert!(absent.verify_inputs().is_err());
    }

    /// Every complete actual original remains rechecked, including a changed tail after preparation.
    #[test]
    fn retained_actual_original_change_refuses_final_publication_proof() {
        let fixture = fixture();
        let prepared = prepared(&fixture, "catalog-main", "native/out.json");
        let path = fixture.root.join("native/component.json");
        let mut bytes = std::fs::read(&path).unwrap();
        let last = bytes.len() - 1;
        bytes[last] = if bytes[last] == b'\n' { b' ' } else { b'\n' };
        std::fs::write(&path, bytes).unwrap();
        assert!(prepared.verify_inputs().is_err());
        assert!(!fixture.root.join("native/out.json").exists());
    }

    /// Unused declared evidence directories remain proof dependencies instead of disappearing from denominators.
    #[cfg(unix)]
    #[test]
    fn unused_directory_removal_refuses_actual_complete_proof() {
        let fixture = fixture();
        std::fs::create_dir(fixture.root.join("unused")).unwrap();
        let mut manifest = read_json(&fixture.manifest);
        manifest["evidence_roots"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":"unused","path":"unused"}));
        write_json(&fixture.manifest, &manifest);
        let prepared = prepared(&fixture, "catalog-main", "native/out.json");
        assert!(prepared.input_paths().any(|path| path == fixture.root.join("unused")));
        std::fs::remove_dir(fixture.root.join("unused")).unwrap();
        assert!(prepared.verify_inputs().is_err());
    }

    /// Explicit calendar dates reject signed/expanded/noncanonical tokens while admitting a leap day.
    #[test]
    fn canonical_dates_are_literal_calendar_inputs() {
        assert_eq!(
            canonical_date("2028-02-29").unwrap(),
            NaiveDate::from_ymd_opt(2028, 2, 29).unwrap()
        );
        for token in ["-0001-01-01", "+10000-01-01", "2027-02-29", "2026-2-04", "2026-10-04 "] {
            assert!(canonical_date(token).is_err(), "admitted {token}");
        }
    }

    /// Both root and nested destinations stay portable, absent and beside their actual selected target.
    #[test]
    fn output_preflight_refuses_existing_alias_unsafe_and_wrong_parent_destinations() {
        let fixture = fixture();
        for output in [
            "out.json",
            "native/catalog.json",
            "native/../out.json",
            "native/out.JSON",
            "missing/out.json",
        ] {
            assert!(
                prepare(&fixture.manifest, "catalog-main", "2026-10-04", Path::new(output))
                    .is_err(),
                "admitted {output}"
            );
        }
        assert!(
            prepare(&fixture.manifest, "absent-key", "2026-10-04", Path::new("native/out.json"))
                .is_err()
        );
        assert_eq!(std::fs::read_dir(fixture.root.join("native")).unwrap().count(), 2);
    }

    /// Inspect the raw caller spelling before absolute/join can erase dot or redundant components.
    #[test]
    fn raw_manifest_aliases_are_refused_before_root_composition() {
        let fixture = fixture();
        for suffix in ["/./links.json", "//links.json", "/native/../links.json"] {
            let mut raw = fixture.root.as_os_str().to_os_string();
            raw.push(suffix);
            assert!(
                prepare(
                    Path::new(&raw),
                    "catalog-main",
                    "2026-10-04",
                    Path::new("native/out.json")
                )
                .is_err(),
                "admitted raw {suffix}"
            );
        }
        let mut declaration = read_json(&fixture.manifest);
        declaration["requirement_resources"][0]["artifact"] = json!("native/./catalog.json");
        write_json(&fixture.manifest, &declaration);
        assert!(
            prepare(&fixture.manifest, "catalog-main", "2026-10-04", Path::new("native/out.json"))
                .is_err()
        );
    }

    /// Raw root or destination-parent symlink spellings never become safe through canonicalization.
    #[cfg(unix)]
    #[test]
    fn symlink_manifest_ancestry_and_output_parent_are_refused() {
        let fixture = fixture();
        let sibling = tempfile::tempdir().unwrap();
        let alias = sibling.path().canonicalize().unwrap().join("alias");
        std::os::unix::fs::symlink(&fixture.root, &alias).unwrap();
        assert!(
            prepare(
                &alias.join("links.json"),
                "catalog-main",
                "2026-10-04",
                Path::new("native/out.json")
            )
            .is_err()
        );
        std::os::unix::fs::symlink(fixture.root.join("native"), fixture.root.join("alias"))
            .unwrap();
        assert!(preflight_output(&fixture.root, Path::new("alias/out.json")).is_err());
    }

    /// Stable UUID framing matches the frozen vector and excludes original generation/path changes.
    #[test]
    fn resource_uuid_framing_matches_contract_and_separates_domains() {
        let mut source = source();
        let first =
            resource_id("example-project", &target(&source), "link", "many-to-many").unwrap();
        assert_eq!(first.to_string(), "4fd4f7f8-617a-5f76-8352-ac2e76d36e27");
        source.raw_sha256 = "b".repeat(64);
        source.href = "elsewhere.json".into();
        source.document_version = "99".into();
        assert_eq!(
            resource_id("example-project", &target(&source), "link", "many-to-many").unwrap(),
            first
        );
        assert_ne!(
            resource_id("other-project", &target(&source), "link", "many-to-many").unwrap(),
            first
        );
        assert_ne!(
            resource_id("example-project", &target(&source), "evidence", "many-to-many").unwrap(),
            first
        );
        assert_ne!(
            resource_id("example-project", &target(&source), "link", "other-link").unwrap(),
            first
        );
        assert_ne!(
            super::super::stable_bytes(&["a:b", "c"]),
            super::super::stable_bytes(&["a", "b:c"])
        );
    }

    /// Full original UUID spellings collide by value while repeated old references remain counted but valid.
    #[test]
    fn complete_uuid_scan_and_collision_admission_are_not_substring_based() {
        let fixture = fixture();
        let mut capture = CaptureSession::new(&fixture.root).unwrap();
        let id = Uuid::parse_str("4fd4f7f8-617a-5f76-8352-ac2e76d36e27").unwrap();
        let values = json!([
            id.to_string(),
            id.simple().to_string(),
            id.braced().to_string(),
            id.urn().to_string(),
            format!("prefix{id}")
        ]);
        let mut original = BTreeSet::new();
        collect_original_uuids(&values, &mut capture, &mut original).unwrap();
        assert_eq!(original.len(), 1);
        let mut generated = BTreeSet::new();
        assert!(admit_id(id, &original, &mut generated, &mut capture).is_err());
        assert!(generated.is_empty());
        let other = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"other");
        admit_id(other, &original, &mut generated, &mut capture).unwrap();
        assert!(admit_id(other, &original, &mut generated, &mut capture).is_err());
        assert_eq!(generated.len(), 1);
    }

    /// Original UUID occurrence and generated admission share the complete monotonic relationship budget.
    #[test]
    fn complete_relationship_bound_refuses_before_uuid_set_growth() {
        let fixture = fixture();
        let mut capture = CaptureSession::new(&fixture.root).unwrap();
        capture.relationships(crate::evidence_capture::MAX_RELATIONSHIPS).unwrap();
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, b"bound");
        let mut original = BTreeSet::new();
        assert!(
            collect_original_uuids(&json!(id.to_string()), &mut capture, &mut original).is_err()
        );
        assert!(original.is_empty());
        let mut generated = BTreeSet::new();
        assert!(admit_id(id, &original, &mut generated, &mut capture).is_err());
        assert!(generated.is_empty());
    }

    /// An actual native field containing a generated UUID refuses without renumbering or writing output.
    #[test]
    fn actual_original_generated_uuid_collision_is_refused() {
        let fixture = fixture();
        let first = prepared(&fixture, "catalog-main", "native/out.json");
        let original = read_json(&fixture.root.join("native/catalog.json"));
        let derived: Value = serde_json::from_slice(first.bytes()).unwrap();
        let id = derived["catalog"]["back-matter"]["resources"]
            [original_count(&original, "catalog")]["uuid"]
            .as_str()
            .unwrap();
        let mut changed = original;
        changed["catalog"]["metadata"]["title"] = json!(id);
        write_json(&fixture.root.join("native/catalog.json"), &changed);
        let mut manifest = read_json(&fixture.manifest);
        manifest["requirement_resources"][0]["expected_sha256"] =
            json!(crate::hashing::sha256_hex(
                &std::fs::read(fixture.root.join("native/catalog.json")).unwrap()
            ));
        write_json(&fixture.manifest, &manifest);
        assert!(
            prepare(&fixture.manifest, "catalog-main", "2026-10-04", Path::new("native/out.json"))
                .is_err()
        );
        assert!(!fixture.root.join("native/out.json").exists());
    }

    /// The removal oracle restores original absent, empty and populated container distinctions exactly.
    #[test]
    fn complete_removal_oracle_preserves_container_presence_and_prefix_values() {
        for original in [
            json!({"catalog":{"uuid":"kept"}}),
            json!({"catalog":{"back-matter":{}}}),
            json!({"catalog":{"back-matter":{"resources":[{"uuid":"old","base64":{"value":"private"}}],"remarks":"kept"}}}),
        ] {
            let mut derived = original.clone();
            let shape =
                append_resources(&mut derived, "catalog", vec![json!({"uuid":"new"})]).unwrap();
            remove_generated(&mut derived, "catalog", &shape).unwrap();
            assert_eq!(derived, original);
        }
    }

    /// Unexpected suffix length or new container members cannot pass the exact removal oracle.
    #[test]
    fn removal_oracle_refuses_partial_suffix_and_surplus_new_container_fields() {
        let mut value = json!({"catalog":{}});
        let shape = append_resources(&mut value, "catalog", vec![json!({"uuid":"new"})]).unwrap();
        let mut short = value.clone();
        short["catalog"]["back-matter"]["resources"] = json!([]);
        assert!(remove_generated(&mut short, "catalog", &shape).is_err());
        value["catalog"]["back-matter"]["remarks"] = json!("unexpected");
        assert!(remove_generated(&mut value, "catalog", &shape).is_err());
    }

    /// Capped serialization rejects before buffer append and accounts for pretty final newline.
    #[test]
    fn encoded_bounds_include_escaping_and_pretty_newline() {
        assert_eq!(encode_bounded(&"x", 3, false).unwrap(), b"\"x\"");
        assert!(encode_bounded(&"x", 2, false).is_err());
        assert_eq!(encode_bounded(&0, 2, true).unwrap(), b"0\n");
        assert!(encode_bounded(&0, 1, true).is_err());
        let mut sink = LimitedWriter { bytes: vec![1], limit: 2 };
        assert!(sink.write_all(&[2, 3]).is_err());
        assert_eq!(sink.bytes, vec![1]);
    }

    /// Generated strict JSON refuses duplicate keys, BOM and trailing material rather than normalizing them.
    #[test]
    fn strict_embedded_records_refuse_duplicate_bom_and_trailing_json() {
        for bytes in
            [b"{\"a\":1,\"a\":2}".as_slice(), b"\xef\xbb\xbf{}".as_slice(), b"{} {}".as_slice()]
        {
            assert!(strict_generated(bytes).is_err());
        }
    }

    /// Native target decode retains the existing integer-only losslessness boundary for nested numeric values.
    #[test]
    fn native_decode_preserves_integer_only_refusals() {
        let fixture = fixture();
        let original = read_json(&fixture.root.join("native/catalog.json"));
        for number in ["1.0", "1e0", "18446744073709551616", "-9223372036854775809"] {
            let text = serde_json::to_string(&original).unwrap();
            let patched = text.replacen(
                "\"catalog\":{",
                &format!("\"catalog\":{{\"unknown-numeric\":{number},"),
                1,
            );
            assert!(decode_native(patched.as_bytes()).is_err(), "admitted {number}");
        }
    }

    /// Closed native fields and complete embedded records refuse foreign fields and forged associations.
    #[test]
    fn generated_profile_validation_consumes_complete_record_and_resource_fields() {
        let fixture = fixture();
        let prepared = prepared(&fixture, "catalog-main", "native/out.json");
        let original = read_json(&fixture.root.join("native/catalog.json"));
        let derived: Value = serde_json::from_slice(prepared.bytes()).unwrap();
        let resource = &derived["catalog"]["back-matter"]["resources"]
            [original_count(&original, "catalog") + 1];
        let compact = resource["props"][1]["value"].as_str().unwrap();
        let record = strict_generated(compact.as_bytes()).unwrap();
        let id = resource["uuid"].as_str().unwrap();
        let validators = profile_validators().unwrap();
        validate_encoded_resource(resource, id, compact, &record, &validators).unwrap();
        let mut foreign = resource.clone();
        foreign["remarks"] = json!("new private prose");
        assert!(validate_encoded_resource(&foreign, id, compact, &record, &validators).is_err());
        let mut forged = record.clone();
        forged["evidence"][0]["key"] = json!("not-the-actual-key");
        assert!(validate_encoded_resource(resource, id, compact, &forged, &validators).is_err());
        let mut wrong = resource.clone();
        wrong["props"][1]["ns"] = json!("urn:foreign");
        assert!(validate_encoded_resource(&wrong, id, compact, &record, &validators).is_err());
        let mut unknown = record;
        unknown["rationale"] = json!("private");
        assert!(!validators.record.is_valid(&unknown));
    }

    /// Compact embedded data escapes authored control characters while preserving exact decoded values.
    #[test]
    fn compact_records_preserve_tabs_and_newlines_as_escaped_data_only() {
        let value = json!({"key":"author\ncarriage\rtab\tvalue"});
        let bytes = encode_bounded(&value, 100, false).unwrap();
        assert!(!bytes.contains(&b'\n'));
        assert!(!bytes.contains(&b'\r'));
        assert_eq!(strict_generated(&bytes).unwrap(), value);
    }

    /// Cumulative shared metadata admission refuses a next record before a generated resource is retained.
    #[test]
    fn escaped_resource_projection_uses_the_existing_shared_budget() {
        let mut projection = ProjectionBudget::new();
        projection.admit_row(&"x".repeat(MAX_RECORD - 3)).unwrap();
        let validators = profile_validators().unwrap();
        let record = json!({"kind":"evidence","provenance_uuid":"11111111-1111-5111-8111-111111111111","evidence_key":"key","freshness":"unverified-uri","recorded_valid_through":null,"reference":{"kind":"uri","expected_sha256":null}});
        assert!(
            encode_resource(
                Uuid::new_v5(&Uuid::NAMESPACE_URL, b"bound"),
                &record,
                &mut projection,
                &validators
            )
            .is_err()
        );
    }

    /// Forged local hash/size/null or URI state cannot turn recorded observations into new authority.
    #[test]
    fn local_and_uri_null_state_relations_are_consumed_before_projection() {
        let fixture = fixture();
        let root = &fixture.root;
        let mut capture = CaptureSession::new(root).unwrap();
        let mut projection = ProjectionBudget::new();
        let fresh = super::super::fresh::prepare_fresh(
            Path::new("links.json"),
            &crate::hashing::sha256_hex(&std::fs::read(&fixture.manifest).unwrap()),
            canonical_date("2026-10-04").unwrap(),
            &mut capture,
            &mut projection,
        )
        .unwrap();
        let record = fresh.index().evidence[0].clone();
        evidence_view(&record, "11111111-1111-5111-8111-111111111111").unwrap();
        let mut wrong = record.clone();
        wrong.freshness = super::super::EvidenceFreshness::Changed;
        assert!(evidence_view(&wrong, "11111111-1111-5111-8111-111111111111").is_err());
        let mut absent = record.clone();
        absent.freshness = super::super::EvidenceFreshness::Unavailable;
        assert!(evidence_view(&absent, "11111111-1111-5111-8111-111111111111").is_err());
        let mut partial = record;
        if let EvidenceReference::Local { observed_size, .. } = &mut partial.reference {
            *observed_size = None;
        }
        assert!(evidence_view(&partial, "11111111-1111-5111-8111-111111111111").is_err());
        partial.reference = EvidenceReference::Uri {
            redacted_uri: "https://example.test/private".into(),
            expected_sha256: None,
        };
        assert!(evidence_view(&partial, "11111111-1111-5111-8111-111111111111").is_err());
    }
}
