# FORGE Roadmap

> **Last Updated:** 2026-05-21
> **Current Release Line:** v1.0.0
> **Canonical Detail:** [docs/FORGE_PRODUCT_ROADMAP.md](docs/FORGE_PRODUCT_ROADMAP.md)

This roadmap reconciles the historical 50-work-item FORGE plan against the current repository state. FORGE has completed the original Phase 1, Phase 2, and Phase 3 scope and is now in the v1.0.0 release line.

---

## Current Status

FORGE v1.0.0 is the community-ready release of the Markdown-to-OSCAL pipeline. The original roadmap is complete: **50 of 50 work items are Done**.

**What is built:**

- Markdown ingestion, structural extraction, clause extraction, domain model assembly, atomization, deterministic UUID v5 IDs, citation extraction, modality detection, and parameter extraction.
- OSCAL Catalog, Component Definition, Profile, Assessment Plan, and System Security Plan template generation.
- JSON, XML, and YAML output with format export and round-trip validation support.
- Schema validation, semantic validation, human-readable and JSON validation reports.
- Traceability reporting, diff reporting, summary dashboards, batch conversion, and NIST oscal-cli integration.
- Community examples, usage documentation, contributor documentation, architecture documentation, cross-platform CI, release workflow, checksums, and SLSA provenance.

---

## Completed Work Items

### Phase 1 — Foundation — Complete

**Goal:** Users can convert Markdown policies to validated OSCAL Catalogs and Component Definitions with full traceability.

| WI | Title | AR | Status |
|----|-------|----|--------|
| WI-1 | Project scaffolding | [001](docs/AR/001-ar-project-scaffolding.md) | Done |
| WI-2 | Markdown ingestion | [002](docs/AR/002-ar-markdown-ingestion.md) | Done |
| WI-3 | Structural extraction — headings | [003](docs/AR/003-ar-structural-extraction-headings.md) | Done |
| WI-4 | Structural extraction — clauses | [004](docs/AR/004-ar-structural-extraction-clauses.md) | Done |
| WI-5 | Domain model | [005](docs/AR/005-ar-domain-model.md) | Done |
| WI-6 | Requirement atomization | [006](docs/AR/006-ar-requirement-atomization.md) | Done |
| WI-7 | UUID generation | [007](docs/AR/007-ar-uuid-generation.md) | Done |
| WI-8 | Citation extraction | [008](docs/AR/008-ar-citation-extraction.md) | Done |
| WI-9 | Catalog — groups & controls | [009](docs/AR/009-ar-catalog-groups-controls.md) | Done |
| WI-10 | Catalog — statement parts | [010](docs/AR/010-ar-catalog-statement-parts.md) | Done |
| WI-11 | OSCAL metadata | [011](docs/AR/011-ar-oscal-metadata.md) | Done |
| WI-12 | Back matter | [012](docs/AR/012-ar-back-matter.md) | Done |
| WI-13 | Catalog pipeline | [013](docs/AR/013-ar-catalog-pipeline.md) | Done |
| WI-14 | Component Definition — structure | [014](docs/AR/014-ar-component-definition-structure.md) | Done |
| WI-15 | Component — implemented requirements | [015](docs/AR/015-ar-component-implemented-requirements.md) | Done |
| WI-16 | Traceability model | [016](docs/AR/016-ar-traceability-model.md) | Done |
| WI-17 | Traceability embedding | [017](docs/AR/017-ar-traceability-embedding.md) | Done |
| WI-18 | Component pipeline | [018](docs/AR/018-ar-component-pipeline.md) | Done |
| WI-19 | Schema validation | [019](docs/AR/019-ar-schema-validation.md) | Done |
| WI-20 | Validation error reporting | [020](docs/AR/020-ar-validation-error-reporting.md) | Done |
| WI-21 | Golden-file tests | [021](docs/AR/021-ar-golden-file-tests.md) | Done |
| WI-22 | Golden-file edge cases | [022](docs/AR/022-ar-golden-file-edge-cases.md) | Done |
| WI-23 | Error handling | [023](docs/AR/023-ar-error-handling.md) | Done |
| WI-24 | Performance benchmarks | [024](docs/AR/024-ar-performance-benchmark.md) | Done |
| WI-25 | Phase 1 release | [025](docs/AR/025-ar-phase1-release.md) | Done |

### Phase 2 — Control Layer & Multi-Format — Complete

**Goal:** Users can export to XML/YAML, generate Profiles for baseline selection and tailoring, and extract machine-enforceable control metadata.

| WI | Title | AR | Status |
|----|-------|----|--------|
| WI-26 | XML output | [026](docs/AR/026-ar-xml-output.md) | Done |
| WI-27 | YAML output | [027](docs/AR/027-ar-yaml-output.md) | Done |
| WI-28 | Round-trip testing | [028](docs/AR/028-ar-round-trip-testing.md) | Done |
| WI-29 | Export subcommand | [029](docs/AR/029-ar-export-subcommand.md) | Done |
| WI-30 | Profile generation | [030](docs/AR/030-ar-profile-generation.md) | Done |
| WI-31 | Profile parameter tailoring | [031](docs/AR/031-ar-profile-parameter-tailoring.md) | Done |
| WI-32 | Profile validation tests | [032](docs/AR/032-ar-profile-validation-tests.md) | Done |
| WI-33 | Normative/advisory detection | [033](docs/AR/033-ar-normative-advisory-detection.md) | Done |
| WI-34 | Parameter extraction | [034](docs/AR/034-ar-parameter-extraction.md) | Done |
| WI-35 | Phase 2 release | [035](docs/AR/035-ar-phase2-release.md) | Done |

### Phase 3 — Ecosystem — Complete

**Goal:** FORGE integrates with ecosystem tooling, generates Assessment Plan scaffolding and SSP templates, and is ready for community adoption.

| WI | Title | AR | Status |
|----|-------|----|--------|
| WI-36 | oscal-cli profile resolution | [036](docs/AR/036-ar-oscal-cli-profile-resolution.md) | Done |
| WI-37 | oscal-cli round-trip | [037](docs/AR/037-ar-oscal-cli-round-trip.md) | Done |
| WI-38 | Traceability report | [038](docs/AR/038-ar-traceability-report.md) | Done |
| WI-39 | Traceability report excerpts | [039](docs/AR/039-ar-traceability-report-excerpts.md) | Done |
| WI-40 | Batch conversion | [040](docs/AR/040-ar-batch-conversion.md) | Done |
| WI-41 | Assessment Plan — controls | [041](docs/AR/041-ar-assessment-plan-controls.md) | Done |
| WI-42 | Assessment Plan — subjects | [042](docs/AR/042-ar-assessment-plan-subjects.md) | Done |
| WI-43 | Diff report | [043](docs/AR/043-ar-diff-report.md) | Done |
| WI-44 | Summary dashboard | [044](docs/AR/044-ar-summary-dashboard.md) | Done |
| WI-45 | SSP template structure | [045](docs/AR/045-ar-ssp-template-structure.md) | Done |
| WI-46 | SSP template placeholders | [046](docs/AR/046-ar-ssp-template-placeholders.md) | Done |
| WI-47 | Community examples | [047](docs/AR/047-ar-community-examples.md) | Done |
| WI-48 | Community documentation | [048](docs/AR/048-ar-community-documentation.md) | Done |
| WI-49 | Cross-platform release | [049](docs/AR/049-ar-cross-platform-release.md) | Done |
| WI-50 | Phase 3 release | [050](docs/AR/050-ar-phase3-release.md) | Done |

---

## v1.0.0 Release Gate

The v1.0.0 release line is the reconciled release target. The release gate consists of:

- `Cargo.toml` version set to `1.0.0`.
- `CHANGELOG.md` containing the v1.0.0 release notes.
- Cross-platform CI matrix for Linux, macOS, and Windows.
- Release workflow producing platform binaries, checksums, and SLSA provenance.
- Community examples and documentation committed.
- Final quality checks passing in CI.

---

## Future Roadmap Candidates

The original 50-item roadmap is complete. Future planning should start a new roadmap or v1.x section rather than reopening Phase 1–3 work.

Candidates already deferred in the PRD and architecture docs:

- OSCAL Assessment Results / SAR generation.
- OSCAL POA&M generation.
- Built-in Profile Resolution engine instead of delegating to NIST oscal-cli.
- OSCAL Control Mapping support for policy-to-framework crosswalks.
- External GRC, ticketing, and CI/CD integrations.
- Web UI or API/server mode.
- AI/ML semantic policy understanding beyond structural/syntactic parsing.
- Bidirectional source ↔ OSCAL traceability views.
- HTML or interactive reports.
- Hosted documentation site such as mdBook or GitHub Pages.
- Full SSP generation from external system data sources beyond policy-derived templates.

---

## Strategic Goals

From the [Product Vision](docs/FORGE_PRODUCT_VISION.md):

| ID | Goal | Status |
|----|------|--------|
| G-1 | Reliable Markdown-to-OSCAL pipeline with schema validation and traceability | Complete |
| G-2 | Full OSCAL Control layer with Catalog, Profile, and multi-format output | Complete |
| G-3 | Standard open-source policy-to-OSCAL tool | v1.0.0 release line |
| G-4 | OSCAL Implementation layer with Component Definition and SSP templates | Complete |
