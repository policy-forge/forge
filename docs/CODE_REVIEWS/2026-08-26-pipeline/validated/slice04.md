# slice04 — validated findings (63 medium)

Validated 2026-08-26 against HEAD b22e2d5 ("Harden successor map opening against symlink races").
Verdicts: **47 valid · 13 partial · 3 invalid · 0 duplicate**.

---

## VALID findings

### F0464 — src/framework/model.rs:11-12 — ImpactReport.status unguarded string [maintainability · medium]
- Symbols: `ImpactReport.status` (`&'static str`); rendered by every renderer in `src/framework/mod.rs` (markdown line 98, HTML line 174, text line 332).
- Root cause: unlike the four report enums in the same file (`ChangeClass`, `FindingPriority`, `ReasonCode`, `RequiredAction`), `status` is a free-form static str with no compile-time value guard; the enum-guard test (model.rs tests) cannot catch a typo like `"compleet"`, and nothing ties the value to `REPORT_SCHEMA_VERSION`.
- Evidence: `pub status: &'static str` at model.rs:12; all three renderers interpolate it; fixture hardcodes `status: "complete"` (mod.rs:608).
- Remediation: add `pub enum ReportStatus { Complete }` with `#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]` + `#[serde(rename_all = "kebab-case")]` + `const fn as_str()` mirroring `ChangeClass::as_str`; change `ImpactReport.status` to `ReportStatus`; update construction sites to `ReportStatus::Complete`; extend the existing `enum_as_str_values_match_their_serialized_contracts` test. No snapshot impact (serialized value stays `"complete"`).

### F0432 — src/ingest/mod.rs:119-125 — ingest_file TOCTOU between metadata and read [security · medium]
- Symbols: `ingest_file` (metadata 108-124, `std::fs::read` 125-132; plus second open in `extract_pdf_content` via `pdf_extract::extract_text`; `path.canonicalize()` 146-153).
- Root cause: size/is-file decision comes from `std::fs::metadata`, content from a later independent `std::fs::read`; the file can be swapped/grown between calls, so an oversized or different file slips past the size check and `fingerprint`/`source_path` can describe different snapshots.
- Evidence: two distinct path-based syscalls confirmed in current code.
- Remediation: open once: `File::open(path)` → `file.metadata()` → is_file/size checks on the handle → `read_to_end` on the same handle; canonicalize before validating so all checks apply to the same inode. Keep existing error mapping (`FileNotFound`/`PermissionDenied`/`NotAFile`/`FileTooLarge`).

### F0434 — src/ingest/mod.rs:198-199 — DOCX extractor fidelity loss [bug · medium]
- Symbols: `extract_docx_document_xml` — `reader.config_mut().trim_text(true)` (line 199); catch-all `_ => {}` arms for `w:tab`/`w:br`/`CData`.
- Root cause: `trim_text(true)` strips leading/trailing whitespace per text chunk, merging adjacent `<w:r>` runs ("password " + "reset" → "passwordreset"); `<w:tab>`/`<w:br>` hit the default arm and contribute nothing; `Event::CData` content is dropped.
- Evidence: config confirmed; Text arm pushes decoded text verbatim with no run-boundary separator.
- Remediation: disable `trim_text` (handle `xml:space` manually), emit a space for `w:tab` and a newline for `w:br`, treat `Event::CData` like `Event::Text`. Add a unit test with two runs "password "/"reset", a tab, and a br via the `create_docx` helper.

### F0435 — src/ingest/mod.rs:279 — DOCX table nesting tracked by booleans [bug · medium]
- Symbols: `extract_docx_document_xml` — `in_table`/`in_row`/`in_cell` bools; `End` arm `b"w:tbl" => in_table = false`.
- Root cause: an inner `<w:tbl>` inside a cell sets `in_table=true` again; its `End` clears it while the outer table is still open, leaking outer-cell paragraphs into main output and interleaving rows.
- Evidence: confirmed plain bools, no depth stack.
- Remediation: replace with a `Vec<bool>` table stack pushed on `Start(w:tbl)`, popped on `End(w:tbl)`, deriving `in_table` from the stack top (optionally same for row/cell context). Add a nested-table docx unit test.

### F0453 — src/io.rs:116-117 — sanitize_artifact_path fallback leaks raw path [security · medium]
- Symbols: `sanitize_artifact_path`; callers `src/oscal/assessment_plan.rs:253`, `src/oscal/implemented_requirements.rs:128`, `src/oscal/profile.rs:241`.
- Root cause: for paths with no final component ("/", ".", "..", "sub/..", "C:\\"), `Path::file_name()` is None and the entire input path — absolute, machine-specific — is returned into OSCAL artifact hrefs, defeating the helper's documented purpose.
- Evidence: `map_or_else(|| path.to_string_lossy().into_owned(), ...)` confirmed.
- Remediation: handle `None` explicitly — trim trailing separators and retry `file_name()`, else return a fixed placeholder (e.g. `"artifact"`). Add tests for `"/"`, `"sub/.."`.

### F0455 — src/io.rs:24-29 — write_atomic resets destination permissions [bug · medium]
- Symbols: `write_atomic` (`NamedTempFile::new_in` + `tmp.persist(path)`).
- Root cause: temp file is created 0600 on Unix; `persist` renames it over the destination, so regenerating an existing (e.g. world-readable) output downgrades its mode to 0600 silently.
- Evidence: no permission preservation anywhere in the function.
- Remediation: before `persist`, on Unix `fs::metadata(path)` the existing destination and `set_permissions(tmp.path(), from_mode(mode))` (best-effort). Add a unix-only test: create a 0644 file, overwrite via `write_atomic`, assert mode preserved.

### F0454 — src/io.rs:42-44 — check_file_size follows symlinks, single-sample stat [security · medium]
- Symbols: `check_file_size` vs `regular_file_metadata` (io.rs:56).
- Root cause: `fs::metadata` follows symlinks and the size is sampled before the caller reads; `regular_file_metadata` rejects symlinks but `check_file_size` callers accept symlinked paths, and symlinked special files (proc-style) report len()==0 while reading unbounded bytes.
- Evidence: confirmed; the framework loader pairs both helpers, mapping callers do not.
- Remediation: pair the check with the opened handle at read time (`file.metadata()`) and reject symlinks/non-regular files consistently, or convert callers to a single bounded-read helper on an open handle.

### F0468 — src/json_strict.rs:148-150 — duplicate-key check clones key, echoes it raw [security · medium]
- Symbols: `StrictValueVisitor::visit_map`.
- Root cause: `values.insert(key.clone(), value.0)` pays a clone per map entry on the success path; the error interpolates the raw untrusted key (log-injection) instead of `bounded()`.
- Evidence: code confirmed at lines 148-150.
- Remediation: probe first: `if values.contains_key(&key) { return Err(...format!("duplicate object key '{}'", bounded(&key))...); } values.insert(key, value.0);`. Keep the "duplicate object key" message prefix so existing tests (successor.rs, lifecycle record tests, manifest) still pass.

### F0469 — src/json_strict.rs:39 — eager breadcrumb path formatting in enforce_bounds [performance · medium]
- Symbols: `enforce_bounds` (`format!("{path}[{index}]")`, `format!("{path}.{key}")`).
- Root cause: the path string is eagerly materialized for every array element/object member even when all bounds hold — O(nodes × depth) throwaway allocations on success.
- Evidence: confirmed.
- Remediation: pass a segment stack (indices/names) down the recursion and render the escaped printable path via `bounded()` only when constructing an error. Same shape as F0477's fix.

### F0467 — src/json_strict.rs:53-58 — raw object keys in diagnostic paths [security · medium]
- Symbols: `enforce_bounds` Object arm.
- Root cause: attacker-controlled keys are interpolated verbatim into `{path}.{key}` strings that propagate through `Result<Value, String>` into logs/errors; JSON keys may encode newlines/control bytes/ANSI escapes; the module defines `bounded()` (line 24) but never uses it here.
- Evidence: confirmed.
- Remediation: `enforce_bounds(child, &format!("{path}.{}", bounded(key)), ...)`. No test-visible change for benign keys.

### F0470 — src/json_strict.rs:9-12 — max_string_bytes exempts object keys [bug · medium]
- Symbols: `Limits.max_string_bytes`, `enforce_bounds`.
- Root cause: only child `Value::String` lengths are checked; object keys are never checked, so `{"<huge key>": 1}` exceeds the advertised per-string bound and the key bytes flow unchecked into error paths.
- Evidence: confirmed — Object arm iterates keys without length check.
- Remediation: in the Object arm, before recursing, error when `key.len() > limits.max_string_bytes` (report via `bounded(&key)`) — or document the exemption on `Limits`.

### F0481 — src/lifecycle/mod.rs:463-469 — residual lost-update window on --apply [bug · medium]
- Symbols: `execute_transition` apply branch (re-read compare + `io::write_atomic`).
- Root cause: no lock spans load → prepare → compare → replace; a concurrent writer using atomic rename can satisfy the byte comparison and still be overwritten in the instant between `fs::read` returning and `tmp.persist()`, silently discarding its update.
- Evidence: confirmed — compare-then-rename with no advisory lock or identity re-verification.
- Remediation: hold an advisory lock (flock on a dedicated `.lock` sentinel created with `create_new`) across the whole read-evaluate-write span, or open the record once before rendering, record dev/ino (unix) / GetFileInformationByHandle (windows), and re-verify identity on the same handle before replacement. Add a concurrency test with two overlapping `--apply` transitions.

### F0492 — src/lifecycle/record.rs:881-883 — depth/bounds enforced post-hoc on fully parsed tree [performance · medium]
- Symbols: `record::parse` → `StrictValue::deserialize` then `enforce_value_bounds` (path String allocated per node).
- Root cause: the entire (up to 2 MiB) input is recursive-descent-parsed before bounds are checked; stack protection relies solely on serde_json's 128-frame recursion ceiling (void if `unbounded_depth` is enabled anywhere in the dep graph); on success paths `format!` allocates a fresh path String per visited node.
- Evidence: confirmed — parse builds full `StrictValue`, then walks it; Cargo.toml uses default serde_json features so the crash risk is latent.
- Remediation: enforce depth during the streaming pass (depth counter in `visit_seq`/`visit_map`), defer path-string construction to the failing branch, and assert/document the serde_json feature set (no `unbounded_depth`).

### F0473 — src/main.rs:22-30 — expected-finding list duplicated vs exit_code() [maintainability · medium]
- Symbols: `main` match arm vs `exit_code` (src/error.rs:409-440).
- Root cause: the suppressed-diagnostic arm hardcodes the seven expected-finding variants that `exit_code()` also maps to 1; nothing keeps both sites in sync — a new variant added to `exit_code` but forgotten here prints `Error: …` with exit 1, and empty-Display variants would print a bare `Error: \n`.
- Evidence: both lists confirmed identical today; `RoundTripFailed` is exit-1 with a message and correctly falls to the generic arm.
- Remediation: add `ForgeError::is_expected_finding(&self) -> bool` next to `exit_code()` in src/error.rs with a single exhaustive `matches!`; main.rs becomes `Err(e) if e.is_expected_finding() => ExitCode::from(exit_code(&e))`. Add a unit test pinning `is_expected_finding` for every exit-1 empty-Display variant.

### F0509 — src/mapping/baseline.rs:301-308 — prop_value first-wins on duplicate FORGE props [security · medium]
- Symbols: `prop_value`, `require_prop`.
- Root cause: duplicate FORGE-ns props (`subject-sha256`, `raw-sha256`, `reviewed-at`) resolve first-wins; a seeded second prop can mask or fabricate a content change in the tamper-evidence diff. The framework analyzer explicitly rejects such ambiguity (src/framework/analysis.rs:882-884).
- Evidence: confirmed — `iter().find()` with no duplicate detection.
- Remediation: replace with `require_unique_prop(props, name, path) -> Result<&str, ForgeError>` that errors on >1 matching (name, ns) prop, mirroring framework analysis; use it in `verify_integrity`/`inspect_items`/`compare_resources`/`review_evidence`. Add a test: baseline item with two conflicting `subject-sha256` props fails `analyze`.

### F0537 — src/mapping/inventory.rs:185-191 — inventory snapshot equivalence ignores content [bug · medium]
- Symbols: `LoadedResource::snapshot` / snapshot compare in `load` (`ResourceInventorySnapshot`).
- Root cause: equivalence covers only root uuid, versions, and control/statement id sets; per-subject fingerprints (title/prose digests), excerpts, ineligible parts, and group hierarchies are excluded. With optional `expected_sha256`, an artifact can be edited in place (retitled controls, rewritten prose) and still satisfy `manifest.inventory`, republishing mappings under new `subject-sha256` evidence no reviewer attested.
- Evidence: confirmed — snapshot struct has exactly those five fields.
- Remediation: add `fingerprint_digest: Option<String>` to `ResourceInventorySnapshot` (serde default; decide migration policy for existing manifests), computed as a stable digest over sorted (type, id, fingerprint) pairs; populate in `snapshot()` and the `load` compare. Update mapping manifest fixtures/snapshots carrying `inventory`.

### F0539 — src/mapping/inventory.rs:219-220 — version error evicted by truncation off-by-one [bug · medium]
- Symbols: `validate_schema` (`errors` vec, `MAX_SCHEMA_ERRORS = 100`).
- Root cause: the OSCAL-version error is pushed after schema errors and `truncate(MAX_SCHEMA_ERRORS)` runs afterwards; with exactly `MAX_SCHEMA_ERRORS` schema errors plus a version error, truncation silently deletes the version diagnostic — the reviewer only sees "additional schema errors omitted".
- Evidence: confirmed ordering: collect schema errors → append version error → truncate.
- Remediation: capture the schema-error count taken from the iterator before appending the version error; truncate only schema errors; append the version error after truncation so it always surfaces. Test: `MAX_SCHEMA_ERRORS` schema violations + unsupported oscal-version; assert the version message survives.

### F0538 — src/mapping/inventory.rs:408-414 — canonical fingerprint depends on serde_json build flags [bug · medium]
- Symbols: `canonical_subject_sha256`, `strip_forge_fingerprint_props`.
- Root cause: hashes `serde_json::to_vec` output of a `Value`; key order is BTreeMap-sorted only without the `preserve_order` feature (feature unification is outside this crate's control) and number formatting diverges across toolchains ("1" vs "1.0", "1e2" vs "100"), so re-serialized equivalent subjects fingerprint differently. Also `value.clone()` deep-clones every subject.
- Evidence: confirmed; Cargo.toml uses default serde_json features, so the risk is latent but real.
- Remediation: hash a self-defined canonical form — recursive key-sorted serializer (`write_canonical_json(&mut Vec<u8>, &Value)`, RFC 8785-style) — and strip in place on the owned value instead of cloning. Add tests pinning fingerprints for reordered-object and number-format variants.

### F0540 — src/mapping/inventory.rs:448-452 — read_bounded_json TOCTOU + no symlink rejection [security · medium]
- Symbols: `read_bounded_json` (`io::check_file_size` + `std::fs::read`); callers `load` (artifact + resolved_catalog).
- Root cause: stat-then-read leaves a growth window past the 50 MiB bound; neither this helper nor its callers reject symlinks or verify containment, unlike the framework loader (`io::regular_file_metadata` at framework/analysis.rs:278/374/565) — mapping inputs follow symlinks anywhere on disk.
- Evidence: confirmed; HEAD b22e2d5 hardened only the successor-map open, not this path.
- Remediation: `io::regular_file_metadata(path, label)` first, then `File::open` → `take(MAX_FILE_SIZE + 1)` → `read_to_end` → length assertion on the held descriptor. Make this a shared io:: helper reusable by F0484/F0485/baseline sites.

### F0477 — src/mapping/manifest.rs:510-521 — eager path formatting + raw keys in enforce_value_bounds [performance · medium]
- Symbols: manifest `enforce_value_bounds` (MAX_DEPTH 64, String path).
- Root cause: `format!` allocates a fresh heap String per array/object node on the success path (a 2 MiB manifest holds hundreds of thousands of nodes); object keys are interpolated raw and unbounded into error strings, unlike `bounded()` used elsewhere in the module.
- Evidence: confirmed; module-local `bounded()` exists (manifest.rs:525-527) but is not used in the Object arm.
- Remediation: reusable `&mut String` path buffer with push/truncate per child; render failing keys through `bounded()` only on error. Same pattern as F0469.

### F0485 — src/mapping/mod.rs:128-131 — scaffold_resource TOCTOU on resource read [security · medium]
- Symbols: `scaffold_resource` (`io::check_file_size` + `std::fs::read`).
- Root cause: same stat-then-read window as prepare(); a file grown or symlink-swapped between check and read bypasses MAX_FILE_SIZE.
- Evidence: confirmed.
- Remediation: centralize bounded reads (see F0540 fix) or add a post-read `bytes.len() as u64 > io::MAX_FILE_SIZE` assertion here.

### F0483 — src/mapping/mod.rs:168 — init scaffold attestation=false fails its own workflow [security · medium]
- Symbols: `scaffold_resource` final `resolved_catalog_attestation: resolved_catalog.map(|_| false)`; `manifest::validate_resource` requires `Some(true)` (manifest.rs:442-446).
- Root cause: a fresh `forge mapping init` scaffold containing a Profile records `resolved_catalog_attestation: false`, so `forge mapping build` on the init output errors with "resolved_catalog_attestation must be true". The workflow requires a reviewer edit before build, but nothing documents that; the false value silently encodes "not yet reviewed" on a file whose purpose is reviewer editing. `mapping_cli_test.rs` uses Catalog-only inits, so CI never exercises this path.
- Evidence: both the scaffold line and the strict `Some(true)` requirement confirmed; README says "explicit `resolved_catalog_attestation: true` before analysis" but does not state init output intentionally fails.
- Remediation: document the intentional fail-until-attested contract in `execute_init` docs + README mapping section (init scaffold requires setting `resolved_catalog_attestation: true` after reviewing the companion before `build`/`check`), and improve the validate error to point at the init workflow. Do not flip to `Some(true)` without a product decision — attestation must remain human.

### F0487 — src/mapping/mod.rs:361-369 — finding-code string literals duplicated across modules [maintainability · medium]
- Symbols: `review_required` (mod.rs) vs producer literals in baseline.rs:214/216/233/263.
- Root cause: `MappingFailOn::Stale` gates on literal `"stale_reference" | "subject_type_changed"`, `SubjectChange` on `"subject_changed"`, `GapIncrease` on `"new_gap"` — bare strings duplicated from baseline.rs with no single source of truth; drift silently breaks fail-on gating.
- Evidence: all literals confirmed in both files.
- Remediation: define `pub const CODE_STALE_REFERENCE`, `CODE_SUBJECT_TYPE_CHANGED`, `CODE_SUBJECT_CHANGED`, `CODE_NEW_GAP` (+ `CODE_GAP_CHANGED`, `CODE_MAP_ADDED`, etc.) in src/mapping/baseline.rs and use them at both emit and match sites. Add a test asserting each gate fires for its emitted code.

### F0495 — src/mapping/model.rs:413-419 — ValidationSummary gates fabricated, mutated post-hoc [maintainability · medium]
- Symbols: `build` (model.rs) `ValidationSummary { manifest_valid: true, resources_valid: true, references_valid: true, mapping_schema_valid: false }`; mutation `product.report.validation.mapping_schema_valid = true` at mapping/mod.rs:204; `findings` filled later by `baseline::analyze`.
- Root cause: the machine-readable gate values are hardcoded constants truthful only because callers happen to invoke stages in the right order; any other caller of public `build()` (or a prepare() refactor) silently ships a report claiming schema-invalid or omitting baseline findings. `status: "complete"` is pinned before findings exist.
- Evidence: confirmed at both sites.
- Remediation: encapsulate — a `ValidationSummary` constructor/contract documenting "flags mean all preceding stages succeeded"; pass the schema-validation result into the report instead of mutating it post-hoc (e.g. `finalize_validation(schema_result)`); set `status` only after baseline analysis completes.

### F0496 — src/mapping/model.rs:478-481 — excerpt cap silently drops with biased sample [bug · medium]
- Symbols: `report_excerpts` (`MAX_REPORT_EXCERPTS = 1_000`, early `return excerpts`).
- Root cause: hitting the cap returns early with no truncation marker; because ("source", Control/Statement) is enumerated first, an inventory pair exceeding 1000 yields only source excerpts — a biased sample — while the report claims `status: "complete"`. Sibling bounds fail loudly (framework/analysis.rs:1110-1115, baseline.rs:58).
- Evidence: confirmed early-return; no report field records drops.
- Remediation: either return `Err` past the cap like siblings (preferred for consistency), or add a report field recording dropped count. Update report struct + any snapshot tests.

### F0497 — src/mapping/model.rs:482-487 — unwrap_or("") masks absent excerpts [bug · medium]
- Symbols: `report_excerpts` → `inventory.excerpt(subject_type, &id).unwrap_or("")`.
- Root cause: legitimately absent excerpts become empty-string entries; recipients cannot distinguish "empty text" from "never captured". `Inventory::excerpt` returns Option precisely so absence is observable.
- Evidence: confirmed.
- Remediation: skip absent entries (`let Some(excerpt) = ... else { continue; }`) or mark absence explicitly in `ReportExcerpt`. Update report snapshots including excerpts if any.

### F0520 — src/migration/engine.rs:591-595 — reconciliation invariant error carries no detail [bug · medium]
- Symbols: `validate_reconciliation`.
- Root cause: six unrelated invariant violations (duplicate/missing IDs on either side, counter mismatches) collapse into the constant string "internal reconciliation invariant failed" — for a classifier-bug error that is the primary forensic signal. Sibling modules report specific identifiers.
- Evidence: confirmed (engine.rs:591-595).
- Remediation: compute truncated samples (take(3)) of duplicated_old/duplicated_new IDs, missing_old/missing_new differences, and the two counter pairs; format them into the `MigrationError`. Add a unit test forcing one violation class and asserting the message names the offending ID.

### F0524 — src/migration/formatter.rs:130-141 — escape_controls allocates per character [performance · medium]
- Symbols: `escape_controls`.
- Root cause: `flat_map` heap-allocates a `Vec<char>` per character (a 1-element vec for normal text); labels/section paths/rationales up to 64 KiB each flow through here, so large reports pay millions of tiny allocations.
- Evidence: confirmed.
- Remediation: fast path `if !value.chars().any(char::is_control) { return value.to_string(); }`, else `String::with_capacity(value.len())` pushing chars / `escape_default()` iterators. Keep the existing test.

### F0515 — src/migration/inventory.rs:116-119 — duplicate-ID error lacks location context [other · medium]
- Symbols: `validate_unique_ids`.
- Root cause: the error names only the stable ID; `InventoryRequirement.location` (file_label, section_path, line) is available on both colliding entries but unused, leaving operators to grep the corpus.
- Evidence: confirmed; `RequirementLocation` fields available on `pair[0]`/`pair[1]`.
- Remediation: include both colliding entries' file_label + section_path + line in the message. Update any test asserting the old message.

### F0501 — src/migration/inventory.rs:16-17 — ForgeError flattened into MigrationError(String) [maintainability · medium]
- Symbols: `build_inventory` — `prepare_document(...).map_err(|error| ForgeError::MigrationError(error.to_string()))`.
- Root cause: `prepare_document` already returns `ForgeError`; re-wrapping destroys the causal chain and typed classification (`exit_code` maps FileTooLarge/NoStructureDetected to 1/2 but MigrationError to 2).
- Evidence: confirmed. Interacts with F0527: propagating with `?` here requires the module-boundary normalization there.
- Remediation: propagate with `?` and normalize at the `analyze_paths` boundary (F0527), keeping `MigrationError(String)` for migration-local defects only. Preserve the documented migration exit-2 contract via the boundary normalization.

### F0503 — src/migration/inventory.rs:19-21 — location_basis inferred from extension only [bug · medium]
- Symbols: `build_inventory` — `input_format(path)` from extension → `LocationBasis`.
- Root cause: a PDF renamed `.md` persists `SourceLine` while `source_line` actually refers to normalized extracted-text lines, corrupting provenance metadata downstream consumers rely on.
- Evidence: confirmed — no content-sniff verification against the declared extension.
- Remediation: derive `location_basis` from ingestion facts (the ingested record knows whether raw source lines survive) or cross-check sniffed content type against the extension and fail on mismatch.

### F0516 — src/migration/inventory.rs:91-95 — missing stable ID error lacks localization [maintainability · medium]
- Symbols: `inventory_requirement`.
- Root cause: the error names neither the file nor the requirement's section/title/source_line; in multi-document migrations the operator cannot locate the defect. `file_label` and `section` are in scope.
- Evidence: confirmed.
- Remediation: `format!("shared pipeline returned a requirement without a stable ID in '{file_label}', section '{}', source line {}", section.title, requirement.source_line)`.

### F0527 — src/migration/mod.rs:37-39 — exit-2 normalization contract unenforced [maintainability · medium]
- Symbols: `analyze_paths` (bare `?` on `build_inventory`, `successor::load`, `engine::classify`).
- Root cause: docs promise all failures normalize to `MigrationError` for the exit-2 contract, but nothing structural enforces it; a future `?` in a helper leaking e.g. `FileNotFound` silently turns exit 2 into exit 1.
- Evidence: confirmed; today the invariant holds only because every inner variant is rewritten.
- Remediation: add `fn normalize_to_migration(ForgeError) -> ForgeError` (passthrough for `MigrationError`, wrap otherwise) and apply at each `?` boundary in `analyze_paths`. README documents migration exit 2 — preserve it via normalization (coordinate with F0501).

### F0555 — src/migration/successor.rs:153-154 — chains and cycles validate successfully [bug · medium]
- Symbols: `validate` (per-role `used_old`/`used_new` uniqueness).
- Root cause: {"old":["A"],"new":["B"]} + {"old":["B"],"new":["A"]} passes (each ID once per role, no self-map); chained/cyclic declarations make multi-hop redirection ambiguous. The current engine matches relationships independently, so the harm is contract ambiguity rather than an active infinite loop today.
- Evidence: confirmed — no intersection check between `used_old` and `used_new`.
- Remediation: after the validation loop, reject any identifier appearing in both roles: `if let Some(id) = used_old.intersection(&used_new).next() { return Err(...) }` with `bounded(id)`. Add tests for 2-cycles and chains (A→B, B→C).

### F0556 — src/migration/successor.rs:160 — self-map check depends on hidden sorted precondition [maintainability · medium]
- Symbols: `validate` — `relationship.new_ids.binary_search(id)` after `normalize_ids` sorted in place.
- Root cause: correctness rests on the preceding in-place sort; reordering the calls silently turns the containment test into a no-op. Sizes are capped at `MAX_IDS_PER_RELATIONSHIP = 1000` so `Vec::contains` is free.
- Evidence: confirmed.
- Remediation: replace with `relationship.old_ids.iter().any(|id| relationship.new_ids.contains(id))`, or keep binary_search with a `debug_assert!` sorted precondition + comment. Add a regression test pinning self-map rejection.

### F0532 — src/migration/types.rs:221-224 — MigrationSummary sum contract unguarded [maintainability · medium]
- Symbols: `MigrationSummary` docs ("sum to total_old/total_new"); all fields `pub`.
- Root cause: nothing enforces the documented counting invariants; the struct is constructible field-by-field anywhere and a producer bug publishes internally contradictory reports.
- Evidence: confirmed — the only construction is `engine::summarize`, guarded by `validate_reconciliation`, but the guard lives far from the contract.
- Remediation: add `MigrationSummary::validate(&self) -> Result<(), &'static str>` checking both sums; call it in `validate_reconciliation` and document it as the module contract.

### F0535 — src/migration/types.rs:279-285 — location-drift allowlist is a silent matches! catch [maintainability · medium]
- Symbols: `MigrationReport::has_reviewable_changes` `matches!` over four `EvidenceCode` variants.
- Root cause: a future drift-expressing `EvidenceCode` variant defaults to NOT reviewable unless someone remembers this allowlist; the compiler won't flag it because there is no exhaustive match.
- Evidence: confirmed.
- Remediation: co-locate `impl EvidenceCode { pub const fn indicates_location_drift(self) -> bool }` next to the enum and use `entry.evidence.iter().any(EvidenceCode::indicates_location_drift)`. Add a test enumerating all variants' drift classification.

### F0531 — src/migration/types.rs:62-65 — Classification parallel orderings (derived Ord vs rank()) [maintainability · medium]
- Symbols: `Classification` derive `PartialOrd, Ord` + `rank()` (types.rs:62-107); `sort_entries` (engine.rs:500-507) uses `rank()` only.
- Root cause: two parallel orderings with nothing tying them together; reordering variants shifts derived `Ord` while `rank()` keeps stale numbers. No caller uses Classification's derived Ord (grep across src/ shows only `rank()` used for precedence).
- Evidence: confirmed.
- Remediation: drop `PartialOrd, Ord` from `Classification` and implement `rank` as `self as u8` with a doc comment that declaration order defines precedence. KEEP `EvidenceCode`'s Ord — `evidence.sort_unstable()` (engine.rs:218) relies on it.

### F0533 — src/migration/types.rs:8-13 — SourceProvenance.sha256 unconstrained string [maintainability · medium]
- Symbols: `SourceProvenance.sha256`.
- Root cause: in a versioned external schema the digest ships as a free-form String with no casing/format constraint; nothing marks future algorithm migrations.
- Evidence: confirmed — populated from ingestion's fingerprint (lowercase hex today) but unvalidated at the type boundary.
- Remediation: document the contract (lowercase hex, 64 chars) and/or validate with `crate::json_strict::validate_lowercase_sha256` at inventory build time, or a `Sha256Hex(String)` newtype.

### F0574 — src/model/assemble.rs:211 — dead error contract on assemble_document [maintainability · medium]
- Symbols: `assemble_document` (`# Errors` promises `ForgeError::Parse` on invalid section trees).
- Root cause: the body performs no structural validation and can only return Ok; malformed trees (equal/out-of-order sibling source_lines — the range-math precondition) flow unchecked, and callers propagate an impossible error.
- Evidence: confirmed — no validation in the body.
- Remediation: validate the invariant (`sections.windows(2)` ascending `source_line` check returning `ForgeError::Parse`) since `map_sections_recursive` range math depends on it, or simplify the signature to return `PolicyDocument`. Update `pipeline::prepare_document` callers accordingly.

### F0575 — src/model/assemble.rs:78-90 — three identical PolicyRequirement literals [maintainability · medium]
- Symbols: literals at assemble.rs ~48-56 (Preamble fallback), ~81-89 (preamble_items), ~138-146 (map_sections_recursive).
- Root cause: each copy hardcodes the same placeholder defaults (stable_id: None, atom_index: 0, empty citations/modality/parameters, parent_text: None); adding a field requires editing all three in lockstep.
- Evidence: confirmed three sites.
- Remediation: factor `fn forge_requirement(item: &ExtractedListItem) -> PolicyRequirement` and use it in all three places. No behavior change; no snapshot impact.

### F0571 — src/model/frontmatter.rs:19-20 — unknown frontmatter keys silently discarded [maintainability · medium]
- Symbols: `FrontmatterData` (no `deny_unknown_fields`).
- Root cause: a typo like `Titel:` silently disappears; downstream falls back to H1 titles with no diagnostic.
- Evidence: confirmed.
- Remediation: deserialize into `serde_yaml::Value` first, `tracing::warn!` for keys not in {title, version, author, date}, then deserialize into the typed struct — or at minimum document the silent-drop tradeoff in the struct docs. Consistent with the fault-tolerance posture (never fail on unknown keys).

### F0581 — src/model/frontmatter.rs:61-62 — fence detection needs line-oriented scan [bug · medium]
- Symbols: `parse_frontmatter` closing-delimiter `or_else` chain.
- Root cause: searching for fences as substrings requiring a preceding newline (1) drops immediately-closed fences ("---\n---\n" → `rest` BEGINS with the closer, all four arms fail) and (2) under mixed LF/CRLF, a later LF-style "\n---\n" can beat an earlier CRLF closer, splicing intervening text into yaml_str.
- Evidence: confirmed — for "---\n---\n# Body" no alternative matches.
- Remediation: line-oriented scan: iterate `rest.split_inclusive('\n')`, trim `['\n','\r']`, first line equal to "---" closes at that cursor; treat final unterminated "---" as valid closer; uniform CRLF tolerance. Tests: immediately-closed fence, mixed CRLF/LF ordering, final-line closer. (Supersedes F0569's proposed fix.)

### F0549 — src/model/mod.rs:172-173 — requirement_id bare String [maintainability · medium]
- Symbols: `PolicyParameter.requirement_id`, `Citation.source_requirement_id` (`Option<String>`), `PolicyParameter.id` convention `{requirement_id}_prm_{position}`.
- Root cause: no referential integrity; stale/malformed IDs flow unchecked into OSCAL output; cross-links are indistinguishable from arbitrary strings.
- Evidence: confirmed.
- Remediation: newtype `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)] pub struct RequirementId(pub String)` for both fields; enforce/build the parameter id format via a constructor. Update producers (parameter/mod.rs, citation.rs) and any snapshot carrying these fields.

### F0551 — src/model/mod.rs:267-269 — collect_citations emits duplicate IDs [bug · medium]
- Symbols: `PolicyDocument::collect_citations`; consumer `generate_back_matter` (oscal/back_matter.rs:234+).
- Root cause: blindly extends with cloned citations; `Citation.id` is documented unique, but multiple requirements citing the same source emit duplicate IDs into back matter. `generate_back_matter` pushes one resource per citation with no dedup (only `resource_map` collapses via map.insert), so the resource list duplicates while links survive.
- Evidence: confirmed in both functions.
- Remediation: walk collecting `&Citation` refs, dedup by `id` keeping the first occurrence, clone survivors; pre-reserve with `total_requirements()`. Add a test with two requirements sharing a citation id.

### F0552 — src/model/mod.rs:282-284 — aggregation test coverage gap [test · medium]
- Symbols: `PolicySection::total_requirements`, `PolicyDocument::total_requirements`, `collect_citations`.
- Root cause: model/mod.rs unit tests stop at `total_sections*`; the recursive requirement summation and citation aggregation have no unit tests (tests/pipeline_test.rs and fixture_validity_test.rs exercise them only incidentally on fixtures).
- Evidence: confirmed — no dedicated unit tests in the module.
- Remediation: add unit tests: nested sections with requirements at multiple depths; `total_requirements` over empty doc and children-only doc; `collect_citations` across sibling/child sections including duplicate ids (locking the F0551 dedup semantics once added) and `source_requirement_id` propagation.

### F0550 — src/model/mod.rs:64-75 — DocumentMetadata::default() violates documented contract [bug · medium]
- Symbols: `DocumentMetadata` (`#[derive(Default)]`; docs promise version fallback "0.0.0").
- Root cause: derived `Default` yields `version: ""`; the struct is used via `..Default::default()` in production builders (`src/oscal/assessment_plan.rs:227-230`, `src/oscal/profile.rs:223-226`, `src/oscal/ssp.rs:583-586`) and tests (`parse/modality.rs:289`), so an empty version can leak into OSCAL metadata by convention alone.
- Evidence: confirmed — production `..Default::default()` usages exist (they override version today, but only by convention).
- Remediation: implement `Default` manually with `version: "0.0.0".to_string()` (other fields as derived), or drop the derivation and require complete construction. Add a unit test asserting `DocumentMetadata::default().version == "0.0.0"`.

---

## PARTIAL findings

### F0458 — src/io.rs:74-76 — manifest_relative_path symlink resolution + String errors [maintainability · medium]
- Rationale: the symlink-resolution inconsistency with `regular_file_metadata` and the loss of `io::Error` kinds (flattened to formatted Strings, so callers can't distinguish not-found from permission-denied) are both confirmed. The finding's framing that this can fail on a nonexistent output dir is inaccurate — `manifest_dir_path.is_dir()` is pre-checked (io.rs:80-82). Remediate the real parts: reject/flag symlinked resources consistently with sibling APIs; return a typed error carrying the original `io::Error`.
- Affected: `manifest_relative_path` (+ caller `src/mapping/mod.rs::manifest_relative_path`).

### F0459 — src/io.rs:87 — unwrap_or(target) leaks absolute paths [security · medium]
- Rationale: confirmed that when `relative_path` finds no common prefix (different mounts/drives), the canonical absolute target is emitted into the manifest — the exact leak the helper exists to prevent. The cited "nonexistent output dir" trigger is wrong (pre-checked at io.rs:80). Remediate: fail loudly when relativity cannot be established, or normalize to a URI/copy the resource next to the manifest.
- Affected: `relative_path`, `manifest_relative_path`.

### F0471 — src/json_strict.rs:15 — Result<Value, String> flattens failure classes [maintainability · medium]
- Rationale: the `Result<Value, String>` contract and brittle `contains("invalid trailing ...")` test coupling are confirmed, but this is an internal `pub(crate)` API whose string messages are the user-facing diagnostic style used crate-wide in mapping/lifecycle validators. Classification is genuinely impossible but blast radius is limited to message assertions. Low-priority: introduce a small error enum only if callers need programmatic classification; otherwise record as accepted design.

### F0472 — src/json_strict.rs:17-18 — bounds validated only after full decode [performance · medium]
- Rationale: post-hoc bounds are confirmed and cannot bound aggregate footprint during decode. But the claim that enforcement "relies entirely on callers capping raw input size upstream" is overstated: `successor::parse` checks `bytes.len() > MAX_SUCCESSOR_MAP_BYTES` before parsing (successor.rs:131-135), bounding the worst case to 2 MiB. The serde_json recursion-limit interaction is accurate and worth a module-level doc note. Remediate: document the contract (byte cap is the DoS bound; structural bounds are semantic); optionally enforce during descent.

### F0491 — src/lifecycle/record.rs:697-700 — event_id seed fragility [maintainability · medium]
- Rationale: claims (1) and (2) are valid — the seed embeds collections in stored order (validator enforces sortedness only for `FingerprintSet.generated_artifacts` and assertions; `PolicyIdentity.generated_artifacts` ordering is producer-dependent), and `serde_json::to_vec` shape depends on field order/serde settings, so restructuring silently changes every recomputed event_id. Claim (3) "no regression test" is wrong: `tests/lifecycle_cli_test.rs::event_ids_bind_policy_parties_approval_rules_and_review_schedule` and `assertion_order_and_duplicates_do_not_change_event_bytes` pin recomputation behavior. Remediate the real parts: freeze the seed layout with doc comments + exact-UUID golden-vector tests for both /1 and /2 seeds, and embed a `canonical_seed_version` in the namespace derivation name.
- Affected: `context_event_id`, `legacy_event_id`.

### F0493 — src/lifecycle/record.rs:911-913 — validator test coverage [test · medium]
- Rationale: partially overstated. `tests/lifecycle_cli_test.rs` covers more than claimed: role-count accumulation (configurable_role_counts_accept_distinct_assertions), separation-of-duties (author_separation_requires_declared_author_evidence, separation_failure_and_retired_terminal_leave_record_unchanged), supersession/replaced_by (portfolio_check_rejects_supersession_cycle, superseded_record_can_retire_without_losing_replacement_evidence), duplicate-key rejection. Genuinely missing: golden event-ID vectors (see F0491), timestamp tie/order rejection, per-bound rejection tests (MAX_PARTIES, MAX_ARTIFACTS, MAX_EVENTS, MAX_ASSERTIONS, MAX_IMPACT_FINDINGS, MAX_COLLECTION_ITEMS, MAX_DEPTH, MAX_RECORD_BYTES), and direct parse→validate fixture round-trips. Remediate only the genuinely missing table-driven tests.

### F0474 — src/main.rs:31-34 — exit 1 overloaded [other · medium]
- Rationale: the observation is accurate (benign findings and genuine I/O failures both exit 1; stderr is the only distinguishing signal), but the README already documents per-subcommand exit-code semantics ("Exit codes are 0 ... 1 ... 2" for diff/validate, mapping, lifecycle, applicability, framework). Remediate only: add one explicit sentence in README/CLI help that exit 1 is shared by findings and I/O errors and stderr distinguishes them — or reserve a dedicated exit range for expected findings (breaking change).

### F0507 — src/mapping/baseline.rs:112-116 — compare_resources/compare_gaps diff only first mapping [bug · medium]
- Rationale: confirmed that `compare_resources` and `compare_gaps` use `mappings.first()` with early return while `verify_integrity` and `compare_maps` (`maps_by_uuid`) walk all mappings. But `model::build` produces exactly one mapping (`mappings: vec![OscalMapping {...}]`), so FORGE-authored baselines always carry one; the inconsistency bites only externally authored multi-mapping baselines admitted by the lenient `Vec<OscalMapping>`. Remediate (cheapest correct fix): fail closed with a clear error when `baseline.mapping_collection.mappings.len() != 1` or current's differs, before analyzing.
- Affected: `compare_resources`, `compare_gaps`.

### F0505 — src/mapping/baseline.rs:19 — doc says append, code replaces [bug · medium]
- Rationale: the doc/implementation divergence is confirmed (`report.findings = findings` at line ~62 vs "append findings"), but today's sole caller `mapping::prepare` passes a fresh report with empty findings, so no data is lost in shipped behavior. Latent contract bug only. Remediate: honor the doc (`report.findings.extend(findings)`) or correct the doc to replace semantics — one-line fix either way.

### F0484 — src/mapping/mod.rs:188-191 — prepare() TOCTOU manifest read [bug · medium]
- Rationale: stat-then-read TOCTOU confirmed, but the claim "no defensive check after reading" is inaccurate for the payload: `manifest::parse` re-checks `bytes.len() as u64 > MAX_MANIFEST_BYTES` as its first action (manifest.rs:300-302), so the byte bound IS asserted post-read — the window only affects the pre-parse rejection path and memory spike, not admission. Remediate: assert `manifest_bytes.len()` immediately after `fs::read` (defense in depth) or switch to a bounded read on an open handle (see F0540).

### F0534 — src/migration/types.rs:187-191 — DeclarationEvidence PII/format undocumented [security · medium]
- Rationale: `approved_by`/`approved_at` are indeed unconstrained strings in a published machine-readable report. But the RFC 3339 format IS validated at the successor-map boundary (`chrono::DateTime::parse_from_rfc3339` in successor.rs validate), so "no declared format" holds only for the report output contract, not ingestion. The PII documentation gap is real. Remediate: document on `DeclarationEvidence` that `approved_by` is personal data (must not reach logs/non-report diagnostics) and `approved_at` is RFC 3339 UTC.

### F0576 — src/model/assemble.rs:241-242 — tables/paragraphs dropped without accounting [maintainability · medium]
- Rationale: confirmed `assemble_document` consumes only `clauses.list_items`; `tables`/`paragraphs` are dropped (used only for the NoStructureDetected check in pipeline.rs). Currently by-design (assembly is list-item-driven) but undocumented and silent, conflicting with the file's SEC-5 no-silent-drop posture. Remediate: document the intentional exclusion in the function docs, or emit a `tracing::warn!` with dropped counts when non-empty.

### F0569 — src/model/frontmatter.rs:58-59 — immediately-closed fence missed (fix superseded) [bug · medium]
- Rationale: the defect is real ("---\n---\n" valid frontmatter reported absent), but the finding's own prose admits confusion and its proposed fix (trim one trailing newline) does NOT fix the case — `rest` BEGINS with the closer, so trimming a suffix changes nothing. F0581 correctly diagnoses and fixes this. Marked partial: valid defect, wrong prescription, subsumed by F0581's line-scan fix (not duplicate because the two findings carry distinct remediation content and F0581 explicitly corrects F0569).

---

## INVALID findings

### F0510 — src/mapping/baseline.rs:288-290 — subject_keys ordered comparison
- Rationale: false positive. `model::build_map` sorts both `sources` and `targets` by `(subject_type, id_ref)` before emitting the OSCAL artifact, so every FORGE-produced baseline's item lists are canonically ordered; the claimed reorder false-positives cannot occur on artifacts this tool produces and compares.

### F0498 — src/mapping/model.rs:577-582 — duplicate subjects within one sources/targets list
- Rationale: already enforced upstream. `manifest::validate_subjects` (manifest.rs:467-488) rejects duplicate `(subject_type, id_ref)` within each sources/targets list before `build_items` runs ("{path}[{index}] duplicates ..."), so the described artifact cannot validate cleanly.

### F0504 — src/migration/inventory.rs:77-80 — collect_section unbounded recursion
- Rationale: the section tree is built by `parse::extract_sections`, whose nesting is bounded by heading levels 1-6 (SEC-4 in parse/mod.rs); `collect_section` only walks pipeline-produced trees, so recursion depth cannot approach stack exhaustion. The `MAX_SECTION_DEPTH = 50` guards cited defend other contexts against hand-built trees.

---

## DUPLICATE findings

None within this slice. Close pairs deliberately kept separate:
- F0484/F0485/F0540/F0432: same stat-then-read pattern at distinct callsites with different limits/labels — consolidated remediation is one shared bounded-read helper, but each finding documents a separate location.
- F0569/F0581: same fence bug; F0569 partial (wrong fix), F0581 primary (correct fix).
- F0469/F0477/F0492: same eager-path-formatting shape in three modules — fix together but report separately.

---

## Validation notes

- All verdicts judged against HEAD b22e2d5. The symlink-race hardening landed only for `migration::successor` (O_NOFOLLOW open + take-bounded read); the ingest, mapping, baseline, and io read paths still use stat-then-read.
- Cross-finding remediation order suggestion: (1) shared bounded-read helper in io:: fixes F0432/F0454/F0484/F0485/F0507(baseline)/F0540 together; (2) deferred-path diagnostics fix F0467/F0469/F0477/F0492 together; (3) F0473+F0474+F0527+F0501 form the exit-code contract cluster.
