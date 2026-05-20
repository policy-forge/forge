# Changelog

All notable changes to FORGE will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
