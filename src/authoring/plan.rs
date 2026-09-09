//! Pure authoring planning, exact gap accounting, and dependent context blocking.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::manifest::{self, Answer, ControlAssignment, FamilyAssignment, HumanClause, Topic};
use super::model::{
    AnswerStatus, AuthoringPlan, DraftState, GapAssignment, GapCounts, GapDisposition, GapPlan,
    LoadedAuthorProject, PLAN_SCHEMA_VERSION, PlanProvenance, PolicyPlan, QuestionEvaluation,
    SectionPlan,
};
use crate::ForgeError;
use crate::applicability::model::{ApplicabilityReport, GapClassification};

const MAX_EXPANDED_REFERENCES: usize = 100_000;

/// Build a sorted, reconciled plan from captured, validated authoring inputs.
///
/// No answer value is rendered or interpolated. The loader owns byte-pin and
/// baseline revalidation; this engine owns the accounting and dependency states.
/// Cross-contract validation is intentional here too: this public function also
/// accepts directly constructed in-memory inputs, outside the loader workflow.
///
/// # Errors
///
/// Returns an authoring error for inconsistent baselines, assignments,
/// references, evaluations, or excessive evidence expansion.
pub fn build_plan(loaded: &LoadedAuthorProject) -> Result<AuthoringPlan, ForgeError> {
    if loaded.report_sha256 != loaded.project.baseline.report_sha256
        || loaded.pack_sha256 != loaded.project.authoring_pack.expected_sha256
    {
        return Err(super::error("captured authoring fingerprints do not match project pins"));
    }
    validate_baseline_counts(&loaded.baseline_report)?;
    manifest::validate_relationships(&loaded.pack, &loaded.project, &loaded.baseline_report)?;
    let mut budget = ExpansionBudget::default();
    let questions = evaluate_questions(loaded)?;
    let gaps = build_gaps(loaded, &mut budget)?;
    let counts = count_gaps(&gaps)?;
    let expected = loaded
        .baseline_report
        .counts
        .applicable_unmapped
        .checked_add(loaded.baseline_report.counts.applicable_reviewed_no_relationship)
        .ok_or_else(|| super::error("applicable gap count overflow"))?;
    if counts.total != expected {
        return Err(super::error("applicable gap totals do not match the complete baseline"));
    }
    let policies = build_policies(loaded, &gaps, &questions, &mut budget)?;
    let referenced: BTreeSet<_> = policies
        .iter()
        .flat_map(|policy| &policy.sections)
        .flat_map(|section| &section.questions)
        .map(|question| question.question_key.as_str())
        .collect();
    // The complete question inventory remains visible, but unused pack
    // questions are not represented as required work for this project.
    let unresolved_questions = questions
        .iter()
        .filter(|question| {
            question.state != AnswerStatus::Available
                && referenced.contains(question.question_key.as_str())
        })
        .cloned()
        .collect();
    let unresolved_gaps =
        gaps.iter().filter(|gap| gap.disposition == GapDisposition::Unresolved).cloned().collect();
    Ok(AuthoringPlan {
        schema_version: PLAN_SCHEMA_VERSION.to_string(),
        project_key: loaded.project.project_key.clone(),
        as_of: loaded.project.as_of.clone(),
        provenance: build_provenance(loaded),
        counts,
        gaps,
        policies,
        questions,
        unresolved_questions,
        unresolved_gaps,
    })
}

fn evaluate_questions(loaded: &LoadedAuthorProject) -> Result<Vec<QuestionEvaluation>, ForgeError> {
    let answers: BTreeMap<_, _> = loaded
        .project
        .answers
        .iter()
        .map(|answer| (answer.question_key.as_str(), answer))
        .collect();
    let mut questions = loaded
        .pack
        .questions
        .iter()
        .map(|question| {
            manifest::evaluate_question(
                question,
                answers.get(question.key.as_str()).copied(),
                &loaded.pack_sha256,
                &loaded.project.as_of,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    questions.sort_unstable_by(|left, right| left.question_key.cmp(&right.question_key));
    Ok(questions)
}

fn build_gaps(
    loaded: &LoadedAuthorProject,
    budget: &mut ExpansionBudget,
) -> Result<Vec<GapPlan>, ForgeError> {
    let policies: BTreeMap<_, _> = loaded
        .project
        .policies
        .iter()
        .map(|policy| (policy.policy_family_key.as_str(), policy))
        .collect();
    let mut controls: BTreeMap<&str, Vec<&ControlAssignment>> = BTreeMap::new();
    for assignment in &loaded.pack.control_assignments {
        controls.entry(&assignment.control_id).or_default().push(assignment);
    }
    let mut families: BTreeMap<&str, Vec<&FamilyAssignment>> = BTreeMap::new();
    for assignment in &loaded.pack.family_assignments {
        families.entry(&assignment.topic_key).or_default().push(assignment);
    }
    let deferrals: BTreeMap<_, _> = loaded
        .project
        .deferrals
        .iter()
        .map(|deferral| (deferral.gap_id.as_str(), deferral))
        .collect();
    let mut gaps = Vec::new();
    for control in &loaded.baseline_report.controls {
        if !is_applicable_gap(control.classification) {
            continue;
        }
        let gap_id = manifest::gap_id(&loaded.report_sha256, &control.control_id);
        let mut assignments = Vec::new();
        for control_assignment in controls.get(control.control_id.as_str()).into_iter().flatten() {
            for family_assignment in
                families.get(control_assignment.topic_key.as_str()).into_iter().flatten()
            {
                if let Some(policy) = policies.get(family_assignment.policy_family_key.as_str()) {
                    let assignment = GapAssignment {
                        policy_key: policy.key.clone(),
                        topic_key: control_assignment.topic_key.clone(),
                        control_assignment: (*control_assignment).clone(),
                        family_assignment: (*family_assignment).clone(),
                    };
                    budget.record(&assignment)?;
                    assignments.push(assignment);
                }
            }
        }
        sort_assignments(&mut assignments);
        let deferral = deferrals.get(gap_id.as_str()).copied().cloned();
        let disposition = match (assignments.is_empty(), deferral.is_some()) {
            (false, true) => {
                return Err(super::error("a gap cannot be both assigned and deferred"));
            }
            (false, false) => GapDisposition::Assigned,
            (true, true) => GapDisposition::Deferred,
            (true, false) => GapDisposition::Unresolved,
        };
        gaps.push(GapPlan {
            gap_id,
            control_id: control.control_id.clone(),
            classification: control.classification.as_str().to_string(),
            disposition,
            assignments,
            deferral,
        });
    }
    gaps.sort_unstable_by(|left, right| left.gap_id.cmp(&right.gap_id));
    Ok(gaps)
}

#[derive(Default)]
struct SectionEvidence {
    gap_ids: BTreeSet<String>,
    control_ids: BTreeSet<String>,
    assignments: Vec<GapAssignment>,
}

fn build_policies(
    loaded: &LoadedAuthorProject,
    gaps: &[GapPlan],
    questions: &[QuestionEvaluation],
    budget: &mut ExpansionBudget,
) -> Result<Vec<PolicyPlan>, ForgeError> {
    let mut evidence: BTreeMap<(&str, &str), SectionEvidence> = BTreeMap::new();
    for gap in gaps {
        for assignment in &gap.assignments {
            budget.record(assignment)?;
            let section =
                evidence.entry((&assignment.policy_key, &assignment.topic_key)).or_default();
            section.gap_ids.insert(gap.gap_id.clone());
            section.control_ids.insert(gap.control_id.clone());
            section.assignments.push(assignment.clone());
        }
    }
    let topics: BTreeMap<_, _> =
        loaded.pack.topics.iter().map(|topic| (topic.key.as_str(), topic)).collect();
    let mut families: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for assignment in &loaded.pack.family_assignments {
        families.entry(&assignment.policy_family_key).or_default().push(&assignment.topic_key);
    }
    let evaluations: BTreeMap<_, _> =
        questions.iter().map(|question| (question.question_key.as_str(), question)).collect();
    let answers: BTreeMap<_, _> =
        loaded.project.answers.iter().map(|answer| (answer.key.as_str(), answer)).collect();
    let mut clauses: BTreeMap<(&str, &str), Vec<&HumanClause>> = BTreeMap::new();
    for clause in &loaded.project.human_clauses {
        clauses.entry((&clause.policy_key, &clause.topic_key)).or_default().push(clause);
    }
    let mut policies = Vec::new();
    for policy in &loaded.project.policies {
        let mut sections = Vec::new();
        for topic_key in families.get(policy.policy_family_key.as_str()).into_iter().flatten() {
            let topic =
                topics.get(topic_key).ok_or_else(|| super::error("unknown policy topic"))?;
            let section_evidence =
                evidence.remove(&(policy.key.as_str(), topic_key)).unwrap_or_default();
            let section = build_section(
                topic,
                clauses.get(&(policy.key.as_str(), topic_key)).map_or(&[], Vec::as_slice),
                section_evidence,
                &evaluations,
                &answers,
                budget,
            )?;
            sections.push(section);
        }
        sections.sort_unstable_by(|left, right| {
            (left.order, &left.topic_key).cmp(&(right.order, &right.topic_key))
        });
        policies.push(PolicyPlan {
            policy_key: policy.key.clone(),
            policy_family_key: policy.policy_family_key.clone(),
            title: policy.title.clone(),
            state: policy_state(&sections),
            sections,
        });
    }
    policies.sort_unstable_by(|left, right| left.policy_key.cmp(&right.policy_key));
    Ok(policies)
}

fn build_section(
    topic: &Topic,
    clauses: &[&HumanClause],
    evidence: SectionEvidence,
    evaluations: &BTreeMap<&str, &QuestionEvaluation>,
    answers: &BTreeMap<&str, &Answer>,
    budget: &mut ExpansionBudget,
) -> Result<SectionPlan, ForgeError> {
    let mut question_keys: BTreeSet<&str> =
        topic.question_keys.iter().map(String::as_str).collect();
    let mut clause_keys = Vec::new();
    let mut clause_dependency_blocked = false;
    for clause in clauses {
        // Retain declared keys even when blocked, so the plan preserves the
        // author's intent. Rendering must not include their bytes while blocked.
        clause_keys.push(clause.key.clone());
        for reference in &clause.answer_refs {
            let answer = answers
                .get(reference.answer_key.as_str())
                .ok_or_else(|| super::error("human clause references an unknown answer"))?;
            question_keys.insert(answer.question_key.as_str());
            let evaluation = evaluations.get(answer.question_key.as_str()).ok_or_else(|| {
                super::error("human clause answer references an unknown question")
            })?;
            clause_dependency_blocked |= evaluation.state != AnswerStatus::Available;
        }
    }
    clause_keys.sort_unstable();
    let questions = question_keys
        .into_iter()
        .map(|key| {
            let evaluation = evaluations
                .get(key)
                .ok_or_else(|| super::error("topic references an unknown question"))?;
            budget.record(*evaluation)?;
            Ok((*evaluation).clone())
        })
        .collect::<Result<Vec<_>, ForgeError>>()?;
    let required_blocked = questions
        .iter()
        .any(|question| question.required && question.state != AnswerStatus::Available);
    let state = if required_blocked || clause_dependency_blocked {
        DraftState::BlockedContext
    } else if clause_keys.is_empty() {
        DraftState::SkeletonReady
    } else {
        DraftState::HumanDraftPresent
    };
    let mut assignments = evidence.assignments;
    sort_assignments(&mut assignments);
    Ok(SectionPlan {
        topic_key: topic.key.clone(),
        title: topic.title.clone(),
        order: topic.order,
        state,
        gap_ids: evidence.gap_ids.into_iter().collect(),
        control_ids: evidence.control_ids.into_iter().collect(),
        assignments,
        questions,
        clause_keys,
    })
}

fn policy_state(sections: &[SectionPlan]) -> DraftState {
    if sections.is_empty() {
        DraftState::Planned
    } else if sections.iter().any(|section| section.state == DraftState::BlockedContext) {
        DraftState::BlockedContext
    } else if sections.iter().any(|section| section.state == DraftState::HumanDraftPresent) {
        DraftState::HumanDraftPresent
    } else {
        DraftState::SkeletonReady
    }
}

fn build_provenance(loaded: &LoadedAuthorProject) -> PlanProvenance {
    let mut inputs = loaded.inputs.clone();
    inputs.sort_unstable_by(|left, right| {
        (&left.role, &left.path, &left.sha256, left.byte_length).cmp(&(
            &right.role,
            &right.path,
            &right.sha256,
            right.byte_length,
        ))
    });
    let mut pack_reviewers = loaded.pack.reviewers.clone();
    pack_reviewers.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    let mut project_reviewers = loaded.project.reviewers.clone();
    project_reviewers.sort_unstable_by(|left, right| left.key.cmp(&right.key));
    PlanProvenance {
        project_sha256: loaded.project_sha256.clone(),
        pack_sha256: loaded.pack_sha256.clone(),
        report_sha256: loaded.report_sha256.clone(),
        framework: loaded.baseline_report.framework.clone(),
        inputs,
        baseline_review: loaded.project.baseline_review.clone(),
        pack_reviewers,
        project_reviewers,
    }
}

fn sort_assignments(assignments: &mut [GapAssignment]) {
    assignments.sort_unstable_by(|left, right| {
        (
            &left.policy_key,
            &left.topic_key,
            &left.control_assignment.key,
            &left.family_assignment.key,
        )
            .cmp(&(
                &right.policy_key,
                &right.topic_key,
                &right.control_assignment.key,
                &right.family_assignment.key,
            ))
    });
}

fn count_gaps(gaps: &[GapPlan]) -> Result<GapCounts, ForgeError> {
    let mut counts = GapCounts { total: gaps.len(), ..GapCounts::default() };
    let mut ids = BTreeSet::new();
    for gap in gaps {
        if !ids.insert(&gap.gap_id) {
            return Err(super::error("duplicate applicable gap identity"));
        }
        match gap.disposition {
            GapDisposition::Assigned if !gap.assignments.is_empty() && gap.deferral.is_none() => {
                counts.assigned += 1;
            }
            GapDisposition::Deferred if gap.assignments.is_empty() && gap.deferral.is_some() => {
                counts.deferred += 1;
            }
            GapDisposition::Unresolved if gap.assignments.is_empty() && gap.deferral.is_none() => {
                counts.unresolved += 1;
            }
            _ => {
                return Err(super::error("gap disposition does not match its assignment evidence"));
            }
        }
    }
    if counts
        .assigned
        .checked_add(counts.deferred)
        .and_then(|sum| sum.checked_add(counts.unresolved))
        != Some(counts.total)
    {
        return Err(super::error("authoring gap totals do not reconcile"));
    }
    Ok(counts)
}

fn is_applicable_gap(classification: GapClassification) -> bool {
    matches!(
        classification,
        GapClassification::ApplicableUnmapped | GapClassification::ApplicableReviewedNoRelationship
    )
}

fn validate_baseline_counts(report: &ApplicabilityReport) -> Result<(), ForgeError> {
    let filters = &report.filters;
    if filters.group.is_some()
        || filters.control_prefix.is_some()
        || filters.state.is_some()
        || filters.reviewer.is_some()
        || filters.policy_source.is_some()
        || report.matched_controls != report.counts.total
        || report.controls.len() != report.counts.total
    {
        return Err(super::error("authoring requires a complete, unfiltered applicability report"));
    }
    let mut ids = BTreeSet::new();
    let mut actual = [0usize; 6];
    for control in &report.controls {
        if !ids.insert(&control.control_id) {
            return Err(super::error("duplicate control in applicability report"));
        }
        let index = match control.classification {
            GapClassification::ApplicableMapped => 0,
            GapClassification::ApplicableReviewedNoRelationship => 1,
            GapClassification::ApplicableUnmapped => 2,
            GapClassification::NotApplicable => 3,
            GapClassification::Deferred => 4,
            GapClassification::UnderReview => 5,
        };
        actual[index] += 1;
    }
    let counts = &report.counts;
    if actual
        != [
            counts.applicable_mapped,
            counts.applicable_reviewed_no_relationship,
            counts.applicable_unmapped,
            counts.not_applicable,
            counts.deferred,
            counts.under_review,
        ]
    {
        return Err(super::error("applicability classification totals do not reconcile"));
    }
    Ok(())
}

#[derive(Default)]
struct ExpansionBudget {
    entries: usize,
    bytes: u64,
}

impl ExpansionBudget {
    fn record(&mut self, value: &impl Serialize) -> Result<(), ForgeError> {
        self.entries = self.entries.saturating_add(1);
        if self.entries > MAX_EXPANDED_REFERENCES {
            return Err(super::error("authoring plan exceeds the expanded reference limit"));
        }
        let bytes = serde_json::to_vec(value)
            .map_err(|cause| super::error(format!("cannot measure authoring evidence: {cause}")))?;
        self.bytes = self.bytes.saturating_add(bytes.len() as u64);
        if self.bytes > manifest::MAX_TOTAL_BYTES {
            return Err(super::error("authoring plan exceeds the expanded evidence byte limit"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::applicability::model::{ClassificationCounts, ControlResult, ReportFilters};
    use crate::authoring::manifest::{AnswerPin, AnswerState, Deferral, HumanClause, PinnedFile};
    use crate::authoring::model::LoadedClause;
    use crate::hashing::sha256_hex;
    use crate::mapping::inventory::ResourceEvidence;
    use crate::mapping::manifest::ResourceType;

    const PACK: &[u8] = include_bytes!("../../tests/fixtures/authoring/contracts/valid-pack.json");
    const PROJECT: &[u8] =
        include_bytes!("../../tests/fixtures/authoring/contracts/valid-project.json");

    fn loaded() -> LoadedAuthorProject {
        LoadedAuthorProject {
            project: manifest::parse_project(PROJECT).expect("project fixture"),
            pack: manifest::parse_pack(PACK).expect("pack fixture"),
            baseline_report: ApplicabilityReport {
                schema_version: crate::applicability::model::REPORT_SCHEMA_VERSION,
                manifest_sha256: "c".repeat(64),
                framework: ResourceEvidence {
                    resource_type: ResourceType::Catalog,
                    href: "framework.json".to_string(),
                    raw_sha256: "a".repeat(64),
                    root_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
                    document_version: "1.0.0".to_string(),
                    oscal_version: "1.2.3".to_string(),
                    resolved_catalog_sha256: None,
                },
                mapping_collections: vec![],
                reviewers: vec![],
                counts: ClassificationCounts {
                    total: 1,
                    applicable_unmapped: 1,
                    ..Default::default()
                },
                filters: ReportFilters::default(),
                matched_controls: 1,
                controls: vec![control("sample-1", GapClassification::ApplicableUnmapped)],
                review_queue: vec![],
            },
            inputs: vec![],
            project_sha256: "e".repeat(64),
            pack_sha256: "d".repeat(64),
            report_sha256: "b".repeat(64),
            clauses: BTreeMap::new(),
        }
    }

    fn control(id: &str, classification: GapClassification) -> ControlResult {
        ControlResult {
            control_id: id.to_string(),
            groups: vec![],
            classification,
            reviewer_key: Some("synthetic-reviewer".to_string()),
            reviewed_at: Some("2026-09-01T00:00:00Z".to_string()),
            rationale: None,
            revisit_date: None,
            note: None,
            positive_mapping_count: 0,
            no_relationship_count: usize::from(
                classification == GapClassification::ApplicableReviewedNoRelationship,
            ),
            policy_sources: vec![],
        }
    }

    fn add_answer(loaded: &mut LoadedAuthorProject) {
        let question = &loaded.pack.questions[0];
        loaded.project.answers.push(Answer {
            key: "sample-answer".to_string(),
            question_key: question.key.clone(),
            question_sha256: manifest::question_sha256(question).expect("question hash"),
            authoring_pack_sha256: loaded.pack_sha256.clone(),
            owner: question.owner.clone(),
            source_label: "Synthetic interview".to_string(),
            sensitivity: question.sensitivity,
            review: loaded.project.baseline_review.clone(),
            expires_at: None,
            state: AnswerState::Provided,
            value: Some(json!("Fictional private owner value")),
        });
    }

    fn add_clause(loaded: &mut LoadedAuthorProject) {
        let bytes = b"Explicitly supplied synthetic human clause.\n".to_vec();
        let clause = HumanClause {
            key: "sample-clause".to_string(),
            policy_key: "sample-policy".to_string(),
            topic_key: "sample-topic".to_string(),
            gap_ids: vec![manifest::gap_id(&loaded.report_sha256, "sample-1")],
            answer_refs: loaded
                .project
                .answers
                .iter()
                .map(|answer| AnswerPin {
                    answer_key: answer.key.clone(),
                    expected_sha256: manifest::answer_sha256(answer).expect("answer hash"),
                })
                .collect(),
            source: PinnedFile {
                path: PathBuf::from("clause.md"),
                expected_sha256: sha256_hex(&bytes),
            },
            review: loaded.project.baseline_review.clone(),
        };
        loaded.clauses.insert(clause.key.clone(), LoadedClause { source: clause.clone(), bytes });
        loaded.project.human_clauses.push(clause);
    }

    fn add_independent_section(loaded: &mut LoadedAuthorProject) {
        loaded.pack.topics.push(Topic {
            key: "independent-topic".to_string(),
            title: "Independent section".to_string(),
            order: 2,
            question_keys: vec![],
        });
        loaded.pack.family_assignments.push(FamilyAssignment {
            key: "independent-family-edge".to_string(),
            topic_key: "independent-topic".to_string(),
            policy_family_key: "sample-family".to_string(),
            review: loaded.project.baseline_review.clone(),
        });
    }

    #[test]
    fn missing_context_blocks_only_dependent_sections_and_preserves_assignment_review() {
        let mut loaded = loaded();
        add_independent_section(&mut loaded);
        let plan = build_plan(&loaded).expect("plan");
        assert_eq!(plan.counts, GapCounts { total: 1, assigned: 1, deferred: 0, unresolved: 0 });
        assert_eq!(plan.policies[0].state, DraftState::BlockedContext);
        assert_eq!(plan.policies[0].sections[0].state, DraftState::BlockedContext);
        assert_eq!(plan.policies[0].sections[1].state, DraftState::SkeletonReady);
        assert_eq!(plan.unresolved_questions.len(), 1);
        assert_eq!(plan.unresolved_questions[0].state, AnswerStatus::Missing);
        assert_eq!(
            plan.gaps[0].assignments[0].control_assignment.review,
            loaded.pack.control_assignments[0].review
        );
        assert_eq!(
            plan.gaps[0].assignments[0].family_assignment.review,
            loaded.pack.family_assignments[0].review
        );
    }

    #[test]
    fn one_gap_with_multiple_destinations_is_counted_once() {
        let mut loaded = loaded();
        add_independent_section(&mut loaded);
        loaded.pack.control_assignments.push(ControlAssignment {
            key: "second-control-edge".to_string(),
            control_id: "sample-1".to_string(),
            topic_key: "independent-topic".to_string(),
            review: loaded.project.baseline_review.clone(),
        });
        let plan = build_plan(&loaded).expect("plan");
        assert_eq!(plan.counts.total, 1);
        assert_eq!(plan.counts.assigned, 1);
        assert_eq!(plan.gaps[0].assignments.len(), 2);
        assert_eq!(plan.policies[0].sections[0].gap_ids, plan.policies[0].sections[1].gap_ids);
    }

    #[test]
    fn gaps_without_selected_policies_remain_unresolved_and_deferrals_are_explicit() {
        let mut loaded = loaded();
        loaded.project.policies.clear();
        let plan = build_plan(&loaded).expect("unresolved plan");
        assert_eq!(plan.counts.unresolved, 1);
        assert_eq!(plan.questions.len(), 1);
        assert!(plan.unresolved_questions.is_empty());
        loaded.project.deferrals.push(Deferral {
            key: "postponed-drafting".to_string(),
            gap_id: plan.gaps[0].gap_id.clone(),
            review: loaded.project.baseline_review.clone(),
            revisit_date: Some("2026-10-01".to_string()),
        });
        let deferred = build_plan(&loaded).expect("deferred plan");
        assert_eq!(deferred.counts.deferred, 1);
        assert!(deferred.unresolved_gaps.is_empty());
        loaded.project.policies = manifest::parse_project(PROJECT).unwrap().policies;
        assert!(build_plan(&loaded).unwrap_err().to_string().contains("assigned and deferred"));
    }

    #[test]
    fn optional_unavailable_context_is_visible_and_only_explicit_clause_dependency_blocks() {
        let mut loaded = loaded();
        loaded.pack.questions[0].required = false;
        add_answer(&mut loaded);
        loaded.project.answers[0].state = AnswerState::NoAnswer;
        loaded.project.answers[0].value = None;
        let plan = build_plan(&loaded).expect("optional no-answer");
        assert_eq!(plan.policies[0].state, DraftState::SkeletonReady);
        assert_eq!(plan.unresolved_questions[0].state, AnswerStatus::NoAnswer);
        add_clause(&mut loaded);
        let blocked = build_plan(&loaded).expect("explicit dependency");
        assert_eq!(blocked.policies[0].state, DraftState::BlockedContext);
        assert_eq!(blocked.policies[0].sections[0].clause_keys, ["sample-clause"]);
        assert!(!blocked.policies[0].sections[0].questions[0].required);
    }

    #[test]
    fn invalid_stale_and_expired_answers_block_only_their_section() {
        let mut base = loaded();
        add_answer(&mut base);
        add_independent_section(&mut base);
        for status in [AnswerStatus::Invalid, AnswerStatus::Stale, AnswerStatus::Expired] {
            let mut changed = base.clone();
            match status {
                AnswerStatus::Invalid => changed.project.answers[0].value = Some(json!(42)),
                AnswerStatus::Stale => changed.project.answers[0].question_sha256 = "f".repeat(64),
                AnswerStatus::Expired => {
                    changed.project.answers[0].expires_at =
                        Some("2026-09-07T00:00:00Z".to_string());
                }
                _ => unreachable!(),
            }
            let plan = build_plan(&changed).expect("scoped invalid context");
            assert_eq!(plan.questions[0].state, status);
            assert_eq!(plan.policies[0].sections[0].state, DraftState::BlockedContext);
            assert_eq!(plan.policies[0].sections[1].state, DraftState::SkeletonReady);
        }
    }

    #[test]
    fn explicit_human_clause_yields_draft_state_without_disclosing_answer_values() {
        let mut loaded = loaded();
        add_answer(&mut loaded);
        add_clause(&mut loaded);
        let plan = build_plan(&loaded).expect("human draft");
        assert_eq!(plan.policies[0].state, DraftState::HumanDraftPresent);
        let json = String::from_utf8(super::super::report::render_json(&plan).unwrap()).unwrap();
        let text = super::super::report::render_text(&plan);
        for rendered in [&json, &text] {
            assert!(!rendered.contains("Fictional private owner value"));
            assert!(
                rendered.contains(&manifest::answer_sha256(&loaded.project.answers[0]).unwrap())
            );
            for word in ["compliant", "certified", "implemented", "effective", "approved"] {
                assert!(!rendered.contains(word), "generated terminology: {word}");
            }
        }
        assert!(json.ends_with('\n'));
        loaded.project.answers[0].value = Some(json!("Changed explicit value"));
        assert!(build_plan(&loaded).unwrap_err().to_string().contains("answer pin"));
    }

    #[test]
    fn assignments_for_non_gap_classes_do_not_expand_the_gap_denominator() {
        let mut loaded = loaded();
        let report = &mut loaded.baseline_report;
        report.controls.extend([
            control("sample-2", GapClassification::ApplicableReviewedNoRelationship),
            control("sample-3", GapClassification::ApplicableMapped),
            control("sample-4", GapClassification::NotApplicable),
            control("sample-5", GapClassification::Deferred),
            control("sample-6", GapClassification::UnderReview),
        ]);
        report.counts = ClassificationCounts {
            total: 6,
            applicable_mapped: 1,
            applicable_reviewed_no_relationship: 1,
            applicable_unmapped: 1,
            not_applicable: 1,
            deferred: 1,
            under_review: 1,
        };
        report.matched_controls = 6;
        let plan = build_plan(&loaded).expect("complete baseline");
        assert_eq!(plan.counts, GapCounts { total: 2, assigned: 1, deferred: 0, unresolved: 1 });
        assert_eq!(plan.unresolved_gaps[0].control_id, "sample-2");
        report_counts_corruption_is_rejected(&mut loaded);
    }

    fn report_counts_corruption_is_rejected(loaded: &mut LoadedAuthorProject) {
        loaded.baseline_report.counts.applicable_unmapped += 1;
        assert!(build_plan(loaded).unwrap_err().to_string().contains("totals do not reconcile"));
        loaded.baseline_report.counts.applicable_unmapped -= 1;
        loaded.baseline_report.controls.pop();
        assert!(build_plan(loaded).unwrap_err().to_string().contains("complete, unfiltered"));
    }

    #[test]
    fn filtered_and_duplicate_baselines_are_rejected() {
        let mut loaded = loaded();
        loaded.baseline_report.filters.control_prefix = Some("sample".to_string());
        assert!(build_plan(&loaded).unwrap_err().to_string().contains("unfiltered"));
        loaded.baseline_report.filters = ReportFilters::default();
        loaded.baseline_report.controls.push(loaded.baseline_report.controls[0].clone());
        loaded.baseline_report.counts.total = 2;
        loaded.baseline_report.counts.applicable_unmapped = 2;
        loaded.baseline_report.matched_controls = 2;
        assert!(build_plan(&loaded).unwrap_err().to_string().contains("duplicate control"));
    }

    #[test]
    fn captured_fingerprints_must_match_the_exact_project_pins() {
        let base = loaded();
        let mut changed = base.clone();
        changed.report_sha256 = "0".repeat(64);
        assert!(build_plan(&changed).unwrap_err().to_string().contains("fingerprints"));
        let mut changed = base;
        changed.pack_sha256 = "0".repeat(64);
        assert!(build_plan(&changed).unwrap_err().to_string().contains("fingerprints"));
    }

    #[test]
    fn policy_with_no_explicit_topic_assignment_remains_planned() {
        let mut loaded = loaded();
        loaded.pack.family_assignments.clear();
        let plan = build_plan(&loaded).expect("empty policy");
        assert_eq!(plan.policies[0].state, DraftState::Planned);
        assert!(plan.policies[0].sections.is_empty());
        assert_eq!(plan.counts.unresolved, 1);
    }

    #[test]
    fn semantic_ordering_and_repeated_reports_are_stable() {
        let mut loaded = loaded();
        add_independent_section(&mut loaded);
        add_answer(&mut loaded);
        add_clause(&mut loaded);
        let first = build_plan(&loaded).expect("first plan");
        loaded.pack.topics.reverse();
        loaded.pack.family_assignments.reverse();
        loaded.pack.control_assignments.reverse();
        loaded.project.policies.reverse();
        let second = build_plan(&loaded).expect("reordered plan");
        assert_eq!(
            super::super::report::render_json(&first).unwrap(),
            super::super::report::render_json(&second).unwrap()
        );
        assert_eq!(
            super::super::report::render_text(&first),
            super::super::report::render_text(&second)
        );
    }

    #[test]
    fn evidence_expansion_is_bounded_before_repeated_cloning() {
        let mut budget = ExpansionBudget { entries: MAX_EXPANDED_REFERENCES, bytes: 0 };
        assert!(budget.record(&"one more").unwrap_err().to_string().contains("reference limit"));
        let mut budget = ExpansionBudget { entries: 0, bytes: manifest::MAX_TOTAL_BYTES };
        assert!(budget.record(&"one more").unwrap_err().to_string().contains("byte limit"));
    }
}
