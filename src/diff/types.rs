//! Types for diffing OSCAL artifacts.
//!
//! This module provides data structures for comparing two OSCAL documents
//! (catalogs or component definitions) and reporting per-control differences,
//! including field-level changes and UUID tracking.

use std::fmt;

/// The type of OSCAL artifact being diffed.
///
/// Determines how controls are extracted and compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    /// An OSCAL catalog document (contains controls, parameters, back matter).
    Catalog,
    /// An OSCAL component definition document (contains implemented components).
    ComponentDefinition,
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog => write!(f, "Catalog"),
            Self::ComponentDefinition => write!(f, "ComponentDefinition"),
        }
    }
}

/// A point-in-time snapshot of a control's key fields used for comparison.
///
/// Captures the control id, UUID, optional title/description, and extracted
/// prose from control parts so that two snapshots can be diffed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlSnapshot {
    /// The control identifier (e.g., `"ac-1"`).
    pub control_id: String,
    /// The UUID of the control.
    pub uuid: String,
    /// The control title, if present in the source document.
    pub title: Option<String>,
    /// The control description, if present in the source document.
    pub description: Option<String>,
    /// Prose content extracted from control `part` elements, joined into strings.
    pub parts_prose: Vec<String>,
}

/// A single field-level change between two control snapshots.
///
/// `None` records an absent field, preserving the distinction between absent
/// and present-but-empty values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    /// The name of the field that changed (e.g., `"title"`, `"description"`).
    pub field_name: String,
    /// The previous field value, or `None` if the field was absent.
    pub old_value: Option<String>,
    /// The new field value, or `None` if the field is absent.
    pub new_value: Option<String>,
}

/// A per-control diff entry describing how a control differs between two artifacts.
///
/// Each variant represents one of four possible outcomes: the control is new,
/// removed, has changed field values, or had only its UUID change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffEntry {
    /// The control is present only in the new artifact.
    Added {
        /// The control identifier.
        control_id: String,
        /// The UUID assigned in the new artifact.
        new_uuid: String,
    },
    /// The control is present only in the old artifact.
    Removed {
        /// The control identifier.
        control_id: String,
        /// The UUID from the old artifact.
        old_uuid: String,
    },
    /// The control exists in both artifacts but its field values differ.
    Changed {
        /// The control identifier.
        control_id: String,
        /// The UUID from the old artifact.
        old_uuid: String,
        /// The UUID from the new artifact.
        new_uuid: String,
        /// The list of field-level changes detected.
        field_changes: Vec<FieldChange>,
    },
    /// The control exists in both artifacts with identical fields but a different UUID.
    UuidChanged {
        /// The control identifier.
        control_id: String,
        /// The UUID from the old artifact.
        old_uuid: String,
        /// The UUID from the new artifact.
        new_uuid: String,
    },
}

impl DiffEntry {
    /// Returns the control identifier for this diff entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use forge::diff::types::DiffEntry;
    ///
    /// let entry = DiffEntry::Added {
    ///     control_id: "ac-1".into(),
    ///     new_uuid: "abc-123".into(),
    /// };
    /// assert_eq!(entry.control_id(), "ac-1");
    /// ```
    #[must_use]
    pub fn control_id(&self) -> &str {
        match self {
            Self::Added { control_id, .. }
            | Self::Removed { control_id, .. }
            | Self::Changed { control_id, .. }
            | Self::UuidChanged { control_id, .. } => control_id,
        }
    }

    /// Returns whether this entry records a UUID change.
    #[must_use]
    pub fn uuid_changed(&self) -> bool {
        match self {
            Self::Changed { old_uuid, new_uuid, .. }
            | Self::UuidChanged { old_uuid, new_uuid, .. } => old_uuid != new_uuid,
            Self::Added { .. } | Self::Removed { .. } => false,
        }
    }
}

/// Summary statistics for a diff operation.
///
/// Reports counts of total controls, additions, removals, changes,
/// unchanged controls, and UUID-only changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSummary {
    /// Total number of controls in the old artifact.
    pub total_old: usize,
    /// Total number of controls in the new artifact.
    pub total_new: usize,
    /// Number of controls added (present only in the new artifact).
    pub added: usize,
    /// Number of controls removed (present only in the old artifact).
    pub removed: usize,
    /// Number of controls with field-level changes.
    pub changed: usize,
    /// Number of controls that are identical between artifacts.
    pub unchanged: usize,
    /// Number of controls whose UUIDs changed but fields are otherwise identical.
    pub uuid_changes: usize,
}

impl DiffSummary {
    /// Returns `true` if any differences were detected.
    ///
    /// # Examples
    ///
    /// ```
    /// use forge::diff::types::DiffSummary;
    ///
    /// let summary = DiffSummary {
    ///     total_old: 10,
    ///     total_new: 10,
    ///     added: 0,
    ///     removed: 0,
    ///     changed: 0,
    ///     unchanged: 10,
    ///     uuid_changes: 0,
    /// };
    /// assert!(!summary.has_changes());
    ///
    /// let summary = DiffSummary {
    ///     total_old: 10,
    ///     total_new: 12,
    ///     added: 2,
    ///     removed: 0,
    ///     changed: 0,
    ///     unchanged: 10,
    ///     uuid_changes: 0,
    /// };
    /// assert!(summary.has_changes());
    /// ```
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.removed > 0 || self.changed > 0 || self.uuid_changes > 0
    }
}

/// A complete diff report comparing two OSCAL artifacts.
///
/// Contains file paths, artifact type, per-control diff entries,
/// and a summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffReport {
    /// The file path (or label) for the old/left artifact.
    pub old_file: String,
    /// The file path (or label) for the new/right artifact.
    pub new_file: String,
    /// The type of OSCAL artifact being compared.
    pub artifact_type: ArtifactType,
    /// Per-control diff entries describing each difference.
    pub entries: Vec<DiffEntry>,
    /// Aggregate summary statistics for the diff.
    pub summary: DiffSummary,
}
