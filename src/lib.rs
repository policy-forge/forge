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

/// Batch processing: parallel conversion of multiple policy files.
pub mod batch;
pub mod citation;
/// CLI argument parsing and subcommand dispatch.
pub mod cli;
/// OSCAL artifact diff engine (compare two artifacts).
pub mod diff;
/// Unified error types for all pipeline stages.
pub mod error;
/// OSCAL artifact export (JSON ↔ XML ↔ YAML conversion).
pub mod export;
pub mod ingest;
pub mod io;
pub mod model;
pub mod oscal;
pub mod oscal_cli;
pub mod parameter;
pub mod parse;
pub mod pipeline;
pub mod round_trip;
/// Input sanitization utilities (control-character stripping, etc.).
pub mod sanitize;
pub mod summary;
#[doc(hidden)]
pub mod testing;
/// Traceability report generation (link OSCAL elements back to source policy).
pub mod trace;
pub mod types;
/// Deterministic UUID v5 generation for stable OSCAL identifiers.
pub mod uuid;
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
