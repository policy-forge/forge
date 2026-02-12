# Research: Internal Domain Model

**Feature**: 005-domain-model
**Date**: 2026-02-11
**Status**: Complete

## Overview

This document consolidates research findings and architectural decisions for the internal domain model implementation. All technical unknowns were resolved through formal clarification workflow (see `spec.md` Clarifications section) and architecture review (see `docs/AR/005-ar-domain-model.md`).

---

## Decision 1: Domain Model Structure

**Decision**: Plain struct hierarchy with Option fields (AR Option 1)

**Rationale**:
- Maximum simplicity for data-carrying structures
- `Option` fields enable incremental enrichment by downstream WIs (WI-6, WI-7, WI-8) without breaking changes
- Rust's type system enforces Option handling at compile time, preventing invalid state
- Constitution principle X (YAGNI) explicitly warns against premature abstraction
- No need for trait polymorphism or staged type variants for this use case

**Alternatives Considered**:

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| Trait-based model with builder pattern (AR Option 2) | Maximum abstraction; future-proof | Over-engineering for 3 simple structs; violates "no traits with single implementation" anti-pattern | Unnecessary complexity for data-carrying structs |
| Enum-based model with variants per pipeline stage (AR Option 3) | Type-safe stage transitions | Explosion of types; pattern matching everywhere; verbose data movement | Over-engineered; could reconsider much later if type safety proves necessary |

**References**:
- AR Section: Options Considered
- Constitution principles referenced: X (YAGNI), anti-pattern "Don't create traits with a single implementation"

---

## Decision 2: Pipeline Ownership Semantics

**Decision**: Functional transformation - each WI takes ownership and returns enriched instances

**Rationale**:
- Most idiomatic Rust for pipeline transformations: `assemble_document` returns owned `PolicyDocument`, WI-6 takes ownership and returns enriched version, etc.
- Follows functional programming principles; makes data flow explicit
- Avoids mutable aliasing concerns and borrow checker complexity
- `Option` fields naturally support this pattern

**Alternatives Considered**:

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| In-place mutation (`&mut PolicyDocument`) | Potentially less allocation | Complex borrow checker reasoning; unclear ownership; harder to test | Violates functional transformation best practices in Rust |
| Builder pattern | Progressive construction | Over-engineering; adds boilerplate with no clear benefit | YAGNI - assembly function is sufficient |

**References**:
- Clarification Q1 in spec.md
- Rust idioms: ownership-based transformations for pipelines

---

## Decision 3: Requirement Identity Before stable_id

**Decision**: (source_line, text_hash) tuple for temporary identity

**Rationale**:
- `source_line` alone insufficient for edge cases with duplicate line numbers (nested lists)
- Lightweight hash (first 64 chars + source_line) provides practical uniqueness without prematurely assigning UUIDs
- Enables testing and debugging ("requirement at line 42, hash abc...")
- Not persisted; purely for intermediate pipeline stages

**Alternatives Considered**:

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| source_line alone | Simplest | Fails for duplicate lines | Edge case: nested lists may have requirements at same line |
| Assign temporary UUIDs early, replace in WI-7 | Strong identity from start | Violates WI-7 scope boundary; adds unnecessary UUID generation | Premature - WI-7 explicitly owns UUID generation |
| No uniqueness guarantee before WI-7 | Zero overhead | Cannot reference requirements in tests or debugging | Testing and debugging require some form of identity |

**References**:
- Clarification Q2 in spec.md
- Assumption A-6 in spec.md

---

## Decision 4: Expected Document Scale

**Decision**: Medium (100-1000 requirements, 10-100 sections)

**Rationale**:
- Based on typical security policy documents:
  - NIST SP 800-53: ~1000 controls
  - ISO 27001: ~100 controls
  - Organizational policies: 50-500 requirements
- Vec-based structures (O(n) traversal) perform well for n < 1000
- No need for indexed data structures (HashMap, BTreeMap) or memory optimization
- Enables straightforward, readable implementation

**Implications**:
- Use `Vec<PolicySection>` and `Vec<PolicyRequirement>` without indexing
- Simple linear traversal for section-to-requirement association
- No memory pooling or allocation optimization needed

**References**:
- Clarification Q3 in spec.md
- Assumption A-7 in spec.md

---

## Decision 5: Error Handling Strategy

**Decision**: Unix-like approach - warnings to stderr via eprintln!, fatal errors return Err

**Rationale**:
- Simplest approach; follows Unix philosophy
- Recoverable issues (malformed YAML, missing frontmatter) emit warnings but return `Ok(PolicyDocument)` with fallback values
- Fatal errors (data inconsistency preventing assembly) return `Err(ForgeError)` and halt pipeline
- Keeps domain model code simple; pushes presentation concerns to CLI layer

**Alternatives Considered**:

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| Structured error enum with severity levels | Programmatic handling | Adds complexity for a local CLI tool | Over-engineering; no need for machine-readable error codes |
| Result<(PolicyDocument, Vec<Warning>), Error> | Warnings alongside success | Complicates return type; forces downstream consumers to unpack tuple | Unnecessary complexity; stderr is sufficient for warnings |
| Fatal errors only (no warnings) | Simplest | User loses visibility into recoverable issues | Poor UX; users should know when fallback values are used |

**References**:
- Clarification Q4 in spec.md
- Edge Case EC-4 in spec.md (malformed YAML warning)
- Assumption A-8 in spec.md

---

## Decision 6: Performance Expectations

**Decision**: No specific targets; prioritize correctness and readability

**Rationale**:
- Local CLI development tool processing medium-scale documents (100-1000 requirements)
- Sub-second assembly will naturally result from straightforward implementation
- Investing in optimization (parallel section mapping, indexed lookups) adds complexity without clear benefit
- Optimize only if profiling reveals actual bottlenecks in real usage

**Implications**:
- Use straightforward algorithms (linear traversal, simple mapping)
- No parallelization of section mapping
- No indexed requirement association (line-range-based heuristic is sufficient)
- Focus on readable, testable code

**References**:
- Clarification Q5 in spec.md
- Assumption A-9 in spec.md

---

## Decision 7: Frontmatter Parsing Approach

**Decision**: serde_yaml with fault-tolerant parsing

**Rationale**:
- `serde_yaml` is the de facto standard YAML library in Rust
- Well-maintained, widely used, resistant to common YAML attack vectors
- Fault-tolerant approach: malformed YAML causes warning + fallback to defaults, not error
- Fallback chain: frontmatter → first H1 heading → filename

**Alternatives Considered**:

| Option | Pros | Cons | Rejection Reason |
|--------|------|------|------------------|
| Manual YAML frontmatter parsing | No dependency | Fragile for edge cases; reinventing wheel | serde_yaml is standard; no need to reimplement |
| yaml-rust | Alternative YAML library | Less widely used than serde_yaml | serde_yaml is ecosystem standard |

**Security Considerations**:
- YAML bombs (recursive anchors/aliases): serde_yaml handles common attack vectors; policy frontmatter is small (typically 5-10 lines)
- SEC-005 suggests bounding frontmatter parsing region to first 4KB (low priority)

**References**:
- PRD Selected Approach
- AR Frontmatter Extraction pattern
- SEC-005 Finding F1 (optional: bound frontmatter to 4KB)

---

## Technology Stack Summary

| Component | Technology | Version | Justification |
|-----------|-----------|---------|---------------|
| Language | Rust | Edition 2024, stable 1.93.0 | Existing FORGE stack |
| Serialization | serde | 1.x | Standard Rust serialization; `#[derive(Debug, Clone)]` support |
| YAML Parsing | serde_yaml | Latest stable | De facto standard for YAML in Rust ecosystem |
| Error Handling | thiserror | 2.0.18 | Existing FORGE error handling library |
| Testing | cargo test | Built-in | Standard Rust testing framework; TDD mandatory |

---

## Open Questions

None. All architectural decisions resolved through clarification workflow and architecture review.

---

## Next Steps

Proceed to Phase 1: Data Model & Contracts design.
