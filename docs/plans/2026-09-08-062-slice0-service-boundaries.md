# 2026-09-08 — PRD 062 Slice 0: Service-Boundary Inventory and Extraction Plan

> **Document Type:** Engineering Plan
> **Audience:** LLM agents, human reviewers, engineering
> **Status:** Draft
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

---

## Purpose

PRD 062 requires every browser capability to run through a versioned local HTTP
API whose handlers and the existing CLI both delegate to the same typed Rust
application services (PRD 062, "API-First Architecture and Shared
Application-Service Boundary", M-5, M-19). This document inventories how the
current code couples preparation, rendering, file writes, exit-code
classification, and terminal output; defines the service, effect, and adapter
boundaries later slices will implement; and binds the PRD's initial
service-extraction map to concrete modules and functions.

**Slice 0 changes no production behavior.** No source file under `src/` is
modified in Slice 0. This plan is the reviewed input for the extraction work
scheduled in Slices 1 and 2; ADR-0001 through ADR-0004 in `docs/adr/` record the
decisions that frame it.

Companion artifacts produced in the same slice: the normative OpenAPI contract
`docs/api/forge-workspace-v1.openapi.yaml`, `docs/api/capability-matrix.md`,
`docs/api/compatibility.md`, the threat model
`docs/SEC/062-sec-local-web-workspace.md`, and
`docs/accessibility/062-accessibility-requirements.md`.

## 1. Coupling inventory

The dominant pattern: most commands already compute a fully validated,
in-memory result before writing (a good foundation), but the *execute* layer
then writes files, renders to stderr/stdout, and classifies the CLI exit code
in the same call. Several domain modules also depend *back* on CLI types,
which inverts the intended layering.

### 1.1 Conversion / pipeline

| Aspect | Evidence | What it mixes |
|---|---|---|
| Pipeline core | `src/pipeline.rs` `prepare_document`, `run_catalog_pipeline`, `run_component_pipeline` (plus `run_catalog_pipeline_prepared`, `run_component_pipeline_prepared`); `PipelineOutput` / `SecondaryOutput` | Pure preparation and serialization. Does not write files or print. Already reusable. |
| Single-file command | `src/cli/convert.rs` `execute` | Input guard (`validate_regular_file`), baseline warning emission, pipeline dispatch, primary **and** secondary output writes via `src/cli/output.rs` `write_output`, stderr dashboard via `src/summary/format.rs` `format_summary_dashboard` with `IsTerminal` color detection. |
| Batch command | `src/batch/orchestrator.rs` `run_batch_conversion` invoked from `src/cli/convert.rs` | Batch writes each output; summary rendering to stderr in the CLI layer; failure classification via `ForgeError::BatchConversion`. |

Reusable today: `PipelineOutput` content strings; prepared-document reuse
(`prepare_document` is `pub(crate)`). Blocking reuse: writes and terminal
summaries are interleaved with orchestration in `src/cli/convert.rs`.

### 1.2 Validation

| Aspect | Evidence | What it mixes |
|---|---|---|
| Domain core | `src/validate/mod.rs` `run_full_validation`, `validate_artifact`, `detect_model_type`, `load_schema`; typed `ValidationReport` from `src/validate/error_types.rs` | Schema + semantic passes return a typed report. No terminal formatting inside the model. Already reusable. |
| Rendering | `src/validate/report.rs` `render_text_report`, `render_json_report` | Rendering is separate from the report type; both CLI-grade outputs coexist. Reusable as adapter rendering. |
| Command | `src/cli/validate.rs` `execute` | Reads/parses input, routes rendered report to stdout (valid) vs stderr/file (invalid), and classifies exit via `ForgeError::SchemaValidation`. `execute_round_trip` additionally shells out through `src/oscal_cli/`. |

### 1.3 Mapping

| Aspect | Evidence | What it mixes |
|---|---|---|
| Commands | `src/mapping/mod.rs` `execute_init`, `execute_build`, `execute_check` | Each performs prepare → destination checks → **file writes** through `crate::cli::output::write_output` → returns `bool` used for `ForgeError::MappingReviewRequired` exit classification. |
| Hidden seam | `src/mapping/mod.rs` `prepare` (private) returns `PreparedBuild { artifact_json, report: MappingReport, input_paths }`; `render_report`; `review_required` | Preparation is already fully separated *internally* but private. `render_report` already emits JSON or text. |
| Manifest/IO | `src/mapping/manifest.rs` `parse`, `MAX_MANIFEST_BYTES`, `MANIFEST_SCHEMA_VERSION`; `src/mapping/inventory.rs` `load`, `validate_schema`; `src/mapping/mod.rs` `validate_destinations`, `paths_alias` | Bounded reads, strict schema version, alias rejection, and output/input aliasing checks. Reusable as trust checks. |
| Layering inversion | `src/mapping/mod.rs` line 11: `use crate::cli::{MappingFailOn, MappingReportFormat}` | Domain module depends on CLI argument types defined in `src/cli/mod.rs` (`MappingReportFormat`, `MappingFailOn`). |

### 1.4 Applicability

| Aspect | Evidence | What it mixes |
|---|---|---|
| Commands | `src/applicability/mod.rs` `execute_init`, `execute_analyze` | Same pattern as mapping: prepare → destination check → write via `crate::cli::output::write_output` → `bool` classification (`ForgeError::ApplicabilityReviewRequired`). |
| Hidden seam | `src/applicability/mod.rs` `prepare_analysis` (already `pub(crate)`) returns `PreparedAnalysis`; typed `ApplicabilityReport` in `src/applicability/model.rs`; `review_required` | Nearly identical extraction shape to mapping. |
| Fingerprints | `src/applicability/mod.rs` `capture_input_fingerprint` (SHA-256 + byte length per input), `validate_destination` | The hash-binding primitive the preview-receipt boundary needs already exists here, private. |
| Cross-domain reuse | `src/applicability/mod.rs` imports `crate::mapping::inventory`, `crate::mapping::paths_alias` | Applicability deliberately reuses mapping loaders; extraction must preserve this, not duplicate it. |
| Layering inversion | `src/applicability/mod.rs` imports `crate::cli::{ApplicabilityFailOn, ApplicabilityReportFormat, ApplicabilityStateFilter}` | Same CLI-type dependency as mapping. |

### 1.5 Trace

| Aspect | Evidence | What it mixes |
|---|---|---|
| Domain core | `src/trace/mod.rs` `generate_trace_report` returns typed `TraceReport` (`src/trace/report.rs`), with bounded reads and staleness checks (`src/trace/resolver.rs`) | Pure and reusable. |
| Command | `src/cli/trace.rs` `execute` / `execute_with_composition_provenance` | Formats the table (`src/trace/formatter.rs` `format_trace_table`), writes it, and `eprintln!`s the output path. |

### 1.6 Project / configuration

| Aspect | Evidence | What it mixes |
|---|---|---|
| Config load | `src/config.rs` `load_selected`, `select_path`, `discover_from`, `load_file`, `resolve_convert`, `resolve_validate` | `.forge.toml` discovery/resolution is transport-neutral and reusable. |
| Selector policy | `src/cli/mod.rs` `reject_unsupported_config_selector` | CLI-surface policy living in the dispatcher; fine for the CLI adapter, not a service concern. |
| Root context | none | There is no explicit canonical-root object today; commands receive bare paths. The workspace needs a typed root/resource-index context (new work). |

### 1.7 Diff / summary / export

| Aspect | Evidence | What it mixes |
|---|---|---|
| Diff | `src/diff/mod.rs` `diff_artifacts` returns typed `DiffReport`; `src/diff/formatter.rs` `format_diff_report`; `src/cli/diff.rs` `execute` writes to stdout and collapses classification into `ForgeError::DiffHasChanges` at the dispatcher (`src/cli/mod.rs` `execute`) | Engine reusable; classification and output in the adapter (correct shape) but hardcoded to stdout-only. |
| Summary | `src/summary/format.rs` `format_summary_dashboard` (terminal color) | Rendering only; needs a structured form for API summaries. |
| Export | `src/cli/export.rs` `execute` | Input read, format conversion, and output writing in one call; shares `write_output`. |

### 1.8 Shared infrastructure (already boundary-shaped)

- `src/io.rs` `write_atomic` (temp file + fsync + rename), `read_bounded`,
  `check_file_size`, `sanitize_artifact_path`, `MAX_FILE_SIZE` — the safe-I/O
  core the effect boundary builds on.
- `src/json_strict.rs` `parse_value`, `StrictJsonError`, `Limits` —
  duplicate-key-safe, depth- and string-bounded JSON parsing for untrusted
  input (directly reusable for HTTP request bodies).
- `src/hashing.rs` `sha256_hex` — content hashing for fingerprints and
  receipts.
- `src/error.rs` `ForgeError` + `exit_code` — the *CLI* classification
  boundary. Exit codes stay CLI-owned; services must return typed results
  instead of `ForgeError` booleans-as-classification.

## 2. Reusable service inventory and target layout

### 2.1 Liftable today (mechanical: make visible, stop writing inside the call)

| Capability | Function to lift | Work required |
|---|---|---|
| Conversion prepare | `src/pipeline.rs` `run_catalog_pipeline`, `run_component_pipeline`, `prepare_document` | Make `prepare_document` visibility public within the service layer; call from a service that returns `PipelineOutput` without writing. |
| Validation | `src/validate/mod.rs` `run_full_validation` (+ `detect_model_type`) | Wrap: bytes-in → `ValidationReport` out; rendering stays in adapters. |
| Mapping prepare | `src/mapping/mod.rs` `prepare`, `render_report` | Expose a typed prepared result (artifact bytes, `MappingReport`, input fingerprints, destination intent); move writes out. |
| Mapping scaffold | `src/mapping/mod.rs` `execute_init` internals (`scaffold_resource`) | Split scaffold construction (bytes) from write. |
| Applicability prepare | `src/applicability/mod.rs` `prepare_analysis`, `capture_input_fingerprint` | Same split; fingerprints become part of the prepared result. |
| Trace | `src/trace/mod.rs` `generate_trace_report` | Wrap with excerpt bounding policy; no write. |
| Diff | `src/diff/mod.rs` `diff_artifacts` | Already pure; add structured summary exposure. |
| Project/config | `src/config.rs` loaders | Wrap behind an explicit root context. |

### 2.2 Needs new extraction work (does not exist yet)

1. **Root-scoped project context** — canonical root, `forge.workspace/1` index,
   resource registration, path authority (rejection of symlinks, absolute
   paths, aliasing) per M-3/M-6.
2. **Destination/aliasing trust checks as a shared module** — today
   `validate_destinations` (mapping) and `validate_destination`
   (applicability) are near-duplicates; unify behind the effect boundary.
3. **Preview receipt and commit orchestration** — one-time receipts, hash
   rechecks, exact-byte atomic commit (Section 3.3/3.4).
4. **Operation registry** — stable IDs, state machine, idempotency-key
   retention (Section 3.5).
5. **Structured summary/queue view models** — typed counterparts of
   `format_summary_dashboard` and text renderers, with terminal rendering left
   to adapters.
6. **CLI-type de-inversion** — move `MappingReportFormat`, `MappingFailOn`,
   `ApplicabilityReportFormat`, `ApplicabilityFailOn`,
   `ApplicabilityStateFilter` semantics out of `src/cli/mod.rs` into the
   service layer (CLI re-exports or maps them so syntax is unchanged).

### 2.3 Target module layout (recommended)

New top-level `src/workspace/` module. `src/workspace` does not exist today
and conflicts with no existing module (`src/mapping`, `src/applicability`,
etc. are all distinct names). Rationale over a bare `src/services/`: the
four-phase service shape is the *workspace application layer* — it carries
session context, receipts, operations, and root-scoped path authority that are
PRD-062 concepts. Grouping everything the feature touches under one directory
makes review ownership obvious and prevents a generic `src/services` from
becoming a second attractor for domain logic. The CLI adapter calling
`workspace::services` is intended and one-way (the workspace never calls
`cli::execute`, per the PRD).

```text
src/workspace/
├── mod.rs              feature root: wiring only, no policy
├── services/           shared application services (no HTTP, no terminal, no shell)
│   ├── mod.rs
│   ├── context.rs      ProjectContext: canonical root, forge.workspace/1 index,
│   │                   registered resources, operation-scoped path authority
│   ├── conversion.rs   prepare conversion (catalog/component) -> PreparedOutput
│   ├── validation.rs   bytes/draft -> typed ValidationReport diagnostics
│   ├── mapping.rs      scaffold/build/check -> PreparedOutput + MappingReport
│   ├── applicability.rs scaffold/analyze -> PreparedOutput + ApplicabilityReport + queue model
│   ├── trace.rs        artifact+source -> typed provenance view model, bounded excerpts
│   ├── export.rs       report/export preparation and redaction profile
│   ├── effects.rs      preview receipts, hash recheck, exact-byte commit (Section 3.3-3.4)
│   ├── operations.rs   operation registry: IDs, state machine, idempotency (Section 3.5)
│   └── diagnostics.rs  shared typed diagnostic codes (no terminal formatting)
├── http/               Slice 1+: thin HTTP adapter (routes, JSON codecs, authz checks)
├── session.rs          Slice 1+: unlock, capabilities, machine bootstrap
└── assets.rs           Slice 1+: embedded shell + content-hash manifest (ADR-0003)
```

Domain modules keep their public pure APIs (`src/mapping`, `src/applicability`,
`src/validate`, `src/trace` stay where they are). Services compose them; they
do not fork them. The CLI adapter in `src/cli/` is modified in later slices to
call services and then perform its own rendering/writes — preserving current
syntax, output text, and `src/error.rs` `exit_code` mapping.

## 3. Boundary definitions (normative)

### 3.1 Validation boundary

Purpose: turn untrusted bytes or draft edits into typed diagnostics. No
terminal formatting, no file writes, no exit codes.

```rust
// src/workspace/services/diagnostics.rs + per-service validators
pub struct TypedDiagnostic {
    pub code: DiagnosticCode,        // stable, machine-readable (PRD "stable reason codes")
    pub severity: Severity,
    pub resource: Option<ResourceId>,// registered-resource pointer, not raw path
    pub field: Option<JsonPointer>,  // pointer into the draft document
    pub message: SafeMessage,        // bounded, escaped, no absolute paths or secrets
    pub retryable: bool,
}

pub trait ValidateDraft {
    type Draft;
    fn validate(&self, ctx: &ProjectContext, draft: &Self::Draft) -> ValidationOutcome;
}
```

Rule: adapters may render `TypedDiagnostic` values (text, JSON, HTTP error
envelopes) but never produce or mutate them. `src/json_strict.rs` `parse_value`
is the only accepted parser for request/draft JSON bodies.

### 3.2 Preparation boundary

Purpose: compute the complete proposed effect without performing it.

```rust
// src/workspace/services/{conversion,mapping,applicability,export}.rs
pub struct PreparedOutput {
    pub destination: DestinationIntent,   // project-relative target, create|overwrite
    pub bytes: Vec<u8>,                   // exact proposed artifact bytes
    pub semantic_summary: SemanticSummary, // counts/classifications, no excerpts
    pub classification: ResultClassification, // Ok | ReviewRequired(code) | Invalid(Vec<TypedDiagnostic>)
    pub input_fingerprints: Vec<Fingerprint>, // (resource, sha256, byte_len) — cf. capture_input_fingerprint
    pub base_hash: Option<Sha256>,        // current target content hash when overwriting
}
```

Rule: preparation is deterministic and side-effect free (M-19). It reuses the
existing prepare seams: `src/pipeline.rs` pipelines, `src/mapping/mod.rs`
`prepare`, `src/applicability/mod.rs` `prepare_analysis`. Rendering (JSON vs
text dashboards) is adapter work.

### 3.3 Preview-receipt boundary

Purpose: bind what a human reviewed to what a commit may write (M-13).

```rust
// src/workspace/services/effects.rs
pub struct PreviewReceipt {
    pub id: ReceiptId,                 // unguessable, single use
    pub session: SessionId,            // bound to session; invalidated on shutdown
    pub operation_type: OperationType,
    pub destination: DestinationIdentity, // resolved target identity (path + inode-equivalent)
    pub base_hash: Option<Sha256>,     // target content hash at preview time
    pub input_hashes: Vec<Sha256>,     // every input the preparation consumed
    pub bytes_hash: Sha256,            // hash of the exact proposed bytes
    pub expires_at: Timestamp,         // bounded, short-lived
}
```

Rules: receipts live in process memory only; any mismatch on commit (Section
3.4), expiry, or reuse invalidates the receipt (AC-11). Preview responses
carry the escaped text diff and semantic summary produced at preparation
time; the receipt is only minted after preparation succeeds.

### 3.4 Commit boundary

Purpose: write exactly the previewed bytes or nothing (M-14, AC-10).

```rust
pub struct CommitRequest {
    pub receipt: ReceiptId,
    pub observed_version: ResourceVersion, // conditional-mutation version (M-23)
    pub idempotency_key: IdempotencyKey,   // client-generated
}

pub struct CommitResult {
    pub operation: OperationId,
    pub committed_hash: Sha256,           // MUST equal receipt.bytes_hash
    pub status: CommitStatus,             // Committed | ReplayedOriginal | Rejected { code }
}
```

Rules: the service revalidates destination and input identities and rechecks
all hashes bound to the receipt before writing; the only writer is a shared
atomic writer extending `src/io.rs` `write_atomic` with destination-identity
revalidation immediately before the rename/persist (reject symlinked or
non-regular targets, recheck parent identity; race-resistant directory-relative
primitives where the platform supports them, documented fail-closed equivalent
otherwise — PRD "Project Containment"). A retry with the same idempotency key
and same request returns the original `CommitResult` without a second write
(AC-22).

### 3.5 Operation boundary

Purpose: make every unit of work queryable and safely retryable (M-24).

```rust
pub struct Operation {
    pub id: OperationId,               // stable for the session lifetime
    pub state: OperationState,         // Pending | Running | Succeeded | Failed | Cancelled
    pub terminal: Option<TerminalResult>, // retained for idempotent retry/query after response loss
    pub idempotency_key: Option<IdempotencyKey>,
}

// state machine: Pending -> Running -> Succeeded | Failed | Cancelled (terminal; no exits from terminal)
```

Rules: terminal results are retained in memory for the session so a client
that loses an HTTP response can determine whether an effect committed (AC-23).
Cancellation is cooperative and never converts a committed write into a
reported cancellation.

### 3.6 Effect boundary

A **material write** is any creation or modification of a file beneath the
project root (registered resources, `forge.workspace.json`, generated OSCAL
artifacts, manifests, reports, exports). Everything else — reads, validation,
preparation, diff computation, queue queries — is effect-free. Every material
write follows the two-step contract: `prepare -> preview (receipt)` then
`commit (receipt + version + idempotency key)`, exactly once. There is no
non-previewed write path. **Multi-file effects are out of scope for the MVP**
unless presented, confirmed, and committed as one documented atomic
transaction with rollback (PRD "Write Preview and Transaction Model"); single
prepared documents with an optional secondary output follow the existing
primary/secondary pattern in `src/cli/convert.rs` only if each output is
individually receipt-bound in the same transaction. Statements to stdout/stderr
and terminal exit codes are not effects; they are adapter output.

## 4. Adapter rules

1. HTTP (`src/workspace/http/`) and CLI (`src/cli/`) adapters are thin:
   translate transport input into a typed service request; translate the typed
   result into transport output.
2. No domain policy in adapters: no validation rules, no classification
   decisions, no serialization of domain documents, no diff computation.
3. No shell/subprocess, no `cli::execute` invocation, no stdout scraping, no
   spawning the `forge` binary inside the workspace (PRD M-5). The existing
   `oscal_cli` subprocess integration (`src/oscal_cli/`) remains CLI-only and
   is not exposed through the workspace API.
4. Shared request/result types: adapters bind to the same service request and
   result structs; neither defines its own domain vocabulary.
5. CLI compatibility: current syntax, output text, and `src/error.rs`
   `exit_code` values are unchanged. CLI classification of service results
   (e.g. `MappingReviewRequired`) happens in the CLI adapter.
6. HTTP responses derive from typed results using the same diagnostic codes;
   error envelopes follow the contract at `docs/api/forge-workspace-v1.openapi.yaml`.

### Capability-matrix row → service mapping

| PRD capability row | Service(s) |
|---|---|
| Session | `workspace::session` (Slice 1; not a domain service) |
| Project | `services::context` (summary, resource collection) |
| Resource registration | `services::context` + `services::effects` (index preview/commit) |
| Policy onboarding | `services::conversion` + `services::validation` + `services::effects` |
| Validation | `services::validation` (+ `src/validate/mod.rs` `run_full_validation`) |
| Applicability | `services::applicability` (+ `src/applicability` domain) |
| Mapping | `services::mapping` (+ `src/mapping` domain) |
| Review queues | `services::applicability` / `services::mapping` queue models + `services::diagnostics` codes |
| Provenance | `services::trace` (+ `src/trace`) |
| Effects | `services::effects` |
| Operations | `services::operations` |
| Reports/exports | `services::export` + `services::effects` |

## 5. Slice 0 scope statement

Slice 0 produces contract and architecture artifacts plus the offline drift
gate that validates them: this plan, ADR-0001 through ADR-0004, the
companion artifacts below, and the contract-validation harness
(`tests/api_contract_validation.rs` plus the vendored OpenAPI 3.1 meta-schema
at `schemas/openapi-3.1-schema-2022-10-07.json`) that ADR-0001 requires. It
changes **no** production behavior: no `src/` file, no CLI output. The service
extraction, de-inversion of CLI types, new `src/workspace/` module,
atomic-writer hardening, and parity fixtures are implemented in Slices 1-2
under this plan, after API review approval (PRD "Delivery Slices", Slice 0
exit evidence).

## Verification pointers for implementing slices

- Parity: API and CLI adapters over the same service request must produce
  byte-equal artifacts and equal classifications (M-19, AC-14); fixtures
  compare against the same committed contract fixtures used by the OpenAPI
  validation test (ADR-0001).
- Atomicity tests reuse the transaction cases in the PRD verification plan
  (expired/reused receipts, stale versions, response-loss retry, interruption).
- No adapter-owned policy is enforceable by a lint-level rule: `src/workspace/http`
  and `src/cli` may not import rendering-free domain internals beyond published
  service types.
