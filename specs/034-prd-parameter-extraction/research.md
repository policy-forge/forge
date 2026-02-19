# Research: WI-34 Parameter Extraction

**Date**: 2026-02-17 | **Phase**: 0 — Pre-implementation research
**Source artifacts**: PRD-034, AR-034, SEC-034, codebase analysis

All decisions resolved from PRD/AR artifacts. No NEEDS CLARIFICATION items remain.

---

## Decision Log

### D-1: Pattern Matching Strategy

**Decision:** Regex with named capture groups (AR Option 1)

**Rationale:** Deterministic, auditable, already in dependency tree (`regex = "1"`). Named capture groups (`(?P<qualifier>...)`, `(?P<value>...)`, `(?P<unit>...)`) provide readable, independently-testable patterns per parameter type. Option 2 (template DSL) requires building a parser — over-engineering per constitution X. Option 3 (bracket detection) requires pre-annotated documents, defeating automation.

**Alternatives considered:** Template literal parsing (too complex), bracket/placeholder detection (requires pre-annotation)

---

### D-2: Static Regex Compilation

**Decision:** Use `std::sync::LazyLock` (Rust standard library, stable since 1.80.0)

**Rationale:** Project uses Rust 1.93.0. `LazyLock<Regex>` provides one-time initialization without `once_cell` crate overhead. Avoids adding a new dependency (constitution XI). Functionally equivalent to `once_cell::sync::Lazy`. AR-034 cites `once_cell` — this resolves the dependency without adding it.

**Alternatives considered:** `once_cell::sync::Lazy` (requires new dependency), `regex::RegexSet` (single pass but loses capture group ergonomics), compile-time regex via `regex_macro` (immature)

---

### D-3: Modality Dependency (WI-33)

**Decision:** Parameter extraction runs on ALL `PolicyRequirement` instances, regardless of modality. The `modality` field is not yet present on `PolicyRequirement` (WI-33 not yet implemented).

**Rationale:** PRD A-3 states modality "enables parameter extraction to focus on normative requirements." However, the PRD also states WI-34 "runs in parallel with WI-33." WI-33 has not yet added a `modality` field to `PolicyRequirement`. Running on all requirements is consistent with the "parallel" relationship and avoids a hard dependency. Post-WI-33, parameter extraction can optionally filter to normative requirements as a performance optimization.

**Alternatives considered:** Block on WI-33 completion (violates parallel schedule), add modality field stub here (scope creep for WI-34)

---

### D-4: Domain Type Location

**Decision:** `PolicyParameter`, `ParameterType`, `ParameterConstraint`, `ConstraintType` defined in `src/model/mod.rs`. Extraction logic in new `src/parameter/` module.

**Rationale:** Consistent with `Citation` pattern: domain struct defined in `src/model/mod.rs`, extraction logic in `src/citation.rs`. Having domain types in the model module allows all pipeline consumers (OSCAL builders, future validation) to import them from a single canonical location. Extraction logic is separate to maintain cohesion.

**Alternatives considered:** All types in `src/parameter/mod.rs` with re-export (import path inconsistency), new `src/model/parameter.rs` sub-module (additional indirection for small set of types)

---

### D-5: OSCAL `param` Element Structure

**Decision:** Add `OscalParam` and `OscalParamConstraint` structs to `src/oscal/catalog.rs`. Add `params: Vec<OscalParam>` to `OscalControl`. Emit params during catalog building from `requirement.parameters`.

**Rationale:** OSCAL v1.2.0 Catalog schema places `params` at the control level (as siblings to `parts` and `props`). Adding to `OscalControl` is consistent with how `parts` and `props` are already handled. Schema field name: `"params"` (already used by OSCAL). `OscalParam` structure: `{ id, label, values: Vec<String>, constraints: Vec<OscalParamConstraint> }`. Note: OSCAL uses `"values"` (plural, array) not `"value"` (singular).

**Alternatives considered:** Separate post-processing step (complicates pipeline, risks missed params), embedding in `OscalPart` (wrong OSCAL schema position)

---

### D-6: Parameter ID Format

**Decision:** `"{requirement_id}_prm_{position}"` where position is the 0-based index of the parameter within the requirement.

**Rationale:** Directly from AR-034: `parameter_id(req_id, value, position) -> format!("{}_prm_{}", requirement_id, position)`. Content-based and deterministic — same requirement always produces same IDs at same positions. Does not require hashing for disambiguation at this scale.

**Alternatives considered:** Hash-based IDs (overkill for position-indexed params), UUID v5 from content (acceptable but more complex than necessary)

---

### D-7: Error Variant

**Decision:** Add `ForgeError::ParameterExtraction(String)` variant to `src/error.rs`. Map to exit code 2 (parse/transform error).

**Rationale:** Follows existing pattern (`CatalogBuild(String)`, `BackMatter(String)`). Exit code 2 is appropriate for transform-stage errors. The parameter extraction step is logically equivalent to atomization or citation extraction.

**Alternatives considered:** Reuse `ForgeError::Parse(String)` (less specific, harder to diagnose), propagate as generic `ForgeError::Io` (wrong category)

---

### D-8: Overlap Resolution

**Decision:** Sort all `ParameterMatch` objects by `start` byte offset (ascending). On overlap (match A ends after match B starts), keep the match with the earlier start; ties broken by longer match. Process replacements in reverse order (highest start offset first) to preserve byte offsets of earlier spans.

**Rationale:** Directly from AR-034 "Multi-Matcher Extraction with Span-Based Replacement" algorithm. Reverse-order replacement is a well-known technique for in-place string modification with multiple spans. "First match wins" is deterministic and predictable.

**Alternatives considered:** Collect all matches then merge intervals (equivalent but more complex), allow overlaps with last-write-wins (non-deterministic)

---

### D-9: Idempotence Mechanism

**Decision:** OSCAL insertion placeholders `{{ insert: param, id-ref: <param-id> }}` are not re-matched by any parameter regex because none of the patterns contain `{`, `}`, `insert`, `id-ref`, or `param` as extraction triggers. Idempotence is structural, not guarded.

**Rationale:** Per AR-034: "Insertion placeholders are not re-matched by parameter regexes." Verification via unit test: extract twice on same document, assert text and parameters are identical. No special guard code needed.

**Alternatives considered:** Track extracted parameter IDs in a set and skip on second pass (adds state, complicates function signature), check if text already contains placeholder before extraction (fragile)

---

## Regex Patterns (Confirmed)

Patterns confirmed against PRD implementation guidance and AR-034 interface definitions:

```
Time window (minimum/interval):
  (?i)(?P<qualifier>within|after|every)\s+(?P<value>\d+)\s+(?P<unit>days?|weeks?|months?|years?)

Threshold minimum:
  (?i)(?P<qualifier>at\s+least|minimum|no\s+fewer\s+than|no\s+less\s+than)\s+(?P<value>\d+[\w-]*)

Threshold maximum:
  (?i)(?P<qualifier>no\s+more\s+than|maximum|at\s+most)\s+(?P<value>\d+[\w-]*)

Frequency:
  (?i)(?:at\s+least\s+)?(?P<value>annually|quarterly|monthly|weekly|daily|biannually|semi-annually)

Quantity:
  (?i)(?P<qualifier>at\s+least|no\s+fewer\s+than|minimum)\s+(?P<value>\d+)\s+(?P<unit>\w+)
```

**ReDoS safety confirmed:** All patterns use bounded quantifiers (`\d+`, `\s+`, `\w+`). No nested repetition. `regex` crate's RE2-style engine provides linear-time guarantees regardless of input.

**False positive prevention confirmed:** All patterns require a qualifier word or specific keyword trigger. Bare numbers without context are never extracted. Section references (e.g., "Section 3.2", "NIST SP 800-53") use alphanumeric identifiers without qualifier words and will not be matched.

---

## Constraint Qualifier Mapping (Confirmed)

```
qualifier "within"        → TimeWindow, ConstraintType::Minimum
qualifier "after"         → TimeWindow, ConstraintType::Minimum
qualifier "every"         → TimeWindow, ConstraintType::Exact
qualifier "at least"      → Threshold/Quantity, ConstraintType::Minimum
qualifier "minimum"       → Threshold/Quantity, ConstraintType::Minimum
qualifier "no fewer than" → Threshold/Quantity, ConstraintType::Minimum
qualifier "no less than"  → Threshold/Quantity, ConstraintType::Minimum
qualifier "no more than"  → Threshold, ConstraintType::Maximum
qualifier "maximum"       → Threshold, ConstraintType::Maximum
qualifier "at most"       → Threshold, ConstraintType::Maximum
frequency (no qualifier)  → Frequency, ConstraintType::Exact
```

---

## Codebase Analysis Findings

| Finding | Impact |
|---------|--------|
| `PolicyRequirement` has no `parameters` field | Must add `parameters: Vec<PolicyParameter>` and update all constructors/tests |
| `PolicyRequirement` has no `modality` field | WI-33 not yet implemented; proceed without modality dependency (D-3) |
| `regex = "1"` already in `Cargo.toml` | No new dependency needed |
| `once_cell` NOT in `Cargo.toml` | Use `std::sync::LazyLock` instead (D-2) |
| `OscalControl` has no `params` field | Must add `params: Vec<OscalParam>` to `OscalControl` |
| `Citation` struct lives in `src/model/mod.rs` | Follow same pattern for `PolicyParameter` |
| Pipeline `prepare_document()` ends after citation extraction | Add parameter extraction step here |
| `ForgeError` uses `thiserror` with exit codes | Add `ParameterExtraction` variant, map to exit code 2 |
