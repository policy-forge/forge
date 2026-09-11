# 001-roadmap-forge

> **Document Type:** Product Roadmap
> **Audience:** LLM agents, human reviewers, leadership stakeholders, engineering leads
> **Status:** Active / Post-v1.1.0 Roadmap
> **Last Updated:** 2026-09-09 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->
> **Parent Vision:** docs/FORGE_PRODUCT_VISION.md <!-- @auto -->

---

## Reconciliation Summary

This document is the canonical reconciled roadmap for FORGE. Earlier roadmap
snapshots treated Phase 2 as current and Phase 3 as partially incomplete. The
repository has since moved ahead: the original 50-work-item roadmap is complete,
v1.1.0 is released, and the post-v1.1.0 roadmap is active.

**Current state:**

- 50 of 50 roadmap work items are Done.
- Phase 1 Foundation is complete.
- Phase 2 Control Layer & Multi-Format is complete.
- Phase 3 Ecosystem & Community is complete.
- v1.0.0 established the community-ready distribution baseline.
- v1.1.0 (2026-06-09) added native PDF and DOCX ingestion and policy-derived SSP control implementations.
- The next-wave registry contains 14 PRDs (055–068): 7 have technically complete
  implementations merged into `main`, 2 are in progress, and 5 remain planned.
- PRDs 055–060 and 063 are technically implemented. Their human release,
  pilot, adoption, or other PRD-level gates remain open where the PRDs say so.
- PRD 061 Phases 1–2 merged in PRs #144 and #145. PRD 062 Slice 0
  merged in PR #142; PRDs 064–068 remain planned.

This reconciliation supersedes stale counts such as “43/50 done,” “44/50 done,” “8/15 Phase 3 done,” and “7 remaining.”

---

## Roadmap Context

FORGE translates source policy documents into deterministic, schema-validated
OSCAL artifacts. The completed original roadmap takes the project from
scaffolding through community-ready release:

1. Markdown ingestion and policy parsing.
2. Internal domain modeling and deterministic identifiers.
3. OSCAL Catalog and Component Definition generation.
4. Validation, golden-file coverage, error handling, and performance checks.
5. XML/YAML export and round-trip validation.
6. Profile generation, tailoring, modality tagging, and parameter extraction.
7. oscal-cli integration, traceability reports, diff reports, batch conversion, summary dashboards, Assessment Plan scaffolding, SSP templates, community examples, documentation, CI, and release automation.

The active post-v1.1.0 roadmap extends that foundation into reviewed governance
workflows, reusable policy authoring, assessment and remediation artifacts,
local API-driven interaction, external handoff, and bounded automation. The
original WI-1–WI-50 history remains closed and is not reopened by this work.

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

## Post-v1.1.0 Initiative Registry

The status **Technical implementation complete** means implementation and
remediation changes are merged into `main`. It does not claim completion of
human approvals, design-partner exercises, publication, adoption, or release
gates in the corresponding PRD.

| PRD | Initiative | Status | Repository Evidence | Dependencies / Remaining Gate |
|-----|------------|--------|---------------------|-------------------------------|
| [055](PRD/055-prd-control-mapping.md) | Human-reviewed OSCAL control mapping | Technical implementation complete | Merge `81ad6f9` | Human release gates remain |
| [056](PRD/056-prd-framework-applicability-gap-analysis.md) | Framework applicability and gap analysis | Technical implementation complete | Merge `ffc83b8` | Human release gates remain |
| [057](PRD/057-prd-framework-change-impact-monitoring.md) | Framework-change impact monitoring | Technical implementation complete | Merge `989a684` | Human release gates remain |
| [058](PRD/058-prd-policy-lifecycle-management.md) | Policy lifecycle management | Technical implementation complete | Merges `f8784d1`, `a254d16` | Human release gates remain |
| [059](PRD/059-prd-reusable-policy-components.md) | Reusable policy components | Technical implementation complete | Merge `09f1f37` | PRD release gates remain |
| [060](PRD/060-prd-evidence-implementation-linking.md) | Evidence and implementation linkage | Technical implementation complete | Merge `a5d0aff` | PRD release gates remain |
| [061](PRD/061-prd-framework-guided-policy-authoring.md) | Framework-guided policy authoring | In Progress — Phases 1–2 technically merged | Phase 1 merged in PR #144 ([Phase 1 plan](authoring-phase1-plan.md)); [Phase 2 matrix](authoring-phase2-plan.md) | Components, M-13 impact, HTML and draft handoff merged in PR #145; public Rust API/semver, product, compliance, legal/content, design-partner, readiness, pilot and release gates remain pending |
| [062](PRD/062-prd-local-web-workspace.md) | API-first local web workspace | In Progress — local workspace implementation | Slice 0 delivered in PR #142 | Local API, embedded UI, explicit single-file effects, and headless/browser workflows implemented on the feature branch; full platform, independent review, supply-chain and human gates remain |
| [063](PRD/063-prd-oscal-assessment-results.md) | OSCAL Assessment Results | Technical implementation complete | Merge `34d9869` | PRD release gates remain |
| [064](PRD/064-prd-oscal-poam-workflow.md) | OSCAL POA&M workflow | Planned | No implementation merge | PRD 063 is implemented; schema, status semantics, and editable source-of-truth decisions remain |
| [065](PRD/065-prd-external-workflow-integrations.md) | External workflow integrations | Planned | No implementation merge | Requires a stable source-report contract and approved provider/auth boundary |
| [066](PRD/066-prd-ai-assisted-suggestions.md) | AI-assisted suggestions | Planned | No implementation merge | Requires stable PRD 055/061 schemas, an approved corpus, and privacy/security review |
| [067](PRD/067-prd-read-only-mcp-governance-interface.md) | Read-only MCP governance interface | Planned | No implementation merge | Requires stable project discovery, shared read-only queries, and a completed threat model |
| [068](PRD/068-prd-collaborative-review-queues.md) | Collaborative review queues | Planned | No implementation merge | Requires a selected first review workflow and approved identity/quorum boundaries |

### Now / Next / Later

This is a dependency-oriented sequence, not a date commitment.

| Horizon | Initiative | Intended Outcome / Exit Gate |
|---------|------------|--------------------------------|
| **Now** | PRD 062 Slice 0 | Normative OpenAPI contract, capability matrix, project index schema, threat model, service/effect boundary, representative fixtures, ADRs, and CI drift checks — delivered and validating in CI; human API/security/accessibility approval remains before slice acceptance |
| **Now** | PRDs 055–060 and 063 release-gate reconciliation | Complete or explicitly defer each remaining human, pilot, documentation, and release approval without overstating technical completion as product release |
| **Now** | PRD 061 Phase 2 | Validate explicit components, M-13 impact, offline HTML and draft lifecycle handoff while preserving pending public Rust API/semver, product, compliance, legal/content, design-partner, readiness, pilot and release gates |
| **Now** | PRD 062 implementation and verification | Local explorer, safe mutation and review workflow under technical verification; [usage and limitations](local-workspace.md); pilot/release remain pending |
| **Next** | PRD 064 Phase 1 | Build deterministic POA&M scaffolding and schedule reporting on the completed Assessment Results foundation |
| **Later** | PRDs 065, 067, and 068 | Add external handoff, bounded read-only agent access, and asynchronous collaboration after project, query, identity, and review contracts stabilize |
| **Later** | PRD 066 | Add quarantined AI suggestions after authoring schemas, evaluation data, and provider/privacy gates are approved |

### Active Risks and Dependencies

- PRD 062 Slice 0 contract artifacts (OpenAPI 3.1 contract, capability
  matrix, threat model, ADRs) merged in PR #142 with CI
  drift checks executing, but its product boundary, golden path, API
  ownership, frontend/build approach, containment primitives, and
  accessibility patterns still require human approval for slice acceptance.
  User-authorized implementation is proceeding without claiming those gates pass.
- PRD 064 can proceed only after stable Assessment Results identity and POA&M
  schema/status/source-of-truth decisions are approved.
- PRD 061 Phases 1–2 are technically merged in PRs #144 and #145. Product, compliance,
  public Rust API/semver, legal/content, design-partner readiness, pilot and release
  decisions remain pending; this work does not satisfy those gates. M-13 now has
  explicit Phase 2 comparison tests in the impact unit matrix and CLI suite; that
  is technical evidence only, not human or release acceptance.
- PRDs 065–068 must not bypass their provider, privacy, identity, discovery,
  evaluation, or human-authority boundaries merely to accelerate delivery.

---

## Historical Release Gate: v1.0.0

The completed v1.0.0 release gate required the following:

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

### Post-v1.1.0 PRD Status

| Initiatives Total | Technical Implementation Complete | In Progress | Planned / Not Started | Human Release Gates Pending |
|------------------:|----------------------------------:|------------:|----------------------:|----------------------------:|
| **14** | **7** | **2** | **5** | **7** |

---

## Unscheduled Roadmap Candidates

PRDs 055–068 now own the previously listed Assessment Results, POA&M, control
mapping, integration, web/API, AI, and collaboration opportunities. Remaining
unscheduled candidates are:

| Candidate | Rationale |
|-----------|-----------|
| Built-in Profile Resolution engine | Would remove dependency on NIST oscal-cli for profile resolution |
| Broader bidirectional traceability views | Extends beyond the planned workspace and MCP query contracts |
| Hosted docs site | Improves search and discoverability after community release |
| Full SSP generation from external system data | Moves beyond policy-derived templates using CMDB/cloud/system inventory sources |

---

## Review & Governance

| Review Type | Frequency | Purpose |
|-------------|-----------|---------|
| Release Review | Per release candidate | Verify CI, changelog, version, examples, docs, and artifacts |
| Roadmap Refresh | Monthly and after initiative merges | Reconcile PRD status and dependencies without rewriting completed Phase 1–3 history |
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
| 1.2 | 2026-09-08 | Codex | Added PRDs 055–068; reconciled seven technically implemented and seven planned initiatives; added dependency-oriented Now/Next/Later sequencing and explicit remaining gates. |
| 1.3 | 2026-09-09 | Codex | Reconciled merged PRD-061 Phase 1 and PRD-062 Slice 0; recorded Phase 2 technical evidence and pending API/human/release gates; retired the resolved Phase 1 sequencing question. |

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
| 2026-09-08 | Track post-v1.1.0 work as PRD initiatives | The original 50 work items are complete, while PRDs 055–068 have independent requirements, risks, and release gates | Preserve WI-1–WI-50 as completed history and report next-wave technical versus human-gate status separately | Reopen completed phases; renumber PRDs as work items |

---

## Open Questions

No open roadmap question changes the completed status of WI-1–WI-50 or the
v1.1.0 release. The active post-v1.1.0 roadmap still requires decisions on:

- Which future release will contain the technically implemented PRDs 055–060
  and 063 after their human release gates are dispositioned.
- PRD 062's local-only product boundary, golden path, normative API ownership,
  frontend/build approach, containment primitives, and accessibility patterns.
