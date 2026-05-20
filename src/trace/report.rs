use crate::types::OscalModelType;

/// Type of OSCAL element in a traceability report entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ElementType {
    /// An OSCAL group element (groups controls together).
    Group,
    /// An OSCAL control element (a single control statement).
    Control,
    /// An OSCAL implemented-requirement element (component definition).
    ImplementedRequirement,
}

impl ElementType {
    /// OSCAL-standard string for this element type.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Group => "group",
            Self::Control => "control",
            Self::ImplementedRequirement => "implemented-requirement",
        }
    }
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Trace metadata extracted from an OSCAL element's WI-17 trace props.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMetadata {
    /// Source file name (from `source-file` prop).
    pub source_file: String,
    /// Source section title (from `source-section` prop).
    pub source_section: String,
    /// 1-based source line number (from `source-line` prop, parsed).
    /// 0 means no line number (e.g., groups).
    pub source_line: usize,
}

/// A single entry in the traceability report.
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// OSCAL element identifier (control-id or group-id).
    pub element_id: String,
    /// Type of OSCAL element.
    pub element_type: ElementType,
    /// Resolved trace metadata. `None` if element has no WI-17 trace props.
    pub trace: Option<TraceMetadata>,
}

/// Aggregate statistics for the traceability report.
///
/// Only the two independent fields (`total_elements`, `mapped_elements`) are stored;
/// `unmapped_elements` and `coverage_percent` are computed on access.
#[derive(Debug, Clone)]
pub struct TraceSummary {
    /// Total number of OSCAL elements walked in the artifact.
    pub total_elements: usize,
    /// Number of elements that resolved to a source policy location.
    pub mapped_elements: usize,
}

impl TraceSummary {
    /// Compute summary statistics from a slice of trace entries.
    #[must_use]
    pub fn from_entries(entries: &[TraceEntry]) -> Self {
        let total_elements = entries.len();
        let mapped_elements = entries.iter().filter(|e| e.trace.is_some()).count();
        Self { total_elements, mapped_elements }
    }

    /// Number of elements without trace metadata.
    #[must_use]
    pub fn unmapped_elements(&self) -> usize {
        self.total_elements.saturating_sub(self.mapped_elements)
    }

    /// Coverage percentage (0.0–100.0). Returns 0.0 if `total_elements` is 0.
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn coverage_percent(&self) -> f64 {
        if self.total_elements == 0 {
            0.0
        } else {
            (self.mapped_elements as f64 / self.total_elements as f64) * 100.0
        }
    }
}

/// Full traceability report.
#[derive(Debug)]
pub struct TraceReport {
    /// Path to the OSCAL artifact file.
    pub artifact_path: std::path::PathBuf,
    /// Path to the source policy file.
    pub source_path: std::path::PathBuf,
    /// Detected artifact type.
    pub artifact_type: OscalModelType,
    /// All trace entries (one per walked OSCAL element).
    pub entries: Vec<TraceEntry>,
    /// Computed summary statistics.
    pub summary: TraceSummary,
    /// True if source file mtime > OSCAL metadata.last-modified.
    pub source_stale: bool,
    /// Number of lines in the source policy file, for line reference validation.
    pub source_line_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005: Unit tests for TraceMetadata, TraceEntry, TraceSummary construction

    #[test]
    fn trace_metadata_construction() {
        let meta = TraceMetadata {
            source_file: "policy.md".to_string(),
            source_section: "Access Control".to_string(),
            source_line: 42,
        };
        assert_eq!(meta.source_file, "policy.md");
        assert_eq!(meta.source_section, "Access Control");
        assert_eq!(meta.source_line, 42);
    }

    #[test]
    fn trace_entry_mapped() {
        let entry = TraceEntry {
            element_id: "POL-AC-001".to_string(),
            element_type: ElementType::Control,
            trace: Some(TraceMetadata {
                source_file: "policy.md".to_string(),
                source_section: "Access Control".to_string(),
                source_line: 10,
            }),
        };
        assert!(entry.trace.is_some());
    }

    #[test]
    fn trace_entry_unmapped() {
        let entry = TraceEntry {
            element_id: "POL-AC-002".to_string(),
            element_type: ElementType::Control,
            trace: None,
        };
        assert!(entry.trace.is_none());
    }

    #[test]
    fn summary_full_coverage() {
        let entries = vec![
            TraceEntry {
                element_id: "a".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "p.md".to_string(),
                    source_section: "S".to_string(),
                    source_line: 1,
                }),
            },
            TraceEntry {
                element_id: "b".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "p.md".to_string(),
                    source_section: "S".to_string(),
                    source_line: 2,
                }),
            },
        ];
        let summary = TraceSummary::from_entries(&entries);
        assert_eq!(summary.total_elements, 2);
        assert_eq!(summary.mapped_elements, 2);
        assert_eq!(summary.unmapped_elements(), 0);
        assert!((summary.coverage_percent() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_partial_coverage() {
        let entries = vec![
            TraceEntry {
                element_id: "a".to_string(),
                element_type: ElementType::Control,
                trace: Some(TraceMetadata {
                    source_file: "p.md".to_string(),
                    source_section: "S".to_string(),
                    source_line: 1,
                }),
            },
            TraceEntry {
                element_id: "b".to_string(),
                element_type: ElementType::Control,
                trace: None,
            },
        ];
        let summary = TraceSummary::from_entries(&entries);
        assert_eq!(summary.total_elements, 2);
        assert_eq!(summary.mapped_elements, 1);
        assert_eq!(summary.unmapped_elements(), 1);
        assert!((summary.coverage_percent() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_zero_coverage() {
        let entries = vec![TraceEntry {
            element_id: "a".to_string(),
            element_type: ElementType::Control,
            trace: None,
        }];
        let summary = TraceSummary::from_entries(&entries);
        assert_eq!(summary.total_elements, 1);
        assert_eq!(summary.mapped_elements, 0);
        assert_eq!(summary.unmapped_elements(), 1);
        assert!((summary.coverage_percent() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn summary_empty_entries() {
        let entries: Vec<TraceEntry> = vec![];
        let summary = TraceSummary::from_entries(&entries);
        assert_eq!(summary.total_elements, 0);
        assert_eq!(summary.mapped_elements, 0);
        assert_eq!(summary.unmapped_elements(), 0);
        assert!((summary.coverage_percent() - 0.0).abs() < f64::EPSILON);
    }
}
