//! Traceability: map OSCAL output fields back to source Markdown lines.
//!
//! Produces [`TraceLink`](crate::model::trace::TraceLink) entries for every generated
//! OSCAL control, parameter, and citation, enabling bidirectional drill-down
//! from compliance artifact to policy source.

/// Extract trace links from a parsed policy document.
pub mod extractor;
/// Format trace reports for human-readable output.
pub mod formatter;
/// Trace report types: structured trace output for a conversion run.
pub mod report;
/// Resolve trace links against source files.
pub mod resolver;
/// Walk the OSCAL output tree and collect trace references.
pub mod walker;

use std::fs::{File, Metadata};
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

use crate::error::ForgeError;
use crate::types::OscalModelType;
use report::TraceReport;
use resolver::Staleness;

/// Generate a complete traceability report from an OSCAL artifact and source policy.
///
/// 1. Reads and parses artifact JSON
/// 2. Detects artifact type
/// 3. Walks elements and extracts trace metadata
/// 4. Computes summary statistics
/// 5. Checks source staleness
///
/// # Errors
///
/// - `ForgeError::FileNotFound` if artifact or source file doesn't exist.
/// - `ForgeError::FileTooLarge` if artifact or source exceeds `MAX_FILE_SIZE`.
/// - `ForgeError::PermissionDenied` if either file cannot be read.
/// - `ForgeError::Parse` if artifact is invalid JSON.
/// - `ForgeError::TraceUnsupportedArtifact` if the artifact is unrecognized, unsupported
///   (Profile or Mapping), ambiguous, or does not have an object root.
pub fn generate_trace_report(
    artifact_path: &Path,
    source_path: &Path,
) -> Result<TraceReport, ForgeError> {
    // Missing files surface as ForgeError::FileNotFound from the actual read below.
    let artifact_content = read_file(artifact_path)?;
    let json: serde_json::Value = serde_json::from_str(&artifact_content)
        .map_err(|error| ForgeError::Parse(format!("Invalid JSON in artifact: {error}")))?;

    // Read the source line count and mtime from the same opened-file snapshot.
    let (source_line_count, source_modified) = read_source_file(source_path)?;

    let art_type = walker::detect_artifact_type(&json)?;
    let entries = match art_type {
        OscalModelType::Catalog => {
            let catalog = json.get("catalog").ok_or_else(|| {
                ForgeError::TraceUnsupportedArtifact { detail: "missing 'catalog' key".to_string() }
            })?;
            walker::walk_catalog_elements(catalog)
        }
        OscalModelType::ComponentDefinition => {
            let compdef = json.get("component-definition").ok_or_else(|| {
                ForgeError::TraceUnsupportedArtifact {
                    detail: "missing 'component-definition' key".to_string(),
                }
            })?;
            walker::walk_compdef_elements(compdef)
        }
        OscalModelType::Profile => {
            return Err(ForgeError::TraceUnsupportedArtifact {
                detail: "Profile artifacts are not supported for traceability".to_string(),
            });
        }
        OscalModelType::SystemSecurityPlan => {
            return Err(ForgeError::TraceUnsupportedArtifact {
                detail: "System Security Plan artifacts are not supported for source traceability"
                    .to_string(),
            });
        }
        OscalModelType::Mapping => {
            return Err(ForgeError::TraceUnsupportedArtifact {
                detail: "Control Mapping artifacts are not supported for traceability".to_string(),
            });
        }
    };

    let metadata_last_modified = json
        .get(art_type.as_str())
        .and_then(|value| value.get("metadata"))
        .and_then(|metadata| metadata.get("last-modified"))
        .and_then(serde_json::Value::as_str);
    let source_stale = matches!(
        resolver::check_source_staleness(source_modified, metadata_last_modified),
        Staleness::Stale
    );

    Ok(TraceReport {
        artifact_path: artifact_path.to_path_buf(),
        source_path: source_path.to_path_buf(),
        artifact_type: art_type,
        entries,
        source_stale,
        source_line_count,
    })
}

/// Read a file with consistent path-aware I/O errors and a bounded buffer.
fn read_file(path: &Path) -> Result<String, ForgeError> {
    let (file, _) = open_file(path)?;
    read_open_file(path, file)
}

/// Count source lines with the mtime observed from the same open file handle.
fn read_source_file(path: &Path) -> Result<(usize, Option<SystemTime>), ForgeError> {
    let (file, metadata) = open_file(path)?;
    let source_modified = metadata.modified().ok();
    let mut reader = std::io::BufReader::new(file.take(crate::io::MAX_FILE_SIZE + 1));
    let mut line = String::new();
    let mut source_line_count = 0;
    let mut bytes_read = 0_u64;

    loop {
        line.clear();
        let read = std::io::BufRead::read_line(&mut reader, &mut line)
            .map_err(|error| map_io_error(path, error))?;
        if read == 0 {
            break;
        }
        bytes_read += u64::try_from(read)
            .map_err(|_| ForgeError::Io(std::io::Error::other("line read count exceeds u64")))?;
        if bytes_read > crate::io::MAX_FILE_SIZE {
            return Err(ForgeError::FileTooLarge {
                path: path.to_path_buf(),
                size_bytes: bytes_read,
                limit_bytes: crate::io::MAX_FILE_SIZE,
            });
        }
        source_line_count += 1;
    }

    Ok((source_line_count, source_modified))
}

/// Open a regular trace input and reject known oversized files before reading.
fn open_file(path: &Path) -> Result<(File, Metadata), ForgeError> {
    let file = File::open(path).map_err(|error| map_io_error(path, error))?;
    let metadata = file.metadata().map_err(|error| map_io_error(path, error))?;
    if metadata.len() > crate::io::MAX_FILE_SIZE {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            limit_bytes: crate::io::MAX_FILE_SIZE,
        });
    }
    Ok((file, metadata))
}

/// Read an opened file without ever buffering more than the configured limit.
fn read_open_file(path: &Path, file: File) -> Result<String, ForgeError> {
    read_open_file_bounded(path, file, crate::io::MAX_FILE_SIZE)
}

fn read_open_file_bounded(path: &Path, file: File, max_bytes: u64) -> Result<String, ForgeError> {
    let mut content = String::new();
    let bytes_read = u64::try_from(
        file.take(max_bytes + 1)
            .read_to_string(&mut content)
            .map_err(|error| map_io_error(path, error))?,
    )
    .map_err(|_| ForgeError::Io(std::io::Error::other("bounded read count exceeds u64")))?;
    if bytes_read > max_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: bytes_read,
            limit_bytes: max_bytes,
        });
    }
    Ok(content)
}

/// Normalize I/O errors consistently across metadata and content reads.
fn map_io_error(path: &Path, error: std::io::Error) -> ForgeError {
    match error.kind() {
        std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
        std::io::ErrorKind::PermissionDenied => {
            ForgeError::PermissionDenied { path: path.to_path_buf() }
        }
        _ => ForgeError::Io(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind, Write};

    #[test]
    fn permission_errors_are_normalized_before_or_during_reads() {
        let path = Path::new("protected.md");
        assert!(matches!(
            map_io_error(path, Error::from(ErrorKind::PermissionDenied)),
            ForgeError::PermissionDenied { path: error_path } if error_path == path
        ));
    }

    #[test]
    fn bounded_reader_rejects_content_over_its_limit() {
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        temp.write_all(b"abc").unwrap();
        let file = File::open(temp.path()).unwrap();

        match read_open_file_bounded(temp.path(), file, 2) {
            Err(ForgeError::FileTooLarge { size_bytes, limit_bytes, .. }) => {
                assert_eq!(size_bytes, 3);
                assert_eq!(limit_bytes, 2);
            }
            result => panic!("expected bounded read to reject content, got {result:?}"),
        }
    }

    #[test]
    fn source_read_returns_line_count_and_same_handle_mtime_snapshot() {
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        temp.write_all(b"one\ntwo\n").unwrap();

        let (line_count, modified) = read_source_file(temp.path()).unwrap();
        assert_eq!(line_count, 2);
        assert!(modified.is_some());
    }
}
