//! Actual confined original generations for read-only evidence freshness inspection.
//!
//! Private proof constructors consume held native handles. Exact original bytes
//! are rechecked with bounded streaming; URI references are never fetched.

use std::fs::{File, Metadata};
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use crate::ForgeError;

use super::{
    ResourceEvidence, manifest, relabel_mapping_error, required_string, sanitize_reference_label,
};
use crate::evidence_capture::{
    CaptureLease, CaptureRole, CaptureSession, LocalObservation, ProjectionBudget,
};
use crate::hashing::sha256_hex;
use crate::io;
use crate::mapping::inventory;
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

/// Fixed streaming scratch bound, independent of the retained original budget.
const SCRATCH_BYTES: usize = 32 * 1024;

/// Actual safe directory identity used internally, never serialized as authority.
struct DirectoryGeneration {
    /// Exact normalized observed ancestor path for private revalidation only.
    path: PathBuf,
    /// Actual native volume/file tuple from the held directory handle.
    identity: (u64, u64),
}

/// Qualified actual root, retaining its native directory handles and exact ancestry.
pub(crate) struct RootGeneration {
    /// Normalized absolute original root path.
    path: PathBuf,
    /// Actual final root directory identity established by the qualifying constructor.
    identity: (u64, u64),
    /// Complete original root component generations in traversal order.
    ancestors: Vec<DirectoryGeneration>,
    /// Held actual root ancestors; Windows excludes delete sharing.
    directories: Vec<File>,
}

impl RootGeneration {
    /// Return the actual qualified root directory identity only for internal alias refusal.
    pub(crate) fn identity(&self) -> (u64, u64) {
        self.identity
    }

    /// Borrow the actual root path solely for internal relative-base composition.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

/// Exact original file generation, without a detached caller-authored constructor.
pub(crate) struct FileGeneration {
    /// Qualified actual root used by the original held-handle traversal.
    root: Rc<RootGeneration>,
    /// Exact normalized root-relative path; no displayed evidence href substitution.
    relative: PathBuf,
    /// Actual safe descendant parent generations from that traversal.
    ancestors: Vec<DirectoryGeneration>,
    /// Actual single-link regular-file identity, retained with its original handle.
    identity: (u64, u64),
    /// Held original file instance prevents identity reuse while the proof lives.
    _file: File,
    /// Held original safe descendants; Windows excludes delete sharing.
    _directories: Vec<File>,
}

impl FileGeneration {
    /// Borrow the actual original root descendant without revealing it publicly.
    pub(crate) fn relative(&self) -> &Path {
        &self.relative
    }

    /// Return the actual held original instance identity for internal alias refusal.
    pub(crate) fn identity(&self) -> (u64, u64) {
        self.identity
    }
}

/// Actual first missing component under qualified root and existing safe ancestors.
pub(crate) struct AbsenceGeneration {
    /// Qualified root at the actual missing observation.
    root: Rc<RootGeneration>,
    /// Complete requested normalized descendant, including unavailable trailing components.
    relative: PathBuf,
    /// Exact first component whose native open returned typed `NotFound`.
    missing: PathBuf,
    /// Complete existing safe descendant ancestors preceding that missing component.
    ancestors: Vec<DirectoryGeneration>,
    /// Held safe ancestors, without an invented file identity for missing bytes.
    _directories: Vec<File>,
}

/// Only actual present bytes or typed actual missing generations cross the private reader seam.
pub(crate) enum CapturedLocal {
    /// Complete bounded bytes plus their actual original instance proof.
    Present(Vec<u8>, FileGeneration),
    /// No bytes observed: exact missing component/root ancestry proof only.
    Absent(AbsenceGeneration),
}

/// Held-handle traversal result before any data read, retaining typed `NotFound`.
enum OpenedLocal {
    /// Single-link regular file plus actual safe parent observations.
    Present(File, (u64, u64), Vec<DirectoryGeneration>, Vec<File>),
    /// First missing component plus actual safe existing parents.
    Absent(PathBuf, Vec<DirectoryGeneration>, Vec<File>),
}

/// Require the existing normalized confined descendant spelling without canonicalizing through links.
pub(crate) fn validate_relative(relative: &Path) -> Result<(), ForgeError> {
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
        || !super::has_normalized_path_spelling(relative)
    {
        return Err(error("evidence inspection input is not a normalized descendant"));
    }
    Ok(())
}

/// Qualify every actual normalized absolute root component through no-link native directory opens.
pub(crate) fn qualify_root(root: &Path) -> Result<RootGeneration, ForgeError> {
    if !root.is_absolute()
        || root
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        || !super::has_normalized_path_spelling(root)
    {
        return Err(error("evidence inspection root is not normalized and absolute"));
    }
    let (ancestors, directories) = open_root(root)?;
    if directories.is_empty() {
        return Err(error("evidence inspection root is unsupported"));
    }
    let identity =
        ancestors.last().ok_or_else(|| error("qualified root ancestry is missing"))?.identity;
    Ok(RootGeneration { path: root.to_path_buf(), identity, ancestors, directories })
}

/// Compare actual current root ancestry, not canonicalized path labels or caller-supplied identity.
pub(crate) fn verify_root(original: &RootGeneration) -> Result<(), ForgeError> {
    let current = qualify_root(&original.path)?;
    compare_ancestors(&original.ancestors, &current.ancestors)
}

/// Observe actual local bytes under the remaining limit, or typed missing ancestry when permitted.
pub(crate) fn capture_local(
    root: Rc<RootGeneration>,
    relative: &Path,
    limit: u64,
    allow_missing: bool,
) -> Result<CapturedLocal, ForgeError> {
    validate_relative(relative)?;
    verify_root(&root)?;
    match open_local(&root, relative, allow_missing)? {
        OpenedLocal::Present(mut file, identity, ancestors, directories) => {
            let bytes = read_bounded(&mut file, limit)?;
            if file_identity(&file)? != identity
                || file.metadata().map_err(|_| error("cannot inspect captured original"))?.len()
                    != bytes.len() as u64
            {
                return Err(error("evidence inspection original changed during capture"));
            }
            verify_root(&root)?;
            Ok(CapturedLocal::Present(
                bytes,
                FileGeneration {
                    root,
                    relative: relative.to_path_buf(),
                    ancestors,
                    identity,
                    _file: file,
                    _directories: directories,
                },
            ))
        }
        OpenedLocal::Absent(missing, ancestors, directories) => {
            verify_root(&root)?;
            Ok(CapturedLocal::Absent(AbsenceGeneration {
                root,
                relative: relative.to_path_buf(),
                missing,
                ancestors,
                _directories: directories,
            }))
        }
    }
}

/// Stream-compare exact full bytes, identity, safe ancestry and EOF without retaining a second input Vec.
pub(crate) fn verify_file(original: &FileGeneration, bytes: &[u8]) -> Result<(), ForgeError> {
    verify_root(&original.root)?;
    let OpenedLocal::Present(mut file, identity, ancestors, _directories) =
        open_local(&original.root, &original.relative, false)?
    else {
        return Err(error("evidence inspection original is unavailable"));
    };
    compare_ancestors(&original.ancestors, &ancestors)?;
    if identity != original.identity
        || file.metadata().map_err(|_| error("cannot inspect current original"))?.len()
            != bytes.len() as u64
    {
        return Err(error("evidence inspection original identity or size changed"));
    }
    let mut scratch = vec![0_u8; SCRATCH_BYTES].into_boxed_slice();
    let mut offset = 0_usize;
    loop {
        let count =
            file.read(&mut scratch).map_err(|_| error("cannot recheck current original"))?;
        if count == 0 {
            break;
        }
        let end = offset
            .checked_add(count)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| error("evidence inspection original bytes changed"))?;
        if scratch[..count] != bytes[offset..end] {
            return Err(error("evidence inspection original bytes changed"));
        }
        offset = end;
    }
    if offset != bytes.len()
        || file_identity(&file)? != original.identity
        || file.metadata().map_err(|_| error("cannot inspect rechecked original"))?.len()
            != bytes.len() as u64
    {
        return Err(error("evidence inspection original changed during recheck"));
    }
    verify_root(&original.root)
}

/// Require the same actual first missing component and safe existing ancestry; newly appeared bytes refuse.
pub(crate) fn verify_absence(original: &AbsenceGeneration) -> Result<(), ForgeError> {
    verify_root(&original.root)?;
    let OpenedLocal::Absent(missing, ancestors, _directories) =
        open_local(&original.root, &original.relative, true)?
    else {
        return Err(error("previously unavailable local evidence appeared"));
    };
    if missing != original.missing {
        return Err(error("unavailable local evidence ancestry changed"));
    }
    compare_ancestors(&original.ancestors, &ancestors)?;
    verify_root(&original.root)
}

/// Compare complete exact directory generations without interpreting byte-equal replacements as stable.
fn compare_ancestors(
    original: &[DirectoryGeneration],
    current: &[DirectoryGeneration],
) -> Result<(), ForgeError> {
    if original.len() != current.len()
        || original
            .iter()
            .zip(current)
            .any(|(left, right)| left.path != right.path || left.identity != right.identity)
    {
        return Err(error("evidence inspection root or parent identity changed"));
    }
    Ok(())
}

/// Read through fixed scratch and check actual retained growth before append; never retain an oversize prefix.
fn read_bounded(file: &mut File, limit: u64) -> Result<Vec<u8>, ForgeError> {
    if file.metadata().map_err(|_| error("cannot inspect original input"))?.len() > limit {
        return Err(error("evidence inspection original exceeds remaining byte limit"));
    }
    let mut bytes = Vec::new();
    let mut scratch = vec![0_u8; SCRATCH_BYTES].into_boxed_slice();
    loop {
        let count = file.read(&mut scratch).map_err(|_| error("cannot read original input"))?;
        if count == 0 {
            break;
        }
        let length = bytes
            .len()
            .checked_add(count)
            .ok_or_else(|| error("original byte count overflowed"))?;
        if u64::try_from(length).map_err(|_| error("original byte count conversion failed"))?
            > limit
        {
            return Err(error("evidence inspection original exceeds remaining byte limit"));
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
    Ok(bytes)
}

/// Reuse the current regular/single-link original handle checks without accepting directory aliases.
fn file_identity(file: &File) -> Result<(u64, u64), ForgeError> {
    let identity = super::validate_open_evidence(file, "inspection-input")
        .map_err(|_| error("evidence inspection input kind or identity is unsafe"))?;
    Ok((identity.volume, identity.file))
}

/// Require a safe actual directory handle and return its native identity without raw error disclosure.
fn directory_generation(file: &File, path: &Path) -> Result<DirectoryGeneration, ForgeError> {
    let metadata =
        file.metadata().map_err(|_| error("cannot inspect evidence inspection directory"))?;
    if !metadata.is_dir() {
        return Err(error("evidence inspection ancestor is not a directory"));
    }
    Ok(DirectoryGeneration {
        path: path.to_path_buf(),
        identity: directory_identity(file, &metadata)?,
    })
}

/// Return actual Unix directory volume/inode; regular-file hardlink rules do not apply to directories.
#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)] // Windows and unsupported identity variants can fail.
fn directory_identity(_file: &File, metadata: &Metadata) -> Result<(u64, u64), ForgeError> {
    use std::os::unix::fs::MetadataExt as _;
    Ok((metadata.dev(), metadata.ino()))
}

/// Return actual Windows directory identity only after rejecting a reparse handle.
#[cfg(windows)]
fn directory_identity(file: &File, metadata: &Metadata) -> Result<(u64, u64), ForgeError> {
    use std::os::windows::fs::MetadataExt as _;
    if metadata.file_attributes() & 0x0000_0400 != 0 {
        return Err(error("evidence inspection ancestor is a reparse point"));
    }
    let (volume, index, _) = super::windows_evidence_identity::information(file)
        .map_err(|_| error("cannot inspect evidence inspection directory identity"))?;
    Ok((u64::from(volume), index))
}

/// Refuse unsupported native identity platforms rather than fabricating path-only proof.
#[cfg(not(any(unix, windows)))]
fn directory_identity(_file: &File, _metadata: &Metadata) -> Result<(u64, u64), ForgeError> {
    Err(error("evidence inspection native identities are unsupported"))
}

/// Open one actual Unix child relative to an already held parent, retaining typed native IO errors.
#[cfg(unix)]
#[allow(unsafe_code)] // Existing libc openat/no-follow held-handle confinement; no new dependency.
fn open_child(parent: &File, name: &std::ffi::OsStr, directory: bool) -> std::io::Result<File> {
    use std::ffi::CString;
    use std::os::fd::{AsRawFd as _, FromRawFd as _};
    use std::os::unix::ffi::OsStrExt as _;
    let name = CString::new(name.as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let flags = libc::O_RDONLY
        | libc::O_CLOEXEC
        | libc::O_NOFOLLOW
        | if directory { libc::O_DIRECTORY } else { libc::O_NONBLOCK };
    // SAFETY: the parent descriptor remains live, CString is NUL-terminated, and
    // a successful newly owned descriptor is immediately transferred into File.
    let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if descriptor < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        // SAFETY: successful openat returned this unwrapped owned descriptor.
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }
}

/// Walk actual Unix root components without following links and hold every directory generation.
#[cfg(unix)]
fn open_root(root: &Path) -> Result<(Vec<DirectoryGeneration>, Vec<File>), ForgeError> {
    let initial = File::open("/").map_err(|_| error("cannot open filesystem root"))?;
    let mut path = PathBuf::from("/");
    let mut ancestors = vec![directory_generation(&initial, &path)?];
    let mut directories = vec![initial];
    for component in root.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(name) => {
                let parent =
                    directories.last().ok_or_else(|| error("qualified root handle is missing"))?;
                let directory = open_child(parent, name, true)
                    .map_err(|_| error("cannot open safe evidence inspection root"))?;
                path.push(name);
                ancestors.push(directory_generation(&directory, &path)?);
                directories.push(directory);
            }
            _ => return Err(error("evidence inspection root spelling is unsupported")),
        }
    }
    Ok((ancestors, directories))
}

/// Walk actual Unix descendants, allowing ONLY typed `NotFound` after safe existing ancestors.
#[cfg(unix)]
fn open_local(
    root: &RootGeneration,
    relative: &Path,
    allow_missing: bool,
) -> Result<OpenedLocal, ForgeError> {
    let mut parent = root
        .directories
        .last()
        .ok_or_else(|| error("qualified root handle is missing"))?
        .try_clone()
        .map_err(|_| error("cannot retain qualified root handle"))?;
    let mut path = PathBuf::new();
    let mut ancestors = Vec::new();
    let mut directories = Vec::new();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(error("evidence input is not a descendant"));
        };
        path.push(name);
        let directory = components.peek().is_some();
        let opened = match open_child(&parent, name, directory) {
            Ok(file) => file,
            Err(cause) if allow_missing && cause.kind() == std::io::ErrorKind::NotFound => {
                return Ok(OpenedLocal::Absent(path, ancestors, directories));
            }
            Err(_) => return Err(error("cannot open safe evidence inspection input")),
        };
        if directory {
            ancestors.push(directory_generation(&opened, &root.path.join(&path))?);
            parent =
                opened.try_clone().map_err(|_| error("cannot retain safe descendant handle"))?;
            directories.push(opened);
        } else {
            let identity = file_identity(&opened)?;
            return Ok(OpenedLocal::Present(opened, identity, ancestors, directories));
        }
    }
    Err(error("evidence input path is empty"))
}

/// Open Windows entries without following reparse targets and exclude delete-sharing during traversal.
#[cfg(windows)]
fn open_windows(path: &Path, directory: bool) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt as _;
    std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x0000_0001 | 0x0000_0002)
        .custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 })
        .open(path)
}

/// Walk and hold every actual Windows root component, rejecting native reparse handles.
#[cfg(windows)]
fn open_root(root: &Path) -> Result<(Vec<DirectoryGeneration>, Vec<File>), ForgeError> {
    let mut path = PathBuf::new();
    let mut ancestors = Vec::new();
    let mut directories = Vec::new();
    for component in root.components() {
        match component {
            Component::Prefix(prefix) => path.push(prefix.as_os_str()),
            Component::RootDir => {
                path.push(component.as_os_str());
                let directory = open_windows(&path, true)
                    .map_err(|_| error("cannot open safe evidence inspection root"))?;
                ancestors.push(directory_generation(&directory, &path)?);
                directories.push(directory);
            }
            Component::Normal(name) => {
                path.push(name);
                let directory = open_windows(&path, true)
                    .map_err(|_| error("cannot open safe evidence inspection root"))?;
                ancestors.push(directory_generation(&directory, &path)?);
                directories.push(directory);
            }
            _ => return Err(error("evidence inspection root spelling is unsupported")),
        }
    }
    if directories.is_empty() && path.is_absolute() {
        let directory = open_windows(&path, true)
            .map_err(|_| error("cannot open safe evidence inspection root"))?;
        ancestors.push(directory_generation(&directory, &path)?);
        directories.push(directory);
    }
    Ok((ancestors, directories))
}

/// Traverse Windows descendants with held no-delete ancestors and typed `NotFound`, never raw string absence.
#[cfg(windows)]
fn open_local(
    root: &RootGeneration,
    relative: &Path,
    allow_missing: bool,
) -> Result<OpenedLocal, ForgeError> {
    use std::os::windows::fs::MetadataExt as _;
    let mut full = root.path.clone();
    let mut partial = PathBuf::new();
    let mut ancestors = Vec::new();
    let mut directories = Vec::new();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(error("evidence input is not a descendant"));
        };
        full.push(name);
        partial.push(name);
        let directory = components.peek().is_some();
        let opened = match open_windows(&full, directory) {
            Ok(file) => file,
            Err(cause) if allow_missing && cause.kind() == std::io::ErrorKind::NotFound => {
                return Ok(OpenedLocal::Absent(partial, ancestors, directories));
            }
            Err(_) => return Err(error("cannot open safe evidence inspection input")),
        };
        if opened
            .metadata()
            .map_err(|_| error("cannot inspect current evidence entry"))?
            .file_attributes()
            & 0x0000_0400
            != 0
        {
            return Err(error("evidence inspection input has a reparse point"));
        }
        if directory {
            ancestors.push(directory_generation(&opened, &full)?);
            directories.push(opened);
        } else {
            let identity = file_identity(&opened)?;
            return Ok(OpenedLocal::Present(opened, identity, ancestors, directories));
        }
    }
    Err(error("evidence input path is empty"))
}

/// Refuse platforms without the existing native held-handle confinement primitives.
#[cfg(not(any(unix, windows)))]
fn open_root(_root: &Path) -> Result<(Vec<DirectoryGeneration>, Vec<File>), ForgeError> {
    Err(error("evidence inspection native confinement is unsupported"))
}

/// Refuse missing-platform observations rather than classifying them as unavailable evidence.
#[cfg(not(any(unix, windows)))]
fn open_local(
    _root: &RootGeneration,
    _relative: &Path,
    _allow_missing: bool,
) -> Result<OpenedLocal, ForgeError> {
    Err(error("evidence inspection native confinement is unsupported"))
}

/// Keep all public inspection failures fixed/redacted rather than leaking private IO paths or content.
fn error(reason: &'static str) -> ForgeError {
    ForgeError::PoamBuild(reason.to_string())
}

/// Closed local join result; it carries no evidence sufficiency or terminal authority.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalBindingStatus {
    /// Actual resolved path, all full digests and approved/observed sizes reconcile.
    Matched,
    /// A URI cannot establish a local-file assertion.
    ReferenceKindMismatch,
    /// The asserted root-relative path is not the captured evidence path.
    HrefMismatch,
    /// At least one approved/asserted/observed digest or size differs.
    HashMismatch,
    /// No local original was observed; the exact typed absence is retained.
    Unavailable,
}

/// Original local facts remain visible as facts even when a path/hash join is refused.
pub(crate) struct LocalBindingObservation {
    /// Fixed exact local assertion join state.
    pub(crate) status: LocalBindingStatus,
    /// Actual complete original digest, absent for unavailable or URI references.
    pub(crate) observed_sha256: Option<String>,
    /// Actual complete original length, absent for unavailable or URI references.
    pub(crate) observed_size: Option<u64>,
    /// True only for an actual captured and reconciled local regular-file generation.
    pub(crate) local_bytes_revalidated: bool,
}

/// Private reference to actual once-captured evidence, never built from supplied index JSON.
enum HeldReference {
    /// Actual root-relative evidence path with sealed present/absence observation.
    Local {
        /// Exact joined original path, not the displayed href label.
        relative: PathBuf,
        /// Actual once-captured original or typed qualified absence.
        observation: LocalObservation,
    },
    /// Metadata-only URI, without a network request or local observation.
    Uri,
}

/// Fresh native metadata and its actual local join facts; construction is crate-private and consumed.
pub(crate) struct PreparedFreshLinkage {
    /// Complete native index produced from captured manifest and original supported models.
    index: super::LinkageIndex,
    /// Exact once-captured local reference or URI kind for every validated evidence key.
    references: std::collections::BTreeMap<String, HeldReference>,
}

impl PreparedFreshLinkage {
    /// Borrow the complete freshly produced index, not an admitted standalone prior index.
    pub(crate) fn index(&self) -> &super::LinkageIndex {
        &self.index
    }

    /// Reconcile a local assertion against actual captured full bytes without a per-assertion re-read.
    pub(crate) fn local_binding(
        &self,
        evidence_key: &str,
        assertion_relative: &Path,
        assertion_sha256: &str,
    ) -> Result<LocalBindingObservation, ForgeError> {
        let reference = self
            .references
            .get(evidence_key)
            .ok_or_else(|| error("validated evidence key is not in the actual fresh linkage"))?;
        let empty = |status| LocalBindingObservation {
            status,
            observed_sha256: None,
            observed_size: None,
            local_bytes_revalidated: false,
        };
        let HeldReference::Local { relative, observation } = reference else {
            return Ok(empty(LocalBindingStatus::ReferenceKindMismatch));
        };
        let LocalObservation::Present(lease) = observation else {
            return Ok(empty(LocalBindingStatus::Unavailable));
        };
        let record = self
            .index
            .evidence
            .iter()
            .find(|record| record.key == evidence_key)
            .ok_or_else(|| error("fresh evidence record is inconsistent"))?;
        let super::EvidenceReference::Local {
            approved_sha256,
            approved_size,
            observed_sha256,
            observed_size,
            ..
        } = &record.reference
        else {
            return Err(error("fresh local reference is inconsistent"));
        };
        let actual_size = u64::try_from(lease.bytes().len())
            .map_err(|_| error("actual evidence size cannot be represented"))?;
        if observed_sha256.is_none() || *observed_size != Some(actual_size) {
            return Err(error("fresh local observation is incomplete"));
        }
        let status = if assertion_relative != relative {
            LocalBindingStatus::HrefMismatch
        } else if observed_sha256.as_deref() != Some(assertion_sha256)
            || approved_sha256 != assertion_sha256
            || *approved_size != actual_size
        {
            LocalBindingStatus::HashMismatch
        } else {
            LocalBindingStatus::Matched
        };
        Ok(LocalBindingObservation {
            status,
            observed_sha256: observed_sha256.clone(),
            observed_size: *observed_size,
            local_bytes_revalidated: true,
        })
    }
}

/// Borrowed exact subject fields are counted before the retained strings are cloned.
#[derive(Clone, Copy, serde::Serialize)]
struct SubjectView<'a> {
    /// Exact source declaration key.
    resource_key: &'a str,
    /// Native requirement or implementation side.
    side: &'static str,
    /// Native subject kind, preserving original eligibility rules.
    #[serde(rename = "type")]
    subject_type: &'static str,
    /// Exact admitted subject identity.
    id: &'a str,
    /// Native computed fingerprint, not a raw source-file hash.
    sha256: &'a str,
}

/// Admit a complete subject relationship and encoded row before cloning or growing its array.
fn retain_subject(
    rows: &mut Vec<super::SubjectRecord>,
    row: SubjectView<'_>,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<(), ForgeError> {
    capture.relationships(1)?;
    projection.admit_row(&row)?;
    rows.push(super::SubjectRecord {
        resource_key: row.resource_key.to_string(),
        side: row.side.to_string(),
        subject_type: row.subject_type.to_string(),
        id: row.id.to_string(),
        sha256: row.sha256.to_string(),
    });
    Ok(())
}

/// Admit one individually input-bounded native metadata row before growing its retained array.
fn retain_metadata<T: serde::Serialize>(
    rows: &mut Vec<T>,
    row: T,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<(), ForgeError> {
    capture.relationships(1)?;
    projection.admit_row(&row)?;
    rows.push(row);
    Ok(())
}

/// Produce a complete fresh index using one actual capture session and existing native predicates.
pub(crate) fn prepare_fresh(
    manifest_relative: &Path,
    expected_manifest_sha256: &str,
    as_of: chrono::NaiveDate,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<PreparedFreshLinkage, ForgeError> {
    prepare_fresh_inner(manifest_relative, expected_manifest_sha256, as_of, capture, projection)
        .map_err(|_| {
            error("fresh linkage inputs are invalid, changed, unsafe or over their complete bounds")
        })
}

/// Keep the actual capture, native subject resolutions and full projection admission adjacent.
#[allow(clippy::too_many_lines)] // Keep shared capture and before-growth admission order adjacent.
fn prepare_fresh_inner(
    manifest_relative: &Path,
    expected_manifest_sha256: &str,
    as_of: chrono::NaiveDate,
    capture: &mut CaptureSession,
    projection: &mut ProjectionBudget,
) -> Result<PreparedFreshLinkage, ForgeError> {
    use super::{
        EvidenceFreshness, EvidenceRecord, EvidenceReference, LinkRecord, Provenance,
        ResourceEvidence, manifest,
    };
    use crate::hashing::sha256_hex;
    use crate::mapping::{inventory, manifest::SubjectType};
    use std::collections::{BTreeMap, BTreeSet};
    if expected_manifest_sha256.len() != 64
        || !expected_manifest_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(error("linkage manifest pin must be canonical full SHA-256"));
    }
    let manifest_lease = capture.required(
        manifest_relative,
        CaptureRole::LinkageManifest,
        manifest::MAX_MANIFEST_BYTES,
    )?;
    if sha256_hex(manifest_lease.bytes()) != expected_manifest_sha256 {
        return Err(error("actual linkage manifest does not match the declared pin"));
    }
    let manifest = manifest::parse(manifest_lease.bytes())?;
    if manifest.links.is_empty() {
        return Err(error("fresh linkage requires at least one reviewed link"));
    }
    let base = manifest_relative.parent().unwrap_or_else(|| Path::new(""));
    let absolute_base = capture.root().join(base);
    projection.admit_row(&(&manifest.project, &manifest.reviewers))?;
    let mut requirement_resources = BTreeMap::new();
    let mut requirement_evidence = Vec::new();
    let mut requirement_inventory = Vec::new();
    for resource in &manifest.requirement_resources {
        let role = match resource.resource_type {
            crate::mapping::manifest::ResourceType::Catalog => CaptureRole::Catalog,
            crate::mapping::manifest::ResourceType::Profile => CaptureRole::Profile,
        };
        let lease =
            capture.required(&base.join(&resource.artifact), role, crate::io::MAX_FILE_SIZE)?;
        let companion = resource
            .resolved_catalog
            .as_ref()
            .map(|path| {
                capture.required(&base.join(path), CaptureRole::Catalog, crate::io::MAX_FILE_SIZE)
            })
            .transpose()?;
        let loaded = inventory::load_captured(
            &absolute_base,
            "captured linkage requirement",
            &crate::mapping::manifest::ResourceManifest {
                resource_type: resource.resource_type,
                artifact: resource.artifact.clone(),
                href: resource.href.clone(),
                resolved_catalog: resource.resolved_catalog.clone(),
                resolved_catalog_attestation: resource.resolved_catalog_attestation,
                expected_sha256: Some(resource.expected_sha256.clone()),
                expected_resolved_catalog_sha256: resource.expected_resolved_catalog_sha256.clone(),
                inventory: None,
            },
            lease.bytes(),
            companion.as_ref().map(CaptureLease::bytes),
            capture,
        )?;
        for subject_type in [SubjectType::Control, SubjectType::Statement] {
            for id in loaded.inventory.ids_of_type_refs(subject_type) {
                let fingerprint = loaded
                    .inventory
                    .fingerprint(subject_type, id)
                    .ok_or_else(|| error("native requirement fingerprint disappeared"))?;
                retain_subject(
                    &mut requirement_inventory,
                    SubjectView {
                        resource_key: &resource.key,
                        side: "requirement",
                        subject_type: subject_type.as_str(),
                        id,
                        sha256: fingerprint,
                    },
                    capture,
                    projection,
                )?;
            }
        }
        retain_metadata(
            &mut requirement_evidence,
            ResourceEvidence {
                key: resource.key.clone(),
                resource_type: loaded.evidence.resource_type.as_str().to_string(),
                href: super::sanitize_reference_label(&loaded.evidence.href),
                raw_sha256: loaded.evidence.raw_sha256.clone(),
                root_uuid: loaded.evidence.root_uuid.clone(),
                document_version: loaded.evidence.document_version.clone(),
                oscal_version: loaded.evidence.oscal_version.clone(),
                resolved_catalog_sha256: loaded.evidence.resolved_catalog_sha256.clone(),
            },
            capture,
            projection,
        )?;
        capture.relationships(1)?;
        requirement_resources.insert(resource.key.clone(), loaded);
    }
    requirement_evidence.sort_by(|a, b| a.key.cmp(&b.key));
    requirement_inventory.sort();
    let resource = &manifest.implementation_resource;
    let role = match resource.resource_type {
        manifest::ImplementationResourceType::SystemSecurityPlan => CaptureRole::SystemSecurityPlan,
        manifest::ImplementationResourceType::ComponentDefinition => {
            CaptureRole::ComponentDefinition
        }
    };
    let implementation_lease =
        capture.required(&base.join(&resource.artifact), role, crate::io::MAX_FILE_SIZE)?;
    let implementation = load_implementation_captured(
        &absolute_base,
        resource,
        implementation_lease.bytes(),
        capture,
    )?;
    projection.admit_row(&implementation.evidence)?;
    let mut implementation_inventory = Vec::new();
    for ((kind, id), fingerprint) in &implementation.subjects {
        retain_subject(
            &mut implementation_inventory,
            SubjectView {
                resource_key: &resource.key,
                side: "implementation",
                subject_type: kind.as_str(),
                id,
                sha256: fingerprint,
            },
            capture,
            projection,
        )?;
    }
    implementation_inventory.sort();
    let mut roots = BTreeMap::new();
    let mut root_paths = BTreeSet::new();
    for root in &manifest.evidence_roots {
        let path = base.join(&root.path);
        if !root_paths.insert(path.clone()) {
            return Err(error("declared evidence directories alias"));
        }
        capture.directory(&path)?;
        projection.admit_row(root)?;
        roots.insert(root.key.as_str(), path);
    }
    let mut evidence = Vec::new();
    let mut references = BTreeMap::new();
    let mut local_paths = BTreeSet::new();
    for declaration in &manifest.evidence {
        let date = super::date_freshness(
            declaration.valid_through,
            as_of,
            manifest.project.expiring_window_days,
        )?;
        let (freshness, reference, held) = match &declaration.location {
            manifest::EvidenceLocation::Local {
                root_key,
                path,
                expected_sha256,
                expected_size,
            } => {
                let root = roots
                    .get(root_key.as_str())
                    .ok_or_else(|| error("validated evidence root disappeared"))?;
                let relative = root.join(path);
                let observation =
                    capture.optional_local(&relative, manifest.project.max_evidence_bytes)?;
                if matches!(&observation, LocalObservation::Present(_))
                    && !local_paths.insert(relative.clone())
                {
                    return Err(error("local evidence aliases another declared evidence file"));
                }
                let (observed_sha256, observed_size) = match &observation {
                    LocalObservation::Present(lease) => {
                        lease.verify_inputs()?;
                        (
                            Some(sha256_hex(lease.bytes())),
                            Some(
                                u64::try_from(lease.bytes().len())
                                    .map_err(|_| error("local size cannot be represented"))?,
                            ),
                        )
                    }
                    LocalObservation::Absent(_) => (None, None),
                };
                let freshness = if observed_sha256.is_none() {
                    EvidenceFreshness::Unavailable
                } else if observed_sha256.as_ref() != Some(expected_sha256)
                    || observed_size != Some(*expected_size)
                {
                    EvidenceFreshness::Changed
                } else {
                    date
                };
                (
                    freshness,
                    EvidenceReference::Local {
                        root_key: root_key.clone(),
                        relative_label: super::normalize_relative_label(path),
                        approved_sha256: expected_sha256.clone(),
                        approved_size: *expected_size,
                        observed_sha256,
                        observed_size,
                    },
                    HeldReference::Local { relative, observation },
                )
            }
            manifest::EvidenceLocation::Uri { uri, expected_sha256, .. } => (
                EvidenceFreshness::UnverifiedUri,
                EvidenceReference::Uri {
                    redacted_uri: super::validate_and_redact_uri(
                        uri,
                        &manifest.project.approved_uri_schemes,
                    )?,
                    expected_sha256: expected_sha256.clone(),
                },
                HeldReference::Uri,
            ),
        };
        retain_metadata(
            &mut evidence,
            EvidenceRecord {
                key: declaration.key.clone(),
                title: declaration.title.clone(),
                evidence_type: declaration.evidence_type.clone(),
                owner: declaration.owner.clone(),
                collected_at: declaration.collected_at.clone(),
                valid_through: declaration.valid_through,
                sensitivity_label: declaration.sensitivity_label.clone(),
                source_label: super::sanitize_reference_label(&declaration.source_label),
                freshness,
                reference,
            },
            capture,
            projection,
        )?;
        capture.relationships(1)?;
        references.insert(declaration.key.clone(), held);
    }
    evidence.sort_by(|a, b| a.key.cmp(&b.key));
    let evidence_by_key: BTreeMap<_, _> =
        evidence.iter().map(|record| (record.key.as_str(), record)).collect();
    let mut links = Vec::new();
    let mut findings = Vec::new();
    for link in &manifest.links {
        let mut requirements = Vec::new();
        let mut implementations = Vec::new();
        for subject in &link.requirements {
            let loaded = requirement_resources
                .get(&subject.resource_key)
                .ok_or_else(|| error("validated requirement disappeared"))?;
            let kind = match subject.subject_type {
                manifest::RequirementSubjectType::Control => SubjectType::Control,
                manifest::RequirementSubjectType::Statement => SubjectType::Statement,
            };
            let fingerprint = super::resolve_requirement_subject(loaded, kind, &subject.id_ref)?;
            retain_subject(
                &mut requirements,
                SubjectView {
                    resource_key: &subject.resource_key,
                    side: "requirement",
                    subject_type: subject.subject_type.as_str(),
                    id: &subject.id_ref,
                    sha256: fingerprint,
                },
                capture,
                projection,
            )?;
        }
        for subject in &link.implementations {
            let fingerprint = super::resolve_implementation_subject(
                &implementation,
                subject.subject_type,
                &subject.id_ref,
            )?;
            retain_subject(
                &mut implementations,
                SubjectView {
                    resource_key: &resource.key,
                    side: "implementation",
                    subject_type: subject.subject_type.as_str(),
                    id: &subject.id_ref,
                    sha256: fingerprint,
                },
                capture,
                projection,
            )?;
        }
        requirements.sort();
        implementations.sort();
        capture.relationships(link.evidence_keys.len())?;
        projection.admit_row(&link.evidence_keys)?;
        let mut evidence_keys = link.evidence_keys.clone();
        evidence_keys.sort();
        if evidence_keys.is_empty() {
            retain_metadata(
                &mut findings,
                super::finding(
                    &manifest.project.key,
                    "evidence-missing",
                    Some(&link.key),
                    None,
                    None,
                    None,
                    link.evidence_required,
                    "The reviewed link has no evidence reference.",
                ),
                capture,
                projection,
            )?;
        }
        for key in &evidence_keys {
            let record = evidence_by_key
                .get(key.as_str())
                .ok_or_else(|| error("validated evidence disappeared"))?;
            let mut states = vec![record.freshness.clone()];
            if !matches!(
                record.freshness,
                EvidenceFreshness::Current
                    | EvidenceFreshness::Expiring
                    | EvidenceFreshness::Expired
            ) {
                let date = super::date_freshness(
                    record.valid_through,
                    as_of,
                    manifest.project.expiring_window_days,
                )?;
                if date != EvidenceFreshness::Current {
                    states.push(date);
                }
            }
            for state in states {
                if let Some((reason, message)) = super::freshness_finding(&state) {
                    retain_metadata(
                        &mut findings,
                        super::finding(
                            &manifest.project.key,
                            reason,
                            Some(&link.key),
                            Some(key),
                            None,
                            Some(&record.owner),
                            link.evidence_required && state != EvidenceFreshness::UnverifiedUri,
                            message,
                        ),
                        capture,
                        projection,
                    )?;
                }
            }
        }
        projection.admit_row(link)?;
        retain_metadata(
            &mut links,
            LinkRecord {
                link_id: uuid::Uuid::new_v5(
                    &super::namespace(super::LINK_NAMESPACE_SEED),
                    &super::stable_bytes(&[&manifest.project.key, &link.key]),
                )
                .to_string(),
                key: link.key.clone(),
                requirements,
                implementations,
                evidence_keys,
                evidence_required: link.evidence_required,
                responsible_role: link.responsible_role.clone(),
                implementation_status: link.implementation_status,
                reviewer_key: link.review.reviewer_key.clone(),
                reviewed_at: link.review.reviewed_at.clone(),
                rationale: link.review.rationale.clone(),
                not_applicable_review: link.not_applicable_review.clone(),
                impact_finding_ids: super::sorted(link.impact_finding_ids.clone()),
                policy_version_keys: super::sorted(link.policy_version_keys.clone()),
            },
            capture,
            projection,
        )?;
    }
    links.sort_by(|a, b| a.key.cmp(&b.key));
    let mut linked_implementation_subjects = BTreeSet::new();
    for link in &links {
        for subject in &link.implementations {
            capture.relationships(1)?;
            projection.admit_row(subject)?;
            linked_implementation_subjects.insert(super::subject_identity(subject));
        }
    }
    for subject in &implementation_inventory {
        let identity = super::subject_identity(subject);
        if !linked_implementation_subjects.contains(&identity) {
            retain_metadata(
                &mut findings,
                super::finding(
                    &manifest.project.key,
                    "implementation-subject-unlinked",
                    None,
                    None,
                    Some(&identity),
                    None,
                    true,
                    "An implementation subject in the exact artifact inventory is not covered by a reviewed link.",
                ),
                capture,
                projection,
            )?;
        }
    }
    findings.sort();
    findings.dedup();
    let index = super::LinkageIndex {
        schema_version: super::INDEX_SCHEMA_VERSION.to_string(),
        project_key: manifest.project.key.clone(),
        project_title: manifest.project.title.clone(),
        as_of,
        provenance: Provenance {
            manifest_sha256: expected_manifest_sha256.to_string(),
            requirement_resources: requirement_evidence,
            implementation_resource: implementation.evidence,
        },
        requirement_inventory,
        implementation_inventory,
        evidence,
        links,
        findings,
        trust_boundary: super::TRUST_BOUNDARY.to_string(),
    };
    // The complete bounded writer also checks object scaffolding/pretty whitespace; no encoded prefix escapes.
    let _complete = ProjectionBudget::encode(&index)?;
    manifest_lease.verify_inputs()?;
    Ok(PreparedFreshLinkage { index, references })
}

/// Admit every complete native implementation node before the legacy inventory clones its identities.
fn admit_implementation_nodes(
    root: &serde_json::Map<String, serde_json::Value>,
    kind: super::manifest::ImplementationResourceType,
    capture: &mut CaptureSession,
) -> Result<(), ForgeError> {
    use serde_json::Value;
    match kind {
        super::manifest::ImplementationResourceType::ComponentDefinition => {
            for name in ["components", "capabilities"] {
                if let Some(containers) = root.get(name).and_then(Value::as_array) {
                    for container in containers {
                        capture.relationships(1)?;
                        if let Some(controls) =
                            container.get("control-implementations").and_then(Value::as_array)
                        {
                            for control in controls {
                                capture.relationships(1)?;
                                admit_implementation_requirements(
                                    control.get("implemented-requirements"),
                                    capture,
                                )?;
                            }
                        }
                    }
                }
            }
        }
        super::manifest::ImplementationResourceType::SystemSecurityPlan => {
            admit_implementation_requirements(
                root.get("control-implementation").and_then(|v| v.get("implemented-requirements")),
                capture,
            )?;
        }
    }
    Ok(())
}

/// Count all native requirement and statement rows, never a filtered first prefix.
fn admit_implementation_requirements(
    value: Option<&serde_json::Value>,
    capture: &mut CaptureSession,
) -> Result<(), ForgeError> {
    use serde_json::Value;
    if let Some(requirements) = value.and_then(Value::as_array) {
        for requirement in requirements {
            capture.relationships(1)?;
            if let Some(statements) = requirement.get("statements").and_then(Value::as_array) {
                capture.relationships(statements.len())?;
            }
        }
    }
    Ok(())
}

/// Validate native implementation bytes with original predicates after shared before-growth admission.
fn load_implementation_captured(
    manifest_dir: &Path,
    resource: &manifest::ImplementationResourceManifest,
    bytes: &[u8],
    capture: &mut CaptureSession,
) -> Result<super::ImplementationInventory, ForgeError> {
    let path = manifest_dir.join(&resource.artifact);
    if bytes.len() as u64 > io::MAX_FILE_SIZE {
        return Err(super::error("implementation artifact exceeds its existing bound"));
    }
    let raw_sha256 = sha256_hex(bytes);
    if raw_sha256 != resource.expected_sha256 {
        return Err(super::error(format!(
            "$.implementation_resource.expected_sha256 mismatch: expected {}, got {raw_sha256}",
            resource.expected_sha256
        )));
    }
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|cause| super::error(format!("implementation artifact is not JSON: {cause}")))?;
    let model = match resource.resource_type {
        manifest::ImplementationResourceType::ComponentDefinition => {
            crate::OscalModelType::ComponentDefinition
        }
        manifest::ImplementationResourceType::SystemSecurityPlan => {
            crate::OscalModelType::SystemSecurityPlan
        }
    };
    let detected = crate::validate::detect_model_type(&value)
        .map_err(|cause| super::error(format!("implementation artifact: {cause}")))?;
    if detected != model {
        return Err(super::error(format!(
            "$.implementation_resource.type declares '{}' but artifact root is '{}'",
            resource.resource_type.as_str(),
            detected.as_str()
        )));
    }
    inventory::validate_schema("implementation artifact", &value, model)
        .map_err(relabel_mapping_error)?;
    let root = value
        .get(model.as_str())
        .and_then(Value::as_object)
        .ok_or_else(|| super::error("implementation artifact root is missing"))?;
    let root_uuid = required_string(root.get("uuid"), "implementation root uuid")?;
    Uuid::parse_str(&root_uuid)
        .map_err(|_| super::error("implementation root uuid must be a UUID"))?;
    let metadata = root
        .get("metadata")
        .and_then(Value::as_object)
        .ok_or_else(|| super::error("implementation metadata is required"))?;
    let document_version = required_string(metadata.get("version"), "implementation version")?;
    let oscal_version =
        required_string(metadata.get("oscal-version"), "implementation OSCAL version")?;
    admit_implementation_nodes(root, resource.resource_type, capture)?;
    let mut inventory = super::ImplementationInventory {
        evidence: ResourceEvidence {
            key: resource.key.clone(),
            resource_type: resource.resource_type.as_str().to_string(),
            href: sanitize_reference_label(&resource.href),
            raw_sha256,
            root_uuid,
            document_version,
            oscal_version,
            resolved_catalog_sha256: None,
        },
        subjects: BTreeMap::new(),
        id_types: BTreeMap::new(),
        path,
    };
    match resource.resource_type {
        manifest::ImplementationResourceType::ComponentDefinition => {
            for container_name in ["components", "capabilities"] {
                if let Some(containers) = root.get(container_name).and_then(Value::as_array) {
                    for container in containers {
                        super::inventory_control_implementations(
                            container.get("control-implementations"),
                            &mut inventory,
                        )?;
                    }
                }
            }
        }
        manifest::ImplementationResourceType::SystemSecurityPlan => {
            let control_implementation = root.get("control-implementation");
            super::inventory_implemented_requirements(
                control_implementation.and_then(|value| value.get("implemented-requirements")),
                &mut inventory,
            )?;
        }
    }
    Ok(inventory)
}

/// Proposed source controls for actual capture/native projection; no execution is implied.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hashing::sha256_hex;
    use serde_json::{Value, json};

    /// Native source fixture plus actual original local evidence and closed linkage declaration.
    struct Fixture {
        /// Own the actual private filesystem fixture for the complete proof lifetime.
        dir: tempfile::TempDir,
        /// Closed linkage manifest stored beneath the qualified fixture root.
        manifest: PathBuf,
        /// Actual local evidence original used by native freshness checks.
        evidence: PathBuf,
    }

    /// Hash owned synthetic fixture bytes for explicit author pins, not a supplied proof constructor.
    fn fixture_hash(path: &Path) -> String {
        sha256_hex(&std::fs::read(path).unwrap())
    }

    /// Serialize one complete owned synthetic fixture without invoking the product.
    fn write_json(path: &Path, value: &Value) {
        std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
    }
    /// Use the unchanged native Catalog fixture already consumed by linkage CLI tests.
    fn native_catalog() -> Value {
        json!({
            "catalog": {
                "uuid": "11111111-1111-4111-8111-111111111111",
                "metadata": {
                    "title": "Synthetic requirement catalog",
                    "last-modified": "2026-08-27T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "controls": [{
                    "id": "ac-1",
                    "title": "Synthetic access requirement",
                    "parts": [{
                        "id": "ac-1_smt",
                        "name": "statement",
                        "prose": "Synthetic statement content."
                    }]
                }]
            }
        })
    }

    /// Use the unchanged native Component Definition fixture with all native implementation subjects.
    fn native_component() -> Value {
        json!({
            "component-definition": {
                "uuid": "22222222-2222-4222-8222-222222222222",
                "metadata": {
                    "title": "Synthetic implementation",
                    "last-modified": "2026-08-27T00:00:00Z",
                    "version": "1.0.0",
                    "oscal-version": "1.2.3"
                },
                "components": [{
                    "uuid": "33333333-3333-4333-8333-333333333333",
                    "type": "software",
                    "title": "Synthetic component",
                    "description": "Synthetic component used only for tests.",
                    "control-implementations": [{
                        "uuid": "44444444-4444-4444-8444-444444444444",
                        "source": "catalog.json",
                        "description": "Reviewed implementation set.",
                        "implemented-requirements": [{
                            "uuid": "55555555-5555-4555-8555-555555555555",
                            "control-id": "ac-1",
                            "description": "Reviewer-authored implementation statement.",
                            "statements": [{
                                "statement-id": "ac-1_smt",
                                "uuid": "66666666-6666-4666-8666-666666666666",
                                "description": "Statement-level implementation."
                            }]
                        }]
                    }]
                }]
            }
        })
    }

    /// Create the existing reviewed native linkage fixture without invoking CLI execution.
    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().expect("tempdir");
        let catalog_path = dir.path().join("catalog.json");
        let implementation_path = dir.path().join("component.json");
        let evidence_dir = dir.path().join("evidence");
        let evidence = evidence_dir.join("record.bin");
        std::fs::create_dir(&evidence_dir).expect("evidence dir");
        write_json(&catalog_path, &native_catalog());
        write_json(&implementation_path, &native_component());
        std::fs::write(&evidence, b"private evidence bytes\n").expect("evidence file");
        let manifest = dir.path().join("linkage.json");
        write_json(
            &manifest,
            &json!({
                "schema_version": "forge.linkage/1",
                "project": {
                    "key": "synthetic-project",
                    "title": "Synthetic linkage project",
                    "expiring_window_days": 30,
                    "max_evidence_bytes": 1_048_576,
                    "approved_uri_schemes": ["vault+corp"]
                },
                "reviewers": [{"key": "reviewer", "name": "Test Reviewer"}],
                "requirement_resources": [{
                    "key": "requirements",
                    "type": "catalog",
                    "artifact": "catalog.json",
                    "href": "catalog.json",
                    "expected_sha256": fixture_hash(&catalog_path)
                }],
                "implementation_resource": {
                    "key": "implementation",
                    "type": "component-definition",
                    "artifact": "component.json",
                    "href": "component.json",
                    "expected_sha256": fixture_hash(&implementation_path)
                },
                "evidence_roots": [{"key": "local", "path": "evidence"}],
                "evidence": [{
                    "key": "record",
                    "title": "Synthetic record",
                    "evidence_type": "test-record",
                    "owner": "control-owner",
                    "collected_at": "2026-08-20T00:00:00Z",
                    "valid_through": "2026-12-31",
                    "sensitivity_label": "restricted",
                    "source_label": "reviewed local export",
                    "location": {
                        "kind": "local",
                        "root_key": "local",
                        "path": "record.bin",
                        "expected_sha256": fixture_hash(&evidence),
                        "expected_size": std::fs::metadata(&evidence).unwrap().len()
                    }
                }],
                "links": [{
                    "key": "access-link",
                    "requirements": [
                        {"resource_key": "requirements", "type": "control", "id_ref": "ac-1"},
                        {"resource_key": "requirements", "type": "statement", "id_ref": "ac-1_smt"}
                    ],
                    "implementations": [
                        {"type": "implemented-requirement", "id_ref": "55555555-5555-4555-8555-555555555555"},
                        {"type": "statement", "id_ref": "66666666-6666-4666-8666-666666666666"}
                    ],
                    "evidence_keys": ["record"],
                    "evidence_required": true,
                    "responsible_role": "control-owner",
                    "implementation_status": "implemented",
                    "review": {
                        "reviewer_key": "reviewer",
                        "reviewed_at": "2026-08-21T00:00:00Z",
                        "rationale": "Reviewer associated these exact subjects and evidence metadata."
                    },
                    "impact_finding_ids": ["framework-impact-1"],
                    "policy_version_keys": ["policy-v1"]
                }]
            }),
        );
        Fixture { dir, manifest, evidence }
    }

    /// Prepare actual complete bytes under one qualified root and explicit test date.
    fn prepared(
        fixture: &Fixture,
    ) -> (PreparedFreshLinkage, crate::evidence_capture::CaptureProof) {
        let root = fixture.dir.path().canonicalize().unwrap();
        let mut capture = CaptureSession::new(&root).unwrap();
        let mut projection = ProjectionBudget::new();
        let result = prepare_fresh(
            Path::new("linkage.json"),
            &fixture_hash(&fixture.manifest),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            &mut capture,
            &mut projection,
        )
        .unwrap();
        (result, capture.finish())
    }

    /// Actual new captured producer preserves all native subject fingerprints and complete old index semantics.
    #[test]
    fn fresh_index_matches_existing_native_projection_from_actual_files() {
        let fixture = fixture();
        let old = super::super::prepare(
            &fixture.manifest,
            chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            None,
        )
        .unwrap();
        let (fresh, proof) = prepared(&fixture);
        assert_eq!(fresh.index(), &old.index);
        assert_eq!(fresh.index().requirement_inventory.len(), 2);
        assert_eq!(fresh.index().implementation_inventory.len(), 2);
        assert!(proof.verify_inputs().is_ok());
        let observation = fresh
            .local_binding(
                "record",
                Path::new("evidence/record.bin"),
                &fixture_hash(&fixture.evidence),
            )
            .unwrap();
        assert!(matches!(observation.status, LocalBindingStatus::Matched));
        assert!(observation.local_bytes_revalidated);
        assert_eq!(proof.captured_original_generations(), 4);
    }

    /// Present wrong href/hash assertions preserve actual observed facts without acquiring a matched relation.
    #[test]
    fn path_and_hash_mismatches_preserve_actual_current_facts() {
        let fixture = fixture();
        let (fresh, proof) = prepared(&fixture);
        let expected = fixture_hash(&fixture.evidence);
        let wrong_path = fresh.local_binding("record", Path::new("record.bin"), &expected).unwrap();
        assert!(matches!(wrong_path.status, LocalBindingStatus::HrefMismatch));
        assert_eq!(wrong_path.observed_sha256.as_deref(), Some(expected.as_str()));
        assert_eq!(wrong_path.observed_size, Some(23));
        assert!(wrong_path.local_bytes_revalidated);
        let wrong_hash = fresh
            .local_binding("record", Path::new("evidence/record.bin"), &"0".repeat(64))
            .unwrap();
        assert!(matches!(wrong_hash.status, LocalBindingStatus::HashMismatch));
        assert_eq!(wrong_hash.observed_sha256, wrong_path.observed_sha256);
        assert!(proof.verify_inputs().is_ok());
    }

    /// Recorded approved and actual current bytes remain separate when evidence changed before capture.
    #[test]
    fn changed_full_bytes_do_not_match_an_old_author_assertion() {
        let fixture = fixture();
        let approved = fixture_hash(&fixture.evidence);
        std::fs::write(&fixture.evidence, b"changed evidence bytes!").unwrap();
        let observed = fixture_hash(&fixture.evidence);
        let (fresh, proof) = prepared(&fixture);
        assert!(matches!(
            fresh.index().evidence[0].freshness,
            super::super::EvidenceFreshness::Changed
        ));
        let relation =
            fresh.local_binding("record", Path::new("evidence/record.bin"), &approved).unwrap();
        assert!(matches!(relation.status, LocalBindingStatus::HashMismatch));
        assert_eq!(relation.observed_sha256.as_deref(), Some(observed.as_str()));
        assert!(relation.local_bytes_revalidated);
        assert!(proof.verify_inputs().is_ok());
    }

    /// Missing actual evidence has null observations and later appearance invalidates its retained absence.
    #[test]
    fn unavailable_local_is_not_zero_byte_evidence_or_future_immutability() {
        let fixture = fixture();
        let approved = fixture_hash(&fixture.evidence);
        std::fs::remove_file(&fixture.evidence).unwrap();
        let (fresh, proof) = prepared(&fixture);
        let relation =
            fresh.local_binding("record", Path::new("evidence/record.bin"), &approved).unwrap();
        assert!(matches!(relation.status, LocalBindingStatus::Unavailable));
        assert!(relation.observed_sha256.is_none());
        assert!(relation.observed_size.is_none());
        assert!(!relation.local_bytes_revalidated);
        assert_eq!(proof.captured_original_generations(), 3);
        assert!(proof.verify_inputs().is_ok());
        std::fs::write(&fixture.evidence, b"private evidence bytes\n").unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// URI metadata remains redacted and unverified and cannot satisfy a local-file assertion.
    #[test]
    fn uri_reference_is_never_fetched_or_labeled_local_approval() {
        let fixture = fixture();
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
        value["evidence"][0]["location"] = json!({"kind":"uri", "uri":"https://user:secret@example.com/evidence?token=private#section", "unverified":true, "expected_sha256":"0".repeat(64)});
        write_json(&fixture.manifest, &value);
        let (fresh, proof) = prepared(&fixture);
        let relation = fresh
            .local_binding("record", Path::new("evidence/record.bin"), &"0".repeat(64))
            .unwrap();
        assert!(matches!(relation.status, LocalBindingStatus::ReferenceKindMismatch));
        assert!(relation.observed_sha256.is_none());
        assert!(!relation.local_bytes_revalidated);
        assert!(matches!(
            fresh.index().evidence[0].freshness,
            super::super::EvidenceFreshness::UnverifiedUri
        ));
        let encoded = serde_json::to_string(fresh.index()).unwrap();
        for private in ["secret", "token=private", "private evidence bytes"] {
            assert!(!encoded.contains(private));
        }
        assert_eq!(proof.captured_original_generations(), 3);
    }

    /// Every unused declared directory remains an actual dependency without becoming a file denominator.
    #[cfg(unix)]
    #[test]
    fn unused_declared_directory_removal_invalidates_complete_proof() {
        let fixture = fixture();
        let unused = fixture.dir.path().join("unused");
        std::fs::create_dir(&unused).unwrap();
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
        value["evidence_roots"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":"unused", "path":"unused"}));
        write_json(&fixture.manifest, &value);
        let (_, proof) = prepared(&fixture);
        assert_eq!(proof.captured_original_generations(), 4);
        assert!(proof.input_paths().any(|path| path == unused.canonicalize().unwrap()));
        std::fs::remove_dir(unused).unwrap();
        assert!(proof.verify_inputs().is_err());
    }

    /// Exact compatible Catalog roles share one allocation while native source pins still validate independently.
    #[test]
    fn compatible_native_source_and_requirement_share_only_actual_original_bytes() {
        let fixture = fixture();
        let root = fixture.dir.path().canonicalize().unwrap();
        let mut capture = CaptureSession::new(&root).unwrap();
        let source = capture
            .required(Path::new("catalog.json"), CaptureRole::Catalog, crate::io::MAX_FILE_SIZE)
            .unwrap();
        let mut projection = ProjectionBudget::new();
        let fresh = prepare_fresh(
            Path::new("linkage.json"),
            &fixture_hash(&fixture.manifest),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            &mut capture,
            &mut projection,
        )
        .unwrap();
        assert_eq!(
            fresh.index().provenance.requirement_resources[0].raw_sha256,
            sha256_hex(source.bytes())
        );
        assert_eq!(capture.finish().captured_original_generations(), 4);
    }

    /// Wrong native subject kind refuses the captured index before a successful inspection is returned.
    #[test]
    fn native_subject_kind_and_full_manifest_pin_are_not_replaced_by_role_labels() {
        let fixture = fixture();
        let root = fixture.dir.path().canonicalize().unwrap();
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
        value["links"][0]["requirements"][0]["id_ref"] = json!("ac-1_smt");
        write_json(&fixture.manifest, &value);
        let mut capture = CaptureSession::new(&root).unwrap();
        let mut projection = ProjectionBudget::new();
        assert!(
            prepare_fresh(
                Path::new("linkage.json"),
                &fixture_hash(&fixture.manifest),
                chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
                &mut capture,
                &mut projection
            )
            .is_err()
        );
        let mut capture = CaptureSession::new(&root).unwrap();
        let mut projection = ProjectionBudget::new();
        assert!(
            prepare_fresh(
                Path::new("linkage.json"),
                &"0".repeat(64),
                chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
                &mut capture,
                &mut projection
            )
            .is_err()
        );
        assert_eq!(capture.finish().captured_original_generations(), 1);
    }

    /// Explicit date projection retains expired/expiring semantics without changing complete current byte joins.
    #[test]
    fn explicit_date_changes_only_date_freshness_not_local_original_proof() {
        let fixture = fixture();
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(&fixture.manifest).unwrap()).unwrap();
        value["evidence"][0]["valid_through"] = json!("2026-08-27");
        write_json(&fixture.manifest, &value);
        let (fresh, proof) = prepared(&fixture);
        assert!(matches!(
            fresh.index().evidence[0].freshness,
            super::super::EvidenceFreshness::Expired
        ));
        assert!(matches!(
            fresh
                .local_binding(
                    "record",
                    Path::new("evidence/record.bin"),
                    &fixture_hash(&fixture.evidence)
                )
                .unwrap()
                .status,
            LocalBindingStatus::Matched
        ));
        assert!(proof.verify_inputs().is_ok());
    }
}
