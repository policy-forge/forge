//! Offline complete input checking with one immutable original controller.
//!
//! Only the genuine native-source factory can supply the consumed complete
//! capability. This command writes a fixed status line after its final actual
//! original fence; it builds no index and offers no project output destination.

use std::io::{self, Write};
use std::path::{Component, Path};
use std::time::{Duration, Instant};

use super::declarations_v2::DeclarationAdmission;
use super::native_sources_v2::{self, OfflineBuildGate};
use super::{policy_failure, valid_hash};
use crate::workspace::contract::Error;
use crate::workspace::preparation::{
    Interruption, ProgressUpdate, Stage, WorkControl, WorkError, WorkResult,
};

/// Whole accepted operation limit, established once before root/config IO.
const CHECK_BUDGET: Duration = Duration::from_secs(10);
/// Static authored bytes, containing no actual project, path, key or source fact.
const CHECKED_LINE: &[u8] = b"MCP index inputs captured and checked; no index built.\n";

/// Private monotonic observation; no command argument supplies a clock or deadline.
trait Clock {
    /// Observe monotonic time without replacing the original deadline.
    fn now(&self) -> Instant;
}

/// Actual production monotonic observation for every cooperative checkpoint.
struct SystemClock;

impl Clock for SystemClock {
    /// Sample the actual monotonic clock; calendar scheduling stays independent.
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Original concrete controller retained by the genuine capture through output.
struct CheckControl<C, W> {
    /// One monotonic observer, independent of captured source or caller declarations.
    clock: C,
    /// Immutable accepted deadline, including complete native preparation.
    deadline: Instant,
    /// First actual stop; later observations cannot erase or replace it.
    stopped: Option<Interruption>,
    /// Actual output attempt occurs only while the complete native owner is held.
    output: W,
}

impl<C: Clock, W> WorkControl for CheckControl<C, W> {
    /// Check the original deadline without publishing source-derived progress.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        if let Some(reason) = self.stopped {
            return Err(WorkError::Interrupted(reason));
        }
        if self.clock.now() >= self.deadline {
            self.stopped = Some(Interruption::DeadlineExceeded);
            return Err(WorkError::Interrupted(Interruption::DeadlineExceeded));
        }
        Ok(())
    }

    /// Preserve the first real interruption through terminal error conversion.
    fn interruption(&self) -> Option<Interruption> {
        self.stopped
    }
}

impl<C: Clock, W: Write> CheckControl<C, W> {
    /// Attempt only the complete static line, then fence ordinary output failures too.
    /// A prefix already written cannot be retracted; no index is created by this call.
    fn publish_checked_line(&mut self) -> WorkResult<()> {
        self.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        let ordinary = self.output.write_all(CHECKED_LINE).map_err(|_| {
            WorkError::Failed(Error::new(
                "mcp-index-check-output-failed",
                "MCP index input check output failed; written bytes cannot be retracted.",
                false,
            ))
        });
        self.checkpoint(Stage::RetainPrepared, ProgressUpdate::Unchanged)?;
        ordinary
    }
}

/// Validate original argument spelling and canonical raw pins before any native open.
/// This admits only arguments; it issues no root, source or index capability.
fn admit_arguments(
    project: &Path,
    intent_root: &Path,
    intent_sha256: &str,
    profile_sha256: &str,
) -> WorkResult<()> {
    if !valid_hash(intent_sha256)
        || !valid_hash(profile_sha256)
        || !valid_root_spelling(project)
        || !valid_root_spelling(intent_root)
    {
        return Err(policy_failure());
    }
    Ok(())
}

/// Preserve maintained original UTF-8 root admission without opening or absolutizing it.
fn valid_root_spelling(raw: &Path) -> bool {
    raw.to_str().is_some_and(|spelling| {
        !spelling.is_empty()
            && spelling.len() <= 64 * 1024
            && !raw
                .components()
                .any(|part| matches!(part, Component::CurDir | Component::ParentDir))
            && crate::linkage::has_normalized_path_spelling(raw)
    })
}

/// Consume actual complete offline input proofs, retaining them through stdout.
/// A genuine missing producer/unavailable closure emits no success line.
pub(crate) fn check_index_inputs(
    project: &Path,
    intent_root: &Path,
    intent_sha256: &str,
    profile_sha256: &str,
) -> WorkResult<()> {
    admit_arguments(project, intent_root, intent_sha256, profile_sha256)?;
    let clock = SystemClock;
    let deadline = clock.now().checked_add(CHECK_BUDGET).ok_or_else(policy_failure)?;
    let output = io::stdout().lock();
    let mut control = CheckControl { clock, deadline, stopped: None, output };
    match native_sources_v2::capture_index_build_intent(
        project,
        intent_root,
        intent_sha256,
        profile_sha256,
        &mut control,
    )? {
        OfflineBuildGate::Unavailable => Err(policy_failure()),
        OfflineBuildGate::Validated(captured) => {
            // Static output borrows fixed bytes; it allocates no declaration-derived buffer.
            captured.admission().charge(1)?;
            captured.verify_inputs()?;
            captured.with_control(CheckControl::publish_checked_line)
        }
    }
}

#[cfg(test)]
/// Genuine controller/output regressions; no native capability is fabricated.
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;

    /// Deterministic original monotonic observation used only for controller controls.
    struct TestClock(Rc<Cell<Instant>>);

    impl Clock for TestClock {
        /// Return the exact controlled observation without changing its stored deadline.
        fn now(&self) -> Instant {
            self.0.get()
        }
    }

    /// One real Write port which advances time during an ordinary write failure.
    struct LateFailure {
        /// Same original clock observation used by the actual controller.
        clock: Rc<Cell<Instant>>,
        /// Exact later monotonic observation selected by the failure control.
        after: Instant,
        /// Actual number of write attempts; a preexisting stop must keep this zero.
        writes: usize,
    }

    impl Write for LateFailure {
        /// Reach a genuine ordinary Write failure after advancing the original clock.
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            self.writes += 1;
            self.clock.set(self.after);
            Err(io::Error::other("controlled ordinary output failure"))
        }

        /// No buffered output exists in this deterministic failing port.
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// An observed deadline remains sticky even if the deterministic clock changes later.
    #[test]
    fn original_deadline_stop_is_never_renewed_by_later_observations() {
        let start = Instant::now();
        let now = Rc::new(Cell::new(start));
        let deadline = start + CHECK_BUDGET;
        let mut control = CheckControl {
            clock: TestClock(Rc::clone(&now)),
            deadline,
            stopped: None,
            output: Vec::new(),
        };
        control.checkpoint(Stage::ReadIndex, ProgressUpdate::Unchanged).unwrap();
        now.set(deadline);
        assert!(matches!(
            control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        now.set(start);
        assert!(matches!(
            control.publish_checked_line(),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        assert_eq!(control.deadline, deadline);
        assert_eq!(control.output, [] as [u8; 0]);
    }

    /// A late original deadline wins over an actual ordinary output failure.
    #[test]
    fn ordinary_output_failure_reaches_the_original_post_phase_fence() {
        let start = Instant::now();
        let now = Rc::new(Cell::new(start));
        let deadline = start + CHECK_BUDGET;
        let mut control = CheckControl {
            clock: TestClock(Rc::clone(&now)),
            deadline,
            stopped: None,
            output: LateFailure { clock: now, after: deadline, writes: 0 },
        };
        assert!(matches!(
            control.publish_checked_line(),
            Err(WorkError::Interrupted(Interruption::DeadlineExceeded))
        ));
        assert_eq!(control.output.writes, 1);
        assert_eq!(control.interruption(), Some(Interruption::DeadlineExceeded));
    }

    /// Malformed original raw pins refuse at non-IO argument admission.
    #[test]
    fn malformed_raw_pin_arguments_never_reach_native_capture() {
        for malformed in ["", "ABCDEF", &"F".repeat(64), &"0".repeat(63)] {
            assert!(
                admit_arguments(
                    Path::new("unread-project"),
                    Path::new("unread-intent"),
                    malformed,
                    &"a".repeat(64)
                )
                .is_err()
            );
            assert!(
                admit_arguments(
                    Path::new("unread-project"),
                    Path::new("unread-intent"),
                    &"a".repeat(64),
                    malformed
                )
                .is_err()
            );
        }
    }
}
