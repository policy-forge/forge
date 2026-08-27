//! Closed, bounded lifecycle record contract with legacy `/1` validation and current `/2` IDs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use chrono::{DateTime, NaiveDate};
use clap::ValueEnum;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Number, Value};
use uuid::Uuid;

use crate::ForgeError;

pub const LEGACY_SCHEMA_VERSION: &str = "forge.policy-lifecycle/1";
pub const SCHEMA_VERSION: &str = "forge.policy-lifecycle/2";
pub const APPROVAL_POLICY_VERSION: &str = "forge.approval-policy/1";
pub const MAX_RECORD_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PARTIES: usize = 64;
const MAX_ARTIFACTS: usize = 128;
const MAX_EVENTS: usize = 1024;
const MAX_ASSERTIONS: usize = 64;
const MAX_IMPACT_FINDINGS: usize = 128;
const MAX_STRING_BYTES: usize = 4096;
const MAX_DEPTH: usize = 16;
const MAX_COLLECTION_ITEMS: usize = 4096;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LifecycleRecord {
    pub schema_version: String,
    pub policy: PolicyIdentity,
    pub parties: Vec<Party>,
    pub approval_policy: ApprovalPolicy,
    pub review: ReviewSchedule,
    pub state: LifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<PolicyReference>,
    pub history: Vec<TransitionEvent>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicyIdentity {
    pub policy_key: String,
    pub version_key: String,
    pub title: String,
    pub owner_keys: Vec<String>,
    pub source: ArtifactFingerprint,
    pub generated_artifacts: Vec<ArtifactFingerprint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactFingerprint {
    pub path: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oscal_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Party {
    pub key: String,
    pub roles: Vec<DeclaredRole>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum DeclaredRole {
    Author,
    Reviewer,
    Approver,
    Owner,
    Custodian,
}

impl DeclaredRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Author => "author",
            Self::Reviewer => "reviewer",
            Self::Approver => "approver",
            Self::Owner => "owner",
            Self::Custodian => "custodian",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApprovalPolicy {
    pub schema_version: String,
    pub required_roles: Vec<RoleRequirement>,
    pub separation: SeparationRules,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoleRequirement {
    pub role: DeclaredRole,
    pub count: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct SeparationRules {
    #[serde(default)]
    pub author_reviewer: bool,
    #[serde(default)]
    pub author_approver: bool,
    #[serde(default)]
    pub reviewer_approver: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReviewSchedule {
    pub cadence_days: u16,
    pub next_review_date: NaiveDate,
    pub due_soon_days: u16,
    pub timezone_policy: TimezonePolicy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TimezonePolicy {
    DateOnly,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    Draft,
    InReview,
    Approved,
    Superseded,
    Retired,
}

impl LifecycleState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in-review",
            Self::Approved => "approved",
            Self::Superseded => "superseded",
            Self::Retired => "retired",
        }
    }

    #[must_use]
    pub const fn permits(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::InReview | Self::Retired)
                | (Self::InReview, Self::Draft | Self::Approved | Self::Retired)
                | (Self::Approved, Self::InReview | Self::Superseded | Self::Retired)
                | (Self::Superseded, Self::Retired)
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TransitionEvent {
    pub sequence: u32,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_event_id: Option<String>,
    pub previous_state: LifecycleState,
    pub next_state: LifecycleState,
    pub actor_key: String,
    pub declared_role: DeclaredRole,
    pub timestamp: String,
    pub rationale: String,
    pub fingerprints: FingerprintSet,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assertions: Vec<ActorAssertion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub impact_finding_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<PolicyReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ActorAssertion {
    pub actor_key: String,
    pub declared_role: DeclaredRole,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FingerprintSet {
    pub source_sha256: String,
    pub generated_artifacts: Vec<NamedHash>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct NamedHash {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct PolicyReference {
    pub policy_key: String,
    pub version_key: String,
}

/// Parse and fully validate a lifecycle record, including duplicate-key rejection.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] when JSON syntax, the closed schema, a bound, or an
/// intrinsic lifecycle invariant is invalid.
pub fn parse(bytes: &[u8]) -> Result<LifecycleRecord, ForgeError> {
    if bytes.len() as u64 > MAX_RECORD_BYTES {
        return Err(error(format!("record exceeds the {MAX_RECORD_BYTES} byte limit")));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let strict = StrictValue::deserialize(&mut deserializer)
        .map_err(|source| error(format!("invalid lifecycle JSON: {source}")))?;
    deserializer
        .end()
        .map_err(|source| error(format!("invalid trailing lifecycle data: {source}")))?;
    enforce_value_bounds(&strict.0, "$", 0)?;
    let record: LifecycleRecord = serde_json::from_value(strict.0)
        .map_err(|source| error(format!("invalid lifecycle contract: {source}")))?;
    validate(&record)?;
    Ok(record)
}

/// Validate all intrinsic record invariants and the append-only event chain.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] for an invalid schema version, party, approval policy,
/// fingerprint, schedule, state transition, event identity, or supersession reference.
pub fn validate(record: &LifecycleRecord) -> Result<(), ForgeError> {
    if !matches!(record.schema_version.as_str(), LEGACY_SCHEMA_VERSION | SCHEMA_VERSION) {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {LEGACY_SCHEMA_VERSION} or {SCHEMA_VERSION}",
            bounded(&record.schema_version)
        )));
    }
    non_empty("$.policy.policy_key", &record.policy.policy_key)?;
    non_empty("$.policy.version_key", &record.policy.version_key)?;
    non_empty("$.policy.title", &record.policy.title)?;
    if record.policy.owner_keys.is_empty() {
        return Err(error("$.policy.owner_keys must not be empty"));
    }
    validate_fingerprint("$.policy.source", &record.policy.source)?;
    if record.policy.generated_artifacts.len() > MAX_ARTIFACTS {
        return Err(error(format!(
            "$.policy.generated_artifacts exceeds the {MAX_ARTIFACTS} entry limit"
        )));
    }
    let mut artifact_paths = BTreeSet::new();
    for (index, artifact) in record.policy.generated_artifacts.iter().enumerate() {
        validate_fingerprint(&format!("$.policy.generated_artifacts[{index}]"), artifact)?;
        if !artifact_paths.insert(artifact.path.as_str()) {
            return Err(error(format!(
                "$.policy.generated_artifacts[{index}].path duplicates '{}'",
                bounded(&artifact.path)
            )));
        }
    }
    validate_parties(record)?;
    validate_approval_policy(&record.approval_policy)?;
    if record.review.cadence_days == 0 {
        return Err(error("$.review.cadence_days must be greater than zero"));
    }
    if record.review.due_soon_days > record.review.cadence_days {
        return Err(error("$.review.due_soon_days must not exceed cadence_days"));
    }
    if record.history.len() > MAX_EVENTS {
        return Err(error(format!("$.history exceeds the {MAX_EVENTS} event limit")));
    }
    validate_history(record)?;
    let has_supersession =
        record.history.iter().any(|event| event.next_state == LifecycleState::Superseded);
    match (record.state, &record.replaced_by, has_supersession) {
        (LifecycleState::Superseded, None, _) => {
            return Err(error("a superseded record requires replaced_by"));
        }
        (LifecycleState::Superseded, Some(reference), _)
        | (LifecycleState::Retired, Some(reference), true) => {
            validate_reference("$.replaced_by", reference, record)?;
        }
        (LifecycleState::Retired, None, true) => {
            return Err(error("a retired record with supersession history requires replaced_by"));
        }
        (_, Some(_), _) => {
            return Err(error("replaced_by is only valid after a supersession transition"));
        }
        (_, None, _) => {}
    }
    Ok(())
}

fn validate_parties(record: &LifecycleRecord) -> Result<(), ForgeError> {
    if record.parties.is_empty() || record.parties.len() > MAX_PARTIES {
        return Err(error(format!("$.parties must contain 1..={MAX_PARTIES} entries")));
    }
    let mut parties = BTreeMap::new();
    for (index, party) in record.parties.iter().enumerate() {
        non_empty(&format!("$.parties[{index}].key"), &party.key)?;
        if party.roles.is_empty() {
            return Err(error(format!("$.parties[{index}].roles must not be empty")));
        }
        let roles: BTreeSet<_> = party.roles.iter().copied().collect();
        if roles.len() != party.roles.len() {
            return Err(error(format!("$.parties[{index}].roles contains duplicates")));
        }
        if parties.insert(party.key.as_str(), roles).is_some() {
            return Err(error(format!("$.parties[{index}].key duplicates '{}'", party.key)));
        }
    }
    for owner in &record.policy.owner_keys {
        let roles = parties.get(owner.as_str()).ok_or_else(|| {
            error(format!("owner key '{}' references an unknown party", bounded(owner)))
        })?;
        if !roles.contains(&DeclaredRole::Owner) {
            return Err(error(format!("owner party '{}' lacks the owner role", bounded(owner))));
        }
    }
    if record.policy.owner_keys.iter().collect::<BTreeSet<_>>().len()
        != record.policy.owner_keys.len()
    {
        return Err(error("$.policy.owner_keys contains duplicates"));
    }
    for requirement in &record.approval_policy.required_roles {
        let available = parties.values().filter(|roles| roles.contains(&requirement.role)).count();
        if available < usize::from(requirement.count) {
            return Err(error(format!(
                "approval policy requires {} '{}' parties; only {available} are declared",
                requirement.count,
                requirement.role.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_approval_policy(policy: &ApprovalPolicy) -> Result<(), ForgeError> {
    if policy.schema_version != APPROVAL_POLICY_VERSION {
        return Err(error(format!(
            "unsupported approval policy schema_version '{}'; expected {APPROVAL_POLICY_VERSION}",
            bounded(&policy.schema_version)
        )));
    }
    if policy.required_roles.is_empty() {
        return Err(error("$.approval_policy.required_roles must not be empty"));
    }
    let mut roles = BTreeSet::new();
    for (index, requirement) in policy.required_roles.iter().enumerate() {
        if requirement.count == 0 {
            return Err(error(format!(
                "$.approval_policy.required_roles[{index}].count must be greater than zero"
            )));
        }
        if !roles.insert(requirement.role) {
            return Err(error(format!(
                "$.approval_policy.required_roles[{index}].role is duplicated"
            )));
        }
    }
    Ok(())
}

fn validate_history(record: &LifecycleRecord) -> Result<(), ForgeError> {
    let parties: BTreeMap<_, BTreeSet<_>> = record
        .parties
        .iter()
        .map(|party| (party.key.as_str(), party.roles.iter().copied().collect()))
        .collect();
    let mut prior_state = LifecycleState::Draft;
    let mut prior_time: Option<DateTime<chrono::FixedOffset>> = None;
    for (index, event) in record.history.iter().enumerate() {
        let path = format!("$.history[{index}]");
        let expected_sequence = u32::try_from(index + 1)
            .map_err(|_| error("event sequence exceeds supported range"))?;
        if event.sequence != expected_sequence {
            return Err(error(format!(
                "{path}.sequence is {}; expected contiguous sequence {expected_sequence}",
                event.sequence
            )));
        }
        if event.previous_state != prior_state || !prior_state.permits(event.next_state) {
            return Err(error(format!(
                "{path} contains invalid transition {} -> {}",
                event.previous_state.as_str(),
                event.next_state.as_str()
            )));
        }
        validate_actor(&path, &event.actor_key, event.declared_role, &parties)?;
        if event.assertions.len() > MAX_ASSERTIONS {
            return Err(error(format!(
                "{path}.assertions exceeds the {MAX_ASSERTIONS} entry limit"
            )));
        }
        if event.assertions.windows(2).any(|items| items[0] >= items[1]) {
            return Err(error(format!("{path}.assertions must be unique and sorted")));
        }
        validate_impact_findings(&path, event)?;
        let mut assertions = BTreeSet::new();
        assertions.insert((event.actor_key.as_str(), event.declared_role));
        for (assertion_index, assertion) in event.assertions.iter().enumerate() {
            validate_actor(
                &format!("{path}.assertions[{assertion_index}]"),
                &assertion.actor_key,
                assertion.declared_role,
                &parties,
            )?;
            if !assertions.insert((assertion.actor_key.as_str(), assertion.declared_role)) {
                return Err(error(format!(
                    "{path}.assertions contains duplicate actor-role evidence"
                )));
            }
        }
        let parsed_time = DateTime::parse_from_rfc3339(&event.timestamp)
            .map_err(|source| error(format!("{path}.timestamp is not RFC 3339: {source}")))?;
        if prior_time.is_some_and(|prior| parsed_time < prior) {
            return Err(error(format!("{path}.timestamp is earlier than the prior event")));
        }
        prior_time = Some(parsed_time);
        non_empty(&format!("{path}.rationale"), &event.rationale)?;
        validate_fingerprint_set(&format!("{path}.fingerprints"), &event.fingerprints)?;
        validate_replacement(&path, event, record)?;
        validate_legacy_event_id(&path, record, event)?;
        let expected_id = event_id(record, event)?;
        if event.event_id != expected_id {
            return Err(error(format!(
                "{path}.event_id does not match deterministic event evidence"
            )));
        }
        if event.next_state == LifecycleState::Approved {
            validate_approval(record, index, &assertions, &event.fingerprints)?;
        }
        prior_state = event.next_state;
    }
    if record.state != prior_state {
        return Err(error(format!(
            "$.state is {}; event history ends in {}",
            record.state.as_str(),
            prior_state.as_str()
        )));
    }
    Ok(())
}

fn validate_legacy_event_id(
    path: &str,
    record: &LifecycleRecord,
    event: &TransitionEvent,
) -> Result<(), ForgeError> {
    match (record.schema_version.as_str(), &event.legacy_event_id) {
        (LEGACY_SCHEMA_VERSION, Some(_)) => {
            Err(error(format!("{path}.legacy_event_id is only valid in {SCHEMA_VERSION}")))
        }
        (SCHEMA_VERSION, Some(value)) => {
            Uuid::parse_str(value)
                .map_err(|source| error(format!("{path}.legacy_event_id is invalid: {source}")))?;
            if value != &legacy_event_id(record, event)? {
                return Err(error(format!(
                    "{path}.legacy_event_id does not match legacy event evidence"
                )));
            }
            Ok(())
        }
        (_, None) => Ok(()),
        _ => Err(error("unsupported lifecycle schema version")),
    }
}

fn validate_actor(
    path: &str,
    actor_key: &str,
    role: DeclaredRole,
    parties: &BTreeMap<&str, BTreeSet<DeclaredRole>>,
) -> Result<(), ForgeError> {
    non_empty(&format!("{path}.actor_key"), actor_key)?;
    let roles = parties.get(actor_key).ok_or_else(|| {
        error(format!("{path} references unknown actor '{}'", bounded(actor_key)))
    })?;
    if !roles.contains(&role) {
        return Err(error(format!(
            "{path} actor '{}' does not declare role '{}'",
            bounded(actor_key),
            role.as_str()
        )));
    }
    Ok(())
}

fn validate_replacement(
    path: &str,
    event: &TransitionEvent,
    record: &LifecycleRecord,
) -> Result<(), ForgeError> {
    if event.next_state == LifecycleState::Superseded {
        let replacement = event
            .replacement
            .as_ref()
            .ok_or_else(|| error(format!("{path}.replacement is required for supersession")))?;
        validate_reference(&format!("{path}.replacement"), replacement, record)?;
        if record.replaced_by.as_ref() != Some(replacement) {
            return Err(error(format!("{path}.replacement does not match $.replaced_by")));
        }
    } else if event.replacement.is_some() {
        return Err(error(format!("{path}.replacement is only valid for supersession")));
    }
    Ok(())
}

fn validate_impact_findings(path: &str, event: &TransitionEvent) -> Result<(), ForgeError> {
    if event.impact_finding_ids.len() > MAX_IMPACT_FINDINGS {
        return Err(error(format!(
            "{path}.impact_finding_ids exceeds the {MAX_IMPACT_FINDINGS} entry limit"
        )));
    }
    if !event.impact_finding_ids.is_empty() && event.next_state != LifecycleState::InReview {
        return Err(error(format!(
            "{path}.impact_finding_ids are only valid when entering in-review"
        )));
    }
    let mut prior: Option<&str> = None;
    for (index, finding_id) in event.impact_finding_ids.iter().enumerate() {
        non_empty(&format!("{path}.impact_finding_ids[{index}]"), finding_id)?;
        if prior.is_some_and(|value| value >= finding_id.as_str()) {
            return Err(error(format!("{path}.impact_finding_ids must be unique and sorted")));
        }
        prior = Some(finding_id);
    }
    Ok(())
}

fn validate_reference(
    path: &str,
    reference: &PolicyReference,
    record: &LifecycleRecord,
) -> Result<(), ForgeError> {
    non_empty(&format!("{path}.policy_key"), &reference.policy_key)?;
    non_empty(&format!("{path}.version_key"), &reference.version_key)?;
    if reference.policy_key == record.policy.policy_key
        && reference.version_key == record.policy.version_key
    {
        return Err(error(format!("{path} must not reference the same policy version")));
    }
    Ok(())
}

fn validate_approval(
    record: &LifecycleRecord,
    event_index: usize,
    current_assertions: &BTreeSet<(&str, DeclaredRole)>,
    fingerprints: &FingerprintSet,
) -> Result<(), ForgeError> {
    let review_start = record.history[..event_index]
        .iter()
        .rposition(|event| event.next_state == LifecycleState::InReview)
        .ok_or_else(|| error("approval requires a preceding transition into in-review"))?;
    let mut evidence = BTreeSet::new();
    for event in &record.history[review_start..=event_index] {
        if &event.fingerprints != fingerprints {
            return Err(error("approval evidence must reference identical fingerprints"));
        }
        evidence.insert((event.actor_key.as_str(), event.declared_role));
        evidence.extend(
            event.assertions.iter().map(|item| (item.actor_key.as_str(), item.declared_role)),
        );
    }
    evidence.extend(current_assertions.iter().copied());
    for requirement in &record.approval_policy.required_roles {
        let actual = evidence.iter().filter(|(_, role)| *role == requirement.role).count();
        if actual < usize::from(requirement.count) {
            return Err(error(format!(
                "approval requires {} distinct '{}' assertions; found {actual}",
                requirement.count,
                requirement.role.as_str()
            )));
        }
    }
    // `/1` records and their migrated `/2` legacy-event windows cannot gain
    // historical author assertions retroactively; preserve their established
    // read/migrate contract. New `/2` approvals still fail closed (F0490).
    let require_author_evidence =
        record.schema_version == SCHEMA_VERSION && record.history[event_index].legacy_event_id.is_none();
    validate_separation(
        &record.approval_policy.separation,
        &evidence,
        require_author_evidence,
    )
}

fn validate_separation(
    rules: &SeparationRules,
    evidence: &BTreeSet<(&str, DeclaredRole)>,
    require_author_evidence: bool,
) -> Result<(), ForgeError> {
    let actors_for = |role| {
        evidence
            .iter()
            .filter_map(|(actor, actual)| (*actual == role).then_some(*actor))
            .collect::<BTreeSet<_>>()
    };
    let authors = actors_for(DeclaredRole::Author);
    let reviewers = actors_for(DeclaredRole::Reviewer);
    let approvers = actors_for(DeclaredRole::Approver);
    if require_author_evidence
        && (rules.author_reviewer || rules.author_approver)
        && authors.is_empty()
    {
        return Err(error(
            "approval requires at least one declared author assertion when author separation is enabled",
        ));
    }
    for (required, left, right, label) in [
        (rules.author_reviewer, &authors, &reviewers, "author/reviewer"),
        (rules.author_approver, &authors, &approvers, "author/approver"),
        (rules.reviewer_approver, &reviewers, &approvers, "reviewer/approver"),
    ] {
        if required && left.intersection(right).next().is_some() {
            return Err(error(format!("approval violates {label} separation")));
        }
    }
    Ok(())
}

fn validate_fingerprint(path: &str, value: &ArtifactFingerprint) -> Result<(), ForgeError> {
    non_empty(&format!("{path}.path"), &value.path)?;
    validate_artifact_path_shape(&format!("{path}.path"), &value.path)?;
    validate_hash(&format!("{path}.sha256"), &value.sha256)?;
    if let Some(kind) = &value.oscal_type {
        non_empty(&format!("{path}.oscal_type"), kind)?;
    }
    if let Some(root_uuid) = &value.root_uuid {
        Uuid::parse_str(root_uuid)
            .map_err(|source| error(format!("{path}.root_uuid is not a UUID: {source}")))?;
    }
    Ok(())
}

/// Stored artifact paths must be relative: joining them onto the record
/// directory must keep the record directory as the anchor, so absolute paths
/// and Windows drive prefixes (which discard the anchor entirely when joined)
/// and `.` components (never produced by relative-path storage) are rejected.
/// Parent-directory components remain permitted because records may
/// legitimately reference artifacts in sibling directories.
fn validate_artifact_path_shape(path: &str, value: &str) -> Result<(), ForgeError> {
    let drive_prefixed = value.as_bytes().get(1) == Some(&b':')
        && value.as_bytes().first().is_some_and(u8::is_ascii_alphabetic);
    let relative = !drive_prefixed
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::ParentDir));
    if !relative {
        return Err(error(format!(
            "{path} must be a relative path without '.', absolute, or drive-prefix components \
             ('{}')",
            bounded(value)
        )));
    }
    Ok(())
}

fn validate_fingerprint_set(path: &str, set: &FingerprintSet) -> Result<(), ForgeError> {
    validate_hash(&format!("{path}.source_sha256"), &set.source_sha256)?;
    if set.generated_artifacts.len() > MAX_ARTIFACTS {
        return Err(error(format!(
            "{path}.generated_artifacts exceeds the {MAX_ARTIFACTS} entry limit"
        )));
    }
    let mut prior: Option<&str> = None;
    for (index, artifact) in set.generated_artifacts.iter().enumerate() {
        non_empty(&format!("{path}.generated_artifacts[{index}].path"), &artifact.path)?;
        validate_hash(&format!("{path}.generated_artifacts[{index}].sha256"), &artifact.sha256)?;
        if prior.is_some_and(|value| value >= artifact.path.as_str()) {
            return Err(error(format!(
                "{path}.generated_artifacts must be unique and sorted by path"
            )));
        }
        prior = Some(&artifact.path);
    }
    Ok(())
}

fn validate_hash(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(error(format!("{path} must be 64 lowercase hexadecimal characters")));
    }
    Ok(())
}

/// Calculate the deterministic UUID v5 event ID.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] if the bounded event evidence cannot be serialized.
pub fn event_id(record: &LifecycleRecord, event: &TransitionEvent) -> Result<String, ForgeError> {
    match record.schema_version.as_str() {
        LEGACY_SCHEMA_VERSION => legacy_event_id(record, event),
        SCHEMA_VERSION => context_event_id(record, event),
        _ => Err(error("cannot calculate an event ID for an unsupported schema version")),
    }
}

fn context_event_id(
    record: &LifecycleRecord,
    event: &TransitionEvent,
) -> Result<String, ForgeError> {
    #[derive(Serialize)]
    struct Seed<'a> {
        schema_version: &'a str,
        policy: &'a PolicyIdentity,
        parties: &'a [Party],
        approval_policy: &'a ApprovalPolicy,
        review: &'a ReviewSchedule,
        sequence: u32,
        legacy_event_id: &'a Option<String>,
        previous_state: LifecycleState,
        next_state: LifecycleState,
        actor_key: &'a str,
        declared_role: DeclaredRole,
        timestamp: &'a str,
        rationale: &'a str,
        fingerprints: &'a FingerprintSet,
        assertions: &'a [ActorAssertion],
        impact_finding_ids: &'a [String],
        replacement: &'a Option<PolicyReference>,
    }
    let seed = serde_json::to_vec(&Seed {
        schema_version: &record.schema_version,
        policy: &record.policy,
        parties: &record.parties,
        approval_policy: &record.approval_policy,
        review: &record.review,
        sequence: event.sequence,
        legacy_event_id: &event.legacy_event_id,
        previous_state: event.previous_state,
        next_state: event.next_state,
        actor_key: &event.actor_key,
        declared_role: event.declared_role,
        timestamp: &event.timestamp,
        rationale: &event.rationale,
        fingerprints: &event.fingerprints,
        assertions: &event.assertions,
        impact_finding_ids: &event.impact_finding_ids,
        replacement: &event.replacement,
    })
    .map_err(|source| error(format!("cannot serialize deterministic event evidence: {source}")))?;
    Ok(Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &seed).to_string())
}

/// Calculate a legacy `/1` event ID for validation and migration tooling.
///
/// New records must use [`event_id`] with [`SCHEMA_VERSION`].
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] if legacy evidence serialization fails.
#[doc(hidden)]
pub fn legacy_event_id(
    record: &LifecycleRecord,
    event: &TransitionEvent,
) -> Result<String, ForgeError> {
    #[derive(Serialize)]
    struct LegacySeed<'a> {
        schema_version: &'a str,
        policy_key: &'a str,
        version_key: &'a str,
        sequence: u32,
        previous_state: LifecycleState,
        next_state: LifecycleState,
        actor_key: &'a str,
        declared_role: DeclaredRole,
        timestamp: &'a str,
        rationale: &'a str,
        fingerprints: &'a FingerprintSet,
        assertions: &'a [ActorAssertion],
        impact_finding_ids: &'a [String],
        replacement: &'a Option<PolicyReference>,
    }
    let seed = serde_json::to_vec(&LegacySeed {
        schema_version: LEGACY_SCHEMA_VERSION,
        policy_key: &record.policy.policy_key,
        version_key: &record.policy.version_key,
        sequence: event.sequence,
        previous_state: event.previous_state,
        next_state: event.next_state,
        actor_key: &event.actor_key,
        declared_role: event.declared_role,
        timestamp: &event.timestamp,
        rationale: &event.rationale,
        fingerprints: &event.fingerprints,
        assertions: &event.assertions,
        impact_finding_ids: &event.impact_finding_ids,
        replacement: &event.replacement,
    })
    .map_err(|source| error(format!("cannot serialize legacy event evidence: {source}")))?;
    Ok(Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &seed).to_string())
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() {
        return Err(error(format!("{path} must not be empty")));
    }
    if value.len() > MAX_STRING_BYTES {
        return Err(error(format!("{path} exceeds the {MAX_STRING_BYTES} byte limit")));
    }
    Ok(())
}

fn bounded(value: &str) -> String {
    value.chars().take(128).flat_map(char::escape_default).collect()
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Lifecycle(message.into())
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value.to_string())))
    }
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }
    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }
    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = object.next_entry::<String, StrictValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(de::Error::custom(format!("duplicate object key '{key}'")));
            }
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

fn enforce_value_bounds(value: &Value, path: &str, depth: usize) -> Result<(), ForgeError> {
    if depth > MAX_DEPTH {
        return Err(error(format!("{path} exceeds maximum nesting depth {MAX_DEPTH}")));
    }
    match value {
        Value::String(text) if text.len() > MAX_STRING_BYTES => {
            Err(error(format!("{path} string exceeds the {MAX_STRING_BYTES} byte limit")))
        }
        Value::Array(values) => {
            if values.len() > MAX_COLLECTION_ITEMS {
                return Err(error(format!("{path} exceeds the {MAX_COLLECTION_ITEMS} item limit")));
            }
            for (index, item) in values.iter().enumerate() {
                enforce_value_bounds(item, &format!("{path}[{index}]"), depth + 1)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            if values.len() > MAX_COLLECTION_ITEMS {
                return Err(error(format!("{path} exceeds the {MAX_COLLECTION_ITEMS} key limit")));
            }
            for (key, item) in values {
                enforce_value_bounds(item, &format!("{path}.{key}"), depth + 1)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fingerprints() -> FingerprintSet {
        FingerprintSet { source_sha256: "a".repeat(64), generated_artifacts: Vec::new() }
    }

    fn event(
        sequence: u32,
        previous_state: LifecycleState,
        next_state: LifecycleState,
        actor_key: &str,
        declared_role: DeclaredRole,
        legacy_event_id: bool,
    ) -> TransitionEvent {
        TransitionEvent {
            sequence,
            event_id: String::new(),
            legacy_event_id: legacy_event_id.then(String::new),
            previous_state,
            next_state,
            actor_key: actor_key.to_string(),
            declared_role,
            timestamp: format!("2026-08-26T12:00:0{sequence}+00:00"),
            rationale: "reviewed".to_string(),
            fingerprints: fingerprints(),
            assertions: Vec::new(),
            impact_finding_ids: Vec::new(),
            replacement: None,
        }
    }

    fn record(
        state: LifecycleState,
        required_roles: Vec<RoleRequirement>,
        separation: SeparationRules,
        history: Vec<TransitionEvent>,
    ) -> LifecycleRecord {
        LifecycleRecord {
            schema_version: SCHEMA_VERSION.to_string(),
            policy: PolicyIdentity {
                policy_key: "policy".to_string(),
                version_key: "1.0".to_string(),
                title: "Policy".to_string(),
                owner_keys: vec!["owner".to_string()],
                source: ArtifactFingerprint {
                    path: "policy.md".to_string(),
                    sha256: "a".repeat(64),
                    oscal_type: None,
                    root_uuid: None,
                },
                generated_artifacts: Vec::new(),
            },
            parties: vec![
                Party { key: "owner".to_string(), roles: vec![DeclaredRole::Owner] },
                Party { key: "author".to_string(), roles: vec![DeclaredRole::Author] },
                Party { key: "reviewer".to_string(), roles: vec![DeclaredRole::Reviewer] },
                Party { key: "approver".to_string(), roles: vec![DeclaredRole::Approver] },
            ],
            approval_policy: ApprovalPolicy {
                schema_version: APPROVAL_POLICY_VERSION.to_string(),
                required_roles,
                separation,
            },
            review: ReviewSchedule {
                cadence_days: 30,
                next_review_date: NaiveDate::from_ymd_opt(2026, 9, 25).expect("valid date"),
                due_soon_days: 7,
                timezone_policy: TimezonePolicy::DateOnly,
            },
            state,
            replaced_by: None,
            history,
        }
    }

    fn sign_events(record: &mut LifecycleRecord) {
        for index in 0..record.history.len() {
            let mut event = record.history[index].clone();
            if event.legacy_event_id.is_some() {
                event.legacy_event_id =
                    Some(legacy_event_id(record, &event).expect("legacy event ID"));
            }
            event.event_id = event_id(record, &event).expect("current event ID");
            record.history[index] = event;
        }
    }

    #[test]
    fn current_record_legacy_event_requires_unique_sorted_assertions() {
        let mut in_review = event(
            1,
            LifecycleState::Draft,
            LifecycleState::InReview,
            "reviewer",
            DeclaredRole::Reviewer,
            true,
        );
        in_review.assertions = vec![
            ActorAssertion { actor_key: "author".to_string(), declared_role: DeclaredRole::Author },
            ActorAssertion { actor_key: "author".to_string(), declared_role: DeclaredRole::Author },
        ];
        let mut record = record(
            LifecycleState::InReview,
            vec![RoleRequirement { role: DeclaredRole::Reviewer, count: 1 }],
            SeparationRules::default(),
            vec![in_review],
        );
        sign_events(&mut record);

        let error = validate(&record).expect_err("legacy-carried assertions must be canonical");
        assert!(error.to_string().contains("assertions must be unique and sorted"), "{error}");
    }

    #[test]
    fn legacy_carried_window_preserves_historical_author_evidence_contract() {
        let mut record = record(
            LifecycleState::Approved,
            vec![
                RoleRequirement { role: DeclaredRole::Reviewer, count: 1 },
                RoleRequirement { role: DeclaredRole::Approver, count: 1 },
            ],
            SeparationRules {
                author_reviewer: false,
                author_approver: true,
                reviewer_approver: false,
            },
            vec![
                event(
                    1,
                    LifecycleState::Draft,
                    LifecycleState::InReview,
                    "reviewer",
                    DeclaredRole::Reviewer,
                    true,
                ),
                event(
                    2,
                    LifecycleState::InReview,
                    LifecycleState::Approved,
                    "approver",
                    DeclaredRole::Approver,
                    true,
                ),
            ],
        );
        sign_events(&mut record);

        assert!(
            validate(&record).is_ok(),
            "legacy evidence must remain readable/migratable"
        );
    }

    #[test]
    fn state_machine_matches_prd() {
        let states = [
            LifecycleState::Draft,
            LifecycleState::InReview,
            LifecycleState::Approved,
            LifecycleState::Superseded,
            LifecycleState::Retired,
        ];
        let allowed = BTreeSet::from([
            (LifecycleState::Draft, LifecycleState::InReview),
            (LifecycleState::Draft, LifecycleState::Retired),
            (LifecycleState::InReview, LifecycleState::Draft),
            (LifecycleState::InReview, LifecycleState::Approved),
            (LifecycleState::InReview, LifecycleState::Retired),
            (LifecycleState::Approved, LifecycleState::InReview),
            (LifecycleState::Approved, LifecycleState::Superseded),
            (LifecycleState::Approved, LifecycleState::Retired),
            (LifecycleState::Superseded, LifecycleState::Retired),
        ]);
        for previous in states {
            for next in states {
                assert_eq!(previous.permits(next), allowed.contains(&(previous, next)));
            }
        }
    }

    #[test]
    fn artifact_paths_must_be_relative() {
        let fingerprint = |path: &str| ArtifactFingerprint {
            path: path.to_string(),
            sha256: "a".repeat(64),
            oscal_type: None,
            root_uuid: None,
        };
        validate_fingerprint("$.policy.source", &fingerprint("policy.md"))
            .expect("plain relative path is valid");
        validate_fingerprint("$.policy.source", &fingerprint("output/catalog.json"))
            .expect("nested relative path is valid");
        validate_fingerprint("$.policy.source", &fingerprint("../artifacts/policy.md"))
            .expect("sibling-directory path is valid");
        for raw in ["/etc/passwd", r"C:\\evil\\policy.md", "./policy.md", "."] {
            let error = validate_fingerprint("$.policy.source", &fingerprint(raw)).expect_err(raw);
            assert!(error.to_string().contains("must be a relative path"), "{raw}: {error}");
        }
    }

    #[test]
    fn duplicate_decoded_key_is_rejected() {
        let error = parse(br#"{"schema_version":"forge.policy-lifecycle/1","schema_version":"forge.policy-lifecycle/1"}"#)
            .expect_err("duplicate must fail");
        assert!(error.to_string().contains("duplicate object key 'schema_version'"));
    }
}
