# Validation slice slice09 — 60 findings
Severity mix: low×60


══════ F0367 │ src/diff/engine.rs:137-138 │ [maintainability · low] ══════
[maintainability · low] `total_old` and `total_new` are trusted caller-supplied parameters, and
`saturating_sub` silently absorbs contradictions between them and the derived entry counts: if a
caller passes stale or wrong totals, `unchanged` comes out plausible-looking but wrong, and nothing
ever cross-checks `total_new` against `total_old + added - removed`. At minimum add a debug
assertion deriving the same quantity from the new-artifact side (`total_new - added - changed -
uuid_changes`) and document the consistency precondition on the public API.

      // unchanged = controls in old that were neither removed nor changed nor uuid-changed
      let unchanged = total_old.saturating_sub(removed + changed + uuid_changes);
+     // Cross-check against the new-artifact side to catch callers passing
+     // inconsistent totals; on mismatch the stored summary would otherwise
+     // silently report a bogus `unchanged` figure.
+     debug_assert_eq!(
+         unchanged,
+         total_new.saturating_sub(added + changed + uuid_changes),
+         "inconsistent totals: total_old={total_old}, total_new={total_new}"
+     );


══════ F0368 │ src/diff/engine.rs:15-20 │ [maintainability · low] ══════
[maintainability · low] Public signature leaks the concrete `HashMap<String, ControlSnapshot>` type
into the crate boundary — including the specific `SipHash` `RandomState` hasher — forcing callers
who use a different `BuildHasher` (or another map/btree type) to rebuild a standard `HashMap` just
to call this pure function; the `#[allow(clippy::implicit_hasher)]` silences the lint that flags
exactly this. Making the hasher generic keeps the body unchanged while widening accepted inputs;
alternatively accept `impl IntoIterator<Item = (&str, &ControlSnapshot)>` to drop the collection
requirement entirely.

  #[must_use]
- #[allow(clippy::implicit_hasher)]
- pub fn compare_controls(
-     old_map: &HashMap<String, ControlSnapshot>,
-     new_map: &HashMap<String, ControlSnapshot>,
+ pub fn compare_controls<S: std::hash::BuildHasher>(
+     old_map: &HashMap<String, ControlSnapshot, S>,
+     new_map: &HashMap<String, ControlSnapshot, S>,
  ) -> Vec<DiffEntry> {


══════ F0353 │ src/diff/extractor.rs:41-43 │ [other · low] ══════
[other · low] Controls missing an `id` (or where `id` is not a string) are dropped with no
diagnostic, while the duplicate-id path above does emit a warning. A mistyped id (e.g., a number, or
the common alternative casing) makes the record vanish silently and shows up as a spurious removal
in the diff, which is hard to debug without a log. Emit at least a `tracing::debug!`/`warn!` before
the early return. The identical pattern in `collect_impl_requirements_from_container`
(implemented-requirements without a string `control-id`) deserves the same treatment.

                  let Some(id) = control.get("id").and_then(Value::as_str) else {
+                     tracing::warn!(control_uuid = ?control.get("uuid"), "Skipping control with missing or non-string 'id'");
                      continue;
                  };


══════ F0356 │ src/diff/extractor.rs:419-420 │ [test · low] ══════
[test · low] No test covers the duplicate control-id last-wins behavior that both extractors
explicitly implement (including the component-definition case where duplicates across
components/capabilities are legitimate OSCAL input). Related gaps: controls listed directly under
`catalog.controls` (not in a group) and non-string/missing `id` values. Add regression tests so the
dedup-and-overwrite semantics stay intentional rather than accidental.

+     #[test]
+     fn extract_component_def_duplicate_control_id_last_wins() {
+         let json = make_component_def_json(&[("POL-DUP-001", "u1", "First impl"), ("POL-DUP-001", "u2", "Second impl")]);
+         let result = extract_controls(&json, &ArtifactType::ComponentDefinition);
+         assert_eq!(result.len(), 1);
+         assert_eq!(result["POL-DUP-001"].description.as_deref(), Some("Second impl"));
+     }
+
+     #[test]
+     fn extract_catalog_root_level_controls() {
+         let json = serde_json::json!({
+             "catalog": {
+                 "uuid": "t", "metadata": {"title": "T", "last-modified": "2026-01-01T00:00:00Z", "version": "1.0", "oscal-version": "1.2.0"},
+                 "controls": [{"id": "POL-ROOT-001", "title": "Root", "parts": []}]
+             }
+         });
+         assert!(extract_controls(&json, &ArtifactType::Catalog).contains_key("POL-ROOT-001"));
+     }
+
      #[test]
      fn extract_catalog_deeply_nested() {


══════ F0354 │ src/diff/extractor.rs:44-44 │ [other · low] ══════
[other · low] `uuid` falls back to an empty-string sentinel while `title` and `description`
correctly use `Option<String>` — an inconsistent modeling of absence. `""` conflates all UUID-less
controls in identity comparisons and can mask real content changes downstream; an absent uuid
becomes indistinguishable from a genuinely empty one. Prefer `Option<String>` in `ControlSnapshot`
(requires a corresponding change in `super::types`) or, if the sentinel must stay for compatibility,
document that contract in the struct so consumers know `""` means "missing".

-                 let uuid = control.get("uuid").and_then(Value::as_str).unwrap_or("").to_string();
+                 let uuid = control.get("uuid").and_then(Value::as_str).map(String::from);


══════ F0355 │ src/diff/extractor.rs:76-81 │ [bug · low] ══════
[bug · low] Prose collection only inspects direct children of the control named exactly `statement`
with a plain-string `prose`. OSCAL `part` assemblies are recursive — a statement part may itself
contain nested `parts` (used heavily in NIST baselines and assessment-objective layouts) whose prose
carries meaningful content. Omitted nested prose means later content changes inside those sub-parts
never register in the snapshot. Consider walking `parts` recursively (or at least documenting that
nested-part prose is intentionally out of scope per the project requirements, e.g. FR reference).

-     parts
-         .iter()
-         .filter(|p| p.get("name").and_then(Value::as_str) == Some("statement"))
-         .filter_map(|p| p.get("prose").and_then(Value::as_str))
-         .map(String::from)
-         .collect()
+ fn collect_statement_prose(control: &Value) -> Vec<String> {
+     let Some(parts) = control.get("parts").and_then(Value::as_array) else {
+         return vec![];
+     };
+     let mut prose = vec![];
+     collect_prose_from_parts(parts, &mut prose); // recurses into nested `parts`
+     prose
+ }


══════ F0380 │ src/diff/formatter.rs:101-103 │ [maintainability · low] ══════
[maintainability · low] format_added_section, format_removed_section, and format_uuid_section are
near-identical copies of the same scaffold — build heading with count, '(none)' placeholder, per-row
writeln, trailing blank line — differing only in the marker glyph ('+', '-', '!') and projected
fields. Any future fix touching shared behavior (value escaping, multiline indentation, heading
wording, UTF-8 widths) must be applied four times and will inevitably drift, as the Changed renderer
already deviates. Extract a shared renderer parameterized by title and a row-builder closure.

- fn format_removed_section(out: &mut String, report: &DiffReport) {
-     let removed: Vec<_> =
-         report.entries.iter().filter(|e| matches!(e, DiffEntry::Removed { .. })).collect();
+ /// Shared scaffold: counted heading, "(none)", one row per line, spacing.
+ fn write_rows(out: &mut String, title: &str, rows: &[String]) {
+     write_section_heading(out, &format!("{title} ({})", rows.len()));
+     if rows.is_empty() {
+         writeln!(out, "  (none)").unwrap();
+     } else {
+         for row in rows {
+             writeln!(out, "{row}").unwrap();
+         }
+     }
+     writeln!(out).unwrap();
+ }
+
+ // Callers: write_rows(out, "Added", &rows) where rows were built with
+ // format!("  + {id}  [uuid: {uuid}]") etc.; the Changed renderer appends its
+ // detail lines to the same row buffer before handing it off.


══════ F0378 │ src/diff/formatter.rs:14-15 │ [maintainability · low] ══════
[maintainability · low] Roughly 30 `writeln!(out, ...).unwrap()` calls treat a fallible-looking
Result as infallible with no word explaining why. Today this is safe — `impl fmt::Write for String`
never returns Err — but every call reads like a live panic site, and if the formatter is ever
retargeted at io::Write (file/stdout streaming) they become ~30 real panics that bypass error
propagation. The invariant is undocumented. Document it once and/or funnel all writes through a
single helper so a sink-type change touches one place, not the whole file.

+     // INVARIANT: `std::fmt::Write for String` never returns `Err`, so the
+     // `unwrap()`s below cannot fire for this sink. If this formatter is ever
+     // retargeted at io::Write/File, route all writes through one helper
+     // returning io::Result instead of scattering panic sites.
      writeln!(out, "OSCAL Diff Report").unwrap();
      writeln!(out, "=================").unwrap();


══════ F0382 │ src/diff/formatter.rs:48-48 │ [style · low] ══════
[style · low] title.len() measures UTF-8 bytes, not display columns. All current titles are
generated ASCII ('Added (n)', 'UUID Stability Changes (n)'), so alignment holds today, but
interpolating any non-ASCII label later would produce a visibly overlong rule relative to the
heading text. Cheap insurance: count chars instead of bytes (full display-width parity would need
unicode-width).

-     writeln!(out, "{}", "\u{2500}".repeat(title.len())).unwrap();
+     writeln!(out, "{}", "\u{2500}".repeat(title.chars().count())).unwrap();


══════ F0379 │ src/diff/formatter.rs:52-53 │ [performance · low] ══════
[performance · low] Each of the four renderers allocates a Vec of references via a `matches!`
filter, then redundantly re-matches the same variant inside the loop (and again implicitly for
is_empty()). Same pattern repeats at
format_changed_section/format_removed_section/format_uuid_section — four linear passes over entries
plus a fifth counting pass. A single partitioning pass in format_diff_report yields all four buckets
at once, and lets each renderer take a slice without repeating the variant check (which the compiler
cannot otherwise verify is exclusive).

-     let added: Vec<_> =
-         report.entries.iter().filter(|e| matches!(e, DiffEntry::Added { .. })).collect();
+     // Partition once; pass buckets into the section renderers.
+     let mut added = Vec::new();
+     let mut changed = Vec::new();
+     let mut removed = Vec::new();
+     let mut uuid_changed = Vec::new();
+     for e in &report.entries {
+         match e {
+             DiffEntry::Added { .. } => added.push(e),
+             DiffEntry::Changed { .. } => changed.push(e),
+             DiffEntry::Removed { .. } => removed.push(e),
+             DiffEntry::UuidChanged { .. } => uuid_changed.push(e),
+         }
+     }


══════ F0399 │ src/diff/mod.rs:203-206 │ [test · low] ══════
[test · low] The suite never exercises the rejection paths added/adjusted in this module: Profile
and Mapping artifacts being refused, the ambiguous-artifact diagnostics, or the oversize/size-guard
behavior in `read_diff_file`. As written the 50 MB constant makes the oversize branch effectively
untestable without writing a giant fixture. Add small tests for the Profile/Mapping/ambiguity
messages, and make the limit injectable (e.g. an internal `read_file_with_limit(path, max_bytes)`
called by `read_diff_file`) so the guard branch can be tested with a few-byte threshold.

      #[test]
-     fn test_missing_file_error() {
-         let result =
-             diff_artifacts(Path::new("/nonexistent/old.json"), Path::new("/nonexistent/new.json"));
+     fn test_profile_rejected_with_clear_message() {
+         let f = write_json_file(&serde_json::json!({"profile": {"uuid": "p-uuid"}}));
+         let err = diff_artifacts(f.path(), f.path()).unwrap_err();
+         assert!(matches!(err, ForgeError::DiffError(_)));
+         assert!(err.to_string().contains("Profile"));
+     }
+
+     #[test]
+     fn test_ambiguous_root_keys_reports_ambiguity_not_unrecognized() {
+         let f = write_json_file(&serde_json::json!({
+             "catalog": {"uuid": "c"},
+             "profile": {"uuid": "p"}
+         }));
+         let err = diff_artifacts(f.path(), f.path()).unwrap_err();
+         assert!(err.to_string().contains("multiple") || err.to_string().contains("catalog"));
+     }


══════ F0398 │ src/diff/mod.rs:99-106 │ [maintainability · low] ══════
[maintainability · low] These two arms are verbatim duplicates apart from the human label, inviting
drift (e.g. someone updates the expectation list in one arm and forgets the other). `OscalModelType`
already exposes its standard key via `as_str()`; bind the matched value once and emit one message.
Note the gate itself is compile-safe against future variants (no wildcard `Ok(_)` arm), so this is
purely a duplication cleanup.

-         Ok(OscalModelType::Profile) => Err(ForgeError::DiffError(format!(
-             "'{}': Profile artifacts are not supported by diff; expected Catalog or ComponentDefinition",
-             path.display()
-         ))),
-         Ok(OscalModelType::Mapping) => Err(ForgeError::DiffError(format!(
-             "'{}': Control Mapping artifacts are not supported by diff; expected Catalog or ComponentDefinition",
-             path.display()
-         ))),
+         Ok(unsupported @ (OscalModelType::Profile | OscalModelType::Mapping)) => {
+             Err(ForgeError::DiffError(format!(
+                 "'{}': {} artifacts are not supported by diff; expected Catalog or ComponentDefinition",
+                 path.display(),
+                 unsupported.as_str()
+             )))
+         }


══════ F0371 │ src/diff/types.rs:135-151 │ [maintainability · low] ══════
[maintainability · low] All seven counters are public and unconstrained, so nothing enforces the
relationships a meaningful diff report must satisfy: every old control lands in exactly one bucket,
giving `total_old == removed + changed + uuid_changes + unchanged` and `total_new == added + changed
+ uuid_changes + unchanged`. Any code path that hand-assembles a `DiffSummary` (or adjusts
individual counts incrementally and forgets one) can publish a self-contradictory report — e.g.
`unchanged: 15` with `total_old: 10` compiles and prints fine. Consider a validating constructor
(returning `Result` or `debug_assert!`ing the invariants) or encapsulating the fields behind
accessors so the aggregate can only be produced by the single counting routine.

- #[derive(Debug, Clone, PartialEq, Eq)]
- pub struct DiffSummary {
-     /// Total number of controls in the old artifact.
-     pub total_old: usize,
-     /// Total number of controls in the new artifact.
-     pub total_new: usize,
-     /// Number of controls added (present only in the new artifact).
-     pub added: usize,
-     /// Number of controls removed (present only in the old artifact).
-     pub removed: usize,
-     /// Number of controls with field-level changes.
-     pub changed: usize,
-     /// Number of controls that are identical between artifacts.
-     pub unchanged: usize,
-     /// Number of controls whose UUIDs changed but fields are otherwise identical.
-     pub uuid_changes: usize,
+ impl DiffSummary {
+     /// Validates that the buckets partition both populations:
+     /// `total_old == removed + changed + uuid_changes + unchanged` and
+     /// `total_new == added + changed + uuid_changes + unchanged`.
+     pub fn try_new(
+         total_old: usize,
+         total_new: usize,
+         buckets: DiffBuckets,
+     ) -> Result<Self, SummaryInvariantError> {
+         // construct only after checking the partition identities above
+     }
  }


══════ F0409 │ src/error.rs:134-134 │ [documentation · low] ══════
[documentation · low] Wording is misleading: a symlink that points to a regular file resolves to a
regular file when checked, so it would not trigger this error — only broken symlinks, directories,
and special files would. If callers intentionally use symlink_metadata (rejecting even valid
symlinked files), say so; otherwise the parenthetical should not imply any symlink hits this
variant.

-     /// The path does not refer to a regular file (e.g., it is a directory or symlink).
+     /// The path does not refer to a regular file (e.g., it is a directory,
+     /// device node, or a broken symlink).


══════ F0410 │ src/error.rs:21-26 │ [style · low] ══════
[style · low] The human-readable size ladder stops at MB: a 2 GiB file (plausible for
--max-size-raised limits) renders as '2147.5MB', which reads poorly next to the asserted
'15.0MB'-style diagnostics. Add a GB tier so large-limit messages stay conventional.

      let bytes = *bytes;
      if bytes < 1_048_576 {
          format!("{:.1}KB", bytes as f64 / 1024.0)
-     } else {
+     } else if bytes < 1_073_741_824 {
          format!("{:.1}MB", bytes as f64 / 1_048_576.0)
+     } else {
+         format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
      }


══════ F0407 │ src/error.rs:292-296 │ [maintainability · low] ══════
[maintainability · low] The no-prefix presentation contract couples this variant's Display output to
one specific call site (main's `Error: …` wrapper). Anything else that renders the error directly —
log macros, test harnesses, downstream wrappers that attach context, or a second binary — gets
clap's raw multi-line usage text with no prefix or inconsistent double-wrapping. The invariant is
currently enforced only by a unit test on Display, not on the end-to-end contract; consider an
integration test asserting main's rendered output, or carrying structured data (e.g. the clap usage
string) so the formatting decision stays at the binary boundary.

      /// Displayed without its own `error:` prefix because `main` already wraps
      /// every message in `Error: …`; carrying both prefixes produced
      /// `Error: error: …`.
+     ///
+     /// WARNING: this contract holds only when the value reaches stderr via
+     /// `main`. Other `Display` consumers receive clap's raw message unprefixed.
      #[error("{0}")]
      MissingRequiredArgument(String),


══════ F0364 │ src/export/mod.rs:5-6 │ [maintainability · low] ══════
[maintainability · low] Inconsistent re-export surface in this facade module: `xml_deserializer` and
`yaml` have their public entry points re-exported here, but the publicly declared `xml_serializer`
module does not. Its three public functions (serialize_catalog_to_xml,
serialize_component_definition_to_xml, serialize_profile_to_xml) are genuinely part of the crate's
public API — external users reach them via the long path
`forge::export::xml_serializer::serialize_catalog_to_xml` (e.g.,
tests/oscal_1_2_3_compatibility_test.rs:150, tests/xml_catalog_test.rs:172,
benches/xml_benchmark.rs:98), while internal callers repeat `crate::export::xml_serializer::...`
everywhere (src/cli/export.rs:205, src/cli/profile.rs:127, src/pipeline.rs:219). Since these
functions are deliberately public, the omission looks accidental rather than a scoping decision.
Either add symmetric re-exports so consumers get one flat API surface (`pub use
xml_serializer::{serialize_catalog_to_xml, serialize_component_definition_to_xml,
serialize_profile_to_xml};`) mirroring how tests already do `use
forge::export::{deserialize_catalog_from_xml, serialize_catalog_to_xml}`-style flat imports, or drop
the item-level re-exports of the other two modules entirely and make the whole interface uniformly
module-qualified.

  pub use xml_deserializer::{deserialize_catalog_from_xml, deserialize_component_from_xml};
+ pub use xml_serializer::{
+     serialize_catalog_to_xml, serialize_component_definition_to_xml, serialize_profile_to_xml,
+ };
  pub use yaml::{deserialize_from_yaml, serialize_to_yaml};


══════ F0447 │ src/export/xml_deserializer.rs:344-347 │ [maintainability · low] ══════
[maintainability · low] Semantic mismatch in error usage: `ExportInvalidOscal` is documented in
`src/error.rs` as an *export-subcommand input* diagnostic ("The input is not valid JSON, XML, or
YAML representing an OSCAL artifact"), yet here it is repurposed deep inside the deserialization
conversion helpers while the public wrappers advertise only `ForgeError::Serialization`. Repurposing
export-oriented variants on the deserialize path couples CLI surfaces to library internals; consider
a dedicated invalid-UUID/invariant variant (or reuse of `Serialization`) so error taxonomy matches
the pipeline stage.


══════ F0446 │ src/export/xml_deserializer.rs:481-483 │ [documentation · low] ══════
[documentation · low] The `# Errors` section understates the failure surface: besides
`ForgeError::Serialization` raised by quick-xml, the conversion layer propagates
`ForgeError::ExportInvalidOscal` for invalid UUIDs (resource, capability, control-implementation,
implemented-requirement). Callers filtering/matching on the error variant will miss that this
deserialization entry point can return both. Document the second variant (or unify on one variant
for the import path) in both public functions.

  /// # Errors
- /// Returns `ForgeError::Serialization` if XML parsing fails.
+ /// Returns `ForgeError::Serialization` if XML parsing fails, and
+ /// `ForgeError::ExportInvalidOscal` if converted identifiers (resource,
+ /// capability, control-implementation, implemented-requirement UUIDs)
+ /// are malformed.
  pub fn deserialize_catalog_from_xml(xml: &str) -> Result<CatalogEnvelope, ForgeError> {


══════ F0462 │ src/export/xml_serializer.rs:609-609 │ [documentation · low] ══════
[documentation · low] Version-string inconsistency: this doc header claims "valid OSCAL v1.2.3 XML"
while every other reference in the file targets v1.2.0 (module header `serialize_to_xml` contract,
test fixture `oscal_version: "1.2.0"`, doc comments on the catalog/component-definition
serializers). If unintentional, this looks like a copy/paste typo; if profiles deliberately target a
different OSCAL revision, the distinction should be documented rather than silently diverging.


══════ F0452 │ src/export/xml_serializer.rs:630-630 │ [other · low] ══════
[other · low] Inconsistent `<last-modified>` formatting across exporters: this profile path
reformats via `chrono::DateTime::to_rfc3339()` (yielding a `+00:00` offset), while
serialize_catalog_to_xml and serialize_component_definition_to_xml copy `metadata.last_modified` (a
String) byte-for-byte. Any upstream producer that doesn't emit canonically-formatted timestamps
therefore produces stylistically divergent datetimes across artifact types, complicating
diff/regression comparison. Consider a single shared normalization helper used by all three
serializers, or else match the String-passthrough convention here too.

+     // Keep timestamp rendering consistent with the catalog/component-definition
+     // exporters (all passthrough, or all normalized via one shared helper).
      let last_modified_str = profile.metadata.last_modified.to_rfc3339();


══════ F0393 │ src/export/yaml.rs:21-22 │ [maintainability · low] ══════
[maintainability · low] The typed serde_yaml::Error is flattened into a bare message string here
(and again in deserialize_from_yaml), losing its category metadata (classify()), position
(line/column), and source chain. Human-readable location text survives via Display, but callers who
need to distinguish syntax errors from data-mapping errors programmatically can no longer match on
them. Consider extending ForgeError::Serialization (or adding a wrapper variant carrying #[from]
serde_yaml::Error) so the typed error survives the boundary.

-     let result = serde_yaml::to_string(model)
-         .map_err(|e| ForgeError::Serialization(format!("YAML serialization failed: {e}")));
+ // Ideally: make ForgeError preserve the source, e.g.
+ //   Serialization { context: String, #[source] source: Box<dyn std::error::Error + Send + Sync> }
+ // or add:
+ //   Yaml(#[from] serde_yaml::Error)
+ // Keeping e.position()/e.classify() reachable lets callers branch on
+ // Category::Syntax vs Category::Data instead of parsing message prose.


══════ F0423 │ src/framework/analysis.rs:194-199 │ [bug · low] ══════
[bug · low] [Discovery filter failure mode] --group requires group field sanity only if
`filters.group.is_some()`, yet validate_group_ids merely confirms **zero** ambiguous ids exist
rather than asserting the requested id exists in either inventory. Passing `--group nonexistent`
yields zero findings silently, indistinguishable from genuinely zero matches; combine with above
filters and consumers get the impression the group is clean. Prefer erroring (or clearly flagging)
when the requested group doesn't resolve against old/new inventories, matching the explicit
rejection pattern you already enforce for ambiguity.


══════ F0419 │ src/framework/analysis.rs:478-479 │ [maintainability · low] ══════
[maintainability · low] [Signature Drift Risk] fingerprint keys SubjectFingerprint.sha256 straight
off inventory fingerprint strings, which in turn derive from raw catalog serialization. Two
logically identical control statements differing only by OSCAL formatting (whitespace, property
ordering as re-emitted by some upstream tools) yield different hashes and trigger
ContentChanged/blocking reviews even though no semantic change occurred. Recommend pinning the
fingerprint computation to a normalized/canonical representation (sorted props, stripped
insignificant text nodes) or exposing both raw and canonical hashes so dedupe logic downstream can
choose. Also since fingerprint returns Option<&str> borrowed from a reference already moved around,
consider `.map(String::from)` vs str::to_string equivalence — no bug today, but if Inventory
switches from owned Strings to interned lifetimes this cloning step becomes an accidental
borrow-extension; verify ownership remains copy-on-read.

-                 let old_hash = old.fingerprint(SubjectType::Control, id).map(str::to_string);
-                 let new_hash = new.fingerprint(SubjectType::Control, id).map(str::to_string);
+                 let old_hash = old.fingerprint(SubjectType::Control, id).map(str::to_owned);
+                 let new_hash = new.fingerprint(SubjectType::Control, id).map(str::to_owned);


══════ F0421 │ src/framework/analysis.rs:934-936 │ [maintainability · low] ══════
[maintainability · low] [Trust boundary mismatch on successor_map absence] When
manifest.successor_map is None, every control that disappears from new and appears in new is
classified as plain Added/Removed — even though many of these pairs almost certainly represent
merges/splits renames that simply lacked an approved migration record. This forces Removal findings
down the blocking RepairOrApproveMapping path and skips the IdentityMigrationDeclared informational
flow entirely, blending reviewer fatigue with genuine irrecoverable deletions. Consider
distinguishing findings when the successor_map was omitted versus explicitly empty: emit a
supplementary meta-finding (ReasonCode like MigrationEvidenceMissing) prompting operators to submit
the approval trail rather than silently treating unreviewed renames as true removals. This keeps
deterministic output while avoiding conflation of evidence-gap with evidence-absent.


══════ F0403 │ src/framework/disposition.rs:111-112 │ [bug · low] ══════
[bug · low] `decided_at` is only shape-checked; the raw RFC 3339 string (arbitrary UTC offset,
seconds or fractional precision, timestamps arbitrarily far in the past/future) is persisted
verbatim. Any later consumer that filters or orders by recency using string comparison gets wrong
results across mixed offsets (e.g. `2026-01-01T23:00:00+10:00` sorts before `2026-01-01T00:00:00Z`).
Either normalize the value at the boundary — parse then re-emit UTC — or better, type the field as
`chrono::DateTime<chrono::FixedOffset>` so invalid states are unrepresentable and comparison
semantics are explicit.

-         chrono::DateTime::parse_from_rfc3339(&disposition.decided_at)
+         let decided_at = chrono::DateTime::parse_from_rfc3339(&disposition.decided_at)
              .map_err(|_| error(format!("{path}.decided_at must be an RFC 3339 timestamp")))?;
+         // Persist a canonical form so downstream ordering/comparison is sound.
+         canonical_decided_at.push(
+             decided_at.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
+         );


══════ F0404 │ src/framework/disposition.rs:114-114 │ [maintainability · low] ══════
[maintainability · low] Hidden side effect in a function named `validate`: it silently reorders
`file.dispositions` by `finding_id`. Nothing in the signature, name, or returned value communicates
this ordering invariant, yet the whole point of the normalization (deterministic output/diffability)
depends on consumers knowing they receive sorted records. Rename to `validate_and_normalize` (or
split normalization out of `parse`) and document the post-condition `dispositions are sorted
ascending by finding_id`, including on the public `load` API.

-     file.dispositions.sort_by(|left, right| left.finding_id.cmp(&right.finding_id));
+     // Normalize: callers of `parse`/`load` rely on dispositions being
+     // deterministically ordered by `finding_id`; document this invariant
+     // on the `validate`/`load` signatures.
+     file.dispositions
+         .sort_by(|left, right| left.finding_id.cmp(&right.finding_id));


══════ F0400 │ src/framework/disposition.rs:22-24 │ [documentation · low] ══════
[documentation · low] Format-only digest binding: `prior_report_sha256` is only checked for 64-hex
format, so a struct carrying an arbitrary hash passes validation. The "report-bound" guarantee only
exists if every consumer recomputes the prior report's digest and compares it before applying
dispositions, which is nowhere stated on the type. Add a contract doc on `DispositionFile` making
this caller obligation explicit, so consumers don't treat mere successful validation as proof of
binding.

+ /// Verified disposition contracts for a framework-impact report.
+ ///
+ /// Contract: `prior_report_sha256` is validated as format only — callers MUST
+ /// recompute and compare this digest against the actual prior report before
+ /// applying any disposition, or the report binding is not established.
+ #[derive(Debug, Clone, Deserialize, Serialize)]
+ #[serde(deny_unknown_fields)]
  pub struct DispositionFile {
      pub schema_version: String,
      pub prior_report_sha256: String,


══════ F0415 │ src/framework/manifest.rs:172-173 │ [maintainability · low] ══════
[maintainability · low] The lowercase SHA-256 gate runs first, but validate() unconditionally
continues onto uuid::Uuid::parse_str and the OSCAL pin afterwards - a manifest whose fingerprint is
rejected will surface only the first error per pass, fine, but note the real ordering hazard:
validation order means path-confinement errors dominate identity checks. More importantly for type
design, after successful validation root_uuid survives as an arbitrary-author UUID spelling rather
than a parsed uuid::Uuid or a typed DocumentIdentity newtype. Consumers later in the pipeline must
re-parse (and potentially reject) the same string, and two spellings of one identity ('...' vs
'{...}' or uppercase hex of the same bytes) can diverge silently across artifacts. Normalizing once
at parse time would make identity comparison byte-exact everywhere.

-     uuid::Uuid::parse_str(&resource.root_uuid)
-         .map_err(|_| impact_error(format!("{path}.root_uuid must be a UUID")))?;
+     let normalized = uuid::Uuid::parse_str(&resource.root_uuid).map_err(|_| {
+         impact_error(format!("{path}.root_uuid must be a UUID"))
+     })?;
+     // Consider storing this as resource.root_uuid = normalized.to_string() equivalent
+     // via a post-deserialize hook, or modeling it as `uuid::Uuid` with
+     // #[serde(deserialize_with = ...)] so callers receive canonicalized identities.


══════ F0411 │ src/framework/manifest.rs:19-22 │ [maintainability · low] ══════
[maintainability · low] This public composite manifest type derives neither PartialEq/Eq nor
PartialOrd/Ord. Any caller wanting to diff manifests, cache/pin them in ordered sets/maps, or verify
post-validation immutability must hand-roll field-by-field comparison, inviting drift whenever
fields are added. Structs composed of owned Strings, PathBufs, Vecs, and Copy enums support these
traits structurally; deriving them costs nothing.

- /// A closed portfolio manifest for one exact framework revision comparison.
- #[derive(Debug, Clone, Deserialize, Serialize)]
+ #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
  #[serde(deny_unknown_fields)]
  pub struct ImpactManifest {


══════ F0413 │ src/framework/manifest.rs:191-196 │ [documentation · low] ══════
[documentation · low] The checks performed here are purely lexical: symlinked directories, NTFS
junctions, bind mounts, and concurrent renames can redirect a syntactically valid relative path
outside the intended evidence directory after validation succeeds. Document that guarantee boundary
explicitly - the contract these functions enforce is 'safe spelling', not 'confined target', so
file-open sites must stay responsible for resolving and confining symlinks/junctions
(canonicalize-and-prefix check or equivalent) under the trusted manifest root.

+ /// Lexical confinement only: rejects absolute spellings, parent-directory
+ /// components, and cross-platform separators/roots. Callers opening these paths
+ /// remain responsible for re-checking canonicalized targets and symlink/junction
+ /// redirects (TOCTOU) against the trusted evidence root.
  fn validate_json_path(path: &str, value: &Path) -> Result<(), ForgeError> {
      if value.as_os_str().is_empty()
          || value.extension().and_then(|extension| extension.to_str()) != Some("json")
      {
          return Err(impact_error(format!("{path} must be a local .json file")));
      }


══════ F0416 │ src/framework/manifest.rs:81-84 │ [other · low] ══════
[other · low] Every failure funnel maps foreign errors through `.map_err(impact_error)` and
serializes diagnostics into owned Strings inside ForgeError::FrameworkImpact. Both
crate::json_strict::parse_value's structured parse error (line/offset/depth context that its Limits
already track) and serde_json::Error's own position/tier information are flattened away at this
boundary, so downstream CLI and CI layers cannot distinguish 'too deep' from 'bad UTF-8' or print
precise offsets without re-parsing the raw bytes themselves. If ForgeError::FrameworkImpact can be
widened to carry a source cause alongside the human message, preserving chain semantics here keeps
determinism for golden tests while still letting boundaries render structured diagnostics.

      let value = crate::json_strict::parse_value(bytes, "manifest", STRICT_JSON_LIMITS)
-         .map_err(impact_error)?;
+         .map_err(|error| impact_error_with_source("manifest JSON rejected", error))?;
      let manifest: ImpactManifest = serde_json::from_value(value)
-         .map_err(|error| impact_error(format!("invalid manifest contract: {error}")))?;
+         .map_err(|error| impact_error_with_source("manifest violates contract fields/bounds", error))?;


══════ F0428 │ src/framework/mod.rs:0-0 │ [other · low] ══════
[other · low] All errors here are flattened early into context-free Strings via
impact_error(format!("manifest: {error}")), discarding any typed structure (IO error kind,
size-limit metadata, etc.). Because FrameworkImpact appears to carry a plain String, downstream
consumers cannot distinguish 'input too large' from 'unreadable' programmatically. At minimum ensure
the formatter keeps the io::Error source chain for diagnostics; better, add dedicated data-carrying
variants for these specific failure modes so tests and callers do not depend on English phrasing.

  io::check_file_size(manifest_path, manifest::MAX_MANIFEST_BYTES)
-         .map_err(|error| impact_error(format!("manifest: {error}")))?
+         .map_err(|error| impact_error_with_source("manifest size check", &error))?


══════ F0430 │ src/framework/mod.rs:225-225 │ [test · low] ══════
[test · low] The fixed-size [(&'static str, usize); 18] silently desynchronizes from ChangeSummary:
adding/removing a summary counter shifts indices with no compile-time diagnostic (values just land
under the wrong label, or extra fields fail to appear at all). Derive each tuple from the struct's
own field list (or loop over explicit enum metrics rendered generically) so the renderer cannot go
stale unnoticed.


══════ F0429 │ src/framework/mod.rs:446-448 │ [performance · low] ══════
[performance · low] escape() uses char::escape_default, which renders every non-ASCII character as a
\u{...} sequence. Manifest-sourced document versions, group titles, or control identifiers
containing legitimately readable Unicode will show up garbled (e.g. 'v1.2 – baseline' → v1.2
\u{2013} baseline) in the text report even though plain-text output has no quoting syntax requiring
this. Consider min_escape_chars (only quotes/backslashes/control chars) for human-facing text.


══════ F0431 │ src/framework/mod.rs:70-72 │ [documentation · low] ══════
[documentation · low] This ordering encodes the documented guarantee that no report is written until
all input and destination validation succeeds — validate_destination() and render_report() must
remain ahead of write_output(). Nothing enforces that invariant for future edits (a later refactor
swapping two lines would silently break the no-partial-artifact promise while unit tests still
pass). Add an assertion-style comment plus a regression test asserting write_output is never reached
when destination aliasing fails.

+ // NOTE(ordering): nothing below this line may precede validate_destination();
+ // reports must never be written before every input/destination check succeeds.
  validate_destination(&inputs, output)?;
      let rendered = render_report(&report, format)?;
      crate::cli::output::write_output(&rendered, output)?;


══════ F0465 │ src/framework/model.rs:22-23 │ [maintainability · low] ══════
[maintainability · low] `ImpactReport` derives only `Serialize`, even though its own attributes
anticipate deserialization (`#[serde(default)]` on `prior_only_dispositions` is only meaningful for
`Deserialize`) and the codebase validates prior reports through ad-hoc `serde_json::Value`
inspection (`validate_prior_report` in `src/framework/analysis.rs`). That forces every field name
and the mixed kebab-case/snake-case casing conventions of this schema to be re-declared by hand as
string literals during prior-report validation, with no compile-time check against drift from this
type definition. Derive `Deserialize` as well (all referenced types — `DecisionState`,
`DispositionRecord`, the nested structs/enums — appear derive-compatible, and `Option` fields need
no extra attributes for omitted keys), which enables cheap round-trip tests guarding the published
contract; alternatively, document why the report is deliberately write-only.

+ #[derive(Debug, Clone, Serialize, serde::Deserialize)]
+ #[serde(deny_unknown_fields)] // optional: tightens prior-report validation
+ pub struct ImpactReport {
+     ...
      #[serde(default, skip_serializing_if = "Vec::is_empty")]
      pub prior_only_dispositions: Vec<super::disposition::DispositionRecord>,
+ }


══════ F0438 │ src/ingest/mod.rs:101-107 │ [maintainability · low] ══════
[maintainability · low] Triplicated boilerplate: the identical io-error-kind -> ForgeError mapping
closure is repeated for fs::metadata, fs::read, and canonicalize. Extracting a small helper (e.g. fn
map_io_error(path: &Path) -> impl Fn(std::io::Error) -> ForgeError) removes duplication and
guarantees future error-kind handling stays consistent across all three call sites.

-     let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
-         std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
-         std::io::ErrorKind::PermissionDenied => {
-             ForgeError::PermissionDenied { path: path.to_path_buf() }
-         }
-         _ => ForgeError::Io(e),
-     })?;
+     let metadata = std::fs::metadata(path).map_err(map_io_error(path))?;


══════ F0437 │ src/ingest/mod.rs:308-310 │ [maintainability · low] ══════
[maintainability · low] Irreversible heuristic rewriting: the first non-empty extracted line is
always promoted to "# Title" (it may equally be a page header/footer or confidentiality banner) and
any short line lacking 'must'/'shall'/terminal punctuation becomes an H2. Because downstream OSCAL
conversion treats these headings and shifted line numbers as authoritative, heuristic
misclassifications silently distort the compliance structure with no way back to the raw extraction.
Consider keeping the raw extraction alongside the markdownized variant (or recording which lines
were rewritten) so transformations remain auditable/reversible.


══════ F0436 │ src/ingest/mod.rs:48-50 │ [documentation · low] ══════
[documentation · low] Documentation/correctness contract: str::lines() splits away \r\n and drops
any trailing newline, while reconstruct_content rejoins with "\n", so the reconstructed text never
byte-matches the raw bytes covered by fingerprint for CRLF-terminated or newline-terminated inputs.
Any consumer using the fingerprint to attest the reconstructed content (e.g. embedding both in OSCAL
evidence) will see verification failures or, worse, silently accept a mismatch. Document this
normalization explicitly on reconstruct_content (or preserve EOL data in SourceLine so
round-tripping is exact).


══════ F0461 │ src/io.rs:196-200 │ [test · low] ══════
[test · low] No test exercises the documented failure modes of these helpers:
`sanitize_artifact_path("/")` currently leaks the root path, `write_atomic` is never asserted
against an existing file's permission bits being clobbered, and `check_file_size`'s
symlink-following semantics are untested while `regular_file_metadata_rejects_symlinks` enforces the
opposite policy. Add regression tests pinning the intended behavior (especially the None-component
sanitize inputs and the post-persist mode) so the guard contracts above stay observable.


══════ F0457 │ src/io.rs:21-23 │ [maintainability · low] ══════
[maintainability · low] Errors from temp-file creation, writing, and content fsync propagate as bare
`ForgeError::Io` with zero destination context, while only the `persist` arm formats `'{}'` with
`path.display()` — producing inconsistent diagnostics like "No space left on device (os error 28)"
with no hint of which output file failed. Attach the destination path context to all three steps,
mirroring the persist branch.


══════ F0456 │ src/io.rs:30-33 │ [maintainability · low] ══════
[maintainability · low] If this final directory-fsync fails, the destination has already been
atomically replaced, yet the function returns Err — leaving a torn contract where the caller
believes nothing was written and may retry or report failure although the new content is durably in
place. Either perform the parent-directory sync on a best-effort basis, or document that an error
past `persist` implies the write did land (partial-success ambiguity), so callers behave correctly.


══════ F0460 │ src/io.rs:81-83 │ [performance · low] ══════
[performance · low] The extra `is_dir()` probe before `canonicalize` is both redundant (an absent
directory already makes the subsequent `canonicalize` fail with ENOENT) and a race: the directory
can vanish/be created between the two syscalls, so the cached-metadata check gives no real guarantee
and just widens a TOCTOU window plus costs an extra stat. Drop it and let `canonicalize`'s error
carry the truth.


══════ F0466 │ src/lib.rs:32-32 │ [documentation · low] ══════
[documentation · low] This `pub mod` lacks the one-line crate-doc summary that most of its sibling
modules have (applicability, cli, config, diff, error, export, framework, lifecycle, mapping,
migration, sanitize, trace, uuid are documented). The same omission applies to ingest, io, model,
oscal, oscal_cli, parameter, parse, pipeline, round_trip, summary, types, validate below. For a
library whose rustdoc is its primary interface, this inconsistency leaves over half of the module
list undocumented on docs.rs.

+ /// Citation extraction from normative requirement text.
  pub mod citation;


══════ F0482 │ src/lifecycle/mod.rs:968-971 │ [security · low] ══════
[security · low] On platforms other than unix/windows this stub unconditionally reports a hard-link
count of 1, so `validate_mutation_target` passes for multiply-linked records and `--apply` can
mutate a record that aliases unrelated hard links elsewhere on the filesystem — silently defeating
the platform-specific guarantee upheld on supported targets. Either fall back to a best-effort
identity probe (canonicalized st_dev/st_ino equivalents), or make the unsupported-platform behavior
loud: emit an explicit `ForgeError` like "hard-link verification is not supported on this platform"
so operators consciously opt into the weaker guarantee instead of receiving a false all-clear.


══════ F0494 │ src/lifecycle/record.rs:335-339 │ [performance · low] ══════
[performance · low] This duplicate check builds a fresh `BTreeSet<String>` (one cloned-and-allocated
`String` per owner key) just to compare lengths, on every `validate` call. The `owner → party`
membership was already resolved through the `parties` map immediately above, so a cheaper linear
approach suffices at this scale (≤ 64 owners): pairwise/sorted adjacent comparison, or fold the
duplicate detection into the owner-resolution loop above with a single `BTreeSet<&str>` of seen
keys.

-     if record.policy.owner_keys.iter().collect::<BTreeSet<_>>().len()
-         != record.policy.owner_keys.len()
-     {
-         return Err(error("$.policy.owner_keys contains duplicates"));
-     }
+     // Detect duplicates during resolution above with `seen.insert(owner)`,
+     // avoiding a second allocated set per validation pass.


══════ F0475 │ src/main.rs:11-17 │ [maintainability · low] ══════
[maintainability · low] Building the filter purely from the `--verbose`/`--quiet` strings discards
any `RUST_LOG` value in the environment: `with_env_filter(<str>)` parses only this literal, so
operators lose per-module/per-target tuning (e.g. `RUST_LOG=forge::render=trace,crypto=off`) in
production or test harnesses. Flag precedence itself is fine — clap rejects `--verbose --quiet`
together upstream (see the `verbose_and_quiet_conflict` test in src/cli/mod.rs). Consider letting
`RUST_LOG` take precedence when set, falling back to the flag-derived default otherwise.

-     let filter = if cli.verbose {
+     let default_filter = if cli.verbose {
          "debug"
      } else if cli.quiet {
          "error"
      } else {
          "warn"
      };
+     // Honor RUST_LOG per-module directives when provided; otherwise derive the
+     // level from --verbose/--quiet.
+     let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
+         .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter));
+     tracing_subscriber::fmt().with_env_filter(env_filter).with_writer(std::io::stderr).init();


══════ F0512 │ src/mapping/baseline.rs:137-142 │ [maintainability · low] ══════
[maintainability · low] The `unreachable!(...)` arm is only safe because of the adjacent `if
raw_changed || companion_changed` guard; refactoring the condition without touching the match turns
a logic slip into a user-facing panic in library code, contrary to the propagate-with-`Result`
convention used everywhere else in this file. Structure the branches exhaustively (`if/else
if/else`) so no panic path exists regardless of how the guard evolves.

-             let detail = match (raw_changed, companion_changed) {
-                 (true, true) => "resource and resolved Catalog bytes changed",
-                 (true, false) => "resource bytes changed",
-                 (false, true) => "resolved Catalog bytes changed",
-                 (false, false) => unreachable!("guard requires a changed resource fingerprint"),
+             let detail = if raw_changed && companion_changed {
+                 "resource and resolved Catalog bytes changed"
+             } else if raw_changed {
+                 "resource bytes changed"
+             } else {
+                 "resolved Catalog bytes changed"
              };


══════ F0508 │ src/mapping/baseline.rs:278-286 │ [bug · low] ══════
[bug · low] `maps_by_uuid(current)` collapses duplicate map UUIDs last-wins into the `BTreeMap`,
whereas the baseline path hard-fails on the same collision in `verify_integrity`. The asymmetry
means a collided `current` document yields wrong `map_removed`/`map_added` classifications and a
wrong-map `relationship_changed` comparison instead of the clear error the baseline gets. This
safety relies entirely on an undocumented upstream invariant (`manifest::parse` rejecting duplicate
map keys and `model::build` deriving collision-free UUID v5s); nothing in this module enforces or
states it. Verify UUID uniqueness for both documents symmetrically, or at minimum document the
invariant here.

- fn maps_by_uuid(document: &MappingCollectionEnvelope) -> BTreeMap<String, &OscalMap> {
-     document
-         .mapping_collection
-         .mappings
-         .iter()
-         .flat_map(|mapping| mapping.maps.iter())
-         .map(|map| (map.uuid.to_string(), map))
-         .collect()
+ // NOTE: `current` is machine-generated by model::build, which derives UUID v5 ids
+ // from manifest keys already checked for duplicates; still, guard both sides so a
+ // regression upstream degrades into a typed error rather than silent last-wins diffs.
+ fn ensure_unique_map_uuids(document: &MappingCollectionEnvelope) -> Result<(), ForgeError> {
+     let mut seen = BTreeSet::new();
+     for map in document.mapping_collection.mappings.iter().flat_map(|m| m.maps.iter()) {
+         if !seen.insert(map.uuid.to_string()) {
+             return Err(mapping_error(format!(
+                 "document contains duplicate map UUID '{}'",
+                 map.uuid
+             )));
+         }
+     }
+     Ok(())
  }


══════ F0511 │ src/mapping/baseline.rs:44-48 │ [style · low] ══════
[style · low] Two related issues around the pinned "1.2.3": (1) the literal is hardcoded here
although `validate` owns the supported schema version (`validate/version.rs` documents the accepted
range as 1.2.0–1.2.3 and exposes `SCHEMA_VERSION_USED`), so a future toolchain bump risks this check
drifting out of sync; (2) the exact-equality pin is intentionally stricter than `validate_schema`
(which accepts a baseline declaring 1.2.0 before this check rejects it) — that staggered failure is
confusing for users. Reuse the shared constant from `validate` and add a one-line comment stating
that baselines must be regenerated with the current toolchain version, or accept the same supported
range `validate` does.

      let declared =
          value.pointer("/mapping-collection/metadata/oscal-version").and_then(Value::as_str);
-     if declared != Some("1.2.3") {
-         return Err(mapping_error("baseline must declare OSCAL v1.2.3"));
+     // Baselines must come from the current toolchain generation; sharing the
+     // constant keeps this pin aligned with validate::version support.
+     if declared != Some(validate::SCHEMA_VERSION_USED) {
+         return Err(mapping_error(format!(
+             "baseline must declare OSCAL v{validate::SCHEMA_VERSION_USED}"
+         )));
      }


══════ F0506 │ src/mapping/baseline.rs:57-62 │ [performance · low] ══════
[performance · low] The `MAX_BASELINE_FINDINGS` bound is only enforced after every finding has been
allocated, sorted, and retained — precisely the adversarial-input blowup the limit exists to prevent
(e.g. a hand-edited baseline whose subjects were all deleted produces one finding per item plus gap
churn before the cap trips). Enforce the cap incrementally, e.g. via a small `push`-helper used by
`inspect_items`/`compare_maps`/`compare_gaps` that fails fast past the limit, so memory and CPU stay
bounded for hostile inputs.

-     findings.sort();
-     if findings.len() > MAX_BASELINE_FINDINGS {
+ fn push_finding(
+     out: &mut Vec<ImpactFinding>,
+     finding: ImpactFinding,
+ ) -> Result<(), ForgeError> {
+     out.push(finding);
+     if out.len() > MAX_BASELINE_FINDINGS {
          return Err(mapping_error(format!(
              "baseline impact exceeds the {MAX_BASELINE_FINDINGS} finding limit"
          )));
+     }
+     Ok(())
      }


══════ F0542 │ src/mapping/inventory.rs:185-191 │ [maintainability · low] ══════
[maintainability · low] ResourceInventorySnapshot is constructed twice from (evidence, inventory) —
here and in LoadedResource::snapshot() above — as parallel struct literals that must be kept in sync
by hand. They have already motivated sync-sensitive changes (the staleness comparison needs every
new field added to both sites); a missed site compiles fine yet compares inconsistent shapes.
Extract one shared helper (e.g. fn build_snapshot(evidence: &ResourceEvidence, inventory:
&Inventory) -> ResourceInventorySnapshot) and call it from both paths.

-         let actual = ResourceInventorySnapshot {
+         let actual = build_snapshot(&evidence, &inventory);
+         ...
+     }
+ }
+
+ fn build_snapshot(evidence: &ResourceEvidence, inventory: &Inventory) -> ResourceInventorySnapshot {
+     ResourceInventorySnapshot {
              root_uuid: evidence.root_uuid.clone(),
              document_version: evidence.document_version.clone(),
              oscal_version: evidence.oscal_version.clone(),
              control_ids: inventory.ids_of_type(SubjectType::Control).into_iter().collect(),
              statement_ids: inventory.ids_of_type(SubjectType::Statement).into_iter().collect(),
-         };
+     }
+ }


══════ F0541 │ src/mapping/inventory.rs:388-389 │ [maintainability · low] ══════
[maintainability · low] The eligibility/duplicate registry is mutated before the conflict is
validated: ids.insert unconditionally overwrites any prior entry with the new SubjectType, and the
error path then returns while subjects/excerpts remain untouched, leaving `ids` claiming a type that
the rest of the Inventory no longer agrees with. Harmless today only because load() discards the
whole Inventory on failure; any future caller that tolerates or collects such errors inherits a
silently-corrupted map. Test the key with get()/entry() first, or restore the previous entry before
returning the error so the struct is left coherent.

-     if let Some(existing) = inventory.ids.insert(id.to_string(), subject_type) {
-         let kind = if existing == subject_type { "duplicate" } else { "type-ambiguous" };
+     if let Some(existing) = inventory.ids.get(id) {
+         let kind = if *existing == subject_type { "duplicate" } else { "type-ambiguous" };
+         // Return the error without having mutated `ids`, leaving the Inventory coherent.


══════ F0543 │ src/mapping/inventory.rs:396-399 │ [maintainability · low] ══════
[maintainability · low] Schema-valid but semantically degraded subjects degrade silently: a Control
missing 'title' or a statement part missing 'prose' yields an empty-string excerpt that later
renders in review reports indistinguishable from genuinely empty content, and sibling groups without
ids all collapse onto the inherited parent group path, so controls from distinct anonymous groups
carry identical group metadata and cannot be told apart in framework filtering. Since OSCAL
effectively mandates title/prose for eligible subjects, treating their absence as an error here (or
recording a diagnostic count on Inventory) would surface authoring mistakes at load time instead of
leaking them into reviewer-facing artifacts.


══════ F0478 │ src/mapping/manifest.rs:346-347 │ [maintainability · low] ══════
[maintainability · low] Contract gap: while every other identifier collection here is de-duplicated
(reviewer keys, map keys, subject pairs), `provenance.reviewer_keys` only receives a membership
check, so intra-list duplicates are silently accepted (e.g. `["alice", "alice"]`). For a
deliberately closed/strict schema this lets malformed attribution records through under a contract
that promises to reject them. Track the keys already seen and fail on repeats.

+     let mut cited = BTreeSet::new();
      for key in &manifest.provenance.reviewer_keys {
+         if !cited.insert(key.as_str()) {
+             return Err(mapping_error(format!(
+                 "$.provenance.reviewer_keys duplicates reviewer '{}'",
+                 bounded(key)
+             )));
+         }
          if !reviewers.contains_key(key.as_str()) {


══════ F0479 │ src/mapping/manifest.rs:612-614 │ [performance · low] ══════
[performance · low] Unnecessary clone: every object key in the manifest is cloned only so it
survives past `values.insert(...)` for the rare duplicate-key error, adding one heap allocation per
key — roughly ten thousand keys for a maximum-size manifest's `maps` array. Use the entry API so the
key is moved on success and recovered from the occupied entry on failure.

-             if values.insert(key.clone(), value.0).is_some() {
-                 return Err(de::Error::custom(format!("duplicate object key '{key}'")));
+             match values.entry(key) {
+                 Entry::Occupied(entry) => {
+                     return Err(de::Error::custom(format!(
+                         "duplicate object key '{}'",
+                         entry.key()
+                     )));
+                 }
+                 Entry::Vacant(entry) => {
+                     entry.insert(value.0);
+                 }
              }


══════ F0486 │ src/mapping/mod.rs:122-127 │ [security · low] ══════
[security · low] `std::fs::read` follows symlinks by default and there's no parent-directory or
traversal guard on `path`. In `execute_init`, both source/target come straight from CLI args, but in
`scaffold_resource` nothing distinguishes user-supplied vs manifest-controlled locations if this
helper is ever reused beyond init with a path taken from an untrusted manifest — worth confirming
whether symlinks pointing outside the intended workspace are acceptable for init input files. Low
risk given current call sites pass trusted CLI paths.

- fn scaffold_resource(
-     path: &Path,
-     resolved_catalog: Option<&Path>,
-     output: Option<&Path>,
-     label: &str,
- ) -> Result<manifest::ResourceManifest, ForgeError> {
+ // NOTE: caller-supplied path is opened through std::fs without symlink/traversal restrictions.
+ // If this fn is ever reused to resolve manifest-controlled paths (not just init CLI args), add
+ // containment checks; currently acceptable because both call sites feed it trusted CLI arguments.


══════ F0489 │ src/mapping/mod.rs:465-469 │ [security · low] ══════
[security · low] `same_file_identity` opens files with `File::open`, which requests read access and
fails outright for the destination on Windows if the target lacks a read DACL for this user (rare
but possible), turning an identity check into a spurious hard error. Also, because both `File::open`
calls happen before either handle's info is compared, two handles may reference the same live inode
that was replaced between opens — this is inherent to the check-then-write pattern already noted;
the main improvement here is to keep failure modes explicit: an access-denied error currently aborts
the whole build rather than reporting 'cannot verify destination safety'.

      pub(super) fn same_file(left: &Path, right: &Path) -> io::Result<bool> {
+         // Opened with read access; insufficient-DACL targets surface as a hard error here rather
+         // than being treated as "not the same file". Callers rely on Err to abort unsafe writes.
          let left = identity(&File::open(left)?)?;
          let right = identity(&File::open(right)?)?;
          Ok(left == right)
      }


══════ F0488 │ src/mapping/mod.rs:499-499 │ [bug · low] ══════
[bug · low] `path_identity` on a non-existent path canonicalizes only the parent and then blindly
appends `file_name`. Two problems: (1) if the path has no file_name (e.g., ends in '..' or is a bare
root like 'C:'), `unwrap_or_default()` yields an empty OsStr, so different destinations can collapse
to the same identity string — a genuinely distinct output path could compare equal to an input and
produce a confusing 'aliases a mapping input' error, or (less likely) two truly-aliasing paths could
evade detection. (2) If any intermediate component of a nonexistent destination is itself a symlink
created after this check, canonicalization won't see it. Prefer handling the no-file_name case
explicitly (error or treat as directory-equivalent) instead of silently substituting an empty name.

-         Ok(canonical_parent.join(path.file_name().unwrap_or_default()))
+         let Some(file_name) = path.file_name() else {
+             return Err(mapping_error(format!(
+                 "cannot determine file component of '{}'",
+                 path.display()
+             )));
+         };
+         Ok(canonical_parent.join(file_name))
