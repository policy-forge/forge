# Research: Summary Dashboard (044)

## R1: Pipeline Return Type Change

**Decision**: Modify `run_catalog_pipeline` and `run_component_pipeline` to return `Result<ConversionStatistics, ForgeError>` instead of `Result<(), ForgeError>`.

**Rationale**: Statistics are always cheap to collect (counter increments). Returning them unconditionally avoids conditional logic in the pipeline. Callers that don't need stats can ignore the return value (Rust allows this without warnings for `Result` if the `Err` case is handled via `?`). Actually, since the return is `Result<Stats, Err>`, callers using `?` will still propagate errors. Callers that currently call `.map(|()| ())` or match on `Ok(())` will need minor updates.

**Impact assessment**: Callers of these pipeline functions:
- `src/cli/convert.rs::execute` — primary caller, will use the stats
- `tests/catalog_pipeline_test.rs` — tests call `run_catalog_pipeline` and check `Ok(())`; need update to `Ok(_)` or `Ok(stats)`
- `tests/component_pipeline_test.rs` — same pattern
- `tests/pipeline_test.rs` — same
- Various other integration tests

The test changes are mechanical: `Ok(())` → `Ok(_)` or destructure the stats. Low risk.

**Alternatives considered**:
- `&mut ConversionStatistics` parameter: More invasive, forces all callers to create stats even when unused
- Separate `_with_stats` wrapper: Code duplication
- Collect stats in `convert::execute` by reimplementing pipeline steps: Fragile, couples CLI to pipeline internals

## R2: Section and Requirement Counting

**Decision**: Count sections and requirements from the `PolicyDocument` returned by `prepare_document`.

**Rationale**: The `PolicyDocument` has `sections: Vec<PolicySection>` and each section has `requirements: Vec<PolicyRequirement>` and `children: Vec<PolicySection>`. Counting requires a recursive traversal.

**Implementation**:
```rust
fn count_sections(doc: &PolicyDocument) -> usize {
    fn count_recursive(sections: &[PolicySection]) -> usize {
        sections.iter().map(|s| 1 + count_recursive(&s.children)).sum()
    }
    count_recursive(&doc.sections)
}

fn count_requirements(doc: &PolicyDocument) -> usize {
    fn count_recursive(sections: &[PolicySection]) -> usize {
        sections.iter().map(|s| {
            s.requirements.len() + count_recursive(&s.children)
        }).sum()
    }
    count_recursive(&doc.sections)
}
```

## R3: Controls Counting

**Decision**: Count controls from the generated OSCAL artifacts.

- **Catalog**: Count controls across all groups in the `OscalCatalog`. Groups contain controls; need recursive group traversal.
- **Component Definition**: Count implemented-requirements across all components' control-implementations.

**Implementation locations**: Inside `run_catalog_pipeline` after `build_catalog`, and inside `run_component_pipeline` after `build_component_definition`.

## R4: Validation Status Capture

**Decision**: The existing pipeline validates using `validate_catalog_json` / `validate_component_json`, which returns `Ok(json_string)` on success or `Err(ForgeError::SchemaValidation)` on failure. Currently, validation failure is a hard error that stops the pipeline.

For the dashboard, validation status maps to:
- `validate_*` returns `Ok(_)` → `ValidationStatus::Passed`
- `validate_*` returns `Err(SchemaValidation)` → pipeline returns error, no dashboard printed (per EC-5)
- Validation warnings: The current `ValidationReport` tracks errors but the pipeline only fails on errors (not warnings). If `report.is_valid()` but warnings exist → `PassedWithWarnings` (needs investigation if warnings are currently tracked)

**Finding**: Looking at `validate_catalog_json`, it checks `report.is_valid()` (which means no errors). The `ValidationReport` currently only has `errors()` — no separate warnings concept. The spec mentions "passed with warnings" but the current validation infrastructure doesn't distinguish warnings from errors.

**Resolution**: For the initial implementation, `ValidationStatus` will be:
- `Passed` — validation succeeds (no errors)
- `Failed` — validation fails (has errors; but pipeline aborts so dashboard won't show this)
- `NotRun` — validation not executed

Since validation failure causes the pipeline to abort before the dashboard is printed (per EC-5), the dashboard will only ever show `Passed` or `NotRun`. The `PassedWithWarnings` and `Failed` variants remain in the enum for forward compatibility.

## R5: ANSI Color via std::io::IsTerminal

**Decision**: Use `std::io::IsTerminal` trait (stable since Rust 1.70, well within our 1.93.0 minimum).

**Implementation**:
```rust
use std::io::IsTerminal;

fn is_color_enabled() -> bool {
    std::io::stdout().is_terminal()
}
```

ANSI escape codes:
- Green: `\x1b[32m` (PASSED)
- Red: `\x1b[31m` (FAILED)
- Yellow: `\x1b[33m` (warnings, >100% coverage)
- Reset: `\x1b[0m`

## R6: Box-Drawing Characters

**Decision**: Use Unicode box-drawing characters for the dashboard frame.

Characters used:
```
┌ ─ ┐  Top corners and horizontal line
│      Vertical line
├ ─ ┤  Section separators
└ ─ ┘  Bottom corners
```

Fixed-width dashboard (~45 chars wide). Label alignment using format padding.

**Alternatives considered**: `comfy-table` crate — rejected per constitution (no new dependencies for XS feature).

## R7: Elapsed Time Display Format

**Decision**: Display as human-readable duration:
- `< 1s` → `0.XXs` (e.g., "0.42s")
- `1s - 60s` → `X.Xs` (e.g., "3.2s")
- `> 60s` → `Xm Xs` (e.g., "1m 23s")

Use `std::time::Instant::now()` before pipeline start and `.elapsed()` after file write.
