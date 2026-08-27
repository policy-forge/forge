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
    /// Section title containing the requirement, if it belongs to a section.
    pub section_title: Option<String>,
    /// 1-based line number in the source file.
    pub line_number: usize,
}

/// A single mapping from a policy requirement to an OSCAL element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Borrowed trace links associated with one requirement.
///
/// The collection stores only indices in its forward index; this view resolves
/// those indices against the canonical insertion-order store without cloning
/// links or allocating an intermediate result.
#[derive(Debug, Clone, Copy)]
pub struct RequirementTraceLinks<'a> {
    links: &'a [TraceLink],
    indices: &'a [usize],
}

impl<'a> RequirementTraceLinks<'a> {
    /// Return the number of trace links in this requirement's view.
    #[must_use]
    pub fn len(self) -> usize {
        self.indices.len()
    }

    /// Return whether this requirement has no trace links.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.indices.is_empty()
    }

    /// Iterate over trace links in record order.
    pub fn iter(self) -> impl Iterator<Item = &'a TraceLink> {
        self.indices.iter().filter_map(|&index| self.links.get(index))
    }
}

impl std::ops::Index<usize> for RequirementTraceLinks<'_> {
    type Output = TraceLink;

    fn index(&self, index: usize) -> &Self::Output {
        let link_index = self.indices[index];
        &self.links[link_index]
    }
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
/// Stores trace links in a canonical insertion-order store plus forward and
/// reverse indexes. The forward index stores positions into `links`, never
/// duplicate `TraceLink` values.
#[derive(Debug, Default)]
pub struct TraceLinkCollection {
    links: Vec<TraceLink>,
    by_requirement: HashMap<String, Vec<usize>>,
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
    /// The canonical store is append-only, so every recorded index remains
    /// valid for the lifetime of this collection.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::DuplicateElement`] if `oscal_element_id` is already recorded.
    pub fn record(&mut self, link: TraceLink) -> Result<(), TraceError> {
        if self.by_oscal_element.contains_key(&link.oscal_element_id) {
            return Err(TraceError::DuplicateElement { element_id: link.oscal_element_id });
        }

        let index = self.links.len();
        self.by_requirement.entry(link.requirement_stable_id.clone()).or_default().push(index);
        self.by_oscal_element.insert(link.oscal_element_id.clone(), index);
        self.links.push(link);
        Ok(())
    }

    /// Look up all trace links for a given requirement stable ID.
    ///
    /// Returns an empty view if the requirement is not found.
    #[must_use]
    pub fn by_requirement(&self, stable_id: &str) -> RequirementTraceLinks<'_> {
        RequirementTraceLinks {
            links: &self.links,
            indices: self.by_requirement.get(stable_id).map_or(&[], Vec::as_slice),
        }
    }

    /// Look up the trace link for a given OSCAL element ID.
    ///
    /// Returns `None` if the element is not found.
    #[must_use]
    pub fn by_oscal_element(&self, element_id: &str) -> Option<&TraceLink> {
        self.by_oscal_element.get(element_id).and_then(|&index| self.links.get(index))
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

    fn sample_source_location() -> SourceLocation {
        SourceLocation {
            file_path: PathBuf::from("policies/access-control.md"),
            section_title: Some("Access Control".to_string()),
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

    #[test]
    fn source_location_construction_all_fields() {
        let location = sample_source_location();
        assert_eq!(location.file_path, PathBuf::from("policies/access-control.md"));
        assert_eq!(location.section_title.as_deref(), Some("Access Control"));
        assert_eq!(location.line_number, 42);
    }

    #[test]
    fn source_location_round_trips_without_section() {
        let location = SourceLocation {
            file_path: PathBuf::from("policy.md"),
            section_title: None,
            line_number: 1,
        };
        let json = serde_json::to_string(&location).unwrap();
        let deserialized: SourceLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(location, deserialized);
    }

    #[test]
    fn trace_link_construction_all_fields() {
        let link = sample_trace_link();
        assert_eq!(link.requirement_stable_id, "req-uuid-001");
        assert_eq!(link.oscal_json_path, "catalog.groups[0].controls[0]");
        assert_eq!(link.oscal_element_id, "ctrl-uuid-001");
        assert_eq!(link.source_location.section_title.as_deref(), Some("Access Control"));
    }

    #[test]
    fn collection_record_duplicate_element_id_returns_error() {
        let mut collection = TraceLinkCollection::new();
        collection.record(sample_trace_link()).unwrap();
        let duplicate = TraceLink {
            requirement_stable_id: "req-uuid-002".to_string(),
            oscal_json_path: "catalog.groups[0].controls[1]".to_string(),
            oscal_element_id: "ctrl-uuid-001".to_string(),
            source_location: sample_source_location(),
        };
        assert!(collection.record(duplicate).is_err());
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn requirement_lookup_resolves_noncontiguous_canonical_links() {
        let mut collection = TraceLinkCollection::new();
        for (requirement, element) in
            [("req-a", "elem-a1"), ("req-b", "elem-b"), ("req-a", "elem-a2")]
        {
            collection
                .record(TraceLink {
                    requirement_stable_id: requirement.to_string(),
                    oscal_json_path: format!("catalog.{element}"),
                    oscal_element_id: element.to_string(),
                    source_location: sample_source_location(),
                })
                .unwrap();
        }

        let links = collection.by_requirement("req-a");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].oscal_element_id, "elem-a1");
        assert_eq!(links[1].oscal_element_id, "elem-a2");
        assert_eq!(links.iter().count(), 2);
    }

    #[test]
    fn collection_by_requirement_unknown_id_returns_empty() {
        assert!(TraceLinkCollection::new().by_requirement("missing").is_empty());
    }
}
