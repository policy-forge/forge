# Changelog

All notable changes to FORGE will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

See the [authoring guide](docs/authoring.md) for the plan/build and `/1`
contracts merged in PR #144. This section records the current authoring tranche; it is not a complete release
inventory of all post-v1.1.0 changes.

### Added

- PRD-061 S-1: `forge author scaffold --manifest <FILE>` writes an empty
  `forge.authoring-pack/1` template bound to the project's exact framework
  inventory and baseline. It creates no assignments and leaves the required
  reviewer provenance fields empty, so a reviewer must complete the pack before
  `plan`/`build` accepts it. The new public `AuthorCommand` variant feeds the
  pending public Rust API/semver and migration gate.
- PRD-061 Phase 2: `forge author impact` for M-13 baseline/dependency comparison,
  `forge author handoff` for opt-in draft-only lifecycle records, and
  `--components FILE` / `--html` on author plan/build. Explicit component bindings
  can substitute supplied public/internal answer or literal values into draft
  Markdown; reports and HTML omit raw values. New closed input contracts
  are `forge.author-components/1`, `forge.authoring-impact/1` and
  `forge.author-handoff/1`; component output uses `forge.authoring-plan/2` and
  `forge.authoring-provenance/2`. The [Phase 2 contract table](docs/authoring-phase2.md)
  also documents the impact report, component lock and handoff receipt outputs.
  Static HTML is offline and value-redacted.
  Phase 2 outputs reuse Phase 1's publication boundary: one no-replace directory
  rename on Linux/macOS, failing closed on other platforms. Existing Phase 1
  inputs and default artifacts remain unchanged. Public Rust API migration/semver review and human acceptance,
  legal/content, design-partner, pilot and release gates remain pending.
- PRD-062 draft: a loopback-only `forge workspace` command serves an embedded
  offline review UI over the closed OpenAPI 1.1.0 contract, with explicit
  single-file effects. This is a draft technical implementation; security,
  accessibility, independent-review, supply-chain and release gates remain open.
- PRD-066 Phase 0 (merged, PR #153, merge `7d0d9f3`): `forge author reuse --manifest <FILE> --corpus <FILE>`
  ranks verbatim, span-exact excerpts of operator-supplied `approved` Markdown
  against the unresolved sections of a drafting plan. Retrieval only — no model,
  network, credentials or new dependency — and no write path into a pack,
  project or plan. New closed contracts are `forge.reuse-corpus/1` (input) and
  `forge.authoring-reuse/1` (report: text, JSON, and opt-in inert HTML), with a
  deterministic lexical score, exact spans and source hashes. Usefulness,
  rubric, threat-model, privacy/legal and release gates remain open.

### Changed

- The next release carrying the PRD-061 authoring tranches is a **major**
  (`2.0.0`) release: `AuthorCommand` and `ForgeError` are public and not
  `#[non_exhaustive]`, so the added variants can break downstream exhaustive
  matches. See the [authoring library API migration](docs/authoring-api-migration.md).
  Package version and `Cargo.lock` are unchanged until that release.
- Shared PRD-059 rendering reports effective byte/span limits in budget errors;
  exact error wording changes while existing successful composition bytes and
  provenance semantics remain preserved.
- `AuthorCommand` gains variants and fields. Downstream constructors and
  exhaustive matches may require changes. The API gate carried from Phase 1
  (including `ForgeError`) and the migration/semver release review
  remain pending. No release compatibility is claimed.

## [1.1.0] — 2026-06-09

FORGE v1.1.0 adds native PDF and DOCX ingestion, removing the requirement to pre-convert policy documents to Markdown with external tools.

### Added

- **PDF ingestion** — `forge convert` now accepts `.pdf` input directly (via `pdf-extract`) in addition to Markdown.
- **DOCX ingestion** — `forge convert` now accepts `.docx` input directly; Word heading styles are mapped to Markdown headings and list styles to list items before pipeline processing.
- **Policy-derived SSP control implementations** — `forge convert --to ssp` now builds the SSP skeleton from the generated Catalog so control-implementation entries are derived from the source policy rather than empty placeholders.
- **`--to ssp` output-type guard** — explicit error when SSP conversion is requested with a non-JSON output format.

### Changed

- Strategy selection for batch and single-file conversion now respects the `--to` output type consistently (`catalog`, `component`, `ssp`).

### Dependencies

- Added `pdf-extract`, `zip`; bumped `clap`, `pulldown-cmark`, `tracing-subscriber`, `rand`, `quick-xml`, `jsonschema`.

## [1.0.0] — 2026-05-18

FORGE v1.0.0 marks the completion of the Markdown-to-OSCAL pipeline. This release represents the journey from a proof-of-concept v0.1.0 through a production-ready tool that converts Markdown security policy documents into validated OSCAL artifacts across all major model types.

### Added

#### Complete OSCAL Model Support
- **OSCAL Catalog** generation from Markdown policy documents with full control hierarchies, groups, statement parts, and back-matter resources
- **OSCAL Component Definition** generation with documentary components and implemented requirements mapped from policy controls
- **OSCAL Profile** generation for control selection from source catalogs
- **OSCAL Assessment Plan** generation with reviewed-controls and assessment-subjects derived from component definitions
- **OSCAL System Security Plan (SSP)** template generation with system characteristics, control implementation skeleton, inventory items, users, metadata, and back-matter placeholders

#### Markdown-to-OSCAL Pipeline
- Full 7-stage pipeline: ingest → parse → assemble → atomize → UUID assignment → citation extraction → modality detection → parameter extraction
- Event-based Markdown parsing using `pulldown-cmark` with stack-based O(n) heading tree construction
- Compound requirement atomization — splits "must X and must Y" into atomic controls via regex-based conjunction detection
- Deterministic UUID v5 assignment (content-addressed, stable across re-conversions)
- RFC 2119 verb detection for normative (must/shall) vs advisory (should/may) modality classification
- Parameter extraction for configurable values (time windows, thresholds, frequencies, quantities)
- URL and bibliographic citation extraction from requirement prose

#### CLI — 7 Subcommands
- `forge convert` — Markdown to OSCAL conversion (single file and batch mode with `--jobs` parallelism via rayon)
- `forge export` — Format conversion between JSON, XML, and YAML
- `forge validate` — Schema validation against embedded NIST OSCAL v1.2.0 JSON schemas, with optional `--round-trip` fidelity check
- `forge resolve` — OSCAL Profile resolution into flat Catalog via NIST oscal-cli integration
- `forge profile` — Generate OSCAL Profile by selecting controls from a source Catalog
- `forge diff` — Semantic diff between two OSCAL artifacts
- `forge trace` — Source-to-OSCAL traceability reporting with provenance links

#### Validation & Reliability
- Embedded NIST OSCAL v1.2.0 JSON schemas for offline validation (no network required)
- Semantic validation checks (orphaned links, missing references)
- Round-trip validation chain: JSON → XML → YAML → JSON with semantic equality comparison
- Human-readable and JSON-formatted validation error reports
- `ForgeError` error taxonomy with categorized exit codes (0–5)
- 1,450+ test suite covering unit, integration, golden-file (insta snapshots), property-based (proptest), and benchmark (criterion) tests

#### Cross-Platform Support
- CI test matrix across ubuntu-latest, macos-latest, and windows-latest
- Pre-built binary releases for all 4 platforms (Linux, macOS, Windows, plus additional target)
- SLSA provenance for release artifacts
- Platform-specific binary naming (`.exe` on Windows)
- Installation via `cargo install forge` or pre-built binary download from GitHub Releases

#### Developer Experience
- Comprehensive architecture documentation (`docs/architecture.md`) covering pipeline stages, crate structure, and data flow
- Usage guide (`docs/usage-guide.md`) with end-to-end walkthroughs for all 7 CLI subcommands
- Community examples (`examples/`) with annotated sample policies demonstrating the full pipeline
- CONTRIBUTING.md with dev setup, spec-driven workflow, test conventions, and PR process
- Full API documentation via `cargo doc`
- Structured logging via `tracing` with `env-filter` support

### Changed

- Upgraded from Rust edition 2021 to 2024
- Migrated from minimal dependency set to full production dependencies: clap (CLI), pulldown-cmark (Markdown parsing), quick-xml (XML serialization), serde_yaml_ng (YAML), jsonschema (validation), rayon (parallelism), chrono (timestamps), url (citation parsing), tempfile (test fixtures)
- Replaced ad-hoc serialization with comprehensive serde-based OSCAL data models
- Stabilized model output — deterministic UUID v5 ensures identical inputs produce identical outputs across runs

### Fixed

- Platform-specific test failures (path separators, line endings, `#[cfg]` gates) across macOS and Windows
- Assessment subjects generation from component definitions (WI-42)
- SSP system placeholder population — inventory items, users, metadata, back-matter, leveraged-authorizations (WI-45/46)

### Removed

- Empty/stub OSCAL model implementations replaced with fully generated and validated outputs

---

## [0.1.0] — Initial Release

- Proof-of-concept Markdown-to-OSCAL conversion
- Basic Catalog and Component Definition generation
- Core parsing and atomization pipeline
- Initial test suite
