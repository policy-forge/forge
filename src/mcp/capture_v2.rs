//! Non-authorizing /2 raw original owner over genuine native held generations.
//!
//! One actual caller control is retained for the owner lifetime. Native handles,
//! paths and bytes are admitted before growth and rechecked completely. This
//! foundation creates no native approval, currentness, source rights or index.

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use sha2::{Digest as _, Sha256};

use super::declarations_v2::DeclarationAdmission;
use crate::linkage::fresh::{
    self, AdmittedReadError, CapturedLocal, NativeEnvelope, NativeGeometryError, PreparedLocal,
    RootGeneration,
};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

/// One joint retained original/proof/native-data/eventual output logical byte ceiling.
const MAX_BYTES: usize = 50 * 1024 * 1024;
/// Configurations and all declared present/absent original attempts.
const MAX_ORIGINALS: usize = 1001;
/// Every actual retained or temporarily reserved native handle.
const MAX_HANDLES: usize = 1001;
/// Complete distinct declared directory capture attempts, shared across manifests.
#[cfg(test)]
const MAX_DIRECTORIES: usize = 64;
/// Original monotonic work, shared with codec/native/final verification.
const MAX_WORK: usize = 100_000;
/// Maximum raw ordinary original; tighter per-kind limits are supplied below this.
const MAX_FILE: usize = 10 * 1024 * 1024;
/// Actual complete configuration original bound.
const MAX_CONFIG: usize = 1024 * 1024;
/// Fixed borrowed hashing chunk; native reader scratch has this same bound.
const CHUNK: usize = 32 * 1024;

/// Actual entrypoint purpose, not decoded authority or a currentness assertion.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CapturePurpose {
    ServerRead,
    OfflineBuild,
}
/// One of the two actual qualified roots; no arbitrary additional root is accepted.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RootSlot {
    Project,
    External,
}
/// Actual fixed-route configuration original, independent of its inert shape.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Configuration {
    Discovery,
    Visibility,
    Purpose,
}
/// Explicit typed absence policy; it never converts an ordinary unsafe open error.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MissingPolicy {
    Required,
    Observe,
}

/// First actual caller/admission stop, never replaced by ordinary native failure.
enum Stop {
    Capacity,
    Failed(Error),
    Interrupted(Interruption),
}
impl Stop {
    /// Preserve the first safe failure without retaining private native error text.
    fn error(&self) -> WorkError {
        match self {
            Self::Capacity => WorkError::Failed(Error::new(
                "mcp-capture-capacity",
                "The complete MCP input exceeds the supported bound.",
                false,
            )),
            Self::Failed(error) => WorkError::Failed(error.clone()),
            Self::Interrupted(reason) => WorkError::Interrupted(*reason),
        }
    }
}

/// Counters belong to the one real raw owner, not a caller-created allowance.
#[derive(Default)]
struct Ledger {
    /// Full work consumed before successful or failed phases; never refunded.
    work: usize,
    /// Complete original open attempts including configs, hidden files and absences.
    originals: usize,
    /// Distinct directory open attempts; exact held-path reuse does not open again.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Preserve the original directory-attempt counter layout and unchanged bound for the genuine cfg directory controls; this slice refuses the missing production linkage family."
        )
    )]
    directories: usize,
    /// All live retained and reserved native handle capacity.
    handles: usize,
    /// All live retained/reserved original/path/generation/hash/owned-data logical bytes.
    bytes: usize,
    /// Highest actual reserved live handle observation, plain accounting data.
    peak_handles: usize,
    /// Immutable first actual bound/work stop.
    stop: Option<Stop>,
}

/// The actual original caller reference cannot be swapped or clock-renewed later.
struct SharedWork<'control, C: WorkControl + ?Sized> {
    /// One monotonic admission ledger for the complete operation.
    ledger: Rc<RefCell<Ledger>>,
    /// The original accepted caller control, retained through all final fences.
    control: RefCell<&'control mut C>,
}
impl<C: WorkControl + ?Sized> SharedWork<'_, C> {
    /// Read the first stop without manufacturing approval or resetting counters.
    fn stopped(&self) -> Option<WorkError> {
        self.ledger.borrow().stop.as_ref().map(Stop::error)
    }
    /// Latch one actual native/admission capacity failure before post-fencing.
    fn capacity(&self) -> WorkError {
        let mut ledger = self.ledger.borrow_mut();
        let stop = ledger.stop.get_or_insert(Stop::Capacity);
        stop.error()
    }
    /// Call the original control and observe its interruption even after success.
    fn checkpoint(&self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
        if let Some(error) = self.stopped() {
            return Err(error);
        }
        let (result, interruption) = {
            let mut control = self.control.borrow_mut();
            let result = control.checkpoint(stage, progress);
            (result, control.interruption())
        };
        let stop = match result {
            Err(WorkError::Failed(error)) => Some(Stop::Failed(error)),
            Err(WorkError::Interrupted(reason)) => Some(Stop::Interrupted(reason)),
            Ok(()) => interruption.map(Stop::Interrupted),
        };
        if let Some(stop) = stop {
            self.ledger.borrow_mut().stop.get_or_insert(stop);
        }
        self.stopped().map_or(Ok(()), Err)
    }
    /// Charge actual admitted work before any downstream native/registry growth.
    fn charge(&self, amount: usize) -> WorkResult<()> {
        self.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let used = self.ledger.borrow().work.checked_add(amount);
        let Some(used) = used.filter(|used| *used <= MAX_WORK) else {
            return Err(self.capacity());
        };
        self.ledger.borrow_mut().work = used;
        Ok(())
    }
    /// Record an actual attempted original before its native open.
    fn original(&self) -> WorkResult<()> {
        self.charge(1)?;
        let count = self.ledger.borrow().originals.checked_add(1);
        let Some(count) = count.filter(|count| *count <= MAX_ORIGINALS) else {
            return Err(self.capacity());
        };
        self.ledger.borrow_mut().originals = count;
        Ok(())
    }
    /// Admit a new complete directory observation; reuse consumes work separately.
    #[cfg(test)]
    fn directory(&self) -> WorkResult<()> {
        self.charge(1)?;
        let count = self.ledger.borrow().directories.checked_add(1);
        let Some(count) = count.filter(|count| *count <= MAX_DIRECTORIES) else {
            return Err(self.capacity());
        };
        self.ledger.borrow_mut().directories = count;
        Ok(())
    }
    /// Propagate an ordinary native result only after the unconditional caller fence.
    fn native<T>(&self, result: Result<T, crate::ForgeError>) -> WorkResult<T> {
        let fence = self.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged);
        fence?;
        result.map_err(|_| WorkError::Failed(Error::containment()))
    }
    /// Checked native footprint arithmetic latches actual capacity before any post-fence.
    fn native_geometry<T>(&self, result: Result<T, NativeGeometryError>) -> WorkResult<T> {
        self.read(result.map_err(|error| match error {
            NativeGeometryError::Domain(error) => AdmittedReadError::Domain(error),
            NativeGeometryError::Capacity => AdmittedReadError::Capacity,
        }))
    }
    /// Fence every whole public ordinary result on the retained original control.
    fn phase<T>(&self, result: WorkResult<T>, stage: Stage) -> WorkResult<T> {
        let fence = self.checkpoint(stage, ProgressUpdate::Unchanged);
        fence?;
        result
    }
    /// Full read result preserves first capacity/admission then ordinary post-fence.
    fn read<T>(&self, result: Result<T, AdmittedReadError<WorkError>>) -> WorkResult<T> {
        if matches!(&result, Err(AdmittedReadError::Capacity)) {
            self.capacity();
        }
        if let Err(AdmittedReadError::Admission(error)) = &result {
            let stop = match error {
                WorkError::Failed(error) => Stop::Failed(error.clone()),
                WorkError::Interrupted(reason) => Stop::Interrupted(*reason),
            };
            self.ledger.borrow_mut().stop.get_or_insert(stop);
        }
        let fence = self.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged);
        fence?;
        match result {
            Ok(value) => Ok(value),
            Err(AdmittedReadError::Domain(_)) => Err(WorkError::Failed(Error::containment())),
            Err(AdmittedReadError::Capacity | AdmittedReadError::Admission(_)) => {
                Err(self.capacity())
            }
        }
    }
}

/// Private adapter consumed by the real codec and native helpers, not a new pool.
pub(super) struct Admission<'control, C: WorkControl + ?Sized> {
    shared: Rc<SharedWork<'control, C>>,
}
impl<C: WorkControl + ?Sized> DeclarationAdmission for Admission<'_, C> {
    /// Consume failed and successful decoder work on the original owner ledger.
    fn charge(&mut self, amount: usize) -> WorkResult<()> {
        self.shared.charge(amount)
    }
    /// Fence the ordinary phase using the retained original accepted control.
    fn checkpoint(&mut self, stage: Stage) -> WorkResult<()> {
        self.shared.checkpoint(stage, ProgressUpdate::Unchanged)
    }
    /// Latch native codec raw/roster/arithmetic capacity on the same first stop.
    fn capacity(&mut self) -> WorkError {
        self.shared.capacity()
    }
}
impl<C: WorkControl + ?Sized> WorkControl for Admission<'_, C> {
    /// Delegate the maintained native stage/progress to the original caller.
    fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
        self.shared.checkpoint(stage, progress)
    }
    /// Preserve the first owner interruption; capacity/failed stop remains typed failure.
    fn interruption(&self) -> Option<Interruption> {
        match &self.shared.ledger.borrow().stop {
            Some(Stop::Interrupted(reason)) => Some(*reason),
            _ => None,
        }
    }
}

/// Controlled data allocation on the same owner pool; it grants no native/source authority.
impl<C: WorkControl + ?Sized> Admission<'_, C> {
    /// Retain an entire source-backed upper bound before producing the actual owned value.
    /// The callback must admit its complete native/tree/string work through this adapter.
    pub(super) fn retain<T>(
        &mut self,
        logical_bytes: usize,
        produce: impl FnOnce(&mut Self) -> WorkResult<T>,
    ) -> WorkResult<AdmittedValue<T>> {
        self.shared.charge(1)?;
        let ticket = Ticket::reserve(&self.shared, 0, logical_bytes)?;
        let result = produce(self);
        let value = self.shared.phase(result, Stage::PrepareDomain)?;
        Ok(AdmittedValue { value: Some(value), ticket: Some(ticket) })
    }
    /// Resolve a complete ordinary native result only after the same original phase fence.
    pub(super) fn phase<T>(&mut self, result: WorkResult<T>, stage: Stage) -> WorkResult<T> {
        self.shared.phase(result, stage)
    }
}

/// Actual retained native/decoded data under a conservative logical byte ticket.
/// This ordinary owner is not a native approval and has no extraction constructor.
pub(super) struct AdmittedValue<T> {
    /// Real callback result; the value drops before its byte capacity is released.
    value: Option<T>,
    /// Original shared pool, never a copied allowance or a caller currentness claim.
    ticket: Option<Ticket>,
}
impl<T> AdmittedValue<T> {
    /// Borrow only the actual produced value while its complete ticket remains retained.
    pub(super) fn value(&self) -> &T {
        self.value.as_ref().expect("admitted value invariant")
    }
}
impl<T> Drop for AdmittedValue<T> {
    /// Settle lifetime honestly: owned data drops before returning its reservation.
    fn drop(&mut self) {
        drop(self.value.take());
        drop(self.ticket.take());
    }
}

/// Reserved/live capacity releases only after the corresponding native owner drops.
struct Ticket {
    /// One actual shared caller ledger, retained beyond the native drop.
    ledger: Rc<RefCell<Ledger>>,
    /// Reserved or settled actual native handles.
    handles: usize,
    /// Reserved or settled logical original/proof payload bytes.
    bytes: usize,
}
impl Ticket {
    /// Reserve a complete conservative peak before native open/clone/path growth.
    fn reserve<C: WorkControl + ?Sized>(
        shared: &SharedWork<'_, C>,
        handles: usize,
        bytes: usize,
    ) -> WorkResult<Self> {
        shared.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
        let (next_handles, next_bytes) = {
            let ledger = shared.ledger.borrow();
            (ledger.handles.checked_add(handles), ledger.bytes.checked_add(bytes))
        };
        let (Some(next_handles), Some(next_bytes)) =
            (next_handles.filter(|n| *n <= MAX_HANDLES), next_bytes.filter(|n| *n <= MAX_BYTES))
        else {
            return Err(shared.capacity());
        };
        {
            let mut ledger = shared.ledger.borrow_mut();
            ledger.handles = next_handles;
            ledger.bytes = next_bytes;
            ledger.peak_handles = ledger.peak_handles.max(next_handles);
        }
        Ok(Self { ledger: Rc::clone(&shared.ledger), handles, bytes })
    }
    /// Settle from actual native private vectors only after temporary owners unwind.
    fn settle<C: WorkControl + ?Sized>(
        &mut self,
        shared: &SharedWork<'_, C>,
        handles: usize,
        bytes: usize,
    ) -> WorkResult<()> {
        if handles > self.handles || bytes > self.bytes {
            return Err(shared.capacity());
        }
        {
            let mut ledger = self.ledger.borrow_mut();
            ledger.handles -= self.handles - handles;
            ledger.bytes -= self.bytes - bytes;
        }
        self.handles = handles;
        self.bytes = bytes;
        Ok(())
    }
}
impl Drop for Ticket {
    /// The enclosing owner explicitly drops its real native data before this ticket.
    fn drop(&mut self) {
        let mut ledger = self.ledger.borrow_mut();
        ledger.handles -= self.handles;
        ledger.bytes -= self.bytes;
    }
}

/// Shared actual root and its accounting stay alive through every descendant owner.
struct HeldRoot {
    /// Real qualified native root; no Rc escapes outside this implementation.
    native: Option<Rc<RootGeneration>>,
    /// Settled actual root footprint, released after the last real native proof.
    ticket: Option<Ticket>,
}
impl Drop for HeldRoot {
    /// Last descendant keeps this holder until its real native generation has dropped.
    fn drop(&mut self) {
        drop(self.native.take());
        drop(self.ticket.take());
    }
}
impl HeldRoot {
    /// Borrow the actual private native root without constructing one from its path.
    fn native(&self) -> &Rc<RootGeneration> {
        self.native.as_ref().expect("held root invariant")
    }
}

/// Fixed configurations have no arbitrary key/route constructor.
enum FileKey {
    Config(Configuration),
    Declared(String),
}
/// Borrowed input registration; owned key allocation follows native reservation.
#[derive(Clone, Copy)]
enum RegistrationKey<'a> {
    Config(Configuration),
    Declared(&'a str),
}
/// Native open state changes only through the actual controlled original reader.
enum LocalState {
    Opened(PreparedLocal),
    Read(CapturedLocal),
}
impl LocalState {
    /// Borrow the route from the real native owner, not a duplicated caller field.
    fn relative(&self) -> &Path {
        match self {
            Self::Opened(value) => value.relative(),
            Self::Read(value) => value.relative(),
        }
    }
}
/// One actual present/absent member with its matching root/tickets retained.
struct HeldFile {
    /// Real native opened/captured member; take/drop precedes capacity release.
    native: Option<LocalState>,
    /// Original raw digest after the real complete controlled read, absent on typed absence.
    hash: Option<String>,
    /// Native generation/key payload and handles already admitted before open.
    proof_ticket: Option<Ticket>,
    /// Complete actual raw bytes and fixed observed hash rendering.
    raw_ticket: Option<Ticket>,
    /// Same real accounted root remains live through this original native proof.
    root: Rc<HeldRoot>,
    /// Actual root selection, never a path-inferred replacement.
    slot: RootSlot,
    /// Exact private registration key or fixed configuration purpose.
    key: FileKey,
    /// Tighter consumed kind bound, at most the ordinary original ceiling.
    limit: usize,
}
impl Drop for HeldFile {
    /// Drop actual bytes/hash/native handles before returning any capacity credit.
    fn drop(&mut self) {
        drop(self.native.take());
        drop(self.hash.take());
        drop(std::mem::replace(&mut self.key, FileKey::Config(Configuration::Discovery)));
        drop(self.raw_ticket.take());
        drop(self.proof_ticket.take());
    }
}
/// One complete actual declared directory, including unused declarations.
struct HeldDirectory {
    /// Native generation retaining every real descendant directory.
    native: Option<fresh::HeldDirectoryGeneration>,
    /// Held root remains accounted for this generation's entire lifetime.
    root: Rc<HeldRoot>,
    /// Real native directory footprint, released only after its native data drops.
    ticket: Option<Ticket>,
}
impl Drop for HeldDirectory {
    /// Native proof drops before its ticket and before the same actual root.
    fn drop(&mut self) {
        drop(self.native.take());
        drop(self.ticket.take());
    }
}

/// Non-authorizing borrowed observation; bytes/hash/absence are plain actual facts.
#[derive(Clone, Copy)]
pub(super) enum Observation<'a> {
    Present { bytes: &'a [u8], raw_sha256: &'a str },
    Absent,
}
/// Borrowed plain view issued only from a real same-owner registered original.
/// Role/kind/base-companion meanings are reconciled against the admitted declaration.
pub(super) struct MemberV2<'a> {
    /// Exact private key of this actual registered original.
    key: &'a str,
    /// Route borrowed from its retained native original, never a caller vector index.
    route: &'a Path,
    /// Actual complete bytes/hash or typed absence observed on that same original.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "The owner-issued view preserves its complete validated raw observation; cfg controls inspect the data while production correlates every key and route."
        )
    )]
    observation: Observation<'a>,
}
impl<'a> MemberV2<'a> {
    /// Borrow the actual complete registration key for closed roster correlation.
    pub(super) fn key(&self) -> &'a str {
        self.key
    }
    /// Borrow the original native route without reopening a manifest-selected path.
    pub(super) fn route(&self) -> &'a Path {
        self.route
    }
    /// Observe ordinary raw facts without creating owner/currentness authority.
    #[cfg(test)]
    pub(super) fn observation(&self) -> Observation<'a> {
        self.observation
    }
}
/// Registration chronology, derived only by the actual open/read methods.
enum ReadPhase {
    Opening,
    Reading,
}
/// Mutable raw capture; complete high-level declaration/native predicates remain absent.
pub(super) struct CaptureBuilderV2<'control, C: WorkControl + ?Sized> {
    /// Single original ledger/control, shared by every codec/native callback.
    shared: Rc<SharedWork<'control, C>>,
    /// All actual registered files, including three fixed configuration originals.
    files: Vec<HeldFile>,
    /// Every actual declared directory proof, including unused directory declarations.
    directories: Vec<HeldDirectory>,
    /// Both actual roots remain held for the whole captured operation.
    roots: [Rc<HeldRoot>; 2],
    /// Actual entrypoint purpose; it cannot be changed by a decoded configuration.
    purpose: CapturePurpose,
    /// First declared original read freezes new original registration.
    phase: ReadPhase,
}
/// Held raw capture; neither its name nor its existence asserts native/current authority.
pub(super) struct HeldInputsV2<'control, C: WorkControl + ?Sized> {
    builder: CaptureBuilderV2<'control, C>,
}

/// Fixed safe ordinary invalidity; it cannot erase an already latched owner stop.
fn invalid() -> WorkError {
    WorkError::Failed(Error::invalid())
}
/// Match normalized ASCII components without allocating a folded path copy.
fn paths_overlap(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
        || left.get(..right.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(right))
            && left.as_bytes().get(right.len()) == Some(&b'/')
        || right.get(..left.len()).is_some_and(|prefix| prefix.eq_ignore_ascii_case(left))
            && right.as_bytes().get(left.len()) == Some(&b'/')
}
/// Refuse folded equal/nested native root component spellings without allocations.
fn roots_spelling_overlap(left: &Path, right: &Path) -> bool {
    let mut left = left.components();
    let mut right = right.components();
    loop {
        match (left.next(), right.next()) {
            (Some(left), Some(right)) => {
                let (Some(left), Some(right)) =
                    (left.as_os_str().to_str(), right.as_os_str().to_str())
                else {
                    return true;
                };
                if !left.eq_ignore_ascii_case(right) {
                    return false;
                }
            }
            _ => return true,
        }
    }
}

/// Keep existing portable ASCII/path rules without allocating uppercase components.
fn portable(path: &Path) -> bool {
    let Some(value) = path.to_str() else {
        return false;
    };
    if value.is_empty()
        || value.len() > 1024
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-/".contains(&byte))
        || value.starts_with('/')
    {
        return false;
    }
    value.split('/').all(|part| {
        let prefix = part.split('.').next().unwrap_or("");
        !part.is_empty()
            && part != "."
            && part != ".."
            && part.len() <= 128
            && !part.ends_with('.')
            && !["CON", "PRN", "AUX", "NUL"].iter().any(|name| prefix.eq_ignore_ascii_case(name))
            && !(prefix.len() == 4
                && (prefix.get(..3).is_some_and(|name| {
                    name.eq_ignore_ascii_case("COM") || name.eq_ignore_ascii_case("LPT")
                }))
                && prefix.as_bytes()[3].is_ascii_digit())
    })
}
/// Reserve native operation geometry/work before real open/clone/verification starts.
fn reserve_native<C: WorkControl + ?Sized>(
    shared: &SharedWork<'_, C>,
    envelope: &NativeEnvelope,
    extra: usize,
) -> WorkResult<Ticket> {
    shared.charge(envelope.work())?;
    let bytes = envelope.payload().checked_add(extra).ok_or_else(|| shared.capacity())?;
    Ticket::reserve(shared, envelope.handles(), bytes)
}
/// Qualify one actual root while its complete conservative ticket remains live.
fn qualify<C: WorkControl + ?Sized>(
    shared: &SharedWork<'_, C>,
    path: &Path,
) -> WorkResult<Rc<HeldRoot>> {
    shared.charge(1)?;
    let envelope = shared.native_geometry(fresh::root_envelope(path))?;
    let mut ticket = reserve_native(shared, &envelope, 0)?;
    let result = fresh::qualify_root(path);
    let native = shared.native(result)?;
    let geometry = shared.native_geometry(native.geometry())?;
    ticket.settle(shared, geometry.handles(), geometry.payload())?;
    Ok(Rc::new(HeldRoot { native: Some(Rc::new(native)), ticket: Some(ticket) }))
}

impl<'control, C: WorkControl + ?Sized> CaptureBuilderV2<'control, C> {
    /// Start the real two-root capture and open/read all three fixed config originals.
    pub(super) fn new(
        project: &Path,
        external: &Path,
        purpose: CapturePurpose,
        control: &'control mut C,
    ) -> WorkResult<Self> {
        let shared = Rc::new(SharedWork {
            ledger: Rc::new(RefCell::new(Ledger::default())),
            control: RefCell::new(control),
        });
        let result = Self::new_inner(project, external, purpose, Rc::clone(&shared));
        shared.phase(result, Stage::CaptureResource)
    }
    /// Constructor pipeline retains raw/native errors until the public final fence.
    fn new_inner(
        project: &Path,
        external: &Path,
        purpose: CapturePurpose,
        shared: Rc<SharedWork<'control, C>>,
    ) -> WorkResult<Self> {
        shared.charge(
            project
                .components()
                .count()
                .checked_add(external.components().count())
                .ok_or_else(|| shared.capacity())?,
        )?;
        let overlap = roots_spelling_overlap(project, external);
        shared.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
        if overlap {
            return Err(WorkError::Failed(Error::containment()));
        }
        let project = qualify(&shared, project)?;
        let external = qualify(&shared, external)?;
        shared.charge(1)?;
        let overlap = fresh::roots_overlap(project.native(), external.native());
        shared.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
        if overlap {
            return Err(WorkError::Failed(Error::containment()));
        }
        let mut builder = Self {
            shared,
            files: Vec::new(),
            directories: Vec::new(),
            roots: [project, external],
            purpose,
            phase: ReadPhase::Opening,
        };
        builder.open_config(Configuration::Discovery, RootSlot::Project, "forge.mcp.json")?;
        builder.open_config(
            Configuration::Visibility,
            RootSlot::Project,
            "forge.mcp.visibility.json",
        )?;
        let path = match purpose {
            CapturePurpose::ServerRead => "forge.mcp.disclosure-decision.json",
            CapturePurpose::OfflineBuild => "forge.mcp.index-build-intent.json",
        };
        builder.open_config(Configuration::Purpose, RootSlot::External, path)?;
        for index in 0..builder.files.len() {
            builder.read_slot(index)?;
        }
        Ok(builder)
    }
    /// Lend a callback tied to the same original control/ledger, never an allowance copy.
    pub(super) fn admission(&self) -> Admission<'control, C> {
        Admission { shared: Rc::clone(&self.shared) }
    }
    /// Borrow the complete actual fixed configuration bytes after their genuine read.
    pub(super) fn configuration(&self, configuration: Configuration) -> WorkResult<Option<&[u8]>> {
        let mut selected = None;
        for file in &self.files {
            self.shared.charge(1)?;
            if matches!(&file.key, FileKey::Config(kind) if *kind == configuration) {
                selected = match file.native.as_ref() {
                    Some(LocalState::Read(CapturedLocal::Present(bytes, _))) => {
                        Some(bytes.as_slice())
                    }
                    _ => None,
                };
            }
        }
        self.shared.phase(Ok(selected), Stage::CaptureResource)
    }
    /// Resolve only the real private selected root, never a caller original proof.
    fn root(&self, slot: RootSlot) -> &Rc<HeldRoot> {
        &self.roots[usize::from(matches!(slot, RootSlot::External))]
    }
    /// Fixed config route registration remains private and requires actual presence.
    fn open_config(&mut self, key: Configuration, slot: RootSlot, path: &str) -> WorkResult<()> {
        self.open_file(
            RegistrationKey::Config(key),
            slot,
            Path::new(path),
            MAX_CONFIG,
            MissingPolicy::Required,
        )
    }
    /// Register one actual declared original before the first declared content read.
    pub(super) fn open_original(
        &mut self,
        key: &str,
        path: &Path,
        limit: usize,
        missing: MissingPolicy,
    ) -> WorkResult<()> {
        let result = self.open_original_inner(key, path, limit, missing);
        self.shared.phase(result, Stage::CaptureResource)
    }
    /// Preserve the actual public phase result before its unconditional final fence.
    fn open_original_inner(
        &mut self,
        key: &str,
        path: &Path,
        limit: usize,
        missing: MissingPolicy,
    ) -> WorkResult<()> {
        self.shared.charge(1)?;
        if !matches!(self.phase, ReadPhase::Opening) || !super::safe_key(key) || limit > MAX_FILE {
            return Err(invalid());
        }
        for file in &self.files {
            self.shared.charge(1)?;
            if matches!(&file.key, FileKey::Declared(existing) if existing == key) {
                return Err(invalid());
            }
        }
        self.open_file(RegistrationKey::Declared(key), RootSlot::Project, path, limit, missing)
    }
    /// Pre-admit complete key/path aliases, slot, native geometry and identity before read.
    fn open_file(
        &mut self,
        key: RegistrationKey<'_>,
        slot: RootSlot,
        path: &Path,
        limit: usize,
        missing: MissingPolicy,
    ) -> WorkResult<()> {
        self.shared.charge(1)?;
        if !portable(path) {
            return Err(invalid());
        }
        let route = path.to_str().ok_or_else(invalid)?;
        for file in &self.files {
            self.shared.charge(1)?;
            let original = file
                .native
                .as_ref()
                .ok_or_else(invalid)?
                .relative()
                .to_str()
                .ok_or_else(invalid)?;
            if file.slot == slot && paths_overlap(original, route) {
                return Err(invalid());
            }
        }
        for directory in &self.directories {
            self.shared.charge(1)?;
            let original = directory
                .native
                .as_ref()
                .ok_or_else(invalid)?
                .relative()
                .to_str()
                .ok_or_else(invalid)?;
            if slot == RootSlot::Project
                && (route.eq_ignore_ascii_case(original)
                    || original
                        .get(..route.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(route))
                        && original.as_bytes().get(route.len()) == Some(&b'/'))
            {
                return Err(invalid());
            }
        }
        self.shared.original()?;
        let root = Rc::clone(self.root(slot));
        let envelope =
            self.shared.native_geometry(fresh::descendant_envelope(root.native(), path))?;
        let key_bytes = match &key {
            RegistrationKey::Config(_) => 0,
            RegistrationKey::Declared(key) => key.len(),
        };
        let mut ticket = reserve_native(&self.shared, &envelope, key_bytes)?;
        let result =
            fresh::prepare_local(Rc::clone(root.native()), path, missing == MissingPolicy::Observe);
        let native = self.shared.native(result)?;
        let geometry = self.shared.native_geometry(native.geometry())?;
        if let Some(identity) = native.identity() {
            for file in &self.files {
                self.shared.charge(1)?;
                let actual = match file.native.as_ref().ok_or_else(invalid)? {
                    LocalState::Opened(native) => native.identity(),
                    LocalState::Read(CapturedLocal::Present(_, native)) => Some(native.identity()),
                    LocalState::Read(CapturedLocal::Absent(_)) => None,
                };
                if actual == Some(identity) {
                    return Err(WorkError::Failed(Error::containment()));
                }
            }
        }
        let size = self.shared.native(native.observed_size())?;
        if size > limit as u64 {
            return Err(self.shared.capacity());
        }
        let payload =
            geometry.payload().checked_add(key_bytes).ok_or_else(|| self.shared.capacity())?;
        ticket.settle(&self.shared, geometry.handles(), payload)?;
        let key = match key {
            RegistrationKey::Config(kind) => FileKey::Config(kind),
            RegistrationKey::Declared(key) => FileKey::Declared(key.to_owned()),
        };
        self.files.push(HeldFile {
            native: Some(LocalState::Opened(native)),
            hash: None,
            proof_ticket: Some(ticket),
            raw_ticket: None,
            root,
            slot,
            key,
            limit,
        });
        Ok(())
    }
    /// Admit and retain an actual project-contained evidence directory, including unused ones.
    #[cfg(test)]
    pub(super) fn capture_directory(&mut self, path: &Path) -> WorkResult<()> {
        let result = self.capture_directory_inner(path);
        self.shared.phase(result, Stage::CaptureResource)
    }
    /// Preserve the actual public phase result before its unconditional final fence.
    #[cfg(test)]
    fn capture_directory_inner(&mut self, path: &Path) -> WorkResult<()> {
        self.shared.charge(1)?;
        if !portable(path) {
            return Err(invalid());
        }
        let route = path.to_str().ok_or_else(invalid)?;
        for file in &self.files {
            self.shared.charge(1)?;
            let original = file
                .native
                .as_ref()
                .ok_or_else(invalid)?
                .relative()
                .to_str()
                .ok_or_else(invalid)?;
            if file.slot == RootSlot::Project
                && (route.eq_ignore_ascii_case(original)
                    || route
                        .get(..original.len())
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(original))
                        && route.as_bytes().get(original.len()) == Some(&b'/'))
            {
                return Err(invalid());
            }
        }
        for directory in &self.directories {
            self.shared.charge(1)?;
            let original = directory
                .native
                .as_ref()
                .ok_or_else(invalid)?
                .relative()
                .to_str()
                .ok_or_else(invalid)?;
            if original == route {
                return Ok(());
            }
            if original.eq_ignore_ascii_case(route) {
                return Err(invalid());
            }
        }
        self.shared.directory()?;
        let root = Rc::clone(self.root(RootSlot::Project));
        let envelope =
            self.shared.native_geometry(fresh::descendant_envelope(root.native(), path))?;
        let mut ticket = reserve_native(&self.shared, &envelope, 0)?;
        let result = fresh::prepare_directory(Rc::clone(root.native()), path);
        let native = self.shared.native(result)?;
        for directory in &self.directories {
            self.shared.charge(1)?;
            if directory.native.as_ref().ok_or_else(invalid)?.identity() == native.identity() {
                return Err(WorkError::Failed(Error::containment()));
            }
        }
        let geometry = self.shared.native_geometry(native.geometry())?;
        ticket.settle(&self.shared, geometry.handles(), geometry.payload())?;
        self.directories.push(HeldDirectory { native: Some(native), root, ticket: Some(ticket) });
        Ok(())
    }
    /// Read only an already-registered original; the first call freezes file registration.
    #[cfg(test)]
    pub(super) fn read_original(&mut self, key: &str) -> WorkResult<()> {
        let result = self.read_original_inner(key);
        self.shared.phase(result, Stage::CaptureResource)
    }
    /// Preserve the actual public phase result before its unconditional final fence.
    #[cfg(test)]
    fn read_original_inner(&mut self, key: &str) -> WorkResult<()> {
        self.shared.charge(1)?;
        self.phase = ReadPhase::Reading;
        let mut selected = None;
        for (index, file) in self.files.iter().enumerate() {
            self.shared.charge(1)?;
            if matches!(&file.key, FileKey::Declared(existing) if existing == key) {
                selected = Some(index);
            }
        }
        self.read_slot(selected.ok_or_else(invalid)?)
    }
    /// Read the complete registered roster, including hidden observations and typed absences.
    pub(super) fn read_remaining(&mut self) -> WorkResult<()> {
        let result = self.read_remaining_inner();
        self.shared.phase(result, Stage::CaptureResource)
    }
    /// Preserve the actual public phase result before its unconditional final fence.
    fn read_remaining_inner(&mut self) -> WorkResult<()> {
        self.phase = ReadPhase::Reading;
        for index in 0..self.files.len() {
            self.read_slot(index)?;
        }
        Ok(())
    }
    /// Read one real held original within remaining joint bytes; no path opens again.
    fn read_slot(&mut self, index: usize) -> WorkResult<()> {
        self.shared.charge(1)?;
        let file = self.files.get_mut(index).ok_or_else(invalid)?;
        if matches!(file.native.as_ref(), Some(LocalState::Read(_))) {
            return Ok(());
        }
        let Some(LocalState::Opened(prepared)) = file.native.take() else {
            return Err(invalid());
        };
        let size = self.shared.native(prepared.observed_size())?;
        let geometry = self.shared.native_geometry(prepared.geometry())?;
        let root_envelope =
            self.shared.native_geometry(fresh::root_envelope(file.root.native().path()))?;
        self.shared.charge(root_envelope.work())?;
        let temporary = root_envelope.payload();
        let hash_bytes = usize::from(prepared.identity().is_some()) * 64;
        let used_bytes = self.shared.ledger.borrow().bytes;
        let remaining = MAX_BYTES
            .checked_sub(used_bytes)
            .and_then(|n| n.checked_sub(temporary))
            .and_then(|n| n.checked_sub(hash_bytes))
            .ok_or_else(|| self.shared.capacity())?;
        let limit = file.limit.min(remaining);
        if size > limit as u64 {
            return Err(self.shared.capacity());
        }
        let bytes = temporary
            .checked_add(hash_bytes)
            .and_then(|n| n.checked_add(limit))
            .ok_or_else(|| self.shared.capacity())?;
        let mut ticket = Ticket::reserve(&self.shared, root_envelope.handles(), bytes)?;
        let shared = Rc::clone(&self.shared);
        let result = prepared.read_mcp_admitted(limit as u64, &mut || shared.charge(1));
        let captured = self.shared.read(result)?;
        let captured_geometry = self.shared.native_geometry(captured.geometry())?;
        if captured_geometry.handles() != geometry.handles()
            || captured_geometry.payload() != geometry.payload()
        {
            return Err(invalid());
        }
        let (length, hash) = match &captured {
            CapturedLocal::Present(bytes, _) => (bytes.len(), Some(hash_raw(&self.shared, bytes)?)),
            CapturedLocal::Absent(_) => (0, None),
        };
        let retained = length
            .checked_add(hash.as_ref().map_or(0, String::len))
            .ok_or_else(|| self.shared.capacity())?;
        ticket.settle(&self.shared, 0, retained)?;
        file.native = Some(LocalState::Read(captured));
        file.hash = hash;
        file.raw_ticket = Some(ticket);
        Ok(())
    }
    /// Borrow actual registered raw data; absent and unread observations are distinct.
    pub(super) fn original(&self, key: &str) -> WorkResult<Option<Observation<'_>>> {
        let mut selected = None;
        for file in &self.files {
            self.shared.charge(1)?;
            if matches!(&file.key, FileKey::Declared(existing) if existing == key) {
                selected = observe(file);
            }
        }
        self.shared.phase(Ok(selected), Stage::CaptureResource)
    }
    /// Close only the complete raw registered roster after every actual native fence.
    pub(super) fn finish(self) -> WorkResult<HeldInputsV2<'control, C>> {
        let shared = Rc::clone(&self.shared);
        let result = (|| {
            for file in &self.files {
                self.shared.charge(1)?;
                if !matches!(file.native.as_ref(), Some(LocalState::Read(_))) {
                    return Err(invalid());
                }
            }
            self.verify_complete()?;
            Ok(HeldInputsV2 { builder: self })
        })();
        shared.phase(result, Stage::RetainPrepared)
    }
    /// Recheck both actual roots, every fixed/hidden present/absence and every directory.
    fn verify_complete(&self) -> WorkResult<()> {
        let result = self.verify_complete_inner();
        self.shared.phase(result, Stage::RetainPrepared)
    }
    /// Complete actual native traversal, with ordinary failure retained by the caller.
    fn verify_complete_inner(&self) -> WorkResult<()> {
        self.shared.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
        for root in &self.roots {
            self.shared.charge(1)?;
            let envelope =
                self.shared.native_geometry(fresh::root_envelope(root.native().path()))?;
            let ticket = reserve_native(&self.shared, &envelope, 0)?;
            let result = fresh::verify_root(root.native());
            self.shared.native(result)?;
            drop(ticket);
        }
        for file in &self.files {
            self.shared.charge(1)?;
            let native = file.native.as_ref().ok_or_else(invalid)?;
            let LocalState::Read(native) = native else {
                return Err(invalid());
            };
            let envelope = self.shared.native_geometry(fresh::descendant_envelope(
                file.root.native(),
                native.relative(),
            ))?;
            let ticket = reserve_native(&self.shared, &envelope, 0)?;
            match native {
                CapturedLocal::Present(bytes, generation) => {
                    let shared = Rc::clone(&self.shared);
                    let result = fresh::verify_file_mcp_admitted(generation, bytes, &mut || {
                        shared.charge(1)
                    });
                    self.shared.read(result)?;
                }
                CapturedLocal::Absent(generation) => {
                    let result = fresh::verify_absence(generation);
                    self.shared.native(result)?;
                }
            }
            drop(ticket);
        }
        for directory in &self.directories {
            self.shared.charge(1)?;
            let native = directory.native.as_ref().ok_or_else(invalid)?;
            let envelope = self.shared.native_geometry(fresh::descendant_envelope(
                directory.root.native(),
                native.relative(),
            ))?;
            let ticket = reserve_native(&self.shared, &envelope, 0)?;
            let result = fresh::verify_directory(native);
            self.shared.native(result)?;
            drop(ticket);
        }
        Ok(())
    }
}

/// Render only the complete actual raw digest, admitting each borrowed chunk/output first.
fn hash_raw<C: WorkControl + ?Sized>(
    shared: &SharedWork<'_, C>,
    bytes: &[u8],
) -> WorkResult<String> {
    shared.charge(1)?;
    let mut digest = Sha256::new();
    for chunk in bytes.chunks(CHUNK) {
        shared.charge(1)?;
        digest.update(chunk);
    }
    shared.charge(1)?;
    let result = crate::hashing::lower_hex(&digest.finalize());
    shared.checkpoint(Stage::CaptureResource, ProgressUpdate::Unchanged)?;
    Ok(result)
}
/// Only a real held member creates this borrowed plain raw observation.
fn observe(file: &HeldFile) -> Option<Observation<'_>> {
    match file.native.as_ref()? {
        LocalState::Read(CapturedLocal::Present(bytes, _)) => {
            Some(Observation::Present { bytes, raw_sha256: file.hash.as_deref()? })
        }
        LocalState::Read(CapturedLocal::Absent(_)) => Some(Observation::Absent),
        LocalState::Opened(_) => None,
    }
}
impl<'control, C: WorkControl + ?Sized> HeldInputsV2<'control, C> {
    /// Lend only the retained concrete controller for sequential publication operations.
    /// No owner/admission method may reenter while this callback's mutable borrow is live.
    /// Actual caller publication/terminal methods retain their own stop/output semantics.
    pub(super) fn with_control<T>(&self, use_control: impl FnOnce(&mut C) -> T) -> T {
        let mut control = self.builder.shared.control.borrow_mut();
        use_control(&mut **control)
    }
    /// Visit EVERY actual declared original in registration order, with no subset sealer.
    /// Success requires all callback results; ordinary failure still performs the final fence.
    pub(super) fn visit_declared<'a>(
        &'a self,
        visit: &mut dyn FnMut(MemberV2<'a>) -> WorkResult<()>,
    ) -> WorkResult<()> {
        let result = (|| {
            for file in &self.builder.files {
                self.builder.shared.charge(1)?;
                let FileKey::Declared(key) = &file.key else {
                    continue;
                };
                let route = file.native.as_ref().ok_or_else(invalid)?.relative();
                let observation = observe(file).ok_or_else(invalid)?;
                visit(MemberV2 { key, route, observation })?;
            }
            Ok(())
        })();
        self.builder.shared.phase(result, Stage::PrepareDomain)
    }
    /// Visit EVERY actually held directory route, including hidden/unused declarations.
    /// The real generation stays private and alive under this same owner's final fences.
    pub(super) fn visit_directories<'a>(
        &'a self,
        visit: &mut dyn FnMut(&'a Path) -> WorkResult<()>,
    ) -> WorkResult<()> {
        let result = (|| {
            for directory in &self.builder.directories {
                self.builder.shared.charge(1)?;
                visit(directory.native.as_ref().ok_or_else(invalid)?.relative())?;
            }
            Ok(())
        })();
        self.builder.shared.phase(result, Stage::PrepareDomain)
    }
    /// Retain the same actual ledger/control through later codec/native/final acceptance.
    pub(super) fn admission(&self) -> Admission<'control, C> {
        self.builder.admission()
    }
    /// Borrow the same owner original, not caller-generated raw/proof data.
    pub(super) fn original(&self, key: &str) -> WorkResult<Option<Observation<'_>>> {
        self.builder.original(key)
    }
    /// Observe the actual selected entrypoint purpose; this grants no authority.
    pub(super) fn purpose(&self) -> CapturePurpose {
        self.builder.purpose
    }
    /// Recheck the complete owner before genuine higher-level result acceptance.
    pub(super) fn verify_complete(&self) -> WorkResult<()> {
        self.builder.verify_complete()
    }
}

/// Same-owner admitted complete physical generation; no native/current capability.
#[path = "capture_generation_v2.rs"]
mod generation;

/// Genuine cfg controls exercise the same native factories; zero execution implied.
#[cfg(test)]
#[path = "capture_v2_tests.rs"]
mod tests;
