# FORGE Roadmap

> **Last Updated:** 2026-09-08
> **Current Release Line:** v1.1.0 released; post-v1.1.0 development active
> **Canonical Detail:** [docs/FORGE_PRODUCT_ROADMAP.md](docs/FORGE_PRODUCT_ROADMAP.md)

This roadmap reconciles the historical 50-work-item FORGE plan and the next-wave
PRDs against the current repository state. FORGE has completed the original
Phase 1, Phase 2, and Phase 3 scope, released v1.1.0, and is executing the
post-v1.1.0 product roadmap.

---

## Current Status

FORGE v1.0.0 is the community-ready release of the Markdown-to-OSCAL pipeline,
and v1.1.0 adds native PDF/DOCX ingestion. The original roadmap remains
complete: **50 of 50 work items are Done**.

The post-v1.1.0 roadmap contains **14 PRDs**:

- **7 technically implemented:** PRDs 055–060 and 063. Their implementation is
  merged, while their PRD-level human release gates remain pending where noted.
- **7 planned:** PRDs 061, 062, and 064–068. No implementation merge is present
  for these initiatives.
- **0 currently recorded as in progress** in this repository.

**What is built:**

- Markdown ingestion, structural extraction, clause extraction, domain model assembly, atomization, deterministic UUID v5 IDs, citation extraction, modality detection, and parameter extraction.
- Native PDF (`.pdf`) and DOCX (`.docx`) ingestion with heading/list style mapping, added in v1.1.0.
- OSCAL Catalog, Component Definition, Profile, Assessment Plan, and System Security Plan template generation.
- JSON, XML, and YAML output with format export and round-trip validation support.
- Schema validation, semantic validation, human-readable and JSON validation reports.
- Traceability reporting, diff reporting, summary dashboards, batch conversion, and NIST oscal-cli integration.
- Community examples, usage documentation, contributor documentation, architecture documentation, cross-platform CI, release workflow, checksums, and SLSA provenance.
- Human-reviewed control mappings, framework applicability and gap analysis,
  framework-change impact monitoring, policy lifecycle management, reusable
  policy components, evidence/implementation linkage, and OSCAL Assessment
  Results workflows.

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

## v1.1.0 Release Line (Current)

v1.1.0 (released 2026-06-09) extends the completed pipeline beyond Markdown-only input:

- Native PDF ingestion via `pdf-extract`.
- Native DOCX ingestion via `zip` + OOXML parsing, with Word heading/list styles mapped to the internal document model.
- SSP generation now derives control implementations from the generated Catalog instead of emitting empty placeholders.

Note: this reverses the 2026-02-10 "Markdown-only input" decision recorded in [docs/FORGE_PRODUCT_ROADMAP.md](docs/FORGE_PRODUCT_ROADMAP.md); see that document's decision log for rationale.

## Post-v1.1.0 Roadmap (PRDs 055–068)

"Technical implementation complete" means the implementation and remediation
changes are merged into `main`. It does not claim that human release approvals,
design-partner validation, publication, or adoption gates are complete.

| PRD | Initiative | Repository Status | Evidence / Next Gate |
|-----|------------|-------------------|----------------------|
| [055](docs/PRD/055-prd-control-mapping.md) | Human-reviewed OSCAL control mapping | Technical implementation complete | Merged in `81ad6f9`; human release gates remain |
| [056](docs/PRD/056-prd-framework-applicability-gap-analysis.md) | Framework applicability and gap analysis | Technical implementation complete | Merged in `ffc83b8`; human release gates remain |
| [057](docs/PRD/057-prd-framework-change-impact-monitoring.md) | Framework-change impact monitoring | Technical implementation complete | Merged in `989a684`; human release gates remain |
| [058](docs/PRD/058-prd-policy-lifecycle-management.md) | Policy lifecycle management | Technical implementation complete | Merged in `f8784d1`; remediation merged in `a254d16`; human release gates remain |
| [059](docs/PRD/059-prd-reusable-policy-components.md) | Reusable policy components | Technical implementation complete | Merged in `09f1f37`; PRD release gates remain |
| [060](docs/PRD/060-prd-evidence-implementation-linking.md) | Evidence and implementation linkage | Technical implementation complete | Merged in `a5d0aff`; PRD release gates remain |
| [061](docs/PRD/061-prd-framework-guided-policy-authoring.md) | Framework-guided policy authoring | Planned | Requires PRD 056 output; product, compliance, legal, and design-partner readiness gates remain |
| [062](docs/PRD/062-prd-local-web-workspace.md) | API-first local web workspace | In Progress — Slice 0 technically complete | Contract, `forge.workspace/1` schema, capability matrix, fixtures, threat model, service boundaries, ADRs, and CI drift checks delivered in PR #142; human API/security/accessibility review gates remain before Slice 1 |
| [063](docs/PRD/063-prd-oscal-assessment-results.md) | OSCAL Assessment Results | Technical implementation complete | Merged in `34d9869`; PRD release gates remain |
| [064](docs/PRD/064-prd-oscal-poam-workflow.md) | OSCAL POA&M workflow | Planned | PRD 063 dependency is implemented; schema, status semantics, and source-of-truth decisions remain |
| [065](docs/PRD/065-prd-external-workflow-integrations.md) | External workflow integrations | Planned | Requires a stable source-report contract and approved provider/auth boundary |
| [066](docs/PRD/066-prd-ai-assisted-suggestions.md) | AI-assisted suggestions | Planned | Requires stable PRD 055/061 schemas, an approved corpus, and privacy/security review |
| [067](docs/PRD/067-prd-read-only-mcp-governance-interface.md) | Read-only MCP governance interface | Planned | Requires stable project discovery, shared read-only queries, and a completed threat model |
| [068](docs/PRD/068-prd-collaborative-review-queues.md) | Collaborative review queues | Planned | Requires a selected first review workflow and approved identity/quorum boundaries |

### Now / Next / Later

This sequence communicates dependency order, not committed dates.

| Horizon | Initiative | Outcome / Gate |
|---------|------------|----------------|
| **Now** | PRD 062 Slice 0 | Establish the normative API contract, capability matrix, project index schema, threat model, service boundaries, fixtures, ADRs, and drift checks — artifacts and CI drift gate delivered; human review approval remains |
| **Now** | PRDs 055–060 and 063 release-gate reconciliation | Complete or explicitly defer the human, pilot, documentation, and release approvals still open in the PRDs |
| **Next** | PRD 062 Slices 1–4 | Deliver read-only exploration, safe mutation, applicability/mapping review, and a local pilot in gated increments |
| **Next** | PRD 064 Phase 1 | Add deterministic POA&M scaffolding and schedule reporting on the completed Assessment Results foundation |
| **Next** | PRD 061 Phase 1 | Add traceable drafting plans and skeletons after product, compliance, legal, and design-partner readiness decisions |
| **Later** | PRDs 065, 067, and 068 | Add external handoff, bounded read-only agent access, and asynchronous collaboration after their shared contracts stabilize |
| **Later** | PRD 066 | Add quarantined AI suggestions only after the authoring schema, evaluation corpus, and provider/privacy gates are approved |

## Historical v1.0.0 Release Gate

The completed v1.0.0 release gate consisted of:

- `Cargo.toml` version set to `1.0.0`.
- `CHANGELOG.md` containing the v1.0.0 release notes.
- Cross-platform CI matrix for Linux, macOS, and Windows.
- Release workflow producing platform binaries, checksums, and SLSA provenance.
- Community examples and documentation committed.
- Final quality checks passing in CI.

---

## Unscheduled Roadmap Candidates

PRDs 055–068 now own the previously listed Assessment Results, POA&M, control
mapping, integration, web/API, AI, and collaboration opportunities. Remaining
unscheduled candidates are:

- Built-in Profile Resolution engine instead of delegating to NIST oscal-cli.
- Broader bidirectional source ↔ OSCAL traceability views beyond the planned
  workspace and MCP contracts.
- Hosted documentation site such as mdBook or GitHub Pages.
- Full SSP generation from external system data sources beyond policy-derived
  templates.

---

## Strategic Goals

From the [Product Vision](docs/FORGE_PRODUCT_VISION.md):

| ID | Goal | Status |
|----|------|--------|
| G-1 | Reliable Markdown-to-OSCAL pipeline with schema validation and traceability | Complete |
| G-2 | Full OSCAL Control layer with Catalog, Profile, and multi-format output | Complete |
| G-3 | Standard open-source policy-to-OSCAL tool | v1.1.0 release line |
| G-4 | OSCAL Implementation layer with Component Definition and SSP templates | Complete |
