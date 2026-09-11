//! Publish one complete generation without replacing an existing directory.
//!
//! All files are written and synced in a private sibling staging directory.
//! The only operation exposing the destination is one atomic no-replace rename.
//! A process interruption can leave private staging data, but cannot expose a
//! partial destination. A sync failure after the rename reports an error with a
//! complete destination already present. Filesystem durability after a power
//! failure remains subject to the filesystem's sync guarantees.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::ForgeError;

use super::error;

pub const MAX_OUTPUT_BYTES: usize = 50 * 1024 * 1024;
const MAX_ARTIFACTS: usize = 2_048;

/// One fully rendered artifact; the label is relative to the new generation.
#[derive(Debug, Clone)]
pub struct OutputArtifact {
    pub relative_path: String,
    pub bytes: Vec<u8>,
}

/// Publish a generation below an existing project root and existing parent.
///
/// Existing destinations (including empty directories) are always rejected.
/// Linux and macOS provide the required no-replace directory operation. Other
/// platforms fail before staging rather than degrade to sequential file writes.
///
/// # Errors
///
/// Returns an authoring error for unsafe paths, collisions, bounds, unsupported
/// platforms, or filesystem failures. A post-rename durability error can occur
/// after the complete generation becomes visible; callers must inspect the
/// destination before retrying, and must never replace it automatically.
pub fn publish(
    root: &Path,
    relative_output_dir: &Path,
    artifacts: &[OutputArtifact],
) -> Result<(), ForgeError> {
    validate_artifacts(artifacts)?;
    let destination =
        relative_output_dir.to_str().ok_or_else(|| error("output directory must be UTF-8"))?;
    validate_relative(destination)?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        unix::publish_with_hook(root, destination, artifacts, |_| Ok(()))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = root;
        Err(error(
            "atomic no-replace authoring directory publication is unsupported on this platform",
        ))
    }
}

/// Publication events exposed to tests. Not part of the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilePublishEvent {
    /// The private staging directory exists and the staged file is durably written.
    Staged,
    /// The no-replace rename is about to expose the destination.
    BeforeRename,
}

/// Publish one new file below an existing root without replacing an existing path.
///
/// The destination must be a portable relative descendant of `root`. The file is
/// staged inside a private mode-0700 directory below the destination parent, and
/// the rename source is resolved against that directory's held descriptor, so a
/// concurrent writer that replaces the staging pathname cannot substitute the
/// published object. The only operation exposing the destination is one no-replace
/// rename, so an existing file or symlink is never replaced. Other platforms fail
/// closed, matching directory generation publication.
///
/// # Errors
///
/// Returns an authoring error for unsafe paths, an existing destination, bounds,
/// unsupported platforms, or filesystem failures; a partial destination is never
/// exposed.
pub fn publish_new_file(root: &Path, relative_path: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    publish_new_file_with_hook(root, relative_path, bytes, |_| Ok(()))
}

fn publish_new_file_with_hook(
    root: &Path,
    relative_path: &Path,
    bytes: &[u8],
    hook: impl FnMut(FilePublishEvent) -> Result<(), ForgeError>,
) -> Result<(), ForgeError> {
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(error("output file exceeds the 50 MiB output limit"));
    }
    let destination =
        relative_path.to_str().ok_or_else(|| error("output file path must be UTF-8"))?;
    validate_relative(destination)?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        unix::write_new_file_with_hook(root, destination, bytes, hook)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (root, bytes, hook);
        Err(error("atomic no-replace authoring file publication is unsupported on this platform"))
    }
}

fn validate_artifacts(artifacts: &[OutputArtifact]) -> Result<(), ForgeError> {
    if artifacts.is_empty() || artifacts.len() > MAX_ARTIFACTS {
        return Err(error("generation artifact count is outside the supported bound"));
    }
    let mut total = 0usize;
    let mut files = BTreeSet::new();
    let mut directories = BTreeMap::new();
    for artifact in artifacts {
        validate_relative(&artifact.relative_path)?;
        total = total
            .checked_add(artifact.bytes.len())
            .ok_or_else(|| error("generation byte count overflowed"))?;
        if total > MAX_OUTPUT_BYTES {
            return Err(error("generation exceeds the 50 MiB output limit"));
        }
        let folded = artifact.relative_path.to_ascii_lowercase();
        if !files.insert(folded.clone()) || directories.contains_key(&folded) {
            return Err(error("generation artifact paths alias or collide"));
        }
        let mut prefix = String::new();
        let mut original_prefix = String::new();
        let components = folded.split('/').collect::<Vec<_>>();
        for (component, original_component) in
            components[..components.len() - 1].iter().zip(artifact.relative_path.split('/'))
        {
            if !prefix.is_empty() {
                prefix.push('/');
                original_prefix.push('/');
            }
            prefix.push_str(component);
            original_prefix.push_str(original_component);
            if files.contains(&prefix) {
                return Err(error("generation artifact file aliases a directory"));
            }
            if directories
                .insert(prefix.clone(), original_prefix.clone())
                .is_some_and(|previous| previous != original_prefix)
            {
                return Err(error("generation directory paths have case aliases"));
            }
        }
    }
    Ok(())
}

fn validate_relative(path: &str) -> Result<(), ForgeError> {
    if path.is_empty() || path.len() > 1024 || path.split('/').count() > 32 {
        return Err(error("output path is empty or exceeds its bounds"));
    }
    for component in path.split('/') {
        let stem = component.split('.').next().unwrap_or_default().to_ascii_lowercase();
        let reserved = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
            || stem.strip_prefix("com").is_some_and(is_device_digit)
            || stem.strip_prefix("lpt").is_some_and(is_device_digit);
        if component.is_empty()
            || component.len() > 128
            || component == "."
            || component == ".."
            || component.ends_with('.')
            || reserved
            || !component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(error("output paths require portable descendant components"));
        }
    }
    Ok(())
}

fn is_device_digit(value: &str) -> bool {
    value.len() == 1 && matches!(value.as_bytes()[0], b'1'..=b'9')
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(unsafe_code)] // Descriptor-relative operations prevent intermediate symlink substitution.
mod unix {
    use std::collections::BTreeMap;
    use std::ffi::{CStr, CString, OsStr};
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Component, Path};

    use crate::ForgeError;

    use super::{OutputArtifact, error};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum PublishEvent {
        Staged,
        FileWritten(usize),
        BeforeRename,
        Published,
    }

    struct StagingDirectory {
        parent: File,
        name: CString,
        directories: BTreeMap<String, File>,
        files: Vec<(String, CString)>,
        committed: bool,
    }

    impl StagingDirectory {
        fn create(parent: File) -> Result<Self, ForgeError> {
            let name =
                c_string(OsStr::new(&format!(".forge-authoring-stage-{}", uuid::Uuid::new_v4())))?;
            make_directory(&parent, &name)?;
            let directory = match open_at(&parent, &name, true, false) {
                Ok(directory) => directory,
                Err(cause) => {
                    unlink(&parent, &name, true);
                    return Err(cause);
                }
            };
            Ok(Self {
                parent,
                name,
                directories: BTreeMap::from([(String::new(), directory)]),
                files: Vec::new(),
                committed: false,
            })
        }

        fn write(&mut self, artifact: &OutputArtifact) -> Result<(), ForgeError> {
            let components = artifact.relative_path.split('/').collect::<Vec<_>>();
            let mut parent = String::new();
            for component in &components[..components.len() - 1] {
                let next = if parent.is_empty() {
                    (*component).to_string()
                } else {
                    format!("{parent}/{component}")
                };
                if !self.directories.contains_key(&next) {
                    let directory = self
                        .directories
                        .get(&parent)
                        .ok_or_else(|| error("staging directory ancestry is incomplete"))?;
                    let name = c_string(OsStr::new(component))?;
                    make_directory(directory, &name)?;
                    let child = match open_at(directory, &name, true, false) {
                        Ok(child) => child,
                        Err(cause) => {
                            unlink(directory, &name, true);
                            return Err(cause);
                        }
                    };
                    self.directories.insert(next.clone(), child);
                }
                parent = next;
            }
            let directory = self
                .directories
                .get(&parent)
                .ok_or_else(|| error("staging artifact parent is missing"))?;
            let name = c_string(OsStr::new(
                components.last().ok_or_else(|| error("staging artifact name is missing"))?,
            ))?;
            let mut file = open_at(directory, &name, false, true)?;
            self.files.push((parent, name));
            file.write_all(&artifact.bytes).map_err(io_error)?;
            file.sync_all().map_err(io_error)
        }

        fn sync(&self) -> Result<(), ForgeError> {
            for directory in self.directories.values().rev() {
                directory.sync_all().map_err(io_error)?;
            }
            self.parent.sync_all().map_err(io_error)
        }
    }

    impl Drop for StagingDirectory {
        fn drop(&mut self) {
            if self.committed {
                return;
            }
            for (parent, name) in &self.files {
                if let Some(directory) = self.directories.get(parent) {
                    unlink(directory, name, false);
                }
            }
            for path in self.directories.keys().rev().filter(|path| !path.is_empty()) {
                let (parent, name) = path.rsplit_once('/').unwrap_or(("", path));
                if let (Some(directory), Ok(name)) =
                    (self.directories.get(parent), CString::new(name))
                {
                    unlink(directory, &name, true);
                }
            }
            unlink(&self.parent, &self.name, true);
        }
    }

    pub(super) fn publish_with_hook(
        root: &Path,
        destination: &str,
        artifacts: &[OutputArtifact],
        mut hook: impl FnMut(PublishEvent) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let mut parent = open_root(root)?;
        let components = destination.split('/').collect::<Vec<_>>();
        for component in &components[..components.len() - 1] {
            parent = open_at(&parent, &c_string(OsStr::new(component))?, true, false)?;
        }
        let name = c_string(OsStr::new(
            components.last().ok_or_else(|| error("output directory name is missing"))?,
        ))?;
        reject_existing(
            &parent,
            &name,
            "output directory already exists; use a new generation directory",
        )?;
        let mut staging = StagingDirectory::create(parent)?;
        hook(PublishEvent::Staged)?;
        for (index, artifact) in artifacts.iter().enumerate() {
            staging.write(artifact)?;
            hook(PublishEvent::FileWritten(index))?;
        }
        staging.sync()?;
        hook(PublishEvent::BeforeRename)?;
        rename_no_replace(&staging.parent, &staging.name, &staging.parent, &name, "generation")?;
        staging.committed = true;
        hook(PublishEvent::Published)?;
        staging.parent.sync_all().map_err(|cause| {
            error(format!(
                "complete generation was published, but parent durability sync failed: {cause}"
            ))
        })
    }

    /// Create one new file through a private staging directory and a no-replace rename.
    ///
    /// The rename source is resolved against the held staging descriptor, never by
    /// resolving the staging pathname in the destination parent, so a concurrent
    /// writer cannot substitute the published object.
    pub(super) fn write_new_file_with_hook(
        root: &Path,
        destination: &str,
        bytes: &[u8],
        mut hook: impl FnMut(super::FilePublishEvent) -> Result<(), ForgeError>,
    ) -> Result<(), ForgeError> {
        let mut parent = open_root(root)?;
        let components = destination.split('/').collect::<Vec<_>>();
        for component in &components[..components.len() - 1] {
            parent = open_at(&parent, &c_string(OsStr::new(component))?, true, false)?;
        }
        let name = c_string(OsStr::new(
            components.last().ok_or_else(|| error("output file name is missing"))?,
        ))?;
        reject_existing(&parent, &name, "output file already exists; refusing to replace it")?;
        let staging_label = format!(".forge-authoring-file-{}", uuid::Uuid::new_v4());
        let staging_name = c_string(OsStr::new(&staging_label))?;
        make_directory(&parent, &staging_name)?;
        let staging = match open_at(&parent, &staging_name, true, false) {
            Ok(directory) => directory,
            Err(cause) => {
                unlink(&parent, &staging_name, true);
                return Err(cause);
            }
        };
        let staged = c_string(OsStr::new("file"))?;
        let outcome = (|| -> Result<(), ForgeError> {
            let mut file = open_at(&staging, &staged, false, true)?;
            file.write_all(bytes).map_err(io_error)?;
            file.sync_all().map_err(io_error)?;
            drop(file);
            hook(super::FilePublishEvent::Staged)?;
            staging.sync_all().map_err(io_error)?;
            hook(super::FilePublishEvent::BeforeRename)?;
            rename_no_replace(&staging, &staged, &parent, &name, "file")?;
            parent.sync_all().map_err(io_error)
        })();
        // Best-effort cleanup: the staged file is gone after a successful rename.
        unlink(&staging, &staged, false);
        unlink(&parent, &staging_name, true);
        outcome
    }

    fn open_root(root: &Path) -> Result<File, ForgeError> {
        if !root.is_absolute() {
            return Err(error("publication root must be an absolute resolved directory"));
        }
        let mut directory = File::open("/").map_err(io_error)?;
        for component in root.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    directory = open_at(&directory, &c_string(name)?, true, false)?;
                }
                _ => return Err(error("publication root has an unsafe component")),
            }
        }
        Ok(directory)
    }

    fn c_string(value: &OsStr) -> Result<CString, ForgeError> {
        CString::new(value.as_bytes()).map_err(|_| error("filesystem name contains NUL"))
    }

    fn make_directory(parent: &File, name: &CStr) -> Result<(), ForgeError> {
        // SAFETY: the live directory descriptor and NUL-terminated name remain valid.
        if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
            return Err(io_error(std::io::Error::last_os_error()));
        }
        Ok(())
    }

    fn open_at(
        parent: &File,
        name: &CStr,
        directory: bool,
        create: bool,
    ) -> Result<File, ForgeError> {
        let flags = libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | if directory {
                libc::O_RDONLY | libc::O_DIRECTORY
            } else {
                libc::O_WRONLY | libc::O_NONBLOCK
            }
            | if create { libc::O_CREAT | libc::O_EXCL } else { 0 };
        // SAFETY: parent is live, name is terminated, and mode is supplied for O_CREAT.
        let descriptor = unsafe {
            libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0o600 as libc::c_uint)
        };
        if descriptor < 0 {
            return Err(io_error(std::io::Error::last_os_error()));
        }
        // SAFETY: successful openat yields a fresh descriptor transferred exactly once.
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }

    fn reject_existing(parent: &File, name: &CStr, message: &str) -> Result<(), ForgeError> {
        let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: stat has writable storage, and the name and parent are valid.
        let result = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Err(error(message));
        }
        let cause = std::io::Error::last_os_error();
        if cause.kind() != std::io::ErrorKind::NotFound {
            return Err(io_error(cause));
        }
        Ok(())
    }

    fn rename_no_replace(
        source_dir: &File,
        source: &CStr,
        target_dir: &File,
        target: &CStr,
        label: &str,
    ) -> Result<(), ForgeError> {
        // SAFETY: both names and both held descriptors remain live for this call.
        #[cfg(target_os = "linux")]
        let result = unsafe {
            libc::renameat2(
                source_dir.as_raw_fd(),
                source.as_ptr(),
                target_dir.as_raw_fd(),
                target.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        // SAFETY: same descriptor/name guarantees; RENAME_EXCL forbids replacement.
        #[cfg(target_os = "macos")]
        let result = unsafe {
            libc::renameatx_np(
                source_dir.as_raw_fd(),
                source.as_ptr(),
                target_dir.as_raw_fd(),
                target.as_ptr(),
                libc::RENAME_EXCL,
            )
        };
        if result != 0 {
            return Err(error(format!(
                "atomic no-replace {label} publication failed: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(())
    }

    fn unlink(parent: &File, name: &CStr, directory: bool) {
        // SAFETY: valid held descriptor/name. Cleanup never follows the final entry.
        let _ = unsafe {
            libc::unlinkat(
                parent.as_raw_fd(),
                name.as_ptr(),
                if directory { libc::AT_REMOVEDIR } else { 0 },
            )
        };
    }

    fn io_error(cause: impl std::fmt::Display) -> ForgeError {
        error(format!("authoring output filesystem operation failed: {cause}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifacts() -> Vec<OutputArtifact> {
        vec![
            OutputArtifact { relative_path: "plan.json".into(), bytes: b"{}\n".to_vec() },
            OutputArtifact { relative_path: "policies/one.md".into(), bytes: b"# One\n".to_vec() },
            OutputArtifact { relative_path: "policies/two.md".into(), bytes: b"# Two\n".to_vec() },
            OutputArtifact {
                relative_path: "one.lifecycle.json".into(),
                bytes: b"{\"state\":\"draft\"}\n".to_vec(),
            },
            OutputArtifact {
                relative_path: "two.lifecycle.json".into(),
                bytes: b"{\"state\":\"draft\"}\n".to_vec(),
            },
            OutputArtifact { relative_path: "handoff.json".into(), bytes: b"{}\n".to_vec() },
            OutputArtifact {
                relative_path: "components.lock.json".into(),
                bytes: b"{}\n".to_vec(),
            },
            OutputArtifact { relative_path: "provenance.json".into(), bytes: b"{}\n".to_vec() },
            OutputArtifact {
                relative_path: "plan.html".into(),
                bytes: b"<!doctype html>\n".to_vec(),
            },
        ]
    }

    #[test]
    fn rejects_portable_path_aliases_and_file_directory_collisions() {
        for path in ["../a", "/a", "a\\b", "C:a", "CON.txt", "lpt1", "a.", "a//b", "a/./b"] {
            assert!(validate_relative(path).is_err(), "{path}");
        }
        for paths in [["a", "A"], ["a", "a/b"], ["a/b", "A"], ["a/b", "A/c"]] {
            let artifacts =
                paths.map(|path| OutputArtifact { relative_path: path.into(), bytes: Vec::new() });
            assert!(validate_artifacts(&artifacts).is_err());
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn failures_before_rename_leave_no_destination_or_staging_data() {
        use super::unix::{PublishEvent, publish_with_hook};

        for fault in [
            PublishEvent::Staged,
            PublishEvent::FileWritten(0),
            PublishEvent::FileWritten(2),
            PublishEvent::BeforeRename,
        ] {
            let root = tempfile::tempdir().unwrap();
            let root = root.path().canonicalize().unwrap();
            let result = publish_with_hook(&root, "generated", &artifacts(), |event| {
                assert!(!root.join("generated").exists());
                if event == fault { Err(error("injected interruption")) } else { Ok(()) }
            });
            assert!(result.is_err());
            assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn destination_is_complete_at_the_only_publication_point() {
        use super::unix::{PublishEvent, publish_with_hook};

        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let artifacts = artifacts();
        let result = publish_with_hook(&root, "generated", &artifacts, |event| {
            if event == PublishEvent::Published {
                for artifact in &artifacts {
                    assert_eq!(
                        std::fs::read(root.join("generated").join(&artifact.relative_path))
                            .unwrap(),
                        artifact.bytes
                    );
                }
                return Err(error("injected postcommit interruption"));
            }
            assert!(!root.join("generated").exists());
            Ok(())
        });
        assert!(result.is_err());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    }

    // Invoked in a separate test process so exit skips staging destructors,
    // matching interruption rather than merely exercising error rollback.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn interrupted_publication_child() {
        use super::unix::{PublishEvent, publish_with_hook};

        let Some(root) = std::env::var_os("FORGE_AUTHORING_TEST_INTERRUPT_ROOT") else {
            return;
        };
        let after = std::env::var_os("FORGE_AUTHORING_TEST_INTERRUPT_AFTER").is_some();
        publish_with_hook(Path::new(&root), "generated", &artifacts(), |event| {
            if event == if after { PublishEvent::Published } else { PublishEvent::FileWritten(0) } {
                std::process::exit(88);
            }
            Ok(())
        })
        .unwrap();
        panic!("the publication interruption hook did not run");
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn process_interruption_exposes_either_no_generation_or_the_complete_generation() {
        for after in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            let mut child = std::process::Command::new(std::env::current_exe().unwrap());
            child
                .args(["--exact", "authoring::output::tests::interrupted_publication_child"])
                .env("FORGE_AUTHORING_TEST_INTERRUPT_ROOT", &root)
                .env_remove("FORGE_AUTHORING_TEST_INTERRUPT_AFTER");
            if after {
                child.env("FORGE_AUTHORING_TEST_INTERRUPT_AFTER", "1");
            }
            let result = child.output().unwrap();
            assert_eq!(
                result.status.code(),
                Some(88),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(root.join("generated").exists(), after);
            if after {
                for artifact in artifacts() {
                    assert_eq!(
                        std::fs::read(root.join("generated").join(artifact.relative_path)).unwrap(),
                        artifact.bytes
                    );
                }
            } else {
                // Only the private staging directory can remain after interruption.
                let remaining =
                    std::fs::read_dir(&root).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
                assert_eq!(remaining.len(), 1);
                assert!(
                    remaining[0]
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".forge-authoring-stage-")
                );
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn existing_empty_directory_and_concurrent_destination_are_never_replaced() {
        use super::unix::{PublishEvent, publish_with_hook};

        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        std::fs::create_dir(root.join("existing")).unwrap();
        assert!(publish(&root, Path::new("existing"), &artifacts()).is_err());
        for sentinel in [false, true] {
            let name = if sentinel { "raced-full" } else { "raced-empty" };
            let result = publish_with_hook(&root, name, &artifacts(), |event| {
                if event == PublishEvent::BeforeRename {
                    std::fs::create_dir(root.join(name)).unwrap();
                    if sentinel {
                        std::fs::write(root.join(name).join("sentinel"), b"preserved").unwrap();
                    }
                }
                Ok(())
            });
            assert!(result.is_err());
            assert_eq!(std::fs::read_dir(root.join(name)).unwrap().count(), usize::from(sentinel));
            if sentinel {
                assert_eq!(std::fs::read(root.join(name).join("sentinel")).unwrap(), b"preserved");
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn rejects_symlink_parent_and_destination_without_following_them() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("linked")).unwrap();
        assert!(publish(&root, Path::new("linked/generated"), &artifacts()).is_err());
        assert!(publish(&root, Path::new("linked"), &artifacts()).is_err());
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 0);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn identical_generations_have_identical_bytes_across_directories() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        for root in [first.path(), second.path()] {
            let root = root.canonicalize().unwrap();
            publish(&root, Path::new("generated"), &artifacts()).unwrap();
        }
        for artifact in artifacts() {
            assert_eq!(
                std::fs::read(first.path().join("generated").join(&artifact.relative_path))
                    .unwrap(),
                std::fs::read(second.path().join("generated").join(&artifact.relative_path))
                    .unwrap(),
            );
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn new_file_publication_refuses_an_existing_destination() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        std::fs::write(root.join("pack.json"), b"existing\n").unwrap();
        let failure =
            publish_new_file(&root, Path::new("pack.json"), b"replacement\n").unwrap_err();
        assert!(failure.to_string().contains("already exists"), "{failure}");
        assert!(!failure.to_string().contains("generation directory"));
        assert_eq!(std::fs::read(root.join("pack.json")).unwrap(), b"existing\n");
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn replacing_the_staging_pathname_cannot_substitute_the_published_file() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path().canonicalize().unwrap();
        let mut swapped = false;
        publish_new_file_with_hook(&root, Path::new("pack.json"), b"original\n", |event| {
            if event == FilePublishEvent::Staged {
                // Simulate a concurrent writer replacing the staging entry in the
                // destination parent with an attacker-controlled directory.
                let staging = std::fs::read_dir(&root)
                    .unwrap()
                    .filter_map(Result::ok)
                    .find(|entry| {
                        entry.file_name().to_string_lossy().starts_with(".forge-authoring-file-")
                    })
                    .expect("the private staging directory is visible")
                    .path();
                let staging_name = staging.file_name().unwrap().to_owned();
                let decoy = root.join("decoy");
                std::fs::create_dir(&decoy).unwrap();
                std::fs::write(decoy.join("file"), b"attacker\n").unwrap();
                std::fs::rename(&staging, root.join("swapped-away")).unwrap();
                std::fs::rename(&decoy, root.join(staging_name)).unwrap();
                swapped = true;
            }
            Ok(())
        })
        .unwrap();
        assert!(swapped, "the staging pathname was not exercised");
        // The source rename goes through the held staging descriptor, so the
        // substituted entry cannot reach the destination.
        assert_eq!(std::fs::read(root.join("pack.json")).unwrap(), b"original\n");
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    #[test]
    fn unsupported_platform_fails_without_creating_output() {
        let root = tempfile::tempdir().unwrap();
        let result = publish(root.path(), Path::new("generated"), &artifacts());
        assert!(result.unwrap_err().to_string().contains("unsupported"));
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}
