//! Internal domain model for parsed policy documents.
//!
//! Provides the canonical representation consumed by all downstream pipeline
//! stages (atomization, UUID generation, OSCAL generation). The domain model
//! is format-agnostic: no Markdown-specific fields and no OSCAL-specific fields.

pub mod assemble;
mod frontmatter;
pub mod trace;

use std::path::PathBuf;

use serde::Serialize;

/// Classification of a policy requirement's obligation strength.
///
/// Derived from heuristic verb matching against RFC 2119 keywords in
/// `PolicyRequirement.text`. Applied by the WI-33 modality detection
/// enrichment pass (`annotate_modalities`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    /// Mandatory obligation: triggered by "must", "shall", "will", "required".
    /// Also the default when no modality verb is detected.
    Normative,
    /// Recommendation or optional practice: triggered by "should", "may",
    /// "recommended", "optional".
    Advisory,
}

pub use assemble::assemble_document;

/// Top-level domain model for a parsed policy document.
///
/// This is the canonical internal representation consumed by all
/// downstream pipeline stages (atomization, UUID generation, OSCAL generation).
///
/// # Lifecycle
/// - Created by `assemble_document` function (WI-5)
/// - Enriched by WI-6 (atomization)
/// - Enriched by WI-7 (UUID generation populates `stable_id` fields)
/// - Enriched by WI-8 (citation extraction)
/// - Consumed by WI-9+ (OSCAL generation)
///
/// # Ownership
/// Passed through pipeline using functional transformation: each WI takes
/// ownership and returns an enriched instance.
#[derive(Debug, Clone, Serialize)]
pub struct PolicyDocument {
    /// Document identifier (derived from filename or frontmatter).
    pub id: String,

    /// Document-level metadata.
    pub metadata: DocumentMetadata,

    /// Top-level sections (may contain nested children).
    pub sections: Vec<PolicySection>,
}

/// Metadata about the source policy document.
///
/// Extracted from YAML frontmatter when available, with fallback to
/// heading-based extraction and sensible defaults.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DocumentMetadata {
    /// Document title.
    /// - Frontmatter `title` field, OR
    /// - First H1 heading text, OR
    /// - Filename (without extension)
    pub title: String,

    /// Document version (semantic versioning format preferred).
    /// - Frontmatter `version` field, OR
    /// - Default "0.0.0"
    pub version: String,

    /// Document author (from frontmatter only).
    pub author: Option<String>,

    /// Document date (ISO 8601 format preferred; from frontmatter only).
    pub date: Option<String>,

    /// Path to the source file.
    pub source_path: PathBuf,

    /// SHA-256 hash of source content (from `IngestedDocument`).
    /// `None` if not computed by ingestion layer.
    pub content_hash: Option<String>,
}

/// A hierarchical section within a policy document, mapped from a Markdown heading.
///
/// Sections form a tree structure with parent-child relationships.
#[derive(Debug, Clone, Serialize)]
pub struct PolicySection {
    /// Section title (heading text).
    pub title: String,

    /// Heading level: 1 for H1, 2 for H2, ..., 6 for H6.
    pub heading_level: u8,

    /// Source line number in the original document (1-based).
    pub source_line: usize,

    /// Text content between this heading and first child heading or next sibling.
    /// `None` if section has no body text.
    pub body_text: Option<String>,

    /// Child sections (deeper heading levels).
    pub children: Vec<PolicySection>,

    /// Policy requirements extracted from list items within this section.
    pub requirements: Vec<PolicyRequirement>,
}

/// An individual policy requirement extracted from a list item or clause pattern.
///
/// Requirements represent the atomic units of policy that will map to OSCAL controls.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyRequirement {
    /// Stable UUID for this requirement.
    /// - `None` until populated by WI-7 (UUID generation)
    /// - `Some(uuid)` after WI-7
    pub stable_id: Option<String>,

    /// Full text of the requirement (pre-atomization; may be compound statement).
    pub text: String,

    /// Source line number in the original document (1-based).
    pub source_line: usize,

    /// Nesting depth from extraction (0 = top-level list item, 1 = nested once, etc.).
    pub nesting_depth: u8,

    /// 0-based position in the split. For non-split (atomic) requirements,
    /// this is 0. For split requirements, this is 0..N where N is the
    /// number of atomic parts produced. Added by WI-6 (atomization).
    pub atom_index: usize,

    /// Original compound text if this requirement was produced by splitting.
    /// `None` if the requirement was already atomic (not split). Added by WI-6 (atomization).
    pub parent_text: Option<String>,

    /// Citations extracted from this requirement's text by WI-8.
    /// Empty until citation extraction runs.
    pub citations: Vec<Citation>,

    /// Modality classification for this requirement.
    ///
    /// - `None` until `annotate_modalities` runs (pre-WI-33)
    /// - `Some(Modality::Normative)` for normative requirements (or default)
    /// - `Some(Modality::Advisory)` for advisory requirements
    ///
    /// After `annotate_modalities` completes, all requirements in the
    /// document have `Some(modality)`.
    pub modality: Option<Modality>,

    /// Parameters extracted from this requirement's text by WI-34.
    /// Empty until parameter extraction runs.
    pub parameters: Vec<PolicyParameter>,
}

/// A parameterizable value extracted from a policy requirement (WI-34).
///
/// Represents a configurable criterion (time window, threshold, frequency,
/// or quantity) extracted from requirement prose. Each parameter is linked
/// to its source `PolicyRequirement` via `requirement_id`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyParameter {
    /// Deterministic identifier: `"{requirement_id}_prm_{position}"`.
    pub id: String,
    /// `stable_id` of the `PolicyRequirement` this parameter was extracted from.
    pub requirement_id: String,
    /// Human-readable label describing this parameter (e.g., `"within 30 days"`).
    pub label: String,
    /// The extracted parameter value as a string (e.g., `"30 days"`, `"annually"`).
    pub value: String,
    /// The semantic category of this parameter.
    pub parameter_type: ParameterType,
    /// Value domain constraint inferred from qualifier words.
    pub constraint: Option<ParameterConstraint>,
}

/// The semantic category of a policy parameter (WI-34).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ParameterType {
    /// Duration parameters: "within N days", "after N weeks", "every N months".
    TimeWindow,
    /// Numeric boundary parameters: "at least N", "minimum N", "no more than N".
    Threshold,
    /// Recurrence parameters: "annually", "quarterly", "at least monthly".
    Frequency,
    /// Count parameters: "no fewer than 3 factors", "at least 2 generations".
    Quantity,
}

/// Value domain constraint on a `PolicyParameter` (WI-34).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParameterConstraint {
    /// The direction of the bound.
    pub constraint_type: ConstraintType,
    /// The bound value as a string (same string as `PolicyParameter.value`).
    pub value: String,
}

/// The direction of a value domain bound (WI-34).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ConstraintType {
    /// Value must be at least this amount (triggered by "at least", "within", "after", etc.).
    Minimum,
    /// Value must be at most this amount (triggered by "no more than", "maximum", "at most").
    Maximum,
    /// Value must equal exactly this (triggered by "every N", bare frequency words).
    Exact,
}

/// A citation extracted from a policy document by WI-8.
///
/// This is the input to back matter generation (WI-12).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Citation {
    /// Unique identifier for this citation.
    pub id: String,

    /// Citation text (bibliographic reference or descriptive text).
    pub text: String,

    /// URL if this is a URL-based citation; None for bibliographic-only.
    pub url: Option<String>,

    /// `stable_id` of the `PolicyRequirement` that references this citation.
    pub source_requirement_id: Option<String>,
}

impl PolicySection {
    /// Recursively count all requirements in this section and its children.
    #[must_use]
    pub fn total_requirements(&self) -> usize {
        self.requirements.len()
            + self.children.iter().map(PolicySection::total_requirements).sum::<usize>()
    }
}

impl PolicyDocument {
    /// Recursively count all requirements across all sections.
    #[must_use]
    pub fn total_requirements(&self) -> usize {
        self.sections.iter().map(PolicySection::total_requirements).sum()
    }

    /// Recursively collect all citations from every requirement in this document.
    #[must_use]
    pub fn collect_citations(&self) -> Vec<Citation> {
        fn walk(section: &PolicySection, out: &mut Vec<Citation>) {
            for req in &section.requirements {
                out.extend(req.citations.clone());
            }
            for child in &section.children {
                walk(child, out);
            }
        }
        let mut out = Vec::new();
        for section in &self.sections {
            walk(section, &mut out);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metadata() -> DocumentMetadata {
        DocumentMetadata {
            title: "Security Policy".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            date: Some("2026-02-11".to_string()),
            source_path: PathBuf::from("policy.md"),
            content_hash: Some("abc123".to_string()),
        }
    }

    fn sample_requirement() -> PolicyRequirement {
        PolicyRequirement {
            stable_id: None,
            text: "Users must authenticate before access".to_string(),
            source_line: 15,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: vec![],
            modality: None,
            parameters: vec![],
        }
    }

    fn sample_section() -> PolicySection {
        PolicySection {
            title: "Access Control".to_string(),
            heading_level: 2,
            source_line: 10,
            body_text: Some("This section covers access control.".to_string()),
            children: vec![],
            requirements: vec![sample_requirement()],
        }
    }

    #[test]
    fn policy_document_all_fields_populated() {
        let doc = PolicyDocument {
            id: "policy".to_string(),
            metadata: sample_metadata(),
            sections: vec![sample_section()],
        };

        assert_eq!(doc.id, "policy");
        assert_eq!(doc.metadata.title, "Security Policy");
        assert_eq!(doc.metadata.version, "1.0.0");
        assert_eq!(doc.metadata.author.as_deref(), Some("Test Author"));
        assert_eq!(doc.metadata.date.as_deref(), Some("2026-02-11"));
        assert_eq!(doc.metadata.source_path, PathBuf::from("policy.md"));
        assert_eq!(doc.metadata.content_hash.as_deref(), Some("abc123"));
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "Access Control");
        assert_eq!(doc.sections[0].requirements.len(), 1);
    }

    #[test]
    fn policy_document_empty_sections() {
        let doc = PolicyDocument {
            id: "empty".to_string(),
            metadata: DocumentMetadata {
                title: "Empty".to_string(),
                version: "0.0.0".to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("empty.md"),
                content_hash: None,
            },
            sections: vec![],
        };

        assert!(doc.sections.is_empty());
        assert!(doc.metadata.author.is_none());
        assert!(doc.metadata.date.is_none());
        assert!(doc.metadata.content_hash.is_none());
    }

    #[test]
    fn policy_section_empty_requirements_and_children() {
        let section = PolicySection {
            title: "Introduction".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: vec![],
            requirements: vec![],
        };

        assert!(section.requirements.is_empty());
        assert!(section.children.is_empty());
        assert!(section.body_text.is_none());
    }

    #[test]
    fn policy_requirement_none_stable_id() {
        let req = sample_requirement();
        assert!(req.stable_id.is_none());
        assert_eq!(req.text, "Users must authenticate before access");
        assert_eq!(req.source_line, 15);
        assert_eq!(req.nesting_depth, 0);
    }

    #[test]
    fn debug_derive_produces_output() {
        let doc = PolicyDocument {
            id: "test".to_string(),
            metadata: sample_metadata(),
            sections: vec![],
        };
        let debug = format!("{doc:?}");
        assert!(debug.contains("PolicyDocument"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn clone_derive_produces_independent_copy() {
        let original = PolicyDocument {
            id: "original".to_string(),
            metadata: sample_metadata(),
            sections: vec![sample_section()],
        };
        let cloned = original.clone();

        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.sections.len(), original.sections.len());
        assert_eq!(
            cloned.sections[0].requirements[0].text,
            original.sections[0].requirements[0].text
        );
    }

    #[test]
    fn nested_sections_with_children() {
        let child = PolicySection {
            title: "Sub-section".to_string(),
            heading_level: 3,
            source_line: 20,
            body_text: None,
            children: vec![],
            requirements: vec![],
        };

        let parent = PolicySection {
            title: "Parent".to_string(),
            heading_level: 2,
            source_line: 10,
            body_text: Some("Parent body".to_string()),
            children: vec![child],
            requirements: vec![],
        };

        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].title, "Sub-section");
        assert_eq!(parent.children[0].heading_level, 3);
    }
}
