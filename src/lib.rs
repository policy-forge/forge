//! FORGE — Framework for OSCAL Risk & Governance Execution
//!
//! Converts security policy documents (Markdown) into machine-readable
//! [OSCAL](https://pages.nist.gov/OSCAL/) (Open Security Controls Assessment
//! Language) JSON artifacts.
//!
//! Pipeline: Ingest → Parse → Atomize → Map → Serialize → Validate
//!
//! # Example
//!
//! ```no_run
//! use forge::pipeline;
//! use forge::types::OutputFormat;
//! use std::path::Path;
//!
//! let result = pipeline::run_catalog_pipeline(
//!     Path::new("policy.md"),
//!     50 * 1024 * 1024,       // 50 MB max input size
//!     &OutputFormat::Json,
//!     None,                    // no SSP import
//! );
//! match result {
//!     Ok(output) => println!("Generated {} controls", output.statistics.controls_generated),
//!     Err(e) => eprintln!("Conversion failed: {e}"),
//! }
//! ```

/// Default maximum input size for conversion paths (10 MiB).
pub const DEFAULT_MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Human-reviewed framework applicability and policy-gap analysis.
pub mod applicability;
/// Human-authored OSCAL Assessment Results construction and revision review.
pub mod assessment_results;
/// Batch processing: parallel conversion of multiple policy files.
pub mod batch;
/// Citation extraction from normative requirement text.
pub mod citation;
/// CLI argument parsing and subcommand dispatch.
pub mod cli;
/// Project configuration (`.forge.toml`): selection, validation, resolution.
pub mod config;
/// OSCAL artifact diff engine (compare two artifacts).
pub mod diff;
/// Unified error types for all pipeline stages.
pub mod error;
/// OSCAL artifact export (JSON ↔ XML ↔ YAML conversion).
pub mod export;
/// Read-only framework revision impact analysis.
pub mod framework;
/// Input file ingestion and content reconstruction.
pub mod ingest;
/// Filesystem I/O helpers and file-size limits.
pub mod io;
mod json_strict;
/// Deterministic local policy lifecycle records and review queues.
pub mod lifecycle;
/// Deterministic evidence and implementation linkage indexes and maintenance reports.
pub mod linkage;
/// Human-reviewed OSCAL Control Mapping workflows.
pub mod mapping;
/// Read-only policy revision migration analysis.
pub mod migration;
/// Core policy document domain model and assembly.
pub mod model;
/// OSCAL model construction and serialization structures.
pub mod oscal;
/// Optional external OSCAL CLI discovery and invocation.
pub mod oscal_cli;
/// Parameter extraction from policy requirement prose.
pub mod parameter;
/// Markdown parsing and policy requirement atomization.
pub mod parse;
/// End-to-end policy-to-OSCAL conversion pipelines.
pub mod pipeline;
/// Deterministic composition of local, hash-pinned Markdown policy components.
pub mod policy;
/// OSCAL artifact round-trip validation.
pub mod round_trip;
/// Input sanitization utilities (control-character stripping, etc.).
pub mod sanitize;
/// Conversion statistics and human-readable summaries.
pub mod summary;
#[cfg(any(test, debug_assertions, feature = "testing"))]
#[doc(hidden)]
pub mod testing;
/// Traceability report generation (link OSCAL elements back to source policy).
pub mod trace;
/// Shared CLI-facing output and conversion type enums.
pub mod types;
/// Deterministic UUID v5 generation for stable OSCAL identifiers.
pub mod uuid;
/// OSCAL schema and semantic validation.
pub mod validate;

pub use batch::{BatchSummary, FileOutcome, FileResult, format_batch_summary};
pub use citation::extract_citations;
pub use error::{ForgeError, exit_code};
pub use model::trace::{SourceLocation, TraceError, TraceLink, TraceLinkCollection};
pub use model::{
    Citation, ConstraintType, Modality, ParameterConstraint, ParameterType, PolicyDocument,
    PolicyParameter, PolicyRequirement, PolicySection,
};
pub use oscal::{
    BackMatter, BackMatterResource, ComponentDefinition, ComponentDefinitionEnvelope,
    ComponentDefinitionMetadata, DEFAULT_COMPONENT_TITLE, DocumentaryComponent, OscalLink,
    OscalMetadata, OscalPart, OscalProp, assemble_metadata, build_component_definition,
    generate_back_matter, generate_control_links,
};
pub use oscal::{
    ByComponent, ImplementationStatus, SspControlImplementation, SspImplementedRequirement,
    SspMetadata, SystemImplementation, SystemSecurityPlanEnvelope, build_ssp, build_ssp_skeleton,
    generate_inventory_items,
};
pub use types::{OscalModelType, OutputFormat, Strategy};
pub use validate::{
    SchemaError, ValidateError, ValidationResult, check_file_size, detect_model_type, load_schema,
    validate_artifact,
};
