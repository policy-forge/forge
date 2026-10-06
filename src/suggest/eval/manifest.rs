//! Closed metadata for the first artifact-only evaluation preflight slice.

use crate::suggest::{shared, task::TaskIdentity};
use crate::{
    ForgeError,
    json_strict::{self, Limits},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Metadata contract; this does not publish the other proposed evaluation schemas.
pub const SCHEMA_VERSION: &str = "forge.suggest-eval-preflight-corpus/1";
/// Proposed engineering ceiling for encoded corpus metadata.
pub const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
/// Proposed engineering ceiling for cases in this slice.
pub const MAX_CASES: usize = 256;
/// Proposed engineering ceiling for source references in one case.
pub const MAX_SOURCES: usize = 128;
/// Proposed engineering ceiling for opaque references for one gate.
pub const MAX_EVIDENCE_REFS: usize = 64;
/// Maximum bytes captured from one opaque attestation.
pub const MAX_ATTESTATION_BYTES: u64 = 2 * 1024 * 1024;

/// An exact artifact beneath the explicitly selected corpus root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRef {
    /// Portable root-relative path; imported links use their own document directory.
    pub path: String,
    /// Lowercase SHA-256 of the complete raw file.
    pub sha256: String,
    /// Exact raw byte length, checked before decoding.
    pub bytes: u64,
}
impl ArtifactRef {
    pub(super) fn validate(&self, limit: u64) -> Result<(), ForgeError> {
        shared::relative_path("evaluation artifact path", &self.path)?;
        shared::sha256("evaluation artifact SHA-256", &self.sha256)?;
        if self.bytes == 0 || self.bytes > limit {
            return Err(error("evaluation artifact byte declaration exceeds its bound"));
        }
        Ok(())
    }
}

/// Honestly declared case origin; none of these assertions authenticate authorship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
    /// Operator-supplied real workflow assertion, requiring authentic rights and labels.
    RealWorkflow,
    /// Deliberately constructed attack, requiring separate safety adjudication.
    SyntheticAttack,
    /// Development fixture; never product acceptance evidence.
    SyntheticDevelopment,
}

/// One captured source used by request source-span units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifact {
    /// `SourceRef` key; unique inside this case.
    pub key: String,
    /// Exact complete source bytes beneath the corpus root.
    pub artifact: ArtifactRef,
}

/// Declared output-adjudication state; judgments are not qualified by this slice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Adjudication {
    /// No qualified output judgment is available.
    #[default]
    Missing,
    /// A dispute is explicitly reported and cannot receive success credit.
    Disputed,
}

/// Explicit case execution inventory; failure never becomes a model abstention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Outcome {
    /// No response was supplied.
    NotRun,
    /// Execution was blocked; no live execution is enabled by this slice.
    ExecutionBlocked,
    /// An operator reports a tool failure; no invented successful run record.
    ToolError,
    /// A captured response and its success-only suggestion run record are supplied.
    Response {
        /// Exact forge.suggest-run/1 document.
        run: ArtifactRef,
        /// Exact raw response, which may be malformed untrusted candidate content.
        response: ArtifactRef,
        /// Optional existing quarantine bundle to compare with pure validation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bundle: Option<ArtifactRef>,
        /// Missing or disputed authentic output adjudication.
        #[serde(default)]
        adjudication: Adjudication,
    },
}

/// One immutable expected case in the complete declared denominator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// Unique, opaque stable case key.
    pub case_key: String,
    /// One of the existing versioned mapping/drafting task identities.
    pub task: TaskIdentity,
    /// Explicit origin classification.
    pub origin: Origin,
    /// Declared evaluation stratum; no inferred safety or usefulness label.
    pub stratum: String,
    /// Declared source-family grouping for later freeze review.
    pub source_family: String,
    /// Declared workflow grouping for later freeze review.
    pub workflow_group: String,
    /// Portable directory beneath the selected root for SourceRef.path resolution.
    pub source_base: String,
    /// Prepared request, omitted when preparation has not occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<ArtifactRef>,
    /// Explicit source inventory; there is no directory discovery.
    #[serde(default)]
    pub sources: Vec<SourceArtifact>,
    /// Every case has an explicit state, including unfinished cases.
    pub outcome: Outcome,
}

/// Opaque, hash-bound supporting records; their semantics remain unsupported.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRefs {
    /// Rights references; presence alone grants no permission.
    #[serde(default)]
    pub rights: Vec<ArtifactRef>,
    /// Judgment references; no labels or reviewer independence are inferred.
    #[serde(default)]
    pub judgments: Vec<ArtifactRef>,
    /// Threshold references; no numeric gates are evaluated.
    #[serde(default)]
    pub thresholds: Vec<ArtifactRef>,
    /// Profile references; no approved RSS or wall-time enforcement is claimed.
    #[serde(default)]
    pub execution_profile: Vec<ArtifactRef>,
    /// Freeze references; no split or contamination qualification is inferred.
    #[serde(default)]
    pub freeze: Vec<ArtifactRef>,
    /// Candidate references; model weights or authentic execution are not inferred.
    #[serde(default)]
    pub candidate: Vec<ArtifactRef>,
}
impl EvidenceRefs {
    pub(super) fn groups(&self) -> [(&'static str, &[ArtifactRef]); 6] {
        [
            ("rights", &self.rights),
            ("judgments", &self.judgments),
            ("thresholds", &self.thresholds),
            ("execution-profile", &self.execution_profile),
            ("freeze", &self.freeze),
            ("candidate", &self.candidate),
        ]
    }
}

/// Closed corpus inventory for artifact preflight, with no acceptance credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    /// Must equal forge.suggest-eval-preflight-corpus/1.
    pub schema_version: String,
    /// Opaque corpus key.
    pub corpus_key: String,
    /// Explicit operator-supplied version, never generated from a clock.
    pub version: String,
    /// Complete case denominator, including every blocked or failed case.
    pub cases: Vec<Case>,
    /// Optional supporting records remain opaque and unqualified.
    #[serde(default)]
    pub evidence: EvidenceRefs,
}
impl Corpus {
    /// Parse bounded metadata; reject duplicate/unknown keys, nulls and unsupported versions.
    ///
    /// # Errors
    /// Returns an authoring error for malformed metadata or any fixed-bound violation.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_MANIFEST_BYTES {
            return Err(error("evaluation corpus metadata exceeds its bound"));
        }
        let value = json_strict::parse_value(
            bytes,
            "evaluation corpus",
            Limits { max_depth: 32, max_string_bytes: 16 * 1024 },
        )
        .map_err(|_| error("evaluation corpus JSON is invalid"))?;
        shared::reject_nulls(&value, "evaluation corpus")
            .map_err(|_| error("evaluation corpus must omit null values"))?;
        let corpus: Self = serde_json::from_value(value)
            .map_err(|_| error("evaluation corpus closed contract is invalid"))?;
        corpus.validate()?;
        Ok(corpus)
    }
    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(error("unsupported evaluation corpus schema version"));
        }
        shared::key("evaluation corpus key", &self.corpus_key)?;
        shared::label("evaluation corpus version", &self.version)?;
        if self.cases.is_empty() || self.cases.len() > MAX_CASES {
            return Err(error("evaluation case denominator is empty or exceeds its bound"));
        }
        let mut cases = BTreeSet::new();
        for case in &self.cases {
            shared::key("evaluation case key", &case.case_key)?;
            if !cases.insert(&case.case_key) {
                return Err(error("duplicate evaluation case key"));
            }
            case.task.validate("evaluation case task")?;
            for key in [&case.stratum, &case.source_family, &case.workflow_group] {
                shared::key("evaluation grouping key", key)?;
            }
            shared::relative_path("evaluation source base", &case.source_base)?;
            if let Some(request) = &case.request {
                request.validate(crate::suggest::request::MAX_REQUEST_BYTES)?;
            }
            if case.sources.len() > MAX_SOURCES {
                return Err(error("evaluation source inventory exceeds its bound"));
            }
            let mut sources = BTreeSet::new();
            let mut source_paths = BTreeSet::new();
            for source in &case.sources {
                shared::key("evaluation source key", &source.key)?;
                if !sources.insert(&source.key) {
                    return Err(error("duplicate evaluation source key"));
                }
                if !source_paths.insert(source.artifact.path.to_ascii_lowercase()) {
                    return Err(error("evaluation source paths must be distinct"));
                }
                source.artifact.validate(crate::reuse::corpus::MAX_DOCUMENT_BYTES)?;
            }
            if let Outcome::Response { run, response, bundle, .. } = &case.outcome {
                if case.request.is_none() {
                    return Err(error("a response outcome requires a prepared request"));
                }
                run.validate(crate::suggest::run_record::MAX_RUN_RECORD_BYTES)?;
                response.validate(crate::suggest::response::MAX_RESPONSE_BYTES)?;
                if let Some(bundle) = bundle {
                    bundle.validate(crate::suggest::bundle::MAX_BUNDLE_BYTES)?;
                }
            }
        }
        for (_, references) in self.evidence.groups() {
            if references.len() > MAX_EVIDENCE_REFS {
                return Err(error("evaluation attestation inventory exceeds its bound"));
            }
            for reference in references {
                reference.validate(MAX_ATTESTATION_BYTES)?;
            }
        }
        Ok(())
    }
}
pub(super) fn error(message: &str) -> ForgeError {
    ForgeError::Authoring(message.to_owned())
}
