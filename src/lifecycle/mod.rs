//! Deterministic local policy lifecycle workflows (PRD 058).

pub mod record;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli::{LifecycleGate, LifecycleOutputFormat};
use crate::{ForgeError, io, validate};
use record::{
    APPROVAL_POLICY_VERSION, ActorAssertion, ApprovalPolicy, ArtifactFingerprint, DeclaredRole,
    FingerprintSet, LEGACY_SCHEMA_VERSION, LifecycleRecord, LifecycleState, NamedHash, Party,
    PolicyIdentity, PolicyReference, ReviewSchedule, RoleRequirement, SCHEMA_VERSION,
    SeparationRules, TimezonePolicy, TransitionEvent,
};

const TRUST_BOUNDARY: &str =
    "actor identities and authority are declared locally and are not authenticated by FORGE";
const STATUS_SCHEMA_VERSION: &str = "forge.policy-lifecycle-status/1";
const QUEUE_SCHEMA_VERSION: &str = "forge.policy-lifecycle-queue/1";
const ATTESTATION_SCHEMA_VERSION: &str = "forge.policy-approval-attestation/1";

/// Inputs for a deterministic lifecycle scaffold.
pub struct InitOptions<'a> {
    pub source: &'a Path,
    pub artifacts: &'a [PathBuf],
    pub output: &'a Path,
    pub policy_key: &'a str,
    pub version_key: &'a str,
    pub title: &'a str,
    pub owners: &'a [String],
    pub parties: &'a [String],
    pub next_review: NaiveDate,
    pub cadence_days: u16,
    pub due_soon_days: u16,
    pub required_reviewers: u16,
    pub required_approvers: u16,
    pub separate_author_reviewer: bool,
    pub separate_author_approver: bool,
    pub separate_reviewer_approver: bool,
}

/// Inputs for a proposed or applied state transition.
pub struct TransitionOptions<'a> {
    pub record_path: &'a Path,
    pub next_state: LifecycleState,
    pub actor_key: &'a str,
    pub role: DeclaredRole,
    pub timestamp: &'a str,
    pub rationale: &'a str,
    pub assertions: &'a [String],
    pub impact_finding_ids: &'a [String],
    pub replacement_policy_key: Option<&'a str>,
    pub replacement_version_key: Option<&'a str>,
    pub apply: bool,
    pub output: Option<&'a Path>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct StatusReport {
    schema_version: &'static str,
    policy_key: String,
    version_key: String,
    state: LifecycleState,
    derived_status: String,
    owner_keys: Vec<String>,
    next_review_date: NaiveDate,
    as_of: Option<NaiveDate>,
    blockers: Vec<String>,
    current_fingerprints: FingerprintSet,
    approved_fingerprints: Option<FingerprintSet>,
    artifact_identity_changes: Vec<String>,
    event_ids: Vec<String>,
    impact_finding_ids: Vec<String>,
    replaced_by: Option<PolicyReference>,
    trust_boundary: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueReport {
    schema_version: &'static str,
    as_of: NaiveDate,
    groups: Vec<QueueGroup>,
    trust_boundary: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueGroup {
    owner_key: String,
    next_review_date: NaiveDate,
    items: Vec<QueueItem>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct QueueItem {
    policy_key: String,
    version_key: String,
    state: LifecycleState,
    derived_status: String,
    blockers: Vec<String>,
    impact_finding_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ApprovalAttestation {
    schema_version: &'static str,
    policy: PolicyReference,
    approved_event_id: String,
    approved_at: String,
    assertions: Vec<ActorAssertion>,
    fingerprints: FingerprintSet,
    approval_policy: ApprovalPolicy,
    next_review_date: NaiveDate,
    unsigned: bool,
    trust_boundary: &'static str,
}

struct CurrentArtifacts {
    fingerprints: FingerprintSet,
    identity_changes: Vec<String>,
}

/// Create a closed, versioned draft record tied to current artifact bytes.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] if inputs, artifact identity, the
/// generated contract, destination safety, serialization, or the atomic write fails.
pub fn execute_init(options: &InitOptions<'_>) -> Result<(), ForgeError> {
    if options.required_reviewers == 0 || options.required_approvers == 0 {
        return Err(error("required reviewer and approver counts must be greater than zero"));
    }
    validate_write_target(options.output)?;
    let record_dir = record_directory(options.output)?;
    let source = fingerprint(options.source, &record_dir, false)?;
    let mut generated_artifacts = options
        .artifacts
        .iter()
        .map(|path| fingerprint(path, &record_dir, true))
        .collect::<Result<Vec<_>, _>>()?;
    generated_artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let mut owner_keys = options.owners.to_vec();
    owner_keys.sort();
    owner_keys.dedup();
    if owner_keys.is_empty() {
        return Err(error("at least one owner is required"));
    }
    let mut parties = parse_parties(options.parties)?;
    for owner in &owner_keys {
        match parties.iter_mut().find(|party| party.key == *owner) {
            Some(party) if !party.roles.contains(&DeclaredRole::Owner) => {
                party.roles.push(DeclaredRole::Owner);
                party.roles.sort_unstable();
            }
            Some(_) => {}
            None => parties.push(Party { key: owner.clone(), roles: vec![DeclaredRole::Owner] }),
        }
    }
    parties.sort_by(|left, right| left.key.cmp(&right.key));
    let record = LifecycleRecord {
        schema_version: SCHEMA_VERSION.to_string(),
        policy: PolicyIdentity {
            policy_key: options.policy_key.to_string(),
            version_key: options.version_key.to_string(),
            title: options.title.to_string(),
            owner_keys,
            source,
            generated_artifacts,
        },
        parties,
        approval_policy: ApprovalPolicy {
            schema_version: APPROVAL_POLICY_VERSION.to_string(),
            required_roles: vec![
                RoleRequirement { role: DeclaredRole::Reviewer, count: options.required_reviewers },
                RoleRequirement { role: DeclaredRole::Approver, count: options.required_approvers },
            ],
            separation: SeparationRules {
                author_reviewer: options.separate_author_reviewer,
                author_approver: options.separate_author_approver,
                reviewer_approver: options.separate_reviewer_approver,
            },
        },
        review: ReviewSchedule {
            cadence_days: options.cadence_days,
            next_review_date: options.next_review,
            due_soon_days: options.due_soon_days,
            timezone_policy: TimezonePolicy::DateOnly,
        },
        state: LifecycleState::Draft,
        replaced_by: None,
        history: Vec::new(),
    };
    record::validate(&record)?;
    let rendered = render_record(&record)?;
    let inputs =
        std::iter::once(options.source).chain(options.artifacts.iter().map(PathBuf::as_path));
    for input in inputs {
        if paths_alias(options.output, input)? {
            return Err(error(format!(
                "lifecycle output '{}' aliases an input artifact",
                options.output.display()
            )));
        }
    }
    io::write_atomic(options.output, rendered.as_bytes())
}

/// Validate records, current artifact identity, and supplied supersession graph.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] when a record, current artifact,
/// portfolio relationship, output destination, serialization, or write is invalid.
pub fn execute_check(
    paths: &[PathBuf],
    format: &LifecycleOutputFormat,
    output: Option<&Path>,
) -> Result<bool, ForgeError> {
    let loaded = load_portfolio(paths)?;
    validate_report_destination(output, &loaded)?;
    validate_portfolio(&loaded)?;
    let reports = loaded
        .iter()
        .map(|(path, record)| status_report(path, record, None))
        .collect::<Result<Vec<_>, _>>()?;
    let action_required = reports.iter().any(|report| {
        report
            .blockers
            .iter()
            .any(|item| matches!(item.as_str(), "approved-drifted" | "artifact-identity-changed"))
    });
    write_reports(&reports, format, output)?;
    Ok(action_required)
}

/// Convert a validated legacy lifecycle record into the current context-bound schema.
///
/// The source record remains unchanged. Each migrated event preserves its former deterministic ID
/// in `legacy_event_id` and receives a new current-schema ID.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] if the input is not a valid legacy
/// record, the output aliases lifecycle evidence, migration validation fails, or writing fails.
pub fn execute_migrate(record_path: &Path, output: &Path) -> Result<(), ForgeError> {
    let (_, mut record) = load_record(record_path)?;
    if record.schema_version != LEGACY_SCHEMA_VERSION {
        return Err(error(format!(
            "lifecycle migration requires {LEGACY_SCHEMA_VERSION}; found '{}'",
            record.schema_version
        )));
    }
    validate_report_destination(Some(output), &[(record_path.to_path_buf(), record.clone())])?;
    for event in &mut record.history {
        event.legacy_event_id = Some(std::mem::take(&mut event.event_id));
    }
    record.schema_version = SCHEMA_VERSION.to_string();
    for index in 0..record.history.len() {
        let event_id = record::event_id(&record, &record.history[index])?;
        record.history[index].event_id = event_id;
    }
    record::validate(&record)?;
    io::write_atomic(output, render_record(&record)?.as_bytes())
}

/// Emit deterministic lifecycle status for an explicit date.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] when a record, artifact, portfolio,
/// date calculation, report serialization, destination, or write is invalid.
pub fn execute_status(
    paths: &[PathBuf],
    as_of: NaiveDate,
    format: &LifecycleOutputFormat,
    gate: &LifecycleGate,
    output: Option<&Path>,
) -> Result<bool, ForgeError> {
    let loaded = load_portfolio(paths)?;
    validate_report_destination(output, &loaded)?;
    validate_portfolio(&loaded)?;
    let mut reports = loaded
        .iter()
        .map(|(path, record)| status_report(path, record, Some(as_of)))
        .collect::<Result<Vec<_>, _>>()?;
    reports.sort_by(|left, right| {
        left.next_review_date
            .cmp(&right.next_review_date)
            .then_with(|| left.owner_keys.cmp(&right.owner_keys))
            .then_with(|| left.policy_key.cmp(&right.policy_key))
            .then_with(|| left.version_key.cmp(&right.version_key))
    });
    let action_required = gate_action_required(&reports, gate);
    write_reports(&reports, format, output)?;
    Ok(action_required)
}

/// Emit a deterministic machine-readable review queue grouped by owner and due date.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] when a record, artifact, portfolio,
/// date calculation, destination, serialization, or write is invalid.
pub fn execute_queue(
    paths: &[PathBuf],
    as_of: NaiveDate,
    format: &LifecycleOutputFormat,
    gate: &LifecycleGate,
    output: Option<&Path>,
) -> Result<bool, ForgeError> {
    let loaded = load_portfolio(paths)?;
    validate_report_destination(output, &loaded)?;
    validate_portfolio(&loaded)?;
    let reports = loaded
        .iter()
        .map(|(path, record)| status_report(path, record, Some(as_of)))
        .collect::<Result<Vec<_>, _>>()?;
    let action_required = gate_action_required(&reports, gate);
    let queue = build_queue(reports, as_of);
    write_queue(&queue, format, output)?;
    Ok(action_required)
}

/// Emit deterministic unsigned approval evidence suitable for external signing.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] when the record is not currently
/// approved and clean, its approval evidence is invalid, the destination aliases an input, or
/// serialization/write fails.
pub fn execute_attest(record_path: &Path, output: Option<&Path>) -> Result<(), ForgeError> {
    let (_, record) = load_record(record_path)?;
    if record.schema_version == LEGACY_SCHEMA_VERSION {
        return Err(error("legacy lifecycle records must be migrated before attestation"));
    }
    let loaded = vec![(record_path.to_path_buf(), record.clone())];
    validate_report_destination(output, &loaded)?;
    if record.state != LifecycleState::Approved {
        return Err(error("unsigned attestation requires current approved state"));
    }
    let current = current_fingerprints(record_path, &record)?;
    let approved_index = record
        .history
        .iter()
        .rposition(|event| event.next_state == LifecycleState::Approved)
        .ok_or_else(|| error("approved record lacks an approval event"))?;
    let approved = &record.history[approved_index];
    if approved.fingerprints != current {
        return Err(error("unsigned attestation refuses approved-drifted artifact bytes"));
    }
    let review_start = record.history[..approved_index]
        .iter()
        .rposition(|event| event.next_state == LifecycleState::InReview)
        .ok_or_else(|| error("approval event lacks a preceding review event"))?;
    let mut assertions = BTreeSet::new();
    for event in &record.history[review_start..=approved_index] {
        assertions.insert((event.actor_key.clone(), event.declared_role));
        assertions.extend(
            event.assertions.iter().map(|item| (item.actor_key.clone(), item.declared_role)),
        );
    }
    let attestation = ApprovalAttestation {
        schema_version: ATTESTATION_SCHEMA_VERSION,
        policy: PolicyReference {
            policy_key: record.policy.policy_key.clone(),
            version_key: record.policy.version_key.clone(),
        },
        approved_event_id: approved.event_id.clone(),
        approved_at: approved.timestamp.clone(),
        assertions: assertions
            .into_iter()
            .map(|(actor_key, declared_role)| ActorAssertion { actor_key, declared_role })
            .collect(),
        fingerprints: approved.fingerprints.clone(),
        approval_policy: record.approval_policy.clone(),
        next_review_date: record.review.next_review_date,
        unsigned: true,
        trust_boundary: TRUST_BOUNDARY,
    };
    let mut rendered = serde_json::to_string_pretty(&attestation)
        .map_err(|source| error(format!("cannot serialize approval attestation: {source}")))?;
    rendered.push('\n');
    crate::cli::output::write_output(&rendered, output)
}

/// Propose a transition, or append exactly one event with `--apply`.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] or [`ForgeError::Io`] if the transition, evidence,
/// append-only record, destination, concurrent-write check, serialization, or write is invalid.
pub fn execute_transition(options: &TransitionOptions<'_>) -> Result<(), ForgeError> {
    if options.apply && options.output.is_some() {
        return Err(error("--output cannot be used with --apply"));
    }
    let (original, mut record) = load_record(options.record_path)?;
    if record.schema_version == LEGACY_SCHEMA_VERSION {
        return Err(error("legacy lifecycle records must be migrated before transition"));
    }
    if !record.state.permits(options.next_state) {
        return Err(error(format!(
            "invalid transition {} -> {}",
            record.state.as_str(),
            options.next_state.as_str()
        )));
    }
    let replacement = match (options.replacement_policy_key, options.replacement_version_key) {
        (Some(policy_key), Some(version_key)) => Some(PolicyReference {
            policy_key: policy_key.to_string(),
            version_key: version_key.to_string(),
        }),
        (None, None) => None,
        _ => return Err(error("replacement policy and version keys must be supplied together")),
    };
    if options.next_state == LifecycleState::Superseded && replacement.is_none() {
        return Err(error("supersession requires replacement policy and version keys"));
    }
    let mut assertions = parse_assertions(options.assertions)?;
    assertions.sort();
    assertions.dedup();
    let fingerprints = current_fingerprints(options.record_path, &record)?;
    let sequence = u32::try_from(record.history.len() + 1)
        .map_err(|_| error("event sequence exceeds supported range"))?;
    let mut event = TransitionEvent {
        sequence,
        event_id: String::new(),
        legacy_event_id: None,
        previous_state: record.state,
        next_state: options.next_state,
        actor_key: options.actor_key.to_string(),
        declared_role: options.role,
        timestamp: options.timestamp.to_string(),
        rationale: options.rationale.to_string(),
        fingerprints,
        assertions,
        impact_finding_ids: {
            let mut values = options.impact_finding_ids.to_vec();
            values.sort();
            values.dedup();
            values
        },
        replacement: replacement.clone(),
    };
    event.event_id = record::event_id(&record, &event)?;
    record.state = options.next_state;
    record.replaced_by = replacement.or(record.replaced_by);
    record.history.push(event);
    record::validate(&record)?;
    let rendered = render_record(&record)?;
    if options.apply {
        validate_mutation_target(options.record_path)?;
        let current = std::fs::read(options.record_path).map_err(|source| {
            error(format!("cannot re-read lifecycle record before write: {source}"))
        })?;
        if current != original {
            return Err(error("lifecycle record changed while transition was being prepared"));
        }
        io::write_atomic(options.record_path, rendered.as_bytes())
    } else {
        if let Some(path) = options.output {
            validate_write_target(path)?;
            if paths_alias(path, options.record_path)? {
                return Err(error(
                    "proposal output aliases the lifecycle record; use --apply to mutate",
                ));
            }
            let base = options.record_path.parent().unwrap_or_else(|| Path::new("."));
            for artifact in std::iter::once(&record.policy.source)
                .chain(record.policy.generated_artifacts.iter())
            {
                if paths_alias(path, &base.join(&artifact.path))? {
                    return Err(error("proposal output aliases a lifecycle artifact"));
                }
            }
        }
        crate::cli::output::write_output(&rendered, options.output)
    }
}

fn parse_parties(specs: &[String]) -> Result<Vec<Party>, ForgeError> {
    let mut parties = Vec::new();
    for spec in specs {
        let (key, roles) = spec
            .split_once('=')
            .ok_or_else(|| error(format!("invalid party '{spec}'; expected KEY=ROLE[,ROLE]")))?;
        let mut roles = roles.split(',').map(parse_role).collect::<Result<Vec<_>, _>>()?;
        roles.sort_unstable();
        roles.dedup();
        parties.push(Party { key: key.to_string(), roles });
    }
    Ok(parties)
}

fn parse_assertions(specs: &[String]) -> Result<Vec<ActorAssertion>, ForgeError> {
    specs
        .iter()
        .map(|spec| {
            let (actor_key, role) = spec
                .split_once('=')
                .ok_or_else(|| error(format!("invalid assertion '{spec}'; expected ACTOR=ROLE")))?;
            Ok(ActorAssertion {
                actor_key: actor_key.to_string(),
                declared_role: parse_role(role)?,
            })
        })
        .collect()
}

fn parse_role(value: &str) -> Result<DeclaredRole, ForgeError> {
    match value {
        "author" => Ok(DeclaredRole::Author),
        "reviewer" => Ok(DeclaredRole::Reviewer),
        "approver" => Ok(DeclaredRole::Approver),
        "owner" => Ok(DeclaredRole::Owner),
        "custodian" => Ok(DeclaredRole::Custodian),
        _ => Err(error(format!("unsupported declared role '{value}'"))),
    }
}

fn fingerprint(
    path: &Path,
    record_dir: &Path,
    require_oscal: bool,
) -> Result<ArtifactFingerprint, ForgeError> {
    validate_input_path(path)?;
    io::check_file_size(path, io::MAX_FILE_SIZE)
        .map_err(|source| error(format!("artifact '{}': {source}", path.display())))?;
    let bytes = std::fs::read(path)
        .map_err(|source| error(format!("cannot read artifact '{}': {source}", path.display())))?;
    let (oscal_type, root_uuid) = if require_oscal {
        let value: Value = serde_json::from_slice(&bytes).map_err(|source| {
            error(format!("generated artifact '{}' is not JSON: {source}", path.display()))
        })?;
        let model = validate::detect_model_type(&value).map_err(|source| {
            error(format!("generated artifact '{}': {source}", path.display()))
        })?;
        let root = model_root(model);
        let uuid =
            value.get(root).and_then(|item| item.get("uuid")).and_then(Value::as_str).ok_or_else(
                || error(format!("generated artifact '{}' lacks root uuid", path.display())),
            )?;
        uuid::Uuid::parse_str(uuid).map_err(|source| {
            error(format!("generated artifact '{}' root uuid is invalid: {source}", path.display()))
        })?;
        (Some(model.as_str().to_string()), Some(uuid.to_string()))
    } else {
        (None, None)
    };
    Ok(ArtifactFingerprint {
        path: relative_path(
            record_dir,
            &path.canonicalize().map_err(|source| {
                error(format!("cannot resolve '{}': {source}", path.display()))
            })?,
        )?,
        sha256: sha256(&bytes),
        oscal_type,
        root_uuid,
    })
}

fn model_root(model: crate::OscalModelType) -> &'static str {
    match model {
        crate::OscalModelType::Catalog => "catalog",
        crate::OscalModelType::ComponentDefinition => "component-definition",
        crate::OscalModelType::Profile => "profile",
        crate::OscalModelType::Mapping => "mapping-collection",
    }
}

fn current_fingerprints(
    record_path: &Path,
    record: &LifecycleRecord,
) -> Result<FingerprintSet, ForgeError> {
    let current = current_artifacts(record_path, record)?;
    if !current.identity_changes.is_empty() {
        return Err(error(format!(
            "generated artifact identity changed for: {}",
            current.identity_changes.join(", ")
        )));
    }
    Ok(current.fingerprints)
}

fn current_artifacts(
    record_path: &Path,
    record: &LifecycleRecord,
) -> Result<CurrentArtifacts, ForgeError> {
    let base = record_directory(record_path)?;
    let source_path = base.join(&record.policy.source.path);
    let source = fingerprint(&source_path, &base, false)?;
    let mut generated = Vec::new();
    let mut identity_changes = Vec::new();
    for expected in &record.policy.generated_artifacts {
        let path = base.join(&expected.path);
        let actual = fingerprint(&path, &base, true)?;
        if actual.oscal_type != expected.oscal_type || actual.root_uuid != expected.root_uuid {
            identity_changes.push(expected.path.clone());
        }
        generated.push(NamedHash { path: expected.path.clone(), sha256: actual.sha256 });
    }
    generated.sort();
    Ok(CurrentArtifacts {
        fingerprints: FingerprintSet {
            source_sha256: source.sha256,
            generated_artifacts: generated,
        },
        identity_changes,
    })
}

fn approved_fingerprints(record: &LifecycleRecord) -> Option<FingerprintSet> {
    record
        .history
        .iter()
        .rev()
        .find(|event| event.next_state == LifecycleState::Approved)
        .map(|event| event.fingerprints.clone())
}

fn status_report(
    path: &Path,
    record: &LifecycleRecord,
    as_of: Option<NaiveDate>,
) -> Result<StatusReport, ForgeError> {
    let current = current_artifacts(path, record)?;
    let approved = approved_fingerprints(record);
    let mut blockers = Vec::new();
    if record.state == LifecycleState::Approved
        && (approved.as_ref().is_some_and(|value| value != &current.fingerprints)
            || !current.identity_changes.is_empty())
    {
        blockers.push("approved-drifted".to_string());
    }
    if !current.identity_changes.is_empty() {
        blockers.push("artifact-identity-changed".to_string());
    }
    let derived_status = if blockers.iter().any(|item| item == "approved-drifted") {
        "approved-drifted".to_string()
    } else {
        match as_of {
            Some(as_of) if as_of > record.review.next_review_date => {
                blockers.push("overdue".to_string());
                "overdue".to_string()
            }
            Some(as_of) => {
                let due_soon_boundary = as_of
                    .checked_add_days(chrono::Days::new(u64::from(record.review.due_soon_days)))
                    .ok_or_else(|| error("due-soon date calculation overflowed"))?;
                if record.review.next_review_date <= due_soon_boundary {
                    blockers.push("due-soon".to_string());
                    "due-soon".to_string()
                } else {
                    record.state.as_str().to_string()
                }
            }
            None => record.state.as_str().to_string(),
        }
    };
    Ok(StatusReport {
        schema_version: STATUS_SCHEMA_VERSION,
        policy_key: record.policy.policy_key.clone(),
        version_key: record.policy.version_key.clone(),
        state: record.state,
        derived_status,
        owner_keys: record.policy.owner_keys.clone(),
        next_review_date: record.review.next_review_date,
        as_of,
        blockers,
        current_fingerprints: current.fingerprints,
        approved_fingerprints: approved,
        artifact_identity_changes: current.identity_changes,
        event_ids: record.history.iter().map(|event| event.event_id.clone()).collect(),
        impact_finding_ids: record
            .history
            .iter()
            .flat_map(|event| event.impact_finding_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        replaced_by: record.replaced_by.clone(),
        trust_boundary: TRUST_BOUNDARY,
    })
}

fn gate_action_required(reports: &[StatusReport], gate: &LifecycleGate) -> bool {
    match gate {
        LifecycleGate::None => false,
        LifecycleGate::Publication => reports.iter().any(|report| {
            report.state != LifecycleState::Approved
                || report.blockers.iter().any(|blocker| {
                    matches!(
                        blocker.as_str(),
                        "approved-drifted" | "artifact-identity-changed" | "overdue"
                    )
                })
        }),
    }
}

fn build_queue(reports: Vec<StatusReport>, as_of: NaiveDate) -> QueueReport {
    let mut grouped = BTreeMap::<(String, NaiveDate), Vec<QueueItem>>::new();
    for report in reports {
        for owner_key in &report.owner_keys {
            grouped.entry((owner_key.clone(), report.next_review_date)).or_default().push(
                QueueItem {
                    policy_key: report.policy_key.clone(),
                    version_key: report.version_key.clone(),
                    state: report.state,
                    derived_status: report.derived_status.clone(),
                    blockers: report.blockers.clone(),
                    impact_finding_ids: report.impact_finding_ids.clone(),
                },
            );
        }
    }
    let groups = grouped
        .into_iter()
        .map(|((owner_key, next_review_date), mut items)| {
            items.sort_by(|left, right| {
                left.policy_key
                    .cmp(&right.policy_key)
                    .then_with(|| left.version_key.cmp(&right.version_key))
            });
            QueueGroup { owner_key, next_review_date, items }
        })
        .collect();
    QueueReport {
        schema_version: QUEUE_SCHEMA_VERSION,
        as_of,
        groups,
        trust_boundary: TRUST_BOUNDARY,
    }
}

fn load_portfolio(paths: &[PathBuf]) -> Result<Vec<(PathBuf, LifecycleRecord)>, ForgeError> {
    if paths.is_empty() {
        return Err(error("at least one --record is required"));
    }
    paths.iter().map(|path| load_record(path).map(|(_, record)| (path.clone(), record))).collect()
}

fn load_record(path: &Path) -> Result<(Vec<u8>, LifecycleRecord), ForgeError> {
    validate_input_path(path)?;
    io::check_file_size(path, record::MAX_RECORD_BYTES)
        .map_err(|source| error(format!("lifecycle record '{}': {source}", path.display())))?;
    let bytes = std::fs::read(path).map_err(|source| {
        error(format!("cannot read lifecycle record '{}': {source}", path.display()))
    })?;
    let record = record::parse(&bytes)?;
    Ok((bytes, record))
}

fn validate_portfolio(records: &[(PathBuf, LifecycleRecord)]) -> Result<(), ForgeError> {
    let mut by_key = BTreeMap::new();
    for (_, record) in records {
        let key = (record.policy.policy_key.clone(), record.policy.version_key.clone());
        if by_key.insert(key.clone(), record).is_some() {
            return Err(error(format!(
                "portfolio contains duplicate policy version '{}:{}'",
                key.0, key.1
            )));
        }
    }
    for (_, record) in records {
        if let Some(replacement) = &record.replaced_by {
            let key = (replacement.policy_key.clone(), replacement.version_key.clone());
            let target = by_key.get(&key).ok_or_else(|| {
                error(format!(
                    "supersession replacement '{}:{}' is not in the supplied portfolio",
                    key.0, key.1
                ))
            })?;
            let superseded_at = record
                .history
                .iter()
                .rfind(|event| event.next_state == LifecycleState::Superseded)
                .map(|event| event.timestamp.as_str())
                .ok_or_else(|| error("superseded record lacks transition history"))?;
            let replacement_approved_at = target
                .history
                .iter()
                .rfind(|event| event.next_state == LifecycleState::Approved)
                .map(|event| event.timestamp.as_str())
                .ok_or_else(|| {
                    error(format!("replacement '{}:{}' was never approved", key.0, key.1))
                })?;
            let superseded_at = chrono::DateTime::parse_from_rfc3339(superseded_at)
                .map_err(|source| error(format!("invalid supersession time: {source}")))?;
            let approved_at = chrono::DateTime::parse_from_rfc3339(replacement_approved_at)
                .map_err(|source| error(format!("invalid replacement approval time: {source}")))?;
            if approved_at > superseded_at {
                return Err(error("replacement approval must not be later than supersession"));
            }
        }
    }
    for start in by_key.keys() {
        let mut seen = BTreeSet::new();
        let mut current = start.clone();
        while let Some(next) = by_key.get(&current).and_then(|record| record.replaced_by.as_ref()) {
            if !seen.insert(current.clone()) {
                return Err(error(format!(
                    "supersession cycle includes '{}:{}'",
                    current.0, current.1
                )));
            }
            let next_key = (next.policy_key.clone(), next.version_key.clone());
            if next_key == *start {
                return Err(error(format!(
                    "supersession cycle includes '{}:{}'",
                    start.0, start.1
                )));
            }
            current = next_key;
            if !by_key.contains_key(&current) {
                break;
            }
        }
    }
    Ok(())
}

fn write_reports(
    reports: &[StatusReport],
    format: &LifecycleOutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let rendered = match format {
        LifecycleOutputFormat::Json => {
            let mut value = serde_json::to_string_pretty(reports)
                .map_err(|source| error(format!("cannot serialize lifecycle status: {source}")))?;
            value.push('\n');
            value
        }
        LifecycleOutputFormat::Text => {
            let mut value = String::new();
            for report in reports {
                let _ = writeln!(
                    value,
                    "{}:{} {}",
                    report.policy_key, report.version_key, report.derived_status
                );
                let _ = writeln!(value, "  state: {}", report.state.as_str());
                let _ = writeln!(value, "  owners: {}", report.owner_keys.join(","));
                let _ = writeln!(value, "  next-review: {}", report.next_review_date);
                let _ = writeln!(
                    value,
                    "  blockers: {}",
                    if report.blockers.is_empty() {
                        "none".to_string()
                    } else {
                        report.blockers.join(",")
                    }
                );
            }
            let _ = writeln!(value, "note: {TRUST_BOUNDARY}");
            value
        }
    };
    crate::cli::output::write_output(&rendered, output)
}

fn write_queue(
    queue: &QueueReport,
    format: &LifecycleOutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let rendered = match format {
        LifecycleOutputFormat::Json => {
            let mut value = serde_json::to_string_pretty(queue)
                .map_err(|source| error(format!("cannot serialize lifecycle queue: {source}")))?;
            value.push('\n');
            value
        }
        LifecycleOutputFormat::Text => {
            let mut value = String::new();
            for group in &queue.groups {
                let _ = writeln!(value, "{} {}", group.owner_key, group.next_review_date);
                for item in &group.items {
                    let _ = writeln!(
                        value,
                        "  {}:{} {}",
                        item.policy_key, item.version_key, item.derived_status
                    );
                }
            }
            let _ = writeln!(value, "note: {TRUST_BOUNDARY}");
            value
        }
    };
    crate::cli::output::write_output(&rendered, output)
}

fn validate_report_destination(
    output: Option<&Path>,
    records: &[(PathBuf, LifecycleRecord)],
) -> Result<(), ForgeError> {
    let Some(output) = output else {
        return Ok(());
    };
    validate_write_target(output)?;
    for (record_path, record) in records {
        if paths_alias(output, record_path)? {
            return Err(error("report output aliases a lifecycle record"));
        }
        let base = record_path.parent().unwrap_or_else(|| Path::new("."));
        for artifact in
            std::iter::once(&record.policy.source).chain(record.policy.generated_artifacts.iter())
        {
            if paths_alias(output, &base.join(&artifact.path))? {
                return Err(error("report output aliases a lifecycle artifact"));
            }
        }
    }
    Ok(())
}

fn render_record(record: &LifecycleRecord) -> Result<String, ForgeError> {
    let mut rendered = serde_json::to_string_pretty(record)
        .map_err(|source| error(format!("cannot serialize lifecycle record: {source}")))?;
    rendered.push('\n');
    Ok(rendered)
}

fn validate_input_path(path: &Path) -> Result<(), ForgeError> {
    reject_symlink_components(path)?;
    let metadata = std::fs::metadata(path)
        .map_err(|source| error(format!("cannot inspect '{}': {source}", path.display())))?;
    if !metadata.is_file() {
        return Err(error(format!("'{}' is not a regular file", path.display())));
    }
    Ok(())
}

fn validate_mutation_target(path: &Path) -> Result<(), ForgeError> {
    validate_input_path(path)?;
    if hard_link_count(path)? != 1 {
        return Err(error(format!("lifecycle record '{}' has hard-link aliases", path.display())));
    }
    Ok(())
}

#[cfg(unix)]
fn hard_link_count(path: &Path) -> Result<u64, ForgeError> {
    use std::os::unix::fs::MetadataExt;

    std::fs::metadata(path)
        .map(|metadata| metadata.nlink())
        .map_err(|source| error(format!("cannot inspect '{}': {source}", path.display())))
}

#[cfg(windows)]
fn hard_link_count(path: &Path) -> Result<u64, ForgeError> {
    windows_file_identity::link_count(path).map(u64::from)
}

#[cfg(not(any(unix, windows)))]
fn hard_link_count(_path: &Path) -> Result<u64, ForgeError> {
    Ok(1)
}

fn validate_write_target(path: &Path) -> Result<(), ForgeError> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(error(format!("output parent '{}' does not exist", parent.display())));
    }
    reject_symlink_components(parent)?;
    if path.exists() {
        validate_mutation_target(path)?;
    } else if std::fs::symlink_metadata(path).is_ok() {
        return Err(error(format!("output '{}' is a symlink", path.display())));
    }
    Ok(())
}

fn reject_symlink_components(path: &Path) -> Result<(), ForgeError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if let Ok(metadata) = std::fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            return Err(error(format!("path '{}' is a symlink", current.display())));
        }
    }
    Ok(())
}

fn record_directory(path: &Path) -> Result<PathBuf, ForgeError> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    parent
        .canonicalize()
        .map_err(|source| error(format!("cannot resolve record directory: {source}")))
}

fn relative_path(base: &Path, target: &Path) -> Result<String, ForgeError> {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let common = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| path_components_equal(left, right))
        .count();
    if common == 0 {
        return Err(error("cannot express artifact path relative to lifecycle record"));
    }
    let mut path = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            path.push("..");
        }
    }
    for component in &target_components[common..] {
        path.push(component.as_os_str());
    }
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| error("artifact path is not valid UTF-8 and cannot be stored in JSON"))
}

#[cfg(windows)]
fn path_components_equal(left: &Component<'_>, right: &Component<'_>) -> bool {
    normalized_windows_component(&left.as_os_str().to_string_lossy())
        == normalized_windows_component(&right.as_os_str().to_string_lossy())
}

#[cfg(not(windows))]
fn path_components_equal(left: &Component<'_>, right: &Component<'_>) -> bool {
    left == right
}

#[cfg(any(windows, test))]
fn normalized_windows_component(value: &str) -> String {
    if let Some(value) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{value}").to_ascii_lowercase()
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(value).to_ascii_lowercase()
    }
}

fn paths_alias(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    if left.exists() && right.exists() {
        return same_file_identity(left, right);
    }
    let identity = |path: &Path| -> Result<PathBuf, ForgeError> {
        if path.exists() {
            path.canonicalize().map_err(|source| error(source.to_string()))
        } else {
            let parent = path
                .parent()
                .filter(|value| !value.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let file_name = path
                .file_name()
                .ok_or_else(|| error(format!("path '{}' must name a file", path.display())))?;
            Ok(parent.canonicalize().map_err(|source| error(source.to_string()))?.join(file_name))
        }
    };
    Ok(identity(left)? == identity(right)?)
}

#[cfg(unix)]
fn same_file_identity(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    use std::os::unix::fs::MetadataExt;

    let left = std::fs::metadata(left).map_err(|source| error(source.to_string()))?;
    let right = std::fs::metadata(right).map_err(|source| error(source.to_string()))?;
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
}

#[cfg(windows)]
fn same_file_identity(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    Ok(windows_file_identity::identity(left)? == windows_file_identity::identity(right)?)
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    Ok(left.canonicalize().map_err(|source| error(source.to_string()))?
        == right.canonicalize().map_err(|source| error(source.to_string()))?)
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_file_identity {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    use crate::ForgeError;

    #[repr(C)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    pub(super) fn identity(path: &Path) -> Result<(u32, u64), ForgeError> {
        let information = information(path)?;
        let file_index =
            (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
        Ok((information.volume_serial_number, file_index))
    }

    pub(super) fn link_count(path: &Path) -> Result<u32, ForgeError> {
        Ok(information(path)?.number_of_links)
    }

    fn information(path: &Path) -> Result<ByHandleFileInformation, ForgeError> {
        let file = File::open(path).map_err(ForgeError::Io)?;
        let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
        // SAFETY: `file` remains open, its raw handle is valid, and `information` points to
        // writable storage with the documented C layout. The value is read only after success.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) }
            == 0
        {
            return Err(ForgeError::Io(io::Error::last_os_error()));
        }
        // SAFETY: the successful Windows API call initialized the complete structure.
        Ok(unsafe { information.assume_init() })
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Lifecycle(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_parser_is_closed() {
        assert_eq!(parse_role("approver").expect("role"), DeclaredRole::Approver);
        assert!(parse_role("admin").is_err());
    }

    #[test]
    fn due_date_boundaries_are_inclusive() {
        let as_of = NaiveDate::from_ymd_opt(2026, 8, 25).expect("date");
        let boundary = as_of.checked_add_days(chrono::Days::new(30)).expect("boundary");
        assert_eq!(boundary, NaiveDate::from_ymd_opt(2026, 9, 24).expect("date"));
    }

    #[test]
    fn relative_path_handles_sibling_directories() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let records = temporary.path().join("records");
        let artifacts = temporary.path().join("artifacts");
        std::fs::create_dir_all(&records).expect("records directory");
        std::fs::create_dir_all(&artifacts).expect("artifacts directory");
        let artifact = artifacts.join("policy.md");
        std::fs::write(&artifact, "policy").expect("artifact");

        assert_eq!(
            relative_path(&records, &artifact).expect("relative path"),
            Path::new("..").join("artifacts").join("policy.md").to_string_lossy()
        );
    }

    #[test]
    fn windows_component_normalization_matches_canonical_prefix_forms() {
        assert_eq!(normalized_windows_component(r"\\?\C:"), normalized_windows_component("c:"));
        assert_eq!(
            normalized_windows_component(r"\\?\UNC\Server\Share"),
            normalized_windows_component(r"\\server\share")
        );
    }

    #[test]
    fn alias_check_rejects_a_nonexistent_path_without_a_file_name() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let artifact = temporary.path().join("policy.md");
        std::fs::write(&artifact, "policy").expect("artifact");

        let failure = paths_alias(Path::new(""), &artifact).expect_err("missing file name");
        assert!(failure.to_string().contains("must name a file"));
    }

    #[cfg(windows)]
    #[test]
    fn relative_path_matches_windows_components_case_insensitively() {
        assert_eq!(
            relative_path(
                Path::new(r"\\?\C:\Work\records"),
                Path::new(r"c:\work\artifacts\policy.md")
            )
            .expect("relative path"),
            Path::new("..").join("artifacts").join("policy.md").to_string_lossy()
        );
    }

    #[cfg(windows)]
    #[test]
    fn windows_file_identity_detects_hard_links() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let original = temporary.path().join("record.json");
        let alias = temporary.path().join("record-alias.json");
        std::fs::write(&original, "record").expect("record");
        std::fs::hard_link(&original, &alias).expect("hard link");

        assert_eq!(
            windows_file_identity::identity(&original).expect("original identity"),
            windows_file_identity::identity(&alias).expect("alias identity")
        );
        assert_eq!(windows_file_identity::link_count(&original).expect("link count"), 2);
    }
}
