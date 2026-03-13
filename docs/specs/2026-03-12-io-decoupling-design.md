# I/O Decoupling — Design Spec

> **Date**: 2026-03-12
> **Issue**: #86
> **Status**: Approved

---

## Overview

Decouple terminal I/O from core pipeline logic so forge can be used as a Rust library crate. Pipeline functions return structured results instead of writing to stdout/stderr. The CLI layer becomes the sole owner of all I/O (file writes, stdout, stderr).

## Result Types

New types in `src/pipeline.rs`:

```rust
/// The serialized OSCAL artifact content produced by a pipeline run.
pub struct PipelineOutput {
    /// Serialized content (JSON, XML, or YAML string).
    pub content: String,
    /// The format of the content.
    pub format: OutputFormat,
    /// Optional rendered validation report (when validation had warnings/errors).
    pub validation_report: Option<String>,
    /// Optional secondary artifacts (e.g., assessment plan).
    pub secondary_outputs: Vec<SecondaryOutput>,
    /// Optional summary dashboard (when --summary is enabled).
    pub dashboard: Option<String>,
    /// Conversion statistics (requirements extracted, controls generated, etc.).
    pub statistics: ConversionStatistics,
}

/// A secondary artifact produced alongside the primary output.
pub struct SecondaryOutput {
    /// Suggested filename for this artifact.
    pub filename: String,
    /// Serialized content.
    pub content: String,
}
```

## Pipeline Function Changes

### `run_catalog_pipeline` and `run_component_pipeline`

**Current**: Accept `output: Option<&Path>`, write to file/stdout, return `Result<ConversionStatistics, ForgeError>`.

**New**: Remove `output` parameter, return `Result<PipelineOutput, ForgeError>`. The `ConversionStatistics` data (requirements_extracted, controls_generated, etc.) is always present in `PipelineOutput.statistics`.

### `validate_and_serialize`

**Current**: Prints validation error report to stderr via `eprintln!`, returns `Result<String, ForgeError>` (serialized JSON string).

**New**: Returns `Result<(String, Option<String>), ForgeError>` — serialized content and optional rendered validation report. No I/O.

### `write_assessment_plan`

**Current**: Writes assessment plan directly to a derived file path. Only called when `import_ssp` is `Some(...)`.

**New**: Returns assessment plan content as a `SecondaryOutput` in `PipelineOutput.secondary_outputs`. When `import_ssp` is `None`, `secondary_outputs` is empty. CLI derives the path and writes.

### Summary dashboard

**Current**: `eprintln!` in `cli/convert.rs` (already in CLI layer, just needs `--quiet` gating — done in PR #87).

**New**: Returned in `PipelineOutput.dashboard` field. CLI prints to stderr (respecting `--quiet`).

## CLI Layer Changes

### `write_output()` moves from `pipeline.rs` to CLI

`write_output(content, output_path)` is purely a CLI concern — it decides between stdout and file. Moved to `src/cli/output.rs` as a shared utility used by all CLI commands.

Remove from public API exports in `src/lib.rs`.

### `cli/convert.rs`

Becomes the orchestrator: calls pipeline, writes primary output, writes secondary artifacts, prints dashboard and validation reports.

```rust
pub fn execute(opts: &ConvertOptions) -> Result<(), ForgeError> {
    let result = match opts.strategy {
        Strategy::Catalog => pipeline::run_catalog_pipeline(opts.input, *opts.format, ...),
        Strategy::Component => pipeline::run_component_pipeline(opts.input, *opts.format, ...),
    }?;

    write_output(&result.content, opts.output)?;

    for secondary in &result.secondary_outputs {
        let path = derive_secondary_path(opts.output, &secondary.filename);
        write_output(&secondary.content, Some(&path))?;
    }

    if !opts.quiet {
        if let Some(dashboard) = &result.dashboard { eprint!("{dashboard}"); }
    }
    if let Some(report) = &result.validation_report { eprint!("{report}"); }

    Ok(())
}
```

### `cli/profile.rs`

Already mostly clean. Replace the inline `println!`/`write_atomic` with `write_output()`.

### `cli/trace.rs`

Already clean — generates a table string and writes it. Use shared `write_output()`.

### `cli/validate.rs`

Currently prints "Valid: ..." to stdout and renders reports. Change to return structured result, CLI prints.

### `cli/resolve.rs`

Currently prints success messages. Change to return output path, CLI prints.

### `batch/orchestrator.rs`

Currently calls `run_catalog_pipeline`/`run_component_pipeline` which write to files. After refactor, batch orchestrator receives `PipelineOutput` and writes files itself. Note: `--summary` is not supported in batch mode (already warns and ignores), so `PipelineOutput.dashboard` will always be `None` in the batch path.

## What Stays As-Is

- **`tracing` calls** (`debug!`, `info!`, `warn!`) — these are already properly decoupled. Library consumers attach their own tracing subscriber.
- **`main.rs` error printing** — `eprintln!("Error: {e}")` in the binary entrypoint is appropriate.
- **`main.rs` error printing** — `eprintln!("Error: {e}")` in the binary entrypoint is appropriate.

### Migrated as Part of This Refactor

- **`model/frontmatter.rs` warning** — `eprintln!("Warning: ...")` migrated to `tracing::warn!`.

## Callers to Update

| Caller | File | Changes |
|--------|------|---------|
| CLI convert (single) | `src/cli/convert.rs:240,250` | Receive `PipelineOutput`, handle I/O |
| CLI convert (batch) | `src/batch/orchestrator.rs:136,145` | Receive `PipelineOutput`, write files |
| CLI export | `src/cli/export.rs:303` | Use `cli::output::write_output` instead of `pipeline::write_output` |
| Integration tests (~20) | `tests/*.rs` | Update to handle `PipelineOutput` return type |

## Testing

- All existing tests continue to pass (they currently ignore the return value or check file output)
- Tests that pass `output: Some(path)` will instead receive `PipelineOutput` and verify `result.content`
- Add test: `run_catalog_pipeline` returns `PipelineOutput` with non-empty content
- Add test: `run_catalog_pipeline` with `--summary` returns populated `dashboard` field
- Add test: `run_catalog_pipeline` with `--import-ssp` returns populated `secondary_outputs`
