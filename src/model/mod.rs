//! Domain model types for FORGE policy document processing.
//!
//! These types represent the core data structures used throughout the FORGE
//! pipeline, from ingestion through atomization to OSCAL export.

use serde::Serialize;

/// A single policy requirement extracted from a policy document.
///
/// After atomization, compound requirements (containing multiple obligations
/// joined by conjunctions) are replaced by multiple `PolicyRequirement`s,
/// each representing a single atomic obligation.
///
/// # Examples
///
/// ```
/// use forge::model::PolicyRequirement;
///
/// let req = PolicyRequirement {
///     stable_id: "a".repeat(64),
///     text: "All systems must enforce MFA".to_string(),
///     source_line: 42,
///     atom_index: 0,
///     parent_text: None,
/// };
///
/// assert_eq!(req.source_line, 42);
/// assert!(req.parent_text.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyRequirement {
    /// Preliminary stable ID (SHA-256 hex-encoded, 64 characters).
    /// Generated via `preliminary_id(text, source_line, atom_index)`.
    /// Will be replaced by UUID v5 in WI-7.
    pub stable_id: String,

    /// Atomic obligation text. For split requirements, this is the
    /// reconstructed clause with shared subject prepended. For atomic
    /// (non-split) requirements, this is the original text unchanged.
    pub text: String,

    /// 1-based line number from the original policy document.
    /// Preserved from the parent requirement when split.
    pub source_line: usize,

    /// 0-based position in the split. For non-split (atomic) requirements,
    /// this is 0. For split requirements, this is 0..N where N is the
    /// number of atomic parts produced.
    pub atom_index: usize,

    /// Original compound text if this requirement was produced by splitting.
    /// `None` if the requirement was already atomic (not split).
    pub parent_text: Option<String>,
}

/// A logical section of a policy document, grouping related requirements
/// under a heading.
///
/// # Examples
///
/// ```
/// use forge::model::{PolicySection, PolicyRequirement};
///
/// let section = PolicySection {
///     heading: "Access Control".to_string(),
///     requirements: vec![],
/// };
///
/// assert_eq!(section.heading, "Access Control");
/// assert!(section.requirements.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicySection {
    /// Section heading text (e.g., "Access Control").
    pub heading: String,

    /// Policy requirements within this section.
    pub requirements: Vec<PolicyRequirement>,
}

/// A complete policy document containing multiple sections with requirements.
///
/// # Examples
///
/// ```
/// use forge::model::PolicyDocument;
///
/// let doc = PolicyDocument {
///     title: "Security Policy".to_string(),
///     sections: vec![],
/// };
///
/// assert_eq!(doc.total_requirement_count(), 0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyDocument {
    /// Document title.
    pub title: String,

    /// Sections containing policy requirements.
    pub sections: Vec<PolicySection>,
}

impl PolicyDocument {
    /// Returns the total number of requirements across all sections.
    #[must_use]
    pub fn total_requirement_count(&self) -> usize {
        self.sections.iter().map(|s| s.requirements.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_document_has_zero_requirements() {
        let doc = PolicyDocument { title: String::new(), sections: vec![] };
        assert_eq!(doc.total_requirement_count(), 0);
    }

    #[test]
    fn document_counts_requirements_across_sections() {
        let doc = PolicyDocument {
            title: "Test".to_string(),
            sections: vec![
                PolicySection {
                    heading: "S1".to_string(),
                    requirements: vec![
                        PolicyRequirement {
                            stable_id: "id1".to_string(),
                            text: "req1".to_string(),
                            source_line: 1,
                            atom_index: 0,
                            parent_text: None,
                        },
                        PolicyRequirement {
                            stable_id: "id2".to_string(),
                            text: "req2".to_string(),
                            source_line: 2,
                            atom_index: 0,
                            parent_text: None,
                        },
                    ],
                },
                PolicySection {
                    heading: "S2".to_string(),
                    requirements: vec![PolicyRequirement {
                        stable_id: "id3".to_string(),
                        text: "req3".to_string(),
                        source_line: 3,
                        atom_index: 0,
                        parent_text: None,
                    }],
                },
            ],
        };
        assert_eq!(doc.total_requirement_count(), 3);
    }

    #[test]
    fn policy_requirement_serializes_to_json() {
        let req = PolicyRequirement {
            stable_id: "abc".to_string(),
            text: "test".to_string(),
            source_line: 1,
            atom_index: 0,
            parent_text: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("stable_id"));
        assert!(json.contains("text"));
    }

    #[test]
    fn policy_document_serializes_to_json() {
        let doc = PolicyDocument { title: "Test".to_string(), sections: vec![] };
        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("title"));
        assert!(json.contains("sections"));
    }
}
