# Implementation Plan: OSCAL Catalog Statement Parts & Prose

**Branch**: `010-catalog-statement-parts` | **Date**: 2026-02-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/010-catalog-statement-parts/spec.md`

## Summary

Extend the WI-9 Catalog builder to populate `OscalControl` structs with `parts[]` arrays (statement, guidance) and `props[]` arrays (`forge:source-line`). Add `OscalPart` and `OscalProp` structs to a new `src/oscal/parts.rs` module. Implement composable builder functions (`build_control_parts`, `build_control_props`) that integrate into the existing `build_catalog` function. Statement parts are mandatory for every control; guidance parts are generated when `PolicySection.body_text` is present. Objective parts are deferred (no domain model signal). Structured metadata uses only `forge:source-line` as a prop. No `remarks` field is introduced.

**AR Decision**: Option 1 — Structured Builders Extending WI-9 (see `docs/AR/010-ar-catalog-statement-parts.md`).

**Security**: Low risk per SEC review. SEC-1 through SEC-5 constraints enforced. No network I/O, no unsafe, no external input beyond validated domain model.

## Technical Context

**Language/Version**: Rust (edition 2024, stable 1.93.0)
**Primary Dependencies**: serde 1.x, serde_json 1.x, thiserror 2.0.18, tracing 0.1.44 (all existing — no new dependencies)
**Storage**: N/A (in-memory processing only)
**Testing**: `cargo test`, TDD mandatory (Constitution IV)
**Target Platform**: CLI (local)
**Project Type**: Single Rust binary crate
**Performance Goals**: Trivial computation — string copying and struct construction; no specific targets
**Constraints**: OSCAL v1.2.0 parts structure compliance; no `remarks` misuse (Parent PRD M-11)
**Scale/Scope**: Operates on in-memory `PolicyDocument`; scales with document size (hundreds of controls typical)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | ✅ PASS | Extends existing `oscal` module with new `parts.rs` submodule; no new crate needed (single binary crate) |
| II. Rust-First | ✅ PASS | Rust only; no FFI, no unsafe |
| III. Contract-First | ✅ PASS | `OscalPart`, `OscalProp` structs and builder signatures defined before implementation (see contracts/) |
| IV. Test-First (TDD) | ✅ PASS | TDD mandatory; tests written before implementation |
| V. Complete Implementation | ✅ PASS | All tasks complete before merge |
| VI. Performance-First | ✅ PASS | Trivial computation; no hot paths; no benchmarks needed |
| VII. Security-First | ✅ PASS | SEC review completed (Low risk); SEC-1–5 constraints listed in AR guardrails |
| VIII. Error Handling | ✅ PASS | Uses `ForgeError::CatalogBuild` for errors; `tracing::warn` for warnings |
| IX. Observability | ✅ PASS | `tracing::debug` for part counts per control; `tracing::warn` for empty text edge cases |
| X. Simplicity (YAGNI) | ✅ PASS | Two builder functions, two struct types; no abstractions beyond requirements |
| XI. Current Dependencies | ✅ PASS | No new dependencies; all existing crates at current stable versions |

**Gate result**: PASS — no violations.

## Project Structure

### Documentation (this feature)

```text
specs/010-catalog-statement-parts/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── rust-interfaces.md  # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── oscal/
│   ├── mod.rs           # MODIFY: add `pub mod parts;` and re-exports
│   ├── catalog.rs       # MODIFY: extend OscalControl with parts/props, update build_catalog
│   ├── metadata.rs      # UNCHANGED (WI-11)
│   └── parts.rs         # NEW: OscalPart, OscalProp, builder functions
├── model/
│   └── mod.rs           # UNCHANGED (PolicyRequirement, PolicySection already have needed fields)
├── error.rs             # UNCHANGED (ForgeError::CatalogBuild already exists)
└── lib.rs               # MODIFY: re-export new types
```

**Structure Decision**: Extend the existing `src/oscal/` module with a new `parts.rs` submodule. This keeps parts logic independently testable while composing naturally with the WI-9 Catalog builder. The existing single-crate structure is maintained per Constitution I (no new crate needed for this scope).

## Implementation Guardrails (from AR)

- **DO NOT** store structured data in `remarks` — use `props` (Parent PRD M-11)
- **DO NOT** add `param` elements — deferred to WI-34 (PRD W-1)
- **DO NOT** add back matter links — deferred to WI-12 (PRD W-2)
- **DO NOT** embed JSON/structured formats in `prose` — prose is human-readable text
- **DO NOT** generate parts without IDs — every part gets `{control-id}_{suffix}` (PRD M-3)
- **MUST** generate at least one statement part per control (PRD M-1)
- **MUST** use `{control-id}_smt` convention for statement part IDs (PRD M-3)
- **MUST** use `forge:` prefix for FORGE-specific prop names (Risk R-3 mitigation)
- **MUST** default all requirement text to statement parts; guidance only when explicitly signaled (A-3)

## Security Requirements (from SEC)

| SEC ID | Requirement | Verification |
|--------|-------------|-------------|
| SEC-1 | Statement prose MUST be direct copy of `PolicyRequirement.text` — no transformation/truncation | Unit test |
| SEC-2 | Empty requirement text → statement part with empty prose + logged warning, not panic | Unit test |
| SEC-3 | Structured metadata as `OscalProp`, never in `remarks` | Unit test + code review |
| SEC-4 | Part/prop generation MUST be pure functions — no I/O, no side effects | Code review |
| SEC-5 | `forge:` namespace prefix for all FORGE-specific prop names | Code review |

## Complexity Tracking

No violations to justify — all constitution gates pass.
