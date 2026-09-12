//! Closed, bounded `forge.suggest-response/1`: what a local adapter returned.
//!
//! This is untrusted input. It is decoded strictly, bounded, and rejected whole
//! if it carries unknown fields, tool-call shapes, a task version that does not
//! match its kind, or content of the other task. Nothing here is repaired.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::shared;
use super::task::{self, TaskKind, drafting, mapping};

/// Closed response contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-response/1";
/// Maximum encoded adapter output retained from one run.
pub const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;

const LIMITS: Limits = Limits { max_depth: 32, max_string_bytes: shared::MAX_STRING_BYTES };

/// One adapter response, exactly as the task schema describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestResponse {
    /// Closed contract version.
    pub schema_version: String,
    /// The task the adapter was asked to perform.
    pub task: ResponseTask,
    /// Optional bounded notes from the adapter, recorded but never acted on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// The task-specific payload of one response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseTask {
    /// Task kind the adapter claims to have performed.
    pub kind: TaskKind,
    /// Task schema version; must match the kind.
    pub schema_version: String,
    /// Proposed mapping candidates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mapping_candidates: Vec<mapping::MappingCandidate>,
    /// Proposed draft clauses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub draft_clauses: Vec<drafting::DraftClause>,
}

impl ResponseTask {
    fn validate(&self) -> Result<(), ForgeError> {
        task::check_task_version("response.task", self.kind, &self.schema_version)?;
        match self.kind {
            TaskKind::MappingCandidates => {
                mapping::candidates("response.task.mapping_candidates", &self.mapping_candidates)?;
                if !self.draft_clauses.is_empty() {
                    return Err(shared::error(
                        "response.task.draft_clauses must be empty for a mapping-candidate task",
                    ));
                }
            }
            TaskKind::PolicyDrafting => {
                drafting::clauses("response.task.draft_clauses", &self.draft_clauses)?;
                if !self.mapping_candidates.is_empty() {
                    return Err(shared::error(
                        "response.task.mapping_candidates must be empty for a policy-drafting task",
                    ));
                }
            }
        }
        Ok(())
    }

    /// Number of suggestions the adapter returned for its declared task.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mapping_candidates.len() + self.draft_clauses.len()
    }

    /// Whether the adapter returned no suggestions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl SuggestResponse {
    /// Parse and validate a bounded, closed adapter response.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, a task version that does not match
    /// its kind, or content of the other task.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(shared::error(format!(
                "suggest response exceeds the {MAX_RESPONSE_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "suggest response", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "suggest response")?;
        let response: Self = serde_json::from_value(value).map_err(|cause| {
            shared::error(format!("invalid suggest response contract: {cause}"))
        })?;
        response.validate()?;
        Ok(response)
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "suggest response schema_version must be {SCHEMA_VERSION}"
            )));
        }
        self.task.validate()?;
        shared::text_list("response.notes", &self.notes, task::MAX_SUGGESTIONS)?;
        Ok(())
    }
}

/// A minimal valid drafting response, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_response_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "task": {
            "kind": "policy-drafting",
            "schema_version": drafting::TASK_SCHEMA_VERSION,
            "draft_clauses": [{
                "policy_key": "access-policy",
                "topic_key": "access-control",
                "question_key": "review-cadence",
                "heading": "Review cadence",
                "draft_text": "Accounts must be reviewed every quarter.",
                "citations": [{"unit_id": "unit-0001", "quote": "Accounts must be reviewed every quarter."}],
                "assumptions": ["Quarterly review is the intended cadence."],
                "unresolved_questions": ["Who approves the review?"]
            }]
        },
        "notes": []
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parse(value: &Value) -> Result<SuggestResponse, ForgeError> {
        SuggestResponse::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_response_round_trips_with_every_field() {
        let parsed = parse(&fixture_response_json()).unwrap();
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
        assert_eq!(parsed.task.kind, TaskKind::PolicyDrafting);
        assert_eq!(parsed.task.len(), 1);
        assert!(!parsed.task.is_empty());

        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(SuggestResponse::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn unknown_tool_call_and_oversized_content_are_rejected() {
        let mut tool_call = fixture_response_json();
        tool_call["tool_calls"] = json!([{"name": "read_file", "args": {"path": "/etc/passwd"}}]);
        assert!(parse(&tool_call).is_err());

        let mut nested_tool = fixture_response_json();
        nested_tool["task"]["draft_clauses"][0]["tool_call"] = json!({"name": "exec"});
        assert!(parse(&nested_tool).is_err());

        let mut oversized = fixture_response_json();
        oversized["task"]["draft_clauses"][0]["draft_text"] =
            json!("x".repeat(shared::MAX_STRING_BYTES + 1));
        assert!(parse(&oversized).is_err());

        let raw = vec![b' '; usize::try_from(MAX_RESPONSE_BYTES).unwrap() + 1];
        assert!(SuggestResponse::parse(&raw).is_err());
    }

    #[test]
    fn task_version_and_kind_must_agree() {
        let mut mismatch = fixture_response_json();
        mismatch["task"]["schema_version"] = json!(mapping::TASK_SCHEMA_VERSION);
        assert!(parse(&mismatch).is_err());

        let mut foreign = fixture_response_json();
        foreign["task"]["mapping_candidates"] = json!([{
            "policy_key": "access-policy",
            "topic_key": "access-control",
            "control_id": "ac-2",
            "relationship": "intersects-with",
            "rationale": "Overlaps the supplied statement.",
            "citations": [{"unit_id": "unit-0001"}]
        }]);
        assert!(parse(&foreign).is_err());
    }

    #[test]
    fn empty_and_null_and_duplicate_shapes_are_rejected_or_read_as_absent() {
        let mut empty = fixture_response_json();
        empty["task"]["draft_clauses"] = json!([]);
        // A contract-valid but unusable response; `validate` decides the run.
        assert!(parse(&empty).is_ok());

        let mut null = fixture_response_json();
        null["task"]["notes"] = json!(null);
        assert!(parse(&null).is_err());

        let mut notes_null = fixture_response_json();
        notes_null["notes"] = json!(null);
        assert!(parse(&notes_null).is_err());

        let duplicate = String::from_utf8(serde_json::to_vec(&fixture_response_json()).unwrap())
            .unwrap()
            .replace("\"notes\":[]", "\"notes\":[],\"notes\":[]");
        assert!(SuggestResponse::parse(duplicate.as_bytes()).is_err());
    }

    #[test]
    fn a_response_without_citations_is_refused_by_the_contract() {
        let mut uncited = fixture_response_json();
        uncited["task"]["draft_clauses"][0]["citations"] = json!([]);
        assert!(parse(&uncited).is_err());

        let mut undeclared = fixture_response_json();
        undeclared["task"]["draft_clauses"][0]
            .as_object_mut()
            .unwrap()
            .remove("unresolved_questions");
        assert!(parse(&undeclared).is_err());
    }

    #[test]
    fn response_schema_file_is_published_and_closed() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/forge.suggest-response-1.schema.json"
        ))
        .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-response/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
