# ADR-0002: Shared Application Services and Effect Boundaries

> **Document Type:** Architecture Decision Record
> **Audience:** LLM agents, human reviewers, engineering
> **Status:** Accepted (PRD-062 Slice 0, 2026-09-08)
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

Related: ADR-0001 (contract ownership), ADR-0003 (embedded assets),
`docs/plans/2026-09-08-062-slice0-service-boundaries.md` (extraction map and
boundary definitions), PRD 062 "Required Application-Service Shape", "Write
Preview and Transaction Model", M-5, M-13, M-14, M-23, M-24.

## Context

PRD 062 requires that the HTTP API and the CLI delegate to the same typed
Rust application services, with no shell/subprocess use, no stdout scraping,
and no adapter-owned domain rules (M-5). The workspace must not call
`cli::execute` or spawn the `forge` binary. Every material write must follow
a preview/confirm transaction bound to exact bytes and hashes (M-13, M-14),
and every effect-creating request must be conditional and idempotent with a
queryable operation resource (M-23, M-24).

Repository evidence (detailed in the Slice 0 plan document): the domain
engines are already largely pure — `src/pipeline.rs` returns a `PipelineOutput`
without writing; `src/validate/mod.rs` `run_full_validation` returns a typed
`ValidationReport`; `src/mapping/mod.rs` and `src/applicability/mod.rs` each
contain a private prepare seam (`prepare`, `prepare_analysis`) that fully
computes bytes and reports before any write; `src/trace/mod.rs`
`generate_trace_report` returns a typed report. However, the public
`execute_*` functions then write files through `crate::cli::output::write_output`,
print to stderr, and return booleans consumed for CLI exit classification; and
the domain modules import argument enums from `src/cli/mod.rs`
(`MappingReportFormat`, `MappingFailOn`, `ApplicabilityReportFormat`,
`ApplicabilityFailOn`), inverting the layering. Safe I/O primitives exist
(`src/io.rs` `write_atomic`, `read_bounded`; `src/hashing.rs` `sha256_hex`;
fingerprint capture in `src/applicability/mod.rs` `capture_input_fingerprint`),
but `write_atomic` does not revalidate destination identity at commit time and
is not race-resistant against symlink or parent-directory substitution.

## Decision

1. **Four-phase service shape.** Every API/CLI-supported operation separates:
   (1) load and validate inputs — typed data, fingerprints, structured
   diagnostics, no effects; (2) prepare result — proposed bytes, semantic
   summary, result classification, destination intent; (3) preview effect —
   bind the exact bytes and destination to current input/target hashes and a
   one-time receipt; (4) commit effect — recheck hashes, atomically write only
   the previewed bytes, return the committed fingerprint. The concrete
   extraction map, module layout (`src/workspace/services/...`), boundary
   types, and capability-row mapping are normative in
   `docs/plans/2026-09-08-062-slice0-service-boundaries.md` and are bound to
   this ADR by reference.

2. **Operation and idempotency model.** Bounded work is represented as an
   operation with a stable ID and a terminal-state-only state machine
   (pending, running, succeeded, failed, cancelled). Terminal results are
   retained in process memory for the session so a lost HTTP response can
   always be resolved by querying the operation (AC-23). Effect-creating
   requests require a client-generated idempotency key; an identical retry
   returns the original operation/commit result without repeating the effect,
   and reuse of the key with different content fails (M-23, AC-22).

3. **Receipt model.** A preview receipt is single-use, session-bound,
   short-lived, and binds: session ID, operation type, destination identity,
   base hash of the target, hashes of every consumed input, and the hash of
   the exact proposed bytes (M-13). Confirmation submits only the receipt,
   observed resource version, confirmation intent, and idempotency key. Any
   identity mismatch, expiry, or reuse invalidates the receipt (AC-11). There
   is no last-write-wins path and no autosave.

4. **Atomic writer requirements.** The single shared writer extends
   `src/io.rs` `write_atomic` (temp file, fsync, rename, parent sync) with:
   destination and parent identity revalidation immediately before commit
   (reject symlinks and non-regular files, recheck parent identity);
   race-resistant directory-relative open/create primitives where the
   platform supports them (Linux `openat2`/`O_RESOLVE_BENEATH`-class,
   macOS `openat` with `O_NOFOLLOW`-class checks, Windows
   `CreateFileW`-based equivalents that fail on reparse points); and a
   documented fail-closed equivalent on every supported platform where a
   primitive is unavailable, per PRD "Project Containment". Multi-file
   effects are out of the MVP unless committed as one documented atomic
   transaction with rollback.

5. **Adapter parity rule.** HTTP handlers (`src/workspace/http/`) and CLI
   adapters bind to identical typed service requests and results; adapters
   contain no domain policy, no subprocess, no terminal formatting of domain
   data, and the CLI keeps its current syntax and `src/error.rs` `exit_code`
   mapping. The CLI does not loop back through HTTP.

## Consequences

- Positive: parity becomes testable by construction — the same service
  request through either adapter must yield byte-equal artifacts and equal
  classifications, which is exactly the measurable acceptance criterion
  (M-19, AC-14): parity fixtures compare artifact bytes and typed
  classifications across mapping, applicability, conversion, validation, and
  trace workloads, and mismatches block release as contract defects.
- Positive: the private prepare seams already present in mapping and
  applicability mean extraction is mostly visibility and de-inversion work,
  not reimplementation; `src/mapping`/`src/applicability` cross-reuse
  (`mapping::inventory`, `mapping::paths_alias`) is preserved.
- Negative: the CLI-visible enums must move from `src/cli/mod.rs` into the
  service layer (with CLI re-exports so behavior is unchanged), touching
  modules that today compile against them; this is scheduled work in the
  extraction plan, not a Slice 0 change.
- Negative: retaining terminal operation results in memory costs bounded
  session memory and requires expiry policy; unbounded growth is prevented by
  the PRD's operation/duration bounds.
- Risk: hardening `write_atomic` for race resistance is platform-sensitive;
  it is isolated in the shared writer and exercised by the PRD transaction
  test layer (input/target races, interruption, rollback) on all three
  platforms before Slice 2 exits.

## Alternatives considered

- **Run the CLI binary as the workspace engine (spawn `forge`, scrape
  stdout/exit codes).** Rejected by the PRD outright (M-5): it would make
  terminal text a de facto contract, reintroduce shell-injection surface, and
  prevent typed preview/commit semantics.
- **Force the CLI through the HTTP loopback transport.** Rejected (PRD
  "Architecture Decision"): it adds a server/session dependency to reliable
  offline CLI execution without improving shared-domain semantics. Both
  adapters call services directly instead.
- **Per-adapter services (separate HTTP-facing and CLI-facing
  implementations).** Rejected: guarantees eventual drift in artifacts or
  classifications, the exact failure M-19 exists to prevent; parity fixtures
  would become a permanent tax instead of a regression alarm.
- **Database-backed staging for receipts/operations.** Rejected for MVP:
  files remain authoritative and the PRD prohibits a hidden server database;
  in-memory receipts with session lifetime satisfy the recovery stories
  (relaunch with files unchanged or fully committed, AC-17).
