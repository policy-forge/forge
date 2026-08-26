# Validation slice slice11 — 61 findings
Severity mix: low×61


══════ F0703 │ src/parse/mod.rs:125-136 │ [performance · low] ══════
[performance · low] Avoidable copying on the per-heading hot path: `title_buf.clone()` copies the
whole buffered title only for `title_buf` to be cleared again on the next heading (`clear()` keeps
capacity, the clone always reallocates up to cap). `mem::take(&mut title_buf)` moves the string out
with zero copying while preserving buffer reuse semantics across iterations.

                  let node = SectionNode {
-                     title: title_buf.clone(),
+                     title: std::mem::take(&mut title_buf),
                      heading_level: current_level,
                      source_line: current_line,
                      body_text: None,
                      children: Vec::new(),
                  };

                  pop_to_parent(&mut stack, &mut roots, current_level);

                  stack.push((current_level, node));
              }


══════ F0707 │ src/parse/mod.rs:182-187 │ [other · low] ══════
[other · low] Edge case: standalone text containing only whitespace/newlines that appears
immediately before the next heading still causes this arm to call `get_or_insert_with`, allocating a
`Some("\n")` body on the previous section that carries no content. Current behavior stays correct
only because `finalize_body` later trims it back to `None`; if trimming ever loosens, blank runs
between headings would surface as whitespace-only bodies. Consider skipping accumulation when the
payload is all-whitespace, or relying solely on `finalize_body` as the single normalization point
(and documenting that).


══════ F0704 │ src/parse/mod.rs:222-230 │ [performance · low] ══════
[performance · low] `body.trim_end().to_string()` always allocates a fresh String copy even when
there is no trailing whitespace (the common case), doubling peak memory for every non-empty body
across the whole tree. Trim in place instead: compute the trimmed byte length and `truncate`; only
convert to `None` when everything was whitespace.

  fn finalize_body(node: &mut SectionNode) {
      if let Some(ref mut body) = node.body_text {
-         let trimmed = body.trim_end().to_string();
-         if trimmed.is_empty() {
+         let trimmed_len = body.trim_end().len();
+         if trimmed_len == 0 {
              node.body_text = None;
          } else {
-             *body = trimmed;
+             body.truncate(trimmed_len);
          }
      }


══════ F0706 │ src/parse/mod.rs:262-267 │ [documentation · low] ══════
[documentation · low] The `source_line` convention has edge cases that downstream consumers must
agree with, but they are undocumented here: (a) a byte offset pointing at the '\n' itself resolves
to the *preceding* line, because `partition_point(|&start| start <= offset)` places line N+1's start
after the newline; (b) an offset at EOF of a file ending in '\n' maps past the last real line
(returning `line_starts.len() + ...`, i.e. a phantom line number) instead of being clamped. Since
assemble.rs builds line ranges like `[section.source_line, next_sibling.source_line)` from these
values, any change here silently shifts requirement ownership. Please pin the contract (e.g. "the
line containing the byte at `offset`; newline bytes belong to the line they terminate") and consider
clamping to the last valid line, or at minimum assert `debug_assert!(offset <
line_starts.last()...)` callers pass valid offsets.

  /// Convert a byte offset to a 1-based line number using the line-starts table.
  ///
  /// Uses binary search (`partition_point`) for O(log n) lookup.
+ ///
+ /// Convention: the returned line is the one *containing* the byte at `offset`
+ /// (newline bytes belong to the line they terminate). Offsets at/after EOF of a
+ /// file ending in `'\n'` resolve one line past the last real line; callers must
+ /// pass offsets within the parsed content's ranges.
  pub(crate) fn offset_to_line(offset: usize, line_starts: &[usize]) -> usize {
+     debug_assert!(line_starts.first() == Some(&0));
      line_starts.partition_point(|&start| start <= offset)
  }


══════ F0708 │ src/parse/mod.rs:593-593 │ [test · low] ══════
[test · low] This integration pin hard-codes `md_count == 25`, so adding/removing any example
fixture breaks an unrelated parsing test for a reason unrelated to parsing; prefer asserting
`md_count > 0` plus keeping exact-count pins in dedicated fixture-inventory tests. Additionally,
`extract_sections` advertises `ForgeError::Parse`, but no test exercises the error path (the happy
path is covered extensively here), leaving the documented error branch untested.


══════ F0685 │ src/parse/modality.rs:43-45 │ [documentation · low] ══════
[documentation · low] This "Not part of the public API" claim is inaccurate: `src/lib.rs` declares
`pub mod parse` and `src/parse/mod.rs` declares `pub mod modality`, so
`forge::parse::modality::ModalityResult` and `forge::parse::modality::detect_modality` are publicly
reachable from outside the crate. Either actually restrict them (both have only in-crate callers
today) or drop the disclaimer — otherwise external users may grow dependencies on a diagnostic type
the author intends to keep free-form.

- /// Not part of the public API — used internally by [`annotate_modalities`].
+ /// Internal classification output (crate-visible only).
  #[derive(Debug)]
- pub struct ModalityResult {
+ pub(crate) struct ModalityResult {


══════ F0689 │ src/parse/modality.rs:43-45 │ [documentation · low] ══════
[documentation · low] This "Not part of the public API" claim is inaccurate: `src/lib.rs` declares
`pub mod parse` and `src/parse/mod.rs` declares `pub mod modality`, so
`forge::parse::modality::ModalityResult` and `detect_modality` are publicly reachable from outside
the crate. Either actually restrict them (both have only in-crate callers today) or drop the
disclaimer — otherwise external users may grow dependencies on a diagnostic type the author intends
to keep free-form.

- /// Not part of the public API — used internally by [`annotate_modalities`].
+ /// Internal classification output (crate-visible only).
  #[derive(Debug)]
- pub struct ModalityResult {
+ pub(crate) struct ModalityResult {


══════ F0686 │ src/parse/modality.rs:52-56 │ [maintainability · low] ══════
[maintainability · low] `is_default` and `has_conflict` together encode a single four-way outcome
(explicit normative / explicit advisory / default / conflict), yet their mutual exclusivity and
interplay with `modality` are enforced only by convention and a unit test
(`invariant_is_default_and_has_conflict_mutually_exclusive`). Invalid combinations remain
representable for callers constructing the struct, and consumers must replicate the match logic
instead of relying on exhaustive enum arms. Model the outcome as one enum carrying its verbs
variant-wise so illegal states cannot be constructed.

-     /// `true` when no modality verb was detected; default (Normative) applied.
-     pub is_default: bool,
-     /// `true` when both normative and advisory verbs were detected.
-     /// Normative classification applied (strongest wins).
-     pub has_conflict: bool,
+     /// Full detection outcome; encoding legality at the type level replaces
+     /// the `is_default`/`has_conflict` boolean pair and its invariant test.
+     pub(crate) outcome: ModalityOutcome,


══════ F0690 │ src/parse/modality.rs:52-56 │ [maintainability · low] ══════
[maintainability · low] `is_default` and `has_conflict` together encode a single four-way outcome
(explicit normative / explicit advisory / default / conflict), yet their mutual exclusivity and
interplay with `modality` are enforced only by convention and a unit test
(`invariant_is_default_and_has_conflict_mutually_exclusive`). Invalid combinations remain
representable for any constructor, and consumers must replicate match logic instead of relying on
exhaustive enum arms. Model the outcome as a single enum carrying its verbs variant-wise so illegal
states cannot be constructed.

-     /// `true` when no modality verb was detected; default (Normative) applied.
-     pub is_default: bool,
-     /// `true` when both normative and advisory verbs were detected.
-     /// Normative classification applied (strongest wins).
-     pub has_conflict: bool,
+     /// Detection outcome; encoding legality at the type level replaces
+     /// the `is_default`/`has_conflict` boolean pair and its invariant test.
+     pub(crate) outcome: ModalityOutcome,


══════ F0687 │ src/parse/modality.rs:75-79 │ [performance · low] ══════
[performance · low] Every regex hit is case-mapped into a freshly allocated `String` and collected,
and BOTH vectors are built unconditionally for every requirement even when one side is immediately
discarded (e.g. `advisory_hits` is dropped in the pure-normative arm). Over a full-document
enrichment pass this means O(total keyword hits) heap allocations purely for presence flags and
diagnostics. The captures already borrow from `text`, so a `Vec<&str>` (paired with making the type
crate-private and lifetime-parameterized) eliminates the lowercase allocations entirely.

-     let normative_hits: Vec<String> =
-         NORMATIVE_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();
+     let normative_hits: Vec<&str> =
+         NORMATIVE_PATTERN.captures_iter(text).map(|c| c[1].as_str()).collect();

-     let advisory_hits: Vec<String> =
-         ADVISORY_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();
+     let advisory_hits: Vec<&str> =
+         ADVISORY_PATTERN.captures_iter(text).map(|c| c[1].as_str()).collect();


══════ F0691 │ src/parse/modality.rs:75-79 │ [performance · low] ══════
[performance · low] Every regex hit is case-mapped into a freshly allocated `String` and collected,
and BOTH vectors are built unconditionally for every requirement even when one side is discarded
immediately (e.g. `advisory_hits` is dropped in the pure-normative arm). Over a full-document
enrichment pass this means O(total keyword hits) heap allocations purely for presence flags and
diagnostics. The captures already borrow from `text`, so a `Vec<&str>` (paired with making the type
crate-private and lifetime-parameterized) eliminates the lowercase allocations entirely.

-     let normative_hits: Vec<String> =
-         NORMATIVE_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();
+     let normative_hits: Vec<&str> =
+         NORMATIVE_PATTERN.captures_iter(text).map(|c| c[1].as_str()).collect();

-     let advisory_hits: Vec<String> =
-         ADVISORY_PATTERN.captures_iter(text).map(|c| c[1].to_lowercase()).collect();
+     let advisory_hits: Vec<&str> =
+         ADVISORY_PATTERN.captures_iter(text).map(|c| c[1].as_str()).collect();


══════ F0729 │ src/pipeline.rs:174-175 │ [maintainability · low] ══════
[maintainability · low] The citation→resource UUID map returned by `generate_back_matter` is
discarded (`_resource_map`). Nothing in production code ever wires controls to these resources
(`generate_control_links`, the API designed to consume this map, is referenced only from tests), so
every emitted back-matter resource is unreferenced by any control/group link. Either generate
`href="#<uuid>"` links on the controls whose citations produced each resource, or document why the
map is intentionally unused; otherwise resources silently ship orphaned.

-     let (back_matter_resources, _resource_map) =
+     let (back_matter_resources, resource_map) =
          crate::oscal::generate_back_matter(&all_citations)?;
+     // TODO: thread `resource_map` through build_catalog/embedding so each
+     // cited requirement's control gets an OscalLink { href: "#<uuid>" }
+     // (see oscal::back_matter::generate_control_links).


══════ F0730 │ src/pipeline.rs:184-195 │ [maintainability · low] ══════
[maintainability · low] Hand-rebuilding the catalog drops fields wholesale instead of replacing only
what changed. Today `build_catalog` happens to leave `controls` empty, but hardcoding `controls:
vec![]` means any top-level controls the builder emits (now or later) are silently deleted, and any
new `OscalCatalog` field added later suffers the same fate. Mutate the returned catalog in place
(uuid/metadata/back-matter are the only placeholders) and reuse it.

-     let oscal_catalog = crate::oscal::OscalCatalog {
-         uuid: real_metadata.uuid.to_string(),
-         metadata: crate::oscal::catalog::OscalMetadata {
+     let mut oscal_catalog = catalog;
+     oscal_catalog.uuid = real_metadata.uuid.to_string();
+     oscal_catalog.metadata = crate::oscal::catalog::OscalMetadata {
              title: real_metadata.title,
              last_modified: real_metadata.last_modified.to_rfc3339(),
              version: real_metadata.version,
              oscal_version: real_metadata.oscal_version,
-         },
-         controls: vec![],
-         groups: catalog.groups,
-         back_matter,
      };
+     oscal_catalog.back_matter = back_matter;


══════ F0725 │ src/pipeline.rs:45-48 │ [performance · low] ══════
[performance · low] Redundant serialization round-trip: the envelope is serialized to a pretty
String and immediately re-parsed into a `serde_json::Value`. Serializing directly with
`serde_json::to_value(envelope)` performs validation on the same data and lets you produce the final
pretty string from the `Value` afterwards, avoiding one full String parse (costly for large
catalogs).

-     let json = serde_json::to_string_pretty(envelope)
+     let json_value =
+         serde_json::to_value(envelope).map_err(|e| ForgeError::Serialization(e.to_string()))?;
+     // ... validate `json_value` ...
+     let json = serde_json::to_string_pretty(&json_value)
          .map_err(|e| ForgeError::Serialization(e.to_string()))?;
-     let json_value: serde_json::Value =
-         serde_json::from_str(&json).map_err(|e| ForgeError::Serialization(e.to_string()))?;


══════ F0727 │ src/pipeline.rs:97-99 │ [documentation · low] ══════
[documentation · low] The warning claims the output "will have empty groups", but in the clause-only
case `assemble_document`/`map_sections` (src/model/assemble.rs, SEC-5 path) synthesizes a "Preamble"
section from the orphaned list items, so the resulting catalog gets exactly one non-empty "Preamble"
group rather than empty groups. The message can mislead operators debugging structureless inputs;
state the actual behavior.

      if sections.is_empty() {
-         tracing::warn!("No identifiable sections found in input — output will have empty groups");
+         tracing::warn!(
+             "No identifiable sections found in input — list-item content will be grouped \
+              under a single synthetic 'Preamble' group"
+         );
      }


══════ F0670 │ src/round_trip/chain.rs:49-52 │ [maintainability · low] ══════
[maintainability · low] The successful `ConvertResult` is discarded wholesale, so the `warnings`
field (non-fatal stderr emitted by oscal-cli with exit code 0) never reaches the user. For a
round-trip pipeline these warnings are precisely the fidelity signals you care about —
reserialization between XML/YAML/JSON can coerce types or lose formatting and often announces it
only via such warnings. Log them instead of silently dropping the value.

-         invoker.convert(&args)?;
+         let result = invoker.convert(&args)?;
+         for warning in &result.warnings {
+             tracing::warn!(step = step_num + 1, "oscal-cli convert warning: {warning}");
+         }
      }

      Ok(rt_json_path)


══════ F0697 │ src/round_trip/comparator.rs:57-58 │ [bug · low] ══════
[bug · low] Paths are built via naive `format!("{path}/{key}")` without RFC 6901 escaping (`~` must
become `~0`, `/` must become `~1`). Any object key containing those characters produces a
`json_path` that is not a valid JSON Pointer, so programmatic consumers that resolve paths will
address the wrong node. Apply JSON Pointer escaping when constructing child paths, or explicitly
document `json_path` as a custom, non-RFC 6901 notation so downstream tooling parses it accordingly.


══════ F0675 │ src/round_trip/divergence.rs:48-50 │ [maintainability · low] ══════
[maintainability · low] Naming conventions are inconsistent across the persisted report:
CompatibilityClassification renames variants to kebab-case while DivergenceClass and
ResolutionStatus (which appear in the same JSON document via Divergence/RoundTripResult) serialize
as PascalCase ("ForgeFix" next to "advisory-older-model-baseline"). Additionally, the Display impl
hand-duplicates the rename_all strings, so any future edit to the attribute or the match arms
silently diverges human-readable from machine-readable output. Pick one convention for all enums in
this report, and pin the correspondence with a test such as: for each variant, assert
`serde_json::to_value(v).unwrap() == v.to_string()`.


══════ F0674 │ src/round_trip/divergence.rs:79-79 │ [bug · low] ══════
[bug · low] Exact string comparison against "1.0.3" is brittle: real CLI banners commonly emit forms
such as "v1.0.3", "oscal-cli 1.0.3", "1.0.3+build", or prerelease suffixes ("1.0.3-rc.1"), all of
which silently fall into the catch-all branch and get a different classification than the actual
same version. Parse the input with a semver parser (e.g. the `semver` crate) after stripping a
prefix/tool-name once, and compare the parsed triple so legitimate invocations are classified
deterministically.


══════ F0679 │ src/round_trip/divergence.rs:87-87 │ [style · low] ══════
[style · low] (See previous comment.) Companion enum definition for the strongly-typed artifact_type
field:

- /// Aggregate result of a single round-trip validation run.
+ #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
+ pub enum ArtifactType {
+     Catalog,
+     ComponentDefinition,
+ }


══════ F0677 │ src/round_trip/divergence.rs:88-89 │ [maintainability · low] ══════
[maintainability · low] RoundTripResult derives only Debug + Serialize, while its child type
Divergence derives Debug/Clone/Serialize/Deserialize/PartialEq. As written, the aggregate result
cannot itself be deserialized, compared, cloned, or round-tripped in tests and downstream tooling —
ironic for a round-trip-validation record. Derive the symmetric set so consumers can reload and diff
previously generated reports.

- #[derive(Debug, Serialize)]
+ #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct RoundTripResult {


══════ F0678 │ src/round_trip/divergence.rs:90-91 │ [maintainability · low] ══════
[maintainability · low] `artifact_type: String` documents exactly two legal values ("Catalog" /
"ComponentDefinition"), but any typo'd or arbitrary string remains representable, weakening
downstream matching and report integrity. Model it as a closed enum with serde (which will emit the
OSCAL names) so invalid artifact types cannot be constructed.

-     /// OSCAL artifact type: `"Catalog"` or `"ComponentDefinition"`.
-     pub artifact_type: String,
+     /// OSCAL artifact type: `Catalog` or `ComponentDefinition`.
+     pub artifact_type: ArtifactType,


══════ F0683 │ src/round_trip/log.rs:0-0 │ [test · low] ══════
[test · low] Each test parses the written file back into a `serde_json::Value` before snapshotting,
which normalizes away all whitespace. Since the function's documented contract is *pretty-printed*
JSON output, these snapshots cannot detect a regression from pretty-printing to compact output (or
any formatting change); they only validate logical structure. Assert on the raw file bytes/text
instead (e.g. `insta::assert_snapshot!` on `read_to_string`) if the formatting guarantee matters.


══════ F0681 │ src/round_trip/log.rs:20-20 │ [maintainability · low] ══════
[maintainability · low] The bare `?` on `File::create` propagates a bare `io::Error` with no
indication of which log path was involved. When the parent directory is missing or permission is
denied during a batch run, the operator gets "No such file or directory" with zero actionable
context. Attach `output_path.display()` to the error at this boundary (matching the context-enriched
pattern used elsewhere, e.g. `src/cli/export.rs`'s "JSON serialization failed: {e}").

-     let file = std::fs::File::create(output_path)?;
+     let file = std::fs::File::create(output_path).map_err(|e| {
+         ForgeError::Serialization(format!(
+             "failed to create divergence log '{}': {e}",
+             output_path.display()
+         ))
+     })?;


══════ F0682 │ src/round_trip/log.rs:25-25 │ [maintainability · low] ══════
[maintainability · low] Flattening the `serde_json::Error` to a string here discards its structured
category and line/column location machinery. `Display` retains the line/column text, so this is
tolerable given `ForgeError::Serialization` is `String`-typed and sibling modules follow the same
pattern — but the message should at least say *what* was being serialized (see the suggestion),
otherwise callers see a bare serde message with no hint it came from the divergence log.

-         return Err(ForgeError::Serialization(e.to_string()));
+         return Err(ForgeError::Serialization(format!(
+             "failed to serialize divergence log: {e}"
+         )));


══════ F0699 │ src/round_trip/mod.rs:18-18 │ [maintainability · low] ══════
[maintainability · low] If the submodule is renamed to avoid shadowing the well-known `log` crate,
this re-export path must follow suit (`pub use divergence_log::write_divergence_log;`) along with
internal references such as `crate::round_trip::log::...` and `super::log` in tests.

- pub use log::write_divergence_log;
+ pub use divergence_log::write_divergence_log;


══════ F0698 │ src/round_trip/mod.rs:9-9 │ [maintainability · low] ══════
[maintainability · low] The private submodule `mod log;` shares its name with the ubiquitous `log`
facade crate. Nothing misresolves today because this crate logs exclusively via `tracing` (the `log`
crate is not a direct dependency) and sibling modules refer to this file as `super::log`, but the
collision remains a latent trap: any future addition of the `log` crate, or a bare `use log::...`
introduced inside this module tree, will silently bind to this divergence-log submodule instead of
the logging facade. The name also misleads readers since
`crate::round_trip::log::write_divergence_log` writes a structured divergence report, not
diagnostics. Rename the module (e.g., `divergence_log`) and update the re-export path.

- mod log;
+ mod divergence_log;


══════ F0702 │ src/round_trip/rules.rs:13-23 │ [style · low] ══════
[style · low] Public API exposes concrete `HashSet<String>` / `Vec<String>` fields with no derives
and no validated constructor, coupling callers to the storage representation and allowing arbitrary
entries (including malformed ones) with zero feedback. Consider deriving `Debug, Clone` (a config
struct used across functions almost always needs both), building via `HashSet::from`, and adding a
fallible constructor/builder that validates entries against known OSCAL unordered-array keys instead
of relying on hand-edited public fields.

+ #[derive(Debug, Clone)]
+ pub struct OscalComparisonRules { /* ... */ }
+
  impl Default for OscalComparisonRules {
      fn default() -> Self {
          Self {
-             unordered_array_paths: ["props", "links", "parts"]
-                 .iter()
-                 .map(|s| (*s).to_string())
-                 .collect(),
-             ignored_paths: vec![],
+             unordered_array_paths: HashSet::from([
+                 "props".to_string(),
+                 "links".to_string(),
+                 "parts".to_string(),
+             ]),
+             ignored_paths: Vec::new(),
          }
      }
  }


══════ F0717 │ src/sanitize.rs:1-2 │ [documentation · low] ══════
[documentation · low] Preserving tab/newline is an intentional trade-off, but combined with
untrusted input it lets an attacker fabricate additional plausible lines (and odd column layouts via
tabs) when output goes to logs, CI annotations, or single-line TUI status bars. Please document this
caller obligation in the public doc comment so the SEC-5 contract is explicit instead of implicit.

- /// Strip ASCII control characters (0x00-0x1F, excluding 0x0A newline and 0x09 tab)
+ /// Strip ASCII control characters (0x00-0x1F, excluding 0x09 tab and 0x0A newline)
  /// and DEL (0x7F) from a string. Used to prevent terminal escape injection (SEC-5).
+ ///
+ /// Tab and newline are deliberately preserved. Callers that feed untrusted input
+ /// into single-line surfaces (log entries, status bars) must additionally collapse
+ /// newlines/tabs themselves, since embedded LFs allow attackers to forge extra
+ /// plausible output lines.
+ pub fn strip_control_chars(s: &str) -> String {


══════ F0718 │ src/sanitize.rs:18-23 │ [test · low] ══════
[test · low] The unit tests never exercise the boundaries the numeric predicate exists for: NUL
(0x00), carriage return (0x0D), VT/FF (0x0B/0x0C), the exact 0x1F-stripped vs 0x20-kept boundary,
nor multibyte non-ASCII passthrough (e.g. "中\u{e9}" must survive). An off-by-one change such as `>=
0x21` or accidentally filtering by `as u32 > 0x7F` would go unnoticed. Also note a
stripped-but-not-executed payload remains visible ('ESC[31m' degrades to '[31m'); pinning that
residue explicitly guards downstream regressions, and once C1 handling (see related comment above)
lands, an assertion that "\u{9b}[31m" comes out empty will lock the fix in — today that assertion
would fail, proving the gap.

      #[test]
-     fn strip_ansi_escape() {
-         let input = "Hello\x1b[31m World\x1b[0m";
-         let result = strip_control_chars(input);
-         assert_eq!(result, "Hello[31m World[0m");
+     fn strip_other_c0_and_boundaries() {
+         // NUL, CR, VT, FF, SOH... all removed; last C0 (0x1F) removed, first graphic (space, 0x20) kept.
+         assert_eq!(strip_control_chars("a\u{0}\r\u{b}\u{c}\u{1}b\u{1f} c"), "ab c");
+         // DEL removed.
+         assert_eq!(strip_control_chars("x\u{7f}y"), "xy");
+         // Multibyte non-ASCII content must pass through untouched.
+         assert_eq!(strip_control_chars("中文 café"), "中文 café");
      }


══════ F0760 │ src/summary/format.rs:117-123 │ [bug · low] ══════
[bug · low] The overflow logic mixes two independently supplied sources: `shown` is capped by the
message list while `total` is the separate `validation_errors` counter. Callers passing fewer
messages than expected show "and N more..." (fine), but the reverse — more than 3 stored messages
while `validation_errors <= 3` — silently drops the extras with no indication, yielding a listing
that contradicts itself versus any count-based summary. Since these come from different call sites
today, either derive both from one source or clamp defensively (e.g. `let total =
total.max(msgs.len());`).

+     let total = total.max(msgs.len()); // counters and messages should agree
      let shown = msgs.len().min(3);
      for msg in msgs.iter().take(shown) {
          lines.push(format!("  {msg}"));
      }
      if total > shown {
          lines.push(format!("  and {} more...", total - shown));
      }


══════ F0758 │ src/summary/format.rs:155-157 │ [maintainability · low] ══════
[maintainability · low] Unchecked subtraction chain relies on several silent couplings: MIN_WIDTH
(41) > LABEL_WIDTH (22) > title byte-length (24), and every hardcoded label prefix being exactly
LABEL_WIDTH chars. Today the math is safe, but changing any constant, lengthening a label, or making
the title longer/non-ASCII (`title.len()` counts bytes, not chars) turns `w - title.len()`,
`remaining`, or `w - LABEL_WIDTH` into a usize-underflow panic (debug) or wrapped garbage widths
(release), with silent box misalignment otherwise. Guard the invariants cheaply so edits fail
loudly.

-     // LABEL_WIDTH includes the leading │ border char, so +1 to align
-     // row total (LABEL_WIDTH + vw + 1 for closing │) with border total (1 + w + 1)
+     const _: () = assert!(MIN_WIDTH > LABEL_WIDTH, "MIN_WIDTH must cover LABEL_WIDTH");
+     ...
+     debug_assert!(
+         w >= title.len(),
+         "computed width {w} smaller than title byte length {}",
+         title.len()
+     );
      let vw = w - LABEL_WIDTH + 1;


══════ F0759 │ src/summary/format.rs:27-32 │ [maintainability · low] ══════
[maintainability · low] The escape scanner terminates on any ASCII alphabetic character, which is
only correct for the module's own SGR sequences (\x1b[..letter). Other escape forms that reach a
stats field would be miscounted: OSC sequences terminated by BEL/ST keep scanning until a stray
letter, shrinking the reported visible length and breaking column alignment in ways the plain-text
tests won't catch. At minimum document the supported-subset invariant next to the helper; ideally
parse the full CSI/OSC grammar or rely on a unicode-width crate that understands escapes.

+         // NOTE: assumes SGR-style CSI sequences (ESC '[' + params + final byte).
+         // OSC sequences (terminated by BEL/ST) or letters inside escape payloads
+         // are NOT handled; ensure all colored input comes from the constants above.
          if in_escape {
              if c.is_ascii_alphabetic() {
                  in_escape = false;
              }
          } else if c == '\x1b' {
              in_escape = true;


══════ F0721 │ src/summary/mod.rs:153-155 │ [test · low] ══════
[test · low] The `depth > 64` early-return branch of `count_catalog_controls` has no test coverage:
all existing tests build catalogs shallowly, so the truncation behavior (and any future fix such as
an exhaustive iterative traversal) is unverified. Because the branch is reachable only through deep
nesting, construct the fixture programmatically (a loop chaining 65+ single-group levels containing
one control each) and assert the expected total — either the truncated count (documenting today's
silent under-count) or the full count after switching to an iterative implementation.

+     #[test]
+     fn count_catalog_controls_deep_nesting() {
+         use crate::oscal::catalog::{make_control_for_test, OscalControl, OscalGroup};
+         // Build a 70-level nested chain with one control per level.
+         let leaf = OscalGroup {
+             id: "leaf".into(),
+             title: "Leaf".into(),
+             props: vec![],
+             links: vec![],
+             controls: vec![make_control_for_test("deep")],
+             groups: vec![],
+         };
+         let mut group = leaf;
+         for level in 0..69 {
+             group = OscalGroup {
+                 id: format!("g{level}"),
+                 title: format!("G{level}"),
+                 props: vec![],
+                 links: vec![],
+                 controls: vec![make_control_for_test(&format!("c{level}"))],
+                 groups: vec![group],
+             };
+         }
+         // Assert exact expected total (70 with an exhaustive traversal).
+     }
+
      // T006: count_catalog_controls() tests
      #[test]
      fn count_catalog_controls_empty() {


══════ F0720 │ src/summary/mod.rs:63-73 │ [maintainability · low] ══════
[maintainability · low] The hand-written `Default` impl duplicates the entire field list (relying on
the compiler to catch omissions, but not semantic drift) and bakes in two policy decisions:
`strategy: Strategy::Catalog` means an unpopulated/default statistics record already advertises a
concrete conversion strategy — the dashboard will render "Strategy: Catalog" for values that were
never set, with no sentinel distinguishing 'not populated' from 'real catalog run'. If `Strategy`
grows or if a run's strategy genuinely should default differently, this coupling must be updated
manually. Consider whether `NotRun`-style semantics apply to strategy too (e.g., make `strategy` an
`Option<Strategy>` here, or keep `Default::default()` only as a test fixture), and prefer deriving
`Default` if `Strategy` ever gains a `#[default]` attribute.

- impl Default for ConversionStatistics {
-     fn default() -> Self {
-         Self {
-             sections_parsed: 0,
-             requirements_extracted: 0,
-             controls_generated: 0,
-             validation_status: ValidationStatus::default(),
-             validation_errors: 0,
-             validation_warnings: 0,
-             validation_error_messages: Vec::new(),
-             strategy: Strategy::Catalog,
+ // Either document that `strategy: Strategy::Catalog` in the default is only a
+ // placeholder (callers must always overwrite it), or model 'strategy unknown'
+ // explicitly:
+ //   pub strategy: Option<Strategy>,   // None until the pipeline assigns one
+ // so an unpopulated record cannot masquerade as a Catalog-strategy run.


══════ F0714 │ src/testing/mod.rs:6-6 │ [test · low] ══════
[test · low] This testing-utility submodule is compiled unconditionally into the released library
(`src/lib.rs` declares `pub mod testing;` without any gate), so production builds of every
downstream consumer carry `semantic_eq`'s `serde_json` comparison logic, structs, and its inline
`#[cfg(test)]` unit tests through the public API. Consider gating the whole testing surface, e.g.
`#[cfg_attr(docsrs, doc(cfg(feature = "testing")))] #[cfg(any(test, feature = "testing"))] pub mod
semantic_eq;` and mirroring the gate on `pub mod testing;` in lib.rs, so it stays excluded from
release builds unless opted in.

+ #[cfg(any(test, feature = "testing"))]
  pub mod semantic_eq;


══════ F0715 │ src/testing/mod.rs:6-8 │ [maintainability · low] ══════
[maintainability · low] Because `semantic_eq` is itself declared `pub` *and* its items are
re-exported, every exported symbol becomes reachable via two public paths
(`forge::testing::EquivalenceResult` and `forge::testing::semantic_eq::EquivalenceResult`), doubling
the documented API surface and committing you to keep both paths stable forever. Pick a single
canonical path: either keep the flattened re-export as the only public entry point (drop `pub` from
the module) or keep the module public and drop the manual re-export list.

- pub mod semantic_eq;
+ // Private internal home; the flat re-export below is the public API.
+ mod semantic_eq;

  pub use semantic_eq::{EquivalenceDiff, EquivalenceResult, assert_semantic_equivalence};


══════ F0738 │ src/testing/semantic_eq.rs:120-124 │ [other · low] ══════
[other · low] When array lengths differ, elements beyond min_len are never examined, so callers
learn sizes but not the contents of the extra/missing entries — a single aggregate diff stands in
for potentially many concrete discrepancies. If coarse granularity is intentional (order-significant
comparison cannot reliably align shifted lists), document that contract on
compare_arrays/assert_semantic_equivalence; otherwise push per-index EquivalenceDiffs for the
surplus range so round-trip failure reports point at the offending data.

-     let min_len = exp_arr.len().min(act_arr.len());
-     for i in 0..min_len {
-         let child_path = format!("{path}/{i}");
-         compare_values(&exp_arr[i], &act_arr[i], &child_path, diffs);
-     }
+     // Optionally itemize surplus elements so reports show what is extra/missing:
+     // for i in min_len..exp_arr.len() { diffs.push(... expected = format_value(&exp_arr[i]), actual = None ...) }
+     // for i in min_len..act_arr.len() { diffs.push(... expected = None, actual = format_value(&act_arr[i]) ...) }


══════ F0735 │ src/testing/semantic_eq.rs:173-177 │ [bug · low] ══════
[bug · low] Thresholding and truncation use inconsistent metrics: s.len() is a byte length, but
truncation (chars().take(500)) and the '(N chars total)' total are char counts. A multibyte payload
of ~300 chars (~900 bytes) crosses the byte threshold yet gets no truncation at all, while acquiring
a misleading truncation suffix ('(300 chars total)'). Additionally, Value::String payloads skip
truncation entirely, so a multi-MB string floods the diff output verbatim (and embedded
quotes/newlines are emitted raw). Measure both threshold and total in chars (or both in bytes) and
apply the same guard to the String arm.

              let s =
                  serde_json::to_string(value).unwrap_or_else(|_| "<unrepresentable>".to_string());
-             if s.len() > 500 {
+             let char_count = s.chars().count();
+             if char_count > 500 {
                  let truncated: String = s.chars().take(500).collect();
-                 format!("{truncated}… ({} chars total)", s.chars().count())
+                 format!("{truncated}… ({char_count} chars total)")


══════ F0737 │ src/testing/semantic_eq.rs:47-52 │ [documentation · low] ══════
[documentation · low] The first sentence pair of this doc comment describes the recursive comparison
performed by compare_values, not this function; as a result escape_json_pointer_token is
misdescribed and compare_values itself has no doc comment. Move those two sentences onto
compare_values and keep only the RFC 6901 escaping description here.

- /// Recursive comparison of JSON Value nodes.
- ///
- /// Accumulates all differences into `diffs` with JSON Pointer-style paths.
  /// Escape a JSON object key for use in a JSON Pointer token per RFC 6901.
  /// `~` is replaced with `~0` and `/` is replaced with `~1`.
  fn escape_json_pointer_token(token: &str) -> String {


══════ F0736 │ src/testing/semantic_eq.rs:56-56 │ [bug · low] ══════
[bug · low] Recursion has no depth bound. serde_json's parser caps nesting at deserialize time
(default 128), but Values built programmatically (builders, loops inserting one wrapper level at a
time) bypass that limit; compare_values will recurse one frame per level and overflow the stack,
aborting the whole test process instead of yielding a diff. Add a max-depth parameter (or
thread-local/env-configured cap) that records a single 'nesting deeper than N' EquivalenceDiff and
stops descending.

- fn compare_values(expected: &Value, actual: &Value, path: &str, diffs: &mut Vec<EquivalenceDiff>) {
+ fn compare_values(
+     expected: &Value,
+     actual: &Value,
+     path: &str,
+     diffs: &mut Vec<EquivalenceDiff>,
+     depth: usize,
+ ) {


══════ F0724 │ src/trace/extractor.rs:38-45 │ [maintainability · low] ══════
[maintainability · low] Absence of `source_file` / `source_line` is encoded with raw sentinels (`""`
and `0`) rather than `Option<usize>`, and every consumer must remember that convention:
`source_section != ""` gates "mapped", `source_line == 0` means "no line", and `source_file == ""`
means "unattributed group" — all three conventions differ. Note that a forged-but-empty
`source-section: ""` (a legitimate string value, not just the type-mismatch case above) also passes
the `source_section?` gate, conflating "unmapped" with "mapped to empty heading". At minimum
document these sentinel invariants on the construction site and guard against an empty section
value; better long-term is modelling the tri-state with `Option`s in `TraceMetadata`.

-     // Must have at least source-section to be considered mapped
-     let section = source_section?;
+     // Must have a non-empty source-section to be considered mapped
+     let section = source_section.filter(|s| !s.is_empty())?;

      Some(TraceMetadata {
-         source_file: source_file.unwrap_or_default(),
+         source_file: source_file.filter(|f| !f.is_empty()).unwrap_or_default(),
          source_section: section,
-         source_line: source_line.unwrap_or(0),
+         source_line: source_line.unwrap_or(0), // 0 sentinel: no forge:source-line prop (groups)
      })


══════ F0773 │ src/trace/formatter.rs:109-110 │ [maintainability · low] ══════
[maintainability · low] The footer prints `total/mapped/unapped` percentages from the precomputed,
pub-visible `report.summary` field rather than deriving them from `report.entries`, the same data
actually rendered as `[unmapped]` markers above. `TraceReport` cannot enforce `summary ==
TraceSummary::from_entries(&entries)`, so any caller (or builder bug) that desynchronizes the two
produces a footer that silently contradicts the rows — precisely the failure mode that corrupts
audit evidence. Safer options: recompute locally with `TraceSummary::from_entries(&report.entries)`,
or add a `debug_assert_eq!` consistency check before printing.


══════ F0772 │ src/trace/formatter.rs:124-128 │ [test · low] ══════
[test · low] This helper has three distinct outcome paths — Group+line 0 → em dash, other types with
line 0 → "0 ⚠", and valid-but-out-of-range lines appending "⚠" — but the test module below only
exercises the Group/em-dash path and ordinary in-range lines (`format_with_group_em_dash`, snapshots
with lines 10/25/1). The remaining two warning branches have zero coverage, despite being exactly
the malformed-input cases whose rendering matters for traceability evidence. Add direct unit tests:
e.g. `format_source_line(0, ElementType::Control, 100)` == "0 ⚠" and `format_source_line(101,
ElementType::Control, 100)` == "101 ⚠", plus an ImplementedRequirement line-0 case.


══════ F0771 │ src/trace/formatter.rs:21-24 │ [performance · low] ══════
[performance · low] All rows are eagerly materialized into `Vec<[String; 4]>` (~5 heap allocations
per entry: section string, formatted line, element_id string, type string) purely to enable the
two-pass width computation. For large trace reports this doubles memory beyond the input data.
Widths can be derived in one pass over borrowed `&TraceEntry` values (recomputing each cell's char
count twice without materializing Strings, or storing cached usize widths), and rows written
streaming directly from `report.entries.iter()`.


══════ F0769 │ src/trace/formatter.rs:45-47 │ [bug · low] ══════
[bug · low] Header widths are measured with byte-based `len()` while cell widths use
`chars().count()`, but Rust's `{:<w$}` padding for strings counts chars. With the current ASCII
headers the two metrics coincide, so this works today; however, any future non-ASCII header would
get an undersized width (byte len < char count) and silently shift every subsequent column.
Additionally, neither metric equals terminal display width: wide (CJK) or zero-width/combining
characters can survive strip_control_chars inside element_id/source_section, so such rows render
wider than their allocated column and misalign the table. Prefer `chars().count()` (or a
unicode-width helper) uniformly for both headers and cells.

+     let display_width = |s: &str| -> usize { s.chars().count() };
      for (i, header) in headers.iter().enumerate() {
-         widths[i] = header.len();
+         widths[i] = display_width(header);
      }
+     // Consider the `unicode-width` crate if wide characters must align too.


══════ F0753 │ src/trace/mod.rs:36-36 │ [documentation · low] ══════
[documentation · low] The documented error condition is inaccurate in two ways: (1)
`TraceUnsupportedArtifact` is raised not only for *unrecognized* artifact types but also for
*recognized but unsupported* ones — `generate_trace_report` explicitly rejects Profile and Mapping
inputs, and `walker::detect_artifact_type` additionally rejects ambiguous artifacts (both keys
present) and non-object roots; (2) the list omits `ForgeError::FileTooLarge`, which `read_file`'s
MAX_FILE_SIZE guard can propagate. Update the `# Errors` block so callers can rely on it for
exhaustive error handling.

- /// - `ForgeError::TraceUnsupportedArtifact` if artifact type is unrecognized
+ /// - `ForgeError::TraceUnsupportedArtifact` if the artifact type is unrecognized
+ ///   or unsupported (Profile, Mapping, ambiguous, or non-object root)
+ /// - `ForgeError::FileTooLarge` if artifact or source exceeds `MAX_FILE_SIZE`


══════ F0754 │ src/trace/mod.rs:41-41 │ [documentation · low] ══════
[documentation · low] This comment misstates the actual behavior: `read_file` deliberately catches
`ErrorKind::NotFound` from the size pre-check and lets `read_to_string` surface it, where it is
mapped to the dedicated `ForgeError::FileNotFound { path }` variant — not to a generic 'Io mapping'.
Misleading comments are worse than none; either correct it ('missing files become FileNotFound from
the actual read') or delete the parenthetical.

-     // Read and parse artifact JSON (handles FileNotFound via Io mapping)
+     // Read and parse artifact JSON (missing files surface as
+     // ForgeError::FileNotFound from the actual read below)


══════ F0752 │ src/trace/mod.rs:47-48 │ [performance · low] ══════
[performance · low] The entire source policy is read into a `String` but the content itself is never
used — only `lines().count()` survives. For large policies this needlessly doubles peak memory
(UTF-8 string buffer + the file contents on read). Stream the count instead, e.g.
`BufReader::new(File::open(..)).lines().count()`, ideally from a shared open-and-validate helper
that keeps the existing NotFound/PermissionDenied normalization.

-     let source_content = read_file(source_path)?;
-     let source_line_count = source_content.lines().count();
+     let source_file = std::fs::File::open(source_path)
+         .map_err(|e| match e.kind() {
+             std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: source_path.to_path_buf() },
+             _ => ForgeError::Io(e),
+         })?;
+     let source_line_count = std::io::BufReader::new(source_file).lines().count();


══════ F0755 │ src/trace/mod.rs:53-56 │ [maintainability · low] ══════
[maintainability · low] `Value`'s `Index` impl silently yields `Value::Null` for a missing/mistyped
key, so this line's correctness depends entirely on a cross-module invariant:
`walker::detect_artifact_type` (src/trace/walker.rs:32-38) having just verified the key exists and
is an object. Today that invariant holds, but any future reorder, a second walk entry point, or an
edit to the detector turns a regression here into a valid-looking report with zero trace entries
instead of an error — and the style diverges from the `.get(...)` chain used for
`metadata.last-modified` a few lines below. Make the failure mode local by using the same explicit
access pattern as the rest of the function. Same applies to the `&json["component-definition"]` arm.

          OscalModelType::Catalog => {
-             let catalog = &json["catalog"];
+             let catalog = json.get("catalog").unwrap_or(&serde_json::Value::Null);
              walker::walk_catalog_elements(catalog)
          }


══════ F0756 │ src/trace/mod.rs:57-60 │ [maintainability · low] ══════
[maintainability · low] Same silent-Null indexing hazard as the Catalog arm:
`&json["component-definition"]` relies on `detect_artifact_type` having validated key presence in
another module rather than failing locally and loudly. Prefer the explicit `get` access used
elsewhere in this function so a detector regression surfaces as an error instead of an empty trace
report.

          OscalModelType::ComponentDefinition => {
-             let compdef = &json["component-definition"];
+             let compdef = json.get("component-definition").unwrap_or(&serde_json::Value::Null);
              walker::walk_compdef_elements(compdef)
          }


══════ F0749 │ src/trace/report.rs:224-226 │ [test · low] ══════
[test · low] Missing test coverage for the documented degenerate-vs-inconsistent contracts: there
are no unit tests asserting that (a) a hand-built `TraceSummary { total_elements: X,
mapped_elements: Y }` with `Y > X` clamps via `unmapped_elements().saturating_sub` to 0 — i.e., the
saturating behavior protecting `coverage_percent()` from returning >100.0 for directly-constructed
summaries — nor (b) that mixed multi-mapped/large-N ratios like 1/3 produce sane results.
`from_entries` cannot create state (a), but `TraceSummary` has public fields precisely so users can
build it by hand, so that path deserves a pinning test.

+     #[test]
+     fn summary_inconsistent_fields_saturate() {
+         // Direct construction (public fields) with mapped > total:
+         // unmapped must clamp to 0, coverage must not exceed 100.0.
+         let s = TraceSummary { total_elements: 2, mapped_elements: 5 };
+         assert_eq!(s.unmapped_elements(), 0);
+         assert!((s.coverage_percent() - 250.0).abs() < 1e-9); // documents current behavior
+     }
+
      #[test]
      fn summary_empty_entries() {
          let entries: Vec<TraceEntry> = vec![];


══════ F0747 │ src/trace/report.rs:3-6 │ [style · low] ══════
[style · low] `ElementType` models a closed, fieldless set — derive `Copy`, `Eq`/`PartialEq`
(already present), `Hash`, and crucially `PartialOrd`/`Ord` so determinstic sort order/grouping of
entries is available without hand-written key closures; deriving `Copy` also removes needless
`.clone()` needs since the enum is 1 byte. If the `strum` crate is already a dependency (it is used
in sibling modules), `#[derive(strum::Display)]` with `#[strum(serialize_all = "kebab-case")]`
replaces the boilerplate `as_str`/`Display` pair and keeps the OSCAL strings in lockstep with
variants.

  /// Type of OSCAL element in a traceability report entry.
- #[derive(Debug, Clone, Copy, PartialEq, Eq)]
+ #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  #[non_exhaustive]
  pub enum ElementType {


══════ F0748 │ src/trace/report.rs:83-83 │ [documentation · low] ══════
[documentation · low] Doc typo/range error: contract says "Coverage percentage (0.0–100.0)" but doc
reads `(0.0–100.0)` as `0.0` through `100.0` while the lower bound is written without the decimal
(`0.0` appears nowhere in range notation below because it's literally spelled '0.0'); more
importantly the stated open-closed form matters for display formatting guarantees downstream. State
it precisely: returns a value in `[0.0, 100.0]`.

-     /// Coverage percentage (0.0–100.0). Returns 0.0 if `total_elements` is 0.
+     /// Coverage percentage in `[0.0, 100.0]`; returns `0.0` when `total_elements == 0`.


══════ F0740 │ src/trace/resolver.rs:24-25 │ [bug · low] ══════
[bug · low] Strict `>` misclassifies sources edited moments after artifact generation as fresh when
filesystem mtime granularity is coarser than the RFC 3339 fractional seconds (e.g. ext3/FAT/network
mounts truncate to whole seconds): an edit within the same truncated second yields `source_time ==
oscal_time` and passes as fresh even though it happened after generation. Since this predicate
drives 'regenerate', erring toward stale on ties (`>=`) is safer than missing an edit. At minimum,
document the assumed mtime precision and confirm the intended tie-breaking.

-     let source_time: chrono::DateTime<chrono::Utc> = mtime.into();
-     source_time > oscal_time
+     // Treat ties as stale: coarse mtime granularity can make a post-generation
+     // edit compare equal to the artifact timestamp.
+     source_time >= oscal_time


══════ F0741 │ src/trace/resolver.rs:32-34 │ [maintainability · low] ══════
[maintainability · low] Overloading the bare `usize` with a `0 == group` sentinel makes invalid
states representable: any upstream bug that zeroes out a genuine line number silently passes
validation instead of failing loudly. Per the project's type-design guidelines, model this domain
state explicitly so group-vs-line cannot be confused — e.g. an enum, or `Option<NonZeroUsize>` where
`None` means 'group reference'.

- pub fn validate_line_reference(line_number: usize, source_line_count: usize) -> bool {
-     line_number == 0 || line_number <= source_line_count
+ #[must_use]
+ pub fn validate_line_ref(reference: LineRef, source_line_count: usize) -> bool {
+     match reference {
+         LineRef::Group => true,
+         LineRef::Line(n) => n.get() <= source_line_count,
+     }
+ }
+
+ pub enum LineRef {
+     /// Whole-group reference; not bound to a specific line.
+     Group,
+     Line(std::num::NonZeroUsize),
  }


══════ F0768 │ src/trace/walker.rs:136-140 │ [bug · low] ══════
[bug · low] Downstream callers of these walkers pass `&json["catalog"]` /
`&json["component-definition"]` (see `src/trace/mod.rs`), and this walker treats
`groups`/`controls`/`components` as optional. If the top-level value is a non-object (or an
unexpected array), the walk silently yields an empty report with no diagnostic instead of surfacing
malformed input. Consider returning `Result<Vec<TraceEntry>, ForgeError>` (or at least logging) so a
wrong-typed artifact is reported rather than reported as 'no elements'.

-     if let Some(components) = compdef.get("components").and_then(|c| c.as_array()) {
+     match compdef.get("components") {
+         Some(Value::Array(components)) => {
          for component in components {
              collect_impl_requirements(component, &mut entries);
+             }
+         }
+         Some(_) => {
+             tracing::warn!("'components' is present but not an array; skipping");
          }
+         None => {}
      }


══════ F0763 │ src/types.rs:51-53 │ [maintainability · low] ══════
[maintainability · low] User-facing naming asymmetry: `Strategy::Component` reaches the CLI as
`--strategy component` (default kebab-case), while the semantically identical choice is spelled
`--to component-definition` (`OutputType` carries an explicit `#[value(name =
"component-definition")]`, mirrored in `as_str`). Same concept, two different option values
depending on which flag the user picks — confusing in help text and scripts, and a trap because the
two flags interact in `forge convert`. Either align both spellings to "component-definition", or
document the intentional shorthand here so reviewers know it is deliberate.

      /// Produce an OSCAL Component Definition (implemented requirements).
+     #[value(name = "component-definition")]
      Component,
  }


══════ F0777 │ src/uuid.rs:230-232 │ [performance · low] ══════
[performance · low] &section.title.clone() heap-allocates a fresh String purely to work around an
aliasing borrow conflict (&mut section and &section.title cannot coexist as arguments). The same
shortcoming leaks into assign_stable_ids_to_section(_inner)'s signature, forcing a redundant copy
per top-level section and hiding the borrow-conflict reasoning from readers. Destructuring into
disjoint borrows eliminates the clone entirely and keeps the &str flow explicit.

      for section in &mut document.sections {
-         assign_stable_ids_to_section(section, &section.title.clone());
+         // Destructuring yields disjoint borrows: title is read independently
+         // of requirements/children, so no defensive clone is required.
+         let PolicySection { title, requirements, children, .. } = section;
+         stamp_requirements(requirements, title, 0);
+         walk_children(children, title.clone(), 0);
      }


══════ F0776 │ src/uuid.rs:260-260 │ [security · low] ══════
[security · low] Trust-model considerations for these identifiers: UUID v5 is built on SHA-1, which
offers no collision resistance, and the seed concatenates attacker-influenced free text (requirement
body, section titles) with bare NUL delimiters. Text/path content containing '\0' makes the tuple
encoding ambiguous (two distinct (text, path) pairs can hash identically), and deliberate collisions
become plausible for a determined adversary given SHA-1's known weaknesses. These IDs are safe for
dedup and change tracking in a cooperative setting, but must never serve as integrity,
authorization, or audit-trust anchors — worth stating explicitly near the namespace docs, and
defensively rejecting input text/titles containing NUL bytes.

+         // UUID v5 = SHA-1 based with NUL-delimited seeds: treat these IDs as
+         // best-effort content tags (dedup/change detection), never as
+         // integrity or authorization anchors.
          let uuid = Uuid::new_v5(&FORGE_NAMESPACE_UUID, hash_input.as_bytes());


══════ F0778 │ src/uuid.rs:269-269 │ [maintainability · low] ══════
[maintainability · low] The generated identifier is immediately downgraded to its string form
(uuid.to_string()), allocating 36 bytes per requirement and erasing the typed-ID distinction in the
model — any consumer needing the raw Uuid must repurchase it. Keeping the typed value end-to-end and
converting to string only at serialization boundaries preserves type safety and avoids per-item
allocation; this does require widening the model's stable_id field type from Option<String> to
Option<Uuid>.

-         requirement.stable_id = Some(uuid.to_string());
+         requirement.stable_id = Some(uuid);
