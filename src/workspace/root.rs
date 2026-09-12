//! Descriptor-anchored reads of explicitly named project resources.

use std::fs::File;
use std::path::Path;
#[cfg(any(unix, windows))]
use std::path::PathBuf;

use super::contract::{Error, Result};
use super::index::validate_path;

#[derive(Debug)]
pub(crate) struct Root {
    #[cfg(any(unix, windows))]
    canonical: PathBuf,
    #[cfg(unix)]
    directory: File,
    #[cfg(windows)]
    _ancestors: Vec<File>,
}

pub(crate) struct Captured {
    pub bytes: Vec<u8>,
    pub identity: (u64, u64),
    pub sha256: String,
}

impl Root {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let canonical = path.canonicalize().map_err(|_| Error::containment())?;
        #[cfg(unix)]
        {
            let mut directory = File::open("/").map_err(|_| Error::containment())?;
            for component in canonical.components() {
                match component {
                    std::path::Component::RootDir => {}
                    std::path::Component::Normal(name) => {
                        directory =
                            unix_open(&directory, name, true).map_err(|_| Error::containment())?;
                    }
                    _ => return Err(Error::containment()),
                }
            }
            Ok(Self { canonical, directory })
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::{MetadataExt as _, OpenOptionsExt as _};
            let mut candidate = PathBuf::new();
            let mut ancestors = Vec::new();
            for component in canonical.components() {
                candidate.push(component.as_os_str());
                if matches!(component, std::path::Component::Normal(_)) {
                    // No FILE_SHARE_DELETE: keep the entire canonical ancestry fixed.
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .share_mode(3)
                        .custom_flags(0x0200_0000 | 0x0020_0000)
                        .open(&candidate)
                        .map_err(|_| Error::containment())?;
                    let metadata = file.metadata().map_err(|_| Error::containment())?;
                    if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
                        return Err(Error::containment());
                    }
                    ancestors.push(file);
                }
            }
            if ancestors.is_empty() {
                return Err(Error::containment());
            }
            Ok(Self { canonical, _ancestors: ancestors })
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = canonical;
            Err(Error::containment())
        }
    }

    pub(crate) fn read_config(&self) -> Result<Option<Captured>> {
        self.read_internal(".forge.toml", 1024 * 1024)
    }

    /// The canonical project root this handle was opened on. Consumers resolve
    /// project files through this path, never through the caller's spelling.
    #[cfg(any(unix, windows))]
    pub(crate) fn project_path(&self) -> &Path {
        &self.canonical
    }

    pub(crate) fn read(&self, path: &str, limit: usize) -> Result<Captured> {
        validate_path(path)?;
        self.read_internal(path, limit)?
            .ok_or_else(|| Error::new("not-found", "The registered resource is missing.", false))
    }

    pub(crate) fn read_index(&self) -> Result<Option<Captured>> {
        self.read_internal(super::index::INDEX_PATH, super::index::MAX_INDEX_BYTES)
    }

    #[cfg(unix)]
    fn read_internal(&self, path: &str, limit: usize) -> Result<Option<Captured>> {
        use std::io::Read as _;
        use std::os::unix::fs::MetadataExt as _;
        let mut directory = self.directory.try_clone().map_err(|_| Error::containment())?;
        let mut segments = path.split('/').peekable();
        while let Some(segment) = segments.next() {
            let is_directory = segments.peek().is_some();
            let file = match unix_open(&directory, std::ffi::OsStr::new(segment), is_directory) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(_) => return Err(Error::containment()),
            };
            if is_directory {
                directory = file;
                continue;
            }
            let before = file.metadata().map_err(|_| Error::containment())?;
            if !before.is_file() || before.nlink() != 1 {
                return Err(Error::containment());
            }
            if before.len() > limit as u64 {
                return Err(oversized());
            }
            let mut bytes =
                Vec::with_capacity(usize::try_from(before.len()).map_err(|_| oversized())?);
            (&file)
                .take(limit.saturating_add(1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| Error::containment())?;
            let after = file.metadata().map_err(|_| Error::containment())?;
            if bytes.len() > limit {
                return Err(oversized());
            }
            if before.len() != after.len()
                || before.mtime() != after.mtime()
                || before.mtime_nsec() != after.mtime_nsec()
                || before.ctime() != after.ctime()
                || before.ctime_nsec() != after.ctime_nsec()
                || after.nlink() != 1
            {
                return Err(Error::new(
                    "version-conflict",
                    "The resource changed while being read. Reload it.",
                    true,
                ));
            }
            let sha256 = crate::hashing::sha256_hex(&bytes);
            return Ok(Some(Captured { bytes, identity: (after.dev(), after.ino()), sha256 }));
        }
        Err(Error::containment())
    }

    #[cfg(windows)]
    fn read_internal(&self, path: &str, limit: usize) -> Result<Option<Captured>> {
        // The canonical ancestors are held without delete sharing for this Root's
        // lifetime. The existing Windows reader also holds all relative parents
        // while opening the final non-reparse, single-link regular file.
        match std::fs::symlink_metadata(self.canonical.join(path)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Error::containment()),
            Ok(_) => {}
        }
        let (bytes, identity) = crate::linkage::read_confined_local_file(
            &self.canonical,
            Path::new(path),
            limit as u64,
        )
        .map_err(|_| Error::containment())?;
        let sha256 = crate::hashing::sha256_hex(&bytes);
        Ok(Some(Captured { bytes, identity, sha256 }))
    }

    #[cfg(not(any(unix, windows)))]
    fn read_internal(&self, _path: &str, _limit: usize) -> Result<Option<Captured>> {
        Err(Error::containment())
    }
}

fn oversized() -> Error {
    Error::new("payload-too-large", "The resource exceeds the supported size limit.", false)
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn unix_open(parent: &File, name: &std::ffi::OsStr, directory: bool) -> std::io::Result<File> {
    use std::os::fd::{AsRawFd as _, FromRawFd as _};
    use std::os::unix::ffi::OsStrExt as _;
    let name =
        std::ffi::CString::new(name.as_bytes()).map_err(|_| std::io::ErrorKind::InvalidInput)?;
    let flags = libc::O_RDONLY
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | if directory { libc::O_DIRECTORY } else { libc::O_NONBLOCK };
    // SAFETY: parent is a live owned descriptor, name is NUL-terminated, flags
    // request no creation. Successful descriptors are immediately owned by File.
    let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: descriptor is new, valid, and has no other owner.
    Ok(unsafe { File::from_raw_fd(descriptor) })
}

/// Destination identity captured before preview. Parent creation is not implicit.
pub(crate) struct Target {
    pub path: String,
    pub base: Option<Captured>,
    pub version: String,
    #[cfg(unix)]
    parent: File,
    #[cfg(unix)]
    parent_identity: (u64, u64),
    #[cfg(windows)]
    _parents: Vec<File>,
}

impl Root {
    pub(crate) fn target(&self, path: &str) -> Result<Target> {
        validate_path(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            let mut parent = self.directory.try_clone().map_err(|_| Error::containment())?;
            let segments: Vec<_> = path.split('/').collect();
            for segment in &segments[..segments.len() - 1] {
                parent = unix_open(&parent, std::ffi::OsStr::new(segment), true)
                    .map_err(|_| Error::containment())?;
            }
            let meta = parent.metadata().map_err(|_| Error::containment())?;
            let parent_identity = (meta.dev(), meta.ino());
            let base = self.read_internal(path, 10 * 1024 * 1024)?;
            let identity = base.as_ref().map_or_else(
                || "absent".to_owned(),
                |b| format!("{}:{}:{}", b.identity.0, b.identity.1, b.sha256),
            );
            let version = crate::hashing::sha256_hex(
                format!("{path}:{}:{}:{identity}", meta.dev(), meta.ino()).as_bytes(),
            );
            Ok(Target { path: path.to_owned(), base, version, parent, parent_identity })
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::{MetadataExt as _, OpenOptionsExt as _};
            let mut current = self.canonical.clone();
            let mut parents = Vec::new();
            let segments: Vec<_> = path.split('/').collect();
            for segment in &segments[..segments.len() - 1] {
                current.push(segment);
                let file = std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(3)
                    .custom_flags(0x0200_0000 | 0x0020_0000)
                    .open(&current)
                    .map_err(|_| Error::containment())?;
                let metadata = file.metadata().map_err(|_| Error::containment())?;
                if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
                    return Err(Error::containment());
                }
                parents.push(file);
            }
            let base = self.read_internal(path, 10 * 1024 * 1024)?;
            let identity = base.as_ref().map_or_else(
                || "absent".to_owned(),
                |b| format!("{}:{}:{}", b.identity.0, b.identity.1, b.sha256),
            );
            let version = crate::hashing::sha256_hex(format!("{path}:{identity}").as_bytes());
            Ok(Target { path: path.to_owned(), base, version, _parents: parents })
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(Error::new(
                "resource-containment",
                "This platform does not yet provide the required conditional workspace writer.",
                false,
            ))
        }
    }

    /// One file is published by one directory-relative rename. All fallible
    /// preparation precedes rename; post-rename durability errors never turn an
    /// already committed write into a reported failure. File/parent sync is
    /// attempted. An interrupted process may leave an inert private temp file.
    #[cfg(unix)]
    #[allow(unsafe_code)] // Reviewed descriptor-relative single-file publication seam.
    pub(crate) fn commit(
        &self,
        target: &Target,
        bytes: &[u8],
        revalidate: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        use std::io::Write as _;
        use std::os::fd::{AsRawFd as _, FromRawFd as _};
        struct Cleanup<'a> {
            parent: &'a File,
            name: std::ffi::CString,
        }
        impl Drop for Cleanup<'_> {
            fn drop(&mut self) {
                // SAFETY: live parent and generated NUL-terminated temp name.
                let _ = unsafe { libc::unlinkat(self.parent.as_raw_fd(), self.name.as_ptr(), 0) };
            }
        }
        if bytes.len() > 10 * 1024 * 1024 {
            return Err(oversized());
        }
        let token = super::session::random_token()?;
        let temp = std::ffi::CString::new(format!(".forge-workspace-{}", *token))
            .map_err(|_| Error::invalid())?;
        // SAFETY: owned directory descriptor, NUL-terminated generated name,
        // restrictive mode, exclusive creation and no link following.
        let fd = unsafe {
            libc::openat(
                target.parent.as_raw_fd(),
                temp.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(Error::containment());
        }
        // SAFETY: fresh successful descriptor has exactly this owner.
        let mut staged = unsafe { File::from_raw_fd(fd) };
        let cleanup = Cleanup { parent: &target.parent, name: temp };
        staged
            .write_all(bytes)
            .and_then(|()| staged.sync_all())
            .map_err(|_| Error::containment())?;
        revalidate()?;
        verify_staging(&target.parent, &cleanup.name, &staged, bytes)?;
        let current = self.target(&target.path)?;
        if current.version != target.version || current.parent_identity != target.parent_identity {
            // The 409 envelope carries the current version so the client can
            // reconcile instead of reading the target again.
            let mut mismatch = conflict();
            mismatch.resource_version = Some(current.version);
            return Err(mismatch);
        }
        let leaf =
            std::ffi::CString::new(target.path.rsplit('/').next().ok_or_else(Error::invalid)?)
                .map_err(|_| Error::invalid())?;
        let result = if target.base.is_none() {
            // No-replace publication also closes the absent-target race.
            #[cfg(target_os = "macos")]
            // SAFETY: both names and descriptors live across this atomic call.
            unsafe {
                libc::renameatx_np(
                    target.parent.as_raw_fd(),
                    cleanup.name.as_ptr(),
                    target.parent.as_raw_fd(),
                    leaf.as_ptr(),
                    libc::RENAME_EXCL,
                )
            }
            #[cfg(target_os = "linux")]
            // SAFETY: both names and descriptors live across this atomic call.
            unsafe {
                libc::renameat2(
                    target.parent.as_raw_fd(),
                    cleanup.name.as_ptr(),
                    target.parent.as_raw_fd(),
                    leaf.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            }
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            {
                return Err(Error::containment());
            }
        } else {
            // SAFETY: descriptor-relative rename cannot follow a substituted
            // destination symlink. Original identity was rechecked immediately
            // above; external writers must not race the final rename syscall.
            unsafe {
                libc::renameat(
                    target.parent.as_raw_fd(),
                    cleanup.name.as_ptr(),
                    target.parent.as_raw_fd(),
                    leaf.as_ptr(),
                )
            }
        };
        if result != 0 {
            return Err(rename_failure(&std::io::Error::last_os_error()));
        }
        let _ = target.parent.sync_all();
        Ok(())
    }
    #[cfg(windows)]
    pub(crate) fn commit(
        &self,
        target: &Target,
        bytes: &[u8],
        revalidate: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        use std::io::Write as _;
        use std::os::windows::fs::OpenOptionsExt as _;
        if bytes.len() > 10 * 1024 * 1024 {
            return Err(oversized());
        }
        let absolute = self.canonical.join(&target.path);
        let parent = absolute.parent().ok_or_else(Error::containment)?;
        let temporary =
            parent.join(format!("forge-workspace-{}.tmp", *super::session::random_token()?));
        // DELETE access permits renaming this same held handle. share_mode(0)
        // prevents another process from replacing the staged source before rename.
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .access_mode(0x8000_0000 | 0x4000_0000 | 0x0001_0000)
            .share_mode(0)
            .custom_flags(0x0020_0000)
            .open(&temporary)
            .map_err(|_| Error::containment())?;
        let result = (|| {
            file.write_all(bytes)
                .and_then(|()| file.sync_all())
                .map_err(|_| Error::containment())?;
            revalidate()?;
            let current = self.target(&target.path)?;
            if current.version != target.version {
                let mut mismatch = conflict();
                mismatch.resource_version = Some(current.version);
                return Err(mismatch);
            }
            windows_publish::rename(&file, &absolute, target.base.is_some())
        })();
        drop(file);
        if result.is_err() {
            let _ = std::fs::remove_file(&temporary);
        }
        result
    }
    #[cfg(not(any(unix, windows)))]
    pub(crate) fn commit(
        &self,
        _: &Target,
        _: &[u8],
        _: impl FnOnce() -> Result<()>,
    ) -> Result<()> {
        Err(Error::containment())
    }
}

#[cfg(unix)]
fn verify_staging(
    parent: &File,
    name: &std::ffi::CStr,
    staged: &File,
    expected: &[u8],
) -> Result<()> {
    use std::io::Read as _;
    use std::os::unix::{ffi::OsStrExt as _, fs::MetadataExt as _};
    let reopened = unix_open(parent, std::ffi::OsStr::from_bytes(name.to_bytes()), false)
        .map_err(|_| Error::containment())?;
    let actual = reopened.metadata().map_err(|_| Error::containment())?;
    let held = staged.metadata().map_err(|_| Error::containment())?;
    if !actual.is_file()
        || actual.nlink() != 1
        || (actual.dev(), actual.ino()) != (held.dev(), held.ino())
        || actual.len() != expected.len() as u64
    {
        return Err(conflict());
    }
    let mut reader = std::io::BufReader::new(reopened);
    let mut buffer = [0_u8; 8192];
    let mut offset = 0;
    loop {
        let count = reader.read(&mut buffer).map_err(|_| Error::containment())?;
        if count == 0 {
            break;
        }
        if expected.get(offset..offset + count) != Some(&buffer[..count]) {
            return Err(conflict());
        }
        offset += count;
    }
    if offset != expected.len() {
        return Err(conflict());
    }
    Ok(())
}

pub(crate) fn conflict() -> Error {
    Error::new(
        "version-conflict",
        "Inputs or destination changed. Reload and prepare a new preview.",
        true,
    )
}

/// The destination refused the atomic publication for a reason an identical
/// retry cannot clear (rename flags the filesystem does not implement,
/// permissions, quota or space). Reporting `version-conflict` here would tell
/// the client to reload and retry something that can never succeed.
pub(crate) fn publish_failed() -> Error {
    Error::new(
        "invalid-request",
        "The destination could not be published atomically on this filesystem.",
        false,
    )
}

/// Only a lost destination race (`EEXIST`/`ENOTEMPTY` from the no-replace
/// publication) is a reload-and-retry conflict; every other rename failure is a
/// filesystem limitation an identical retry reproduces.
#[cfg(unix)]
fn rename_failure(error: &std::io::Error) -> Error {
    match error.raw_os_error() {
        Some(libc::EEXIST | libc::ENOTEMPTY) => conflict(),
        _ => publish_failed(),
    }
}

#[cfg(windows)]
#[allow(unsafe_code)] // Small documented Windows handle-based publication seam.
mod windows_publish {
    use super::{Error, File, Path, Result, conflict, publish_failed};
    use std::os::windows::{ffi::OsStrExt as _, io::AsRawHandle as _};
    #[repr(C)]
    struct RenameInfo {
        replace: u32,
        root: *mut std::ffi::c_void,
        length: u32,
        name: [u16; 32768],
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetFileInformationByHandle(
            handle: *mut std::ffi::c_void,
            class: i32,
            info: *const std::ffi::c_void,
            length: u32,
        ) -> i32;
    }
    pub(super) fn rename(file: &File, target: &Path, replace: bool) -> Result<()> {
        /// Win32 `ERROR_FILE_EXISTS`, `ERROR_ALREADY_EXISTS`, `ERROR_DIR_NOT_EMPTY`:
        /// the no-replace publish lost the destination race.
        const ERROR_FILE_EXISTS: i32 = 80;
        const ERROR_ALREADY_EXISTS: i32 = 183;
        const ERROR_DIR_NOT_EMPTY: i32 = 145;
        let name: Vec<_> = target.as_os_str().encode_wide().take(32768).collect();
        if name.len() >= 32768 {
            return Err(Error::containment());
        }
        // Allocate in place: constructing this 64 KiB record before Box::new
        // would put the entire path buffer on the smaller Windows thread stack.
        let mut storage = Box::<RenameInfo>::new_uninit();
        // SAFETY: the allocation is aligned and sized for RenameInfo. Every
        // field (integers, UTF-16 units, and the null raw pointer) permits an
        // all-zero representation; initialize the entire allocation, including
        // padding, before creating a RenameInfo value or passing bytes to Win32.
        let mut info = unsafe {
            storage.as_mut_ptr().write_bytes(0, 1);
            storage.assume_init()
        };
        info.replace = u32::from(replace);
        info.length = u32::try_from(name.len() * 2).map_err(|_| Error::containment())?;
        info.name[..name.len()].copy_from_slice(&name);
        let length = u32::try_from(std::mem::offset_of!(RenameInfo, name) + name.len() * 2)
            .map_err(|_| Error::containment())?;
        // SAFETY: the live exclusive source handle has DELETE access; C-layout
        // aligned buffer includes FileNameLength bytes of UTF-16. Absolute target
        // ancestry is held without FILE_SHARE_DELETE for the entire operation.
        let result = unsafe {
            SetFileInformationByHandle(file.as_raw_handle(), 3, (&raw const *info).cast(), length)
        };
        if result == 0 {
            let raw = std::io::Error::last_os_error().raw_os_error();
            let exists = raw == Some(ERROR_FILE_EXISTS)
                || raw == Some(ERROR_ALREADY_EXISTS)
                || raw == Some(ERROR_DIR_NOT_EMPTY);
            // A replace publish targets a destination that already exists, so
            // only a no-replace publish can lose a destination race.
            return Err(if !replace && exists { conflict() } else { publish_failed() });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn destination_races_are_conflicts_and_publish_limitations_are_not() {
        for code in [libc::EEXIST, libc::ENOTEMPTY] {
            let error = rename_failure(&std::io::Error::from_raw_os_error(code));
            assert_eq!(error.code, "version-conflict", "{code}");
            assert!(error.retryable, "{code}");
            assert_eq!(error.resource_version, None);
        }
        for code in [libc::EINVAL, libc::ENOSPC, libc::EPERM, libc::EDQUOT] {
            let error = rename_failure(&std::io::Error::from_raw_os_error(code));
            assert_eq!(error.code, "invalid-request", "{code}");
            assert_eq!(
                error.message,
                "The destination could not be published atomically on this filesystem."
            );
            assert!(!error.retryable, "{code}");
        }
    }

    /// A target replaced after preview must report the current version, which is
    /// already known at the revalidation point.
    #[test]
    fn a_stale_target_conflict_carries_the_current_resource_version() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        let stale = root.target("output.json").unwrap();
        assert!(stale.base.is_none());
        std::fs::write(dir.path().join("output.json"), b"external").unwrap();
        let error = root.commit(&stale, b"new", || Ok(())).unwrap_err();
        assert_eq!(error.code, "version-conflict");
        assert!(error.retryable);
        let current = root.target("output.json").unwrap();
        assert_eq!(error.resource_version.as_deref(), Some(current.version.as_str()));
        assert_eq!(std::fs::read(dir.path().join("output.json")).unwrap(), b"external");
    }

    #[test]
    fn failing_revalidation_preserves_original_and_cleans_staging() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("output.json"), b"old").unwrap();
        let root = Root::open(dir.path()).unwrap();
        let target = root.target("output.json").unwrap();
        assert!(root.commit(&target, b"new", || Err(conflict())).is_err());
        assert_eq!(std::fs::read(dir.path().join("output.json")).unwrap(), b"old");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[test]
    fn staged_source_tampering_and_parent_swap_fail_before_publication() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        std::fs::write(dir.path().join("output.json"), b"old").unwrap();
        let target = root.target("output.json").unwrap();
        assert!(
            root.commit(&target, b"new", || {
                let staged = std::fs::read_dir(dir.path())
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .find(|path| {
                        path.file_name().unwrap().to_string_lossy().starts_with(".forge-workspace-")
                    })
                    .unwrap();
                std::fs::write(staged, b"malicious").unwrap();
                Ok(())
            })
            .is_err()
        );
        assert_eq!(std::fs::read(dir.path().join("output.json")).unwrap(), b"old");
        std::fs::create_dir(dir.path().join("parent")).unwrap();
        let target = root.target("parent/new.json").unwrap();
        assert!(
            root.commit(&target, b"new", || {
                std::fs::rename(dir.path().join("parent"), dir.path().join("moved")).unwrap();
                std::fs::create_dir(dir.path().join("parent")).unwrap();
                Ok(())
            })
            .is_err()
        );
        assert!(!dir.path().join("parent/new.json").exists());
        assert!(!dir.path().join("moved/new.json").exists());
    }

    #[test]
    fn missing_index_and_bounded_explicit_read() {
        let dir = tempfile::tempdir().unwrap();
        let root = Root::open(dir.path()).unwrap();
        assert!(root.read_index().unwrap().is_none());
        std::fs::write(dir.path().join("policy.md"), b"Policy").unwrap();
        assert_eq!(root.read("policy.md", 6).unwrap().bytes, b"Policy");
        assert!(root.read("policy.md", 5).is_err());
        assert!(root.read("../policy.md", 100).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn descriptor_anchor_survives_root_path_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let selected = dir.path().join("selected");
        std::fs::create_dir(&selected).unwrap();
        std::fs::write(selected.join("policy.md"), b"inside").unwrap();
        let root = Root::open(&selected).unwrap();
        std::fs::rename(&selected, dir.path().join("moved")).unwrap();
        std::fs::create_dir(&selected).unwrap();
        std::fs::write(selected.join("policy.md"), b"outside").unwrap();
        assert_eq!(root.read("policy.md", 100).unwrap().bytes, b"inside");
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_hard_links_directories_and_parent_replacement_fail() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("private.md"), b"private").unwrap();
        symlink(outside.path().join("private.md"), dir.path().join("symlink.md")).unwrap();
        std::fs::hard_link(outside.path().join("private.md"), dir.path().join("hard.md")).unwrap();
        symlink(outside.path(), dir.path().join("parent")).unwrap();
        std::fs::create_dir(dir.path().join("directory")).unwrap();
        let root = Root::open(dir.path()).unwrap();
        for path in ["symlink.md", "hard.md", "parent/private.md", "directory"] {
            assert!(root.read(path, 100).is_err(), "{path}");
        }
    }
}
