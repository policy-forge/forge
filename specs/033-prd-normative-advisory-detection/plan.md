# Implementation Plan: Normative/Advisory Detection (WI-33)

**Branch**: `033-prd-normative-advisory-detection` | **Date**: 2026-02-17 | **Spec**: [PRD](../../docs/PRD/033-prd-normative-advisory-detection.md)
**Input**: [PRD](../../docs/PRD/033-prd-normative-advisory-detection.md) · [AR](../../docs/AR/033-ar-normative-advisory-detection.md) · [SEC](../../docs/SEC/033-sec-normative-advisory-detection.md)

## Summary

Add modality detection (WI-33) to the FORGE pipeline: extend `PolicyRequirement` with a `Modality` enum field, implement heuristic verb matching using cached `std::sync::LazyLock<Regex>` patterns with `\b` word boundary anchors, add an `annotate_modalities` enrichment pass to `prepare_document`, and emit an OSCAL `prop { name: "modality", value: "normative"|"advisory" }` on controls in both Catalog and Component Definition output paths. No new dependencies — `regex = "1"` is already in `Cargo.toml` and `std::sync::LazyLock` is stdlib (Rust 1.80+, project targets 1.93+).

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: `regex = "1"` (already in `Cargo.toml`), `std::sync::LazyLock` (stdlib, matching `citation.rs` pattern)
**Storage**: N/A — in-memory pipeline enrichment pass
**Testing**: `cargo test`, `cargo nextest`, `insta` (snapshot tests for OSCAL output)
**Target Platform**: Local CLI (cross-platform: macOS, Linux, Windows)
**Performance Goals**: Sub-microsecond per requirement classification (cached regex, no allocation per call)
**Constraints**: No NLP/ML, word-boundary-aware matching required (PRD Technical Constraints), TDD mandatory, no new crate dependencies
**Scale/Scope**: Policy documents with tens to thousands of atomized `PolicyRequirement`s

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-first | ✅ Pass | Feature added to existing `forge` library crate as a module (`src/parse/modality.rs`); `Modality` enum exported from `lib.rs` |
| II. Rust-first | ✅ Pass | Pure Rust; no `unsafe` code required |
| III. Contract-first | ✅ Pass | `Modality` enum, `ModalityResult` struct, and function signatures defined in contracts before implementation |
| IV. TDD | ✅ Pass | Tests written before implementation; all AC scenarios have corresponding test cases |
| V. Complete implementation | ✅ Pass | All M-1 through M-6 requirements must be complete before merge |
| VI. Performance | ✅ Pass | `LazyLock<Regex>` compiles patterns once; sub-microsecond per requirement |
| VII. Security | ✅ Pass | Static patterns (no user-supplied regex), no ReDoS risk (simple alternation), no network |
| VIII. Error handling | ✅ Pass | `ForgeError` propagated via `?`; no `unwrap` in production code (regex panic at startup is acceptable per AR) |
| IX. Observability | ✅ Pass | `tracing::warn!` for defaults/conflicts; `tracing::debug!` for matched verbs |
| X. Simplicity | ✅ Pass | Two static patterns, one enrichment function, one pipeline step added |
| XI. Dependencies | ✅ Pass | No new crate dependencies; `regex` already in `Cargo.toml`; `LazyLock` from stdlib |

**Constitution violations**: None. No complexity tracking required.

## Project Structure

### Documentation (this feature)

```text
specs/033-prd-normative-advisory-detection/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   └── rust-interface.md
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── model/
│   └── mod.rs           # EXTEND: add Modality enum; add modality: Option<Modality> to PolicyRequirement
├── parse/
│   ├── mod.rs           # EXTEND: pub mod modality; re-export annotate_modalities
│   └── modality.rs      # NEW: Modality, ModalityResult, detect_modality, annotate_modalities
├── oscal/
│   ├── catalog.rs       # EXTEND: add modality prop to OscalControl.props in build_catalog
│   └── implemented_requirements.rs  # EXTEND: add modality prop on implemented-requirement
├── pipeline.rs          # EXTEND: add Step 7c annotate_modalities in prepare_document
└── lib.rs               # EXTEND: re-export Modality

tests/
├── fixtures/            # EXTEND: add modality-focused fixture documents
└── (existing structure)
```

**Structure Decision**: Single-project (existing `forge` crate). The feature adds one new source module (`src/parse/modality.rs`) and extends five existing files. No new crates, no workspace restructuring.

## Phase 0: Research

See [`research.md`](research.md) — all NEEDS CLARIFICATION items resolved. No external research tasks required; all technical decisions are settled in the AR.

## Phase 1: Design & Contracts

See:
- [`data-model.md`](data-model.md) — entity definitions and field extensions
- [`contracts/rust-interface.md`](contracts/rust-interface.md) — Rust type signatures and OSCAL output format
- [`quickstart.md`](quickstart.md) — end-to-end usage example

## Implementation Order

Follow the AR's suggested implementation order:

1. **Domain model** (`src/model/mod.rs`): Add `Modality` enum; extend `PolicyRequirement` with `modality: Option<Modality>`
2. **Modality module** (`src/parse/modality.rs`): Define `ModalityResult`, `NORMATIVE_PATTERN`, `ADVISORY_PATTERN` as `LazyLock<Regex>`, implement `detect_modality`
3. **Unit tests — detection** (`src/parse/modality.rs` `#[cfg(test)]`): All normative verbs, all advisory verbs, mixed (conflict), missing (default), case variants, word-boundary false-positive prevention
4. **Enrichment pass** (`src/parse/modality.rs`): Implement `annotate_modalities`
5. **Unit tests — enrichment** (`src/parse/modality.rs` `#[cfg(test)]`): Document with mixed requirements, empty document, default/conflict warning paths
6. **Pipeline integration** (`src/pipeline.rs`): Add Step 7c `annotate_modalities` in `prepare_document` after citations step
7. **Catalog builder** (`src/oscal/catalog.rs`): Add modality prop to `OscalControl.props` when `requirement.modality` is `Some`
8. **Component Definition builder** (`src/oscal/implemented_requirements.rs`): Add modality prop on implemented-requirements
9. **Integration tests**: Mixed-modality fixture document → OSCAL output → verify prop presence on controls
10. **re-export** (`src/lib.rs`, `src/parse/mod.rs`): Expose `Modality` publicly *(Note: tasks.md places these re-exports in Phase 2 Foundational as T002/T007, before implementation — follow tasks.md ordering)*

## Security Requirements Integration

| SEC ID | Requirement | Implementation Location |
|--------|-------------|------------------------|
| SEC-1 | Requirement text logged at DEBUG only | `tracing::debug!` in `detect_modality`; no INFO-level text logging |
| SEC-2 | Word boundary anchors (`\b`) | `NORMATIVE_PATTERN` and `ADVISORY_PATTERN` use `\b` anchors |
| SEC-3 | No nested quantifiers (ReDoS prevention) | Simple alternation `(must\|shall\|will\|required)` — no nesting |
| SEC-4 | Compile once via `LazyLock` | `static NORMATIVE_PATTERN: LazyLock<Regex>` and `ADVISORY_PATTERN` |

## Acceptance Criteria Checklist

| AC ID | Requirement | Test Location |
|-------|-------------|---------------|
| AC-1 | "must" → normative | `modality.rs` unit test |
| AC-2 | "should" → advisory | `modality.rs` unit test |
| AC-3 | normative → OSCAL prop normative | integration test |
| AC-4 | advisory → OSCAL prop advisory | integration test |
| AC-5 | `modality` field on `PolicyRequirement` | compile-time |
| AC-6 | All controls have `props` with modality | integration test (Catalog + CompDef) |
| AC-7 | "MUST" case-insensitive → normative | `modality.rs` unit test |
| AC-8 | No verb → normative + warning | `modality.rs` unit test |
| AC-9 | Mixed verbs → normative + conflict warning | `modality.rs` unit test |
| AC-10 | Modality visible in JSON output | integration test |

## Definition of Done

- [ ] All tests passing: `cargo test --workspace` and `cargo nextest run --workspace`
- [ ] Coverage ≥ 90% for `modality.rs` (per AR Testing Strategy)
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo fmt --check` passes
- [ ] `cargo audit` passes (no new advisories — no new deps)
- [ ] Modality prop appears in OSCAL JSON for both Catalog and Component Definition output
- [ ] Snapshot tests updated with modality prop in output
- [ ] All PRD acceptance criteria (AC-1 through AC-10) have passing tests
- [ ] All SEC requirements (SEC-1 through SEC-4) verified
