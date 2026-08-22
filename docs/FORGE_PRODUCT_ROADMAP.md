# 001-roadmap-forge

> **Document Type:** Product Roadmap
> **Audience:** LLM agents, human reviewers, leadership stakeholders, engineering leads
> **Status:** Complete / v1.1.0 Release Line
> **Last Updated:** 2026-08-22 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->
> **Parent Vision:** docs/FORGE_PRODUCT_VISION.md <!-- @auto -->

---

## Reconciliation Summary

This document is the canonical reconciled roadmap for FORGE. Earlier roadmap snapshots treated Phase 2 as current and Phase 3 as partially incomplete. The repository has since moved ahead: the original 50-work-item roadmap is complete and the release line is **v1.1.0**.

**Current state:**

- 50 of 50 roadmap work items are Done.
- Phase 1 Foundation is complete.
- Phase 2 Control Layer & Multi-Format is complete.
- Phase 3 Ecosystem & Community is complete.
- v1.0.0 is the target/current release line for community-ready distribution.
- v1.1.0 (2026-06-09) added native PDF and DOCX ingestion and policy-derived SSP control implementations.

This reconciliation supersedes stale counts such as “43/50 done,” “44/50 done,” “8/15 Phase 3 done,” and “7 remaining.”

---

## Roadmap Context

FORGE translates Markdown security policy documents into deterministic, schema-validated OSCAL artifacts. The completed roadmap takes the project from scaffolding through community-ready release:

1. Markdown ingestion and policy parsing.
2. Internal domain modeling and deterministic identifiers.
3. OSCAL Catalog and Component Definition generation.
4. Validation, golden-file coverage, error handling, and performance checks.
5. XML/YAML export and round-trip validation.
6. Profile generation, tailoring, modality tagging, and parameter extraction.
7. oscal-cli integration, traceability reports, diff reports, batch conversion, summary dashboards, Assessment Plan scaffolding, SSP templates, community examples, documentation, CI, and release automation.

---

## Strategic Alignment

| Vision Goal | Mapped Themes | Coverage Status |
|-------------|---------------|-----------------|
| G-1: Markdown-to-OSCAL Pipeline | T-1, T-2, T-3 | Complete |
| G-2: Full Control Layer + Multi-Format | T-2, T-4, T-5 | Complete |
| G-3: Community Adoption | T-6 | v1.1.0 release line |
| G-4: Implementation Layer | T-6 | Complete |

---

## Themes

| ID | Theme | Description | Strategic Goal(s) | Owner | Status |
|----|-------|-------------|-------------------|-------|--------|
| T-1 | Core Pipeline | Ingestion, parsing, atomization, and internal domain model | G-1 | Brian Luby | Complete |
| T-2 | OSCAL Model Generation | Catalog and Component Definition artifacts with metadata, back matter, and traceability | G-1, G-2 | Brian Luby | Complete |
| T-3 | Validation & Quality | Schema validation, golden-file testing, error handling, and performance benchmarking | G-1 | Brian Luby | Complete |
| T-4 | Output Format Expansion | XML/YAML output, round-trip verification, and format conversion | G-2 | Brian Luby | Complete |
| T-5 | Profile & Tailoring | Profile generation, parameter setting, and normative/advisory tagging | G-2 | Brian Luby | Complete |
| T-6 | Ecosystem & Community | oscal-cli integration, Assessment Plan scaffolding, SSP templates, batch conversion, community docs | G-3, G-4 | Brian Luby | Complete |

---

## Milestones

| ID | Milestone | Theme(s) | Status | Exit Criteria |
|----|-----------|----------|--------|---------------|
| MS-1 | Markdown parsed into internal domain model | T-1 | Complete | PolicyDocument with sections, requirements, and stable IDs produced from Markdown input |
| MS-2 | First valid OSCAL Catalog from Markdown | T-1, T-2 | Complete | `forge convert policy.md --strategy catalog` produces schema-valid OSCAL Catalog |
| MS-3 | Component Definition + traceability working | T-2 | Complete | `forge convert policy.md --strategy component` produces valid Component Definition with trace links |
| MS-4 | Phase 1 complete — validated, tested, released | T-2, T-3 | Complete | Must-have requirements passing; golden-file coverage; v0.1.0 line established |
| MS-5 | Multi-format output and round-trip verified | T-4 | Complete | JSON/XML/YAML output validated; round-trip equivalence confirmed |
| MS-6 | Profile generation with tailoring | T-5 | Complete | `forge profile` generates valid Profiles with include/exclude and parameter setting |
| MS-7 | Ecosystem integration and community release | T-6 | Complete / v1.0.0 | oscal-cli integration tested; Assessment Plan + SSP template support; community examples and docs published; release automation ready |

---

## Work Item Registry

### Phase 1 — Foundation — Complete

| ID | Work Item | Theme | Status |
|----|-----------|-------|--------|
| WI-1 | Project scaffolding: clap CLI, module structure, error types, CI setup | T-1 | Done |
| WI-2 | Markdown ingestion: file reading, format detection | T-1 | Done |
| WI-3 | Markdown structural extraction: headings, section hierarchy | T-1 | Done |
| WI-4 | Markdown clause extraction: numbered lists, bullets, tables, paragraphs | T-1 | Done |
| WI-5 | Internal domain model: PolicyDocument, PolicySection, PolicyRequirement structs | T-1 | Done |
| WI-6 | Requirement atomization: compound statement splitting | T-1 | Done |
| WI-7 | Deterministic UUID v5 generation with content-based stability | T-1 | Done |
| WI-8 | Citation and reference extraction into internal Citation model | T-1 | Done |
| WI-9 | OSCAL Catalog JSON: groups and controls from domain model | T-2 | Done |
| WI-10 | OSCAL Catalog JSON: statement parts, prose, control structure | T-2 | Done |
| WI-11 | OSCAL metadata: uuid, title, last-modified, version, oscal-version | T-2 | Done |
| WI-12 | OSCAL back matter: resources from citations, link patterns | T-2 | Done |
| WI-13 | End-to-end Catalog pipeline | T-2 | Done |
| WI-14 | Component Definition: documentary component structure | T-2 | Done |
| WI-15 | Component Definition: implemented-requirements with control-id mapping | T-2 | Done |
| WI-16 | Traceability: TraceLink model, source location to OSCAL element mapping | T-2 | Done |
| WI-17 | Traceability: embed trace metadata as props/links in generated artifacts | T-2 | Done |
| WI-18 | End-to-end Component pipeline | T-2 | Done |
| WI-19 | Schema validation: integrate OSCAL schemas, `forge validate` | T-3 | Done |
| WI-20 | Schema validation: actionable error reporting with field locations | T-3 | Done |
| WI-21 | Golden-file test suite: Markdown fixtures and expected OSCAL outputs | T-3 | Done |
| WI-22 | Golden-file edge cases | T-3 | Done |
| WI-23 | Error handling: graceful failures, descriptive messages, exit codes | T-3 | Done |
| WI-24 | Performance benchmark: 50-page document target | T-3 | Done |
| WI-25 | Phase 1 integration testing, CLI polish, v0.1.0 release prep | T-3 | Done |

### Phase 2 — Control Layer & Multi-Format — Complete

| ID | Work Item | Theme | Status |
|----|-----------|-------|--------|
| WI-26 | XML output | T-4 | Done |
| WI-27 | YAML output | T-4 | Done |
| WI-28 | Multi-format round-trip testing | T-4 | Done |
| WI-29 | `forge export` subcommand | T-4 | Done |
| WI-30 | Profile generation with include/exclude controls | T-5 | Done |
| WI-31 | Profile parameter tailoring | T-5 | Done |
| WI-32 | Profile validation and golden-file tests | T-5 | Done |
| WI-33 | Normative vs advisory detection | T-5 | Done |
| WI-34 | Parameter extraction | T-5 | Done |
| WI-35 | Phase 2 integration testing and release prep | T-5 | Done |

### Phase 3 — Ecosystem & Community — Complete

| ID | Work Item | Theme | Status | Evidence |
|----|-----------|-------|--------|----------|
| WI-36 | oscal-cli integration: profile resolution delegation | T-6 | Done | `src/oscal_cli/`, `src/cli/resolve.rs` |
| WI-37 | oscal-cli integration: round-trip validation | T-6 | Done | `src/round_trip/` |
| WI-38 | Traceability report: `forge trace` source-to-OSCAL mapping | T-6 | Done | `src/trace/` |
| WI-39 | Traceability report excerpts and line numbers | T-6 | Done | Integrated into trace report support |
| WI-40 | Batch conversion: multiple documents in one invocation | T-6 | Done | `src/batch/` |
| WI-41 | Assessment Plan scaffolding: reviewed-controls and tasks | T-6 | Done | `src/oscal/assessment_plan.rs` |
| WI-42 | Assessment Plan scaffolding: assessment-subjects from components | T-6 | Done | `generate_assessment_tasks`, `create_assessment_subjects`, pipeline wiring |
| WI-43 | Diff report between OSCAL artifacts | T-6 | Done | `src/diff/`, `src/cli/diff.rs` |
| WI-44 | Summary dashboard conversion statistics | T-6 | Done | `src/summary/` |
| WI-45 | SSP template generation: structure and trace links | T-6 | Done | `src/oscal/ssp.rs`, `tests/ssp_template_test.rs` |
| WI-46 | SSP placeholders for inventory, users, system fields | T-6 | Done | `generate_inventory_items`, placeholder users/fields, SSP golden tests |
| WI-47 | Community examples | T-6 | Done | `examples/` |
| WI-48 | Community documentation | T-6 | Done | `CONTRIBUTING.md`, `docs/usage-guide.md`, `docs/architecture.md` |
| WI-49 | Cross-platform release | T-6 | Done | `.github/workflows/ci.yml`, `.github/workflows/release.yml`, README install docs |
| WI-50 | Phase 3 integration testing and v1.0.0 release prep | T-6 | Done | v1.0.0 release docs, changelog, release gate documented |

---

## Release Gate: v1.0.0

v1.0.0 is the reconciled version target. A release candidate is ready when the following are true:

- `Cargo.toml` package version is `1.0.0`.
- `CHANGELOG.md` includes v1.0.0 release notes.
- `README.md` describes current v1.0.0 capabilities rather than stale Phase 2/Phase 3 plans.
- Cross-platform CI tests run on Linux, macOS, and Windows.
- Release workflow produces binaries, checksums, and SLSA provenance.
- Community examples and docs are present.
- Final CI run passes.

---

## Status Tracking & Health

| Theme | Work Items Total | Done | In Progress | Blocked | Not Started | Health |
|-------|------------------|------|-------------|---------|-------------|--------|
| T-1: Core Pipeline | 8 | 8 | 0 | 0 | 0 | Complete |
| T-2: OSCAL Model Generation | 10 | 10 | 0 | 0 | 0 | Complete |
| T-3: Validation & Quality | 7 | 7 | 0 | 0 | 0 | Complete |
| T-4: Output Format Expansion | 4 | 4 | 0 | 0 | 0 | Complete |
| T-5: Profile & Tailoring | 6 | 6 | 0 | 0 | 0 | Complete |
| T-6: Ecosystem & Community | 15 | 15 | 0 | 0 | 0 | Complete |
| **Total** | **50** | **50** | **0** | **0** | **0** | **v1.1.0** |

---

## Future Roadmap Candidates

The original roadmap is complete. Future work should be tracked in a new roadmap for v1.x or v2 rather than by reopening WI-1 through WI-50.

Candidate future work from PRD deferrals and architecture notes:

| Candidate | Rationale |
|-----------|-----------|
| Assessment Results / SAR generation | Requires actual assessment observations and findings |
| POA&M generation | Requires remediation data and milestone tracking |
| Built-in Profile Resolution engine | Would remove dependency on NIST oscal-cli for profile resolution |
| OSCAL Control Mapping model | Enables policy-to-framework crosswalks |
| External GRC/ticketing/CI integrations | Connects FORGE outputs to operational compliance workflows |
| Web UI or API/server mode | Makes FORGE usable beyond CLI workflows |
| AI/ML semantic policy understanding | Improves intent extraction beyond structural/syntactic parsing |
| Bidirectional traceability view | Interleaves source-to-OSCAL and OSCAL-to-source navigation |
| HTML/interactive reporting | Rich report consumption for non-terminal users |
| Hosted docs site | Improves search and discoverability after community release |
| Full SSP generation from external system data | Moves beyond policy-derived templates using CMDB/cloud/system inventory sources |

---

## Review & Governance

| Review Type | Frequency | Purpose |
|-------------|-----------|---------|
| Release Review | Per release candidate | Verify CI, changelog, version, examples, docs, and artifacts |
| Roadmap Refresh | When planning v1.x/v2 | Create a new roadmap rather than editing completed Phase 1–3 history |
| Post-release Retrospective | After v1.0.0 publication | Identify maintenance and next-roadmap priorities |

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-02-10 | LLM (Claude) | Initial draft with 58 work items across 3 phases in 1-week sprints |
| 0.2 | 2026-02-10 | Brian Luby | Constrained to Markdown-only input; removed PDF/DOCX work items; renumbered to 50 work items |
| 0.3–0.8 | 2026-02-11 to 2026-02-14 | LLM (Claude) | Progressive updates through WI-24 |
| 0.9 | 2026-05-18 | Hermes (Vega) | Major refresh: Phase 1+2 complete, Phase 3 partially complete |
| **1.0** | **2026-05-21** | **Hermes** | **Roadmap reconciliation for v1.0.0: all 50 work items marked Done, stale Phase 3 remaining-work sections removed, future work moved to separate candidate list.** |
| 1.1 | 2026-08-22 | ox-alpha | Reconciled status to v1.1.0 release line; recorded v1.1.0 PDF/DOCX ingestion and SSP control-implementation changes; added decision-log entry superseding the Markdown-only decision. |

---

## Decision Log

| Date | Decision | Rationale | Impact | Alternatives Considered |
|------|----------|-----------|--------|------------------------|
| 2026-02-10 | 1-week sprint cadence | Small scope per sprint reduces risk; enables fast feedback | Higher planning overhead accepted | 2-week sprints; kanban |
| 2026-02-10 | Markdown-only input | Mature external converters exist; in-house PDF/DOCX adds high-risk scope | Removed PDF/DOCX ingestion from initial roadmap | Include PDF/DOCX in Phase 1 |
| 2026-06-09 | Native PDF/DOCX ingestion in v1.1.0 | User demand to remove the external pandoc/markitdown pre-conversion step; matured `pdf-extract`/OOXML parsing made in-house support low-risk | Supersedes the 2026-02-10 Markdown-only decision; v1.1.0 accepts `.pdf` and `.docx` directly | Keep external converter guidance |
| 2026-02-10 | Could Have items in Phase 3 | Must Have and Should Have items came first | Ecosystem work moved later | Include C-items in Phase 2 |
| 2026-05-18 | MS-7 target compressed | Roadmap execution was far ahead of original plan | Phase 3 pulled into near-term release plan | Keep April 2027 target |
| 2026-05-21 | v1.0.0 is the release line | User confirmed version is 1.0; repo contains completed Phase 3 evidence | Roadmap, README, package version, and completion plan reconciled to v1.0.0 | Keep an interim 0.x release |

---

## Open Questions

No open roadmap questions block v1.0.0. Future product questions should be captured in a new v1.x/v2 roadmap.
