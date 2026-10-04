//! Proposed narrow Windows admission/read/lifetime controls; no execution credit.

use super::*;
use std::os::windows::io::IntoRawHandle as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// Synthetic owned handle records actual Rust destruction without any native call.
struct CountedHandle {
    /// Shared observer survives a deliberately retained uncertain query owner.
    dropped: Arc<AtomicUsize>,
}

impl AsRawHandle for CountedHandle {
    /// Return an inert sentinel used only by the explicit mock query closure.
    fn as_raw_handle(&self) -> RawHandle {
        1_usize as RawHandle
    }
}
impl Drop for CountedHandle {
    /// Mark destruction, making premature pending-buffer ownership release observable.
    fn drop(&mut self) {
        self.dropped.fetch_add(1, AtomicOrdering::SeqCst);
    }
}

/// Create one synthetic owner plus an independent destruction observer.
fn counted_handle() -> (CountedHandle, Arc<AtomicUsize>) {
    let dropped = Arc::new(AtomicUsize::new(0));
    (CountedHandle { dropped: Arc::clone(&dropped) }, dropped)
}

/// Fill actual supplied mock output addresses only while this synchronous closure owns them.
fn mock_completion(
    status: *mut IoStatusBlock,
    mode: *mut FileModeInformation,
    code: i32,
    information: usize,
    flags: u32,
) {
    // SAFETY: qualify_mode_with supplies live exclusive heap outputs to this mock
    // synchronous call; no OS operation is submitted or retained by the mock.
    unsafe {
        (*status).status.status = code;
        (*status).information = information;
        (*mode).mode = flags;
    }
}

/// Construct genuine native anonymous-pipe endpoints with caller-owned handles.
fn anonymous_pipe() -> (OwnedHandle, OwnedHandle) {
    let mut read = std::ptr::null_mut();
    let mut write = std::ptr::null_mut();
    // SAFETY: two live output pointers receive native owned handles; no security
    // attributes/inheritance are requested, and each is installed immediately.
    let result = unsafe {
        control_native::CreatePipe(&raw mut read, &raw mut write, std::ptr::null_mut(), 65_536)
    };
    assert_ne!(result, 0, "native anonymous pipe creation must succeed");
    assert!(!read.is_null() && !write.is_null());
    unsafe { (OwnedHandle::from_raw_handle(read), OwnedHandle::from_raw_handle(write)) }
}

/// Write one small real fixture payload to a synchronous owned pipe, not a mocked response.
fn write_bytes(handle: &OwnedHandle, bytes: &[u8]) {
    let size = u32::try_from(bytes.len()).unwrap();
    let mut written = 0;
    // SAFETY: the fixture owns the synchronous pipe and keeps its complete live
    // payload borrowed through this one bounded native call.
    let result = unsafe {
        control_native::WriteFile(
            handle.as_raw_handle(),
            bytes.as_ptr(),
            size,
            &raw mut written,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(result, 0, "native fixture write must succeed");
    assert_eq!(written, size);
}

/// Create a unique real local pipe pair; overlapped servers are never given a null-`OVERLAPPED` connect call.
fn named_pipe(message: bool, overlapped: bool) -> (OwnedHandle, OwnedHandle) {
    static SEQUENCE: AtomicUsize = AtomicUsize::new(0);
    let name = format!(
        "\\\\.\\pipe\\forge-mcp-input-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, AtomicOrdering::SeqCst)
    );
    let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let mode = if message { 4 | 2 } else { 0 };
    // SAFETY: the null-terminated fixture name remains live; inbound synchronous
    // server handles and writable synchronous client handles transfer ownership.
    let open_mode = 1 | if overlapped { 0x4000_0000 } else { 0 };
    let server = unsafe {
        control_native::CreateNamedPipeW(
            wide.as_ptr(),
            open_mode,
            mode,
            1,
            4096,
            4096,
            0,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(server, (-1_isize) as RawHandle);
    assert!(!server.is_null());
    let server = unsafe { OwnedHandle::from_raw_handle(server) };
    let client = unsafe {
        control_native::CreateFileW(
            wide.as_ptr(),
            0x4000_0000,
            0,
            std::ptr::null_mut(),
            3,
            0,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(client, (-1_isize) as RawHandle);
    assert!(!client.is_null());
    let client = unsafe { OwnedHandle::from_raw_handle(client) };
    if !overlapped {
        let connected = unsafe {
            control_native::ConnectNamedPipe(server.as_raw_handle(), std::ptr::null_mut())
        };
        if connected == 0 {
            assert_eq!(
                io::Error::last_os_error().raw_os_error(),
                Some(535),
                "only the already-connected native race is accepted"
            );
        }
    }
    (server, client)
}

/// One startup attempt remains consumed after any failure; retries cannot multiply pending owners.
#[test]
fn startup_attempt_is_sticky_without_lifetime_renewal() {
    let gate = AtomicBool::new(false);
    assert!(claim_startup(&gate).is_ok());
    assert!(claim_startup(&gate).is_err());
    assert!(claim_startup(&gate).is_err());
}

/// Actual C layout keeps ULONG mode and pointer-width union/information fields distinct.
#[test]
fn native_mode_layout_matches_pointer_width() {
    assert_eq!(std::mem::size_of::<FileModeInformation>(), 4);
    assert_eq!(std::mem::size_of::<IoStatus>(), std::mem::size_of::<usize>());
    assert_eq!(std::mem::offset_of!(IoStatusBlock, information), std::mem::size_of::<usize>());
    assert_eq!(std::mem::size_of::<IoStatusBlock>(), 2 * std::mem::size_of::<usize>());
}

/// Both real endpoint flags and either byte wait state preserve byte-stream admission.
#[test]
fn byte_endpoint_flags_and_wait_states_are_admitted() {
    for flags in [0, PIPE_SERVER_END] {
        for state in [0, PIPE_NOWAIT] {
            assert!(byte_pipe(FILE_TYPE_PIPE, flags, state));
        }
    }
}

/// Unsupported file/console/message/unknown metadata never enters a byte read profile.
#[test]
fn nonbyte_profiles_are_refused_without_relabeling() {
    for file_type in [0, 1, 2, 3 | 32768, 4] {
        assert!(!byte_pipe(file_type, 0, 0));
    }
    for flags in [PIPE_TYPE_MESSAGE, 5, 8] {
        assert!(!byte_pipe(FILE_TYPE_PIPE, flags, 0));
    }
    for state in [2, 3, 4] {
        assert!(!byte_pipe(FILE_TYPE_PIPE, 0, state));
    }
}

/// Either synchronous bit alone is required; independent native mode bits remain untouched.
#[test]
fn native_synchronous_mode_is_explicit_not_inferred_from_pipe_type() {
    assert!(complete_synchronous(0, 0, 4, SYNCHRONOUS_ALERT));
    assert!(complete_synchronous(0, 0, 4, SYNCHRONOUS_NONALERT));
    assert!(complete_synchronous(0, 0, 4, SYNCHRONOUS_NONALERT | 2));
    assert!(!complete_synchronous(0, 0, 4, 0));
    assert!(!complete_synchronous(0, 0, 4, SYNCHRONOUS_ALERT | SYNCHRONOUS_NONALERT));
}

/// Informational/pending/error status or incomplete/overlong output cannot qualify a mode.
#[test]
fn status_and_exact_output_size_are_complete_admission_conditions() {
    for returned in [-1, 1, STATUS_PENDING] {
        assert!(!complete_synchronous(returned, 0, 4, SYNCHRONOUS_NONALERT));
    }
    for status in [-1, 1, STATUS_PENDING] {
        assert!(!complete_synchronous(0, status, 4, SYNCHRONOUS_NONALERT));
    }
    for size in [0, 3, 5, usize::MAX] {
        assert!(!complete_synchronous(0, 0, size, SYNCHRONOUS_NONALERT));
    }
}

/// A pending native return keeps the actual owner even when outputs resemble success.
#[test]
fn pending_query_retains_owner_and_never_uses_success_looking_output() {
    let (handle, dropped) = counted_handle();
    let result = qualify_mode_with(handle, |_raw, status, mode| {
        mock_completion(status, mode, 0, 4, SYNCHRONOUS_NONALERT);
        STATUS_PENDING
    });
    assert!(result.is_err());
    assert_eq!(
        dropped.load(AtomicOrdering::SeqCst),
        0,
        "uncertain owner must remain retained, not cancelled or completed"
    );
}

/// A contradictory pending `IOSB` record is uncertainty even after a zero function return.
#[test]
fn pending_completion_record_keeps_owned_storage() {
    let (handle, dropped) = counted_handle();
    let result = qualify_mode_with(handle, |_raw, status, mode| {
        mock_completion(status, mode, STATUS_PENDING, 4, SYNCHRONOUS_NONALERT);
        STATUS_SUCCESS
    });
    assert!(result.is_err());
    assert_eq!(dropped.load(AtomicOrdering::SeqCst), 0);
}

/// Unwritten output sentinels cannot become a fabricated synchronous-mode success.
#[test]
fn missing_native_output_is_not_default_success() {
    let (handle, dropped) = counted_handle();
    assert!(qualify_mode_with(handle, |_raw, _status, _mode| 0).is_err());
    assert_eq!(
        dropped.load(AtomicOrdering::SeqCst),
        0,
        "pending sentinel remains uncertain rather than being dropped"
    );
}

/// A final nonpending query error releases ownership once without interpreting output.
#[test]
fn completed_query_failure_releases_owner_once() {
    let (handle, dropped) = counted_handle();
    let result = qualify_mode_with(handle, |_raw, _status, _mode| -1);
    assert!(result.is_err());
    assert_eq!(dropped.load(AtomicOrdering::SeqCst), 1);
}

/// Exact completed synchronous mode transfers the original owner without a copied handle.
#[test]
fn completed_query_preserves_owner_until_driver_release() {
    let (handle, dropped) = counted_handle();
    let result = qualify_mode_with(handle, |_raw, status, mode| {
        mock_completion(status, mode, 0, 4, SYNCHRONOUS_ALERT);
        0
    });
    assert!(result.is_ok());
    assert_eq!(dropped.load(AtomicOrdering::SeqCst), 0);
    drop(result);
    assert_eq!(dropped.load(AtomicOrdering::SeqCst), 1);
}

/// Genuine native open-empty byte pipes return idle, which is distinct from EOF.
#[test]
fn open_empty_anonymous_pipe_is_idle() {
    let (read, _write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    let mut destination = [0_u8; 16];
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Idle));
}

/// One actual ready byte is delivered before any newline or complete protocol frame exists.
#[test]
fn partial_frame_byte_does_not_wait_for_a_line() {
    let (read, write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    write_bytes(&write, b"{");
    let mut destination = [0_u8; 16];
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(1)
    ));
    assert_eq!(destination[0], b'{');
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Idle));
}

/// Caller capacity constrains a genuine read and leaves unconsumed bytes for the next fence.
#[test]
fn actual_pipe_read_respects_smaller_destination() {
    let (read, write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    write_bytes(&write, b"abcdef");
    let mut destination = [0_u8; 2];
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(2)
    ));
    assert_eq!(&destination, b"ab");
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(2)
    ));
    assert_eq!(&destination, b"cd");
}

/// Actual pipe buffering larger than one burst is consumed in separate bounded fences.
#[test]
fn actual_pipe_read_never_exceeds_the_4096_byte_burst() {
    let (read, write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    let bytes = vec![b'x'; 4097];
    write_bytes(&write, &bytes);
    let mut destination = [0_u8; 8192];
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(4096)
    ));
    assert!(destination[..4096].iter().all(|byte| *byte == b'x'));
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(1)
    ));
    assert_eq!(destination[0], b'x');
}

/// Buffered real bytes precede native peer-close EOF, which remains sticky.
#[test]
fn buffered_close_drains_then_retires_input() {
    let (read, write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    write_bytes(&write, b"tail");
    drop(write);
    let mut destination = [0_u8; 16];
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(4)
    ));
    assert_eq!(&destination[..4], b"tail");
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Eof));
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Eof));
}

/// Zero-capacity caller misuse fails permanently instead of manufacturing input EOF.
#[test]
fn empty_destination_fails_without_revival() {
    let (read, _write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    assert!(input.read_ready(Duration::ZERO, &mut []).is_err());
    assert!(input.read_ready(Duration::ZERO, &mut [0_u8; 16]).is_err());
}

/// Real disk-file input is refused before a byte-pipe reader can be created.
#[test]
fn actual_regular_file_is_an_unsupported_profile() {
    let file = tempfile::tempfile().unwrap();
    // SAFETY: into_raw_handle transfers this live File's single owned handle.
    let handle = unsafe { OwnedHandle::from_raw_handle(file.into_raw_handle()) };
    assert!(WindowsInput::from_pipe(handle).is_err());
}

/// A genuinely connected local synchronous named byte pipe uses the same single-owner port.
#[test]
fn actual_named_byte_pipe_delivers_partial_stream_bytes() {
    let (read, write) = named_pipe(false, false);
    let mut input = WindowsInput::from_pipe(read).unwrap();
    write_bytes(&write, b"fragment");
    let mut destination = [0_u8; 16];
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(8)
    ));
    assert_eq!(&destination[..8], b"fragment");
}

/// Actual message-pipe fixtures are refused, preserving their boundary semantics.
#[test]
fn actual_named_message_pipe_is_refused() {
    let (read, _write) = named_pipe(true, false);
    assert!(WindowsInput::from_pipe(read).is_err());
}

/// Exact completed async mode refuses and releases the owner without a read request.
#[test]
fn completed_nonsynchronous_mode_refuses_and_releases_owner() {
    let (handle, dropped) = counted_handle();
    let result = qualify_mode_with(handle, |_raw, status, mode| {
        mock_completion(status, mode, 0, 4, 0);
        0
    });
    assert!(result.is_err());
    assert_eq!(dropped.load(AtomicOrdering::SeqCst), 1);
}

/// A genuine overlapped local byte-pipe handle is rejected before any driver read.
#[test]
fn actual_overlapped_named_pipe_is_not_a_synchronous_profile() {
    let (read, _write) = named_pipe(false, true);
    assert!(WindowsInput::from_pipe(read).is_err());
}

/// Kernel32 fixture creation/write declarations are used only by proposed native controls.
mod control_native {
    use super::c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        /// Create a genuine synchronous anonymous-pipe pair without inheritance.
        pub(super) fn CreatePipe(
            read: *mut *mut c_void,
            write: *mut *mut c_void,
            security: *mut c_void,
            size: u32,
        ) -> i32;
        /// Send small actual fixture bytes through a synchronous owned pipe.
        pub(super) fn WriteFile(
            handle: *mut c_void,
            bytes: *const u8,
            size: u32,
            written: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
        /// Create one local fixture with explicitly selected storage and I/O creation modes.
        pub(super) fn CreateNamedPipeW(
            name: *const u16,
            open_mode: u32,
            pipe_mode: u32,
            instances: u32,
            output: u32,
            input: u32,
            timeout: u32,
            security: *mut c_void,
        ) -> *mut c_void;
        /// Open the existing local client endpoint without overlapped flags.
        pub(super) fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *mut c_void,
            disposition: u32,
            flags: u32,
            template: *mut c_void,
        ) -> *mut c_void;
        /// Complete server connection, accepting only the explicit already-connected race.
        pub(super) fn ConnectNamedPipe(handle: *mut c_void, overlapped: *mut c_void) -> i32;
    }
}

/// Synthetic completion facts exercise the consumed classifier, not a reached native zero read.
/// An open zero result accepts later data, then only broken-pipe failure settles sticky EOF.
#[test]
fn successful_zero_completion_keeps_open_for_later_data_and_broken_pipe() {
    let (read, _write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    assert!(matches!(input.complete_read(1, Ok(0)).unwrap(), InputRead::Idle));
    assert!(matches!(input.state, StreamState::Open));
    assert!(matches!(input.complete_read(1, Ok(1)).unwrap(), InputRead::Data(1)));
    assert!(matches!(input.state, StreamState::Open));
    assert!(matches!(
        input.complete_read(1, Err(io::Error::from_raw_os_error(ERROR_BROKEN_PIPE))).unwrap(),
        InputRead::Eof
    ));
    assert!(matches!(input.state, StreamState::Eof));
    assert!(matches!(input.complete_read(1, Ok(1)).unwrap(), InputRead::Eof));
    assert!(matches!(input.read_ready(Duration::ZERO, &mut [0_u8; 8]).unwrap(), InputRead::Eof));
}

/// An explicitly injected zero completion preserves readiness for actual later pipe bytes and close.
/// Native data/EOF are genuine here; the successful-zero `ReadFile` condition is not OS-produced.
#[test]
fn classified_zero_preserves_actual_later_payload_and_peer_close() {
    let (read, write) = anonymous_pipe();
    let mut input = WindowsInput::from_pipe(read).unwrap();
    assert!(matches!(input.complete_read(8, Ok(0)).unwrap(), InputRead::Idle));
    assert!(matches!(input.state, StreamState::Open));
    let mut destination = [0_u8; 8];
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Idle));
    write_bytes(&write, b"later");
    assert!(matches!(
        input.read_ready(Duration::ZERO, &mut destination).unwrap(),
        InputRead::Data(5)
    ));
    assert_eq!(&destination[..5], b"later");
    drop(write);
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Eof));
    assert!(matches!(input.read_ready(Duration::ZERO, &mut destination).unwrap(), InputRead::Eof));
}

/// Synthetic invalid completed counts and unrelated errors fail permanently rather than inventing EOF.
/// Bounds are checked on the exact request/count pair before any later fact could revive the input.
#[test]
fn malformed_completed_reads_and_nonbroken_errors_fail_without_revival() {
    for (requested, completion) in
        [(0, Ok(0)), (1, Ok(2)), (u32::MAX, Ok(1)), (1, Err(io::Error::from_raw_os_error(5)))]
    {
        let (read, _write) = anonymous_pipe();
        let mut input = WindowsInput::from_pipe(read).unwrap();
        assert!(input.complete_read(requested, completion).is_err());
        assert!(matches!(input.state, StreamState::Failed));
        assert!(input.complete_read(1, Ok(1)).is_err());
        assert!(
            input.complete_read(1, Err(io::Error::from_raw_os_error(ERROR_BROKEN_PIPE))).is_err()
        );
        assert!(input.read_ready(Duration::ZERO, &mut [0_u8; 8]).is_err());
    }
}
