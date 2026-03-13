# I/O Decoupling Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decouple terminal I/O from core pipeline logic so forge can be used as a Rust library crate — pipeline functions return structured results instead of writing to stdout/stderr.

**Architecture:** Define `PipelineOutput` result type that carries serialized content, validation reports, secondary artifacts, and statistics. Remove `output_path` parameter from pipeline functions. Move `write_output()` from `pipeline.rs` to a shared CLI helper. Each CLI command becomes a thin wrapper: call library, handle I/O.

**Tech Stack:** Rust 1.93.0, existing crates only (no new dependencies)

**Spec:** `docs/specs/2026-03-12-io-decoupling-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/cli/output.rs` | Shared `write_output()` for all CLI commands |

### Modified Files
| File | Changes |
|------|---------|
| `src/pipeline.rs` | Add `PipelineOutput`/`SecondaryOutput` structs, remove `write_output()`, update pipeline functions to return `PipelineOutput` |
| `src/cli/convert.rs` | Use `PipelineOutput`, handle file writes and dashboard printing |
| `src/cli/mod.rs` | Register `output` module |
| `src/batch/orchestrator.rs` | Receive `PipelineOutput`, write files |
| `src/cli/profile.rs` | Use shared `write_output()` |
| `src/cli/trace.rs` | Use shared `write_output()` |
| `src/cli/validate.rs` | No structural change needed (already in CLI layer), but use shared `write_output()` for consistency |
| `src/cli/resolve.rs` | Move println success messages to return values |
| `src/cli/export.rs` | Use `cli::output::write_output` instead of `pipeline::write_output` |
| `src/model/frontmatter.rs` | Replace `eprintln!` with `tracing::warn!` |
| `src/lib.rs` | Remove `write_output` from pub exports |
| ~20 integration tests | Update to handle `PipelineOutput` return type |

---

## Chunk 1: Define PipelineOutput and move write_output

### Task 1: Add `PipelineOutput` and `SecondaryOutput` structs

**Files:**
- Modify: `src/pipeline.rs`

- [ ] **Step 1: Add the struct definitions**

Add after the imports in `src/pipeline.rs`:

```rust
/// The serialized OSCAL artifact content produced by a pipeline run.
pub struct PipelineOutput {
    /// Serialized content (JSON, XML, or YAML string).
    pub content: String,
    /// The format of the content.
    pub format: OutputFormat,
    /// Optional rendered validation report (when validation produced warnings/errors
    /// but did not block output — currently always None since validation failure aborts).
    pub validation_report: Option<String>,
    /// Optional secondary artifacts (e.g., assessment plan).
    pub secondary_outputs: Vec<SecondaryOutput>,
    /// Optional summary dashboard text (when --summary is enabled).
    pub dashboard: Option<String>,
    /// Conversion statistics (requirements extracted, controls generated, etc.).
    pub statistics: crate::summary::ConversionStatistics,
}

/// A secondary artifact produced alongside the primary output.
pub struct SecondaryOutput {
    /// Suggested filename for this artifact.
    pub filename: String,
    /// Serialized content.
    pub content: String,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`
Expected: compiles (structs not used yet)

- [ ] **Step 3: Commit**

```bash
git add src/pipeline.rs
git commit -m "feat: add PipelineOutput and SecondaryOutput structs (#86)"
```

### Task 2: Create `src/cli/output.rs` with shared `write_output`

**Files:**
- Create: `src/cli/output.rs`
- Modify: `src/cli/mod.rs`

- [ ] **Step 1: Create `src/cli/output.rs`**

Copy the `write_output` function from `src/pipeline.rs` into a new file:

```rust
//! Shared CLI output utilities.

use std::path::Path;

use crate::error::ForgeError;

/// Write content to a file (atomically) or stdout.
///
/// # Errors
/// * `ForgeError::Validation` if parent directory does not exist
/// * `ForgeError::Io` if file write fails
pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<(), ForgeError> {
    match output_path {
        None => {
            println!("{content}");
            Ok(())
        }
        Some(path) => {
            if let Some(parent) = path.parent()
                && !parent.as_os_str().is_empty()
                && !parent.exists()
            {
                return Err(ForgeError::Validation(format!(
                    "Output directory '{}' does not exist",
                    parent.display()
                )));
            }
            crate::io::write_atomic(path, content.as_bytes())?;
            Ok(())
        }
    }
}
```

- [ ] **Step 2: Register the module in `src/cli/mod.rs`**

Add `pub mod output;` in the module declarations.

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add src/cli/output.rs src/cli/mod.rs
git commit -m "feat: add shared cli::output::write_output (#86)"
```

---

## Chunk 2: Refactor pipeline functions

### Task 3: Refactor `validate_and_serialize` to return report instead of printing

**Files:**
- Modify: `src/pipeline.rs`

- [ ] **Step 1: Change `validate_and_serialize` to return validation report as data**

Change the return type from `Result<String, ForgeError>` to `Result<(String, Option<String>), ForgeError>`.

Replace the `eprintln!("{rendered}")` block:

```rust
fn validate_and_serialize<T: serde::Serialize>(
    envelope: &T,
    label: &str,
    model_type: crate::validate::OscalModelType,
) -> Result<(String, Option<String>), ForgeError> {
    let json = serde_json::to_string_pretty(envelope)
        .map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let json_value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let report = crate::validate::run_full_validation(
        &format!("generated {label}"),
        &json_value,
        model_type,
    )
    .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;
    if !report.is_valid() {
        let rendered = crate::validate::report::render_text_report(&report);
        // PRD EC-7: do NOT write output file on validation failure
        return Err(ForgeError::SchemaValidation(format!(
            "{} validation error(s) in generated {label}",
            report.errors().len()
        )));
    }
    Ok((json, None))
}
```

Note: The rendered report is currently only shown on validation *failure*, which becomes an `Err`. For now the `Option<String>` is always `None` on success. This preserves the existing behavior (validation errors abort the pipeline) while providing the hook for future `--force` or warning-level reports.

- [ ] **Step 2: Update both callers of `validate_and_serialize`**

In `run_catalog_pipeline` (~line 232), change:
```rust
let json = validate_and_serialize(...)?;
```
to:
```rust
let (json, _validation_report) = validate_and_serialize(...)?;
```

Same in `run_component_pipeline` (~line 334).

- [ ] **Step 3: Verify tests pass**

Run: `cargo test --lib pipeline`

- [ ] **Step 4: Commit**

```bash
git add src/pipeline.rs
git commit -m "refactor: validate_and_serialize returns report as data (#86)"
```

### Task 4: Refactor `run_catalog_pipeline` to return `PipelineOutput`

**Files:**
- Modify: `src/pipeline.rs`

- [ ] **Step 1: Change the function signature**

Remove `output_path: Option<&Path>` parameter. Change return type to `Result<PipelineOutput, ForgeError>`.

```rust
pub fn run_catalog_pipeline(
    input_path: &Path,
    max_size_bytes: u64,
    format: &OutputFormat,
    import_ssp_href: Option<&str>,
) -> Result<PipelineOutput, ForgeError> {
```

- [ ] **Step 2: Replace the format match + write_output block**

Replace the current match block that calls `write_output` with one that just produces `content`:

```rust
let content = match format {
    OutputFormat::Json => json,
    OutputFormat::Xml => {
        crate::export::xml_serializer::serialize_catalog_to_xml(&envelope.catalog)?
    }
    OutputFormat::Yaml => crate::export::yaml::serialize_to_yaml(&envelope)?,
};
```

- [ ] **Step 3: Replace the assessment plan write with secondary output**

Replace the `write_assessment_plan` call with:

```rust
let mut secondary_outputs = Vec::new();
if let Some(href) = import_ssp_href {
    let control_ids =
        crate::oscal::catalog::collect_control_ids_from_catalog(&envelope.catalog);
    let ap_envelope =
        crate::oscal::build_assessment_plan(&control_ids, href, &envelope.catalog.metadata.title)?;
    let ap_json = serde_json::to_string_pretty(&ap_envelope)
        .map_err(|e| ForgeError::Serialization(e.to_string()))?;
    let ap_filename = crate::oscal::derive_ap_output_path(input_path, None)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| "assessment-plan.json".to_string());
    secondary_outputs.push(SecondaryOutput {
        filename: ap_filename,
        content: ap_json,
    });
}
```

- [ ] **Step 4: Build and return PipelineOutput**

```rust
Ok(PipelineOutput {
    content,
    format: *format,
    validation_report: None,
    secondary_outputs,
    dashboard: None,
    statistics: stats,
})
```

Remove the `stats.output_path = ...` line (no longer relevant — CLI owns paths).

- [ ] **Step 5: Fix compilation errors in callers (temporarily use `.content`)**

This will break callers. For now, just get `pipeline.rs` compiling. Callers are fixed in the next tasks.

- [ ] **Step 6: Commit**

```bash
git add src/pipeline.rs
git commit -m "refactor: run_catalog_pipeline returns PipelineOutput (#86)"
```

### Task 5: Refactor `run_component_pipeline` to return `PipelineOutput`

**Files:**
- Modify: `src/pipeline.rs`

- [ ] **Step 1: Apply the same pattern as Task 4**

Remove `output_path: Option<&Path>` parameter. Change return type to `Result<PipelineOutput, ForgeError>`. Replace `write_output` calls with `content` assignment. Replace `write_assessment_plan` with `secondary_outputs`. Return `PipelineOutput`.

- [ ] **Step 2: Commit**

```bash
git add src/pipeline.rs
git commit -m "refactor: run_component_pipeline returns PipelineOutput (#86)"
```

### Task 6: Remove `write_output` and `write_assessment_plan` from pipeline.rs

**Files:**
- Modify: `src/pipeline.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Delete `write_output()` from pipeline.rs**

Remove the function entirely (it now lives in `cli/output.rs`).

- [ ] **Step 2: Delete `write_assessment_plan()` from pipeline.rs**

The logic has been inlined into the pipeline return values.

- [ ] **Step 3: Update `src/lib.rs` exports**

Remove `write_output` from the pub re-exports if present. Add `PipelineOutput` and `SecondaryOutput` to exports.

- [ ] **Step 4: Delete the `write_output` unit tests from pipeline.rs**

These tests now belong in `cli/output.rs`. Move them there (or recreate — they're simple).

- [ ] **Step 5: Commit**

```bash
git add src/pipeline.rs src/lib.rs src/cli/output.rs
git commit -m "refactor: remove write_output and write_assessment_plan from pipeline (#86)"
```

---

## Chunk 3: Update CLI callers

### Task 7: Update `cli/convert.rs` to use `PipelineOutput`

**Files:**
- Modify: `src/cli/convert.rs`

- [ ] **Step 1: Update `execute()` to handle `PipelineOutput`**

```rust
use crate::cli::output::write_output;

pub fn execute(opts: &ConvertOptions<'_>) -> Result<(), ForgeError> {
    let max_size_bytes = max_size_to_bytes(opts.max_size)?;

    if let Some(baseline) = opts.stable_id_baseline {
        validate_regular_file(baseline, "--stable-id-baseline")?;
        emit_stable_id_change_warning_if_needed(opts.input, baseline, max_size_bytes)?;
    }

    let start = std::time::Instant::now();

    let mut result = match opts.strategy {
        Strategy::Catalog => crate::pipeline::run_catalog_pipeline(
            opts.input,
            max_size_bytes,
            opts.format,
            opts.import_ssp,
        )?,
        Strategy::Component => {
            let profile_ref = resolve_source_profile(opts.source_profile)?;
            crate::pipeline::run_component_pipeline(
                opts.input,
                max_size_bytes,
                profile_ref,
                opts.format,
                opts.import_ssp,
            )?
        }
    };

    // Write primary output
    write_output(&result.content, opts.output)?;

    // Write secondary artifacts (e.g., assessment plan)
    for secondary in &result.secondary_outputs {
        let ap_dir = opts.output.and_then(|p| p.parent());
        let ap_path = ap_dir
            .map(|d| d.join(&secondary.filename))
            .unwrap_or_else(|| std::path::PathBuf::from(&secondary.filename));
        write_output(&secondary.content, Some(&ap_path))?;
        tracing::info!(path = %ap_path.display(), "Secondary artifact written");
    }

    // Summary dashboard
    if opts.summary && !opts.quiet {
        result.statistics.elapsed = start.elapsed();
        let use_color = std::io::IsTerminal::is_terminal(&std::io::stderr());
        let dashboard = crate::summary::format::format_summary_dashboard(
            &result.statistics,
            use_color,
        );
        eprint!("{dashboard}");
    }

    Ok(())
}
```

- [ ] **Step 2: Verify tests pass**

Run: `cargo test --lib cli::convert`

- [ ] **Step 3: Commit**

```bash
git add src/cli/convert.rs
git commit -m "refactor: cli/convert uses PipelineOutput (#86)"
```

### Task 8: Update `batch/orchestrator.rs`

**Files:**
- Modify: `src/batch/orchestrator.rs`

- [ ] **Step 1: Update `run_pipeline()` to handle `PipelineOutput`**

```rust
fn run_pipeline(
    input: &Path,
    output: &Path,
    strategy: Strategy,
    format: OutputFormat,
    max_size_bytes: u64,
    source_profile: Option<&str>,
) -> Result<(), ForgeError> {
    let result = match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, max_size_bytes, &format, None)?
        }
        Strategy::Component => {
            crate::pipeline::run_component_pipeline(
                input, max_size_bytes, source_profile, &format, None,
            )?
        }
    };
    crate::cli::output::write_output(&result.content, Some(output))?;
    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add src/batch/orchestrator.rs
git commit -m "refactor: batch orchestrator uses PipelineOutput (#86)"
```

### Task 9: Update remaining CLI commands

**Files:**
- Modify: `src/cli/profile.rs`
- Modify: `src/cli/trace.rs`
- Modify: `src/cli/export.rs`
- Modify: `src/model/frontmatter.rs`

- [ ] **Step 1: `cli/profile.rs` — use shared `write_output`**

Replace the inline match block:
```rust
match output {
    Some(path) => crate::io::write_atomic(path, serialized.as_bytes())?,
    None => println!("{serialized}"),
}
```
with:
```rust
crate::cli::output::write_output(&serialized, output)?;
```

- [ ] **Step 2: `cli/trace.rs` — use shared `write_output`**

Replace the match block with:
```rust
crate::cli::output::write_output(&table, output)?;
if output.is_some() {
    eprintln!("Trace report written to {}", output.unwrap().display());
}
```

- [ ] **Step 3: `cli/export.rs` — use `cli::output::write_output`**

Replace `crate::pipeline::write_output(...)` with `crate::cli::output::write_output(...)`.

- [ ] **Step 4: `model/frontmatter.rs` — replace `eprintln!` with `tracing::warn!`**

Change:
```rust
eprintln!("Warning: failed to parse YAML frontmatter: {e}");
```
to:
```rust
tracing::warn!("Failed to parse YAML frontmatter: {e}");
```

- [ ] **Step 5: Run full test suite**

Run: `cargo test`

- [ ] **Step 6: Commit**

```bash
git add src/cli/profile.rs src/cli/trace.rs src/cli/export.rs src/model/frontmatter.rs
git commit -m "refactor: CLI commands use shared write_output, frontmatter uses tracing (#86)"
```

---

## Chunk 4: Update integration tests

### Task 10: Update integration tests for new pipeline signatures

**Files:**
- Modify: All test files that call `run_catalog_pipeline` or `run_component_pipeline`

The signature changes are:
- `output_path` parameter removed
- Returns `PipelineOutput` instead of `ConversionStatistics`
- Statistics available via `result.statistics`
- Content available via `result.content`

For tests that passed `output: Some(path)`:
```rust
// OLD:
let stats = run_catalog_pipeline(&input, Some(&output), max_size, &format, None)?;
assert!(output.exists());

// NEW:
let result = run_catalog_pipeline(&input, max_size, &format, None)?;
std::fs::write(&output, &result.content).unwrap(); // or use write_output
assert!(!result.content.is_empty());
```

For tests that passed `output: None`:
```rust
// OLD:
let stats = run_catalog_pipeline(&input, None, max_size, &format, None)?;

// NEW:
let result = run_catalog_pipeline(&input, max_size, &format, None)?;
// Use result.content or result.statistics as needed
```

For tests that used `stats.controls_generated` etc.:
```rust
// OLD:
let stats = run_catalog_pipeline(...)?;
assert!(stats.controls_generated > 0);

// NEW:
let result = run_catalog_pipeline(...)?;
assert!(result.statistics.controls_generated > 0);
```

Apply this pattern to all ~20 test files. The affected test files are:
- `tests/catalog_pipeline_test.rs`
- `tests/component_pipeline_test.rs`
- `tests/assessment_plan_test.rs`
- `tests/golden_file_tests.rs`
- `tests/yaml_equivalence_test.rs`
- `tests/yaml_security_test.rs`
- `tests/xml_catalog_test.rs`
- `tests/xml_component_test.rs`
- `tests/xml_validation_test.rs`
- `tests/oscal_cli_round_trip.rs`

- [ ] **Step 1: Update all test files**

Read each test file, update the pipeline calls to match the new signatures.

- [ ] **Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add tests/
git commit -m "test: update integration tests for PipelineOutput return type (#86)"
```

### Task 11: Update pipeline.rs unit tests

**Files:**
- Modify: `src/pipeline.rs`

- [ ] **Step 1: Update unit tests**

The `write_output` tests were moved/deleted in Task 6. The remaining pipeline unit tests (`empty_file_pipeline_returns_empty_input`, `structureless_file_pipeline_returns_no_structure_detected`, `catalog_pipeline_valid_input_succeeds`, `component_pipeline_auto_validation_uses_report_format`) need their signatures updated.

- [ ] **Step 2: Run tests**

Run: `cargo test --lib pipeline`

- [ ] **Step 3: Commit**

```bash
git add src/pipeline.rs
git commit -m "test: update pipeline unit tests for PipelineOutput (#86)"
```

---

## Chunk 5: Final verification

### Task 12: Verify and clean up

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All 1424+ tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: Zero warnings

- [ ] **Step 3: Run fmt**

Run: `cargo fmt --check`
Expected: Clean

- [ ] **Step 4: Verify no remaining I/O in pipeline.rs**

Search for `println!`, `eprintln!`, `print!`, `eprint!` in `src/pipeline.rs` — should find zero matches.

- [ ] **Step 5: Verify library API**

Check that `PipelineOutput`, `SecondaryOutput`, `run_catalog_pipeline`, `run_component_pipeline` are publicly accessible from `forge::pipeline::*`.

- [ ] **Step 6: Commit any final fixes**
