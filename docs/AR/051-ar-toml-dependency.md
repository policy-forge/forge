# 051-ar-toml-dependency

> **Document Type:** Architecture Review / Decision Record
> **Audience:** LLM agents, human reviewers
> **Status:** Accepted
> **Last Updated:** 2026-08-23 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->
> **Deciders:** Brian Luby <!-- @human-required -->

---

## Linkage ⚪ `@auto`

| Document | ID | Relationship |
|----------|-----|--------------|
| Parent PRD | [051-prd-project-configuration](../PRD/051-prd-project-configuration.md) | Requirements this decision supports |
| Review Thread | [PR #116 — Cargo.toml](https://github.com/policy-forge/forge/pull/116#discussion_r3837390792) | Finding that motivated this record |
| Supersedes | — | N/A |
| Superseded By | — | |

---

## Summary

### Decision 🔴 `@human-required`
> Approve the direct dependency on the `toml` crate (`toml = { version = "0.9", default-features = false, features = ["parse", "serde"] }`) in `Cargo.toml` as the TOML parsing layer for `.forge.toml` project configuration files.

### TL;DR for Agents 🟡 `@human-review`
> FORGE parses checked-in `.forge.toml` files using the `toml` crate v0.9+ with minimal feature surface (`parse`, `serde`; no defaults). This is an explicitly approved new production dependency. Do NOT hand-roll a TOML parser, do NOT enable additional features (`display`, default features), and do NOT add other TOML crates. Any change to this dependency must go through `cargo vet` and an updated ADR.

---

## Context

### Problem Space 🔴 `@human-required`
PRD 051 introduces repository-level configuration via `.forge.toml`. This requires reading and deserializing TOML documents into typed Rust structs with strict schema validation, informative key-suggestion errors, and deterministic behavior across platforms. No crate in FORGE's existing approved production dependency set can parse TOML, so the feature cannot ship without either adding a dependency or writing a parser.

### Driving Requirements 🟡 `@human-review`

| Req ID | Requirement Summary | Architectural Implication |
|--------|---------------------|---------------------------|
| M-1 | Parse `.forge.toml` into typed config structs | Requires a spec-compliant TOML parser |
| M-2 | Strict unknown-key rejection with key suggestions | Deserialization must surface raw keys before type conversion |
| S-* | Supply-chain integrity maintained | New dependency must pass `cargo vet --locked` with minimal exemption surface |

---

## Decision Drivers 🔴 `@human-required`

1. **Correctness:** TOML has a formal specification; hand-rolled parsers risk subtle divergence *(traces to M-1)*
2. **Supply-chain minimalism:** Minimize dependency count, feature surface, and transitive graph growth
3. **Maintainability:** Use a well-maintained, spec-compliant crate rather than owning parser code
4. **Vetting compliance:** All new crates must be registered in `supply-chain/config.toml`

---

## Options Considered 🟡 `@human-review`

### Option 0: Status Quo / No TOML Support
**Description:** Ship project configuration in JSON or environment variables only.

| Driver | Rating | Notes |
|--------|--------|-------|
| Correctness | N/A | Feature not delivered |
| Supply-chain minimalism | ✅ Good | No new crates |
| Maintainability | ⚠️ Medium | Pushes config burden onto users |
| Vetting compliance | ✅ Good | Nothing to vet |

**Why not viable:** PRD 051 mandates `.forge.toml` as the deliverable; reviewable, comment-friendly config requires TOML.

---

### Option 1: Adopt `toml` Crate with Minimal Features (Selected) ✅

**Description:** Add `toml = { version = "0.9", default-features = false, features = ["parse", "serde"] }`.

| Driver | Rating | Notes |
|--------|--------|-------|
| Correctness | ✅ Good | Spec-compliant parser; serde integration gives typed structs + rich error spans |
| Supply-chain minimalism | ⚠️ Medium | New crate + transitive stack (`toml_parser`, `toml_datetime`, `serde_spanned`, `winnow`), mitigated by minimal features and cargo-vet registration |
| Maintainability | ✅ Good | Actively maintained; de-facto standard TOML crate in the Rust ecosystem |
| Vetting compliance | ✅ Good | Exemptions registered; `cargo vet --locked` passes |

**Pros:** Standard ecosystem choice; minimal feature surface; strong error messages.
**Cons:** Adds a transitive dependency stack requiring temporary exemptions pending upstream audits.

---

### Option 2: Hand-Rolled TOML Parser
**Description:** Implement TOML parsing within `src/config.rs` using only stdlib + `serde_json::Value`-style intermediate representation.

| Driver | Rating | Notes |
|--------|--------|-------|
| Correctness | ❌ Poor | High risk of spec divergence (dates, inline tables, multi-line strings, escapes) |
| Supply-chain minimalism | ✅ Good | Zero new crates |
| Maintainability | ❌ Poor | FORGE would own a full language parser indefinitely |
| Vetting compliance | ✅ Good | Nothing to vet |

**Why not viable:** Owning a parser is strictly worse from both correctness and long-term supply-chain perspectives than vetting a maintained, widely-audited crate.

---

## Decision

### Selected Option 🔴 `@human-required`
> **Option 1: Adopt `toml` v0.9 with `default-features = false, features = ["parse", "serde"]`**

### Rationale 🔴 `@human-required`
TOML parsing is unavoidable for PRD 051's core deliverable. Option 0 abandons the requirement; Option 2 trades a small, auditable, temporary exemption set for permanent ownership of high-risk parser code. `toml` is the ecosystem-standard crate, and the restricted feature set keeps the added graph minimal (`parse` + `serde` only; `display` and default `std` extras excluded). All introduced crates are registered in `supply-chain/config.toml`; `cargo vet --locked` passes. This ADR constitutes the explicit human approval required for the new production dependency raised in PR #116 review.

### Constraints Added by this Decision 🟡 `@human-review`
- Only the `parse` and `serde` features may be enabled; do not add `display` or re-enable defaults without revisiting this ADR.
- No other TOML-parsing crates may be added while this decision stands.
- Every crate pulled in by `toml` must remain covered in `supply-chain/config.toml`; convert exemptions to imported audits as they become available upstream.
- Config parsing must consume `toml::Value` without redundant clones and reject unknown keys before typed conversion (see `src/config.rs`).

---

## Consequences 🟡 `@human-review`

### Positive
- Spec-compliant TOML parsing with high-quality spanned error messages
- Typed deserialization via serde integrates directly with strict schema validation
- Minimal, reviewable dependency delta with vetting coverage

### Negative
- Temporary cargo-vet exemptions (e.g., two `winnow` versions serving distinct dependents) until upstream audits are importable

### Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Transitive crate publishes a vulnerable release | Low | Med | Dependabot + `cargo vet suggest` during regular dep refreshes |
| Feature creep inflates dependency graph | Low | Med | Guardrail: features locked by this ADR |

---

## Changelog ⚪ `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-08-23 | Brian Luby | Accepted: approve `toml` dependency per PR #116 review finding |

---

## Decision Record ⚪ `@auto`

| Date | Event | Details |
|------|-------|---------|
| 2026-08-23 | Accepted | Explicit approval of `toml = "0.9"` (features: parse, serde), resolving [PR #116 discussion r3837390792](https://github.com/policy-forge/forge/pull/116#discussion_r3837390792) as won't-fix-by-requester with documented approval |
