//! Deterministic evidence and implementation linkage indexes.
//!
//! The workflow records identities, hashes, reviewer assertions, and freshness metadata. It never
//! copies evidence content, follows evidence URIs, or derives control-outcome judgments.

pub mod manifest;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::File;
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use url::Url;
use uuid::Uuid;

use crate::cli::{LinkageFailOn, LinkageReportFormat};
use crate::mapping::inventory::{self, LoadedResource};
use crate::mapping::manifest::{ResourceManifest, SubjectType};
use crate::{ForgeError, OscalModelType, io, validate};

const INDEX_SCHEMA_VERSION: &str = "forge.linkage-index/1";
const REPORT_SCHEMA_VERSION: &str = "forge.linkage-report/1";
const QUEUE_SCHEMA_VERSION: &str = "forge.linkage-queue/1";
const TRUST_BOUNDARY: &str = "Association and byte-change metadata only; no evidence content, origin proof, control outcome, or assessment judgment.";
const LINK_NAMESPACE_SEED: &str = "forge.linkage-index/1:link";
const FINDING_NAMESPACE_SEED: &str = "forge.linkage-index/1:finding";
const MAX_BASELINE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinkageIndex {
    pub schema_version: String,
    pub project_key: String,
    pub project_title: String,
    pub as_of: NaiveDate,
    pub provenance: Provenance,
    pub requirement_inventory: Vec<SubjectRecord>,
    pub implementation_inventory: Vec<SubjectRecord>,
    pub evidence: Vec<EvidenceRecord>,
    pub links: Vec<LinkRecord>,
    pub findings: Vec<Finding>,
    pub trust_boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub manifest_sha256: String,
    pub requirement_resources: Vec<ResourceEvidence>,
    pub implementation_resource: ResourceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceEvidence {
    pub key: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub href: String,
    pub raw_sha256: String,
    pub root_uuid: String,
    pub document_version: String,
    pub oscal_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_catalog_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct SubjectRecord {
    pub resource_key: String,
    pub side: String,
    #[serde(rename = "type")]
    pub subject_type: String,
    pub id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceFreshness {
    Current,
    Expiring,
    Expired,
    Changed,
    Unavailable,
    UnverifiedUri,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind", deny_unknown_fields)]
pub enum EvidenceReference {
    Local {
        root_key: String,
        relative_label: String,
        approved_sha256: String,
        approved_size: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        observed_sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        observed_size: Option<u64>,
    },
    Uri {
        redacted_uri: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_sha256: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub key: String,
    pub title: String,
    pub evidence_type: String,
    pub owner: String,
    pub collected_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_through: Option<NaiveDate>,
    pub sensitivity_label: String,
    pub source_label: String,
    pub freshness: EvidenceFreshness,
    pub reference: EvidenceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinkRecord {
    pub link_id: String,
    pub key: String,
    pub requirements: Vec<SubjectRecord>,
    pub implementations: Vec<SubjectRecord>,
    pub evidence_keys: Vec<String>,
    pub evidence_required: bool,
    pub responsible_role: String,
    pub implementation_status: manifest::ImplementationStatus,
    pub reviewer_key: String,
    pub reviewed_at: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_applicable_review: Option<manifest::ReviewEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impact_finding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_version_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub finding_id: String,
    pub reason_code: String,
    pub project_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    pub action_required: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct LinkageReport<'a> {
    schema_version: &'static str,
    project_key: &'a str,
    as_of: NaiveDate,
    link_count: usize,
    evidence_count: usize,
    provenance: &'a Provenance,
    requirement_inventory: &'a [SubjectRecord],
    implementation_inventory: &'a [SubjectRecord],
    evidence: &'a [EvidenceRecord],
    findings: &'a [Finding],
    links: &'a [LinkRecord],
    trust_boundary: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueReport {
    schema_version: &'static str,
    as_of: NaiveDate,
    groups: Vec<QueueGroup>,
    trust_boundary: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueGroup {
    owner: String,
    items: Vec<QueueItem>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueItem {
    project_key: String,
    reason_code: String,
    finding_id: String,
    link_key: Option<String>,
    evidence_key: Option<String>,
}

struct Prepared {
    index: LinkageIndex,
    input_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct EvidenceFileIdentity {
    volume: u64,
    file: u64,
}

#[derive(Debug)]
struct ImplementationInventory {
    evidence: ResourceEvidence,
    subjects: BTreeMap<(manifest::ImplementationSubjectType, String), String>,
    id_types: BTreeMap<String, BTreeSet<manifest::ImplementationSubjectType>>,
    path: PathBuf,
}

/// Create a manifest scaffold bound to exact OSCAL artifact hashes and identities.
///
/// # Errors
///
/// Returns [`ForgeError`] when either artifact or a required Profile companion is missing,
/// unsafe, oversized, malformed, schema-invalid, unsupported, aliased, or cannot be serialized or
/// written atomically.
#[allow(clippy::too_many_lines)] // Keeping scaffold construction adjacent makes every pinned field auditable.
pub fn execute_init(
    requirement_path: &Path,
    resolved_catalog: Option<&Path>,
    implementation_path: &Path,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let requirement_value = read_json(requirement_path, io::MAX_FILE_SIZE, "requirement artifact")?;
    let resource_type = match validate::detect_model_type(&requirement_value)
        .map_err(|cause| error(format!("requirement artifact: {cause}")))?
    {
        OscalModelType::Catalog => crate::mapping::manifest::ResourceType::Catalog,
        OscalModelType::Profile => crate::mapping::manifest::ResourceType::Profile,
        other => {
            return Err(error(format!(
                "requirement artifact uses '{}'; expected Catalog or Profile",
                other.as_str()
            )));
        }
    };
    if resource_type == crate::mapping::manifest::ResourceType::Profile
        && resolved_catalog.is_none()
    {
        return Err(error("--resolved-catalog is required for a Profile"));
    }
    if resource_type == crate::mapping::manifest::ResourceType::Catalog
        && resolved_catalog.is_some()
    {
        return Err(error("--resolved-catalog is only valid for a Profile"));
    }
    let requirement_bytes =
        inventory::read_bounded_file(requirement_path, io::MAX_FILE_SIZE, "requirement artifact")?;
    let companion_hash = resolved_catalog
        .map(|path| {
            inventory::read_bounded_file(path, io::MAX_FILE_SIZE, "resolved Catalog")
                .map(|bytes| digest(&bytes))
        })
        .transpose()?;
    let temporary_requirement = manifest::RequirementResourceManifest {
        key: "requirements".to_string(),
        resource_type,
        artifact: requirement_path.to_path_buf(),
        href: safe_file_label(requirement_path),
        resolved_catalog: resolved_catalog.map(Path::to_path_buf),
        resolved_catalog_attestation: resolved_catalog.map(|_| true),
        expected_sha256: digest(&requirement_bytes),
        expected_resolved_catalog_sha256: companion_hash,
    };
    let loaded_requirement = load_requirement(Path::new("."), &temporary_requirement)?;

    let implementation_bytes = inventory::read_bounded_file(
        implementation_path,
        io::MAX_FILE_SIZE,
        "implementation artifact",
    )?;
    let implementation_value: Value = serde_json::from_slice(&implementation_bytes)
        .map_err(|cause| error(format!("implementation artifact is not JSON: {cause}")))?;
    let implementation_type = match validate::detect_model_type(&implementation_value)
        .map_err(|cause| error(format!("implementation artifact: {cause}")))?
    {
        OscalModelType::ComponentDefinition => {
            manifest::ImplementationResourceType::ComponentDefinition
        }
        OscalModelType::SystemSecurityPlan => {
            manifest::ImplementationResourceType::SystemSecurityPlan
        }
        other => {
            return Err(error(format!(
                "implementation artifact uses '{}'; expected Component Definition or System Security Plan",
                other.as_str()
            )));
        }
    };
    let temporary_implementation = manifest::ImplementationResourceManifest {
        key: "implementation".to_string(),
        resource_type: implementation_type,
        artifact: implementation_path.to_path_buf(),
        href: safe_file_label(implementation_path),
        expected_sha256: digest(&implementation_bytes),
    };
    let _ = load_implementation(Path::new("."), &temporary_implementation)?;

    let scaffold = manifest::LinkageManifest {
        schema_version: manifest::SCHEMA_VERSION.to_string(),
        project: manifest::ProjectManifest {
            key: "replace-with-stable-project-key".to_string(),
            title: "REPLACE WITH LINKAGE PROJECT TITLE".to_string(),
            expiring_window_days: 30,
            max_evidence_bytes: 10 * 1024 * 1024,
            approved_uri_schemes: Vec::new(),
        },
        reviewers: Vec::new(),
        requirement_resources: vec![manifest::RequirementResourceManifest {
            artifact: descendant_path(requirement_path, output)?,
            resolved_catalog: resolved_catalog
                .map(|path| descendant_path(path, output))
                .transpose()?,
            resolved_catalog_attestation: resolved_catalog.map(|_| false),
            expected_sha256: loaded_requirement.evidence.raw_sha256,
            expected_resolved_catalog_sha256: loaded_requirement.evidence.resolved_catalog_sha256,
            ..temporary_requirement
        }],
        implementation_resource: manifest::ImplementationResourceManifest {
            artifact: descendant_path(implementation_path, output)?,
            ..temporary_implementation
        },
        evidence_roots: Vec::new(),
        evidence: Vec::new(),
        links: Vec::new(),
    };
    let mut rendered = serde_json::to_string_pretty(&scaffold)
        .map_err(|cause| error(format!("manifest scaffold serialization failed: {cause}")))?;
    rendered.push('\n');
    let mut inputs = vec![requirement_path.to_path_buf(), implementation_path.to_path_buf()];
    inputs.extend(resolved_catalog.map(Path::to_path_buf));
    validate_destinations(&inputs, output, None)?;
    crate::cli::output::write_output(&rendered, output)
}

/// Build and write a deterministic linkage index plus an optional report.
///
/// # Errors
///
/// Returns [`ForgeError`] when manifest, artifact, subject, evidence, URI, baseline, destination,
/// serialization, or atomic output validation fails.
pub fn execute_build(
    manifest_path: &Path,
    as_of: NaiveDate,
    output: Option<&Path>,
    report: Option<&Path>,
    format: &LinkageReportFormat,
    baseline: Option<&Path>,
    fail_on: &LinkageFailOn,
) -> Result<bool, ForgeError> {
    let prepared = prepare(manifest_path, as_of, baseline)?;
    validate_destinations(&prepared.input_paths, output, report)?;
    let mut index_json = serde_json::to_string_pretty(&prepared.index)
        .map_err(|cause| error(format!("index serialization failed: {cause}")))?;
    index_json.push('\n');
    let rendered_report = report.map(|_| render_report(&prepared.index, format)).transpose()?;
    crate::cli::output::write_output(&index_json, output)?;
    if let (Some(report_path), Some(rendered)) = (report, rendered_report) {
        crate::cli::output::write_output(&rendered, Some(report_path))?;
    }
    Ok(gate_fires(&prepared.index.findings, fail_on))
}

/// Validate current inputs and render a deterministic report without writing an index.
///
/// # Errors
///
/// Returns [`ForgeError`] for any invalid input, unsafe destination, incomplete analysis, report
/// serialization failure, or atomic output failure.
pub fn execute_check(
    manifest_path: &Path,
    as_of: NaiveDate,
    baseline: Option<&Path>,
    format: &LinkageReportFormat,
    output: Option<&Path>,
    fail_on: &LinkageFailOn,
) -> Result<bool, ForgeError> {
    let prepared = prepare(manifest_path, as_of, baseline)?;
    validate_destinations(&prepared.input_paths, output, None)?;
    crate::cli::output::write_output(&render_report(&prepared.index, format)?, output)?;
    Ok(gate_fires(&prepared.index.findings, fail_on))
}

/// Aggregate explicit linkage projects into a deterministic owner queue.
///
/// # Errors
///
/// Returns [`ForgeError`] if a supplied project cannot be analyzed completely, manifests alias,
/// or the queue cannot be serialized or written atomically.
pub fn execute_queue(
    manifests: &[PathBuf],
    as_of: NaiveDate,
    format: &LinkageReportFormat,
    output: Option<&Path>,
    fail_on: &LinkageFailOn,
) -> Result<bool, ForgeError> {
    let mut seen = Vec::<PathBuf>::new();
    let mut input_paths = Vec::<PathBuf>::new();
    let mut findings = Vec::new();
    for path in manifests {
        for prior in &seen {
            if crate::mapping::paths_alias(path, prior).map_err(relabel_mapping_error)? {
                return Err(error("queue manifests must not alias each other"));
            }
        }
        seen.push(path.clone());
        let prepared = prepare(path, as_of, None)?;
        input_paths.extend(prepared.input_paths);
        findings.extend(prepared.index.findings);
    }
    findings.sort();
    let mut groups = BTreeMap::<String, Vec<QueueItem>>::new();
    for finding in &findings {
        groups
            .entry(finding.owner.clone().unwrap_or_else(|| "unassigned".to_string()))
            .or_default()
            .push(QueueItem {
                project_key: finding.project_key.clone(),
                reason_code: finding.reason_code.clone(),
                finding_id: finding.finding_id.clone(),
                link_key: finding.link_key.clone(),
                evidence_key: finding.evidence_key.clone(),
            });
    }
    let queue = QueueReport {
        schema_version: QUEUE_SCHEMA_VERSION,
        as_of,
        groups: groups.into_iter().map(|(owner, items)| QueueGroup { owner, items }).collect(),
        trust_boundary: TRUST_BOUNDARY,
    };
    let rendered = match format {
        LinkageReportFormat::Json => pretty_json(&queue, "queue")?,
        LinkageReportFormat::Text => render_queue_text(&queue),
        LinkageReportFormat::Html => render_queue_html(&queue),
    };
    validate_destinations(&input_paths, output, None)?;
    crate::cli::output::write_output(&rendered, output)?;
    Ok(gate_fires(&findings, fail_on))
}

#[allow(clippy::too_many_lines)] // This is the single fail-before-write trust-boundary coordinator.
fn prepare(
    manifest_path: &Path,
    as_of: NaiveDate,
    baseline_path: Option<&Path>,
) -> Result<Prepared, ForgeError> {
    let manifest_bytes = inventory::read_bounded_file(
        manifest_path,
        manifest::MAX_MANIFEST_BYTES,
        "linkage manifest",
    )?;
    let manifest = manifest::parse(&manifest_bytes)?;
    if manifest.links.is_empty() {
        return Err(error("$.links must contain at least one reviewed link"));
    }
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut requirement_resources = BTreeMap::new();
    let mut requirement_evidence = Vec::new();
    let mut requirement_inventory = Vec::new();
    let mut input_paths = vec![manifest_path.to_path_buf()];
    for resource in &manifest.requirement_resources {
        let loaded = load_requirement(manifest_dir, resource)?;
        input_paths.push(loaded.path.clone());
        if let Some(companion) = &resource.resolved_catalog {
            input_paths.push(manifest_dir.join(companion));
        }
        for subject_type in [SubjectType::Control, SubjectType::Statement] {
            for id in loaded.inventory.ids_of_type(subject_type) {
                let fingerprint = loaded
                    .inventory
                    .fingerprint(subject_type, &id)
                    .ok_or_else(|| error("requirement inventory fingerprint disappeared"))?;
                requirement_inventory.push(SubjectRecord {
                    resource_key: resource.key.clone(),
                    side: "requirement".to_string(),
                    subject_type: subject_type.as_str().to_string(),
                    id,
                    sha256: fingerprint.to_string(),
                });
            }
        }
        requirement_evidence.push(ResourceEvidence {
            key: resource.key.clone(),
            resource_type: loaded.evidence.resource_type.as_str().to_string(),
            href: sanitize_reference_label(&loaded.evidence.href),
            raw_sha256: loaded.evidence.raw_sha256.clone(),
            root_uuid: loaded.evidence.root_uuid.clone(),
            document_version: loaded.evidence.document_version.clone(),
            oscal_version: loaded.evidence.oscal_version.clone(),
            resolved_catalog_sha256: loaded.evidence.resolved_catalog_sha256.clone(),
        });
        requirement_resources.insert(resource.key.clone(), loaded);
    }
    requirement_evidence.sort_by(|left, right| left.key.cmp(&right.key));
    requirement_inventory.sort();

    let implementation = load_implementation(manifest_dir, &manifest.implementation_resource)?;
    input_paths.push(implementation.path.clone());
    let mut implementation_inventory = implementation
        .subjects
        .iter()
        .map(|((subject_type, id), fingerprint)| SubjectRecord {
            resource_key: manifest.implementation_resource.key.clone(),
            side: "implementation".to_string(),
            subject_type: subject_type.as_str().to_string(),
            id: id.clone(),
            sha256: fingerprint.clone(),
        })
        .collect::<Vec<_>>();
    implementation_inventory.sort();

    let (mut evidence, evidence_paths) = load_evidence(&manifest, manifest_dir, as_of)?;
    input_paths.extend(evidence_paths);
    evidence.sort_by(|left, right| left.key.cmp(&right.key));
    let evidence_by_key: BTreeMap<_, _> =
        evidence.iter().map(|record| (record.key.as_str(), record)).collect();

    let link_namespace = namespace(LINK_NAMESPACE_SEED);
    let mut links = Vec::new();
    let mut findings = Vec::new();
    for link in &manifest.links {
        let mut requirements = Vec::new();
        for subject in &link.requirements {
            let loaded = requirement_resources
                .get(&subject.resource_key)
                .ok_or_else(|| error("validated requirement resource disappeared"))?;
            let subject_type = match subject.subject_type {
                manifest::RequirementSubjectType::Control => SubjectType::Control,
                manifest::RequirementSubjectType::Statement => SubjectType::Statement,
            };
            let fingerprint = resolve_requirement_subject(loaded, subject_type, &subject.id_ref)?;
            requirements.push(SubjectRecord {
                resource_key: subject.resource_key.clone(),
                side: "requirement".to_string(),
                subject_type: subject.subject_type.as_str().to_string(),
                id: subject.id_ref.clone(),
                sha256: fingerprint.to_string(),
            });
        }
        requirements.sort();
        let mut implementations = Vec::new();
        for subject in &link.implementations {
            let fingerprint = resolve_implementation_subject(
                &implementation,
                subject.subject_type,
                &subject.id_ref,
            )?;
            implementations.push(SubjectRecord {
                resource_key: manifest.implementation_resource.key.clone(),
                side: "implementation".to_string(),
                subject_type: subject.subject_type.as_str().to_string(),
                id: subject.id_ref.clone(),
                sha256: fingerprint.to_string(),
            });
        }
        implementations.sort();
        let mut evidence_keys = link.evidence_keys.clone();
        evidence_keys.sort();
        if evidence_keys.is_empty() {
            findings.push(finding(
                &manifest.project.key,
                "evidence-missing",
                Some(&link.key),
                None,
                None,
                None,
                link.evidence_required,
                "The reviewed link has no evidence reference.",
            ));
        }
        for evidence_key in &evidence_keys {
            let record = evidence_by_key
                .get(evidence_key.as_str())
                .ok_or_else(|| error("validated evidence reference disappeared"))?;
            let mut maintenance_states = vec![record.freshness.clone()];
            if !matches!(
                record.freshness,
                EvidenceFreshness::Current
                    | EvidenceFreshness::Expiring
                    | EvidenceFreshness::Expired
            ) {
                let date_state = date_freshness(
                    record.valid_through,
                    as_of,
                    manifest.project.expiring_window_days,
                )?;
                if date_state != EvidenceFreshness::Current {
                    maintenance_states.push(date_state);
                }
            }
            for state in maintenance_states {
                let Some((reason, message)) = freshness_finding(&state) else {
                    continue;
                };
                findings.push(finding(
                    &manifest.project.key,
                    reason,
                    Some(&link.key),
                    Some(evidence_key),
                    None,
                    Some(&record.owner),
                    link.evidence_required && state != EvidenceFreshness::UnverifiedUri,
                    message,
                ));
            }
        }
        let link_id = Uuid::new_v5(
            &link_namespace,
            &stable_bytes(&[manifest.project.key.as_str(), link.key.as_str()]),
        )
        .to_string();
        links.push(LinkRecord {
            link_id,
            key: link.key.clone(),
            requirements,
            implementations,
            evidence_keys,
            evidence_required: link.evidence_required,
            responsible_role: link.responsible_role.clone(),
            implementation_status: link.implementation_status,
            reviewer_key: link.review.reviewer_key.clone(),
            reviewed_at: link.review.reviewed_at.clone(),
            rationale: link.review.rationale.clone(),
            not_applicable_review: link.not_applicable_review.clone(),
            impact_finding_ids: sorted(link.impact_finding_ids.clone()),
            policy_version_keys: sorted(link.policy_version_keys.clone()),
        });
    }
    links.sort_by(|left, right| left.key.cmp(&right.key));
    let linked_implementation_subjects = links
        .iter()
        .flat_map(|link| link.implementations.iter().map(subject_identity))
        .collect::<BTreeSet<_>>();
    for subject in &implementation_inventory {
        let identity = subject_identity(subject);
        if !linked_implementation_subjects.contains(&identity) {
            findings.push(finding(
                &manifest.project.key,
                "implementation-subject-unlinked",
                None,
                None,
                Some(&identity),
                None,
                true,
                "An implementation subject in the exact artifact inventory is not covered by a reviewed link.",
            ));
        }
    }

    let mut index = LinkageIndex {
        schema_version: INDEX_SCHEMA_VERSION.to_string(),
        project_key: manifest.project.key.clone(),
        project_title: manifest.project.title.clone(),
        as_of,
        provenance: Provenance {
            manifest_sha256: digest(&manifest_bytes),
            requirement_resources: requirement_evidence,
            implementation_resource: implementation.evidence,
        },
        requirement_inventory,
        implementation_inventory,
        evidence,
        links,
        findings: Vec::new(),
        trust_boundary: TRUST_BOUNDARY.to_string(),
    };
    if let Some(path) = baseline_path {
        input_paths.push(path.to_path_buf());
        let baseline = load_baseline(path)?;
        if baseline.project_key != index.project_key {
            return Err(error(format!(
                "baseline project_key '{}' does not match current project_key '{}'",
                safe(&baseline.project_key),
                safe(&index.project_key)
            )));
        }
        compare_baseline(&baseline, &index, &mut findings);
    }
    findings.sort();
    findings.dedup();
    index.findings = findings;
    Ok(Prepared { index, input_paths })
}

fn load_requirement(
    manifest_dir: &Path,
    resource: &manifest::RequirementResourceManifest,
) -> Result<LoadedResource, ForgeError> {
    inventory::load(
        manifest_dir,
        &format!("$.requirement_resources[{}]", safe(&resource.key)),
        &ResourceManifest {
            resource_type: resource.resource_type,
            artifact: resource.artifact.clone(),
            href: resource.href.clone(),
            resolved_catalog: resource.resolved_catalog.clone(),
            resolved_catalog_attestation: resource.resolved_catalog_attestation,
            expected_sha256: Some(resource.expected_sha256.clone()),
            expected_resolved_catalog_sha256: resource.expected_resolved_catalog_sha256.clone(),
            inventory: None,
        },
    )
    .map_err(relabel_mapping_error)
}

fn load_implementation(
    manifest_dir: &Path,
    resource: &manifest::ImplementationResourceManifest,
) -> Result<ImplementationInventory, ForgeError> {
    let path = manifest_dir.join(&resource.artifact);
    let bytes = inventory::read_bounded_file(&path, io::MAX_FILE_SIZE, "implementation artifact")?;
    let raw_sha256 = digest(&bytes);
    if raw_sha256 != resource.expected_sha256 {
        return Err(error(format!(
            "$.implementation_resource.expected_sha256 mismatch: expected {}, got {raw_sha256}",
            resource.expected_sha256
        )));
    }
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|cause| error(format!("implementation artifact is not JSON: {cause}")))?;
    let model = match resource.resource_type {
        manifest::ImplementationResourceType::ComponentDefinition => {
            OscalModelType::ComponentDefinition
        }
        manifest::ImplementationResourceType::SystemSecurityPlan => {
            OscalModelType::SystemSecurityPlan
        }
    };
    let detected = validate::detect_model_type(&value)
        .map_err(|cause| error(format!("implementation artifact: {cause}")))?;
    if detected != model {
        return Err(error(format!(
            "$.implementation_resource.type declares '{}' but artifact root is '{}'",
            resource.resource_type.as_str(),
            detected.as_str()
        )));
    }
    inventory::validate_schema("implementation artifact", &value, model)
        .map_err(relabel_mapping_error)?;
    let root = value
        .get(model.as_str())
        .and_then(Value::as_object)
        .ok_or_else(|| error("implementation artifact root is missing"))?;
    let root_uuid = required_string(root.get("uuid"), "implementation root uuid")?;
    Uuid::parse_str(&root_uuid).map_err(|_| error("implementation root uuid must be a UUID"))?;
    let metadata = root
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| error("implementation metadata is required"))?;
    let document_version = required_string(metadata.get("version"), "implementation version")?;
    let oscal_version =
        required_string(metadata.get("oscal-version"), "implementation OSCAL version")?;
    let mut inventory = ImplementationInventory {
        evidence: ResourceEvidence {
            key: resource.key.clone(),
            resource_type: resource.resource_type.as_str().to_string(),
            href: sanitize_reference_label(&resource.href),
            raw_sha256,
            root_uuid,
            document_version,
            oscal_version,
            resolved_catalog_sha256: None,
        },
        subjects: BTreeMap::new(),
        id_types: BTreeMap::new(),
        path,
    };
    match resource.resource_type {
        manifest::ImplementationResourceType::ComponentDefinition => {
            for container_name in ["components", "capabilities"] {
                if let Some(containers) = root.get(container_name).and_then(Value::as_array) {
                    for container in containers {
                        inventory_control_implementations(
                            container.get("control-implementations"),
                            &mut inventory,
                        )?;
                    }
                }
            }
        }
        manifest::ImplementationResourceType::SystemSecurityPlan => {
            let control_implementation = root.get("control-implementation");
            inventory_implemented_requirements(
                control_implementation.and_then(|value| value.get("implemented-requirements")),
                &mut inventory,
            )?;
        }
    }
    Ok(inventory)
}

fn inventory_control_implementations(
    value: Option<&Value>,
    inventory: &mut ImplementationInventory,
) -> Result<(), ForgeError> {
    if let Some(control_implementations) = value.and_then(Value::as_array) {
        for implementation in control_implementations {
            inventory_implemented_requirements(
                implementation.get("implemented-requirements"),
                inventory,
            )?;
        }
    }
    Ok(())
}

fn inventory_implemented_requirements(
    value: Option<&Value>,
    inventory: &mut ImplementationInventory,
) -> Result<(), ForgeError> {
    if let Some(requirements) = value.and_then(Value::as_array) {
        for requirement in requirements {
            insert_implementation_subject(
                inventory,
                manifest::ImplementationSubjectType::ImplementedRequirement,
                requirement,
            )?;
            if let Some(statements) = requirement.get("statements").and_then(Value::as_array) {
                for statement in statements {
                    insert_implementation_subject(
                        inventory,
                        manifest::ImplementationSubjectType::Statement,
                        statement,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn insert_implementation_subject(
    inventory: &mut ImplementationInventory,
    subject_type: manifest::ImplementationSubjectType,
    value: &Value,
) -> Result<(), ForgeError> {
    validate_implementation_inventory_capacity(inventory.subjects.len())?;
    let id =
        value.get("uuid").and_then(Value::as_str).filter(|id| !id.trim().is_empty()).ok_or_else(
            || error(format!("{} subject lacks a stable uuid", subject_type.as_str())),
        )?;
    Uuid::parse_str(id).map_err(|_| {
        error(format!("{} subject uuid '{}' is invalid", subject_type.as_str(), safe(id)))
    })?;
    let key = (subject_type, id.to_string());
    if inventory.subjects.insert(key, canonical_sha256(value)).is_some() {
        return Err(error(format!(
            "duplicate {} subject uuid '{}'",
            subject_type.as_str(),
            safe(id)
        )));
    }
    let id_types = inventory.id_types.entry(id.to_string()).or_default();
    if !id_types.is_empty() && !id_types.contains(&subject_type) {
        return Err(error(format!(
            "implementation subject uuid '{}' is ambiguous across subject types",
            safe(id)
        )));
    }
    id_types.insert(subject_type);
    Ok(())
}

fn validate_implementation_inventory_capacity(current_len: usize) -> Result<(), ForgeError> {
    if current_len >= inventory::MAX_INVENTORY_SUBJECTS {
        Err(error(format!(
            "implementation artifact exceeds the {} subject inventory limit",
            inventory::MAX_INVENTORY_SUBJECTS
        )))
    } else {
        Ok(())
    }
}

fn resolve_requirement_subject<'a>(
    resource: &'a LoadedResource,
    subject_type: SubjectType,
    id: &str,
) -> Result<&'a str, ForgeError> {
    if let Some(fingerprint) = resource.inventory.fingerprint(subject_type, id) {
        return Ok(fingerprint);
    }
    if let Some(actual) = resource.inventory.type_for_id(id) {
        return Err(error(format!(
            "requirement subject '{}' has type '{}', not '{}'",
            safe(id),
            actual.as_str(),
            subject_type.as_str()
        )));
    }
    if let Some(name) = resource.inventory.ineligible_part_name(id) {
        return Err(error(format!(
            "requirement subject '{}' is an ineligible '{}' part",
            safe(id),
            safe(name)
        )));
    }
    Err(error(format!("requirement subject '{}' does not exist", safe(id))))
}

fn resolve_implementation_subject<'a>(
    inventory: &'a ImplementationInventory,
    subject_type: manifest::ImplementationSubjectType,
    id: &str,
) -> Result<&'a str, ForgeError> {
    if let Some(fingerprint) = inventory.subjects.get(&(subject_type, id.to_string())) {
        return Ok(fingerprint);
    }
    if let Some(types) = inventory.id_types.get(id) {
        let actual = types.iter().map(|value| value.as_str()).collect::<Vec<_>>().join(", ");
        return Err(error(format!(
            "implementation subject '{}' has type {actual}, not {}",
            safe(id),
            subject_type.as_str()
        )));
    }
    Err(error(format!("implementation subject '{}' does not exist", safe(id))))
}

#[allow(clippy::too_many_lines)] // Evidence hazards and the metadata-only result stay visibly paired.
fn load_evidence(
    manifest: &manifest::LinkageManifest,
    manifest_dir: &Path,
    as_of: NaiveDate,
) -> Result<(Vec<EvidenceRecord>, Vec<PathBuf>), ForgeError> {
    let mut roots = BTreeMap::new();
    let mut canonical_roots = BTreeSet::new();
    for root in &manifest.evidence_roots {
        let path = manifest_dir.join(&root.path);
        reject_symlink_components(manifest_dir, &root.path, false)?;
        let canonical = path
            .canonicalize()
            .map_err(|cause| error(format!("evidence root '{}': {cause}", safe(&root.key))))?;
        if !canonical.is_dir() {
            return Err(error(format!("evidence root '{}' must be a directory", safe(&root.key))));
        }
        if !canonical_roots.insert(canonical.clone()) {
            return Err(error(format!(
                "evidence root '{}' aliases another declared root",
                safe(&root.key)
            )));
        }
        roots.insert(root.key.as_str(), canonical);
    }
    let mut records = Vec::new();
    let mut paths = Vec::new();
    let mut existing_identities = BTreeSet::<EvidenceFileIdentity>::new();
    for evidence in &manifest.evidence {
        let base_freshness =
            date_freshness(evidence.valid_through, as_of, manifest.project.expiring_window_days)?;
        let (freshness, reference) = match &evidence.location {
            manifest::EvidenceLocation::Local {
                root_key,
                path: relative,
                expected_sha256,
                expected_size,
            } => {
                let root = roots
                    .get(root_key.as_str())
                    .ok_or_else(|| error("validated evidence root disappeared"))?;
                let candidate = root.join(relative);
                reject_symlink_components(root, relative, true)?;
                paths.push(candidate.clone());
                if std::fs::symlink_metadata(&candidate)
                    .is_err_and(|cause| cause.kind() == std::io::ErrorKind::NotFound)
                {
                    (
                        EvidenceFreshness::Unavailable,
                        EvidenceReference::Local {
                            root_key: root_key.clone(),
                            relative_label: normalize_relative_label(relative),
                            approved_sha256: expected_sha256.clone(),
                            approved_size: *expected_size,
                            observed_sha256: None,
                            observed_size: None,
                        },
                    )
                } else {
                    let (file, identity) = open_confined_evidence(root, relative, &evidence.key)?;
                    if !existing_identities.insert(identity) {
                        return Err(error(format!(
                            "local evidence '{}' aliases another evidence file",
                            safe(&evidence.key)
                        )));
                    }
                    let bytes = read_open_evidence(
                        file,
                        manifest.project.max_evidence_bytes,
                        &evidence.key,
                    )?;
                    let observed_sha256 = digest(&bytes);
                    let observed_size = bytes.len() as u64;
                    let freshness =
                        if &observed_sha256 != expected_sha256 || observed_size != *expected_size {
                            EvidenceFreshness::Changed
                        } else {
                            base_freshness
                        };
                    (
                        freshness,
                        EvidenceReference::Local {
                            root_key: root_key.clone(),
                            relative_label: normalize_relative_label(relative),
                            approved_sha256: expected_sha256.clone(),
                            approved_size: *expected_size,
                            observed_sha256: Some(observed_sha256),
                            observed_size: Some(observed_size),
                        },
                    )
                }
            }
            manifest::EvidenceLocation::Uri { uri, expected_sha256, .. } => (
                EvidenceFreshness::UnverifiedUri,
                EvidenceReference::Uri {
                    redacted_uri: validate_and_redact_uri(
                        uri,
                        &manifest.project.approved_uri_schemes,
                    )?,
                    expected_sha256: expected_sha256.clone(),
                },
            ),
        };
        records.push(EvidenceRecord {
            key: evidence.key.clone(),
            title: evidence.title.clone(),
            evidence_type: evidence.evidence_type.clone(),
            owner: evidence.owner.clone(),
            collected_at: evidence.collected_at.clone(),
            valid_through: evidence.valid_through,
            sensitivity_label: evidence.sensitivity_label.clone(),
            source_label: sanitize_reference_label(&evidence.source_label),
            freshness,
            reference,
        });
    }
    Ok((records, paths))
}

fn reject_symlink_components(
    base: &Path,
    relative: &Path,
    final_file: bool,
) -> Result<(), ForgeError> {
    let mut current = base.to_path_buf();
    let count = relative.components().count();
    for (index, component) in relative.components().enumerate() {
        let Component::Normal(name) = component else {
            return Err(error("evidence path contains a non-descendant component"));
        };
        current.push(name);
        let metadata = match std::fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(cause) => {
                return Err(error(format!("cannot inspect evidence path component: {cause}")));
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(error("evidence paths must not contain symbolic links"));
        }
        if index + 1 < count && !metadata.is_dir() {
            return Err(error("intermediate evidence path component must be a directory"));
        }
        if final_file && index + 1 == count && !metadata.is_file() {
            return Err(error(
                "local evidence must be a regular file, not a device, FIFO, socket, or directory",
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
#[allow(unsafe_code)] // `openat` plus owned-fd transfer are required for descriptor-relative confinement.
fn open_confined_evidence(
    root: &Path,
    relative: &Path,
    evidence_key: &str,
) -> Result<(File, EvidenceFileIdentity), ForgeError> {
    use std::ffi::{CString, OsStr};
    use std::os::fd::{AsRawFd as _, FromRawFd as _};
    use std::os::unix::ffi::OsStrExt as _;

    fn open_at(parent: &File, name: &OsStr, directory: bool) -> std::io::Result<File> {
        let name = CString::new(name.as_bytes()).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path component contains NUL")
        })?;
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if directory { libc::O_DIRECTORY } else { libc::O_NONBLOCK };
        // SAFETY: `parent` owns a live directory descriptor, `name` is NUL-terminated for the
        // duration of the call, and a successful descriptor is immediately transferred to `File`.
        let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
        if descriptor < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            // SAFETY: `openat` returned a new owned descriptor that has not been wrapped elsewhere.
            Ok(unsafe { File::from_raw_fd(descriptor) })
        }
    }

    let mut directory = File::open("/")
        .map_err(|cause| evidence_open_error(evidence_key, "open filesystem root", &cause))?;
    for component in root.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                directory = open_at(&directory, name, true).map_err(|cause| {
                    evidence_open_error(evidence_key, "open evidence root component", &cause)
                })?;
            }
            Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                return Err(error(format!(
                    "local evidence '{}' has an unsupported canonical root",
                    safe(evidence_key)
                )));
            }
        }
    }

    let components = relative.components().collect::<Vec<_>>();
    let mut final_file = None;
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(error(format!(
                "local evidence '{}' contains a non-descendant component",
                safe(evidence_key)
            )));
        };
        if index + 1 == components.len() {
            final_file = Some(open_at(&directory, name, false).map_err(|cause| {
                evidence_open_error(evidence_key, "open confined evidence file", &cause)
            })?);
        } else {
            directory = open_at(&directory, name, true).map_err(|cause| {
                evidence_open_error(evidence_key, "open evidence path component", &cause)
            })?;
        }
    }
    let file = final_file
        .ok_or_else(|| error(format!("local evidence '{}' path is empty", safe(evidence_key))))?;
    let identity = validate_open_evidence(&file, evidence_key)?;
    Ok((file, identity))
}

#[cfg(windows)]
fn open_confined_evidence(
    root: &Path,
    relative: &Path,
    evidence_key: &str,
) -> Result<(File, EvidenceFileIdentity), ForgeError> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::{MetadataExt as _, OpenOptionsExt as _};

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;

    fn open_path(path: &Path, directory: bool) -> std::io::Result<File> {
        let mut options = OpenOptions::new();
        options
            .read(true)
            // Excluding FILE_SHARE_DELETE prevents a held ancestor from being renamed or replaced.
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(
                FILE_FLAG_OPEN_REPARSE_POINT
                    | if directory { FILE_FLAG_BACKUP_SEMANTICS } else { 0 },
            );
        options.open(path)
    }

    fn reject_reparse(file: &File, evidence_key: &str, directory: bool) -> Result<(), ForgeError> {
        let metadata = file.metadata().map_err(|cause| {
            evidence_open_error(evidence_key, "inspect confined evidence handle", &cause)
        })?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(error(format!(
                "local evidence '{}' path must not contain reparse points",
                safe(evidence_key)
            )));
        }
        if directory && !metadata.is_dir() {
            return Err(error(format!(
                "local evidence '{}' intermediate component is not a directory",
                safe(evidence_key)
            )));
        }
        Ok(())
    }

    let mut held_directories = Vec::new();
    let mut candidate = PathBuf::new();
    for component in root.components() {
        match component {
            Component::Prefix(prefix) => candidate.push(prefix.as_os_str()),
            Component::RootDir => candidate.push(component.as_os_str()),
            Component::Normal(name) => {
                candidate.push(name);
                let directory = open_path(&candidate, true).map_err(|cause| {
                    evidence_open_error(evidence_key, "open evidence root component", &cause)
                })?;
                reject_reparse(&directory, evidence_key, true)?;
                held_directories.push(directory);
            }
            Component::CurDir | Component::ParentDir => {
                return Err(error(format!(
                    "local evidence '{}' has an unsupported canonical root",
                    safe(evidence_key)
                )));
            }
        }
    }
    if held_directories.is_empty() {
        return Err(error(format!(
            "local evidence '{}' has an unsupported canonical root",
            safe(evidence_key)
        )));
    }
    let components = relative.components().collect::<Vec<_>>();
    let mut final_file = None;
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err(error(format!(
                "local evidence '{}' contains a non-descendant component",
                safe(evidence_key)
            )));
        };
        candidate.push(name);
        if index + 1 == components.len() {
            let file = open_path(&candidate, false).map_err(|cause| {
                evidence_open_error(evidence_key, "open confined evidence file", &cause)
            })?;
            reject_reparse(&file, evidence_key, false)?;
            final_file = Some(file);
        } else {
            let directory = open_path(&candidate, true).map_err(|cause| {
                evidence_open_error(evidence_key, "open evidence path component", &cause)
            })?;
            reject_reparse(&directory, evidence_key, true)?;
            held_directories.push(directory);
        }
    }
    let file = final_file
        .ok_or_else(|| error(format!("local evidence '{}' path is empty", safe(evidence_key))))?;
    let identity = validate_open_evidence(&file, evidence_key)?;
    drop(held_directories);
    Ok((file, identity))
}

#[cfg(not(any(unix, windows)))]
fn open_confined_evidence(
    _root: &Path,
    _relative: &Path,
    evidence_key: &str,
) -> Result<(File, EvidenceFileIdentity), ForgeError> {
    Err(error(format!(
        "local evidence '{}' cannot be opened safely on this platform",
        safe(evidence_key)
    )))
}

#[cfg(unix)]
fn validate_open_evidence(
    file: &File,
    evidence_key: &str,
) -> Result<EvidenceFileIdentity, ForgeError> {
    use std::os::unix::fs::MetadataExt as _;

    let metadata = file.metadata().map_err(|cause| {
        evidence_open_error(evidence_key, "inspect confined evidence handle", &cause)
    })?;
    if !metadata.is_file() {
        return Err(error(format!(
            "local evidence '{}' must be a regular file",
            safe(evidence_key)
        )));
    }
    if metadata.nlink() != 1 {
        return Err(error(format!(
            "local evidence '{}' must not have hard-link aliases",
            safe(evidence_key)
        )));
    }
    Ok(EvidenceFileIdentity { volume: metadata.dev(), file: metadata.ino() })
}

#[cfg(windows)]
fn validate_open_evidence(
    file: &File,
    evidence_key: &str,
) -> Result<EvidenceFileIdentity, ForgeError> {
    let metadata = file.metadata().map_err(|cause| {
        evidence_open_error(evidence_key, "inspect confined evidence handle", &cause)
    })?;
    if !metadata.is_file() {
        return Err(error(format!(
            "local evidence '{}' must be a regular file",
            safe(evidence_key)
        )));
    }
    let (volume, file_index, link_count) =
        windows_evidence_identity::information(file).map_err(|cause| {
            evidence_open_error(evidence_key, "read confined evidence identity", &cause)
        })?;
    if link_count != 1 {
        return Err(error(format!(
            "local evidence '{}' must not have hard-link aliases",
            safe(evidence_key)
        )));
    }
    Ok(EvidenceFileIdentity { volume: u64::from(volume), file: file_index })
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_evidence_identity {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::windows::io::AsRawHandle as _;

    #[repr(C)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    pub(super) fn information(file: &File) -> io::Result<(u32, u64, u32)> {
        let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
        // SAFETY: `file` remains open, its raw handle is valid, and `information` points to
        // writable storage with the documented C layout. The value is read only after success.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: the successful Windows API call initialized the complete structure.
        let information = unsafe { information.assume_init() };
        let file_index =
            (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
        Ok((information.volume_serial_number, file_index, information.number_of_links))
    }
}

#[cfg(not(any(unix, windows)))]
fn validate_open_evidence(
    _file: &File,
    evidence_key: &str,
) -> Result<EvidenceFileIdentity, ForgeError> {
    Err(error(format!(
        "local evidence '{}' file identity is unsupported on this platform",
        safe(evidence_key)
    )))
}

fn read_open_evidence(
    file: File,
    max_bytes: u64,
    evidence_key: &str,
) -> Result<Vec<u8>, ForgeError> {
    let metadata = file.metadata().map_err(|cause| {
        evidence_open_error(evidence_key, "inspect confined evidence handle", &cause)
    })?;
    if metadata.len() > max_bytes {
        return Err(error(format!(
            "local evidence '{}' exceeds the configured {} byte limit",
            safe(evidence_key),
            max_bytes
        )));
    }
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|cause| evidence_open_error(evidence_key, "read confined evidence", &cause))?;
    if bytes.len() as u64 > max_bytes {
        return Err(error(format!(
            "local evidence '{}' exceeds the configured {} byte limit",
            safe(evidence_key),
            max_bytes
        )));
    }
    Ok(bytes)
}

fn evidence_open_error(evidence_key: &str, operation: &str, cause: &std::io::Error) -> ForgeError {
    error(format!("local evidence '{}': cannot {operation}: {cause}", safe(evidence_key)))
}

fn date_freshness(
    valid_through: Option<NaiveDate>,
    as_of: NaiveDate,
    window_days: u16,
) -> Result<EvidenceFreshness, ForgeError> {
    let Some(valid_through) = valid_through else {
        return Ok(EvidenceFreshness::Current);
    };
    if valid_through <= as_of {
        return Ok(EvidenceFreshness::Expired);
    }
    let window_end = as_of
        .checked_add_signed(Duration::days(i64::from(window_days)))
        .ok_or_else(|| error("freshness window exceeds the supported date range"))?;
    if valid_through <= window_end {
        Ok(EvidenceFreshness::Expiring)
    } else {
        Ok(EvidenceFreshness::Current)
    }
}

fn validate_and_redact_uri(raw: &str, custom_schemes: &[String]) -> Result<String, ForgeError> {
    let mut uri = Url::parse(raw).map_err(|_| error("URI evidence must be an absolute URI"))?;
    if uri.scheme() != "https" && !custom_schemes.iter().any(|scheme| scheme == uri.scheme()) {
        return Err(error(format!(
            "URI scheme '{}' is not allowed for evidence",
            safe(uri.scheme())
        )));
    }
    if uri.scheme() == "https" && uri.host_str().is_none() {
        return Err(error("https evidence URI requires a host"));
    }
    uri.set_query(None);
    uri.set_fragment(None);
    let _ = uri.set_username("");
    let _ = uri.set_password(None);
    Ok(uri.to_string())
}

fn load_baseline(path: &Path) -> Result<LinkageIndex, ForgeError> {
    let bytes = inventory::read_bounded_file(path, MAX_BASELINE_BYTES, "linkage baseline")?;
    let value = crate::json_strict::parse_value(
        &bytes,
        "linkage baseline",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes: manifest::MAX_STRING_BYTES },
    )
    .map_err(|cause| error(cause.to_string()))?;
    let baseline: LinkageIndex = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid linkage baseline: {cause}")))?;
    if baseline.schema_version != INDEX_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported baseline schema '{}'; expected {INDEX_SCHEMA_VERSION}",
            safe(&baseline.schema_version)
        )));
    }
    Ok(baseline)
}

#[allow(clippy::too_many_lines)] // Each required baseline category is explicit and independently keyed.
fn compare_baseline(baseline: &LinkageIndex, current: &LinkageIndex, findings: &mut Vec<Finding>) {
    let project = &current.project_key;
    compare_inventory(
        project,
        "requirement",
        &baseline.requirement_inventory,
        &current.requirement_inventory,
        findings,
    );
    compare_inventory(
        project,
        "implementation",
        &baseline.implementation_inventory,
        &current.implementation_inventory,
        findings,
    );
    let old_evidence: BTreeMap<_, _> =
        baseline.evidence.iter().map(|item| (&item.key, item)).collect();
    let new_evidence: BTreeMap<_, _> =
        current.evidence.iter().map(|item| (&item.key, item)).collect();
    for (key, old) in &old_evidence {
        match new_evidence.get(key) {
            None => findings.push(finding(
                project,
                "evidence-removed",
                None,
                Some(key),
                None,
                Some(&old.owner),
                true,
                "An evidence record present in the baseline was removed.",
            )),
            Some(new) => {
                if local_content_changed(old, new) {
                    findings.push(finding(
                        project,
                        "evidence-content-changed",
                        None,
                        Some(key),
                        None,
                        Some(&new.owner),
                        true,
                        "Observed local evidence bytes changed from the baseline.",
                    ));
                }
                if !declared_evidence_reference_equal(&old.reference, &new.reference) {
                    findings.push(finding(
                        project,
                        "evidence-reference-changed",
                        None,
                        Some(key),
                        None,
                        Some(&new.owner),
                        true,
                        "The declared local or URI evidence reference changed from the baseline.",
                    ));
                }
                if old.valid_through != new.valid_through {
                    findings.push(finding(
                        project,
                        "evidence-expiry-changed",
                        None,
                        Some(key),
                        None,
                        Some(&new.owner),
                        true,
                        "Evidence validity metadata changed from the baseline.",
                    ));
                }
            }
        }
    }
    for (key, new) in &new_evidence {
        if !old_evidence.contains_key(key) {
            findings.push(finding(
                project,
                "evidence-added",
                None,
                Some(key),
                None,
                Some(&new.owner),
                false,
                "An evidence record was added after the baseline.",
            ));
        }
    }
    let old_links: BTreeMap<_, _> = baseline.links.iter().map(|link| (&link.key, link)).collect();
    let new_links: BTreeMap<_, _> = current.links.iter().map(|link| (&link.key, link)).collect();
    for (key, old) in &old_links {
        match new_links.get(key) {
            None => findings.push(finding(
                project,
                "link-removed",
                Some(key),
                None,
                None,
                None,
                true,
                "A reviewed link present in the baseline was removed.",
            )),
            Some(new) if relationship_signature(old) != relationship_signature(new) => {
                findings.push(finding(
                    project,
                    "relationship-edited",
                    Some(key),
                    None,
                    None,
                    None,
                    true,
                    "Requirement, implementation, or evidence membership changed from the baseline.",
                ));
            }
            Some(_) => {}
        }
    }
    for key in new_links.keys() {
        if !old_links.contains_key(key) {
            findings.push(finding(
                project,
                "link-added",
                Some(key),
                None,
                None,
                None,
                false,
                "A reviewed link was added after the baseline.",
            ));
        }
    }
}

fn compare_inventory(
    project: &str,
    side: &str,
    old: &[SubjectRecord],
    new: &[SubjectRecord],
    findings: &mut Vec<Finding>,
) {
    let old_items: BTreeMap<_, _> = old.iter().map(|item| (subject_identity(item), item)).collect();
    let new_items: BTreeMap<_, _> = new.iter().map(|item| (subject_identity(item), item)).collect();
    for (key, prior) in &old_items {
        match new_items.get(key) {
            None => findings.push(finding(
                project,
                &format!("{side}-subject-removed"),
                None,
                None,
                Some(key),
                None,
                true,
                "A subject present in the baseline inventory was removed.",
            )),
            Some(current) if prior.sha256 != current.sha256 => findings.push(finding(
                project,
                &format!("{side}-subject-content-changed"),
                None,
                None,
                Some(key),
                None,
                true,
                "A subject's canonical content changed from the baseline.",
            )),
            Some(_) => {}
        }
    }
    for key in new_items.keys() {
        if !old_items.contains_key(key) {
            findings.push(finding(
                project,
                &format!("{side}-subject-added"),
                None,
                None,
                Some(key),
                None,
                false,
                "A subject was added after the baseline.",
            ));
        }
    }
}

fn freshness_finding(freshness: &EvidenceFreshness) -> Option<(&'static str, &'static str)> {
    match freshness {
        EvidenceFreshness::Current => None,
        EvidenceFreshness::Expiring => Some((
            "evidence-expiring",
            "Evidence validity metadata falls within the configured review window.",
        )),
        EvidenceFreshness::Expired => Some((
            "evidence-expired",
            "Evidence validity metadata is expired for the explicit as-of date.",
        )),
        EvidenceFreshness::Changed => Some((
            "evidence-changed",
            "Observed local evidence hash or size differs from the approved values.",
        )),
        EvidenceFreshness::Unavailable => {
            Some(("evidence-unavailable", "The declared local evidence file is unavailable."))
        }
        EvidenceFreshness::UnverifiedUri => Some((
            "evidence-uri-unverified",
            "The URI reference was recorded without retrieval or local verification.",
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn finding(
    project: &str,
    reason: &str,
    link: Option<&str>,
    evidence: Option<&str>,
    subject: Option<&str>,
    owner: Option<&str>,
    action_required: bool,
    message: &str,
) -> Finding {
    let identity = stable_bytes(&[
        project,
        reason,
        link.unwrap_or(""),
        evidence.unwrap_or(""),
        subject.unwrap_or(""),
    ]);
    Finding {
        finding_id: Uuid::new_v5(&namespace(FINDING_NAMESPACE_SEED), &identity).to_string(),
        reason_code: reason.to_string(),
        project_key: project.to_string(),
        link_key: link.map(str::to_string),
        evidence_key: evidence.map(str::to_string),
        subject_key: subject.map(str::to_string),
        owner: owner.map(str::to_string),
        action_required,
        message: message.to_string(),
    }
}

fn gate_fires(findings: &[Finding], fail_on: &LinkageFailOn) -> bool {
    findings.iter().any(|finding| match fail_on {
        LinkageFailOn::Required => finding.action_required,
        LinkageFailOn::Changed => {
            finding.reason_code.contains("changed")
                || finding.reason_code.ends_with("removed")
                || finding.reason_code == "relationship-edited"
        }
        LinkageFailOn::Expired => finding.reason_code == "evidence-expired",
        LinkageFailOn::Any => true,
        LinkageFailOn::Never => false,
    })
}

fn render_report(index: &LinkageIndex, format: &LinkageReportFormat) -> Result<String, ForgeError> {
    let report = LinkageReport {
        schema_version: REPORT_SCHEMA_VERSION,
        project_key: &index.project_key,
        as_of: index.as_of,
        link_count: index.links.len(),
        evidence_count: index.evidence.len(),
        provenance: &index.provenance,
        requirement_inventory: &index.requirement_inventory,
        implementation_inventory: &index.implementation_inventory,
        evidence: &index.evidence,
        findings: &index.findings,
        links: &index.links,
        trust_boundary: TRUST_BOUNDARY,
    };
    match format {
        LinkageReportFormat::Json => pretty_json(&report, "report"),
        LinkageReportFormat::Text => Ok(render_text(&report)),
        LinkageReportFormat::Html => Ok(render_html(&report)),
    }
}

fn render_text(report: &LinkageReport<'_>) -> String {
    let mut output = String::new();
    output.push_str("FORGE evidence linkage maintenance report\n");
    let _ = writeln!(output, "schema: {}", report.schema_version);
    let _ = writeln!(output, "project: {}", escape(report.project_key));
    let _ = writeln!(output, "as-of: {}", report.as_of);
    let _ = writeln!(output, "links: {}", report.link_count);
    let _ = writeln!(output, "evidence records: {}", report.evidence_count);
    let _ = writeln!(output, "manifest sha256: {}", report.provenance.manifest_sha256);
    let _ = writeln!(output, "boundary: {}", report.trust_boundary);
    output.push_str("evidence metadata:\n");
    for evidence in report.evidence {
        let _ = write!(
            output,
            "- key={} owner={} freshness={:?}",
            escape(&evidence.key),
            escape(&evidence.owner),
            evidence.freshness
        );
        match &evidence.reference {
            EvidenceReference::Local {
                approved_sha256,
                approved_size,
                observed_sha256,
                observed_size,
                ..
            } => {
                let _ = write!(
                    output,
                    " approved-sha256={approved_sha256} approved-size={approved_size} observed-sha256={} observed-size={}",
                    observed_sha256.as_deref().unwrap_or("unavailable"),
                    observed_size
                        .map_or_else(|| "unavailable".to_string(), |size| size.to_string())
                );
            }
            EvidenceReference::Uri { redacted_uri, expected_sha256 } => {
                let _ = write!(
                    output,
                    " uri={} expected-sha256={}",
                    escape(redacted_uri),
                    expected_sha256.as_deref().unwrap_or("unspecified")
                );
            }
        }
        output.push('\n');
    }
    let _ = writeln!(output, "findings: {}", report.findings.len());
    for finding in report.findings {
        let _ = writeln!(
            output,
            "- {} id={} link={} evidence={} owner={}: {}",
            escape(&finding.reason_code),
            finding.finding_id,
            finding.link_key.as_deref().map_or("none".to_string(), escape),
            finding.evidence_key.as_deref().map_or("none".to_string(), escape),
            finding.owner.as_deref().map_or("unassigned".to_string(), escape),
            escape(&finding.message)
        );
    }
    output
}

fn render_html(report: &LinkageReport<'_>) -> String {
    let mut output = String::from(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>FORGE linkage trace</title></head><body>",
    );
    let _ = write!(
        output,
        "<h1>Evidence linkage trace</h1><p>Project: {}</p><p>As of: {}</p><p>{}</p>",
        html(report.project_key),
        report.as_of,
        html(report.trust_boundary)
    );
    output.push_str("<h2>Links</h2><table><thead><tr><th>Key</th><th>Requirements</th><th>Implementations</th><th>Evidence keys</th><th>Status assertion</th></tr></thead><tbody>");
    for link in report.links {
        let requirements =
            link.requirements.iter().map(subject_identity).collect::<Vec<_>>().join(", ");
        let implementations =
            link.implementations.iter().map(subject_identity).collect::<Vec<_>>().join(", ");
        let _ = write!(
            output,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>",
            html(&link.key),
            html(&requirements),
            html(&implementations),
            html(&link.evidence_keys.join(", ")),
            link.implementation_status
        );
    }
    output.push_str("</tbody></table><h2>Maintenance findings</h2><ul>");
    for finding in report.findings {
        let _ =
            write!(output, "<li>{}: {}</li>", html(&finding.reason_code), html(&finding.message));
    }
    output.push_str("</ul><h2>Evidence metadata</h2><table><thead><tr><th>Key</th><th>Owner</th><th>Type</th><th>Freshness</th><th>Reference</th></tr></thead><tbody>");
    for evidence in report.evidence {
        let reference = match &evidence.reference {
            EvidenceReference::Local {
                relative_label, approved_sha256, observed_sha256, ..
            } => format!(
                "{}; approved-sha256={}; observed-sha256={}",
                relative_label,
                approved_sha256,
                observed_sha256.as_deref().unwrap_or("unavailable")
            ),
            EvidenceReference::Uri { redacted_uri, expected_sha256 } => format!(
                "{}; expected-sha256={}",
                redacted_uri,
                expected_sha256.as_deref().unwrap_or("unspecified")
            ),
        };
        let _ = write!(
            output,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td><td>{}</td></tr>",
            html(&evidence.key),
            html(&evidence.owner),
            html(&evidence.evidence_type),
            evidence.freshness,
            html(&reference)
        );
    }
    output.push_str("</tbody></table></body></html>\n");
    output
}

fn render_queue_text(queue: &QueueReport) -> String {
    let mut output = format!(
        "FORGE evidence linkage owner queue\nschema: {}\nas-of: {}\nboundary: {}\n",
        queue.schema_version, queue.as_of, queue.trust_boundary
    );
    for group in &queue.groups {
        let _ = writeln!(output, "owner: {}", escape(&group.owner));
        for item in &group.items {
            let _ = writeln!(
                output,
                "- project={} reason={} id={}",
                escape(&item.project_key),
                escape(&item.reason_code),
                item.finding_id
            );
        }
    }
    output
}

fn render_queue_html(queue: &QueueReport) -> String {
    let mut output = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>FORGE linkage queue</title></head><body><h1>Owner maintenance queue</h1><p>As of: {}</p>",
        queue.as_of
    );
    for group in &queue.groups {
        let _ = write!(output, "<h2>{}</h2><ul>", html(&group.owner));
        for item in &group.items {
            let _ = write!(
                output,
                "<li>{}: {} ({})</li>",
                html(&item.project_key),
                html(&item.reason_code),
                item.finding_id
            );
        }
        output.push_str("</ul>");
    }
    output.push_str("</body></html>\n");
    output
}

fn pretty_json(value: &impl Serialize, label: &str) -> Result<String, ForgeError> {
    let mut rendered = serde_json::to_string_pretty(value)
        .map_err(|cause| error(format!("{label} serialization failed: {cause}")))?;
    rendered.push('\n');
    Ok(rendered)
}

fn validate_destinations(
    inputs: &[PathBuf],
    output: Option<&Path>,
    report: Option<&Path>,
) -> Result<(), ForgeError> {
    let destinations: Vec<_> = [output, report].into_iter().flatten().collect();
    for destination in &destinations {
        for input in inputs {
            if crate::mapping::paths_alias(destination, input).map_err(relabel_mapping_error)? {
                return Err(error("linkage destination aliases an input"));
            }
        }
    }
    if destinations.len() == 2
        && crate::mapping::paths_alias(destinations[0], destinations[1])
            .map_err(relabel_mapping_error)?
    {
        return Err(error("--output and --report must be different files"));
    }
    Ok(())
}

fn descendant_path(path: &Path, output: Option<&Path>) -> Result<PathBuf, ForgeError> {
    let base = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|cause| error(format!("cannot resolve manifest directory: {cause}")))?;
    let target = path
        .canonicalize()
        .map_err(|cause| error(format!("cannot resolve input artifact: {cause}")))?;
    target
        .strip_prefix(&base)
        .map(Path::to_path_buf)
        .map_err(|_| error("init inputs must be descendants of the manifest directory"))
}

fn read_json(path: &Path, max: u64, label: &str) -> Result<Value, ForgeError> {
    let bytes = inventory::read_bounded_file(path, max, label)?;
    serde_json::from_slice(&bytes).map_err(|cause| error(format!("{label} is not JSON: {cause}")))
}

fn required_string(value: Option<&Value>, label: &str) -> Result<String, ForgeError> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| error(format!("{label} must be a non-empty string")))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_sha256(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).expect("serde_json::Value serialization cannot fail");
    digest(&bytes)
}

fn namespace(seed: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes())
}

fn stable_bytes(values: &[&str]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for value in values {
        bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    bytes
}

fn relationship_signature(link: &LinkRecord) -> (Vec<String>, Vec<String>, Vec<String>) {
    (
        link.requirements.iter().map(subject_identity).collect(),
        link.implementations.iter().map(subject_identity).collect(),
        link.evidence_keys.clone(),
    )
}

fn subject_identity(subject: &SubjectRecord) -> String {
    format!("{}:{}:{}", subject.resource_key, subject.subject_type, subject.id)
}

fn local_content_changed(old: &EvidenceRecord, new: &EvidenceRecord) -> bool {
    match (&old.reference, &new.reference) {
        (
            EvidenceReference::Local { observed_sha256: old_hash, .. },
            EvidenceReference::Local { observed_sha256: new_hash, .. },
        ) => old_hash != new_hash,
        _ => false,
    }
}

fn declared_evidence_reference_equal(old: &EvidenceReference, new: &EvidenceReference) -> bool {
    match (old, new) {
        (
            EvidenceReference::Local {
                root_key: old_root,
                relative_label: old_label,
                approved_sha256: old_hash,
                approved_size: old_size,
                ..
            },
            EvidenceReference::Local {
                root_key: new_root,
                relative_label: new_label,
                approved_sha256: new_hash,
                approved_size: new_size,
                ..
            },
        ) => {
            old_root == new_root
                && old_label == new_label
                && old_hash == new_hash
                && old_size == new_size
        }
        (
            EvidenceReference::Uri { redacted_uri: old_uri, expected_sha256: old_hash },
            EvidenceReference::Uri { redacted_uri: new_uri, expected_sha256: new_hash },
        ) => old_uri == new_uri && old_hash == new_hash,
        _ => false,
    }
}

fn normalize_relative_label(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn sanitize_reference_label(value: &str) -> String {
    if let Ok(uri) = Url::parse(value)
        && !uri.scheme().is_empty()
    {
        return validate_and_redact_uri(value, &[uri.scheme().to_string()])
            .unwrap_or_else(|_| "redacted-reference".to_string());
    }
    let looks_windows_absolute = value.as_bytes().get(1) == Some(&b':')
        && value.as_bytes().get(2).is_some_and(|byte| matches!(byte, b'/' | b'\\'));
    let path = Path::new(value);
    if path.is_absolute() || looks_windows_absolute {
        return safe_file_label(path);
    }
    escape(value)
}

fn safe_file_label(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| "artifact.json".to_string(), |name| name.to_string_lossy().into_owned())
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn safe(value: &str) -> String {
    crate::json_strict::bounded(value)
}

fn escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            character if character.is_control() => escaped.extend(character.escape_default()),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn relabel_mapping_error(error: ForgeError) -> ForgeError {
    match error {
        ForgeError::MappingBuild(detail) => ForgeError::Linkage(detail),
        other => other,
    }
}

fn error(detail: impl Into<String>) -> ForgeError {
    ForgeError::Linkage(detail.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_boundaries_use_only_explicit_date() {
        let as_of = NaiveDate::from_ymd_opt(2026, 8, 27).unwrap();
        assert_eq!(date_freshness(Some(as_of), as_of, 30).unwrap(), EvidenceFreshness::Expired);
        assert_eq!(
            date_freshness(Some(as_of + Duration::days(30)), as_of, 30).unwrap(),
            EvidenceFreshness::Expiring
        );
        assert_eq!(
            date_freshness(Some(as_of + Duration::days(31)), as_of, 30).unwrap(),
            EvidenceFreshness::Current
        );
    }

    #[test]
    fn uri_redaction_removes_credentials_query_and_fragment() {
        let redacted = validate_and_redact_uri(
            "https://user:secret@example.com/report?id=credential#section",
            &[],
        )
        .unwrap();
        assert_eq!(redacted, "https://example.com/report");
        assert!(!redacted.contains("secret"));
        assert!(!redacted.contains("credential"));
    }

    #[test]
    fn uri_policy_rejects_http_and_allows_explicit_custom_scheme() {
        assert!(validate_and_redact_uri("http://example.com/evidence", &[]).is_err());
        assert!(validate_and_redact_uri("vault+corp://records/1", &["vault+corp".into()]).is_ok());
    }

    #[test]
    fn stable_ids_ignore_paths_order_and_prose() {
        let first = finding(
            "project",
            "evidence-changed",
            Some("link"),
            Some("ev"),
            None,
            None,
            true,
            "one",
        );
        let second = finding(
            "project",
            "evidence-changed",
            Some("link"),
            Some("ev"),
            None,
            None,
            true,
            "two",
        );
        assert_eq!(first.finding_id, second.finding_id);
    }

    #[test]
    fn implementation_inventory_enforces_canonical_subject_limit() {
        assert!(
            validate_implementation_inventory_capacity(inventory::MAX_INVENTORY_SUBJECTS - 1)
                .is_ok()
        );
        let failure = validate_implementation_inventory_capacity(inventory::MAX_INVENTORY_SUBJECTS)
            .expect_err("limit must reject another subject");
        assert!(failure.to_string().contains("subject inventory limit"));
    }

    #[test]
    fn reports_escape_terminal_and_html_injection() {
        assert_eq!(escape("safe\nnext"), "safe\\nnext");
        assert_eq!(html("<script>\n"), "&lt;script&gt;\\n");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_and_special_file_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        let link = dir.path().join("link");
        std::fs::write(&target, b"evidence").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(reject_symlink_components(dir.path(), Path::new("link"), true).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn descriptor_relative_open_rejects_intermediate_symlink_escape() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("root");
        let outside = dir.path().join("outside");
        std::fs::create_dir(&root).expect("root");
        std::fs::create_dir(&outside).expect("outside");
        std::fs::write(outside.join("secret.bin"), b"must not be read").expect("secret");
        std::os::unix::fs::symlink(&outside, root.join("escape")).expect("symlink");

        let failure = open_confined_evidence(
            &root.canonicalize().expect("canonical root"),
            Path::new("escape/secret.bin"),
            "record",
        )
        .expect_err("intermediate symlink must fail closed");
        assert!(failure.to_string().contains("open evidence path component"));
    }
}
