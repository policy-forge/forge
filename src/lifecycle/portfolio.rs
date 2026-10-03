//! Shared deterministic portfolio validation over already admitted lifecycle records.
//!
//! Ordinary CLI loading and workspace captured loading feed this same validator;
//! it reads no paths and preserves the CLI duplicate, replacement and cycle errors.

use std::collections::{BTreeMap, BTreeSet};

use crate::ForgeError;

use super::error;
use super::record::{LifecycleRecord, LifecycleState};

/// Validate complete supplied policy/version identities and replacement chronology.
///
/// Each record must already have passed intrinsic lifecycle validation. The caller
/// retains its captured or ordinary CLI provenance; no filesystem access, schema
/// freshness, actor authentication or publication authority is established here.
/// Errors retain the ordinary CLI text and precedence for duplicates, unavailable
/// replacements, missing approval, chronology and cycles.
pub(crate) fn validate(records: &[&LifecycleRecord]) -> Result<(), ForgeError> {
    let mut by_key = BTreeMap::new();
    for &record in records {
        let key = (record.policy.policy_key.clone(), record.policy.version_key.clone());
        if by_key.insert(key.clone(), record).is_some() {
            return Err(error(format!(
                "portfolio contains duplicate policy version '{}:{}'",
                key.0, key.1
            )));
        }
    }
    for &record in records {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::record::{
        self, DeclaredRole, FingerprintSet, PolicyReference, TransitionEvent,
    };
    use serde_json::json;

    /// Build an intrinsically validated synthetic record without opening any path.
    fn draft(policy: &str, version: &str) -> LifecycleRecord {
        let record: LifecycleRecord = serde_json::from_value(json!({
            "schema_version":"forge.policy-lifecycle/2",
            "policy":{"policy_key":policy,"version_key":version,"title":"PRIVATE title",
                "owner_keys":["owner"],"source":{"path":"source.bin","sha256":"a".repeat(64)},"generated_artifacts":[]},
            "parties":[{"key":"owner","roles":["owner","reviewer","approver"]}],
            "approval_policy":{"schema_version":"forge.approval-policy/1","required_roles":[{"role":"reviewer","count":1},{"role":"approver","count":1}],"separation":{}},
            "review":{"cadence_days":30,"next_review_date":"2026-10-12","due_soon_days":7,"timezone_policy":"date-only"},
            "state":"draft","history":[]
        })).expect("typed synthetic record");
        record::validate(&record).expect("intrinsic fixture");
        record
    }

    /// Append a real deterministic event and revalidate the synthetic declared history.
    fn transition(
        record: &mut LifecycleRecord,
        next: LifecycleState,
        role: DeclaredRole,
        time: &str,
        replacement: Option<PolicyReference>,
    ) {
        let mut event = TransitionEvent {
            sequence: u32::try_from(record.history.len() + 1).expect("small fixture"),
            event_id: String::new(),
            legacy_event_id: None,
            previous_state: record.state,
            next_state: next,
            actor_key: "owner".to_owned(),
            declared_role: role,
            timestamp: time.to_owned(),
            rationale: "PRIVATE rationale".to_owned(),
            fingerprints: FingerprintSet {
                source_sha256: record.policy.source.sha256.clone(),
                generated_artifacts: vec![],
            },
            assertions: vec![],
            impact_finding_ids: vec![],
            replacement: replacement.clone(),
        };
        event.event_id = record::event_id(record, &event).expect("real deterministic event ID");
        record.state = next;
        record.history.push(event);
        if replacement.is_some() {
            record.replaced_by = replacement;
        }
        record::validate(record).expect("intrinsic transitioned fixture");
    }

    /// Establish a locally declared review/approval history with no authenticated authority.
    fn approved(policy: &str, version: &str, approved_at: &str) -> LifecycleRecord {
        let mut record = draft(policy, version);
        transition(
            &mut record,
            LifecycleState::InReview,
            DeclaredRole::Reviewer,
            "2026-10-01T00:00:00Z",
            None,
        );
        transition(
            &mut record,
            LifecycleState::Approved,
            DeclaredRole::Approver,
            approved_at,
            None,
        );
        record
    }

    /// Preserve exact duplicate policy/version error text in the consumed pure seam.
    #[test]
    fn duplicate_policy_versions_preserve_cli_error() {
        let one = draft("policy", "v1");
        let two = one.clone();
        assert!(matches!(validate(&[&one,&two]),Err(ForgeError::Lifecycle(message))
            if message=="portfolio contains duplicate policy version 'policy:v1'"));
    }

    /// Require the supplied replacement rather than silently projecting a partial portfolio.
    #[test]
    fn missing_replacement_preserves_cli_error() {
        let mut one = approved("old", "v1", "2026-10-02T00:00:00Z");
        transition(
            &mut one,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "new".to_owned(), version_key: "v2".to_owned() }),
        );
        assert!(matches!(validate(&[&one]),Err(ForgeError::Lifecycle(message))
            if message=="supersession replacement 'new:v2' is not in the supplied portfolio"));
    }

    /// Retain approval-before-supersession chronology and accept the inclusive boundary.
    #[test]
    fn replacement_chronology_keeps_exact_error_and_equal_boundary() {
        let mut old = approved("old", "v1", "2026-10-02T00:00:00Z");
        transition(
            &mut old,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "new".to_owned(), version_key: "v2".to_owned() }),
        );
        let late = approved("new", "v2", "2026-10-04T00:00:00Z");
        assert!(matches!(validate(&[&old,&late]),Err(ForgeError::Lifecycle(message))
            if message=="replacement approval must not be later than supersession"));
        let equal = approved("new", "v2", "2026-10-03T00:00:00Z");
        assert!(validate(&[&old, &equal]).is_ok());
    }

    /// Reject a fully supplied cycle after intrinsic event IDs and chronology are valid.
    #[test]
    fn complete_replacement_cycle_preserves_cli_error() {
        let mut one = approved("one", "v1", "2026-10-02T00:00:00Z");
        let mut two = approved("two", "v1", "2026-10-02T00:00:00Z");
        transition(
            &mut one,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "two".to_owned(), version_key: "v1".to_owned() }),
        );
        transition(
            &mut two,
            LifecycleState::Superseded,
            DeclaredRole::Owner,
            "2026-10-03T00:00:00Z",
            Some(PolicyReference { policy_key: "one".to_owned(), version_key: "v1".to_owned() }),
        );
        assert!(matches!(validate(&[&one,&two]),Err(ForgeError::Lifecycle(message))
            if message=="supersession cycle includes 'one:v1'"));
    }
}
