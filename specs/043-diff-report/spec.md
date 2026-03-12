# Feature Specification: OSCAL Diff Report

**Feature Branch**: `043-diff-report`
**Created**: 2026-03-12
**Status**: Draft
**Input**: WI-43 — Derived from FORGE Product Roadmap, Sprint S-43, Theme T-6: Ecosystem & Community

## Clarifications

### Session 2026-03-12

- Q: When a control has both a UUID change and field-level content changes simultaneously, how should it be classified? → A: Classified as `Changed` with a UUID stability flag on the same entry — content diff and UUID change both surface in one entry; no separate `UuidChanged` entry is emitted for that control-id.
- Q: Should Catalog control extraction recurse into nested groups (arbitrary depth) or only traverse one level deep (`catalog.groups[].controls[]`)? → A: Recursive — traverse `groups[]` at any depth, collecting all `controls[]` found at every level.
- Q: What exit code should `forge diff` return when differences are found? → A: Follow `diff(1)` convention — `0` = no differences, `1` = differences found, `2` = error (non-OSCAL input, missing file, type mismatch).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Compare Two Conversion Outputs (Priority: P1)

A compliance engineer has re-converted an updated version of a security policy through FORGE and needs to understand what changed in the OSCAL output compared to the previous conversion. They run `forge diff old-catalog.json new-catalog.json` and receive a clear, categorized report showing which controls were added, removed, or changed — along with the specific fields that changed within each modified control.

**Why this priority**: This is the core function of the feature. Without the ability to compare two OSCAL outputs and categorize changes by control-id, the diff command has no value. All other stories build on this foundation.

**Independent Test**: Convert two versions of a policy (one with 10 controls, one with 12 controls where 2 are new and 1 existing control is modified), run `forge diff`, and verify the report shows 2 added, 0 removed, 1 changed with field-level detail.

**Acceptance Scenarios**:

1. **Given** two Catalog JSON files where the new version has 2 additional controls, **When** running `forge diff old.json new.json`, **Then** the report lists 2 controls as "added" with their control-ids and a summary count at the top.
2. **Given** two Catalog JSON files where one control's description has changed, **When** running `forge diff old.json new.json`, **Then** the report lists that control as "changed" and shows the old and new description.
3. **Given** two Catalog JSON files where the old version has a control absent from the new version, **When** running `forge diff old.json new.json`, **Then** the report lists that control as "removed" with its control-id.
4. **Given** two identical Catalog JSON files, **When** running `forge diff old.json new.json`, **Then** the report prints a summary indicating zero differences found.

---

### User Story 2 - Detect UUID Stability Changes (Priority: P1)

A compliance engineer needs to know when a control's unique identifier has changed between two conversion runs — even though the control-id (e.g., "POL-AC-001") stayed the same. UUID changes indicate that a control's content was substantively modified, which can break downstream tools (SSP imports, Assessment Plans) that reference controls by UUID.

**Why this priority**: UUID stability is critical for downstream tool integration. Silent UUID changes cause broken references in dependent artifacts. Making these changes visible prevents integration failures that would otherwise require manual auditing.

**Independent Test**: Convert two versions of a policy where one control's text has been substantively modified (changing its deterministic UUID), run `forge diff`, and verify the report explicitly flags the UUID change for that control-id with both old and new UUID values.

**Acceptance Scenarios**:

1. **Given** two Catalog JSON files where control "POL-AC-001" appears in both but has a different UUID, **When** running `forge diff`, **Then** the report highlights "POL-AC-001" as having a UUID stability change and shows both old and new UUID values.
2. **Given** two Catalog JSON files where all control UUIDs are identical, **When** running `forge diff`, **Then** no UUID stability changes are reported.

---

### User Story 3 - Diff Component Definition Outputs (Priority: P2)

A compliance engineer compares two Component Definition outputs from different versions of the same policy to see how implemented-requirements and control-implementations have changed.

**Why this priority**: Component Definitions are the primary FORGE output for the component-first strategy. Providing diff support for this artifact type makes the feature complete for the two most common output formats.

**Independent Test**: Convert two versions of a policy using the component strategy, run `forge diff` on the two Component Definition JSON files, and verify the report shows changes in implemented-requirements.

**Acceptance Scenarios**:

1. **Given** two Component Definition JSON files with different implemented-requirement counts, **When** running `forge diff`, **Then** the report shows added and removed implemented-requirements with their control-ids.
2. **Given** two Component Definition JSON files where an implementation narrative changed, **When** running `forge diff`, **Then** the report shows the specific narrative field that changed with old and new text.

---

### Edge Cases

- When both files are identical: report shows "No differences found" with zero counts in the summary — not an error.
- When the old file has zero controls: all controls in the new file are reported as "added."
- When the new file has zero controls: all controls in the old file are reported as "removed."
- When both files are valid JSON but are different artifact types (one Catalog, one Component Definition): a descriptive error is produced explaining the type mismatch — not a crash.
- When a file path does not exist: a descriptive error identifies the missing file.
- When a file is valid JSON but not an OSCAL artifact (missing expected root keys): a descriptive error indicates the file is not a recognized OSCAL artifact.
- When only a control's title changed but description stayed the same: only the title field change is reported for that control.
- When many controls change simultaneously (e.g., bulk policy reorganization): the summary counts give an at-a-glance overview; all changes appear in the detailed section.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to invoke `forge diff <old-artifact> <new-artifact>` with two OSCAL artifact file paths as arguments.
- **FR-002**: The diff MUST identify and report controls present in the new artifact but absent from the old, classified as "added," matched by control-id.
- **FR-003**: The diff MUST identify and report controls present in the old artifact but absent from the new, classified as "removed," matched by control-id.
- **FR-004**: The diff MUST identify and report controls that exist in both artifacts (same control-id) but have different content (title, description, or statement prose), classified as "changed," showing the specific fields and their old and new values.
- **FR-005**: The diff MUST detect and explicitly report when a control's unique identifier (UUID) differs between the two artifacts for the same control-id, flagging this as a UUID stability change with both old and new values. When UUID change co-occurs with field-level content changes, the entry is classified as `Changed` with a UUID stability flag — not as a separate `UuidChanged` entry. A standalone `UuidChanged` entry is only emitted when the UUID differs but no diffable field values changed.
- **FR-006**: The diff report MUST be printed to stdout in a human-readable format with a summary section at the top showing counts: total controls in old, total in new, added, removed, changed, unchanged, and UUID changes.
- **FR-007**: The diff MUST support Catalog artifacts, extracting controls by recursively traversing `groups[]` at any depth and collecting all `controls[]` found at every level — not limited to one level deep.
- **FR-008**: The diff MUST produce a descriptive, non-crashing error when input files are missing, contain invalid data, or are different artifact types from each other.
- **FR-009**: The diff SHOULD support Component Definition artifacts, extracting implemented-requirements from control-implementations. *(Matches PRD S-1: Should Have — high value, not a launch blocker. Phase 5 tasks are optional for MVP.)*
- **FR-010**: The diff output MUST be sorted by control-id for consistent, reproducible ordering across runs.
- **FR-011**: `forge diff` MUST exit with code `0` when no differences are found, code `1` when differences are found, and code `2` on any error condition (missing file, invalid JSON, non-OSCAL artifact, type mismatch) — following the `diff(1)` convention to enable CI pipeline integration.

### Key Entities

- **DiffReport**: The complete result of comparing two artifacts — includes file paths, detected artifact type, a summary of counts (added, removed, changed, unchanged, UUID changes), and a list of categorized diff entries sorted by control-id.
- **DiffEntry**: A single comparison result for one control-id — categorized as Added, Removed, Changed, or UUID-Changed, carrying the control-id, relevant UUIDs, and (for Changed entries) a list of field-level differences. When a control has both field-level content changes and a UUID change simultaneously, it is classified as `Changed` (with a UUID stability flag on the entry) — not as `UuidChanged`. A `UuidChanged` entry is only emitted when the UUID differs but no diffable field values changed.
- **FieldChange**: A single field-level difference within a changed control — identifies the field name (e.g., title, description, statement prose) and its old and new values.
- **ControlSnapshot**: The set of diffable fields captured from a single control during extraction — control-id, UUID, title (Catalog only), description (Component Definition only), and statement prose (Catalog only) — used as the unit of comparison. Each field carries a stable human-readable label used in `FieldChange.field_name`: `"title"`, `"description"`, or `"statement[N]"`. The `description` field holds the implemented-requirement narrative for Component Definition artifacts; it is `None` for Catalog artifacts (which use `parts_prose` for statement text instead).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of added, removed, and changed controls are correctly identified in all test scenarios — zero false positives and zero false negatives.
- **SC-002**: 100% of UUID stability changes (same control-id, different UUID) are detected and explicitly flagged in the report.
- **SC-003**: The diff report stdout includes a summary section with counts (added, removed, changed, unchanged, UUID changes), at least one labeled detail section (Added / Changed / Removed / UUID Stability), and field-level old→new values for all Changed entries — a compliance engineer can determine what changed between policy versions without consulting any other file.
- **SC-004**: Invalid or mismatched inputs always produce a descriptive error message — zero panics or crashes on bad input across all defined error scenarios.
- **SC-005**: Diff output is deterministically ordered — running `forge diff` twice on the same inputs produces byte-for-byte identical stdout output.
- **SC-006**: Exit codes follow `diff(1)` convention — `forge diff` exits `0` (no differences), `1` (differences found), or `2` (error), enabling scriptable and CI pipeline use without stdout parsing.

## Assumptions

- Both input files are valid OSCAL JSON produced by FORGE (Catalog or Component Definition). Arbitrary third-party OSCAL files may work but are not the primary use case.
- Control-id is the stable, human-assigned identifier that persists across re-conversions even when UUIDs change due to content modifications — it is the sole matching key.
- The diff operates on the final OSCAL JSON output, not on intermediate representations or the source policy document.
- Users will provide two files of the same OSCAL artifact type (both Catalogs or both Component Definitions) — cross-type diffing produces an error, not a best-effort result.
- FORGE-generated OSCAL artifacts are small to moderately sized (KB to low MB range) — memory consumption from loading two files simultaneously is acceptable on typical developer hardware.
- Diff report output sensitivity: reports revealing policy changes should be treated with the same sensitivity as the source OSCAL artifacts themselves; this is documented guidance, not enforced by the tool.

## Dependencies

- Requires WI-35 (Phase 2 integration testing) to be complete — ensures Catalog and Component Definition pipelines produce stable output suitable for diffing.
- No new external libraries or services are introduced; all comparison logic uses standard library data structures.
