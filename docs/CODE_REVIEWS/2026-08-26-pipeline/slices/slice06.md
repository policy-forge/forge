# Validation slice slice06 — 61 findings
Severity mix: medium×61


══════ F0668 │ src/round_trip/chain.rs:23-25 │ [bug · medium] ══════
[bug · medium] Intermediate filenames are hardcoded relative to the caller-supplied `temp_dir`, and
nothing here enforces exclusivity. Two chains pointed at the same directory (easy with a
shared/long-lived temp dir or parallel tests) will overwrite each other's
artifact.xml/artifact.yaml/artifact-rt.json and silently produce a corrupted round-trip rather than
failing loudly. Either generate unique per-run names, or document the precondition that callers must
pass a freshly created, exclusively-owned directory.

-     let xml_path = temp_dir.join("artifact.xml");
-     let yaml_path = temp_dir.join("artifact.yaml");
-     let rt_json_path = temp_dir.join("artifact-rt.json");
+     // Make intermediate names unique per invocation so concurrent runs
+     // sharing a temp root cannot clobber each other.
+     let run_id = std::process::id().to_string(); // or a uuid/atomic-counter token
+     let xml_path = temp_dir.join(format!("artifact-{run_id}.xml"));
+     let yaml_path = temp_dir.join(format!("artifact-{run_id}.yaml"));
+     let rt_json_path = temp_dir.join(format!("artifact-{run_id}-rt.json"));


══════ F0669 │ src/round_trip/chain.rs:49-49 │ [bug · medium] ══════
[bug · medium] The `?` propagates the inner error unchanged.
`ForgeError::OscalCliExecution`/`OscalCliTimeout` carry neither paths nor stage info, and the
contextual logging above is only at `debug` level, so a caller whose chain fails cannot tell which
leg failed (JSON→XML, XML→YAML, or YAML→JSON) or even which input was being converted. Preserve the
source error but add step attribution — ideally a dedicated error variant carrying step number and
from/to paths (with `#[source]`), at minimum an `error!` log plus wrapped message.

-         invoker.convert(&args)?;
+         invoker.convert(&args).map_err(|source| {
+             tracing::error!(
+                 step = step_num + 1,
+                 from = %input.display(),
+                 to = %output.display(),
+                 "Round-trip conversion step failed: {source}"
+             );
+             // Prefer a typed variant, e.g. ForgeError::RoundTripStep {
+             //   step: step_num + 1, from: .., to: .., #[source] source }
+             source
+         })?;


══════ F0671 │ src/round_trip/chain.rs:52-52 │ [bug · medium] ══════
[bug · medium] On a mid-chain failure, earlier intermediates remain on disk with no cleanup, and
pre-existing files are never removed before the run starts — so an
`artifact-rt.json`/`artifact.yaml` left over from a previous run survives a failing run and can
later be mistaken for a fresh, valid round-trip result by tooling scanning the temp dir. Delete
stale targets up front and best-effort-remove the intermediates when any step fails.

-     Ok(rt_json_path)
+     // Never leave a previous run's artifacts where they can be mistaken
+     // for this run's output.
+     for path in [&xml_path, &yaml_path, &rt_json_path] {
+         let _ = std::fs::remove_file(path);
+     }
+
+     let attempt = || -> Result<(), ForgeError> {
+         for /* each step */ {}
+         Ok(())
+     };
+
+     let outcome = attempt();
+     if outcome.is_err() {
+         for path in [&xml_path, &yaml_path] {
+             let _ = std::fs::remove_file(path);
+         }
+     }
+
+     outcome.map(|_| rt_json_path)


══════ F0672 │ src/round_trip/chain.rs:82-83 │ [test · medium] ══════
[test · medium] The mock ignores `args.input_path` and every test asserts only the aggregate call
count, so a regression that reorders the Xml→Yaml→Json legs, chains the wrong file as a step's
input, or drops the per-step `timeout` would still pass all three tests. Record the (input, output,
format, timeout) tuple of each call and assert the exact expected sequence in addition to the count.

          fn convert(&self, args: &ConvertArgs) -> Result<ConvertResult, ForgeError> {
+             self.calls.lock().unwrap().push((
+                 args.output_format,
+                 args.input_path.clone(),
+                 args.output_path.clone(),
+                 args.timeout,
+             ));
              let call = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
+             // ... in tests:
+             // assert_eq!(*mock.calls.lock().unwrap(), vec![
+             //     (OscalFormat::Xml, input.clone(), xml_path.clone(), timeout),
+             //     (OscalFormat::Yaml, xml_path, yaml_path.clone(), timeout),
+             //     (OscalFormat::Json, yaml_path, rt_json_path, timeout),
+             // ]);


══════ F0696 │ src/round_trip/comparator.rs:116-124 │ [maintainability · medium] ══════
[maintainability · medium] Timestamp equivalencing is hardwired to the `/last-modified` suffix only,
yet OSCAL defines sibling timestamp fields with the same serialization instability
(`metadata/published`, `metadata/updated`). Writers emitting `Z` versus `+00:00`, or differing
fractional-second precision, make those fields surface as ForgeFix 'Value mismatch' noise instead of
Acceptable. Mirror the existing configurability pattern: move the accepted-suffix list into
`OscalComparisonRules` (like `unordered_array_paths`) and consult it here, keeping the whitelist
extensible and unit-testable.

-     if path.ends_with("/last-modified")
+     if rules.acceptable_timestamp_paths.iter().any(|p| path.ends_with(p))
          && let (Ok(expected), Ok(actual)) = (
              chrono::DateTime::parse_from_rfc3339(expected),
              chrono::DateTime::parse_from_rfc3339(actual),
          )
          && expected == actual
      {
          return Some("Equivalent RFC 3339 timestamp representation");
      }


══════ F0695 │ src/round_trip/comparator.rs:250-251 │ [performance · medium] ══════
[performance · medium] Unordered matching degrades to O(n^2): every expected element performs up to
three sequential linear rescans of `act_arr` (`find_by_exact_equality` uses deep structural `Value`
equality, the most expensive one first), so large catalogs with thousands of props/links/parts pay
repeated quadratic subtree comparisons. Precompute identity maps once per array call (uuid -> index,
and (name, ns) -> indices) plus a hashed exact-equality bucket (e.g. serialize or convert to
BTreeMap) to make each lookup near O(1). Related hot-path costs worth addressing together: `format!`
allocates a new path String for every visited node, and each Divergence clones entire subtrees
(`exp_map[key].clone()`, `.clone()` arms here) — consider incremental push/pop path building and
lighter-weight summaries or Arc-sharing for diverged values.

-     for (exp_idx, exp_elem) in exp_arr.iter().enumerate() {
-         let match_idx = find_matching_element(exp_elem, act_arr, &matched_actual);
+     let mut by_uuid: std::collections::HashMap<&str, usize> = Default::default();
+     let mut by_name_ns: std::collections::HashMap<(&str, Option<&str>), Vec<usize>> =
+         Default::default();
+     for (i, act_elem) in act_arr.iter().enumerate() {
+         if let Some(u) = act_elem.get("uuid").and_then(Value::as_str) {
+             by_uuid.insert(u, i);
+         }
+         if let Some(n) = act_elem.get("name").and_then(Value::as_str) {
+             by_name_ns
+                 .entry((n, act_elem.get("ns").and_then(Value::as_str)))
+                 .or_default()
+                 .push(i);
+         }
+     }


══════ F0694 │ src/round_trip/comparator.rs:255-256 │ [bug · medium] ══════
[bug · medium] For a matched pair, `json_path` encodes the expected-side index (`{path}/{exp_idx}`)
while the compared value — and any nested divergences produced from it — originates from
`act_arr[act_idx]`. Downstream fix tooling keyed on these paths will edit the wrong array slot.
Worse, the same function already uses `{path}/{act_idx}` for extra-element reports just below, so
consumers cannot tell from the path alone which document/position an index refers to. Carry both
positions explicitly (e.g. `expected_index`/`actual_index` fields on `Divergence`, or include the
actual position in the description/resolution).


══════ F0693 │ src/round_trip/comparator.rs:299-303 │ [bug · medium] ══════
[bug · medium] Contradicts the documented matching priority ("Priority: uuid → name+ns composite →
positional fallback"): an element carrying a `uuid` returns immediately and never falls through to
name/ns or positional matching when its uuid has no unmatched counterpart. If the producing tool
regenerates uuids between runs (common on fresh exports), each affected element yields paired
'Element in expected ... not found' + 'Extra element ... not found in expected' divergences
reporting whole-object addition/removal instead of a single targeted divergence on the changed
`uuid` field. Additionally, greedy first-match pairing without any scoring/optimal assignment can
mispair near-duplicate elements and cascade spurious divergences.

      if let Some(exp_uuid) = exp_elem.get("uuid").and_then(Value::as_str) {
-         return find_by_uuid(exp_uuid, act_arr, already_matched);
+         if let Some(i) = find_by_uuid(exp_uuid, act_arr, already_matched) {
+             return Some(i);
+         }
+         // Fall through to name+ns / positional strategies per the documented
+         // priority; residual uuid drift is still surfaced by the recursive
+         // field-by-field comparison.
      }

      if let Some(exp_name) = exp_elem.get("name").and_then(Value::as_str) {


══════ F0676 │ src/round_trip/divergence.rs:104-105 │ [bug · medium] ══════
[bug · medium] `passed` is fully derivable from `divergences` yet is a free-standing public bool
with no constructor enforcing the invariant. Hand-built or partially-updated RoundTripResults can
carry `passed: true` alongside ForgeFix/OscalCliDiff entries and emit self-contradictory validation
reports. Remove the field and expose a computed method (or make the field private and set it only in
a constructor that computes it from `divergences`), so the aggregate cannot represent this invalid
state.

-     /// `true` if all divergences are `Acceptable` (zero `ForgeFix` or `OscalCliDiff`).
-     pub passed: bool,
+ impl RoundTripResult {
+     /// `true` only when every divergence is `Acceptable`.
+     #[must_use]
+     pub fn passed(&self) -> bool {
+         self.divergences
+             .iter()
+             .all(|d| d.classification == DivergenceClass::Acceptable)
+     }
+ }


══════ F0673 │ src/round_trip/divergence.rs:83-83 │ [bug · medium] ══════
[bug · medium] Unknown oscal-cli versions are classified as AdvisoryOlderModelBaseline, whose
contract states the tool 'is documented as using an older OSCAL model'. For a version with no
documentation (e.g. "9.9.9", "0.9.0", or any future release) this asserts an undocumented baseline
claim in persisted evidence reports — the classifier fabricates facts it does not have. Introduce a
distinct variant (e.g. UnverifiedBaseline) that honestly represents 'conversion observed / version
unrecognized, baseline unassessed', keeping AdvisoryOlderModelBaseline strictly for versions with a
known older documented baseline.

-         Some(_) => (CompatibilityClassification::AdvisoryOlderModelBaseline, None),
+         // Unknown tool versions carry no documented model evidence; do not
+         // reuse the "older model" wording, which asserts facts we lack.
+         Some(_) => (CompatibilityClassification::UnverifiedBaseline, None),


══════ F0680 │ src/round_trip/log.rs:21-26 │ [bug · medium] ══════
[bug · medium] `File::create` truncates `output_path` up front, and the JSON is streamed straight
into the file. If serialization fails partway (or the disk fills up / the writer hits an I/O error
mid-stream), an existing divergences.json is replaced with a partial, corrupt file — destroying the
very divergence evidence this log exists to record. Serialize to an in-memory buffer (the payload is
small diagnostic metadata) and write it only after serialization succeeds; for strict crash-safety,
write to a temp file in the same directory and `rename` it over the destination.

-     if let Err(e) = serde_json::to_writer_pretty(file, result) {
-         if e.is_io() {
-             return Err(ForgeError::Io(e.into()));
-         }
-         return Err(ForgeError::Serialization(e.to_string()));
-     }
+     // Serialize fully before touching the destination so a failure
+     // cannot truncate an already-valid divergence log.
+     let json = serde_json::to_vec_pretty(result)
+         .map_err(|e| ForgeError::Serialization(format!("divergence log serialization failed: {e}")))?;
+     std::fs::write(output_path, json)
+         .map_err(|e| ForgeError::Io(e))?


══════ F0701 │ src/round_trip/rules.rs:7-8 │ [maintainability · medium] ══════
[maintainability · medium] Name/semantics mismatch: `unordered_array_paths` does not hold paths — it
holds bare object-key names. The comparator matches an array as unordered whenever the *last
segment* of its JSON Pointer equals one of these strings (comparator.rs: `let key_name =
path.rsplit('/').next(); ... contains(key_name)`). Two consequences users of this public field
should be told about: (1) passing an RFC 6901 pointer such as "/system-security-plan/props" matches
nothing, silently disabling the rule; (2) matching is key-name-based at *any* nesting depth, so
every `props`/`links`/`parts` array in the whole tree is compared unordered. Please either rename
(e.g. `unordered_array_keys`) or extend the doc comment to state the exact last-segment, any-depth
matching contract.

-     /// JSON key names whose array values are compared without regard to element order.
-     pub unordered_array_paths: HashSet<String>,
+     /// Bare object-key names (not JSON Pointers): any array whose value sits under
+     /// a key with one of these names — at any nesting depth — is compared
+     /// element-order-insensitively by the comparator.
+     pub unordered_array_keys: HashSet<String>,


══════ F0700 │ src/round_trip/rules.rs:9-10 │ [maintainability · medium] ══════
[maintainability · medium] `ignored_paths` is a public configuration field that is silently dead:
the sole consumer of `OscalComparisonRules` (`src/round_trip/comparator.rs`) never reads
`rules.ignored_paths` (verified by search across `src/`). A caller who populates it believes those
JSON Pointer prefixes are skipped from comparison, but nothing honors the setting — the diff will
still report divergences under those paths, producing confidently wrong tool output. Either wire the
field into `compare_values` before recursing, or mark it `#[doc(hidden)]`/non-public until it is
actually implemented so the API doesn't advertise unimplemented behavior.

-     /// JSON Pointer prefixes to skip entirely (reserved for future use; empty by default).
+     /// JSON Pointer prefixes to skip entirely.
+     ///
+     /// WARNING: currently unused by the comparator; setting this has no effect.
+     #[doc(hidden)]
      pub ignored_paths: Vec<String>,


══════ F0716 │ src/sanitize.rs:6-10 │ [security · medium] ══════
[security · medium] The whitelist only covers C0 controls (0x00–0x1F) plus DEL, so C1 control
characters U+0080–U+009F — e.g. U+009B (single-byte CSI), U+009D (OSC introducer), U+0085 (NEL) —
pass through unchanged. Terminals that honor ECMA-48 8-bit/C1 codes treat a lone U+009B exactly like
ESC-[ ', so a sanitizer whose stated purpose is preventing terminal escape injection (SEC-5) can
still be bypassed. Invisible/bidi characters (U+200B–U+200F, U+202A–U+202E RLO/LRO, U+2066–U+2069)
also survive and allow log/output spoofing. Rust's `char::is_control()` already covers U+0000–U+001F
*and* U+007F–U+009F, so building on it removes the hand-rolled numeric predicate and closes the C1
gap in one step; optionally also drop format-category (Cf) characters for spoofing resistance.

-         .filter(|&c| {
-             let b = c as u32;
-             // Keep everything except control chars 0x00-0x1F and DEL 0x7F, but preserve tab (0x09) and newline (0x0A)
-             (b >= 0x20 && b != 0x7F) || b == 0x09 || b == 0x0A
-         })
+         .filter(|&c| !(c.is_control() && c != '\t' && c != '\n'))


══════ F0757 │ src/summary/format.rs:60-64 │ [bug · medium] ══════
[bug · medium] Double-rounding leaves a gap the doc claims to prevent: values whose 2dp-rounded secs
land in [59.95, 60.00) pass the `rounded >= 60.0` check (so they take the seconds branch) but are
then re-rounded by `{rounded:.1}`, producing "60.0s" — a seconds-style label showing a 60-second
magnitude. This is exactly the case the comment above says is avoided. The existing test
`format_elapsed_boundary_just_under_60s` masks this: it only asserts the result lacks 'm', while the
actual output is "60.0s". Select the bucket based on the value as it will finally render, e.g. treat
>=59.95 as the minute bucket.

+     if rounded >= 59.95 {
+         #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
+         let total = rounded.round() as u64;
+         format!("{}m {}s", total / 60, total % 60)
      } else if rounded >= 1.0 {
          format!("{rounded:.1}s")
      } else {
          format!("{rounded:.2}s")
      }
+     // And strengthen the test:
+     // assert_eq!(format_elapsed(Duration::from_secs_f64(59.994)), "1m 0s");


══════ F0719 │ src/summary/mod.rs:98-105 │ [bug · medium] ══════
[bug · medium] This function's contract says it counts 'root-level + all groups, recursively', but
once nesting exceeds the undocumented magic depth of 64 the traversal silently stops descending:
controls in deeper groups are permanently dropped from the total, yet the return value still looks
authoritative. In production (pipeline.rs Step 11b) this number feeds `controls_generated` and thus
`mapping_coverage()` on the user-facing dashboard, so a truncated count yields a confidently wrong
percentage. The cutoff also logs `tracing::warn!` once per offending call, which spams logs on
wide-and-deep inputs, and callers have no programmatic way to detect truncation. Since an
OscaalCatalog is an acyclic owned tree, an explicit-stack iterative traversal gives the exact total
for any depth, removing the magic number, the truncation hazard, and the recursion budget in one
change.

-     fn count_group_controls(groups: &[crate::oscal::catalog::OscalGroup], depth: usize) -> usize {
-         if depth > 64 {
-             tracing::warn!("OSCAL group nesting > 64; truncating recursive count");
-             return groups.iter().map(|g| g.controls.len()).sum();
+ use crate::oscal::catalog::OscalGroup;
+
+ #[must_use]
+ pub fn count_catalog_controls(catalog: &OscalCatalog) -> usize {
+     let mut total = catalog.controls.len();
+     let mut stack: Vec<&OscalGroup> = catalog.groups.iter().collect();
+     while let Some(group) = stack.pop() {
+         total += group.controls.len();
+         stack.extend(group.groups.iter());
          }
-         groups.iter().map(|g| g.controls.len() + count_group_controls(&g.groups, depth + 1)).sum()
+     total
      }
-     catalog.controls.len() + count_group_controls(&catalog.groups, 0)


══════ F0734 │ src/testing/semantic_eq.rs:132-135 │ [bug · medium] ══════
[bug · medium] Numeric equality relies on serde_json's derived PartialEq, which distinguishes
representations: Number(1) (PosInt) != Number(1.0) (Float). Any round-trip through a
serializer/toolchain that normalizes integers to floats (e.g., YAML/TOML mapping, some JSON writers
emitting "1.0") will report spurious 'value mismatch' failures, defeating the purpose of this
comparator. Note the PRD M-8 'type' distinction means JSON types (string vs number), not i64-vs-f64
storage. Consider special-casing Value::Number pairs to compare numerically (both as_f64/as_i64
agreement) while keeping cross-type mismatch detection.

- ) {
      if expected == actual {
+         return;
+     }
+     // Compare numbers numerically so 1 == 1.0 (representation-only difference).
+     if let (Value::Number(e), Value::Number(a)) = (expected, actual) {
+         if e.as_f64() == a.as_f64() {
          return;
+         }
      }


══════ F0723 │ src/trace/extractor.rs:27-28 │ [bug · medium] ══════
[bug · medium] `unwrap_or("")` coerces props whose `name`/`value` are missing or non-strings into
empty strings, which silently fabricates data: a prop such as `{"name": "source-section", "ns":
FORGE_TRACE_NS, "value": 42}` produces `source_section = Some("")`, so the element passes the
section gate and is reported as "mapped" with an empty section title (downstream excerpt lookup then
runs against `""`). Typed-mismatched props should be skipped, not defaulted, so they cannot turn
malformed input into a false-positive mapping.

-         let name = prop.get("name").and_then(|v| v.as_str()).unwrap_or("");
-         let value = prop.get("value").and_then(|v| v.as_str()).unwrap_or("");
+         // Skip props whose name/value are missing, non-string, or empty.
+         let (Some(name), Some(value)) = (
+             prop.get("name").and_then(|v| v.as_str()),
+             prop.get("value").and_then(|v| v.as_str()),
+         ) else {
+             continue;
+         };


══════ F0722 │ src/trace/extractor.rs:33-33 │ [bug · medium] ══════
[bug · medium] Unconditional assignment means a *later* malformed prop silently clobbers an earlier
valid value. For example, props containing two `source-line` entries -- `"42"` followed by
`"not-a-number"` -- end up with `source_line = 0` (reported as "unmapped") even though perfectly
valid trace data was present; likewise the last duplicate of any prop always wins, whether or not it
is well-formed. Prefer first-valid-wins (or at least valid-over-invalid) semantics so corrupt
duplicate data cannot degrade correct results.

-             PROP_SOURCE_LINE => source_line = value.parse::<usize>().ok(),
+             PROP_SOURCE_LINE => {
+                 // First valid value wins; a malformed later value must not clobber it.
+                 if let Ok(line) = value.parse::<usize>() {
+                     source_line = Some(line);
+                 }
+             }


══════ F0770 │ src/trace/formatter.rs:26-27 │ [bug · medium] ══════
[bug · medium] `strip_control_chars` deliberately preserves `\n` (0x0A) and `\t` (0x09). If
`meta.source_section` or `entry.element_id` contains either, those characters pass straight through
into a fixed-width table row: a newline injects additional physical lines mid-table (breaking the
one-line-per-entry invariant and the separator/footer positioning), and tabs distort column
alignment. Since ESC-sequence injection is handled in sanitize.rs, the remaining exposure here is
structural corruption of the report; this function is the last boundary before output, so it should
additionally normalize whitespace per cell, e.g. `section.replace(['\n', '\r', '\t'], " ")`, after
stripping control chars.

                  Some(meta) => {
-                     let section = strip_control_chars(&meta.source_section);
+                     let section =
+                         strip_control_chars(&meta.source_section).replace(['\r', '\n', '\t'], " ");


══════ F0751 │ src/trace/mod.rs:82-82 │ [other · medium] ══════
[other · medium] The two source-derived fields of the report come from different observations:
`source_line_count` is computed from content read earlier via `read_file`, while `source_stale`
re-stats the file's mtime inside `resolver::check_source_staleness` (a fresh `std::fs::metadata`
call). If the source is edited concurrently, the report can simultaneously claim an 'old' line count
and a staleness verdict computed against the new mtime (or vice versa), i.e. a check-then-act race
producing a self-contradictory report. Capture the mtime once next to the content read and pass that
snapshot to the staleness check so both fields describe the same revision of the file.

-     let source_stale = resolver::check_source_staleness(source_path, metadata_last_modified);
+     // Snapshot the mtime together with the content read so line count and
+     // staleness always refer to the same revision of the source.
+     let source_mtime = std::fs::metadata(source_path)
+         .and_then(|m| m.modified())
+         .ok();
+     let source_stale = resolver::check_source_staleness_with_mtime(
+         source_mtime,
+         metadata_last_modified,
+     );


══════ F0750 │ src/trace/mod.rs:99-103 │ [bug · medium] ══════
[bug · medium] Uneven error mapping between the size pre-check and the actual read:
`check_file_size` propagates I/O failures via `?` as a raw `ForgeError::Io`, so a `PermissionDenied`
(or any non-NotFound error) surfacing from the metadata() call returns the generic variant, while
the very same failure hitting `read_to_string` below returns the normalized
`ForgeError::PermissionDenied { path }`. Callers therefore observe different error types depending
on which syscall happened to fail. Additionally, this check-then-read structure leaves a TOCTOU
window: a file that grows past MAX_FILE_SIZE between metadata() and read_to_string is fully buffered
into memory, defeating the size guard. Normalize the pre-check errors the same way as the read (or
better, open the file once and clamp with `File::take(max + 1)` so the bound cannot be raced).

      match crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE) {
          Ok(_) => {}
          Err(ForgeError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
+         Err(ForgeError::Io(e)) => {
+             return Err(match e.kind() {
+                 std::io::ErrorKind::PermissionDenied => {
+                     ForgeError::PermissionDenied { path: path.to_path_buf() }
+                 }
+                 _ => ForgeError::Io(e),
+             });
+         }
          Err(e) => return Err(e),
      }
+     // Residual TOCTOU: consider opening once and reading via
+     // `file.take(crate::io::MAX_FILE_SIZE + 1)` to make the bound race-free.


══════ F0744 │ src/trace/report.rs:104-107 │ [bug · medium] ══════
[bug · medium] Snapshot consistency hazard: `TraceSummary::from_entries(&self.entries)` copies
`total_elements`/`mapped_elements` into a separate public field, but `entries` remains a publicly
mutable `Vec`. Any post-construction push/truncate/mutation of `entries` silently desynchronizes
`coverage_percent()`/`unmapped_elements()` from reality, and nothing marks `summary` stale. Since
this is the 'single source of truth' contract for the report (consumed by CLI output/JSON
serialization), derive the stats rather than storing them — either compute lazily or expose
`TraceReport::summary()` recomputed from `&self.entries`.

      /// All trace entries (one per walked OSCAL element).
      pub entries: Vec<TraceEntry>,
-     /// Computed summary statistics.
-     pub summary: TraceSummary,
+
+ impl TraceReport {
+     /// Summary statistics always derived from current `entries`.
+     #[must_use]
+     pub fn summary(&self) -> TraceSummary {
+         TraceSummary::from_entries(&self.entries)
+     }
+ }


══════ F0746 │ src/trace/report.rs:110-111 │ [other · medium] ══════
[other · medium] The doc claims validation of line refs ('for line reference validation'), yet no
accessor or invariant on this type enforces it — that logic lives elsewhere
(`resolver::validate_line_reference`, used only by the formatter) and callers can construct a
`TraceReport` whose mapped entries point past EOF with nothing flagged at the data-model level. As a
cross-field invariant this belongs near the data: add a checked constructor/validation pass over the
fully-assembled fields, e.g. return Result from a `TraceReport::new(...)` or provide
`lines_out_of_range() -> impl Iterator<Item = &TraceEntry>` so consumers can't forget the rule.
Additionally, the sibling convention used by the formatter (sentinel `usize::MAX` meaning
'deliberately suppress line ref') is entirely undocumented here, inviting future misuse.

-     /// Number of lines in the source policy file, for line reference validation.
+     /// Total lines in the source policy file (source files are 1-based, minimum 1
+     /// when present; 0 only if the source file was empty).
+     ///
+     /// Consumers validating trace references MUST use
+     /// [`crate::trace::resolver::validate_line_reference`] rather than raw
+     /// comparisons, so out-of-range or sentinel-suppressed references are handled uniformly.
      pub source_line_count: usize,


══════ F0745 │ src/trace/report.rs:207-207 │ [test · medium] ══════
[test · medium] Float comparison weakness: `assert!((x - 50.0).abs() < f64::EPSILON)` scales
tolerance with the magnitude of x. For 50.0/100.0 the tolerance (~7e-15 / ~1.4e-14) happens to pass,
but the same idiom applied later to non-representable percentages like 33.333… would silently demand
near-machine-precision agreement where an epsilon of ~1e-9 is intended; conversely documenting this
pattern invites copy-paste bugs. Use a fixed small tolerance, and since this function only ever
yields n/total*100 with exact binary quotients in these fixtures (1.0, 0.5, 0.0), plain `==`
assertions are equally valid here and stricter.

-         assert!((summary.coverage_percent() - 50.0).abs() < f64::EPSILON);
+         assert!((summary.coverage_percent() - 50.0).abs() < 1e-9);
+         // or, for these exactly representable fixtures:
+         assert_eq!(summary.coverage_percent(), 50.0);


══════ F0743 │ src/trace/report.rs:40-42 │ [other · medium] ══════
[other · medium] Design issue: `source_line: usize` uses the sentinel value 0 to mean 'no line
number' (e.g., groups), instead of `Option<usize>`. This lets invalid states be represented and
leaks protocol details to every consumer: formatter.rs must remember that `source_line == 0 &&
element_type == Group` means 'omit' while bare `source_line == 0` means 'invalid', and extractor.rs
manufactures the fake 0 via `unwrap_or(0)`. A real line number is never 0 (files are 1-based), so
any confusion silently corrupts reported references.

-     /// 1-based source line number (from `source-line` prop, parsed).
-     /// 0 means no line number (e.g., groups).
-     pub source_line: usize,
+     /// 1-based source line number parsed from the `source-line` prop.
+     /// `None` when the element carries no line prop (e.g., groups).
+     pub source_line: Option<usize>,
+     ...
+     // extractor.rs would become:
+     // source_line,
+     ...
+     // formatter.rs checks a single predicate instead of type-and-sentinel:
+     // match meta.source_line { Some(n) => ..., None => write!(f, "-")? }


══════ F0739 │ src/trace/resolver.rs:20-22 │ [maintainability · medium] ══════
[maintainability · medium] Every failure path in this function (missing `metadata_last_modified`,
unparseable RFC 3339 timestamp, unreadable/nonexistent source file, errored `modified()`) silently
collapses to `false` = 'fresh'. Callers cannot distinguish 'verified fresh' from 'could not
determine', so a deleted source file or corrupted OSCAL artifact looks identical to a healthy pair
and regeneration may be silently skipped. Consider returning a tri-state value (e.g.
`Staleness::Fresh | Staleness::Stale | Staleness::Unknown`, or `Option<bool>`/`Result<bool,
StalenessError>`) so callers can decide policy for indeterminate results — regenerating, warning, or
skipping.

+ pub enum Staleness {
+     /// Source mtime <= artifact `last-modified`; artifact is usable.
+     Fresh,
+     /// Source mtime > artifact `last-modified`; regeneration needed.
+     Stale,
+     /// A timestamp could not be determined or parsed; caller decides policy.
+     Unknown,
+ }
+
      let Ok(mtime) = std::fs::metadata(source_path).and_then(|m| m.modified()) else {
-         return false;
+     return Ok(Staleness::Unknown);
      };


══════ F0742 │ src/trace/resolver.rs:42-50 │ [test · medium] ══════
[test · medium] Both staleness tests exercise only fallback branches; the primary logic — parsing a
valid timestamp, reading real file metadata, and the comparison itself — is never asserted. There is
no test for the genuine stale outcome (source newer than the timestamp), the fresh outcome, nor
non-UTC timezone-offset input (e.g. `+05:30`) through `parse_from_rfc3339`/`to_utc`. Add cases
anchored on a known file (or a temp file created under `std::env::temp_dir()`); using an ancient and
a far-future timestamp avoids dependence on wall-clock time.

+     #[test]
+     fn staleness_detects_newer_source_and_fresh_case() {
+         let src = Path::new("Cargo.toml");
+         // Cargo runs tests with CWD at the package root, so the file exists.
+         assert!(check_source_staleness(src, Some("2000-01-01T00:00:00Z")));
+         assert!(!check_source_staleness(src, Some("2999-12-31T23:59:59Z")));
+     }
+
+     #[test]
+     fn staleness_parses_non_utc_offsets() {
+         // 2020-01-01T05:30:00+05:30 == 2020-01-01T00:00:00Z
+         assert!(check_source_staleness(
+             Path::new("Cargo.toml"),
+             Some("2020-01-01T05:30:00+05:30")
+         ));
+     }
+
      #[test]
      fn staleness_missing_last_modified_returns_false() {
          assert!(!check_source_staleness(Path::new("nonexistent.md"), None));
      }

      #[test]
      fn staleness_unparseable_timestamp_returns_false() {
          assert!(!check_source_staleness(Path::new("nonexistent.md"), Some("not-a-date")));
      }


══════ F0765 │ src/trace/walker.rs:167-171 │ [bug · medium] ══════
[bug · medium] Fallback sentinel IDs for elements missing `id` (or `control-id`) are shared
constants: multiple malformed elements each produce a `TraceEntry` with the same `element_id` (and
same `ElementType`), so distinct elements become indistinguishable/aliased in the report, and
unique-per-entry keys downstream can silently collide or overwrite. At minimum, make the fallback
unique (e.g., `format!("unknown-group-{index}")` using the iteration index, or include a
JSON-pointer path) so each entry is identifiable. Relatedly, the `unwrap_or("unknown-requirement")`
fallback here logs no warning, unlike the group/control fallbacks — add a `tracing::warn!` for
consistency.

              let control_id = req
                  .get("control-id")
                  .and_then(|v| v.as_str())
-                 .unwrap_or("unknown-requirement")
-                 .to_string();
+                 .map_or_else(
+                     || {
+                         tracing::warn!("implemented-requirement is missing required 'control-id' field");
+                         "unknown-requirement".to_string()
+                     },
+                     str::to_string,
+                 );


══════ F0767 │ src/trace/walker.rs:173-177 │ [bug · medium] ══════
[bug · medium] Implemented-requirement entries discard their parent context: the
`TraceEntry.element_id` is only the `control-id`, and neither the component/capability UUID nor the
`control-implementations` UUID (whose `source` field selects which framework the requirement
implements) is recorded. Different components routinely implement the same control-id, and a compdef
may carry multiple control-implementations for different frameworks, so entries for the same
control-id become ambiguous during trace matching. Include the parent component/capability id (and
implementation `source`/uuid) in the entry or its trace metadata.

+             // Preserve parent context so same control-id across components/
+             // implementations stays distinguishable during trace matching.
+             let parent_id = container
+                 .get("uuid")
+                 .and_then(|v| v.as_str())
+                 .unwrap_or("unknown-container");
              entries.push(TraceEntry {
-                 element_id: control_id,
+                 element_id: format!("{parent_id}/{control_id}"),
                  element_type: ElementType::ImplementedRequirement,
                  trace,
              });


══════ F0764 │ src/trace/walker.rs:19-24 │ [maintainability · medium] ══════
[maintainability · medium] The `map_err(|_| ...)` discards the underlying `ValidateError` entirely.
`detect_model_type` can fail for three distinct reasons (missing key → `UnknownModelType`, multiple
recognized keys → `AmbiguousArtifact`, plus non-object values handled below), yet all are collapsed
into one fixed message that only mentions 'catalog'/'component-definition'. A caller whose artifact
actually has both keys gets a misleading message, and the ambiguity detail produced upstream
(`found.join(", ")`) is thrown away. Preserve the original error (e.g., include its `Display` in
`detail`) or match on the variants to emit an accurate message.

-     let model_type =
-         validate::detect_model_type(json).map_err(|_| ForgeError::TraceUnsupportedArtifact {
-             detail:
-                 "Expected top-level key 'catalog' or 'component-definition' with an object value"
-                     .to_string(),
+     let model_type = validate::detect_model_type(json).map_err(|e| {
+         ForgeError::TraceUnsupportedArtifact {
+             detail: format!("{e}: expected top-level key 'catalog' or 'component-definition' with an object value"),
+         }
          })?;


══════ F0762 │ src/types.rs:23-34 │ [maintainability · medium] ══════
[maintainability · medium] These values are documented as OSCAL-standard string keys that drive
detection/routing, yet they are duplicated as raw literals across consumers instead of being routed
through this single source of truth: `src/lifecycle/mod.rs:576-578` re-maps `OscalModelType` to
"component-definition"/"mapping-collection" by hand, and `src/cli/config_check.rs` / `src/config.rs`
keep their own "component-definition" tables. With no co-located parse direction
(`FromStr`/`TryFrom<&str>`) there is no compile-time tie between `as_str` and its callers, so a
renamed key or a new variant is easy to miss. Add a parser plus an `ALL` list next to the
declarations and route consumers (and round-trip tests) through them.

  impl OscalModelType {
+     /// All model types, for exhaustive iteration and round-trip tests.
+     pub const ALL: [Self; 4] = [
+         Self::Catalog,
+         Self::ComponentDefinition,
+         Self::Profile,
+         Self::Mapping,
+     ];
+
      /// OSCAL-standard string key for this model type.
      #[must_use]
-     pub fn as_str(&self) -> &'static str {
+     pub const fn as_str(&self) -> &'static str {
          match self {
              Self::Catalog => "catalog",
              Self::ComponentDefinition => "component-definition",
              Self::Profile => "profile",
              Self::Mapping => "mapping-collection",
+         }
+     }
          }
+
+ impl std::str::FromStr for OscalModelType {
+     type Err = String;
+
+     fn from_str(s: &str) -> Result<Self, Self::Err> {
+         Self::ALL
+             .iter()
+             .find(|model| model.as_str() == s)
+             .copied()
+             .ok_or_else(|| format!("unknown OSCAL model type: '{s}'"))
      }
  }


══════ F0761 │ src/types.rs:48-53 │ [maintainability · medium] ══════
[maintainability · medium] `Strategy` and `OutputType` duplicate the same domain decision (catalog
vs component definition) with disjoint variant sets (`OutputType::Ssp` has no `Strategy`
counterpart) and divergent labels ('component' vs 'component-definition'). The only linkage lives
outside this file in `src/cli/convert.rs::effective_strategy` (match over `OutputType`) and
`src/config.rs:518-520`, so when either enum gains a variant the mapping can silently diverge: a new
`OutputType` forces an update there only because the match is exhaustive, but a new `Strategy`
variant compiles fine and falls through `Some(Ssp) | None => *opts.strategy` unreviewed. Prefer one
enum (drop `Strategy` in favor of `OutputType`), or co-locate the relationship here with an explicit
conversion and a round-trip test.

- pub enum Strategy {
-     /// Produce an OSCAL Catalog (groups → controls → statements).
-     Catalog,
-     /// Produce an OSCAL Component Definition (implemented requirements).
-     Component,
+ impl Strategy {
+     /// Output artifact produced by this strategy.
+     #[must_use]
+     pub fn output_type(self) -> OutputType {
+         match self {
+             Self::Catalog => OutputType::Catalog,
+             Self::Component => OutputType::ComponentDefinition,
+         }
+     }
+ }
+
+ #[cfg(test)]
+ mod tests {
+     #[test]
+     fn strategy_maps_to_output_type() {
+         assert_eq!(Strategy::Catalog.output_type(), OutputType::Catalog);
+         assert_eq!(Strategy::Component.output_type(), OutputType::ComponentDefinition);
+     }
  }


══════ F0775 │ src/uuid.rs:271-278 │ [bug · medium] ══════
[bug · medium] When the depth cap trips, traversal stops silently: all requirements in the skipped
subtree keep stable_id = None, directly violating the public contract on assign_stable_ids ('After
this function returns, no PolicyRequirement in the document will have stable_id = None'). The breach
is logged only at trace level (effectively invisible in production configs) and the function cannot
report the degraded result to the caller — any downstream code assuming Some() may misbehave. Also
note the off-by-one: recursion runs while depth <= 50, so sections at depth 51 are still processed
and only their children are cut, meaning the effective cap is 52 levels despite MAX_SECTION_DEPTH =
50. Log at warn/error, compare with >= for the named cap, and ideally signal truncation (return the
count of skipped requirements or a Result) so callers can enforce the postcondition.

-     if depth > MAX_SECTION_DEPTH {
-         tracing::trace!(
-             depth,
-             max = MAX_SECTION_DEPTH,
-             "max section depth exceeded; skipping child traversal"
-         );
+     if depth >= MAX_SECTION_DEPTH {
+         tracing::warn!(depth, max = MAX_SECTION_DEPTH, "max section depth reached; skipping descendant traversal");
+         // Surface the degradation instead of failing silently:
+         // return/report the number of unvisited requirements so callers can
+         // verify the "every requirement got a stable_id" postcondition.
          return;
      }


══════ F0780 │ src/validate/error_types.rs:41-44 │ [security · medium] ══════
[security · medium] The SEC-1 contract ("actual MUST be truncated to 100 chars") is only documented,
not enforced by the type. Because `ValidationError`'s fields are public and there is no constructor
in this module, any construction site can embed arbitrarily long `actual` payloads directly,
bypassing `formatter::truncate_value()` — leaking oversized or potentially sensitive instance
content into serialized reports. Enforce the cap where the value enters the type, e.g. a checked
constructor, or make fields private and expose accessors.

      /// What was actually found (e.g., "field not present").
-     /// Truncated to 100 characters with "..." suffix (SEC-1).
-     pub actual: String,
+     /// Guaranteed <= 103 characters (100 + "...") (SEC-1).
+     actual: String,
+ }
+
+ impl ValidationError {
+     /// Build an error, enforcing the SEC-1 cap on `actual` at the type boundary.
+     #[must_use]
+     pub fn new(
+         category: ValidationErrorCategory,
+         path: String,
+         message: String,
+         expected: String,
+         actual: impl std::fmt::Display,
+     ) -> Self {
+         Self {
+             category,
+             path,
+             message,
+             expected,
+             actual: crate::validate::formatter::truncate_value(&actual.to_string(), 100),
+         }
+     }
+     pub fn actual(&self) -> &str { &self.actual }
  }


══════ F0779 │ src/validate/error_types.rs:83-91 │ [bug · medium] ══════
[bug · medium] The legacy-report fallback infers `supported_input` from `is_valid`: any old report
with zero errors is silently loaded as a *supported* input. Historically `is_valid ==
errors.is_empty()` said nothing about whether the declared OSCAL version was accepted by the
compatibility policy — a document that passed schema/semantic checks under an unsupported version
was `is_valid: true` yet unsupported. After this fallback runs, `supported_input()` returns true for
such reports, so downstream gating/auditing that trusts this flag (see src/cli/validate.rs,
src/validate/report.rs output) misreports provenance. Prefer defaulting to the safe value (`false`)
when `supported_input` is absent, or recompute support from `declared_oscal_version` against the
current policy instead of borrowing an unrelated boolean.

-             // Legacy reports had `is_valid` but no `supported_input`. It is
-             // used only as a compatibility default; derived validity is still
-             // recomputed from `errors` below.
-             #[serde(default)]
-             is_valid: Option<bool>,
-             errors: Vec<ValidationError>,
-         }
-         let raw = Raw::deserialize(deserializer)?;
-         let supported_input = raw.supported_input.or(raw.is_valid).unwrap_or(false);
+             // Legacy reports predate `supported_input`. Do NOT reuse `is_valid`
+             // as a proxy: validity never encoded version compatibility.
+             // Either fall back conservatively or re-evaluate the recorded
+             // declared version against the current policy:
+             let supported_input = raw.supported_input.unwrap_or_else(|| {
+                 raw.declared_oscal_version.as_deref()
+                     .map(|v| crate::validate::version::is_supported_declaration(v))
+                     .unwrap_or(false)
+             });


══════ F0792 │ src/validate/formatter.rs:46-53 │ [bug · medium] ══════
[bug · medium] Purely numeric object keys are indistinguishable from array indices once the pointer
is stringified, so the emitted JSON Path lies about the document structure: property `"123"` under
root renders as `$[123]`, sending users to a nonexistent array position instead of `$.cards['123']`.
Likewise, keys whose unescaped form contains `.` or other notation metacharacters (including `/`
restored from `~1`) are appended raw after `.`, so key `a.b` becomes the ambiguous `$.a.b`.
Bracket-quote any segment that is not a strict identifier (`$['a.b']`) unless you can prove from the
instance tree that it is a genuine array index.

-         if unescaped.chars().all(|c| c.is_ascii_digit()) && !unescaped.is_empty() {
-             result.push('[');
-             result.push_str(&unescaped);
-             result.push(']');
-         } else {
+         let is_identifier = !unescaped.is_empty()
+             && !unescaped.starts_with(|c: char| c.is_ascii_digit())
+             && !unescaped
+                 .chars()
+                 .any(|c| matches!(c, '.' | '[' | ']' | '\'' | '"' | '/'));
+         if is_identifier {
              result.push('.');
              result.push_str(&unescaped);
+         } else {
+             result.push_str("['");
+             result.push_str(&unescaped.replace('\'', "\\'"));
+             result.push_str("']");
          }


══════ F0793 │ src/validate/formatter.rs:64-67 │ [security · medium] ══════
[security · medium] SEC-1 caps the length of the echoed value but not its sensitivity: the `actual`
field copies verbatim instance content into user-facing diagnostics. Fields named
password/token/secret/api_key/session_id that fail a constraint (or any PII element) will surface
their real content up to 100 characters, and a root-level failure (e.g. missing top-level `required`
field) serializes the *entire document* and forwards its first 100 characters. The existing tests
all use tiny fake payloads so they never exercise this. Add redaction based on well-known
secret-bearing key names, or gate raw-value echoes behind an opt-in debug configuration.

+ const REDACTED: &str = "[redacted]";
+
+ fn looks_sensitive(value: &Value) -> bool {
+     value.as_str().is_some_and(|_| false) // hook for caller-supplied policy
+ }
+
  fn extract_actual_value(json: &Value, pointer: &str) -> String {
      if pointer.is_empty() {
-         return truncate_value(&json.to_string(), 100);
+         return "<document summary withheld>".to_string();
+     }
+     // ...
      }


══════ F0802 │ src/validate/mod.rs:138-148 │ [bug · medium] ══════
[bug · medium] Presence-only detection treats roots holding JSON null (or any non-object) as
recognized: `{"catalog": null}` is classified as `Catalog` instead of being rejected as
UnknownModelType, so reports/paths are built on a bogus model label. Each key is also probed twice
(ambiguity pass, then this selection pass), which invites drift. Probe once requiring
`Some(Value::Object(_))` and select directly from the collected hits.

-     if json.get("catalog").is_some() {
-         Ok(OscalModelType::Catalog)
-     } else if json.get("component-definition").is_some() {
-         Ok(OscalModelType::ComponentDefinition)
-     } else if json.get("profile").is_some() {
-         Ok(OscalModelType::Profile)
-     } else if json.get("mapping-collection").is_some() {
-         Ok(OscalModelType::Mapping)
-     } else {
-         Err(ValidateError::UnknownModelType)
+     let mut found: Vec<&str> = Vec::new();
+     if matches!(json.get("catalog"), Some(Value::Object(_))) {
+         found.push("catalog");
+     }
+     if matches!(json.get("component-definition"), Some(Value::Object(_))) {
+         found.push("component-definition");
+     }
+     if matches!(json.get("profile"), Some(Value::Object(_))) {
+         found.push("profile");
+     }
+     if matches!(json.get("mapping-collection"), Some(Value::Object(_))) {
+         found.push("mapping-collection");
+     }
+     if found.len() > 1 {
+         return Err(ValidateError::AmbiguousArtifact { detail: found.join(", ") });
+     }
+
+     // Select directly from the single collected hit (no second membership scan).
+     match found.first().copied() {
+         Some("catalog") => Ok(OscalModelType::Catalog),
+         Some("component-definition") => Ok(OscalModelType::ComponentDefinition),
+         Some("profile") => Ok(OscalModelType::Profile),
+         Some("mapping-collection") => Ok(OscalModelType::Mapping),
+         _ => Err(ValidateError::UnknownModelType),
      }


══════ F0803 │ src/validate/mod.rs:202-205 │ [maintainability · medium] ══════
[maintainability · medium] Compilation failures are flattened to `String` before entering the cache,
permanently discarding the typed error and its source chain (parse failure vs. schema-build failure
become indistinguishable), and the failure stays cached process-wide with no way to recover richer
diagnostics. Cache the structured failure alongside the preformatted message so the deterministic
cached text is preserved while `#[source]`-style context remains available.

+ // Structured negative-cache slot: stable rendered message + retainable cause.
+ type CachedCompile = Result<jsonschema::Validator, (String, Box<dyn std::error::Error + Send + Sync>)>;
+
      let result = cell.get_or_init(|| {
-         let schema = load_schema(model_type).map_err(|error| error.to_string())?;
-         jsonschema::validator_for(&schema).map_err(|error| error.to_string())
+     let schema = load_schema(model_type)
+         .map_err(|error| (error.to_string(), Box::new(error) as _))?;
+     jsonschema::validator_for(&schema)
+         .map_err(|error| (error.to_string(), Box::new(error) as _))
      });


══════ F0804 │ src/validate/mod.rs:271-273 │ [security · medium] ══════
[security · medium] SEC-3 gate is advisory against the bytes actually validated: `metadata.len()` is
sampled at check time while the file is read later by other code, so a file swapped/appended between
the check and the read (TOCTOU) bypasses the 50MB cap. `std::fs::metadata` also silently follows
symlinks, letting a link point past the policy unless that is intended. Bind the decision to the
read itself (e.g. a bounded `take(MAX + 1)` reader) or re-verify after reading.

      let metadata = std::fs::metadata(path)
          .map_err(|e| ValidateError::FileRead { path: path.to_path_buf(), source: e })?;
      let size = metadata.len();
+     // Plus, bound the actual read elsewhere:
+     // let file = std::fs::File::open(path)?;
+     // let mut limited = file.take(MAX_VALIDATE_FILE_SIZE + 1);
+     // ... err if `limited.limit() == 0` (stream reached MAX+1 => oversized)


══════ F0805 │ src/validate/mod.rs:301-307 │ [maintainability · medium] ══════
[maintainability · medium] `run_full_validation` re-implements the error pipeline that
`validate_artifact` owns (iter_errors collection + injecting the oscal-version violation), and the
copies have already diverged in one visible way: this path emits the version error with JSONPath
notation (`$.catalog.metadata.oscal-version`, asserted in tests) while `validate_artifact`
hand-builds a JSON Pointer (`/catalog/metadata/oscal-version`). Extract the shared collection +
version-injection step into one helper so a fix lands in both entry points.

-     let validator = compiled_validator(model_type)?;
-
-     // Schema validation: collect all raw errors and format them
-     let mut schema_errors: Vec<error_types::ValidationError> = validator
-         .iter_errors(json)
-         .map(|error| formatter::format_schema_error(&error, json))
-         .collect();
+     // Shared implementation consumed by both validate_artifact and run_full_validation
+     // (same traversal, same canonical path notation for the injected oscal-version error).
+     let mut schema_errors = collect_schema_and_version_errors(&validator, json, model_type);


══════ F0797 │ src/validate/report.rs:105-112 │ [maintainability · medium] ══════
[maintainability · medium] The SEC-3 fallback duplicates the entire `ValidationReport` schema as a
hand-written JSON literal. The struct in error_types.rs is the single source of truth; if a field is
renamed, added (with `skip_serializing_if`) or reordered there, this string silently diverges from
the PRD S-1 output shape, and since `to_string_pretty` over owned `String`/`Vec` fields is
practically infallible, no test will ever exercise the drift. Build the fallback by serializing a
synthesized `ValidationReport` instead of embedding raw JSON.

      serde_json::to_string_pretty(report).unwrap_or_else(|e| {
          tracing::error!(error = %e, "ValidationReport serialization failed; returning fallback JSON structure");
-         // SEC-3: fallback must conform to ValidationReport schema (no extra fields).
-         format!(
-             r#"{{"artifact_path":"","model_type":"unknown","declared_oscal_version":null,"schema_version_used":"{}","supported_input":false,"is_valid":false,"errors":[],"schema_error_count":0,"semantic_error_count":0}}"#,
-             crate::validate::version::SCHEMA_VERSION_USED
-         )
+         // SEC-3: derive the fallback from the type itself so the emitted key
+         // set can never drift from `ValidationReport`.
+         let fallback = ValidationReport::new(String::new(), Vec::new());
+         serde_json::to_string(&fallback).unwrap_or_default()
      })


══════ F0800 │ src/validate/report.rs:52-52 │ [bug · medium] ══════
[bug · medium] An invalid report can contain zero categorized errors (e.g.
`ValidationReport::new_with_context(.., supported_input = false, vec![])` for an unsupported-input
rejection), yet this branch still emits the "Validation failed:" header followed by a bare
section-less report. The summary is also unconditionally printed even when both counters are 0.
Guard the header (and consider an explicit unsupported-input note) so failure output always explains
*why* it failed.

+     if parts.is_empty() {
+         // No categorized errors — surface the unsupported-input signal so
+         // the text report still explains the failure.
+         parts.push(if report.supported_input() {
+             "unknown validation failure".to_string()
+         } else {
+             "unsupported input".to_string()
+         });
+     }
      let _ = writeln!(output, "Validation failed: {}", parts.join(", "));


══════ F0787 │ src/validate/semantic.rs:88-91 │ [maintainability · medium] ══════
[maintainability · medium] When MAX_WALK_DEPTH is hit, traversal silently stops and only a
tracing::trace is emitted. For a validation tool this makes the result indistinguishable from
'validated successfully': malformed or orphaned links in deeply nested documents pass unnoticed,
weakening the data-integrity guarantee (note serde_json's default 128-level recursion limit still
lets documents exceed this 100 threshold). At minimum record a ValidationError/category warning (or
return a 'validation incomplete' signal) so callers know part of the document was not examined.


══════ F0786 │ src/validate/semantic.rs:98-99 │ [bug · medium] ══════
[bug · medium] UUID equality is done with raw, case-sensitive string lookup against the HashSet. Per
RFC 4122 the hexadecimal digits of a UUID may appear in either case, and OSCAL does not mandate a
single canonical casing between a link fragment and the referenced resource's `uuid`, so `#ABC…` in
an href and `abc…` in back-matter will produce a false positive (and vice-versa form issues).
Normalize both sides (e.g. `to_ascii_lowercase()`) before insertion/lookup. Two related edge cases
in the same expression: (a) a bare `"#"` yields an empty uuid and gets a confusing "orphaned link:
reference # not found" error — either treat empty fragments separately or improve the message; (b)
compound fragments such as `#uuid/extra` never match, so confirm whether sub-path fragments should
be supported.

-                 && let Some(uuid) = href_str.strip_prefix('#')
-                 && !resource_uuids.contains(uuid)
+                 && let Some(mut uuid) = href_str.strip_prefix('#').filter(|u| !u.is_empty())
+                     .map(|u| u.to_ascii_lowercase())
+                 && !resource_uuids.contains(&uuid)


══════ F0812 │ src/validate/version.rs:125-127 │ [bug · medium] ══════
[bug · medium] Sanitization order breaks the crate's SEC-1 length bound. The input is capped at 100
chars *before* escaping, but `char::escape_default` expands non-printables to up to 10 chars (e.g.
`\u{e9}`), so the resulting `actual`/`message` payload can approach ~1000 characters.
`ValidationError` is documented in `error_types.rs` as "Truncated to 100 characters with \"...\"
suffix (SEC-1)", and every other producer enforces this via `formatter::truncate_value(&value, 100)`
— so values emitted here can be ~10x larger than every other error in the same report, diverging in
size guarantees for logs/consumers. Escape first, then bound the final string (or reuse
`formatter::truncate_value`) so the invariant holds after transformation.

  fn escape_for_diagnostic(value: &str) -> String {
-     value.chars().take(100).flat_map(char::escape_default).collect()
+     // Escape first, then enforce the SEC-1 bound *after* expansion, matching
+     // the crate-wide contract enforced by formatter::truncate_value.
+     super::formatter::truncate_value(
+         &value.chars().flat_map(char::escape_default).collect::<String>(),
+         100,
+     )
  }


══════ F0811 │ src/validate/version.rs:82-93 │ [bug · medium] ══════
[bug · medium] `declared` violates its documented contract. The doc says "Exact string declaration
when the field is a string", but for unsupported declarations this stores the escaped/truncated
diagnostic rendering (`safe_declared`) instead of the raw `declared` value. Downstream consumers
surface this verbatim: `ValidationResult.declared_oscal_version` and
`ValidationReport.declared_oscal_version` serialize it into JSON reports, and `cli/validate.rs`
prints it as `declared_oscal_version` in round-trip summaries. So a document declaring e.g. "1.2.4
'" or a >100-char string yields a mutated/mangled version string in machine-readable output, which
can break diffing against the source artifact and misleads anyone correlating reports with inputs.
Keep `declared: Some(declared.to_string())` (the exact declaration) and reserve sanitization purely
for the error fields (message/actual).

          let safe_declared = escape_for_diagnostic(declared);
          return VersionInspection {
-             declared: Some(safe_declared.clone()),
+             // Preserve the exact declaration for reporting (see doc on `VersionInspection::declared`).
+             declared: Some(declared.to_string()),
              supported: false,
              error: Some(version_error(
                  path,
                  format!(
                      "unsupported OSCAL version declaration '{safe_declared}'; available schema baseline is {SCHEMA_VERSION_USED}"
                  ),
                  safe_declared,
              )),
          };


══════ F1018 │ supply-chain/audits.toml:2-4 │ [maintainability · medium] ══════
[maintainability · medium] This file ships only as the cargo-vet default scaffold (header comment +
empty table), yet the exemption surface in the sibling config.toml keeps growing, implying no
process ever moves crates out of exemptions into audited/imported entries. As-is, an empty
audits.toml gives false assurance to reviewers reading the supply-chain directory: the 'audit'
artifact exists but records nothing. Consider either (a) converting exemptions into imported-audit
entries here per the follow-up mandated by docs/AR/051-ar-toml-dependency.md, or (b) adding a short
comment stating the intended cadence (e.g., run `cargo vet suggest` / import upstream audits each
release cycle) so the placeholder's purpose and ownership are explicit.


══════ F1047 │ supply-chain/config.toml:175-181 │ [maintainability · medium] ══════
[maintainability · medium] Seven crates appear twice under different versions (cpufeatures
0.2.17/0.3.0, foldhash 0.1.5/0.2.0, getrandom 0.3.4/0.4.2, hashbrown 0.15.5/0.16.1, r-efi
5.3.0/6.0.0, rand 0.9.4/0.10.2 + rand_core 0.9.5/0.10.1, windows-sys 0.59.0/0.61.2, winnow
0.7.15/1.0.4). Such pairs are legitimate only when genuinely semver-incompatible copies coexist in
Cargo.lock; any leftover entry whose version is no longer resolved dead-locks an old, unaudited
snapshot into the allow-list forever, because `cargo vet prune` is the only thing that removes it
and it only acts on versions present in the current graph. Please regenerate this file via `cargo
vet` (not by hand) after each dependency update and verify `cargo vet prune` removes stale halves;
the cpufeatures pair (a leaf crate that rarely needs two majors) is the most suspicious.


══════ F1050 │ supply-chain/config.toml:4-5 │ [security · medium] ══════
[security · medium] This file is the entire trust story of the workspace: there is no [[policy]]
section naming first-party crates (they default to unverifiable), no [imports] from third-party
auditor registries, hence roughly 280 exemptions carry 100% of the supply-chain assurance. Two
hardening steps are cheap: add [[policy]] entries asserting criteria for in-workspace crates so
regressions there are caught too, and wire up imports.lock from public audit streams so future
dependency bumps can ride existing audits instead of growing this exemption list. Also gate CI on
`cargo vet check --locked` so rows cannot be hand-edited.


══════ F0810 │ tests/atomize_integration.rs:113-115 │ [test · medium] ══════
[test · medium] SEC-9 requires an atomic input to be preserved byte-for-byte, which implicitly means
exactly one output requirement, yet the test indexes `result.requirements[0]` without first checking
the vector length. An atomizer that spuriously splits the atomic sentence into multiple pieces would
emit the preserved original first (and drop/regress only later positions), satisfying this test
while violating the guarantee. Assert `len() == 1` before indexing so the cardinality part of the
contract is actually enforced.

      let result = atomize_requirement(&req).unwrap();

+     assert_eq!(result.requirements.len(), 1, "atomic input must yield exactly one output");
      assert_eq!(result.requirements[0].text, original_text);


══════ F0808 │ tests/atomize_integration.rs:131-142 │ [test · medium] ══════
[test · medium] Only the format (length/hex digits) of each stable_id is checked in isolation; the
test never compares IDs against each other. A regression where the hash ignores the text (constant
output) or collides between distinct requirements would pass every assertion here, defeating the
primary purpose of a unique identifier. Insert each ID into a HashSet and fail on repetition (e.g.
`assert!(seen.insert(id))`), and consider also covering two byte-identical requirement texts — the
scenario where cross-document/section ID distinctness matters most.

      let result = atomize_document(&doc).unwrap();
+     let mut seen = std::collections::HashSet::new();
      for section in &result.sections {
          for req in &section.requirements {
              let id = req.stable_id.as_deref().expect("stable_id should be set after atomization");
              assert_eq!(id.len(), 64, "ID length mismatch for: {}", req.text);
              assert!(
                  id.chars().all(|c| c.is_ascii_hexdigit()),
                  "Non-hex char in ID for: {}",
                  req.text
              );
+             assert!(!seen.contains(id), "Duplicate stable_id {} for: {}", id, req.text);
+             seen.insert(id.to_string());
          }
      }


══════ F0809 │ tests/atomize_integration.rs:53-61 │ [test · medium] ══════
[test · medium] The oracle covers only structural metadata: fragment count, index continuity, source
line, and parent-text presence. The actual fragment `text` is never inspected, so a splitter that
emits wrong substrings, reorders clauses, mangles whitespace/punctuation, or produces
empty/duplicate fragments still passes whenever the count stays aligned. Capture the parent sentence
up front and strengthen the loop: assert each fragment text is non-empty, mutually distinct, and
that `parent_text` links back to exactly that captured sentence.

+     let parent = doc.sections[0].requirements[0].text.clone();
+
      let result = atomize_document(&doc).unwrap();
      let reqs = &result.sections[0].requirements;

      assert_eq!(reqs.len(), 3);
+     let mut texts = std::collections::HashSet::new();
      for (i, req) in reqs.iter().enumerate() {
          assert_eq!(req.atom_index, i);
          assert_eq!(req.source_line, 42);
-         assert!(req.parent_text.is_some());
+         assert!(!req.text.is_empty(), "fragment {} text is empty", i);
+         assert!(texts.insert(&req.text), "duplicated fragment text: {}", req.text);
+         assert_eq!(req.parent_text.as_deref(), Some(parent.as_str()));
      }


══════ F0816 │ tests/cli_integration.rs:938-939 │ [test · medium] ══════
[test · medium] This test feeds a NONEXISTENT file ('any.json') yet pins exit code 3 ('validation
error'). The production mapping sends it there only because `cli::validate` rewraps the read failure
as `ForgeError::Validation` — so the same user mistake (missing input path) exits 1 for `forge
convert` (`ForgeError::FileNotFound`) but 3 for `forge validate`. If that asymmetry is intended CLI
contract, document the rationale here so nobody 'fixes' either suite independently; if it is
accidental classifier leakage, fix the production mapping instead of cementing it in an integration
assertion. Either way a clarifying comment (or an accompanying true-invalid-document exit-code test)
is needed to distinguish 'not found' from 'structurally invalid OSCAL'.

+     // NOTE: `validate` intentionally maps unreadable/missing inputs to
+     // ForgeError::Validation (exit 3), unlike `convert` where the same missing-file
+     // scenario is ForgeError::FileNotFound (exit 1). Revisit BOTH exit-code suites
+     // together if the taxonomy in src/error.rs changes.
      assert!(!output.status.success(), "Expected non-zero exit code");
      assert_eq!(output.status.code(), Some(3), "Validation error should exit with code 3");


══════ F0824 │ tests/common/fixture_generator.rs:1294-1294 │ [test · medium] ══════
[test · medium] The module contract is byte-identical determinism plus specific corpus
characteristics (~150KB / ~25k words, 10 tables, 40 subsections, 200 requirements), but nothing in
this file enforces any of it. Silent fixture drift (an edited body string, a dropped table) would
invalidate historical benchmark comparisons with no signal. Add a #[cfg(test)] module that (a)
asserts generate_synthetic_policy() produces equal output across two calls, (b) asserts size falls
within an accepted range (e.g. 140_000..250_000 bytes, which also keeps the with_capacity(200_000)
honest), and (c) counts structural elements (H2/H3/Table/H4 markers) against expected constants.

- pub fn generate_synthetic_policy() -> String {
+ #[cfg(test)]
+ mod tests {
+     use super::*;
+
+     #[test]
+     fn output_is_deterministic() {
+         assert_eq!(generate_synthetic_policy(), generate_synthetic_policy());
+     }
+
+     #[test]
+     fn corpus_metrics_stable() {
+         let doc = generate_synthetic_policy();
+         assert!((140_000..250_000).contains(&doc.len()));
+         assert_eq!(doc.matches("\n## ").count(), DOMAINS.len());
+         assert_eq!(doc.matches("| --- |").count(), 10);
+     }
+ }


══════ F0820 │ tests/common/fixture_generator.rs:212-212 │ [bug · medium] ══════
[bug · medium] The generator emits tables as bare pipe-blocks with no 'Table N' captions anywhere in
the document, yet this requirement text says "See Table 2". Nothing in the generated output binds
the classification matrix to the identifier "Table 2", so any downstream
cross-reference/citation-resolution benchmark will score this as a dangling reference purely because
of how the fixture is written. Emit an explicit caption (e.g. "Table 2: Classification Matrix")
alongside the table using a running counter, or interpolate the table number programmatically so
text and emission order stay in lockstep.

-                     "The data governance committee shall maintain the authoritative classification guide and shall publish updates within fourteen business days of any regulatory change. See Table 2 for the classification matrix.",
+ if let Some(table) = sub.table {
+     table_num += 1;
+     let _ = writeln!(doc, "**Table {}**\n", table_num);
+     doc.push_str(table);
+     doc.push_str("\n\n");
+ }


══════ F0821 │ tests/common/fixture_generator.rs:289-289 │ [bug · medium] ══════
[bug · medium] "See Appendix A for approved disposal methods." points to an appendix that the
generator never produces — there is no appendix emission path at all, so this reference is dangling
unconditionally, independent of content ordering. Cross-reference-resolution checks over this corpus
will permanently classify it as broken. Either emit the corresponding Appendix A/B sections at the
end of the document, or remove/soften these inline pointers.


══════ F0822 │ tests/common/fixture_generator.rs:525-525 │ [bug · medium] ══════
[bug · medium] Same dangling-reference problem: "See Appendix B for the vendor assessment
questionnaire." refers to content that is never generated. Since the whole point of this fixture is
exercising cross-reference extraction deterministically, unresolvable targets should be deliberate
(and documented) rather than accidental.


══════ F0828 │ tests/common/mod.rs:50-54 │ [test · medium] ══════
[test · medium] This normalization over-collapses: every string merely beginning with '/' (e.g.,
OSCAL remap hrefs like "/oscal/cat/1.1.3", link hrefs, citation anchors, or any prose em-dash line
that starts with a slash) is replaced with "NORMALIZED_PATH", so genuine content differences in
those fields are masked instead of caught by golden-file comparison. At the same time, UNC
("\\\\server\\share\\...") and extended-length ("\\\\?\\C:\\...") paths escape both this check and
is_windows_path(), leaking machine-specific absolute paths into snapshots on Windows CI. Normalize
only strings that look like this repo's own output paths (e.g., require a known prefix such as
env!("CARGO_MANIFEST_DIR") or tests/fixtures), rather than 'starts with /'.

              if UUID_RE.is_match(s) {
                  Value::String("00000000-0000-0000-0000-000000000000".to_string())
-             } else if s.starts_with('/') || is_windows_path(s) {
+             } else if looks_like_local_path(s) {
+                 // Only normalize repo-local absolute paths; keep OSCAL-style
+                 // "#..."/"/uri" references intact so content diffs stay visible.
                  Value::String("NORMALIZED_PATH".to_string())
              } else {


══════ F0829 │ tests/common/mod.rs:63-69 │ [bug · medium] ══════
[bug · medium] Windows path detection only recognizes drive-letter form (C:\ or C:/) in positions
0..=2. Windows absolute paths can also be verbatim/extended-length form ('\\\\?\\C:\\Users\\...') or
UNC form ('\\\\server\\share\\fixture.md'); canonicalize() commonly returns the \\\\?\\ form. Such
strings fall through unnormalized, producing environment-dependent snapshot values and flaky
cross-machine golden tests. Consider matching an optional leading r"\\\\?\\" / "\\\\UNC\\" prefix
(e.g., via a single regex like ^(?i:(?:\\\\\\\\\?|\\\\\\\\UNC)?\\\\(?:[^\\\\/:*?"<>|]+\\\\)+) before
falling back to this fast-path check.
