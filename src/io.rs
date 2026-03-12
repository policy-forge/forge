//! Shared I/O utilities: atomic writes, size guardrails, path sanitization.

use std::path::Path;

use crate::error::ForgeError;

/// Maximum file size for all file-reading operations (50 MB).
pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Write content to a file atomically using temp-file + rename.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), ForgeError> {
    use std::io::Write;

    let parent = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.persist(path).map_err(|e| {
        ForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist temp file to '{}': {e}", path.display()),
        ))
    })?;
    Ok(())
}

/// Check that a file does not exceed `max_bytes` before reading.
pub fn check_file_size(path: &Path, max_bytes: u64) -> Result<u64, ForgeError> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();
    if size > max_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: size,
            limit_bytes: max_bytes,
        });
    }
    Ok(size)
}

/// Extract filename from a path to prevent absolute path leaks in OSCAL artifacts.
#[must_use]
pub fn sanitize_artifact_path(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_file_with_correct_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        write_atomic(&path, b"hello world").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn write_atomic_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        std::fs::write(&path, "old").unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    fn write_atomic_fails_on_nonexistent_parent() {
        let path = Path::new("/nonexistent_forge_test_dir/out.json");
        assert!(write_atomic(path, b"data").is_err());
    }

    #[test]
    fn check_file_size_accepts_small_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("small.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(check_file_size(&path, 1024).is_ok());
    }

    #[test]
    fn check_file_size_rejects_oversized_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.json");
        std::fs::write(&path, vec![b'x'; 100]).unwrap();
        let result = check_file_size(&path, 50);
        assert!(result.is_err());
    }

    #[test]
    fn sanitize_extracts_filename_from_absolute_path() {
        let path = Path::new("/home/user/docs/catalog.json");
        assert_eq!(sanitize_artifact_path(path), "catalog.json");
    }

    #[test]
    fn sanitize_preserves_bare_filename() {
        let path = Path::new("catalog.json");
        assert_eq!(sanitize_artifact_path(path), "catalog.json");
    }

    #[test]
    fn sanitize_handles_relative_path() {
        let path = Path::new("../docs/catalog.json");
        assert_eq!(sanitize_artifact_path(path), "catalog.json");
    }
}
