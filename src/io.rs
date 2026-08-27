//! Shared I/O utilities: atomic writes, size guardrails, path sanitization.

use std::path::{Component, Path, PathBuf};

use crate::error::ForgeError;

/// Maximum file size for all file-reading operations (50 MB).
pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Write content durably and atomically using flush + temp-file rename.
///
/// Once the rename succeeds, `path` contains the new content even if a subsequent
/// file or directory sync returns an error.
///
/// # Errors
///
/// Returns `ForgeError::Io` if the temporary file cannot be created, written, flushed, persisted,
/// or durably recorded by the containing directory on Unix.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), ForgeError> {
    use std::io::Write;

    let parent =
        path.parent().filter(|value| !value.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let temp_error = |operation: &str, error: std::io::Error| {
        ForgeError::Io(std::io::Error::other(format!(
            "Failed to {operation} temporary file for '{}': {error}",
            path.display()
        )))
    };
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| temp_error("create", error))?;
    tmp.write_all(content).map_err(|error| temp_error("write", error))?;
    tmp.as_file().sync_all().map_err(|error| temp_error("sync", error))?;
    #[cfg(unix)]
    if let Ok(existing) = std::fs::metadata(path) {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(existing.permissions().mode());
        // Best effort: a concurrent removal must not prevent an otherwise-safe write.
        let _ = std::fs::set_permissions(tmp.path(), permissions);
    }
    let persisted = tmp.persist(path).map_err(|error| {
        ForgeError::Io(std::io::Error::other(format!(
            "Failed to persist temp file to '{}': {error}",
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
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ForgeError::Io(std::io::Error::other(format!(
            "'{}' must be a regular file, not a symbolic link or special file",
            path.display()
        ))));
    }
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

/// Read a regular, non-symlink file through one bounded file handle.
///
/// The returned bytes are capped at `max_bytes`; a file that grows while being
/// read is rejected after at most one additional byte rather than buffered without
/// limit. Callers that consume content MUST prefer this over `check_file_size`
/// followed by a separate path-based read.
///
/// # Errors
///
/// Returns `ForgeError::FileTooLarge` when the held handle reports or produces
/// more than `max_bytes`, and `ForgeError::Io` for open/read or file-type errors.
pub fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ForgeError> {
    use std::io::Read;

    let path_metadata = std::fs::symlink_metadata(path)?;
    if path_metadata.file_type().is_symlink() || !path_metadata.is_file() {
        return Err(ForgeError::Io(std::io::Error::other(format!(
            "'{}' must be a regular file, not a symbolic link or special file",
            path.display()
        ))));
    }

    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(ForgeError::Io(std::io::Error::other(format!(
            "'{}' must be a regular file",
            path.display()
        ))));
    }
    if metadata.len() > max_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: metadata.len(),
            limit_bytes: max_bytes,
        });
    }

    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: bytes.len() as u64,
            limit_bytes: max_bytes,
        });
    }
    Ok(bytes)
}

/// Return metadata only when `path` names a regular file directly, never through a symlink.
pub(crate) fn regular_file_metadata(path: &Path, label: &str) -> Result<std::fs::Metadata, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|cause| format!("{label} '{}': {cause}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{label} must not be a symbolic link"));
    }
    if !metadata.is_file() {
        return Err(format!("{label} must be a regular file"));
    }
    Ok(metadata)
}

/// Express a local resource relative to the manifest output directory when possible.
pub(crate) fn manifest_relative_path(
    path: &Path,
    output: Option<&Path>,
    resource_kind: &str,
) -> Result<PathBuf, ForgeError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|cause| {
        ForgeError::Io(std::io::Error::new(
            cause.kind(),
            format!("cannot resolve {resource_kind} '{}': {cause}", path.display()),
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ForgeError::InvalidArgument(format!(
            "{resource_kind} '{}' must be a regular non-symlink file",
            path.display()
        )));
    }
    let target = path.canonicalize().map_err(|cause| {
        ForgeError::Io(std::io::Error::new(
            cause.kind(),
            format!("cannot resolve {resource_kind} '{}': {cause}", path.display()),
        ))
    })?;
    let manifest_dir_path = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let manifest_dir = manifest_dir_path.canonicalize().map_err(|cause| {
        ForgeError::Io(std::io::Error::new(
            cause.kind(),
            format!("cannot resolve manifest directory '{}': {cause}", manifest_dir_path.display()),
        ))
    })?;
    relative_path(&manifest_dir, &target).ok_or_else(|| {
        ForgeError::InvalidArgument(format!(
            "cannot express {resource_kind} '{}' relative to manifest directory '{}'",
            path.display(),
            manifest_dir_path.display()
        ))
    })
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
    path.components()
        .rev()
        .find_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .unwrap_or_else(|| "artifact".to_string())
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
    fn read_bounded_rejects_content_beyond_the_held_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.json");
        std::fs::write(&path, b"12345").unwrap();

        assert!(matches!(
            read_bounded(&path, 4),
            Err(ForgeError::FileTooLarge { limit_bytes: 4, .. })
        ));
    }

    #[test]
    fn regular_file_metadata_rejects_directories() {
        let dir = tempfile::tempdir().unwrap();
        let error = regular_file_metadata(dir.path(), "input").unwrap_err();
        assert_eq!(error, "input must be a regular file");
    }

    #[cfg(unix)]
    #[test]
    fn regular_file_metadata_rejects_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.json");
        let link = dir.path().join("link.json");
        std::fs::write(&target, "{}").unwrap();
        std::os::unix::fs::symlink(target, &link).unwrap();
        let error = regular_file_metadata(&link, "input").unwrap_err();
        assert_eq!(error, "input must not be a symbolic link");
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

    #[test]
    fn sanitize_componentless_paths_never_echoes_absolute_input() {
        assert_eq!(sanitize_artifact_path(Path::new("/")), "artifact");
        assert_eq!(sanitize_artifact_path(Path::new("sub/..")), "sub");
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_existing_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        write_atomic(&path, b"new").unwrap();

        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn check_file_size_rejects_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.json");
        let link = dir.path().join("link.json");
        std::fs::write(&target, "{}").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(check_file_size(&link, 1024).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn manifest_relative_path_rejects_resource_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target.json");
        let link = dir.path().join("link.json");
        let output = dir.path().join("manifest.json");
        std::fs::write(&target, "{}").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let error = manifest_relative_path(&link, Some(&output), "resource").unwrap_err();
        assert!(
            matches!(error, ForgeError::InvalidArgument(message) if message.contains("non-symlink"))
        );
    }

    #[test]
    fn relative_path_refuses_targets_without_a_common_prefix() {
        assert_eq!(relative_path(Path::new("/manifest"), Path::new("resource.json")), None);
    }
}
