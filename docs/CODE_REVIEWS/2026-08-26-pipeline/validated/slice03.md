# Slice03 Validation Report

Reviewed: 2026-08-26 against HEAD b22e2d5 ("Harden successor map opening against symlink races").
Slice: 62 medium findings.

**Counts: valid 57 · partial 2 · invalid 2 · duplicate 1**

Valid: F0299, F0300, F0312, F0313, F0314, F0315, F0316, F0320, F0321, F0326, F0327, F0330, F0333, F0340, F0343, F0346, F0347, F0351, F0352, F0357, F0358, F0359, F0360, F0365, F0369, F0370, F0372, F0373, F0377, F0381, F0394, F0395, F0396, F0397, F0401, F0405, F0406, F0408, F0412, F0414, F0417, F0418, F0420, F0422, F0424, F0425, F0426, F0427, F0439, F0440, F0441, F0442, F0443, F0444, F0445, F0450, F0451
Partial: F0322, F0388
Invalid: F0385, F0387
Duplicate: F0366 (of F0369)

---

## VALID / PARTIAL findings

### F0299 — valid
- File:lines: `src/citation.rs:196-199` (call site), `strip_matches` 212-235, `normalize_prose` 237-272
- Symbols: `extract_citations_from_text`, `strip_matches`, `normalize_prose`
- Category: bug | Severity: medium
- Root cause: `extract_citations_from_text` unconditionally calls `strip_matches` then `normalize_prose` even when `matched_ranges` is empty. `strip_matches` returns text verbatim for empty ranges, but `normalize_prose` still collapses all whitespace runs to single spaces, strips "( )"/"()", rewrites punctuation artifacts (", ." → ".", " ," → ","), and trims. Every requirement with no citations therefore has interior double spaces, tabs/newlines, and deliberate formatting silently flattened on every pipeline run.
- Evidence: lines 196-197 `let cleaned = strip_matches(text, &matched_ranges); let cleaned = normalize_prose(&cleaned);` — no `matched_ranges.is_empty()` gate. Requirement text fed in already comes from the parser with internal whitespace preserved (atomize trims clauses but does not collapse interior whitespace), so fidelity loss is real.
- Remediation: In `extract_citations_from_text`, replace the two unconditional calls with: if `matched_ranges.is_empty()` return `(text.to_string(), citations)`; else `normalize_prose(&strip_matches(text, &matched_ranges))`. Add unit test `no_citations_text_unchanged` asserting a text with tabs, double spaces, and leading/trailing whitespace round-trips byte-identical when it contains no URL/bibliographic/cross-ref patterns. Snapshot impact: tests/snapshots for catalog/component generation may change only if fixtures previously relied on whitespace collapse for citation-free requirements — run `cargo insta review` to confirm none shift (they should not, since existing fixtures with citations are unaffected).

### F0300 — valid
- File:lines: `src/citation.rs:83-87`
- Symbols: `extract_citations_from_section`, `ForgeError::Parse`
- Category: maintainability | Severity: medium
- Root cause: The missing-`stable_id` precondition (WI-7 must have run before WI-8) is reported as `ForgeError::Parse`, which semantically means malformed Markdown input. A stage-ordering violation is a programming/pipeline invariant failure, not a parse failure, and `exit_code(Parse) == 2` with "Parse error:" prefix misleads operators. One bad requirement also aborts the whole document.
- Evidence: line 83-86 constructs `ForgeError::Parse(format!("Requirement at line {} missing stable_id before citation extraction (run UUID assignment first)", ...))`.
- Remediation: Add a dedicated variant (e.g. `ForgeError::PipelineInvariant(String)` or reuse `CatalogBuild`-style stage errors) carrying `source_line` and requirement context; place it in exit-code bucket 2; update `extract_citations_from_section` to use it; add a display+exit-code test in `src/error.rs` mirroring `catalog_build_error_display`.

### F0312 — valid
- File:lines: `src/cli/convert.rs:88-91`
- Symbols: `resolve_source_profile`
- Category: documentation | Severity: medium
- Root cause: Doc comment says "Returns `Ok(None)` if no profile was provided (with a warning)" but the `None` arm returns `Err(ForgeError::InvalidArgument("--source-profile is required..."))`. The test `component_strategy_none_source_profile_errors_with_required_message` (convert.rs:541) asserts the error, so the contract is mandatory-profile; the comment is stale and dangerous.
- Evidence: line 93-96 returns Err on None; comment at 89 promises Ok(None)+warning.
- Remediation: Rewrite the doc to: "Returns `Err(ForgeError::InvalidArgument)` if no profile was provided (mandatory for schema-valid component definitions), `Err(ForgeError::Validation)` if empty/whitespace-only, `Err` if the path is missing/not a regular file, else `Ok(Some(path))`." No code change; no test change.

### F0313 — valid
- File:lines: `src/cli/convert.rs:196-198` (SSP path) vs 362-366 (catalog/component path)
- Symbols: `execute_ssp`, `emit_stable_id_change_warning_if_needed`
- Category: bug | Severity: medium
- Root cause: `execute_ssp` validates `--stable-id-baseline` existence (line 197 `validate_regular_file`) but never calls `emit_stable_id_change_warning_if_needed`, so `--to ssp --stable-id-baseline` silently skips drift detection that the catalog/component path performs. `--summary` is likewise accepted but ignored on the SSP path.
- Evidence: `execute()` at 362-366 calls the warning emitter; `execute_ssp` only validates and drops the value.
- Remediation: After the `validate_regular_file(baseline, ...)` call in `execute_ssp`, add `emit_stable_id_change_warning_if_needed(opts.input, baseline, max_size_bytes).map_err(add_max_size_guidance)?;`. Add integration test in tests/ asserting `forge convert --to ssp --stable-id-baseline` emits the stable-id warning for a substantive change (assert stderr contains "stable id changed").

### F0314 — valid
- File:lines: `src/cli/convert.rs:217` (use site), `build_ssp_skeleton` in `src/oscal/ssp.rs:650-678`
- Symbols: `execute_ssp`, `build_ssp_skeleton`
- Category: bug | Severity: medium
- Root cause: `execute_ssp` passes `opts.source_profile.unwrap_or("")` straight into `build_ssp_skeleton`, which only substitutes "TODO-profile.json" for the fully-empty (post-trim) case. A whitespace-only or nonexistent file path is embedded verbatim into `import_profile.href` without the SEC-3/SEC-4 validation (`resolve_source_profile`) applied to component strategy.
- Evidence: convert.rs:217 `opts.source_profile.unwrap_or("")`; ssp.rs:674-678 `if source_profile.trim().is_empty() { "TODO-profile.json" } else { source_profile.to_string() }`.
- Remediation: In `execute_ssp`, trim and normalize the value: `let source_profile = opts.source_profile.map(str::trim).filter(|p| !p.is_empty()); if let Some(p) = source_profile { validate_regular_file(Path::new(p), "--source-profile")?; }` and pass `source_profile.unwrap_or("")`. Add tests: `--to ssp --source-profile nonexistent.json` errors with a validation message; `--source-profile "  "` yields the TODO placeholder.

### F0315 — valid
- File:lines: `src/cli/convert.rs:397-404`
- Symbols: `execute` (secondary artifact loop)
- Category: bug | Severity: medium
- Root cause: When `--output` is omitted the primary artifact goes to stdout, but secondary assessment-plan artifacts are written as files relative to CWD (`PathBuf::from(&secondary.filename)`), surprising users piping output. The loop writes eagerly, so a later failure leaves earlier artifacts with no cleanup.
- Evidence: lines 398-404 compute `ap_dir = opts.output.and_then(|p| p.parent())` and write each secondary immediately.
- Remediation: When `opts.output.is_none()`, skip secondary emission and emit a stderr notice ("assessment plan suppressed: primary output is stdout"), or write them beside the input; when `opts.output.is_some()`, resolve all destinations and pre-check writability (parent exists) before the first write. Update/extend tests around assessment-plan generation in tests/ (search for `secondary_outputs`/`assessment-plan` fixtures) to assert the stdout-suppression behavior.

### F0316 — valid
- File:lines: `src/cli/convert.rs:129-130`
- Symbols: `emit_stable_id_change_warning_if_needed`, `prepare_document` (`src/pipeline.rs:76-79`), `run_catalog_pipeline` (145), `run_component_pipeline` (310)
- Category: performance | Severity: medium
- Root cause: With `--stable-id-baseline`, `prepare_document(input, ...)` runs fully (ingest→parse→atomize→UUID→citations→parameters) inside the warning pass at line 130, then again inside `run_catalog_pipeline`/`run_component_pipeline` — doubling CPU, I/O, and failure surface for the main input.
- Evidence: pipeline.rs:145 and 310 each call `prepare_document(input_path, max_size_bytes)` independently.
- Remediation: Extend `run_catalog_pipeline`/`run_component_pipeline` with an overload or `PreparedDoc` parameter accepting a pre-built `PolicyDocument`; have `execute()` call `prepare_document` once for the input and share it between the baseline comparison and the pipeline. Keep baseline preparation separate. Tests: existing stable-id warning tests continue to pass; add a unit test asserting a single `prepare_document` call path (e.g., via a doc with a unique marker that would trip twice if re-parsed, or simply rely on behavior equivalence).


### F0320 — valid
- File:lines: `src/cli/profile.rs:91-94`
- Symbols: `execute` (profile CLI handler)
- Category: bug | Severity: medium
- Root cause: `catalog.exists()` is a check-then-act probe that also returns true for directories. A directory path passes validation and fails later inside `build_profile`'s open (or not at all, since `build_profile` doesn't read the catalog — but the misleading typed `FileNotFound` never fires for directories). The window between `exists()` and any subsequent open is a TOCTOU.
- Evidence: line 92-94 `if !catalog.exists() { return Err(ForgeError::FileNotFound { path: catalog.to_path_buf() }); }`.
- Remediation: Replace with `match std::fs::metadata(catalog) { Ok(m) if m.is_file() => {}, Ok(_) => return Err(ForgeError::InvalidArgument(format!("catalog path '{}' is not a regular file", catalog.display()))), Err(e) if e.kind() == NotFound => return Err(ForgeError::FileNotFound { path: catalog.to_path_buf() }), Err(e) => return Err(ForgeError::Io(e)) }`. Add test: `--catalog <directory>` yields InvalidArgument naming the path.

### F0321 — valid
- File:lines: `src/cli/profile.rs:110`
- Symbols: `execute`, `build_profile` (`src/oscal/profile.rs:241` via `sanitize_artifact_path`)
- Category: bug | Severity: medium
- Root cause: `catalog.to_string_lossy()` silently substitutes U+FFFD for non-UTF-8 bytes. The resulting string feeds `build_profile` → `sanitize_artifact_path` → `ProfileImport.href`, producing a permanently corrupted reference in JSON/XML/YAML output (all UTF-8) with no error.
- Evidence: profile.rs:110 `let catalog_str = catalog.to_string_lossy();`.
- Remediation: Replace with `let catalog_str = catalog.to_str().ok_or_else(|| ForgeError::InvalidArgument(format!("catalog path '{}' is not valid UTF-8; refusing to embed a corrupted href", catalog.display())))?;`. On Unix, construct a test using `std::os::unix::ffi::OsStrExt` with an invalid byte to assert the error.

### F0322 — partial
- File:lines: `src/cli/profile.rs:51-57`
- Symbols: `execute`, `parse_set_param_pairs`, `build_modify_section` (`src/oscal/profile.rs:175-189`)
- Category: bug | Severity: medium
- Root cause: The empty-ID check trims for validation (`id.trim().is_empty()`) but forwards the original untrimmed ID to `build_profile`, so `--set-param " ac-1" x` passes validation and emits a whitespace-padded `param-id`. However, the second half of the finding (duplicate IDs "never detected, semantics undeclared") is wrong: `build_modify_section` explicitly documents and tests duplicate aggregation — duplicate `param_id`s merge their values into one `SetParameter` (profile.rs:155-156 doc + test `build_modify_section_duplicate_param_id_aggregated` at line 399). That is declared behavior, not a defect.
- Evidence: profile.rs:52 `if id.trim().is_empty()` uses the trimmed view only for the check; `pairs` retains originals (parsed at 160-167 from raw clap strings).
- Remediation: In the validation loop, trim each ID and store the trimmed form: build `let pairs: Vec<(String, String)> = pairs.into_iter().map(|(id, v)| (id.trim().to_string(), v)).collect();` before the empty check, rejecting empties after trim. Keep documented duplicate aggregation (last-wins is not the semantics; values accumulate). Test: `--set-param " prm " v` produces `param_id == "prm"`.

### F0326 — valid
- File:lines: `src/cli/output.rs:22-24`
- Symbols: `write_output`
- Category: maintainability | Severity: medium
- Root cause: Stdout write errors propagate as raw `ForgeError::Io(e)` with no indication the target was stdout; diagnostics like "No space left on device" appear as generic pipeline failures.
- Evidence: line 24 `Err(e) => return Err(ForgeError::Io(e))` (and the flush arm at line 29).
- Remediation: Wrap both arms: `Err(e) => return Err(ForgeError::Io(io::Error::new(e.kind(), format!("failed writing to stdout: {e}"))))`. No snapshot impact.

### F0327 — valid
- File:lines: `src/cli/output.rs:43`
- Symbols: `write_output`, `write_atomic` (`src/io.rs:16-34`)
- Category: maintainability | Severity: medium
- Root cause: `write_atomic` only embeds the path in its persist error; temp-file creation, `write_all`, and both `sync_all` calls propagate as bare `io::Error` with no target path. The `?` at the CLI boundary loses context.
- Evidence: output.rs:43 `crate::io::write_atomic(path, content.as_bytes())?;` and io.rs:21-30 where only the persist arm is wrapped.
- Remediation: Change to `crate::io::write_atomic(path, content.as_bytes()).map_err(|e| ForgeError::Io(io::Error::other(format!("failed writing output to '{}': {e}", path.display()))))?;`. Alternatively wrap inside `write_atomic` itself so every caller benefits — prefer the io.rs-level fix (wrap each `?` in `write_atomic` with path context) since it is shared plumbing.

### F0330 — valid
- File:lines: `src/cli/export.rs:282-286`
- Symbols: `export_artifact`, `check_file_size` (`src/io.rs:42-45`)
- Category: security | Severity: medium
- Root cause: `check_file_size` stats the path, then `std::fs::read` re-opens it; a file grown/swapped between the two bypasses `MAX_FILE_SIZE` and buffers unbounded bytes.
- Evidence: lines 283-286 stat-then-read pattern.
- Remediation: Open once: `let mut file = std::fs::File::open(input_path)?; let metadata = file.metadata()?; if !metadata.is_file() { return Err(ForgeError::ExportInvalidOscal { detail: format!("'{}' is not a regular file", input_path.display()) }); } if metadata.len() > crate::io::MAX_FILE_SIZE { return Err(...too-large...); } let mut bytes = Vec::with_capacity(metadata.len().min(MAX) as usize); file.take(MAX_FILE_SIZE + 1).read_to_end(&mut bytes)?; if bytes.len() > MAX_FILE_SIZE { return Err(too-large) }`. Preserve the existing UTF-8 and empty checks afterward. Note this mirrors fixes needed in F0358/F0372/F0395/F0401 — a shared `read_bounded(path, limit) -> Result<Vec<u8>, ForgeError>` helper in `src/io.rs` is the clean cross-cutting fix.

### F0333 — valid
- File:lines: `src/cli/export.rs:64-96` (JSON) and 154-186 (YAML)
- Symbols: `deserialize_from_json`, `deserialize_from_yaml_format`
- Category: maintainability | Severity: medium
- Root cause: Both functions duplicate the `detect_model_type` → envelope-cast → error flow verbatim, including the misleading "; use JSON" suffix in the Mapping branch (nonsensical in the JSON branch where the input already is JSON).
- Evidence: lines 92-95 and 182-185 carry identical "not yet supported; use JSON" strings.
- Remediation: Extract `fn oscal_model_from_value(value: serde_json::Value, origin: &str) -> Result<OscalModel, ForgeError>` handling Catalog/ComponentDefinition casts and Profile/Mapping rejections with a single unsupported-type table (drop "; use JSON" or make it format-agnostic). Both callers parse to `serde_json::Value` first (YAML already does) and delegate. Add a test asserting the Profile/Mapping rejection message is identical across JSON and YAML inputs. No snapshot impact (error paths only).

### F0340 — valid
- File:lines: `src/cli/mod.rs:88-89`
- Symbols: `Commands::Convert { max_size }`, `resolve_convert` (`src/config.rs:847`)
- Category: security | Severity: medium
- Root cause: `convert --max-size` is `Option<u64>` with no clap `range(...)`, while `migrate` clamps to `1..=51_200` (cli/mod.rs Migrate arm) and config-derived values are bounded by `parse_range("convert.max-size-mb", ..., 1, 51_200)` (config.rs:523). `resolve_convert` at config.rs:847 short-circuits CLI values via `.or_else`, so `--max-size 0` (confusing "too large" for every file) and `--max-size 4294967296` (disables the guard entirely) are accepted.
- Evidence: cli/mod.rs:89 `max_size: Option<u64>,` with bare `#[arg(long)]` at line 88.
- Remediation: `#[arg(long, value_parser = clap::value_parser!(u64).range(1..=51_200))] max_size: Option<u64>,` matching the Migrate arm. Add a test in the CLI test suite asserting `forge convert --max-size 0` and `--max-size 51201` fail at parse time with clap's range message.

### F0343 — valid
- File:lines: `src/cli/migrate.rs:67-84` (`path_identity`), used by `reject_output_alias` (38-63)
- Symbols: `path_identity`, `reject_output_alias`
- Category: security | Severity: medium
- Root cause: (1) `path.exists()` followed by `path.canonicalize()` is TOCTOU — a path swapped between probes resolves attacker-controlled targets or yields misleading errors. (2) Identity is compared as canonicalized path strings, missing hard links (same inode, different names) — unlike `src/mapping/mod.rs:395-408` `paths_alias`, which already compares dev+ino via `same_file_identity`. (3) The check runs once before `analyze_paths` (which can parse slow PDF/DOCX) and is not re-verified before `write_output`.
- Evidence: migrate.rs:68-71 `if path.exists() { return path.canonicalize()... }`. The repo's own `mapping::paths_alias` proves the inode-comparison pattern exists in-tree.
- Remediation: Drop the `exists()` probe — `canonicalize()` alone distinguishes missing paths (map its error); for identity comparison reuse the `same_file_identity` dev/ino approach (extract it from `src/mapping/mod.rs` into `src/io.rs` or a shared helper and compare metadata dev+ino for existing paths, falling back to canonical-path equality). Re-run the alias check immediately before `write_output` in `execute`. Extend the existing tests `rejects_output_that_aliases_an_input` and add a hard-link alias test (create a hard link of old policy as output; expect rejection).


### F0346 — valid
- File:lines: `src/cli/trace.rs:15`
- Symbols: `execute` (trace CLI), `generate_trace_report` (`src/trace/mod.rs:37-44`)
- Category: bug | Severity: medium
- Root cause: Errors from `generate_trace_report` propagate without identifying which of the two inputs (`artifact` vs `source`) failed: e.g. `ForgeError::Parse("Invalid JSON in artifact: ...")` (trace/mod.rs:43-44 names the artifact but not its path) and raw `ForgeError::Io("stream did not contain valid UTF-8")` for a non-UTF-8 source. With two user paths the diagnosis is ambiguous.
- Evidence: trace.rs:15 `let report = generate_trace_report(artifact, source)?;` — bare `?`. trace/mod.rs:44 `ForgeError::Parse(format!("Invalid JSON in artifact: {e}"))` has no path.
- Remediation: Prefer fixing at the source in `src/trace/mod.rs`: interpolate `artifact_path.display()` into the Parse error and map `read_file` errors with the offending path (read_file already maps NotFound/PermissionDenied with paths; wrap the UTF-8 `Io` case with `path.display()`). Then trace.rs needs no wrapper. Add a test asserting the error message contains the source path for a non-UTF-8 source file.

### F0347 — valid
- File:lines: `src/cli/resolve.rs:59-67`
- Symbols: `execute` (resolve CLI)
- Category: bug | Severity: medium
- Root cause: Validation is extension check + `canonicalize()`, which succeeds for any existing path including directories and symlinks to directories. A directory named `profile.json` passes and later fails inside oscal-cli as an opaque `ForgeError::OscalCliExecution` instead of a typed local error.
- Evidence: resolve.rs:60-67 canonicalize with NotFound/PermissionDenied mapping only; no `is_file()`.
- Remediation: After canonicalize add `if !std::fs::metadata(&canonical_input).map_err(ForgeError::Io)?.is_file() { return Err(ForgeError::InvalidArgument(format!("input '{}' is not a regular file", canonical_input.display()))); }`. Add test: directory named `*.json` → InvalidArgument.

### F0351 — valid
- File:lines: `src/diff/extractor.rs:126-141` (`collect_impl_requirements_from_container`)
- Symbols: `collect_impl_requirements_from_container`, `ControlSnapshot`
- Category: bug | Severity: medium
- Root cause: In component definitions it is legal for multiple components/capabilities to implement the same control-id, but the extractor's `HashMap` insert is last-write-wins: every implementation except the last is silently dropped (a `tracing::warn!` acknowledges the collision without preserving data). Downstream diffs therefore cannot see removals/changes of the discarded requirements.
- Evidence: extractor.rs:126-141 `if map.contains_key(control_id) { tracing::warn!(...) } map.insert(...)`.
- Remediation: Aggregate colliding entries: key by control-id but merge — e.g. keep a `Vec` of descriptions/parts per control-id, or key snapshots by `(container uuid, control_id)` and adapt `compare_controls` to match on control-id across composite keys. Simplest contract-preserving fix: merge `description` into a `\n`-joined aggregate and warn. Add test: two components implementing the same control-id with different descriptions → both visible in the diff (one Changed entry, or two entries). Snapshot impact: diff CLI output tests involving multi-component fixtures may change; review with `cargo insta review`.

### F0352 — valid
- File:lines: `src/diff/extractor.rs:28-34` (`extract_catalog_controls`)
- Symbols: `extract_catalog_controls`, `collect_controls_from_groups`
- Category: bug | Severity: medium
- Root cause: Only `/catalog/groups` is walked; OSCAL catalogs may also carry controls directly under `catalog.controls`. Such controls are silently ignored, producing empty/partial diff maps for valid hand-written catalogs.
- Evidence: extractor.rs:28-34 reads only `json.pointer("/catalog/groups")`; `XmlCatalog`/`OscalCatalog` models both have `controls` fields, confirming root-level controls are supported elsewhere in the crate.
- Remediation: Factor the per-control loop out of `collect_controls_from_groups` into `collect_controls(controls: &[Value], map: &mut HashMap<...>)`; call it for `json.pointer("/catalog/controls")` and for each group's controls. Add test `extract_catalog_root_level_controls` with a catalog JSON containing only `catalog.controls`. No snapshot impact (diff fixtures use groups).

### F0357 — valid
- File:lines: `src/cli/validate.rs:249-252` (`build_round_trip_result`)
- Symbols: `build_round_trip_result`, `detect_model_type`
- Category: bug | Severity: medium
- Root cause: `detect_model_type(original_json).ok()` swallows detection failure, producing `artifact_type = "Unknown"` and dropping `declared_oscal_version`. Since the input already parsed as JSON, a detection failure is meaningful (wrong file type fed to `--round-trip`) and should surface rather than degrade the report to placeholder metadata.
- Evidence: validate.rs:249-252 `.ok()` + `map_or_else(|| "Unknown".to_string(), ...)`.
- Remediation: In `execute_round_trip`, after parsing `original_json`, run `let model_type = validate::detect_model_type(&original_json).map_err(|e| ForgeError::Validation(format!("round-trip input '{}' is not a recognized OSCAL model: {e}", input.display())))?;` and pass it into `build_round_trip_result` (change signature to take `OscalModelType`). Note: this rejects non-catalog/component inputs earlier, which may change behavior for `profile` round-trips — check existing round-trip tests; if profile round-trip is intentionally supported, restrict the error to UnknownModelType/Ambiguous and keep Profile handling. Add test: non-OSCAL JSON → Validation error naming the path.

### F0358 — valid
- File:lines: `src/cli/validate.rs:43-56`
- Symbols: `execute` (validate CLI), `validate::check_file_size`
- Category: security | Severity: medium
- Root cause: SEC-3 size cap is enforced by a stat-like check before `std::fs::read_to_string`; bytes actually consumed are unbounded — a file swapped/grown between check and read bypasses the limit. Also `execute()` reads from the raw `input` path (canonicalization happens only in `execute_round_trip`), so guard and read operate on the same path object but without symlink resolution.
- Evidence: validate.rs:44-53 check_file_size then 56 read_to_string(input).
- Remediation: After reading, re-enforce: `if content.len() as u64 > validate::MAX_VALIDATE_FILE_SIZE (or the same limit constant) { return Err(ForgeError::Validation(format!("artifact file grew past the size limit during read"))) }`. Better: shared bounded-read helper (see F0330) reading through `Take(limit+1)`. Tests: existing size tests unaffected; add a unit test for the post-read re-check using the limit constant directly.

### F0359 — valid
- File:lines: `src/cli/validate.rs:133-140` (`execute_round_trip` extension gate)
- Symbols: `execute_round_trip`
- Category: bug | Severity: medium
- Root cause: The extension gate is case-sensitive (`Some("json")`), rejecting `*.JSON`/`*.Json` even though `forge validate` accepts them and `detect_format` in export lowercases extensions.
- Evidence: validate.rs:134 `Some("json") => {}`.
- Remediation: `Some(ext) if ext.eq_ignore_ascii_case("json") => {}`. Add test: `FILE.JSON` passes the gate (fails later only on oscal-cli availability, so assert the error is not the extension rejection).

### F0360 — valid
- File:lines: `src/cli/validate.rs:44-53, 79-84`
- Symbols: `execute`, `ValidateError`, `run_full_validation`
- Category: maintainability | Severity: medium
- Root cause: Typed `ValidateError` variants are flattened to strings (`other.to_string()`, `ForgeError::Validation(e.to_string())`, `ForgeError::SchemaValidation(e.to_string())`), destroying the error-class/source chain needed for exit codes or programmatic handling.
- Evidence: validate.rs:44-53 match with `other => ForgeError::Validation(other.to_string())`; line 83-84 `.map_err(|e| ForgeError::SchemaValidation(e.to_string()))`.
- Remediation: Add `ForgeError::FullValidation(#[from] ValidateFullError)`-style variants or map each typed variant explicitly (FileTooLarge→FileTooLargeWithMaxSize guidance path, FileRead→Io with path). At minimum keep `#[source]` via `io::Error::other` wrappers. Update display/exit-code tests accordingly. This is a refactor with behavioral parity on messages — keep user-facing strings stable to avoid test churn.

### F0365 — valid
- File:lines: `src/diff/engine.rs:99-110` (parts prose comparison)
- Symbols: `compute_field_changes`
- Category: bug | Severity: medium
- Root cause: Positional (index-based) comparison of `parts_prose`: inserting/deleting/reordering one statement shifts every subsequent index, reporting unrelated statements as changed. Out-of-range sides render as `""`, conflating deletion with emptied content except at the tail.
- Evidence: engine.rs:99-110 `for i in 0..max_len { old.parts_prose.get(i).map_or("", ...) ... }`.
- Remediation: Trim common prefix/suffix before the positional scan (as the finding's patch shows), emitting `statement[i]` only for the genuinely shifted region; document that middle insertions still shift indices and prefer anchoring on part ids at snapshot time (extract part `id` alongside prose in `collect_statement_prose`, extractor.rs:72-83) as the durable fix. Add tests: prepend a statement → only one change entry after prefix trim; mid-list insertion → bounded entry count.

### F0366 — duplicate (of F0369)
- File:lines: `src/diff/engine.rs:82-89`
- Symbols: `compute_field_changes`, `FieldChange`
- Category: bug | Severity: medium
- Rationale: The `clone().unwrap_or_default()` None-vs-`Some("")` collapse is a direct symptom of the empty-string sentinel encoding documented in `FieldChange` (`src/diff/types.rs:50-60`), which F0369 targets as the root design. Fixing F0369 (`Option<String>` old/new values) eliminates this case; the engine then compares raw `Option<String>` and stores them directly.

### F0369 — valid
- File:lines: `src/diff/types.rs:46-60`
- Symbols: `FieldChange`
- Category: maintainability | Severity: medium
- Root cause: Encoding added/removed via empty-string sentinels conflates absence with present-but-empty: `{old_value: "", new_value: ""}` is simultaneously "added" and "removed" per the documented rule and indistinguishable from a no-op. Renderers can misclassify change kind.
- Evidence: types.rs:49-60 doc comments define the sentinel convention.
- Remediation: Change `old_value`/`new_value` to `Option<String>` (`None` = field absent); update `compute_field_changes` (engine.rs:82-97) to store raw Options and the formatter (formatter.rs:84-91) to render `None` as `(absent)`. Update the formatter test `test_format_changed_with_field_changes` and any snapshot exercising FieldChange output.

### F0370 — valid
- File:lines: `src/diff/types.rs:87-91`
- Symbols: `DiffEntry::Changed { uuid_changed }`
- Category: maintainability | Severity: medium
- Root cause: `uuid_changed` is derivable from `old_uuid != new_uuid` but stored as an independently assignable bool, permitting inconsistent variants (flag true with equal UUIDs, or false with differing UUIDs — the latter bypassing the `UuidChanged` contract and undercounting `uuid_changes` for consumers matching on it).
- Evidence: types.rs:90-91 field; engine.rs:55-68 constructs both arms manually.
- Remediation: Remove the field from `Changed` and add `impl DiffEntry { pub fn uuid_changed(&self) -> bool }` deriving from the payload for `Changed`/`UuidChanged`. Update engine.rs construction sites, formatter.rs:94 (the `if *uuid_changed` arm → `if entry.uuid_changed()`), and all tests constructing `Changed` literals (engine.rs and formatter.rs tests). Snapshot impact: none (Display output unchanged).


### F0372 — valid
- File:lines: `src/diff/canonical.rs:113-128` (`parse_artifact`)
- Symbols: `parse_artifact`
- Category: security | Severity: medium
- Root cause: `fs::metadata(path)` drives both the `is_file()` and `MAX_FILE_SIZE` guards, then `read_to_string(path)` re-resolves and re-opens the path. A swap/symlink-retarget between the two reads a special or oversized file, defeating both guards.
- Evidence: canonical.rs:115-128 stat-based checks then separate read.
- Remediation: Open once; validate metadata on the held handle; read via `file.take(MAX_FILE_SIZE + 1)` and reject when the buffered text exceeds the limit (see the finding's patch). Error strings keep the existing `role_name` phrasing so tests like `detects_policy_content_change_without_returning_content` are unaffected. Add test: a file grown between open and read is bounded (simulate by asserting the take-based path rejects >MAX content).

### F0373 — valid
- File:lines: `src/diff/canonical.rs:170-183` (exclusions in `canonicalize`), constant at line 20
- Symbols: `canonicalize`, `DRIFT_COMPARISON_CONTRACT_VERSION`
- Category: maintainability | Severity: medium
- Root cause: The v1 exclusion set exists only as imperative `remove` calls plus a prose warning; nothing structurally ties it to `DRIFT_COMPARISON_CONTRACT_VERSION`, so the two can silently desync. `DriftComparison` also doesn't embed the contract version — `cli/drift.rs:48-53` stamps it from the global constant at render time, trusting the caller.
- Evidence: canonical.rs:174-183 `root.remove("uuid")` / `metadata.remove("last-modified")` with a comment-only coupling; cli/drift.rs:52 reads the global.
- Remediation: Declare `const EXCLUDED_FIELDS: &[&[&str]] = &[["uuid"], ["metadata", "last-modified"]];` adjacent to the version constant and have `canonicalize` walk it (pointer-style removal); add a compile-time `const _: () = assert!(...)` or a unit test asserting the table is non-empty and matches the documented contract. Optionally add `contract_version: u8` to `DriftComparison` populated from the constant so reports self-describe. Update canonical.rs tests if the walk changes ordering (it should not).

### F0377 — valid
- File:lines: `src/diff/formatter.rs:33-36` (gate) vs 51-52, 69-70, 104-105, 121-122 (section re-counts)
- Symbols: `format_diff_report`, `DiffSummary::has_changes`
- Category: maintainability | Severity: medium
- Root cause: The early-return gate consults `summary.has_changes()` while every section heading independently re-counts `report.entries`. A desynced `DiffReport` (summary says no changes, entries non-empty) hides ALL detail rows behind "No differences found."; the inverse prints self-contradicting counters ("Added: 0" under "Added (3)").
- Evidence: formatter.rs:34 `if !s.has_changes()` early return; format_added_section at 52-53 filters entries independently.
- Remediation: Make entries authoritative: replace the gate with `if report.entries.is_empty()` OR derive summary counts from entries at format time; additionally add `debug_assert_eq!` cross-checks (entries-filtered counts vs summary fields) at the top of `format_diff_report`. Add a unit test building a desynced report (summary zero, one Added entry) asserting the detail is still rendered (debug builds panic on the assert — acceptable per finding).

### F0381 — valid
- File:lines: `src/diff/formatter.rs:84-91`
- Symbols: `format_changed_section`, `FieldChange`
- Category: maintainability | Severity: medium
- Root cause: Old/new values are written verbatim inside quotes. Values come from control titles/descriptions/parts prose — potentially large multi-line free text; embedded newlines break the one-line `field: old -> new` layout, and a value containing U+2192 makes output ambiguous for grep/split consumers.
- Evidence: formatter.rs:86-90 `writeln!(out, "      {}: \"{}\"  \u{2192}  \"{}\"", fc.field_name, fc.old_value, fc.new_value)`.
- Remediation: Add `fn one_line(value: &str, max_chars: usize) -> String` collapsing `\n`/`\r`/`\t` to spaces and truncating with a `[...truncated]` marker (cap ~200 chars); apply to both values at the use site. Update formatter tests to cover a multi-line description value (assert single-line output). No snapshot impact (existing fixtures use single-line values).

### F0385 — invalid
- File:lines: `src/config.rs:612-627` (`resolve_inside_root`)
- Symbols: `resolve_inside_root`, `reject_windows_device_name`, `ensure_symlink_containment`
- Category: bug (claimed) | Severity: medium (claimed)
- Rationale: The finding claims inputs like `CON:stream` on Unix get the misleading "outside the project root" message because the device-name check runs after containment. That is wrong: `CON:stream` is a relative single-component path; `project_root.join("CON:stream")` stays inside the root, so the containment check at line 621-624 passes, and `reject_windows_device_name` at line 625 then fires with the correct "reserved device name" diagnostic. The test `windows_reserved_device_names_rejected_cross_platform` (config.rs:1215-1234) asserts exactly `contains("reserved device name")` for `CON:stream` and passes. Only genuinely-escaping paths produce the containment message. No defect.

### F0387 — invalid
- File:lines: `src/config.rs:241-254` (`env_config_path`)
- Symbols: `env_config_path`, `env_jobs`
- Category: bug (claimed) | Severity: medium (claimed)
- Rationale: The finding claims `FORGE_CONFIG` only rejects literally-empty values while `FORGE_JOBS` rejects post-trim empties. Current code at config.rs:243 already uses `Ok(value) if value.trim().is_empty() => Err(...)` — whitespace-only values are rejected with the actionable unset-the-variable hint, symmetric with `FORGE_JOBS` (config.rs:270). The defect does not exist at HEAD.

### F0388 — partial
- File:lines: `src/config.rs:459-473` (`edit_distance`), 430-457 (`closest_key`), 383-427 (`check_unknown_keys`)
- Symbols: `edit_distance`, `closest_key`, `check_unknown_keys`
- Category: performance | Severity: medium
- Root cause (valid part): `edit_distance` is unbounded O(len(a)×len(b)) with two heap `Vec<char>` allocations per call, invoked once per known key per unknown key. Keys come from an attacker-controlled config file (up to `MAX_CONFIG_SIZE` = 1 MiB, config.rs:32), so a multi-hundred-KB bogus key forces ~50 × N² char comparisons per unknown key — a real CPU-exhaustion vector on merely loading config.
- Invalid part: The finding's arithmetic ("~tens of GB of work from running `forge --help`") overstates: `--help` exits before config loading, and 100k² = 10^10 char ops per candidate is upper bound; also early termination (d>2 prune) is absent but Levenshtein still short-rows. The DoS direction stands; the stated trigger and magnitude do not.
- Remediation: In `closest_key`, skip candidates when `key.chars().count().abs_diff(candidate.chars().count()) > 2` (length difference alone exceeds the distance-2 threshold) and cap absolute key length (e.g. reject suggestion search for keys > 64 chars). Optionally reuse row buffers across candidates. Add a test with a 100KB unknown key asserting `load_str` returns the unknown-key error promptly (no timeout).

### F0394 — valid
- File:lines: `src/export/yaml.rs:29-38`
- Symbols: `deserialize_from_yaml`
- Category: security | Severity: medium
- Root cause: The generic entry point accepts arbitrarily sized/nested YAML with no documented trust assumption; deeply nested structures or anchor/alias expansion can exhaust stack/memory, and neither serde nor the YAML layer imposes limits. The pipeline has FileTooLarge machinery upstream but this function's contract says nothing.
- Evidence: yaml.rs:35-38 bare `serde_yaml::from_str` wrapper with only error-mapping docs.
- Remediation: Add a `# Security` doc section stating callers must enforce byte-size caps and input provenance before calling (cite `crate::io::MAX_FILE_SIZE` / FileTooLarge); optionally add a length guard (`yaml.len() > MAX` → Serialization error) for defense in depth. Doc-only fix is acceptable per the finding; no snapshot impact.

### F0395 — valid
- File:lines: `src/diff/mod.rs:80-93` (`read_diff_file`)
- Symbols: `read_diff_file`, `check_file_size`
- Category: security | Severity: medium
- Root cause: `check_file_size` compares a one-shot `metadata().len()`, then `read_to_string` reads unbounded — a file grown between stat and read bypasses `MAX_FILE_SIZE`. Additionally `Err(e) => DiffError(e.to_string())` flattens structured `FileTooLarge`/`Io` errors, dropping the source chain.
- Evidence: diff/mod.rs:83-88 match on check_file_size with string flattening at line 86, then unbounded read at 89.
- Remediation: Replace with a bounded read: open the file, wrap in `std::io::Take` limited to `MAX_FILE_SIZE + 1`, `read_to_string`, reject when consumed bytes exceed the limit; propagate open/read errors with `path.display()` context preserving kind. Keep the NotFound → "File not found: '{}'" mapping (tests assert it: `test_missing_file_error`). Part of the shared bounded-read helper recommended under F0330.

### F0396 — valid
- File:lines: `src/diff/mod.rs:32-45` (`diff_artifacts`)
- Symbols: `diff_artifacts`, `read_diff_file`
- Category: performance | Severity: medium
- Root cause: Both artifacts' raw strings stay live across both `serde_json::Value` parses, and both trees stay live through extraction: peak ≈ 2 texts + 2 DOM trees. With the 50 MB cap per file a single invocation can transiently approach ~1 GB.
- Evidence: diff/mod.rs:34-35 reads both texts before parsing either; 38-44 parses both before extracting.
- Remediation: Extract a `fn load_snapshot(path: &Path) -> Result<(ArtifactType, HashMap<String, ControlSnapshot>), ForgeError>` that reads, parses, detects type, extracts, and drops the String+Value before returning; call it for old then new; compare types afterward (error message format must match `test_type_mismatch_error`'s substring expectations: "mismatch"/"different"). Tests unchanged.

### F0397 — valid
- File:lines: `src/diff/mod.rs:107-110` (`to_artifact_type`)
- Symbols: `to_artifact_type`, `detect_model_type`, `ValidateError::AmbiguousArtifact` (`src/validate/mod.rs:93-99`)
- Category: bug | Severity: medium
- Root cause: The `Err(_)` arm collapses every `ValidateError` into "not a recognized OSCAL artifact" — including `AmbiguousArtifact` (two OSCAL root keys), where the user is told the file is unrecognized despite it being recognized twice. Hides the detail list needed to fix the input.
- Evidence: diff/mod.rs:107-110 `Err(_) => ..."not a recognized OSCAL artifact..."`; validate/mod.rs:135 returns `AmbiguousArtifact { detail: found.join(", ") }`.
- Remediation: `Err(e) => Err(ForgeError::DiffError(format!("'{}': expected a single supported OSCAL root key ('catalog' or 'component-definition'): {e}", path.display())))`. The existing test `test_non_oscal_json_returns_error` asserts substrings "OSCAL" or "recognized" — keep one of those words in the message (e.g. "...single supported OSCAL root key..." contains "OSCAL"). Add test: JSON with both `catalog` and `component-definition` keys → error mentions both.


### F0401 — valid
- File:lines: `src/framework/disposition.rs:62-72` (`load`)
- Symbols: `load`, `regular_file_metadata` (`src/io.rs:56-58`), `MAX_DISPOSITION_BYTES` (line 12)
- Category: security | Severity: medium
- Root cause: `regular_file_metadata` stats the path, then `std::fs::read` opens it again; a file replaced/grown between the two steps bypasses `MAX_DISPOSITION_BYTES` (the parser's strict limits cover depth/per-string size, not total input).
- Evidence: disposition.rs:63-71 metadata check then separate read.
- Remediation: After reading, re-check `if bytes.len() as u64 > MAX_DISPOSITION_BYTES { return Err(error(format!("disposition file exceeds the {MAX_DISPOSITION_BYTES} byte limit"))); }` before `parse`. Alternatively read via a single opened handle with `Take(MAX + 1)`. Add a unit test asserting `parse`/`load` rejects oversized byte slices at the length boundary.

### F0405 — valid
- File:lines: `src/error.rs:383-388` (exit-code category doc), match at 409-455
- Symbols: `exit_code`
- Category: documentation | Severity: medium
- Root cause: The doc header declares 1=Input/IO, 2=Parse/Structure, 3=Validation/Config, 4=external-dependency, but the match below places `OscalCliExecution`/`OscalCliTimeout` in 1 (siblings `OscalCliNotFound`/`NotFunctional` are 4), `SchemaValidation` in 3, `MissingRequiredArgument` in 2, and seven review/drift sentinels in 1. CI gates on these codes; the grouping-by-proximity docs mislead.
- Evidence: error.rs:385-388 doc vs 411-455 actual arms.
- Remediation: Rewrite the doc header per the finding's suggested text: 1 = Input/IO + export + argument + batch + oscal-cli execution/timeout + review/drift sentinels + RoundTripFailed; 2 = parse/structure/build + MissingRequiredArgument + diff/analysis computation failures (DiffError, MigrationError, SspBuild, MappingBuild, Lifecycle, ApplicabilityAnalysis, FrameworkImpact, TraceUnsupportedArtifact, AmbiguousArtifact); 3 = Validation/Config/SchemaValidation; 4 = OscalCliNotFound/NotFunctional. Keep code unchanged. Update the doctest examples if needed (they still hold).

### F0406 — valid
- File:lines: `src/error.rs:343-357` (sentinel variants), `src/main.rs:22-30` (parallel match)
- Symbols: `ForgeError::DiffHasChanges`, `DriftDetected`, `MigrationHasChanges`, `MappingReviewRequired`, `LifecycleActionRequired`, `ApplicabilityReviewRequired`, `FrameworkReviewRequired`
- Category: maintainability | Severity: medium
- Root cause: Successful-but-actionable outcomes are encoded as errors with empty `#[error("")]` Display; generic consumers (logging, anyhow wrapping, `{:?}`) print a blank line. The sentinel set is duplicated as a parallel match in main.rs:22-29, so adding a sentinel touches three places (variant + exit_code arm + main.rs arm) and a miss exits 1 with zero visible output.
- Evidence: error.rs:346 `#[error("")] DiffHasChanges,` (and 351, 356); main.rs:22-29 the sentinel match. Test `diff_has_changes_display` (error.rs:888-891) asserts the empty string — codifying the fragility.
- Remediation: Give each sentinel a non-empty message (e.g. `#[error("diff detected changes")]`, `#[error("drift detected between committed and generated artifacts")]`, etc.). main.rs continues to suppress re-printing (it matches before the generic `eprintln!`), so user-visible CLI output is unchanged except for generic Display consumers. Update `diff_has_changes_display` and `drift_detected_has_no_message_and_exit_code_one` tests accordingly.

### F0408 — valid
- File:lines: `src/error.rs` tests module (ends line 919); exit_code match at 409-455
- Symbols: `exit_code`
- Category: test | Severity: medium
- Root cause: Many variants lack exit-code regression coverage: ExportUnsupportedExtension, ExportNoExtension, ExportInvalidOscal, ExportEmptyInput, InvalidArgument, OcrNotSupported, MappingReviewRequired, FrameworkReviewRequired, MigrationHasChanges, MigrationError, SspBuild, MappingBuild, FrameworkImpact, and SchemaValidation (the sole exit-3 occupant besides Validation/Config, completely unasserted — `exit_code_validation_config_errors_return_3` at 621-624 covers only Validation and Config). Moving any of these between arms passes CI.
- Evidence: grep of test names confirms no assertions for the listed variants; the matrix tests at 589-624 cover only the listed subsets.
- Remediation: Add a data-driven test `exit_code_matrix_covers_remaining_variants` enumerating `(ForgeError, expected)` pairs per the finding's sketch (MigrationError/SspBuild/MappingBuild/FrameworkImpact → 2; SchemaValidation → 3; the rest → 1). Verify expectations against the current match (error.rs:411-455) before asserting.

### F0412 — valid
- File:lines: `src/framework/manifest.rs:106-116` (duplicate detection in `validate`)
- Symbols: `validate`, `MappingDependency`
- Category: security | Severity: medium
- Root cause: Duplication is detected with a byte-exact `BTreeSet<&Path>`, rejecting only lexically identical spellings. "Controls.json" and "controls.json" pass validation yet resolve to the same file on Windows/macOS/case-folding shares, double-counting a collection in impact attribution. The seed set also never includes `$.old.artifact`/`$.new.artifact`/resolved_catalog paths, so a mapping entry can shadow primary evidence.
- Evidence: manifest.rs:112 `if !paths.insert(dependency.artifact.as_path())`. Note the runtime loader (`load_mapping_references`, analysis.rs:644-650) does run `paths_alias` (dev/ino) per pair — so this is defense-in-depth at manifest validation time, not the only guard; still valid as a lexical-level gap.
- Remediation: Normalize the key: `let key = dependency.artifact.to_str().ok_or(...)?.to_ascii_lowercase();` insert into `BTreeSet<String>` (validate_json_path already guarantees relative local JSON paths). Additionally seed the set with `manifest.old.artifact`, `manifest.new.artifact`, and any `resolved_catalog` companions so collisions against primary evidence are rejected at validation. Add tests: case-variant duplicate rejected; mapping entry colliding with old.artifact rejected.

### F0414 — valid
- File:lines: `src/framework/manifest.rs:219-241` (tests module)
- Symbols: `validate`, `validate_resource`, tests module
- Category: test | Severity: medium
- Root cause: Unit coverage exercises only `validate_json_path`. The stateful contracts in `validate()`/`validate_resource()` — schema_version pinning, old/new type agreement, Mapping Collection bounds and duplicate rejection, resolved_catalog_attestation strictly `Some(true)` for Profiles, forbidden resolved-catalog companion fields on Catalogs, prior_report/disposition_file pairing, pinned OSCAL version — are untested at unit level. (Integration tests in tests/framework_impact_cli_test.rs exercise some end-to-end, but unit-level rule isolation is missing.)
- Evidence: manifest.rs:220-241 imports only `validate_json_path`; single test `manifest_paths_are_relative_and_cannot_traverse_parent_directories`.
- Remediation: Add table-driven tests using a baseline-builder helper (construct a minimal valid `ImpactManifest` via JSON, apply per-case overrides, call `parse`/`validate`, assert error substring per rule): unsupported schema_version; old/new type mismatch; missing resolved_catalog_attestation for Profile; attestation Some(false); resolved_catalog present for Catalog; prior_report without disposition_file and vice versa; mapping_collections over limit; duplicate artifact paths. Assert per-rule message substrings for stability.


### F0417 — valid
- File:lines: `src/framework/analysis.rs:162-163` (ordering), 214-223 (`apply_filters`), 347-366 (`update_disposition_summary`)
- Symbols: `analyze`, `update_disposition_summary`, `apply_filters`
- Category: bug | Severity: medium
- Root cause: `update_disposition_summary` counts the pre-filter `findings` array (line 162 runs before `apply_filters` at 163); `apply_filters` then partitions matched findings into `findings`/`filtered_out_findings` without feeding back into the summary. Consumers reading `undispositioned`/`dispositioned_*` alongside the emitted (filtered) findings see contradictory numbers whenever filters apply — e.g. 40 findings, 35 hidden by `--group` leaves 5 visible while `undispositioned` still claims 35.
- Evidence: analysis.rs:162-163 ordering; `update_disposition_summary` at 347-366 iterates `report.findings` (pre-filter); `apply_filters` at 214-223 partitions afterward.
- Remediation: Recompute the disposition summary after filtering (swap the two calls: `apply_filters` then `update_disposition_summary`), OR split summary into `visible_*`/`hidden_*` buckets. If swapping, verify the report contract intent — the `ImpactFilters` doc (model.rs:27-29) says "Summary counts always describe the complete validated analysis", which contradicts recompute-after-filter; reconcile by documenting whichever semantics win and add a unit test: 2 findings, 1 filtered out, assert summary arithmetic matches the chosen semantics. This doc-vs-code tension is the crux — the fix must update the model.rs doc comment too.

### F0418 — valid
- File:lines: `src/framework/analysis.rs:1128-1141` (`finding`), seed uses old/new `raw_sha256` + subject + class
- Symbols: `finding`, `apply_dispositions` (272-326)
- Category: maintainability | Severity: medium
- Root cause: `finding_id` is UUIDv5 over `REPORT_SCHEMA_VERSION + old/new raw_sha256 (+resolved) + subject + change_class + reason_code + context.identity`. Two disjoint analyses can mint identical ids when a lineage overlaps: run A (old=X, new=Y) Removed-finding for control C and run B (old=Y, new=Z) Added-finding for C share no hash fields only if change_class/reason differ — but matching (class, reason, identity) tuples across chained comparisons collide. `apply_dispositions` keys purely on `finding_id` (line 317), so dispositions from one lineage silently reattach to unrelated findings in another without collision signal. `validate_prior_report` (329-344) checks old/new evidence equality against the prior report, which mitigates cross-lineage attachment when the prior report is honest — but the finding's concern (no run/pair identifier baked into the seed) stands for chained re-runs where the operator supplies the previous run's report as prior.
- Evidence: analysis.rs:1128-1136 seed composition; 317-318 `current_finding_indexes.get(&disposition.finding_id)` → assign.
- Remediation: The prior-report evidence check already ties dispositions to a specific old/new pair; harden by also embedding a comparison-scoped token — e.g. include both `old.evidence.raw_sha256` and `new.evidence.raw_sha256` AND a prior-report-linkage field (prior_report_sha256 when present) in the seed, or document that finding_ids are only valid within a fixed (old,new) pair and add a test asserting a disposition for a same-id finding from a different-pair prior report is rejected. Given validate_prior_report already enforces pair equality, the minimal fix is documentation + a regression test proving cross-pair disposition reuse fails.

### F0420 — valid
- File:lines: `src/framework/analysis.rs:304-321` (`apply_dispositions`)
- Symbols: `apply_dispositions`, `current_finding_indexes`
- Category: bug | Severity: medium
- Root cause: `current_finding_indexes` is built via `.map(|(index, finding)| (finding.finding_id.clone(), index)).collect()` — duplicate finding_ids within current findings silently fold (last index wins), so a second matching disposition record overwrites the first assignment without detection. (The disposition file itself rejects duplicate finding_ids at disposition.rs:103-105, so within-file duplication is guarded; the gap is duplicate ids in `report.findings`, which the generator can produce if two finding builders collide — the seed analysis in F0418 shows this is possible for chained comparisons.) Overlapping-but-not-identical ids go to `prior_only_dispositions` without any diagnostic.
- Evidence: analysis.rs:304-309 `.collect()` into BTreeMap (silent last-wins on key collision); 317-321 assignment.
- Remediation: Build the map with collision detection: replace `.collect()` with an explicit loop using `BTreeMap::insert` and `if map.insert(id, index).is_some() { return Err(impact_error(format!("analysis produced duplicate finding id '{id}'"))) }`. Optionally emit a `tracing::warn!` with count when `prior_only_dispositions` grows past a threshold. Add unit test: report with two findings sharing an id + matching disposition → error.

### F0422 — valid
- File:lines: `src/framework/analysis.rs:612-616` (`load_applicability` tail)
- Symbols: `load_applicability`, `regular_file_metadata`
- Category: security | Severity: medium
- Root cause: The `prepared.input_paths` from the applicability subsystem receive only a stat-based existence/regular-file check (`regular_file_metadata`) after `prepare_analysis` already consumed them — contents aren't re-hashed, so a swapped/truncated file between prepare and verify passes. This contrasts with Mapping Collection inputs, which get SHA256 + size + strict parse (analysis.rs:644-660), and with `same_resource_identity` (622-629) which cross-checks applicability's framework evidence against the old resource by hash.
- Evidence: analysis.rs:612-615 loop over `prepared.input_paths` calling only `regular_file_metadata`.
- Remediation: Mitigation is largely structural — `prepare_analysis` already hashes its inputs into the report (`manifest_sha256`, mapping `raw_sha256` per applicability/mod.rs:133,269) and `load_applicability` cross-checks `applicability_mapping_hashes != portfolio.target_collection_sha256s` (585-589) and `same_resource_identity` (571). The residual window is between prepare and the stat re-check. Fix: drop the redundant post-hoc stat loop (it adds no integrity) OR capture `(path, sha256, size)` tuples from `prepare_analysis` and re-verify hashes here. Also add `paths_alias` comparison for applicability inputs mirroring the mapping-collection loop (analysis.rs:645-650). Choose the hash-capture option for parity with the mapping branch.

### F0424 — valid
- File:lines: `src/framework/mod.rs:134-136` (markdown), 202-204 (html), 383-385 (github)
- Symbols: `render_markdown`, `render_html`, `render_github_annotations`, `markdown_escape`, `html_escape`
- Category: security | Severity: medium
- Root cause: SHA columns (`change.old_sha256`/`new_sha256`) are emitted without `markdown_escape`/`html_escape` in all three renderers. Safe today only because analysis derives them from `single_sha256()`/`sha256()` hex output, not manifest strings — a fragile invariant. Any future change propagating a manifest-derived value through those fields turns all three tables into injection vectors with no compiler signal. The finding's point 1 (markdown `\n` → `&#10;` asymmetry with html literal newlines) is subsumed by F0425.
- Evidence: mod.rs:135-136, 203-204, 384-385 all emit `.as_deref().unwrap_or("none")` unescaped while adjacent columns use the escape helpers.
- Remediation: Wrap all three sites: `change.old_sha256.as_deref().map_or_else(|| "none".to_owned(), markdown_escape)` (and html_escape/github variants). Better: enforce at the type level with a `Sha256Hex(String)` newtype whose only constructor computes from bytes, making accidental manifest propagation impossible. Add tests: a ControlChange with a pipe-containing fake sha renders escaped in markdown and does not split the html row.

### F0425 — valid
- File:lines: `src/framework/mod.rs:268-281` (`html_escape`)
- Symbols: `html_escape`
- Category: security | Severity: medium
- Root cause: `html_escape` encodes `&<>"'` but not control characters (`\n`, `\r`, NUL). Manifest-supplied strings (document_version, control ids, dependency_path segments) are emitted inside `<td>...</td>` cells with newlines intact, letting an attacker-crafted value restructure the rendered HTML (row splitting) — the same threat the `|` escaping guards against in Markdown. GitHub annotation rendering already percent-encodes %0A/%0D (github_data, mod.rs:320-322), showing the threat model is acknowledged elsewhere.
- Evidence: mod.rs:268-281 match arms lack `\n`/`\r`; render_html emits manifest fields via html_escape into table cells.
- Remediation: Add `'\n' => escaped.push_str("&#10;"), '\r' => escaped.push_str("&#13;"),` to `html_escape`. Add test: finding with dependency_path containing `\n` renders without a literal newline inside the td.

### F0426 — valid
- File:lines: `src/framework/mod.rs:324-326` (`github_property`), used at 299-316 (`render_github_annotations`)
- Symbols: `github_property`, `github_data`, `render_github_annotations`
- Category: maintainability | Severity: medium
- Root cause: `github_property` escapes `:` and `,` on top of `github_data`, but the annotation message is built with `github_data` only (mod.rs:309 `let message = github_data(&format!(...))`), so a manifest-controlled `finding_id`/`subject_id` containing `,` or `:` can smuggle additional annotation properties or corrupt the `title=..::message` structure. The title IS run through github_property (line 299) but the message is not.
- Evidence: mod.rs:309-316 message construction uses github_data; GitHub runner docs require `,`/`:` escaping in both title and message property values.
- Remediation: Change line 309 to `let message = github_property(&format!(...))` so delimiters cannot be smuggled into annotation payloads. Add test: finding with subject_id containing `,` → rendered annotation has `%2C`, no raw comma in message.

### F0427 — valid
- File:lines: `src/framework/mod.rs:440-444` (`paths_alias` wrapper)
- Symbols: `paths_alias` (framework wrapper), `crate::mapping::paths_alias`, `ForgeError::MappingBuild`
- Category: maintainability | Severity: medium
- Root cause: The wrapper strips the "Control Mapping build error: " prefix from `ForgeError::MappingBuild`'s Display string (error.rs:181-182) via string surgery. If that Display wording ever changes upstream, errors silently stop being stripped and leak the wrong variant prefix into `ForgeError::FrameworkImpact` messages.
- Evidence: mod.rs:441-443 `impact_error(error.to_string().replace("Control Mapping build error: ", ""))`; error.rs:181 `#[error("Control Mapping build error: {0}")]`.
- Remediation: Match on the typed variant instead: `crate::mapping::paths_alias(left, right).map_err(|error| match error { ForgeError::MappingBuild(msg) => impact_error(msg), other => impact_error(other.to_string()) })`. Same pattern applies to `strip_applicability_error_prefix`/`strip_error_prefix` call sites (analysis.rs:173, 570, 647) — convert each to typed matching. Add a test asserting the framework error message for an aliased path contains no "Control Mapping build error" prefix.


### F0439 — valid
- File:lines: `src/export/xml_deserializer.rs:415-431` (`convert_control_implementation`)
- Symbols: `convert_control_implementation`, `ControlImplementation`
- Category: bug | Severity: medium
- Root cause: `Uuid::try_parse(&xml.uuid)` validates but the parsed value is discarded — the raw string (which may carry braces, `urn:` prefix, uppercase hex) is stored into `ControlImplementation.uuid`, while sibling `convert_capability` (line 398-412) normalizes via `uuid.to_string()`. Equally-validated identifiers thus round-trip with divergent representations in the same document, and serializers re-emit the non-canonical forms.
- Evidence: xml_deserializer.rs:417-419 parse-and-discard; 427 `uuid: xml.uuid`.
- Remediation: Bind the parse result: `let uuid = Uuid::try_parse(&xml.uuid).map_err(...)?;` and store `uuid: uuid.to_string()` at line 427, matching `convert_capability`. Add round-trip test: control-implementation with braced/uppercase UUID input serializes to lowercase-hyphenated form.

### F0440 — valid
- File:lines: `src/export/xml_deserializer.rs:434-447` (`convert_implemented_requirement`)
- Symbols: `convert_implemented_requirement`, `ImplementedRequirement`
- Category: bug | Severity: medium
- Root cause: Same discarded-validation pattern as F0439: parsed `Uuid` dropped, raw non-canonical string stored into `ImplementedRequirement.uuid` (line 442), while `convert_capability` normalizes.
- Evidence: xml_deserializer.rs:437-439 parse-and-discard; 442 `uuid: xml.uuid`.
- Remediation: Bind and store `uuid.to_string()` as in F0439. Extend the round-trip test with an implemented-requirement UUID in braced form.

### F0441 — valid
- File:lines: `src/export/xml_deserializer.rs:368-375` (`convert_catalog`), 378-394 (`convert_component`), 451-468 (`convert_component_definition`)
- Symbols: `convert_catalog`, `convert_component`, `convert_component_definition`
- Category: bug | Severity: medium
- Root cause: UUID validation is partial: resource, capability, control-implementation, and implemented-requirement UUIDs go through `Uuid::try_parse`, but the root catalog/component-definition `@uuid` and each component `@uuid` flow raw into the model unchecked (lines 370, 386, 457). Malformed identifiers enter `OscalCatalog.uuid`/`DocumentaryComponent.uuid`, get re-emitted by exporters, and undermine the artifact's OSCAL validity.
- Evidence: convert_catalog line 370 `uuid: xml.uuid` with no parse; convert_component line 386 same.
- Remediation: In all three functions, parse and store canonical: `let uuid = Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal { detail: format!("invalid UUID in catalog/component: '{}' — {e}", xml.uuid) })?; uuid: uuid.to_string()`. Note: the XXE test fixture uses `uuid="test"` on the catalog root (xml_deserializer.rs:574) — after this fix it will fail on the UUID before entity handling; update that fixture to a valid UUID or assert the new rejection. Also update `deserialize_catalog_xml_fixture` expectations if fixture UUIDs are already canonical (they are: "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d" normalizes to itself, so only non-canonical inputs change).

### F0442 — valid
- File:lines: `src/export/xml_deserializer.rs:280-286` (`convert_link`), `OscalLink` in src/oscal
- Symbols: `convert_link`, `OscalLink`
- Category: bug | Severity: medium
- Root cause: Missing `@rel` is silently fabricated as `"reference"` (line 283 `xml.rel.unwrap_or_else(|| "reference".to_string())`). A third-party `<link href="#x"/>` deserializes asserting `rel="reference"`, and re-serialization emits an attribute the source never had — violating the lossless round-trip goal and potentially changing semantics for consumers keying on `rel`.
- Evidence: xml_deserializer.rs:282-284.
- Remediation: Widen `OscalLink.rel` to `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]`, store `xml.rel` directly, and update the XML serializer `write_link` (xml_serializer.rs:117-134) to emit `rel` only when present. This is a model change — audit all `OscalLink` constructors (grep `rel:` across src/ and tests/) and update round-trip snapshots. If the model change is too invasive, at minimum document the substitution on `convert_link` and `OscalLink.rel` (the finding accepts documentation as minimum).

### F0443 — valid
- File:lines: `src/export/xml_deserializer.rs:288-297` (`convert_part`), line 291
- Symbols: `convert_part`, `OscalPart`
- Category: bug | Severity: medium
- Root cause: Missing `<part>` `@id` is silently replaced with an empty string (`xml.id.unwrap_or_default()`, line 291). In the OSCAL catalog metaschema `part`'s `id` is required, so this fabricates a value producing `<part id="">` on re-export, indistinguishable from a legitimately-empty id. Elsewhere this module rejects invalid input with `ExportInvalidOscal` (e.g. bad UUIDs).
- Evidence: xml_deserializer.rs:291; `XmlPart.id` is `Option<String>` (line 126).
- Remediation: Change `convert_part` to return `Result<OscalPart, ForgeError>` and `let id = xml.id.ok_or_else(|| ForgeError::ExportInvalidOscal { detail: format!("missing required @id on <part name=\"{}\"/>", xml.name) })?;` propagating through the recursive `parts` map with `collect::<Result<Vec<_>, _>>()?`. Update `convert_control`/`convert_group` call chains to propagate. Check fixtures: tests/fixtures/export/catalog.xml parts all carry ids (assert in the fixture); add a test with a missing part id asserting ExportInvalidOscal.

### F0444 — valid
- File:lines: `src/export/xml_deserializer.rs:126-136` (`XmlPart.paragraphs`), also 200-205 (`XmlDescription`)
- Symbols: `XmlPart`, `XmlDescription`
- Category: bug | Severity: medium
- Root cause: Prose is modeled as flat plain text (`paragraphs: Vec<String>` over `<p>`), but OSCAL XML prose is markup-multiline legally containing `<em>`, `<strong>`, `<ol>/<li>`, `<insert>`. Any conformant third-party document with rich prose fails quick-xml's serde deserialization with an opaque string-type error — hard-rejecting valid OSCAL XML with misleading diagnostics.
- Evidence: xml_deserializer.rs:132-133 `#[serde(default, rename = "p")] paragraphs: Vec<String>`.
- Remediation: Minimum: add a prominent LIMITATION doc on both `XmlPart.paragraphs` and `XmlDescription.paragraphs` and detect non-textual `<p>` content to raise an explicit error naming the unsupported construct (requires intercepting the serde error or a pre-scan). Better: model `<p>` as an enum accepting text-or-markup. The doc-only path matches the finding's minimum ask; add a test feeding `<p><em>x</em></p>` asserting the diagnostic mentions unsupported markup (or documents the failure mode).

### F0445 — valid
- File:lines: `src/export/xml_deserializer.rs:567-595` (`xxe_prevention_no_entity_expansion`)
- Symbols: `xxe_prevention_no_entity_expansion`, `deserialize_catalog_from_xml`
- Category: test | Severity: medium
- Root cause: The security regression test is self-defeating: if deserialization fails for ANY reason the `else` branch silently accepts (lines 590-592 empty else), so the entity-expansion guard provides zero signal on parser drift. Success is asserted only as `title != "INJECTED"` rather than the positive invariant that `&xxe;` remains unexpanded literal text.
- Evidence: xml_deserializer.rs:584-593 `if let Ok(envelope) = result { assert_ne!(...) } else { /* also acceptable */ }`.
- Remediation: First determine actual quick-xml behavior with a one-off run: if `deserialize_catalog_from_xml(malicious_xml)` currently returns Ok with title `"&xxe;"`, assert positively: `let envelope = deserialize_catalog_from_xml(malicious_xml).expect("must parse past DOCTYPE without expanding entities"); assert_eq!(envelope.catalog.metadata.title, "&xxe;")`. If it returns Err, assert `is_err()` explicitly with a comment. Either way eliminate the silent else. Note interaction with F0441: the fixture's `uuid="test"` will fail UUID validation after that fix — update the fixture UUID before tightening this assertion.

### F0450 — valid
- File:lines: `src/export/xml_serializer.rs:205-209` (`write_resource` props loop)
- Symbols: `write_resource`, `write_prop` (66-78), `Prop` (`src/oscal/back_matter.rs:90-100`)
- Category: bug | Severity: medium
- Root cause: Resource-level props are emitted by a hand-rolled block writing only name/value, dropping `back_matter::Prop.ns: Option<String>` (back_matter.rs:99). Namespaced resource props lose their qualification namespace on every export — a lossy round-trip that can also merge two otherwise-distinct props. `write_prop` already handles `ns` (xml_serializer.rs:73-75); the drift is exactly the duplication the finding describes.
- Evidence: xml_serializer.rs:206-209 pushes only name+value; write_prop at 68-75 conditionally pushes ns.
- Remediation: Delegate to `write_prop`, converting `back_matter::Prop` → `OscalProp` (`OscalProp { name: prop.name.clone(), value: prop.value.clone(), ns: prop.ns.clone() }`). Add a round-trip test: resource prop with `ns: Some("https://example.com/ns")` → serialized XML contains `ns=` attribute and deserializes back with ns intact. Snapshot impact: XML export snapshots for back-matter resources with namespaced props will gain `ns` attributes — review with `cargo insta review`.

### F0451 — valid
- File:lines: `src/export/xml_serializer.rs:141-164` (`write_part`)
- Symbols: `write_part`
- Category: bug | Severity: medium
- Root cause: `write_part` recurses once per nesting level of `part.parts` with no depth bound (lines 158-160). A crafted document with pathological part depth turns serialization into a stack overflow, which aborts the process (uncatchable in Rust) — an availability risk on any path serializing untrusted models. The doc comment's "thousands of levels safely" (lines 139-140) is an unenforced assumption; the deepest test is 3 levels.
- Evidence: xml_serializer.rs:158-160 unbounded recursion.
- Remediation: Thread a depth parameter: `fn write_part_bounded<W: Write>(writer, part, depth: usize)` erroring with `ForgeError::Serialization(format!("part nesting exceeds maximum depth {MAX_PART_DEPTH}"))` when `depth > MAX_PART_DEPTH` (128 is ample); the public `write_part` calls it with depth 0. Update the doc comment to state the enforced bound. Add a test constructing a 129-deep nested OscalPart asserting the error, and keep the existing shallow round-trip tests green.

---

## INVALID findings (one-line rationale)

- **F0385** (`src/config.rs:0-0`, claimed bug): Containment does not reject device names — `CON:stream` stays inside the root, passes containment, then `reject_windows_device_name` fires with the correct "reserved device name" message; test `windows_reserved_device_names_rejected_cross_platform` (config.rs:1215-1234) proves the correct diagnostic. No defect.
- **F0387** (`src/config.rs:0-0`, claimed bug): `FORGE_CONFIG` whitespace handling already matches `FORGE_JOBS` — config.rs:243 uses `value.trim().is_empty()`, rejecting `' '` with the actionable unset-the-variable hint. Defect absent at HEAD.

## DUPLICATE findings (one-line rationale)

- **F0366** (`src/diff/engine.rs:82-89`) → duplicate of **F0369**: the None-vs-empty-string collapse in `compute_field_changes` is a symptom of the empty-string sentinel encoding in `FieldChange` (`src/diff/types.rs:50-60`); the F0369 fix (Option<String> values) removes the ambiguity at the source.
