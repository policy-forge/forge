//! Pure due/overdue review-alert projection over the existing schedule classification.
//!
//! Alerts borrow the already-derived `due-soon`/`overdue` blockers from a status report.
//! They read no paths, sample no clock, mutate no record, and imply no approval or
//! transition authority. Delivery channels remain out of scope (PRDs 057/065/068).

use chrono::NaiveDate;
use serde::Serialize;

use super::status::StatusReport;
use super::{ALERTS_SCHEMA_VERSION, TRUST_BOUNDARY};

/// One informational review alert for a policy whose review is due soon or overdue.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewAlert {
    /// Locally declared policy key from the validated record.
    pub(crate) policy_key: String,
    /// Locally declared policy version key.
    pub(crate) version_key: String,
    /// Validated recorded lifecycle state; alerts never replace it.
    pub(crate) state: super::record::LifecycleState,
    /// Existing schedule projection: `due-soon` or `overdue`.
    pub(crate) derived_status: String,
    /// Owner keys in the record order.
    pub(crate) owner_keys: Vec<String>,
    /// Date-only next review date supplied by the record.
    pub(crate) next_review_date: NaiveDate,
    /// Whole days from the explicit date to the review date; negative when overdue.
    pub(crate) days_until_review: i64,
}

/// Deterministic alert report for one explicit date.
#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AlertsReport {
    /// Existing alert wire version; no new public format beyond the lifecycle family.
    pub(crate) schema_version: &'static str,
    /// Explicit caller date; alerts are never computed from a wall clock.
    pub(crate) as_of: NaiveDate,
    /// Alerts ordered by review urgency, then owner-independent policy identity.
    pub(crate) alerts: Vec<ReviewAlert>,
    /// Declared-identity qualification retained from the status family.
    pub(crate) trust_boundary: &'static str,
}

/// Project informational alerts from already-derived status reports.
///
/// Only the existing `due-soon` and `overdue` schedule blockers produce alerts; drift and
/// identity blockers remain status evidence, not schedule alerts. Ordering is deterministic:
/// soonest review date first, then policy key, then version key. The input reports are
/// borrowed unchanged.
pub(crate) fn build_alerts(reports: &[StatusReport], as_of: NaiveDate) -> AlertsReport {
    let mut alerts: Vec<ReviewAlert> = reports
        .iter()
        .filter(|report| {
            report.blockers.iter().any(|blocker| matches!(blocker.as_str(), "due-soon" | "overdue"))
        })
        .map(|report| ReviewAlert {
            policy_key: report.policy_key.clone(),
            version_key: report.version_key.clone(),
            state: report.state,
            derived_status: report.derived_status.clone(),
            owner_keys: report.owner_keys.clone(),
            next_review_date: report.next_review_date,
            days_until_review: (report.next_review_date - as_of).num_days(),
        })
        .collect();
    alerts.sort_by(|left, right| {
        left.next_review_date
            .cmp(&right.next_review_date)
            .then_with(|| left.policy_key.cmp(&right.policy_key))
            .then_with(|| left.version_key.cmp(&right.version_key))
    });
    AlertsReport {
        schema_version: ALERTS_SCHEMA_VERSION,
        as_of,
        alerts,
        trust_boundary: TRUST_BOUNDARY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::record::{LifecycleRecord, LifecycleState};
    use crate::lifecycle::status::{CurrentArtifacts, status_from_captured};

    /// Reuse the intrinsic captured fixture from the status projector tests.
    fn report(next_review: &str, due_soon_days: u16, as_of: &str) -> StatusReport {
        let record: LifecycleRecord =
            serde_json::from_value(serde_json::json!({
                "schema_version": "forge.policy-lifecycle/2",
                "policy": {
                    "policy_key": "alert-policy", "version_key": "v1", "title": "Synthetic",
                    "owner_keys": ["owner"],
                    "source": {"path": "policy.md", "sha256": "a".repeat(64)},
                    "generated_artifacts": []
                },
                "parties": [{"key": "owner", "roles": ["owner", "reviewer", "approver"]}],
                "approval_policy": {
                    "schema_version": "forge.approval-policy/1",
                    "required_roles": [{"role": "reviewer", "count": 1}, {"role": "approver", "count": 1}],
                    "separation": {}
                },
                "review": {
                    "cadence_days": 365, "next_review_date": next_review,
                    "due_soon_days": due_soon_days, "timezone_policy": "date-only"
                },
                "state": "draft", "history": []
            }))
            .expect("synthetic record");
        let current = CurrentArtifacts {
            fingerprints: crate::lifecycle::record::FingerprintSet {
                source_sha256: record.policy.source.sha256.clone(),
                generated_artifacts: Vec::new(),
            },
            identity_changes: Vec::new(),
        };
        status_from_captured(
            &record,
            &current,
            Some(NaiveDate::parse_from_str(as_of, "%Y-%m-%d").expect("as-of date")),
        )
        .expect("captured status")
    }

    #[test]
    fn due_today_is_due_soon_alert_with_zero_days() {
        let alert = build_alerts(&[report("2026-09-24", 30, "2026-09-24")], date("2026-09-24"));
        assert_eq!(alert.alerts.len(), 1);
        assert_eq!(alert.alerts[0].derived_status, "due-soon");
        assert_eq!(alert.alerts[0].days_until_review, 0);
        assert_eq!(alert.alerts[0].state, LifecycleState::Draft);
    }

    #[test]
    fn overdue_alert_carries_negative_days() {
        let alert = build_alerts(&[report("2026-09-24", 30, "2026-09-25")], date("2026-09-25"));
        assert_eq!(alert.alerts.len(), 1);
        assert_eq!(alert.alerts[0].derived_status, "overdue");
        assert_eq!(alert.alerts[0].days_until_review, -1);
    }

    #[test]
    fn future_review_outside_window_produces_no_alert() {
        let alert = build_alerts(&[report("2026-12-25", 30, "2026-09-24")], date("2026-09-24"));
        assert!(alert.alerts.is_empty());
        assert_eq!(alert.as_of, date("2026-09-24"));
    }

    #[test]
    fn due_soon_boundary_is_inclusive_and_ordering_is_deterministic() {
        let boundary = report("2026-10-24", 30, "2026-09-24");
        let outside = report("2026-10-25", 30, "2026-09-24");
        let overdue = report("2026-09-01", 30, "2026-09-24");
        let alert = build_alerts(&[boundary, outside, overdue], date("2026-09-24"));
        assert_eq!(alert.alerts.len(), 2);
        assert_eq!(alert.alerts[0].derived_status, "overdue");
        assert_eq!(alert.alerts[1].derived_status, "due-soon");
        assert_eq!(alert.alerts[1].days_until_review, 30);
        let repeated = build_alerts(
            &[report("2026-10-24", 30, "2026-09-24"), report("2026-09-01", 30, "2026-09-24")],
            date("2026-09-24"),
        );
        assert_eq!(
            serde_json::to_vec(&alert).expect("alerts"),
            serde_json::to_vec(&repeated).expect("repeat alerts")
        );
    }

    #[test]
    fn drift_blockers_alone_do_not_create_schedule_alerts() {
        let mut drifted = report("2026-12-25", 30, "2026-09-24");
        drifted.blockers.push("approved-drifted".to_string());
        drifted.derived_status = "approved-drifted".to_string();
        let alert = build_alerts(&[drifted], date("2026-09-24"));
        assert!(alert.alerts.is_empty());
    }

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("date")
    }
}
