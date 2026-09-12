//! Closed, bounded `forge.suggest-promotion/1`: a proposed downstream patch.
//!
//! A promotion is a proposal, never an approval. Each entry names the exact
//! proposed artifact, its digest, and the destination contract validator that
//! accepted it in-process. The proposal records its own unapproved status so no
//! reader can mistake it for an applied change.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::shared;

/// Closed promotion contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-promotion/1";
/// Maximum encoded promotion proposal size.
pub const MAX_PROMOTION_BYTES: u64 = 2 * 1024 * 1024;
/// Maximum promoted entries in one proposal.
pub const MAX_ENTRIES: usize = 1_000;
/// Maximum size of one proposed downstream record.
pub const MAX_ENTRY_BYTES: u64 = 1024 * 1024;
/// Maximum size of the proposed destination document.
pub const MAX_PATCH_BYTES: u64 = 2 * 1024 * 1024;

const LIMITS: Limits = Limits { max_depth: 32, max_string_bytes: shared::MAX_STRING_BYTES };

/// The downstream artifact a proposal targets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    /// Which downstream contract the proposal targets.
    pub kind: DestinationKind,
    /// Portable relative path of the destination artifact.
    pub path: String,
    /// Digest of the destination artifact as it was read.
    pub expected_sha256: String,
}

/// The downstream contracts a proposal may target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DestinationKind {
    /// A PRD-055 control mapping manifest.
    MappingManifest,
    /// A PRD-061 authoring project, which owns human clauses.
    AuthoringProject,
}

impl DestinationKind {
    /// The one contract version this destination is validated with.
    #[must_use]
    pub const fn contract(self) -> &'static str {
        match self {
            Self::MappingManifest => crate::mapping::manifest::MANIFEST_SCHEMA_VERSION,
            Self::AuthoringProject => crate::authoring::manifest::PROJECT_SCHEMA_VERSION,
        }
    }
}

/// Promotion is always a proposal; this enum names that state explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PromotionStatus {
    /// Proposed and unapproved; the destination is unchanged.
    ProposedUnapproved,
}

impl PromotionStatus {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProposedUnapproved => "proposed-unapproved",
        }
    }
}

/// The proposed destination document that was validated before publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchArtifact {
    /// Portable relative path of the proposed destination document.
    pub artifact: String,
    /// Digest of the proposed destination document.
    pub sha256: String,
    /// Length of the proposed destination document.
    pub bytes: u64,
}

/// One proposed downstream record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionEntry {
    /// Suggestion this entry promotes.
    pub suggestion_id: String,
    /// Destination contract the proposed record was validated against.
    pub contract: String,
    /// Portable relative path of the proposed record artifact.
    pub artifact: String,
    /// Digest of the proposed record bytes.
    pub sha256: String,
    /// Length of the proposed record bytes.
    pub bytes: u64,
}

/// One validated promotion proposal for one bundle and disposition manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionProposal {
    /// Closed contract version.
    pub schema_version: String,
    /// Identifier of the bundle the entries came from.
    pub bundle_id: String,
    /// Digest of the bundle document.
    pub bundle_sha256: String,
    /// Digest of the disposition manifest that authorised the entries.
    pub dispositions_sha256: String,
    /// The downstream artifact the proposal targets.
    pub destination: Destination,
    /// The proposed destination document, validated with the destination's own
    /// contract validator in-process.
    pub patch: PatchArtifact,
    /// Always `proposed-unapproved`.
    pub status: PromotionStatus,
    /// Proposed records, one per promoted suggestion.
    pub entries: Vec<PromotionEntry>,
}

impl PromotionProposal {
    /// Parse and validate a bounded, closed promotion proposal.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, an empty or over-long entry list, a
    /// repeated suggestion, or an entry whose contract is not the destination's.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_PROMOTION_BYTES {
            return Err(shared::error(format!(
                "promotion proposal exceeds the {MAX_PROMOTION_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "promotion proposal", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "promotion proposal")?;
        let proposal: Self = serde_json::from_value(value).map_err(|cause| {
            shared::error(format!("invalid promotion proposal contract: {cause}"))
        })?;
        proposal.validate()?;
        Ok(proposal)
    }

    /// Proposed entries in proposal order.
    #[must_use]
    pub fn entries(&self) -> &[PromotionEntry] {
        &self.entries
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "promotion proposal schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::uuid("promotion.bundle_id", &self.bundle_id)?;
        shared::sha256("promotion.bundle_sha256", &self.bundle_sha256)?;
        shared::sha256("promotion.dispositions_sha256", &self.dispositions_sha256)?;
        shared::relative_path("promotion.destination.path", &self.destination.path)?;
        shared::sha256("promotion.destination.expected_sha256", &self.destination.expected_sha256)?;
        shared::relative_path("promotion.patch.artifact", &self.patch.artifact)?;
        shared::sha256("promotion.patch.sha256", &self.patch.sha256)?;
        if self.patch.bytes == 0 || self.patch.bytes > MAX_PATCH_BYTES {
            return Err(shared::error(format!(
                "promotion.patch.bytes must be between 1 and {MAX_PATCH_BYTES}"
            )));
        }
        if self.entries.is_empty() {
            return Err(shared::error("promotion proposal must promote at least one suggestion"));
        }
        if self.entries.len() > MAX_ENTRIES {
            return Err(shared::error(format!("promotion.entries exceeds {MAX_ENTRIES} entries")));
        }
        let expected_contract = self.destination.kind.contract();
        let mut seen = std::collections::BTreeSet::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let name = format!("promotion.entries[{index}]");
            shared::uuid(&format!("{name}.suggestion_id"), &entry.suggestion_id)?;
            if entry.contract != expected_contract {
                return Err(shared::error(format!(
                    "{name}.contract must be {expected_contract} for this destination"
                )));
            }
            shared::relative_path(&format!("{name}.artifact"), &entry.artifact)?;
            shared::sha256(&format!("{name}.sha256"), &entry.sha256)?;
            if entry.bytes == 0 || entry.bytes > MAX_ENTRY_BYTES {
                return Err(shared::error(format!(
                    "{name}.bytes must be between 1 and {MAX_ENTRY_BYTES}"
                )));
            }
            if !seen.insert(entry.suggestion_id.as_str()) {
                return Err(shared::error(
                    "promotion.entries must promote each suggestion at most once",
                ));
            }
        }
        Ok(())
    }
}

/// A minimal valid promotion proposal, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_promotion_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "bundle_id": "1b2c3d4e-5f60-4718-9a2b-3c4d5e6f7081",
        "bundle_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "dispositions_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "destination": {
            "kind": "authoring-project",
            "path": "project.json",
            "expected_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "patch": {
            "artifact": "promotion/proposed-project.json",
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bytes": 1024
        },
        "status": "proposed-unapproved",
        "entries": [{
            "suggestion_id": "3f2b1a4c-5d6e-4f70-8a91-b2c3d4e5f607",
            "contract": "forge.author-project/1",
            "artifact": "promotion/entry-0001.json",
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bytes": 256
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parse(value: &Value) -> Result<PromotionProposal, ForgeError> {
        PromotionProposal::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_proposal_round_trips_and_stays_unapproved() {
        let parsed = parse(&fixture_promotion_json()).unwrap();
        assert_eq!(parsed.status.as_str(), "proposed-unapproved");
        assert_eq!(parsed.entries().len(), 1);
        assert_eq!(parsed.destination.kind.contract(), "forge.author-project/1");

        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(PromotionProposal::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn promotion_status_cannot_be_anything_but_a_proposal() {
        for status in ["approved", "applied", "pending", "Accepted"] {
            let mut wrong = fixture_promotion_json();
            wrong["status"] = json!(status);
            assert!(parse(&wrong).is_err(), "{status}");
        }
    }

    #[test]
    fn entry_contract_must_match_the_destination() {
        let mut wrong_contract = fixture_promotion_json();
        wrong_contract["entries"][0]["contract"] = json!("forge.mapping-manifest/1");
        assert!(parse(&wrong_contract).is_err());

        let mut mapping_destination = fixture_promotion_json();
        mapping_destination["destination"]["kind"] = json!("mapping-manifest");
        mapping_destination["destination"]["path"] = json!("mapping.json");
        mapping_destination["entries"][0]["contract"] = json!("forge.mapping-manifest/1");
        assert!(parse(&mapping_destination).is_ok());
    }

    #[test]
    fn entries_must_be_present_unique_and_bounded() {
        let mut empty = fixture_promotion_json();
        empty["entries"] = json!([]);
        assert!(parse(&empty).is_err());

        let mut repeated = fixture_promotion_json();
        let entry = repeated["entries"][0].clone();
        repeated["entries"] = json!([entry.clone(), entry]);
        assert!(parse(&repeated).is_err());

        let mut zero_bytes = fixture_promotion_json();
        zero_bytes["entries"][0]["bytes"] = json!(0);
        assert!(parse(&zero_bytes).is_err());

        let mut too_large = fixture_promotion_json();
        too_large["entries"][0]["bytes"] = json!(MAX_ENTRY_BYTES + 1);
        assert!(parse(&too_large).is_err());

        let mut escaping = fixture_promotion_json();
        escaping["entries"][0]["artifact"] = json!("../escape.json");
        assert!(parse(&escaping).is_err());
    }

    #[test]
    fn unknown_null_and_duplicate_keys_are_rejected() {
        let mut unknown = fixture_promotion_json();
        unknown["provider"] = json!("local");
        assert!(parse(&unknown).is_err());

        let mut null = fixture_promotion_json();
        null["destination"]["path"] = json!(null);
        assert!(parse(&null).is_err());

        let duplicate = String::from_utf8(serde_json::to_vec(&fixture_promotion_json()).unwrap())
            .unwrap()
            .replace(
                "\"status\":\"proposed-unapproved\"",
                "\"status\":\"proposed-unapproved\",\"status\":\"proposed-unapproved\"",
            );
        assert!(PromotionProposal::parse(duplicate.as_bytes()).is_err());
    }

    #[test]
    fn a_proposal_past_its_byte_bound_is_refused_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_PROMOTION_BYTES).unwrap() + 1];
        assert!(PromotionProposal::parse(&oversized).is_err());
    }

    #[test]
    fn a_patch_past_its_byte_bound_is_refused() {
        let mut oversized = fixture_promotion_json();
        oversized["patch"]["bytes"] = json!(MAX_PATCH_BYTES + 1);
        assert!(parse(&oversized).is_err());
    }

    #[test]
    fn promotion_schema_file_is_published_and_closed() {
        let schema: Value = serde_json::from_str(include_str!(
            "../../schemas/forge.suggest-promotion-1.schema.json"
        ))
        .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-promotion/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
