//! Single-owner Windows byte-pipe stdin with observed synchronous native I/O mode.
//!
//! No console/file/message fallback or reader thread is installed. Mode queries
//! that remain pending refuse startup while their one owned allocation is kept
//! alive; this is not completion, cancellation, or a platform qualification.

use std::ffi::c_void;
use std::io;
use std::marker::PhantomData;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle, RawHandle};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use super::{IDLE_WAIT, InputPort, InputRead, READ_BURST};

/// A process launch may inspect stdin only once, bounding a pending query retention.
static STDIN_ATTEMPTED: AtomicBool = AtomicBool::new(false);
/// Actual byte-pipe file type; other native stdin types are unsupported.
const FILE_TYPE_PIPE: u32 = 3;
/// Named-pipe type information bit denoting message storage.
const PIPE_TYPE_MESSAGE: u32 = 4;
/// Named-pipe endpoint flag permitted by the native info query.
const PIPE_SERVER_END: u32 = 1;
/// Native read-state bit permitting a nonblocking byte-pipe state.
const PIPE_NOWAIT: u32 = 1;
/// Only a verified broken pipe is classified as EOF from a failed native operation.
const ERROR_BROKEN_PIPE: i32 = 109;
/// Native mode-information class selected by ROOT's primary-source review.
const FILE_MODE_INFORMATION: i32 = 16;
/// Exact completed native status; informational statuses are not completion proof.
const STATUS_SUCCESS: i32 = 0;
/// A pending query keeps both output buffers and its owned handle alive.
const STATUS_PENDING: i32 = 0x103;
/// Synchronous alertable I/O flag in the returned native file mode.
const SYNCHRONOUS_ALERT: u32 = 0x10;
/// Synchronous nonalertable I/O flag in the returned native file mode.
const SYNCHRONOUS_NONALERT: u32 = 0x20;

/// C-compatible status/pointer union used by the native completion record.
#[repr(C)]
union IoStatus {
    /// Native final status, read only after an exact successful query return.
    status: i32,
    /// Pointer-sized alternative supplies the native union layout and alignment.
    pointer: *mut c_void,
}

/// Native query completion record; pointer width governs its information field.
#[repr(C)]
struct IoStatusBlock {
    /// Final status or native pointer union; pending is never treated as complete.
    status: IoStatus,
    /// Exact initialized output size, required to equal one `ULONG`.
    information: usize,
}

/// Complete class16 output; no launch flag is inferred from pipe type alone.
#[repr(C)]
struct FileModeInformation {
    /// Observed native I/O flags; exactly one synchronous bit is required.
    mode: u32,
}

/// Stable allocation retaining the actual handle and both native output buffers.
struct QueryStorage<H> {
    /// Owned handle cannot be dropped while an uncertain native query may use it.
    handle: H,
    /// Heap-stable status output; never a pointer to a freed stack temporary.
    status: IoStatusBlock,
    /// Heap-stable native mode output admitted only after exact completion.
    mode: FileModeInformation,
}

/// Sticky input settlement; no later observation can revive EOF or failure.
#[derive(Clone, Copy)]
enum StreamState {
    /// The admitted held byte pipe may be checked for ready bytes.
    Open,
    /// Genuine EOF permanently retires this input.
    Eof,
    /// A native or local invariant failure permanently retires this input.
    Failed,
}

/// Sole worker's held pipe; it grants no project disclosure or peer trust.
pub(super) struct WindowsInput {
    /// Noninheritable duplicate pins the original input object for this worker.
    handle: OwnedHandle,
    /// First EOF/failure is retained independently of later handle observations.
    state: StreamState,
    /// Prevent transport ownership from being sent/shared across worker threads.
    thread_owner: PhantomData<Rc<()>>,
}

impl WindowsInput {
    /// Capture one actual stdin duplicate without changing the original handle or modes.
    pub(super) fn stdin() -> io::Result<Self> {
        claim_startup(&STDIN_ATTEMPTED)?;
        // SAFETY: the native calls have no borrowed output beyond this call;
        // successful duplication is immediately installed into one owned handle.
        let original = unsafe { native::GetStdHandle(u32::MAX - 9) };
        if original.is_null() || original == (-1_isize) as RawHandle {
            return Err(input_error());
        }
        let mut duplicate = std::ptr::null_mut();
        let process = unsafe { native::GetCurrentProcess() };
        let result = unsafe {
            native::DuplicateHandle(process, original, process, &raw mut duplicate, 0, 0, 2)
        };
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        if duplicate.is_null() || duplicate == (-1_isize) as RawHandle {
            return Err(input_error());
        }
        // SAFETY: successful DuplicateHandle transferred one valid noninheritable
        // handle, independent of the borrowed original process standard handle.
        let handle = unsafe { OwnedHandle::from_raw_handle(duplicate) };
        Self::from_pipe(handle)
    }

    /// Admit observed byte-pipe metadata and complete synchronous mode before any read.
    fn from_pipe(handle: OwnedHandle) -> io::Result<Self> {
        let mut flags = 0;
        let mut state = 0;
        // SAFETY: the actual owned handle is retained through both native queries;
        // each non-null output points to one initialized writable DWORD.
        let file_type = unsafe { native::GetFileType(handle.as_raw_handle()) };
        if file_type != FILE_TYPE_PIPE {
            return Err(input_error());
        }
        let info = unsafe {
            native::GetNamedPipeInfo(
                handle.as_raw_handle(),
                &raw mut flags,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if info == 0 {
            return Err(io::Error::last_os_error());
        }
        let read_state = unsafe {
            native::GetNamedPipeHandleStateW(
                handle.as_raw_handle(),
                &raw mut state,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        };
        if read_state == 0 {
            return Err(io::Error::last_os_error());
        }
        if !byte_pipe(file_type, flags, state) {
            return Err(input_error());
        }
        let handle = qualify_mode_with(handle, |raw, status, mode| {
            // SAFETY: both outputs belong to a live heap allocation retained
            // until exact completion, or intentionally kept alive if pending.
            unsafe {
                native::NtQueryInformationFile(raw, status, mode.cast(), 4, FILE_MODE_INFORMATION)
            }
        })?;
        Ok(Self { handle, state: StreamState::Open, thread_owner: PhantomData })
    }

    /// Classify only a verified broken pipe as sticky EOF; every other error fails closed.
    fn native_failure(&mut self, error: io::Error) -> io::Result<InputRead> {
        if error.raw_os_error() == Some(ERROR_BROKEN_PIPE) {
            self.state = StreamState::Eof;
            Ok(InputRead::Eof)
        } else {
            self.state = StreamState::Failed;
            Err(error)
        }
    }

    /// Consume exactly one ready synchronous chunk; actual count cannot exceed its request.
    fn read_chunk(&mut self, destination: &mut [u8], available: u32) -> io::Result<InputRead> {
        let bound = destination.len().min(READ_BURST).min(available as usize);
        let requested = u32::try_from(bound).map_err(|_| input_error())?;
        if requested == 0 {
            self.state = StreamState::Failed;
            return Err(input_error());
        }
        let mut count = 0;
        // SAFETY: class16 proved synchronous mode; the held byte-pipe handle and
        // live destination remain owned for this call. Sole reader is a required
        // launch contract. Peek availability and this cap do not preempt a syscall.
        let result = unsafe {
            native::ReadFile(
                self.handle.as_raw_handle(),
                destination.as_mut_ptr(),
                requested,
                &raw mut count,
                std::ptr::null_mut(),
            )
        };
        let completion = if result == 0 { Err(io::Error::last_os_error()) } else { Ok(count) };
        self.complete_read(requested, completion)
    }

    /// Settle actual completed-read facts without inferring peer closure from a zero count.
    /// Successful zero-byte pipe reads remain open and idle; only broken-pipe failure is EOF.
    /// This consumed port also admits synthetic completion facts in explicitly labeled controls.
    fn complete_read(
        &mut self,
        requested: u32,
        completion: io::Result<u32>,
    ) -> io::Result<InputRead> {
        match self.state {
            StreamState::Eof => return Ok(InputRead::Eof),
            StreamState::Failed => return Err(input_error()),
            StreamState::Open => {}
        }
        if requested == 0 || requested as usize > READ_BURST {
            self.state = StreamState::Failed;
            return Err(input_error());
        }
        let count = match completion {
            Ok(count) => count,
            Err(error) => return self.native_failure(error),
        };
        if count > requested {
            self.state = StreamState::Failed;
            return Err(input_error());
        }
        if count == 0 { Ok(InputRead::Idle) } else { Ok(InputRead::Data(count as usize)) }
    }
}

impl InputPort for WindowsInput {
    /// Peek the held pipe, wait finitely when idle, and read at most 4096 ready bytes once.
    fn read_ready(&mut self, wait: Duration, destination: &mut [u8]) -> io::Result<InputRead> {
        match self.state {
            StreamState::Eof => return Ok(InputRead::Eof),
            StreamState::Failed => return Err(input_error()),
            StreamState::Open => {}
        }
        if destination.is_empty() {
            self.state = StreamState::Failed;
            return Err(input_error());
        }
        let deadline = Instant::now().checked_add(wait.min(IDLE_WAIT)).ok_or_else(input_error)?;
        loop {
            let mut available = 0;
            // SAFETY: no data buffer is supplied; the held pipe and one DWORD
            // availability output remain live. No second reader exists in Forge.
            let result = unsafe {
                native::PeekNamedPipe(
                    self.handle.as_raw_handle(),
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null_mut(),
                    &raw mut available,
                    std::ptr::null_mut(),
                )
            };
            if result == 0 {
                return self.native_failure(io::Error::last_os_error());
            }
            if available != 0 {
                return self.read_chunk(destination, available);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(InputRead::Idle);
            }
            std::thread::sleep((deadline - now).min(Duration::from_millis(5)));
        }
    }
}

/// Claim one launch attempt before any handle copy/query, with no reset or lifetime renewal.
fn claim_startup(gate: &AtomicBool) -> io::Result<()> {
    if gate.swap(true, Ordering::SeqCst) { Err(input_error()) } else { Ok(()) }
}

/// Require byte storage/read mode; unknown flags and message semantics are refused.
fn byte_pipe(file_type: u32, flags: u32, state: u32) -> bool {
    file_type == FILE_TYPE_PIPE
        && flags & !(PIPE_TYPE_MESSAGE | PIPE_SERVER_END) == 0
        && flags & PIPE_TYPE_MESSAGE == 0
        && state & !PIPE_NOWAIT == 0
}

/// Exact native success/size plus one synchronous flag, never an `NT_SUCCESS` shortcut.
fn complete_synchronous(returned: i32, status: i32, information: usize, mode: u32) -> bool {
    let synchronous = mode & (SYNCHRONOUS_ALERT | SYNCHRONOUS_NONALERT);
    returned == STATUS_SUCCESS
        && status == STATUS_SUCCESS
        && information == 4
        && matches!(synchronous, SYNCHRONOUS_ALERT | SYNCHRONOUS_NONALERT)
}

/// Execute one heap-backed mode query, retaining uncertain pending storage without retry.
///
/// Production calls this only after the single stdin-attempt gate. `H` is an owned
/// handle in production; synthetic controls supply a counted owner without OS I/O.
fn qualify_mode_with<H: AsRawHandle + 'static>(
    handle: H,
    query: impl FnOnce(RawHandle, *mut IoStatusBlock, *mut FileModeInformation) -> i32,
) -> io::Result<H> {
    let mut storage = Box::new(QueryStorage {
        handle,
        status: IoStatusBlock {
            status: IoStatus { pointer: std::ptr::null_mut() },
            information: usize::MAX,
        },
        mode: FileModeInformation { mode: u32::MAX },
    });
    storage.status.status.status = STATUS_PENDING;
    let returned =
        query(storage.handle.as_raw_handle(), &raw mut storage.status, &raw mut storage.mode);
    if returned == STATUS_PENDING {
        // No status/mode read or drop follows a pending return. Both output
        // addresses and the handle remain valid until process teardown.
        let _retained = Box::leak(storage);
        return Err(input_error());
    }
    if returned != STATUS_SUCCESS {
        return Err(input_error());
    }
    // SAFETY: exact successful native return completed the output record.
    let status = unsafe { storage.status.status.status };
    if status == STATUS_PENDING {
        // An inconsistent completion record is also uncertainty, never success.
        let _retained = Box::leak(storage);
        return Err(input_error());
    }
    if !complete_synchronous(returned, status, storage.status.information, storage.mode.mode) {
        return Err(input_error());
    }
    let QueryStorage { handle, .. } = *storage;
    Ok(handle)
}

/// Fixed local driver classification; native private details never become protocol text.
fn input_error() -> io::Error {
    io::Error::other("MCP input profile is unavailable")
}

/// Narrow reviewed native ABI; no crate/SDK feature or managed source dependency is added.
mod native {
    use super::{IoStatusBlock, c_void};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        /// Borrow the process standard handle; this call transfers no ownership.
        pub(super) fn GetStdHandle(kind: u32) -> *mut c_void;
        /// Obtain the pseudo process handle used only by `DuplicateHandle`.
        pub(super) fn GetCurrentProcess() -> *mut c_void;
        /// Produce one noninheritable owned duplicate with the same access.
        pub(super) fn DuplicateHandle(
            source: *mut c_void,
            handle: *mut c_void,
            target: *mut c_void,
            duplicate: *mut *mut c_void,
            access: u32,
            inherit: i32,
            options: u32,
        ) -> i32;
        /// Observe native file type; only exact pipe type is admitted.
        pub(super) fn GetFileType(handle: *mut c_void) -> u32;
        /// Observe pipe type/end flags before any byte read.
        pub(super) fn GetNamedPipeInfo(
            handle: *mut c_void,
            flags: *mut u32,
            output: *mut u32,
            input: *mut u32,
            instances: *mut u32,
        ) -> i32;
        /// Observe read/wait mode without mutating inherited pipe state.
        pub(super) fn GetNamedPipeHandleStateW(
            handle: *mut c_void,
            state: *mut u32,
            instances: *mut u32,
            collection: *mut u32,
            timeout: *mut u32,
            user: *mut u16,
            user_size: u32,
        ) -> i32;
        /// Inspect buffered bytes; a successful zero availability is not EOF.
        pub(super) fn PeekNamedPipe(
            handle: *mut c_void,
            buffer: *mut c_void,
            size: u32,
            read: *mut u32,
            available: *mut u32,
            remaining: *mut u32,
        ) -> i32;
        /// Perform one bounded read only after exact synchronous-mode qualification.
        pub(super) fn ReadFile(
            handle: *mut c_void,
            buffer: *mut u8,
            size: u32,
            read: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        /// Query user-mode class16 into stable owned buffers; pending keeps them alive.
        pub(super) fn NtQueryInformationFile(
            handle: *mut c_void,
            status: *mut IoStatusBlock,
            information: *mut c_void,
            length: u32,
            class: i32,
        ) -> i32;
    }
}

/// Proposed Windows-only source/native controls; ROOT owns actual execution.
#[cfg(test)]
#[path = "windows_input_tests.rs"]
mod tests;
