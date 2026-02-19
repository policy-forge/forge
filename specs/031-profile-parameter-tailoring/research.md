# Research: 031 Profile Parameter Tailoring

**Phase**: 0 | **Date**: 2026-02-18 | **Branch**: `031-profile-parameter-tailoring`

## Finding 1: clap 4 Two-Value Repeatable Argument

**Decision**: Use `#[arg(long = "set-param", num_args = 2, action = clap::ArgAction::Append, value_names = ["PARAM_ID", "VALUE"])]` on a `Vec<String>` field in `Commands::Profile`.

**Rationale**: In clap 4, `num_args = 2` causes each `--set-param prm1 "60 days"` invocation to consume exactly two tokens from the command line. `ArgAction::Append` accumulates values across repeated occurrences. The result is a flattened `Vec<String>` — e.g., `["prm1", "60 days", "prm2", "4 hours"]` — which `chunks_exact(2)` in `parse_set_param_pairs` converts to `[("prm1", "60 days"), ("prm2", "4 hours")]`. `value_names` improves `--help` output.

**Alternatives considered**:
- `key=value` string parsing (e.g., `--set-param prm1="60 days"`) — rejected: fragile for values containing `=`
- Separate `--param-id`/`--param-value` flags — rejected: cannot associate pairs correctly across multiple invocations
- JSON file bulk input (`--set-param-file`) — deferred as C-1 optional feature; not in WI-31 scope

---

## Finding 2: Aggregation and Ordering Strategy

**Decision**: Use `BTreeMap<String, Vec<String>>` to group param-id → values, then collect in BTreeMap iteration order into `Vec<SetParameter>`.

**Rationale**: `BTreeMap` provides guaranteed alphabetical key ordering, making the output deterministic without an explicit sort step. This directly satisfies FR-008 and S-2 (deterministic ordering) with minimal code. The `entry().or_default().push()` idiom handles duplicate param-id aggregation (FR-007, S-1) cleanly.

**Alternatives considered**:
- `HashMap` + `sort_by_key` — valid but adds an extra step; BTreeMap is simpler
- `IndexMap` (insertion-ordered) — rejected: does not produce alphabetical output

---

## Finding 3: Return Type — Strongly-Typed vs. Raw `serde_json::Value`

**Decision**: `build_modify_section` returns `Option<Modify>` (strongly-typed struct), not `Option<serde_json::Value>`.

**Rationale**: Every other Profile section builder in the codebase uses strongly-typed serde structs (`OscalMetadata`, `ProfileImport`, `ControlSelection`). Using `serde_json::Value` would be inconsistent and would bypass compile-time correctness guarantees on field names and types. The AR interface contract used `Value` as a conceptual shorthand; the implementation follows the established codebase pattern.

**Alternatives considered**:
- `Option<serde_json::Value>` (as shown in AR) — rejected: inconsistent with codebase; bypasses type safety

---

## Finding 4: Pre-existing Compile Errors (Prerequisite Fix)

**Decision**: Fix both compile errors as Task 0 (prerequisite) before any WI-31 code is added.

**Error 1** — `src/parse/atomize.rs` doctest (diagnostic: line 12:5):
`PolicyRequirement` struct literal in a `///` doctest example is missing `modality` and `parameters` fields added by WI-33 and WI-34. Fix: add `modality: None, parameters: vec![]` to the doctest struct literal.

**Error 2** — `src/parse/modality.rs` test helper `req()` (line 178):
`PolicyRequirement` initializer in the test module is missing `parameters: vec![]` (added by WI-34). The `modality: None` field was already present. Fix: add `parameters: vec![]`.

**Rationale**: The codebase must compile (`cargo build`) before WI-31 changes can be integrated. Both fixes are one-line additions to struct initializers; they do not change behavior.

---

## Finding 5: C-2 Warning Behavior (Clarified)

**Decision**: When `--set-param` is provided but neither `--include` nor `--exclude` is specified, emit a non-fatal warning to stderr and continue generating the Profile (exit 0).

**Source**: Clarification session 2026-02-18, Answer A.

**Implementation**: In `cli/profile.rs execute`, before building the Profile, check if `raw_ids_opt` is `None` and `!pairs.is_empty()`. If so:
```rust
eprintln!("warning: --set-param specified without --include or --exclude; the Profile will have no control imports");
tracing::warn!("--set-param used with no control selection; Profile has no imports");
```
Then continue. The existing guard requiring `--include` or `--exclude` must be relaxed to allow `None`/`None` when `set_params` is non-empty.

---

## Finding 6: `OscalProfile` Serde Annotation Strategy

**Decision**: Add `#[serde(skip_serializing_if = "Option::is_none")]` to the new `modify` field on `OscalProfile`.

**Rationale**: FR-006 requires the `modify` key to be absent from JSON when no `--set-param` flags are provided. `skip_serializing_if = "Option::is_none"` is the idiomatic serde approach, consistent with the pattern used on `ProfileImport::include_controls` and `ProfileImport::exclude_controls`. No structural change to `ProfileRoot` is needed.

---

## Finding 7: C-2 Gate Logic — Relaxing the Include/Exclude Guard

**Decision**: When `--set-param` is provided but neither `--include` nor `--exclude` is supplied, the existing `None/None` error branch in `cli/profile.rs execute` must be relaxed.

**Current behavior**: `(None, None)` returns `Err(InvalidArgument(...))`.

**New behavior**: When `set_params` is non-empty and `(None, None)`, emit C-2 warning and proceed with an empty `control_ids` and no `imports` selection, OR generate the Profile with a `modify` section only.

**OSCAL note**: A Profile with only a `modify` section and no `imports` is technically valid OSCAL (modify can stand alone), but it is semantically unusual. The warning communicates this intent to the user. WI-32 validation will catch any schema issues.

**Implementation choice**: When `(None, None)` + `!set_params.is_empty()`, emit warning and call `build_profile_modify_only` — or more simply, allow `build_profile` to accept an empty `control_ids` and skip import construction. This avoids changes to `build_profile`'s error guard. A simpler approach: make `--include`/`--exclude` optional at the CLI level (remove the `Err` branch for `None/None` when params present) and let `build_profile` accept `None` for the selection, returning a Profile with no `imports`. This requires a small refactor of `build_profile` to allow empty control_ids when params are provided.

> **Planning decision**: Keep `build_profile` strict (require non-empty control_ids OR params). The `execute` function handles the C-2 case by providing an empty `control_ids` only when params are present and no selection flags given. This is a "Could Have" feature — if complexity grows, skip C-2 for WI-31 and defer to WI-32.
