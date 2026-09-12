//! `forge.suggest-task-mapping/1`: mapping-candidate task payloads.
//!
//! The task is versioned separately from the request and response envelopes so
//! a new task can be added without reissuing the pipeline contracts. FORGE
//! never asks the model for a confidence score: candidate evidence is rated
//! deterministically from citations, not from self-report.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::mapping::manifest::Relationship;

use super::super::shared;
use super::{Citation, citations};

/// Version of the mapping-candidate task payload.
pub const TASK_SCHEMA_VERSION: &str = "forge.suggest-task-mapping/1";
/// Maximum subjects in one mapping-candidate request.
pub const MAX_SUBJECTS: usize = 1_000;
/// Maximum control or gap hints per subject.
pub const MAX_HINTS: usize = 128;

/// One policy subject the model may relate to framework controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingSubject {
    /// Policy key in the supplied project.
    pub policy_key: String,
    /// Topic key in the supplied authoring plan.
    pub topic_key: String,
    /// Human title of the subject.
    pub title: String,
    /// Exact supplied subject text.
    pub text: String,
    /// Framework control identifiers in scope for this subject.
    #[serde(default)]
    pub control_ids: Vec<String>,
    /// Gap identifiers this subject is expected to address.
    #[serde(default)]
    pub gap_ids: Vec<String>,
}

impl MappingSubject {
    fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.policy_key"), &self.policy_key)?;
        shared::key(&format!("{name}.topic_key"), &self.topic_key)?;
        shared::single_line(&format!("{name}.title"), &self.title)?;
        shared::text(&format!("{name}.text"), &self.text)?;
        identifiers(&format!("{name}.control_ids"), &self.control_ids)?;
        identifiers(&format!("{name}.gap_ids"), &self.gap_ids)?;
        Ok(())
    }
}

/// One proposed control relationship for a subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingCandidate {
    /// Policy key the candidate relates.
    pub policy_key: String,
    /// Topic key the candidate relates.
    pub topic_key: String,
    /// Framework control identifier the candidate proposes.
    pub control_id: String,
    /// Proposed relationship in the destination contract's vocabulary.
    pub relationship: Relationship,
    /// Why the candidate is proposed, in the model's own words.
    pub rationale: String,
    /// Allowlisted units supporting the candidate.
    pub citations: Vec<Citation>,
    /// Explicit assumptions the reviewer must check.
    #[serde(default)]
    pub assumptions: Vec<String>,
    /// Questions the model could not resolve from the supplied context.
    #[serde(default)]
    pub unresolved_questions: Vec<String>,
}

impl MappingCandidate {
    /// Validate one candidate's shape. Citation resolution needs the request.
    pub(in crate::suggest) fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.policy_key"), &self.policy_key)?;
        shared::key(&format!("{name}.topic_key"), &self.topic_key)?;
        identifier(&format!("{name}.control_id"), &self.control_id)?;
        shared::text(&format!("{name}.rationale"), &self.rationale)?;
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

/// Validate the subject list of one request.
pub(in crate::suggest) fn subjects(
    name: &str,
    values: &[MappingSubject],
) -> Result<(), ForgeError> {
    if values.len() > MAX_SUBJECTS {
        return Err(shared::error(format!("{name} exceeds {MAX_SUBJECTS} entries")));
    }
    for (index, subject) in values.iter().enumerate() {
        subject.validate(&format!("{name}[{index}]"))?;
    }
    Ok(())
}

/// Validate the candidate list of one response or bundle.
pub(in crate::suggest) fn candidates(
    name: &str,
    values: &[MappingCandidate],
) -> Result<(), ForgeError> {
    if values.len() > super::MAX_SUGGESTIONS {
        return Err(shared::error(format!("{name} exceeds {} entries", super::MAX_SUGGESTIONS)));
    }
    for (index, candidate) in values.iter().enumerate() {
        candidate.validate(&format!("{name}[{index}]"))?;
    }
    Ok(())
}

/// Non-empty, unique framework or gap identifiers.
pub(super) fn identifiers(name: &str, values: &[String]) -> Result<(), ForgeError> {
    if values.len() > MAX_HINTS {
        return Err(shared::error(format!("{name} exceeds {MAX_HINTS} entries")));
    }
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        identifier(name, value)?;
        if !seen.insert(value.as_str()) {
            return Err(shared::error(format!("{name} entries must be unique")));
        }
    }
    Ok(())
}

fn identifier(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > shared::MAX_LABEL_BYTES
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(shared::error(format!(
            "{name} must be a non-empty identifier of at most {} bytes",
            shared::MAX_LABEL_BYTES
        )));
    }
    Ok(())
}

/// A minimal valid candidate, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_candidate_json() -> serde_json::Value {
    serde_json::json!({
        "policy_key": "access-policy",
        "topic_key": "access-control",
        "control_id": "ac-2",
        "relationship": "intersects-with",
        "rationale": "The supplied statement establishes account review duties.",
        "citations": [{"unit_id": "unit-0001"}],
        "assumptions": ["Account review cadence is set elsewhere."],
        "unresolved_questions": ["Which owner approves accounts?"]
    })
}

/// A minimal valid subject, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_subject_json() -> serde_json::Value {
    serde_json::json!({
        "policy_key": "access-policy",
        "topic_key": "access-control",
        "title": "Access control",
        "text": "Accounts must be reviewed quarterly.",
        "control_ids": ["ac-2"],
        "gap_ids": ["gap-0001"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn candidate() -> MappingCandidate {
        MappingCandidate {
            policy_key: "access-policy".to_string(),
            topic_key: "access-control".to_string(),
            control_id: "ac-2".to_string(),
            relationship: Relationship::IntersectsWith,
            rationale: "The supplied statement establishes account review duties.".to_string(),
            citations: vec![Citation { unit_id: "unit-0001".to_string(), quote: None }],
            assumptions: vec!["Account review cadence is set elsewhere.".to_string()],
            unresolved_questions: vec!["Which owner approves accounts?".to_string()],
        }
    }

    #[test]
    fn candidates_accept_a_complete_grounded_shape() {
        assert!(candidates("candidates", &[candidate()]).is_ok());
    }

    #[test]
    fn candidates_reject_missing_rationale_or_unsafe_keys() {
        let mut blank = candidate();
        blank.rationale = "   ".to_string();
        assert!(candidates("candidates", &[blank]).is_err());

        let mut unsafe_key = candidate();
        unsafe_key.policy_key = "../escape".to_string();
        assert!(candidates("candidates", &[unsafe_key]).is_err());

        let mut unsafe_control = candidate();
        unsafe_control.control_id = "ac 2".to_string();
        assert!(candidates("candidates", &[unsafe_control]).is_err());
    }

    #[test]
    fn subjects_require_canonical_keys_and_bounded_text() {
        let subject = MappingSubject {
            policy_key: "access-policy".to_string(),
            topic_key: "access-control".to_string(),
            title: "Access control".to_string(),
            text: "Accounts must be reviewed quarterly.".to_string(),
            control_ids: vec!["ac-2".to_string()],
            gap_ids: vec!["gap-0001".to_string()],
        };
        assert!(subjects("subjects", std::slice::from_ref(&subject)).is_ok());

        let mut duplicated = subject.clone();
        duplicated.control_ids = vec!["ac-2".to_string(), "ac-2".to_string()];
        assert!(subjects("subjects", &[duplicated]).is_err());

        let mut empty_text = subject;
        empty_text.text = "\n".to_string();
        assert!(subjects("subjects", &[empty_text]).is_err());
    }

    #[test]
    fn mapping_task_schema_file_is_published_and_closed() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../../schemas/forge.suggest-task-mapping-1.schema.json"
        ))
        .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-task-mapping/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
