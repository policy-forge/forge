# Research: 043-diff-report

**Date**: 2026-03-12 | **Branch**: `043-diff-report`

## RES-1: UUID Availability in OSCAL Catalog JSON

### Question
`OscalControl.uuid` has `#[serde(skip_serializing, default)]` in `src/oscal/catalog.rs`. This means UUIDs are NOT present in FORGE-generated Catalog output JSON. Is UUID stability tracking feasible for Catalog artifacts?

### Findings
- **OSCAL spec**: The OSCAL 1.2.0 JSON schema for Catalog DOES allow `uuid` on controls. The `skip_serializing` annotation in FORGE is a FORGE implementation choice, not a schema requirement. The comment "OSCAL catalog schema does not allow uuid on controls" appears to be incorrect.
- **Deserialization**: `skip_serializing` (not `skip`) means the field will be populated if `uuid` is present in JSON during deserialization. So any Catalog JSON with explicit `uuid` fields on controls will work correctly.
- **FORGE Catalog output**: Since FORGE omits `uuid` during serialization, FORGE-generated Catalog outputs will have empty UUIDs when loaded by the diff engine.
- **Impact on UUID stability tracking**: UUID stability tracking (FR-005) will be a no-op for FORGE-generated Catalog outputs (both old and new will have empty UUIDs → no UUID change detected). It WILL work correctly for:
  - Component Definition outputs (uuid IS serialized on `implemented-requirement`)
  - Any Catalog files with explicit `uuid` fields (e.g., hand-crafted or third-party OSCAL)

### Decision
**Accept current behavior.** UUID stability tracking operates on whatever UUID value is present in the JSON. For FORGE Catalog outputs this means UUID stability is not tracked (empty UUID in both inputs = no change). For Component Definitions and UUID-enriched Catalogs, full UUID stability tracking works. Test fixtures for UUID stability scenarios will use Component Definition artifacts (or manually-crafted Catalog JSON with explicit UUIDs).

**Future consideration**: If FORGE should track UUID stability for Catalog outputs, a separate work item should remove `skip_serializing` from `OscalControl.uuid` (verifying OSCAL schema allows it) and emit UUIDs in Catalog JSON output. This is out of scope for WI-43.

**Rationale**: WI-43 is a read-only diff engine that compares existing OSCAL JSON outputs as-is. Modifying how FORGE serializes Catalog output is a separate concern.

---

## RES-2: Exit Code Scheme — diff(1) Convention

### Question
The spec (FR-011) requires `diff(1)` convention: `0` = no differences, `1` = differences found, `2` = errors. But FORGE's existing `exit_code()` function maps ForgeErrors to codes 1–4. How to reconcile?

### Findings
- `main.rs` calls `cli::execute()` and dispatches `Err(e) → ExitCode::from(exit_code(&e))` universally.
- `diff(1)` convention requires exit `1` on "has changes" — which is a normal/expected outcome, not an error.
- Printing "Error: ..." to stderr for "differences found" would be semantically wrong.

### Decision
**Two new ForgeError variants:**

1. `ForgeError::DiffHasChanges` — sentinel for "differences were found." Maps to exit code `1`. `main.rs` handles this variant specially (no `eprintln!("Error: ...")` — silent exit 1).
2. `ForgeError::DiffError(String)` — wraps all diff-specific errors (file not found, invalid JSON, non-OSCAL input, type mismatch). Maps to exit code `2` in `exit_code()`.

**main.rs change:**
```rust
match cli::execute(&cli) {
    Ok(()) => ExitCode::SUCCESS,
    Err(ForgeError::DiffHasChanges) => ExitCode::from(1u8),  // silent, not an error
    Err(e) => {
        eprintln!("Error: {e}");
        ExitCode::from(exit_code(&e))
    }
}
```

**Rationale**: Minimal change to existing architecture. Existing commands unaffected. `DiffHasChanges` is a clean sentinel pattern (similar to how some CLIs use `anyhow::bail` semantics).

---

## RES-3: Catalog Group Traversal Depth

### Question
The AR and PRD reference `catalog.groups[].controls[]` — one level deep. But real OSCAL Catalogs (NIST SP 800-53) have nested groups. Should traversal be recursive?

### Finding (from spec clarification — Session 2026-03-12)
**Recursive traversal confirmed.** Extract controls from `groups[]` at any depth, collecting all `controls[]` found at every nesting level.

### Decision
`extract_controls` for Catalog artifacts uses a recursive helper `collect_controls_from_groups` that:
1. Iterates `group["controls"]` and extracts each control
2. Recursively calls itself on `group["groups"]` if present

**Rationale**: FORGE-generated Catalogs are single-level (sections → controls) but OSCAL allows nested groups. Recursive traversal handles both and is only marginally more complex than flat traversal.

---

## RES-4: Co-occurring UUID + Field Changes

### Question
What happens when a control has both a UUID change AND field-level content changes simultaneously?

### Finding (from spec clarification — Session 2026-03-12)
**Single `Changed` entry with UUID stability flag.** When both conditions are true:
- Entry is classified as `DiffEntry::Changed { uuid_changed: true, field_changes: [...] }`
- `DiffEntry::UuidChanged` is only emitted when UUID differs but NO field values changed
- Summary counts: such an entry increments `changed` (not `uuid_changes`)

**Rationale**: Avoids double-counting in summary statistics; keeps all information about one control-id in one entry.

---

## RES-5: Component Definition Extraction Path

### Question
`ImplementedRequirement` uses `control-id` (renamed via serde). What other fields are available for diffing?

### Finding (from `src/oscal/implemented_requirements.rs`)
```rust
pub struct ImplementedRequirement {
    pub uuid: String,                    // present in JSON ✓
    pub control_id: String,              // "control-id" in JSON ✓
    pub description: String,             // implementation narrative ✓
    pub props: Vec<OscalProp>,
    pub links: Vec<OscalLink>,
}
```

`ControlSnapshot` for Component Definition will capture:
- `control_id` = `ir["control-id"]`
- `uuid` = `ir["uuid"]`
- `description` = `ir["description"]` (maps to `parts_prose[0]` for comparison purposes)
- `title` = `None` (implemented-requirements don't have a title field)
- `parts_prose` = `[]` (no parts; description IS the content)

### Decision
`ControlSnapshot.parts_prose` will contain `[description]` when extracted from a Component Definition, so the unified comparison engine works identically for both artifact types. `FieldChange` for Component Definitions will report `description` as the field name when content changes.

---

## RES-6: Existing Module Patterns

### Finding
The `src/trace/` module provides a strong precedent:
- `src/trace/mod.rs` — `generate_trace_report()` orchestration function
- `src/trace/extractor.rs` — extraction logic
- `src/trace/formatter.rs` — text formatting
- `src/trace/report.rs` — data types
- `src/cli/trace.rs` — thin CLI handler calling `generate_trace_report`

The `src/diff/` module will follow this exact pattern. `diff_artifacts()` in `mod.rs` orchestrates extraction + comparison + report assembly. The CLI handler in `src/cli/diff.rs` calls `diff_artifacts()` then `format_diff_report()`.
