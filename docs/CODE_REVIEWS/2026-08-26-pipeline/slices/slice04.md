# Validation slice slice04 — 63 findings
Severity mix: medium×63


══════ F0464 │ src/framework/model.rs:11-12 │ [maintainability · medium] ══════
[maintainability · medium] `status` is a free-form `&'static str` that is copied verbatim into the
versioned, externally published report (every renderer in `src/framework/mod.rs` prints it: markdown
`- Status: {}`, HTML, text YAML). Unlike the four report enums defined below, its value has no
compile-time guard — the existing enum guard test cannot catch a typo such as `"compleet"` — and
nothing formally ties it to the `REPORT_SCHEMA_VERSION` contract. Model it as a small enum following
the same `Serialize` + `as_str()` pattern as
`ChangeClass`/`FindingPriority`/`ReasonCode`/`RequiredAction`.

      pub schema_version: &'static str,
-     pub status: &'static str,
+     pub status: ReportStatus,
+
+ ...
+
+ #[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
+ #[serde(rename_all = "kebab-case")]
+ pub enum ReportStatus {
+     Complete,
+ }
+
+ impl ReportStatus {
+     #[must_use]
+     pub const fn as_str(self) -> &'static str {
+         match self {
+             Self::Complete => "complete",
+         }
+     }
+ }


══════ F0432 │ src/ingest/mod.rs:119-125 │ [security · medium] ══════
[security · medium] TOCTOU race: the size/is-file decision comes from std::fs::metadata, but the
content comes from a later std::fs::read (and extract_pdf_content opens the path yet again via
pdf_extract::extract_text, while canonicalize is a third filesystem touch). The file can be swapped,
grown, or replaced between these calls, so an oversized or different file can slip past the size
check and the fingerprint/source_path can describe different snapshots of the file. Safer pattern:
open the file once and use File::open + handle.metadata() + read_to_end on the same handle, and
derive the canonical path before validating so all checks apply to the same inode.

-     let bytes = std::fs::read(path).map_err(|e| match e.kind() {
-         std::io::ErrorKind::NotFound => ForgeError::FileNotFound { path: path.to_path_buf() },
-         std::io::ErrorKind::PermissionDenied => {
-             ForgeError::PermissionDenied { path: path.to_path_buf() }
+     // Open once and validate/read through the same handle to close the TOCTOU window.
+     let mut file = std::fs::File::open(path).map_err(io_error(path))?;
+     let metadata = file.metadata().map_err(io_error(path))?;
+     if !metadata.is_file() {
+         return Err(ForgeError::NotAFile { path: path.to_path_buf() });
          }
-         _ => ForgeError::Io(e),
-     })?;
+     if metadata.len() > max_size_bytes {
+         return Err(ForgeError::FileTooLarge { path: path.to_path_buf(), size_bytes: metadata.len(), limit_bytes: max_size_bytes });
+     }
+     let mut bytes = Vec::new();
+     file.read_to_end(&mut bytes).map_err(io_error(path))?;


══════ F0434 │ src/ingest/mod.rs:198-199 │ [bug · medium] ══════
[bug · medium] Fidelity loss in the DOCX extractor: trim_text(true) strips leading/trailing
whitespace on every text chunk, so adjacent <w:r> runs like "password " + "reset" silently merge
words whenever whitespace lives at run boundaries (xml:space="preserve" content is destroyed).
Additionally, <w:tab> and <w:br> elements hit the catch-all arm and contribute nothing,
concatenating what should be separate words/columns and separate lines, and CDATA-only content is
dropped entirely because CData events fall into the default arm. Consider disabling trim_text
(handling xml:space properly), emitting a space for w:tab and a newline for w:br, and treating
Event::CData the same as Text.


══════ F0435 │ src/ingest/mod.rs:279-279 │ [bug · medium] ══════
[bug · medium] Table state is tracked with plain booleans, which cannot represent nesting: an inner
<w:tbl> inside a table cell sets in_table=true again, but its End event clears in_table=false while
the outer table is still open, so subsequent outer-cell paragraphs leak into the main output (and
vice versa, rows/cells of outer tables interleave with inner ones). Use a Vec<bool> stack for table
depth (and optionally for row/cell context) so nested structures are rendered consistently instead
of silently corrupting extracted text.

-                 b"w:tbl" => in_table = false,
+                 b"w:tbl" => {
+                     // Track nesting explicitly; plain booleans cannot represent nested tables.
+                     if let Some(open) = table_depth.pop() {
+                         if !open {
+                             /* restore enclosing in_table context */
+                         }
+                     }
+                 }


══════ F0453 │ src/io.rs:116-117 │ [security · medium] ══════
[security · medium] The fallback defeats the function's sole purpose. For paths whose final
component is absent ("/", ".", "..", "sub/..", "C:\\") `Path::file_name()` returns None and the
*entire* input path — absolute, machine-specific, possibly containing usernames/home directories —
is returned and ends up verbatim in OSCAL artifact hrefs, i.e. precisely the absolute-path leak this
helper documents itself as preventing. Handle the None case explicitly (strip the trailing
dot-dot/root component, or return a fixed placeholder/error) instead of echoing the raw input.

-     path.file_name()
-         .map_or_else(|| path.to_string_lossy().into_owned(), |n| n.to_string_lossy().into_owned())
+ pub fn sanitize_artifact_path(path: &Path) -> String {
+     match path.file_name() {
+         Some(name) => name.to_string_lossy().into_owned(),
+         // Paths like "/" or "a/.." have no final component; never echo the raw path.
+         None => {
+             let trimmed = path.to_string_lossy().trim_end_matches(['/', '\\']);
+             Path::new(trimmed)
+                 .file_name()
+                 .map_or_else(|| "artifact".to_owned(), |n| n.to_string_lossy().into_owned())
+         }
+     }
+ }


══════ F0455 │ src/io.rs:24-29 │ [bug · medium] ══════
[bug · medium] Persisting a `NamedTempFile` silently resets the destination's permissions: the temp
file is created with mode 0600 on Unix, and `persist` renames it over `path`, so re-generating an
existing output file (e.g. a previously world-readable catalog.json) downgrades it to 0600 without
warning and drops group/other access that tools consuming these artifacts may rely on. Snapshot the
existing file's mode and apply it to the temp file before persisting.

+     #[cfg(unix)]
+     if let Ok(existing) = std::fs::metadata(path) {
+         use std::os::unix::fs::PermissionsExt;
+         let mode = existing.permissions().mode();
+         let _ = std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(mode));
+     }
      let persisted = tmp.persist(path).map_err(|e| {
          ForgeError::Io(std::io::Error::other(format!(
              "Failed to persist temp file to '{}': {e}",
              path.display()
          )))
      })?;


══════ F0454 │ src/io.rs:42-44 │ [security · medium] ══════
[security · medium] `fs::metadata` follows symlinks and the size is a single sample taken before the
caller opens and reads the file, so this guard is advisory and internally inconsistent:
`regular_file_metadata` below deliberately rejects symlinks, but any reader using `check_file_size`
happily accepts a symlinked path. Worse, symlinked special files (e.g. entries under /proc) report
len()==0 while reading yields unbounded bytes, so MAX_FILE_SIZE can be bypassed entirely. Consider
pairing the check with the opened file handle (`handle.metadata()`) just before/at read time, and
rejecting non-regular files or symlinks consistently with `regular_file_metadata`.


══════ F0458 │ src/io.rs:74-76 │ [maintainability · medium] ══════
[maintainability · medium] `canonicalize` here silently resolves *all* symlinks along the resource
path, contradicting the crate's own stricter convention enforced by `regular_file_metadata`
(symlinks rejected outright) and turning declared-relative references into absolute targets without
notice. Additionally, both io::Error results are flattened into formatted `String`s, destroying the
error kind so upstream callers cannot distinguish not-found from permission-denied. Validate the
resource like sibling APIs do (reject/flag symlinks) and return a typed error carrying the original
`io::Error`.


══════ F0459 │ src/io.rs:87-87 │ [bug · medium] ══════
[bug · medium] When no common prefix exists between the manifest directory and the canonicalized
resource (different filesystem mount/drive, or a nonexistent output dir path), `unwrap_or(target)`
quietly emits an absolute machine-specific path into the OSCAL artifact — which leaks the build
machine's layout (usernames, directory structure) and breaks reproducible/golden-file outputs, the
exact problem this helper exists to prevent. Decide the policy explicitly: either fail loudly when
relativity cannot be established, or normalize to a URI/copy the resource next to the manifest.


══════ F0468 │ src/json_strict.rs:148-150 │ [security · medium] ══════
[security · medium] The duplicate-key error echoes the raw, untrusted key into diagnostics without
escaping (same log-injection exposure as enforce_bounds) and the success path pays an avoidable
`key.clone()` for every single map entry just to detect duplicates. Probe membership first and
report the bounded key, then insert the owned key uncloned: `if values.contains_key(&key) { return
Err(...bounded(&key)...); } values.insert(key, value.0);`.

-             if values.insert(key.clone(), value.0).is_some() {
-                 return Err(de::Error::custom(format!("duplicate object key '{key}'")));
+             if values.contains_key(&key) {
+                 return Err(de::Error::custom(format!(
+                     "duplicate object key '{}'",
+                     bounded(&key)
+                 )));
              }
+             values.insert(key, value.0);


══════ F0471 │ src/json_strict.rs:15-15 │ [maintainability · medium] ══════
[maintainability · medium] `Result<Value, String>` flattens four distinct failure classes —
malformed input, trailing data, duplicate keys, and bounds violations — into free-form prose at what
is a crate-wide reuse boundary, forcing callers (and tests, see the brittle `contains("invalid
trailing ...")` assertions below) to couple to exact message wording and making programmatic
classification impossible. Prefer a small error enum wrapping serde_json::Error with
Display/std::error::Error implementations, keeping the human-readable context behind its Display
output.


══════ F0472 │ src/json_strict.rs:17-18 │ [performance · medium] ══════
[performance · medium] Structural limits are validated only after the entire tree has been decoded
and allocated, so they cannot bound aggregate footprint (array element counts, number of strings,
total byte mass) nor reject oversized payloads early — a huge but shallow `[]` or a multi-megabyte
key materializes fully before rejection, relying entirely on callers capping raw input size
upstream. Also note the local `max_depth` interacts with serde_json's own fixed recursion limit: any
configured depth above it changes which error fires (the deserializer's, not "exceeds maximum JSON
depth"). Worth documenting this contract at the module level or enforcing bounds during descent.


══════ F0469 │ src/json_strict.rs:39-39 │ [performance · medium] ══════
[performance · medium] The breadcrumb path is eagerly materialized with `format!` for every array
element and object member even when all bounds hold, and each level recopies the entire ancestor
path plus the whole raw key, so a successful parse performs O(nodes x depth) String
allocations/copies of pure throwaway work. Pass lightweight segments (e.g. &[&dyn Display] or an
index/name enum stack) down the recursion instead, and render — escaped via `bounded()` — the
printable path only when an error is actually produced.


══════ F0467 │ src/json_strict.rs:53-58 │ [security · medium] ══════
[security · medium] Attacker-controlled object keys are interpolated verbatim into diagnostic path
strings ("{path}.{key}") that propagate through the returned `Result<Value, String>` into logs/error
surfaces. JSON keys legally encode newlines, tabs, other control bytes and ANSI escapes (via \u
escapes), so hostile payloads can forge or pollute multi-line log output; the emitted path also
embeds the key in full with no length cap. This module defines `bounded()` precisely for
caller-controlled diagnostics but never uses it here.

          Value::Object(values) => {
              for (key, child) in values {
-                 enforce_bounds(child, &format!("{path}.{key}"), depth + 1, limits)?;
+                 enforce_bounds(child, &format!("{path}.{}", bounded(key)), depth + 1, limits)?;
              }
              Ok(())
          }


══════ F0470 │ src/json_strict.rs:9-12 │ [bug · medium] ══════
[bug · medium] `max_string_bytes` reads as a per-string bound for the decoded tree, but
enforce_bounds only inspects child values: object keys are never checked, so `{"<huge key>": 1}`
yields a Value whose key exceeds the advertised limit (and whose bytes later flow unchecked into
error paths). Either enforce the same byte limit against keys in the Object arm or document
explicitly in this type that keys are exempt.


══════ F0481 │ src/lifecycle/mod.rs:463-469 │ [bug · medium] ══════
[bug · medium] Residual lost-update window on the --apply path. The byte-for-byte re-read comparison
closes most of the race, but there is no lock spanning load -> prepare -> compare -> replace:
another process can rewrite the record (via the same write_atomic rename) in the instant between
`std::fs::read` returning and `tmp.persist()` replacing the file, and this writer then silently
discards that concurrent update while reporting success. Even matching bytes do not prove inode
continuity, since a competing writer using rename would satisfy the comparison yet still be
overwritten. Recommend holding an advisory lock (flock/FsLock or a dedicated `.lock` sentinel
created with create_new) across the whole read-evaluate-write span, or re-verifying identity via
open file handle (dev/ino on unix, GetFileInformationByHandle on windows) taken before rendering and
reused for the atomic replacement.


══════ F0491 │ src/lifecycle/record.rs:697-700 │ [maintainability · medium] ══════
[maintainability · medium] `context_event_id` derives the tamper-evident `event_id` by UUIDv5-ing
whatever `serde_json` emits for this `Seed` struct. That makes the ID contract fragile in three ways
worth documenting/test-guarding: (1) the seed embeds `generated_artifacts` vectors and other nested
collections **in stored order**, while the validator only enforces sortedness for
`FingerprintSet.generated_artifacts` — equivalent-but-reordered JSON (e.g. reordered `Party.roles`,
differently ordered legacy-event artifacts admitted by the same validator) yields different event
IDs, so canonicalization currently depends on accidental producer ordering rather than an enforced
canonical form; (2) `serde_json::to_vec` output depends on serializer behavior (field declaration
order, float/string escaping, `None` emitted as `null` because there is no `skip_serializing_if`
here) — upgrading or restructuring `TransitionEvent`/these structs, or changing serde settings, will
silently change every recomputed `event_id` and break validation of previously issued records, with
no compile-time signal; (3) there is no regression test pinning the current bytes. Suggest: freeze
this seed layout with golden vector tests plus an explicit `canonical_seed_version` constant/name
embedded in the namespace name string (e.g. derive via a dedicated lifecycle namespace like
`Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"forge.policy-lifecycle/2")`) so future seed changes must opt
into a new namespace instead of corrupting old records.

+ /// Deterministic event-ID seed.
+ ///
+ /// # Wire-shape stability
+ /// The JSON byte layout produced by this struct IS the event-ID contract:
+ /// do not reorder, rename, add, or remove fields, and do not change serde
+ /// attributes without minting a new derivation namespace and documenting a
+ /// migration path (see `FORGE_NAMESPACE_UUID` breaking-change policy).
      #[derive(Serialize)]
-     struct Seed<'a> {
-         schema_version: &'a str,
-         policy: &'a PolicyIdentity,
+ struct Seed<'a> { /* fields unchanged */ }


══════ F0492 │ src/lifecycle/record.rs:881-883 │ [performance · medium] ══════
[performance · medium] Depth limiting happens only here, after `StrictValue` has already fully
recursive-descent-parsed the entire (up to 2 MiB) input — protection against stack exhaustion during
that parse relies solely on serde_json's internal 128-frame recursion ceiling (`RECURSION_LIMIT`,
disabled entirely if the `unbounded_depth` feature is enabled somewhere in the dependency graph, in
which case `[[[[…]]]]` padding crashes the process before this check ever runs). Additionally, on
success paths this function allocates and formats a fresh `path: String` for **every** node
(~hundreds of thousands of throwaway allocations for adversarial input: 2 MiB of `[0],…` at depth ≤
16 is still several hundred thousand visited nodes), which materially slows parsing of hostile
payloads. Recommendations: enforce the depth limit during the streaming pass (e.g. deserialize with
a depth-counting wrapper or track nesting in `visit_seq`/`visit_map`) instead of post-hoc,
assert/document the serde_json feature set (no `unbounded_depth`), and defer path-string
construction to the failing branch (pass an enum/borrowed trail down, materialize `format!` only
right before returning `Err`).


══════ F0493 │ src/lifecycle/record.rs:911-913 │ [test · medium] ══════
[test · medium] Test coverage for this authorization-critical validator is thin: only the state
machine and duplicate-key rejection are pinned. Missing regression tests include: acceptance of each
hand-built fixture class through the full `parse` → `validate` round trip;
`event_id`/`legacy_event_id` golden-vector stability across `/1` and `/2` (guards the
serialization-dependent seed shapes above); approval-role-count accumulation from a mix of asserted
and declaring actors (the `actor` who declares the role himself must count exactly once despite
being inserted twice in `validate_approval`); separation-of-duties violations and the legacy-record
bypass; timestamp ties/out-of-order rejection; supersession `replaced_by` consistency; and every
resource bound (`MAX_PARTIES`, `MAX_ARTIFACTS`, `MAX_EVENTS`, `MAX_ASSERTIONS`,
`MAX_IMPACT_FINDINGS`, `MAX_COLLECTION_ITEMS`, `MAX_DEPTH`, `MAX_RECORD_BYTES`). Please add
table-driven fixtures for these before extending the schema further.


══════ F0473 │ src/main.rs:22-30 │ [maintainability · medium] ══════
[maintainability · medium] This hard-coded 'expected finding' list duplicates knowledge that also
lives in `ForgeError`/`exit_code()` (src/error.rs): every variant here maps to 1, and the variants
have empty Display strings because their reports were already emitted by `cli::execute`. There is no
compile-time link keeping the two sites in sync: when a future review/applicability variant (like
MigrationHasChanges today) is added to `exit_code()` returning 1 but forgotten here, it silently
changes behavior — a non-empty message starts being printed as `Error: …` with exit 1, collapsing
into the generic-failure channel and breaking the CI contract that expected findings stay off
stderr; if its message is also empty, users get a bare `Error: \n` line. Centralize the predicate on
ForgeError (e.g. `is_expected_finding()` kept next to `exit_code`) so both the exit code and the
suppressed-diagnostic decision share one exhaustive match.

-         Err(
-             ForgeError::DiffHasChanges
-             | ForgeError::DriftDetected
-             | ForgeError::MigrationHasChanges
-             | ForgeError::MappingReviewRequired
-             | ForgeError::LifecycleActionRequired
-             | ForgeError::ApplicabilityReviewRequired
-             | ForgeError::FrameworkReviewRequired,
-         ) => ExitCode::from(1u8),
+ // src/error.rs — single source of truth next to exit_code()
+ impl ForgeError {
+     /// "Expected finding" outcomes: exit code 1, but their report was already
+     /// printed by the command and they must NOT surface as runtime errors.
+     #[must_use]
+     pub fn is_expected_finding(&self) -> bool {
+         matches!(
+             self,
+             Self::DiffHasChanges
+                 | Self::DriftDetected
+                 | Self::MigrationHasChanges
+                 | Self::MappingReviewRequired
+                 | Self::LifecycleActionRequired
+                 | Self::ApplicabilityReviewRequired
+                 | Self::FrameworkReviewRequired
+         )
+     }
+ }
+
+ // src/main.rs
+ match cli::execute(&cli) {
+     Ok(()) => ExitCode::SUCCESS,
+     Err(e) if e.is_expected_finding() => ExitCode::from(exit_code(&e)),
+     Err(e) => {
+         eprintln!("Error: {e}");
+         ExitCode::from(exit_code(&e))
+     }
+ }


══════ F0474 │ src/main.rs:31-34 │ [other · medium] ══════
[other · medium] The two arms of this match collapse to the same process exit code 1: genuine
failures mapped by `exit_code()` to 1 (e.g. `Io`, `FileNotFound`, `Serialization`,
`OscalCliExecution`, `RoundTripFailed`) land in this arm, while the benign diff/drift/review
outcomes take the arm above. Shell/CI wrappers keying on the exit status alone therefore cannot
distinguish 'changes/review required' from a real input or I/O failure — they must scrape stderr,
yet the distinguishing signal (the `Error:` line) is exactly what the first arm suppresses. Either
reserve a dedicated exit-code range for expected findings (so `exit_code` never returns 1 for them)
or document prominently in the CLI help/README that exit 1 is overloaded and stderr parsing is
required.


══════ F0507 │ src/mapping/baseline.rs:112-116 │ [bug · medium] ══════
[bug · medium] `verify_integrity` deliberately handles multi-mapping documents (it labels props
`mappings[{index}]...` and walks every entry), and `compare_maps` diffs maps across *all* baseline
mappings — yet `compare_resources` and `compare_gaps` diff only `mappings.first()` and return early.
For a baseline carrying more than one mapping entry (structurally permitted by the deserialized
`Vec<OscalMapping>`), provenance/resource/gap changes beyond the first mapping silently go
unreported while its maps still produce findings: an incomplete, inconsistent impact analysis that
looks authoritative. Either iterate all mapping index pairs, or fail closed on documents with != 1
mapping entry so nothing is analyzed partially.

- ) {
-     let Some(old) = baseline.mapping_collection.mappings.first() else { return };
-     let Some(new) = current.mapping_collection.mappings.first() else { return };
-     if old.method != new.method
-         || old.matching_rationale != new.matching_rationale
+ if baseline.mapping_collection.mappings.len() != 1
+     || current.mapping_collection.mappings.len() != 1
+ {
+     return Err(mapping_error(
+         "baseline impact analysis requires exactly one mapping entry",
+     ));
+ }


══════ F0505 │ src/mapping/baseline.rs:19-19 │ [bug · medium] ══════
[bug · medium] The doc comment promises "append findings", but `report.findings = findings` discards
any findings already in the report. Today the only caller (`mapping::prepare`) passes a freshly
constructed report whose `findings` vec is still empty, so the divergence is latent rather than
active data loss — but the documented contract and implementation disagree, and any future caller
that accumulates findings before invoking `analyze` will silently lose them (with `--fail-on any`
acting on truncated results). Either honor the doc (`report.findings.extend/findings.append(...)`)
or correct the doc to state replace semantics.

- /// Compare a valid FORGE mapping baseline with current resources and append findings.
+ /// Compare a valid FORGE mapping baseline with current resources and
+ /// extend the report's finding list.
+ ...
+     report.findings.extend(findings);
+     Ok(())


══════ F0510 │ src/mapping/baseline.rs:288-290 │ [bug · medium] ══════
[bug · medium] `subject_keys` compares sources/targets as *ordered* `Vec`s, so merely reordering
subjects fires `relationship_changed` with the message "relationship, rationale, or subject set
changed" — a semantically false claim and a guaranteed false-positive/noise generator for any
reserialized-but-equivalent document, since subject lists carry no documented ordering semantics
(identity is `(SubjectType, id_ref)`, as `Inventory` lookups show). Compare identity multisets (or
sort before comparing) so only genuine membership/content changes are reported.

- fn subject_keys(items: &[MappingItem]) -> Vec<(SubjectType, &str)> {
-     items.iter().map(|item| (item.subject_type, item.id_ref.as_str())).collect()
+ use std::collections::BTreeSet;
+
+ fn subject_keys(items: &[MappingItem]) -> BTreeSet<(SubjectType, String)> {
+     items
+         .iter()
+         .map(|item| (item.subject_type, item.id_ref.clone()))
+         .collect()
  }


══════ F0509 │ src/mapping/baseline.rs:301-308 │ [security · medium] ══════
[security · medium] `prop_value` resolves FORGE-namespaced integrity evidence first-wins: if a
baseline item (or resource/map metadata) carries two conflicting props named
`subject-sha256`/`raw-sha256`/`reviewed-at`, `require_prop` passes and the diff silently compares
whichever copy serializes first. These fingerprints are the tamper-evidence contract, and a seeded
second prop can therefore mask a genuine content change or fabricate one. The sibling framework
analyzer explicitly rejects such ambiguity (`"{path} contains ambiguous duplicate FORGE property
'{name}'"` in src/framework/analysis.rs), so this module is both weaker than project precedent and
weaker than its own threat model. Treat duplicate (name, ns) FORGE props on the baseline as fatal,
mirroring `require_prop`.

- fn prop_value<'a>(props: &'a [OscalProp], name: &str) -> Option<&'a str> {
-     props
-         .iter()
-         .find(|prop| {
+ fn require_unique_prop(props: &[OscalProp], name: &str, path: &str) -> Result<&str, ForgeError> {
+     let mut values = props.iter().filter(|prop| {
              prop.name == name && prop.ns.as_deref() == Some(super::inventory::FORGE_MAPPING_NS)
+     });
+     let value = values.next();
+     if values.next().is_some() {
+         return Err(mapping_error(format!(
+             "baseline {path} contains ambiguous duplicate FORGE property '{name}'"
+         )));
+     }
+     value.map(|prop| prop.value.as_str()).ok_or_else(|| {
+         mapping_error(format!("baseline {path} lacks required FORGE property '{name}'"))
          })
-         .map(|prop| prop.value.as_str())
  }


══════ F0537 │ src/mapping/inventory.rs:185-191 │ [bug · medium] ══════
[bug · medium] Content-drift blindness: the regenerated-inventory equivalence check only covers root
uuid, versions and the control/statement id sets. Per-subject fingerprints (title/prose digests),
excerpts, ineligible_parts and group hierarchies are excluded, so an artifact whose sha256 was never
pinned (expected_sha256 is optional in ResourceManifest) can be edited in place — retitled controls,
rewritten statement prose, reshuffled groups — and still satisfy manifest.inventory, silently
republishing mappings with new subject-sha256 evidence that no reviewer attested. Including a stable
digest over the fingerprint map in ResourceInventorySnapshot (and/or refusing inventories built
solely on this weak check) would make the staleness detector catch semantic content changes, not
just structural id changes.

          let actual = ResourceInventorySnapshot {
              root_uuid: evidence.root_uuid.clone(),
              document_version: evidence.document_version.clone(),
              oscal_version: evidence.oscal_version.clone(),
              control_ids: inventory.ids_of_type(SubjectType::Control).into_iter().collect(),
              statement_ids: inventory.ids_of_type(SubjectType::Statement).into_iter().collect(),
+             // Deterministic digest over id -> fingerprint pairs so title/prose edits invalidate.
+             fingerprint_digest: Some(inventory.fingerprint_digest()),
          };


══════ F0539 │ src/mapping/inventory.rs:219-220 │ [bug · medium] ══════
[bug · medium] Off-by-one accounting mixes two error classes: the OSCAL-version inspection failure
is pushed onto the same vec as schema errors, and only afterwards is `truncated` decided. With
exactly MAX_SCHEMA_ERRORS schema errors plus an unsupported oscal-version declaration, errors.len()
reaches MAX_SCHEMA_ERRORS + 1, the version diagnostic lands at index MAX_SCHEMA_ERRORS, and
truncate() silently deletes it — the reviewer-facing error then claims only 'additional schema
errors omitted' while the fact that the document declares an unsupported OSCAL version has vanished
entirely. Track the schema-error count taken from the iterator before appending the version error,
and keep the version error exempt from truncation so it is always surfaced.

          let truncated = errors.len() > MAX_SCHEMA_ERRORS;
          errors.truncate(MAX_SCHEMA_ERRORS);
+         // Re-run version inspection after truncation and report its error unconditionally,
+         // so a genuine version mismatch cannot be evicted by schema-error volume.


══════ F0538 │ src/mapping/inventory.rs:408-414 │ [bug · medium] ══════
[bug · medium] Fingerprint determinism is only incidental: canonical_subject_sha256 hashes whatever
byte layout serde_json::to_vec emits for a Value, which is BTreeMap-ordered by default but switches
to insertion-order IndexMap whenever the `preserve_order` cargo feature is enabled anywhere in the
dependency graph (feature unification, outside this crate's control). Two semantically identical
subjects whose JSON members were reordered (e.g. a catalog round-tripped through jq or re-serialized
by another toolchain) then produce different "canonical" fingerprints, breaking the
deterministic/equivalence contract the review tooling depends on; number formatting diverges
similarly ("1" vs "1.0", "1e2" vs "100"). Hash a self-defined canonical form instead (recursive
key-sorted serialization writer or RFC 8785 JCS) so reproducibility does not hinge on build flags.
Secondary cost: value.clone() deep-clones the entire subtree per subject — up to 100k subjects —
largely to support the strip pass; an in-place strip on the already-owned Value would halve peak
allocation.

  fn canonical_subject_sha256(value: &Value) -> Result<String, ForgeError> {
-     let mut canonical = value.clone();
-     strip_forge_fingerprint_props(&mut canonical);
-     let bytes = serde_json::to_vec(&canonical)
-         .map_err(|error| mapping_error(format!("subject fingerprint failed: {error}")))?;
-     Ok(sha256(&bytes))
+     // Serialize with keys force-sorted regardless of the serde_json `preserve_order`
+     // feature so fingerprints stay stable across builds and reserialization tools.
+     let mut buffer = Vec::new();
+     write_canonical_json(&mut buffer, value)?;
+     Ok(sha256(&buffer))
  }
+
+ fn write_canonical_json(out: &mut Vec<u8>, value: &Value) -> Result<(), ForgeError> { /* sorted-object serializer */ }


══════ F0540 │ src/mapping/inventory.rs:448-452 │ [security · medium] ══════
[security · medium] Two weaknesses in this guard: (1) TOCTOU — check_file_size stat()s the path and
then std::fs::read slurps whatever occupies it at read time, so a file swapped or grown between the
two calls (CI workspace races, attacker-influenced inputs) is fully ingested past the 50 MiB DoS
bound; the check buys no protection for the read itself. Open the File once and enforce the limit on
the same descriptor (File::open + take(limit + 1) + length assertion) to close the window. (2)
Neither this helper nor its callers reject symlinks or verify containment: unlike the framework
loader, which pre-flights companions with io::regular_file_metadata (symlink-rejecting), the mapping
path happily follows artifact/resolved_catalog symlinks pointing anywhere on disk, hashing foreign
files and echoing their content fragments back through schema-error text; combined with unchecked
'..'-bearing manifest paths this weakens the local-input sandbox this module otherwise advertises.

  fn read_bounded_json(path: &Path, label: &str) -> Result<Vec<u8>, ForgeError> {
-     io::check_file_size(path, io::MAX_FILE_SIZE)
+     // Reject symlinks up front, mirroring the framework loader's hygiene for companions.
+     io::regular_file_metadata(path, label)
+         .map_err(|message| mapping_error(message))?;
+     let file = std::fs::File::open(path)
          .map_err(|error| mapping_error(format!("{label}: {error}")))?;
-     std::fs::read(path).map_err(|error| mapping_error(format!("{label}: {error}")))
+     // Bound the read on the held descriptor so a post-stat swap cannot evade the limit.
+     let mut limited = file.take(io::MAX_FILE_SIZE + 1);
+     let mut bytes = Vec::new();
+     limited.read_to_end(&mut bytes)
+         .map_err(|error| mapping_error(format!("{label}: {error}")))?;
+     if bytes.len() as u64 > io::MAX_FILE_SIZE {
+         return Err(mapping_error(format!("{label}: file exceeds {} bytes", io::MAX_FILE_SIZE)));
+     }
+     Ok(bytes)
  }


══════ F0477 │ src/mapping/manifest.rs:510-521 │ [performance · medium] ══════
[performance · medium] Performance/log-hygiene: this function eagerly `format!`s a fresh heap
`String` for every array element and object member even on the success path. A legal manifest near
the 2 MiB limit can hold hundreds of thousands of JSON nodes, so validating burns O(nodes)
short-lived allocations (each including the full accumulating path prefix) purely to have a label
ready for an error that usually never occurs. Walk with a reusable buffer that is truncated after
each child instead. Additionally, object keys are interpolated raw and unbounded into error messages
(contrast with `bounded()` used everywhere else), so attacker-controlled keys with newlines/quotes
can forge diagnostic output — render and escape the key lazily, only on failure.

+ // Take a growable path buffer and push/truncate segments in place; render the
+ // failing segment only when an error is actually produced.
+ // Caller seeds the buffer with "$".
          Value::Array(values) => {
              for (index, child) in values.iter().enumerate() {
-                 enforce_value_bounds(child, &format!("{path}[{index}]"), depth + 1)?;
+         let base = path.len();
+         let _ = core::write!(path, "[{index}]");
+         enforce_value_bounds_buf(child, path, depth + 1)?;
+         path.truncate(base);
              }
              Ok(())
          }
          Value::Object(values) => {
              for (key, child) in values {
-                 enforce_value_bounds(child, &format!("{path}.{key}"), depth + 1)?;
+         if child.len_bound_exceeded(depth + 1) /* cheap pre-check */ {
+             return Err(mapping_error(format!(
+                 "$.{}.{} exceeds maximum bounds",
+                 bounded(key), /* escaped like every other diagnostic */
+                 describe_kind(child)
+             )));
+         }
+         let base = path.len();
+         let _ = core::write!(path, ".{}", bounded(key));
+         enforce_value_bounds_buf(child, path, depth + 1)?;
+         path.truncate(base);
              }
              Ok(())
          }


══════ F0485 │ src/mapping/mod.rs:128-131 │ [security · medium] ══════
[security · medium] Same TOCTOU pattern as `prepare()` above: this helper feeds `inventory::load` a
throwaway manifest whose only purpose is to reuse the load path for fingerprint/snapshot
computation, then re-serializes the raw bytes just to hash them. A file that grows between the
metadata check and the read (or is replaced by a symlink after validation) can still bypass
MAX_FILE_SIZE. Add a post-read length assertion here too, or centralize bounded reads in io:: so all
callers benefit.

      io::check_file_size(path, io::MAX_FILE_SIZE)
          .map_err(|error| mapping_error(format!("{label} resource: {error}")))?;
      let bytes =
          std::fs::read(path).map_err(|error| mapping_error(format!("{label} resource: {error}")))?;
+     if bytes.len() as u64 > io::MAX_FILE_SIZE {
+         return Err(mapping_error(format!(
+             "{label} resource grew past the {io.MAX_FILE_SIZE} byte limit between check and read"
+         )));
+     }


══════ F0483 │ src/mapping/mod.rs:168-168 │ [security · medium] ══════
[security · medium] The generated scaffold records `resolved_catalog_attestation: false`, but
`manifest::validate_resource` only accepts the companion path because it requires attestation ==
Some(true); a fresh skeleton containing a Profile therefore fails validation on its own documented
workflow ('forge mapping build' on the init output errors with 'resolved_catalog_attestation must be
true'). Meanwhile nothing here lets an unattested draft ship (validation is gated separately in
prepare()), so the real problem is inconsistency, not privilege escalation. Either document that
`init` output intentionally fails parsing until a reviewer flips this field to true, or use
Some(true) so the scaffold round-trips; as written the false value silently encodes 'not yet
reviewed' for a file whose whole purpose is to be edited by a reviewer, which is subtle and easy to
misread as broken tooling.

+ // Intentional: `init` never asserts a human has reviewed the Profile's companion catalog.
+ // Documented contract: the scaffold fails manifest::parse() during `build`/`check` until
+ // a reviewer sets resolved_catalog_attestation = true after verifying the companion.
          resolved_catalog_attestation: resolved_catalog.map(|_| false),


══════ F0484 │ src/mapping/mod.rs:188-191 │ [bug · medium] ══════
[bug · medium] Both `prepare()` and `inventory::read_bounded_json` rely on `io::check_file_size`
followed immediately by unbounded `std::fs::read`. Between the metadata check and the read, a file
can grow (or be swapped via symlink), so MAX_MANIFEST_BYTES / MAX_FILE_SIZE are advisory rather than
guaranteed — an attacker able to race writes can exhaust memory with an arbitrary-size allocation.
There is also no defensive check after reading (bytes.len() <= limit), so the size bound isn't even
asserted post-hoc.

      io::check_file_size(manifest_path, manifest::MAX_MANIFEST_BYTES)
          .map_err(|error| mapping_error(format!("manifest: {error}")))?;
      let manifest_bytes = std::fs::read(manifest_path)
          .map_err(|error| mapping_error(format!("manifest: {error}")))?;
+     // Re-check after read: TOCTOU gap means metadata-then-read alone doesn't enforce the cap.
+     if manifest_bytes.len() as u64 > manifest::MAX_MANIFEST_BYTES {
+         return Err(mapping_error(format!(
+             "manifest exceeded {} bytes between size check and read",
+             manifest::MAX_MANIFEST_BYTES
+         )));
+     }


══════ F0487 │ src/mapping/mod.rs:361-369 │ [maintainability · medium] ══════
[maintainability · medium] `MappingFailOn::Stale` gates on the literal "subject_type_changed", while
`SubjectChange` gates only on "subject_changed". Both codes are produced as bare string literals in
baseline.rs (lines 214/216/233). This split is consistent with the PRD's classification of finding
codes, but there is no single source of truth — every one of these inline strings could silently
drift from what baseline.rs emits, causing fail-on to miss its gate. Define these as pub consts on
FindingCode (or use a typed enum) so both producer and consumer reference the same names.

-         MappingFailOn::Stale => report.findings.iter().any(|finding| {
-             matches!(finding.code.as_str(), "stale_reference" | "subject_type_changed")
-         }),
-         MappingFailOn::SubjectChange => {
-             report.findings.iter().any(|finding| finding.code == "subject_changed")
-         }
-         MappingFailOn::GapIncrease => {
-             report.findings.iter().any(|finding| finding.code == "new_gap")
-         }
+ // Prefer shared constants over inline literals, e.g.:
+ //   pub const CODE_STALE_REFERENCE: &str = "stale_reference";
+ //   pub const CODE_SUBJECT_TYPE_CHANGED: &str = "subject_type_changed";
+ //   pub const CODE_SUBJECT_CHANGED: &str = "subject_changed";
+ //   pub const CODE_NEW_GAP: &str = "new_gap";
+ // defined once (e.g., src/mapping/baseline.rs) and reused at both emit and match sites.


══════ F0495 │ src/mapping/model.rs:413-419 │ [maintainability · medium] ══════
[maintainability · medium] The machine-readable gates are fabricated constants rather than derived
results: manifest_valid/resources_valid/references_valid are unconditionally true (truthful only
because callers happen to invoke build strictly after manifest::parse and inventory::load succeed,
and reference validation inside build_items failed fast), mapping_schema_valid is unconditionally
false and then mutated post-hoc by src/mapping/mod.rs:204
('product.report.validation.mapping_schema_valid = true'), and findings is always empty because
baseline::analyze mutates the same report later. Nothing beside convention enforces this ordering:
any other caller of the public build() (or a refactor of prepare()) silently produces a report
claiming schema-invalid or omitting baseline findings. Also note status is pinned to "complete" even
when later stages append findings. Suggest encapsulating the invariant — e.g., a ValidationSummary
constructor/document next to the struct stating that three flags mean 'all preceding stages
succeeded', and flipping mapping_schema_valid at the point schema validation runs (pass the result
in), instead of spreading mutable truth across modules.

+         // Contract: reached only after manifest::parse and inventory::load succeeded;
+         // build_items reference validation above failed fast on any bad subject.
+         // mapping_schema_valid starts false and MUST be set true by the caller only
+         // after inventory::validate_schema(OscalModelType::Mapping) succeeds.
          validation: ValidationSummary {
              manifest_valid: true,
              resources_valid: true,
              references_valid: true,
              mapping_schema_valid: false,
          },
          findings: Vec::new(),


══════ F0496 │ src/mapping/model.rs:478-481 │ [bug · medium] ══════
[bug · medium] The equality-based cap check silently discards all remaining excerpts with no
truncation marker anywhere in the report. Because ["source", source] is enumerated first, an
inventory pair that together exceeds MAX_REPORT_EXCERPTS yields a full slate of source excerpts and
zero target excerpts — a biased, unrepresentable sample — yet the report still claims status
"complete". Every other bounded-size guard in this crate treats overflow as fatal instead
(framework/analysis.rs:1111 returns ForgeError past MAX_FINDINGS; mapping/baseline.rs:58 does the
same past MAX_BASELINE_FINDINGS). Prefer failing loudly like siblings, or at minimum emit an
ImpactFinding/report field recording how many excerpts were dropped.


══════ F0497 │ src/mapping/model.rs:482-487 │ [bug · medium] ══════
[bug · medium] unwrap_or("") converts a legitimately absent inventory excerpt into a misleading
empty-string excerpt: the recipient cannot distinguish 'subject whose text is empty/unavailable'
from 'excerpt was never captured', which violates the spirit of masking swallowed Options.
Inventory::excerpt returns Option precisely so absence is observable. Either skip absent entries
entirely or mark them explicitly in ReportExcerpt.

+                 let Some(excerpt) = inventory.excerpt(subject_type, &id) else {
+                     continue; // or push an excerpt marked missing so absence stays visible
+                 };
                  excerpts.push(ReportExcerpt {
                      side,
                      subject_type,
-                     excerpt: inventory.excerpt(subject_type, &id).unwrap_or("").to_string(),
+                     excerpt: excerpt.to_string(),
                      id,
                  });


══════ F0498 │ src/mapping/model.rs:577-582 │ [bug · medium] ══════
[bug · medium] No duplicate detection exists within a single sources/targets list: repeating the
same (subject_type, id_ref) in one map emits duplicate MappingItems carrying identical
subject-sha256 props into the OSCAL artifact, while the report's participation counting collapses
them to one entry via the shared referenced BTreeSet (build() initializes it globally precisely
because the same control may legitimately repeat across different maps). Inline validation already
exists in the same shape — build()'s control-only statement check fails with
"$.mapping.maps[{index}] …" — so mirror that here with a per-list (or per-map) local seen-set,
otherwise manifests shipping accidental repetitions validate cleanly yet serialize contradictory
data.

+             // Local set (per build_items call), NOT the shared `referenced` set:
+             // re-referencing a subject across different maps is legal.
+             if !local_seen.insert((subject.subject_type, subject.id_ref.clone())) {
+                 return Err(mapping_error(format!(
+                     "{path}[{index}] repeats '{}' '{}'",
+                     subject.subject_type.as_str(),
+                     bounded(&subject.id_ref)
+                 )));
+             }
              referenced.insert((subject.subject_type, subject.id_ref.clone()));
              Ok(MappingItem {
                  subject_type: subject.subject_type,
                  id_ref: subject.id_ref.clone(),
                  props: vec![forge_prop("subject-sha256", fingerprint)],
              })


══════ F0520 │ src/migration/engine.rs:591-595 │ [bug · medium] ══════
[bug · medium] Contradictory/too-coarse diagnostics for the sole reconciliation backstop. Six
unrelated invariant violations (duplicated IDs on either side, missing IDs on either side, or
summary counters disagreeing with totals) collapse into the constant string "internal reconciliation
invariant failed", discarding which ID or counter diverged. Since this error indicates a classifier
bug rather than user input error, the message is the primary forensic signal for incidents; compare
with every sibling module here (applicability, lifecycle, mapping) that reports the specific
offending identifier. Emitting even truncated sample IDs makes failures triageable.

+     let duplicated_old: Vec<_> = (0..actual_old.len())
+         .flat_map(|index| {
+             if actual_old.iter().skip(index + 1).any(|id| *id == actual_old[index]) {
+                 Some(actual_old[index])
+             } else {
+                 None
+             }
+         })
+         .take(3)
+         .collect();
+     let missing_old: Vec<_> =
+         expected_old.difference(&actual_old_set).take(3).copied().collect();
+     let missing_new: Vec<_> =
+         expected_new.difference(&actual_new_set).take(3).copied().collect();
+     if !duplicated_old.is_empty()
+         || !missing_old.is_empty()
+         || !missing_new.is_empty()
+         || actual_old_set != expected_old
+         || actual_new_set != expected_new
+         || summary.old_requirements.total() != summary.total_old
+         || summary.new_requirements.total() != summary.total_new
      {
-         return Err(ForgeError::MigrationError(
-             "internal reconciliation invariant failed".to_string(),
-         ));
+         return Err(ForgeError::MigrationError(format!(
+             "internal reconciliation invariant failed: \
+              missing_old={missing_old:?} missing_new={missing_new:?} \
+              duplicated_old={duplicated_old:?} old_counter={} wanted={} new_counter={} wanted={}",
+             summary.old_requirements.total(),
+             summary.total_old,
+             summary.new_requirements.total(),
+             summary.total_new,
+         )));
      }


══════ F0524 │ src/migration/formatter.rs:130-141 │ [performance · medium] ══════
[performance · medium] escape_controls heap-allocates a fresh Vec<char> for every single character
via flat_map (a 1-element vec![character] for normal text, another collect for escape groups), so
rendering a report performs O(total free-text length) tiny allocations. Labels, section paths, and
reviewer rationales (up to 64 KiB each from successor maps) flow through here for every entry/item,
so large reports pay millions of short-lived allocations. Collect once into a
String::with_capacity(value.len()) and push characters directly, with a copy-only fast path when the
string contains no controls.

  fn escape_controls(value: &str) -> String {
-     value
-         .chars()
-         .flat_map(|character| {
+     if !value.chars().any(char::is_control) {
+         return value.to_string();
+     }
+     let mut escaped = String::with_capacity(value.len());
+     for character in value.chars() {
              if character.is_control() {
-                 character.escape_default().collect::<Vec<_>>()
+             escaped.extend(character.escape_default());
              } else {
-                 vec![character]
+             escaped.push(character);
              }
-         })
-         .collect()
+     }
+     escaped
  }


══════ F0515 │ src/migration/inventory.rs:116-119 │ [other · medium] ══════
[other · medium] On duplicate detection the error reports only the offending ID and neither the file
label nor any requirement location, even though both are available in InventoryRequirement
(file_label, section_path, line). For a migration-integrity failure whose whole purpose is
traceability, include at least the source label and the colliding entries' locations so operators
can find and remediate the conflict instead of grepping the corpus for the ID.

              return Err(ForgeError::MigrationError(format!(
-                 "stable-ID integrity anomaly for '{}'",
-                 pair[0].stable_id
+                 "stable-ID integrity anomaly for '{}': '{}' at {}:{}:{} collides with '{}' at {}:{}:{}",
+                 pair[0].stable_id,
+                 pair[0].normalized_text_sha256,
+                 pair[0].location.file_label,
+                 pair[0].location.section_path,
+                 pair[0].location.line.map_or(0, |line| line),
+                 pair[1].stable_id,
+                 pair[1].normalized_text_sha256,
+                 pair[1].location.file_label,
+                 pair[1].location.section_path,
+                 pair[1].location.line.map_or(0, |line| line)
              )));


══════ F0501 │ src/migration/inventory.rs:16-17 │ [maintainability · medium] ══════
[maintainability · medium] Flattening the `prepare_document` result into
`ForgeError::MigrationError(error.to_string())` destroys both the causal chain and the typed error
classification. `prepare_document` already returns `ForgeError`; re-wrapping every variant (e.g.
`FileTooLarge`, `NoStructureDetected`) as a free-form string means `error.rs`'s exit-code mapping
(Input/IO=1, Parse=2, ...) can no longer classify these failures correctly, and callers cannot match
on variants. Since the inner type is already `ForgeError`, propagate it directly with `?`, and
reserve `MigrationError(String)` for genuinely migration-local defects. If a dedicated migration
error type is introduced later, carry the original error as `#[source]` and include the source label
for actionability.

-     let document = crate::pipeline::prepare_document(path, max_size_bytes)
-         .map_err(|error| ForgeError::MigrationError(error.to_string()))?;
+     let document = crate::pipeline::prepare_document(path, max_size_bytes)?;


══════ F0503 │ src/migration/inventory.rs:19-21 │ [bug · medium] ══════
[bug · medium] The format — and thus every recorded requirement's `location_basis` — is inferred
solely from the filename extension. If a PDF is renamed to `.md` (or vice versa), the inventory will
persist `SourceLine` while `source_line` values actually refer to normalized-extracted-text lines,
silently corrupting the provenance metadata that downstream consumers rely on to locate requirements
in the original artifact. Consider deriving `location_basis` from facts established during ingestion
(the ingested record knows whether raw source lines survive) or at minimum verifying the sniffed
content type against the declared extension rather than trusting either alone.


══════ F0504 │ src/migration/inventory.rs:77-80 │ [bug · medium] ══════
[bug · medium] `collect_section` recurses once per nesting level of the parsed section tree with no
depth bound. While the shared Markdown parser bounds structure by heading levels (H1–H6), other
modules in this codebase defensively cap traversal depth (see `MAX_SECTION_DEPTH = 50` in
`src/parse/atomize.rs` and `component_definition.rs`, applied precisely because host-supplied
documents are treated as a DoS surface); this walker skips that protection, so a pathologically
structured document could exhaust the call stack and abort the process instead of surfacing a
recoverable `ForgeError`. Add a depth parameter capped consistently with the rest of the crate, or
convert to an explicit `Vec`-based stack like `extract_sections` does.

-     for child in &section.children {
-         let child_path = format!("{section_path}/{}", child.title);
-         collect_section(child, &child_path, file_label, location_basis, output)?;
+     const MAX_SECTION_DEPTH: usize = 50; // consistent with parse/atomize.rs
+
+     fn collect_section(
+         section: &PolicySection,
+         section_path: &str,
+         file_label: &str,
+         location_basis: LocationBasis,
+         output: &mut Vec<InventoryRequirement>,
+         depth: usize,
+     ) -> Result<(), ForgeError> {
+         anyhow_guard_or_err(depth > MAX_SECTION_DEPTH)?;
+         ...
      }


══════ F0516 │ src/migration/inventory.rs:91-95 │ [maintainability · medium] ══════
[maintainability · medium] When the shared pipeline yields a requirement without a stable ID, this
error discards all localization context: it names neither the file being migrated nor the
requirement's section/title/text (nor its source_line). In a multi-document migration run the
operator cannot tell which of many inputs produced the defect. Include file_label (available in
scope) and the requirement's title/snippet or source_line in the message.

      let stable_id = requirement.stable_id.clone().ok_or_else(|| {
-         ForgeError::MigrationError(
-             "shared pipeline returned a requirement without a stable ID".to_string(),
-         )
+         ForgeError::MigrationError(format!(
+             "shared pipeline returned a requirement without a stable ID in '{}', \
+              section '{}', source line {}",
+             file_label, section.title, requirement.source_line
+         ))
      })?;


══════ F0527 │ src/migration/mod.rs:37-39 │ [maintainability · medium] ══════
[maintainability · medium] The doc comment promises that "all analysis failures are normalized to
ForgeError::MigrationError" for the CLI's exit-2 contract, but the function body uses bare '?'
propagation and enforces nothing locally. The invariant actually holds only because deep inside
inventory::build_inventory / successor::load / engine::classify every variant happens to be
rewritten today (e.g., inventory.rs collapses the entire shared pipeline's dozens of ForgeError
variants through a single `.to_string()` map_err). Nothing structural prevents a future `?` inside
one of those helpers from leaking e.g. ForgeError::FileNotFound past this facade, silently turning
the documented exit-2 behavior into exit-1. Normalize at this boundary so the advertised contract is
guaranteed by construction.

-     let old = inventory::build_inventory(old_path, max_size_bytes)?;
-     let new = inventory::build_inventory(new_path, max_size_bytes)?;
-     let successor_map = successor_map_path.map(successor::load).transpose()?;
+     let old =
+         inventory::build_inventory(old_path, max_size_bytes).map_err(normalize_to_migration)?;
+     let new =
+         inventory::build_inventory(new_path, max_size_bytes).map_err(normalize_to_migration)?;
+     let successor_map = successor_map_path
+         .map(successor::load)
+         .map(|result| result.map_err(normalize_to_migration))
+         .transpose()?;
+     engine::classify(old, new, successor_map.as_ref())
+ }
+
+ /// Guarantees this module's documented exit-2 contract at its boundary.
+ fn normalize_to_migration(error: ForgeError) -> ForgeError {
+     match error {
+         error @ ForgeError::MigrationError(_) => error,
+         other => ForgeError::MigrationError(other.to_string()),
+     }
+ }


══════ F0555 │ src/migration/successor.rs:153-154 │ [bug · medium] ══════
[bug · medium] Uniqueness is enforced only within each role, so chained and cyclic graphs validate
successfully: {"old_ids":["A"],"new_ids":["B"]} plus {"old_ids":["B"],"new_ids":["A"]} passes (A and
B each occur once per role, and neither declaration self-maps). Downstream code that follows
successor links transitively will loop forever on such cycles, and overlapping multi-hop
declarations contradict the documented "conflicting declarations" guarantee by making the
reviewer-approved redirection ambiguous. After the validation loop, reject maps where any identifier
acts as both a source and a destination across different relationships (this forbids chains and
cycles alike), or run explicit cycle detection over the consolidated graph.

-     let mut used_old = BTreeSet::new();
-     let mut used_new = BTreeSet::new();
+     if let Some(id) = used_old.intersection(&used_new).next() {
+         return Err(error(format!(
+             "identifier '{}' participates in chained or cyclic declarations; an entry must not \
+              be both a source and a destination",
+             crate::json_strict::bounded(id)
+         )));
+     }


══════ F0556 │ src/migration/successor.rs:160-160 │ [maintainability · medium] ══════
[maintainability · medium] The self-map detection depends on a hidden invariant: normalize_ids()
sorted new_ids in place two statements above, which is the only reason binary_search finds anything.
Reordering these helper calls (or replacing normalize_ids with a checking-only implementation)
silently turns the containment test into a no-op without any compiler help. Either make the
containment check order-independent (sizes are capped at MAX_IDS_PER_RELATIONSHIP = 1000, so
Vec::contains costs nothing meaningful), or document and debug_assert the sorted precondition next
to the binary_search.

-         if relationship.old_ids.iter().any(|id| relationship.new_ids.binary_search(id).is_ok()) {
+         // Order-independent self-map check; safe regardless of whether new_ids
+         // has been sorted by normalize_ids.
+         if relationship.old_ids.iter().any(|id| relationship.new_ids.contains(id)) {


══════ F0534 │ src/migration/types.rs:187-191 │ [security · medium] ══════
[security · medium] `DeclarationEvidence` embeds reviewer identity (`approved_by`) and a timestamp
verbatim into the published machine-readable report, both as unconstrained strings. Two concerns for
a public contract: (1) reviewer names are personal data reaching every report consumer and every
log/stdout path that prints entries — callers must know not to leak them; (2) `approved_at` has no
declared format (RFC 3339? locale?), so downstream parsing/comparison is guesswork across producers.
Document the intended format and the PII caveat on the struct at minimum.

+ /// Reviewer evidence preserved verbatim for a declared identity relationship.
+ ///
+ /// NOTE: `approved_by` contains personally identifying reviewer data and
+ /// MUST NOT be written to logs or non-report diagnostics.
+ /// `approved_at` is RFC 3339 UTC (e.g. "2026-08-26T09:41:00Z").
+ #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
  pub struct DeclarationEvidence {
      pub approved_by: String,
      pub approved_at: String,
      pub rationale: String,
  }


══════ F0532 │ src/migration/types.rs:221-224 │ [maintainability · medium] ══════
[maintainability · medium] These doc comments are the module's public contract:
`old_requirements`/`new_requirements` "sum to `total_old`/`total_new`", and grouping rules for the
top-level counts. Nothing enforces this — `MigrationSummary` is constructible field-by-field
anywhere (all fields are `pub`), and a producer bug will publish an internally contradictory report
whose totals don't reconcile. Add a cheap consistency guard that runs wherever the summary is
finalized (or in a `try_new`/validation helper), so contradictions fail fast instead of shipping in
the machine-readable report.

-     /// Old-side requirement counts by outcome; these sum to `total_old`.
-     pub old_requirements: MigrationOutcomeCounts,
-     /// New-side requirement counts by outcome; these sum to `total_new`.
-     pub new_requirements: MigrationOutcomeCounts,
+ impl MigrationSummary {
+     /// Debug-build guard for the documented counting invariants;
+     /// returns Err describing the contradiction in release builds.
+     pub fn validate(&self) -> Result<(), &'static str> {
+         if self.old_requirements.total() != self.total_old {
+             return Err("old_requirements do not sum to total_old");
+         }
+         if self.new_requirements.total() != self.total_new {
+             return Err("new_requirements do not sum to total_new");
+         }
+         Ok(())
+     }
+ }


══════ F0535 │ src/migration/types.rs:279-285 │ [maintainability · medium] ══════
[maintainability · medium] This allowlist hardcodes exactly which evidence codes count as reviewable
location drift. `EvidenceCode` lives in this same file and grows freely — if a future variant
expressing real drift (e.g. section-title or file-label relocation) is added to the enum, it
defaults to NOT reviewable unless someone remembers this non-exhaustive-safe `matches!` catch (the
compiler won't flag it because the wildcard arm swallows everything). The result would be a silent
'no review needed' signal for genuine changes. Consider treating drift explicitly (e.g. an
`EvidenceCode::is_location_drift(&self)` method co-located with the enum definition, or requiring
the allowlist be maintained via an exhaustive match that fails compilation when variants are added).

+ impl EvidenceCode {
+     /// True when the evidence records source-location drift that must
+     /// force a review signal regardless of classification.
+     #[must_use]
+     pub const fn indicates_location_drift(self) -> bool {
                          matches!(
-                             evidence,
-                             EvidenceCode::SourceFileChanged
-                                 | EvidenceCode::SectionPathChanged
-                                 | EvidenceCode::SourceLineChanged
-                                 | EvidenceCode::AtomIndexChanged
+             self,
+             Self::SourceFileChanged
+                 | Self::SectionPathChanged
+                 | Self::SourceLineChanged
+                 | Self::AtomIndexChanged
                          )
+     }
+ }
+
+ // Usage:
+ //     entry.evidence.iter().any(EvidenceCode::indicates_location_drift)


══════ F0531 │ src/migration/types.rs:62-65 │ [maintainability · medium] ══════
[maintainability · medium] `Classification` maintains TWO parallel orderings: the derived `Ord`
(declaration order) and the hardcoded numeric `rank()` used for group precedence in
migration/engine.rs sort_entries(). They currently agree, but nothing ties them together: reordering
variants or inserting a new one shifts the derived `Ord` while `rank()` keeps stale numbers (or vice
versa), silently changing group-precedence behavior. A code search shows no caller uses the derived
`Ord`/`PartialOrd` at all — the discriminant-free `rank()` match already encodes precedence. Remove
the unused `PartialOrd, Ord` derives and express rank directly from declaration order (`self as u8`)
so a single source of truth exists; `EvidenceCode` likewise derives `PartialOrd, Ord` with no
ordering use and should drop them too.

- #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
+ // Doc-comment that variant declaration order defines group precedence.
+ #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
  #[serde(rename_all = "snake_case")]
  pub enum Classification {
      Unchanged,
+
+ // ...
+ impl Classification {
+     /// Precedence equals declaration order; a single source of truth.
+     pub(crate) const fn rank(self) -> u8 {
+         self as u8
+     }
+ }


══════ F0533 │ src/migration/types.rs:8-13 │ [maintainability · medium] ══════
[maintainability · medium] In a versioned external schema (`forge.migration-report/1`) these digest
fields ship as free-form `String`s with no casing/format constraint. Producers can emit
uppercase/lowercase hex or even non-hex garbage, breaking downstream comparison/dedup by hash, and
nothing marks future algorithm migrations (e.g. a non-SHA-256 value) except renaming a field. At
minimum document the encoding contract here; better, validate hex form at inventory build time or
use a lightweight `Sha256Hex(String)` newtype so misuse is visible at the API boundary.

- pub struct SourceProvenance {
-     pub label: String,
-     pub format: InputFormat,
-     pub sha256: String,
-     pub location_basis: LocationBasis,
- }
+ /// Lower-case hexadecimal SHA-256 digest (64 chars) of the source bytes.
+ pub struct Sha256Hex(String);


══════ F0574 │ src/model/assemble.rs:211-211 │ [maintainability · medium] ══════
[maintainability · medium] Dead error contract: the # Errors docs promise ForgeError::Parse when the
section tree is structurally invalid, but the body performs no validation whatsoever and can only
ever return Ok. Malformed trees (e.g. siblings with equal/out-of-order source_lines — precisely the
precondition the range math relies on) flow through unchecked, while callers are forced to
unwrap/propagate an error that can never occur. Either actually validate the tree invariants (the
ascending-sibling check is the load-bearing one) or simplify the signature to return PolicyDocument
directly.

- ) -> Result<PolicyDocument, ForgeError> {
+     // Validate the structure this function's correctness depends on:
+     for window in sections.windows(2) {
+         if window[0].source_line >= window[1].source_line {
+             return Err(ForgeError::Parse(format!(
+                 "sections out of document order: '{}' at line {} precedes '{}' at line {}",
+                 window[0].title, window[0].source_line,
+                 window[1].title, window[1].source_line
+             )));
+         }
+     }
+     let item_refs: Vec<&ExtractedListItem> = clauses.list_items.iter().collect();


══════ F0576 │ src/model/assemble.rs:241-242 │ [maintainability · medium] ══════
[maintainability · medium] Silent data drop with no accounting: assemble_document consumes
ExtractedContent but reads only `clauses.list_items`; anything extracted into `tables` or
`paragraphs` vanishes without warning or trace, undermining the file's own SEC-5 no-silent-drop
posture that list items enjoy. If a later pipeline stage attaches tables/paragraphs to the document,
document that here; otherwise emit a diagnostic (e.g. eprintln warning consistent with the
malformed-YAML path) or account for the dropped counts when either slice is non-empty.

+     if !clauses.tables.is_empty() || !clauses.paragraphs.is_empty() {
+         eprintln!(
+             "warning: {} table(s) and {} paragraph(s) extracted but not represented in the assembled document (not yet supported)",
+             clauses.tables.len(),
+             clauses.paragraphs.len()
+         );
+     }
      let item_refs: Vec<&ExtractedListItem> = clauses.list_items.iter().collect();
      let mapped_sections = map_sections(sections, &item_refs);


══════ F0575 │ src/model/assemble.rs:78-90 │ [maintainability · medium] ══════
[maintainability · medium] Third of three near-identical PolicyRequirement literals (also lines
48-56 and 81-89): each copy hardcodes the same placeholder default set (stable_id: None, atom_index:
0, empty citations/modality/parameters, parent_text: None). Adding a field to PolicyRequirement
requires editing all three sites in lockstep, and the copies can drift apart unnoticed. Factor into
a single local constructor (e.g. `fn forge_requirement(item: &ExtractedListItem) ->
PolicyRequirement`) and reuse it in all three places.

-             .map(|item| PolicyRequirement {
-                 stable_id: None,
-                 text: item.text.clone(),
-                 source_line: item.source_line,
-                 nesting_depth: item.nesting_depth,
-                 atom_index: 0,
-                 parent_text: None,
-                 citations: vec![],
-                 modality: None,
-                 parameters: vec![],
-             })
+             .map(forge_requirement)
              .collect();

          result.push(PolicySection {


══════ F0571 │ src/model/frontmatter.rs:19-20 │ [maintainability · medium] ══════
[maintainability · medium] Unknown or misspelled keys are accepted and silently discarded (no
deny_unknown_fields, no alias support, no diagnostic). A typo like 'Titel:' makes the intended
metadata disappear, downstream assembly falls back to H1 titles, and the author never learns why.
Given this crate's explicit fault-tolerance posture, at least emit a tracing::warn for unrecognized
top-level keys (deserialize into serde_yaml::Value/Mapping first, warn, then deserialize into
FrontmatterData), or document the silent-drop tradeoff in the struct docs.

+ /// NOTE: unrecognized keys are currently ignored without diagnostics;
+ /// typo'd fields (e.g. `Titel:`) silently lose their metadata.
  #[derive(Debug, Deserialize)]
  pub(crate) struct FrontmatterData {


══════ F0569 │ src/model/frontmatter.rs:58-59 │ [bug · medium] ══════
[bug · medium] Closing-delimiter detection misses an immediately-closed fence: for input
"---\n---\n# Body", rest starts with another '\n'?? No — rest becomes "---\n# Body\n", which
contains no '\n' before the closing marker, so all four alternatives fail and a complete, valid
(empty-field) frontmatter is reported as absent/malformed. Every mainstream frontmatter parser
treats "---\n---\n" as valid. Additionally, the or_else chain evaluates .find() and .strip_suffix()
over overlapping patterns without guaranteeing the EARLIEST closing delimiter, so the selected end
may splice content beyond a legitimate closer into yaml_str. Root cause is requiring a preceding
'\n'; compensate by trimming one trailing newline before searching.

-     let end = rest
+     // Trim one trailing newline so an immediately-closed fence ("---\n---\n")
+     // follows the same matching rules as a fence closed after fields.
+     let trimmed = rest.strip_suffix('\n').unwrap_or(rest);
+     let end = trimmed
          .find("\n---\n")
+         .or_else(|| trimmed.find("\n---\r\n"))
+         .or_else(|| trimmed.strip_suffix("\n---").map(str::len))
+         .or_else(|| trimmed.strip_suffix("\r\n---").map(str::len))?;
+     let yaml_str = &trimmed[..end];


══════ F0581 │ src/model/frontmatter.rs:61-62 │ [bug · medium] ══════
[bug · medium] Correction to the earlier suggestion on this block: trimming a trailing newline does
NOT fix the immediately-closed fence. For "---\n---\n# Body", after the opener strip, `rest` itself
BEGINS with the closing marker ("---\n# Body\n"), so every "\n---..." pattern still fails and valid
empty frontmatter is dropped. Additionally, each or_else arm scans the entire remainder
independently, so in a mixed-LF/CRLF file a later LF-style "\n---\n" can beat an earlier CRLF-style
"\n---\r\n" occurrence, splicing the intervening text (including the first real fence) into
yaml_str. Both defects share one root cause: searching for fences as substrings requiring a
preceding '\n' instead of scanning whole lines. A line-oriented scan fixes all cases at once: it
recognizes a fence at ANY line boundary (including the first line of `rest`), always selects the
earliest fence, uniformly tolerates CRLF (trim_end_matches ['\n','\r']), and naturally treats a
final unterminated "---" as a valid closer.

-         .or_else(|| rest.strip_suffix("\n---").map(str::len))
-         .or_else(|| rest.strip_suffix("\r\n---").map(str::len))?;
+     // Scan line-by-line: the closing fence must occupy a whole line, may be
+     // the very first line of `rest` (immediately-closed fence), and the
+     // earliest occurrence wins even under mixed LF/CRLF endings.
+     let mut end = None;
+     let mut cursor = 0;
+     for line in rest.split_inclusive('\n') {
+         if line.trim_end_matches(['\n', '\r']) == "---" {
+             end = Some(cursor);
+             break;
+         }
+         cursor += line.len();
+     }
+     let yaml_str = match end {
+         Some(e) => &rest[..e],
+         None => return None, // no closing delimiter
+     };


══════ F0549 │ src/model/mod.rs:172-173 │ [maintainability · medium] ══════
[maintainability · medium] `requirement_id` is a bare `String` referencing
`PolicyRequirement.stable_id: Option<String>`, and `Citation.source_requirement_id` mirrors this
with another raw string. Nothing enforces referential integrity: if WI-7 regenerates UUIDs after
WI-34/WI-8 ran, stale `requirement_id`s silently point at nonexistent requirements, and malformed
values flow unchecked into OSCAL output. Prefer a newtype (`struct RequirementId(String)` with
`Debug, Clone, PartialEq, Eq, Hash, Serialize`) for both fields so IDs cannot be confused with
arbitrary strings and cross-links are self-documenting. Relatedly, `PolicyParameter.id` format
`{requirement_id}_prm_{position}` is convention-only — consider enforcing/building it via a
constructor rather than leaving it open.

-     /// `stable_id` of the `PolicyRequirement` this parameter was extracted from.
-     pub requirement_id: String,
+ /// Newtyped identifier linking parameters/citations back to their requirement.
+ #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
+ pub struct RequirementId(pub String);
+
+ // In PolicyParameter:
+ /// Newtyped ID of the requirement this parameter was extracted from.
+ pub requirement_id: RequirementId,
+
+ // In Citation:
+ /// Newtyped `stable_id` of the requirement that references this citation.
+ pub source_requirement_id: Option<RequirementId>,


══════ F0551 │ src/model/mod.rs:267-269 │ [bug · medium] ══════
[bug · medium] `collect_citations` blindly extends with cloned citations from every requirement, and
`Citation.id` is documented as unique. Multiple requirements legitimately citing the same source
(same `id`) — or upstream extraction bugs — will emit duplicate IDs straight into WI-12 back matter,
where OSCAL expects unique resource/prose identifiers. Deduplicate by `id` here (this method is the
documented single input to back-matter generation). While touching it, avoid eagerly cloning every
citation: walk collecting `&Citation` references, dedup, and clone only survivors; optionally
pre-reserve using `self.total_requirements()`.

+     #[must_use]
+     pub fn collect_citations(&self) -> Vec<Citation> {
+         fn walk<'a>(section: &'a PolicySection, out: &mut Vec<&'a Citation>) {
              for req in &section.requirements {
-                 out.extend(req.citations.clone());
+                 out.extend(req.citations.iter());
+             }
+             for child in &section.children {
+                 walk(child, out);
+             }
+         }
+
+         let mut refs = Vec::with_capacity(self.total_requirements());
+         for section in &self.sections {
+             walk(section, &mut refs);
+         }
+
+         // Keep the first occurrence per citation id; OSCAL back matter
+         // requires unique identifiers.
+         let mut seen = std::collections::HashSet::new();
+         refs.into_iter()
+             .filter(|c| seen.insert(c.id.as_str()))
+             .cloned()
+             .collect()
              }


══════ F0552 │ src/model/mod.rs:282-284 │ [test · medium] ══════
[test · medium] Test coverage stops at `total_sections*`: `PolicySection::total_requirements` /
`PolicyDocument::total_requirements` (the recursive summation path) and `collect_citations` have
zero tests. These aggregations feed progress stats and WI-12 back-matter input, so add tests
exercising: nested sections carrying requirements at multiple depths; `total_requirements` over an
empty doc and a doc with requirements only in children; and `collect_citations` with citations
spread across sibling/child sections plus duplicate `id`s (to lock in dedup semantics once added)
and `source_requirement_id` propagation.

  #[cfg(test)]
  mod tests {
      use super::*;
+
+     #[test]
+     fn total_requirements_recursive_and_citation_aggregation() {
+         // Build grandparent > parent > child with one requirement each,
+         // overlapping citation ids, then assert totals and uniqueness.
+     }


══════ F0550 │ src/model/mod.rs:64-75 │ [bug · medium] ══════
[bug · medium] The field docs promise version falls back to `"0.0.0"` (and title has
heading/filename fallbacks), but the derived `Default` yields `version: ""` and `title: ""` —
`DocumentMetadata::default()` violates the documented contract anywhere it is used directly (the
file's own test `total_sections_empty_doc` already constructs such a metadata with an empty version,
which would leak an empty version into OSCAL generation). Implement `Default` manually so `version`
defaults to `"0.0.0"`, or drop the derivation and require callers to build complete metadata.

- #[derive(Debug, Clone, Default, Serialize)]
- pub struct DocumentMetadata {
-     /// Document title.
-     /// - Frontmatter `title` field, OR
-     /// - First H1 heading text, OR
-     /// - Filename (without extension)
-     pub title: String,
-
-     /// Document version (semantic versioning format preferred).
-     /// - Frontmatter `version` field, OR
-     /// - Default "0.0.0"
-     pub version: String,
+ impl Default for DocumentMetadata {
+     fn default() -> Self {
+         Self {
+             title: String::new(),
+             version: "0.0.0".to_string(),
+             author: None,
+             date: None,
+             source_path: PathBuf::new(),
+             content_hash: None,
+         }
+     }
+ }
