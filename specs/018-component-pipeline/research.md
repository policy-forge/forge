# Research: End-to-End Component Definition Pipeline

**Feature**: 018-component-pipeline
**Date**: 2026-02-13

## R-1: Optional `--source-profile` Threading

**Decision**: Change `run_component_pipeline` from `source_profile: &str` to `source_profile: Option<&str>`.

**Rationale**: The downstream `build_component_definition()` already accepts `Option<&str>` and produces an empty `control-implementations` array when `None`. Only the pipeline function signature and CLI handler need updating. The CLI layer emits `tracing::warn!()` to stderr when omitted.

**Alternatives considered**:
- Sentinel empty string — rejected; bypasses type safety and requires defensive checks
- New error variant — rejected; spec S-1 requires success with warning, not failure

## R-2: Source Profile File Validation

**Decision**: Validate in `cli/convert.rs` before pipeline execution: (1) path exists, (2) is regular file, (3) is readable. Do NOT parse JSON content at CLI level per AR anti-pattern guidance.

**Rationale**: AR §Anti-patterns: "Don't parse the source profile eagerly at CLI argument parsing time." Since the current design treats `--source-profile` as a reference string (not parsed for control IDs), SEC-4 (JSON parsability) is deferred to a future WI that implements profile resolution (W-3).

**Alternatives considered**:
- Full JSON deserialization at CLI level — rejected per AR anti-pattern guidance
- No validation at all — rejected per SEC-3

## R-3: Absolute Path Mitigation (SEC-1)

**Decision**: Use `input_path.file_name()` (filename only) for the `source_file` parameter passed to `build_component_definition`. This prevents absolute filesystem paths from appearing in OSCAL `props`.

**Rationale**: SEC-1: "shall not leak absolute filesystem paths beyond what the user explicitly provides." Filename-only is the most conservative approach. The existing test `component_pipeline_documentary_component_has_source_file_prop()` already checks for `full_policy.md` — it will continue to pass with filename-only.

**Alternatives considered**:
- Relative path calculation — requires a reference directory; more complex; deferred
- Pass input exactly as user typed it — user may type absolute path; violates SEC-1

## R-4: `--format` Default Value (EC-1)

**Decision**: Add `default_value = "json"` to the `--format` clap argument. Simple attribute change.

**Rationale**: EC-1 requires JSON default when `--format` omitted. All existing tests specifying `--format json` continue unchanged.

**Alternatives considered**:
- `Option<OutputFormat>` with code default — rejected; more complex than clap default_value
