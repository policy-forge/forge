//! Traceability model: bidirectional mapping between policy requirements
//! and generated OSCAL elements.
//!
//! Provides [`SourceLocation`], [`TraceLink`], [`TraceLinkCollection`], and
//! [`TraceError`] for recording and querying the provenance of every OSCAL
//! element back to its source policy requirement.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Source location of a policy requirement in the original document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Path to the source policy file.
    pub file_path: PathBuf,
    /// Section title containing the requirement. Empty string if no parent section (EC-4).
    pub section_title: String,
    /// 1-based line number in the source file.
    pub line_number: usize,
}

/// A single mapping from a policy requirement to an OSCAL element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLink {
    /// Stable UUID of the source requirement.
    pub requirement_stable_id: String,
    /// Dot-notation logical path in the OSCAL output (e.g., `catalog.groups[0].controls[2]`).
    pub oscal_json_path: String,
    /// Unique OSCAL element identifier (one-to-one reverse constraint).
    pub oscal_element_id: String,
    /// Where the requirement lives in the source document.
    pub source_location: SourceLocation,
}

/// Error types for traceability operations.
#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    /// Attempted to record a duplicate OSCAL element ID.
    #[error("Duplicate OSCAL element ID: {element_id} already recorded")]
    DuplicateElement {
        /// The duplicate element ID.
        element_id: String,
    },
}

/// Append-only collection with dual-store bidirectional lookup.
///
/// Stores trace links in two places:
/// - `links`: insertion-order canonical store (for `iter`, `len`, `is_empty`, `by_oscal_element`)
/// - `by_requirement`: grouped forward index (for `by_requirement` returning `&[TraceLink]`)
/// - `by_oscal_element`: reverse index mapping element ID to position in `links`
#[derive(Debug, Default)]
pub struct TraceLinkCollection {
    /// Canonical store preserving insertion order.
    links: Vec<TraceLink>,
    /// Forward index: `requirement_stable_id` -> grouped Vec of cloned `TraceLink`s.
    by_requirement: HashMap<String, Vec<TraceLink>>,
    /// Reverse index: `oscal_element_id` -> index into `links`.
    by_oscal_element: HashMap<String, usize>,
}

impl TraceLinkCollection {
    /// Create an empty collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a trace link, updating both indexes.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::DuplicateElement`] if `oscal_element_id` is already recorded.
    pub fn record(&mut self, link: TraceLink) -> Result<(), TraceError> {
        if self.by_oscal_element.contains_key(&link.oscal_element_id) {
            return Err(TraceError::DuplicateElement { element_id: link.oscal_element_id });
        }

        // Clone into forward index grouped by requirement
        self.by_requirement
            .entry(link.requirement_stable_id.clone())
            .or_default()
            .push(link.clone());

        // Append original to canonical store and record position in reverse index
        let index = self.links.len();
        self.by_oscal_element.insert(link.oscal_element_id.clone(), index);
        self.links.push(link);

        Ok(())
    }

    /// Look up all trace links for a given requirement stable ID.
    ///
    /// Returns an empty slice if the requirement is not found.
    #[must_use]
    pub fn by_requirement(&self, stable_id: &str) -> &[TraceLink] {
        self.by_requirement.get(stable_id).map_or(&[], Vec::as_slice)
    }

    /// Look up the trace link for a given OSCAL element ID.
    ///
    /// Returns `None` if the element is not found.
    #[must_use]
    pub fn by_oscal_element(&self, element_id: &str) -> Option<&TraceLink> {
        self.by_oscal_element.get(element_id).map(|&idx| &self.links[idx])
    }

    /// Iterate over all trace links in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &TraceLink> {
        self.links.iter()
    }

    /// Return the number of recorded trace links.
    #[must_use]
    pub fn len(&self) -> usize {
        self.links.len()
    }

    /// Return `true` if no trace links have been recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.links.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────

    fn sample_source_location() -> SourceLocation {
        SourceLocation {
            file_path: PathBuf::from("policies/access-control.md"),
            section_title: "Access Control".to_string(),
            line_number: 42,
        }
    }

    fn sample_trace_link() -> TraceLink {
        TraceLink {
            requirement_stable_id: "req-uuid-001".to_string(),
            oscal_json_path: "catalog.groups[0].controls[0]".to_string(),
            oscal_element_id: "ctrl-uuid-001".to_string(),
            source_location: sample_source_location(),
        }
    }

    // ── T002: SourceLocation tests ──────────────────────────────────

    #[test]
    fn source_location_construction_all_fields() {
        let loc = sample_source_location();
        assert_eq!(loc.file_path, PathBuf::from("policies/access-control.md"));
        assert_eq!(loc.section_title, "Access Control");
        assert_eq!(loc.line_number, 42);
    }

    #[test]
    fn source_location_debug_clone_partial_eq() {
        let loc = sample_source_location();

        // Debug
        let debug = format!("{loc:?}");
        assert!(debug.contains("SourceLocation"));

        // Clone + PartialEq + Eq
        let cloned = loc.clone();
        assert_eq!(loc, cloned);
    }

    #[test]
    fn source_location_serialize_deserialize_round_trip() {
        let loc = sample_source_location();
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: SourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(loc, deserialized);
    }

    #[test]
    fn source_location_empty_section_title_ec4() {
        let loc = SourceLocation {
            file_path: PathBuf::from("policy.md"),
            section_title: String::new(),
            line_number: 1,
        };
        assert_eq!(loc.section_title, "");
        // Round-trip with empty section_title
        let json = serde_json::to_string(&loc).unwrap();
        let deserialized: SourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(loc, deserialized);
    }

    // ── T003: TraceLink tests ───────────────────────────────────────

    #[test]
    fn trace_link_construction_all_fields() {
        let link = sample_trace_link();
        assert_eq!(link.requirement_stable_id, "req-uuid-001");
        assert_eq!(link.oscal_json_path, "catalog.groups[0].controls[0]");
        assert_eq!(link.oscal_element_id, "ctrl-uuid-001");
        assert_eq!(link.source_location.file_path, PathBuf::from("policies/access-control.md"));
        assert_eq!(link.source_location.section_title, "Access Control");
        assert_eq!(link.source_location.line_number, 42);
    }

    #[test]
    fn trace_link_debug_clone() {
        let link = sample_trace_link();

        let debug = format!("{link:?}");
        assert!(debug.contains("TraceLink"));

        let cloned = link.clone();
        assert_eq!(cloned.requirement_stable_id, link.requirement_stable_id);
        assert_eq!(cloned.oscal_element_id, link.oscal_element_id);
    }

    #[test]
    fn trace_link_serialize_deserialize_round_trip() {
        let link = sample_trace_link();
        let json = serde_json::to_string(&link).unwrap();
        let deserialized: TraceLink = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.requirement_stable_id, link.requirement_stable_id);
        assert_eq!(deserialized.oscal_element_id, link.oscal_element_id);
        assert_eq!(deserialized.oscal_json_path, link.oscal_json_path);
        assert_eq!(deserialized.source_location, link.source_location);
    }

    // ── T004: TraceError tests ──────────────────────────────────────

    #[test]
    fn trace_error_duplicate_element_display() {
        let err = TraceError::DuplicateElement { element_id: "elem-123".to_string() };
        let msg = err.to_string();
        assert_eq!(msg, "Duplicate OSCAL element ID: elem-123 already recorded");
    }

    #[test]
    fn trace_error_debug() {
        let err = TraceError::DuplicateElement { element_id: "elem-123".to_string() };
        let debug = format!("{err:?}");
        assert!(debug.contains("DuplicateElement"));
    }

    // ── T008: TraceLinkCollection::new / len / is_empty ─────────────

    #[test]
    fn collection_new_is_empty() {
        let coll = TraceLinkCollection::new();
        assert!(coll.is_empty());
        assert_eq!(coll.len(), 0);
    }

    // ── T009: record() happy path and duplicate detection ───────────

    #[test]
    fn collection_record_happy_path() {
        let mut coll = TraceLinkCollection::new();
        let link = sample_trace_link();
        let result = coll.record(link);
        assert!(result.is_ok());
        assert_eq!(coll.len(), 1);
        assert!(!coll.is_empty());
    }

    #[test]
    fn collection_record_duplicate_element_id_returns_error() {
        let mut coll = TraceLinkCollection::new();
        let link1 = sample_trace_link();
        coll.record(link1).unwrap();

        // Same oscal_element_id → duplicate error
        let link2 = TraceLink {
            requirement_stable_id: "req-uuid-002".to_string(),
            oscal_json_path: "catalog.groups[0].controls[1]".to_string(),
            oscal_element_id: "ctrl-uuid-001".to_string(), // duplicate!
            source_location: sample_source_location(),
        };
        let result = coll.record(link2);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("ctrl-uuid-001"));
        // Collection still has only 1 link
        assert_eq!(coll.len(), 1);
    }

    // ── T010: by_requirement() ──────────────────────────────────────

    #[test]
    fn collection_by_requirement_known_id() {
        let mut coll = TraceLinkCollection::new();
        coll.record(sample_trace_link()).unwrap();

        let links = coll.by_requirement("req-uuid-001");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].oscal_element_id, "ctrl-uuid-001");
    }

    #[test]
    fn collection_by_requirement_unknown_id_returns_empty() {
        let coll = TraceLinkCollection::new();
        let links = coll.by_requirement("nonexistent");
        assert!(links.is_empty());
    }

    #[test]
    fn collection_by_requirement_multiple_links() {
        let mut coll = TraceLinkCollection::new();
        // Same requirement maps to two OSCAL elements
        coll.record(TraceLink {
            requirement_stable_id: "req-001".to_string(),
            oscal_json_path: "catalog.groups[0].controls[0]".to_string(),
            oscal_element_id: "elem-a".to_string(),
            source_location: sample_source_location(),
        })
        .unwrap();
        coll.record(TraceLink {
            requirement_stable_id: "req-001".to_string(),
            oscal_json_path: "catalog.groups[0].controls[1]".to_string(),
            oscal_element_id: "elem-b".to_string(),
            source_location: sample_source_location(),
        })
        .unwrap();

        let links = coll.by_requirement("req-001");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].oscal_element_id, "elem-a");
        assert_eq!(links[1].oscal_element_id, "elem-b");
    }

    // ── T011: by_oscal_element() ────────────────────────────────────

    #[test]
    fn collection_by_oscal_element_known_id() {
        let mut coll = TraceLinkCollection::new();
        coll.record(sample_trace_link()).unwrap();

        let link = coll.by_oscal_element("ctrl-uuid-001");
        assert!(link.is_some());
        assert_eq!(link.unwrap().requirement_stable_id, "req-uuid-001");
    }

    #[test]
    fn collection_by_oscal_element_unknown_id_returns_none() {
        let coll = TraceLinkCollection::new();
        assert!(coll.by_oscal_element("nonexistent").is_none());
    }

    // ── T012: iter() ────────────────────────────────────────────────

    #[test]
    fn collection_iter_insertion_order() {
        let mut coll = TraceLinkCollection::new();
        for i in 0..3 {
            coll.record(TraceLink {
                requirement_stable_id: format!("req-{i}"),
                oscal_json_path: format!("catalog.groups[0].controls[{i}]"),
                oscal_element_id: format!("elem-{i}"),
                source_location: sample_source_location(),
            })
            .unwrap();
        }

        let ids: Vec<&str> = coll.iter().map(|l| l.oscal_element_id.as_str()).collect();
        assert_eq!(ids, vec!["elem-0", "elem-1", "elem-2"]);
    }

    #[test]
    fn collection_iter_empty() {
        let coll = TraceLinkCollection::new();
        assert_eq!(coll.iter().count(), 0);
    }
}
