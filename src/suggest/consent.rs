//! Closed, bounded `forge.suggest-consent/1`: the operator's go-ahead for one
//! exact local payload.
//!
//! Consent is not optional merely because the adapter is local: the operator is
//! still the only party who can decide what leaves their project. The token
//! binds the payload digest, the adapter executable digest, the model
//! identifier and the retention notice, so any change between preview and run
//! invalidates it.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::request::SuggestRequest;
use super::shared;

/// Closed consent contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-consent/1";
/// Maximum encoded consent size.
pub const MAX_CONSENT_BYTES: u64 = 256 * 1024;

const LIMITS: Limits = Limits { max_depth: 8, max_string_bytes: shared::MAX_STRING_BYTES };

/// One operator consent record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsentToken {
    /// Closed contract version.
    pub schema_version: String,
    /// Exact digest of the payload preview the operator approved.
    pub payload_sha256: String,
    /// Exact digest of the adapter executable the operator approved.
    pub adapter_sha256: String,
    /// Operator-supplied model identifier the operator approved.
    pub model_id: String,
    /// Adapter arguments the operator approved, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv: Vec<String>,
    /// Retention notice the operator accepted, recorded verbatim.
    pub retention_notice: String,
    /// Reviewer key of the operator granting consent.
    pub operator_key: String,
    /// Supplied project timestamp; never a wall clock.
    pub as_of: String,
}

impl ConsentToken {
    /// Parse and validate a bounded, closed consent token.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, or malformed digests and keys.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_CONSENT_BYTES {
            return Err(shared::error(format!(
                "suggest consent exceeds the {MAX_CONSENT_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "suggest consent", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "suggest consent")?;
        let token: Self = serde_json::from_value(value)
            .map_err(|cause| shared::error(format!("invalid suggest consent contract: {cause}")))?;
        token.validate()?;
        Ok(token)
    }

    /// Whether this token authorises exactly this request.
    ///
    /// # Errors
    /// Returns an authoring error naming the first binding that does not match.
    pub fn authorises(&self, request: &SuggestRequest) -> Result<(), ForgeError> {
        let bindings = [
            ("payload_sha256", &self.payload_sha256, &request.payload.sha256),
            ("adapter_sha256", &self.adapter_sha256, &request.adapter.executable_sha256),
            ("model_id", &self.model_id, &request.adapter.model_id),
            ("retention_notice", &self.retention_notice, &request.retention_notice),
        ];
        for (name, consented, requested) in bindings {
            if consented != requested {
                return Err(shared::error(format!(
                    "consent {name} does not match the prepared request; re-run prepare and consent again"
                )));
            }
        }
        // Arguments change what the adapter does with the same payload, so they
        // are part of what the operator approved.
        if self.argv != request.adapter.argv {
            return Err(shared::error(
                "consent argv does not match the prepared request; re-run prepare and consent again",
            ));
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "suggest consent schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::sha256("consent.payload_sha256", &self.payload_sha256)?;
        shared::sha256("consent.adapter_sha256", &self.adapter_sha256)?;
        if self.model_id.is_empty() || self.model_id.len() > super::request::MAX_MODEL_ID_BYTES {
            return Err(shared::error(
                "consent.model_id must be a bounded operator-supplied identifier",
            ));
        }
        if self.argv.len() > super::request::MAX_ARGV {
            return Err(shared::error(format!(
                "consent.argv exceeds {} entries",
                super::request::MAX_ARGV
            )));
        }
        for argument in &self.argv {
            if argument.len() > super::request::MAX_ARG_BYTES
                || argument.chars().any(char::is_control)
            {
                return Err(shared::error(format!(
                    "consent.argv entries must be at most {} bytes",
                    super::request::MAX_ARG_BYTES
                )));
            }
        }
        shared::single_line("consent.retention_notice", &self.retention_notice)?;
        shared::key("consent.operator_key", &self.operator_key)?;
        shared::single_line("consent.as_of", &self.as_of)?;
        Ok(())
    }
}

/// A minimal valid consent token, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_consent_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "payload_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "adapter_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "model_id": "synthetic-model",
        "argv": ["--task", "draft"],
        "retention_notice": "Operator retains local adapter output for this run only.",
        "operator_key": "brian-luby",
        "as_of": "2026-09-12T00:00:00Z"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggest::request::fixture_request;
    use serde_json::{Value, json};

    fn parse(value: &Value) -> Result<ConsentToken, ForgeError> {
        ConsentToken::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_token_round_trips_with_every_field() {
        let parsed = parse(&fixture_consent_json()).unwrap();
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
        assert_eq!(parsed.operator_key, "brian-luby");
        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(ConsentToken::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn token_authorises_only_the_exact_prepared_request() {
        let request = fixture_request();
        assert!(parse(&fixture_consent_json()).unwrap().authorises(&request).is_ok());

        let mut wrong_payload = fixture_consent_json();
        wrong_payload["payload_sha256"] =
            json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
        assert!(parse(&wrong_payload).unwrap().authorises(&request).is_err());

        let mut wrong_model = fixture_consent_json();
        wrong_model["model_id"] = json!("other-model");
        assert!(parse(&wrong_model).unwrap().authorises(&request).is_err());

        let mut wrong_retention = fixture_consent_json();
        wrong_retention["retention_notice"] = json!("Keep everything.");
        assert!(parse(&wrong_retention).unwrap().authorises(&request).is_err());
    }

    #[test]
    fn token_rejects_unknown_null_and_unsupported_values() {
        let mut unknown = fixture_consent_json();
        unknown["provider"] = json!("local");
        assert!(parse(&unknown).is_err());

        let mut null = fixture_consent_json();
        null["as_of"] = json!(null);
        assert!(parse(&null).is_err());

        let mut forward = fixture_consent_json();
        forward["schema_version"] = json!("forge.suggest-consent/2");
        assert!(parse(&forward).is_err());

        let mut bad_operator = fixture_consent_json();
        bad_operator["operator_key"] = json!("Brian Luby");
        assert!(parse(&bad_operator).is_err());
    }

    #[test]
    fn consent_past_its_byte_bound_is_refused_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_CONSENT_BYTES).unwrap() + 1];
        assert!(ConsentToken::parse(&oversized).is_err());
    }

    #[test]
    fn arguments_the_operator_did_not_approve_are_refused() {
        let request = fixture_request();
        assert!(parse(&fixture_consent_json()).unwrap().authorises(&request).is_ok());

        let mut swapped = fixture_consent_json();
        swapped["argv"] = json!(["--task", "draft", "--extra"]);
        assert!(parse(&swapped).unwrap().authorises(&request).is_err());

        let mut missing = fixture_consent_json();
        missing.as_object_mut().unwrap().remove("argv");
        assert!(parse(&missing).unwrap().authorises(&request).is_err());

        let mut unbounded = fixture_consent_json();
        unbounded["argv"] = json!(["x".repeat(1025)]);
        assert!(parse(&unbounded).is_err());
    }

    #[test]
    fn consent_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.suggest-consent-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-consent/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
