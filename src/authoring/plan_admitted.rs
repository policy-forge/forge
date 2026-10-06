//! Pre-growth descriptors around unchanged complete native plan stages.
//!
//! This is a plain evaluator. It neither captures an original nor issues currentness.
//! The128-byte retained-record quantum and JSON escape bounds are declared logical
//! accounting units, not observations of allocator or regex-engine heap usage.

use super::super::admitted::{Admission, Metrics};
use super::super::borrowed::{AuthoringCharge, BorrowedAuthoringError};
use super::super::model::{AuthoringPlan, LoadedAuthorProject, PLAN_SCHEMA_VERSION};
use serde::Serialize;

/// Build complete ordinary native plan data using the caller's original admission/control.
/// Each unchanged native stage receives its full descriptor before growth; success and
/// ordinary Domain failure retain their result through the same post-stage checkpoint.
pub(in crate::authoring) fn build_plan_admitted_inner<
    E,
    F: FnMut(AuthoringCharge) -> Result<(), E>,
>(
    loaded: &LoadedAuthorProject,
    a: &mut Admission<'_, E, F>,
) -> Result<AuthoringPlan, BorrowedAuthoringError<E>> {
    a.step()?;
    let first = a.equal(&loaded.report_sha256, &loaded.project.baseline.report_sha256)?;
    let second = a.equal(&loaded.pack_sha256, &loaded.project.authoring_pack.expected_sha256)?;
    if !first || !second {
        return Err(BorrowedAuthoringError::Domain(super::super::error(
            "captured authoring fingerprints do not match project pins",
        )));
    }
    let report = a.measure(&loaded.baseline_report)?;
    registry(report, loaded.baseline_report.controls.len(), 1, a)?;
    a.native(|| super::validate_baseline_counts(&loaded.baseline_report))?;
    super::super::borrowed::admit_relationships(
        &loaded.pack,
        &loaded.project,
        &loaded.baseline_report,
        a,
    )?;
    a.native(|| {
        super::manifest::validate_relationships(
            &loaded.pack,
            &loaded.project,
            &loaded.baseline_report,
        )
    })?;
    let mut budget = super::ExpansionBudget::default();
    admit_questions(loaded, a)?;
    let questions = a.native(|| super::evaluate_questions(loaded))?;
    admit_gaps(loaded, a)?;
    let gaps = a.native(|| super::build_gaps(loaded, &mut budget))?;
    let gap_metric = a.measure(&gaps)?;
    registry(gap_metric, gaps.len(), 1, a)?;
    let counts = a.native(|| super::count_gaps(&gaps))?;
    a.step()?;
    let expected = loaded
        .baseline_report
        .counts
        .applicable_unmapped
        .checked_add(loaded.baseline_report.counts.applicable_reviewed_no_relationship)
        .ok_or_else(|| {
            BorrowedAuthoringError::Domain(super::super::error("applicable gap count overflow"))
        })?;
    if counts.total != expected {
        return Err(BorrowedAuthoringError::Domain(super::super::error(
            "applicable gap totals do not match the complete baseline",
        )));
    }
    admit_policies(loaded, &gaps, &questions, a)?;
    let policies = a.native(|| super::build_policies(loaded, &gaps, &questions, &mut budget))?;
    // Admit every actual question occurrence, without taking deduplication credit.
    // Unresolved filters retain native semantics; reserve every possible full clone.
    let policy_metric = a.measure(&policies)?;
    let mut references = 0usize;
    for policy in &policies {
        for section in &policy.sections {
            a.step()?;
            references = a.add(references, section.questions.len())?;
        }
    }
    let reference_slots = a.add(references, questions.len())?;
    registry(policy_metric, reference_slots, 1, a)?;
    clones(&questions, 1, 1, a)?;
    clones(&gaps, 1, 1, a)?;
    let unresolved = a.native(|| Ok(collect_unresolved(&policies, &questions, &gaps)))?;
    admit_provenance(loaded, a)?;
    let provenance = a.native(|| Ok(super::build_provenance(loaded)))?;
    let header = a.add(loaded.project.project_key.len(), loaded.project.as_of.len())?;
    let header = a.add(header, PLAN_SCHEMA_VERSION.len() + 512)?;
    a.reserve(header)?;
    a.work(3, header, 0)?;
    let result = Ok(AuthoringPlan {
        schema_version: PLAN_SCHEMA_VERSION.to_owned(),
        project_key: loaded.project.project_key.clone(),
        as_of: loaded.project.as_of.clone(),
        provenance,
        counts,
        gaps,
        policies,
        questions,
        unresolved_questions: unresolved.0,
        unresolved_gaps: unresolved.1,
    });
    a.finish(result)
}

/// Collect exactly the unchanged native unresolved rows after their complete clone/set admission.
fn collect_unresolved(
    policies: &[super::PolicyPlan],
    questions: &[super::QuestionEvaluation],
    gaps: &[super::GapPlan],
) -> (Vec<super::QuestionEvaluation>, Vec<super::GapPlan>) {
    let referenced_keys: std::collections::BTreeSet<_> = policies
        .iter()
        .flat_map(|policy| &policy.sections)
        .flat_map(|section| &section.questions)
        .map(|question| question.question_key.as_str())
        .collect();
    let questions = questions
        .iter()
        .filter(|question| {
            question.state != super::AnswerStatus::Available
                && referenced_keys.contains(question.question_key.as_str())
        })
        .cloned()
        .collect();
    let gaps = gaps
        .iter()
        .filter(|gap| gap.disposition == super::GapDisposition::Unresolved)
        .cloned()
        .collect();
    (questions, gaps)
}

/// Fixed traversal measures complete private fields before any later clone/materialization.
fn clones<E, F: FnMut(AuthoringCharge) -> Result<(), E>, T: Serialize + ?Sized>(
    value: &T,
    copies: usize,
    passes: usize,
    a: &mut Admission<'_, E, F>,
) -> Result<Metrics, BorrowedAuthoringError<E>> {
    let metric = a.measure(value)?;
    a.materialize(metric, copies, passes)?;
    Ok(metric)
}

/// Full input-set cardinality bounds native `BTree` entries and key comparisons before use.
/// A square comparison envelope admits the complete supplied set, not a caller subset.
pub(in crate::authoring) fn registry<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    metric: Metrics,
    slots: usize,
    phases: usize,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let slots = a.add(slots, 1)?;
    let headers = a.mul(slots, 128)?;
    let headers = a.mul(headers, phases)?;
    a.reserve(headers)?;
    let matching = a.mul(slots, slots)?;
    let matching = a.mul(matching, phases)?;
    let bytes = a.mul(metric.text, slots)?;
    let bytes = a.mul(bytes, phases)?;
    let visits = a.mul(slots, phases)?;
    a.work(visits, bytes, matching)
}

/// Canonical Value/object-map rebuild/JSON/frame/hash are five complete operand passes.
/// Reserve Value plus moved sorted map plus encoded/framed buffers before native hashing.
pub(in crate::authoring) fn canonical<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    metric: Metrics,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    a.materialize(metric, 3, 5)?;
    registry(metric, metric.nodes, 2, a)?;
    a.reserve(128)?;
    a.work(1, 128, 0)
}

/// Admit every native regex compile and bounded-engine matching of complete private operands.
pub(in crate::authoring) fn regex<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    pattern: &str,
    text: usize,
    compiles: usize,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let engine = a.mul(2 * 1024 * 1024, compiles)?;
    a.reserve(engine)?;
    let compile_work = a.mul(pattern.len(), 1024 * 1024)?;
    let compile_work = a.mul(compile_work, compiles)?;
    let scan_work = a.mul(text, 1024 * 1024)?;
    let bytes = a.add(compile_work, scan_work)?;
    a.work(compiles, bytes, scan_work)
}

/// Precharge whole context evaluation, complete hashes, timestamps, regex and retained fields.
fn admit_questions<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    loaded: &LoadedAuthorProject,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let answers = a.measure(&loaded.project.answers)?;
    registry(answers, loaded.project.answers.len(), 1, a)?;
    let all = a.measure(&loaded.pack.questions)?;
    registry(all, loaded.pack.questions.len(), 1, a)?;
    for question in &loaded.pack.questions {
        a.step()?;
        let metric = a.measure(question)?;
        canonical(metric, a)?;
        // Full question data upper-bounds QuestionEvaluation's copied private text;
        // three new fixed hashes and answer optional fields are separately retained.
        let output = a.add(metric.logical, 512)?;
        a.reserve(output)?;
        a.work(1, loaded.project.as_of.len(), 0)?;
        for answer in &loaded.project.answers {
            if a.equal(&question.key, &answer.question_key)? {
                let metric = a.measure(answer)?;
                canonical(metric, a)?;
                a.materialize(metric, 1, 3)?;
                if let Some(pattern) = &question.constraints.regex {
                    let allowed = a.measure(&question.constraints.allowed_values)?;
                    let text = a.add(metric.text, allowed.text)?;
                    regex(pattern, text, 2, a)?;
                }
            }
        }
        // validate_question executes even with no supplied answer.
        let allowed = a.measure(&question.constraints.allowed_values)?;
        a.materialize(allowed, 1, 2)?;
        registry(allowed, question.constraints.allowed_values.len(), 1, a)?;
        if let Some(pattern) = &question.constraints.regex {
            regex(pattern, allowed.text, 1, a)?;
        }
    }
    Ok(())
}

/// Admit exact complete control/topic/family/policy Cartesian assignment rows before cloning.
fn admit_gaps<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    loaded: &LoadedAuthorProject,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let mut metric = a.measure(&loaded.pack)?;
    let project_metric = a.measure(&loaded.project)?;
    metric.text = a.add(metric.text, project_metric.text)?;
    let slots =
        a.add(loaded.pack.control_assignments.len(), loaded.pack.family_assignments.len())?;
    let slots = a.add(slots, loaded.project.policies.len())?;
    let slots = a.add(slots, loaded.project.deferrals.len())?;
    registry(metric, slots, 2, a)?;
    let deferrals = a.measure(&loaded.project.deferrals)?;
    a.materialize(deferrals, 1, 2)?;
    let mut rows = 0usize;
    let mut assignment_text = 0usize;
    for control in &loaded.baseline_report.controls {
        a.step()?;
        a.work(1, control.control_id.len(), 0)?;
        if !super::is_applicable_gap(control.classification) {
            continue;
        }
        // Gap hash owns a framed buffer, digest and complete control/classification strings.
        let extent = a.add(control.control_id.len(), loaded.report_sha256.len())?;
        let payload = a.add(extent, 512)?;
        a.reserve(payload)?;
        let work_extent = a.add(extent, 128)?;
        let work = a.mul(work_extent, 3)?;
        a.work(1, work, 1)?;
        for edge in &loaded.pack.control_assignments {
            if !a.equal(&edge.control_id, &control.control_id)? {
                continue;
            }
            for family in &loaded.pack.family_assignments {
                if !a.equal(&edge.topic_key, &family.topic_key)? {
                    continue;
                }
                for policy in &loaded.project.policies {
                    if !a.equal(&family.policy_family_key, &policy.policy_family_key)? {
                        continue;
                    }
                    let control_metric = a.measure(edge)?;
                    let family_metric = a.measure(family)?;
                    let private = a.add(control_metric.logical, family_metric.logical)?;
                    let text = a.add(policy.key.len(), edge.topic_key.len())?;
                    let private = a.add(private, text)?;
                    let private = a.add(private, 512)?;
                    // Actual expanded native evidence encoder has one full temporary buffer.
                    let encoded = a.add(control_metric.encoded, family_metric.encoded)?;
                    let encoded_text = a.mul(text, 6)?;
                    let encoded = a.add(encoded, encoded_text)?;
                    let encoded = a.add(encoded, 512)?;
                    let payload = a.add(private, encoded)?;
                    a.reserve(payload)?;
                    let nodes = a.add(control_metric.nodes, family_metric.nodes)?;
                    let nodes = a.add(nodes, 9)?;
                    a.work(nodes, encoded, 0)?;
                    rows = a.add(rows, 1)?;
                    assignment_text = a.add(assignment_text, text)?;
                    assignment_text = a.add(assignment_text, control_metric.text)?;
                    assignment_text = a.add(assignment_text, family_metric.text)?;
                }
            }
        }
    }
    let rows_square = a.mul(rows, rows)?;
    let sort_bytes = a.mul(assignment_text, rows)?;
    a.work(rows, sort_bytes, rows_square)?;
    let mut report = a.measure(&loaded.baseline_report)?;
    let gap_keys = a.mul(loaded.baseline_report.controls.len(), 64)?;
    report.text = a.add(report.text, gap_keys)?;
    registry(report, loaded.baseline_report.controls.len(), 1, a)
}

/// Admit complete section evidence clones and all question/clause occurrences before dedup.
fn admit_policies<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    loaded: &LoadedAuthorProject,
    gaps: &[super::GapPlan],
    questions: &[super::QuestionEvaluation],
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let pack = a.measure(&loaded.pack)?;
    let project = a.measure(&loaded.project)?;
    let slots = a.add(loaded.pack.topics.len(), loaded.pack.family_assignments.len())?;
    let slots = a.add(slots, loaded.project.answers.len())?;
    let slots = a.add(slots, loaded.project.human_clauses.len())?;
    let slots = a.add(slots, questions.len())?;
    registry(pack, slots, 4, a)?;
    registry(project, slots, 2, a)?;
    let mut assignment_rows = 0usize;
    for gap in gaps {
        a.step()?;
        assignment_rows = a.add(assignment_rows, gap.assignments.len())?;
    }
    let all_gaps = a.measure(gaps)?;
    registry(all_gaps, assignment_rows, 4, a)?;
    registry(project, loaded.project.policies.len(), 2, a)?;
    for gap in gaps {
        for assignment in &gap.assignments {
            clones(assignment, 1, 2, a)?;
            let ids = a.add(gap.gap_id.len(), gap.control_id.len())?;
            let payload = a.add(ids, 512)?;
            a.reserve(payload)?;
        }
    }
    for policy in &loaded.project.policies {
        clones(policy, 1, 1, a)?;
        for family in &loaded.pack.family_assignments {
            if !a.equal(&policy.policy_family_key, &family.policy_family_key)? {
                continue;
            }
            for topic in &loaded.pack.topics {
                if !a.equal(&family.topic_key, &topic.key)? {
                    continue;
                }
                clones(topic, 1, 1, a)?;
                // Section keys/sets retain every occurrence before native deduplication.
                for key in &topic.question_keys {
                    admit_question_occurrence(key, questions, a)?;
                }
                for clause in &loaded.project.human_clauses {
                    if !a.equal(&policy.key, &clause.policy_key)?
                        || !a.equal(&topic.key, &clause.topic_key)?
                    {
                        continue;
                    }
                    let key = a.add(clause.key.len(), 128)?;
                    a.reserve(key)?;
                    for reference in &clause.answer_refs {
                        for answer in &loaded.project.answers {
                            if a.equal(&reference.answer_key, &answer.key)? {
                                admit_question_occurrence(&answer.question_key, questions, a)?;
                            }
                        }
                    }
                }
            }
        }
    }
    // Whole completed arrays drive each native sort, not a caller-supplied projected count.
    let questions_metric = a.measure(questions)?;
    registry(questions_metric, questions.len(), 2, a)
}

/// Native expansion records and cloned evaluations include every private review/provenance field.
fn admit_question_occurrence<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    key: &str,
    questions: &[super::QuestionEvaluation],
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    a.reserve(128)?;
    for question in questions {
        if a.equal(key, &question.question_key)? {
            clones(question, 1, 2, a)?;
        }
    }
    Ok(())
}

/// The native provenance denominator excludes stored plan, review locator and init policy.
fn admit_provenance<E, F: FnMut(AuthoringCharge) -> Result<(), E>>(
    loaded: &LoadedAuthorProject,
    a: &mut Admission<'_, E, F>,
) -> Result<(), BorrowedAuthoringError<E>> {
    let inputs = clones(&loaded.inputs, 1, 1, a)?;
    registry(inputs, loaded.inputs.len(), 1, a)?;
    let pack = clones(&loaded.pack.reviewers, 1, 1, a)?;
    registry(pack, loaded.pack.reviewers.len(), 1, a)?;
    let project = clones(&loaded.project.reviewers, 1, 1, a)?;
    registry(project, loaded.project.reviewers.len(), 1, a)?;
    clones(&loaded.baseline_report.framework, 1, 1, a)?;
    clones(&loaded.project.baseline_review, 1, 1, a)?;
    a.reserve(3 * 64 + 512)
}
