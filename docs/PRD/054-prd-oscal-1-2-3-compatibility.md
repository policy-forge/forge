# 054-prd-oscal-1-2-3-compatibility

> **Document Type:** Product Requirements Document
> **Audience:** LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-08-22 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

**Feature Branch**: `054-oscal-1-2-3-compatibility`
**Created**: 2026-08-22
**Status**: Draft
**Input**: FORGE v1.2 roadmap priority 4

## Executive Summary 🟡 `@human-review`

FORGE currently targets OSCAL v1.2.0 through a shared `OSCAL_VERSION` constant, three embedded JSON schemas, v1.2.0-oriented XML fixtures, and a broad set of golden files. That baseline is now stale. NIST published OSCAL v1.2.3 on 2026-08-07 as a patch release focused on security and build maintenance, while the intervening v1.2.1 and v1.2.2 releases also corrected generated schema content.

This PRD upgrades FORGE's standards baseline from OSCAL v1.2.0 to v1.2.3 and makes the upgrade a release gate. It pins official NIST release assets and SHA-256 digests; updates generated metadata; verifies Catalog, Component Definition, and Profile behavior across JSON, XML, and YAML; verifies the existing JSON-only Assessment Plan and SSP serializers; preserves existing v1.2.0 input compatibility; reports the document-declared OSCAL version separately from the schema baseline actually used; and documents a repeatable, integrity-checked process for future upgrades.

This is not a new-model initiative. It does not add Control Mapping, Assessment Results, POA&M, new AP/SSP commands, or a multi-version schema engine. PRD 055 owns Control Mapping and is blocked until this compatibility gate passes.

## Context

### Background 🔴 `@human-required`

FORGE's deterministic policy conversion pipeline emits Catalog and Component Definition artifacts in JSON, XML, and YAML; generates Profiles in all three formats; emits Assessment Plan and SSP artifacts in JSON; imports Catalog and Component Definition artifacts through `forge export`; and validates Catalog, Component Definition, and Profile JSON against schemas compiled into the binary. The schema bundle and most generated fixtures still identify OSCAL v1.2.0.

NIST's [OSCAL v1.2.3 release](https://github.com/usnistgov/OSCAL/releases/tag/v1.2.3) is tagged at commit `e061961` and was published on 2026-08-07. NIST describes v1.2.3 as a patch release with no new models or model changes relative to v1.2.2, focused on dependency, security, and build-environment maintenance. However, upgrading directly from FORGE's v1.2.0 assets includes generated-schema corrections introduced in v1.2.1 and v1.2.2, so version-string replacement alone is insufficient.

NIST's [release-testing guidance](https://pages.nist.gov/OSCAL/learn/tutorials/general/releases/) says release schemas are distributed as GitHub release assets, encourages validation of real content against new releases, and generally provides backward compatibility across minor and patch releases subject to announced exceptions. NIST's [validation guidance](https://pages.nist.gov/OSCAL/learn/concepts/validation/) distinguishes well-formed data from schema-valid OSCAL and warns that selecting the wrong schema is a separate failure mode.

### Current-State Evidence 🟡 `@human-review`

- `src/oscal/metadata.rs` defines `OSCAL_VERSION` as `1.2.0`; Catalog, Component Definition, Profile, Assessment Plan, and SSP builders derive their metadata from it.
- `src/validate/mod.rs` embeds three Draft-07 JSON schemas for Catalog, Component Definition, and Profile and selects among them by detected root model, not by `metadata.oscal-version`.
- `forge validate` accepts JSON only. It auto-detects Catalog, Component Definition, and Profile, although its explicit `--schema-type` override currently lists only Catalog and Component Definition.
- Catalog and Component Definition conversion auto-validates the JSON representation before JSON/XML/YAML serialization. Profile generation has schema tests but no runtime auto-validation step.
- `forge export` consumes and re-emits Catalog and Component Definition JSON/XML/YAML only. Profile export is explicitly rejected.
- `forge validate --round-trip` and PRD 037 exercise Catalog and Component Definition through oscal-cli's JSON → XML → YAML → JSON chain.
- Assessment Plan is an optional JSON secondary output from Catalog/Component conversion; SSP is a JSON-only `--to ssp` output. Neither is in `OscalModelType`, `forge validate`, or `forge export`, and neither currently passes through runtime schema validation.
- The committed Catalog and Profile JSON schemas match the official v1.2.0 release digests. The Component JSON schema was manually changed after import, so it is not byte-identical to its NIST release asset. The Catalog and complete XSD fixtures identify v1.2.0, while the Component XSD fixture still identifies v1.1.3.
- Hard-coded `1.2.0` expectations appear across production comments, unit tests, integration tests, snapshots, export fixtures, and golden files. The migration must distinguish intentionally refreshed generated fixtures from retained legacy-input fixtures.

### Problem Statement 🔴 `@human-required`

Compliance engineers cannot confidently claim that newly generated FORGE artifacts target NIST's current OSCAL release while the binary emits `oscal-version: 1.2.0`, validates against a mixed-provenance schema bundle, and uses inconsistent XSD fixtures. A naive global replacement would risk hiding schema incompatibilities, rewriting legacy document metadata during export, and breaking users with existing v1.2.0 artifacts. FORGE needs a verifiable baseline upgrade that proves interoperability across every currently emitted model and format while retaining an explicit, testable backward-compatibility contract.

### Target Users 🟡 `@human-review`

- **Primary:** Compliance engineers who need current, schema-conformant OSCAL output accepted by downstream tools.
- **Primary:** DevSecOps engineers who need deterministic offline validation and a stable CI compatibility contract.
- **Secondary:** Auditors who need reports to distinguish a document's declared OSCAL version from the validator's schema baseline.
- **Secondary:** FORGE maintainers who need a repeatable, supply-chain-safe standards upgrade procedure.

### Scope Boundaries 🟡 `@human-review`

**In Scope:**

- Pinning OSCAL v1.2.3 official release identity, assets, and SHA-256 digests.
- Replacing the runtime Catalog, Component Definition, and Profile JSON schemas with unmodified official v1.2.3 assets.
- Updating the shared generated-output baseline from `OSCAL_VERSION = "1.2.0"` to `"1.2.3"`.
- Catalog, Component Definition, and Profile JSON/XML/YAML compatibility fixtures and regression coverage.
- Offline v1.2.3 schema compatibility tests for the existing JSON-only Assessment Plan and SSP serializers.
- Continued acceptance of existing OSCAL v1.2.0 inputs on currently supported input paths.
- Reporting `declared_oscal_version` separately from `schema_version_used`.
- oscal-cli version/baseline compatibility reporting and divergence classification.
- A documented and automatable future schema-upgrade workflow.

**Out of Scope:**

- Control Mapping generation, validation, import, export, or schema embedding; PRD 055 owns that model.
- Assessment Results or POA&M generation.
- Adding Assessment Plan or SSP to the public `forge validate`, `forge export`, or round-trip model set.
- Adding XML/YAML output for Assessment Plan or SSP.
- A native Profile Resolution implementation or changes to the resolution algorithm.
- Runtime schema downloads or automatic selection of arbitrary historical schemas.
- A general `--schema-version` flag or parallel multi-version schema registry.
- Reinterpreting or semantically migrating user content between OSCAL model versions.

### Key Terms 🟡 `@human-review`

The **standards baseline** is the single pinned OSCAL release embedded by FORGE: v1.2.3. The **declared OSCAL version** is the artifact's `metadata.oscal-version`; the **schema version used** is the actual pinned validation baseline; and the user-owned **document version** is `metadata.version`. A **legacy fixture** retains v1.2.0 to prove input compatibility, while a **generated fixture** represents current v1.2.3 output. The **provenance manifest** records each asset's release, URL, size, digest, role, and path. Passing the **compatibility gate** is required to ship this upgrade.

### Related Documents ⚪ `@auto`

| Document | Link | Relationship |
|----------|------|--------------|
| Product Roadmap | `docs/FORGE_PRODUCT_ROADMAP.md` | v1.2 planning context |
| Product Vision | `docs/FORGE_PRODUCT_VISION.md` | Correctness, determinism, offline operation, and standards-native principles |
| Schema Validation PRD | `docs/PRD/019-prd-schema-validation.md` | Original embedded v1.2.0 JSON validation contract |
| Round-Trip Testing PRD | `docs/PRD/028-prd-round-trip-testing.md` | Internal JSON/XML/YAML semantic-equivalence contract |
| Profile Validation PRD | `docs/PRD/032-prd-profile-validation-tests.md` | Profile schema and golden-file coverage |
| oscal-cli Round-Trip PRD | `docs/PRD/037-prd-oscal-cli-round-trip.md` | External conversion compatibility and divergence logging |
| Control Mapping PRD | `docs/PRD/055-prd-control-mapping.md` | Blocked downstream initiative; not part of this scope |
| NIST v1.2.3 Reference | [OSCAL v1.2.3 model documentation](https://pages.nist.gov/OSCAL-Reference/models/v1.2.3/) | Normative model reference for this baseline |

## Goals 🔴 `@human-required`

- **G-1 — Current output baseline:** 100% of artifacts newly generated by FORGE declare `metadata.oscal-version` as `1.2.3` without changing the user-owned `metadata.version` value.
- **G-2 — Compatibility proof:** 100% of representative Catalog, Component Definition, Profile, Assessment Plan, and SSP generated fixtures pass the applicable official OSCAL v1.2.3 schema gate.
- **G-3 — Preserve existing users:** 100% of retained valid v1.2.0 compatibility fixtures remain accepted through every input/validation/export path that supported them before this upgrade.
- **G-4 — Verifiable supply chain:** Every vendored NIST schema byte matches its recorded official GitHub release digest, with zero unexplained or hand-edited differences.
- **G-5 — Honest diagnostics:** Every validation and oscal-cli compatibility report identifies both the document-declared version and the schema/tool baseline used, so no result implies validation against a schema that was not actually used.
- **G-6 — Repeatable maintenance:** A maintainer can evaluate and stage a future OSCAL patch upgrade from a clean checkout using one documented procedure, without runtime network access or manual asset substitution.

## Non-Goals 🔴 `@human-required`

- **NG-1 — New OSCAL models:** This PRD will not expose Control Mapping, Assessment Results, or POA&M behavior; compatibility work must not become model expansion.
- **NG-2 — Multi-version validation engine:** FORGE will not embed v1.2.0, v1.2.1, v1.2.2, and v1.2.3 as parallel selectable schema sets. A single current schema validates the supported patch line.
- **NG-3 — Metadata-driven downloads:** `metadata.oscal-version` will not trigger network access, fetch schemas, or execute remote references.
- **NG-4 — Metadata rewriting on import:** `forge export` will not silently replace an imported artifact's `oscal-version` with `1.2.3`; conversion between formats must preserve document semantics.
- **NG-5 — Expanded AP/SSP surface:** Passing compatibility tests does not add AP/SSP to `OscalModelType`, `--schema-type`, `forge export`, or oscal-cli round trips.
- **NG-6 — oscal-cli certification:** FORGE will report tested oscal-cli interoperability, not claim that an oscal-cli release based on an older OSCAL model validates v1.2.3 conformance.

## Supported-Version Policy 🟡 `@human-review`

### Generated output

- All newly generated Catalog, Component Definition, Profile, Assessment Plan, and SSP documents must set `metadata.oscal-version` to `1.2.3` through the shared constant.
- `metadata.version` remains the source document/artifact revision and must not be changed by this standards upgrade.
- Generated golden files and snapshots are refreshed to v1.2.3 only after they pass the new schema and format gates.

### Existing input

- **Existing v1.2.0 input remains accepted.** v1.2.0, v1.2.1, v1.2.2, and v1.2.3 declarations are supported on the Catalog, Component Definition, and Profile paths that already accept those models and formats.
- Supported v1.2.x artifacts are validated against the single pinned v1.2.3 schema for their model. FORGE does not claim that it loaded the exact historical schema named in the metadata.
- A v1.2.0 artifact that relied on a defect or looser constraint corrected by a later v1.2.x schema may fail. The diagnostic must say that the document declared v1.2.0 and failed the v1.2.3 compatibility schema; it must not call the file malformed without the failing path and rule.
- `forge export` must preserve the imported `metadata.oscal-version` value across JSON/XML/YAML conversion. Successful export of a legacy artifact does not silently upgrade its declared conformance.
- Input declaring an OSCAL version outside `1.2.0`–`1.2.3` is unsupported for this release. `forge validate` and `forge export` must return a non-zero unsupported-version diagnostic rather than report generic v1.2.3 validity as if it proved conformance to the declared version.

### Schema selection and reporting

- Model/root detection or an existing explicit model override selects the schema family. `metadata.oscal-version` is inspected for support and reporting but does not choose a schema file.
- Human-readable validation output must name the model, declared version, and `schema_version_used: 1.2.3`.
- Machine-readable validation and round-trip output must expose stable fields for `declared_oscal_version`, `schema_version_used`, and, when applicable, `oscal_cli_version`.
- Missing or non-string `metadata.oscal-version` remains a schema violation. It must not be silently defaulted on imported artifacts.

## User Stories & Priorities 🔴 `@human-required`

| ID | Priority | User story |
|----|----------|------------|
| US-1 | P0 | As a compliance engineer, I want every newly generated artifact to target OSCAL v1.2.3 so that I can exchange it with current standards-aware tooling. |
| US-2 | P0 | As a repository maintainer, I want valid v1.2.0 artifacts accepted by existing validation and export workflows so that upgrading FORGE does not force immediate migration. |
| US-3 | P0 | As an auditor, I want the declared and actually validated versions reported separately so that I can evaluate the conformance claim. |
| US-4 | P0 | As a FORGE maintainer, I want schema gates for all five emitted models so that a standards update cannot silently break a serializer. |
| US-5 | P0 | As a DevSecOps engineer, I want JSON, XML, and YAML fixtures valid and semantically equivalent so that format conversions do not drift. |
| US-6 | P1 | As a security-conscious maintainer, I want every schema traced to an official asset and checksum so that reviewed bytes are the bytes compiled or tested. |
| US-7 | P1 | As a developer, I want round-trip output to identify FORGE and oscal-cli baselines so that tool-version differences are visible. |
| US-8 | P1 | As a maintainer, I want a reviewable upgrade workflow so that the next patch does not recreate mixed provenance or stale fixtures. |

## Requirements

### Must Have (M) — Release blockers 🔴 `@human-required`

- [ ] **M-1 — Release pin:** The repository shall identify the source as `usnistgov/OSCAL`, tag `v1.2.3`, release commit `e061961`, and publication date `2026-08-07`.
- [ ] **M-2 — Provenance manifest:** Every vendored runtime or compatibility-test schema shall have a manifest entry containing exact asset name, official release URL, local path, byte size, SHA-256, OSCAL version, format, model, and runtime/test role.
- [ ] **M-3 — Unmodified assets:** Vendored schemas shall be byte-identical to their official release assets. Local edits to `$id`, comments, whitespace, constraints, or namespaces are prohibited; FORGE-specific annotations belong in the manifest.
- [ ] **M-4 — Runtime JSON bundle:** The embedded Catalog, Component Definition, and Profile JSON schemas shall be replaced with the official v1.2.3 release assets and shall compile successfully with the existing `jsonschema` validator without network resolution.
- [ ] **M-5 — Shared output version:** The shared `OSCAL_VERSION` shall be `1.2.3`, and all five currently emitted model families shall derive `metadata.oscal-version` from that single authority.
- [ ] **M-6 — Version semantics:** Code, CLI help, and documentation shall distinguish user-owned `metadata.version`, document-declared `metadata.oscal-version`, and FORGE's `schema_version_used`.
- [ ] **M-7 — Supported input range:** Existing Catalog, Component Definition, and Profile inputs declaring OSCAL v1.2.0 through v1.2.3 shall remain accepted on currently supported paths when compatible with the v1.2.3 schema.
- [ ] **M-8 — Unsupported declarations:** Inputs declaring a version outside the supported range shall fail with an unsupported-version diagnostic naming both the declaration and the available v1.2.3 baseline; FORGE shall not imply validation against the unavailable version.
- [ ] **M-9 — Metadata preservation:** `forge export` shall preserve the imported `metadata.oscal-version` and `metadata.version` across supported JSON/XML/YAML format pairs.
- [ ] **M-10 — Validation reporting:** Text and JSON validation results shall report model type, `declared_oscal_version`, and `schema_version_used`; round-trip results shall also report `oscal_cli_version` when oscal-cli runs.
- [ ] **M-11 — Catalog gate:** Generated Catalog JSON shall validate against the v1.2.3 Catalog JSON schema, generated XML shall validate against the v1.2.3 Catalog XSD, and YAML shall parse to a JSON value that validates against the v1.2.3 Catalog JSON schema.
- [ ] **M-12 — Component gate:** Generated Component Definition JSON/XML/YAML shall pass the equivalent v1.2.3 model-specific gates.
- [ ] **M-13 — Profile gate:** Generated Profile JSON/XML/YAML, including include, exclude, and parameter-tailoring cases, shall pass the equivalent v1.2.3 model-specific gates. This requirement adds tests, not Profile support to `forge export`.
- [ ] **M-14 — Assessment Plan gate:** Existing generated Assessment Plan JSON fixtures shall validate offline against the official v1.2.3 Assessment Plan JSON schema, including reviewed controls, tasks, and assessment subjects.
- [ ] **M-15 — SSP gate:** Existing generated SSP JSON fixtures shall validate offline against the official v1.2.3 SSP JSON schema, including metadata, system implementation, control implementation, users, components, and back matter.
- [ ] **M-16 — Legacy fixtures:** At least one representative v1.2.0 fixture per currently supported model/format input path shall remain immutable and be labeled as legacy input rather than refreshed generated output.
- [ ] **M-17 — Golden refresh discipline:** Generated golden files and snapshots shall be updated only for intentional v1.2.3 changes; review must show that non-version structural diffs are explained by official schema compatibility fixes or explicit model corrections.
- [ ] **M-18 — Internal round trips:** Catalog and Component Definition shall retain 100% semantic equivalence across all existing JSON/XML/YAML export pairs; Profile shall retain equivalence across its generation serializers and tests.
- [ ] **M-19 — oscal-cli round trips:** Existing Catalog and Component Definition JSON → XML → YAML → JSON checks shall run when oscal-cli is available, record tool/baseline versions, and finish with zero unresolved FORGE-caused divergences.
- [ ] **M-20 — Honest oscal-cli status:** An oscal-cli version built against an older OSCAL model may provide advisory conversion evidence but shall not be described as authoritative v1.2.3 schema validation. The compatibility matrix shall identify its embedded/claimed model baseline when known.
- [ ] **M-21 — Offline operation:** Normal conversion, export, validation, and all non-oscal-cli compatibility tests shall make zero network requests. XML validation shall use local schemas with network access disabled.
- [ ] **M-22 — Future upgrade procedure:** A documented maintainer workflow shall fetch allowlisted assets into a temporary directory, verify release metadata and SHA-256 before replacement, generate/update the manifest, show semantic/schema diffs for review, run the full compatibility matrix, and fail closed on any mismatch.
- [ ] **M-23 — Regression gate:** `cargo test`, strict Clippy, formatting checks, cross-platform CI, schema checksum verification, and all v1.2.3/legacy compatibility tests shall pass before release.
- [ ] **M-24 — Scope guard:** Tests shall confirm that this PRD does not add Control Mapping roots/schemas to runtime selection, new AP/SSP public commands, runtime downloads, or a `--schema-version` option.
- [ ] **M-25 — Documentation:** README/usage/reference documentation shall state the generated baseline, supported input range, schema-selection rule, legacy behavior, offline guarantee, and oscal-cli compatibility limitation.

### Should Have (S) — High-priority follow-through 🟡 `@human-review`

- [ ] **S-1 — Machine-readable manifest:** The provenance manifest should use a stable JSON or TOML schema so CI can verify assets without scraping Markdown.
- [ ] **S-2 — Upgrade diff artifact:** The maintainer workflow should produce a concise prior-baseline → candidate-baseline schema diff summary by model, including changed definitions, constraints, enums, IDs, and namespaces.
- [ ] **S-3 — Validation JSON contract:** Machine-readable validation results should add version fields additively and retain all existing report fields so current consumers do not break.
- [ ] **S-4 — Explicit fixture layout:** Generated-current, legacy-input, and adversarial-invalid fixtures should live in visibly separate directories or carry manifest labels that prevent accidental bulk replacement.
- [ ] **S-5 — Cross-platform XML gate:** CI should run local XSD validation on at least Linux and macOS; Windows may use an equivalent pinned validator if `xmllint` is unavailable.
- [ ] **S-6 — Release notes:** The FORGE release notes should call out that output metadata changes to 1.2.3 while v1.2.0 inputs remain supported and preserved on export.
- [ ] **S-7 — Drift check:** CI should fail if a vendored schema's bytes, size, version identifier, or checksum diverge from the provenance manifest.

### Could Have (C) — Optional if low-cost 🟢 `@llm-autonomous`

- [ ] **C-1 — Binary baseline display:** `forge --version` could add a separate line or structured diagnostic that exposes `OSCAL schema baseline 1.2.3` without changing the package semantic version.
- [ ] **C-2 — Upgrade dry run:** The maintainer workflow could support a no-write mode that downloads and verifies a candidate release, then emits only the compatibility/diff plan.
- [ ] **C-3 — Schema inventory test:** A compile-time or unit test could assert that every `OscalModelType` has exactly one runtime schema and provenance entry.
- [ ] **C-4 — Upstream issue links:** Known schema or oscal-cli divergences could include an upstream NIST issue/discussion URL in the compatibility matrix.

### Won't Have (W) — Explicitly excluded from this PRD 🟡 `@human-review`

- [ ] **W-1 — Control Mapping:** No mapping model types, commands, schemas, or fixtures; PRD 055 depends on this PRD instead.
- [ ] **W-2 — Assessment Results or POA&M:** No assessment-result or remediation model generation.
- [ ] **W-3 — Historical schema registry:** No embedded v1.2.0–v1.2.2 runtime schema copies and no arbitrary schema selection.
- [ ] **W-4 — Runtime download:** No fetching from NIST, GitHub, schema `$id`, or other network locations during normal use.
- [ ] **W-5 — AP/SSP public validation/export:** Compatibility gates do not expand the current CLI model surface.
- [ ] **W-6 — AP/SSP XML/YAML:** Existing JSON-only scope remains unchanged.
- [ ] **W-7 — Native resolver:** No change to delegated Profile Resolution behavior.
- [ ] **W-8 — Content migration:** No automatic rewrite of imported artifacts from 1.2.0 metadata or structure to 1.2.3.

## Official Asset Pin 🟡 `@human-review`

All digests below are from the official v1.2.3 GitHub release asset metadata. The implementation must re-verify them before vendoring and record final local paths in the provenance manifest.

| Model / role | Official asset | Size | SHA-256 |
|--------------|----------------|-----:|---------|
| Catalog runtime JSON | `oscal_catalog_schema.json` | 55,199 | `ab95836e9e8dfeb6fde80007f6cc76fa3192f595d427c751a3f3923c3f474fc2` |
| Component runtime JSON | `oscal_component_schema.json` | 82,323 | `95e76881151ececd5cb1a93ff0f70ad74b8cc1aa58771626ac8b262bf2c8e001` |
| Profile runtime JSON | `oscal_profile_schema.json` | 68,516 | `7c5ff5a92683b6a80ce6a3474dea04e0ea8680a4ecb60702ffa08ebfc677e6cb` |
| Assessment Plan test JSON | `oscal_assessment-plan_schema.json` | 144,952 | `ea687b9d0ab1d84c9cb11ee0a5e22b17956fe892ee93f5acca937bef81d23ea2` |
| SSP test JSON | `oscal_ssp_schema.json` | 105,538 | `99b1bee2df5604a7d4f1faeab43148e0bd4c4a1f513cd0d2d0c7896aa216a1e7` |
| Catalog XML gate | `oscal_catalog_schema.xsd` | 115,340 | `07e8131af5efb67be209cd5403a9eaaf4650b9ffb73d7a1d9a6ccdac388525be` |
| Component XML gate | `oscal_component_schema.xsd` | 174,249 | `9ee6573f835708bf282c47eaafc6e30a5c9b4dfcd6ffacf8a0a08f9f5b3a4895` |
| Profile XML gate | `oscal_profile_schema.xsd` | 138,462 | `02b8c7a08547b40da9448a7810ad403d5e7b7a4c11ec95c77875976a1c1e56f3` |
| Cross-model XML fallback/test | `oscal_complete_schema.xsd` | 571,954 | `c7bccd69cfe1e7e9fbc225f5969bf021d09476fc8d3a3842250d10dc77d3f003` |

Direct assets must use URLs under `https://github.com/usnistgov/OSCAL/releases/download/v1.2.3/`. The aggregate archives are not required for the implementation; if used during maintenance, their digests must also be recorded and archive extraction must reject unsafe paths and links.

## Compatibility Fixture Matrix 🟡 `@human-review`

| Model | FORGE behavior | Current formats | v1.2.3 gate | Legacy v1.2.0 gate |
|-------|----------------|-----------------|-------------|--------------------|
| Catalog | Generate, validate, export, diff, round trip | JSON/XML/YAML | Model JSON schema; model XSD; YAML→JSON schema; semantic pairs | Validate and export all existing formats; preserve declaration |
| Component Definition | Generate, validate, export, diff, round trip | JSON/XML/YAML | Model JSON schema; model XSD; YAML→JSON schema; semantic pairs | Validate and export all existing formats; preserve declaration |
| Profile | Generate, validate, resolve; export unsupported | JSON/XML/YAML generation | Model JSON schema; model XSD; YAML→JSON schema; include/exclude/params | Validate JSON and exercise existing generated-format paths |
| Assessment Plan | Optional secondary generation | JSON only | Model JSON schema; reviewed-controls/tasks/subjects fixtures | Existing v1.2.0 fixture validates as compatible or documented correction |
| SSP | `convert --to ssp` generation | JSON only | Model JSON schema; structure and golden fixtures | Existing v1.2.0 fixture validates as compatible or documented correction |

For AP/SSP, a failure against v1.2.3 is a release-blocking compatibility defect in existing serialization. Fixing a field or structure needed for conformance is allowed; adding new model features, formats, or commands is not.

## Acceptance Criteria — Given / When / Then 🟡 `@human-review`

| AC | Requirements | Given | When | Then |
|----|--------------|-------|------|------|
| AC-1 | M-1–M-4 | A clean checkout and the provenance manifest | Verifying vendored runtime schemas | Each byte, size, release URL, version ID, and SHA-256 matches the official v1.2.3 asset |
| AC-2 | M-3 | A maintainer changes one byte in an embedded schema | Running the provenance check | CI fails and identifies the asset; no automatic manifest rewrite occurs |
| AC-3 | M-5, M-6 | A policy with `version: 7.4` | Generating each current model | `metadata.version` remains `7.4` and `metadata.oscal-version` is `1.2.3` |
| AC-4 | M-7, M-10 | A valid legacy Catalog declaring `1.2.0` | Running `forge validate --format json` | Exit is zero and output reports declared `1.2.0` and schema used `1.2.3` |
| AC-5 | M-7, M-9 | A v1.2.0 Catalog in JSON | Exporting JSON→XML→YAML→JSON | Every output preserves `oscal-version: 1.2.0` and remains semantically equivalent |
| AC-6 | M-8, M-10 | An artifact declaring `1.3.0` | Running validation or export | Command fails non-zero and names unsupported `1.3.0` plus available baseline `1.2.3` |
| AC-7 | M-6 | An artifact whose `metadata.version` also equals `1.2.3` | Running validation | Output labels document version and OSCAL/schema versions separately; equality is not treated as semantic equivalence |
| AC-8 | M-11 | A generated Catalog fixture | Exercising JSON, XML, and YAML gates | All three validate offline and normalize to semantically equivalent content |
| AC-9 | M-12 | A generated Component Definition fixture | Exercising JSON, XML, and YAML gates | All three validate offline and normalize to semantically equivalent content |
| AC-10 | M-13 | Include, exclude, and parameter-tailored Profiles | Exercising JSON, XML, and YAML gates | Each passes its v1.2.3 schema gate with no Profile export support added |
| AC-11 | M-14 | Catalog- and component-derived Assessment Plans | Validating JSON offline | Both pass the v1.2.3 Assessment Plan schema, including tasks and subjects |
| AC-12 | M-15 | A generated SSP skeleton with controls and components | Validating JSON offline | It passes the v1.2.3 SSP schema with zero validation errors |
| AC-13 | M-16, M-17 | Current generated and legacy-input fixtures | Reviewing the fixture diff | Generated fixtures declare 1.2.3; legacy fixtures still declare 1.2.0 and are not bulk-updated |
| AC-14 | M-18 | Catalog and Component fixtures in all three formats | Running all nine existing export pairs per model | Semantic equivalence remains 100% and validation uses v1.2.3 schemas |
| AC-15 | M-19, M-20 | A functional oscal-cli | Running Catalog and Component round trips | Result records oscal-cli version, declared version, schema baseline, and zero unresolved FORGE-caused divergence |
| AC-16 | M-20 | oscal-cli reports a model baseline older than 1.2.3 | Rendering compatibility status | Output says conversion compatibility is advisory and does not call it v1.2.3 validation |
| AC-17 | M-21 | Network access is disabled | Running conversion, export, validation, and compatibility tests except optional oscal-cli cases | All required operations complete without network attempts |
| AC-18 | M-21 | Local v1.2.3 XSDs and XML fixtures | Running XML validation | Validator runs in no-network mode and accepts current output or returns a local actionable schema error |
| AC-19 | M-22 | A candidate future release tag and clean checkout | Running the documented upgrade workflow in dry-review order | Assets are staged in temporary storage, verified before replacement, diffed, manifested, and tested |
| AC-20 | M-22 | A candidate asset digest differs from release metadata | Running the update workflow | Workflow stops before changing vendored schemas and reports expected/actual digests |
| AC-21 | M-23 | The implementation branch | Running all quality gates | Tests, strict Clippy, formatting, provenance verification, and supported-platform CI pass |
| AC-22 | M-24 | The completed compatibility change | Inspecting CLI/schema inventory | No Control Mapping runtime support, AP/SSP command expansion, runtime download, or version-selection flag exists |
| AC-23 | M-25 | The release documentation | Reviewing baseline and compatibility guidance | Generated, accepted-input, schema-selection, offline, and oscal-cli policies match this PRD |

### Edge Cases 🟢 `@llm-autonomous`

- [ ] **EC-1:** `metadata.oscal-version` is missing, null, numeric, empty, or whitespace-only; validation fails at the metadata path and does not substitute `1.2.3`.
- [ ] **EC-2:** A JSON document contains multiple recognized OSCAL roots; the existing ambiguous-artifact failure remains and no version is inferred.
- [ ] **EC-3:** A YAML document uses quoted `"1.2.0"`; deserialization preserves it as a string and export does not coerce it to a number.
- [ ] **EC-4:** XML uses the stable OSCAL namespace while metadata declares v1.2.0; schema namespace matching and document version reporting remain separate.
- [ ] **EC-5:** A v1.2.0 artifact passes the old schema but violates a v1.2.3-corrected constraint; error output names the compatibility boundary and exact instance/schema paths.
- [ ] **EC-6:** Numeric version parsing rejects unsupported `1.2.10` and prerelease `1.2.3-rc1`; no lexical or normalization shortcut widens the policy.
- [ ] **EC-7:** A correctly named release asset with wrong bytes, or a schema with a non-local reference, fails the integrity gate closed.
- [ ] **EC-8:** If oscal-cli is absent, core gates still pass and external compatibility reports unavailable; if it changes `oscal-version`, the change is a classified divergence.
- [ ] **EC-9:** If AP/SSP validation exposes a serializer defect, release remains blocked until output conforms or the product owner explicitly removes that output from the release.

## Technical Constraints 🟡 `@human-review`

- **Language/toolchain:** Rust edition 2024, stable 1.93.0, using existing project dependencies unless an implementation spike proves a gap.
- **JSON Schema:** Retain the existing `jsonschema` crate and Draft-07 compilation path; v1.2.3 assets must compile without remote resolution.
- **Schema authority:** Official release assets are authoritative. Generated NIST reference pages help interpretation but are not substituted for release bytes.
- **Runtime bundle:** Catalog, Component Definition, and Profile JSON schemas remain compile-time embedded via `include_str!` or an equivalent offline mechanism.
- **Test schemas:** AP/SSP JSON and model XSD assets may remain test/release-gate data; their presence does not imply runtime command support.
- **YAML validation:** Parse YAML into the same JSON value representation, then validate with the corresponding official JSON schema; NIST publishes JSON schemas for JSON/YAML validation.
- **XML validation:** Use model-specific v1.2.3 XSDs where available. Run validators with network disabled and no external entity/schema fetching.
- **Schema selection:** Root model detection or current explicit override chooses the schema; version metadata enforces support/reporting but never becomes a file path or URL.
- **Serialization:** Legacy import/export must preserve metadata values. New generation uses the shared current constant.
- **No hidden fallback:** If a v1.2.3 schema fails to compile, FORGE must fail; it must not silently fall back to v1.2.0 or oscal-cli.
- **Bounds:** Existing input-size, timeout, subprocess, and temporary-file limits remain in force.
- **Determinism:** Identical inputs and options must remain deterministic under existing normalization rules; the baseline update must not change IDs solely because schema files changed.

### Validation Result Contract 🟡 `@human-review`

The exact Rust types are an engineering decision, but the externally observable result must be equivalent to:

```json
{
  "artifact": "policy-catalog.json",
  "model_type": "catalog",
  "declared_oscal_version": "1.2.0",
  "schema_version_used": "1.2.3",
  "supported_input": true,
  "valid": true,
  "errors": []
}
```

Round-trip results add `oscal_cli_version` and a compatibility classification such as `verified-conversion`, `advisory-older-model-baseline`, or `unavailable`. Existing fields remain additive and stable.

## oscal-cli Compatibility Policy 🟡 `@human-review`

As of this PRD, the latest official [oscal-cli v1.0.3 release](https://github.com/usnistgov/oscal-cli/releases/tag/v1.0.3) states that it uses OSCAL v1.1.2 models. FORGE therefore cannot treat the tool's successful conversion as proof of v1.2.3 schema conformance.

- The embedded v1.2.3 schemas are the release's structural-validation authority.
- oscal-cli is an optional external interoperability oracle for conversion and Profile Resolution behavior.
- Compatibility reports must record the detected oscal-cli version and documented model baseline when known.
- A pass using an older-model oscal-cli means the tested artifact survived that conversion path with no unresolved Forge-caused divergence; it does not supersede schema validation.
- A divergence caused by the older external model is classified and documented per PRD 037, not hidden or automatically assigned to FORGE.
- The compatibility matrix must be refreshed when NIST publishes a newer oscal-cli release.

## Future Upgrade Procedure 🟡 `@human-review`

1. **Select:** Require an explicit stable OSCAL release tag; reject `main`, branches, drafts, and prereleases unless the product owner approves a pre-release evaluation.
2. **Inspect:** Read official release notes, release commit/signature state, compatibility announcements, and model/reference changes before downloading assets.
3. **Fetch:** Download only the allowlisted model JSON/XSD assets needed by current FORGE behavior into a newly created temporary directory.
4. **Verify:** Compare filenames, sizes, GitHub release digests, locally computed SHA-256, schema `$id`/version, and XSD `schema-version`; stop on any mismatch.
5. **Diff:** Produce prior→candidate schema diffs and classify changes to definitions, required fields, enums, data types, namespaces, and external references.
6. **Stage:** Replace vendored files without editing them and update the provenance manifest in the same reviewable change.
7. **Test current output:** Run the full generated-model/format matrix, including AP/SSP compatibility-only gates.
8. **Test legacy input:** Run retained fixtures for every supported historical patch version and existing import/export path.
9. **Test ecosystem:** Run internal round trips, the supported oscal-cli matrix when available, and classify every divergence.
10. **Review and release:** Require human review of schema diffs, generated fixture changes, compatibility exceptions, documentation, and release notes before merge.

The procedure may be implemented as a script, task runner command, or documented CI workflow. It must support a no-replacement verification stage and must never fetch assets during normal end-user commands.

## Security, Privacy, and Supply-Chain Integrity 🟡 `@human-review`

| Risk | Impact | Required mitigation |
|------|--------|---------------------|
| Tampered or wrong schema asset | FORGE could certify invalid artifacts or reject valid ones | Official release URLs, pinned tag/commit, SHA-256 manifest, byte-for-byte CI verification, fail closed |
| Hand-edited vendored schema | Provenance becomes unverifiable and upgrades become non-repeatable | Prohibit local schema edits; record explanations outside asset bytes |
| Remote `$ref` or XSD import | Validation could make network requests or consume attacker-controlled content | Assert references are local/internal; disable network in XML validator; no runtime resolver |
| Malicious OSCAL input | Parser/validator exhaustion or terminal injection | Preserve size limits, bounded error collection, control-character sanitization, no panics |
| XML entity expansion | File disclosure or denial of service in test/import tooling | Keep quick-xml's non-resolving path, reject dangerous declarations as applicable, use `xmllint --nonet` for gates |
| YAML alias/type behavior | Resource exhaustion or semantic coercion | Use existing bounded parser path, quoted version fixtures, normalize to JSON before schema validation |
| Sensitive policy disclosure | Validation or round-trip output could leak policy content/paths | Keep processing local, report paths/rules rather than full sensitive values, preserve temp cleanup |
| External subprocess mismatch | Older oscal-cli output may be mistaken for current conformance | Report tool/model baseline and classify evidence as advisory when appropriate |
| Unsafe archive extraction | Aggregate release archive could overwrite arbitrary paths | Prefer direct assets; if archives are used, reject absolute paths, `..`, symlinks, and unexpected files |

No policy document or generated artifact is uploaded by this feature. Network access is confined to the maintainer-only schema update workflow and optional pre-existing external-tool installation; runtime validation remains offline.

## Dependencies & Interactions 🟡 `@human-review`

- **Requires:** Existing schema loader and validator (`src/validate`), shared metadata assembly (`src/oscal/metadata.rs`), JSON/XML/YAML serializers, golden fixtures, export matrix, Profile tests, AP tests, SSP tests, and PRD 037 round-trip infrastructure.
- **External:** Official NIST OSCAL v1.2.3 release assets and reference documentation.
- **Blocks:** PRD 055 Control Mapping. Mapping implementation must not pin or introduce a second standards baseline.
- **Interacts with project configuration:** PRD 051 may later expose validation policy, but `.forge.toml` must not override the compiled standards baseline in v1.2.
- **Interacts with GitHub Action:** CI/drift work should consume the reported schema baseline and provenance check rather than infer it from artifact metadata.
- **Independent of migration tooling:** PRD 053 stable-ID migration compares internal policy requirements and does not depend on OSCAL schema version.

## Risks & Mitigations 🟡 `@human-review`

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|------------|--------|------------|
| R-1 | v1.2.3 exposes existing AP/SSP structural defects | Medium | High | Gate current serializers against official schemas early; fix conformance without adding scope |
| R-2 | Bulk fixture refresh erases backward-compatibility evidence | High | High | Separate generated and legacy fixtures; forbid blanket replacement of `1.2.0` strings |
| R-3 | Validation output changes break JSON consumers | Medium | Medium | Add fields additively; retain existing keys; version fixtures and document the contract |
| R-4 | A v1.2.0 file relied on a corrected schema defect | Low | Medium | Report declared/baseline versions and exact rule; document limited compatibility exception |
| R-5 | oscal-cli's older embedded model creates false failures | High | Medium | Treat it as advisory interoperability evidence; keep embedded schemas authoritative |
| R-6 | Profile runtime behavior is assumed validated because tests pass | Medium | Medium | Document that Profile generation has compatibility tests but no new runtime validation contract |
| R-7 | Version comparison is implemented lexically | Medium | High | Parse numeric semantic components and test `1.2.3`, `1.2.10`, and prerelease cases |
| R-8 | Future upgrades repeat manual provenance drift | Medium | High | Machine-readable manifest, checksum CI, automated staging/diff procedure |

## Success Metrics — Hypotheses 🔴 `@human-required`

### Leading indicators

| Hypothesis | Success threshold | Stretch | Measurement |
|------------|-------------------|---------|-------------|
| H-1: Current artifacts are conformant | 100% of required model/format fixtures pass v1.2.3 gates | 100% plus NIST examples | CI compatibility matrix |
| H-2: Legacy users avoid forced migration | 100% of retained valid v1.2.0 fixtures pass supported existing paths | Add sanitized design-partner artifacts | Legacy fixture and pilot suite |
| H-3: Provenance is independently verifiable | 100% of vendored assets match manifest and official digests | Automated release-API cross-check | CI checksum job |
| H-4: Version claims are unambiguous | 100% of validation/round-trip snapshots contain declared and actual baseline fields | Practitioner comprehension ≥90% | Contract tests and five-person review task |
| H-5: Offline behavior is preserved | Zero network attempts in required runtime/compatibility tests | Network-denied cross-platform CI | Sandboxed CI observation |

### Lagging indicators

| Hypothesis | Success threshold | Evaluation window | Measurement |
|------------|-------------------|-------------------|-------------|
| H-6: Upgrade reduces interoperability failures | Zero confirmed issues caused by stale v1.2.0 schemas | 90 days after release | Issue tracker and design-partner feedback |
| H-7: Existing repositories upgrade safely | At least 3 external repositories validate/export existing v1.2.0 artifacts successfully | 60 days | Opt-in pilot verification; no telemetry required |
| H-8: Future baseline work is cheaper | Next patch evaluation reaches a complete compatibility report in ≤2 maintainer hours | Next OSCAL release | Maintainer time log and checklist completion |

### Technical quality gates

- 100% pass for generated Catalog, Component Definition, and Profile JSON/XML/YAML fixtures.
- 100% pass for generated Assessment Plan and SSP JSON fixtures.
- 100% pass for retained compatible v1.2.0 fixtures on existing paths.
- Zero unverified schema bytes and zero remote schema references.
- Zero unresolved FORGE-caused oscal-cli divergences in supported Catalog/Component checks when the tool is available.
- `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check` pass.

## Rollout & Phasing 🟡 `@human-review`

### Phase 0 — Baseline inventory and contract lock

- Approve the supported-version policy and validation-result fields.
- Record current schema provenance gaps and lock legacy v1.2.0 fixtures before any bulk changes.
- Verify the official v1.2.3 asset allowlist and digests.

### Phase 1 — Schema and metadata baseline

- Vendor pristine runtime/test schemas and provenance manifest.
- Update the shared generated-output constant and focused unit expectations.
- Add declared-version versus schema-used reporting and supported-range enforcement.

### Phase 2 — Model and format compatibility

- Run/fix Catalog, Component Definition, and Profile JSON/XML/YAML gates.
- Run/fix Assessment Plan and SSP JSON compatibility gates.
- Refresh only generated-current golden files; prove immutable legacy fixture behavior.

### Phase 3 — Ecosystem and release gate

- Run internal format pairs and optional version-aware oscal-cli round trips.
- Complete documentation, upgrade procedure, cross-platform CI, and release notes.
- Block the release and PRD 055 until all Must Have acceptance criteria pass.

### Rollback

If a release-blocking incompatibility cannot be corrected without broad product changes, do not publish a partially upgraded binary. Revert the entire baseline change as one atomic release item, retain the compatibility findings, and rescope the model defect explicitly. Never ship v1.2.3 metadata with v1.2.0 schemas or vice versa.

## Open Questions 🟡 `@human-review`

- **[Engineering, blocking before implementation]** Should the provenance manifest be JSON or TOML, and which existing CI job should own digest verification?
- **[Product + Engineering, non-blocking]** Should `forge --version` expose the OSCAL baseline now, or is validation/round-trip reporting plus documentation sufficient for v1.2?
- **[Engineering, non-blocking]** Should AP/SSP compatibility schemas live beside runtime schemas with a `role` field or under a clearly test-only fixture directory?
- **[Product, non-blocking]** After this release, should valid v1.2.0 remain supported for the entire v1.x line or for a documented minimum number of FORGE releases?
- **[Engineering + NIST ecosystem, non-blocking]** Which future oscal-cli release first claims OSCAL v1.2.3 support, and when should that version become a required rather than advisory CI lane?

## Decision Log 🟡 `@human-review`

| Date | Decision | Rationale | Alternatives considered |
|------|----------|-----------|-------------------------|
| 2026-08-22 | Treat v1.2.3 as a release compatibility gate, not a string-only update | Existing schemas, XSDs, fixtures, serializers, and external integrations can drift independently | Change only `OSCAL_VERSION`; defer schema update |
| 2026-08-22 | Emit v1.2.3 while accepting v1.2.0–v1.2.3 input | Preserves current users across NIST's backward-compatible patch line while moving generated artifacts forward | Reject all old input; keep emitting v1.2.0 |
| 2026-08-22 | Validate supported patch inputs against one pinned v1.2.3 schema | Maintains a small offline binary and a clear current conformance target | Embed a schema per patch; download on demand |
| 2026-08-22 | Preserve imported `oscal-version` during export | Format conversion is not a conformance migration and must not falsify document metadata | Rewrite all output to 1.2.3 |
| 2026-08-22 | Report declared and actual schema versions separately | NIST describes `oscal-version` as document conformance metadata/a schema hint, while FORGE selects by root model and pinned baseline | Treat metadata as proof of schema used |
| 2026-08-22 | Compatibility-test AP/SSP without expanding public commands | They are existing generated outputs that must remain valid, but new model surface would exceed a baseline upgrade | Ignore AP/SSP; add full validate/export support |
| 2026-08-22 | Keep oscal-cli evidence advisory when its model baseline is older | Successful conversion by an older model cannot prove v1.2.3 schema conformance | Call oscal-cli authoritative; remove external checks |
| 2026-08-22 | Block Control Mapping PRD 055 on this gate | New model support must use the same verified current baseline, not introduce another schema lineage | Implement mapping against a separate schema copy |

## Definition of Ready 🔴 `@human-required`

### Readiness Checklist

- [ ] Supported v1.2.0–v1.2.3 input policy approved by Product and Engineering
- [ ] Unsupported-version exit/reporting behavior approved
- [ ] Provenance manifest format and owner selected
- [ ] Legacy fixtures identified and protected from generated-fixture refresh
- [ ] AP/SSP compatibility-only boundary accepted
- [ ] oscal-cli advisory compatibility language reviewed
- [ ] Supply-chain and offline-validation controls reviewed
- [ ] All Must Have requirements map to testable acceptance criteria

### Sign-off

| Role | Name | Date | Decision |
|------|------|------|----------|
| Product Owner | Brian Luby | YYYY-MM-DD | [Ready / Not Ready] |

## Changelog ⚪ `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-08-22 | Codex | Initial draft from FORGE v1.2 roadmap priority 4, grounded in current validation, serialization, export, round-trip, AP/SSP, fixture, and official OSCAL v1.2.3 release behavior |
