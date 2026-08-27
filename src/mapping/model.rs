//! Typed OSCAL Control Mapping construction and deterministic review reporting.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::inventory::{FORGE_MAPPING_NS, Inventory, LoadedResource, ResourceEvidence};
use super::manifest::{
    ConfidenceCategory, ConfidenceScoreManifest, CoverageGenerationMethod, MapManifest,
    MappingManifest, MappingMethod, MappingStatus, MatchingRationale, QualifierCategory,
    QualifierPredicate, QualifierSubject, Relationship, ReviewScope, ReviewerType, SubjectManifest,
    SubjectType,
};
use crate::{ForgeError, uuid::FORGE_NAMESPACE_UUID};

pub const REPORT_SCHEMA_VERSION: &str = "forge.mapping-report/1";
const UUID_SEED_VERSION: &str = "forge.mapping/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingCollectionEnvelope {
    #[serde(rename = "mapping-collection")]
    pub mapping_collection: MappingCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MappingCollection {
    pub uuid: Uuid,
    pub metadata: MappingMetadata,
    pub provenance: MappingProvenance,
    pub mappings: Vec<OscalMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MappingMetadata {
    pub title: String,
    pub last_modified: String,
    pub version: String,
    pub oscal_version: String,
    #[serde(default)]
    pub props: Vec<OscalProp>,
    #[serde(default)]
    pub roles: Vec<OscalRole>,
    #[serde(default)]
    pub parties: Vec<OscalParty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRole {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalParty {
    pub uuid: Uuid,
    #[serde(rename = "type")]
    pub party_type: ReviewerType,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MappingProvenance {
    pub method: MappingMethod,
    pub matching_rationale: MatchingRationale,
    pub status: MappingStatus,
    pub mapping_description: String,
    #[serde(default)]
    pub responsible_parties: Vec<ResponsibleParty>,
    #[serde(default)]
    pub props: Vec<OscalProp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResponsibleParty {
    pub role_id: String,
    pub party_uuids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscalMapping {
    pub uuid: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<MappingMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_rationale: Option<MatchingRationale>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MappingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_description: Option<String>,
    pub source_resource: MappingResourceReference,
    pub target_resource: MappingResourceReference,
    pub maps: Vec<OscalMap>,
    #[serde(default)]
    pub props: Vec<OscalProp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<OscalConfidenceScore>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<OscalCoverage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_gap_summary: Option<GapSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_gap_summary: Option<GapSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResourceReference {
    #[serde(rename = "type")]
    pub resource_type: super::manifest::ResourceType,
    pub href: String,
    #[serde(default)]
    pub props: Vec<OscalProp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct OscalMap {
    pub uuid: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching_rationale: Option<MatchingRationale>,
    pub relationship: Relationship,
    pub sources: Vec<MappingItem>,
    pub targets: Vec<MappingItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<OscalConfidenceScore>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage: Option<OscalCoverage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<OscalQualifier>,
    #[serde(default)]
    pub props: Vec<OscalProp>,
    #[serde(default)]
    pub remarks: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OscalCoverage {
    pub generation_method: CoverageGenerationMethod,
    pub target_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OscalQualifier {
    pub subject: QualifierSubject,
    pub predicate: QualifierPredicate,
    pub category: QualifierCategory,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MappingItem {
    #[serde(rename = "type")]
    pub subject_type: SubjectType,
    pub id_ref: String,
    #[serde(default)]
    pub props: Vec<OscalProp>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OscalConfidenceScore {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ConfidenceCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalProp {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GapSummary {
    pub uuid: Uuid,
    pub unmapped_controls: Vec<ControlSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ControlSelection {
    pub with_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MappingReport {
    pub schema_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'static str>,
    pub source: ResourceEvidence,
    pub target: ResourceEvidence,
    pub scope: ReviewScope,
    pub source_controls: Participation,
    pub target_controls: Participation,
    pub source_statements: Participation,
    pub target_statements: Participation,
    pub validation: ValidationSummary,
    pub findings: Vec<ImpactFinding>,
    pub author_estimates: Vec<AuthorEstimate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub excerpts: Vec<ReportExcerpt>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorEstimate {
    pub map_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ConfidenceScoreManifest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_coverage: Option<f64>,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportExcerpt {
    pub side: &'static str,
    pub subject_type: SubjectType,
    pub id: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Participation {
    pub eligible: usize,
    pub referenced: usize,
    pub ratio: f64,
    pub unmapped_ids: Vec<String>,
}

/// Validation gates for the stages completed before a report is rendered.
///
/// Each `true` value means the corresponding stage and every preceding stage completed
/// successfully; mapping-schema validation is set only by [`BuildProduct::finalize_report`].
#[derive(Debug, Clone, Serialize)]
#[allow(clippy::struct_excessive_bools)] // Explicit gates make the machine report auditable.
pub struct ValidationSummary {
    pub manifest_valid: bool,
    pub resources_valid: bool,
    pub references_valid: bool,
    pub mapping_schema_valid: bool,
}

impl ValidationSummary {
    const fn after_reference_validation() -> Self {
        Self {
            manifest_valid: true,
            resources_valid: true,
            references_valid: true,
            mapping_schema_valid: false,
        }
    }

    fn finalize_mapping_schema(&mut self, mapping_schema_valid: bool) {
        self.mapping_schema_valid = mapping_schema_valid;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImpactFinding {
    pub severity: FindingSeverity,
    pub code: String,
    pub path: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Review,
    Info,
}

pub struct BuildProduct {
    pub artifact: MappingCollectionEnvelope,
    pub report: MappingReport,
}

impl BuildProduct {
    /// Mark a fully analyzed report as complete after Mapping schema validation succeeds.
    pub(crate) fn finalize_report(&mut self, mapping_schema_valid: bool) {
        self.report.validation.finalize_mapping_schema(mapping_schema_valid);
        self.report.status = mapping_schema_valid.then_some("complete");
    }
}

/// Build typed OSCAL Mapping and deterministic report structures.
///
/// # Errors
///
/// Returns [`ForgeError::MappingBuild`] for invalid references or stable UUID collisions.
#[allow(clippy::too_many_lines)] // Keeps the complete deterministic assembly sequence auditable.
pub fn build(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
) -> Result<BuildProduct, ForgeError> {
    let reviewer_uuids: BTreeMap<_, _> = manifest
        .reviewers
        .iter()
        .map(|reviewer| (reviewer.key.as_str(), stable_uuid("party", &reviewer.key)))
        .collect();
    ensure_unique_uuids(manifest, &reviewer_uuids)?;
    let mut source_referenced = BTreeSet::new();
    let mut target_referenced = BTreeSet::new();
    let mut maps = Vec::with_capacity(manifest.mapping.maps.len());
    for (index, map) in manifest.mapping.maps.iter().enumerate() {
        if manifest.mapping.scope == ReviewScope::ControlOnly
            && map
                .sources
                .iter()
                .chain(&map.targets)
                .any(|subject| subject.subject_type == SubjectType::Statement)
        {
            return Err(mapping_error(format!(
                "$.mapping.maps[{index}] contains a statement outside control-only scope"
            )));
        }
        maps.push(build_map(
            index,
            map,
            &source.inventory,
            &target.inventory,
            &mut source_referenced,
            &mut target_referenced,
        )?);
    }
    maps.sort_by_key(|map| map.uuid);

    let source_controls =
        participation(&source.inventory, SubjectType::Control, &source_referenced);
    let target_controls =
        participation(&target.inventory, SubjectType::Control, &target_referenced);
    let (source_statements, target_statements) =
        if manifest.mapping.scope == ReviewScope::ControlPlusStatement {
            (
                participation(&source.inventory, SubjectType::Statement, &source_referenced),
                participation(&target.inventory, SubjectType::Statement, &target_referenced),
            )
        } else {
            (empty_participation(), empty_participation())
        };

    let source_gap_summary =
        gap_summary("source-gap-summary", &manifest.mapping.key, &source_controls.unmapped_ids);
    let target_gap_summary =
        gap_summary("target-gap-summary", &manifest.mapping.key, &target_controls.unmapped_ids);

    let parties = manifest
        .reviewers
        .iter()
        .map(|reviewer| OscalParty {
            uuid: reviewer_uuids[reviewer.key.as_str()],
            party_type: reviewer.party_type,
            name: reviewer.name.clone(),
        })
        .collect();
    let responsible_party_uuids = manifest
        .provenance
        .reviewer_keys
        .iter()
        .map(|key| {
            reviewer_uuids.get(key.as_str()).copied().ok_or_else(|| {
                mapping_error(format!(
                    "$.provenance.reviewer_keys references unknown reviewer '{}'",
                    bounded(key)
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let artifact = MappingCollectionEnvelope {
        mapping_collection: MappingCollection {
            uuid: stable_uuid("collection", &manifest.collection.key),
            metadata: MappingMetadata {
                title: manifest.collection.title.clone(),
                last_modified: manifest.collection.last_modified.clone(),
                version: manifest.collection.version.clone(),
                oscal_version: crate::oscal::OSCAL_VERSION.to_string(),
                props: vec![forge_prop("collection-key", &manifest.collection.key)],
                roles: vec![OscalRole {
                    id: "mapping-reviewer".to_string(),
                    title: "Mapping Reviewer".to_string(),
                }],
                parties,
            },
            provenance: MappingProvenance {
                method: manifest.provenance.method,
                matching_rationale: manifest.provenance.matching_rationale,
                status: manifest.provenance.status,
                mapping_description: manifest.provenance.mapping_description.clone(),
                responsible_parties: vec![ResponsibleParty {
                    role_id: "mapping-reviewer".to_string(),
                    party_uuids: responsible_party_uuids,
                }],
                props: vec![forge_prop("reviewed-at", &manifest.provenance.reviewed_at)],
            },
            mappings: vec![OscalMapping {
                uuid: stable_uuid("mapping", &manifest.mapping.key),
                method: manifest.mapping.method,
                matching_rationale: manifest.mapping.matching_rationale,
                status: manifest.mapping.status,
                mapping_description: manifest.mapping.mapping_description.clone(),
                source_resource: resource_reference(&source.evidence),
                target_resource: resource_reference(&target.evidence),
                maps,
                props: vec![forge_prop("mapping-key", &manifest.mapping.key)],
                confidence_score: manifest.mapping.confidence_score.as_ref().map(confidence_score),
                coverage: manifest.mapping.coverage.as_ref().map(|coverage| OscalCoverage {
                    generation_method: coverage.generation_method,
                    target_coverage: coverage.target_coverage,
                }),
                source_gap_summary,
                target_gap_summary,
            }],
        },
    };
    let report = MappingReport {
        schema_version: REPORT_SCHEMA_VERSION,
        status: None,
        source: source.evidence.clone(),
        target: target.evidence.clone(),
        scope: manifest.mapping.scope,
        source_controls,
        target_controls,
        source_statements,
        target_statements,
        validation: ValidationSummary::after_reference_validation(),
        findings: Vec::new(),
        author_estimates: author_estimates(manifest),
        excerpts: if include_excerpts {
            report_excerpts(&source.inventory, &target.inventory, manifest.mapping.scope)?
        } else {
            Vec::new()
        },
    };
    Ok(BuildProduct { artifact, report })
}

fn author_estimates(manifest: &MappingManifest) -> Vec<AuthorEstimate> {
    let mut estimates = Vec::new();
    if manifest.mapping.confidence_score.is_some() || manifest.mapping.coverage.is_some() {
        estimates.push(AuthorEstimate {
            map_key: manifest.mapping.key.clone(),
            confidence: manifest.mapping.confidence_score.clone(),
            target_coverage: manifest
                .mapping
                .coverage
                .as_ref()
                .map(|coverage| coverage.target_coverage),
            label: "reviewer_estimate_not_compliance_coverage",
        });
    }
    estimates.extend(
        manifest
            .mapping
            .maps
            .iter()
            .filter(|map| map.confidence_score.is_some() || map.coverage.is_some())
            .map(|map| AuthorEstimate {
                map_key: map.key.clone(),
                confidence: map.confidence_score.clone(),
                target_coverage: map.coverage.as_ref().map(|coverage| coverage.target_coverage),
                label: "reviewer_estimate_not_compliance_coverage",
            }),
    );
    estimates
}

fn empty_participation() -> Participation {
    Participation { eligible: 0, referenced: 0, ratio: 0.0, unmapped_ids: Vec::new() }
}

fn report_excerpts(
    source: &Inventory,
    target: &Inventory,
    scope: ReviewScope,
) -> Result<Vec<ReportExcerpt>, ForgeError> {
    const MAX_REPORT_EXCERPTS: usize = 1_000;
    let kinds: &[SubjectType] = if scope == ReviewScope::ControlOnly {
        &[SubjectType::Control]
    } else {
        &[SubjectType::Control, SubjectType::Statement]
    };
    let mut excerpts = Vec::new();
    for (side, inventory) in [("source", source), ("target", target)] {
        for &subject_type in kinds {
            for id in inventory.ids_of_type(subject_type) {
                let Some(excerpt) = inventory.excerpt(subject_type, &id) else {
                    continue;
                };
                if excerpts.len() == MAX_REPORT_EXCERPTS {
                    return Err(mapping_error(format!(
                        "report excerpts exceed the {MAX_REPORT_EXCERPTS} entry limit"
                    )));
                }
                excerpts.push(ReportExcerpt {
                    side,
                    subject_type,
                    excerpt: excerpt.to_string(),
                    id,
                });
            }
        }
    }
    Ok(excerpts)
}

fn build_map(
    index: usize,
    map: &MapManifest,
    source: &Inventory,
    target: &Inventory,
    source_referenced: &mut BTreeSet<(SubjectType, String)>,
    target_referenced: &mut BTreeSet<(SubjectType, String)>,
) -> Result<OscalMap, ForgeError> {
    let mut sources = build_items(
        &format!("$.mapping.maps[{index}].sources"),
        &map.sources,
        source,
        source_referenced,
    )?;
    let mut targets = build_items(
        &format!("$.mapping.maps[{index}].targets"),
        &map.targets,
        target,
        target_referenced,
    )?;
    sources.sort_by(|left, right| {
        (left.subject_type, left.id_ref.as_str()).cmp(&(right.subject_type, right.id_ref.as_str()))
    });
    targets.sort_by(|left, right| {
        (left.subject_type, left.id_ref.as_str()).cmp(&(right.subject_type, right.id_ref.as_str()))
    });
    Ok(OscalMap {
        uuid: stable_uuid("map", &map.key),
        matching_rationale: map.matching_rationale,
        relationship: map.relationship,
        sources,
        targets,
        confidence_score: map.confidence_score.as_ref().map(confidence_score),
        coverage: map.coverage.as_ref().map(|coverage| OscalCoverage {
            generation_method: coverage.generation_method,
            target_coverage: coverage.target_coverage,
        }),
        qualifiers: map
            .qualifiers
            .iter()
            .map(|qualifier| OscalQualifier {
                subject: qualifier.subject,
                predicate: qualifier.predicate,
                category: qualifier.category,
                description: qualifier.description.clone(),
            })
            .collect(),
        props: vec![
            forge_prop("map-key", &map.key),
            forge_prop("reviewer-key", &map.reviewer_key),
            forge_prop("reviewed-at", &map.reviewed_at),
        ],
        remarks: map.rationale.clone(),
    })
}

fn build_items(
    path: &str,
    subjects: &[SubjectManifest],
    inventory: &Inventory,
    referenced: &mut BTreeSet<(SubjectType, String)>,
) -> Result<Vec<MappingItem>, ForgeError> {
    subjects
        .iter()
        .enumerate()
        .map(|(index, subject)| {
            let Some(fingerprint) = inventory.fingerprint(subject.subject_type, &subject.id_ref)
            else {
                let detail = inventory.type_for_id(&subject.id_ref).map_or_else(
                    || {
                        inventory.ineligible_part_name(&subject.id_ref).map_or_else(
                            || "does not exist".to_string(),
                            |name| format!("exists as ineligible part type '{name}'"),
                        )
                    },
                    |actual| format!("exists as type '{}'", actual.as_str()),
                );
                return Err(mapping_error(format!(
                    "{path}[{index}] {} '{}' {detail}",
                    subject.subject_type.as_str(),
                    bounded(&subject.id_ref)
                )));
            };
            referenced.insert((subject.subject_type, subject.id_ref.clone()));
            Ok(MappingItem {
                subject_type: subject.subject_type,
                id_ref: subject.id_ref.clone(),
                props: vec![forge_prop("subject-sha256", fingerprint)],
            })
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)] // Counts are bounded to 100,000 subjects.
fn participation(
    inventory: &Inventory,
    subject_type: SubjectType,
    referenced: &BTreeSet<(SubjectType, String)>,
) -> Participation {
    let all = inventory.ids_of_type(subject_type);
    let used: BTreeSet<_> = referenced
        .iter()
        .filter(|(kind, _)| *kind == subject_type)
        .map(|(_, id)| id.clone())
        .collect();
    let eligible = inventory.count(subject_type);
    let referenced = used.len();
    let ratio = if eligible == 0 { 0.0 } else { referenced as f64 / eligible as f64 };
    Participation {
        eligible,
        referenced,
        ratio,
        unmapped_ids: all.difference(&used).cloned().collect(),
    }
}

fn gap_summary(kind: &str, mapping_key: &str, ids: &[String]) -> Option<GapSummary> {
    if ids.is_empty() {
        None
    } else {
        Some(GapSummary {
            uuid: stable_uuid(kind, mapping_key),
            unmapped_controls: vec![ControlSelection { with_ids: ids.to_vec() }],
        })
    }
}

fn resource_reference(evidence: &ResourceEvidence) -> MappingResourceReference {
    let mut props = vec![
        forge_prop("raw-sha256", &evidence.raw_sha256),
        forge_prop("root-uuid", &evidence.root_uuid),
        forge_prop("document-version", &evidence.document_version),
        forge_prop("oscal-version", &evidence.oscal_version),
    ];
    if let Some(hash) = &evidence.resolved_catalog_sha256 {
        props.push(forge_prop("resolved-catalog-sha256", hash));
    }
    MappingResourceReference {
        resource_type: evidence.resource_type,
        href: evidence.href.clone(),
        props,
    }
}

fn confidence_score(score: &ConfidenceScoreManifest) -> OscalConfidenceScore {
    OscalConfidenceScore { category: score.category, percentage: score.percentage }
}

#[must_use]
pub fn forge_prop(name: &str, value: &str) -> OscalProp {
    OscalProp {
        name: name.to_string(),
        ns: Some(FORGE_MAPPING_NS.to_string()),
        value: value.to_string(),
    }
}

#[must_use]
pub fn stable_uuid(kind: &str, key: &str) -> Uuid {
    Uuid::new_v5(&FORGE_NAMESPACE_UUID, format!("{UUID_SEED_VERSION}:{kind}:{key}").as_bytes())
}

fn ensure_unique_uuids(
    manifest: &MappingManifest,
    reviewer_uuids: &BTreeMap<&str, Uuid>,
) -> Result<(), ForgeError> {
    let mut seen = BTreeSet::new();
    let candidates = [
        stable_uuid("collection", &manifest.collection.key),
        stable_uuid("mapping", &manifest.mapping.key),
        stable_uuid("source-gap-summary", &manifest.mapping.key),
        stable_uuid("target-gap-summary", &manifest.mapping.key),
    ];
    for uuid in candidates
        .into_iter()
        .chain(reviewer_uuids.values().copied())
        .chain(manifest.mapping.maps.iter().map(|map| stable_uuid("map", &map.key)))
    {
        if !seen.insert(uuid) {
            return Err(mapping_error(format!("stable UUID collision detected for {uuid}")));
        }
    }
    Ok(())
}

fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

fn mapping_error(message: impl Into<String>) -> ForgeError {
    ForgeError::MappingBuild(message.into())
}
