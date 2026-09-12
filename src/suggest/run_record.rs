//! Closed, bounded `forge.suggest-run/1`: the receipt for one adapter run.
//!
//! The record is the only bridge between `run` and `validate`: it binds the
//! request, the exact payload, the raw response and the adapter identity, and
//! records what was measured (wall time, exit code) separately from what was
//! supplied (model identifier, arguments, redaction policy).

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::request::{
    MAX_ARG_BYTES, MAX_ARGV, MAX_MODEL_ID_BYTES, MAX_REDACTIONS, RedactionRecord,
};
use super::response::MAX_RESPONSE_BYTES;
use super::shared;

/// Closed run-record contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-run/1";
/// Maximum encoded run-record size.
pub const MAX_RUN_RECORD_BYTES: u64 = 256 * 1024;

const LIMITS: Limits = Limits { max_depth: 16, max_string_bytes: shared::MAX_STRING_BYTES };

/// How the response was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunMode {
    /// A local child process was invoked.
    Process,
    /// An operator-recorded response was replayed; no process ran.
    RecordedResponse,
}

impl RunMode {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Process => "process",
            Self::RecordedResponse => "recorded-response",
        }
    }
}

/// One completed adapter run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    /// Closed contract version.
    pub schema_version: String,
    /// Digest of the request document that authorised this run.
    pub request_sha256: String,
    /// Digest of the exact payload handed to the adapter.
    pub payload_sha256: String,
    /// Digest of the raw response bytes.
    pub response_sha256: String,
    /// Raw response length in bytes.
    pub response_bytes: u64,
    /// Portable relative path of the raw response.
    pub response_artifact: String,
    /// Digest of the adapter executable, as consented.
    pub adapter_executable_sha256: String,
    /// Operator-supplied model identifier, recorded as supplied.
    pub model_id: String,
    /// Arguments the adapter was invoked with.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv: Vec<String>,
    /// Redactions applied before the payload was written.
    #[serde(default)]
    pub redactions: Vec<RedactionRecord>,
    /// Measured adapter wall time in milliseconds.
    pub elapsed_ms: u64,
    /// The adapter's exit code; a record is only written for a successful run.
    pub exit_code: i32,
    /// Whether a process ran.
    pub mode: RunMode,
}

impl RunRecord {
    /// Parse and validate a bounded, closed run record.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, malformed digests, or unsafe paths.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_RUN_RECORD_BYTES {
            return Err(shared::error(format!(
                "suggest run record exceeds the {MAX_RUN_RECORD_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "suggest run record", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "suggest run record")?;
        let record: Self = serde_json::from_value(value)
            .map_err(|cause| shared::error(format!("invalid suggest run record: {cause}")))?;
        record.validate()?;
        Ok(record)
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "suggest run record schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::sha256("run.request_sha256", &self.request_sha256)?;
        shared::sha256("run.payload_sha256", &self.payload_sha256)?;
        shared::sha256("run.response_sha256", &self.response_sha256)?;
        shared::relative_path("run.response_artifact", &self.response_artifact)?;
        shared::sha256("run.adapter_executable_sha256", &self.adapter_executable_sha256)?;
        if self.model_id.is_empty() || self.model_id.len() > MAX_MODEL_ID_BYTES {
            return Err(shared::error("run.model_id must be a bounded identifier"));
        }
        if self.response_bytes == 0 || self.response_bytes > MAX_RESPONSE_BYTES {
            return Err(shared::error(format!(
                "run.response_bytes must be between 1 and {MAX_RESPONSE_BYTES}"
            )));
        }
        if self.argv.len() > MAX_ARGV {
            return Err(shared::error(format!("run.argv exceeds {MAX_ARGV} entries")));
        }
        for argument in &self.argv {
            if argument.len() > MAX_ARG_BYTES || argument.chars().any(char::is_control) {
                return Err(shared::error(format!(
                    "run.argv entries must be at most {MAX_ARG_BYTES} bytes"
                )));
            }
        }
        if self.redactions.len() > MAX_REDACTIONS {
            return Err(shared::error(format!("run.redactions exceeds {MAX_REDACTIONS} entries")));
        }
        super::request::redaction_records("run.redactions", &self.redactions)?;
        if self.exit_code != 0 {
            return Err(shared::error(
                "run.exit_code must be 0: a record is only written for a successful adapter run",
            ));
        }
        Ok(())
    }

    /// Whether this record matches one request, payload and response exactly.
    ///
    /// # Errors
    /// Returns an authoring error naming the first binding that does not match.
    pub fn authorises(
        &self,
        request_sha256: &str,
        request: &super::request::SuggestRequest,
        payload_sha256: &str,
        response: &[u8],
    ) -> Result<(), ForgeError> {
        let bindings = [
            ("request_sha256", &self.request_sha256, request_sha256),
            ("payload_sha256", &self.payload_sha256, payload_sha256),
            ("response_sha256", &self.response_sha256, &crate::hashing::sha256_hex(response)),
            (
                "adapter_executable_sha256",
                &self.adapter_executable_sha256,
                &request.adapter.executable_sha256,
            ),
            ("model_id", &self.model_id, &request.adapter.model_id),
        ];
        for (name, recorded, actual) in bindings {
            if recorded != actual {
                return Err(shared::error(format!(
                    "run record {name} does not match the request, payload or response"
                )));
            }
        }
        if self.response_bytes != response.len() as u64 {
            return Err(shared::error("run record response_bytes does not match the response"));
        }
        Ok(())
    }
}

/// A minimal valid run record, shared by tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_run_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "request_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "payload_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "response_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "response_bytes": 512,
        "response_artifact": "response.raw",
        "adapter_executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "model_id": "synthetic-model",
        "argv": ["--task", "draft"],
        "redactions": [],
        "elapsed_ms": 120,
        "exit_code": 0,
        "mode": "process"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parse(value: &Value) -> Result<RunRecord, ForgeError> {
        RunRecord::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn a_valid_record_round_trips_and_names_its_mode() {
        let parsed = parse(&fixture_run_json()).unwrap();
        assert_eq!(parsed.mode, RunMode::Process);
        assert_eq!(parsed.mode.as_str(), "process");
        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(RunRecord::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn a_failed_exit_code_can_never_be_recorded() {
        let mut failed = fixture_run_json();
        failed["exit_code"] = json!(3);
        assert!(parse(&failed).is_err());
    }

    #[test]
    fn unknown_null_forward_and_unsafe_values_are_rejected() {
        let mut unknown = fixture_run_json();
        unknown["provider"] = json!("local");
        assert!(parse(&unknown).is_err());

        let mut null = fixture_run_json();
        null["mode"] = json!(null);
        assert!(parse(&null).is_err());

        let mut forward = fixture_run_json();
        forward["schema_version"] = json!("forge.suggest-run/2");
        assert!(parse(&forward).is_err());

        let mut escaping = fixture_run_json();
        escaping["response_artifact"] = json!("../response.raw");
        assert!(parse(&escaping).is_err());

        let mut zero = fixture_run_json();
        zero["response_bytes"] = json!(0);
        assert!(parse(&zero).is_err());

        let mut huge = fixture_run_json();
        huge["response_bytes"] = json!(MAX_RESPONSE_BYTES + 1);
        assert!(parse(&huge).is_err());
    }

    #[test]
    fn a_record_authorises_only_its_own_request_payload_and_response() {
        let request = crate::suggest::request::fixture_request();
        let response = b"{\"schema_version\":\"forge.suggest-response/1\"}";
        let mut record = parse(&fixture_run_json()).unwrap();
        record.response_bytes = response.len() as u64;
        record.response_sha256 = crate::hashing::sha256_hex(response);
        record.request_sha256 = "b".repeat(64);
        record.payload_sha256 = "a".repeat(64);
        record.adapter_executable_sha256 = request.adapter.executable_sha256.clone();
        record.model_id = request.adapter.model_id.clone();
        assert!(record.authorises(&"b".repeat(64), &request, &"a".repeat(64), response).is_ok());
        assert!(record.authorises(&"c".repeat(64), &request, &"a".repeat(64), response).is_err());
        assert!(record.authorises(&"b".repeat(64), &request, &"a".repeat(64), b"other").is_err());
    }

    #[test]
    fn a_record_past_its_byte_bound_is_refused_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_RUN_RECORD_BYTES).unwrap() + 1];
        assert!(RunRecord::parse(&oversized).is_err());
    }

    #[test]
    fn run_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.suggest-run-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-run/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
