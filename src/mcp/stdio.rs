//! Single-thread modern stdio dispatch over genuine captured query proofs.
//!
//! The sole thread owns stdin framing, query admission, every Rc/non-Send scope,
//! and stdout publication. Fences pump a bounded amount of ready input without
//! recursively dispatching queued work. Poll readiness never licenses read_line.
//! Windows admits observed synchronous byte-pipe input; other profiles refuse.
//! Native syscall cooperation and the sole-reader launch contract remain required.

use std::collections::{BTreeSet, VecDeque};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::catalog::{Catalog, DiscoverResult, Operation};
use super::disclosure::{self, DisclosureGate};
use super::queries::{self, Query, QueryError, Reason};
use super::wire::{self, Fault, Framer, Message, Rejection, RequestId, ToolResult};

/// Observed-mode Windows byte-pipe input; no reader thread or permissive fallback.
#[cfg(windows)]
#[allow(unsafe_code)] // Narrow native handle/buffer seam; each operation documents its ownership.
#[path = "windows_input.rs"]
mod windows_input;

use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

/// Immutable whole accepted request budget, including its original queue wait.
const QUERY_BUDGET: Duration = Duration::from_secs(10);
/// Maximum waiting accepted requests, separate from the one active slot.
const QUEUED_LIMIT: usize = 2;
/// Complete previously accepted ID roster; no eviction/reuse can revive late work.
const ID_LIMIT: usize = 1024;
/// Each pump examines at most this many actual raw bytes.
const READ_BURST: usize = 4096;
/// Each pump decodes at most this many records, preventing notification-flood loops.
const FRAME_BURST: usize = 8;
/// Idle wait is bounded so the transport remains owned without a blocked reader.
const IDLE_WAIT: Duration = Duration::from_millis(100);

/// Worker-private monotonic time observation; no client or startup option supplies it.
trait Clock {
    /// Observe monotonic time without renewing any stored accepted deadline.
    fn now(&self) -> Instant;
}

/// Actual production monotonic clock; deterministic controls use a private alternative.
struct SystemClock;
impl Clock for SystemClock {
    /// Return the real monotonic observation used for production admission and fences.
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Raw operator startup selection; no field is a detached approval/capture proof.
pub(crate) struct Startup {
    /// Original explicit project-root spelling; gate validates it before transforms.
    pub(crate) project_root: PathBuf,
    /// Original external decision-root spelling, or absent static-only selection.
    pub(crate) decision_root: Option<PathBuf>,
    /// Exact operator-selected decision raw pin; not human authentication.
    pub(crate) decision_sha256: Option<String>,
    /// Exact operator-selected visibility profile raw pin; not owner authority.
    pub(crate) profile_sha256: String,
}

/// Bounded driver observations; no read result itself authorizes project disclosure.
pub(crate) enum InputRead {
    /// Number of bytes actually installed into the caller's bounded destination.
    Data(usize),
    /// No bytes ready within the selected bounded wait.
    Idle,
    /// Actual input EOF; all accepted work is retired without success publication.
    Eof,
}

/// Root owns the native input driver; generic synthetic ports permit deterministic controls.
pub(crate) trait InputPort {
    /// Read at most `destination.len()` ready bytes; never wait for a complete line.
    /// A platform driver must not spawn or detach a blocked reader thread.
    fn read_ready(&mut self, wait: Duration, destination: &mut [u8]) -> io::Result<InputRead>;
}

/// Fixed runtime outcome; private OS/domain/serde error strings are never serialized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RunError {
    /// Fixed shipped catalog/schema construction could not be admitted.
    Catalog,
    /// Input driver failed; no detached thread remains running.
    Input,
    /// Complete-buffer write/flush failed; a prefix may already be nonretractable.
    Output,
    /// Complete response schema or encoded bound could not be admitted.
    Encoding,
    /// A previously accepted ID was repeated; no competing response is published.
    DuplicateId,
    /// Required native driver is not implemented on this platform.
    #[cfg(not(any(unix, windows)))]
    UnsupportedPlatform,
}

/// Accepted complete operation owns only bounded arguments and its original deadline.
struct Accepted {
    /// Exact safely admitted correlation ID.
    id: RequestId,
    /// Complete method-specific typed selection, with no authority proof.
    operation: Operation,
    /// Absolute accepted instant plus ten seconds; dequeuing never renews it.
    deadline: Instant,
}

/// One exact active slot, including a request admitted but not yet dispatched.
struct Active {
    /// ID currently owned by the sole query worker.
    id: RequestId,
    /// Original accepted deadline, shared by every cooperative fence.
    deadline: Instant,
    /// First observed typed stop; no later error conversion erases it.
    stop: Option<Interruption>,
    /// A received cancellation independently suppresses any later terminal response.
    cancelled: bool,
}

/// Single-owner bounded protocol state, not a project/approval/session capability.
#[derive(Default)]
struct Machine {
    /// First admitted operation occupies the active slot before its next dispatch.
    ready: Option<Accepted>,
    /// At most two complete waiting requests; no detached processing occurs here.
    queued: VecDeque<Accepted>,
    /// Exact active identity and sticky stop through borrowed scope destruction.
    active: Option<Active>,
    /// Complete accepted ID history; capacity refuses rather than evicting/reusing.
    ids: BTreeSet<RequestId>,
    /// EOF/input/output failure prevents all new accepted/successful work.
    shutdown: bool,
}

impl Machine {
    /// Admit capacity/uniqueness before growing any retained ID/operation container.
    fn accept(&mut self, id: RequestId, operation: Operation, now: Instant) -> Result<(), Fault> {
        if self.shutdown || self.ids.contains(&id) {
            return Err(Fault::Request);
        }
        if self.ids.len() >= ID_LIMIT
            || (self.active.is_some() && self.queued.len() >= QUEUED_LIMIT)
        {
            return Err(Fault::Capacity);
        }
        let deadline = now.checked_add(QUERY_BUDGET).ok_or(Fault::Internal)?;
        let accepted = Accepted { id: id.clone(), operation, deadline };
        self.ids.insert(id.clone());
        if self.active.is_none() {
            self.active = Some(Active { id, deadline, stop: None, cancelled: false });
            self.ready = Some(accepted);
        } else {
            self.queued.push_back(accepted);
        }
        Ok(())
    }
    /// Take the reserved active operation or promote one waiting original deadline.
    fn next(&mut self) -> Option<Accepted> {
        if self.shutdown {
            return None;
        }
        if let Some(ready) = self.ready.take() {
            return Some(ready);
        }
        if self.active.is_some() {
            return None;
        }
        let accepted = self.queued.pop_front()?;
        self.active = Some(Active {
            id: accepted.id.clone(),
            deadline: accepted.deadline,
            stop: None,
            cancelled: false,
        });
        Some(accepted)
    }
    /// Stop known active work or remove known queued work; unknown/late IDs do nothing.
    fn cancel(&mut self, id: &RequestId) {
        if let Some(active) = self.active.as_mut().filter(|active| &active.id == id) {
            active.cancelled = true;
            active.stop.get_or_insert(Interruption::CancelRequested);
            return;
        }
        if let Some(index) = self.queued.iter().position(|request| &request.id == id) {
            self.queued.remove(index);
        }
    }
    /// Latch only the first stop, preserving cancellation even through EOF cleanup.
    fn stop(&mut self, now: Instant) -> Option<Interruption> {
        let active = self.active.as_mut()?;
        if active.stop.is_none() {
            if self.shutdown {
                active.stop = Some(Interruption::Shutdown);
            } else if now >= active.deadline {
                active.stop = Some(Interruption::DeadlineExceeded);
            }
        }
        active.stop
    }
    /// EOF/fatal IO retires complete pending work and releases the partial frame elsewhere.
    fn shutdown(&mut self) {
        self.shutdown = true;
        self.ready = None;
        self.queued.clear();
        if let Some(active) = &mut self.active {
            active.stop.get_or_insert(Interruption::Shutdown);
        }
    }
    /// Retire active state only after the actual borrowed prepared/scope stack has unwound.
    fn finish(&mut self, id: &RequestId) {
        if self.active.as_ref().is_some_and(|active| &active.id == id) {
            self.active = None;
        }
    }
    /// Cancellation/EOF never becomes a successful or terminal request response.
    fn suppress(&self) -> bool {
        self.shutdown || self.active.as_ref().is_some_and(|active| active.cancelled)
    }
}

/// Bounded input/framing and complete stdout writer owned by the same worker.
struct Pump<'a, P: InputPort, W: Write> {
    /// Actual platform or controlled fixture input, never project bytes.
    input: P,
    /// Sole protocol output; no domain logger writes to it.
    output: W,
    /// Static complete offline tool schemas for actual closed admission.
    catalog: &'a Catalog,
    /// Actual private monotonic observer, never a caller-authority input.
    clock: &'a dyn Clock,
    /// Complete shared accepted/cancelled/EOF state.
    machine: Machine,
    /// At most one bounded partial raw LF record.
    framer: Framer,
    /// Fixed intake buffer; unconsumed bytes survive a finite frame-burst fence.
    intake: [u8; READ_BURST],
    /// First not-yet-consumed intake byte.
    start: usize,
    /// Exclusive last actual installed intake byte.
    end: usize,
}

impl<P: InputPort, W: Write> Pump<'_, P, W> {
    /// Pump at most one read burst/eight frames, never a full-line blocking read.
    fn service(&mut self, wait: Duration) -> Result<(), RunError> {
        if self.machine.shutdown {
            return Ok(());
        }
        if self.start == self.end {
            match self.input.read_ready(wait, &mut self.intake).map_err(|_| RunError::Input)? {
                InputRead::Idle => return Ok(()),
                InputRead::Eof => {
                    self.machine.shutdown();
                    self.framer.clear();
                    return Ok(());
                }
                InputRead::Data(count) if count > 0 && count <= self.intake.len() => {
                    self.start = 0;
                    self.end = count;
                }
                InputRead::Data(_) => {
                    self.machine.shutdown();
                    return Err(RunError::Input);
                }
            }
        }
        let mut frames = 0;
        let mut bytes = 0;
        while self.start < self.end && frames < FRAME_BURST && bytes < READ_BURST {
            let byte = self.intake[self.start];
            self.start += 1;
            bytes += 1;
            if let Some(frame) = self.framer.push(byte) {
                frames += 1;
                match frame {
                    Ok(bytes) => match wire::parse(&bytes) {
                        Ok(message) => self.message(message)?,
                        Err(error) => self.reject(&error)?,
                    },
                    Err(fault) => self.reject(&Rejection { fault, id: None, requested: None })?,
                }
            }
        }
        Ok(())
    }
    /// Queue one admitted request or apply cancellation without recursive dispatch.
    fn message(&mut self, message: Message) -> Result<(), RunError> {
        match message {
            Message::Ignore => Ok(()),
            Message::Cancel(id) => {
                self.machine.cancel(&id);
                Ok(())
            }
            Message::Request(mut request) => {
                let id = request.id.clone();
                if self.machine.ids.contains(&id) {
                    self.machine.shutdown();
                    return Err(RunError::DuplicateId);
                }
                let admitted = self.catalog.selected_admit(&mut request).and_then(|operation| {
                    self.machine.accept(id.clone(), operation, self.clock.now())
                });
                match admitted {
                    Ok(()) => Ok(()),
                    Err(fault) => self.reject(&Rejection { fault, id: Some(id), requested: None }),
                }
            }
        }
    }
    /// Publish only a fixed complete protocol failure, never an invalid notification.
    fn reject(&mut self, error: &Rejection) -> Result<(), RunError> {
        if error.id.as_ref().is_some_and(|id| self.machine.ids.contains(id)) {
            self.machine.shutdown();
            return Err(RunError::DuplicateId);
        }
        let encoded = wire::error(error).map_err(|_| RunError::Encoding)?;
        self.write_complete(&encoded)
    }
    /// Write a fully admitted buffer; faults may leave a nonretractable prefix.
    fn write_complete(&mut self, bytes: &[u8]) -> Result<(), RunError> {
        if bytes.len() > wire::MAX_RESPONSE || bytes.last() != Some(&b'\n') {
            return Err(RunError::Encoding);
        }
        if self.output.write_all(bytes).and_then(|()| self.output.flush()).is_err() {
            self.machine.shutdown();
            return Err(RunError::Output);
        }
        Ok(())
    }
}

/// Same immutable control through queue wait, capture, domain, encoding and publication.
struct Control<'a, 'b, P: InputPort, W: Write> {
    /// Sole mutable IO/admission owner; no Rc scope crosses a thread or callback.
    pump: &'a mut Pump<'b, P, W>,
    /// First actual IO failure; checked by the outer runner after stack cleanup.
    io_failure: Option<RunError>,
    /// Final proof/publication phase forbids parsing/queued-error writes after its input observation.
    quiescent: bool,
}

impl<P: InputPort, W: Write> Control<'_, '_, P, W> {
    /// Observe fixed deadline then finite ready input and retain the first typed stop.
    fn observe(&mut self) -> Option<Interruption> {
        self.pump.machine.stop(self.pump.clock.now());
        if !self.quiescent && self.io_failure.is_none() {
            if let Err(error) = self.pump.service(Duration::ZERO) {
                self.io_failure = Some(error);
                self.pump.machine.shutdown();
            }
        }
        self.pump.machine.stop(self.pump.clock.now())
    }
    /// Pump ready input before the complete original fence, then forbid parser/other writes.
    /// New arrivals after this sequential observation are not an atomic cancellation snapshot.
    fn enter_publication(&mut self) -> WorkResult<()> {
        if let Some(reason) = self.observe() {
            return Err(WorkError::Interrupted(reason));
        }
        self.quiescent = true;
        Ok(())
    }
    /// Final safe publication fence for actual complete trusted data.
    fn publish(&mut self, bytes: &[u8]) -> Result<Option<Interruption>, RunError> {
        if let Some(reason) = self.observe() {
            return Ok(Some(reason));
        }
        if self.pump.machine.suppress() {
            return Ok(Some(Interruption::Shutdown));
        }
        self.pump.write_complete(bytes)?;
        Ok(None)
    }
    /// Fixed null-data deadline failure is permitted after deadline, never cancel/EOF.
    fn terminal(&mut self, bytes: &[u8]) -> Result<(), RunError> {
        self.observe();
        if self.pump.machine.suppress() {
            return Ok(());
        }
        self.pump.write_complete(bytes)
    }
}

impl<P: InputPort, W: Write> WorkControl for Control<'_, '_, P, W> {
    /// Pump finite ready input and preserve the same original absolute request deadline.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        self.observe().map_or(Ok(()), |reason| Err(WorkError::Interrupted(reason)))
    }
    /// Return the exact previously latched stop without additional IO or interpretation.
    fn interruption(&self) -> Option<Interruption> {
        self.pump.machine.active.as_ref().and_then(|active| active.stop)
    }
}

/// Execute all accepted work on this thread; scope creation and destruction stay here.
pub(crate) fn run<P: InputPort, W: Write>(
    startup: &Startup,
    input: P,
    output: W,
) -> Result<(), RunError> {
    run_with_clock(startup, input, output, &SystemClock)
}

/// Private consumed worker body also permits actual deterministic deadline controls.
fn run_with_clock<P: InputPort, W: Write>(
    startup: &Startup,
    input: P,
    output: W,
    clock: &dyn Clock,
) -> Result<(), RunError> {
    let catalog = Catalog::new().map_err(|_| RunError::Catalog)?;
    let mut pump = Pump {
        input,
        output,
        catalog: &catalog,
        clock,
        machine: Machine::default(),
        framer: Framer::default(),
        intake: [0; READ_BURST],
        start: 0,
        end: 0,
    };
    loop {
        if pump.machine.shutdown {
            return Ok(());
        }
        if let Some(accepted) = pump.machine.next() {
            let id = accepted.id.clone();
            let mut control = Control { pump: &mut pump, io_failure: None, quiescent: false };
            let result = execute(startup, &catalog, accepted, &mut control);
            let failure = control.io_failure;
            // execute has returned: every PreparedQuery and scope is now actually dropped.
            control.pump.machine.finish(&id);
            if let Some(error) = failure {
                return Err(error);
            }
            result?;
        } else if let Err(error) = pump.service(IDLE_WAIT) {
            pump.machine.shutdown();
            return Err(error);
        }
    }
}

/// Dispatch only after full wire/query admission; no static operation opens project roots.
fn execute<P: InputPort, W: Write>(
    startup: &Startup,
    catalog: &Catalog,
    accepted: Accepted,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    let id = accepted.id;
    match accepted.operation {
        Operation::Discover => {
            if control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).is_err() {
                return static_stop(&id, control);
            }
            let encoded =
                wire::success(&id, &DiscoverResult::new()).map_err(|_| RunError::Encoding)?;
            if control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).is_err() {
                return static_stop(&id, control);
            }
            publish_static(&id, &encoded, control)
        }
        Operation::List => {
            if control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged).is_err() {
                return static_stop(&id, control);
            }
            let encoded = wire::success(&id, &catalog.list()).map_err(|_| RunError::Encoding)?;
            if control.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged).is_err() {
                return static_stop(&id, control);
            }
            publish_static(&id, &encoded, control)
        }
        Operation::Call(query) => call(startup, catalog, &id, query, control),
    }
}

/// Select a fixed deadline response when the final static publication fence stops.
fn publish_static<P: InputPort, W: Write>(
    id: &RequestId,
    encoded: &[u8],
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    match control.publish(encoded)? {
        None => Ok(()),
        Some(_) => static_stop(id, control),
    }
}

/// Emit a fixed static timeout only; cancellation/EOF produce no request response.
fn static_stop<P: InputPort, W: Write>(
    id: &RequestId,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    if control.interruption() != Some(Interruption::DeadlineExceeded) {
        return Ok(());
    }
    let encoded =
        wire::error(&Rejection { fault: Fault::Deadline, id: Some(id.clone()), requested: None })
            .map_err(|_| RunError::Encoding)?;
    control.terminal(&encoded)
}

/// Load a real gate per request, preserving operator raw spellings and all actual originals.
#[allow(clippy::needless_pass_by_value)] // This worker owns and retires the accepted query arguments with the actual scope.
fn call<P: InputPort, W: Write>(
    startup: &Startup,
    catalog: &Catalog,
    id: &RequestId,
    query: Query,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    let gate = disclosure::load_disclosure_decision(
        &startup.project_root,
        startup.decision_root.as_deref(),
        startup.decision_sha256.as_deref(),
        &startup.profile_sha256,
        control,
    );
    match gate {
        Ok(DisclosureGate::Unavailable) => {
            unavailable(catalog, id, &query, Reason::ApprovalUnavailable, control, false)
        }
        Err(error) => stopped(catalog, id, &query, error, control),
        Ok(DisclosureGate::Validated(scope)) => {
            match queries::prepare(&scope, query.clone(), control) {
                Ok(prepared) => {
                    let Ok(encoded) =
                        wire::success(id, &ToolResult::new(prepared.response(), false))
                    else {
                        return unavailable(
                            catalog,
                            id,
                            &query,
                            Reason::OutputBoundExceeded,
                            control,
                            false,
                        );
                    };
                    if catalog.validate_encoded(query.tool(), &encoded).is_err() {
                        return unavailable(
                            catalog,
                            id,
                            &query,
                            Reason::InvalidArtifact,
                            control,
                            false,
                        );
                    }
                    match control.enter_publication().and_then(|()| prepared.verify_inputs(control))
                    {
                        Ok(()) => match control.publish(&encoded)? {
                            None => Ok(()),
                            Some(reason) => stopped(
                                catalog,
                                id,
                                &query,
                                WorkError::Interrupted(reason),
                                control,
                            ),
                        },
                        Err(error) => {
                            control.quiescent = false;
                            stopped(catalog, id, &query, error, control)
                        }
                    }
                }
                Err(QueryError::Work(error)) => stopped(catalog, id, &query, error, control),
                Err(QueryError::Unavailable(reason)) => {
                    // Unavailability has no trusted partial data, but an admitted scope still fences.
                    match scope.verify_inputs(control) {
                        Ok(()) => unavailable(catalog, id, &query, reason, control, false),
                        Err(error) => stopped(catalog, id, &query, error, control),
                    }
                }
            }
        }
    }
}

/// Preserve typed cancellation/shutdown, with only fixed null-data timeout/failure results.
#[allow(clippy::needless_pass_by_value)] // Consume the actual typed stop after the borrowed preparation has unwound.
fn stopped<P: InputPort, W: Write>(
    catalog: &Catalog,
    id: &RequestId,
    query: &Query,
    error: WorkError,
    control: &mut Control<'_, '_, P, W>,
) -> Result<(), RunError> {
    match error {
        WorkError::Interrupted(Interruption::CancelRequested | Interruption::Shutdown) => Ok(()),
        WorkError::Interrupted(Interruption::DeadlineExceeded) => {
            unavailable(catalog, id, query, Reason::QueryBudgetExceeded, control, true)
        }
        WorkError::Failed(_) => {
            unavailable(catalog, id, query, Reason::ApprovalUnavailable, control, false)
        }
    }
}

/// Fixed closed tool-domain failure is wrapped and schema checked under the whole wire cap.
fn unavailable<P: InputPort, W: Write>(
    catalog: &Catalog,
    id: &RequestId,
    query: &Query,
    reason: Reason,
    control: &mut Control<'_, '_, P, W>,
    terminal: bool,
) -> Result<(), RunError> {
    let response = queries::unavailable(query, reason);
    let encoded =
        wire::success(id, &ToolResult::new(&response, true)).map_err(|_| RunError::Encoding)?;
    catalog.validate_encoded(query.tool(), &encoded).map_err(|_| RunError::Encoding)?;
    if terminal {
        return control.terminal(&encoded);
    }
    match control.publish(&encoded)? {
        None | Some(Interruption::CancelRequested | Interruption::Shutdown) => Ok(()),
        Some(Interruption::DeadlineExceeded) => {
            let timeout = queries::unavailable(query, Reason::QueryBudgetExceeded);
            let encoded = wire::success(id, &ToolResult::new(&timeout, true))
                .map_err(|_| RunError::Encoding)?;
            catalog.validate_encoded(query.tool(), &encoded).map_err(|_| RunError::Encoding)?;
            control.terminal(&encoded)
        }
    }
}

/// Actual Unix stdin driver borrows the process-owned descriptor without changing flags.
#[cfg(unix)]
struct UnixInput {
    /// Sole process-owned stdin descriptor; Root must not install another reader.
    fd: libc::c_int,
}
#[cfg(unix)]
#[allow(unsafe_code)] // Narrow libc poll/read FFI; both calls document buffer and ownership safety.
impl InputPort for UnixInput {
    /// Poll only bounded wait then read one bounded ready chunk, never a full line.
    fn read_ready(&mut self, wait: Duration, destination: &mut [u8]) -> io::Result<InputRead> {
        let millis = i32::try_from(wait.as_millis().min(i32::MAX as u128))
            .map_err(|_| io::Error::other("MCP input"))?;
        let mut descriptor = libc::pollfd { fd: self.fd, events: libc::POLLIN, revents: 0 };
        // SAFETY: one initialized pollfd is valid for this bounded synchronous call;
        // stdin ownership remains in this thread and poll does not transfer/close it.
        let ready = unsafe { libc::poll(&raw mut descriptor, 1, millis) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            return if error.kind() == io::ErrorKind::Interrupted {
                Ok(InputRead::Idle)
            } else {
                Err(error)
            };
        }
        if ready == 0 {
            return Ok(InputRead::Idle);
        }
        if descriptor.revents & (libc::POLLERR | libc::POLLNVAL) != 0 {
            return Err(io::Error::other("MCP input"));
        }
        if descriptor.revents & (libc::POLLIN | libc::POLLHUP) == 0 {
            return Ok(InputRead::Idle);
        }
        // SAFETY: destination is a live writable slice for exactly its supplied length;
        // sole reader ownership prevents another thread from consuming the ready bytes.
        let count =
            unsafe { libc::read(self.fd, destination.as_mut_ptr().cast(), destination.len()) };
        if count < 0 {
            let error = io::Error::last_os_error();
            return if matches!(error.kind(), io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock)
            {
                Ok(InputRead::Idle)
            } else {
                Err(error)
            };
        }
        if count == 0 {
            Ok(InputRead::Eof)
        } else {
            usize::try_from(count).map(InputRead::Data).map_err(|_| io::Error::other("MCP input"))
        }
    }
}

/// The MCP CLI consumes this native input driver on the sole query thread.
/// Blocking parser/native/OS/stdout work is not preempted; writes can expose a prefix.
#[cfg(unix)]
pub(crate) fn run_stdio(startup: &Startup) -> Result<(), RunError> {
    let stdout = io::stdout();
    run(startup, UnixInput { fd: libc::STDIN_FILENO }, stdout.lock())
}

/// Consume observed synchronous Windows byte-pipe input on the sole captured-query thread.
#[cfg(windows)]
pub(crate) fn run_stdio(startup: &Startup) -> Result<(), RunError> {
    let input = windows_input::WindowsInput::stdin().map_err(|_| RunError::Input)?;
    let stdout = io::stdout();
    run(startup, input, stdout.lock())
}

/// No successful stub exists on platforms without a native input implementation.
#[cfg(not(any(unix, windows)))]
pub(crate) fn run_stdio(_startup: &Startup) -> Result<(), RunError> {
    Err(RunError::UnsupportedPlatform)
}

/// Deterministic controls with separate scripted, captured-domain and native-input scopes.
#[cfg(test)]
#[path = "stdio_tests.rs"]
mod tests;

/// Additive family dispatch; /1 remains the default and consumes its unchanged worker.
#[path = "stdio_v2.rs"]
mod v2;

/// Select the declaration family before any project capture starts.
pub(crate) fn run_selected<P: InputPort, W: Write>(
    startup: &Startup,
    family: crate::cli::McpDeclarationFamily,
    input: P,
    output: W,
) -> Result<(), RunError> {
    match family {
        crate::cli::McpDeclarationFamily::V1 => run(startup, input, output),
        crate::cli::McpDeclarationFamily::V2 => v2::run(startup, input, output),
    }
}
/// Same native sole-reader Unix driver, with family selection before dispatch.
#[cfg(unix)]
pub(crate) fn run_stdio_selected(
    startup: &Startup,
    family: crate::cli::McpDeclarationFamily,
) -> Result<(), RunError> {
    let stdout = io::stdout();
    run_selected(startup, family, UnixInput { fd: libc::STDIN_FILENO }, stdout.lock())
}
/// Same observed synchronous Windows byte-pipe driver, without a reader thread.
#[cfg(windows)]
pub(crate) fn run_stdio_selected(
    startup: &Startup,
    family: crate::cli::McpDeclarationFamily,
) -> Result<(), RunError> {
    let input = windows_input::WindowsInput::stdin().map_err(|_| RunError::Input)?;
    let stdout = io::stdout();
    run_selected(startup, family, input, stdout.lock())
}
/// No permissive success exists for platforms without a maintained native input driver.
#[cfg(not(any(unix, windows)))]
pub(crate) fn run_stdio_selected(
    _startup: &Startup,
    _family: crate::cli::McpDeclarationFamily,
) -> Result<(), RunError> {
    Err(RunError::UnsupportedPlatform)
}
