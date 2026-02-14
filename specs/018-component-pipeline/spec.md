# Feature Specification: End-to-End Component Definition Pipeline

**Feature Branch**: `018-component-pipeline`
**Created**: 2026-02-13
**Status**: Draft
**Input**: Derived from PRD `docs/PRD/018-prd-component-pipeline.md` (WI-18)

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Convert Markdown Policy to OSCAL Component Definition via CLI (Priority: P1)

A compliance engineer converts a Markdown security policy into an OSCAL Component Definition mapped to a baseline control framework. This is the MS-3 milestone exit criteria: the component-first conversion strategy becomes accessible through a single CLI command.

> As a compliance engineer, I want to run `forge convert policy.md --strategy component --source-profile baseline.json --format json` so that I receive a valid OSCAL Component Definition with my policy requirements mapped as implemented-requirements against baseline control IDs.

**Why this priority**: This is the primary deliverable for MS-3 and directly fulfills Parent PRD requirements M-4 and M-7. The component-first strategy is a P1 capability per Parent PRD US-2.

**Independent Test**: Run the convert command with a sample Markdown policy and a baseline profile path, and verify the output is a complete OSCAL Component Definition JSON with a documentary component, implemented-requirements referencing control IDs, traceability props, and back matter.

**Acceptance Scenarios**:

1. **Given** a Markdown policy document and a baseline profile path, **When** running `forge convert policy.md --strategy component --source-profile baseline.json --format json`, **Then** a complete OSCAL Component Definition JSON is written to stdout containing a documentary component with control-implementations referencing the baseline. *(AC-1: M-1, M-3)*
2. **Given** a policy with 5 requirements mapped to 3 control IDs, **When** converting with `--strategy component`, **Then** the Component Definition contains 5 implemented-requirements, each referencing the correct control-id and containing the policy-derived narrative. *(AC-2: M-4)*
3. **Given** any component pipeline execution, **When** inspecting the output metadata, **Then** all required fields are present: `uuid`, `title`, `last-modified`, `version`, `oscal-version` set to `"1.2.0"`. *(AC-3: M-5)*
4. **Given** a policy with citations, **When** converting with `--strategy component`, **Then** citations appear in `back-matter.resources` with `link` elements in the body referencing back matter resource UUIDs. *(AC-5: M-7)*
5. **Given** the `--output report.json` flag, **When** converting, **Then** the Component Definition JSON is written to `report.json` instead of stdout. *(AC-6: M-8)*

---

### User Story 2 — Traceability in Component Definition Output (Priority: P1)

A compliance engineer verifies that every implemented-requirement in the Component Definition traces back to the source policy location. Traceability is non-negotiable per product principle P-2 and Parent PRD requirement M-10.

> As a compliance engineer, I want the generated Component Definition to contain traceability metadata so that I can audit which policy section and line produced each implemented-requirement.

**Why this priority**: Traceability is a launch-blocking requirement (Parent PRD M-10). The end-to-end pipeline must preserve trace links through all stages.

**Independent Test**: Inspect the generated Component Definition JSON and verify each implemented-requirement contains trace props linking back to source file, section, and line number.

**Acceptance Scenarios**:

1. **Given** a generated Component Definition, **When** inspecting any implemented-requirement, **Then** it contains `prop` annotations with source file path, section title, and source line number. *(AC-4: M-6)*
2. **Given** a generated Component Definition, **When** inspecting the traceability metadata, **Then** every implemented-requirement has a bidirectional trace link to its source PolicyRequirement.

---

### User Story 3 — Component Strategy Without Source Profile (Priority: P2)

A user runs the component strategy without specifying a source profile to get a Component Definition with unmapped requirements. Not all users will have a baseline profile available immediately; the tool should still produce useful output.

> As a compliance engineer, I want to run `forge convert policy.md --strategy component --format json` without a `--source-profile` flag so that I get a Component Definition with requirements that I can manually map to controls later.

**Why this priority**: Supports exploratory workflows where users generate an unmapped Component Definition first and add control-id mappings later.

**Independent Test**: Run `forge convert policy.md --strategy component --format json` without `--source-profile` and verify a Component Definition is produced with an empty `control-implementations` array and a warning is emitted to stderr.

**Acceptance Scenarios**:

1. **Given** no `--source-profile` flag, **When** running `forge convert policy.md --strategy component --format json`, **Then** a valid Component Definition is produced with an empty `control-implementations` array (OSCAL control-implementations require a `source` reference, which cannot be constructed without a profile). *(AC-7: M-2, S-1)*
2. **Given** no `--source-profile` flag, **When** the output is generated, **Then** a warning is emitted to stderr indicating that control-id mapping was skipped due to missing source profile. *(AC-7: S-1)*

---

### User Story 4 — Source Profile Validation (Priority: P2)

A compliance engineer receives a clear error when specifying an invalid source profile path, preventing confusing failures mid-pipeline.

> As a compliance engineer, I want a descriptive error message when my `--source-profile` path is invalid so that I can correct it before rerunning the conversion.

**Why this priority**: Error feedback is essential for usability but does not block the core conversion flow.

**Independent Test**: Run the convert command with a non-existent `--source-profile` path and verify a descriptive error is printed with a non-zero exit code.

**Acceptance Scenarios**:

1. **Given** a non-existent `--source-profile` path, **When** running the convert command, **Then** a descriptive error is printed and the process exits with non-zero status. *(AC-8: S-2)*

---

### Edge Cases

- **EC-1** (M-1): When `--strategy component` is specified without `--format json`, the default format is JSON and a Component Definition is produced.
- **EC-2** (M-3): When the input Markdown has zero extractable requirements, a Component Definition is produced with an empty `control-implementations` array and a warning is emitted.
- **EC-3** (M-4): When the source profile contains no control IDs, implemented-requirements are generated without control-id references and a warning is emitted.
- **EC-4** (M-8): When `--output` points to a directory that does not exist, a descriptive filesystem error is printed and the process exits with non-zero status.
- **EC-5** (M-2): When `--source-profile` is provided with `--strategy catalog`, the flag is ignored (it is only meaningful for component strategy).
- **EC-6** (M-3): When the pipeline encounters an error mid-way (e.g., empty input document), a descriptive error is printed with the failing stage and file context, and the process exits with non-zero status. *(Profile JSON parsing errors deferred per W-3.)*

## Requirements *(mandatory)*

### Functional Requirements

#### Must Have (M) — MVP, launch blockers

- **M-1**: The `forge convert` command SHALL accept `--strategy component` to invoke the Component Definition generation pipeline. *(Traces to: Parent PRD M-4)*
- **M-2**: The `forge convert` command SHALL accept `--source-profile <path>` to specify the baseline catalog/profile for control-id mapping when using the component strategy. *(Traces to: Parent PRD M-4, US-2)*
- **M-3**: The component pipeline SHALL wire the full processing chain — ingest, parse, normalize, map, assemble, serialize — producing a complete OSCAL Component Definition JSON. *(Traces to: Parent PRD M-4, M-7)*
- **M-4**: The generated Component Definition SHALL include a documentary component with `type: "policy"`, containing `control-implementations` with `implemented-requirements` mapped to control IDs from the source profile. *(Traces to: Parent PRD M-4)*
- **M-5**: The generated Component Definition SHALL include all required OSCAL metadata fields: `uuid`, `title`, `last-modified`, `version`, `oscal-version`. *(Traces to: Parent PRD M-5)*
- **M-6**: The generated Component Definition SHALL include traceability metadata as `prop` and `link` annotations on implemented-requirements, linking each to its source policy section and line. *(Traces to: Parent PRD M-10, M-11)*
- **M-7**: The generated Component Definition SHALL include back matter `resources` for any extracted citations, with `link` elements in the body referencing back matter resource UUIDs. *(Traces to: Parent PRD M-9, M-11)*
- **M-8**: The output SHALL be valid JSON written to stdout by default, or to a file when `--output <path>` is specified. *(Traces to: Parent PRD M-7)*

#### Should Have (S) — High value, not blocking

- **S-1**: When `--source-profile` is omitted, the component pipeline SHALL produce a Component Definition with an empty `control-implementations` array and emit a warning to stderr. *(OSCAL control-implementations require a `source` reference.)*
- **S-2**: The pipeline SHALL validate that the `--source-profile` path exists and is a regular file before processing, and exit with a descriptive error if not. *(JSON content validation deferred per W-3.)*
- **S-3**: The `--verbose` flag SHALL print pipeline stage progress to stderr (e.g., "Ingesting...", "Building component...", "Serializing...").

#### Could Have (C) — Nice to have, if time permits

- **C-1**: The pipeline COULD print a summary to stderr after completion: number of requirements extracted, number of implemented-requirements generated, number of control IDs mapped.

#### Won't Have (W) — Explicitly deferred

- **W-1**: Schema validation of generated output — *Deferred to WI-19 (schema validation integration)*
- **W-2**: XML or YAML output — *Deferred to WI-26/WI-27 (Phase 2)*
- **W-3**: Profile resolution (resolving imports/merges in the source profile) — *Deferred to WI-36 (Phase 3, oscal-cli integration)*
- **W-4**: Auto-detection of strategy from input content — *Strategy is always explicit via `--strategy` flag*

### Key Entities

- **Component Definition**: The output OSCAL model — describes how controls are implemented by reusable components (in this case, documentary components of type "policy"), with metadata, back matter, and traceability annotations.
- **Documentary Component**: An OSCAL component of type "policy", "procedure", or "process" representing non-technical control implementations derived from the source policy document.
- **Source Profile**: A baseline catalog or profile referenced by `--source-profile` that provides control IDs for implemented-requirement mapping. Accepted as a JSON file path.
- **Implemented Requirement**: An OSCAL element within a Component Definition that maps a component's implementation narrative to a specific control-id, annotated with traceability props linking back to the source policy.
- **Control Implementation**: An OSCAL structure within a Component Definition that groups implemented-requirements under a source profile reference.
- **Pipeline**: The full processing chain: ingest → parse → normalize → map → trace → assemble → serialize. Reuses shared infrastructure from WI-13 (Catalog pipeline) with a strategy branch point after domain model construction.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A compliance engineer can convert a Markdown policy to an OSCAL Component Definition JSON using a single CLI command, producing complete output in one step.
- **SC-002**: The output document contains all expected structural elements: a `component-definition` root, metadata (uuid, title, version, last-modified, oscal-version), a documentary component, control-implementations, and implemented-requirements with correct control-id mappings.
- **SC-003**: 100% of implemented-requirements in the output contain traceability props (source-file, source-section, source-line) linking back to the originating policy content.
- **SC-004**: The automated smoke test passes consistently, verifying end-to-end pipeline integration from Markdown input to Component Definition JSON output.
- **SC-005**: The output can be written to a named file or printed to the terminal, with the terminal as the default for composability with other tools.
- **SC-006**: When `--source-profile` is omitted, the pipeline produces an unmapped Component Definition and emits a warning, enabling iterative workflows.
- **SC-007**: When invalid inputs are provided (missing file, bad profile path, empty document), the user receives a clear error message and the command exits with a failure indicator.

## Assumptions

- **A-1**: WI-14 (Component Definition structure) and WI-15 (implemented-requirements) are complete and produce correct OSCAL Component Definition JSON fragments.
- **A-2**: WI-16 (TraceLink model) and WI-17 (embedded trace metadata) are complete and can annotate Component Definition elements.
- **A-3**: The full ingest → parse → normalize pipeline from WI-1 through WI-8 is operational and produces a valid PolicyDocument.
- **A-4**: The `--strategy catalog` pipeline (WI-13) is operational and can serve as an architectural reference for the component pipeline wiring.
- **A-5**: The `--source-profile` flag accepts a file path to a JSON catalog or profile; parsing the referenced profile for control IDs is handled by the mapping logic from WI-15.
- **A-6**: JSON serialization capabilities are already available from prior work items (serde_json).

## Dependencies

- **Requires**: WI-14 (Component Definition structure), WI-15 (implemented-requirements with control-id mapping), WI-16 (TraceLink model), WI-17 (trace metadata embedding) — and transitively all of WI-1 through WI-12
- **Shared Infrastructure**: WI-11 (OSCAL Metadata), WI-12 (Back Matter), WI-13 (Catalog pipeline — architectural reference and shared pipeline stages)
- **Blocks**: WI-19 (Schema Validation — needs generated artifacts to validate)
