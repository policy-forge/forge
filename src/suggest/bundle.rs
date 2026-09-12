//! Closed, bounded `forge.suggestions/1`: the quarantine bundle.
//!
//! A bundle is the only artifact class a suggestion can live in. It carries
//! validated model output, the provenance of the run that produced it, and the
//! request/response digests that bind it to exactly one payload. It never
//! carries a status: a suggestion with no disposition record is pending, and
//! every other state lives in `forge.suggest-dispositions/1`.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::request::{MAX_ARG_BYTES, MAX_ARGV, MAX_MODEL_ID_BYTES, MAX_REDACTIONS};
use super::response::MAX_RESPONSE_BYTES;
use super::shared;
use super::task::{self, SuggestionBody, TaskIdentity};

/// Closed bundle contract version.
pub const SCHEMA_VERSION: &str = "forge.suggestions/1";
/// Maximum encoded bundle size, matching the shared authoring output bound.
pub const MAX_BUNDLE_BYTES: u64 = 50 * 1024 * 1024;

const LIMITS: Limits = Limits { max_depth: 32, max_string_bytes: shared::MAX_STRING_BYTES };

/// The prepared request a bundle belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestRef {
    /// Digest of the request document.
    pub sha256: String,
    /// Digest of the payload the request named.
    pub payload_sha256: String,
}

/// The adapter response a bundle was validated from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseRef {
    /// Digest of the raw adapter output.
    pub sha256: String,
    /// Raw adapter output length in bytes.
    pub bytes: u64,
    /// Whether the raw output was retained beside the bundle.
    pub retained: bool,
    /// Portable relative path of the retained raw output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
}

/// Measured and supplied provenance for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Digest of the adapter executable that ran.
    pub adapter_sha256: String,
    /// Operator-supplied model identifier, recorded as supplied.
    pub model_id: String,
    /// Arguments the adapter was invoked with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv: Vec<String>,
    /// Measured adapter wall time in milliseconds.
    pub elapsed_ms: u64,
    /// Adapter process exit code.
    pub exit_code: i32,
    /// Redactions applied before the payload was written.
    #[serde(default)]
    pub redactions: Vec<super::request::RedactionRecord>,
}

/// Deterministic evidence rating, computed from citations and never self-reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceSupport {
    /// Every citation resolves verbatim to a selected source span.
    High,
    /// Every citation resolves, but at least one cites supplied metadata only.
    Medium,
    /// Every citation resolves, but none cites a selected source span.
    Low,
}

impl EvidenceSupport {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// One quarantined suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suggestion {
    /// Deterministic identifier derived from the suggestion content.
    pub suggestion_id: String,
    /// Rating computed by the deterministic citation check.
    pub evidence_support: EvidenceSupport,
    /// The validated task payload.
    pub body: SuggestionBody,
}

/// Recomputed suggestion totals, so a bundle cannot overstate its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestionCounts {
    /// Total suggestions.
    pub suggestions: u64,
    /// Mapping-candidate suggestions.
    pub mapping: u64,
    /// Draft-clause suggestions.
    pub drafting: u64,
    /// Suggestions rated high.
    pub high: u64,
    /// Suggestions rated medium.
    pub medium: u64,
    /// Suggestions rated low.
    pub low: u64,
}

/// One validated quarantine bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestionsBundle {
    /// Closed contract version.
    pub schema_version: String,
    /// Deterministic identifier derived from the request and response digests.
    pub bundle_id: String,
    /// Project key the context was selected from.
    pub project_key: String,
    /// Supplied project timestamp; never a wall clock.
    pub as_of: String,
    /// The task this bundle answers.
    pub task: TaskIdentity,
    /// Request and payload digests.
    pub request: RequestRef,
    /// Response digest and retention.
    pub response: ResponseRef,
    /// Run provenance.
    pub provenance: Provenance,
    /// Quarantined suggestions.
    pub suggestions: Vec<Suggestion>,
    /// Recomputed totals.
    pub counts: SuggestionCounts,
}

impl SuggestionsBundle {
    /// Parse and validate a bounded, closed bundle.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, a payload that does not match the
    /// declared task, a withheld raw response without an artifact (or the
    /// reverse), or counts that do not match the suggestions present.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_BUNDLE_BYTES {
            return Err(shared::error(format!(
                "suggestions bundle exceeds the {MAX_BUNDLE_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "suggestions bundle", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "suggestions bundle")?;
        let bundle: Self = serde_json::from_value(value).map_err(|cause| {
            shared::error(format!("invalid suggestions bundle contract: {cause}"))
        })?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Quarantined suggestions in bundle order.
    #[must_use]
    pub fn suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "suggestions bundle schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::uuid("bundle.bundle_id", &self.bundle_id)?;
        shared::key("bundle.project_key", &self.project_key)?;
        shared::single_line("bundle.as_of", &self.as_of)?;
        self.task.validate("bundle.task")?;
        shared::sha256("bundle.request.sha256", &self.request.sha256)?;
        shared::sha256("bundle.request.payload_sha256", &self.request.payload_sha256)?;
        shared::sha256("bundle.response.sha256", &self.response.sha256)?;
        if self.response.bytes == 0 || self.response.bytes > MAX_RESPONSE_BYTES {
            return Err(shared::error(format!(
                "bundle.response.bytes must be between 1 and {MAX_RESPONSE_BYTES}"
            )));
        }
        match (&self.response.artifact, self.response.retained) {
            (Some(artifact), true) => {
                shared::relative_path("bundle.response.artifact", artifact)?;
            }
            (None, false) => {}
            (Some(_), false) => {
                return Err(shared::error(
                    "bundle.response.artifact must be absent when the raw response is not retained",
                ));
            }
            (None, true) => {
                return Err(shared::error(
                    "bundle.response.artifact is required when the raw response is retained",
                ));
            }
        }
        self.provenance.validate()?;
        if self.suggestions.len() > task::MAX_SUGGESTIONS {
            return Err(shared::error(format!(
                "bundle.suggestions exceeds {} entries",
                task::MAX_SUGGESTIONS
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for (index, suggestion) in self.suggestions.iter().enumerate() {
            let name = format!("bundle.suggestions[{index}]");
            shared::uuid(&format!("{name}.suggestion_id"), &suggestion.suggestion_id)?;
            if !seen.insert(suggestion.suggestion_id.as_str()) {
                return Err(shared::error("bundle.suggestions ids must be unique"));
            }
            suggestion.body.validate(&format!("{name}.body"), self.task.kind)?;
        }
        if self.counts != self.recomputed_counts() {
            return Err(shared::error("bundle.counts must equal the suggestions actually present"));
        }
        Ok(())
    }

    fn recomputed_counts(&self) -> SuggestionCounts {
        let mut counts = SuggestionCounts {
            suggestions: self.suggestions.len() as u64,
            mapping: 0,
            drafting: 0,
            high: 0,
            medium: 0,
            low: 0,
        };
        for suggestion in &self.suggestions {
            if suggestion.body.mapping.is_some() {
                counts.mapping += 1;
            }
            if suggestion.body.drafting.is_some() {
                counts.drafting += 1;
            }
            match suggestion.evidence_support {
                EvidenceSupport::High => counts.high += 1,
                EvidenceSupport::Medium => counts.medium += 1,
                EvidenceSupport::Low => counts.low += 1,
            }
        }
        counts
    }
}

impl Provenance {
    fn validate(&self) -> Result<(), ForgeError> {
        shared::sha256("bundle.provenance.adapter_sha256", &self.adapter_sha256)?;
        if self.model_id.is_empty() || self.model_id.len() > MAX_MODEL_ID_BYTES {
            return Err(shared::error("bundle.provenance.model_id must be a bounded identifier"));
        }
        if self.argv.len() > MAX_ARGV {
            return Err(shared::error(format!(
                "bundle.provenance.argv exceeds {MAX_ARGV} entries"
            )));
        }
        for argument in &self.argv {
            if argument.len() > MAX_ARG_BYTES || argument.chars().any(char::is_control) {
                return Err(shared::error(format!(
                    "bundle.provenance.argv entries must be at most {MAX_ARG_BYTES} bytes"
                )));
            }
        }
        if self.redactions.len() > MAX_REDACTIONS {
            return Err(shared::error(format!(
                "bundle.provenance.redactions exceeds {MAX_REDACTIONS} entries"
            )));
        }
        super::request::redaction_records("bundle.provenance.redactions", &self.redactions)
    }
}

/// A minimal valid bundle, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_bundle_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "bundle_id": "1b2c3d4e-5f60-4718-9a2b-3c4d5e6f7081",
        "project_key": "synthetic-project",
        "as_of": "2026-09-12T00:00:00Z",
        "task": {"kind": "policy-drafting", "schema_version": "forge.suggest-task-drafting/1"},
        "request": {
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "payload_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "response": {
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bytes": 512,
            "retained": false
        },
        "provenance": {
            "adapter_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "model_id": "synthetic-model",
            "argv": ["--task", "draft"],
            "elapsed_ms": 120,
            "exit_code": 0,
            "redactions": []
        },
        "suggestions": [{
            "suggestion_id": "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607",
            "evidence_support": "high",
            "body": {
                "drafting": {
                    "policy_key": "access-policy",
                    "topic_key": "access-control",
                    "question_key": "review-cadence",
                    "heading": "Review cadence",
                    "draft_text": "Accounts must be reviewed every quarter.",
                    "citations": [{"unit_id": "unit-0001"}],
                    "assumptions": [],
                    "unresolved_questions": ["Who approves the review?"]
                }
            }
        }],
        "counts": {
            "suggestions": 1,
            "mapping": 0,
            "drafting": 1,
            "high": 1,
            "medium": 0,
            "low": 0
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parse(value: &Value) -> Result<SuggestionsBundle, ForgeError> {
        SuggestionsBundle::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_bundle_round_trips_and_recomputes_its_counts() {
        let parsed = parse(&fixture_bundle_json()).unwrap();
        assert_eq!(parsed.suggestions().len(), 1);
        assert_eq!(parsed.suggestions()[0].evidence_support, EvidenceSupport::High);
        assert_eq!(parsed.counts, parsed.recomputed_counts());

        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(SuggestionsBundle::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn counts_must_match_the_suggestions_present() {
        let mut inflated = fixture_bundle_json();
        inflated["counts"]["suggestions"] = json!(2);
        assert!(parse(&inflated).is_err());

        let mut mislabelled = fixture_bundle_json();
        mislabelled["counts"]["mapping"] = json!(1);
        mislabelled["counts"]["drafting"] = json!(0);
        assert!(parse(&mislabelled).is_err());

        let mut wrong_rating = fixture_bundle_json();
        wrong_rating["counts"]["high"] = json!(0);
        wrong_rating["counts"]["low"] = json!(1);
        assert!(parse(&wrong_rating).is_err());
    }

    #[test]
    fn body_must_match_the_bundle_task() {
        let mut wrong_arm = fixture_bundle_json();
        wrong_arm["task"]["kind"] = json!("mapping-candidates");
        wrong_arm["task"]["schema_version"] = json!("forge.suggest-task-mapping/1");
        assert!(parse(&wrong_arm).is_err());

        let mut both_arms = fixture_bundle_json();
        both_arms["suggestions"][0]["body"]["mapping"] = json!({
            "policy_key": "access-policy",
            "topic_key": "access-control",
            "control_id": "ac-2",
            "relationship": "intersects-with",
            "rationale": "Overlaps the supplied statement.",
            "citations": [{"unit_id": "unit-0001"}]
        });
        both_arms["counts"]["mapping"] = json!(1);
        both_arms["counts"]["drafting"] = json!(1);
        both_arms["counts"]["suggestions"] = json!(1);
        assert!(parse(&both_arms).is_err());

        let mut empty_body = fixture_bundle_json();
        empty_body["suggestions"][0]["body"] = json!({});
        assert!(parse(&empty_body).is_err());
    }

    #[test]
    fn raw_retention_and_artifact_presence_must_agree() {
        let mut claimed_but_absent = fixture_bundle_json();
        claimed_but_absent["response"]["retained"] = json!(true);
        assert!(parse(&claimed_but_absent).is_err());

        let mut present_but_unretained = fixture_bundle_json();
        present_but_unretained["response"]["artifact"] = json!("response.raw");
        assert!(parse(&present_but_unretained).is_err());

        let mut retained = fixture_bundle_json();
        retained["response"]["retained"] = json!(true);
        retained["response"]["artifact"] = json!("response.raw");
        assert!(parse(&retained).is_ok());
    }

    #[test]
    fn unknown_null_and_duplicate_keys_are_rejected() {
        let mut unknown = fixture_bundle_json();
        unknown["status"] = json!("pending");
        assert!(parse(&unknown).is_err());

        let mut null = fixture_bundle_json();
        null["provenance"]["argv"] = json!(null);
        assert!(parse(&null).is_err());

        let mut duplicate_id = fixture_bundle_json();
        let suggestion = duplicate_id["suggestions"][0].clone();
        duplicate_id["suggestions"] = json!([suggestion.clone(), suggestion]);
        duplicate_id["counts"]["suggestions"] = json!(2);
        duplicate_id["counts"]["drafting"] = json!(2);
        duplicate_id["counts"]["high"] = json!(2);
        assert!(parse(&duplicate_id).is_err());
    }

    #[test]
    fn bundle_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.suggestions-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggestions/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
