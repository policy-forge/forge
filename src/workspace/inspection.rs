//! Bounded, captured-only projection primitives shared by lifecycle and impact reads.
//!
//! Runtime owns admission and the immutable read deadline. These helpers publish no effect,
//! operation progress, approval or file-read authority; encoded bounds are post-serialization
//! checks and do not establish total parser/allocation or syscall confinement.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::NaiveDate;
use serde_json::{Value, json};

use super::contract::{Error, Result};
use super::index::Role;
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::services::{Item, Snapshot};

/// Closed exact decoded query fields; duplicate keys cannot be silently overwritten.
pub(crate) struct Query<'a> {
    /// Borrowed exact fields sorted for deterministic cursor/version binding.
    values: BTreeMap<&'a str, &'a str>,
}

impl<'a> Query<'a> {
    /// Admit only this endpoint's keys, with bounded nonempty decoded values and no duplicates.
    pub(crate) fn new(raw: &'a [(String, String)], allowed: &[&str]) -> Result<Self> {
        if raw.len() > 16 {
            return Err(Error::invalid());
        }
        let mut values = BTreeMap::new();
        for (key, value) in raw {
            if !allowed.contains(&key.as_str())
                || key.len() > 50
                || value.is_empty()
                || value.len() > 65536
                || values.insert(key.as_str(), value.as_str()).is_some()
            {
                return Err(Error::invalid());
            }
        }
        for name in ["owner", "group", "policy_source"] {
            if values.get(name).is_some_and(|value| {
                value.len() > 4096 || value.trim() != *value || value.chars().any(char::is_control)
            }) {
                return Err(Error::invalid());
            }
        }
        if let Some(size) = values.get("page_size") {
            if !size.bytes().all(|byte| byte.is_ascii_digit())
                || !size.parse::<usize>().is_ok_and(|size| (1..=200).contains(&size))
            {
                return Err(Error::invalid());
            }
        }
        for (name, variants) in [
            ("state", &["draft", "in-review", "approved", "superseded", "retired"][..]),
            (
                "change_class",
                &["added", "removed", "content-changed", "identity-migrated", "unchanged"][..],
            ),
            ("decision_state", &["applicable", "not-applicable", "deferred", "under-review"][..]),
            ("priority", &["blocking", "review-required", "informational"][..]),
        ] {
            if values.get(name).is_some_and(|value| !variants.contains(value)) {
                return Err(Error::invalid());
            }
        }
        if let Some(cursor) = values.get("cursor") {
            let parts = cursor.split(':').collect::<Vec<_>>();
            if cursor.len() > 256
                || parts.len() != 3
                || [parts[0], parts[2]].iter().any(|part| {
                    part.len() != 64
                        || !part
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                })
                || parts[1].is_empty()
                || !parts[1].bytes().all(|byte| byte.is_ascii_digit())
                || parts[1].parse::<usize>().is_err()
            {
                return Err(Error::invalid());
            }
        }
        Ok(Self { values })
    }

    /// Return an exact admitted token; callers must not replace it with a display label.
    pub(crate) fn optional(&self, name: &str) -> Option<&'a str> {
        self.values.get(name).copied()
    }

    /// Parse the explicit date-only ISO spelling; no clock/default or timezone is sampled.
    pub(crate) fn date(&self, required: bool) -> Result<Option<NaiveDate>> {
        let Some(text) = self.optional("as_of") else {
            return if required { Err(Error::invalid()) } else { Ok(None) };
        };
        if text.len() != 10
            || text.as_bytes()[4] != b'-'
            || text.as_bytes()[7] != b'-'
            || text
                .bytes()
                .enumerate()
                .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
        {
            return Err(Error::invalid());
        }
        let date = NaiveDate::parse_from_str(text, "%Y-%m-%d").map_err(|_| Error::invalid())?;
        Ok(Some(date))
    }
}

/// Fixed safe input/domain failure; private parser paths, rationale and errors never become wire text.
pub(crate) fn domain_failure() -> Error {
    Error::new(
        "validation-failed",
        "The captured inputs cannot produce this complete inspection.",
        false,
    )
}

/// Require the explicit compatible index2 profile for a computed S-3 read.
pub(crate) fn require_index2(snapshot: &Snapshot) -> Result<()> {
    if !snapshot.index_present || snapshot.index.schema_version != "forge.workspace/2" {
        return Err(Error::new(
            "validation-failed",
            "Register or explicitly migrate an index version 2 before this inspection.",
            false,
        ));
    }
    Ok(())
}

/// Distinguish an absent index, retained legacy profile, explicit empty family and available inventory.
pub(crate) fn family_availability(snapshot: &Snapshot, total: usize) -> &'static str {
    if !snapshot.index_present {
        "absent-index"
    } else if snapshot.index.schema_version != "forge.workspace/2" {
        "index-upgrade-required"
    } else if total == 0 {
        "empty"
    } else {
        "available"
    }
}

/// Bind every registered byte/identity, endpoint, selection and exact date/filter tuple.
/// Page size is cursor-bound separately and cannot change the collection's semantic version.
pub(crate) fn version(
    snapshot: &Snapshot,
    domain: &str,
    selected: &str,
    query: &Query<'_>,
) -> String {
    let filters = query
        .values
        .iter()
        .filter(|(key, _)| !matches!(**key, "cursor" | "page_size"))
        .collect::<Vec<_>>();
    let seed = json!(["forge.workspace-s3-view/1", snapshot.version, domain, selected, filters]);
    crate::hashing::sha256_hex(
        &serde_json::to_vec(&seed).expect("bounded string tuple is serializable"),
    )
}

/// Project only the selected maximum200 rows after complete counting, with context-bound cursors.
/// Caller adds entity-specific full/matched counts, `snapshot_version` and availability fields.
pub(crate) fn page(
    total: usize,
    resource_version: &str,
    query: &Query<'_>,
    mut project: impl FnMut(usize) -> Result<Value>,
) -> Result<Value> {
    if total > 100_000 {
        return Err(Error::new(
            "payload-too-large",
            "The complete inspection exceeds its supported bound.",
            false,
        ));
    }
    let size = query.optional("page_size").map_or(Ok(50), |value| {
        if !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Error::invalid());
        }
        value.parse::<usize>().map_err(|_| Error::invalid())
    })?;
    if !(1..=200).contains(&size) {
        return Err(Error::invalid());
    }
    let bound_fields = query.values.iter().filter(|(key, _)| **key != "cursor").collect::<Vec<_>>();
    let filter_hash = crate::hashing::sha256_hex(
        &serde_json::to_vec(&bound_fields).map_err(|_| Error::invalid())?,
    );
    let offset = if let Some(cursor) = query.optional("cursor") {
        let parts = cursor.split(':').collect::<Vec<_>>();
        if parts.len() != 3 || parts[0] != resource_version || parts[2] != filter_hash {
            let mut error = Error::new(
                "version-conflict",
                "The collection changed. Restart from its first page.",
                true,
            );
            error.resource_version = Some(resource_version.to_owned());
            return Err(error);
        }
        if parts[1].is_empty() || !parts[1].bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(Error::invalid());
        }
        parts[1].parse::<usize>().map_err(|_| Error::invalid())?
    } else {
        0
    };
    if offset > total {
        return Err(Error::invalid());
    }
    let end = offset.saturating_add(size).min(total);
    let items = (offset..end).map(&mut project).collect::<Result<Vec<_>>>()?;
    let next_cursor = (end < total).then(|| format!("{resource_version}:{end}:{filter_hash}"));
    Ok(
        json!({"resource_version":resource_version,"page":{"items":items,"next_cursor":next_cursor,"total_matching":total}}),
    )
}

/// Resolve only a role-admitted captured registration; never open a supplied reference path.
/// Lifecycle callers enforce their stronger no-dot/no-parent grammar before this shared resolver.
pub(crate) fn reference<'a>(
    snapshot: &'a Snapshot,
    base: &str,
    relative: &Path,
    roles: &[Role],
) -> Result<&'a Item> {
    let path = super::domain::resolve_reference(base, relative)?;
    let mut matching = snapshot.items.iter().filter(|item| item.registration.path == path);
    let item = matching.next().ok_or_else(domain_failure)?;
    if matching.next().is_some() || !roles.contains(&item.registration.role) {
        return Err(domain_failure());
    }
    Ok(item)
}

/// Revalidate supplied captured facts before direct service projection, retaining typed stops.
/// This checks ordered registration identity, aliases and raw byte hashes; it does not rebuild
/// Snapshot.version or claim original raw-index provenance absent from this captured structure.
pub(crate) fn validate_snapshot(
    snapshot: &Snapshot,
    control: &mut dyn WorkControl,
) -> WorkResult<()> {
    if snapshot.items.len() > 1000 || snapshot.index.resources.len() > 1000 {
        return Err(Error::new(
            "payload-too-large",
            "The complete capture exceeds its supported bound.",
            false,
        )
        .into());
    }
    if snapshot.items.len() != snapshot.index.resources.len()
        || (!snapshot.index_present && !snapshot.items.is_empty())
        || !matches!(
            snapshot.index.schema_version.as_str(),
            "forge.workspace/1" | "forge.workspace/2"
        )
        || snapshot.version.len() != 64
        || !snapshot
            .version
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(domain_failure().into());
    }
    let mut total = 0usize;
    let mut paths = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut keys = BTreeSet::new();
    for (item, registered) in snapshot.items.iter().zip(&snapshot.index.resources) {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        if item.registration.key != registered.key
            || item.registration.path != registered.path
            || item.registration.role != registered.role
            || !registered.role.admitted_by(&snapshot.index.schema_version)
            || !matches!(
                item.metadata["validation_state"].as_str(),
                Some("valid" | "invalid" | "stale" | "not-validated")
            )
            || item.metadata["resource_id"].as_str()
                != Some(super::services::resource_id(registered).as_str())
        {
            return Err(domain_failure().into());
        }
        if super::index::validate_path(&registered.path).is_err()
            || !paths.insert(registered.path.to_ascii_lowercase())
            || !identities.insert(item.captured.identity)
            || !keys.insert(&registered.key)
        {
            return Err(Error::containment().into());
        }
        if item.captured.bytes.len() > 10 * 1024 * 1024 {
            return Err(Error::new(
                "payload-too-large",
                "The complete capture exceeds its supported bound.",
                false,
            )
            .into());
        }
        total = total.checked_add(item.captured.bytes.len()).ok_or_else(|| {
            Error::new(
                "payload-too-large",
                "The complete capture exceeds its supported bound.",
                false,
            )
        })?;
        if total > 50 * 1024 * 1024 {
            return Err(Error::new(
                "payload-too-large",
                "The complete capture exceeds its supported bound.",
                false,
            )
            .into());
        }
        let actual = crate::hashing::sha256_hex(&item.captured.bytes);
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        if actual != item.captured.sha256 {
            return Err(domain_failure().into());
        }
    }
    Ok(())
}

/// Emit unique captured metadata pins in authorial registration order, without paths or prose.
/// Validation state is the captured resource classification, separate from a selected domain's
/// exact closure/pair/freshness proof; lifecycle drift does not imply schema validity.
pub(crate) fn provenance(snapshot: &Snapshot, indices: &[usize]) -> Result<Vec<Value>> {
    let ordered = indices.iter().copied().collect::<BTreeSet<_>>();
    if ordered.len() > 1000 || ordered.iter().any(|index| *index >= snapshot.items.len()) {
        return Err(domain_failure());
    }
    Ok(ordered
        .into_iter()
        .map(|index| {
            let item = &snapshot.items[index];
            json!({"resource_id":item.metadata["resource_id"],"role":item.registration.role,
            "sha256":item.captured.sha256,"size_bytes":item.captured.bytes.len(),
            "validation_state":item.metadata["validation_state"]})
        })
        .collect())
}

/// Check the same runtime budget before/after complete bounded encoding and return no partial DTO.
/// A post-encoding cap does not imply allocation/CPU confinement; interruption remains typed.
pub(crate) fn finish(value: Value, control: &mut dyn WorkControl) -> WorkResult<Value> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let encoded = serde_json::to_vec(&value).map_err(|_| Error::invalid())?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if encoded.len() > 4 * 1024 * 1024 {
        return Err(Error::new(
            "payload-too-large",
            "The complete inspection response exceeds its supported bound.",
            false,
        )
        .into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::super::preparation::{Interruption, NoopControl, WorkError};
    use super::*;

    /// Convert exact test tokens to the same decoded query structure the HTTP reader supplies.
    fn fields(values: &[(&str, &str)]) -> Vec<(String, String)> {
        values.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect()
    }

    /// A deterministic sticky cancellation proves typed propagation, without production pause hooks.
    struct Stopped;

    impl WorkControl for Stopped {
        /// Reject every boundary with the exact internal stop rather than an ordinary Invalid error.
        fn checkpoint(&mut self, _: Stage, _: ProgressUpdate) -> WorkResult<()> {
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        }

        /// Retain the same first cancellation for final runtime handling.
        fn interruption(&self) -> Option<Interruption> {
            Some(Interruption::CancelRequested)
        }
    }

    /// Reject duplicated/unknown fields and native-invalid tokens before any captured domain work.
    #[test]
    fn query_admission_preserves_closed_exact_filter_contracts() {
        for pairs in [
            fields(&[("owner", "a"), ("owner", "b")]),
            fields(&[("other", "x")]),
            fields(&[("owner", " a")]),
            fields(&[("owner", "a\n")]),
            fields(&[("state", "ready")]),
            fields(&[("page_size", "0")]),
            fields(&[("page_size", "201")]),
            fields(&[("page_size", "+2")]),
        ] {
            assert_eq!(
                Query::new(&pairs, &["owner", "state", "page_size"]).err().unwrap().code,
                "invalid-request"
            );
        }
        let boundary = fields(&[("owner", &"x".repeat(4096)), ("page_size", "200")]);
        assert!(Query::new(&boundary, &["owner", "page_size"]).is_ok());
        let excessive = fields(&[("owner", &"x".repeat(4097))]);
        assert!(Query::new(&excessive, &["owner"]).is_err());
    }

    /// Exact explicit dates admit valid leap days and reject implicit/noncalendar/expanded queries.
    #[test]
    fn query_date_requires_actual_four_digit_calendar_spelling() {
        for text in ["2025-02-29", "2026-2-02", "+10000-01-01", "2026-10-02T00:00:00Z"] {
            let raw = fields(&[("as_of", text)]);
            assert!(Query::new(&raw, &["as_of"]).unwrap().date(true).is_err());
        }
        let raw = fields(&[("as_of", "2024-02-29")]);
        assert_eq!(
            Query::new(&raw, &["as_of"]).unwrap().date(true).unwrap().unwrap().to_string(),
            "2024-02-29"
        );
        assert!(Query::new(&[], &[]).unwrap().date(true).is_err());
        assert!(Query::new(&[], &[]).unwrap().date(false).unwrap().is_none());
    }

    /// A complete page cursor binds exact filters, page size and collection version with no truncation.
    #[test]
    fn page_cursor_cannot_cross_version_filter_or_page_size() {
        let raw = fields(&[("page_size", "2"), ("owner", "owner-a")]);
        let query = Query::new(&raw, &["page_size", "owner", "cursor"]).unwrap();
        let version = "a".repeat(64);
        let first = page(3, &version, &query, |index| Ok(json!(index))).unwrap();
        assert_eq!(first["page"]["items"], json!([0, 1]));
        assert_eq!(first["page"]["total_matching"], 3);
        let cursor = first["page"]["next_cursor"].as_str().unwrap();
        let raw = fields(&[("page_size", "2"), ("owner", "owner-a"), ("cursor", cursor)]);
        let next = Query::new(&raw, &["page_size", "owner", "cursor"]).unwrap();
        assert_eq!(
            page(3, &version, &next, |index| Ok(json!(index))).unwrap()["page"]["items"],
            json!([2])
        );
        for pairs in [
            fields(&[("page_size", "1"), ("owner", "owner-a"), ("cursor", cursor)]),
            fields(&[("page_size", "2"), ("owner", "owner-b"), ("cursor", cursor)]),
        ] {
            let error = page(
                3,
                &version,
                &Query::new(&pairs, &["page_size", "owner", "cursor"]).unwrap(),
                |index| Ok(json!(index)),
            )
            .unwrap_err();
            assert_eq!(error.code, "version-conflict");
            assert_eq!(error.resource_version, Some(version.clone()));
        }
        assert_eq!(
            page(3, &"b".repeat(64), &next, |index| Ok(json!(index))).unwrap_err().code,
            "version-conflict"
        );
    }

    /// Whole cardinality and encoded limits reject excess while allowing exact supported boundaries.
    #[test]
    fn whole_bounds_and_typed_stop_cannot_publish_partial_results() {
        let query = Query::new(&[], &[]).unwrap();
        assert_eq!(
            page(100_001, &"a".repeat(64), &query, |index| Ok(json!(index))).unwrap_err().code,
            "payload-too-large"
        );
        let complete = page(100_000, &"a".repeat(64), &query, |index| Ok(json!(index))).unwrap();
        assert_eq!(complete["page"]["items"].as_array().unwrap().len(), 50);
        assert_eq!(complete["page"]["total_matching"], 100_000);
        assert!(finish(json!("x".repeat(4 * 1024 * 1024 - 2)), &mut NoopControl).is_ok());
        assert!(matches!(finish(json!("x".repeat(4 * 1024 * 1024 - 1)), &mut NoopControl),
            Err(WorkError::Failed(error)) if error.code == "payload-too-large"));
        assert!(matches!(
            finish(json!({"would":"publish"}), &mut Stopped),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
    }
}
