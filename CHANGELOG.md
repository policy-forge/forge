# Changelog

All notable changes to FORGE are documented in this file.

## [0.2.0] — 2026-02-19

### Phase 2 Features

#### WI-26 — XML Output
Added XML serialization for Catalog and Component Definition artifacts via `forge convert --format xml` and `forge export --format xml`.

#### WI-27 — YAML Output
Added YAML serialization for Catalog and Component Definition artifacts via `forge convert --format yaml` and `forge export --format yaml`.

#### WI-28 — Round-Trip Testing
Added bidirectional format conversion verification infrastructure (`forge::testing::assert_semantic_equivalence`) confirming JSON↔XML↔YAML round-trips preserve all semantic content.

#### WI-29 — `forge export` Subcommand
New `forge export <INPUT> --format <json|xml|yaml>` subcommand for converting existing OSCAL artifacts between formats. Auto-detects input format from file extension.

#### WI-30 — Profile Generation
New `forge profile --catalog <path> [--include <ids>] [--exclude <ids>]` subcommand that generates OSCAL v1.2.0 Profile JSON/XML/YAML from a source Catalog.

#### WI-31 — Profile Parameter Tailoring
Extended `forge profile` with `--set-param <id> <value>` flag (repeatable) that populates `modify.set-parameters` in the generated Profile.

#### WI-32 — Profile Schema Validation
`forge validate` now supports OSCAL Profile artifacts in addition to Catalog and Component Definition. Added golden-file snapshot tests for Profile schema conformance.

#### WI-33 — Normative/Advisory Detection
Pipeline enrichment pass that classifies each policy requirement as normative (must/shall/will/required) or advisory (should/may/recommended/optional) using RFC 2119 modal verb detection. Classification stored as `prop[name=modality]` on OSCAL controls.

#### WI-34 — Parameter Extraction
Pipeline enrichment pass that detects parameterized values in policy text (time windows, thresholds, frequencies, quantities) and emits OSCAL `param` elements on the corresponding controls.

#### WI-35 — Phase 2 Integration Testing
21 new integration tests across 4 files verifying end-to-end behavior of all Phase 2 features:
- `tests/integration_round_trip.rs` — multi-format round-trip semantic equivalence (US1)
- `tests/integration_profile_e2e.rs` — full Profile generation pipeline (US2)
- `tests/integration_regression.rs` — Phase 1 structural regression verification (US5)
- `tests/integration_cross_feature.rs` — normative/advisory and param cross-feature verification (US3, US4)

### Quality Gates (v0.2.0)
- `cargo test`: 1091 passed, 0 failed, 3 ignored
- `cargo clippy -- -D warnings`: 0 warnings
- `cargo fmt --check`: 0 violations
- `cargo deny check`: 0 violations

## [0.1.0] — Phase 1

Initial release covering WI-1 through WI-24: Markdown ingestion, OSCAL Catalog and Component Definition generation, control atomization, citation extraction, `forge convert`, `forge validate`, and foundational pipeline infrastructure.
