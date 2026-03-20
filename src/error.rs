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

#[derive(Debug, Error)]
pub enum ForgeError {
    // --- Input/IO errors (exit code 1) ---
    #[error("File not found: '{}'", path.display())]
    FileNotFound { path: PathBuf },

    #[error("Permission denied: '{}'", path.display())]
    PermissionDenied { path: PathBuf },

    #[error(
        "File is empty: '{}' — provide a non-empty Markdown policy document",
        path.display()
    )]
    EmptyInput { path: PathBuf },

    #[error(
        "File appears to be binary, not a text document: '{}'. \
         FORGE accepts UTF-8 Markdown (.md) files.",
        path.display()
    )]
    BinaryFile { path: PathBuf },

    #[error(
        "Unsupported file format '.{extension}'. Only Markdown files (.md, .markdown) are supported. \
         Consider converting with pandoc or markitdown."
    )]
    UnsupportedFormat { extension: String },

    #[error(
        "File '{}' is {}, exceeding the {} limit. Use --max-size to increase the limit.",
        path.display(),
        format_size(.size_bytes),
        format_size(.limit_bytes)
    )]
    FileTooLarge { path: PathBuf, size_bytes: u64, limit_bytes: u64 },

    #[error(
        "File '{}' is not valid UTF-8 text. FORGE requires UTF-8 encoded Markdown files.",
        path.display()
    )]
    InvalidEncoding { path: PathBuf },

    #[error("'{}' is not a regular file.", path.display())]
    NotAFile { path: PathBuf },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // --- Parse/Structure errors (exit code 2) ---
    #[error(
        "No policy structure detected in '{}' — expected Markdown headings (# Section) or numbered clauses",
        path.display()
    )]
    NoStructureDetected { path: PathBuf },

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Catalog build error: {0}")]
    CatalogBuild(String),

    #[error("Back matter error: {0}")]
    BackMatter(String),

    #[error("Component definition build error: {0}")]
    ComponentDefinitionBuild(String),

    #[error("Assessment plan build error: {0}")]
    AssessmentPlanBuild(String),

    #[error("Parameter extraction error: {0}")]
    ParameterExtraction(String),

    #[error("Unsupported OSCAL artifact type for tracing: {detail}")]
    TraceUnsupportedArtifact { detail: String },

    #[error(
        "Ambiguous OSCAL artifact: file contains multiple model types ({detail}). Each file must contain exactly one OSCAL model."
    )]
    AmbiguousArtifact { detail: String },

    // --- Validation/Config errors (exit code 3) ---
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // --- Export errors (exit code 1) ---
    #[error(
        "Unrecognized file extension '.{extension}' on input file. \
         Expected .json, .xml, .yaml, or .yml for OSCAL artifacts."
    )]
    ExportUnsupportedExtension { extension: String },

    #[error(
        "No file extension on input file '{}'. \
         Cannot determine OSCAL format. Expected .json, .xml, .yaml, or .yml.",
        path.display()
    )]
    ExportNoExtension { path: PathBuf },

    #[error("Input is not a valid OSCAL artifact: {detail}")]
    ExportInvalidOscal { detail: String },

    #[error("Export input file is empty: '{}'", path.display())]
    ExportEmptyInput { path: PathBuf },

    // --- Argument errors (exit code 1) ---
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    // --- Batch errors (exit code 1) ---
    #[error("Batch conversion error: {0}")]
    BatchConversion(String),

    // --- External dependency errors (exit code 4) ---
    #[error(
        "oscal-cli not found on system PATH. Install from: https://github.com/usnistgov/oscal-cli"
    )]
    OscalCliNotFound,

    #[error("oscal-cli found at '{}' but is not functional: {detail}", path.display())]
    OscalCliNotFunctional { path: PathBuf, detail: String },

    // --- oscal-cli execution errors (exit code 1) ---
    #[error("oscal-cli execution failed (exit code {exit_code:?}): {message}")]
    OscalCliExecution { exit_code: Option<i32>, message: String },

    #[error("oscal-cli execution timed out after {timeout:?}")]
    OscalCliTimeout { timeout: std::time::Duration },

    #[error(
        "Expected JSON input file, got: '{}'. Only .json files are supported for profile resolution.",
        path.display()
    )]
    ResolveInputNotJson { path: PathBuf },

    // --- Diff errors (exit code 2 for DiffError, exit code 1 for DiffHasChanges) ---
    #[error("")]
    DiffHasChanges,

    #[error("Round-trip validation failed: {0} unresolved divergence(s)")]
    RoundTripFailed(usize),

    #[error("Diff error: {0}")]
    DiffError(String),

    // --- Other (exit code 1) ---
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
}

/// Map a `ForgeError` to a CLI exit code.
///
/// Exit code categories:
/// - 0: Success (not handled here — only error cases)
/// - 1: Input/IO errors (file not found, permission denied, empty, binary, encoding, size, I/O)
/// - 2: Parse/Structure errors (no structure, parse failure, build errors)
/// - 3: Validation/Config errors (schema violations, config issues)
/// - 4: External dependency unavailable (oscal-cli not found or not functional)
#[must_use]
pub fn exit_code(err: &ForgeError) -> u8 {
    match err {
        // Exit 1: Input/IO errors
        ForgeError::FileNotFound { .. }
        | ForgeError::PermissionDenied { .. }
        | ForgeError::EmptyInput { .. }
        | ForgeError::BinaryFile { .. }
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

        // Exit 2: Parse/Structure errors + Diff errors
        ForgeError::DiffError(_)
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
