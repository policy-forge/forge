//! Shared I/O utilities: atomic writes, size guardrails, path sanitization.

use std::path::{Component, Path, PathBuf};

use crate::error::ForgeError;

/// Maximum file size for all file-reading operations (50 MB).
pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Write content durably and atomically using flush + temp-file rename.
///
/// # Errors
///
/// Returns `ForgeError::Io` if the temporary file cannot be created, written, flushed, persisted,
/// or durably recorded by the containing directory on Unix.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), ForgeError> {
    use std::io::Write;

    let parent =
        path.parent().filter(|value| !value.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.as_file().sync_all()?;
    let persisted = tmp.persist(path).map_err(|e| {
        ForgeError::Io(std::io::Error::other(format!(
            "Failed to persist temp file to '{}': {e}",
            path.display()
        )))
    })?;
    persisted.sync_all()?;
    #[cfg(unix)]
    std::fs::File::open(parent)?.sync_all()?;
    Ok(())
}

/// Check that a file does not exceed `max_bytes` before reading.
///
/// # Errors
///
/// Returns `ForgeError::FileTooLarge` if the file exceeds `max_bytes`, or
/// `ForgeError::Io` if file metadata cannot be read.
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

/// Express a local resource relative to the manifest output directory when possible.
pub(crate) fn manifest_relative_path(
    path: &Path,
    output: Option<&Path>,
    resource_kind: &str,
) -> Result<PathBuf, String> {
    let target = path
        .canonicalize()
        .map_err(|cause| format!("cannot resolve {resource_kind} '{}': {cause}", path.display()))?;
    let manifest_dir_path = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !manifest_dir_path.is_dir() {
        return Err(format!("output directory '{}' does not exist", manifest_dir_path.display()));
    }
    let manifest_dir = manifest_dir_path
        .canonicalize()
        .map_err(|cause| format!("cannot resolve manifest directory: {cause}"))?;
    Ok(relative_path(&manifest_dir, &target).unwrap_or(target))
}

fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let common = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return None;
    }
    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &target_components[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

/// Extract filename from a path to prevent absolute path leaks in OSCAL artifacts.
#[must_use]
pub fn sanitize_artifact_path(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.to_string_lossy().into_owned(), |n| n.to_string_lossy().into_owned())
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
        let dir = tempfile::tempdir().unwrap();
        let missing_child = dir.path().join("nonexistent_subdir").join("out.json");
        assert!(write_atomic(&missing_child, b"data").is_err());
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
