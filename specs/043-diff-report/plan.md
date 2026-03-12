# Implementation Plan: OSCAL Diff Report

**Branch**: `043-diff-report` | **Date**: 2026-03-12 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/043-diff-report/spec.md`

## Summary

Implement a `forge diff <old-artifact> <new-artifact>` subcommand that loads two OSCAL JSON artifacts (Catalog or Component Definition), extracts controls into HashMaps keyed by control-id, performs a set-based comparison (added/removed/changed/unchanged), detects UUID stability changes, and prints a human-readable diff report to stdout. Exit code follows `diff(1)` convention: `0` = no differences, `1` = differences found, `2` = error.

## Technical Context

**Language/Version**: Rust 1.93.0 (Edition 2024)
**Primary Dependencies**: `serde_json` (existing), `std::collections::HashMap` (stdlib), `clap 4.x` (existing), `thiserror 2.0.18` (existing), `tracing 0.1.44` (existing)
**Storage**: N/A — reads two local JSON files into memory; no writes
**Testing**: `cargo test` (unit tests); `insta` for snapshot tests (existing)
**Target Platform**: macOS/Linux CLI
**Project Type**: Single Rust binary crate
**Performance Goals**: Sub-second for KB–low MB OSCAL artifacts; acceptable for typical FORGE output sizes
**Constraints**: No new crate dependencies; all diff logic uses stdlib + existing deps
**Scale/Scope**: KB–low MB OSCAL files; single-pass in-memory comparison

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Crate-First Architecture | ✅ Pass | New `src/diff/` module within existing crate; no new crate |
| II. Rust-First Implementation | ✅ Pass | Rust stable only; no unsafe code |
| III. Contract-First Development | ✅ Pass | Interface contract defined in AR; formalized in `contracts/diff.rs` |
| IV. Test-First Development | ✅ Pass | TDD mandatory per constitution; unit tests for all categories |
| V. Complete Requirement Delivery | ✅ Pass | All M requirements (M-1 through M-8) + FR-011 covered |
| VI. Performance/Scope Discipline | ✅ Pass | No benchmarks in scope; size assumption documented in Assumptions |
| VII. Security-First Design | ✅ Pass | SEC-2 through SEC-7 mapped to tasks; no new attack surface |
| VIII. Error Handling Standards | ✅ Pass | ForgeError variants with descriptive messages; stable substring tests |
| IX. Observability | ✅ Pass | tracing INFO (file paths/type) + DEBUG (control counts) |
| X. Simplicity | ✅ Pass | HashMap set comparison; no new frameworks |
| XI. Dependency Policy | ✅ Pass | Zero new dependencies |

**No violations. Proceeding.**

## Project Structure

### Documentation (this feature)

```text
specs/043-diff-report/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── diff.rs          # Rust interface contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── diff/
│   ├── mod.rs           # Public re-exports; pub mod declarations
│   ├── types.rs         # ControlSnapshot, DiffEntry, DiffSummary, DiffReport, ArtifactType
│   ├── extractor.rs     # extract_controls(): Catalog + ComponentDef traversal
│   ├── engine.rs        # compare_controls(): set-based diff logic + UUID detection
│   └── formatter.rs     # format_diff_report(): human-readable text output
├── cli/
│   ├── mod.rs           # Add Commands::Diff variant + dispatch
│   └── diff.rs          # execute(old: &Path, new: &Path) → Result<bool, ForgeError>
├── error.rs             # Add ForgeError::DiffHasChanges + ForgeError::DiffError
├── lib.rs               # Add pub mod diff
└── main.rs              # Handle ForgeError::DiffHasChanges (exit 1, no message)

tests/
└── (unit tests colocated in each src/diff/*.rs module via #[cfg(test)])
```

**Structure Decision**: Single-project (Option 1). New `src/diff/` module mirrors the pattern of `src/trace/` (extractor, formatter, report) and `src/batch/` (multi-file submodule). CLI handler in `src/cli/diff.rs` follows the `src/cli/trace.rs` pattern.

## Complexity Tracking

No constitution violations requiring justification.

## Phase 0: Research

See [`research.md`](research.md) for full findings. Key decisions:

| Decision | Outcome |
|----------|---------|
| UUID availability in Catalog JSON | `OscalControl.uuid` has `skip_serializing` — not present in FORGE Catalog output. The extractor reads only `control["uuid"]`; when that field is omitted (the common case for FORGE Catalogs), UUID stability cannot be recovered and is treated as empty/best-effort. Tests use Component Definitions or manually-crafted Catalog fixtures with explicit UUIDs for UUID stability scenarios. |
| Exit code scheme | `0` = no diff, `1` = has changes (new `ForgeError::DiffHasChanges` sentinel, silent in main.rs), `2` = all diff errors (`ForgeError::DiffError` → exit_code 2) |
| Catalog traversal depth | Recursive over nested `groups[]` at any depth (from spec clarification) |
| Co-occurring UUID+field changes | Classified as `Changed` with UUID stability flag (from spec clarification) |

## Phase 1: Design

### Data Model

See [`data-model.md`](data-model.md) for full entity definitions.

Core types in `src/diff/types.rs`:

```rust
pub enum ArtifactType { Catalog, ComponentDefinition }

pub struct ControlSnapshot {
    pub control_id: String,
    pub uuid: String,               // empty for Catalog (skip_serializing); populated for ComponentDef
    pub title: Option<String>,      // Catalog only (control["title"]); None for ComponentDef
    pub description: Option<String>, // ComponentDef only (ir["description"]); None for Catalog
    pub parts_prose: Vec<String>,   // Catalog only (statement prose); empty vec for ComponentDef
}

pub enum DiffEntry {
    Added  { control_id: String, new_uuid: String },
    Removed { control_id: String, old_uuid: String },
    Changed {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
        uuid_changed: bool,       // true when UUID differs AND fields also changed
        field_changes: Vec<FieldChange>,
    },
    UuidChanged { control_id: String, old_uuid: String, new_uuid: String },
}
```

**Key design decision**: `DiffEntry::Changed` carries a `uuid_changed: bool` flag. When both UUID and field values differ, the entry is `Changed { uuid_changed: true, field_changes: [...] }` — NOT a separate `UuidChanged` entry. A standalone `UuidChanged` entry is only emitted when UUID differs but all diffable fields are identical.

### Interface Contract

See [`contracts/diff.rs`](contracts/diff.rs) for full Rust interface.

Public API surface:
```rust
// src/diff/mod.rs
pub fn diff_artifacts(old_path: &Path, new_path: &Path) -> Result<DiffReport, ForgeError>;
pub fn format_diff_report(report: &DiffReport) -> String;
```

### CLI Integration

New `Commands::Diff` variant in `src/cli/mod.rs`:
```
forge diff <old-artifact> <new-artifact>
```
Two positional `PathBuf` arguments. No optional flags for MVP (C-1 `--format json`, C-2 `--summary-only` are Could-Have).

### Error Strategy

New `ForgeError` variants:
- `DiffError(String)` — maps to exit code `2`; wraps all diff-specific errors (file not found, invalid JSON, non-OSCAL input, type mismatch)
- `DiffHasChanges` — maps to exit code `1`; silent sentinel (no error message printed); signals "differences found"

`main.rs` special-cases `DiffHasChanges`:
```rust
Err(ForgeError::DiffHasChanges) => ExitCode::from(1u8),   // no eprintln
```

### Stdout Report Format

```
OSCAL Diff Report
=================
Old: old-catalog.json  (Catalog)
New: new-catalog.json  (Catalog)

Summary
-------
Controls (old): 10  |  Controls (new): 12
Added: 2  |  Removed: 0  |  Changed: 1  |  Unchanged: 9  |  UUID changes: 0

No differences found.      ← when zero changes
─── OR ───
Added (2)
─────────
  + POL-AC-011  [uuid: abc123]
  + POL-AC-012  [uuid: def456]

Changed (1)
───────────
  ~ POL-IA-002
      title:       "Old title"  →  "New title"
      description: "Old desc"   →  "New desc"

Removed (0)
───────────
  (none)

UUID Stability Changes (0)
──────────────────────────
  (none)
```

### Catalog Extraction Algorithm

```
fn collect_controls_from_groups(groups: &[Value]) -> HashMap<String, ControlSnapshot>:
  for each group:
    for each control in group["controls"]:
      let id = control["id"].as_str()
      let uuid = control["uuid"].as_str().unwrap_or("")  // empty for FORGE Catalog output
      let title = control["title"].as_str()
      let parts_prose = collect_parts_prose(&control["parts"])
      insert (id → ControlSnapshot { ... })
    recurse into group["groups"] (nested groups)  ← FR-007 clarification
```

### Component Definition Extraction

```
fn collect_controls_from_component_def(root: &Value) -> HashMap<String, ControlSnapshot>:
  for each component in root["component-definition"]["components"]:
    for each ci in component["control-implementations"]:
      for each ir in ci["implemented-requirements"]:
        let id = ir["control-id"].as_str()
        let uuid = ir["uuid"].as_str().unwrap_or("")
        let description = Some(ir["description"].as_str().unwrap_or(""))  // → ControlSnapshot.description
        // title = None; parts_prose = vec![]                              // unused for ComponentDef
        insert (id → ControlSnapshot { control_id: id, uuid, title: None, description, parts_prose: vec![] })
```

### Re-evaluation Constitution Check (Post-Design)

All principles still pass. No new complexity introduced beyond what was planned.

## Implementation Order

1. `src/diff/types.rs` — data types (no deps, pure data)
2. `src/diff/extractor.rs` — extract_controls (uses serde_json Value)
3. `src/diff/engine.rs` — compare_controls (pure logic)
4. `src/diff/formatter.rs` — format_diff_report (pure string building)
5. `src/diff/mod.rs` — wire up + re-export `diff_artifacts`
6. `src/error.rs` — add `DiffHasChanges` + `DiffError` variants
7. `src/lib.rs` — add `pub mod diff`
8. `src/cli/diff.rs` — CLI handler
9. `src/cli/mod.rs` — add `Commands::Diff` + dispatch
10. `src/main.rs` — handle `DiffHasChanges` exit code

Tests are written first (RED) for each module before implementation (GREEN).

## Requirement Coverage

| FR / SC / SEC ID | Covered By | Module |
|------------------|------------|--------|
| FR-001 | `Commands::Diff` in cli/mod.rs | cli |
| FR-002 | `compare_controls` added detection | diff/engine.rs |
| FR-003 | `compare_controls` removed detection | diff/engine.rs |
| FR-004 | `compare_controls` field comparison (title/description/parts_prose) | diff/engine.rs |
| FR-005 | UUID comparison in `compare_controls`; co-occurrence rule | diff/engine.rs |
| FR-006 | `format_diff_report` stdout output | diff/formatter.rs |
| FR-007 | `collect_controls_from_groups` recursive | diff/extractor.rs |
| FR-008 | `ForgeError::DiffError` + validate in diff_artifacts | diff/mod.rs |
| FR-009 | `collect_controls_from_component_def` | diff/extractor.rs |
| FR-010 | Sort entries by control-id in `compare_controls` | diff/engine.rs |
| FR-011 | `DiffHasChanges` sentinel + main.rs handling | error.rs, main.rs |
| SC-001 | Unit tests: AC-2, AC-3, AC-4 | diff/engine.rs tests |
| SC-002 | Unit tests: AC-5 | diff/engine.rs tests |
| SC-003 | Manual review of format output | diff/formatter.rs |
| SC-004 | Unit tests: AC-7, EC-4, EC-5 | diff/mod.rs tests |
| SC-005 | Deterministic: sorted entries | diff/engine.rs |
| SC-006 | DiffHasChanges sentinel | error.rs, main.rs |
| SEC-2 | File existence check in diff_artifacts | diff/mod.rs |
| SEC-3 | serde_json parse error → DiffError | diff/mod.rs |
| SEC-4 | Root key check → DiffError | diff/mod.rs |
| SEC-5 | Type mismatch check → DiffError | diff/mod.rs |
| SEC-6 | All errors → DiffError (no panics) | diff/mod.rs |
| SEC-7 | Sort by control-id | diff/engine.rs |

## Acceptance Criteria Coverage

| AC ID | Test Name | Location |
|-------|-----------|----------|
| AC-1 (M-1) | `test_diff_produces_report` | diff/mod.rs tests |
| AC-2 (M-2) | `test_added_controls_detected` | diff/engine.rs tests |
| AC-3 (M-3) | `test_removed_controls_detected` | diff/engine.rs tests |
| AC-4 (M-4) | `test_changed_controls_with_field_detail` | diff/engine.rs tests |
| AC-5 (M-5) | `test_uuid_stability_change_detected` | diff/engine.rs tests |
| AC-6 (M-6) | `test_format_output_contains_summary` | diff/formatter.rs tests |
| AC-7 (M-8) | `test_invalid_json_returns_error`, `test_non_oscal_json_returns_error` | diff/mod.rs tests |
| AC-8 (S-2) | `test_summary_section_in_output` | diff/formatter.rs tests |
| EC-1 | `test_identical_files_no_differences` | diff/engine.rs tests |
| EC-2 | `test_empty_old_all_added` | diff/engine.rs tests |
| EC-3 | `test_empty_new_all_removed` | diff/engine.rs tests |
| EC-4 | `test_type_mismatch_error` | diff/mod.rs tests |
| EC-5 | `test_missing_file_error` | diff/mod.rs tests |
| EC-6 | `test_title_only_change_reported` | diff/engine.rs tests |
| EC-7 | `test_same_uuid_different_content_is_changed` | diff/engine.rs tests |
