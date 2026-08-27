# Validation slice slice03 — 62 findings
Severity mix: medium×62


══════ F0299 │ src/citation.rs:196-199 │ [bug · medium] ══════
[bug · medium] strip_matches/normalize_prose run even when nothing matched, so requirements with no
citations are still rewritten: tabs/newlines folded to single spaces, interior double spacing
collapsed, "( )" dropped, ", ." -> ".", " ," -> ",", leading/trailing text trimmed. This contradicts
the "no citations -> text unchanged" expectation implied by EC-1 (which only holds for already-clean
input) and silently degrades prose fidelity (e.g., Markdown line wrapping inside a requirement
disappears) on every pipeline run, not just around removed spans. Gate the cleanup so untouched text
passes through verbatim.

-     let cleaned = strip_matches(text, &matched_ranges);
-     let cleaned = normalize_prose(&cleaned);
+     let cleaned = if matched_ranges.is_empty() {
+         text.to_string()
+     } else {
+         normalize_prose(&strip_matches(text, &matched_ranges))
+     };

      Ok((cleaned, citations))


══════ F0300 │ src/citation.rs:82-87 │ [maintainability · medium] ══════
[maintainability · medium] ForgeError::Parse is reused for a pipeline-ordering precondition, which
misclassifies the failure: this is not malformed input but stage misuse (WI-7 UUID assignment must
run first). Callers that match on Parse to surface parse diagnostics will receive misleading
context, and a single offending requirement hard-fails the entire document, discarding enrichment
already applied to earlier sections. Prefer a dedicated error variant carrying the requirement
identity/source_line (or log-and-skip if the pipeline can tolerate partial enrichment).


══════ F0316 │ src/cli/convert.rs:129-130 │ [performance · medium] ══════
[performance · medium] For every `--stable-id-baseline` invocation the input document is fully
prepared twice: once here (ingest → parse → atomize → UUID → citations → parameters) and again
moments later inside `run_catalog_pipeline`/`run_component_pipeline` via
`prepare_document(opts.input, ...)`. That doubles CPU and I/O on the main file, doubles the failure
surface (a transient ingest/citation error now fires twice), and even duplicates tracing side
effects for a result that is discarded. `prepare_document` is already shared plumbing — extend the
pipeline functions to accept a pre-built document (or return it alongside the output) so the warning
pass can reuse it and only parse the baseline separately.

      let baseline_doc = crate::pipeline::prepare_document(baseline, max_size_bytes)?;
      let current_doc = crate::pipeline::prepare_document(input, max_size_bytes)?;
+     // TODO(perf): thread `current_doc` into the subsequent pipeline run so the
+     // input is not parsed/UUID-assigned twice per invocation.


══════ F0313 │ src/cli/convert.rs:196-200 │ [bug · medium] ══════
[bug · medium] `--stable-id-baseline` is accepted and existence-checked here, but never actually
used in the SSP path: unlike `execute()` (which calls `emit_stable_id_change_warning_if_needed`),
`execute_ssp` validates the file and then silently drops it. Users relying on stable-ID drift
detection in CI get a false sense of coverage when combining `--to ssp --stable-id-baseline`; the
same inconsistency applies to `--summary`, which is also silently ignored on this path. Either honor
the baseline exactly like the single-file path, or reject/warn that the combination is unsupported.

      if let Some(baseline) = opts.stable_id_baseline {
          validate_regular_file(baseline, "--stable-id-baseline")?;
+         // Honor the baseline the same way the catalog/component path does,
+         // instead of validating it and silently ignoring it.
+         emit_stable_id_change_warning_if_needed(opts.input, baseline, max_size_bytes)
+             .map_err(add_max_size_guidance)?;
      }

      let start = std::time::Instant::now();


══════ F0314 │ src/cli/convert.rs:209-218 │ [bug · medium] ══════
[bug · medium] `execute_ssp` bypasses `resolve_source_profile` entirely, so the SEC-3/SEC-4
validation enforced for the catalog/component strategies does not apply to `--to ssp`: a nonexistent
file (or a whitespace-only string that trims to empty being treated as "absent") flows straight into
`build_ssp_skeleton`, which embeds it verbatim in `import_profile.href` (only the fully-empty
default becomes the "TODO-profile.json" placeholder). The generated SSP then points at a profile
that was never verified to exist. At minimum, route the value through the same validation/trimming,
or document why SSP conversions are exempt from the profile checks.

      let catalog = crate::oscal::build_catalog(&doc, None)?;
+
+     // Apply the same --source-profile validation as the component strategy,
+     // or record an explicit exemption for --to ssp skeletons.
+     let source_profile = opts.source_profile.map(str::trim).filter(|p| !p.is_empty());
+     if let Some(p) = source_profile {
+         validate_regular_file(Path::new(p), "--source-profile")?;
+     }

      // Build SSP skeleton with policy-derived control-implementation entries.
      let envelope = crate::oscal::build_ssp_skeleton(
          &title,
          &version,
          &catalog,
          &[],
-         opts.source_profile.unwrap_or(""),
+         source_profile.unwrap_or(""),
      )?;


══════ F0315 │ src/cli/convert.rs:398-402 │ [bug · medium] ══════
[bug · medium] When `--output` is omitted, the primary conversion result goes to stdout, but these
secondary artifacts (assessment plans) are unconditionally written as files relative to the current
working directory — a surprising, side-effecting behavior for users who piped output expecting pure
stdout. Additionally, each iteration writes immediately, so if a later secondary write fails (e.g.,
unwritable CWD), earlier artifacts are left partially written with no cleanup or indication. Decide
on a deterministic location (e.g., suppress secondary files when targeting stdout and say so, or
place them next to the input) and fail before writing when subsequent targets cannot be written.

+         // Mirror the primary-output policy: when --output is absent the main
+         // artifact goes to stdout, yet secondary artifacts silently land in
+         // the CWD and a mid-loop failure leaves partial files behind. Resolve
+         // every destination up front (erroring before any write) and skip or
+         // announce secondary emission when the primary target is stdout.
          let ap_dir = opts.output.and_then(|p| p.parent());
          let ap_path = ap_dir.map_or_else(
              || std::path::PathBuf::from(&secondary.filename),
              |d| d.join(&secondary.filename),
          );


══════ F0312 │ src/cli/convert.rs:89-91 │ [documentation · medium] ══════
[documentation · medium] Doc comment contradicts the implementation (and the tests): it promises
"Returns `Ok(None)` if no profile was provided (with a warning)", but the `None` arm returns
`Err(ForgeError::InvalidArgument(...))`, and
`component_strategy_none_source_profile_errors_with_required_message` asserts exactly that error
behavior. A future maintainer reading this contract could easily "fix" the code back to warning-only
mode and break SEC-3/EC-4. Update the doc to state that a profile is mandatory.

- /// Returns `Ok(None)` if no profile was provided (with a warning),
- /// `Ok(Some(path))` if valid, or `Err` if empty/whitespace-only or file not found.
- fn resolve_source_profile(source_profile: Option<&str>) -> Result<Option<&str>, ForgeError> {
+ /// Validate and resolve `--source-profile` for component strategy.
+ ///
+ /// Returns `Err` if no profile was provided (mandatory for schema-valid
+ /// component definitions), if the value is empty/whitespace-only, or if the
+ /// referenced file is missing or not a regular file.


══════ F0330 │ src/cli/export.rs:282-286 │ [other · medium] ══════
[other · medium] TOCTOU on the size guard: check_file_size stats the path, then a separate
std::fs::read loads the whole file, so a file swapped or grown in between bypasses MAX_FILE_SIZE
entirely and buffers unbounded bytes in memory (memory-exhaustion risk on hostile or concurrently
written inputs). Open a single File handle, validate its metadata().len(), then read through
Read::take(limit + 1) and treat filling limit+1 bytes as too-large — that bounds the buffered
allocation regardless of any race between the check and the read.

-     // Step 2: Guard against oversized files before reading
-     crate::io::check_file_size(input_path, crate::io::MAX_FILE_SIZE)?;
+     use std::io::Read;

-     // Step 3: Read input file (read bytes first for actionable encoding errors)
-     let bytes = std::fs::read(input_path)?;
+     let mut file = std::fs::File::open(input_path)?;
+     let declared_len = file.metadata()?.len();
+     // Bound the bytes actually buffered, not just the pre-read stat, so a
+     // concurrent grow/swap cannot bypass the limit.
+     let mut bytes = Vec::with_capacity(
+         usize::try_from(declared_len).unwrap_or(0).min(crate::io::MAX_FILE_SIZE),
+     );
+     file.take((crate::io::MAX_FILE_SIZE + 1) as u64)
+         .read_to_end(&mut bytes)?;
+     if bytes.len() > crate::io::MAX_FILE_SIZE {
+         // TODO: reuse the existing too-large error constructor from crate::io
+         return Err(ForgeError::ExportInvalidOscal {
+             detail: format!("File '{}' exceeds the maximum supported size", input_path.display()),
+         });
+     }


══════ F0333 │ src/cli/export.rs:89-99 │ [maintainability · medium] ══════
[maintainability · medium] deserialize_from_json and deserialize_from_yaml_format duplicate the
detect_model_type -> envelope-cast -> error-mapping flow verbatim, including copy-pasted
unsupported-type strings; note that "; use JSON" is meaningless advice in the JSON branch where the
input already is JSON, evidence of unreviewed duplication. The YAML path already owns a
serde_json::Value, so route both through one shared helper, e.g. fn oscal_model_from_value(value:
serde_json::Value) -> Result<OscalModel, ForgeError>. Otherwise every new OSCAL model type or
unsupported-type message must be edited in three places (JSON/YAML/XML) and will inevitably drift.

-         crate::validate::OscalModelType::Profile => Err(ForgeError::ExportInvalidOscal {
-             detail: "Export of OSCAL Profile documents is not yet supported".to_string(),
-         }),
-         crate::validate::OscalModelType::Mapping => Err(ForgeError::ExportInvalidOscal {
-             detail: "Export of OSCAL Control Mapping documents is not yet supported; use JSON"
+ /// Shared JSON/YAML ingest path: both formats funnel into a
+ /// serde_json::Value before model-type detection and envelope casting,
+ /// keeping the unsupported-type table in a single place.
+ fn oscal_model_from_value(value: serde_json::Value) -> Result<OscalModel, ForgeError> {
+     match crate::validate::detect_model_type(&value).map_err(|_| ForgeError::ExportInvalidOscal {
+         detail:
+             "Input does not contain a recognized OSCAL root key ('catalog' or 'component-definition')"
                  .to_string(),
-         }),
+     })? {
+         crate::validate::OscalModelType::Catalog => {
+             let envelope: CatalogEnvelope = serde_json::from_value(value).map_err(|e| {
+                 ForgeError::ExportInvalidOscal {
+                     detail: format!("Failed to parse OSCAL catalog: {e}"),
      }
+             })?;
+             Ok(OscalModel::Catalog(envelope))
  }
-
- /// Deserialize an OSCAL artifact from XML.
+         crate::validate::OscalModelType::ComponentDefinition => {
+             let envelope: ComponentDefinitionEnvelope = serde_json::from_value(value)
+                 .map_err(|e| ForgeError::ExportInvalidOscal {
+                     detail: format!("Failed to parse OSCAL component-definition: {e}"),
+                 })?;
+             Ok(OscalModel::Component(envelope))
+         }
+         // Unsupported types (Profile, Mapping) reported here, once.
+         other => Err(unsupported_model_error(other)),
+     }
+ }


══════ F0343 │ src/cli/migrate.rs:67-72 │ [security · medium] ══════
[security · medium] Check-then-act race and identity-comparison gaps in the alias guard. (1)
`path.exists()` followed by `path.canonicalize()` is a classic TOCTOU: if the path disappears or is
swapped between the two calls, the guard either reports a misleading "unable to resolve {role}"
error or resolves an attacker-swapped path. (2) Identity is compared as canonicalized *strings*,
which misses same-inode aliases such as hard links and can diverge on case-insensitive or
Unicode-normalizing filesystems where the stored name's spelling differs from the user-supplied one.
(3) The guard runs once, before `analyze_paths`, and the verdict is trusted again at `write_output`
— seconds later after potentially slow PDF/DOCX parsing — so the destination path can legally change
in between (the deferred rename in `write_atomic` limits symlink-following damage, but the directory
entry is still replaced blindly). Consider dropping the `exists()` probe entirely (canonicalize
alone distinguishes missing paths), comparing device+inode via `Metadata` (`same_file::Handle` or
`fs::metadata(...).dev()/ino()`) instead of path strings, and re-running the alias check immediately
before `write_output`, ideally creating/reserving the destination handle up front.

  fn path_identity(path: &Path, role: &str) -> Result<PathBuf, ForgeError> {
-     if path.exists() {
-         return path.canonicalize().map_err(|error| {
-             ForgeError::MigrationError(format!("unable to resolve {role}: {error}"))
-         });
+     // canonicalize() subsumes the existence probe and avoids the
+     // exists()-then-resolve race; a vanishing/symlinked path is surfaced as a
+     // typed io::Error rather than assumed to "exist".
+     match path.canonicalize() {
+         Ok(identity) => return Ok(identity),
+         Err(error) if error.kind() != io::ErrorKind::NotFound => {
+             return Err(ForgeError::MigrationError(format!(
+                 "unable to resolve {role}: {error}"
+             )));
+         }
+         Err(_) => {}
      }
+     // ... fall through to parent-resolution for not-yet-created paths


══════ F0340 │ src/cli/mod.rs:87-89 │ [security · medium] ══════
[security · medium] `convert --max-size` accepts any `u64` with no `range(...)`, unlike: (a) the
project-config path, where `src/config.rs` enforces `parse_range("convert.max-size-mb", ..., 1,
51_200)` via `resolve_convert`, and (b) the `migrate` command here, which clamps to `1..=51_200`.
Because the resolver only applies the range to config-file values (the CLI value short-circuits with
`.or_else`), `forge convert --max-size 0` and `--max-size 4294967296` are accepted verbatim: a value
above the sanity cap disables the oversized-input rejection entirely for untrusted policy inputs
(resource-exhaustion vector in CI), while 0 produces a confusing 'file too large' failure for every
non-empty input. Apply the same `value_parser = clap::value_parser!(u64).range(1..=51_200)` used by
Migrate so the CLI cannot bypass the guardrail that config-derived values must satisfy.

          /// Maximum input file size in MB (default: 10 or project configuration)
-         #[arg(long)]
+         #[arg(
+             long,
+             value_parser = clap::value_parser!(u64).range(1..=51_200)
+         )]
          max_size: Option<u64>,


══════ F0326 │ src/cli/output.rs:20-24 │ [maintainability · medium] ══════
[maintainability · medium] On stdout errors, the raw io::Error is propagated without any indication
that the failed target was stdout. At this user-facing boundary, actionable context should be
preserved (project rule: add context at boundaries). Map remaining error kinds with a message
identifying stdout, e.g. `io::Error::new(e.kind(), format!("failed writing to stdout: {e}"))`, so
diagnostics such as 'No space left on device' point at stdout rather than looking like a generic
pipeline failure.

              match stdout.write_all(content.as_bytes()) {
                  Ok(()) => {}
                  Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
-                 Err(e) => return Err(ForgeError::Io(e)),
+                 Err(e) => {
+                     return Err(ForgeError::Io(io::Error::new(
+                         e.kind(),
+                         format!("failed writing to stdout: {e}"),
+                     )));
+                 }
              }


══════ F0327 │ src/cli/output.rs:42-42 │ [maintainability · medium] ══════
[maintainability · medium] Delegating with a bare `?` loses target-path context for most failure
modes: `crate::io::write_atomic` only embeds `path.display()` in its *persist* error path, while
temp-file creation (`NamedTempFile::new_in`), `write_all`, and the two `sync_all` calls propagate as
raw io::Errors (e.g., a bare 'No space left on device' naming no file). Since this is the CLI
boundary, wrap the delegation so every emitted error names the output path.

- crate::io::write_atomic(path, content.as_bytes())?;
+             crate::io::write_atomic(path, content.as_bytes()).map_err(|e| {
+                 ForgeError::Io(io::Error::other(format!(
+                     "failed writing output to '{}': {e}",
+                     path.display()
+                 )))
+             })?;


══════ F0321 │ src/cli/profile.rs:110-110 │ [bug · medium] ══════
[bug · medium] `to_string_lossy()` silently substitutes U+FFFD for any non-UTF-8 bytes in the path.
This string becomes the Catalog href embedded in the generated Profile, so on a system where the
path contains non-UTF-8 bytes (legal on Unix) you produce a permanently broken reference with no
error raised. Since the profile is emitted as JSON/XML/YAML (all UTF-8), fail fast instead:

-     let catalog_str = catalog.to_string_lossy();
+     let catalog_str = catalog.to_str().ok_or_else(|| {
+         ForgeError::InvalidArgument(format!(
+             "catalog path {} is not valid UTF-8; refusing to embed a corrupted href",
+             catalog.display()
+         ))
+     })?;


══════ F0322 │ src/cli/profile.rs:51-57 │ [bug · medium] ══════
[bug · medium] --set-param ID validation is inconsistent with how the ID is actually used. The check
trims the ID before deciding it is non-empty, but the original (possibly whitespace-padded) string
is forwarded untrimmed to `build_profile`, so e.g. `--set-param " ac-1" x` passes validation and
emits an ID containing spaces. Additionally, repeated IDs across multiple `--set-param` flags are
never detected; whether the last or first override wins depends entirely on undeclared downstream
dedup semantics, risking a silently applied wrong override. Validate-and-store the trimmed ID and
reject duplicate IDs (or explicitly document last-wins).

-     for (id, _) in &pairs {
-         if id.trim().is_empty() {
+     let mut seen = std::collections::HashSet::new();
+     for (id, value) in &pairs {
+         let id = id.trim();
+         if id.is_empty() {
              return Err(ForgeError::InvalidArgument(
                  "Empty or whitespace-only --set-param ID".to_string(),
              ));
+         }
+         if !seen.insert(id.to_string()) {
+             return Err(ForgeError::InvalidArgument(format!(
+                 "duplicate --set-param for ID '{id}'"
+             )));
          }
+         let _ = value;
      }


══════ F0320 │ src/cli/profile.rs:91-94 │ [bug · medium] ══════
[bug · medium] TOCTOU-style existence probe that also misclassifies directories. `catalog.exists()`
is a check-then-act: the file can disappear/be replaced between this check and the actual open
inside `build_profile`, producing a spurious success here followed by a confusing failure later.
Worse, `exists()` returns true for directories, so a directory path passes validation and only fails
deeper in `build_profile` with a less precise error. Prefer one atomic probe:
`std::fs::metadata(catalog)` and require `is_file()` (mapping only `NotFound` to `FileNotFound`), or
skip the pre-check entirely and let the open error surface.

-     // Step 2: check catalog exists
-     if !catalog.exists() {
+     // Step 2: single atomic probe — avoids TOCTOU and directory-vs-file ambiguity
+     match std::fs::metadata(catalog) {
+         Ok(meta) if meta.is_file() => {}
+         Ok(_) => {
+             return Err(ForgeError::InvalidArgument(format!(
+                 "catalog path {} is not a regular file",
+                 catalog.display()
+             )));
+         }
+         Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
          return Err(ForgeError::FileNotFound { path: catalog.to_path_buf() });
+         }
+         Err(e) => return Err(ForgeError::Io(e)),
      }


══════ F0347 │ src/cli/resolve.rs:59-66 │ [bug · medium] ══════
[bug · medium] Input validation stops at extension name + canonicalize(), which succeeds for any
existing path — including directories and symlinks-to-directories. A directory named 'profile.json'
passes both checks and later fails deep inside oscal-cli as an opaque ForgeError::OscalCliExecution
(exit code from the external tool) instead of a typed input error produced here. Verify the
canonicalized path is a regular file so invalid input is diagnosed locally.

-     // FR-007 + FR-014: Canonicalize input path (validates existence implicitly)
-     let canonical_input = input.canonicalize().map_err(|e| match e.kind() {
-         std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: input.to_path_buf() },
-         std::io::ErrorKind::PermissionDenied => {
-             ForgeError::PermissionDenied { path: input.to_path_buf() }
+     // Ensure the canonicalized path denotes a regular file (rejects directories
+     // such as a folder coincidentally named 'profile.json').
+     if !std::fs::metadata(&canonical_input).map_err(ForgeError::Io)?.is_file() {
+         return Err(ForgeError::InvalidArgument(format!(
+             "Input '{}' is not a regular file",
+             canonical_input.display()
+         )));
          }
-         _ => ForgeError::Io(e),
-     })?;


══════ F0346 │ src/cli/trace.rs:14-16 │ [bug · medium] ══════
[bug · medium] At this CLI boundary, errors from `generate_trace_report` propagate to the user
without distinguishing which of the two input files (`artifact` vs `source`) caused the failure. For
example, a non-UTF-8 file surfaces as `ForgeError::Io("stream did not contain valid UTF-8")` and
JSON parse errors appear as "Invalid JSON in artifact: ..." with no path attached — with two
user-supplied paths the diagnosis is ambiguous. Add actionable context here while preserving the
original cause, e.g. wrap the failures of each phase with the corresponding path via
`.map_err(...)`.

  pub fn execute(artifact: &Path, source: &Path, output: Option<&Path>) -> Result<(), ForgeError> {
-     let report = generate_trace_report(artifact, source)?;
+     let report = generate_trace_report(artifact, source)
+         .map_err(|e| ForgeError::Validation(format!(
+             "failed to build trace report from artifact '{}' and source '{}': {e}",
+             artifact.display(),
+             source.display()
+         )))?;
      let table = format_trace_table(&report);


══════ F0359 │ src/cli/validate.rs:133-140 │ [bug · medium] ══════
[bug · medium] The extension gate is case-sensitive: files named `*.JSON` or `*.Json` are rejected
even though they are valid OSCAL JSON that plain `forge validate` would accept. Normalize the
comparison so matching is ASCII case-insensitive.

      match input.extension().and_then(|e| e.to_str()) {
-         Some("json") => {}
+         Some(ext) if ext.eq_ignore_ascii_case("json") => {}
          _ => {
              return Err(ForgeError::Validation(
                  "Round-trip validation requires a JSON input file".to_string(),
              ));
          }
      }


══════ F0357 │ src/cli/validate.rs:249-252 │ [bug · medium] ══════
[bug · medium] Detection failures are swallowed here: `detect_model_type(...).ok()` turns an
unrecognized/non-OSCAL input into artifact_type "Unknown" and drops declared_oscal_version, so
divergences are reported against placeholder metadata and the most probable root cause (wrong file
type fed to --round-trip) is hidden from the operator. Since the input was already parsed
successfully, detection failing is meaningful signal — propagate it as a validation error instead of
degrading silently.

-     let model_type = validate::detect_model_type(original_json).ok();
-     let artifact_type = model_type.map_or_else(|| "Unknown".to_string(), |model| model.to_string());
-     let declared_oscal_version = model_type
-         .and_then(|model| validate::version::inspect_oscal_version(original_json, model).declared);
+     let model_type = match validate::detect_model_type(original_json) {
+         Ok(model) => model,
+         Err(e) => {
+             return Err(ForgeError::Validation(format!(
+                 "Round-trip input '{}' is not a recognized OSCAL model: {e}",
+                 input.display()
+             )));
+         }
+     };
+     let artifact_type = model_type.to_string();
+     let declared_oscal_version =
+         validate::version::inspect_oscal_version(original_json, model_type).declared;


══════ F0358 │ src/cli/validate.rs:43-58 │ [security · medium] ══════
[security · medium] The size cap (SEC-3) is enforced by a stat-like check before
`std::fs::read_to_string`, so the bytes actually consumed are unbounded: a file swapped,
truncated-in, or retargeted via symlinks between check and read bypasses the limit entirely (classic
TOCTOU). Either fold the limit into the read (bounded reader) or re-validate `content.len()` after
reading. Note also that `execute()` reads from the raw `input` path without canonicalizing, unlike
`execute_round_trip()`, so the guard protects a different path object than the one subsequently
read.

      // Step 1: Check file size (SEC-3)
-     validate::check_file_size(input).map_err(|e| match e {
-         ValidateError::FileTooLarge { size_mb, limit_mb } => ForgeError::Validation(format!(
-             "Artifact file is too large ({size_mb:.1}MB, limit: {limit_mb}MB)"
-         )),
-         ValidateError::FileRead { path, source } => ForgeError::Validation(format!(
-             "Failed to read artifact file '{}': {source}",
-             path.display()
-         )),
-         other => ForgeError::Validation(other.to_string()),
-     })?;
-
+     validate::check_file_size(input).map_err(map_validate_error)?;
      // Step 2: Read file
      let content = std::fs::read_to_string(input).map_err(|e| {
          ForgeError::Validation(format!("Failed to read artifact file '{}': {e}", input.display()))
      })?;
+     // Re-enforce the cap on what was actually read (closes check-vs-read TOCTOU)
+     validate::ensure_read_within_limit(content.len()).map_err(map_validate_error)?;


══════ F0360 │ src/cli/validate.rs:81-84 │ [maintainability · medium] ══════
[maintainability · medium] Structured errors are repeatedly flattened into formatted strings
(`ValidateError` variants -> `other.to_string()`, detect errors ->
`ForgeError::Validation(e.to_string())`, and `run_full_validation` failures ->
`ForgeError::SchemaValidation(e.to_string())`). This destroys the source chain and any error-class
information downstream consumers need for exit codes or programmatic handling. Map typed variants
onto dedicated `ForgeError` variants (or carry the source via `#[from]`) at these boundaries instead
of stringifying.

-     // Step 6: Run full validation (schema + semantic) via WI-20 orchestrator
-     let artifact_path = input.display().to_string();
      let report = validate::run_full_validation(&artifact_path, &json, model_type)
-         .map_err(|e| ForgeError::SchemaValidation(e.to_string()))?;
+         .map_err(ForgeError::FullValidation)?; // preserves source chain / variant info


══════ F0385 │ src/config.rs:0-0 │ [bug · medium] ══════
[bug · medium] `reject_windows_device_name` and `ensure_symlink_containment` run only AFTER the
containment pre-check returns early — every rejected device-name path short-circuits before reaching
them (fine), but more importantly the ordering means the 'outside the project root' wording is
produced for inputs like `CON:stream` on Unix, where the actual problem is the device name,
producing misleading diagnostics. Reorder so syntactic validation (absolute / device names)
completes before containment evaluation.


══════ F0387 │ src/config.rs:0-0 │ [bug · medium] ══════
[bug · medium] Inconsistent trimming policy (EC-3 analogue): `FORGE_JOBS` explicitly rejects values
that are empty *after trimming* (`raw.trim().is_empty()`), but `FORGE_CONFIG` only rejects values
that are literally empty — `' '` passes through and becomes the config path `' '`. Either apply the
same whitespace rule here (preferred for symmetry with the documented EC-3 behavior) or document why
whitespace-padded selectors are acceptable. As written, a stray-space export (`export FORGE_CONFIG="
"`) fails later with 'cannot read config file  : …' instead of the actionable unset-the-variable
hint.


══════ F0388 │ src/config.rs:0-0 │ [performance · medium] ══════
[performance · medium] `edit_distance` runs unbounded O(len(a)*len(b)) with two heap-allocated char
vectors per invocation, and it is called once per known key for every unknown TOML key string.
`check_unknown_keys` accepts keys of arbitrary length from a ≤1 MiB attacker-controlled file
(Pre-scan focus #3): a handful of multi-hundred-kilobyte bogus keys forces ~50 × 100k² char
comparisons (~tens of GB of work) from merely running `forge --help` next to such a config, enabling
trivial CPU exhaustion / soft-DoS. Bound the comparison first (keys here are short identifiers;
anything longer than a few dozen chars can never plausibly match), and reuse buffers across
candidates.


══════ F0372 │ src/diff/canonical.rs:119-128 │ [security · medium] ══════
[security · medium] Stat/open TOCTOU weakens both guards performed here. `is_file()` and the
`MAX_FILE_SIZE` check run against `fs::metadata(path)`, but the subsequent `read_to_string(path)`
re-resolves and re-opens the path. If the path is swapped in between (replacement, symlink
retarget), the code can read a special/non-regular file or an arbitrarily large file, defeating the
regular-file and 50 MB memory-safety limits that every other reader in this crate enforces. Open the
file once, validate metadata on the held handle, and read (bounded, e.g. `Read::take`) from that
same handle so checks and reads always observe the same inode.

+     // Open once and validate against the held handle so checks and reads
+     // observe the same inode even if the path is swapped concurrently.
+     let mut file = std::fs::File::open(path).map_err(|error| {
+         ForgeError::DiffError(format!("unable to open {role_name} artifact ({:?})", error.kind()))
+     })?;
+     let metadata = file.metadata().map_err(|error| {
+         ForgeError::DiffError(format!(
+             "unable to inspect {role_name} artifact ({:?})",
+             error.kind()
+         ))
+     })?;
+     if !metadata.is_file() {
+         return Err(ForgeError::DiffError(format!("{role_name} artifact is not a regular file")));
+     }
      if metadata.len() > crate::io::MAX_FILE_SIZE {
          return Err(ForgeError::DiffError(format!(
              "{role_name} artifact exceeds the {} byte comparison limit",
              crate::io::MAX_FILE_SIZE
          )));
      }

-     let text = std::fs::read_to_string(path).map_err(|error| {
-         ForgeError::DiffError(format!("unable to read {role_name} artifact ({:?})", error.kind()))
+     use std::io::Read;
+     let mut text = String::new();
+     file.take(crate::io::MAX_FILE_SIZE)
+         .read_to_string(&mut text)
+         .map_err(|error| {
+             ForgeError::DiffError(format!(
+                 "unable to read {role_name} artifact ({:?})",
+                 error.kind()
+             ))
      })?;


══════ F0373 │ src/diff/canonical.rs:174-183 │ [maintainability · medium] ══════
[maintainability · medium] The v1 exclusion set exists only as imperative `remove` calls plus this
prose warning; nothing structurally ties the list to `DRIFT_COMPARISON_CONTRACT_VERSION`, so the two
can silently desync (an added/removed exclusion with a stale version stamped into `cli/drift.rs`
reports). Model the exclusions declaratively next to the version constant and have `canonicalize`
walk the table, making the sync mechanical and reviewable. Consider also embedding
`DRIFT_COMPARISON_CONTRACT_VERSION` in `DriftComparison` so verdicts are self-describing rather than
relying on the caller to record the right global.

-     // Contract v1 exclusions. Do not add fields here without incrementing
-     // DRIFT_COMPARISON_CONTRACT_VERSION and documenting the security impact.
-     root.remove("uuid");
-     let metadata = root.get_mut("metadata").and_then(Value::as_object_mut).ok_or_else(|| {
-         ForgeError::DiffError(format!(
-             "{} artifact must contain a '{root_key}.metadata' JSON object",
-             role.as_str()
-         ))
-     })?;
-     metadata.remove("last-modified");
+ /// JSON-pointer-style volatile fields excluded by each comparison contract.
+ /// Must be extended/edited together with `DRIFT_COMPARISON_CONTRACT_VERSION`.
+ const EXCLUDED_FIELDS: &[&[&str]] = &[["uuid"], ["metadata", "last-modified"]];
+
+ // Inside canonicalize():
+ for path in EXCLUDED_FIELDS {
+     let mut node = root.as_object_mut();
+     if let Some((last, parents)) = path.split_last() {
+         for parent in parents {
+             node = node.and_then(|obj| obj.get_mut(*parent)).and_then(Value::as_object_mut);
+         }
+     }
+     if let Some(obj) = node {
+         obj.remove(last);
+     }
+ }


══════ F0365 │ src/diff/engine.rs:100-104 │ [bug · medium] ══════
[bug · medium] Positional (index-based) comparison of `parts_prose` produces cascading false
positives: inserting, deleting, or reordering any statement shifts every subsequent index, so
unrelated statements are reported as "changed" while their real counterparts are never compared.
Additionally, rendering an out-of-range side as `""` makes a deleted statement indistinguishable
from emptied content except at the tail of the vector. Consider anchoring on stable part identifiers
upstream, or at minimum trimming the longest common prefix/suffix before the positional scan so only
the genuinely shifted region emits entries.

-     // Parts prose comparison (Catalog statements)
-     let max_len = old.parts_prose.len().max(new.parts_prose.len());
-     for i in 0..max_len {
-         let old_val = old.parts_prose.get(i).map_or("", String::as_str);
-         let new_val = new.parts_prose.get(i).map_or("", String::as_str);
+     // Trim the common prefix/suffix first so pure appends/prepends and edits
+     // away from the list ends stop shifting every reported index.
+     let o = &old.parts_prose;
+     let n = &new.parts_prose;
+     let mut start = 0;
+     while start < o.len() && start < n.len() && o[start] == n[start] {
+         start += 1;
+     }
+     let (mut end_o, mut end_n) = (o.len(), n.len());
+     while end_o > start && end_n > start && o[end_o - 1] == n[end_n - 1] {
+         end_o -= 1;
+         end_n -= 1;
+     }
+     for i in start..end_o.max(end_n) {
+         let old_val = o.get(i).map_or("", String::as_str);
+         let new_val = n.get(i).map_or("", String::as_str);
+         if old_val != new_val {
+             changes.push(FieldChange {
+                 field_name: format!("statement[{i}]"),
+                 old_value: old_val.to_string(),
+                 new_value: new_val.to_string(),
+             });
+         }
+     }
+     // Note: middle insertions still shift subsequent indices; prefer anchoring
+     // on part identifiers at snapshot time or an LCS-based alignment.


══════ F0366 │ src/diff/engine.rs:82-89 │ [bug · medium] ══════
[bug · medium] `clone().unwrap_or_default()` collapses `None` and `Some("")` to the same empty
string, but the enclosing `!=` on the `Option`s still detects a difference between them. The result
is a `FieldChange` whose `old_value` and `new_value` are both `""`, which per `FieldChange`'s
documented contract ("empty `old_value` indicates the field was added; empty `new_value` indicates
the field was removed") is self-contradictory and renders as a no-op edit in the report. Absence and
genuinely-empty text should stay distinguishable — e.g., propagate `Option<String>` through
`FieldChange`, or normalize `Some(s) if s.is_empty()` to `None` before comparing.

-     // Title comparison
+     // Title comparison — compare raw Options and report Optionals so an
+     // absent field is distinguishable from an empty-string field (requires
+     // FieldChange.old_value/new_value to become Option<String> in
+     // src/diff/types.rs).
      if old.title != new.title {
          changes.push(FieldChange {
              field_name: "title".to_string(),
-             old_value: old.title.clone().unwrap_or_default(),
-             new_value: new.title.clone().unwrap_or_default(),
+             old_value: old.title.clone(),
+             new_value: new.title.clone(),
          });
      }


══════ F0351 │ src/diff/extractor.rs:126-141 │ [bug · medium] ══════
[bug · medium] Last-write-wins on duplicate control-ids silently discards earlier snapshots here.
Unlike catalogs (where control ids should be unique), in an OSCAL component-definition it is normal
and valid for *multiple* components and/or capabilities to implement the same control-id (e.g., two
services both implementing AC-2). Every implementation except the last is dropped from the map
before the downstream diff ever sees it, so removals/changes to those discarded requirements will be
invisible. The tracing::warn acknowledges collisions but doesn't prevent loss. Consider aggregating
colliding entries instead of overwriting (e.g., merging descriptions/parts_prose into lists per
control-id), or keying snapshots by (ir.uuid, control-id) / returning a MultiMap-style structure so
the diff can compare every implemented requirement.

-             if map.contains_key(control_id) {
-                 tracing::warn!(
-                     control_id,
-                     "Duplicate control-id in component definition; last occurrence wins"
-                 );
-             }
-             map.insert(
-                 control_id.to_string(),
-                 ControlSnapshot {
+ // Aggregate instead of overwrite so legitimately repeated control-ids survive:
+ let entry = map.entry(control_id.to_string()).or_insert_with(|| ControlSnapshot {
                      control_id: control_id.to_string(),
-                     uuid,
+     uuid: String::new(),
                      title: None,
-                     description,
+     description: None,
                      parts_prose: vec![],
-                 },
-             );
+ });
+ if entry.description.is_none() {
+     entry.description = description;
+ }


══════ F0352 │ src/diff/extractor.rs:28-34 │ [bug · medium] ══════
[bug · medium] Only `/catalog/groups` is walked, but the OSCAL catalog model also allows controls to
appear directly under `catalog.controls` (outside any group). Such controls are silently ignored,
yielding an empty/partial map for otherwise valid catalogs (small hand-written catalogs frequently
use root-level controls). Extract `/catalog/controls` in addition to groups — e.g., factor the
per-control collection loop out of `collect_controls_from_groups` into a shared helper taking
`&[Value]` and call it for both `catalog.controls` and every group's `controls`.

      let mut map = HashMap::new();
+     // OSCAL permits controls directly under the catalog, outside any group.
+     if let Some(controls) = json.pointer("/catalog/controls").and_then(Value::as_array) {
+         collect_controls(controls, &mut map);
+     }
      let groups = json
          .pointer("/catalog/groups")
          .and_then(Value::as_array)
          .map_or(EMPTY_ARRAY, Vec::as_slice);
      collect_controls_from_groups(groups, &mut map);
      map


══════ F0377 │ src/diff/formatter.rs:33-36 │ [maintainability · medium] ══════
[maintainability · medium] Dual source of truth: the early-return gate consults
summary.has_changes(), but every section heading re-counts report.entries independently. Two
divergence modes: (1) a DiffReport whose summary says 'no changes' while entries is non-empty
silently drops ALL detail rows behind 'No differences found.'; (2) mismatched counters print
self-contradicting output, e.g. 'Added: 0' in the Summary block under a heading rendered as 'Added
(3)'. Pick one authority: derive the Summary numbers from the same filtered-entry counts used for
headings, or at minimum add debug_assert_eq! cross-checks (plus a unit test that builds an
intentionally desynced report) so desync fails loudly in development.

+     // Cross-check the summary against the authoritative entries list so a
+     // desync aborts in debug builds instead of printing a self-contradicting
+     // report (or hiding detail behind "No differences found.").
+     debug_assert_eq!(
+         report.entries.iter().filter(|e| matches!(e, DiffEntry::Added { .. })).count(),
+         s.added,
+         "summary.added does not match entries"
+     );
+
      if !s.has_changes() {
          writeln!(out, "No differences found.").unwrap();
          return out;
      }


══════ F0381 │ src/diff/formatter.rs:84-91 │ [maintainability · medium] ══════
[maintainability · medium] Old/new values are written verbatim inside quotes. FieldChange values
come from control titles, descriptions, and parts prose (per ControlSnapshot docs), i.e. potentially
large multi-paragraph free text: a single changed description renders as one enormous wrapped blob,
embedded newlines break the one-line 'field: old -> new' layout and the heading-based structure, and
a value that itself contains '\u{2192}' makes output ambiguous for any grep/split-based consumer.
Flatten newlines to a visible escape and cap length so each field stays one bounded line (or emit
explicit OLD:/NEW: blocks for multi-line values).

-                 for fc in field_changes {
-                     writeln!(
-                         out,
-                         "      {}: \"{}\"  \u{2192}  \"{}\"",
-                         fc.field_name, fc.old_value, fc.new_value
-                     )
-                     .unwrap();
+ const MAX_VALUE_CHARS: usize = 200;
+
+ /// Collapses newlines/tabs into one line and truncates long values so a
+ /// free-text field cannot blow up the report layout.
+ fn one_line(value: &str, max_chars: usize) -> String {
+     let flat: String = value.chars().map(|c| match c { '\n' | '\r' | '\t' => ' ', c => c }).collect();
+     if flat.chars().count() <= max_chars {
+         flat
+     } else {
+         format!("{} [...truncated]", flat.chars().take(max_chars).collect::<String>())
+     }
                  }
+
+ // At the use site:
+ //   writeln!(out, "      {}: \"{}\"  \u{2192}  \"{}\"",
+ //       fc.field_name, one_line(&fc.old_value, MAX_VALUE_CHARS),
+ //       one_line(&fc.new_value, MAX_VALUE_CHARS)).unwrap();


══════ F0397 │ src/diff/mod.rs:107-110 │ [bug · medium] ══════
[bug · medium] This arm collapses every `ValidateError` into a fixed "not a recognized OSCAL
artifact" message, discarding the underlying cause. When the file genuinely contains two OSCAL root
keys, `detect_model_type` returns `ValidateError::AmbiguousArtifact` with a detail list — yet the
user is told the file is unrecognized, which is factually wrong and hides exactly the information
needed to fix the input. Match the specific variants (or interpolate the error itself) so ambiguity
is reported accurately.

-         Err(_) => Err(ForgeError::DiffError(format!(
-             "'{}': not a recognized OSCAL artifact; expected 'catalog' or 'component-definition' root key",
+         Err(e) => Err(ForgeError::DiffError(format!(
+             "'{}': expected a single supported OSCAL root key ('catalog' or 'component-definition'): {e}",
              path.display()
          ))),


══════ F0396 │ src/diff/mod.rs:34-35 │ [performance · medium] ══════
[performance · medium] Both artifacts' raw strings stay live across both `serde_json::Value` parses,
and both parsed trees stay live until extraction finishes: peak footprint ≈ two full texts plus two
DOM trees. `serde_json::Value` nodes typically cost several times the input byte count, so with the
50 MB-per-file cap a single CLI invocation can transiently approach ~1 GB even when both inputs are
individually "legal". Reorder the work so at most one artifact's buffers are live at a time:
load+parse each artifact, extract its `ControlSnapshot` map, then drop the `String` and `Value`
before touching the second file.

-     let old_text = read_diff_file(old_path)?;
-     let new_text = read_diff_file(new_path)?;
+ // Per-artifact step returns (ArtifactType, HashMap<String, ControlSnapshot>)
+ // and drops the raw String and the serde_json::Value before returning.
+ let old = load_snapshot(old_path)?;
+ let new = load_snapshot(new_path)?;
+ if old.artifact_type != new.artifact_type {
+     return Err(ForgeError::DiffError(format!(
+         "Artifact type mismatch: old is {}, new is {}",
+         old.artifact_type, new.artifact_type
+     )));
+ }


══════ F0395 │ src/diff/mod.rs:83-87 │ [security · medium] ══════
[security · medium] The size guard is a check-then-read window: `check_file_size` compares a
one-shot `metadata().len()`, then `read_to_string` reads with no bound at all. A file that grows (or
is appended) between the stat and the read, or a special file that reports length 0 (FIFO,
/dev/stdin via redirection), passes the guard and gets buffered arbitrarily far past
`MAX_FILE_SIZE`, defeating the intended memory cap. Additionally, the `Err(e) =>
DiffError(e.to_string())` mapping flattens the structured `FileTooLarge`/`Io` errors (including
their source chain) into a plain string. Bound the read itself instead of trusting the pre-stat:
open the file and wrap it in `std::io::Take` limited to `MAX_FILE_SIZE + 1`, reject when more than
the limit was consumed, and propagate the original error (source chain intact) for the open/read
failures.

-     match crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE) {
-         Ok(_) | Err(ForgeError::Io(_)) => {}
-         Err(e) => return Err(ForgeError::DiffError(e.to_string())),
+ use std::io::Read;
+
+ let mut file = std::fs::File::open(path)
+     .map_err(|e| ForgeError::DiffError(format!("Failed to open '{}': {e}", path.display())));
+ let mut buf = String::new();
+ {
+     let mut limited = file.take(crate::io::MAX_FILE_SIZE.saturating_add(1));
+     limited
+         .read_to_string(&mut buf)
+         .map_err(|e| ForgeError::DiffError(format!("Failed to read '{}': {e}", path.display())))?;
      }
-     std::fs::read_to_string(path).map_err(|e| match e.kind() {
+ if buf.len() as u64 > crate::io::MAX_FILE_SIZE {
+     return Err(ForgeError::DiffError(format!(
+         "'{}' exceeds {} bytes",
+         path.display(),
+         crate::io::MAX_FILE_SIZE
+     )));
+ }
+ Ok(buf)


══════ F0369 │ src/diff/types.rs:50-60 │ [maintainability · medium] ══════
[maintainability · medium] Encoding "field added"/"field removed" via empty-string sentinels
conflates "absent" with "present but empty". If a control legitimately has an empty
title/description (or an empty prose part) in one snapshot, producers cannot distinguish a removal
from an addition: `{field_name: "title", old_value: "", new_value: ""}` is simultaneously 'added' by
the documented rule and indistinguishable from a no-op 'unchanged' case, and a downstream renderer
can silently misclassify the change kind. Since this enum is the wire format consumed by report
generators, encode presence explicitly with `Option<String>` (or a dedicated change-kind enum) so
that absence is representable separately from the empty value.

- /// An empty `old_value` indicates the field was added; an empty `new_value`
- /// indicates the field was removed.
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub struct FieldChange {
      /// The name of the field that changed (e.g., `"title"`, `"description"`).
      pub field_name: String,
-     /// The previous value of the field (empty string if the field was added).
-     pub old_value: String,
-     /// The new value of the field (empty string if the field was removed).
-     pub new_value: String,
+     /// Previous value; `None` only when the field did not exist before.
+     pub old_value: Option<String>,
+     /// New value; `None` only when the field no longer exists.
+     pub new_value: Option<String>,
  }
+ // Alternatively, model the kind explicitly:
+ // pub enum FieldChangeKind { Added { new_value: String }, Removed { old_value: String }, Modified { old_value: String, new_value: String } }


══════ F0370 │ src/diff/types.rs:90-91 │ [maintainability · medium] ══════
[maintainability · medium] `uuid_changed` is fully derivable from `old_uuid != new_uuid`, yet it is
stored as an independently assignable boolean, so internally inconsistent variants are
constructible: e.g. `Changed { old_uuid: "a", new_uuid: "a", uuid_changed: true, .. }` (flag lies)
or `Changed { old_uuid: "a", new_uuid: "b", uuid_changed: false, field_changes: vec![] }`, which
semantically duplicates the separate `UuidChanged` variant while bypassing it — consumers that match
on `UuidChanged` will miss such controls and undercount `uuid_changes`. This violates the principle
of making invalid states unrepresentable. Either drop the flag and let consumers compare the UUID
pair, or replace it with a derived accessor/method (`fn uuid_changed(&self) -> bool`) so it can
never disagree with the payload.

-         /// Whether the UUID itself changed between artifacts.
-         uuid_changed: bool,
+ impl DiffEntry {
+     /// Whether the UUID changed; derived from the payload so it can never
+     /// disagree with `old_uuid`/`new_uuid`.
+     #[must_use]
+     pub fn uuid_changed(&self) -> bool {
+         match self {
+             Self::Changed { old_uuid, new_uuid, .. }
+             | Self::UuidChanged { old_uuid, new_uuid, .. } => old_uuid != new_uuid,
+             _ => false,
+         }
+     }
+ }


══════ F0406 │ src/error.rs:344-347 │ [maintainability · medium] ══════
[maintainability · medium] Encoding successful-but-actionable outcomes as errors with empty Display
strings is fragile: any generic consumer (logging frameworks, `anyhow` wrapping, `{:?}` debugging,
future entry points) either prints a blank line or a bare variant name, silently losing the
diagnostic. The sentinel set is also duplicated as a parallel match in main.rs, so adding a new
sentinel requires touching three places (variant + exit_code arm + main.rs arm) and a miss makes the
CLI exit 1 with zero visible output. Prefer a non-empty Display message (details remain printable
separately by the command layer), or a dedicated result/outcome enum kept out of ForgeError
entirely.

-     /// This variant has an empty error message because the diff details are
-     /// printed separately by the CLI.
-     #[error("")]
+     /// Details were already printed by the CLI; this message backs generic
+     /// `Display` consumers so error reporting never emits a blank line.
+     #[error("diff detected changes between golden and current output")]
      DiffHasChanges,


══════ F0405 │ src/error.rs:383-388 │ [documentation · medium] ══════
[documentation · medium] Documented contract drifts from the implementation below. Several variants
land outside their declared category: OscalCliExecution/OscalCliTimeout exit 1 although they are
external-dependency errors (their siblings OscalCliNotFound/OscalCliNotFunctional exit 4);
SchemaValidation (declared under '--- Other ---') exits 3; MissingRequiredArgument exits 2; and
seven review-required/drift sentinels exit 1. Since CI/scripts gate on these exact numeric codes,
document the full mapping here (and keep the module-level header in sync), otherwise readers will
trust proximity-based groupings that no longer hold.

  /// Exit code categories:
  /// - `0`: Success (not handled here — only error cases)
- /// - `1`: Input/IO errors (file not found, permission denied, empty, binary, encoding, size, I/O)
- /// - `2`: Parse/Structure errors (no structure, parse failure, build errors)
- /// - `3`: Validation/Config errors (schema violations, config issues)
- /// - `4`: External dependency unavailable (oscal-cli not found or not functional)
+ /// - `1`: Input/IO, export, argument, and batch errors; oscal-cli execution
+ ///   failures/timeouts; and "review required" / change-detected sentinels
+ /// - `2`: Parse/structure/build errors, usage errors
+ ///   (`MissingRequiredArgument`), and diff/analysis computation failures
+ /// - `3`: Validation/Config errors, including `SchemaValidation`
+ /// - `4`: External dependency unavailable (`oscal-cli` missing or not functional)


══════ F0408 │ src/error.rs:909-919 │ [test · medium] ══════
[test · medium] The exit-code matrix is this module's core behavioral contract, but many variants
lack regression coverage: ExportUnsupportedExtension, ExportNoExtension, ExportInvalidOscal,
ExportEmptyInput, InvalidArgument, OcrNotSupported, MappingReviewRequired, FrameworkReviewRequired,
MigrationHasChanges, MigrationError, SspBuild, MappingBuild, FrameworkImpact, and SchemaValidation
(which is the third occupant of the exit-3 bucket and completely unasserted). Moving any of these
variants between match arms would pass CI. Prefer a data-driven test enumerating every variant so
exhaustiveness is compiler-checked via the existing match.

      #[test]
      fn round_trip_failed_display() {
          let err = ForgeError::RoundTripFailed(3);
          assert_eq!(err.to_string(), "Round-trip validation failed: 3 unresolved divergence(s)");
      }

      #[test]
      fn round_trip_failed_exit_code_is_1() {
          assert_eq!(exit_code(&ForgeError::RoundTripFailed(1)), 1);
+     }
+
+     #[test]
+     fn exit_code_matrix_covers_remaining_variants() {
+         let cases: Vec<(ForgeError, u8)> = vec![
+             (ForgeError::ExportUnsupportedExtension { extension: "txt".into() }, 1),
+             (ForgeError::ExportNoExtension { path: PathBuf::from("a") }, 1),
+             (ForgeError::ExportInvalidOscal { detail: "d".into() }, 1),
+             (ForgeError::ExportEmptyInput { path: PathBuf::from("a") }, 1),
+             (ForgeError::InvalidArgument("a".into()), 1),
+             (ForgeError::OcrNotSupported { path: PathBuf::from("a") }, 1),
+             (ForgeError::MappingReviewRequired, 1),
+             (ForgeError::FrameworkReviewRequired, 1),
+             (ForgeError::MigrationHasChanges, 1),
+             (ForgeError::MigrationError("m".into()), 2),
+             (ForgeError::SspBuild("s".into()), 2),
+             (ForgeError::MappingBuild("m".into()), 2),
+             (ForgeError::FrameworkImpact("f".into()), 2),
+             (ForgeError::SchemaValidation("v".into()), 3),
+         ];
+         for (err, want) in cases {
+             assert_eq!(exit_code(&err), want, "unexpected exit code for: {err}");
+         }
      }
  }


══════ F0444 │ src/export/xml_deserializer.rs:130-133 │ [bug · medium] ══════
[bug · medium] Prose is assumed to be flat plain text (`Vec<String>` over `<p>`), but OSCAL XML
prose is markup-multiline and legally contains inline/block children such as `<em>`, `<strong>`,
`<ol>/<li>`, and `<insert>`. Any conformant third-party document using rich prose makes quick-xml's
serde deserializer fail here with an opaque string-type error (or drop structure), i.e. the importer
hard-rejects valid OSCAL XML with misleading diagnostics. At minimum document this limitation
prominently; better, detect non-textual `<p>` content and raise an explicit error naming the
unsupported construct.

      /// Prose content wrapped in `<p>` elements in OSCAL XML.
      /// Multiple `<p>` nodes are preserved and joined with newlines.
+     ///
+     /// LIMITATION: only plain-text paragraphs are supported. Nested inline/
+     /// block markup (<em>, <strong>, lists, <insert>) inside <p> — legal
+     /// markup-multiline in OSCAL XML — causes deserialization to fail.
      #[serde(default, rename = "p")]
      paragraphs: Vec<String>,


══════ F0442 │ src/export/xml_deserializer.rs:280-286 │ [bug · medium] ══════
[bug · medium] `@rel` is silently fabricated as "reference" when the source link omits it. This
distorts imported data: a third-party OSCAL document with `<link href="#x"/>` deserializes into a
model asserting `rel="reference"`, and any subsequent serialization (XML/JSON/YAML export) emits an
attribute the source never had — violating the project's lossless round-trip goal (WI-28) and
potentially changing semantics for downstream consumers that key off `rel`. Since `OscalLink.rel` is
mandated as `String`, at minimum document this substitution at the type/API level; ideally widen the
model to `Option<String>` so absence is representable.

+ // NOTE: documents the deliberate, lossy substitution required by
+ // `OscalLink.rel: String`; absence of @rel in the source is recorded as "reference".
  fn convert_link(xml: XmlLink) -> OscalLink {
      OscalLink {
          href: xml.href,
          rel: xml.rel.unwrap_or_else(|| "reference".to_string()),
          text: xml.text,
      }
  }


══════ F0443 │ src/export/xml_deserializer.rs:291-291 │ [bug · medium] ══════
[bug · medium] A missing `<part>` `@id` is silently replaced with an empty string. In the OSCAL
catalog metaschema, `part`'s `id` flag is required, so this fabricates a value that makes the
rebuilt control unrepresentable/invalid on re-export (an `<part id="">`) and is indistinguishable
from a legitimately-empty id. Elsewhere in this module invalid input produces
`ForgeError::ExportInvalidOscal` (e.g. bad resource UUIDs); invalid parts deserve the same explicit
rejection instead of `unwrap_or_default()` masking it.

-         id: xml.id.unwrap_or_default(),
+         id: xml.id.ok_or(ForgeError::ExportInvalidOscal {
+             detail: format!("missing required @id on <part name=\"{}\"/>", xml.name),
+         })?,


══════ F0441 │ src/export/xml_deserializer.rs:369-371 │ [bug · medium] ══════
[bug · medium] Validation coverage is partial: resource, capability, control-implementation and
implemented-requirement UUIDs are verified via `UUid::try_parse`, but the root
catalog/component-definition `@uuid` and each component `@uuid` flow straight from the raw string
into the model with no check. Malformed identifiers ('', 'not-a-uuid', arbitrary text) thus enter
`OscalCatalog.uuid` / `DocumentaryComponent.uuid` on half the codepaths, get re-emitted unchanged by
exporters, and undermine the exported artifact's OSCAL validity claims. Validate these three
attributes the same way (parse and store the canonical form) instead of trusting the input.

+     let uuid = Uuid::try_parse(&xml.uuid)
+         .map_err(|e| ForgeError::ExportInvalidOscal {
+             detail: format!("Invalid UUID in catalog: '{}' — {e}", xml.uuid),
+         })?;
      Ok(OscalCatalog {
-         uuid: xml.uuid,
+         uuid: uuid.to_string(),
          metadata: convert_metadata(xml.metadata),


══════ F0439 │ src/export/xml_deserializer.rs:417-419 │ [bug · medium] ══════
[bug · medium] UUID handling is inconsistent and the validated value is thrown away.
`Uuid::try_parse` accepts braced, URN-style and uppercase-hex forms, but the raw string is stored
here instead of a canonical form, while `convert_capability` stores `uuid.to_string()`. A
control-implementation like `{880E8400-E29B-41D4-A716-446655440000}` therefore round-trips verbatim
while the sibling Capability field is normalized to lowercase-hyphenated form, yielding divergent
representations of equally 'validated' identifiers in the same document (and those non-canonical
strings get re-emitted by serializers). Parse once and serialize the parsed `Uuid` back, matching
`convert_capability`.

-     Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
+     let uuid = Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
          detail: format!("Invalid UUID in control-implementation: '{uuid}' — {e}", uuid = xml.uuid),
      })?;
+     // ...
+     Ok(ControlImplementation {
+         uuid: uuid.to_string(),
+         source: xml.source,
+         // ...


══════ F0440 │ src/export/xml_deserializer.rs:437-439 │ [bug · medium] ══════
[bug · medium] Same discarded-validation pattern as `convert_control_implementation`: the checked
`Uuid` result is dropped and the unparsed, non-canonical string (braces, `urn:` prefix, uppercase
hex) is propagated into `ImplementedRequirement.uuid`, while `convert_capability` normalizes via
`to_string()`. Prefer binding the parsed `Uuid` and storing `uuid.to_string()` for consistent
representation across all OSCAL identifier fields.

-     Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
+     let uuid = Uuid::try_parse(&xml.uuid).map_err(|e| ForgeError::ExportInvalidOscal {
          detail: format!("Invalid UUID in implemented-requirement: '{uuid}' — {e}", uuid = xml.uuid),
      })?;
+     // ...
+     Ok(ImplementedRequirement {
+         uuid: uuid.to_string(),
+         control_id: xml.control_id,
+         // ...


══════ F0445 │ src/export/xml_deserializer.rs:582-592 │ [test · medium] ══════
[test · medium] This security regression test is self-defeating: if deserialization ever starts
failing for *any* reason (including a parser change), the `else` branch silently accepts the outcome
and the guard against entity expansion provides zero signal. Additionally, success is only asserted
against `!= "INJECTED"`; the intended safe behavior is that the literal `&xxe;` remains unexpanded
text. Assert the positive invariant (Ok, with title equal to the unexpanded literal) so any
behavioral drift in quick-xml fails loudly instead of vacuously passing.

-         // quick-xml should either error or not expand the entity
-         let result = deserialize_catalog_from_xml(malicious_xml);
-         if let Ok(envelope) = result {
-             // If parsing succeeds, entity must NOT have been expanded
-             assert_ne!(
-                 envelope.catalog.metadata.title, "INJECTED",
+         // SEC-1: the entity must remain unexpanded — reject the contract drift.
+         let envelope =
+             deserialize_catalog_from_xml(malicious_xml)
+                 .expect("quick-xml must parse past the DOCTYPE without expanding entities");
+         assert_eq!(
+             envelope.catalog.metadata.title, "&xxe;",
                  "XXE entity expansion detected — security vulnerability!"
              );
-         } else {
-             // Rejecting the document is also acceptable (safe behavior)
-         }


══════ F0451 │ src/export/xml_serializer.rs:157-160 │ [bug · medium] ══════
[bug · medium] `write_part` recurses once per input-controlled nesting level of `part.parts` with no
depth bound. A crafted document with pathological part depth (e.g. deeply chained structures) turns
serialization into a thread-stack overflow, which aborts the process (unrecoverable — Rust has no
catchable stack-overflow) — an availability risk on any path serializing untrusted models. The
doc-comment's 'thousands of levels safely' assumption depends on frame size and 8 MiB stacks and is
not enforced here (the deepest test is only 3 levels). Threading a small depth counter (or
converting to an explicit worklist) keeps the crash impossible:

-     // Nested parts (position 4)
+     // Nested parts (position 4), bounded to prevent stack exhaustion on
+     // hostile/deeply-nested inputs.
+     const MAX_PART_DEPTH: usize = 128;
      for sub_part in &part.parts {
-         write_part(writer, sub_part)?;
+         write_part_bounded(writer, sub_part, depth + 1)?;
      }
+     // fn write_part_bounded(.., depth: usize):
+     //   if depth > MAX_PART_DEPTH {
+     //       return Err(ForgeError::Serialization(format!(
+     //           "part nesting exceeds maximum depth {MAX_PART_DEPTH}")));
+     //   }


══════ F0450 │ src/export/xml_serializer.rs:206-209 │ [bug · medium] ══════
[bug · medium] Resource-level props drop the optional `ns` attribute: `back_matter::Prop` carries
`ns: Option<String>` (see src/oscal/back_matter.rs:99), but this hand-rolled block emits only
name/value, so namespaced resource props lose their qualification namespace on every export — a
lossy round-trip that can also merge two otherwise-distinct props with equal name/value. This
duplicates the logic already provided by `write_prop`, whose drift caused exactly this divergence;
delegate to it instead (adjust signature/types or convert `Prop` → `OscalProp`), and add a test with
`ns: Some(..)`.

-         let mut prop_elem = BytesStart::new("prop");
-         prop_elem.push_attribute(("name", prop.name.as_str()));
-         prop_elem.push_attribute(("value", prop.value.as_str()));
-         writer.write_event(Event::Empty(prop_elem)).map_err(map_xml_err)?;
+         // Reuse write_prop so the optional ns attribute is preserved
+         write_prop(
+             writer,
+             &OscalProp {
+                 name: prop.name.clone(),
+                 value: prop.value.clone(),
+                 ns: prop.ns.clone(),
+             },
+         )?


══════ F0394 │ src/export/yaml.rs:35-35 │ [security · medium] ══════
[security · medium] This generic entry point accepts arbitrarily sized and arbitrarily nested YAML
with no documented trust assumption. Adversarial input (deeply nested sequences/maps, anchor/alias
'billion laughs' expansion) drives recursive deserialization that can exhaust the stack or memory;
neither serde nor libyaml imposes alias-expansion depth limits. At minimum document the caller's
obligation to enforce size/provenance limits (the pipeline already has FileTooLarge machinery), and
consider capping input length or depth before handing data to this function.

- pub fn deserialize_from_yaml<T: DeserializeOwned>(yaml: &str) -> Result<T, ForgeError> {
+ /// Deserialize a YAML string to any serde-deserializable type.
+ ///
+ /// # Security
+ ///
+ /// Callers MUST NOT pass fully untrusted input: deeply nested structures or
+ /// alias bombs can cause stack exhaustion or disproportionate memory usage,
+ /// since YAML provides no built-in expansion/depth limits and deserialization
+ /// recurses per nesting level. Enforce a byte-size cap (see FileTooLarge)
+ /// and validate input provenance before calling.


══════ F0418 │ src/framework/analysis.rs:1118-1127 │ [maintainability · medium] ══════
[maintainability · medium] [Finding Identity Reuse Across Runs] building deterministic finding_id
via UUIDv5 over `old/new fingerprints + subject + class` means two fully-disjoint analyses (e.g.,
old=X,new=Y run Monday; old=Y,new=Z evaluated later under the same schema version) mint
byte-identical finding_ids for their respective Removed/Added rows. Because `apply_dispositions`
keys purely on finding_id, dispositions recorded against one lineage will reattach to unrelated
findings in another comparison without any collision signal. There's no generation timestamp/epoch
or run-id baked into the seed (the system clock isn't used anywhere else either), so nothing breaks
the tie. Unless every operator stores prior reports exclusively per old->new pair, embed an explicit
comparison/pair identifier (or prior-report linkage token) into the seed so dedupe only happens
within the intended reruns.


══════ F0417 │ src/framework/analysis.rs:162-163 │ [bug · medium] ══════
[bug · medium] The [Risk of Skew] summary.counts-vs-findings mismatch is real here:
update_disposition_summary counts the pre-filter findings array, then apply_filters moves matched
findings into `findings` and stashes the rest in `filtered_out_findings`, which never feeds back
into the summary. So consumers relying on undispositioned/disp* counters alongside the emitted
findings see contradictory numbers whenever filters apply (e.g., 40 total findings, 35 hidden by
--group leaves `findings` = 5 while undispositioned still claims 35). Downstream gates keyed off
findings then disagree with disposition progress derived from summary. Either recompute after
filtering (partition first), or split summary into `visible_*` and `hidden_*` buckets so arithmetic
checks out and the dual-array model is explicit.


══════ F0420 │ src/framework/analysis.rs:317-321 │ [bug · medium] ══════
[bug · medium] [Duplicate prior IDs silent swallow duplicates; non-current go to holding pen without
notice] The gate rejects *duplicate IDs present in the current findings* nowhere — instead,
duplicate finding_id entries silently fold into `current_finding_indexes`, last-writer-wins during
`current_finding_indexes.get(...)`, so the SECOND matching disposition record silently overwrites
the first assignment via `report.findings[*index].disposition = Some(disposition)`. Meanwhile
overlapping-but-not-identical IDs slip into `prior_only_dispositions` without a log/warning, hiding
drift. Current_finding_indexes should detect collision on insert (like `insert(key, idx).is_some()`
returning Err) or require unique keys via try_insert. Also consider emitting diagnostics (count or
sample IDs) when prior_only_dispositions grows large so users notice lineage mismatches early.


══════ F0422 │ src/framework/analysis.rs:612-616 │ [security · medium] ══════
[security · medium] TOCTOU window and trust asymmetry: unlike Mapping Collection inputs which get
full SHA256 + size + strict parse treatment here, the `prepared.input_paths` from the applicability
subsystem receive only a stat-based existence check (regular_file_metadata) after prepare_analysis
already consumed them. A swapped/truncated file — or one whose canonical resolution differs between
load and verification — passes cleanly because contents aren't re-hashed. Additionally, scanning
every dependency path on EVERY analyze() run makes complexity proportional to number of inputs, with
no memoization even though check_file_size regular_file_metadata syscall the file metadata multiple
times. Prefer capturing sha256 (+size+mtime) once inside prepare_analysis return value and
validating recomputed hashes here against expectations, mirroring how MAPPING_JSON_LIMITS +
expected_resolved_catalog_sha256 guard the other branch. Path-based identity also fails to catch
aliases through symlinked parents, unlike the explicit `paths_alias` comparison performed for
mapping collections.


══════ F0401 │ src/framework/disposition.rs:69-71 │ [security · medium] ══════
[security · medium] Check-then-act race on the size cap (TOCTOU): `regular_file_metadata` stats the
path, then `std::fs::read` opens it again. If the file is replaced/grown between the two steps, an
arbitrarily large file is read and fully buffered — the parser's strict limits cover depth and
per-string size, not total input size, so `MAX_DISPOSITION_BYTES` is effectively bypassable. Enforce
the cap on the bytes actually read (and preferably derive metadata and content from a single opened
handle) rather than trusting the earlier stat.

      let bytes = std::fs::read(path).map_err(|cause| {
          error(format!("cannot read disposition file '{}': {cause}", path.display()))
      })?;
+     // Defense in depth: metadata was taken from a separate lookup, so the
+     // cap can be raced. Re-check the bytes actually read.
+     if bytes.len() as u64 > MAX_DISPOSITION_BYTES {
+         return Err(error(format!(
+             "disposition file exceeds the {MAX_DISPOSITION_BYTES} byte limit"
+         )));
+     }


══════ F0412 │ src/framework/manifest.rs:106-117 │ [security · medium] ══════
[security · medium] Duplication is detected with a byte-exact BTreeSet<&Path>, which only rejects
lexically identical spellings. Two Mapping Collections citing "Controls.json" and "controls.json"
pass validation yet resolve to the same physical file on Windows, macOS, and most Samba/NFS
deployments, letting one collection be double-counted in impact attribution. Related narrow scopes:
the seed set never includes $.old.artifact, $.new.artifact, or $.old/new.resolved_catalog, so a
mapping entry can silently shadow primary evidence (e.g. deliberately claim old-side Target
membership while reading the counterpart whose checksum actually matches). Normalize each artifact
to a canonical lookup key (ASCII-lowercased spelling is adequate since validate_json_path already
guarantees relative local JSON paths) and additionally probe collisions against the artifact
identities referenced by the companion pair.

      let mut paths = BTreeSet::new();
      for (index, dependency) in manifest.mapping_collections.iter().enumerate() {
          validate_json_path(
              &format!("$.mapping_collections[{index}].artifact"),
              &dependency.artifact,
          )?;
-         if !paths.insert(dependency.artifact.as_path()) {
+         // Byte-exact deduplication misses case-only spelling variants, which address the
+         // same file on Windows/macOS and case-folding network shares.
+         let key = match dependency.artifact.to_str() {
+             Some(text) => text.to_ascii_lowercase(),
+             None => return Err(impact_error(format!(
+                 "$.mapping_collections[{index}].artifact must be valid UTF-8"
+             ))),
+         };
+         if !paths.insert(key) {
              return Err(impact_error(format!(
                  "$.mapping_collections[{index}].artifact duplicates another Mapping Collection"
              )));
          }
      }


══════ F0414 │ src/framework/manifest.rs:211-213 │ [test · medium] ══════
[test · medium] Unit coverage exercises only validate_json_path. The stateful contracts in
validate()/validate_resource() - schema_version pinning, old/new type agreement, Mapping Collection
bounds and duplicate rejection, resolved_catalog_attestation being strictly Some(true) for Profiles,
forbidden resolved-catalog companion fields on Catalogs, the prior_report/disposition_file pairing
rule, and the pinned OSCAL version - are all untested, so any future refactor can silently loosen
them. Add table-driven tests using a small helper that builds a baseline ImpactManifest and copies
overrides per case, asserting per-rule error-message substrings.

- fn validate_sha256(path: &str, value: &str) -> Result<(), ForgeError> {
-     crate::json_strict::validate_lowercase_sha256(path, value).map_err(impact_error)
- }
+ // Table-driven tests wiring fixtures alongside the json-path suite, for example:
+ // assert_rejected(|m| m.schema_version = "forge.framework-impact/0".into(), "unsupported schema_version");
+ // assert_rejected(|m| m.new.resource_type = ResourceType::Catalog, "must describe the same OSCAL model");
+ // assert_rejected(|m| m.old.resource_type = ResourceType::Profile, "resolved_catalog is required for a Profile");
+ // assert_rejected(|m| m.old.resolved_catalog_attestation = Some(false), "attestation must be true for a Profile");
+ // assert_rejected(|m| m.old.resolved_catalog = None, ...);
+ // assert_rejected(|m| m.disposition_file = None, "both be present or both be absent");


══════ F0424 │ src/framework/mod.rs:135-136 │ [security · medium] ══════
[security · medium] Two injection-consistency gaps across the renderers:

1. markdown_escape maps raw '\n'/'\r' to &#10;/&#13;, which is correct inside inline text but
incorrect for Markdown table cells: when these values are embedded into a table row (render_markdown
tables above), '&#10;' does NOT terminate the row in Markdown source — but more importantly there is
no mechanism that prevents a hostile manifest document_version/control id containing a real pipe
sequences from being neutralized incorrectly... Actually the pipes ARE backslash-escaped here, good
— however note asymmetry with render_html where literal '\n' characters are emitted verbatim inside
<td> elements (see finding.dependency_path handling: html_escape leaves control characters
untouched), allowing newline-based row splitting that mirrors the '|'-row-splitting issue the tests
guard against.

2. In this very block the SHA columns are emitted without markdown_escape/html_escape. That happens
to be safe today only because analysis.rs derives them from single_sha256()/sha256() hex output
rather than manifest strings — a fragile invariant. Any future change that propagates a
manifest-derived value through old_sha256/new_sha256 turns both tables into an injection vector
without any compiler signal.

Recommendation: defensively escape all manifest-originated columns uniformly (or centralize a
checked constructor for ControlChange sha fields proving they are 64-hex) so escaping does not
depend on distant derivation-site guarantees.

- change.old_sha256.as_deref().unwrap_or("none"),
-             change.new_sha256.as_deref().unwrap_or("none")
+ // Prefer treating every manifest-derived field uniformly:
+ //   markdown_escape(change.old_sha256.as_deref().unwrap_or("none"))
+ // or enforce via type construction:
+ //   pub struct Sha256Hex(String);
+ //   impl Sha256Hex { pub fn compute(bytes: &[u8]) -> Self { ... } }
+ // and print via as_str(), making accidental manifest propagation impossible.


══════ F0425 │ src/framework/mod.rs:275-277 │ [security · medium] ══════
[security · medium] html_escape does not encode control characters (notably \n, \r, NUL).
Manifest-supplied strings such as document_version, control ids, or dependency_path segments are
emitted verbatim (newlines included) inside <td>...</td> cells, so an attacker-crafted value can
restructure the rendered HTML report (e.g., split rows) similarly to what the '|' escaping guards
against in Markdown. GitHub workflow annotation rendering correctly percent-encodes %0A/%0D, showing
the threat model is acknowledged elsewhere. Consider escaping newlines as &#10;/&#13; (or stripping
them) inside cell values.

  '"' => escaped.push_str("&quot;"),
              '\'' => escaped.push_str("&#39;"),
+             '\n' => escaped.push_str("&#10;"),
+             '\r' => escaped.push_str("&#13;"),
              _ => escaped.push(character),


══════ F0426 │ src/framework/mod.rs:324-326 │ [maintainability · medium] ══════
[maintainability · medium] github_property only extends github_data to escape ':' and ',', but the
GitHub runner docs require ':' and ',' to be escaped in *both* title and message properties
('name,value' pair delimiters can appear anywhere in either side of ::command title=..::message::).
message= values produced from finding fields are not run through github_property, so a
manifest-controlled finding_id/subject_id containing ',' can still spoof additional annotation
properties.

  fn github_property(value: &str) -> String {
      github_data(value).replace(':', "%3A").replace(',', "%2C")
  }
+ // Render as: "::{command} title={title}::{github_property(&message)}"
+ // so delimiters cannot be smuggled into annotation payloads.


══════ F0427 │ src/framework/mod.rs:441-443 │ [maintainability · medium] ══════
[maintainability · medium] This couples framework's user-facing error text to mapping's diagnostic
wording: if 'Control Mapping build error: ' ever changes upstream, errors silently stop being
stripped and every failed path-alias check leaks the wrong error variant prefix into
ForgeError::FrameworkImpact messages. Prefer exposing a typed variant/source accessor on ForgeError
(e.g., match on the underlying kind) instead of string surgery.

- crate::mapping::paths_alias(left, right).map_err(|error| {
-         impact_error(error.to_string().replace("Control Mapping build error: ", ""))
-     })
+ // e.g. match on ForgeError variants instead of editing Display output:
+ // let aliased = crate::mapping::paths_alias(left, right)
+ //     .map_err(impact_error_from_mapping)?;
