//! Closed, bounded `forge.suggest-dispositions/1`: the human review record.
//!
//! A suggestion with no record here is pending; every other state is an
//! explicit reviewer decision with a key, a supplied time and a rationale.
//! Editing preserves both the original digest and the edited content, so the
//! original is never lost.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::shared;
use super::task::{SuggestionBody, TaskIdentity, TaskKind};

/// Closed disposition contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-dispositions/1";
/// Maximum encoded disposition manifest size.
pub const MAX_DISPOSITIONS_BYTES: u64 = 2 * 1024 * 1024;
/// Maximum disposition records in one manifest.
pub const MAX_RECORDS: usize = 1_000;

const LIMITS: Limits = Limits { max_depth: 32, max_string_bytes: shared::MAX_STRING_BYTES };

/// One reviewer decision about one quarantined suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRecord {
    /// Identifier of the suggestion this decision is about.
    pub suggestion_id: String,
    /// The decision.
    pub status: DispositionStatus,
    /// Reviewer key of the person who decided.
    pub reviewer_key: String,
    /// Supplied decision timestamp; never a wall clock.
    pub decided_as_of: String,
    /// Why the reviewer decided this.
    pub rationale: String,
    /// Digest of the original quarantine content.
    pub original_sha256: String,
    /// Reviewer-edited content, required for an edited acceptance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited: Option<SuggestionBody>,
    /// Digest of the reviewer-edited content, required for an edited acceptance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_sha256: Option<String>,
}

/// What a reviewer decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DispositionStatus {
    /// Accepted without change.
    AcceptAsIs,
    /// Accepted after reviewer edits.
    AcceptEdited,
    /// Rejected; the suggestion is retained for the record.
    Reject,
    /// No longer applicable as of the supplied time.
    Expired,
}

impl DispositionStatus {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AcceptAsIs => "accept-as-is",
            Self::AcceptEdited => "accept-edited",
            Self::Reject => "reject",
            Self::Expired => "expired",
        }
    }

    /// Whether this decision authorises a promotion.
    #[must_use]
    pub const fn promotes(self) -> bool {
        matches!(self, Self::AcceptAsIs | Self::AcceptEdited)
    }
}

/// One validated disposition manifest for one bundle digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionManifest {
    /// Closed contract version.
    pub schema_version: String,
    /// Identifier of the bundle this manifest decides.
    pub bundle_id: String,
    /// Digest of the bundle document this manifest decides.
    pub bundle_sha256: String,
    /// The task the bundle answers.
    pub task: TaskIdentity,
    /// Supplied review timestamp; never a wall clock.
    pub as_of: String,
    /// One record per decided suggestion.
    pub records: Vec<DispositionRecord>,
}

impl DispositionManifest {
    /// Parse and validate a bounded, closed disposition manifest.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, a record that repeats a suggestion,
    /// or an edit that does not agree with its status.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_DISPOSITIONS_BYTES {
            return Err(shared::error(format!(
                "disposition manifest exceeds the {MAX_DISPOSITIONS_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "disposition manifest", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "disposition manifest")?;
        let manifest: Self = serde_json::from_value(value).map_err(|cause| {
            shared::error(format!("invalid disposition manifest contract: {cause}"))
        })?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Reviewer decisions in manifest order.
    #[must_use]
    pub fn records(&self) -> &[DispositionRecord] {
        &self.records
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "disposition manifest schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::uuid("dispositions.bundle_id", &self.bundle_id)?;
        shared::sha256("dispositions.bundle_sha256", &self.bundle_sha256)?;
        self.task.validate("dispositions.task")?;
        shared::single_line("dispositions.as_of", &self.as_of)?;
        if self.records.len() > MAX_RECORDS {
            return Err(shared::error(format!(
                "dispositions.records exceeds {MAX_RECORDS} entries"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        for (index, record) in self.records.iter().enumerate() {
            record.validate(&format!("dispositions.records[{index}]"), self.task.kind)?;
            if !seen.insert(record.suggestion_id.as_str()) {
                return Err(shared::error(
                    "dispositions.records must decide each suggestion at most once",
                ));
            }
        }
        Ok(())
    }

    /// Records that authorise a promotion, in manifest order.
    #[must_use]
    pub fn promotable(&self) -> Vec<&DispositionRecord> {
        self.records.iter().filter(|record| record.status.promotes()).collect()
    }
}

impl DispositionRecord {
    fn validate(&self, name: &str, kind: TaskKind) -> Result<(), ForgeError> {
        shared::uuid(&format!("{name}.suggestion_id"), &self.suggestion_id)?;
        shared::key(&format!("{name}.reviewer_key"), &self.reviewer_key)?;
        shared::single_line(&format!("{name}.decided_as_of"), &self.decided_as_of)?;
        shared::text(&format!("{name}.rationale"), &self.rationale)?;
        shared::sha256(&format!("{name}.original_sha256"), &self.original_sha256)?;
        match (self.status, &self.edited, &self.edited_sha256) {
            (DispositionStatus::AcceptEdited, Some(edited), Some(edited_sha256)) => {
                shared::sha256(&format!("{name}.edited_sha256"), edited_sha256)?;
                edited.validate(&format!("{name}.edited"), kind)
            }
            (DispositionStatus::AcceptEdited, None, _) => {
                Err(shared::error(format!("{name}.edited is required for an edited acceptance")))
            }
            (DispositionStatus::AcceptEdited, _, None) => Err(shared::error(format!(
                "{name}.edited_sha256 is required for an edited acceptance"
            ))),
            (_, Some(_), _) | (_, _, Some(_)) => Err(shared::error(format!(
                "{name} must not carry edited content for status {}",
                self.status.as_str()
            ))),
            (_, None, None) => Ok(()),
        }
    }
}

/// A minimal valid disposition manifest, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_dispositions_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "bundle_id": "1b2c3d4e-5f60-4718-9a2b-3c4d5e6f7081",
        "bundle_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "task": {"kind": "policy-drafting", "schema_version": "forge.suggest-task-drafting/1"},
        "as_of": "2026-09-12T00:00:00Z",
        "records": [{
            "suggestion_id": "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607",
            "status": "accept-as-is",
            "reviewer_key": "brian-luby",
            "decided_as_of": "2026-09-12T00:00:00Z",
            "rationale": "The clause matches the supplied policy text.",
            "original_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn parse(value: &Value) -> Result<DispositionManifest, ForgeError> {
        DispositionManifest::parse(&serde_json::to_vec(value).unwrap())
    }

    fn edited_body() -> Value {
        json!({
            "drafting": {
                "policy_key": "access-policy",
                "topic_key": "access-control",
                "draft_text": "Accounts must be reviewed every quarter by the account owner.",
                "citations": [{"unit_id": "unit-0001"}],
                "assumptions": [],
                "unresolved_questions": []
            }
        })
    }

    #[test]
    fn valid_manifest_round_trips_and_reports_promotable_records() {
        let parsed = parse(&fixture_dispositions_json()).unwrap();
        assert_eq!(parsed.records().len(), 1);
        assert_eq!(parsed.promotable().len(), 1);
        assert_eq!(parsed.records()[0].status.as_str(), "accept-as-is");
        assert!(parsed.records()[0].status.promotes());
        assert!(!DispositionStatus::Reject.promotes());
        assert!(!DispositionStatus::Expired.promotes());

        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(DispositionManifest::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn edited_acceptance_requires_content_and_its_digest() {
        let mut edited = fixture_dispositions_json();
        edited["records"][0]["status"] = json!("accept-edited");
        edited["records"][0]["edited"] = edited_body();
        edited["records"][0]["edited_sha256"] = json!(DIGEST);
        assert!(parse(&edited).is_ok());

        let mut missing_content = fixture_dispositions_json();
        missing_content["records"][0]["status"] = json!("accept-edited");
        missing_content["records"][0]["edited_sha256"] = json!(DIGEST);
        assert!(parse(&missing_content).is_err());

        let mut missing_digest = fixture_dispositions_json();
        missing_digest["records"][0]["status"] = json!("accept-edited");
        missing_digest["records"][0]["edited"] = edited_body();
        assert!(parse(&missing_digest).is_err());
    }

    #[test]
    fn non_editing_statuses_must_not_carry_edited_content() {
        for status in ["accept-as-is", "reject", "expired"] {
            let mut record = fixture_dispositions_json();
            record["records"][0]["status"] = json!(status);
            record["records"][0]["edited"] = edited_body();
            record["records"][0]["edited_sha256"] = json!(DIGEST);
            assert!(parse(&record).is_err(), "{status}");
        }
    }

    #[test]
    fn records_need_a_rationale_and_may_not_repeat_a_suggestion() {
        let mut blank = fixture_dispositions_json();
        blank["records"][0]["rationale"] = json!("   ");
        assert!(parse(&blank).is_err());

        let mut repeated = fixture_dispositions_json();
        let record = repeated["records"][0].clone();
        repeated["records"] = json!([record.clone(), record]);
        assert!(parse(&repeated).is_err());

        let mut unknown = fixture_dispositions_json();
        unknown["records"][0]["accepted"] = json!(true);
        assert!(parse(&unknown).is_err());

        let mut null = fixture_dispositions_json();
        null["records"][0]["edited"] = json!(null);
        assert!(parse(&null).is_err());
    }

    #[test]
    fn task_version_and_supplied_time_are_validated() {
        let mut mismatch = fixture_dispositions_json();
        mismatch["task"]["schema_version"] = json!("forge.suggest-task-drafting/2");
        assert!(parse(&mismatch).is_err());

        let mut blank_time = fixture_dispositions_json();
        blank_time["records"][0]["decided_as_of"] = json!("");
        assert!(parse(&blank_time).is_err());
    }

    #[test]
    fn a_manifest_past_its_byte_bound_is_refused_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_DISPOSITIONS_BYTES).unwrap() + 1];
        assert!(DispositionManifest::parse(&oversized).is_err());
    }

    #[test]
    fn disposition_schema_file_is_published_and_closed() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/forge.suggest-dispositions-1.schema.json"
        ))
        .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-dispositions/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
