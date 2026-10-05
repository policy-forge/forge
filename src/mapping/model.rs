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
const SOURCE_GAP_SUMMARY_KIND: &str = "source-gap-summary";
const TARGET_GAP_SUMMARY_KIND: &str = "target-gap-summary";

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
    /// OSCAL gap summaries carry controls only; unmapped statements remain in the review report.
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
/// successfully; mapping-schema validation is set only by `BuildProduct::finalize_report`.
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
pub fn build(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
) -> Result<BuildProduct, ForgeError> {
    let mut unlimited = |_charge| Ok::<(), std::convert::Infallible>(());
    let mut context = BuildContext { admit: &mut unlimited, controlled: false };
    build_inner(manifest, source, target, include_excerpts, &mut context).map_err(|error| {
        match error {
            CapturedBuildError::Domain(error) => error,
            CapturedBuildError::Admission(never) => match never {},
        }
    })
}

/// Build the complete maintained native model under a caller-owned admission ledger.
///
/// Whole borrowed expansion is reserved before derived allocation. Every controlled
/// comparison/registry/phase uses the same callback; no source/currentness authority
/// is granted. Domain errors and exact caller stops remain distinct. Parsing/schema
/// internals and external IO are owned by the actual capture consumer.
pub(crate) fn build_admitted<E>(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
    admit: &mut impl FnMut(BuildCharge) -> Result<(), E>,
) -> AdmittedResult<BuildProduct, E> {
    let mut context = BuildContext { admit, controlled: true };
    build_inner(manifest, source, target, include_excerpts, &mut context)
}

/// Shared complete native assembly; unlimited callers preserve existing algorithms.
#[allow(clippy::too_many_lines)] // Keeps the complete deterministic assembly sequence auditable.
fn build_inner<E>(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<BuildProduct, E> {
    context.checkpoint()?;
    let plan = if context.controlled {
        Some(preflight_model(manifest, source, target, include_excerpts, context)?)
    } else {
        None
    };
    let reviewer_uuids = ReviewerRegistry::create(manifest, context)?;
    unique_admitted(manifest, &reviewer_uuids, context)?;
    let mut source_referenced = NativeReferences::default();
    let mut target_referenced = NativeReferences::default();
    if let Some(plan) = &plan {
        source_referenced.controlled = Vec::with_capacity(plan.source);
        target_referenced.controlled = Vec::with_capacity(plan.target);
    }
    let mut maps = Vec::with_capacity(manifest.mapping.maps.len());
    for (index, map) in manifest.mapping.maps.iter().enumerate() {
        context.checkpoint()?;
        context.work(1, map.key.len())?;
        if manifest.mapping.scope == ReviewScope::ControlOnly
            && map
                .sources
                .iter()
                .chain(&map.targets)
                .any(|subject| subject.subject_type == SubjectType::Statement)
        {
            return Err(mapping_error(format!(
                "$.mapping.maps[{index}] contains a statement outside control-only scope"
            ))
            .into());
        }
        maps.push(map_admitted(
            index,
            map,
            &source.inventory,
            &target.inventory,
            &mut source_referenced,
            &mut target_referenced,
            context,
        )?);
    }
    if context.controlled {
        sort_admitted(
            &mut maps,
            context,
            |_left, _right| Some(32),
            |left, right| left.uuid.cmp(&right.uuid),
        )?;
    } else {
        maps.sort_by_key(|map| map.uuid);
    }

    let source_controls = participation_admitted(
        &source.inventory,
        SubjectType::Control,
        &source_referenced,
        context,
    )?;
    let target_controls = participation_admitted(
        &target.inventory,
        SubjectType::Control,
        &target_referenced,
        context,
    )?;
    let (source_statements, target_statements) =
        if manifest.mapping.scope == ReviewScope::ControlPlusStatement {
            (
                participation_admitted(
                    &source.inventory,
                    SubjectType::Statement,
                    &source_referenced,
                    context,
                )?,
                participation_admitted(
                    &target.inventory,
                    SubjectType::Statement,
                    &target_referenced,
                    context,
                )?,
            )
        } else {
            (empty_participation(), empty_participation())
        };

    context.checkpoint()?;
    if context.controlled {
        charge_uuid(SOURCE_GAP_SUMMARY_KIND, &manifest.mapping.key, context)?;
        charge_uuid(TARGET_GAP_SUMMARY_KIND, &manifest.mapping.key, context)?;
        charge_uuid("collection", &manifest.collection.key, context)?;
        charge_uuid("mapping", &manifest.mapping.key, context)?;
    }
    let source_gap_summary =
        gap_summary(SOURCE_GAP_SUMMARY_KIND, &manifest.mapping.key, &source_controls.unmapped_ids);
    let target_gap_summary =
        gap_summary(TARGET_GAP_SUMMARY_KIND, &manifest.mapping.key, &target_controls.unmapped_ids);

    context.checkpoint()?;
    let mut parties = Vec::with_capacity(manifest.reviewers.len());
    for reviewer in &manifest.reviewers {
        context.checkpoint()?;
        context.work(1, reviewer.name.len())?;
        let uuid = reviewer_uuids
            .get(&reviewer.key, context)?
            .ok_or_else(|| mapping_error("native reviewer lookup invalid"))?;
        parties.push(OscalParty {
            uuid,
            party_type: reviewer.party_type,
            name: reviewer.name.clone(),
        });
    }
    let mut responsible_party_uuids = Vec::with_capacity(manifest.provenance.reviewer_keys.len());
    for key in &manifest.provenance.reviewer_keys {
        context.checkpoint()?;
        let uuid = reviewer_uuids.get(key, context)?.ok_or_else(|| {
            mapping_error(format!(
                "$.provenance.reviewer_keys references unknown reviewer '{}'",
                bounded(key)
            ))
        })?;
        responsible_party_uuids.push(uuid);
    }
    context.checkpoint()?;
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
    context.checkpoint()?;
    context.work(
        manifest.mapping.maps.len(),
        manifest
            .mapping
            .maps
            .iter()
            .try_fold(0_usize, |sum, row| sum.checked_add(row.key.len()))
            .unwrap_or(usize::MAX),
    )?;
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
            excerpts_admitted(
                &source.inventory,
                &target.inventory,
                manifest.mapping.scope,
                plan.as_ref().map_or(0, |plan| plan.excerpts),
                context,
            )?
        } else {
            Vec::new()
        },
    };
    context.checkpoint()?;
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

/// Build and sort one legacy Mapping assertion from validated subjects.
/// The controlled producer uses its separately admitted counterpart.
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
    Ok(assemble_map(map, sources, targets))
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
        stable_uuid(SOURCE_GAP_SUMMARY_KIND, &manifest.mapping.key),
        stable_uuid(TARGET_GAP_SUMMARY_KIND, &manifest.mapping.key),
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

/// Preserve the maintained native assertion fields in one shared constructor.
fn assemble_map(
    map: &MapManifest,
    sources: Vec<MappingItem>,
    targets: Vec<MappingItem>,
) -> OscalMap {
    OscalMap {
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
    }
}

/// One before-growth request to the caller-owned native/work/storage ledger.
#[derive(Clone, Copy, Debug)]
pub(crate) enum BuildCharge {
    /// Check the same accepted deadline and sticky control reason.
    Checkpoint,
    /// Admit complete actual inspection/comparison extents before work.
    Work {
        /// Inspected rows, conservative complete relationship steps, or moved fixed slots.
        visits: usize,
        /// Complete primitive byte extent, including both comparison operands.
        byte_work: usize,
    },
    /// Reserve complete logical derived capacity before model growth.
    Reserve {
        /// Conservative retained/transient payload allowance, not total heap.
        logical_bytes: usize,
    },
}

/// Keep caller admission/control failure distinct from maintained native errors.
#[derive(Debug)]
pub(crate) enum CapturedBuildError<E> {
    /// Existing native reference/collision/schema-shape error.
    Domain(ForgeError),
    /// Exact caller-owned ledger/control stop, never formatted as native prose.
    Admission(E),
}

impl<E> From<ForgeError> for CapturedBuildError<E> {
    /// Preserve native failures without authorizing an incomplete model.
    fn from(error: ForgeError) -> Self {
        Self::Domain(error)
    }
}

/// Result shared by actual admitted native constructors and their callbacks.
pub(crate) type AdmittedResult<T, E> = Result<T, CapturedBuildError<E>>;

/// Actual callback owner for one complete model; no independent ledger exists here.
struct BuildContext<'a, E> {
    /// Root-owned monotonic callback used by every native phase.
    admit: &'a mut dyn FnMut(BuildCharge) -> Result<(), E>,
    /// Only the new admitted path uses explicit fallible registries/comparisons.
    controlled: bool,
}

impl<E> BuildContext<'_, E> {
    /// Preserve the actual first callback error at the caller boundary.
    fn charge(&mut self, charge: BuildCharge) -> AdmittedResult<(), E> {
        (self.admit)(charge).map_err(CapturedBuildError::Admission)
    }
    /// Check the shared accepted control before a bounded phase.
    fn checkpoint(&mut self) -> AdmittedResult<(), E> {
        self.charge(BuildCharge::Checkpoint)
    }
    /// Admit all repeated primitive work before reading/comparing/copying its bytes.
    fn work(&mut self, visits: usize, byte_work: usize) -> AdmittedResult<(), E> {
        self.charge(BuildCharge::Work { visits, byte_work })
    }
    /// Force checked-overflow/profile exhaustion through the actual callback first.
    fn exhausted<T>(&mut self) -> AdmittedResult<T, E> {
        self.work(usize::MAX, usize::MAX)?;
        Err(mapping_error("admitted Mapping capacity exceeded").into())
    }
    /// Checked addition never wraps a complete expansion or comparison extent.
    fn add(&mut self, left: usize, right: usize) -> AdmittedResult<usize, E> {
        match left.checked_add(right) {
            Some(value) => Ok(value),
            None => self.exhausted(),
        }
    }
    /// Checked multiplication never truncates a complete expansion denominator.
    fn mul(&mut self, left: usize, right: usize) -> AdmittedResult<usize, E> {
        match left.checked_mul(right) {
            Some(value) => Ok(value),
            None => self.exhausted(),
        }
    }
    /// Meter the two complete string operands before native lexicographic comparison.
    fn compare(&mut self, left: &str, right: &str) -> AdmittedResult<std::cmp::Ordering, E> {
        let extent = self.add(left.len(), right.len())?;
        self.work(1, extent)?;
        Ok(left.cmp(right))
    }
}

/// Borrowed complete preflight totals; no input string or model is cloned here.
#[derive(Default)]
struct ModelPreflight {
    /// Conservative semantic record/container/temporary registry slots.
    records: usize,
    /// Complete owned string occurrences, including pessimistic unmapped copies.
    strings: usize,
    /// Exact completed compact numeric byte occurrences plus bounded native ratios.
    numbers: usize,
}

impl ModelPreflight {
    /// Add slots before any corresponding constructor or registry allocation.
    fn records<E>(
        &mut self,
        amount: usize,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<(), E> {
        self.records = context.add(self.records, amount)?;
        Ok(())
    }
    /// Borrow and charge the complete UTF-8 extent and all declared clone occurrences.
    fn string<E>(
        &mut self,
        value: &str,
        copies: usize,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<(), E> {
        let amount = context.mul(value.len(), copies)?;
        context.work(1, amount)?;
        self.strings = context.add(self.strings, amount)?;
        Ok(())
    }
    /// Count the actual borrowed numeric scalar without retaining serialized bytes.
    fn number<E>(
        &mut self,
        value: f64,
        copies: usize,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<(), E> {
        let mut counter = PrimitiveCounter { context, length: 0, failed: None };
        if serde_json::to_writer(&mut counter, &value).is_err() {
            return Err(counter
                .failed
                .unwrap_or_else(|| mapping_error("invalid admitted numeric primitive").into()));
        }
        let bytes = counter.context.mul(counter.length, copies)?;
        self.numbers = counter.context.add(self.numbers, bytes)?;
        Ok(())
    }
    /// Reserve three logical representation allowances and complete retained encoding.
    ///
    /// This conservative profile covers the typed product/temporary registries,
    /// one independent schema/native Value and one combined native/report encoded
    /// allowance. It is not allocator/parser confinement; additional copies must
    /// be separately admitted by the actual caller before they grow.
    fn reserve<E>(&self, context: &mut BuildContext<'_, E>) -> AdmittedResult<(), E> {
        let slot_bytes = context.mul(self.records, 256)?;
        let payload = context.add(slot_bytes, self.strings)?;
        let representations = context.mul(payload, 3)?;
        let escaped = context.mul(self.strings, 6)?;
        let syntax = context.mul(self.records, 512)?;
        let encoded = context.add(escaped, syntax)?;
        let encoded = context.add(encoded, self.numbers)?;
        let encoded = context.add(encoded, 4096)?;
        let logical_bytes = context.add(representations, encoded)?;
        context.charge(BuildCharge::Reserve { logical_bytes })
    }
}

/// Nonretaining numeric primitive counter with an explicit conservative profile ceiling.
struct PrimitiveCounter<'a, 'b, E> {
    /// Actual shared control/work callback, not an independent parser budget.
    context: &'a mut BuildContext<'b, E>,
    /// Exact scalar bytes observed so far.
    length: usize,
    /// Precise first caller/native stop, moved back without prose conversion.
    failed: Option<CapturedBuildError<E>>,
}

impl<E> std::io::Write for PrimitiveCounter<'_, '_, E> {
    /// Check whole scalar chunks before counting; no JSON bytes are retained.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.failed.is_some() {
            return Err(std::io::Error::other("numeric count refused"));
        }
        let result = (|| {
            self.context.checkpoint()?;
            let next = self.context.add(self.length, bytes.len())?;
            if next > 4096 {
                return self.context.exhausted();
            }
            self.context.work(1, bytes.len())?;
            self.length = next;
            Ok(())
        })();
        if let Err(error) = result {
            self.failed = Some(error);
            return Err(std::io::Error::other("numeric count refused"));
        }
        Ok(bytes.len())
    }
    /// Preserve the same actual caller stop at serialization completion.
    fn flush(&mut self) -> std::io::Result<()> {
        if self.failed.is_some() {
            return Err(std::io::Error::other("numeric count refused"));
        }
        self.context.checkpoint().map_err(|error| {
            self.failed = Some(error);
            std::io::Error::other("numeric count refused")
        })
    }
}

/// Exact vector capacities admitted without constructing any derived native registry.
struct ModelPlan {
    /// Complete source reference occurrences, including repeated native subjects.
    source: usize,
    /// Complete target reference occurrences, including repeated native subjects.
    target: usize,
    /// Complete emitted native excerpt count, at most the maintained one thousand.
    excerpts: usize,
}

/// Count the complete admitted expansion before the first derived model allocation.
fn preflight_model<E>(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<ModelPlan, E> {
    context.checkpoint()?;
    if manifest.mapping.maps.len() > 10_000 {
        return context.exhausted();
    }
    let mut count = ModelPreflight::default();
    count.records(64, context)?;
    // Four eventual participation ratios use this same 4096-byte primitive
    // profile; actual renderer output must stay beneath the admitted ceiling.
    count.numbers = 4 * 4096;
    if let Some(value) =
        manifest.mapping.confidence_score.as_ref().and_then(|score| score.percentage)
    {
        count.number(value, 2, context)?;
    }
    if let Some(value) = manifest.mapping.coverage.as_ref().map(|coverage| coverage.target_coverage)
    {
        count.number(value, 2, context)?;
    }
    for text in [
        &manifest.collection.key,
        &manifest.collection.title,
        &manifest.collection.version,
        &manifest.collection.last_modified,
        &manifest.provenance.mapping_description,
        &manifest.provenance.reviewed_at,
        &manifest.mapping.key,
    ] {
        count.string(text, 8, context)?;
    }
    if let Some(text) = &manifest.mapping.mapping_description {
        count.string(text, 1, context)?;
    }
    count.records(context.mul(manifest.reviewers.len(), 4)?, context)?;
    for reviewer in &manifest.reviewers {
        context.checkpoint()?;
        count.string(&reviewer.key, 2, context)?;
        count.string(&reviewer.name, 1, context)?;
    }
    count.records(manifest.provenance.reviewer_keys.len(), context)?;
    for key in &manifest.provenance.reviewer_keys {
        count.string(key, 1, context)?;
    }
    let mut pairs = 0_usize;
    let mut source_slots = 0_usize;
    let mut target_slots = 0_usize;
    for map in &manifest.mapping.maps {
        context.checkpoint()?;
        source_slots = context.add(source_slots, map.sources.len())?;
        target_slots = context.add(target_slots, map.targets.len())?;
        let pair_count = context.mul(map.sources.len(), map.targets.len())?;
        pairs = context.add(pairs, pair_count)?;
        if pairs > 100_000 {
            return context.exhausted();
        }
        count.records(12, context)?;
        for text in [&map.key, &map.reviewer_key, &map.reviewed_at] {
            count.string(text, 3, context)?;
        }
        count.string(&map.rationale, 1, context)?;
        if let Some(value) = map.confidence_score.as_ref().and_then(|score| score.percentage) {
            count.number(value, 2, context)?;
        }
        if let Some(value) = map.coverage.as_ref().map(|coverage| coverage.target_coverage) {
            count.number(value, 2, context)?;
        }
        for subject in map.sources.iter().chain(&map.targets) {
            count.records(8, context)?;
            count.string(&subject.id_ref, 4, context)?;
            // The native subject-fingerprint prop owns a fixed 64-byte digest.
            count.string(
                "0000000000000000000000000000000000000000000000000000000000000000",
                1,
                context,
            )?;
        }
        for qualifier in &map.qualifiers {
            count.records(2, context)?;
            count.string(&qualifier.description, 1, context)?;
        }
    }
    let excerpts =
        preflight_resources(manifest, source, target, include_excerpts, &mut count, context)?;
    // Conservatively admit the complete declared many-to-many relationship
    // denominator before native growth, even though the native map stores two arrays.
    context.work(pairs, 0)?;
    // Stable namespace/property/role strings and every fixed native field/key are
    // covered by the per-record allowance, not by a guessed f64 display width.
    count.reserve(context)?;
    context.checkpoint()?;
    Ok(ModelPlan { source: source_slots, target: target_slots, excerpts })
}

/// Count complete borrowed resource evidence/subjects/excerpts before model reservation.
fn preflight_resources<E>(
    manifest: &MappingManifest,
    source: &LoadedResource,
    target: &LoadedResource,
    include_excerpts: bool,
    count: &mut ModelPreflight,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<usize, E> {
    let mut excerpts = 0_usize;
    for resource in [source, target] {
        count.records(16, context)?;
        let evidence = &resource.evidence;
        for text in [
            &evidence.href,
            &evidence.raw_sha256,
            &evidence.root_uuid,
            &evidence.document_version,
            &evidence.oscal_version,
        ] {
            count.string(text, 2, context)?;
        }
        if let Some(hash) = &evidence.resolved_catalog_sha256 {
            count.string(hash, 2, context)?;
        }
        for (_kind, id, _fingerprint) in resource.inventory.subject_entries_refs() {
            context.checkpoint()?;
            // The captured loader admits unique resource/type/ID registry
            // entries once. Both semantic uses still pay repeated work/storage.
            // All-ID, used-ID, unmapped report, gap-summary and comparison copies.
            count.records(5, context)?;
            count.string(id, 5, context)?;
        }
        if include_excerpts {
            for (kind, id, excerpt) in resource.inventory.excerpt_entries_refs() {
                context.checkpoint()?;
                context.work(1, id.len())?;
                if kind != SubjectType::Control
                    && manifest.mapping.scope != ReviewScope::ControlPlusStatement
                {
                    continue;
                }
                excerpts = context.add(excerpts, 1)?;
                if excerpts > 1_000 {
                    return Err(mapping_error("report excerpts exceed the 1000 entry limit").into());
                }
                count.records(1, context)?;
                count.string(id, 1, context)?;
                count.string(excerpt, 1, context)?;
            }
        }
    }
    Ok(excerpts)
}

/// Complete reviewer lookup; controlled native comparisons cannot hide in a tree.
struct ReviewerRegistry<'a> {
    /// Original fast map retained by the unlimited public path.
    legacy: BTreeMap<&'a str, Uuid>,
    /// Controlled sorted borrowed-key registry with actual fallible comparisons.
    controlled: Vec<(&'a str, Uuid)>,
}

impl<'a> ReviewerRegistry<'a> {
    /// Build the actual party UUID registry after whole expansion admission.
    fn create<E>(
        manifest: &'a MappingManifest,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<Self, E> {
        if !context.controlled {
            return Ok(Self {
                legacy: manifest
                    .reviewers
                    .iter()
                    .map(|reviewer| (reviewer.key.as_str(), stable_uuid("party", &reviewer.key)))
                    .collect(),
                controlled: Vec::new(),
            });
        }
        let mut result = Self {
            legacy: BTreeMap::new(),
            controlled: Vec::with_capacity(manifest.reviewers.len()),
        };
        for reviewer in &manifest.reviewers {
            context.checkpoint()?;
            let uuid = uuid_admitted("party", &reviewer.key, context)?;
            let mut low = 0;
            let mut high = result.controlled.len();
            let mut present = false;
            while low < high {
                let middle = low + (high - low) / 2;
                match context.compare(result.controlled[middle].0, &reviewer.key)? {
                    std::cmp::Ordering::Less => low = middle + 1,
                    std::cmp::Ordering::Greater => high = middle,
                    std::cmp::Ordering::Equal => {
                        low = middle;
                        high = middle;
                        present = true;
                    }
                }
            }
            if present {
                // Native manifest parse forbids duplicates; retain last-key behavior
                // for a directly constructed typed declaration as the old map did.
                result.controlled[low].1 = uuid;
            } else {
                let shifted = result.controlled.len() - low;
                let extent = context.mul(shifted, std::mem::size_of::<(&str, Uuid)>())?;
                context.work(shifted, extent)?;
                result.controlled.insert(low, (&reviewer.key, uuid));
            }
        }
        Ok(result)
    }
    /// Resolve a real reviewer key with full operand charges before comparison.
    fn get<E>(
        &self,
        key: &str,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<Option<Uuid>, E> {
        if !context.controlled {
            return Ok(self.legacy.get(key).copied());
        }
        let mut low = 0;
        let mut high = self.controlled.len();
        while low < high {
            context.checkpoint()?;
            let middle = low + (high - low) / 2;
            match context.compare(self.controlled[middle].0, key)? {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok(Some(self.controlled[middle].1)),
            }
        }
        Ok(None)
    }
}

/// Actual full referenced native subjects, never a selected map subset.
#[derive(Default)]
struct NativeReferences {
    /// Existing fast legacy registry; controlled use leaves it empty.
    legacy: BTreeSet<(SubjectType, String)>,
    /// Actual controlled unique sorted type/ID registry.
    controlled: Vec<(SubjectType, String)>,
}

impl NativeReferences {
    /// Compare before inserting/cloning or shifting any controlled native entry.
    fn insert<E>(
        &mut self,
        kind: SubjectType,
        id: &str,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<(), E> {
        let (index, present) = self.locate(kind, id, context)?;
        if present {
            return Ok(());
        }
        let shifted = self.controlled.len() - index;
        let extent = context.mul(shifted, std::mem::size_of::<(SubjectType, String)>())?;
        context.work(shifted, extent)?;
        context.work(1, id.len())?;
        if self.controlled.len() == self.controlled.capacity() {
            return context.exhausted();
        }
        self.controlled.insert(index, (kind, id.to_owned()));
        Ok(())
    }
    /// Meter every exact type/ID comparison of the controlled sorted registry.
    fn locate<E>(
        &self,
        kind: SubjectType,
        id: &str,
        context: &mut BuildContext<'_, E>,
    ) -> AdmittedResult<(usize, bool), E> {
        let mut low = 0;
        let mut high = self.controlled.len();
        while low < high {
            context.checkpoint()?;
            let middle = low + (high - low) / 2;
            let row = &self.controlled[middle];
            let extent = context.add(row.1.len(), id.len())?;
            context.work(1, extent)?;
            match (row.0, row.1.as_str()).cmp(&(kind, id)) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => return Ok((middle, true)),
            }
        }
        Ok((low, false))
    }
}

/// Charge the complete native UUID seed before a maintained helper consumes it.
fn charge_uuid<E>(
    kind: &str,
    key: &str,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<(), E> {
    context.checkpoint()?;
    let extent = context.add(UUID_SEED_VERSION.len(), kind.len())?;
    let extent = context.add(extent, key.len())?;
    let extent = context.add(extent, 2)?;
    context.work(1, extent)
}

/// Return the actual maintained native UUID after its complete seed work is admitted.
fn uuid_admitted<E>(
    kind: &str,
    key: &str,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<Uuid, E> {
    charge_uuid(kind, key, context)?;
    Ok(stable_uuid(kind, key))
}

/// Admit full generated UUID uniqueness with actual fallible native comparisons.
fn unique_admitted<E>(
    manifest: &MappingManifest,
    reviewers: &ReviewerRegistry<'_>,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<(), E> {
    if !context.controlled {
        return ensure_unique_uuids(manifest, &reviewers.legacy).map_err(Into::into);
    }
    let count = context.add(4, reviewers.controlled.len())?;
    let count = context.add(count, manifest.mapping.maps.len())?;
    let mut seen = Vec::<Uuid>::with_capacity(count);
    for (kind, key) in [
        ("collection", manifest.collection.key.as_str()),
        ("mapping", manifest.mapping.key.as_str()),
        (SOURCE_GAP_SUMMARY_KIND, manifest.mapping.key.as_str()),
        (TARGET_GAP_SUMMARY_KIND, manifest.mapping.key.as_str()),
    ]
    .into_iter()
    .chain(manifest.reviewers.iter().map(|row| ("party", row.key.as_str())))
    .chain(manifest.mapping.maps.iter().map(|row| ("map", row.key.as_str())))
    {
        let uuid = uuid_admitted(kind, key, context)?;
        let mut low = 0;
        let mut high = seen.len();
        while low < high {
            context.checkpoint()?;
            context.work(1, 32)?;
            let middle = low + (high - low) / 2;
            match seen[middle].cmp(&uuid) {
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
                std::cmp::Ordering::Equal => {
                    return Err(mapping_error(format!(
                        "stable UUID collision detected for {uuid}"
                    ))
                    .into());
                }
            }
        }
        let shifted = seen.len() - low;
        let extent = context.mul(shifted, std::mem::size_of::<Uuid>())?;
        context.work(shifted, extent)?;
        seen.insert(low, uuid);
    }
    Ok(())
}

/// Sort a pre-admitted vector without allocation or comparisons after refusal.
///
/// The admitted path uses deterministic in-place heap sorting. Every comparison
/// and swap is charged before work; callback errors return immediately instead
/// of changing a standard-library comparator's ordering while it is running.
fn sort_admitted<T, E>(
    rows: &mut [T],
    context: &mut BuildContext<'_, E>,
    extent: impl Fn(&T, &T) -> Option<usize>,
    compare: impl Fn(&T, &T) -> std::cmp::Ordering,
) -> AdmittedResult<(), E> {
    context.checkpoint()?;
    let length = rows.len();
    for start in (0..length / 2).rev() {
        sift_admitted(rows, start, length, context, &extent, &compare)?;
    }
    for end in (1..length).rev() {
        swap_admitted(rows, 0, end, context)?;
        sift_admitted(rows, 0, end, context, &extent, &compare)?;
    }
    context.checkpoint()
}

/// Restore one heap path with exact fallible operand charges before comparisons.
fn sift_admitted<T, E>(
    rows: &mut [T],
    mut root: usize,
    length: usize,
    context: &mut BuildContext<'_, E>,
    extent: &impl Fn(&T, &T) -> Option<usize>,
    compare: &impl Fn(&T, &T) -> std::cmp::Ordering,
) -> AdmittedResult<(), E> {
    loop {
        context.checkpoint()?;
        let child = context.mul(root, 2)?;
        let mut child = context.add(child, 1)?;
        if child >= length {
            return Ok(());
        }
        let right = context.add(child, 1)?;
        if right < length
            && compare_admitted(&rows[child], &rows[right], context, extent, compare)?
                == std::cmp::Ordering::Less
        {
            child = right;
        }
        if compare_admitted(&rows[root], &rows[child], context, extent, compare)?
            != std::cmp::Ordering::Less
        {
            return Ok(());
        }
        swap_admitted(rows, root, child, context)?;
        root = child;
    }
}

/// Charge both complete comparison operands before calling the native ordering.
fn compare_admitted<T, E>(
    left: &T,
    right: &T,
    context: &mut BuildContext<'_, E>,
    extent: &impl Fn(&T, &T) -> Option<usize>,
    compare: &impl Fn(&T, &T) -> std::cmp::Ordering,
) -> AdmittedResult<std::cmp::Ordering, E> {
    context.checkpoint()?;
    let Some(bytes) = extent(left, right) else {
        return context.exhausted();
    };
    context.work(1, bytes)?;
    Ok(compare(left, right))
}

/// Admit complete moved native slots before an in-place swap.
fn swap_admitted<T, E>(
    rows: &mut [T],
    left: usize,
    right: usize,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<(), E> {
    context.checkpoint()?;
    let bytes = context.mul(std::mem::size_of::<T>(), 2)?;
    context.work(2, bytes)?;
    rows.swap(left, right);
    Ok(())
}

/// Resolve actual native fingerprints with explicit fallible, complete row comparisons.
fn resolve_admitted<'a, E>(
    path: &str,
    index: usize,
    subject: &SubjectManifest,
    inventory: &'a Inventory,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<&'a str, E> {
    let mut wrong_kind = None;
    for (kind, id, fingerprint) in inventory.subject_entries_refs() {
        context.checkpoint()?;
        if context.compare(id, &subject.id_ref)? == std::cmp::Ordering::Equal {
            if kind == subject.subject_type {
                return Ok(fingerprint);
            }
            wrong_kind = Some(kind);
        }
    }
    let mut part_name = None;
    for (id, name) in inventory.ineligible_parts_refs() {
        context.checkpoint()?;
        if context.compare(id, &subject.id_ref)? == std::cmp::Ordering::Equal {
            part_name = Some(name);
            break;
        }
    }
    // Error strings are private native diagnostics; actual callers minimize them.
    // Admit their complete escaped/dynamic extent before formatting on refusal.
    let name_extent = part_name.map_or(0, str::len);
    let extent = context.add(path.len(), name_extent)?;
    let escaped_key = context.mul(subject.id_ref.len(), 10)?;
    let extent = context.add(extent, escaped_key)?;
    let logical_bytes = context.add(extent, 256)?;
    context.charge(BuildCharge::Reserve { logical_bytes })?;
    let detail = wrong_kind.map_or_else(
        || {
            part_name.map_or_else(
                || "does not exist".to_string(),
                |name| format!("exists as ineligible part type '{name}'"),
            )
        },
        |kind| format!("exists as type '{}'", kind.as_str()),
    );
    Err(mapping_error(format!(
        "{path}[{index}] {} '{}' {detail}",
        subject.subject_type.as_str(),
        bounded(&subject.id_ref)
    ))
    .into())
}

/// Shared item construction; controlled mode meters every lookup/reference/sort primitive.
fn items_admitted<E>(
    path: &str,
    subjects: &[SubjectManifest],
    inventory: &Inventory,
    references: &mut NativeReferences,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<Vec<MappingItem>, E> {
    if !context.controlled {
        return build_items(path, subjects, inventory, &mut references.legacy).map_err(Into::into);
    }
    let mut items = Vec::with_capacity(subjects.len());
    for (index, subject) in subjects.iter().enumerate() {
        context.checkpoint()?;
        let fingerprint = resolve_admitted(path, index, subject, inventory, context)?;
        references.insert(subject.subject_type, &subject.id_ref, context)?;
        let extent = context.add(subject.id_ref.len(), fingerprint.len())?;
        context.work(1, extent)?;
        items.push(MappingItem {
            subject_type: subject.subject_type,
            id_ref: subject.id_ref.clone(),
            props: vec![forge_prop("subject-sha256", fingerprint)],
        });
    }
    Ok(items)
}

/// Shared native assertion construction after complete source/target admission.
fn map_admitted<E>(
    index: usize,
    map: &MapManifest,
    source: &Inventory,
    target: &Inventory,
    source_refs: &mut NativeReferences,
    target_refs: &mut NativeReferences,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<OscalMap, E> {
    if !context.controlled {
        return build_map(
            index,
            map,
            source,
            target,
            &mut source_refs.legacy,
            &mut target_refs.legacy,
        )
        .map_err(Into::into);
    }
    context.checkpoint()?;
    let mut sources = items_admitted(
        &format!("$.mapping.maps[{index}].sources"),
        &map.sources,
        source,
        source_refs,
        context,
    )?;
    let mut targets = items_admitted(
        &format!("$.mapping.maps[{index}].targets"),
        &map.targets,
        target,
        target_refs,
        context,
    )?;
    for rows in [&mut sources, &mut targets] {
        sort_admitted(
            rows,
            context,
            |left, right| left.id_ref.len().checked_add(right.id_ref.len()),
            |left, right| {
                (left.subject_type, left.id_ref.as_str())
                    .cmp(&(right.subject_type, right.id_ref.as_str()))
            },
        )?;
    }
    context.checkpoint()?;
    charge_uuid("map", &map.key, context)?;
    Ok(assemble_map(map, sources, targets))
}

/// Compute the complete actual participation denominator and pessimistically admitted gaps.
fn participation_admitted<E>(
    inventory: &Inventory,
    kind: SubjectType,
    references: &NativeReferences,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<Participation, E> {
    if !context.controlled {
        return Ok(participation(inventory, kind, &references.legacy));
    }
    let mut used = 0_usize;
    for (subject_type, id) in &references.controlled {
        context.checkpoint()?;
        context.work(1, id.len())?;
        if *subject_type == kind {
            used = context.add(used, 1)?;
        }
    }
    let mut eligible = 0_usize;
    let mut unmapped_ids = Vec::with_capacity(inventory.count(kind));
    for (subject_type, id, _fingerprint) in inventory.subject_entries_refs() {
        context.checkpoint()?;
        context.work(1, id.len())?;
        if subject_type != kind {
            continue;
        }
        eligible = context.add(eligible, 1)?;
        if !references.locate(kind, id, context)?.1 {
            context.work(1, id.len())?;
            unmapped_ids.push(id.to_owned());
        }
    }
    #[allow(clippy::cast_precision_loss)] // Native subjects are pre-admitted <=100,000.
    let ratio = if eligible == 0 { 0.0 } else { used as f64 / eligible as f64 };
    Ok(Participation { eligible, referenced: used, ratio, unmapped_ids })
}

/// Produce bounded actual excerpts in the same native source/kind/ID order.
fn excerpts_admitted<E>(
    source: &Inventory,
    target: &Inventory,
    scope: ReviewScope,
    planned: usize,
    context: &mut BuildContext<'_, E>,
) -> AdmittedResult<Vec<ReportExcerpt>, E> {
    if !context.controlled {
        return report_excerpts(source, target, scope).map_err(Into::into);
    }
    let mut result = Vec::with_capacity(planned);
    for (side, inventory) in [("source", source), ("target", target)] {
        for wanted in [SubjectType::Control, SubjectType::Statement] {
            if wanted == SubjectType::Statement && scope != ReviewScope::ControlPlusStatement {
                continue;
            }
            for (kind, id, excerpt) in inventory.excerpt_entries_refs() {
                context.checkpoint()?;
                context.work(1, id.len())?;
                if kind != wanted {
                    continue;
                }
                if result.len() == 1_000 {
                    return Err(mapping_error("report excerpts exceed the 1000 entry limit").into());
                }
                let extent = context.add(id.len(), excerpt.len())?;
                context.work(1, extent)?;
                result.push(ReportExcerpt {
                    side,
                    subject_type: kind,
                    id: id.to_owned(),
                    excerpt: excerpt.to_owned(),
                });
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
/// Genuine maintained native-loader/model controls, with no lifecycle approval or IO proof.
mod admitted_tests {
    use super::*;
    use serde_json::{Value, json};
    use std::path::Path;

    /// Fixed caller-owned failures used solely by these synthetic native fixtures.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Stop {
        /// Actual injected callback interruption at a reached model boundary.
        Cancel,
        /// Actual precharge refusal, not an ordinary native reference error.
        Capacity,
    }

    /// Redistributable native Catalog fixture with mapped and genuinely unmapped controls.
    fn catalog(uuid: &str, prefix: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({"catalog":{"uuid":uuid,"metadata":{
            "title":"Synthetic native model control","last-modified":"2026-08-22T17:00:00Z",
            "version":"1.0.0","oscal-version":"1.2.3"},"controls":[
            {"id":format!("{prefix}-1"),"title":"Mapped","parts":[{"id":format!("{prefix}-1_smt"),"name":"statement","prose":"Private fixture statement."}]},
            {"id":format!("{prefix}-2"),"title":"Unmapped","parts":[{"id":format!("{prefix}-2_smt"),"name":"statement","prose":"Unmapped fixture statement."}]}]}})).unwrap()
    }

    /// Parse the real manifest and offline-validate actual native Catalog originals.
    /// This calls no filesystem port and cannot construct a current approval proof.
    fn fixture() -> (MappingManifest, LoadedResource, LoadedResource) {
        let raw = serde_json::to_vec(&json!({"schema_version":"forge.mapping-manifest/1",
            "collection":{"key":"collection","title":"Native fixture","version":"1.0.0","last_modified":"2026-08-22T17:00:00Z"},
            "reviewers":[{"key":"reviewer-1","type":"person","name":"Synthetic reviewer"}],
            "provenance":{"method":"human","matching_rationale":"semantic","status":"draft","mapping_description":"Fixture only.","reviewer_keys":["reviewer-1"],"reviewed_at":"2026-08-22T17:00:00Z"},
            "mapping":{"key":"mapping","source":{"type":"catalog","artifact":"source.json","href":"source.json"},
                "target":{"type":"catalog","artifact":"target.json","href":"target.json"},"maps":[{
                "key":"map-1","relationship":"subset-of","sources":[{"type":"control","id_ref":"a-1"}],
                "targets":[{"type":"control","id_ref":"b-1"}],"reviewer_key":"reviewer-1",
                "reviewed_at":"2026-08-22T17:00:00Z","rationale":"Synthetic reviewed relationship."}]}})).unwrap();
        let manifest = super::super::manifest::parse(&raw).unwrap();
        let source_raw = catalog("11111111-1111-4111-8111-111111111111", "a");
        let target_raw = catalog("22222222-2222-4222-8222-222222222222", "b");
        let mut noop = |_value: &Value| Ok::<(), std::convert::Infallible>(());
        let source = super::super::inventory::load_captured_admitted(
            Path::new("."),
            "source",
            &manifest.mapping.source,
            &source_raw,
            None,
            &mut noop,
        )
        .unwrap_or_else(|_| panic!("synthetic native Catalog fixture rejected"));
        let target = super::super::inventory::load_captured_admitted(
            Path::new("."),
            "target",
            &manifest.mapping.target,
            &target_raw,
            None,
            &mut noop,
        )
        .unwrap_or_else(|_| panic!("synthetic native Catalog fixture rejected"));
        (manifest, source, target)
    }

    /// Compare every compact and pretty native/report byte; no selected-field oracle.
    fn same_product(left: &BuildProduct, right: &BuildProduct) {
        assert_eq!(
            serde_json::to_vec(&left.artifact).unwrap(),
            serde_json::to_vec(&right.artifact).unwrap()
        );
        assert_eq!(
            serde_json::to_vec_pretty(&left.artifact).unwrap(),
            serde_json::to_vec_pretty(&right.artifact).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&left.report).unwrap(),
            serde_json::to_vec(&right.report).unwrap()
        );
        assert_eq!(
            serde_json::to_vec_pretty(&left.report).unwrap(),
            serde_json::to_vec_pretty(&right.report).unwrap()
        );
    }

    /// Complete admitted model equals the real unlimited native output and schema.
    #[test]
    fn complete_admitted_native_and_report_equal_unlimited() {
        let (manifest, source, target) = fixture();
        let old = build(&manifest, &source, &target, false).unwrap();
        let mut noop = |_charge| Ok::<(), Stop>(());
        let actual = build_admitted(&manifest, &source, &target, false, &mut noop).unwrap();
        same_product(&old, &actual);
        let value = serde_json::to_value(&actual.artifact).unwrap();
        let schema =
            crate::validate::validate_artifact(&value, crate::OscalModelType::Mapping).unwrap();
        assert!(schema.is_valid, "synthetic generated native schema must be valid");
        assert_eq!(actual.report.source_controls.eligible, 2);
        assert_eq!(actual.report.source_controls.referenced, 1);
        assert_eq!(actual.report.source_controls.unmapped_ids, vec!["a-2"]);
        assert!(!actual.report.validation.mapping_schema_valid);
        assert!(actual.report.status.is_none());
    }

    /// True excerpt mode preserves all actual side/type/ID ordering and notes.
    #[test]
    fn actual_native_excerpts_are_complete_and_default_bytes_preserved() {
        let (manifest, source, target) = fixture();
        let old = build(&manifest, &source, &target, true).unwrap();
        let mut noop = |_charge| Ok::<(), Stop>(());
        let actual = build_admitted(&manifest, &source, &target, true, &mut noop).unwrap();
        same_product(&old, &actual);
        assert!(
            actual.report.excerpts.iter().any(|row| row.side == "source" && row.id == "a-2_smt")
        );
        assert!(actual.report.excerpts.iter().any(|row| row.side == "target" && row.id == "b-1"));
    }

    /// Tiny, subnormal and negative-zero scalar spellings use actual native serialization.
    #[test]
    fn numeric_extremes_preserve_complete_native_value_and_byte_outputs() {
        let (mut manifest, source, target) = fixture();
        for value in [1e-100, f64::from_bits(1), -0.0, 0.0, 1.0] {
            manifest.mapping.maps[0].confidence_score =
                Some(ConfidenceScoreManifest { category: None, percentage: Some(value) });
            let old = build(&manifest, &source, &target, false).unwrap();
            let mut noop = |_charge| Ok::<(), Stop>(());
            let actual = build_admitted(&manifest, &source, &target, false, &mut noop).unwrap();
            same_product(&old, &actual);
        }
    }

    /// Repeated native references are legal across maps and participation remains once per subject.
    #[test]
    fn repeated_subjects_do_not_become_duplicate_participation_or_flattened_rejection() {
        let (mut manifest, source, target) = fixture();
        let mut second = manifest.mapping.maps[0].clone();
        second.key = "map-2".into();
        second.relationship = Relationship::EqualTo;
        manifest.mapping.maps.push(second);
        let old = build(&manifest, &source, &target, false).unwrap();
        let mut noop = |_charge| Ok::<(), Stop>(());
        let actual = build_admitted(&manifest, &source, &target, false, &mut noop).unwrap();
        same_product(&old, &actual);
        assert_eq!(actual.report.source_controls.referenced, 1);
        assert_eq!(actual.artifact.mapping_collection.mappings[0].maps.len(), 2);
    }

    /// Admission failure at the whole expansion reservation cannot return a partial model.
    #[test]
    fn complete_preflight_reservation_refusal_is_exact_admission_error() {
        let (manifest, source, target) = fixture();
        let mut reservations = 0;
        let mut stop = |charge| match charge {
            BuildCharge::Reserve { .. } => {
                reservations += 1;
                Err(Stop::Capacity)
            }
            _ => Ok(()),
        };
        let result = build_admitted(&manifest, &source, &target, false, &mut stop);
        assert!(matches!(result, Err(CapturedBuildError::Admission(Stop::Capacity))));
        assert_eq!(reservations, 1);
    }

    /// Actual post-reservation UUID comparison cancellation is not a Domain or success.
    #[test]
    fn reached_native_registry_comparator_preserves_exact_callback_stop() {
        let (manifest, source, target) = fixture();
        let mut reserved = false;
        let mut stop = |charge| {
            match charge {
                BuildCharge::Reserve { .. } => reserved = true,
                BuildCharge::Work { byte_work: 32, .. } if reserved => return Err(Stop::Cancel),
                _ => {}
            }
            Ok(())
        };
        let result = build_admitted(&manifest, &source, &target, false, &mut stop);
        assert!(matches!(result, Err(CapturedBuildError::Admission(Stop::Cancel))));
        assert!(reserved);
    }

    /// Invalid actual native reference retains the same public error and no model success.
    #[test]
    fn admitted_invalid_reference_preserves_maintained_native_diagnostic() {
        let (mut manifest, source, target) = fixture();
        manifest.mapping.maps[0].sources[0].id_ref = "missing-native-control".into();
        let original = build(&manifest, &source, &target, false).err().unwrap();
        let mut noop = |_charge| Ok::<(), Stop>(());
        let actual = build_admitted(&manifest, &source, &target, false, &mut noop).err().unwrap();
        match actual {
            CapturedBuildError::Domain(error) => {
                assert_eq!(original.to_string(), error.to_string());
            }
            other @ CapturedBuildError::Admission(_) => panic!("wrong failure class {other:?}"),
        }
    }

    /// Borrowed long escaped rationale raises the reservation before any derived native model.
    #[test]
    fn escaped_private_text_growth_is_precharged_and_complete_output_matches() {
        let (mut manifest, source, target) = fixture();
        let mut baseline = 0;
        let mut count = |charge| {
            if let BuildCharge::Reserve { logical_bytes } = charge {
                baseline = logical_bytes;
            }
            Ok::<(), Stop>(())
        };
        build_admitted(&manifest, &source, &target, false, &mut count).unwrap();
        manifest.mapping.maps[0].rationale = "Private\\\"\n\tλ\u{202e}".repeat(32);
        let mut expanded = 0;
        let mut count = |charge| {
            if let BuildCharge::Reserve { logical_bytes } = charge {
                expanded = logical_bytes;
            }
            Ok::<(), Stop>(())
        };
        let actual = build_admitted(&manifest, &source, &target, false, &mut count).unwrap();
        assert!(expanded > baseline);
        same_product(&build(&manifest, &source, &target, false).unwrap(), &actual);
    }

    /// Controlled native sorting returns immediately on the exact reached comparison refusal.
    #[test]
    fn sort_failure_cannot_be_translated_into_successful_native_order() {
        let mut values = vec![7_u64, 1, 5, 3];
        let mut comparisons = 0;
        let mut stop = |charge| {
            if let BuildCharge::Work { visits: 1, .. } = charge {
                comparisons += 1;
                if comparisons == 2 {
                    return Err(Stop::Cancel);
                }
            }
            Ok(())
        };
        let mut context = BuildContext { admit: &mut stop, controlled: true };
        let result = sort_admitted(&mut values, &mut context, |_left, _right| Some(16), Ord::cmp);
        assert!(matches!(result, Err(CapturedBuildError::Admission(Stop::Cancel))));
        assert_eq!(comparisons, 2);
    }

    /// Registry capacity is admitted exactly, and plus-one stops before insertion/string cloning.
    #[test]
    fn controlled_reference_capacity_never_renews_on_implicit_vec_growth() {
        let mut refs =
            NativeReferences { legacy: BTreeSet::new(), controlled: Vec::with_capacity(1) };
        let mut count = |charge| match charge {
            BuildCharge::Work { visits: usize::MAX, .. } => Err(Stop::Capacity),
            _ => Ok(()),
        };
        let mut context = BuildContext { admit: &mut count, controlled: true };
        refs.insert(SubjectType::Control, "a", &mut context).unwrap();
        assert!(matches!(
            refs.insert(SubjectType::Control, "b", &mut context),
            Err(CapturedBuildError::Admission(Stop::Capacity))
        ));
        assert_eq!(refs.controlled.len(), 1);
        assert_eq!(refs.controlled[0].1, "a");
    }
}
