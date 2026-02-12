# FORGE Roadmap

> **Last Updated:** 2026-02-11

This roadmap translates the FORGE product vision into a sequenced execution plan. See [FORGE_PRODUCT_VISION.md](docs/FORGE_PRODUCT_VISION.md) for strategic goals and [FORGE_PRD.md](docs/FORGE_PRD.md) for detailed requirements.

---

## Completed Work Items

All foundation work (WI-1 through WI-7) is merged to `main`.

| WI | Title | AR | Status |
|----|-------|----|--------|
| WI-1 | Project scaffolding | [001](docs/AR/001-ar-project-scaffolding.md) | Done |
| WI-2 | Markdown ingestion | [002](docs/AR/002-ar-markdown-ingestion.md) | Done |
| WI-3 | Structural extraction — headings | [003](docs/AR/003-ar-structural-extraction-headings.md) | Done |
| WI-4 | Structural extraction — clauses | [004](docs/AR/004-ar-structural-extraction-clauses.md) | Done |
| WI-5 | Domain model | [005](docs/AR/005-ar-domain-model.md) | Done |
| WI-6 | Requirement atomization | [006](docs/AR/006-ar-requirement-atomization.md) | Done |
| WI-7 | UUID generation | [007](docs/AR/007-ar-uuid-generation.md) | Done |

**What's built:** A complete Markdown-to-domain-model pipeline that ingests policy documents, extracts structural hierarchy, parses clauses (lists, tables, paragraphs), assembles a `PolicyDocument` with YAML frontmatter metadata, atomizes compound requirements, and assigns deterministic UUID v5 stable identifiers. Output is JSON of the internal domain model.

---

## Phase 1 — Foundation (Target: 2026-08-01)

**Goal:** Users can convert Markdown policies to validated OSCAL Catalogs and Component Definitions (JSON) with full traceability.

**Exit Criteria:** All Must Have requirements (M-1 through M-11) passing; golden-file test suite >95% accuracy; `cargo test` green.

| WI | Title | AR | PRD Req | Status |
|----|-------|----|---------|--------|
| WI-8 | Citation extraction | [008](docs/AR/008-ar-citation-extraction.md) | M-9 | Not Started |
| WI-9 | Catalog — groups & controls | [009](docs/AR/009-ar-catalog-groups-controls.md) | M-3 | Not Started |
| WI-10 | Catalog — statement parts | [010](docs/AR/010-ar-catalog-statement-parts.md) | M-3 | Not Started |
| WI-11 | OSCAL metadata | [011](docs/AR/011-ar-oscal-metadata.md) | M-5 | Not Started |
| WI-12 | Back matter | [012](docs/AR/012-ar-back-matter.md) | M-9 | Not Started |
| WI-13 | Catalog pipeline | [013](docs/AR/013-ar-catalog-pipeline.md) | M-3 | Not Started |
| WI-14 | Component Definition — structure | [014](docs/AR/014-ar-component-definition-structure.md) | M-4 | Not Started |
| WI-15 | Component — implemented requirements | [015](docs/AR/015-ar-component-implemented-requirements.md) | M-4 | Not Started |
| WI-16 | Traceability model | [016](docs/AR/016-ar-traceability-model.md) | M-10 | Not Started |
| WI-17 | Traceability embedding | [017](docs/AR/017-ar-traceability-embedding.md) | M-10 | Not Started |
| WI-18 | Component pipeline | [018](docs/AR/018-ar-component-pipeline.md) | M-4 | Not Started |
| WI-19 | Schema validation | [019](docs/AR/019-ar-schema-validation.md) | M-6 | Not Started |
| WI-20 | Validation error reporting | [020](docs/AR/020-ar-validation-error-reporting.md) | M-6 | Not Started |
| WI-21 | Golden-file tests | [021](docs/AR/021-ar-golden-file-tests.md) | — | Not Started |
| WI-22 | Golden-file edge cases | [022](docs/AR/022-ar-golden-file-edge-cases.md) | — | Not Started |
| WI-23 | Error handling | [023](docs/AR/023-ar-error-handling.md) | — | Not Started |
| WI-24 | Performance benchmarks | [024](docs/AR/024-ar-performance-benchmark.md) | — | Not Started |
| WI-25 | Phase 1 release | [025](docs/AR/025-ar-phase1-release.md) | — | Not Started |

---

## Phase 2 — Control Layer & Multi-Format (Target: 2026-10-31)

**Goal:** Users can export to XML/YAML, generate Profiles for baseline selection and tailoring, with normative/advisory tagging and parameter extraction.

**Exit Criteria:** Multi-format round-trip verified; Profile generation with tailoring working; v0.2.0 tagged.

| WI | Title | AR | PRD Req | Status |
|----|-------|----|---------|--------|
| WI-26 | XML output | [026](docs/AR/026-ar-xml-output.md) | S-3 | Not Started |
| WI-27 | YAML output | [027](docs/AR/027-ar-yaml-output.md) | S-4 | Not Started |
| WI-28 | Round-trip testing | [028](docs/AR/028-ar-round-trip-testing.md) | — | Not Started |
| WI-29 | Export subcommand | [029](docs/AR/029-ar-export-subcommand.md) | S-3, S-4 | Not Started |
| WI-30 | Profile generation | [030](docs/AR/030-ar-profile-generation.md) | S-5 | Not Started |
| WI-31 | Profile parameter tailoring | [031](docs/AR/031-ar-profile-parameter-tailoring.md) | S-5 | Not Started |
| WI-32 | Profile validation tests | [032](docs/AR/032-ar-profile-validation-tests.md) | S-5 | Not Started |
| WI-33 | Normative/advisory detection | [033](docs/AR/033-ar-normative-advisory-detection.md) | S-7 | Not Started |
| WI-34 | Parameter extraction | [034](docs/AR/034-ar-parameter-extraction.md) | S-8 | Not Started |
| WI-35 | Phase 2 release | [035](docs/AR/035-ar-phase2-release.md) | — | Not Started |

---

## Phase 3 — Ecosystem (Target: 2027-04-01)

**Goal:** FORGE integrates with NIST oscal-cli for Profile Resolution, generates Assessment Plan scaffolding and SSP templates, and establishes community adoption.

**Exit Criteria:** oscal-cli integration tested; community examples published; 5+ organizations using FORGE.

| WI | Title | AR | PRD Req | Status |
|----|-------|----|---------|--------|
| WI-36 | oscal-cli profile resolution | [036](docs/AR/036-ar-oscal-cli-profile-resolution.md) | — | Not Started |
| WI-37 | oscal-cli round-trip | [037](docs/AR/037-ar-oscal-cli-round-trip.md) | — | Not Started |
| WI-38 | Traceability report | [038](docs/AR/038-ar-traceability-report.md) | S-6 | Not Started |
| WI-39 | Traceability report excerpts | [039](docs/AR/039-ar-traceability-report-excerpts.md) | S-6 | Not Started |
| WI-40 | Batch conversion | [040](docs/AR/040-ar-batch-conversion.md) | C-1 | Not Started |
| WI-41 | Assessment Plan — controls | [041](docs/AR/041-ar-assessment-plan-controls.md) | C-2 | Not Started |
| WI-42 | Assessment Plan — subjects | [042](docs/AR/042-ar-assessment-plan-subjects.md) | C-2 | Not Started |
| WI-43 | Diff report | [043](docs/AR/043-ar-diff-report.md) | C-3 | Not Started |
| WI-44 | Summary dashboard | [044](docs/AR/044-ar-summary-dashboard.md) | C-4 | Not Started |
| WI-45 | SSP template structure | [045](docs/AR/045-ar-ssp-template-structure.md) | — | Not Started |
| WI-46 | SSP template placeholders | [046](docs/AR/046-ar-ssp-template-placeholders.md) | — | Not Started |
| WI-47 | Community examples | [047](docs/AR/047-ar-community-examples.md) | — | Not Started |
| WI-48 | Community documentation | [048](docs/AR/048-ar-community-documentation.md) | — | Not Started |
| WI-49 | Cross-platform release | [049](docs/AR/049-ar-cross-platform-release.md) | — | Not Started |
| WI-50 | Phase 3 release | [050](docs/AR/050-ar-phase3-release.md) | — | Not Started |

---

## Strategic Goals

From the [Product Vision](docs/FORGE_PRODUCT_VISION.md):

| ID | Goal | Phase |
|----|------|-------|
| G-1 | Reliable Markdown-to-OSCAL pipeline with schema validation and traceability | Phase 1 |
| G-2 | Full OSCAL Control layer (Catalog + Profile) with multi-format output | Phase 2 |
| G-3 | Standard open-source policy-to-OSCAL tool | Phase 3 |
| G-4 | OSCAL Implementation layer (Component Definition + SSP templates) | Phase 1 + Phase 3 |
