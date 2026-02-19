# Implementation Plan: WI-34 Parameter Extraction

**Branch**: `034-prd-parameter-extraction` | **Date**: 2026-02-17 | **Spec**: docs/PRD/034-prd-parameter-extraction.md
**Input**: PRD `docs/PRD/034-prd-parameter-extraction.md`, AR `docs/AR/034-ar-parameter-extraction.md`, SEC `docs/SEC/034-sec-parameter-extraction.md`

## Summary

Implement OSCAL parameter extraction as a pipeline enrichment pass that detects time windows, thresholds, frequencies, and quantities embedded in `PolicyRequirement` text, extracts them into `PolicyParameter` objects with value domains and deterministic IDs, replaces the matched spans with OSCAL insertion placeholders (`{{ insert: param, id-ref: <param-id> }}`), and emits OSCAL `param` elements within the corresponding catalog controls. Implementation uses regex with named capture groups and four type-specific matchers composing a common `ParameterMatcher` trait. No new crate dependencies are required — `regex = "1"` is already present; `std::sync::LazyLock` (Rust 1.80+) replaces `once_cell`.

## Technical Context

**Language/Version**: Rust, Edition 2024, stable 1.93.0
**Primary Dependencies**: `regex = "1"` (already in Cargo.toml), `std::sync::LazyLock` (std, no new dep), `thiserror = "2.0.18"`, `serde = "1.0.228"`, `tracing = "0.1.44"`
**Storage**: N/A — transforms in-memory `PolicyDocument`; writes OSCAL JSON/XML/YAML to local filesystem (unchanged)
**Testing**: `cargo test`, `insta` (snapshot); 90%+ line coverage. Idempotence verified via manual double-extraction assertions (T034) — no `proptest` dependency required.
**Target Platform**: Linux/macOS CLI (local filesystem)
**Project Type**: Single binary crate (`forge`) — no workspace split needed
**Performance Goals**: O(n) linear time in number of requirements; document with hundreds of requirements processes without noticeable delay
**Constraints**: No NLP/ML; no ISO 8601 value normalization; idempotent enrichment pass; deterministic parameter IDs
**Scale/Scope**: Single-crate enrichment pass; adds ~400–600 LOC across 3 new files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First | ⚠️ DOCUMENTED EXCEPTION | Constitution Principle I states "every feature MUST begin as a standalone crate." WI-34 deviates from this by implementing the `parameter` module within the existing `forge` crate. **Formal exception rationale**: All prior pipeline enrichment modules (WI-6 atomization, WI-7 UUID, WI-8 citation, WI-33 modality) follow module-scoped isolation rather than crate isolation, establishing a project-wide precedent. The `PolicyDocument` domain model is shared across all pipeline stages; a standalone crate boundary would require either duplicating domain types or adding a heavy inter-crate dependency. This exception applies exclusively to pipeline enrichment passes that (a) operate directly on `PolicyDocument` in-memory, (b) have no external network or storage dependencies, and (c) are independently testable at the module level. This exception is reviewed at Phase 2 release (WI-35). |
| II. Rust-First | ✅ PASS | Pure Rust; no FFI; no unsafe code. |
| III. Contract-First | ✅ PASS | `PolicyParameter`, `ParameterType`, `ParameterConstraint`, `ConstraintType`, `ParameterMatch`, `ParameterMatcher` trait fully specified in AR before implementation. |
| IV. Test-First | ✅ PASS | TDD mandatory; tests written before implementation per constitution; 90%+ coverage target. |
| V. Complete Implementation | ✅ PASS | All tasks in tasks.md must complete before merge. |
| VI. Performance | ✅ PASS | Regex is O(n) in text length per requirement; enrichment pass is O(k) in number of requirements. |
| VII. Security | ✅ PASS | No nested quantifiers in patterns; `std::sync::LazyLock` for one-time compilation; qualifier words prevent false positives; SEC requirements SEC-1 through SEC-6 incorporated. |
| VIII. Error Handling | ✅ PASS | `thiserror`-based `ForgeError` extended with `ParameterExtraction` variant; no unwrap in production. |
| IX. Observability | ✅ PASS | INFO-level extraction count; DEBUG-level per-parameter; TRACE-level pattern matches. |
| X. Simplicity | ✅ PASS | `ParameterMatcher` trait has 4 implementations (justified in PRD per extensibility docs); no premature abstraction. |
| XI. Dependencies | ✅ PASS | Zero new dependencies; `regex` already present; `once_cell` replaced by `std::sync::LazyLock`. |

**Complexity Tracking**: No violations. No overrides required.

## Project Structure

### Documentation (this feature)

```text
specs/034-prd-parameter-extraction/
├── plan.md              # This file
├── research.md          # Phase 0 output (decisions from PRD/AR)
├── data-model.md        # Phase 1 output (domain types + OSCAL types)
├── quickstart.md        # Phase 1 output (build/test/run guide)
├── contracts/           # Phase 1 output (Rust interface contracts)
│   ├── parameter_types.rs   # Domain types contract
│   └── parameter_api.rs     # Public API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── model/
│   └── mod.rs           # MODIFIED: Add PolicyParameter, ParameterType, ParameterConstraint,
│                        #           ConstraintType types; add `parameters` field to PolicyRequirement
├── parameter/
│   ├── mod.rs           # NEW: Public API: extract_parameters, extract_parameters_from_text,
│   │                    #      parameter_id, to_oscal_param; re-exports
│   └── matchers.rs      # NEW: ParameterMatcher trait, ParameterMatch struct,
│                        #      TimeWindowMatcher, ThresholdMatcher, FrequencyMatcher, QuantityMatcher
├── oscal/
│   └── catalog.rs       # MODIFIED: Add OscalParam, OscalParamConstraint structs;
│                        #           add `params` field to OscalControl;
│                        #           update build_catalog to emit param elements
├── pipeline.rs           # MODIFIED: Add extract_parameters step after citation extraction
├── error.rs              # MODIFIED: Add ParameterExtraction variant to ForgeError
└── lib.rs                # MODIFIED: pub mod parameter;
```

**Structure Decision**: Single-crate, module-based following existing patterns (`src/citation.rs` → `src/parameter/`). Module directory (not single file) chosen because the parameter module contains a trait + 4 implementations + 2 public API functions — estimated 500 LOC which would exceed the 400-line guideline for a single file. Splitting into `mod.rs` (public API) + `matchers.rs` (trait + implementations) stays within bounds.

## Phase 0: Research Summary

*All decisions resolved from PRD/AR artifacts. No NEEDS CLARIFICATION items remain.*

See `research.md` for full decision log.

**Key resolved decisions:**

| Decision | Resolution |
|----------|-----------|
| Pattern matching strategy | Regex with named capture groups (AR Option 1) |
| `once_cell` dependency | Use `std::sync::LazyLock` (std 1.80+, project uses 1.93) — no new dep |
| WI-33 modality dependency | WI-34 runs on ALL requirements (modality field absent until WI-33 completes); consistent with "runs in parallel" stance |
| Domain type location | `PolicyParameter` + supporting enums in `src/model/mod.rs` (same as `Citation`); extraction logic in `src/parameter/` |
| OSCAL param placement | `OscalParam` added to `src/oscal/catalog.rs`; emitted as `params` array on `OscalControl` |
| Parameter ID format | `"{requirement_id}_prm_{position}"` (AR spec) |
| Error variant | `ForgeError::ParameterExtraction(String)` — exit code 2 (parse/transform error) |
| Overlap resolution | Sort matches by `start`, first-match-wins on overlap, reverse-order replacement |

## Phase 1: Design Summary

See `data-model.md` for entity definitions, `contracts/` for Rust interface contracts, `quickstart.md` for build/test guide.

**Data model additions** (see `data-model.md`):
- `PolicyParameter` — extracted parameter with id, requirement_id, label, value, parameter_type, constraint
- `ParameterType` — `TimeWindow | Threshold | Frequency | Quantity`
- `ParameterConstraint` — `constraint_type: ConstraintType, value: String`
- `ConstraintType` — `Minimum | Maximum | Exact`
- `PolicyRequirement.parameters: Vec<PolicyParameter>` field (always empty until WI-34 enrichment runs)

**Internal types** (in `src/parameter/matchers.rs`):
- `ParameterMatch` — intermediate extraction result with byte offsets
- `ParameterMatcher` trait — `fn find_parameters(&self, text: &str) -> Vec<ParameterMatch>`
- `TimeWindowMatcher`, `ThresholdMatcher`, `FrequencyMatcher`, `QuantityMatcher`

**OSCAL output types** (in `src/oscal/catalog.rs`):
- `OscalParam` — `{ id, label, values, constraints }` matching OSCAL v1.2.0 schema
- `OscalParamConstraint` — `{ description }`
- `OscalControl.params: Vec<OscalParam>` field (skip_serializing_if empty)

**Pipeline integration** (in `src/pipeline.rs`, `prepare_document()`):
```
// Step 7c: Extract parameters (WI-34, after UUID assignment and citation extraction)
// extract_parameters takes &mut PolicyDocument and returns Result<(), ForgeError>
crate::parameter::extract_parameters(&mut doc)?;
```

## Implementation Order

Per AR `Suggested Implementation Order`:

1. Add `PolicyParameter`, `ParameterType`, `ParameterConstraint`, `ConstraintType` to `src/model/mod.rs`; add `parameters` field to `PolicyRequirement`
2. Add `ForgeError::ParameterExtraction` variant; update `exit_code()`
3. Create `src/parameter/matchers.rs`: `ParameterMatch`, `ParameterMatcher` trait, `TimeWindowMatcher` (with tests — TDD)
4. Add `ThresholdMatcher` (with tests)
5. Add `FrequencyMatcher` (with tests)
6. Add `QuantityMatcher` (with tests)
7. Create `src/parameter/mod.rs`: `extract_parameters_from_text` with multi-matcher orchestration + span replacement (with tests)
8. Add `parameter_id` deterministic ID generator (with tests)
9. Add `extract_parameters` document-level enrichment pass (with tests)
10. Add `to_oscal_param` converter (with tests)
11. Add `OscalParam`, `OscalParamConstraint`, `params` field to `src/oscal/catalog.rs`; update catalog builder
12. Wire into `src/pipeline.rs`
13. Add `pub mod parameter` to `src/lib.rs`
14. Integration tests: OSCAL output with param elements, idempotence, negative fixtures

## Acceptance Criteria Traceability

| AC ID | PRD Req | Implementation Target |
|-------|---------|----------------------|
| AC-1 | M-1, M-4 | `TimeWindowMatcher` + `extract_parameters_from_text` |
| AC-2 | M-2, M-4 | `ThresholdMatcher` (minimum) |
| AC-3 | M-2, M-4 | `ThresholdMatcher` (maximum) |
| AC-4 | M-3, M-4 | `FrequencyMatcher` |
| AC-5 | M-5 | `to_oscal_param` |
| AC-6 | M-6 | `OscalControl.params` linkage in catalog builder |
| AC-7 | M-7 | Span replacement in `extract_parameters_from_text` |
| AC-8 | M-8 | `extract_parameters` document-level pass |
| AC-9 | S-1 | `QuantityMatcher` |
| AC-10 | S-2 | Constraint inference in all matchers |

## Security Requirements Integration

| SEC ID | Implementation Location |
|--------|------------------------|
| SEC-1 | `tracing::debug!` for param values; no `tracing::info!` of values |
| SEC-2 | All matchers require qualifier words; negative test fixtures required |
| SEC-3 | All patterns use bounded quantifiers; no nested repetition |
| SEC-4 | `std::sync::LazyLock` for one-time regex compilation |
| SEC-5 | Position-sorted + reverse-order replacement in `extract_parameters_from_text` |
| SEC-6 | Idempotence test: extract twice on same document, assert identical results |
