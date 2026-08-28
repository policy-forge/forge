//! Typed OSCAL Assessment Results construction.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::context::LoadedContext;
use super::manifest::{
    AssessmentResultsManifest, ConclusionType, FindingManifest, ObservationManifest, PartyType,
    ProvenanceManifest, RiskManifest,
};
use crate::ForgeError;

/// FORGE extension namespace for stable reviewer keys and provenance hashes.
pub const FORGE_ASSESSMENT_RESULTS_NS: &str =
    "https://policy-forge.github.io/ns/assessment-results";
/// Versioned deterministic identifier seed contract.
pub const UUID_SEED_VERSION: &str = "forge.assessment-results/1";

#[derive(Debug, Clone, Serialize)]
pub struct AssessmentResultsEnvelope {
    #[serde(rename = "assessment-results")]
    pub assessment_results: AssessmentResults,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssessmentResults {
    pub uuid: String,
    pub metadata: AssessmentResultsMetadata,
    #[serde(rename = "import-ap")]
    pub import_ap: ImportAssessmentPlan,
    pub results: Vec<AssessmentResult>,
    #[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<BackMatter>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssessmentResultsMetadata {
    pub title: String,
    #[serde(rename = "last-modified")]
    pub last_modified: String,
    pub version: String,
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
    pub props: Vec<OscalProperty>,
    pub roles: Vec<OscalRole>,
    pub parties: Vec<OscalParty>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OscalRole {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OscalParty {
    pub uuid: String,
    #[serde(rename = "type")]
    pub party_type: &'static str,
    pub name: String,
    pub props: Vec<OscalProperty>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportAssessmentPlan {
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssessmentResult {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    pub props: Vec<OscalProperty>,
    #[serde(rename = "reviewed-controls")]
    pub reviewed_controls: ReviewedControls,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<Observation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<Risk>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewedControls {
    #[serde(rename = "control-selections")]
    pub control_selections: Vec<ControlSelection>,
    #[serde(rename = "control-objective-selections", skip_serializing_if = "Vec::is_empty")]
    pub control_objective_selections: Vec<ObjectiveSelection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlSelection {
    #[serde(rename = "include-controls")]
    pub include_controls: Vec<ControlIdReference>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlIdReference {
    #[serde(rename = "control-id")]
    pub control_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveSelection {
    #[serde(rename = "include-objectives")]
    pub include_objectives: Vec<ObjectiveIdReference>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObjectiveIdReference {
    #[serde(rename = "objective-id")]
    pub objective_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Observation {
    pub uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    pub props: Vec<OscalProperty>,
    pub methods: Vec<&'static str>,
    pub origins: Vec<Origin>,
    pub subjects: Vec<SubjectReference>,
    #[serde(rename = "relevant-evidence", skip_serializing_if = "Vec::is_empty")]
    pub relevant_evidence: Vec<RelevantEvidence>,
    pub collected: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub props: Vec<OscalProperty>,
    pub origins: Vec<Origin>,
    pub target: FindingTarget,
    #[serde(rename = "implementation-statement-uuid", skip_serializing_if = "Option::is_none")]
    pub implementation_statement_uuid: Option<String>,
    #[serde(rename = "related-observations")]
    pub related_observations: Vec<RelatedObservation>,
    #[serde(rename = "related-risks", skip_serializing_if = "Vec::is_empty")]
    pub related_risks: Vec<AssociatedRisk>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingTarget {
    #[serde(rename = "type")]
    pub target_type: &'static str,
    #[serde(rename = "target-id")]
    pub target_id: String,
    pub status: FindingTargetStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct FindingTargetStatus {
    pub state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Risk {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub statement: String,
    pub props: Vec<OscalProperty>,
    pub status: &'static str,
    pub origins: Vec<Origin>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Origin {
    pub actors: Vec<OriginActor>,
    #[serde(rename = "related-tasks", skip_serializing_if = "Vec::is_empty")]
    pub related_tasks: Vec<RelatedTask>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginActor {
    #[serde(rename = "type")]
    pub actor_type: &'static str,
    #[serde(rename = "actor-uuid")]
    pub actor_uuid: String,
    #[serde(rename = "role-id")]
    pub role_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedTask {
    #[serde(rename = "task-uuid")]
    pub task_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubjectReference {
    #[serde(rename = "subject-uuid")]
    pub subject_uuid: String,
    #[serde(rename = "type")]
    pub subject_type: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelevantEvidence {
    pub href: String,
    pub description: &'static str,
    pub props: Vec<OscalProperty>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedObservation {
    #[serde(rename = "observation-uuid")]
    pub observation_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssociatedRisk {
    #[serde(rename = "risk-uuid")]
    pub risk_uuid: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OscalProperty {
    pub name: &'static str,
    pub ns: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackMatter {
    pub resources: Vec<BackMatterResource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackMatterResource {
    pub uuid: String,
    pub title: String,
    pub props: Vec<OscalProperty>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rlinks: Vec<ResourceLink>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceLink {
    pub href: String,
    pub hashes: Vec<OscalHash>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OscalHash {
    pub algorithm: &'static str,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct BuiltAssessmentResults {
    pub artifact: AssessmentResultsEnvelope,
    pub object_snapshots: BTreeMap<(ConclusionType, String), ObjectSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSnapshot {
    pub uuid: String,
    pub content_sha256: String,
    pub rationale_sha256: String,
    pub status: Option<String>,
}

/// Validate all manifest references and construct a typed OSCAL artifact.
///
/// # Errors
///
/// Returns an error when a reviewed reference is absent from the exact context
/// or when deterministic content fingerprinting cannot be serialized.
#[allow(
    clippy::too_many_lines,
    reason = "typed deterministic assembly is kept together so ordering and trust-boundary fields remain auditable"
)]
pub fn build(
    manifest: &AssessmentResultsManifest,
    context: &LoadedContext,
) -> Result<BuiltAssessmentResults, ForgeError> {
    validate_references(manifest, context)?;

    let document_key = &manifest.document.key;
    let party_uuids: BTreeMap<_, _> = manifest
        .parties
        .iter()
        .map(|party| {
            (party.key.clone(), stable_uuid(document_key, "party", &party.key).to_string())
        })
        .collect();

    let mut roles = manifest.roles.clone();
    roles.sort_by(|left, right| left.id.cmp(&right.id));
    let roles =
        roles.into_iter().map(|role| OscalRole { id: role.id, title: role.title }).collect();
    let mut parties = manifest.parties.clone();
    parties.sort_by(|left, right| left.key.cmp(&right.key));
    let parties = parties
        .into_iter()
        .map(|party| OscalParty {
            uuid: party_uuids[&party.key].clone(),
            party_type: match party.party_type {
                PartyType::Person => "person",
                PartyType::Organization => "organization",
            },
            name: party.name,
            props: vec![prop("stable-key", party.key)],
        })
        .collect();

    let mut observations = manifest.result.observations.clone();
    observations.sort_by(|left, right| left.key.cmp(&right.key));
    let mut findings = manifest.result.findings.clone();
    findings.sort_by(|left, right| left.key.cmp(&right.key));
    let mut risks = manifest.result.risks.clone();
    risks.sort_by(|left, right| left.key.cmp(&right.key));
    let observation_uuids: BTreeMap<_, _> = observations
        .iter()
        .map(|value| {
            (value.key.clone(), stable_uuid(document_key, "observation", &value.key).to_string())
        })
        .collect();
    let risk_uuids: BTreeMap<_, _> = risks
        .iter()
        .map(|value| (value.key.clone(), stable_uuid(document_key, "risk", &value.key).to_string()))
        .collect();

    let mut object_snapshots = BTreeMap::new();
    let built_observations = observations
        .iter()
        .map(|observation| {
            build_observation(
                document_key,
                observation,
                context,
                &party_uuids,
                &observation_uuids,
                &mut object_snapshots,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let relationships = relationship_indexes(manifest);
    let built_findings = findings
        .iter()
        .map(|finding| {
            build_finding(
                document_key,
                finding,
                &party_uuids,
                &observation_uuids,
                &risk_uuids,
                &relationships,
                &mut object_snapshots,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let built_risks = risks
        .iter()
        .map(|risk| build_risk(risk, &party_uuids, &risk_uuids, &mut object_snapshots))
        .collect::<Result<Vec<_>, _>>()?;

    let mut control_ids = manifest.result.control_ids.clone();
    control_ids.sort();
    let mut objective_ids = manifest.result.objective_ids.clone();
    objective_ids.sort();

    let mut metadata_props = vec![prop(
        "trust-boundary",
        "Reviewer-authored assertions only; FORGE validates structure and exact references but does not authenticate identity or evaluate evidence.",
    )];
    for identity in context.artifact_identities() {
        metadata_props
            .push(prop("context-sha256", format!("{}:{}", identity.kind, identity.sha256)));
    }
    if let Some(hash) = &context.evidence_index_sha256 {
        metadata_props.push(prop("evidence-index-sha256", hash.clone()));
    }

    let artifact = AssessmentResultsEnvelope {
        assessment_results: AssessmentResults {
            uuid: stable_uuid(document_key, "document", document_key).to_string(),
            metadata: AssessmentResultsMetadata {
                title: manifest.document.title.clone(),
                last_modified: manifest.document.last_modified.clone(),
                version: manifest.document.version.clone(),
                oscal_version: crate::oscal::OSCAL_VERSION.to_string(),
                props: metadata_props,
                roles,
                parties,
            },
            import_ap: ImportAssessmentPlan { href: context.assessment_plan.href.clone() },
            results: vec![AssessmentResult {
                uuid: stable_uuid(document_key, "result", &manifest.result.key).to_string(),
                title: manifest.result.title.clone(),
                description: manifest.result.description.clone(),
                start: manifest.result.start.clone(),
                end: manifest.result.end.clone(),
                props: vec![prop("stable-key", manifest.result.key.clone())],
                reviewed_controls: ReviewedControls {
                    control_selections: vec![ControlSelection {
                        include_controls: control_ids
                            .into_iter()
                            .map(|control_id| ControlIdReference { control_id })
                            .collect(),
                    }],
                    control_objective_selections: if objective_ids.is_empty() {
                        Vec::new()
                    } else {
                        vec![ObjectiveSelection {
                            include_objectives: objective_ids
                                .into_iter()
                                .map(|objective_id| ObjectiveIdReference { objective_id })
                                .collect(),
                        }]
                    },
                },
                observations: built_observations,
                risks: built_risks,
                findings: built_findings,
            }],
            back_matter: Some(build_back_matter(document_key, context, manifest)),
        },
    };
    Ok(BuiltAssessmentResults { artifact, object_snapshots })
}

fn validate_references(
    manifest: &AssessmentResultsManifest,
    context: &LoadedContext,
) -> Result<(), ForgeError> {
    for control_id in &manifest.result.control_ids {
        if !context.controls.contains(control_id) || !context.reviewed_controls.contains(control_id)
        {
            return Err(error(format!(
                "control '{}' is not present in both the exact Catalog and Assessment Plan scope",
                bounded(control_id)
            )));
        }
    }
    for objective_id in &manifest.result.objective_ids {
        if !context.objectives.contains(objective_id)
            || !context.reviewed_objectives.contains(objective_id)
        {
            return Err(error(format!(
                "objective '{}' is not present in both the exact Catalog and Assessment Plan scope",
                bounded(objective_id)
            )));
        }
    }
    for observation in &manifest.result.observations {
        for subject in &observation.subjects {
            if !context.subject_is_in_scope(subject.subject_type, &subject.uuid) {
                return Err(error(format!(
                    "observation '{}' references out-of-scope {} subject '{}'",
                    bounded(&observation.key),
                    subject.subject_type.as_str(),
                    bounded(&subject.uuid)
                )));
            }
        }
        for task_uuid in &observation.task_uuids {
            if !context.tasks.contains(task_uuid) {
                return Err(error(format!(
                    "observation '{}' references unknown Assessment Plan task '{}'",
                    bounded(&observation.key),
                    bounded(task_uuid)
                )));
            }
        }
        for evidence_key in &observation.evidence_keys {
            if !context.evidence.contains_key(evidence_key) {
                return Err(error(format!(
                    "observation '{}' references evidence key '{}' absent from the exact PRD 060 index",
                    bounded(&observation.key),
                    bounded(evidence_key)
                )));
            }
        }
    }
    for finding in &manifest.result.findings {
        let target_control = match finding.target.target_type {
            super::manifest::FindingTargetType::StatementId => context
                .statement_controls
                .get(&finding.target.id)
                .filter(|control_id| manifest.result.control_ids.contains(control_id)),
            super::manifest::FindingTargetType::ObjectiveId => context
                .objective_controls
                .get(&finding.target.id)
                .filter(|_| manifest.result.objective_ids.contains(&finding.target.id)),
        };
        let Some(target_control) = target_control else {
            return Err(error(format!(
                "finding '{}' references unknown or out-of-scope {} '{}'",
                bounded(&finding.key),
                finding.target.target_type.as_str(),
                bounded(&finding.target.id)
            )));
        };
        if let Some(implementation_uuid) = &finding.implementation_statement_uuid {
            match context.implementation_statement_controls.get(implementation_uuid) {
                Some(implementation_control) if implementation_control == target_control => {}
                Some(implementation_control) => {
                    return Err(error(format!(
                        "finding '{}' target belongs to control '{}' but SSP implementation '{}' belongs to control '{}'",
                        bounded(&finding.key),
                        bounded(target_control),
                        bounded(implementation_uuid),
                        bounded(implementation_control)
                    )));
                }
                None => {
                    return Err(error(format!(
                        "finding '{}' references unknown SSP implementation statement '{}'",
                        bounded(&finding.key),
                        bounded(implementation_uuid)
                    )));
                }
            }
        }
    }
    Ok(())
}

type RelationshipIndexes = (BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<String>>);

fn relationship_indexes(manifest: &AssessmentResultsManifest) -> RelationshipIndexes {
    let mut observations_by_finding: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut risks_by_finding: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for relationship in &manifest.result.relationships {
        match (relationship.from.object_type, relationship.to.object_type) {
            (ConclusionType::Observation, ConclusionType::Finding) => observations_by_finding
                .entry(relationship.to.key.clone())
                .or_default()
                .push(relationship.from.key.clone()),
            (ConclusionType::Finding, ConclusionType::Risk) => risks_by_finding
                .entry(relationship.from.key.clone())
                .or_default()
                .push(relationship.to.key.clone()),
            _ => {}
        }
    }
    for values in observations_by_finding.values_mut().chain(risks_by_finding.values_mut()) {
        values.sort();
    }
    (observations_by_finding, risks_by_finding)
}

fn build_observation(
    document_key: &str,
    manifest: &ObservationManifest,
    context: &LoadedContext,
    party_uuids: &BTreeMap<String, String>,
    observation_uuids: &BTreeMap<String, String>,
    snapshots: &mut BTreeMap<(ConclusionType, String), ObjectSnapshot>,
) -> Result<Observation, ForgeError> {
    let content_sha256 = hash_fields(&(
        &manifest.title,
        &manifest.description,
        &manifest.provenance.assessor_key,
        &manifest.provenance.role_id,
        &manifest.provenance.start,
        &manifest.provenance.end,
        manifest.provenance.method,
        &manifest.subjects,
        &manifest.task_uuids,
        &manifest.evidence_keys,
    ))?;
    let rationale_sha256 = sha256(manifest.provenance.rationale.as_bytes());
    let uuid = observation_uuids[&manifest.key].clone();
    snapshots.insert(
        (ConclusionType::Observation, manifest.key.clone()),
        ObjectSnapshot {
            uuid: uuid.clone(),
            content_sha256: content_sha256.clone(),
            rationale_sha256: rationale_sha256.clone(),
            status: None,
        },
    );
    let mut subjects = manifest.subjects.clone();
    subjects.sort();
    let mut tasks = manifest.task_uuids.clone();
    tasks.sort();
    let mut evidence_keys = manifest.evidence_keys.clone();
    evidence_keys.sort();
    let relevant_evidence = evidence_keys
        .into_iter()
        .map(|key| RelevantEvidence {
            href: format!(
                "#{}",
                stable_uuid(document_key, "evidence-resource", &key)
            ),
            description: "Reviewer-linked evidence identity; no evidence content was copied.",
            props: vec![
                prop("stable-key", key.clone()),
                prop("sha256", context.evidence[&key].clone()),
                prop(
                    "evidence-boundary",
                    "A key and hash record byte identity only; FORGE makes no evidentiary evaluation.",
                ),
            ],
        })
        .collect();
    Ok(Observation {
        uuid,
        title: manifest.title.clone(),
        description: manifest.description.clone(),
        props: conclusion_props(
            &manifest.key,
            &content_sha256,
            &rationale_sha256,
            &manifest.provenance,
        ),
        methods: vec![manifest.provenance.method.as_str()],
        origins: vec![origin(&manifest.provenance, party_uuids, tasks)],
        subjects: subjects
            .into_iter()
            .map(|subject| SubjectReference {
                subject_uuid: subject.uuid,
                subject_type: subject.subject_type.as_str(),
            })
            .collect(),
        relevant_evidence,
        collected: manifest.provenance.start.clone(),
    })
}

#[allow(clippy::too_many_arguments)]
fn build_finding(
    document_key: &str,
    manifest: &FindingManifest,
    party_uuids: &BTreeMap<String, String>,
    observation_uuids: &BTreeMap<String, String>,
    risk_uuids: &BTreeMap<String, String>,
    relationships: &RelationshipIndexes,
    snapshots: &mut BTreeMap<(ConclusionType, String), ObjectSnapshot>,
) -> Result<Finding, ForgeError> {
    let content_sha256 = hash_fields(&(
        &manifest.title,
        &manifest.description,
        &manifest.provenance.assessor_key,
        &manifest.provenance.role_id,
        &manifest.provenance.start,
        &manifest.provenance.end,
        manifest.provenance.method,
        manifest.target.target_type,
        &manifest.target.id,
        &manifest.implementation_statement_uuid,
    ))?;
    let rationale_sha256 = sha256(manifest.provenance.rationale.as_bytes());
    let status = format!(
        "{}:{}",
        manifest.target.state.as_str(),
        manifest.target.reason.map_or("none", super::manifest::FindingReason::as_str)
    );
    let uuid = stable_uuid(document_key, "finding", &manifest.key).to_string();
    snapshots.insert(
        (ConclusionType::Finding, manifest.key.clone()),
        ObjectSnapshot {
            uuid: uuid.clone(),
            content_sha256: content_sha256.clone(),
            rationale_sha256: rationale_sha256.clone(),
            status: Some(status),
        },
    );
    let observations = relationships
        .0
        .get(&manifest.key)
        .into_iter()
        .flatten()
        .map(|key| RelatedObservation { observation_uuid: observation_uuids[key].clone() })
        .collect();
    let risks = relationships
        .1
        .get(&manifest.key)
        .into_iter()
        .flatten()
        .map(|key| AssociatedRisk { risk_uuid: risk_uuids[key].clone() })
        .collect();
    Ok(Finding {
        uuid,
        title: manifest.title.clone(),
        description: manifest.description.clone(),
        props: conclusion_props(
            &manifest.key,
            &content_sha256,
            &rationale_sha256,
            &manifest.provenance,
        ),
        origins: vec![origin(&manifest.provenance, party_uuids, Vec::new())],
        target: FindingTarget {
            target_type: manifest.target.target_type.as_str(),
            target_id: manifest.target.id.clone(),
            status: FindingTargetStatus {
                state: manifest.target.state.as_str(),
                reason: manifest.target.reason.map(super::manifest::FindingReason::as_str),
            },
        },
        implementation_statement_uuid: manifest.implementation_statement_uuid.clone(),
        related_observations: observations,
        related_risks: risks,
    })
}

fn build_risk(
    manifest: &RiskManifest,
    party_uuids: &BTreeMap<String, String>,
    risk_uuids: &BTreeMap<String, String>,
    snapshots: &mut BTreeMap<(ConclusionType, String), ObjectSnapshot>,
) -> Result<Risk, ForgeError> {
    let content_sha256 = hash_fields(&(
        &manifest.title,
        &manifest.description,
        &manifest.statement,
        &manifest.provenance.assessor_key,
        &manifest.provenance.role_id,
        &manifest.provenance.start,
        &manifest.provenance.end,
        manifest.provenance.method,
        &manifest.severity,
        manifest.confidence.map(f64::to_bits),
    ))?;
    let rationale_sha256 = sha256(manifest.provenance.rationale.as_bytes());
    let uuid = risk_uuids[&manifest.key].clone();
    snapshots.insert(
        (ConclusionType::Risk, manifest.key.clone()),
        ObjectSnapshot {
            uuid: uuid.clone(),
            content_sha256: content_sha256.clone(),
            rationale_sha256: rationale_sha256.clone(),
            status: Some(manifest.status.as_str().to_string()),
        },
    );
    let mut props =
        conclusion_props(&manifest.key, &content_sha256, &rationale_sha256, &manifest.provenance);
    if let Some(severity) = &manifest.severity {
        props.push(prop("reviewer-declared-severity", severity.clone()));
    }
    if let Some(confidence) = manifest.confidence {
        props.push(prop("reviewer-declared-confidence", confidence.to_string()));
    }
    Ok(Risk {
        uuid,
        title: manifest.title.clone(),
        description: manifest.description.clone(),
        statement: manifest.statement.clone(),
        props,
        status: manifest.status.as_str(),
        origins: vec![origin(&manifest.provenance, party_uuids, Vec::new())],
    })
}

fn origin(
    provenance: &ProvenanceManifest,
    party_uuids: &BTreeMap<String, String>,
    task_uuids: Vec<String>,
) -> Origin {
    Origin {
        actors: vec![OriginActor {
            actor_type: "party",
            actor_uuid: party_uuids[&provenance.assessor_key].clone(),
            role_id: provenance.role_id.clone(),
        }],
        related_tasks: task_uuids.into_iter().map(|task_uuid| RelatedTask { task_uuid }).collect(),
    }
}

fn conclusion_props(
    key: &str,
    content_sha256: &str,
    rationale_sha256: &str,
    provenance: &ProvenanceManifest,
) -> Vec<OscalProperty> {
    let mut props = vec![
        prop("stable-key", key.to_string()),
        prop("content-sha256", content_sha256.to_string()),
        prop("rationale-sha256", rationale_sha256.to_string()),
        prop("assessor-key", provenance.assessor_key.clone()),
        prop("assessment-start", provenance.start.clone()),
        prop("assessment-method", provenance.method.as_str().to_string()),
        prop("rationale", provenance.rationale.clone()),
    ];
    if let Some(end) = &provenance.end {
        props.push(prop("assessment-end", end.clone()));
    }
    props
}

fn build_back_matter(
    document_key: &str,
    context: &LoadedContext,
    manifest: &AssessmentResultsManifest,
) -> BackMatter {
    let mut resources: Vec<_> = context
        .artifact_identities()
        .into_iter()
        .map(|identity| BackMatterResource {
            uuid: stable_uuid(document_key, "context-resource", identity.kind).to_string(),
            title: format!("{} context identity", identity.kind),
            props: vec![
                prop("context-kind", identity.kind.to_string()),
                prop("root-uuid", identity.root_uuid.clone()),
                prop("document-version", identity.document_version.clone()),
                prop("oscal-version", identity.oscal_version.clone()),
            ],
            rlinks: vec![ResourceLink {
                href: identity.href.clone(),
                hashes: vec![OscalHash { algorithm: "SHA-256", value: identity.sha256.clone() }],
            }],
        })
        .collect();
    if let Some(index_hash) = &context.evidence_index_sha256 {
        resources.push(BackMatterResource {
            uuid: stable_uuid(document_key, "context-resource", "evidence-index").to_string(),
            title: "PRD 060 evidence index identity".to_string(),
            props: vec![
                prop("context-kind", "evidence-index"),
                prop("sha256", index_hash.clone()),
                prop(
                    "evidence-boundary",
                    "The index is used only to resolve stable evidence keys and hashes.",
                ),
            ],
            rlinks: Vec::new(),
        });
    }
    let used_evidence: BTreeSet<_> = manifest
        .result
        .observations
        .iter()
        .flat_map(|observation| observation.evidence_keys.iter().cloned())
        .collect();
    resources.extend(used_evidence.into_iter().map(|key| BackMatterResource {
        uuid: stable_uuid(document_key, "evidence-resource", &key).to_string(),
        title: "Evidence identity".to_string(),
        props: vec![
            prop("stable-key", key.clone()),
            prop("sha256", context.evidence[&key].clone()),
            prop(
                "evidence-boundary",
                "A key and hash record byte identity only; FORGE makes no evidentiary evaluation.",
            ),
        ],
        rlinks: Vec::new(),
    }));
    resources.sort_by(|left, right| left.uuid.cmp(&right.uuid));
    BackMatter { resources }
}

fn stable_uuid(document_key: &str, kind: &str, key: &str) -> Uuid {
    let mut seed = Vec::new();
    for value in [UUID_SEED_VERSION, document_key, kind, key] {
        seed.extend_from_slice(&(value.len() as u64).to_be_bytes());
        seed.extend_from_slice(value.as_bytes());
    }
    Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &seed)
}

fn prop(name: &'static str, value: impl Into<String>) -> OscalProperty {
    OscalProperty { name, ns: FORGE_ASSESSMENT_RESULTS_NS, value: value.into() }
}

fn hash_fields(value: &impl Serialize) -> Result<String, ForgeError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|cause| error(format!("content fingerprint serialization failed: {cause}")))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn bounded(value: &str) -> String {
    crate::json_strict::bounded(value)
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::AssessmentResultsBuild(message.into())
}
