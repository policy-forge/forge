# Implementation Plan: OSCAL Back Matter Generation

**Branch**: `012-back-matter` | **Date**: 2026-02-12 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/012-back-matter/spec.md`

## Summary

Convert extracted `Citation` objects from WI-8 into OSCAL `back-matter.resources[]` entries and wire `link` elements into control bodies. Architecture follows AR Option 1 (Coordinated Two-Output Builder): `generate_back_matter` produces resources + resource map, `generate_control_links` produces link elements. URL validation uses the `url` crate; only `http`/`https` schemes are considered valid (per clarification). UUIDs are deterministic v5 with a dedicated back-matter namespace. No data in `remarks` fields.

## Technical Context

**Language/Version**: Rust edition 2024, stable 1.93.0
**Primary Dependencies**: serde 1.x, serde_json 1.x, uuid 1.20.0 (v5 feature, existing), url (NEW — latest stable), thiserror 2.0.18 (existing), tracing 0.1.44 (existing)
**Storage**: N/A — in-memory processing only
**Testing**: `cargo test` (unit + integration), TDD mandatory per Constitution IV
**Target Platform**: Local CLI (macOS/Linux)
**Project Type**: Single crate (`forge`)
**Performance Goals**: N/A — batch CLI, no latency targets; must handle documents with hundreds of citations without measurable overhead
**Constraints**: No arbitrary data in `remarks` (M-7/Parent PRD M-11); deterministic UUIDs (M-4); malformed URL preservation (M-8)
**Scale/Scope**: Typical policy documents contain 5–50 citations; edge case up to hundreds

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | PASS | New code in existing `src/oscal/back_matter.rs`; no new crate needed |
| II. Rust-First | PASS | Pure Rust, no FFI, no unsafe |
| III. Contract-First Development | PASS | Types and signatures defined in AR interface contract; implement types before logic |
| IV. Test-First Development | PASS | TDD mandatory; tests before implementation |
| V. Complete Implementation | PASS | All tasks must complete before merge |
| VI. Performance-First Design | PASS | Batch processing; `&str` parameters; iterator-based |
| VII. Security-First Design | PASS | SEC review complete (Low risk); SEC-1–4 incorporated |
| VIII. Error Handling Standards | PASS | `thiserror` for errors; no unwrap in production |
| IX. Observability | PASS | `tracing` for malformed URL warnings |
| X. Simplicity & Pragmatism | PASS | Two focused functions; concrete types; no trait objects |
| XI. Current Dependency Policy | PASS | `url` crate: MIT/Apache-2.0, latest stable, actively maintained |

**Gate result: PASS** — no violations.

## Project Structure

### Documentation (this feature)

```text
specs/012-back-matter/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── back_matter.rs   # Rust type contracts
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── oscal/
│   ├── mod.rs            # Add `pub mod back_matter;` and re-exports
│   ├── back_matter.rs    # NEW: structs + generate_back_matter + generate_control_links
│   ├── catalog.rs        # MODIFY: Add back_matter field to OscalCatalog, links to OscalControl
│   └── metadata.rs       # (unchanged)
├── error.rs              # MODIFY: Add BackMatter error variant
├── uuid.rs               # MODIFY: Add BACK_MATTER_NAMESPACE constant
├── model/
│   └── mod.rs            # MODIFY: Add Citation struct (WI-8 input contract)
└── lib.rs                # MODIFY: Add re-exports for back matter types
```

**Structure Decision**: New `back_matter.rs` module within existing `src/oscal/`. Follows established pattern (catalog.rs, metadata.rs). No new crate — module is small and tightly coupled to OSCAL output.

## Complexity Tracking

No constitution violations to justify.

## Implementation Guardrails (from AR)

- **DO NOT** embed citation text inline in control prose
- **DO NOT** store citation text, URLs, or structured metadata in `remarks` fields
- **DO NOT** use UUID v4 (random) for back matter resource UUIDs
- **DO NOT** silently drop malformed URLs — preserve with `prop` annotation
- **MUST** use dedicated UUID v5 namespace for back matter resources
- **MUST** generate `link` elements with `rel: "reference"` and `href: "#<uuid>"`
- **MUST** flag malformed URLs with `prop name="url-status" value="unvalidated"`

## Security Requirements (from SEC)

| SEC ID | Requirement | Verification |
|--------|-------------|--------------|
| SEC-1 | Output shall not contain data beyond source policy | Unit test |
| SEC-2 | No arbitrary data in OSCAL `remarks` fields | Unit test |
| SEC-3 | Citation URLs validated; malformed annotated with `prop` | Unit test |
| SEC-4 | Empty citation URLs treated as malformed and annotated | Unit test |

## Clarifications Applied

1. **URL scheme validation**: Only `http` and `https` valid; all others flagged `unvalidated`
2. **Zero citations**: Omit `back-matter` entirely from output
3. **Title derivation (M-5)**: Citation text preferred; URL-only citations use full URL as title

## Phase Summary

| Phase | Output | Status |
|-------|--------|--------|
| Phase 0: Research | `research.md` | Complete |
| Phase 1: Design & Contracts | `data-model.md`, `contracts/`, `quickstart.md` | Complete |
| Phase 2: Tasks | `tasks.md` | `/speckit.tasks` |
