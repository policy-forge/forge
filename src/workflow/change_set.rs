//! Deterministic connector-neutral change-set schema (PRD 065 M-2/M-12).
//!
//! The same validated inputs always render the same bytes: items are sorted by
//! their stable FORGE ID, every identifier is derived from content, and no
//! wall-clock value is sampled. Default payloads carry IDs, titles, reason
//! codes, owners, dates and repository-relative links only; source excerpts
//! require an explicit opt-in and are never credential-shaped.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{error, read_bounded, reject_secret_text, render_json, repo_relative};
use crate::ForgeError;
use crate::hashing::sha256_hex;
use crate::json_strict::{self, Limits};
use crate::uuid::FORGE_NAMESPACE_UUID;

/// Versioned connector-neutral change-set format.
pub const CHANGE_SET_SCHEMA_VERSION: &str = "forge.integration-change-set/1";
/// Raw change-set byte ceiling, before JSON allocations.
pub const MAX_CHANGE_SET_BYTES: u64 = 10 * 1024 * 1024;
/// Maximum work items in one change set.
pub const MAX_ITEMS: usize = 10_000;
/// Maximum excerpt bytes retained per item when explicitly opted in.
pub const MAX_EXCERPT_BYTES: usize = 4096;

/// The FORGE work-item domains exported by Phase 1.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum WorkItemKind {
    /// Framework revision review finding (PRD 057).
    Finding,
    /// Applicability/mapping gap (PRD 056).
    Gap,
    /// Policy lifecycle review (PRD 058).
    LifecycleReview,
    /// Framework control change impact (PRD 057).
    FrameworkImpact,
    /// POA&M remediation milestone (PRD 064).
    PoamMilestone,
    /// Portable asserted review queue item (PRD 059).
    Review,
}

impl WorkItemKind {
    /// Exact wire label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finding => "finding",
            Self::Gap => "gap",
            Self::LifecycleReview => "lifecycle-review",
            Self::FrameworkImpact => "framework-impact",
            Self::PoamMilestone => "poam-milestone",
            Self::Review => "review",
        }
    }
}

/// Provenance binding one work item to its exact source artifact (M-2).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// Source artifact format identifier, e.g. `forge.framework-impact-report/1`.
    pub schema_version: String,
    /// Repository-relative source link (M-12); never an absolute path.
    pub source: String,
    /// Lowercase SHA-256 of the exact source artifact bytes.
    pub sha256: String,
}

/// One connector-neutral work item with a stable content-addressed FORGE ID.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkItem {
    /// Stable UUID v5 derived from kind, source identity and item key.
    pub id: String,
    /// Source domain of this work item.
    pub kind: WorkItemKind,
    /// Exact stable key inside the source artifact (finding ID, control ID,
    /// policy/version keys, or plan/item/milestone keys).
    pub key: String,
    /// Human-readable title.
    pub title: String,
    /// Machine-readable reason codes from the source report.
    pub reason_codes: Vec<String>,
    /// Declared owner keys; FORGE does not authenticate them.
    pub owners: Vec<String>,
    /// Explicit source-supplied due/revisit/target date, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    /// Exact source artifact provenance.
    pub provenance: Provenance,
    /// Optional source excerpt; present only under explicit opt-in (M-12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

/// Complete deterministic change set.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChangeSet {
    /// Exact [`CHANGE_SET_SCHEMA_VERSION`].
    pub schema_version: String,
    /// Sorted, duplicate-free work items.
    pub items: Vec<WorkItem>,
}

impl ChangeSet {
    /// Build a validated change set from exported items, sorting by stable ID.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] for duplicate IDs, over-limit item
    /// counts, invalid identifiers, or credential-shaped content.
    pub fn new(mut items: Vec<WorkItem>) -> Result<Self, ForgeError> {
        if items.len() > MAX_ITEMS {
            return Err(error(format!("change set exceeds the {MAX_ITEMS} item limit")));
        }
        items.sort_by(|left, right| left.id.cmp(&right.id));
        let mut ids = BTreeSet::new();
        for item in &items {
            validate_item(item)?;
            if !ids.insert(item.id.as_str()) {
                return Err(error(format!("duplicate change-set item id '{}'", item.id)));
            }
        }
        Ok(Self { schema_version: CHANGE_SET_SCHEMA_VERSION.to_string(), items })
    }

    /// Render deterministic JSON bytes with one trailing newline.
    ///
    /// # Errors
    ///
    /// Returns [`ForgeError::Integration`] when serialization fails.
    pub fn render(&self) -> Result<String, ForgeError> {
        render_json(self, "change set")
    }
}

/// Parse and validate a bounded change-set document.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] for oversized, malformed, or invalid documents.
pub fn parse(bytes: &[u8]) -> Result<ChangeSet, ForgeError> {
    if bytes.len() as u64 > MAX_CHANGE_SET_BYTES {
        return Err(error(format!("change set exceeds the {MAX_CHANGE_SET_BYTES} byte limit")));
    }
    let value = json_strict::parse_value(
        bytes,
        "integration change set",
        Limits { max_depth: 32, max_string_bytes: 64 * 1024 },
    )
    .map_err(|_| error("invalid bounded duplicate-free change-set JSON"))?;
    let set: ChangeSet = serde_json::from_value(value)
        .map_err(|_| error("change-set fields do not match the closed contract"))?;
    if set.schema_version != CHANGE_SET_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported schema_version '{}'; expected {CHANGE_SET_SCHEMA_VERSION}",
            set.schema_version
        )));
    }
    // Re-run the constructor invariants over the decoded items.
    ChangeSet::new(set.items)
}

fn validate_item(item: &WorkItem) -> Result<(), ForgeError> {
    Uuid::parse_str(&item.id).map_err(|_| error(format!("invalid item id '{}'", item.id)))?;
    if item.key.is_empty() || item.title.is_empty() {
        return Err(error("work items require a nonempty key and title"));
    }
    if item.provenance.source.is_empty()
        || item.provenance.source.starts_with('/')
        || item.provenance.source.contains("..")
    {
        return Err(error("provenance must be a repository-relative source link"));
    }
    json_strict::validate_lowercase_sha256("provenance hash", &item.provenance.sha256)
        .map_err(error)?;
    for text in std::iter::once(&item.title)
        .chain(item.reason_codes.iter())
        .chain(item.owners.iter())
        .chain(item.excerpt.iter())
    {
        reject_secret_text(text, "work item")?;
    }
    if let Some(excerpt) = &item.excerpt
        && excerpt.len() > MAX_EXCERPT_BYTES
    {
        return Err(error(format!("excerpt exceeds the {MAX_EXCERPT_BYTES} byte limit")));
    }
    Ok(())
}

/// Derive the stable FORGE work-item ID from its exact source identity.
#[must_use]
pub fn item_id(kind: WorkItemKind, provenance_sha256: &str, key: &str) -> String {
    let seed =
        format!("forge.integration-work-item/1\0{}\0{provenance_sha256}\0{key}", kind.as_str());
    Uuid::new_v5(&FORGE_NAMESPACE_UUID, seed.as_bytes()).to_string()
}

/// Options shared by every source exporter.
pub struct ExportOptions<'a> {
    /// Project root used to compute repository-relative links.
    pub root: &'a Path,
    /// Explicit opt-in for source excerpts (M-12); default payloads omit them.
    pub include_excerpts: bool,
}

/// Export framework revision findings and control-change impacts from a
/// `forge.framework-impact-report/1` JSON report using the framework loader.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the report cannot be read, is not a
/// complete framework impact report, or yields invalid work items.
pub fn export_framework_report(
    path: &Path,
    options: &ExportOptions<'_>,
) -> Result<Vec<WorkItem>, ForgeError> {
    let bytes = read_bounded(path, MAX_CHANGE_SET_BYTES)?;
    let report: crate::framework::model::ImpactReport = serde_json::from_slice(&bytes)
        .map_err(|cause| error(format!("invalid framework impact report: {cause}")))?;
    if report.schema_version != crate::framework::model::REPORT_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported framework report schema_version '{}'",
            report.schema_version
        )));
    }
    if report.status != crate::framework::model::ReportStatus::Complete {
        return Err(error("framework impact report is not complete"));
    }
    let link = repo_relative(options.root, path)?;
    let hash = sha256_hex(&bytes);
    let provenance =
        Provenance { schema_version: report.schema_version.clone(), source: link, sha256: hash };
    let mut items = Vec::new();
    for finding in &report.findings {
        let mut reason_codes = vec![finding.reason_code.as_str().to_string()];
        reason_codes.push(finding.required_action.as_str().to_string());
        items.push(WorkItem {
            id: item_id(WorkItemKind::Finding, &provenance.sha256, &finding.finding_id),
            kind: WorkItemKind::Finding,
            key: finding.finding_id.clone(),
            title: format!("{}: {}", finding.priority.as_str(), finding.subject_id),
            reason_codes,
            owners: finding.owner.clone().into_iter().collect(),
            due_date: None,
            provenance: provenance.clone(),
            excerpt: None,
        });
    }
    for change in &report.changes {
        if change.change_class == crate::framework::model::ChangeClass::Unchanged {
            continue;
        }
        items.push(WorkItem {
            id: item_id(WorkItemKind::FrameworkImpact, &provenance.sha256, &change.subject_id),
            kind: WorkItemKind::FrameworkImpact,
            key: change.subject_id.clone(),
            title: format!("{}: {}", change.change_class.as_str(), change.subject_id),
            reason_codes: vec![change.change_class.as_str().to_string()],
            owners: Vec::new(),
            due_date: None,
            provenance: provenance.clone(),
            excerpt: None,
        });
    }
    Ok(items)
}

/// Export review-queue gaps from a stored `forge.applicability-report/1` JSON
/// report using the applicability stored-report loader.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the report cannot be read or parsed.
pub fn export_applicability_report(
    path: &Path,
    options: &ExportOptions<'_>,
) -> Result<Vec<WorkItem>, ForgeError> {
    let bytes = read_bounded(path, MAX_CHANGE_SET_BYTES)?;
    let report = crate::applicability::parse_stored_report(&bytes)?;
    let link = repo_relative(options.root, path)?;
    let provenance = Provenance {
        schema_version: report.schema_version.to_string(),
        source: link,
        sha256: sha256_hex(&bytes),
    };
    let mut items = Vec::new();
    for row in &report.review_queue {
        let key = format!("{}:{}", row.control_id, row.reason_code.as_str());
        items.push(WorkItem {
            id: item_id(WorkItemKind::Gap, &provenance.sha256, &key),
            kind: WorkItemKind::Gap,
            key,
            title: format!("gap: {}", row.control_id),
            reason_codes: vec![row.reason_code.as_str().to_string()],
            owners: row.owner.clone().into_iter().collect(),
            due_date: row.revisit_date.clone(),
            provenance: provenance.clone(),
            excerpt: None,
        });
    }
    Ok(items)
}

/// Export lifecycle review work items from validated `forge.policy-lifecycle`
/// records using the lifecycle record parser.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when a record cannot be read or parsed.
pub fn export_lifecycle_records(
    paths: &[std::path::PathBuf],
    options: &ExportOptions<'_>,
) -> Result<Vec<WorkItem>, ForgeError> {
    let mut items = Vec::new();
    for path in paths {
        let bytes = read_bounded(path, crate::lifecycle::record::MAX_RECORD_BYTES)?;
        let record = crate::lifecycle::record::parse(&bytes)?;
        let provenance = Provenance {
            schema_version: record.schema_version.clone(),
            source: repo_relative(options.root, path)?,
            sha256: sha256_hex(&bytes),
        };
        let key = format!("{}/{}", record.policy.policy_key, record.policy.version_key);
        items.push(WorkItem {
            id: item_id(WorkItemKind::LifecycleReview, &provenance.sha256, &key),
            kind: WorkItemKind::LifecycleReview,
            key,
            title: record.policy.title.clone(),
            reason_codes: vec![record.state.as_str().to_string()],
            owners: record.policy.owner_keys.clone(),
            due_date: Some(record.review.next_review_date.to_string()),
            provenance,
            excerpt: None,
        });
    }
    Ok(items)
}

/// Export portable review work items from a captured `forge.review-queue/1`
/// document using the maintained review decoder. Queue items are asserted
/// declarations; exporting them grants no native currentness or approval.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the queue cannot be read or decoded.
pub fn export_review_queue(
    path: &Path,
    options: &ExportOptions<'_>,
) -> Result<Vec<WorkItem>, ForgeError> {
    let bytes = read_bounded(path, 10 * 1024 * 1024)?;
    let mut ledger = crate::review::decode::ContractLedger::default();
    let mut control = crate::workspace::preparation::NoopControl;
    let decoded = crate::review::decode::decode_queue(&bytes, &mut ledger, &mut control)
        .map_err(|cause| error(format!("invalid review queue: {cause}")))?;
    let queue = decoded.document();
    let provenance = Provenance {
        schema_version: queue.schema_version.clone(),
        source: repo_relative(options.root, path)?,
        sha256: sha256_hex(&bytes),
    };
    let mut items = Vec::new();
    for item in &queue.items {
        items.push(WorkItem {
            id: item_id(WorkItemKind::Review, &provenance.sha256, &item.item_id),
            kind: WorkItemKind::Review,
            key: item.item_id.clone(),
            title: format!("review: {}", item.subject_id),
            reason_codes: item.context.reason_codes.clone(),
            owners: item.assignments.iter().map(|pair| pair.reviewer_key.clone()).collect(),
            due_date: item.due_at.clone(),
            provenance: provenance.clone(),
            excerpt: None,
        });
    }
    Ok(items)
}

/// Export POA&M milestones from a `forge.poam/1` workflow manifest using the
/// workflow declaration parser. Source bytes are not re-read here; the manifest
/// parser validates declarations and the recorded source identities travel as
/// provenance keys.
///
/// # Errors
///
/// Returns [`ForgeError::Integration`] when the manifest cannot be read or parsed.
pub fn export_poam_milestones(
    path: &Path,
    options: &ExportOptions<'_>,
) -> Result<Vec<WorkItem>, ForgeError> {
    let bytes = read_bounded(path, crate::poam::manifest::MAX_MANIFEST_BYTES)?;
    let manifest = crate::poam::workflow::parse(&bytes)?;
    let provenance = Provenance {
        schema_version: manifest.schema_version.clone(),
        source: repo_relative(options.root, path)?,
        sha256: sha256_hex(&bytes),
    };
    let mut items = Vec::new();
    for item in &manifest.items {
        for milestone in &item.milestones {
            let key = format!("{}/{}/{}", manifest.document.key, item.key, milestone.key);
            items.push(WorkItem {
                id: item_id(WorkItemKind::PoamMilestone, &provenance.sha256, &key),
                kind: WorkItemKind::PoamMilestone,
                key,
                title: milestone.outcome.clone(),
                reason_codes: vec![milestone.state.as_str().to_string()],
                owners: milestone.owners.iter().map(|owner| owner.party_key.clone()).collect(),
                due_date: Some(milestone.target_date.clone()),
                provenance: provenance.clone(),
                excerpt: excerpt(options.include_excerpts, &item.description)?,
            });
        }
    }
    Ok(items)
}

/// Retain a bounded excerpt only under explicit opt-in; refuse secrets.
fn excerpt(include: bool, text: &str) -> Result<Option<String>, ForgeError> {
    if !include {
        return Ok(None);
    }
    // `String::truncate` panics inside a multi-byte character, so back up to a boundary.
    let mut end = text.len().min(MAX_EXCERPT_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let bounded = &text[..end];
    reject_secret_text(bounded, "excerpt")?;
    Ok(Some(bounded.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(key: &str) -> WorkItem {
        WorkItem {
            id: item_id(WorkItemKind::Gap, &"a".repeat(64), key),
            kind: WorkItemKind::Gap,
            key: key.to_string(),
            title: format!("gap {key}"),
            reason_codes: vec!["no-reviewed-mapping".to_string()],
            owners: vec!["owner".to_string()],
            due_date: Some("2026-12-01".to_string()),
            provenance: Provenance {
                schema_version: "forge.applicability-report/1".to_string(),
                source: "reports/gaps.json".to_string(),
                sha256: "a".repeat(64),
            },
            excerpt: None,
        }
    }

    #[test]
    fn ids_are_stable_and_content_addressed() {
        let one = item_id(WorkItemKind::Gap, &"a".repeat(64), "ac-1");
        assert_eq!(one, item_id(WorkItemKind::Gap, &"a".repeat(64), "ac-1"));
        assert_ne!(one, item_id(WorkItemKind::Gap, &"b".repeat(64), "ac-1"));
        assert_ne!(one, item_id(WorkItemKind::Finding, &"a".repeat(64), "ac-1"));
        Uuid::parse_str(&one).unwrap();
    }

    #[test]
    fn render_is_deterministic_and_sorted() {
        let set = ChangeSet::new(vec![item("z"), item("a")]).unwrap();
        let bytes = set.render().unwrap();
        assert_eq!(bytes, set.render().unwrap());
        assert_eq!(bytes, ChangeSet::new(vec![item("a"), item("z")]).unwrap().render().unwrap());
        assert!(set.items[0].id < set.items[1].id);
        let round = parse(bytes.as_bytes()).unwrap();
        assert_eq!(round, set);
    }

    #[test]
    fn duplicate_ids_are_refused() {
        assert!(ChangeSet::new(vec![item("a"), item("a")]).is_err());
    }

    #[test]
    fn unknown_fields_and_versions_are_refused() {
        let set = ChangeSet::new(vec![item("a")]).unwrap();
        let mut value = serde_json::to_value(&set).unwrap();
        value["extra"] = serde_json::json!(1);
        assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
        value = serde_json::to_value(&set).unwrap();
        value["schema_version"] = serde_json::json!("forge.integration-change-set/2");
        assert!(parse(&serde_json::to_vec(&value).unwrap()).is_err());
    }

    #[test]
    fn excerpt_truncates_on_a_char_boundary() {
        // An em dash (3 bytes) straddles the byte limit; truncation must not split it.
        let text = format!("{}\u{2014}tail", "a".repeat(MAX_EXCERPT_BYTES - 1));
        let kept = excerpt(true, &text).unwrap().unwrap();
        assert_eq!(kept, "a".repeat(MAX_EXCERPT_BYTES - 1));
        // Plain ASCII still keeps exactly the limit, and opting out keeps nothing.
        let ascii = excerpt(true, &"b".repeat(MAX_EXCERPT_BYTES + 10)).unwrap().unwrap();
        assert_eq!(ascii.len(), MAX_EXCERPT_BYTES);
        assert_eq!(excerpt(false, &text).unwrap(), None);
    }

    #[test]
    fn secrets_and_absolute_links_are_refused() {
        let mut secret = item("a");
        secret.title = "token ghp_0123456789abcdef".to_string();
        assert!(ChangeSet::new(vec![secret]).is_err());
        let mut absolute = item("b");
        absolute.provenance.source = "/etc/passwd".to_string();
        assert!(ChangeSet::new(vec![absolute]).is_err());
    }
}
