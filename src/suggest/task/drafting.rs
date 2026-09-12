//! `forge.suggest-task-drafting/1`: policy-drafting task payloads.
//!
//! A clause is proposed against one authoring topic; the destination is a
//! clause file a human still owns. The task carries no organization facts of
//! its own — anything the model asserts must be cited to a supplied unit.

use serde::{Deserialize, Serialize};

use crate::ForgeError;

use super::super::shared;
use super::{Citation, citations};

/// Version of the policy-drafting task payload.
pub const TASK_SCHEMA_VERSION: &str = "forge.suggest-task-drafting/1";
/// Maximum sections in one drafting request.
pub const MAX_SECTIONS: usize = 1_000;
/// Maximum section order accepted from an authoring plan.
pub const MAX_ORDER: u32 = 100_000;

/// One unresolved section the model may draft against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftingSection {
    /// Policy key in the supplied project.
    pub policy_key: String,
    /// Topic key in the supplied authoring plan.
    pub topic_key: String,
    /// Plan order of the section.
    pub order: u32,
    /// Human title of the section.
    pub title: String,
    /// Exact supplied prompt or question text for the section.
    pub prompt: String,
    /// Gap identifiers this section is expected to address.
    #[serde(default)]
    pub gap_ids: Vec<String>,
    /// Framework control identifiers in scope for this section.
    #[serde(default)]
    pub control_ids: Vec<String>,
}

impl DraftingSection {
    fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.policy_key"), &self.policy_key)?;
        shared::key(&format!("{name}.topic_key"), &self.topic_key)?;
        if self.order > MAX_ORDER {
            return Err(shared::error(format!("{name}.order must be at most {MAX_ORDER}")));
        }
        shared::single_line(&format!("{name}.title"), &self.title)?;
        shared::text(&format!("{name}.prompt"), &self.prompt)?;
        super::mapping::identifiers(&format!("{name}.gap_ids"), &self.gap_ids)?;
        super::mapping::identifiers(&format!("{name}.control_ids"), &self.control_ids)?;
        Ok(())
    }
}

/// One proposed draft clause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DraftClause {
    /// Policy key the clause belongs to.
    pub policy_key: String,
    /// Topic key the clause belongs to.
    pub topic_key: String,
    /// Optional authoring question the clause answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question_key: Option<String>,
    /// Proposed heading line, or the section's own heading when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,
    /// Proposed clause text, in the model's own words.
    pub draft_text: String,
    /// Allowlisted units supporting the clause.
    pub citations: Vec<Citation>,
    /// Explicit assumptions the reviewer must check.
    pub assumptions: Vec<String>,
    /// Questions the model could not resolve from the supplied context.
    pub unresolved_questions: Vec<String>,
}

impl DraftClause {
    /// Validate one clause's shape. Citation resolution needs the request.
    pub(in crate::suggest) fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.policy_key"), &self.policy_key)?;
        shared::key(&format!("{name}.topic_key"), &self.topic_key)?;
        if let Some(question_key) = &self.question_key {
            shared::key(&format!("{name}.question_key"), question_key)?;
        }
        if let Some(heading) = &self.heading {
            shared::single_line(&format!("{name}.heading"), heading)?;
        }
        shared::text(&format!("{name}.draft_text"), &self.draft_text)?;
        citations(&format!("{name}.citations"), &self.citations)?;
        shared::text_list(
            &format!("{name}.assumptions"),
            &self.assumptions,
            super::MAX_SUGGESTIONS,
        )?;
        shared::text_list(
            &format!("{name}.unresolved_questions"),
            &self.unresolved_questions,
            super::MAX_SUGGESTIONS,
        )?;
        Ok(())
    }
}

/// Validate the section list of one request.
pub(in crate::suggest) fn sections(
    name: &str,
    values: &[DraftingSection],
) -> Result<(), ForgeError> {
    if values.len() > MAX_SECTIONS {
        return Err(shared::error(format!("{name} exceeds {MAX_SECTIONS} entries")));
    }
    for (index, section) in values.iter().enumerate() {
        section.validate(&format!("{name}[{index}]"))?;
    }
    Ok(())
}

/// Validate the clause list of one response or bundle.
pub(in crate::suggest) fn clauses(name: &str, values: &[DraftClause]) -> Result<(), ForgeError> {
    if values.len() > super::MAX_SUGGESTIONS {
        return Err(shared::error(format!("{name} exceeds {} entries", super::MAX_SUGGESTIONS)));
    }
    for (index, clause) in values.iter().enumerate() {
        clause.validate(&format!("{name}[{index}]"))?;
    }
    Ok(())
}

/// A minimal valid clause, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_clause_json() -> serde_json::Value {
    serde_json::json!({
        "policy_key": "access-policy",
        "topic_key": "access-control",
        "question_key": "review-cadence",
        "heading": "Review cadence",
        "draft_text": "Accounts must be reviewed every quarter.",
        "citations": [{"unit_id": "unit-0001", "quote": "Accounts must be reviewed every quarter."}],
        "assumptions": ["Quarterly review is the intended cadence."],
        "unresolved_questions": ["Who approves the review?"]
    })
}

/// A minimal valid section, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_section_json() -> serde_json::Value {
    serde_json::json!({
        "policy_key": "access-policy",
        "topic_key": "access-control",
        "order": 1,
        "title": "Access control",
        "prompt": "Describe how accounts are reviewed.",
        "gap_ids": ["gap-0001"],
        "control_ids": ["ac-2"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn clause() -> DraftClause {
        DraftClause {
            policy_key: "access-policy".to_string(),
            topic_key: "access-control".to_string(),
            question_key: Some("review-cadence".to_string()),
            heading: Some("Review cadence".to_string()),
            draft_text: "Accounts must be reviewed quarterly.".to_string(),
            citations: vec![Citation { unit_id: "unit-0001".to_string(), quote: None }],
            assumptions: vec![],
            unresolved_questions: vec!["Who owns the review?".to_string()],
        }
    }

    #[test]
    fn clauses_accept_a_complete_grounded_shape() {
        assert!(clauses("clauses", &[clause()]).is_ok());
    }

    #[test]
    fn clauses_reject_multiline_headings_and_blank_drafts() {
        let mut heading = clause();
        heading.heading = Some("two\nlines".to_string());
        assert!(clauses("clauses", &[heading]).is_err());

        let mut blank = clause();
        blank.draft_text = " \t ".to_string();
        assert!(clauses("clauses", &[blank]).is_err());

        let mut bad_question = clause();
        bad_question.question_key = Some("Review Cadence".to_string());
        assert!(clauses("clauses", &[bad_question]).is_err());
    }

    #[test]
    fn sections_bound_the_plan_order() {
        let section = DraftingSection {
            policy_key: "access-policy".to_string(),
            topic_key: "access-control".to_string(),
            order: 3,
            title: "Access control".to_string(),
            prompt: "Describe how accounts are reviewed.".to_string(),
            gap_ids: vec!["gap-0001".to_string()],
            control_ids: vec!["ac-2".to_string()],
        };
        assert!(sections("sections", std::slice::from_ref(&section)).is_ok());

        let mut unbounded = section;
        unbounded.order = MAX_ORDER + 1;
        assert!(sections("sections", &[unbounded]).is_err());
    }

    #[test]
    fn section_counts_are_bounded() {
        let section: DraftingSection = serde_json::from_value(fixture_section_json()).unwrap();
        let too_many = vec![section; MAX_SECTIONS + 1];
        assert!(sections("sections", &too_many).is_err());
    }

    #[test]
    fn drafting_task_schema_file_is_published_and_closed() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/forge.suggest-task-drafting-1.schema.json"
        ))
        .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-task-drafting/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
