//! Deterministic, bounded preflight receipts without source or candidate prose.

use super::manifest::{MAX_CASES, Origin, error};
use crate::{
    ForgeError,
    suggest::{RunMode, bundle::SuggestionCounts, task::TaskKind},
};
use serde::Serialize;
use std::io::{self, Write};

/// Closed report wire version; distinct from a full evaluation acceptance report.
pub const SCHEMA_VERSION: &str = "forge.suggest-eval-preflight/1";
/// Proposed engineering ceiling for encoded report bytes.
pub const MAX_REPORT_BYTES: usize = 2 * 1024 * 1024;

/// Readiness of this artifact-only slice, never full evaluation success.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Authentic evidence and full evaluation qualification remain open.
    Incomplete,
    /// At least one supplied candidate response fails structural validation.
    Failed,
}
/// One accounted case state; none means a completed quality evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseState {
    /// No response was supplied.
    NotRun,
    /// Execution was explicitly blocked.
    ExecutionBlocked,
    /// The operator reported a tool failure.
    ToolError,
    /// Pinned candidate bytes were refused by ordinary suggestion validation.
    ResponseInvalid,
    /// Structurally valid artifacts await authentic output judgments.
    AwaitingAdjudication,
    /// Structurally valid artifacts carry a declared unresolved dispute.
    AdjudicationDisputed,
}
/// Exact expected denominator and every case-state count.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// Complete declared case denominator.
    pub expected: u64,
    /// Cases with a structurally valid request/run/response chain.
    pub artifacts_valid: u64,
    /// Cases with no supplied response.
    pub not_run: u64,
    /// Cases blocked before a response.
    pub execution_blocked: u64,
    /// Operator-declared tool failures.
    pub tool_error: u64,
    /// Structurally invalid candidate responses.
    pub response_invalid: u64,
    /// Structurally valid cases missing authentic judgments.
    pub awaiting_adjudication: u64,
    /// Structurally valid cases with unresolved declared disputes.
    pub adjudication_disputed: u64,
}
/// Content-free exact artifact fingerprint; portable file paths stay private.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactPin {
    /// Fixed artifact role or bounded opaque source key.
    pub role: String,
    /// Complete raw-file SHA-256.
    pub sha256: String,
    /// Exact raw byte count.
    pub bytes: u64,
}
/// One case receipt in stable key order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaseReport {
    /// Stable case key; the operator must choose an appropriate opaque key.
    pub case_key: String,
    /// Existing task kind.
    pub task: TaskKind,
    /// Declared origin, preserved without upgrading synthetic evidence.
    pub origin: Origin,
    /// Accounted case state.
    pub state: CaseState,
    /// Whether the complete supplied candidate chain passed structural validation.
    pub artifacts_valid: bool,
    /// Runtime-recomputed suggestion counts; these are not correctness scores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanical_counts: Option<SuggestionCounts>,
    /// Mode declared by the run record; this is not authenticated execution evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_run_mode: Option<RunMode>,
    /// Exact content-free pins consumed for this case.
    pub artifacts: Vec<ArtifactPin>,
}
/// Supporting-record gate status; opaque records never earn approval credit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    /// No supporting records were supplied.
    Missing,
    /// Bytes were pinned, but semantic qualification is unsupported in this slice.
    OpaqueUnsupported,
}
/// One unqualified supporting-record group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvidenceGate {
    /// Fixed gate name.
    pub gate: String,
    /// Missing or explicitly unsupported qualification.
    pub status: EvidenceStatus,
    /// Exact submitted fingerprints, excluding paths and record content.
    pub artifacts: Vec<ArtifactPin>,
}
/// Bounded report from artifact-only preflight; acceptance eligibility is always false.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::struct_excessive_bools)] // Closed wire exposes completeness and three immutable qualification refusals.
pub struct Report {
    /// forge.suggest-eval-preflight/1.
    pub schema_version: String,
    /// Always artifact-preflight.
    pub scope: String,
    /// Incomplete or structurally failed; never passed evaluation.
    pub status: Status,
    /// Whether every expected case has a structurally valid candidate chain.
    pub preflight_complete: bool,
    /// Always false: no full quality/safety/usefulness gate is evaluated.
    pub acceptance_eligible: bool,
    /// Always false: this API cannot invoke any model.
    pub generation_enabled: bool,
    /// Always false: first-task selection requires authentic corpus evidence.
    pub task_selection_ready: bool,
    /// Operator-supplied opaque corpus key.
    pub corpus_key: String,
    /// Operator-supplied explicit corpus version.
    pub corpus_version: String,
    /// Exact raw corpus metadata digest.
    pub manifest_sha256: String,
    /// Exact complete denominator and each state count.
    pub counts: Counts,
    /// One result per expected case, in stable key order.
    pub cases: Vec<CaseReport>,
    /// Rights, judgments, thresholds, profile, freeze and candidate qualification remain open.
    pub evidence_gates: Vec<EvidenceGate>,
    /// Fixed remaining gates, including the final full documentation review.
    pub pending_gates: Vec<String>,
}
impl Report {
    /// Encode deterministic compact JSON plus LF within the fixed report bound.
    ///
    /// # Errors
    /// Returns an authoring error for an internally inconsistent or upgraded
    /// receipt, or if bounded serialization cannot complete. This guard does
    /// not authenticate supplied evidence or a caller-constructed report.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, ForgeError> {
        self.validate()?;
        let mut writer = BoundedWriter { bytes: Vec::new() };
        serde_json::to_writer(&mut writer, self)
            .map_err(|_| error("evaluation report encoding exceeds its bound or failed"))?;
        writer
            .write_all(b"\n")
            .map_err(|_| error("evaluation report encoding exceeds its bound"))?;
        Ok(writer.bytes)
    }
    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION || self.scope != "artifact-preflight" {
            return Err(error("evaluation report schema version or scope is invalid"));
        }
        if self.acceptance_eligible || self.generation_enabled || self.task_selection_ready {
            return Err(error(
                "evaluation artifact preflight cannot claim acceptance or generation readiness",
            ));
        }
        if self.cases.is_empty() || self.cases.len() > MAX_CASES {
            return Err(error("evaluation report case denominator is empty or exceeds its bound"));
        }
        let mut counts = Counts { expected: self.cases.len() as u64, ..Counts::default() };
        let mut previous_key: Option<&str> = None;
        for case in &self.cases {
            if previous_key.is_some_and(|previous| previous >= case.case_key.as_str()) {
                return Err(error("evaluation report case keys are duplicated or out of order"));
            }
            previous_key = Some(&case.case_key);
            let valid_state = matches!(
                case.state,
                CaseState::AwaitingAdjudication | CaseState::AdjudicationDisputed
            );
            let response_state = valid_state || case.state == CaseState::ResponseInvalid;
            if case.artifacts_valid != valid_state
                || case.mechanical_counts.is_some() != valid_state
                || case.declared_run_mode.is_some() != response_state
            {
                return Err(error("evaluation report case state contradicts its structural chain"));
            }
            counts.artifacts_valid += u64::from(case.artifacts_valid);
            match case.state {
                CaseState::NotRun => counts.not_run += 1,
                CaseState::ExecutionBlocked => counts.execution_blocked += 1,
                CaseState::ToolError => counts.tool_error += 1,
                CaseState::ResponseInvalid => counts.response_invalid += 1,
                CaseState::AwaitingAdjudication => counts.awaiting_adjudication += 1,
                CaseState::AdjudicationDisputed => counts.adjudication_disputed += 1,
            }
        }
        if self.counts != counts {
            return Err(error(
                "evaluation report denominator or state counts disagree with its cases",
            ));
        }
        if self.preflight_complete != (counts.artifacts_valid == counts.expected) {
            return Err(error("evaluation report completeness contradicts its exact denominator"));
        }
        let status = if counts.response_invalid == 0 { Status::Incomplete } else { Status::Failed };
        if self.status != status {
            return Err(error("evaluation report status contradicts its candidate outcomes"));
        }
        Ok(())
    }
}
struct BoundedWriter {
    bytes: Vec<u8>,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_REPORT_BYTES.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("evaluation report byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoded_escape_growth_is_bounded_without_truncation_credit() {
        let mut writer = BoundedWriter { bytes: Vec::new() };
        // JSON escaping expands each decoded control byte; bound encoded output.
        let text = "\u{0001}".repeat((MAX_REPORT_BYTES - 2) / 6);
        assert!(serde_json::to_writer(&mut writer, &text).is_ok());
        assert_eq!(writer.bytes.len(), text.len() * 6 + 2);
        let remaining = MAX_REPORT_BYTES - writer.bytes.len();
        writer.write_all(&vec![b' '; remaining]).unwrap();
        assert_eq!(writer.bytes.len(), MAX_REPORT_BYTES);
        assert!(writer.write_all(b"x").is_err());
        assert_eq!(writer.bytes.len(), MAX_REPORT_BYTES);
    }
}
