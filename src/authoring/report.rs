//! Deterministic, value-redacted authoring-plan reports.

use std::fmt::Write as _;

use super::manifest::{Review, Sensitivity};
use super::model::{AuthoringPlan, GapPlan, QuestionEvaluation};
use crate::ForgeError;

/// Serialize one plan using FORGE's pretty JSON and trailing-LF convention.
///
/// # Errors
///
/// Returns an authoring error when serialization fails.
pub fn render_json(plan: &AuthoringPlan) -> Result<Vec<u8>, ForgeError> {
    let mut destination = BoundedJson(Vec::new());
    serde_json::to_writer_pretty(&mut destination, plan)
        .map_err(|cause| super::error(format!("cannot serialize authoring plan: {cause}")))?;
    std::io::Write::write_all(&mut destination, b"\n")
        .map_err(|cause| super::error(format!("cannot finish authoring plan: {cause}")))?;
    Ok(destination.0)
}

struct BoundedJson(Vec<u8>);

impl std::io::Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len().saturating_add(bytes.len()) as u64 > super::manifest::MAX_TOTAL_BYTES {
            return Err(std::io::Error::other("authoring plan exceeds the report byte limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Render planning states and complete assignment evidence without answer values.
#[must_use]
pub fn render_text(plan: &AuthoringPlan) -> String {
    let mut output = String::from("FORGE policy drafting plan\n");
    let _ = writeln!(output, "schema: {}", escaped(&plan.schema_version));
    let _ = writeln!(output, "project: {}", escaped(&plan.project_key));
    let _ = writeln!(output, "as-of: {}", escaped(&plan.as_of));
    append_provenance(&mut output, plan);
    let counts = &plan.counts;
    let _ = writeln!(
        output,
        "applicable gaps: total={} assigned={} deferred={} unresolved={}",
        counts.total, counts.assigned, counts.deferred, counts.unresolved
    );
    let _ = writeln!(output, "gaps: {}", plan.gaps.len());
    for gap in &plan.gaps {
        append_gap(&mut output, gap);
    }
    let _ = writeln!(output, "policies: {}", plan.policies.len());
    for policy in &plan.policies {
        let _ = writeln!(
            output,
            "- policy={} family={} title={} state={}",
            escaped(&policy.policy_key),
            escaped(&policy.policy_family_key),
            escaped(&policy.title),
            policy.state.as_str(),
        );
        for section in &policy.sections {
            let _ = writeln!(
                output,
                "  topic={} order={} title={} state={}",
                escaped(&section.topic_key),
                section.order,
                escaped(&section.title),
                section.state.as_str(),
            );
            let _ = writeln!(output, "    gaps: {}", labels(&section.gap_ids));
            let _ = writeln!(output, "    controls: {}", labels(&section.control_ids));
            let _ = writeln!(output, "    human clauses: {}", labels(&section.clause_keys));
            for question in &section.questions {
                append_question(&mut output, "    ", question);
            }
        }
    }
    let _ = writeln!(output, "questions: {}", plan.questions.len());
    for question in &plan.questions {
        append_question(&mut output, "- ", question);
    }
    let _ = writeln!(output, "unresolved questions: {}", plan.unresolved_questions.len());
    for question in &plan.unresolved_questions {
        append_question(&mut output, "- ", question);
    }
    let _ = writeln!(output, "unresolved gaps: {}", plan.unresolved_gaps.len());
    for gap in &plan.unresolved_gaps {
        let _ = writeln!(
            output,
            "- gap={} control={} classification={}",
            escaped(&gap.gap_id),
            escaped(&gap.control_id),
            escaped(&gap.classification),
        );
    }
    output
}

fn append_provenance(output: &mut String, plan: &AuthoringPlan) {
    let provenance = &plan.provenance;
    let _ = writeln!(output, "project-sha256: {}", provenance.project_sha256);
    let _ = writeln!(output, "authoring-pack-sha256: {}", provenance.pack_sha256);
    let _ = writeln!(output, "gap-report-sha256: {}", provenance.report_sha256);
    let framework = &provenance.framework;
    let _ = writeln!(
        output,
        "framework: type={} href={} raw-sha256={} root-uuid={} document-version={} oscal-version={}",
        framework.resource_type.as_str(),
        escaped(&framework.href),
        framework.raw_sha256,
        escaped(&framework.root_uuid),
        escaped(&framework.document_version),
        escaped(&framework.oscal_version),
    );
    if let Some(hash) = &framework.resolved_catalog_sha256 {
        let _ = writeln!(output, "resolved-catalog-sha256: {hash}");
    }
    append_review(output, "baseline review", &provenance.baseline_review);
    output.push_str("Reviewer metadata records asserted provenance.\n");
    let _ = writeln!(output, "pack reviewers: {}", provenance.pack_reviewers.len());
    for reviewer in &provenance.pack_reviewers {
        let _ =
            writeln!(output, "- key={} name={}", escaped(&reviewer.key), escaped(&reviewer.name));
    }
    let _ = writeln!(output, "project reviewers: {}", provenance.project_reviewers.len());
    for reviewer in &provenance.project_reviewers {
        let _ =
            writeln!(output, "- key={} name={}", escaped(&reviewer.key), escaped(&reviewer.name));
    }
    let _ = writeln!(output, "inputs: {}", provenance.inputs.len());
    for input in &provenance.inputs {
        let _ = writeln!(
            output,
            "- role={} path={} sha256={} bytes={}",
            escaped(&input.role),
            escaped(&input.path),
            input.sha256,
            input.byte_length,
        );
    }
}

fn append_gap(output: &mut String, gap: &GapPlan) {
    let _ = writeln!(
        output,
        "- gap={} control={} classification={} disposition={}",
        escaped(&gap.gap_id),
        escaped(&gap.control_id),
        escaped(&gap.classification),
        gap.disposition.as_str(),
    );
    for assignment in &gap.assignments {
        let _ = writeln!(
            output,
            "  policy={} topic={} control-assignment={} family-assignment={}",
            escaped(&assignment.policy_key),
            escaped(&assignment.topic_key),
            escaped(&assignment.control_assignment.key),
            escaped(&assignment.family_assignment.key),
        );
        append_review(
            output,
            "    control assignment review",
            &assignment.control_assignment.review,
        );
        append_review(output, "    family assignment review", &assignment.family_assignment.review);
    }
    if let Some(deferral) = &gap.deferral {
        let _ = writeln!(
            output,
            "  deferral={} revisit-date={}",
            escaped(&deferral.key),
            deferral.revisit_date.as_deref().map_or_else(|| "none".to_string(), escaped),
        );
        append_review(output, "    deferral review", &deferral.review);
    }
}

fn append_question(output: &mut String, prefix: &str, question: &QuestionEvaluation) {
    let _ = writeln!(
        output,
        "{prefix}question={} question-sha256={} required={} state={} owner={} sensitivity={} source={}",
        escaped(&question.question_key),
        question.question_sha256,
        question.required,
        question.state.as_str(),
        escaped(&question.owner),
        sensitivity_label(question.sensitivity),
        escaped(&question.source_label),
    );
    if let Some(key) = &question.answer_key {
        let _ = writeln!(
            output,
            "{prefix}  answer={} answer-sha256={}",
            escaped(key),
            question.answer_sha256.as_deref().unwrap_or("none"),
        );
    }
    if let Some(review) = &question.review {
        append_review(output, &format!("{prefix}  answer review"), review);
    }
    if let Some(expires_at) = &question.expires_at {
        let _ = writeln!(output, "{prefix}  expires-at={}", escaped(expires_at));
    }
}

fn append_review(output: &mut String, prefix: &str, review: &Review) {
    let _ = writeln!(
        output,
        "{prefix}: reviewer={} reviewed-at={} rationale={}",
        escaped(&review.reviewer_key),
        escaped(&review.reviewed_at),
        escaped(&review.rationale),
    );
}

fn labels(values: &[String]) -> String {
    values.iter().map(|value| escaped(value)).collect::<Vec<_>>().join(", ")
}

fn escaped(value: &str) -> String {
    value.chars().flat_map(char::escape_debug).collect()
}

fn sensitivity_label(value: Sensitivity) -> &'static str {
    match value {
        Sensitivity::Public => "public",
        Sensitivity::Internal => "internal",
        Sensitivity::Confidential => "confidential",
        Sensitivity::Restricted => "restricted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_metadata_escapes_control_characters_without_reflow() {
        assert_eq!(escaped("line\nnext\t\r\u{1b}"), "line\\nnext\\t\\r\\u{1b}");
        assert_eq!(labels(&["a\nb".to_string(), "c".to_string()]), "a\\nb, c");
    }
}
