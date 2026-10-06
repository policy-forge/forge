//! Pure lifecycle status projection from an intrinsically validated record and captured hashes.
//!
//! Ordinary CLI workflows retain their confined artifact reads in the parent module. This
//! crate-internal seam adds no path reads, clock sampling, public route, or authority decision.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::Serialize;

use crate::ForgeError;

use super::record::{self, FingerprintSet, LifecycleRecord, LifecycleState, PolicyReference};
use super::{STATUS_SCHEMA_VERSION, TRUST_BOUNDARY, error};

/// Existing lifecycle status shape projected from validated captured facts.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StatusReport {
    /// Existing status wire version; extraction does not create a new public format.
    pub(crate) schema_version: &'static str,
    /// Locally declared policy key from the validated record.
    pub(crate) policy_key: String,
    /// Locally declared policy version key.
    pub(crate) version_key: String,
    /// Validated recorded lifecycle state, without schedule-derived replacement.
    pub(crate) state: LifecycleState,
    /// Existing state, due-soon, overdue, or approved-drifted projection.
    pub(crate) derived_status: String,
    /// Owner keys in the record order.
    pub(crate) owner_keys: Vec<String>,
    /// Date-only next review date supplied by the record.
    pub(crate) next_review_date: NaiveDate,
    /// Explicit caller date; None keeps check schedule-neutral.
    pub(crate) as_of: Option<NaiveDate>,
    /// Existing ordered reasons for drift, identity change, or schedule status.
    pub(crate) blockers: Vec<String>,
    /// Captured current hashes; no file is read by the projector.
    pub(crate) current_fingerprints: FingerprintSet,
    /// Latest approved event hashes, if any.
    pub(crate) approved_fingerprints: Option<FingerprintSet>,
    /// Captured declared paths with a changed OSCAL root identity or model.
    pub(crate) artifact_identity_changes: Vec<String>,
    /// Validated event identifiers in history order.
    pub(crate) event_ids: Vec<String>,
    /// Unique sorted impact identifiers carried by recorded review events.
    pub(crate) impact_finding_ids: Vec<String>,
    /// Validated declared successor reference, if any.
    pub(crate) replaced_by: Option<PolicyReference>,
    /// Existing qualification that declared identities and authority are unauthenticated.
    pub(crate) trust_boundary: &'static str,
}

/// Current observed hashes and identity differences supplied by the existing capture wrapper.
///
/// These facts are validated structurally; the projector does not establish their byte provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CurrentArtifacts {
    /// Source and ordered generated-artifact hashes from the same capture.
    pub(crate) fingerprints: FingerprintSet,
    /// Unique declared generated paths whose observed root identity or model changed.
    pub(crate) identity_changes: Vec<String>,
}

/// Derive the existing status report without reading files or sampling a clock.
///
/// The caller supplies the explicit schedule date and the captured artifact facts. The record
/// is intrinsically revalidated, and the generated hashes must form an exact ordered bijection
/// with its declared paths. Identity changes are a unique subset of those paths; their supplied
/// order is retained. Hash differences are valid observations and remain drift evidence.
///
/// # Errors
///
/// Returns [`ForgeError::Lifecycle`] for an invalid record, malformed or mismatched captured
/// facts, or an overflowing date-only due-soon calculation.
pub(crate) fn status_from_captured(
    record: &LifecycleRecord,
    current: &CurrentArtifacts,
    as_of: Option<NaiveDate>,
) -> Result<StatusReport, ForgeError> {
    record::validate(record)?;
    validate_captured(record, current)?;
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
        current_fingerprints: current.fingerprints.clone(),
        approved_fingerprints: approved,
        artifact_identity_changes: current.identity_changes.clone(),
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

/// Validate a bounded exact captured-path inventory before projecting any report.
fn validate_captured(
    record: &LifecycleRecord,
    current: &CurrentArtifacts,
) -> Result<(), ForgeError> {
    if !valid_hash(&current.fingerprints.source_sha256) {
        return Err(error("captured source hash must be 64 lowercase hexadecimal characters"));
    }
    let expected = record
        .policy
        .generated_artifacts
        .iter()
        .map(|artifact| artifact.path.as_str())
        .collect::<BTreeSet<_>>();
    if current.fingerprints.generated_artifacts.len() != expected.len() {
        return Err(error("captured generated artifacts must exactly match declared paths"));
    }
    let mut prior: Option<&str> = None;
    for artifact in &current.fingerprints.generated_artifacts {
        if !expected.contains(artifact.path.as_str())
            || prior.is_some_and(|path| path >= artifact.path.as_str())
        {
            return Err(error("captured generated paths must be declared, unique, and sorted"));
        }
        if !valid_hash(&artifact.sha256) {
            return Err(error(
                "captured generated hash must be 64 lowercase hexadecimal characters",
            ));
        }
        prior = Some(&artifact.path);
    }
    if current.identity_changes.len() > expected.len() {
        return Err(error("captured identity changes must be unique declared generated paths"));
    }
    let mut seen = BTreeSet::new();
    for path in &current.identity_changes {
        if !expected.contains(path.as_str()) || !seen.insert(path.as_str()) {
            return Err(error("captured identity changes must be unique declared generated paths"));
        }
    }
    Ok(())
}

/// Recognize the same lowercase SHA-256 representation required by lifecycle records.
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Retain the latest recorded approval hashes even when a later review window exists.
fn approved_fingerprints(record: &LifecycleRecord) -> Option<FingerprintSet> {
    record
        .history
        .iter()
        .rev()
        .find(|event| event.next_state == LifecycleState::Approved)
        .map(|event| event.fingerprints.clone())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::lifecycle::record::{
        APPROVAL_POLICY_VERSION, ApprovalPolicy, ArtifactFingerprint, DeclaredRole, NamedHash,
        Party, PolicyIdentity, ReviewSchedule, RoleRequirement, SCHEMA_VERSION, SeparationRules,
        TimezonePolicy, TransitionEvent,
    };

    /// Construct a synthetic declared record with two generated artifacts and no external authority.
    fn draft_record() -> LifecycleRecord {
        LifecycleRecord {
            schema_version: SCHEMA_VERSION.to_string(),
            policy: PolicyIdentity {
                policy_key: "synthetic-policy".to_string(),
                version_key: "v1".to_string(),
                title: "Synthetic captured policy".to_string(),
                owner_keys: vec!["owner".to_string()],
                source: ArtifactFingerprint {
                    path: "captured-only/policy.md".to_string(),
                    sha256: "a".repeat(64),
                    oscal_type: None,
                    root_uuid: None,
                },
                generated_artifacts: vec![
                    ArtifactFingerprint {
                        path: "captured-only/catalog-a.json".to_string(),
                        sha256: "b".repeat(64),
                        oscal_type: Some("catalog".to_string()),
                        root_uuid: Some("11111111-1111-4111-8111-111111111111".to_string()),
                    },
                    ArtifactFingerprint {
                        path: "captured-only/catalog-b.json".to_string(),
                        sha256: "c".repeat(64),
                        oscal_type: Some("catalog".to_string()),
                        root_uuid: Some("22222222-2222-4222-8222-222222222222".to_string()),
                    },
                ],
            },
            parties: vec![
                Party { key: "owner".to_string(), roles: vec![DeclaredRole::Owner] },
                Party { key: "reviewer".to_string(), roles: vec![DeclaredRole::Reviewer] },
                Party { key: "approver".to_string(), roles: vec![DeclaredRole::Approver] },
            ],
            approval_policy: ApprovalPolicy {
                schema_version: APPROVAL_POLICY_VERSION.to_string(),
                required_roles: vec![
                    RoleRequirement { role: DeclaredRole::Reviewer, count: 1 },
                    RoleRequirement { role: DeclaredRole::Approver, count: 1 },
                ],
                separation: SeparationRules::default(),
            },
            review: ReviewSchedule {
                cadence_days: 30,
                next_review_date: NaiveDate::from_ymd_opt(2026, 9, 25).expect("valid fixture date"),
                due_soon_days: 7,
                timezone_policy: TimezonePolicy::DateOnly,
            },
            state: LifecycleState::Draft,
            replaced_by: None,
            history: Vec::new(),
        }
    }

    /// Supply ordered synthetic hashes in exactly the existing capture-wrapper representation.
    fn captured(record: &LifecycleRecord) -> CurrentArtifacts {
        let mut generated_artifacts = record
            .policy
            .generated_artifacts
            .iter()
            .map(|artifact| NamedHash {
                path: artifact.path.clone(),
                sha256: artifact.sha256.clone(),
            })
            .collect::<Vec<_>>();
        generated_artifacts.sort();
        CurrentArtifacts {
            fingerprints: FingerprintSet {
                source_sha256: record.policy.source.sha256.clone(),
                generated_artifacts,
            },
            identity_changes: Vec::new(),
        }
    }

    /// Append a valid synthetic review or approval using the real deterministic event-ID contract.
    fn append_event(
        record: &mut LifecycleRecord,
        next_state: LifecycleState,
        fingerprints: &FingerprintSet,
    ) {
        let sequence = u32::try_from(record.history.len() + 1).expect("bounded fixture history");
        let (actor_key, declared_role) = match next_state {
            LifecycleState::InReview => ("reviewer", DeclaredRole::Reviewer),
            LifecycleState::Approved => ("approver", DeclaredRole::Approver),
            _ => panic!("fixture only creates review and approval events"),
        };
        let mut event = TransitionEvent {
            sequence,
            event_id: String::new(),
            legacy_event_id: None,
            previous_state: record.state,
            next_state,
            actor_key: actor_key.to_string(),
            declared_role,
            timestamp: format!("2026-09-01T{sequence:02}:00:00Z"),
            rationale: "Synthetic declared lifecycle evidence".to_string(),
            fingerprints: fingerprints.clone(),
            assertions: Vec::new(),
            impact_finding_ids: if next_state == LifecycleState::InReview {
                vec!["synthetic-impact-a".to_string()]
            } else {
                Vec::new()
            },
            replacement: None,
        };
        event.event_id = record::event_id(record, &event).expect("deterministic fixture event");
        record.history.push(event);
        record.state = next_state;
        record::validate(record).expect("intrinsically valid synthetic history");
    }

    /// Create a validated synthetic approval window without asserting authenticated acceptance.
    fn approved_record() -> LifecycleRecord {
        let mut record = draft_record();
        let fingerprints = captured(&record).fingerprints;
        append_event(&mut record, LifecycleState::InReview, &fingerprints);
        append_event(&mut record, LifecycleState::Approved, &fingerprints);
        record
    }

    /// Preserve the complete existing report shape and borrowed inputs on a captured-only call.
    #[test]
    fn captured_status_preserves_full_report_and_inputs() {
        let record = draft_record();
        let current = captured(&record);
        let record_before = record.clone();
        let current_before = current.clone();
        let report = status_from_captured(&record, &current, None).expect("valid captured status");
        assert_eq!(
            serde_json::to_value(&report).expect("status value"),
            json!({
                "schema_version": "forge.policy-lifecycle-status/1",
                "policy_key": "synthetic-policy", "version_key": "v1", "state": "draft",
                "derived_status": "draft", "owner_keys": ["owner"],
                "next_review_date": "2026-09-25", "as_of": null, "blockers": [],
                "current_fingerprints": current.fingerprints,
                "approved_fingerprints": null, "artifact_identity_changes": [],
                "event_ids": [], "impact_finding_ids": [], "replaced_by": null,
                "trust_boundary": TRUST_BOUNDARY
            })
        );
        let repeated =
            status_from_captured(&record, &current, None).expect("repeat captured status");
        assert_eq!(serde_json::to_vec(&report).unwrap(), serde_json::to_vec(&repeated).unwrap());
        assert_eq!(record, record_before);
        assert_eq!(current, current_before);
    }

    /// Keep drift priority, ordered identity differences, history, and recorded impact references.
    #[test]
    fn captured_status_keeps_approval_drift_priority_and_evidence() {
        let record = approved_record();
        let approved = captured(&record).fingerprints;
        let mut current = captured(&record);
        current.fingerprints.source_sha256 = "d".repeat(64);
        current.identity_changes = vec![
            record.policy.generated_artifacts[1].path.clone(),
            record.policy.generated_artifacts[0].path.clone(),
        ];
        let report = status_from_captured(
            &record,
            &current,
            Some(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()),
        )
        .expect("drift is a valid captured observation");
        assert_eq!(report.derived_status, "approved-drifted");
        assert_eq!(report.blockers, ["approved-drifted", "artifact-identity-changed"]);
        assert_eq!(report.approved_fingerprints, Some(approved));
        assert_eq!(report.current_fingerprints, current.fingerprints);
        assert_eq!(report.artifact_identity_changes, current.identity_changes);
        assert_eq!(
            report.event_ids,
            record.history.iter().map(|event| event.event_id.clone()).collect::<Vec<_>>()
        );
        assert_eq!(report.impact_finding_ids, ["synthetic-impact-a"]);
    }

    /// Preserve neutral checks and the existing inclusive due-soon and strict-overdue boundaries.
    #[test]
    fn captured_status_retains_date_only_boundaries() {
        let record = draft_record();
        let current = captured(&record);
        for (as_of, status, blockers) in [
            (None, "draft", Vec::<String>::new()),
            (Some("2026-09-17"), "draft", Vec::new()),
            (Some("2026-09-18"), "due-soon", vec!["due-soon".to_string()]),
            (Some("2026-09-25"), "due-soon", vec!["due-soon".to_string()]),
            (Some("2026-09-26"), "overdue", vec!["overdue".to_string()]),
        ] {
            let date = as_of.map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap());
            let report =
                status_from_captured(&record, &current, date).expect("date-only projection");
            assert_eq!(report.derived_status, status);
            assert_eq!(report.blockers, blockers);
            assert_eq!(report.as_of, date);
        }
    }

    /// Reject an overflowing due-soon date with the same existing lifecycle diagnostic.
    #[test]
    fn captured_status_date_overflow_is_a_lifecycle_error() {
        let mut record = draft_record();
        record.review.next_review_date = NaiveDate::MAX;
        let result = status_from_captured(&record, &captured(&record), Some(NaiveDate::MAX));
        assert!(matches!(
            result,
            Err(ForgeError::Lifecycle(message))
                if message == "due-soon date calculation overflowed"
        ));
    }

    /// Fail closed on malformed, partial, extra, aliased, duplicate, or unordered captured facts.
    #[test]
    fn captured_status_rejects_invalid_capture_correspondence() {
        let record = draft_record();
        let valid = captured(&record);
        let mut cases = Vec::new();
        let mut changed = valid.clone();
        changed.fingerprints.source_sha256 = "A".repeat(64);
        cases.push((changed, "captured source hash must be 64 lowercase hexadecimal characters"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts.pop();
        cases.push((changed, "captured generated artifacts must exactly match declared paths"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts.push(NamedHash {
            path: "captured-only/extra.json".to_string(),
            sha256: "a".repeat(64),
        });
        cases.push((changed, "captured generated artifacts must exactly match declared paths"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts[1] =
            changed.fingerprints.generated_artifacts[0].clone();
        cases.push((changed, "captured generated paths must be declared, unique, and sorted"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts[0].path =
            "captured-only//catalog-a.json".to_string();
        cases.push((changed, "captured generated paths must be declared, unique, and sorted"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts.reverse();
        cases.push((changed, "captured generated paths must be declared, unique, and sorted"));
        let mut changed = valid.clone();
        changed.fingerprints.generated_artifacts[0].sha256 = "0".repeat(63);
        cases
            .push((changed, "captured generated hash must be 64 lowercase hexadecimal characters"));
        let mut changed = valid.clone();
        changed.identity_changes.push(record.policy.source.path.clone());
        cases.push((changed, "captured identity changes must be unique declared generated paths"));
        let mut changed = valid.clone();
        changed.identity_changes = vec![record.policy.generated_artifacts[0].path.clone(); 2];
        cases.push((changed, "captured identity changes must be unique declared generated paths"));
        let mut changed = valid.clone();
        changed.identity_changes = vec![record.policy.generated_artifacts[0].path.clone(); 3];
        cases.push((changed, "captured identity changes must be unique declared generated paths"));
        for (current, expected_message) in cases {
            let before = current.clone();
            assert!(matches!(
                status_from_captured(&record, &current, None),
                Err(ForgeError::Lifecycle(message)) if message == expected_message
            ));
            assert_eq!(current, before);
        }
    }

    /// Revalidate directly constructed records before accepting otherwise well-shaped captures.
    #[test]
    fn captured_status_rejects_invalid_intrinsic_record() {
        let mut record = draft_record();
        let current = captured(&record);
        record.schema_version = "forge.policy-lifecycle/999".to_string();
        assert!(matches!(
            status_from_captured(&record, &current, None),
            Err(ForgeError::Lifecycle(message)) if message.starts_with("unsupported schema_version")
        ));
        record.schema_version = SCHEMA_VERSION.to_string();
        record.review.cadence_days = 0;
        assert!(matches!(
            status_from_captured(&record, &current, None),
            Err(ForgeError::Lifecycle(message))
                if message == "$.review.cadence_days must be greater than zero"
        ));
    }

    /// Support source-only legacy records and preserve accepted raw declared path spelling.
    #[test]
    fn captured_status_preserves_legacy_empty_and_raw_path_contracts() {
        let mut record = draft_record();
        record.schema_version = record::LEGACY_SCHEMA_VERSION.to_string();
        record.policy.generated_artifacts.clear();
        let report = status_from_captured(&record, &captured(&record), None).unwrap();
        assert_eq!(report.current_fingerprints.generated_artifacts.len(), 0);
        record.policy.generated_artifacts = draft_record().policy.generated_artifacts;
        record.policy.generated_artifacts[0].path = "captured-only//catalog-a.json".to_string();
        let current = captured(&record);
        let report = status_from_captured(&record, &current, None).unwrap();
        assert_eq!(report.current_fingerprints, current.fingerprints);
    }

    /// Select the latest approved window without dropping older event IDs or duplicating impacts.
    #[test]
    fn captured_status_retains_latest_approved_window() {
        let mut record = approved_record();
        let mut current = captured(&record);
        current.fingerprints.source_sha256 = "e".repeat(64);
        append_event(&mut record, LifecycleState::InReview, &current.fingerprints);
        append_event(&mut record, LifecycleState::Approved, &current.fingerprints);
        let report = status_from_captured(&record, &current, None).unwrap();
        assert_eq!(report.derived_status, "approved");
        assert_eq!(report.approved_fingerprints, Some(current.fingerprints));
        assert_eq!(report.event_ids.len(), 4);
        assert_eq!(report.impact_finding_ids, ["synthetic-impact-a"]);
    }
}
