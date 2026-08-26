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
/// # Errors
///
/// Returns [`ForgeError::FrameworkImpact`] for unsafe files or invalid contracts.
pub fn load(path: &Path) -> Result<DispositionFile, ForgeError> {
    let metadata = crate::io::regular_file_metadata(path, "disposition file").map_err(error)?;
    if metadata.len() > MAX_DISPOSITION_BYTES {
        return Err(error(format!(
            "disposition file exceeds the {MAX_DISPOSITION_BYTES} byte limit"
        )));
    }
    let bytes = std::fs::read(path).map_err(|cause| {
        error(format!("cannot read disposition file '{}': {cause}", path.display()))
    })?;
    parse(&bytes)
}

fn parse(bytes: &[u8]) -> Result<DispositionFile, ForgeError> {
    let value = parse_strict_value(bytes, "disposition")?;
    let mut file: DispositionFile = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid disposition contract: {cause}")))?;
    validate(&mut file)?;
    Ok(file)
}

pub(crate) fn parse_strict_value(bytes: &[u8], label: &str) -> Result<Value, ForgeError> {
    crate::json_strict::parse_value(bytes, label, STRICT_JSON_LIMITS).map_err(error)
}

fn validate(file: &mut DispositionFile) -> Result<(), ForgeError> {
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
    for (index, disposition) in file.dispositions.iter().enumerate() {
        let path = format!("$.dispositions[{index}]");
        uuid::Uuid::parse_str(&disposition.finding_id)
            .map_err(|_| error(format!("{path}.finding_id must be a UUID")))?;
        if !finding_ids.insert(disposition.finding_id.as_str()) {
            return Err(error(format!("{path}.finding_id duplicates another disposition")));
        }
        validate_nonempty(&format!("{path}.decided_by"), &disposition.decided_by)?;
        validate_nonempty(&format!("{path}.decided_at"), &disposition.decided_at)?;
        validate_nonempty(&format!("{path}.rationale"), &disposition.rationale)?;
        chrono::DateTime::parse_from_rfc3339(&disposition.decided_at)
            .map_err(|_| error(format!("{path}.decided_at must be an RFC 3339 timestamp")))?;
    }
    file.dispositions.sort_by(|left, right| left.finding_id.cmp(&right.finding_id));
    Ok(())
}

fn validate_sha256(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(error(format!("{path} must be 64 lowercase hexadecimal characters")));
    }
    Ok(())
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
    use super::parse;

    #[test]
    fn duplicate_keys_are_rejected_before_contract_validation() {
        let error = parse(
            br#"{"schema_version":"forge.framework-impact-dispositions/1","schema_version":"forge.framework-impact-dispositions/1","prior_report_sha256":"0000000000000000000000000000000000000000000000000000000000000000","dispositions":[]}"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("duplicate object key"));
    }
}
