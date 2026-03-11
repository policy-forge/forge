// Contract: Summary Dashboard (044)
// This file defines the public interface for the summary module.
// Implementation must match these signatures exactly.

use std::time::Duration;

/// Outcome of OSCAL schema validation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ValidationStatus {
    /// Schema validation passed with no errors.
    Passed,
    /// Passed but with warnings (reserved for future use).
    PassedWithWarnings,
    /// Schema validation found errors. Dashboard will not be shown in this case
    /// (pipeline aborts on validation failure per EC-5), but the variant exists
    /// for unit testing the formatter.
    Failed,
    /// Validation was not executed.
    #[default]
    NotRun,
}

/// Statistics collected during a conversion pipeline run.
///
/// Populated at pipeline stage boundaries with negligible overhead
/// (counter increments only). Always returned from pipeline functions
/// regardless of whether `--summary` is set.
#[derive(Debug, Clone)]
pub struct ConversionStatistics {
    pub sections_parsed: usize,
    pub requirements_extracted: usize,
    pub controls_generated: usize,
    pub validation_status: ValidationStatus,
    pub validation_errors: usize,
    pub validation_warnings: usize,
    /// Up to 3 validation error messages for dashboard display.
    pub validation_error_messages: Vec<String>,
    pub strategy: String,
    pub output_path: String,
    pub elapsed: Duration,
}

impl Default for ConversionStatistics {
    fn default() -> Self {
        Self {
            sections_parsed: 0,
            requirements_extracted: 0,
            controls_generated: 0,
            validation_status: ValidationStatus::default(),
            validation_errors: 0,
            validation_warnings: 0,
            validation_error_messages: Vec::new(),
            strategy: String::new(),
            output_path: String::new(),
            elapsed: Duration::ZERO,
        }
    }
}

impl ConversionStatistics {
    /// Calculate mapping coverage as a percentage.
    ///
    /// Returns 0.0 when `requirements_extracted` is 0 (zero-division guard).
    /// Can exceed 100.0 when atomization produces more controls than input requirements.
    pub fn mapping_coverage(&self) -> f64 {
        if self.requirements_extracted == 0 {
            return 0.0;
        }
        (self.controls_generated as f64 / self.requirements_extracted as f64) * 100.0
    }
}

/// Format conversion statistics as a human-readable box-drawing dashboard string.
///
/// Uses ANSI color codes when `use_color` is true:
/// - Green for PASSED validation status
/// - Red for FAILED validation status
/// - Yellow for warnings and >100% coverage
///
/// # Arguments
/// * `stats` - The conversion statistics to format
/// * `use_color` - Whether to include ANSI color escape codes
///
/// # Returns
/// A formatted string suitable for printing to stdout.
pub fn format_summary_dashboard(stats: &ConversionStatistics, use_color: bool) -> String;

/// Count total sections recursively in a PolicyDocument.
pub fn count_sections(doc: &crate::model::PolicyDocument) -> usize;

/// Count total requirements recursively in a PolicyDocument.
pub fn count_requirements(doc: &crate::model::PolicyDocument) -> usize;

/// Count total controls in an OSCAL Catalog (recursive across groups).
pub fn count_catalog_controls(catalog: &crate::oscal::OscalCatalog) -> usize;

/// Format a Duration as a human-readable string.
/// - < 1s: "0.42s"
/// - 1s-60s: "3.2s"
/// - > 60s: "1m 23s"
pub fn format_elapsed(duration: Duration) -> String;
