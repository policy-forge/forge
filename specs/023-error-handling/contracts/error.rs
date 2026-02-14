// Contract: ForgeError enum (WI-23 target state)
// This file defines the target API contract. Implementation must match.

use std::path::PathBuf;

use thiserror::Error;

#[allow(clippy::cast_precision_loss)]
fn format_size(bytes: u64) -> String {
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
        format_size(*.size_bytes),
        format_size(*.limit_bytes)
    )]
    FileTooLarge {
        path: PathBuf,
        size_bytes: u64,
        limit_bytes: u64,
    },

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

    // --- Validation/Config errors (exit code 3) ---

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // --- Other (exit code 1) ---

    #[error("Serialization error: {0}")]
    Serialization(String),
}
