# ADR-0001: OpenAPI Contract Ownership and Drift Enforcement

> **Document Type:** Architecture Decision Record
> **Audience:** LLM agents, human reviewers, engineering
> **Status:** Accepted (PRD-062 Slice 0, 2026-09-08)
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

Related: ADR-0002 (service and effect boundaries), ADR-0003 (embedded assets),
`docs/api/forge-workspace-v1.openapi.yaml`, `docs/api/capability-matrix.md`,
`docs/api/compatibility.md`, PRD 062 "Contract-First Artifacts", M-21, AC-19.

## Context

PRD 062 requires a committed, normative OpenAPI 3.1 description of the local
workspace API before any browser implementation, with CI that fails when
handlers, fixtures, clients, or the bundled UI drift from it (M-21, AC-19).
The PRD explicitly leaves ownership open: contract-authored with generated
checks, or code-authored with deterministic generation — but "in either case,
one artifact is normative and automated drift detection is mandatory; two
independently maintained schemas are prohibited" (Open Questions, blocking,
engineering).

Repository constraints that bound the choice:

- A single offline Rust crate. CI is `cargo test`, `cargo clippy`, and a
  release build on Linux, macOS, and Windows (`.github/workflows/ci.yml`),
  plus cargo-audit/deny/vet supply-chain jobs. There is no Node.js toolchain
  anywhere in CI or the release workflow (`.github/workflows/release.yml`
  builds with cargo, packages archives, and produces CycloneDX SBOM, SHA-256
  checksums, and SLSA provenance).
- Strict-JSON conventions already exist: bounded, duplicate-key-rejecting
  parsing in `src/json_strict.rs` (`parse_value`, `Limits`), vendored JSON
  schemas embedded via `include_str!` and compiled in `src/validate/mod.rs`
  (`load_schema`, `validate_artifact`), and deterministic serialization
  asserted by `insta` golden tests.
- The PRD's risk table flags "new frontend/toolchain harms reproducibility"
  as a launch-blocking risk; the same reasoning applies to the API toolchain.
- The contract must be reviewable by product and security *before*
  implementation exists, which favors a document that is authored and
  reviewed on its own rather than derived from handler code.

## Decision

The API contract is **contract-authored**. The single normative artifact is
`docs/api/forge-workspace-v1.openapi.yaml`. No second schema source is
maintained anywhere; handlers, fixtures, clients, and the UI are checked
against this document, never against a re-derived copy.

Drift enforcement is deterministic and offline, implemented as a Rust
integration test run by the existing `cargo test` CI job (no new toolchain,
no network access, identical behavior on all three CI operating systems).
From Slice 0 onward the test performs:

1. **Document validation.** Parse the committed YAML (the crate already
   depends on `serde_yaml_ng` for `src/export/yaml.rs`), convert to JSON, and
   validate structurally against a vendored OpenAPI 3.1 meta-schema using the
   same schema-compilation machinery as `src/validate/mod.rs`.
2. **Fixture validation.** Every fixture registered in
   `docs/api/fixtures/index.json` is validated against the component schemas
   resolved live from the document itself — the document is the only schema
   source; fixtures cannot pass against a stale copy.
3. **Coverage enforcement.** Two-directional capability-matrix checks against
   `docs/api/capability-matrix.md`: every documented operation appears in a
   matrix row, and every matrix row names at least one operation that exists
   in the document. A browser feature without a matrix entry cannot enter
   implementation (PRD "Contract-First Artifacts").

Drift enforcement phases:

- **Slice 0 (this ADR):** contract + fixture + matrix validation as above.
  This is the PRD Slice 0 exit evidence ("schema/fixture validation and drift
  checks execute before UI code exists").
- **Slice 1+:** handler conformance. HTTP handlers must satisfy every fixture
  (request and response) and mismatches fail CI (AC-19). Typed-client checks
  are hand-mapped Rust structs in the headless conformance client initially;
  generating them from the document remains a deferred, separately reviewed
  option (see Alternatives).

Rationale: keeping validation inside `cargo test` preserves the project's
offline, deterministic property bit-for-bit; the committed document stays the
single reviewable normative artifact; and no build-time codegen step can
introduce tool or platform nondeterminism into the release path.

## Consequences

- Positive: product/security review of the contract precedes and is
  independent of implementation; a single normative schema; CI needs no new
  toolchain; contract validation runs on all three CI platforms unchanged;
  the document ships with source and release artifacts as the PRD requires.
- Positive: fixture tests double as executable examples for local client
  developers (M-25 support promise).
- Negative: hand-authored YAML must be kept internally consistent
  (`$ref` resolution, discriminator use, example correctness) — mitigated by
  check 1 catching structural drift and check 2 catching semantic drift.
- Negative: typed Rust request/response structs are written by hand and kept
  aligned by conformance tests rather than by a generator — an accepted,
  bounded cost for the MVP-sized surface, revisited only if the surface grows.
- Constraint: schema changes now require a contract commit that passes
  validation before handler code can merge, which is exactly the intended
  PRD gate.

## Alternatives considered

- **Code-authored contract via `utoipa` derives (generate the document from
  handler types).** Rejected for PRD 062. The PRD requires the contract to be
  reviewed and approved *before* browser/handler implementation exists; a
  derived document cannot lead implementation. It also couples the contract's
  shape to Rust type ergonomics rather than product vocabulary, and fixture
  conformance tests would still be needed, so it removes less work than it
  appears to. Reconsider only if a future slice shows double-entry pain that
  outweighs contract-first review.
- **Client generation via `progenitor` from the committed document.** Deferred,
  not rejected. It fits the contract-authored direction, but adds a generated
  crate and a generator dependency to an offline build for a client surface
  small enough to hand-map. The hand-mapped structs remain deliberately
  aligned by the Slice 1 fixture tests; switching to generation later does
  not change the normative artifact.
- **OpenAPI Generator (Node/JVM toolchain).** Rejected. Requires installing
  and pinning a non-Rust toolchain and network-fetched artifacts in CI,
  directly conflicting with the offline deterministic build the PRD risk
  table calls out and with the existing cargo-only CI.
- **Two artifacts (authored spec plus generated server types kept in sync by
  review discipline).** Prohibited by the PRD ("two independently maintained
  schemas are prohibited"); not pursued.
