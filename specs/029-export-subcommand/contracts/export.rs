// ─── Contract: Export Subcommand (WI-29) ─────────────────────────────────
//
// This file defines the interface contract for the export subcommand.
// Implementation MUST conform to these type definitions and function signatures.
// Source: AR 029-ar-export-subcommand.md + research.md decisions.
//
// NOTE: This is a design contract, not compilable code. It references types
// from the existing codebase and defines the new public API surface.

use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};

use crate::cli::OutputFormat; // Reuse existing enum (RQ-5)
use crate::error::ForgeError;

// ─── CLI Arguments ────────────────────────────────────────────────────────

/// CLI arguments for the `forge export` subcommand.
///
/// Usage: `forge export <input> --format <json|xml|yaml> [--output <path>]`
///
/// Traces to: PRD M-1, AR ExportArgs
#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Path to the input OSCAL artifact (JSON, XML, or YAML)
    pub input: PathBuf,

    /// Target output format
    #[arg(long, value_enum)]
    pub format: OutputFormat,

    /// Output file path (default: stdout)
    #[arg(long)]
    pub output: Option<PathBuf>,
}

// ─── Internal Model Wrapper ──────────────────────────────────────────────

/// Wrapper enum for deserialized OSCAL models.
///
/// Used during the export pipeline to hold the deserialized model
/// before re-serialization to the target format.
pub enum OscalModel {
    /// OSCAL Catalog (envelope: `{"catalog": {...}}`)
    Catalog(crate::oscal::CatalogEnvelope),
    /// OSCAL Component Definition (envelope: `{"component-definition": {...}}`)
    Component(crate::oscal::ComponentDefinitionEnvelope),
}

// ─── Core Functions ──────────────────────────────────────────────────────

/// Detect the OSCAL format of an input file from its file extension.
///
/// Strictly extension-based detection (PRD M-2, AR constraint).
///
/// # Supported Extensions
/// - `.json` → `OutputFormat::Json`
/// - `.xml` → `OutputFormat::Xml`
/// - `.yaml`, `.yml` → `OutputFormat::Yaml`
///
/// # Errors
/// - `ForgeError::ExportUnsupportedExtension` if extension is unrecognized
/// - `ForgeError::ExportNoExtension` if no extension is present
///
/// Traces to: PRD M-2, SEC-4
pub fn detect_format(path: &Path) -> Result<OutputFormat, ForgeError>;

/// Deserialize an OSCAL artifact from a string in the specified format.
///
/// Detects the OSCAL model type (Catalog vs ComponentDefinition) and
/// deserializes into the appropriate envelope struct.
///
/// # Errors
/// - `ForgeError::ExportInvalidOscal` if input is not a valid OSCAL artifact
/// - `ForgeError::Serialization` if format-specific parsing fails
///
/// Traces to: PRD M-3, PRD M-6
pub fn deserialize_oscal(content: &str, format: OutputFormat) -> Result<OscalModel, ForgeError>;

/// Serialize an OSCAL model to a string in the specified format.
///
/// Delegates to the appropriate serializer:
/// - JSON: `serde_json::to_string_pretty()`
/// - XML: `serialize_catalog_to_xml()` / `serialize_component_definition_to_xml()`
/// - YAML: `serialize_to_yaml()`
///
/// # Errors
/// - `ForgeError::Serialization` if serialization fails
///
/// Traces to: PRD M-3
pub fn serialize_oscal(model: &OscalModel, format: OutputFormat) -> Result<String, ForgeError>;

/// Validate an OSCAL model against the JSON schema.
///
/// Serializes the model to JSON, then validates against the appropriate
/// OSCAL v1.2.0 JSON schema using the existing `validate_artifact()`.
///
/// # Errors
/// - `ForgeError::SchemaValidation` if validation fails
///
/// Traces to: PRD M-4, SEC-7
pub fn validate_oscal_model(model: &OscalModel) -> Result<(), ForgeError>;

/// Execute the full export pipeline: read → detect → deserialize → validate → serialize → write.
///
/// This is the top-level orchestration function called by the CLI handler.
///
/// Pipeline stages:
/// 1. Read input file to string
/// 2. Detect input format from file extension
/// 3. Deserialize to internal OSCAL model
/// 4. Validate model against OSCAL JSON schema
/// 5. Serialize model to target format
/// 6. Write output to stdout or file
///
/// # Errors
/// - `ForgeError::FileNotFound` if input file does not exist
/// - `ForgeError::ExportEmptyInput` if input file is empty
/// - `ForgeError::ExportUnsupportedExtension` / `ExportNoExtension` for format detection
/// - `ForgeError::ExportInvalidOscal` if input is not valid OSCAL
/// - `ForgeError::SchemaValidation` if model fails validation
/// - `ForgeError::Serialization` if output serialization fails
/// - `ForgeError::Io` if file write fails
///
/// Traces to: PRD M-1 through M-6, AR export_artifact
pub fn export_artifact(
    input_path: &Path,
    target_format: OutputFormat,
    output: Option<&Path>,
) -> Result<(), ForgeError>;

// ─── Error Variants (additions to ForgeError) ────────────────────────────

// New variants to add to the ForgeError enum in src/error.rs:
//
// /// Unrecognized file extension for OSCAL format detection.
// #[error(
//     "Unrecognized file extension '.{extension}' on input file. \
//      Expected .json, .xml, .yaml, or .yml for OSCAL artifacts."
// )]
// ExportUnsupportedExtension { extension: String },
//
// /// No file extension found on input file.
// #[error(
//     "No file extension on input file '{}'. \
//      Cannot determine OSCAL format. Expected .json, .xml, .yaml, or .yml.",
//     path.display()
// )]
// ExportNoExtension { path: PathBuf },
//
// /// Input file is not a valid OSCAL artifact.
// #[error("Input is not a valid OSCAL artifact: {detail}")]
// ExportInvalidOscal { detail: String },
//
// /// Export input file is empty.
// #[error("Export input file is empty: '{}'", path.display())]
// ExportEmptyInput { path: PathBuf },
//
// Exit code mapping: all four map to exit code 1 (input/IO errors).
