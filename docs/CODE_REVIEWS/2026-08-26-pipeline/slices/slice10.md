# Validation slice slice10 — 60 findings
Severity mix: low×60


══════ F0500 │ src/mapping/model.rs:330-333 │ [documentation · low] ══════
[documentation · low] In ControlPlusStatement scope the artifact-level GapSummary is derived solely
from controls.unmapped_ids, so unmapped STATEMENTS never reach source_gap_summary/target_gap_summary
— they surface only as Participation.unmapped_ids in the machine report. Automation consuming the
OSCAL artifact (e.g., tooling walking GapSummary/ControlSelection rather than the forge-internal
report) will believe statement coverage is complete.
GapSummary.unmapped_controls/ControlSelection.with_ids naming suggests controls-only was intended;
if so, document that restriction on GapSummary/ControlSelection so the asymmetry is discoverable; if
not intended, derive statement gaps separately (or aggregate both types) for statement-scoped
mappings.


══════ F0499 │ src/mapping/model.rs:610-610 │ [maintainability · low] ══════
[maintainability · low] The UUIDv5 seed kinds are passed as ad-hoc string literals here and at the
callsites ("source-gap-summary" / "target-gap-summary" fed to stable_uuid), while
ensure_unique_uuids independently re-declares the very same literals inside its candidates array.
Renaming or retyping one site silently desynchronizes the other and disables the collision detection
these two functions exist to guarantee. Hoist the kind strings into shared consts used by both
gap_summary callers and ensure_unique_uuids.

- fn gap_summary(kind: &str, mapping_key: &str, ids: &[String]) -> Option<GapSummary> {
+ const SOURCE_GAP_SUMMARY_KIND: &str = "source-gap-summary";
+ const TARGET_GAP_SUMMARY_KIND: &str = "target-gap-summary";
+
+ fn gap_summary(
+     kind: SourceOrTargetGapSummaryKind,
+     mapping_key: &str,
+     ids: &[String],
+ ) -> Option<GapSummary> {


══════ F0519 │ src/migration/engine.rs:22-24 │ [documentation · low] ══════
[documentation · low] `BTreeMap::collect` silently keeps the last index when two items share a
`stable_id`, so this map embeds a hidden intra-inventory uniqueness assumption. It currently holds
only because `inventory::build_inventory` rejects duplicate IDs and `validate_reconciliation`
backstops it at the very end — but with a generic message and after producing a full report's worth
of work. Make the dependency explicit (documented invariant or a cheap debug_assert on collected
keys) so future refactors that bypass `build_inventory` (e.g., new API entry points into `classify`)
fail fast at the source of corruption rather than deep in the cascade.

-         .map(|(index, item)| (item.stable_id.as_str(), index))
+     let mut seen_ids = BTreeSet::new();
+     let new_by_id: BTreeMap<&str, usize> = new
+         .requirements
+         .iter()
+         .enumerate()
+         .map(|(index, item)| {
+             debug_assert!(seen_ids.insert(item.stable_id.as_str()), "caller contract violated: inventory must contain unique stable_ids");
+             (item.stable_id.as_str(), index)
+         })
          .collect();
-     validate_cross_inventory_ids(&old.requirements, &new.requirements, &new_by_id)?;


══════ F0521 │ src/migration/engine.rs:427-431 │ [performance · low] ══════
[performance · low] Unnecessary allocations in the grouping hot path: each cascade stage clones the
full `normalized_text` paragraph (and `section_path`) just to serve as a map key, and the same text
groups are rebuilt in both match_unique_normalized_text and group_ambiguities. Keying by borrowed
slices avoids copying requirement prose for large policy documents; the map never outlives the
borrow of `items` within each pass.

- fn group_unmatched_by<K: Ord>(
-     items: &[InventoryRequirement],
+ fn group_unmatched_by<'a>(
+     items: &'a [InventoryRequirement],
      matched: &[bool],
-     key: impl Fn(&InventoryRequirement) -> K,
- ) -> BTreeMap<K, Vec<usize>> {
+     key: impl Fn(&InventoryRequirement) -> &'a str,
+ ) -> BTreeMap<&'a str, Vec<usize>> {


══════ F0522 │ src/migration/engine.rs:434-434 │ [style · low] ══════
[style · low] `group_unmatched_by` uses the legacy `or_insert_with(Vec::new)` push idiom; the modern
equivalent is shorter and equally readable (`Vec` implements `FromIterator`, so `or_default()`
works), matching newer house style used elsewhere in this repo's aggregations.

-         groups.entry(key(item)).or_insert_with(Vec::new).push(index);
+         groups.entry(key(item)).or_default().push(index);


══════ F0526 │ src/migration/formatter.rs:21-22 │ [maintainability · low] ══════
[maintainability · low] format_text and its helpers discard every writeln! Result via `let _`. This
is currently sound because `impl fmt::Write for String` is infallible, but nothing records that
reasoning, and any future change of the sink to an `io::Write` target (file, pipe, terminal with
partial writes) would silently hide truncation/errors and ship incomplete reports. Capture the
assumption once in a small local emit helper (or a comment where the pattern starts) so a sink swap
forces a conscious review of all these sites.

+     // Writing to a String never fails (`fmt::Write for String` is infallible);
+     // deliberate `let _` — revisit all discard sites if this ever renders to an io sink.
      let mut output = String::new();
      let _ = writeln!(output, "FORGE policy migration report");


══════ F0525 │ src/migration/formatter.rs:67-69 │ [performance · low] ══════
[performance · low] The evidence list is collected into an intermediate Vec<&str> solely to call
join(","). Impact is bounded because evidence codes come from a small closed enum (<= 10 codes per
entry), but the collection is still unnecessary: stream the separator between elements straight into
the output buffer alongside the neighbouring writeln! calls.

-         let evidence =
-             entry.evidence.iter().map(|evidence| evidence.as_str()).collect::<Vec<_>>().join(",");
-         let _ = writeln!(output, "  evidence: {evidence}");
+         let _ = write!(output, "  evidence: ");
+         for (index, evidence) in entry.evidence.iter().enumerate() {
+             if index > 0 {
+                 let _ = output.write_char(',');
+             }
+             let _ = output.write_str(evidence.as_str());
+         }
+         let _ = writeln!(output);


══════ F0502 │ src/migration/inventory.rs:18-18 │ [performance · low] ══════
[performance · low] `input_format` is checked only after the entire document has been ingested,
parsed, atomized, and annotated. An unsupported extension therefore pays the full pipeline cost
(potentially seconds for large PDFs/DOCX files) before failing immediately. Validate the extension
before calling `prepare_document` so invalid inputs fail fast, keeping the expensive work on the
happy path.

      let format = input_format(path)?;
+     let document = crate::pipeline::prepare_document(path, max_size_bytes)?;


══════ F0514 │ src/migration/inventory.rs:39-40 │ [maintainability · low] ══════
[maintainability · low] Correctness of validate_unique_ids depends on an invariant that lives two
statements away: it only detects duplicates because requirements were sorted by stable_id on the
previous line. If the sort is ever removed, moved after validation, or keyed differently, this check
silently degrades to detecting only adjacent duplicates without any compile-time or runtime signal.
Add a debug assertion that the input is sorted (or derive the uniqueness check from a
HashMap/HashSet) so the correctness of the validation does not hinge on call-site ordering.

      requirements.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
+     debug_assert!(
+         requirements.windows(2).all(|pair| pair[0].stable_id <= pair[1].stable_id),
+         "validate_unique_ids requires requirements sorted by stable_id"
+     );
      validate_unique_ids(&requirements)?;


══════ F0523 │ src/migration/inventory.rs:55-57 │ [other · low] ══════
[other · low] This rolls the unsupported-extension failure into the free-form MigrationError variant
even though src/error.rs already defines a dedicated, user-facing ForgeError::UnsupportedFormat {
extension } variant with tailored remediation guidance. Using the ad hoc string bypasses that typed
classification (exit-code mapping, matching, consistent messaging) for exactly the case it exists
for; map the fallthrough arm to ForgeError::UnsupportedFormat instead (updating the shared variant's
help text to cover .pdf/.docx as supported inputs).

          extension => {
-             Err(ForgeError::MigrationError(format!("unsupported policy format '.{extension}'")))
+             Err(ForgeError::UnsupportedFormat { extension: extension.to_string() })
          }


══════ F0517 │ src/migration/inventory.rs:78-78 │ [bug · low] ══════
[bug · low] Section paths are joined with '/' and titles are never escaped, so a title containing a
slash (e.g. 'Access Control / Audit') produces a section_path string that is indistinguishable from
the equivalent real nesting hierarchy ('Access Control' with child 'Audit'). Downstream consumers
cannot unambiguously map a recorded section_path back to one unique location in the document,
weakening the traceability this inventory exists to provide. Either escape/encode separator
characters in titles when building paths, or record structured parent links/heading levels rather
than a single ambiguous flattened string.


══════ F0528 │ src/migration/mod.rs:39-39 │ [maintainability · low] ══════
[maintainability · low] This delegation point owns the knowledge of *which* successor-map file was
requested (the CLI-resolved path), yet forwards no context with a load failure. Whether the
resulting MigrationError identifies the offending file is entirely up to successor::load's ad-hoc
messages today — its open/read failures do embed `path.display()`, but the hardening failures
('successor map must not be a symbolic link', 'must be a regular file', the byte-limit errors) and
the JSON diagnostics do not, leaving users unable to tell which configured map failed. Adding the
path context here makes the public entry point self-sufficient instead of depending on a callee's
incidental message strings.

-     let successor_map = successor_map_path.map(successor::load).transpose()?;
+     let successor_map = successor_map_path
+         .map(|map_path| {
+             successor::load(map_path).map_err(|error| {
+                 ForgeError::MigrationError(format!(
+                     "while loading successor map '{}': {error}",
+                     map_path.display()
+                 ))
+             })
+         })
+         .transpose()?;


══════ F0529 │ src/migration/mod.rs:6-6 │ [maintainability · low] ══════
[maintainability · low] `pub mod successor` punches a hole in the otherwise-curated facade: every
other submodule is private with its public items hand-picked for re-export above, while successor
exposes its whole namespace (including lower-level helpers like `parse`) directly as
`migration::successor::*`. This freezes a wider crate boundary than the facade documents, creates
two canonical paths for the same types (`crate::migration::successor::SuccessorMap` vs. the
re-exported types), and makes future refactors of successor a semver-relevant change even though
this module presents itself as the migration API. Make it private and re-export exactly the items
that belong to the public contract, mirroring the formatter/types pattern.

- pub mod successor;
+ mod successor;


══════ F0559 │ src/migration/successor.rs:182-183 │ [other · low] ══════
[other · low] The parsed RFC 3339 instant is parsed only to be thrown away: approved_at remains a
bare String on the public SuccessorRelationship, so the timestamp-validity invariant lives solely in
validate() and every consumer of the struct must continue to treat the field as untrusted prose.
Deserializing the field as chrono::DateTime<FixedOffset> (which implements Serialize/Deserialize) —
or wrapping approved_by/approved_at/schema_version in validating newtypes — would move the invariant
into the type system and eliminate the parse-then-discard step.


══════ F0557 │ src/migration/successor.rs:222-225 │ [security · low] ══════
[security · low] Identifiers are accepted verbatim: leading/trailing whitespace survives validation,
so "old-a" and " old-a" coexist as distinct, visually indistinguishable entries in reviewer-approved
evidence, and they can diverge from how real migrated IDs resolve downstream. Since this file is the
trust boundary for approval records, either reject non-canonical identifiers (where id.trim() != id,
or containing control characters) at validation time, or apply one consistent canonicalization to
every comparison key, mirroring the trim discipline already applied to prose fields.


══════ F0558 │ src/migration/successor.rs:92-97 │ [maintainability · low] ══════
[maintainability · low] These two rejection messages are the only diagnostics in the module that
omit the offending path, unlike every neighboring format!("... '{}' ...", path.display()) message.
More broadly, all IO/inspection failures are Display-flattened into
ForgeError::MigrationError(String), so operators cannot programmatically distinguish a symlink-swap
race (ELOOP), a permission failure, or a vanished path except by pattern-matching message text. At
minimum thread the path into these messages; a structured variant carrying the io::Error source
would make the distinctions machine-readable.

      if metadata.file_type().is_symlink() {
-         return Err(error("successor map must not be a symbolic link"));
+         return Err(error(format!(
+             "successor map '{}' must not be a symbolic link",
+             path.display()
+         )));
      }
      if !metadata.is_file() {
-         return Err(error("successor map must be a regular file"));
+         return Err(error(format!(
+             "successor map '{}' must be a regular file",
+             path.display()
+         )));
      }


══════ F0578 │ src/model/assemble.rs:120-124 │ [performance · low] ══════
[performance · low] Potential O(sections x items) scanning: for each sibling node, items_in_range
linearly rescans the entire list_items slice, and this repeats recursively at every level.
clfileauses.list_items arrives sorted by source_line (parse/clauses.rs sorts it), so each partition
can use binary search (slice::partition_point) to make each level O(log n + k) instead of O(n),
which matters for large policies with many headings and items.

-         let items_in_range: Vec<&ExtractedListItem> = list_items
-             .iter()
-             .copied()
-             .filter(|item| item.source_line >= range_start && item.source_line < range_end)
-             .collect();
+         // list_items is sorted by source_line (parse/clauses.rs), so a
+         // binary-searched subslice beats a full linear filter per node.
+         let lo = list_items.partition_point(|item| item.source_line < range_start);
+         let hi = list_items.partition_point(|item| item.source_line < range_end);
+         let items_in_range: &[&ExtractedListItem] = &list_items[lo..hi];


══════ F0580 │ src/model/assemble.rs:169-170 │ [style · low] ══════
[style · low] usize::MAX as an open-ended sentinel end is fragile: arithmetic or comparisons against
a cast sentinel (e.g. (end - 1)) would silently wrap, and future changes to line-number types (u32)
make MAX-bound assumptions easy to break. Consider modeling the optional upper bound explicitly,
e.g. Range<usize> with an Option<u32> or a small struct carrying Option<end>, letting the type
system carry the 'unbounded' state instead of a magic value.


══════ F0579 │ src/model/assemble.rs:181-183 │ [maintainability · low] ══════
[maintainability · low] Shared collision-prone fallback: non-UTF-8 or stem-less paths collapse onto
the constant string "untitled", which assemble_document uses both as doc.id and as the
metadata.title fallback. Two distinct files (e.g. '/' and './untitled.md') then share the same
document identity and lose provenance with no signal that a fallback occurred. Consider falling back
to the content hash/fingerprint for id uniqueness, or at least emitting a warning that the true
filename was unrecoverable.


══════ F0577 │ src/model/assemble.rs:75-75 │ [maintainability · low] ══════
[maintainability · low] Asymmetric handling of leading/orphaned lines: when section_nodes is
non-empty, items before the first heading become a Preamble; when section_nodes is empty, ALL items
become Preamble regardless of position, and the empty-items case silently returns Vec::new()
instead. Out-of-range corner cases (items sorted oddly relative to headings) therefore hit different
code paths depending on whether any heading exists at all. If this is intentional for the
heading-less-document case, document the rule; otherwise unify through one Preamble-building path.

+     // Rule: items preceding the first heading attach to a synthetic Preamble;
+     // a document with no headings at all is entirely Preamble (handled above).
      if !preamble_items.is_empty() {


══════ F0572 │ src/model/frontmatter.rs:48-48 │ [maintainability · low] ══════
[maintainability · low] Malformed YAML and genuinely-absent frontmatter collapse into the same None
return (only a tracing log distinguishes them). Callers such as assemble_document cannot tell a
correctable authoring mistake from intentional absence, which can hide policy-metadata bugs.
Consider returning a small enum (Frontmatter::Absent / Malformed(String)) or a Result<Option<..>,
String> so upstream can surface a targeted warning while defaulting gracefully.

+ /// Outcome kind: Absent (no fence) vs Malformed (fence found but YAML invalid).
+ /// Returning the distinction lets assemble_document warn accurately instead of
+ /// treating author errors as intentional absence.
  pub(crate) fn parse_frontmatter(content: &str) -> Option<FrontmatterData> {


══════ F0570 │ src/model/frontmatter.rs:62-63 │ [bug · low] ══════
[bug · low] When the closer is matched via the "\r\n" branches, end points just before "\r\n", so
the last field's value keeps a trailing '\r' (yaml_str like "title: Example\r") and, once the
CRLF-opener fix above lands, interior lines would too. If a string-typed value is expected verbatim
(documented as ISO-8601 dates etc.), a stray CR corrupts stored metadata. Strip the residual CR (or
normalize CRLF to LF for the YAML slice) before deserializing.

          .or_else(|| rest.strip_suffix("\r\n---").map(str::len))?;
-     let yaml_str = &rest[..end];
+     // Remove any CR leftover from CRLF delimiters so serde_yaml sees clean LF text.
+     let raw = &rest[..end];
+     let yaml_str = raw.strip_suffix('\r').unwrap_or(raw);


══════ F0554 │ src/model/mod.rs:121-124 │ [maintainability · low] ══════
[maintainability · low] Progressive-enrichment placeholders (`stable_id` until WI-7, `citations`
until WI-8, `modality` until WI-33, `parameters` until WI-34) leave partial states representable
with no compile-time protection, and field interlocks are order-coupled:
`PolicyParameter.requirement_id` must hold an already-populated WI-7 value even though `stable_id`
is declared optional here. Since stages pass ownership sequentially, consider (a) typestate/wrapper
types per phase (e.g., `AnnotatedRequirement`) or simpler: (b) `debug_assert!` postconditions
exported from this module — e.g., an `invariants::assert_fully_enriched(&PolicyDocument)` that
enrich passes invoke — so running passes out of order or dropping one fails fast in CI instead of
silently emitting incomplete OSCAL.

-     /// Stable UUID for this requirement.
-     /// - `None` until populated by WI-7 (UUID generation)
-     /// - `Some(uuid)` after WI-7
-     pub stable_id: Option<String>,
+ /// Debug-checked postconditions for the enrichment pipeline.
+ pub mod invariants {
+     use super::*;
+
+     /// Panics under `debug_assertions` if any requirement lacks a stable_id,
+     /// modality annotation, or contains a dangling parameter/citation link.
+     #[track_caller]
+     pub fn assert_fully_enriched(doc: &PolicyDocument) {
+         // recursively walk sections/requirements and validate documented
+         // post-WI-7/WI-33/WI-34 invariants before OSCAL generation.
+     }
+ }


══════ F0553 │ src/model/mod.rs:99-100 │ [maintainability · low] ══════
[maintainability · low] `heading_level: u8` accepts values outside 1..=6 — the range is
documentation-only, so malformed parses flow unchecked into OSCAL control mapping (H1..H6 determine
nesting there). Encode the constraint in the type: introduce `HeadingLevel(u8)` with `TryFrom<u8>`
validating `1..=6` (and derive needed traits), or at minimum validate/normalize in
`assemble_document` so the model invariant holds for every consumer.

-     /// Heading level: 1 for H1, 2 for H2, ..., 6 for H6.
-     pub heading_level: u8,
+ /// Validated heading level (1..=6).
+ #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
+ pub struct HeadingLevel(u8);
+
+ impl TryFrom<u8> for HeadingLevel {
+     type Error = HeadingLevelRangeError;
+     fn try_from(v: u8) -> Result<Self, Self::Error> {
+         if (1..=6).contains(&v) {
+             Ok(Self(v))
+         } else {
+             Err(HeadingLevelRangeError(v))
+         }
+     }
+ }


══════ F0547 │ src/model/trace.rs:107-109 │ [maintainability · low] ══════
[maintainability · low] `by_oscal_element` dereferences a stored position with `self.links[idx]`,
which panics if the dual-store invariant is ever violated, and the three writes in `record` are only
atomic by convention. Nothing compile-time prevents a future `remove`/`retain`/reorder method from
desynchronizing the indexes (stale positions → wrong link or panic; stale grouped clones → divergent
`by_requirement` results). Encode the constraint explicitly: either forbid mutation APIs, replace
raw indexing with checked access that surfaces the violation, or at minimum record the invariant on
the struct so maintainers know `record` must remain the sole writer.

+     /// # Panics
+     ///
+     /// Never panics under the documented invariant: `links` is strictly
+     /// append-only via `record` (the sole mutation path), so every position
+     /// stored in `by_oscal_element` remains valid.
      pub fn by_oscal_element(&self, element_id: &str) -> Option<&TraceLink> {
          self.by_oscal_element.get(element_id).map(|&idx| &self.links[idx])
      }


══════ F0546 │ src/model/trace.rs:25-26 │ [style · low] ══════
[style · low] `TraceLink` omits the `PartialEq`/`Eq` derives its embedded `SourceLocation` has,
making equality assertions asymmetric across the module's public types — the round-trip test below
has to compare four fields individually and will silently miss new fields added later unless
updated. Both traits are derivable here (all fields are `Eq`). Also consider deriving `Hash` on the
ID fields via newtypes if these IDs become map keys elsewhere.

- #[derive(Debug, Clone, Serialize, Deserialize)]
+ #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct TraceLink {


══════ F0548 │ src/model/trace.rs:75-76 │ [documentation · low] ══════
[documentation · low] The uniqueness constraint applied here covers only `oscal_element_id`;
duplicate `oscal_json_path` values and repeated records for the same requirement are silently
accepted. That appears deliberate (the one-to-one guarantee is stated only on the element-ID field
and exercised by `collection_by_requirement_multiple_links`), but spell it out in the method docs so
callers don't assume full-link deduplication.

      /// Returns [`TraceError::DuplicateElement`] if `oscal_element_id` is already recorded.
+     ///
+     /// Only `oscal_element_id` participates in uniqueness checking;
+     /// duplicate `oscal_json_path` values and multiple links sharing one
+     /// requirement stable ID are accepted intentionally.
      pub fn record(&mut self, link: TraceLink) -> Result<(), TraceError> {


══════ F0566 │ src/oscal/assessment_plan.rs:112-113 │ [documentation · low] ══════
[documentation · low] Doc comment claims the title truncates to the "first 80 characters" of the
requirement text, but `assessment_task_title` keeps 77 characters and appends "..." (total title
length can exceed 88 characters including the "Assess: " prefix). Align the doc with the actual
limit to avoid misleading API consumers sizing the title.

-     /// Human-readable task title (truncated to first 80 characters of the requirement text).
+     /// Human-readable task title (`"Assess: "` plus up to 77 chars of the requirement
+     /// text, suffixed with `"..."` when truncated).
      pub title: String,


══════ F0567 │ src/oscal/assessment_plan.rs:168-170 │ [maintainability · low] ══════
[maintainability · low] The public type permits `include_all` and `include_subjects` to be set
simultaneously, which represents a state OSCAL rejects (a selection must use either `include-all` or
`include-subjects`). `create_assessment_subjects` enforces exclusivity internally, but any other
constructor or hand-built value can emit an invalid document without detection. Model this as an
enum (`IncludeAll | Subjects(Vec<SubjectRef>)`) or validate before writing.


══════ F0563 │ src/oscal/assessment_plan.rs:232-233 │ [maintainability · low] ══════
[maintainability · low] Mapping the `assemble_metadata` error straight into `e.to_string()` discards
the original error's variant and structured context at a boundary where callers benefit most from
actionable diagnostics. At minimum, label the operation in the message; ideally preserve the source
error (`#[source]`/`Error::source`) if `ForgeError` supports chaining.

-     let real_metadata = crate::oscal::assemble_metadata(&doc_meta, None)
-         .map_err(|e| ForgeError::AssessmentPlanBuild(e.to_string()))?;
+     let real_metadata = crate::oscal::assemble_metadata(&doc_meta, None).map_err(|e| {
+         ForgeError::AssessmentPlanBuild(format!("assemble_metadata failed: {e}"))
+     })?;


══════ F0565 │ src/oscal/assessment_plan.rs:349-353 │ [maintainability · low] ══════
[maintainability · low] Only the task `description` guards against empty/whitespace-only requirement
text; the linked activity's `title` and `description` interpolate `req.text` unguarded, so an empty
requirement yields `"Review: "` and `"Examine evidence that  is implemented and operating
effectively."` — inconsistent placeholders in the same generated document. Apply the same trim-guard
used for the task description.

-                 title: format!("Review: {}", req.text.chars().take(60).collect::<String>()),
-                 description: format!(
+                 title: format!(
+                     "Review: {}",
+                     if req.text.trim().is_empty() {
+                         "(empty requirement)".to_string()
+                     } else {
+                         req.text.chars().take(60).collect::<String>()
+                     }
+                 ),
+                 description: if req.text.trim().is_empty() {
+                     "No examination guidance available — requirement text is empty.".to_string()
+                 } else {
+                     format!(
                      "Examine evidence that {} is implemented and operating effectively.",
                      req.text
-                 ),
+                     )
+                 },


══════ F0621 │ src/oscal/back_matter.rs:161-163 │ [maintainability · low] ══════
[maintainability · low] The href (and therefore the UUID hash input) is built from the raw, unparsed
string even though parsing succeeded: a citation URL like `"  https://nist.gov  "` validates as
http/https but produces an href with literal surrounding spaces and a UUID differing from the
canonical form. The `unwrap_or_default()` also masks a proven invariant (the `Some(url)` existence
is guaranteed by the `Valid` variant) and would silently emit an empty href if the invariant were
ever broken. Use the canonicalized URL returned by the classifier.

          UrlClassification::Valid(parsed_url) => {
              let media_type = infer_media_type(&parsed_url);
-             let href = citation.url.as_deref().unwrap_or_default().to_string();
+             // Canonical form: parse success guarantees scheme validation and
+             // strips any surrounding whitespace the raw string may carry.
+             let href = parsed_url.to_string();


══════ F0607 │ src/oscal/component_definition.rs:159-160 │ [bug · low] ══════
[bug · low] The identity anchor hashes the *defaulted* title and version: an untitled document
inherits `"Untitled Policy Document"` and an unversioned one `"0.0.0"`, so unrelated
untitled/unversioned policies collide into identical component UUIDs as soon as their document ids
coincide — undermining the stated goal that the UUID stays unique across source documents. Derive
the UUID from the raw values (falling back to the raw empty marker internally) or incorporate
another guaranteed-distinct input (e.g., content hash from `metadata.content_hash`) so defaulting
cannot forge accidental identity equality.


══════ F0603 │ src/oscal/component_definition.rs:272-274 │ [bug · low] ══════
[bug · low] When the citation budget is exhausted mid-section, this bare `return` silently aborts
collection for ALL remaining citations and, importantly, skips traversing this section's `children`
entirely — not just the remaining citations of the current request. The result is silently truncated
back matter with no error/warning returned to the caller (the top-of-function warning only fires
again per remaining section, producing log spam). Consider signalling truncation (e.g., return the
cap-hit status or emit one aggregate warning) and using `break` scoped to the loops plus explicit
child-traversal continuation.


══════ F0604 │ src/oscal/component_definition.rs:275-277 │ [performance · low] ══════
[performance · low] `seen.insert(citation.id.clone())` allocates a fresh `String` for every citation
encountered, including duplicates that are immediately discarded. Check membership before cloning so
the allocation only happens for genuinely new citation ids.

-             if seen.insert(citation.id.clone()) {
+             if !seen.contains(citation.id.as_str()) {
+                 seen.insert(citation.id.clone());
                  citations.push(citation.clone());
              }


══════ F0587 │ src/oscal/implemented_requirements.rs:220-220 │ [documentation · low] ══════
[documentation · low] Doc comment contradicts the implementation below: it claims the fallback is
`REQ-{zero-padded global_index}`, but the code emits `REQ-{:03}` of `global_index + 1` (1-based
display, matching `REQ-001` for index 0 per the T020 tests). Update the doc so consumers scripting
against the format don't off-by-one, and note that `{:03}` is a minimum width so values past 999
render as four digits (e.g., `REQ-1000`).

- /// Fallback (EC-2): `REQ-{zero-padded global_index}` when `stable_id` is `None`.
+ /// Fallback (EC-2): `REQ-{NNN}` where NNN is the 1-based global position
+ /// (i.e., `global_index + 1`, zero-padded to 3 digits minimum) when `stable_id` is `None`.


══════ F0590 │ src/oscal/implemented_requirements.rs:227-232 │ [maintainability · low] ══════
[maintainability · low] Passing `has_stable_id: bool` alongside indices discards information the
caller already had (`req.stable_id.is_some()` is recomputed here just to feed it back), the exact
primitive-boolean anti-pattern the API guidelines call out: the invalid state (wanting a fallback
for a present ID) is representable. Take `stable_id: Option<&str>` (or the requirement itself) so
the compiler pins the relationship, and drop `global_index`'s dual role of "fallback numbering" vs
"warning payload" from this API.

  fn derive_control_id_or_fallback(
      abbreviation: &str,
      req_index_in_section: usize,
-     global_index: usize,
-     has_stable_id: bool,
+     stable_id: Option<&str>,
  ) -> String {
+     match stable_id {
+         Some(_) => generate_control_id(abbreviation, req_index_in_section, "POL"),
+         None => /* log + REQ fallback */ todo!(),
+     }
+ }


══════ F0584 │ src/oscal/mod.rs:46-47 │ [other · low] ══════
[other · low] The curated namespace re-exports `metadata::OscalMetadata` here, but `catalog.rs` also
defines a separate, differently-shaped `OscalMetadata` (a placeholder with
title/last_modified/version fields). Flattening exactly one of the two same-named types makes
`crate::OscalMetadata` ambiguous to consumers — someone building a catalog envelope will naturally
reach for the wrong struct and get serialize/deserialize mismatches. Document the intended canonical
type (or disambiguate/rename the placeholder) so the public API exposes one authoritative
`OscalMetadata` per artifact kind.

  /// Metadata types and assembly function.
+ ///
+ /// Note: this is the canonical [`crate::oscal::metadata::OscalMetadata`] used by
+ /// `assemble_metadata`; [`crate::oscal::catalog::OscalMetadata`] is an internal
+ /// placeholder shape and is intentionally NOT part of the public API.
  pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};


══════ F0596 │ src/oscal/profile.rs:237-240 │ [maintainability · low] ══════
[maintainability · low] The doc comment states `mode` is "Ignored when `control_ids` is empty" (C-2
modify-only case), but `mode_str` is unconditionally folded into the UUID seed. Two functionally
identical modify-only profiles built with different `--include` vs `--exclude` flags therefore get
different UUIDs despite producing byte-identical imports. To match the documented contract, only
include `mode_str` in the seed when `!control_ids.is_empty()`.

+     // Only fold mode into the identity when controls are selected;
+     // modify-only profiles must be independent of an unused flag.
+     if !control_ids.is_empty() {
      let mode_str = match mode {
          SelectionMode::Include => "include",
          SelectionMode::Exclude => "exclude",
      };
+         seed_parts.push(mode_str);
+     }


══════ F0600 │ src/oscal/profile.rs:242-243 │ [performance · low] ══════
[performance · low] Unnecessary `.clone()` of the entire ID vector just to compute the sorted seed —
`control_ids` is consumed later and could instead be sorted/deduped once up front, then reused for
both the seed and the emitted `with_ids`, eliminating the duplication and making the sort-order
decision explicit. Alternatively reserve capacity and reuse the original vec, dropping the clone.


══════ F0597 │ src/oscal/profile.rs:256-259 │ [maintainability · low] ══════
[maintainability · low] Determinism inconsistency: the UUID sorts `control_ids`, but the emitted
JSON preserves caller order in `with_ids`. So two invocations differing only in ID ordering yield
(a) the SAME UUID with DIFFERENT JSON bodies (if callers pass distinct orderings), which breaks
UUID-as-content-checksum semantics, while conversely `build_modify_section` guarantees sorted param
output. Pick one convention and document it — ideally canonicalize the with_ids order (sort,
consistent with dedup done case-sensitively upstream) so equal UUID ⟺ equal body.

+     // Canonicalize emission order to match UUID derivation:
+     // sorted_ids was already sorted+seeded above, use it here.
      let imports = if control_ids.is_empty() {
          vec![]
      } else {
-         let selection = ControlSelection { with_ids: control_ids };
+         let selection = ControlSelection { with_ids: sorted_ids };


══════ F0599 │ src/oscal/profile.rs:308-311 │ [security · low] ══════
[security · low] `parse_control_ids` accepts any non-empty comma token without validating the
expected control-ID charset/format, and these raw strings are embedded verbatim in the generated
JSON artifact and hashed into the profile UUID seed. Near-duplicate variants like "ac-1" vs "AC-1"
silently produce two distinct controls, corrupting include/exclude selection. Validate tokens (e.g.,
`[A-Za-z0-9._-]+` pattern matching OSCAL control identifiers) and reject/normalize otherwise.

          .map(|s| s.trim().to_string())
          .filter(|s| !s.is_empty())
          .filter(|s| seen.insert(s.clone()))
-         .collect();
+         .map(|id| {
+             if id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
+                 Ok(id)
+             } else {
+                 Err(ForgeError::InvalidArgument(format!(
+                     "Invalid control ID {id:?}: expected [A-Za-z0-9._-] characters"
+                 )))
+             }
+         })
+         .collect::<Result<Vec<_>, _>>()?;


══════ F0598 │ src/oscal/profile.rs:65-66 │ [documentation · low] ══════
[documentation · low] Documentation claims the href is "stored as-is from --catalog", but
build_profile passes it through `crate::io::sanitize_artifact_path`, which strips directories and
stores only the bare filename (see test `profile_import_href_uses_filename_only`). Consumers relying
on the original path/relative-reference (e.g., resolvers that expect the catalog next to the
profile) will be misled. Update the doc to state a sanitized filename is used, or preserve a
lossless reference field.


══════ F0630 │ src/oscal/ssp.rs:239-244 │ [maintainability · low] ══════
[maintainability · low] Enumerated domains are modeled as free-form strings throughout this module:
`component_type` ("software"/"hardware"/"service"/"network"), `ComponentStatus.state`
("operational", "under-development", "disposition"), `ImplementationStatus.state`
("planned"/"implemented"/"partial"/"not-applicable"), and `UserStatus.state`
("active"/"inactive"/"pending"/"disabled"). Invalid states are representable and will only be caught
by external OSCAL validation far downstream. Model these with enums (with `AsRef<str>`/`Serialize`
to keep the JSON wire format) or validated newtypes.


══════ F0626 │ src/oscal/ssp.rs:588-589 │ [maintainability · low] ══════
[maintainability · low] `assemble_metadata` failures are flattened with `e.to_string()` at the
earliest boundary, discarding the original error type and its source chain; callers of `build_ssp`
cannot match on the underlying cause and logs lose structured context. Preserve the source error
(e.g., give `ForgeError::SspBuild` a `#[source]`/boxed inner error) and only add SSP-specific
context.

-     let real_metadata =
-         assemble_metadata(&doc_meta, None).map_err(|e| ForgeError::SspBuild(e.to_string()))?;
+     let real_metadata = assemble_metadata(&doc_meta, None)
+         .map_err(|e| ForgeError::SspBuildWithSource { context: "metadata assembly".into(), source: e })?;


══════ F0633 │ src/oscal/trace_embedding.rs:46-46 │ [performance · low] ══════
[performance · low] The DoS/allocation guard measures the *pre-encoding* length, but the encoder
expands each `%`, space, or `#` from 1 byte to 3, so a 4096-byte adversarial path yields up to ~12
KB hrefs — the bound this constant documents ("Maximum file path length accepted for href encoding")
is not actually enforced on the produced string. Additionally, the truncation happens silently: the
output link points to a clipped file name that no longer resolves, with no log record. Cap (and
preferably `tracing::warn!` about) the encoded result.

-     let safe_path = if path.len() > MAX_HREF_PATH_LENGTH {
+     let mut encoded = safe_path.replace('%', "%25").replace(' ', "%20").replace('#', "%23");
+     if encoded.len() > MAX_HREF_PATH_LENGTH {
+         tracing::warn!(original_len = encoded.len(), "Encoded href exceeded {} bytes; truncating", MAX_HREF_PATH_LENGTH);
+         encoded.truncate(MAX_HREF_PATH_LENGTH);
+     }


══════ F0636 │ src/oscal/trace_embedding.rs:62-67 │ [documentation · low] ══════
[documentation · low] `build_trace_props` and `build_trace_link` are public and take arbitrary
`&str` paths, but the SEC-1 policy (strip directories to prevent absolute-path leakage into
published OSCAL output) is enforced only inside `embed_trace_in_catalog`. Other public call sites —
e.g. the component-definition pipeline in `implemented_requirements.rs`, which the inline comment
itself relies on being handled 'at pipeline.rs:205' — must remember to apply the same stripping or
they leak host filesystem paths into released documents. Document the precondition on these public
builders (callers MUST pass filename-only paths), or move the `file_name()` normalization into them
so the invariant cannot be violated.

+ /// # Security
+ /// Callers MUST pass a bare file name, not a full filesystem path:
+ /// absolute paths are embedded verbatim and leak host layout into
+ /// published OSCAL artifacts (see SEC-1). Prefer normalizing with
+ /// [`std::path::Path::file_name`] before calling, or centralize that
+ /// normalization here so no call site can forget it.
  #[must_use]
  pub fn build_trace_props(
      source_file: &str,
      section_title: &str,
      line_number: usize,
  ) -> Vec<OscalProp> {


══════ F0650 │ src/oscal_cli/invoker.rs:25-29 │ [other · low] ══════
[other · low] Allowlisted variables that are unset (or non-UTF-8, which `std::env::var` reports as
an error) are silently dropped, so an incomplete environment (e.g., missing JAVA_HOME, or a
stripped-down CI container) fails later with an opaque JVM startup error inside oscal-cli stderr.
Since rustc does NOT emit metadata/env-var sections at link time (only objcopy/LLVM tools strip
sections), consider a tracing::trace/debug line recording which allowlist entries were found versus
absent — it costs nothing and converts a confusing downstream failure into an actionable hint.


══════ F0645 │ src/oscal_cli/invoker.rs:65-70 │ [bug · low] ══════
[bug · low] `read_to_string` fails entirely on invalid UTF-8; because the resulting io::Error is
discarded, the thread yields an empty string. Non-UTF-8 bytes anywhere in stderr (plausible with
unusual locales/file paths echoed by the JVM) therefore truncate ALL stderr, degrading both
`warnings` and `extract_error_message` for otherwise-successful runs. Prefer a lossy byte-level read
(e.g., BufReader + read_until chunks appended via String::from_utf8_lossy), which also composes with
the control-char sanitization already applied afterwards.


══════ F0651 │ src/oscal_cli/invoker.rs:84-85 │ [performance · low] ══════
[performance · low] This helper blocks the caller's thread: 100ms sleeps in a poll loop plus a
full-duration blocking `JoinHandle::join` and blocking `std::process` usage. That is appropriate for
today's synchronous CLI flows (the OscalCliInvoke trait is used from command handlers), but it will
stall a tokio worker if ever invoked from async context; if that integration arrives, wrap in
`spawn_blocking` or move to a waitable-primitive-based approach. Consider at minimum documenting the
synchronous/blocking contract on the trait method's rustdoc. (Side note: `kill()` terminates the JVM
itself; any child processes it spawned may survive on Unix unless a process group is used.)


══════ F0653 │ src/oscal_cli/mod.rs:38-39 │ [security · low] ══════
[security · low] "Canonicalized absolute path" is documented purely as a convention on these raw
PathBuf fields — nothing in the type enforces it, and every value ultimately lands as a subprocess
argument for oscal-cli. If any caller-supplied or config/plugin-derived path were to start with `-`,
the child process could parse it as a command-line flag rather than a filename (option injection),
e.g. hijacking `--to=`/output-related flags. Enforce validation at the boundary (a TryFrom/factory
constructor that rejects relative or dash-prefixed paths), or at minimum extend the contract docs so
implementers of OscalCliInvoke know they must invoke oscal-cli with strictly positional arguments
separated by `--` and never route paths through a shell.

      /// Canonicalized absolute path to input Profile JSON.
+     ///
+     /// # Implementer contract
+     /// Implementations of [`OscalCliInvoke`] must pass this as a strictly
+     /// positional argument after a `--` separator (never through a shell) so
+     /// filenames beginning with `-` cannot be parsed as oscal-cli flags.
+     /// Prefer constructing values via [`ResolveArgs::try_new`], which rejects
+     /// non-canonical paths.
      pub profile_path: PathBuf,


══════ F0654 │ src/oscal_cli/mod.rs:89-90 │ [documentation · low] ══════
[documentation · low] The doc comment promises a "(default: 30 seconds)" timeout, but no such
default exists: `ConvertArgs` has no `Default` impl (verified project-wide — the only `Default`
impls are for PathDetector, OscalComparisonRules, and ConversionStatistics), and all production call
sites (e.g., round_trip/chain.rs) supply their own explicit Duration, so every caller invents its
own constant. Either introduce a named constant (and reference it from the doc so the "default" is a
single source of truth), or drop the misleading "default" wording.

-     /// Per-invocation timeout (default: 30 seconds).
+     /// Per-invocation timeout.
+     ///
+     /// There is no built-in default; callers should reuse
+     /// [`DEFAULT_CONVERT_TIMEOUT`] unless they have a reason not to.
      pub timeout: Duration,


══════ F0661 │ src/parameter/matchers.rs:208-210 │ [bug · low] ══════
[bug · low] The open-ended \w+ unit lets QuantityMatcher claim duration phrasing and win the
caller's longer-match tiebreak: "Records retained for at least 90 days" produces TimeWindow=nothing
(TIME_WINDOW needs within/after/every), Quantity='3 factors'-style match over 'at least 90 days',
discarding the shorter THRESHOLD_MIN span. The published PolicyParameter is therefore typed Quantity
with Minimum '90' and a 'days' unit baked only into the label — the AR-034 'longer match wins' rule
alone cannot recover the correct TimeWindow classification here. Restrict the unit alternation to
count nouns (factor(s)|generation(s)|token(s)... ) or classify recognized time units
(day(s)|week(s)|month(s)|year(s)) as ParameterType::TimeWindow so durations keep their semantic
type.

+ // Count nouns only — duration units belong to TimeWindowMatcher.
  Regex::new(
-     r"(?i)(?P<qualifier>at\s+least|no\s+fewer\s+than|minimum)\s+(?P<value>\d+)\s+(?P<unit>\w+)",
+     r"(?i)(?P<qualifier>\bat\s+least|\bno\s+fewer\s+than|\bminimum)\s+(?P<value>\d+)\s+(?P<unit>factors?|generations?|tokens?|copies|roles?|accounts?)",
  )


══════ F0662 │ src/parameter/matchers.rs:63-66 │ [maintainability · low] ══════
[maintainability · low] These expect() calls encode a fragile cross-cutting invariant: every named
group in the regex statics must remain mandatory, otherwise ordinary non-matching text turns into a
runtime panic in library paths (the only optional group, FREQUENCY's prefix, is deliberately handled
with is_some_and). Nothing ties the static patterns to these unwraps except convention — a future
edit making a group optional compiles cleanly and panics at runtime. Prefer cap.name(...).map(...)
falling back to skipping the candidate, or unwrap_or_default(), so an accidental regex edit degrades
precision instead of crashing extraction. Similarly consider documenting on ParameterMatch.start/end
that offsets are BYTE indices valid only against the same &str passed to find_parameters, so
downstream replace_range users never reinterpret them as char counts.


══════ F0657 │ src/parameter/mod.rs:212-216 │ [maintainability · low] ══════
[maintainability · low] Wrapping the inner `ForgeError` result into
`ForgeError::ParameterExtraction(String)` flattens it to its `Display` output, severing
`std::error::Error::source()` chains for downstream consumers and leaving the variant's `String`
payload to mix structured context with free-form text. Note also that the callee's only realistic
failure mode today is a `panic!` from `replace_range` on an out-of-bounds/char-boundary-invalid span
— it never actually returns `Err` — so this conversion layer adds cost without adding
recoverability. Either preserve typing (add a context-preserving construction such as
`ForgeError::ParameterExtraction` carrying the source error, or use a dedicated constructor that
keeps `#[source]` wired), or simplify the contract: since extraction succeeds or panics, returning
`(String, Vec<PolicyParameter>)` without a `Result` would be honest, with the boundary panic guarded
by debug assertions that spans lie on char boundaries.


══════ F0711 │ src/parse/atomize.rs:360-367 │ [bug · low] ══════
[bug · low] Divergent semantics around the depth cap: in this branch the requirements of sub-cap
levels are still atomized, then the entire deeper subtree is frozen by `clone()` (its compound
requirements remain un-atomized forever, only noted by a trace log), while
`count_requirements_recursive_inner` hard-zeros at the same threshold — so for documents exceeding
MAX_SECTION_DEPTH the `debug!(total = total_after, ...)` summary emitted by `atomize_document`
misreports what actually happened (atomized-but-not-counted at the boundary,
silently-skipped-and-not-counted below). Both functions are independent reimplementations of "the
same" limit, and the constant is duplicated verbatim in `src/uuid.rs`
(`assign_stable_ids_to_section_inner`) and per comments also in `component_definition.rs`, inviting
drift. Define the constant once in a shared module and make rewrite/count/ID-assignment agree on
whether capped levels are counted, warned at the requirement level, or skipped entirely.

      let new_children = if depth > MAX_SECTION_DEPTH {
          tracing::trace!(
              depth,
              max = MAX_SECTION_DEPTH,
              "max section depth exceeded; skipping child traversal"
          );
          section.children.clone()
      } else {
+         /* recurse */
+     };
+ // Prefer a single shared constant (e.g., `crate::model::MAX_SECTION_DEPTH`) reused by
+ // `count_requirements_recursive_inner` and `uuid.rs`, and align their behavior: either
+ // count every section the pass touched/kept, or report skipped subtrees so the
+ // "Atomization complete" totals stay truthful for over-deep documents.


══════ F0713 │ src/parse/atomize.rs:68-72 │ [bug · low] ══════
[bug · low] The collision domain here is `{text}|{source_line}|{atom_index}` only: two distinct
compound statements landing on the same source line whose clauses happen to produce identical text
collapse to identical `stable_id`s, as do a compound split and a standalone atomic requirement with
the same wording on that line. The production replacement in `src/uuid.rs` hit exactly this problem
and had to widen its UUIDv5 input with `section_path` plus normalized text — evidence the same
hazard exists for this preliminary ID, which is exposed via the public `preliminary_id` and
persisted in `AtomizationResult.requirements` (direct API consumers relying on these IDs can merge
distinct controls). Consider salting the hash with `parent_text`/document-section context and
running the text through the same `normalize_for_hashing` used upstream, so determinism is preserved
but accidental merges of differently-scoped identical text become unlikely.

- pub fn preliminary_id(text: &str, source_line: usize, atom_index: usize) -> String {
-     let input = format!("{text}|{source_line}|{atom_index}");
+ pub fn preliminary_id(context_salt: &str, text: &str, source_line: usize, atom_index: usize) -> String {
+     // Mirror uuid.rs's hardened scheme: include contextual scope (e.g., section
+     // path or parent_text hash) and normalized text so identical wording in
+     // different scopes cannot collide.
+     let input = format!("{context_salt}|{text.trim()}|{source_line}|{atom_index}");
      let hash = Sha256::digest(input.as_bytes());
      format!("{hash:x}")
  }


══════ F0712 │ src/parse/atomize.rs:77-80 │ [maintainability · low] ══════
[maintainability · low] Byte-offset slicing on a `&str` is only sound if every offset lies on a
UTF-8 char boundary. That holds today purely as a side effect of the regex crate returning
char-boundary-aligned match spans for `str` input and the patterns being pure ASCII — nothing in
this signature documents or defends that precondition (the doc comment shows a byte position being
passed in). If a future change makes the patterns non-ASCII (e.g., matching em dashes or accented
words) or computes offsets manually, `text[..first_verb_pos]` becomes a runtime panic ("byte index N
is not a char boundary") — notably on exactly the multibyte inputs the SEC-4 test suite cares about.
The same unstated assumption guards the clause loop's `text[last_end..full_match.start()]` /
`text[last_end..]`. Use `get(..)` with graceful fallback or assert `is_char_boundary` to turn a
latent panic into a documented invariant.

  fn extract_subject(text: &str, first_verb_pos: usize) -> Option<String> {
-     let subject = text[..first_verb_pos].trim();
+     // Regex-provided offsets are always char boundaries; defend the precondition so
+     // future non-ASCII patterns fail loudly instead of panicking mid-byte.
+     debug_assert!(text.is_char_boundary(first_verb_pos));
+     let subject = text.get(..first_verb_pos)?.trim();
      if subject.is_empty() { None } else { Some(subject.to_string()) }
  }


══════ F0666 │ src/parse/clauses.rs:215-218 │ [maintainability · low] ══════
[maintainability · low] exclude_depth pairing relies on an unstated invariant: the Start arm
increments only when item_stack is non-empty and this End arm decrements only when it is currently
non-empty, with saturating_sub silently masking any imbalance. Nothing asserts the depth stays
paired (no debug_assert!(exclude_depth > 0)), so a future edit that pops/reorders items around block
boundaries — or any deviation in the event stream — would silently lift the exclusion and leak
code/quote content into item text with zero signal. At minimum add a debug assertion that a matching
open exists, or record the pending close tag to make the pairing structural rather than incidental.

          Event::End(TagEnd::CodeBlock | TagEnd::BlockQuote(_)) if !state.item_stack.is_empty() => {
+             debug_assert!(
+                 state.exclude_depth > 0,
+                 "unpaired CodeBlock/BlockQuote end in list-item exclusion"
+             );
              state.exclude_depth = state.exclude_depth.saturating_sub(1);
              true
          }


══════ F0667 │ src/parse/clauses.rs:423-423 │ [documentation · low] ══════
[documentation · low] Dead error contract: the doc comment promises `ForgeError::Parse` when
extraction fails, but nothing in the body can fail — pulldown-cmark's Parser is infallible
(malformed input degrades to best-effort events, no Err variants exist), and the rest of the
function only pushes to Vecs. Every caller therefore faces an Ok-always Result based on a misleading
premise. Either introduce a real validation/failure path or change the signature to return
ExtractedContent directly and drop the Errors section.
