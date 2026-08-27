//! Durable, report-bound review dispositions for framework-impact findings.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ForgeError;

pub const DISPOSITION_SCHEMA_VERSION: &str = "forge.framework-impact-dispositions/1";
pub const MAX_DISPOSITION_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_PRIOR_REPORT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_DISPOSITIONS: usize = 100_000;
const MAX_STRING_BYTES: usize = 64 * 1024;
const MAX_JSON_DEPTH: usize = 64;
const STRICT_JSON_LIMITS: crate::json_strict::Limits =
    crate::json_strict::Limits { max_depth: MAX_JSON_DEPTH, max_string_bytes: MAX_STRING_BYTES };

/// Parsed disposition records for one exact prior report.
///
/// `prior_report_sha256` is format-validated here only; callers MUST recompute and compare it
/// with the actual prior report before applying dispositions.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionFile {
    pub schema_version: String,
    pub prior_report_sha256: String,
    pub dispositions: Vec<DispositionRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionRecord {
    pub finding_id: String,
    pub status: DispositionStatus,
    pub decided_by: String,
    pub decided_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DispositionStatus {
    Resolved,
    AcceptedRisk,
    StillOpen,
}

impl DispositionStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::AcceptedRisk => "accepted-risk",
            Self::StillOpen => "still-open",
        }
    }
}

/// Read and validate a local disposition file.
///
/// Returned dispositions are sorted ascending by `finding_id` and their `decided_at` values are
/// normalized to UTC RFC 3339 seconds.
///
/// # Errors
///
/// Returns [`ForgeError::FrameworkImpact`] for unsafe files or invalid contracts.
pub fn load(path: &Path) -> Result<DispositionFile, ForgeError> {
    let bytes = crate::io::read_bounded(path, MAX_DISPOSITION_BYTES).map_err(|cause| {
        error(format!("cannot read disposition file '{}': {cause}", path.display()))
    })?;
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<DispositionFile, ForgeError> {
    if bytes.len() as u64 > MAX_DISPOSITION_BYTES {
        return Err(error(format!(
            "disposition file exceeds the {MAX_DISPOSITION_BYTES} byte limit"
        )));
    }
    let value = parse_strict_value(bytes, "disposition")?;
    let mut file: DispositionFile = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid disposition contract: {cause}")))?;
    validate_and_normalize(&mut file)?;
    Ok(file)
}

pub(crate) fn parse_strict_value(bytes: &[u8], label: &str) -> Result<Value, ForgeError> {
    crate::json_strict::parse_value(bytes, label, STRICT_JSON_LIMITS)
        .map_err(|cause| error(cause.to_string()))
}

/// Validate disposition contracts and normalize the result by ascending `finding_id`.
fn validate_and_normalize(file: &mut DispositionFile) -> Result<(), ForgeError> {
    if file.schema_version != DISPOSITION_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported disposition schema_version '{}'; expected {DISPOSITION_SCHEMA_VERSION}",
            crate::json_strict::bounded(&file.schema_version)
        )));
    }
    validate_sha256("$.prior_report_sha256", &file.prior_report_sha256)?;
    if file.dispositions.is_empty() || file.dispositions.len() > MAX_DISPOSITIONS {
        return Err(error(format!(
            "$.dispositions must contain between 1 and {MAX_DISPOSITIONS} records"
        )));
    }
    let mut finding_ids = BTreeSet::new();
    for (index, disposition) in file.dispositions.iter_mut().enumerate() {
        let path = format!("$.dispositions[{index}]");
        uuid::Uuid::parse_str(&disposition.finding_id)
            .map_err(|_| error(format!("{path}.finding_id must be a UUID")))?;
        if !finding_ids.insert(disposition.finding_id.as_str()) {
            return Err(error(format!("{path}.finding_id duplicates another disposition")));
        }
        validate_nonempty(&format!("{path}.decided_by"), &disposition.decided_by)?;
        validate_nonempty(&format!("{path}.decided_at"), &disposition.decided_at)?;
        validate_nonempty(&format!("{path}.rationale"), &disposition.rationale)?;
        let decided_at = chrono::DateTime::parse_from_rfc3339(&disposition.decided_at)
            .map_err(|_| error(format!("{path}.decided_at must be an RFC 3339 timestamp")))?;
        disposition.decided_at =
            decided_at.to_utc().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    }
    file.dispositions.sort_unstable_by(|left, right| left.finding_id.cmp(&right.finding_id));
    Ok(())
}

fn validate_sha256(path: &str, value: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256(path, value).map_err(error)
}

fn validate_nonempty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() || value.len() > MAX_STRING_BYTES {
        return Err(error(format!("{path} must contain between 1 and {MAX_STRING_BYTES} bytes")));
    }
    Ok(())
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::FrameworkImpact(message.into())
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        DISPOSITION_SCHEMA_VERSION, DispositionFile, DispositionRecord, DispositionStatus,
        MAX_DISPOSITION_BYTES, MAX_DISPOSITIONS, MAX_STRING_BYTES, parse, validate_and_normalize,
    };
    const FIRST_ID: &str = "00000000-0000-4000-8000-000000000001";
    const SECOND_ID: &str = "00000000-0000-4000-8000-000000000002";

    fn valid_value() -> Value {
        json!({
            "schema_version": DISPOSITION_SCHEMA_VERSION,
            "prior_report_sha256": "0".repeat(64),
            "dispositions": [{
                "finding_id": FIRST_ID,
                "status": "resolved",
                "decided_by": "reviewer",
                "decided_at": "2026-08-25T12:00:00Z",
                "rationale": "Reviewed finding."
            }]
        })
    }

    fn parse_error(value: &Value) -> String {
        parse(&serde_json::to_vec(value).expect("serialize disposition fixture"))
            .unwrap_err()
            .to_string()
    }

    fn valid_record(finding_id: &str) -> DispositionRecord {
        DispositionRecord {
            finding_id: finding_id.to_string(),
            status: DispositionStatus::Resolved,
            decided_by: "reviewer".to_string(),
            decided_at: "2026-08-25T12:00:00Z".to_string(),
            rationale: "Reviewed finding.".to_string(),
        }
    }

    fn valid_file() -> DispositionFile {
        DispositionFile {
            schema_version: DISPOSITION_SCHEMA_VERSION.to_string(),
            prior_report_sha256: "0".repeat(64),
            dispositions: vec![valid_record(FIRST_ID)],
        }
    }

    #[test]
    fn parser_rejects_oversized_disposition_bytes() {
        let bytes = vec![
            b' ';
            usize::try_from(MAX_DISPOSITION_BYTES)
                .expect("disposition byte limit fits usize")
                + 1
        ];
        let error = parse(&bytes).expect_err("oversized disposition must fail");
        assert!(error.to_string().contains("disposition file exceeds"));
    }

    #[test]
    fn duplicate_keys_are_rejected_before_contract_validation() {
        let error = parse(
            br#"{"schema_version":"forge.framework-impact-dispositions/1","schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":"0000000000000000000000000000000000000000000000000000000000000000","dispositions":[]}"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate object key"));
    }

    #[test]
    fn contract_rejects_unknown_fields_schema_hash_status_and_empty_records() {
        let mut unknown = valid_value();
        unknown["unexpected"] = json!(true);
        assert!(parse_error(&unknown).contains("unknown field"));

        let mut nested_unknown = valid_value();
        nested_unknown["dispositions"][0]["unexpected"] = json!(true);
        assert!(parse_error(&nested_unknown).contains("unknown field"));

        let mut schema = valid_value();
        schema["schema_version"] = json!("forge.framework-impact-dispositions/2");
        assert!(parse_error(&schema).contains("unsupported disposition schema_version"));

        for hash in ["0".repeat(63), "A".repeat(64), "g".repeat(64)] {
            let mut invalid = valid_value();
            invalid["prior_report_sha256"] = json!(hash);
            assert!(parse_error(&invalid).contains("64 lowercase hexadecimal characters"));
        }

        let mut status = valid_value();
        status["dispositions"][0]["status"] = json!("waived");
        assert!(parse_error(&status).contains("unknown variant"));

        let mut empty = valid_value();
        empty["dispositions"] = json!([]);
        assert!(parse_error(&empty).contains("must contain between 1"));
    }

    #[test]
    fn record_validation_rejects_invalid_ids_duplicates_evidence_and_timestamps() {
        let mut invalid_id = valid_value();
        invalid_id["dispositions"][0]["finding_id"] = json!("not-a-uuid");
        assert!(parse_error(&invalid_id).contains("finding_id must be a UUID"));

        let mut duplicate = valid_value();
        let duplicate_record = duplicate["dispositions"][0].clone();
        duplicate["dispositions"].as_array_mut().expect("disposition array").push(duplicate_record);
        assert!(parse_error(&duplicate).contains("duplicates another disposition"));

        for field in ["decided_by", "decided_at", "rationale"] {
            let mut empty = valid_value();
            empty["dispositions"][0][field] = json!(" \t ");
            assert!(
                parse_error(&empty)
                    .contains(&format!("$.dispositions[0].{field} must contain between 1"))
            );
        }

        let mut invalid_time = valid_value();
        invalid_time["dispositions"][0]["decided_at"] = json!("not-a-time");
        assert!(parse_error(&invalid_time).contains("must be an RFC 3339 timestamp"));

        let mut offset_time = valid_value();
        offset_time["dispositions"][0]["decided_at"] = json!("2026-08-25T12:34:56+02:00");
        let normalized =
            parse(&serde_json::to_vec(&offset_time).expect("serialize offset disposition fixture"))
                .expect("valid offset timestamp");
        assert_eq!(normalized.dispositions[0].decided_at, "2026-08-25T10:34:56Z");

        let mut oversized = valid_file();
        oversized.dispositions[0].rationale = "x".repeat(MAX_STRING_BYTES + 1);
        assert!(
            validate_and_normalize(&mut oversized)
                .unwrap_err()
                .to_string()
                .contains("must contain between 1")
        );
    }

    #[test]
    fn record_count_is_bounded_and_valid_records_sort_by_finding_id() {
        let record = valid_record(FIRST_ID);
        let mut too_many = valid_file();
        too_many.dispositions = vec![record; MAX_DISPOSITIONS + 1];
        assert!(
            validate_and_normalize(&mut too_many)
                .unwrap_err()
                .to_string()
                .contains("must contain between 1")
        );

        let mut sortable = valid_file();
        sortable.dispositions = vec![valid_record(SECOND_ID), valid_record(FIRST_ID)];
        validate_and_normalize(&mut sortable).expect("valid disposition records");
        assert_eq!(sortable.dispositions[0].finding_id, FIRST_ID);
        assert_eq!(sortable.dispositions[1].finding_id, SECOND_ID);
    }
}
