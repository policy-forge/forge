//! Internal cooperative boundaries for session-owned preparation.
//!
//! These types never extend the wire contract. Runtime control owns the single
//! admission deadline, operation state and sticky interruption reason. A
//! checkpoint cannot preempt an already-running parser, engine or syscall.

use super::contract::Error;

/// Why runtime control stopped preparing an uncommitted effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Interruption {
    /// The current session recorded an explicit cancellation request.
    CancelRequested,
    /// The current workspace session is shutting down.
    Shutdown,
    /// The immutable accepted-operation deadline has been reached.
    DeadlineExceeded,
}

/// A safe ordinary failure or an interruption that must bypass best-effort
/// invalid/stale classification. Controlled callers preserve this distinction.
#[derive(Debug)]
pub(crate) enum WorkError {
    /// An existing safe workspace contract error.
    Failed(Error),
    /// Preparation stopped before retaining an effect; no wire reason is added.
    Interrupted(Interruption),
}

impl WorkError {
    /// Return a safe error at a synchronous wrapper or final worker fence.
    /// A no-op cannot originate interruption. A worker must preserve its sticky
    /// stop and choose cancellation before converting at the terminal boundary;
    /// capture/domain fallbacks must propagate the typed interruption instead.
    pub(crate) fn into_error(self) -> Error {
        match self {
            Self::Failed(error) => error,
            Self::Interrupted(_) => Error::invalid(),
        }
    }
}

impl From<Error> for WorkError {
    /// Preserve an existing safe contract failure without classifying it as a
    /// cancellation, timeout or partially successful result.
    fn from(error: Error) -> Self {
        Self::Failed(error)
    }
}

/// An internal preparation result; interruptions have no public error envelope.
pub(crate) type WorkResult<T> = std::result::Result<T, WorkError>;

/// Cooperative boundary names, never source labels or wire progress categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    /// Before and after capturing/parsing the explicit project index.
    ReadIndex,
    /// Before/after registered reads and installation of captured resource facts.
    CaptureResource,
    /// Before and after bounded classification of captured resource bytes.
    ValidateResource,
    /// Before and after captured applicability analysis.
    SnapshotAnalysis,
    /// Before and after derived mapping and subject inventory work.
    SnapshotMapping,
    /// Before and after stored report/staleness comparisons.
    SnapshotReports,
    /// Before and after each private temporary copy of captured bytes.
    CopyInputs,
    /// Before and after bounded domain engine, parser and encoding calls.
    PrepareDomain,
    /// Before constructing a local, uncommitted preview.
    PreparePreview,
    /// After local preparation and at the shared-store fence before retention.
    RetainPrepared,
}

/// Facts to publish through the existing optional/null `Operation.progress`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProgressUpdate {
    /// Preserve the current measured phase while checking for interruption.
    Unchanged,
    /// Count complete captured/classified registrations, including invalid ones.
    Capture {
        /// Successfully installed items in the current registered capture.
        completed: usize,
        /// The complete ordered registration denominator, at most 1,000.
        total: usize,
    },
    /// Unmeasured work or a terminal state has no numeric progress claim.
    Clear,
}

/// Runtime-owned cooperative control. Implementations latch their first stop
/// reason, check the same absolute deadline each time, and release state locks
/// before any input read, temporary write, parser or domain computation.
pub(crate) trait WorkControl {
    /// Check whether preparation may continue and, when running and not already
    /// cancel-requested, publish the specified measured capture fact.
    fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()>;

    /// Return the first latched interruption for the worker's final retention
    /// fence. Ordinary failure conversion must never erase this runtime state.
    fn interruption(&self) -> Option<Interruption>;
}

/// Consumed synchronous control for launch, queries and direct effects. It does
/// not create operation state or invent a deadline for those existing callers.
pub(crate) struct NoopControl;

impl WorkControl for NoopControl {
    /// Continue ordinary synchronous execution without publishing operation facts.
    fn checkpoint(&mut self, _stage: Stage, _progress: ProgressUpdate) -> WorkResult<()> {
        Ok(())
    }

    /// A synchronous no-op has no interruption to transfer to a worker.
    fn interruption(&self) -> Option<Interruption> {
        None
    }
}

#[cfg(test)]
/// Recording controls shared by actual capture/staging regressions. These are
/// deterministic observers, not runtime delay hooks or claimed HTTP evidence.
pub(crate) mod test_support {
    use super::*;

    /// A concrete cooperative boundary selected by a core test.
    #[derive(Clone, Copy)]
    enum StopPoint {
        /// Interrupt a specific observed occurrence of a named boundary.
        Stage {
            /// Boundary whose occurrences are counted.
            stage: super::Stage,
            /// One-based occurrence to interrupt.
            visit: usize,
        },
        /// Stop before the next read after a complete capture fact is observed.
        NextRead {
            /// Completed items that precede the next registered read.
            completed: usize,
        },
    }

    /// Record actual producer boundaries and optionally return sticky cancellation.
    #[derive(Default)]
    pub(crate) struct Recorder {
        /// Requested facts in their actual producer order, without source labels.
        pub(crate) events: Vec<(Stage, ProgressUpdate)>,
        /// Optional deterministic boundary chosen by the owning regression.
        stop: Option<StopPoint>,
        /// First returned interruption, preserved on all later calls.
        interrupted: Option<Interruption>,
    }

    impl Recorder {
        /// Interrupt the given one-based occurrence of a producer boundary.
        pub(crate) fn at(stage: Stage, visit: usize) -> Self {
            Self { stop: Some(StopPoint::Stage { stage, visit }), ..Self::default() }
        }

        /// Allow complete Items, then interrupt before opening the next input.
        pub(crate) fn before_read_after(completed: usize) -> Self {
            Self { stop: Some(StopPoint::NextRead { completed }), ..Self::default() }
        }
    }

    impl WorkControl for Recorder {
        /// Observe an actual core boundary and preserve the first interruption.
        fn checkpoint(&mut self, stage: Stage, progress: ProgressUpdate) -> WorkResult<()> {
            if let Some(reason) = self.interrupted {
                return Err(WorkError::Interrupted(reason));
            }
            self.events.push((stage, progress));
            let stop = match self.stop {
                Some(StopPoint::Stage { stage: wanted, visit }) => {
                    stage == wanted
                        && self.events.iter().filter(|(seen, _)| *seen == wanted).count() == visit
                }
                Some(StopPoint::NextRead { completed }) => {
                    stage == Stage::CaptureResource
                        && progress == ProgressUpdate::Unchanged
                        && self.events.iter().rev().find_map(|(_, value)| match value {
                            ProgressUpdate::Capture { completed, .. } => Some(*completed),
                            _ => None,
                        }) == Some(completed)
                }
                None => false,
            };
            if stop {
                self.interrupted = Some(Interruption::CancelRequested);
                return Err(WorkError::Interrupted(Interruption::CancelRequested));
            }
            Ok(())
        }

        /// Expose the same test cancellation to the caller's final fence.
        fn interruption(&self) -> Option<Interruption> {
            self.interrupted
        }
    }
}
