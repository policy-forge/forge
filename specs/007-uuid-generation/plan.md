# Implementation Plan: Deterministic UUID Generation

**Branch**: `007-uuid-generation` | **Date**: 2026-02-11 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-uuid-generation/spec.md`

## Summary

Implement deterministic UUID v5 generation for `PolicyRequirement` elements using a fixed FORGE namespace UUID and whitespace-normalized requirement text. This ensures identical policy content always produces identical OSCAL identifiers across conversion runs, satisfying product principle P-3 (Deterministic and auditable) and parent PRD requirements M-8, AC-8, EC-5, EC-6, and Spike-4.

**Technical approach**: Add the `uuid` crate with v5 feature. Create a dedicated `src/uuid.rs` module containing a `FORGE_NAMESPACE_UUID` constant (project-specific UUID v4), `normalize_for_hashing()`, and `generate_stable_id()` as pure functions. Implement `assign_stable_ids()` to walk the `PolicyDocument` tree and populate `stable_id` on every `PolicyRequirement`. Integration into the conversion pipeline (automatic invocation after requirement atomization, WI-6) is deferred to a follow-up WI/PR; this WI only introduces the pure functions and tree-walk helper.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: uuid (v5 feature, NEW), tracing (NEW, Constitution IX), pulldown-cmark 0.13.x, serde 1.x, serde_json 1.x, sha2 0.10.x, clap 4, thiserror 2.0.18 (existing)
**Storage**: N/A — in-memory processing only
**Testing**: `cargo test` (TDD mandatory); `cargo clippy -- -D warnings`; `cargo fmt --check`; `cargo mutants`
**Target Platform**: Local CLI (macOS, Linux)
**Project Type**: Single Rust binary crate
**Performance Goals**: Negligible overhead — UUID v5 is SHA-1 hash of short strings (microseconds per requirement)
**Constraints**: Pure functions with no side effects; no I/O; no runtime configuration of namespace UUID
**Scale/Scope**: Hundreds of requirements per document (not billions); policy documents up to 10 MB

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v3.2.0 ratified with 11 core principles. Key alignment:

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First (NON-NEG) | **DEFERRED** | Project is a single binary crate; workspace migration deferred to separate WI |
| III. Contract-First (NON-NEG) | ✅ Pass | Contracts defined in contracts/uuid-module.md; types defined before implementation |
| IV. Test-First (NON-NEG) | ✅ Pass | TDD mandatory; tests written before implementation in US1 |
| VI. Performance-First (NON-NEG) | ✅ Pass | Criterion benchmark task added (T035) |
| IX. Observability | ✅ Pass | Uses `tracing` (NOT `log`); `#[instrument]` on public functions (T034) |
| X. Simplicity | ✅ Pass | Minimal implementation; 3 functions + 1 constant |
| XI. Dependency Policy (NON-NEG) | ✅ Pass | `cargo audit` check task added (T032); latest stable versions via `cargo add` |

## Project Structure

### Documentation (this feature)

```text
specs/007-uuid-generation/
├── plan.md              # This file
├── research.md          # Phase 0 output — technology decisions
├── data-model.md        # Phase 1 output — entity definitions
├── quickstart.md        # Phase 1 output — developer onboarding
├── contracts/           # Phase 1 output — function contracts
│   └── uuid-module.md   # Public API contract for src/uuid.rs
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point (existing)
├── lib.rs               # Module declarations (add `pub mod uuid;`)
├── error.rs             # ForgeError enum (existing, no changes needed)
├── uuid.rs              # NEW: UUID generation module
│   ├── FORGE_NAMESPACE_UUID constant
│   ├── normalize_for_hashing()
│   ├── generate_stable_id()
│   └── assign_stable_ids() + assign_stable_ids_to_section()
├── model/
│   └── mod.rs           # Domain model (PolicyDocument, PolicySection, PolicyRequirement)
│                        # NOTE: Currently empty — depends on WI-5/WI-6
├── cli/
│   └── convert.rs       # Pipeline orchestration (will call assign_stable_ids)
├── ingest/
│   └── mod.rs           # File ingestion (existing, no changes)
├── parse/
│   ├── mod.rs           # Section extraction (existing, no changes)
│   └── clauses.rs       # Clause extraction (existing, no changes)
├── export/
│   └── mod.rs           # (existing, no changes)
├── oscal/
│   └── mod.rs           # (existing, no changes)
└── validate/
    └── mod.rs           # (existing, no changes)
```

**Structure Decision**: Single Rust binary crate. UUID generation lives in a dedicated `src/uuid.rs` module (per clarification session decision) for separation of concerns and reusability (PRD S-2). The module depends only on the `uuid` crate and the domain model types from `src/model/`.

## Complexity Tracking

> No constitution violations. Feature is a single-module addition with 3 public functions and 1 constant.

| Aspect | Justification |
|--------|---------------|
| Separate uuid.rs module | PRD S-2 requires reusable function for any string input; separation enables this |
| Dependency on uuid crate | Required for RFC 4122 UUID v5 compliance; OSCAL expects standard UUIDs |

## Implementation Guardrails (from AR)

- **DO NOT** use UUID v4 (random) for requirement identifiers — violates determinism (parent PRD Decision Log)
- **DO NOT** hash raw un-normalized text — whitespace changes would produce different UUIDs (PRD M-2)
- **DO NOT** make the namespace UUID configurable at runtime — accidental changes break ID stability (PRD S-1)
- **DO NOT** over-normalize (lowercasing, punctuation removal) — risks false collisions (PRD W-3)
- **MUST** use `uuid` crate with UUID v5 and the fixed `FORGE_NAMESPACE_UUID` (PRD M-1)
- **MUST** normalize with `split_whitespace().join(" ")` before hashing (PRD M-2)
- **MUST** populate `stable_id` on every `PolicyRequirement` in the document tree (PRD M-3)

## Security Requirements (from SEC)

| SEC ID | Requirement | Verification |
|--------|-------------|-------------|
| SEC-1 | UUID generation must handle empty requirement text without panicking | Unit test |
| SEC-2 | Whitespace normalization must handle adversarial patterns (Unicode, mixed tabs/newlines) | Unit test |
| SEC-3 | FORGE_NAMESPACE_UUID must not be configurable at runtime | Code review (compile-time const) |
| SEC-4 | UUID generation function must be pure (no side effects, no I/O) | Code review |

## Dependency Analysis

### Upstream Dependencies (WI-5, WI-6)

The `model/mod.rs` file is currently **empty**. This feature requires:

- **WI-5** (`PolicyRequirement.stable_id: Option<String>` field definition)
- **WI-6** (atomized requirements — `PolicyDocument` with populated `PolicySection`/`PolicyRequirement` trees)

**Strategy**: The core UUID functions (`normalize_for_hashing`, `generate_stable_id`) are self-contained and depend only on `&str` input. These can be implemented and fully tested independently. The `assign_stable_ids` function and pipeline integration depend on the domain model types. Two approaches:

1. **If WI-5/WI-6 are complete before implementation**: Implement against real types
2. **If WI-5/WI-6 are not yet complete**: Define minimal stub types in `model/mod.rs` sufficient for UUID assignment (can be expanded by WI-5/WI-6 later)

The plan below sequences core UUID logic first (no domain model dependency), then integration second.

### New Dependency

| Crate | Version | Feature | License | Purpose |
|-------|---------|---------|---------|---------|
| uuid | Latest stable | `v5` | MIT/Apache-2.0 | UUID v5 generation (SHA-1 namespace + name) |

## Phase Summary

| Phase | Deliverable | Status |
|-------|-------------|--------|
| Phase 0 | research.md | Complete |
| Phase 1 | data-model.md, contracts/, quickstart.md | Complete |
| Phase 2 | tasks.md | Pending (/speckit.tasks) |
