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
}
