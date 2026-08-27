//! Shared deterministic benchmark fixtures.
//!
//! Bench targets cannot import `tests/common`, so shared ingest limits live
//! here rather than drifting independently (F0041).

/// Maximum input size used by fixture-driven benchmarks (10 MiB).
pub const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024;
