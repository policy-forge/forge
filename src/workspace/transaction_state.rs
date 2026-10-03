//! Trusted, bounded root-bound transaction state for the proposed source restore.
//!
//! State qualification precedes credentials
//! and the listener. Journal checksums detect corruption, not hostile same-user
//! forgery. The journal never comes from project content or a public state path.

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::super::contract::{Error, Result};
use super::Root;
use super::root_transaction::{JournalRecord, RootIdentity};

/// Shared retention ceiling, not a second private-state allowance.
const MAX_RETAINED_BYTES: usize = 20 * 1024 * 1024;
/// Accepted and retained terminal outcomes share this complete count ceiling.
const MAX_RECORDS: usize = 256;
/// Finished records may retire only after this elapsed wall-clock interval.
const FINISHED_SECONDS: u64 = 600;
/// Bounded journal metadata; no source byte vectors or receipt tokens are encoded.
const MAX_JOURNAL_BYTES: usize = 1024 * 1024;
/// Native journal name within the independently qualified root namespace.
const JOURNAL_NAME: &str = "transactions.json";
/// Exclusive replacement name; an unknown prior partial file blocks recovery.
#[cfg(unix)]
const NEXT_NAME: &str = "transactions.next";
/// Persistent fixed private lock identity, independent of journal replacement.
#[cfg(unix)]
const LOCK_NAME: &str = "workspace.lock";

/// Complete private root-bound records; normal API readers see sanitized outcomes.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalPayload {
    /// Fixed native state revision, independent of the public bundle profile.
    schema_version: String,
    /// Actual held-root binding; never a web-supplied project identity.
    root: RootIdentity,
    /// Full retained records, with no arbitrary directory discovery.
    records: Vec<JournalRecord>,
}

/// Checksum envelope for bounded corruption detection, not a signing authority.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalEnvelope {
    /// Exact canonical serde payload whose checksum is independently repeated.
    payload: JournalPayload,
    /// SHA256 of the exact serialized payload bytes.
    payload_sha256: String,
}

/// Live qualified journal owner; a poisoned or failed durability path blocks writes.
struct StateInner {
    /// Native container and lifetime-scoped exclusive state owner.
    storage: Box<dyn JournalStorage + Send>,
    /// Last completely verified durable records, never a partly merged update.
    payload: JournalPayload,
    /// A failed publication/durability/verification requires a fresh recovery attempt.
    blocked: bool,
    /// Exact last verified fixed journal generation, absent only before first acceptance.
    durable: Option<Vec<u8>>,
}

/// Trusted per-user, actual-root-bound durable state; public requests cannot construct it.
/// Its native constructor is deliberately separate from project resource parsing.
pub(crate) struct TransactionState {
    /// Fixed actual-root binding qualified during before-listener construction.
    root: RootIdentity,
    /// Serialization protects native journal replacement; Store locks remain outside.
    inner: Mutex<StateInner>,
}

impl TransactionState {
    /// Refuse memory-only status/admission after actual journal/container substitution.
    fn checked_inner(&self) -> Result<std::sync::MutexGuard<'_, StateInner>> {
        let mut inner = self.inner.lock().map_err(|_| recovery_required())?;
        if inner.blocked {
            return Err(recovery_required());
        }
        match inner.storage.read(JOURNAL_NAME) {
            Ok(raw) if raw.as_deref() == inner.durable.as_deref() => Ok(inner),
            _ => {
                inner.blocked = true;
                Err(recovery_required())
            }
        }
    }

    /// Qualify OS-selected private state, held no-link ancestry and native durability.
    /// Unsupported ACL/sync ports fail unavailable; no weaker path fallback exists.
    #[cfg(unix)]
    pub(crate) fn open_qualified(root: &Root) -> Result<Self> {
        let identity = root.restore_identity()?;
        let storage = NativeJournal::open(&identity)?;
        let durable = storage.read(JOURNAL_NAME)?;
        let payload = match durable.as_deref() {
            Some(raw) => decode(raw, &identity)?,
            None => JournalPayload {
                schema_version: "forge.workspace-restore-state/1".to_owned(),
                root: identity.clone(),
                records: Vec::new(),
            },
        };
        Ok(Self {
            root: identity,
            inner: Mutex::new(StateInner {
                storage: Box::new(storage),
                payload,
                blocked: false,
                durable,
            }),
        })
    }

    /// Reopen only known accepted root records; read-only launch never settles writes.
    /// Native transaction recovery uses the exact recorded identities, not a new preview.
    pub(crate) fn recover(&self, root: &Root, writable: bool) -> Result<()> {
        if root.restore_identity()? != self.root {
            return Err(recovery_required());
        }
        let records = self.records()?;
        for record in records {
            if record.unresolved() {
                if !writable {
                    return Err(recovery_required());
                }
                root.recover_restore(self, record)?;
            }
        }
        self.require_access()
    }

    /// Observe safe persisted facts under fresh same-root API2 read authority supplied
    /// by the runtime. No retained outcome is returned before acceptance or after
    /// expiry;404 does not prove no write or authorize resend.
    pub(crate) fn observe_outcome(&self, operation_id: &str) -> Result<serde_json::Value> {
        validate_operation_id(operation_id)?;
        let inner = self.checked_inner()?;
        if inner.blocked {
            return Err(recovery_required());
        }
        let record = inner
            .payload
            .records
            .iter()
            .find(|record| record.operation_id == operation_id)
            .ok_or_else(|| {
                Error::new("not-found", "The retained restore outcome was not found.", false)
            })?;
        let now = unix_seconds()?;
        if record.is_finished()
            && record.finished_at.is_some_and(|at| now.saturating_sub(at) >= FINISHED_SECONDS)
        {
            return Err(Error::new(
                "not-found",
                "The retained restore outcome was not found.",
                false,
            ));
        }
        record.safe_outcome()
    }

    /// Seed the same Root-owned shared retention pool after restart. Public wrapper/
    /// preview reservations are additionally charged; unresolved entries never expire.
    pub(crate) fn retained_charges(&self) -> Result<Vec<(String, usize, Option<u64>)>> {
        let inner = self.checked_inner()?;
        if inner.blocked {
            return Err(recovery_required());
        }
        let now = unix_seconds()?;
        Ok(inner
            .payload
            .records
            .iter()
            .filter(|record| {
                record.unresolved()
                    || record.finished_at.is_none_or(|at| now.saturating_sub(at) < FINISHED_SECONDS)
            })
            .map(|record| (record.operation_id.clone(), record.retained_bytes, record.finished_at))
            .collect())
    }

    /// Abstain from participating root IO while any accepted state is unresolved.
    pub(crate) fn require_access(&self) -> Result<()> {
        let inner = self.checked_inner()?;
        if inner.blocked || inner.payload.records.iter().any(JournalRecord::unresolved) {
            return Err(recovery_required());
        }
        Ok(())
    }

    /// Return a bounded complete clone of metadata-only records for native recovery.
    pub(super) fn records(&self) -> Result<Vec<JournalRecord>> {
        let inner = self.checked_inner()?;
        if inner.blocked {
            return Err(recovery_required());
        }
        Ok(inner.payload.records.clone())
    }

    /// Permit settlement only from the checked actual journal generation and exact
    /// accepted owner. A durable committed decision cannot authorize stale rollback;
    /// blocked owners require a fresh qualified reload before any project effects.
    #[cfg(unix)]
    pub(super) fn verify_settlement_owner(&self, record: &JournalRecord) -> Result<()> {
        let inner = self.checked_inner()?;
        let old = inner
            .payload
            .records
            .iter()
            .find(|old| old.operation_id == record.operation_id)
            .ok_or_else(recovery_required)?;
        old.validate_settlement_owner(record)
    }

    /// Durably install accepted intent before the runtime may publish202. Complete
    /// Store quota/receipt/nonce/deadline admission is a separately consumed authority.
    pub(super) fn accept(&self, record: JournalRecord) -> Result<()> {
        record.validate(&self.root)?;
        let mut inner = self.checked_inner()?;
        if inner.blocked || inner.payload.records.iter().any(JournalRecord::unresolved) {
            return Err(recovery_required());
        }
        let now = unix_seconds()?;
        let mut proposed = inner.payload.clone();
        proposed.records.retain(|old| {
            !old.is_finished()
                || old.finished_at.is_none_or(|at| now.saturating_sub(at) < FINISHED_SECONDS)
        });
        if proposed.records.iter().any(|old| old.operation_id == record.operation_id) {
            return Err(Error::new(
                "idempotency-key-conflict",
                "The restore identifier is already retained.",
                false,
            ));
        }
        if proposed.records.len() >= MAX_RECORDS {
            return Err(capacity());
        }
        proposed.records.push(record);
        replace(&mut inner, proposed)
    }

    /// Replace one exact accepted owner without changing its root, ID, request digest
    /// or nonce. All states are journaled before/after their corresponding native phase.
    pub(super) fn update(&self, record: &JournalRecord) -> Result<()> {
        record.validate(&self.root)?;
        let mut inner = self.checked_inner()?;
        if inner.blocked {
            return Err(recovery_required());
        }
        let mut proposed = inner.payload.clone();
        let old = proposed
            .records
            .iter_mut()
            .find(|old| old.operation_id == record.operation_id)
            .ok_or_else(recovery_required)?;
        if old.nonce != record.nonce
            || old.request_sha256 != record.request_sha256
            || old.exact_manifest_sha256 != record.exact_manifest_sha256
            || old.retained_bytes != record.retained_bytes
        {
            return Err(recovery_required());
        }
        old.validate_transition(record)?;
        *old = record.clone();
        replace(&mut inner, proposed)
    }
}

/// Validate every retained record and charge the complete bounded metadata before I/O.
fn encode(payload: &JournalPayload) -> Result<Vec<u8>> {
    if payload.schema_version != "forge.workspace-restore-state/1"
        || payload.records.len() > MAX_RECORDS
    {
        return Err(recovery_required());
    }
    let mut ids = std::collections::BTreeSet::new();
    let mut charge = 0_usize;
    for record in &payload.records {
        record.validate(&payload.root)?;
        if !ids.insert(&record.operation_id) {
            return Err(recovery_required());
        }
        charge = charge.checked_add(record.retained_bytes).ok_or_else(capacity)?;
    }
    if charge > MAX_RETAINED_BYTES {
        return Err(capacity());
    }
    let bytes = serde_json::to_vec(payload).map_err(|_| recovery_required())?;
    let envelope = JournalEnvelope {
        payload: payload.clone(),
        payload_sha256: crate::hashing::sha256_hex(&bytes),
    };
    let raw = serde_json::to_vec(&envelope).map_err(|_| recovery_required())?;
    if raw.len() > MAX_JOURNAL_BYTES || (!payload.records.is_empty() && raw.len() > charge) {
        return Err(capacity());
    }
    Ok(raw)
}

/// Read only the known bounded journal, reject duplicates/unknown fields, and repeat
/// root/hash/shape checks. Project-supplied JSON never reaches this authority path.
#[cfg(any(unix, test))]
fn decode(raw: &[u8], root: &RootIdentity) -> Result<JournalPayload> {
    if raw.len() > MAX_JOURNAL_BYTES {
        return Err(recovery_required());
    }
    let value = super::super::contract::parse(raw, MAX_JOURNAL_BYTES, 4096)
        .map_err(|_| recovery_required())?;
    let envelope: JournalEnvelope =
        serde_json::from_value(value).map_err(|_| recovery_required())?;
    if &envelope.payload.root != root {
        return Err(recovery_required());
    }
    let bytes = serde_json::to_vec(&envelope.payload).map_err(|_| recovery_required())?;
    if crate::hashing::sha256_hex(&bytes) != envelope.payload_sha256 {
        return Err(recovery_required());
    }
    let _ = encode(&envelope.payload)?;
    Ok(envelope.payload)
}

/// Install the whole replacement only after checked native write/sync/rename/verify;
/// an uncertain durability path blocks rather than admitting a partial memory merge.
fn replace(inner: &mut StateInner, proposed: JournalPayload) -> Result<()> {
    let raw = encode(&proposed)?;
    if let Err(error) = inner.storage.replace(&raw, inner.durable.as_deref()) {
        inner.blocked = true;
        return Err(error);
    }
    inner.durable = Some(raw);
    inner.payload = proposed;
    Ok(())
}

/// Validate a nonauthorizing outcome lookup handle before any journal query.
pub(super) fn validate_operation_id(value: &str) -> Result<()> {
    let suffix = value.strip_prefix("op_").ok_or_else(Error::invalid)?;
    if !(12..=80).contains(&suffix.len())
        || !suffix.bytes().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Observe a bounded wall-clock timestamp; backward time never expires a record early.
pub(super) fn unix_seconds() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| recovery_required())
}

/// Refuse complete capacity before acceptance or replacement, without eviction of recovery.
fn capacity() -> Error {
    Error::new("payload-too-large", "Complete source restore retention exceeds its bound.", false)
}

/// Preserve uncertainty instead of exposing raw I/O errors or private paths.
pub(super) fn recovery_required() -> Error {
    Error::new(
        "bundle-restore-recovery-required",
        "Source restore requires recovery before further project access.",
        false,
    )
}

/// Refuse unqualified native trust or durability rather than weakening the storage port.
pub(super) fn unavailable() -> Error {
    Error::new(
        "bundle-restore-unavailable",
        "A qualified source restore transaction is unavailable for this project.",
        false,
    )
}

/// Actual native journal container; unsupported ports cannot construct a live owner.
#[cfg(unix)]
struct NativeJournal {
    /// Held trusted root namespace and exclusive advisory state lock.
    #[cfg(unix)]
    directory: File,
    /// Exact no-follow0600 owner/single-link lock file held for this state lifetime.
    #[cfg(unix)]
    lock: File,
    /// Actual opened native ancestry, retained rather than re-derived from project JSON.
    #[cfg(unix)]
    ancestors: Vec<File>,
    /// OS-selected private spelling; None exists only in the internal test constructor.
    #[cfg(unix)]
    path: Option<PathBuf>,
}

#[allow(unsafe_code)] // Narrow held-descriptor native journal port, individually documented.
#[cfg(unix)]
impl NativeJournal {
    /// Select only OS-private defaults and qualify every existing ancestry component.
    fn open(root: &RootIdentity) -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd as _;
            let path = private_default()?.join(format!("root-{}", root.binding_sha256));
            let (directory, ancestors) = qualified_directory(&path)?;
            let lock = qualified_lock(&directory)?;
            // SAFETY: qualified live private lock file; exclusive nonblocking lifetime lock.
            if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
                return Err(unavailable());
            }
            directory.sync_all().map_err(|_| unavailable())?;
            let storage = Self { directory, lock, ancestors, path: Some(path) };
            storage.revalidate_container()?;
            // A crash-created, unrecorded replacement is not deleted by name alone.
            if storage.read(NEXT_NAME)?.is_some() {
                return Err(recovery_required());
            }
            Ok(storage)
        }
        #[cfg(not(unix))]
        {
            let _ = root;
            // Windows owner/DACL/directory durability has no guessed stdlib fallback.
            Err(unavailable())
        }
    }

    /// Repeat the selected held ancestry and exact fixed lock identity before state I/O.
    /// A substituted path/container cannot create a second uncoordinated journal owner.
    fn revalidate_container(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt as _;
            // SAFETY: geteuid has no pointer input or mutation.
            let uid = unsafe { libc::geteuid() };
            if let Some(path) = &self.path {
                let mut actual = File::open("/").map_err(|_| recovery_required())?;
                let mut offset = 0;
                for component in path.components() {
                    match component {
                        std::path::Component::RootDir => {}
                        std::path::Component::Normal(name) => {
                            actual = super::unix_open(&actual, name, true)
                                .map_err(|_| recovery_required())?;
                            offset += 1;
                        }
                        _ => return Err(recovery_required()),
                    }
                    let held = self
                        .ancestors
                        .get(offset)
                        .ok_or_else(recovery_required)?
                        .metadata()
                        .map_err(|_| recovery_required())?;
                    let observed = actual.metadata().map_err(|_| recovery_required())?;
                    if !observed.is_dir()
                        || ![0, uid].contains(&observed.uid())
                        || observed.mode() & 0o022 != 0
                        || (held.dev(), held.ino()) != (observed.dev(), observed.ino())
                    {
                        return Err(recovery_required());
                    }
                }
                if offset + 1 != self.ancestors.len() {
                    return Err(recovery_required());
                }
            } else {
                #[cfg(not(test))]
                return Err(unavailable());
            }
            let container = self.directory.metadata().map_err(|_| recovery_required())?;
            if !container.is_dir() || container.uid() != uid || container.mode() & 0o077 != 0 {
                return Err(recovery_required());
            }
            let reopened =
                super::unix_open(&self.directory, std::ffi::OsStr::new(LOCK_NAME), false)
                    .map_err(|_| recovery_required())?;
            let held = self.lock.metadata().map_err(|_| recovery_required())?;
            let actual = reopened.metadata().map_err(|_| recovery_required())?;
            if !actual.is_file()
                || actual.uid() != uid
                || actual.nlink() != 1
                || actual.mode() & 0o777 != 0o600
                || actual.len() != 0
                || (held.dev(), held.ino()) != (actual.dev(), actual.ino())
            {
                return Err(recovery_required());
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            Err(unavailable())
        }
    }

    /// Read a single fixed filename with exact UID/mode/link/size/held identity checks.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        #[cfg(unix)]
        {
            use std::io::Read as _;
            use std::os::unix::fs::MetadataExt as _;
            self.revalidate_container()?;
            let file = match super::unix_open(&self.directory, std::ffi::OsStr::new(name), false) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(_) => return Err(recovery_required()),
            };
            let before = file.metadata().map_err(|_| recovery_required())?;
            // SAFETY: geteuid has no pointer input or side effect.
            let uid = unsafe { libc::geteuid() };
            if !before.is_file()
                || before.uid() != uid
                || before.mode() & 0o077 != 0
                || before.nlink() != 1
                || before.len() > MAX_JOURNAL_BYTES as u64
            {
                return Err(recovery_required());
            }
            let mut raw = Vec::new();
            let mut reader = std::io::BufReader::new(file);
            reader
                .by_ref()
                .take((MAX_JOURNAL_BYTES + 1) as u64)
                .read_to_end(&mut raw)
                .map_err(|_| recovery_required())?;
            let after = reader.get_ref().metadata().map_err(|_| recovery_required())?;
            if raw.len() > MAX_JOURNAL_BYTES
                || raw.len() as u64 != before.len()
                || (
                    before.dev(),
                    before.ino(),
                    before.len(),
                    before.mode(),
                    before.uid(),
                    before.nlink(),
                    before.mtime(),
                    before.mtime_nsec(),
                    before.ctime(),
                    before.ctime_nsec(),
                ) != (
                    after.dev(),
                    after.ino(),
                    after.len(),
                    after.mode(),
                    after.uid(),
                    after.nlink(),
                    after.mtime(),
                    after.mtime_nsec(),
                    after.ctime(),
                    after.ctime_nsec(),
                )
            {
                return Err(recovery_required());
            }
            Ok(Some(raw))
        }
        #[cfg(not(unix))]
        {
            let _ = name;
            Err(unavailable())
        }
    }

    /// Replace a fixed journal through an exclusively created, checked file and
    /// checked directory sync. Unknown partial replacements are preserved on failure.
    fn replace(&self, raw: &[u8], expected: Option<&[u8]>) -> Result<()> {
        #[cfg(unix)]
        {
            use std::io::Write as _;
            use std::os::fd::{AsRawFd as _, FromRawFd as _};
            use std::os::unix::fs::MetadataExt as _;
            if self.read(JOURNAL_NAME)?.as_deref() != expected {
                return Err(recovery_required());
            }
            let next = std::ffi::CString::new(NEXT_NAME).map_err(|_| recovery_required())?;
            let dest = std::ffi::CString::new(JOURNAL_NAME).map_err(|_| recovery_required())?;
            // SAFETY: held directory, fixed NUL-terminated name, exclusive/no-follow flags.
            let descriptor = unsafe {
                libc::openat(
                    self.directory.as_raw_fd(),
                    next.as_ptr(),
                    libc::O_WRONLY
                        | libc::O_CREAT
                        | libc::O_EXCL
                        | libc::O_NOFOLLOW
                        | libc::O_CLOEXEC,
                    0o600,
                )
            };
            if descriptor < 0 {
                return Err(recovery_required());
            }
            // SAFETY: fresh successful descriptor has exactly this owning File.
            let mut file = unsafe { File::from_raw_fd(descriptor) };
            file.write_all(raw).and_then(|()| file.sync_all()).map_err(|_| recovery_required())?;
            let meta = file.metadata().map_err(|_| recovery_required())?;
            if !meta.is_file() || meta.nlink() != 1 || meta.len() != raw.len() as u64 {
                return Err(recovery_required());
            }
            // SAFETY: both fixed names and held directory descriptor remain live.
            if unsafe {
                libc::renameat(
                    self.directory.as_raw_fd(),
                    next.as_ptr(),
                    self.directory.as_raw_fd(),
                    dest.as_ptr(),
                )
            } != 0
            {
                return Err(recovery_required());
            }
            self.directory.sync_all().map_err(|_| recovery_required())?;
            if self.read(JOURNAL_NAME)?.as_deref() != Some(raw) {
                return Err(recovery_required());
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (raw, expected);
            Err(unavailable())
        }
    }
}

/// Choose OS-private state deterministically; an invalid explicit setting is refused.
#[cfg(unix)]
fn private_default() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let base = match std::env::var_os("XDG_STATE_HOME") {
            Some(value) => {
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err(unavailable());
                }
                path
            }
            None => {
                let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(unavailable)?;
                if !home.is_absolute() {
                    return Err(unavailable());
                }
                home.join(".local/state")
            }
        };
        Ok(base.join("forge/source-restores"))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(unavailable)?;
        if !home.is_absolute() {
            return Err(unavailable());
        }
        Ok(home.join("Library/Application Support/Forge/source-restores"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(unavailable())
    }
}

/// Hold each no-follow ancestry component; create only owned private missing
/// directories with exclusive mkdir and checked identity/mode/sync before use.
#[cfg(unix)]
#[allow(unsafe_code)] // No-follow component open/mkdir against a held trusted parent.
fn qualified_directory(path: &Path) -> Result<(File, Vec<File>)> {
    use std::os::unix::{ffi::OsStrExt as _, fs::MetadataExt as _};
    // SAFETY: geteuid has no pointer input or side effect.
    let uid = unsafe { libc::geteuid() };
    let mut directory = File::open("/").map_err(|_| unavailable())?;
    let mut ancestors = vec![directory.try_clone().map_err(|_| unavailable())?];
    let components: Vec<_> = path.components().collect();
    if components.len() > 64 || path.as_os_str().as_bytes().len() > 4096 {
        return Err(unavailable());
    }
    for (offset, component) in components.iter().enumerate() {
        let name = match component {
            std::path::Component::RootDir => continue,
            std::path::Component::Normal(name) => *name,
            _ => return Err(unavailable()),
        };
        let parent = directory.metadata().map_err(|_| unavailable())?;
        if !parent.is_dir() || ![0, uid].contains(&parent.uid()) || parent.mode() & 0o022 != 0 {
            return Err(unavailable());
        }
        directory = match super::unix_open(&directory, name, true) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                create_private_component(&directory, name)?
            }
            Err(_) => return Err(unavailable()),
        };
        let meta = directory.metadata().map_err(|_| unavailable())?;
        if !meta.is_dir() || ![0, uid].contains(&meta.uid()) || meta.mode() & 0o022 != 0 {
            return Err(unavailable());
        }
        if offset + 1 == components.len() && (meta.uid() != uid || meta.mode() & 0o077 != 0) {
            return Err(unavailable());
        }
        ancestors.push(directory.try_clone().map_err(|_| unavailable())?);
    }
    Ok((directory, ancestors))
}

/// Create beneath a held qualified parent, or reopen a concurrent mkdir winner.
/// EEXIST alone permits a no-follow directory reopen; callers still check full
/// owner/mode/identity ancestry and final private-container qualification.
#[cfg(unix)]
#[allow(unsafe_code)] // One exclusive mkdir against the already held trusted parent.
fn create_private_component(parent: &File, name: &std::ffi::OsStr) -> Result<File> {
    use std::os::{fd::AsRawFd as _, unix::ffi::OsStrExt as _};
    let leaf = std::ffi::CString::new(name.as_bytes()).map_err(|_| unavailable())?;
    // SAFETY: held trusted parent and one bounded NUL-terminated path component.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), leaf.as_ptr(), 0o700) } != 0
        && std::io::Error::last_os_error().raw_os_error() != Some(libc::EEXIST)
    {
        return Err(unavailable());
    }
    parent.sync_all().map_err(|_| unavailable())?;
    let file = super::unix_open(parent, name, true).map_err(|_| unavailable())?;
    file.sync_all().map_err(|_| unavailable())?;
    Ok(file)
}

/// Open or exclusively create the fixed lock file and hold its exact owner/mode/ID.
/// Cross-process serialization covers only cooperating qualified workspace launches.
#[cfg(unix)]
#[allow(unsafe_code)] // exclusive fixed-name creation and one descriptor transfer.
fn qualified_lock(directory: &File) -> Result<File> {
    use std::os::{
        fd::{AsRawFd as _, FromRawFd as _},
        unix::fs::MetadataExt as _,
    };
    let name = std::ffi::CString::new(LOCK_NAME).map_err(|_| unavailable())?;
    // SAFETY: fixed relative name, held trusted parent, no-follow exclusive creation.
    let descriptor = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    let file = if descriptor >= 0 {
        // SAFETY: exactly one File owns this newly returned descriptor.
        let file = unsafe { File::from_raw_fd(descriptor) };
        file.sync_all().and_then(|()| directory.sync_all()).map_err(|_| unavailable())?;
        file
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EEXIST) {
            return Err(unavailable());
        }
        super::unix_open(directory, std::ffi::OsStr::new(LOCK_NAME), false)
            .map_err(|_| unavailable())?
    };
    // SAFETY: geteuid has no pointer input or mutation.
    let uid = unsafe { libc::geteuid() };
    let meta = file.metadata().map_err(|_| unavailable())?;
    if !meta.is_file()
        || meta.uid() != uid
        || meta.nlink() != 1
        || meta.mode() & 0o777 != 0o600
        || meta.len() != 0
    {
        return Err(unavailable());
    }
    let reopened = super::unix_open(directory, std::ffi::OsStr::new(LOCK_NAME), false)
        .map_err(|_| unavailable())?;
    let check = reopened.metadata().map_err(|_| unavailable())?;
    if (meta.dev(), meta.ino(), meta.mode(), meta.uid(), meta.nlink(), meta.len())
        != (check.dev(), check.ino(), check.mode(), check.uid(), check.nlink(), check.len())
    {
        return Err(unavailable());
    }
    Ok(file)
}

/// Narrow storage boundary used by checked replacement and deterministic fault controls.
/// Production constructs only a qualified native journal; tests cannot expose a web path.
trait JournalStorage {
    /// Observe the known fixed file generation without scanning arbitrary state entries.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>>;
    /// Conditionally replace exactly the previously observed durable journal generation.
    fn replace(&self, raw: &[u8], expected: Option<&[u8]>) -> Result<()>;
}

#[cfg(unix)]
impl JournalStorage for NativeJournal {
    /// Preserve the production bounded no-follow reader through the storage boundary.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        NativeJournal::read(self, name)
    }
    /// Preserve all native owner/durability/identity checks under fault-free production.
    fn replace(&self, raw: &[u8], expected: Option<&[u8]>) -> Result<()> {
        NativeJournal::replace(self, raw, expected)
    }
}

/// Internal test-only constructor taking an already owned private directory descriptor.
/// There is no production state-path override or relaxation of journal/lock predicates.
#[cfg(all(test, unix))]
pub(super) fn owned_test_state(root: &Root, directory: File) -> Result<TransactionState> {
    use std::os::fd::AsRawFd as _;
    use std::os::unix::fs::MetadataExt as _;
    let meta = directory.metadata().map_err(|_| unavailable())?;
    // SAFETY: observed process UID without pointers or mutation.
    #[allow(unsafe_code)]
    let uid = unsafe { libc::geteuid() };
    if !meta.is_dir() || meta.uid() != uid || meta.mode() & 0o077 != 0 {
        return Err(unavailable());
    }
    let lock = qualified_lock(&directory)?;
    // SAFETY: owned qualified private fixed lock descriptor.
    #[allow(unsafe_code)]
    if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(unavailable());
    }
    directory.sync_all().map_err(|_| unavailable())?;
    let identity = root.restore_identity()?;
    let ancestors = vec![directory.try_clone().map_err(|_| unavailable())?];
    let storage = NativeJournal { directory, lock, ancestors, path: None };
    if storage.read(NEXT_NAME)?.is_some() {
        return Err(recovery_required());
    }
    let durable = storage.read(JOURNAL_NAME)?;
    let payload = match durable.as_deref() {
        Some(raw) => decode(raw, &identity)?,
        None => JournalPayload {
            schema_version: "forge.workspace-restore-state/1".into(),
            root: identity.clone(),
            records: Vec::new(),
        },
    };
    Ok(TransactionState {
        root: identity,
        inner: Mutex::new(StateInner {
            storage: Box::new(storage),
            payload,
            blocked: false,
            durable,
        }),
    })
}

/// Test-only wrapper that performs the actual native replacement before reporting
/// a `CommitDecided` verification failure. No production branch injects this fault.
#[cfg(all(test, unix))]
struct AfterCommitDecisionStorage {
    /// Actual qualified native storage moved from the same owned private constructor.
    native: Box<dyn JournalStorage + Send>,
}

#[cfg(all(test, unix))]
impl JournalStorage for AfterCommitDecisionStorage {
    /// Delegate exact fixed-file reads to the actual native owner.
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>> {
        self.native.read(name)
    }
    /// Install and sync the real generation, then fail only after its committed phase.
    fn replace(&self, raw: &[u8], expected: Option<&[u8]>) -> Result<()> {
        let envelope: serde_json::Value =
            serde_json::from_slice(raw).map_err(|_| recovery_required())?;
        let commit_decided = envelope["payload"]["records"]
            .as_array()
            .ok_or_else(recovery_required)?
            .iter()
            .any(|record| record["phase"] == "commit-decided");
        self.native.replace(raw, expected)?;
        if commit_decided { Err(recovery_required()) } else { Ok(()) }
    }
}

/// Consume a fully qualified owned test state and wrap its actual native storage.
/// The original lock, source predicates and fixed journal names remain unchanged.
#[cfg(all(test, unix))]
pub(super) fn owned_test_state_after_commit_decision_fault(
    root: &Root,
    directory: File,
) -> Result<TransactionState> {
    let TransactionState { root, inner } = owned_test_state(root, directory)?;
    let StateInner { storage, payload, blocked, durable } =
        inner.into_inner().map_err(|_| recovery_required())?;
    Ok(TransactionState {
        root,
        inner: Mutex::new(StateInner {
            storage: Box::new(AfterCommitDecisionStorage { native: storage }),
            payload,
            blocked,
            durable,
        }),
    })
}

/// Source-only storage fault proposals; execution is exclusively Root-owned.
#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory exact generation port with an explicit before/after-publication fault.
    struct FaultStorage {
        /// Last actual mock generation, changed only by this same port.
        raw: Mutex<Option<Vec<u8>>>,
        /// A fault after replacement represents unknown durability, never no-write proof.
        after_write: bool,
    }

    impl JournalStorage for FaultStorage {
        /// Return the exact mock generation for expected-byte comparison.
        fn read(&self, _name: &str) -> Result<Option<Vec<u8>>> {
            Ok(self.raw.lock().map_err(|_| recovery_required())?.clone())
        }
        /// Fail at one storage phase while preserving truthful actual mock generation.
        fn replace(&self, raw: &[u8], expected: Option<&[u8]>) -> Result<()> {
            let mut held = self.raw.lock().map_err(|_| recovery_required())?;
            if held.as_deref() != expected {
                return Err(recovery_required());
            }
            if self.after_write {
                *held = Some(raw.to_vec());
            }
            Err(recovery_required())
        }
    }

    /// Neither before nor uncertain after-publication failure merges the new payload.
    #[test]
    fn storage_phase_faults_block_without_memory_only_acceptance() {
        for after_write in [false, true] {
            let root = RootIdentity { volume: 1, object: 2, binding_sha256: "a".repeat(64) };
            let payload = JournalPayload {
                schema_version: "forge.workspace-restore-state/1".into(),
                root,
                records: Vec::new(),
            };
            let mut inner = StateInner {
                storage: Box::new(FaultStorage { raw: Mutex::new(None), after_write }),
                payload: payload.clone(),
                blocked: false,
                durable: None,
            };
            assert!(replace(&mut inner, payload).is_err());
            assert!(inner.blocked);
            assert!(inner.durable.is_none());
            assert!(inner.payload.records.is_empty());
            assert_eq!(inner.storage.read(JOURNAL_NAME).unwrap().is_some(), after_write);
        }
    }

    /// Lookup IDs are not journal paths, UUIDs, receipt tokens or acceptance authority.
    #[test]
    fn closed_lookup_identifier_and_corrupt_journal_reject() {
        for value in ["../transactions.json", "op_a", "op_ABCDEF123456", "op_aaaaaaaaaaaa/child"] {
            assert!(validate_operation_id(value).is_err());
        }
        assert!(validate_operation_id("op_aaaaaaaaaaaa").is_ok());
        let root = RootIdentity { volume: 1, object: 2, binding_sha256: "a".repeat(64) };
        assert!(decode(br#"{"schema_version":"public","path":"project"}"#, &root).is_err());
        assert!(decode(br#"{"payload":{},"payload":{},"payload_sha256":"bad"}"#, &root).is_err());
    }

    /// Concurrent missing-component creators reopen the same real private winner.
    #[cfg(unix)]
    #[test]
    fn concurrent_cold_components_reopen_one_private_winner() {
        use std::os::unix::fs::MetadataExt as _;
        let fixture = tempfile::tempdir().unwrap();
        let parent = File::open(fixture.path()).unwrap();
        let owner = parent.metadata().unwrap().uid();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let mut threads = Vec::new();
        for _ in 0..8 {
            let parent = parent.try_clone().unwrap();
            let barrier = std::sync::Arc::clone(&barrier);
            threads.push(std::thread::spawn(move || {
                barrier.wait();
                let file =
                    create_private_component(&parent, std::ffi::OsStr::new("shared")).unwrap();
                let metadata = file.metadata().unwrap();
                assert!(metadata.is_dir());
                assert_eq!(metadata.uid(), owner);
                assert_eq!(metadata.mode() & 0o777, 0o700);
                (metadata.dev(), metadata.ino())
            }));
        }
        let identities: Vec<_> = threads.into_iter().map(|thread| thread.join().unwrap()).collect();
        assert!(identities.iter().all(|identity| *identity == identities[0]));
        // This final call deterministically exercises the already-existing winner.
        let reopened = create_private_component(&parent, std::ffi::OsStr::new("shared")).unwrap();
        let metadata = reopened.metadata().unwrap();
        assert_eq!((metadata.dev(), metadata.ino()), identities[0]);
    }

    /// Existing conflicting files/links and insecure directory modes remain refused.
    #[cfg(unix)]
    #[test]
    fn raced_components_preserve_no_follow_and_private_mode_checks() {
        use std::os::unix::fs::{PermissionsExt as _, symlink};
        let fixture_home = std::env::var_os("HOME").expect("native private-state test home");
        let fixture = tempfile::tempdir_in(fixture_home).unwrap();
        let parent = File::open(fixture.path()).unwrap();
        let child = fixture.path().join("shared");
        std::fs::write(&child, b"conflicting file").unwrap();
        assert!(create_private_component(&parent, std::ffi::OsStr::new("shared")).is_err());
        assert!(qualified_directory(&child).is_err());
        std::fs::remove_file(&child).unwrap();
        symlink(fixture.path(), &child).unwrap();
        assert!(create_private_component(&parent, std::ffi::OsStr::new("shared")).is_err());
        assert!(qualified_directory(&child).is_err());
        std::fs::remove_file(&child).unwrap();
        std::fs::create_dir(&child).unwrap();
        std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o777)).unwrap();
        assert!(qualified_directory(&child).is_err());
        std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(qualified_directory(&child).is_ok());
    }
}
