//! Error types for the FORGE Markdown-to-OSCAL pipeline.
//!
//! This module defines [`ForgeError`], a comprehensive error enum covering
//! all failure modes in the pipeline: input/IO errors, parse/structure errors,
//! validation/config errors, export errors, argument errors, batch errors,
//! external dependency errors, and diff errors.
//!
//! The [`exit_code`] function maps each variant to a CLI exit code:
//! - `1`: Input/IO errors
//! - `2`: Parse/Structure errors
//! - `3`: Validation/Config errors
//! - `4`: External dependency unavailable

use std::path::PathBuf;

use thiserror::Error;

#[allow(clippy::cast_precision_loss)] // Precision loss is acceptable for human-readable file sizes
#[allow(clippy::trivially_copy_pass_by_ref)] // thiserror passes struct fields by reference
fn format_size(bytes: &u64) -> String {
    let bytes = *bytes;
    if bytes < 1_048_576 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    }
}

/// Comprehensive error type for all FORGE pipeline failures.
///
/// Each variant is annotated with `#[error("...")]` (via [`thiserror`]) for
/// automatic [`Display`](std::fmt::Display) and [`Error`](std::error::Error) impls. Variants are organized into
/// logical groups matching CLI exit code categories.
#[derive(Debug, Error)]
pub enum ForgeError {
    // --- Input/IO errors (exit code 1) ---
    /// The specified input file does not exist.
    #[error("File not found: '{}'", path.display())]
    FileNotFound {
        /// The path that could not be found.
        path: PathBuf,
    },

    /// The current user lacks read permission for the specified file.
    #[error("Permission denied: '{}'", path.display())]
    PermissionDenied {
        /// The path with insufficient permissions.
        path: PathBuf,
    },

    /// The input file exists but contains no content.
    #[error(
        "File is empty: '{}' — provide a non-empty Markdown policy document",
        path.display()
    )]
    EmptyInput {
        /// The path to the empty file.
        path: PathBuf,
    },

    /// The file appears to contain binary data rather than UTF-8 text.
    #[error(
        "File appears to be binary, not a text document: '{}'. \
         FORGE accepts UTF-8 Markdown (.md) files.",
        path.display()
    )]
    BinaryFile {
        /// The path to the binary file.
        path: PathBuf,
    },

    /// A PDF contained no extractable text; OCR is not supported by this release.
    #[error(
        "PDF has no extractable text: '{}'. OCR for scanned/image-only PDFs is not supported; provide a text-based PDF, DOCX, or Markdown source.",
        path.display()
    )]
    OcrNotSupported {
        /// The path to the scanned/image-only PDF.
        path: PathBuf,
    },

    /// The file extension is not a supported Markdown format.
    #[error(
        "Unsupported file format '.{extension}'. Only Markdown files (.md, .markdown) are supported. \
         Consider converting with pandoc or markitdown."
    )]
    UnsupportedFormat {
        /// The unsupported file extension (without leading dot).
        extension: String,
    },

    /// The input file exceeds the maximum allowed size.
    #[error(
        "File '{}' is {}, exceeding the {} limit. Use --max-size to increase the limit.",
        path.display(),
        format_size(.size_bytes),
        format_size(.limit_bytes)
    )]
    FileTooLarge {
        /// The path to the oversized file.
        path: PathBuf,
        /// The actual file size in bytes.
        size_bytes: u64,
        /// The configured size limit in bytes.
        limit_bytes: u64,
    },

    /// The file is not valid UTF-8 text.
    #[error(
        "File '{}' is not valid UTF-8 text. FORGE requires UTF-8 encoded Markdown files.",
        path.display()
    )]
    InvalidEncoding {
        /// The path to the file with invalid encoding.
        path: PathBuf,
    },

    /// The path does not refer to a regular file (e.g., it is a directory or symlink).
    #[error("'{}' is not a regular file.", path.display())]
    NotAFile {
        /// The path that is not a regular file.
        path: PathBuf,
    },

    /// A low-level I/O error (from [`std::io`]).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // --- Parse/Structure errors (exit code 2) ---
    /// The input file contains no recognizable policy structure (headings or numbered clauses).
    #[error(
        "No policy structure detected in '{}' — expected Markdown headings (# Section) or numbered clauses",
        path.display()
    )]
    NoStructureDetected {
        /// The path to the file lacking structure.
        path: PathBuf,
    },

    /// A generic Markdown parsing error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// An error occurred while building an OSCAL catalog.
    #[error("Catalog build error: {0}")]
    CatalogBuild(String),

    /// An error occurred while processing back matter.
    #[error("Back matter error: {0}")]
    BackMatter(String),

    /// An error occurred while building a component definition.
    #[error("Component definition build error: {0}")]
    ComponentDefinitionBuild(String),

    /// An error occurred while building an assessment plan.
    #[error("Assessment plan build error: {0}")]
    AssessmentPlanBuild(String),

    /// An error occurred while building an SSP (System Security Plan).
    #[error("SSP build error: {0}")]
    SspBuild(String),

    /// An error occurred during parameter extraction.
    #[error("Parameter extraction error: {0}")]
    ParameterExtraction(String),

    /// The OSCAL artifact type is not supported for tracing.
    #[error("Unsupported OSCAL artifact type for tracing: {detail}")]
    TraceUnsupportedArtifact {
        /// Human-readable detail about the unsupported type.
        detail: String,
    },

    /// The input file contains multiple OSCAL model types instead of exactly one.
    #[error(
        "Ambiguous OSCAL artifact: file contains multiple model types ({detail}). Each file must contain exactly one OSCAL model."
    )]
    AmbiguousArtifact {
        /// A description of the conflicting model types found.
        detail: String,
    },

    // --- Validation/Config errors (exit code 3) ---
    /// A document validation error (content rule violations).
    #[error("Validation error: {0}")]
    Validation(String),

    /// A configuration error (invalid CLI options, environment, etc.).
    #[error("Configuration error: {0}")]
    Config(String),

    // --- Export errors (exit code 1) ---
    /// The input file extension does not match a supported OSCAL serialization format.
    #[error(
        "Unrecognized file extension '.{extension}' on input file. \
         Expected .json, .xml, .yaml, or .yml for OSCAL artifacts."
    )]
    ExportUnsupportedExtension {
        /// The unrecognized file extension.
        extension: String,
    },

    /// The input file has no extension, so the OSCAL format cannot be determined.
    #[error(
        "No file extension on input file '{}'. \
         Cannot determine OSCAL format. Expected .json, .xml, .yaml, or .yml.",
        path.display()
    )]
    ExportNoExtension {
        /// The path to the file with no extension.
        path: PathBuf,
    },

    /// The input is not valid JSON, XML, or YAML representing an OSCAL artifact.
    #[error("Input is not a valid OSCAL artifact: {detail}")]
    ExportInvalidOscal {
        /// Human-readable detail about why the artifact is invalid.
        detail: String,
    },

    /// The export input file is empty.
    #[error("Export input file is empty: '{}'", path.display())]
    ExportEmptyInput {
        /// The path to the empty export input file.
        path: PathBuf,
    },

    // --- Argument errors (exit code 1) ---
    /// An invalid command-line argument was provided.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    // --- Batch errors (exit code 1) ---
    /// An error occurred during batch conversion of multiple files.
    #[error("Batch conversion error: {0}")]
    BatchConversion(String),

    // --- Usage errors (exit code 2) ---
    /// A required command argument was not supplied by any layer (CLI or
    /// project configuration). Mirrors clap's usage-error behavior so the
    /// no-config exit code remains unchanged (PRD 051 M-14).
    ///
    /// Displayed without its own `error:` prefix because `main` already wraps
    /// every message in `Error: …`; carrying both prefixes produced
    /// `Error: error: …`.
    #[error("{0}")]
    MissingRequiredArgument(String),

    // --- External dependency errors (exit code 4) ---
    /// The `oscal-cli` tool was not found on the system PATH.
    #[error(
        "oscal-cli not found on system PATH. Install from: https://github.com/usnistgov/oscal-cli"
    )]
    OscalCliNotFound,

    /// The `oscal-cli` binary was located but is not functional.
    #[error("oscal-cli found at '{}' but is not functional: {detail}", path.display())]
    OscalCliNotFunctional {
        /// The path where `oscal-cli` was found.
        path: PathBuf,
        /// Human-readable detail about why the tool is not functional.
        detail: String,
    },

    // --- oscal-cli execution errors (exit code 1) ---
    /// The `oscal-cli` process finished with a non-zero exit code.
    #[error("oscal-cli execution failed (exit code {exit_code:?}): {message}")]
    OscalCliExecution {
        /// The exit code returned by `oscal-cli`, or `None` if terminated by signal.
        exit_code: Option<i32>,
        /// The error message or combined stderr/stdout output.
        message: String,
    },

    /// The `oscal-cli` process exceeded the configured timeout.
    #[error("oscal-cli execution timed out after {timeout:?}")]
    OscalCliTimeout {
        /// The timeout duration that was exceeded.
        timeout: std::time::Duration,
    },

    /// Profile resolution requires JSON input, but a different format was provided.
    #[error(
        "Expected JSON input file, got: '{}'. Only .json files are supported for profile resolution.",
        path.display()
    )]
    ResolveInputNotJson {
        /// The path to the non-JSON input file.
        path: PathBuf,
    },

    // --- Diff errors (exit code 2 for DiffError, exit code 1 for DiffHasChanges) ---
    /// The diff operation detected changes between golden and current output.
    ///
    /// This variant has an empty error message because the diff details are
    /// printed separately by the CLI.
    #[error("")]
    DiffHasChanges,

    /// Round-trip validation failed: the re-parsed output does not match the original.
    #[error("Round-trip validation failed: {0} unresolved divergence(s)")]
    RoundTripFailed(usize),

    /// A generic error during diff computation.
    #[error("Diff error: {0}")]
    DiffError(String),

    // --- Other (exit code 1) ---
    /// Serialization to JSON, XML, or YAML failed.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Validation against the OSCAL JSON Schema failed.
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
}

/// Map a [`ForgeError`] to a CLI exit code.
///
/// Exit code categories:
/// - `0`: Success (not handled here — only error cases)
/// - `1`: Input/IO errors (file not found, permission denied, empty, binary, encoding, size, I/O)
/// - `2`: Parse/Structure errors (no structure, parse failure, build errors)
/// - `3`: Validation/Config errors (schema violations, config issues)
/// - `4`: External dependency unavailable (oscal-cli not found or not functional)
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use forge::error::{ForgeError, exit_code};
///
/// let err = ForgeError::FileNotFound { path: PathBuf::from("missing.md") };
/// assert_eq!(exit_code(&err), 1);
///
/// let err = ForgeError::Parse("bad syntax".into());
/// assert_eq!(exit_code(&err), 2);
///
/// let err = ForgeError::Validation("bad field".into());
/// assert_eq!(exit_code(&err), 3);
///
/// let err = ForgeError::OscalCliNotFound;
/// assert_eq!(exit_code(&err), 4);
/// ```
#[must_use]
pub fn exit_code(err: &ForgeError) -> u8 {
    match err {
        // Exit 1: Input/IO errors
        ForgeError::FileNotFound { .. }
        | ForgeError::PermissionDenied { .. }
        | ForgeError::EmptyInput { .. }
        | ForgeError::BinaryFile { .. }
        | ForgeError::OcrNotSupported { .. }
        | ForgeError::UnsupportedFormat { .. }
        | ForgeError::FileTooLarge { .. }
        | ForgeError::InvalidEncoding { .. }
        | ForgeError::NotAFile { .. }
        | ForgeError::Io(_)
        | ForgeError::ExportUnsupportedExtension { .. }
        | ForgeError::ExportNoExtension { .. }
        | ForgeError::ExportInvalidOscal { .. }
        | ForgeError::ExportEmptyInput { .. }
        | ForgeError::InvalidArgument(_)
        | ForgeError::BatchConversion(_)
        | ForgeError::OscalCliExecution { .. }
        | ForgeError::OscalCliTimeout { .. }
        | ForgeError::ResolveInputNotJson { .. }
        | ForgeError::Serialization(_)
        | ForgeError::DiffHasChanges
        | ForgeError::RoundTripFailed(_) => 1,

        // Exit 2: Parse/Structure errors + usage/required-argument errors
        ForgeError::MissingRequiredArgument(_)
        | ForgeError::DiffError(_)
        | ForgeError::SspBuild(_)
        | ForgeError::NoStructureDetected { .. }
        | ForgeError::Parse(_)
        | ForgeError::CatalogBuild(_)
        | ForgeError::BackMatter(_)
        | ForgeError::ComponentDefinitionBuild(_)
        | ForgeError::AssessmentPlanBuild(_)
        | ForgeError::ParameterExtraction(_)
        | ForgeError::TraceUnsupportedArtifact { .. }
        | ForgeError::AmbiguousArtifact { .. } => 2,

        // Exit 3: Validation/Config errors
        ForgeError::Validation(_) | ForgeError::Config(_) | ForgeError::SchemaValidation(_) => 3,

        // Exit 4: External dependency unavailable
        ForgeError::OscalCliNotFound | ForgeError::OscalCliNotFunctional { .. } => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- T004: New variant Display tests ---

    #[test]
    fn file_not_found_display() {
        let err = ForgeError::FileNotFound { path: PathBuf::from("missing.md") };
        assert_eq!(err.to_string(), "File not found: 'missing.md'");
    }

    #[test]
    fn permission_denied_display() {
        let err = ForgeError::PermissionDenied { path: PathBuf::from("restricted.md") };
        assert_eq!(err.to_string(), "Permission denied: 'restricted.md'");
    }

    #[test]
    fn empty_input_display() {
        let err = ForgeError::EmptyInput { path: PathBuf::from("empty.md") };
        assert_eq!(
            err.to_string(),
            "File is empty: 'empty.md' — provide a non-empty Markdown policy document"
        );
    }

    #[test]
    fn binary_file_display() {
        let err = ForgeError::BinaryFile { path: PathBuf::from("image.png") };
        assert_eq!(
            err.to_string(),
            "File appears to be binary, not a text document: 'image.png'. \
             FORGE accepts UTF-8 Markdown (.md) files."
        );
    }

    #[test]
    fn no_structure_detected_display() {
        let err = ForgeError::NoStructureDetected { path: PathBuf::from("flat.md") };
        assert_eq!(
            err.to_string(),
            "No policy structure detected in 'flat.md' — expected Markdown headings (# Section) or numbered clauses"
        );
    }

    // --- T005: exit_code tests ---

    #[test]
    fn missing_required_argument_display_has_no_extra_prefix() {
        let err = ForgeError::MissingRequiredArgument(
            "the following required arguments were not provided:\n  --strategy <STRATEGY>"
                .to_string(),
        );
        let rendered = err.to_string();
        assert!(rendered.starts_with("the following required arguments"), "{rendered}");
        assert!(!rendered.starts_with("error:"), "prefix duplication with main's `Error:` wrapper");
        assert!(rendered.contains("--strategy"), "{rendered}");
    }

    #[test]
    fn missing_required_argument_exit_code_is_2() {
        assert_eq!(exit_code(&ForgeError::MissingRequiredArgument("test".into())), 2);
    }

    #[test]
    fn batch_conversion_error_display() {
        let err = ForgeError::BatchConversion("thread pool failed".to_string());
        assert_eq!(err.to_string(), "Batch conversion error: thread pool failed");
    }

    #[test]
    fn batch_conversion_exit_code_is_1() {
        assert_eq!(exit_code(&ForgeError::BatchConversion("test".into())), 1);
    }

    #[test]
    fn exit_code_input_io_errors_return_1() {
        assert_eq!(exit_code(&ForgeError::FileNotFound { path: PathBuf::from("a") }), 1);
        assert_eq!(exit_code(&ForgeError::PermissionDenied { path: PathBuf::from("a") }), 1);
        assert_eq!(exit_code(&ForgeError::EmptyInput { path: PathBuf::from("a") }), 1);
        assert_eq!(exit_code(&ForgeError::BinaryFile { path: PathBuf::from("a") }), 1);
        assert_eq!(exit_code(&ForgeError::UnsupportedFormat { extension: "pdf".into() }), 1);
        assert_eq!(
            exit_code(&ForgeError::FileTooLarge {
                path: PathBuf::from("a"),
                size_bytes: 100,
                limit_bytes: 50
            }),
            1
        );
        assert_eq!(exit_code(&ForgeError::InvalidEncoding { path: PathBuf::from("a") }), 1);
        assert_eq!(exit_code(&ForgeError::NotAFile { path: PathBuf::from("a") }), 1);
        let io_err = std::io::Error::other("test");
        assert_eq!(exit_code(&ForgeError::Io(io_err)), 1);
        assert_eq!(exit_code(&ForgeError::Serialization("s".into())), 1);
    }

    #[test]
    fn exit_code_parse_structure_errors_return_2() {
        assert_eq!(exit_code(&ForgeError::NoStructureDetected { path: PathBuf::from("a") }), 2);
        assert_eq!(exit_code(&ForgeError::Parse("p".into())), 2);
        assert_eq!(exit_code(&ForgeError::CatalogBuild("c".into())), 2);
        assert_eq!(exit_code(&ForgeError::BackMatter("b".into())), 2);
        assert_eq!(exit_code(&ForgeError::ComponentDefinitionBuild("d".into())), 2);
        assert_eq!(exit_code(&ForgeError::ParameterExtraction("x".into())), 2);
    }

    #[test]
    fn exit_code_validation_config_errors_return_3() {
        assert_eq!(exit_code(&ForgeError::Validation("v".into())), 3);
        assert_eq!(exit_code(&ForgeError::Config("c".into())), 3);
    }

    // --- Existing tests ---

    #[test]
    fn io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ForgeError::Io(io_err);
        assert_eq!(err.to_string(), "I/O error: file not found");
    }

    #[test]
    fn parse_error_display() {
        let err = ForgeError::Parse("unexpected token".to_string());
        assert_eq!(err.to_string(), "Parse error: unexpected token");
    }

    #[test]
    fn validation_error_display() {
        let err = ForgeError::Validation("missing required field".to_string());
        assert_eq!(err.to_string(), "Validation error: missing required field");
    }

    #[test]
    fn config_error_display() {
        let err = ForgeError::Config("invalid setting".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid setting");
    }

    #[test]
    fn unsupported_format_display() {
        let err = ForgeError::UnsupportedFormat { extension: "pdf".to_string() };
        assert_eq!(
            err.to_string(),
            "Unsupported file format '.pdf'. Only Markdown files (.md, .markdown) are supported. \
             Consider converting with pandoc or markitdown."
        );
    }

    #[test]
    fn unsupported_format_empty_extension_display() {
        let err = ForgeError::UnsupportedFormat { extension: String::new() };
        assert_eq!(
            err.to_string(),
            "Unsupported file format '.'. Only Markdown files (.md, .markdown) are supported. \
             Consider converting with pandoc or markitdown."
        );
    }

    #[test]
    fn file_too_large_display() {
        let err = ForgeError::FileTooLarge {
            path: PathBuf::from("/tmp/huge.md"),
            size_bytes: 15 * 1_048_576,
            limit_bytes: 10 * 1_048_576,
        };
        assert_eq!(
            err.to_string(),
            "File '/tmp/huge.md' is 15.0MB, exceeding the 10.0MB limit. Use --max-size to increase the limit."
        );
    }

    #[test]
    fn invalid_encoding_display() {
        let err = ForgeError::InvalidEncoding { path: PathBuf::from("/tmp/binary.md") };
        assert_eq!(
            err.to_string(),
            "File '/tmp/binary.md' is not valid UTF-8 text. FORGE requires UTF-8 encoded Markdown files."
        );
    }

    #[test]
    fn not_a_file_display() {
        let err = ForgeError::NotAFile { path: PathBuf::from("/tmp/somedir") };
        assert_eq!(err.to_string(), "'/tmp/somedir' is not a regular file.");
    }

    #[test]
    fn catalog_build_error_display() {
        let err = ForgeError::CatalogBuild("missing stable_id".to_string());
        assert_eq!(err.to_string(), "Catalog build error: missing stable_id");
    }

    #[test]
    fn io_error_from_conversion() {
        fn produce_io_error() -> Result<(), ForgeError> {
            let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
            Err(io_err)?
        }

        let result = produce_io_error();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ForgeError::Io(_)), "Expected ForgeError::Io, got: {err:?}");
        assert_eq!(err.to_string(), "I/O error: access denied");
    }

    #[test]
    fn back_matter_error_display() {
        let err = ForgeError::BackMatter("invalid citation".to_string());
        assert_eq!(err.to_string(), "Back matter error: invalid citation");
    }

    #[test]
    fn serialization_error_display() {
        let err = ForgeError::Serialization("failed to serialize".to_string());
        assert_eq!(err.to_string(), "Serialization error: failed to serialize");
    }

    #[test]
    fn assessment_plan_build_error_display() {
        let err = ForgeError::AssessmentPlanBuild("metadata assembly failed".to_string());
        assert_eq!(err.to_string(), "Assessment plan build error: metadata assembly failed");
    }

    #[test]
    fn assessment_plan_build_exit_code_is_2() {
        assert_eq!(exit_code(&ForgeError::AssessmentPlanBuild("test".into())), 2);
    }

    #[test]
    fn component_definition_build_error_display() {
        let err = ForgeError::ComponentDefinitionBuild("missing field".to_string());
        assert_eq!(err.to_string(), "Component definition build error: missing field");
    }

    #[test]
    fn trace_unsupported_artifact_display() {
        let err = ForgeError::TraceUnsupportedArtifact {
            detail: "Expected 'catalog' or 'component-definition'".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Unsupported OSCAL artifact type for tracing: Expected 'catalog' or 'component-definition'"
        );
    }

    #[test]
    fn trace_unsupported_artifact_exit_code() {
        assert_eq!(
            exit_code(&ForgeError::TraceUnsupportedArtifact { detail: "test".to_string() }),
            2
        );
    }

    #[test]
    fn ambiguous_artifact_display() {
        let err =
            ForgeError::AmbiguousArtifact { detail: "catalog, component-definition".to_string() };
        assert!(err.to_string().contains("Ambiguous OSCAL artifact"));
        assert!(err.to_string().contains("catalog, component-definition"));
    }

    #[test]
    fn ambiguous_artifact_exit_code_is_2() {
        assert_eq!(exit_code(&ForgeError::AmbiguousArtifact { detail: "test".to_string() }), 2);
    }

    #[test]
    fn schema_validation_error_display() {
        let err = ForgeError::SchemaValidation("3 errors found".to_string());
        assert_eq!(err.to_string(), "Schema validation failed: 3 errors found");
    }

    // --- T007: oscal-cli error variant Display tests ---

    #[test]
    fn oscal_cli_not_found_display() {
        let err = ForgeError::OscalCliNotFound;
        assert!(err.to_string().contains("oscal-cli not found"));
        assert!(err.to_string().contains("https://github.com/usnistgov/oscal-cli"));
    }

    #[test]
    fn oscal_cli_not_functional_display() {
        let err = ForgeError::OscalCliNotFunctional {
            path: PathBuf::from("/usr/bin/oscal-cli"),
            detail: "Java runtime not found".to_string(),
        };
        assert!(err.to_string().contains("/usr/bin/oscal-cli"));
        assert!(err.to_string().contains("not functional"));
        assert!(err.to_string().contains("Java runtime not found"));
    }

    #[test]
    fn oscal_cli_execution_display() {
        let err = ForgeError::OscalCliExecution {
            exit_code: Some(1),
            message: "Invalid profile".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("oscal-cli execution failed"));
        assert!(display.contains("Invalid profile"));
        assert!(!display.contains("stderr"), "stderr should not appear in display");
    }

    #[test]
    #[allow(clippy::duration_suboptimal_units)]
    fn oscal_cli_timeout_display() {
        let err = ForgeError::OscalCliTimeout { timeout: std::time::Duration::from_secs(60) };
        assert!(err.to_string().contains("timed out"));
        assert!(err.to_string().contains("60"));
    }

    #[test]
    fn resolve_input_not_json_display() {
        let err = ForgeError::ResolveInputNotJson { path: PathBuf::from("profile.xml") };
        assert!(err.to_string().contains("profile.xml"));
        assert!(err.to_string().contains("Only .json files"));
    }

    // --- T008: oscal-cli exit code mapping tests ---

    #[test]
    fn exit_code_oscal_cli_not_found_returns_4() {
        assert_eq!(exit_code(&ForgeError::OscalCliNotFound), 4);
    }

    #[test]
    fn exit_code_oscal_cli_not_functional_returns_4() {
        assert_eq!(
            exit_code(&ForgeError::OscalCliNotFunctional {
                path: PathBuf::from("/usr/bin/oscal-cli"),
                detail: "broken".to_string(),
            }),
            4
        );
    }

    #[test]
    fn exit_code_oscal_cli_execution_returns_1() {
        assert_eq!(
            exit_code(&ForgeError::OscalCliExecution {
                exit_code: Some(1),
                message: "err".to_string(),
            }),
            1
        );
    }

    #[test]
    #[allow(clippy::duration_suboptimal_units)]
    fn exit_code_oscal_cli_timeout_returns_1() {
        assert_eq!(
            exit_code(&ForgeError::OscalCliTimeout { timeout: std::time::Duration::from_secs(60) }),
            1
        );
    }

    #[test]
    fn exit_code_resolve_input_not_json_returns_1() {
        assert_eq!(exit_code(&ForgeError::ResolveInputNotJson { path: PathBuf::from("x.xml") }), 1);
    }

    #[test]
    fn diff_has_changes_display() {
        let err = ForgeError::DiffHasChanges;
        assert_eq!(err.to_string(), "");
    }

    #[test]
    fn diff_has_changes_exit_code_is_1() {
        assert_eq!(exit_code(&ForgeError::DiffHasChanges), 1);
    }

    #[test]
    fn diff_error_display() {
        let err = ForgeError::DiffError("type mismatch".to_string());
        assert_eq!(err.to_string(), "Diff error: type mismatch");
    }

    #[test]
    fn diff_error_exit_code_is_2() {
        assert_eq!(exit_code(&ForgeError::DiffError("test".into())), 2);
    }

    #[test]
    fn round_trip_failed_display() {
        let err = ForgeError::RoundTripFailed(3);
        assert_eq!(err.to_string(), "Round-trip validation failed: 3 unresolved divergence(s)");
    }

    #[test]
    fn round_trip_failed_exit_code_is_1() {
        assert_eq!(exit_code(&ForgeError::RoundTripFailed(1)), 1);
    }
}
