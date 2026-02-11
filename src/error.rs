use std::path::PathBuf;

use thiserror::Error;

#[allow(clippy::cast_precision_loss)] // Precision loss is acceptable for human-readable file sizes
/// Format a byte count into a compact human-readable string using KB or MB with one decimal place.
///
/// Values less than 1,048,576 bytes are rendered in kilobytes (KB); values greater than or
/// equal to 1,048,576 bytes are rendered in megabytes (MB). The output includes one digit
/// after the decimal point (e.g., `15.0MB`, `512.0KB`).
///
/// # Examples
///
/// ```
/// assert_eq!(format_size(15_728_640), "15.0MB"); // 15 * 1_048_576
/// assert_eq!(format_size(524_288), "512.0KB");  // 512 * 1024
/// ```
fn format_size(bytes: u64) -> String {
    if bytes < 1_048_576 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    }
}

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

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
    FileTooLarge { path: PathBuf, size_bytes: u64, limit_bytes: u64 },

    #[error(
        "File '{}' is not valid UTF-8 text. FORGE requires UTF-8 encoded Markdown files.",
        path.display()
    )]
    InvalidEncoding { path: PathBuf },

    #[error("'{}' is not a regular file.", path.display())]
    NotAFile { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn io_error_from_conversion() {
        /// Produces a `ForgeError::Io` representing a permission-denied I/O error.
        ///
        /// The returned error wraps an `std::io::Error` with kind `PermissionDenied` and message
        /// `"access denied"`.
        ///
        /// # Examples
        ///
        /// ```
        /// let err = produce_io_error().unwrap_err();
        /// assert_eq!(err.to_string(), "I/O error: access denied");
        /// ```
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
}