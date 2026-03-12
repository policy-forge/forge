//! Summary dashboard: conversion statistics and counting helpers.
//!
//! Provides `ConversionStatistics` (accumulated during pipeline execution)
//! and a helper for counting catalog controls.

pub mod format;

use std::path::PathBuf;
use std::time::Duration;

use crate::oscal::catalog::OscalCatalog;
use crate::types::Strategy;

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
    /// Currently always 0 in production: pipeline aborts on validation failure (EC-5)
    /// before stats are constructed. Retained for formatter unit tests and future
    /// `--force` flag that could continue past validation errors.
    pub validation_errors: usize,
    /// Currently always 0 in production (see `validation_errors` note).
    pub validation_warnings: usize,
    /// Up to 3 validation error messages for dashboard display.
    /// Currently always empty in production (see `validation_errors` note).
    pub validation_error_messages: Vec<String>,
    pub strategy: Strategy,
    pub output_path: Option<PathBuf>,
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
            strategy: Strategy::Catalog,
            output_path: None,
            elapsed: Duration::ZERO,
        }
    }
}

impl ConversionStatistics {
    /// Calculate mapping coverage as a percentage.
    ///
    /// Returns 0.0 when `requirements_extracted` is 0 (zero-division guard).
    /// Can exceed 100.0 when atomization produces more controls than input requirements.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn mapping_coverage(&self) -> f64 {
        if self.requirements_extracted == 0 {
            return 0.0;
        }
        (self.controls_generated as f64 / self.requirements_extracted as f64) * 100.0
    }
}

/// Count total controls in an OSCAL Catalog (across groups).
#[must_use]
pub fn count_catalog_controls(catalog: &OscalCatalog) -> usize {
    use crate::oscal::catalog::OscalGroup;
    fn count_group_controls(groups: &[OscalGroup]) -> usize {
        groups.iter().map(|g| g.controls.len()).sum()
    }
    count_group_controls(&catalog.groups)
}

#[cfg(test)]
mod tests {
    use super::*;

    // T004: mapping_coverage() tests
    #[test]
    fn mapping_coverage_zero_requirements_returns_zero() {
        let stats = ConversionStatistics {
            requirements_extracted: 0,
            controls_generated: 5,
            ..Default::default()
        };
        assert!((stats.mapping_coverage() - 0.0).abs() < 0.01);
    }

    #[test]
    fn mapping_coverage_partial() {
        let stats = ConversionStatistics {
            requirements_extracted: 15,
            controls_generated: 12,
            ..Default::default()
        };
        assert!((stats.mapping_coverage() - 80.0).abs() < 0.01);
    }

    #[test]
    fn mapping_coverage_full() {
        let stats = ConversionStatistics {
            requirements_extracted: 15,
            controls_generated: 15,
            ..Default::default()
        };
        assert!((stats.mapping_coverage() - 100.0).abs() < 0.01);
    }

    #[test]
    fn mapping_coverage_exceeds() {
        let stats = ConversionStatistics {
            requirements_extracted: 15,
            controls_generated: 18,
            ..Default::default()
        };
        assert!((stats.mapping_coverage() - 120.0).abs() < 0.01);
    }

    // T006: count_catalog_controls() tests
    #[test]
    fn count_catalog_controls_empty() {
        let catalog = OscalCatalog {
            uuid: String::new(),
            metadata: crate::oscal::catalog::OscalMetadata {
                title: String::new(),
                last_modified: String::new(),
                version: String::new(),
                oscal_version: String::new(),
            },
            controls: vec![],
            groups: vec![],
            back_matter: None,
        };
        assert_eq!(count_catalog_controls(&catalog), 0);
    }

    #[test]
    fn count_catalog_controls_single_group() {
        use crate::oscal::catalog::{OscalControl, OscalGroup, OscalMetadata};
        let catalog = OscalCatalog {
            uuid: String::new(),
            metadata: OscalMetadata {
                title: String::new(),
                last_modified: String::new(),
                version: String::new(),
                oscal_version: String::new(),
            },
            controls: vec![],
            groups: vec![OscalGroup {
                id: "g1".into(),
                title: "Group 1".into(),
                props: vec![],
                links: vec![],
                controls: vec![
                    OscalControl {
                        id: "c1".into(),
                        uuid: String::new(),
                        title: "Control 1".into(),
                        links: vec![],
                        params: vec![],
                        parts: vec![],
                        props: vec![],
                    },
                    OscalControl {
                        id: "c2".into(),
                        uuid: String::new(),
                        title: "Control 2".into(),
                        links: vec![],
                        params: vec![],
                        parts: vec![],
                        props: vec![],
                    },
                ],
                groups: vec![],
            }],
            back_matter: None,
        };
        assert_eq!(count_catalog_controls(&catalog), 2);
    }

    #[test]
    fn count_catalog_controls_multiple_groups() {
        use crate::oscal::catalog::{OscalControl, OscalGroup, OscalMetadata};
        let make_control = |id: &str| OscalControl {
            id: id.into(),
            uuid: String::new(),
            title: format!("Control {id}"),
            links: vec![],
            params: vec![],
            parts: vec![],
            props: vec![],
        };
        let catalog = OscalCatalog {
            uuid: String::new(),
            metadata: OscalMetadata {
                title: String::new(),
                last_modified: String::new(),
                version: String::new(),
                oscal_version: String::new(),
            },
            controls: vec![],
            groups: vec![
                OscalGroup {
                    id: "g1".into(),
                    title: "Group 1".into(),
                    props: vec![],
                    links: vec![],
                    controls: vec![make_control("c1"), make_control("c2")],
                    groups: vec![],
                },
                OscalGroup {
                    id: "g2".into(),
                    title: "Group 2".into(),
                    props: vec![],
                    links: vec![],
                    controls: vec![make_control("c3")],
                    groups: vec![],
                },
            ],
            back_matter: None,
        };
        assert_eq!(count_catalog_controls(&catalog), 3);
    }
}
