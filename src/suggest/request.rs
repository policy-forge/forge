//! Closed, bounded `forge.suggest-request/1`: the exact payload a local
//! adapter would receive, and nothing else.
//!
//! `prepare` writes this contract together with the payload artifact it names.
//! The request is the allowlist: a unit that is not in this document cannot be
//! read by the adapter, and a citation to a unit that is not here cannot pass
//! validation. Redactions record the operator's declared substitutions by rule
//! hash, never by the literal they removed.

use serde::{Deserialize, Serialize};

use crate::ForgeError;
use crate::json_strict::{self, Limits};

use super::shared;
use super::task::{self, TaskKind, drafting, mapping};

/// Closed request contract version.
pub const SCHEMA_VERSION: &str = "forge.suggest-request/1";
/// Maximum encoded request size.
pub const MAX_REQUEST_BYTES: u64 = 2 * 1024 * 1024;
/// Maximum payload bytes handed to a local adapter.
pub const MAX_PAYLOAD_BYTES: u64 = 8 * 1024 * 1024;
/// Maximum allowlisted units in one request.
pub const MAX_UNITS: usize = 4_096;
/// Maximum declared redactions in one request.
pub const MAX_REDACTIONS: usize = 64;
/// Maximum adapter arguments.
pub const MAX_ARGV: usize = 32;
/// Maximum bytes in one adapter argument or the executable name.
pub const MAX_ARG_BYTES: usize = 1_024;
/// Maximum bytes in one operator-supplied model identifier.
pub const MAX_MODEL_ID_BYTES: usize = 256;
/// Maximum bytes in one cited span, matching the per-document capture bound.
pub const MAX_UNIT_SPAN_BYTES: u64 = 1024 * 1024;

/// Fixed notice written into every prepared preview.
pub const DATA_HANDLING_NOTICE: &str = "The payload below is the exact byte sequence this machine will hand to the local adapter. \
     FORGE opens no network connection and transmits nothing. The adapter is your own program: it runs with your user's \
     privileges and FORGE does not sandbox it, so it can read, write and transmit whatever you have allowed it to. \
     Choose an adapter you trust.";

const LIMITS: Limits = Limits { max_depth: 32, max_string_bytes: shared::MAX_STRING_BYTES };

/// The exact, redacted payload preview a local adapter would receive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestRequest {
    /// Closed contract version.
    pub schema_version: String,
    /// Project key the context was selected from.
    pub project_key: String,
    /// Supplied project timestamp; never a wall clock.
    pub as_of: String,
    /// The selected task and its versioned payload.
    pub task: RequestTask,
    /// The adapter identity consent is bound to.
    pub adapter: AdapterTarget,
    /// The only content the adapter may be shown.
    pub context: ContextBundle,
    /// Operator-declared substitutions applied before the payload was written.
    #[serde(default)]
    pub redactions: Vec<RedactionRecord>,
    /// The payload artifact and its exact digest.
    pub payload: PayloadPreview,
    /// Operator-supplied retention notice, recorded verbatim.
    pub retention_notice: String,
}

/// One versioned task payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestTask {
    /// Which task this request performs.
    pub kind: TaskKind,
    /// Task schema version; must match the kind.
    pub schema_version: String,
    /// Subjects for a mapping-candidate request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mapping_subjects: Vec<mapping::MappingSubject>,
    /// Sections for a policy-drafting request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drafting_sections: Vec<drafting::DraftingSection>,
}

impl RequestTask {
    fn validate(&self) -> Result<(), ForgeError> {
        task::check_task_version("request.task", self.kind, &self.schema_version)?;
        match self.kind {
            TaskKind::MappingCandidates => {
                if self.drafting_sections.is_empty() {
                    mapping::subjects("request.task.mapping_subjects", &self.mapping_subjects)?;
                }
                if self.mapping_subjects.is_empty() {
                    return Err(shared::error(
                        "request.task.mapping_subjects must not be empty for this task",
                    ));
                }
                if !self.drafting_sections.is_empty() {
                    return Err(shared::error(
                        "request.task.drafting_sections must be empty for a mapping-candidate task",
                    ));
                }
            }
            TaskKind::PolicyDrafting => {
                if self.mapping_subjects.is_empty() {
                    drafting::sections("request.task.drafting_sections", &self.drafting_sections)?;
                }
                if self.drafting_sections.is_empty() {
                    return Err(shared::error(
                        "request.task.drafting_sections must not be empty for this task",
                    ));
                }
                if !self.mapping_subjects.is_empty() {
                    return Err(shared::error(
                        "request.task.mapping_subjects must be empty for a policy-drafting task",
                    ));
                }
            }
        }
        Ok(())
    }
}

/// The local adapter a run is bound to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterTarget {
    /// Operator-supplied executable path or name, recorded verbatim.
    pub executable: String,
    /// Exact SHA-256 of the executable at prepare time.
    pub executable_sha256: String,
    /// Operator-supplied model identifier; FORGE never probes it.
    pub model_id: String,
    /// Extra arguments passed to the adapter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub argv: Vec<String>,
}

impl AdapterTarget {
    fn validate(&self) -> Result<(), ForgeError> {
        if self.executable.is_empty()
            || self.executable.len() > MAX_ARG_BYTES
            || self.executable.chars().any(char::is_control)
        {
            return Err(shared::error(format!(
                "request.adapter.executable must be one path or name of at most {MAX_ARG_BYTES} bytes"
            )));
        }
        shared::sha256("request.adapter.executable_sha256", &self.executable_sha256)?;
        if self.model_id.is_empty()
            || self.model_id.len() > MAX_MODEL_ID_BYTES
            || self.model_id.chars().any(char::is_control)
        {
            return Err(shared::error(format!(
                "request.adapter.model_id must be one identifier of at most {MAX_MODEL_ID_BYTES} bytes"
            )));
        }
        if self.argv.len() > MAX_ARGV {
            return Err(shared::error(format!("request.adapter.argv exceeds {MAX_ARGV} entries")));
        }
        for argument in &self.argv {
            if argument.len() > MAX_ARG_BYTES || argument.chars().any(char::is_control) {
                return Err(shared::error(format!(
                    "request.adapter.argv entries must be at most {MAX_ARG_BYTES} bytes"
                )));
            }
        }
        Ok(())
    }

    /// The adapter identity consent is bound to, as one comparison string.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.executable_sha256
    }
}

/// The allowlisted content for one run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextBundle {
    /// Selected units in the order the adapter receives them.
    pub units: Vec<ContextUnit>,
}

impl ContextBundle {
    fn validate(&self) -> Result<(), ForgeError> {
        if self.units.is_empty() {
            return Err(shared::error(
                "request.context.units must not be empty; nothing is sent for an empty allowlist",
            ));
        }
        if self.units.len() > MAX_UNITS {
            return Err(shared::error(format!(
                "request.context.units exceeds {MAX_UNITS} entries"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut previous_end = 0_u64;
        for (index, unit) in self.units.iter().enumerate() {
            unit.validate(&format!("request.context.units[{index}]"))?;
            if unit.payload.start < previous_end {
                return Err(shared::error(
                    "request.context.units must be ordered by ascending, non-overlapping payload spans",
                ));
            }
            previous_end = unit.payload.end;
            if !seen.insert(unit.unit_id.as_str()) {
                return Err(shared::error("request.context unit_ids must be unique"));
            }
        }
        Ok(())
    }

    fn unit(&self, unit_id: &str) -> Option<&ContextUnit> {
        self.units.iter().find(|unit| unit.unit_id == unit_id)
    }
}

/// What kind of supplied material one unit carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContextKind {
    /// A section of the supplied authoring plan.
    PlanSection,
    /// A supplied gap record.
    Gap,
    /// An approved answer from the supplied pack.
    Answer,
    /// A framework control identifier and its supplied label.
    Control,
    /// A selected span of a supplied document.
    SourceSpan,
}

impl ContextKind {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlanSection => "plan-section",
            Self::Gap => "gap",
            Self::Answer => "answer",
            Self::Control => "control",
            Self::SourceSpan => "source-span",
        }
    }
}

/// Declared sensitivity of the supplied material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sensitivity {
    /// Public material.
    Public,
    /// Internal material.
    Internal,
    /// Confidential material; never selected without the explicit flag.
    Confidential,
    /// Restricted material; never selected without the explicit flag.
    Restricted,
}

impl Sensitivity {
    /// The stricter of two declared sensitivities.
    #[must_use]
    pub const fn strictest(self, other: Self) -> Self {
        if self.rank() >= other.rank() { self } else { other }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::Public => 0,
            Self::Internal => 1,
            Self::Confidential => 2,
            Self::Restricted => 3,
        }
    }

    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Confidential => "confidential",
            Self::Restricted => "restricted",
        }
    }

    /// Whether the operator must acknowledge this material explicitly.
    #[must_use]
    pub const fn requires_acknowledgement(self) -> bool {
        matches!(self, Self::Confidential | Self::Restricted)
    }
}

/// Provenance of one unit, when the unit names a supplied source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRef {
    /// Stable key of the captured source.
    pub key: String,
    /// Portable descendant path of the captured source.
    pub path: String,
    /// Lowercase SHA-256 of the captured source bytes.
    pub sha256: String,
    /// Inclusive zero-based byte offset.
    pub start: u64,
    /// Exclusive zero-based byte offset.
    pub end: u64,
}

impl SourceRef {
    fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.key"), &self.key)?;
        shared::relative_path(&format!("{name}.path"), &self.path)?;
        shared::sha256(&format!("{name}.sha256"), &self.sha256)?;
        if self.end <= self.start {
            return Err(shared::error(format!("{name} must name a non-empty half-open byte span")));
        }
        if self.end - self.start > MAX_UNIT_SPAN_BYTES {
            return Err(shared::error(format!(
                "{name} must not exceed {MAX_UNIT_SPAN_BYTES} bytes"
            )));
        }
        Ok(())
    }
}

/// Half-open byte range of one unit inside the payload artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadSpan {
    /// Inclusive zero-based byte offset in the payload artifact.
    pub start: u64,
    /// Exclusive zero-based byte offset in the payload artifact.
    pub end: u64,
}

impl PayloadSpan {
    fn validate(&self, name: &str) -> Result<(), ForgeError> {
        if self.end <= self.start {
            return Err(shared::error(format!("{name} must name a non-empty half-open byte span")));
        }
        if self.end - self.start > MAX_UNIT_SPAN_BYTES {
            return Err(shared::error(format!(
                "{name} must not exceed {MAX_UNIT_SPAN_BYTES} bytes"
            )));
        }
        Ok(())
    }
}

/// One allowlisted unit: an exact byte range of the payload, plus provenance.
///
/// The unit does not carry its own copy of the text: the payload artifact is
/// the single source of truth for what the adapter receives, and a unit span
/// must lie inside it. That is what makes a citation checkable against the
/// exact bytes rather than against a second copy that could disagree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextUnit {
    /// Stable identifier citations refer to.
    pub unit_id: String,
    /// What kind of supplied material this is.
    pub kind: ContextKind,
    /// Human label shown in the preview.
    pub label: String,
    /// Declared sensitivity of the material.
    pub sensitivity: Sensitivity,
    /// Provenance, required for a selected source span.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceRef>,
    /// This unit's exact bytes inside the payload artifact.
    pub payload: PayloadSpan,
}

impl ContextUnit {
    fn validate(&self, name: &str) -> Result<(), ForgeError> {
        shared::key(&format!("{name}.unit_id"), &self.unit_id)?;
        shared::label(&format!("{name}.label"), &self.label)?;
        self.payload.validate(&format!("{name}.payload"))?;
        match (self.kind, &self.source) {
            (ContextKind::SourceSpan, None) => {
                Err(shared::error(format!("{name}.source is required for a source-span unit")))
            }
            (_, Some(source)) => source.validate(&format!("{name}.source")),
            (_, None) => Ok(()),
        }
    }
}

/// One operator-declared redaction, recorded by rule hash and never by literal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactionRecord {
    /// Unit the substitution was applied to.
    pub unit_id: String,
    /// Stable identifier of the operator's redaction rule.
    pub rule_id: String,
    /// SHA-256 of the rule text, so the rule is auditable without recording it.
    pub rule_sha256: String,
}

/// The payload artifact and its exact size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadPreview {
    /// Portable relative path of the payload artifact.
    pub artifact: String,
    /// Lowercase SHA-256 of the exact payload bytes.
    pub sha256: String,
    /// Exact payload byte length.
    pub bytes: u64,
    /// Number of units the payload contains.
    pub units: u64,
}

impl SuggestRequest {
    /// Parse and validate a bounded, closed request.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, duplicate or unknown
    /// keys, nulls, an unsupported version, unsafe paths or digests, an
    /// inconsistent task payload, an empty allowlist, or a redaction that names
    /// no unit.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_REQUEST_BYTES {
            return Err(shared::error(format!(
                "suggest request exceeds the {MAX_REQUEST_BYTES} byte limit"
            )));
        }
        let value = json_strict::parse_value(bytes, "suggest request", LIMITS)
            .map_err(|cause| shared::error(cause.to_string()))?;
        shared::reject_nulls(&value, "suggest request")?;
        let request: Self = serde_json::from_value(value)
            .map_err(|cause| shared::error(format!("invalid suggest request contract: {cause}")))?;
        request.validate()?;
        Ok(request)
    }

    /// The allowlisted units, in payload order.
    #[must_use]
    pub fn units(&self) -> &[ContextUnit] {
        &self.context.units
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(shared::error(format!(
                "suggest request schema_version must be {SCHEMA_VERSION}"
            )));
        }
        shared::key("request.project_key", &self.project_key)?;
        shared::single_line("request.as_of", &self.as_of)?;
        self.task.validate()?;
        self.adapter.validate()?;
        self.context.validate()?;
        redaction_records("request.redactions", &self.redactions)?;
        for (index, redaction) in self.redactions.iter().enumerate() {
            if self.context.unit(&redaction.unit_id).is_none() {
                return Err(shared::error(format!(
                    "request.redactions[{index}].unit_id must name a unit in this request"
                )));
            }
        }
        shared::relative_path("request.payload.artifact", &self.payload.artifact)?;
        shared::sha256("request.payload.sha256", &self.payload.sha256)?;
        if self.payload.bytes == 0 || self.payload.bytes > MAX_PAYLOAD_BYTES {
            return Err(shared::error(format!(
                "request.payload.bytes must be between 1 and {MAX_PAYLOAD_BYTES}"
            )));
        }
        if self.payload.units != self.context.units.len() as u64 {
            return Err(shared::error(
                "request.payload.units must equal the number of context units",
            ));
        }
        for (index, unit) in self.context.units.iter().enumerate() {
            if unit.payload.end > self.payload.bytes {
                return Err(shared::error(format!(
                    "request.context.units[{index}].payload must lie inside the payload artifact"
                )));
            }
        }
        shared::single_line("request.retention_notice", &self.retention_notice)?;
        Ok(())
    }
}

/// Validate redaction records: shape, bound and uniqueness.
///
/// Unit membership is checked by the contract that knows its own allowlist.
pub(in crate::suggest) fn redaction_records(
    name: &str,
    values: &[RedactionRecord],
) -> Result<(), ForgeError> {
    if values.len() > MAX_REDACTIONS {
        return Err(shared::error(format!("{name} exceeds {MAX_REDACTIONS} entries")));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (index, redaction) in values.iter().enumerate() {
        let path = format!("{name}[{index}]");
        shared::key(&format!("{path}.unit_id"), &redaction.unit_id)?;
        shared::key(&format!("{path}.rule_id"), &redaction.rule_id)?;
        shared::sha256(&format!("{path}.rule_sha256"), &redaction.rule_sha256)?;
        if !seen.insert((redaction.unit_id.as_str(), redaction.rule_id.as_str())) {
            return Err(shared::error(format!("{name} must not repeat a unit and rule pair")));
        }
    }
    Ok(())
}

/// A minimal valid request, shared by the contract tests in this module family.
#[cfg(test)]
pub(in crate::suggest) fn fixture_request() -> SuggestRequest {
    SuggestRequest::parse(&serde_json::to_vec(&fixture_request_json()).unwrap()).unwrap()
}

#[cfg(test)]
pub(in crate::suggest) fn fixture_request_json() -> serde_json::Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "project_key": "synthetic-project",
        "as_of": "2026-09-12T00:00:00Z",
        "task": {
            "kind": "policy-drafting",
            "schema_version": drafting::TASK_SCHEMA_VERSION,
            "drafting_sections": [{
                "policy_key": "access-policy",
                "topic_key": "access-control",
                "order": 1,
                "title": "Access control",
                "prompt": "Describe how accounts are reviewed.",
                "gap_ids": ["gap-0001"],
                "control_ids": ["ac-2"]
            }]
        },
        "adapter": {
            "executable": "local-model",
            "executable_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "model_id": "synthetic-model",
            "argv": ["--task", "draft"]
        },
        "context": {
            "units": [{
                "unit_id": "unit-0001",
                "kind": "source-span",
                "label": "Access control policy, review paragraph",
                "sensitivity": "internal",
                "source": {
                    "key": "access-policy",
                    "path": "prior/access-policy.md",
                    "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "start": 0,
                    "end": 64
                },
                "payload": {"start": 0, "end": 37}
            }]
        },
        "redactions": [],
        "payload": {
            "artifact": "payload.txt",
            "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "bytes": 37,
            "units": 1
        },
        "retention_notice": "Operator retains local adapter output for this run only."
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn parse(value: &Value) -> Result<SuggestRequest, ForgeError> {
        SuggestRequest::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_request_round_trips_with_every_field() {
        let parsed = parse(&fixture_request_json()).unwrap();
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
        assert_eq!(parsed.units().len(), 1);
        assert_eq!(parsed.task.kind, TaskKind::PolicyDrafting);
        assert_eq!(parsed.adapter.identity(), DIGEST);
        assert_eq!(parsed.units()[0].sensitivity, Sensitivity::Internal);
        assert!(!parsed.units()[0].sensitivity.requires_acknowledgement());

        let serialized = serde_json::to_vec(&parsed).unwrap();
        assert_eq!(SuggestRequest::parse(&serialized).unwrap(), parsed);
    }

    #[test]
    fn mapping_requests_require_subjects_and_reject_drafting_sections() {
        let mut mapping = fixture_request_json();
        mapping["task"] = json!({
            "kind": "mapping-candidates",
            "schema_version": mapping::TASK_SCHEMA_VERSION,
            "mapping_subjects": [{
                "policy_key": "access-policy",
                "topic_key": "access-control",
                "title": "Access control",
                "text": "Accounts must be reviewed quarterly.",
                "control_ids": ["ac-2"],
                "gap_ids": []
            }]
        });
        assert!(parse(&mapping).is_ok());

        mapping["task"]["drafting_sections"] = json!([]);
        mapping["task"]["mapping_subjects"] = json!([]);
        assert!(parse(&mapping).is_err());

        mapping["task"]["mapping_subjects"] = json!([{
            "policy_key": "access-policy",
            "topic_key": "access-control",
            "title": "Access control",
            "text": "Accounts must be reviewed quarterly."
        }]);
        mapping["task"]["drafting_sections"] = json!([{
            "policy_key": "access-policy",
            "topic_key": "access-control",
            "order": 1,
            "title": "Access control",
            "prompt": "Describe how accounts are reviewed."
        }]);
        assert!(parse(&mapping).is_err());
    }

    #[test]
    fn task_version_must_match_the_declared_kind() {
        let mut mismatched = fixture_request_json();
        mismatched["task"]["schema_version"] = json!(mapping::TASK_SCHEMA_VERSION);
        assert!(parse(&mismatched).is_err());

        let mut unsupported = fixture_request_json();
        unsupported["task"]["schema_version"] = json!("forge.suggest-task-drafting/2");
        assert!(parse(&unsupported).is_err());
    }

    #[test]
    fn unknown_null_and_duplicate_keys_are_rejected() {
        let mut unknown = fixture_request_json();
        unknown["extra"] = json!(true);
        assert!(parse(&unknown).is_err());

        let mut null = fixture_request_json();
        null["adapter"]["argv"] = json!(null);
        assert!(parse(&null).is_err());

        let duplicate = String::from_utf8(serde_json::to_vec(&fixture_request_json()).unwrap())
            .unwrap()
            .replace(
                "\"project_key\":\"synthetic-project\"",
                "\"project_key\":\"synthetic-project\",\"project_key\":\"other\"",
            );
        assert!(SuggestRequest::parse(duplicate.as_bytes()).is_err());
    }

    #[test]
    fn allowlist_bounds_and_source_rules_hold() {
        let mut empty = fixture_request_json();
        empty["context"]["units"] = json!([]);
        assert!(parse(&empty).is_err());

        let mut spanless = fixture_request_json();
        spanless["context"]["units"][0].as_object_mut().unwrap().remove("source");
        assert!(parse(&spanless).is_err());

        let mut empty_span = fixture_request_json();
        empty_span["context"]["units"][0]["source"]["end"] = json!(0);
        assert!(parse(&empty_span).is_err());

        let mut duplicate_unit = fixture_request_json();
        let unit = duplicate_unit["context"]["units"][0].clone();
        duplicate_unit["context"]["units"] = json!([unit.clone(), unit]);
        duplicate_unit["payload"]["units"] = json!(2);
        assert!(parse(&duplicate_unit).is_err());

        let mut mismatched_count = fixture_request_json();
        mismatched_count["payload"]["units"] = json!(2);
        assert!(parse(&mismatched_count).is_err());
    }

    #[test]
    fn redactions_must_name_a_unit_and_stay_unique() {
        let mut valid = fixture_request_json();
        valid["redactions"] = json!([{
            "unit_id": "unit-0001",
            "rule_id": "bearer-token",
            "rule_sha256": DIGEST
        }]);
        assert!(parse(&valid).is_ok());

        let mut unknown_unit = fixture_request_json();
        unknown_unit["redactions"] = json!([{
            "unit_id": "unit-9999",
            "rule_id": "bearer-token",
            "rule_sha256": DIGEST
        }]);
        assert!(parse(&unknown_unit).is_err());

        let mut repeated = fixture_request_json();
        repeated["redactions"] = json!([
            {"unit_id": "unit-0001", "rule_id": "bearer-token", "rule_sha256": DIGEST},
            {"unit_id": "unit-0001", "rule_id": "bearer-token", "rule_sha256": DIGEST}
        ]);
        assert!(parse(&repeated).is_err());
    }

    #[test]
    fn payload_and_request_bounds_are_enforced_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_REQUEST_BYTES).unwrap() + 1];
        assert!(SuggestRequest::parse(&oversized).is_err());

        let mut zero_payload = fixture_request_json();
        zero_payload["payload"]["bytes"] = json!(0);
        assert!(parse(&zero_payload).is_err());

        let mut huge_payload = fixture_request_json();
        huge_payload["payload"]["bytes"] = json!(MAX_PAYLOAD_BYTES + 1);
        assert!(parse(&huge_payload).is_err());
    }

    #[test]
    fn a_unit_span_past_its_bound_is_refused() {
        let mut oversized = fixture_request_json();
        oversized["context"]["units"][0]["payload"] = json!({"start": 0, "end": 0});
        assert!(parse(&oversized).is_err());

        let mut too_long = fixture_request_json();
        too_long["context"]["units"][0]["payload"] =
            json!({"start": 0, "end": MAX_UNIT_SPAN_BYTES + 1});
        too_long["payload"]["bytes"] = json!(MAX_UNIT_SPAN_BYTES + 1);
        assert!(parse(&too_long).is_err());

        let mut outside = fixture_request_json();
        outside["context"]["units"][0]["payload"] = json!({"start": 0, "end": 64});
        outside["payload"]["bytes"] = json!(32);
        assert!(parse(&outside).is_err());
    }

    #[test]
    fn overlapping_or_unordered_unit_spans_are_refused() {
        let mut reversed = fixture_request_json();
        let unit = reversed["context"]["units"][0].clone();
        let mut second = unit.clone();
        second["unit_id"] = json!("unit-0002");
        second["payload"] = json!({"start": 0, "end": 5});
        reversed["context"]["units"] = json!([unit, second]);
        reversed["payload"]["units"] = json!(2);
        reversed["payload"]["bytes"] = json!(37);
        assert!(parse(&reversed).is_err());
    }

    #[test]
    fn request_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.suggest-request-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/suggest-request/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
