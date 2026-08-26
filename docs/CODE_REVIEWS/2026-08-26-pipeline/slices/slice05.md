# Validation slice slice05 — 60 findings
Severity mix: medium×60


══════ F0545 │ src/model/trace.rs:18-19 │ [other · medium] ══════
[other · medium] Using an empty `String` as a sentinel for "no parent section" makes a genuinely
empty heading indistinguishable from an absent one, forcing callers to compare against `""`. Model
the domain state with `Option<String>` (serialized as null) per typed-state design; the current
convention should at least be enforced with a constructor/validation helper if it must remain for
EC-4 compatibility.

-     /// Section title containing the requirement. Empty string if no parent section (EC-4).
-     pub section_title: String,
+     /// Section title containing the requirement, if any.
+     pub section_title: Option<String>,


══════ F0544 │ src/model/trace.rs:81-85 │ [performance · medium] ══════
[performance · medium] `record` deep-clones every `TraceLink` (requirement ID, JSON path, element
ID, section title, and PathBuf) into the `by_requirement` forward index while keeping the original
in `links`, doubling memory per entry purely to support lookups. Since neither index ever mutates
after insertion, storing stable positions instead of clones preserves behavior at roughly half the
allocation cost: e.g., `by_requirement: HashMap<String, Vec<usize>>` pushing `index`, resolved via
`self.by_requirement.get(id).into_iter().flatten().map(|&i| &self.links[i])` (changing
`by_requirement` to return an iterator or `Vec<&TraceLink>`). If the returned `&[TraceLink]` slice
contract must be kept verbatim, at least document why the eager clone is intentional.

-         // Clone into forward index grouped by requirement
+         let index = self.links.len();
          self.by_requirement
              .entry(link.requirement_stable_id.clone())
              .or_default()
-             .push(link.clone());
+             .push(index);
+         self.by_oscal_element.insert(link.oscal_element_id.clone(), index);
+         self.links.push(link);


══════ F0562 │ src/oscal/assessment_plan.rs:222-224 │ [bug · medium] ══════
[bug · medium] An empty control set only logs a warning and proceeds to emit `control-selections`
containing a control-selection whose `include-controls` array is empty. OSCAL cardinally requires at
least one included control entry per control-selection (min 1), so the pipeline silently produces a
schema-invalid artifact rather than surfacing a typed `ForgeError::Validation` the way the SEC-2
empty-href case does. Prefer failing fast here; if the lenient behavior is genuinely desired, gate
it behind an explicit opt-in and update the `ec1_zero_controls_empty_include_controls` test
accordingly.

      if sorted_ids.is_empty() {
-         tracing::warn!("Zero controls found — Assessment Plan will have empty include-controls");
+         return Err(ForgeError::Validation(
+             "conversion produced zero controls; refusing to emit an Assessment Plan \
+              with an empty include-controls array"
+                 .to_string(),
+         ));
      }


══════ F0560 │ src/oscal/assessment_plan.rs:235-237 │ [bug · medium] ══════
[bug · medium] The plan UUID seed embeds the raw `import_ssp_href` while the emitted
`import-ssp.href` is passed through `sanitize_artifact_path` (basename-only). Two callers pointing
at the same SSP file but spelling the path differently (`./ssp/system-ssp.json` vs
`/abs/dir/system-ssp.json`) therefore produce different plan UUIDs for logically identical
documents, weakening the stated determinism guarantee (SEC-4). Additionally, `sorted_ids.join(",")`
inside a pipe-delimited seed is not injective: control sets ["A,B","C"] and ["A","B,C"] collapse to
the same seed and collide. Seed a canonical form instead: sanitize the href first (and reuse it for
`ImportSsp.href`) and length-prefix each control ID so the encoding is unambiguous.

-     // UUID v5 seed: deterministic from sorted control IDs + SSP href (SEC-4)
-     let seed = format!("assessment-plan|{}|{}", sorted_ids.join(","), import_ssp_href);
+     // Canonicalize inputs before hashing so equivalent logical inputs yield identical UUIDs.
+     let canonical_href = crate::io::sanitize_artifact_path(std::path::Path::new(import_ssp_href));
+     // Length-prefixed entries keep the seed injective even if IDs contain ',' or '|'.
+     let mut seed = format!("assessment-plan|{}", sorted_ids.len());
+     for id in &sorted_ids {
+         seed.push('|');
+         seed.push_str(&id.len().to_string());
+         seed.push(':');
+         seed.push_str(id);
+     }
+     seed.push('|');
+     seed.push_str(&canonical_href);
      let uuid = generate_stable_id(&seed);


══════ F0564 │ src/oscal/assessment_plan.rs:454-457 │ [performance · medium] ══════
[performance · medium] `activity.subjects.clone_from(&subjects)` deep-clones the entire subjects
vector (strings included) once per activity — i.e., O(tasks × subjects) element clones and
allocations — even though every resulting `AssessmentSubject` byte-identically duplicates the single
shared value later stored in `assessment_plan.assessment_subjects`. Consider sharing the data
instead of replicating it (e.g., hold `Arc<AssessmentSubject>` refs inside activities, or inject
subjects only at serialization time), which keeps memory proportional to the subject list rather
than the task count.


══════ F0622 │ src/oscal/back_matter.rs:173-173 │ [security · medium] ══════
[security · medium] The `Malformed` arm preserves fully attacker-controlled strings verbatim in
`href`, protected only by the advisory `url-status="unvalidated"` prop. Concretely, scheme-smuggling
payloads such as `"jav\tascript:alert(1)"` or `"\nj avascript:..."` fail `Url::parse` (invalid
scheme characters) and therefore bypass the `DANGEROUS_SCHEMES` filter entirely — yet browsers
historically normalize tab/newline inside href attributes, resurrecting the javascript: scheme at
render time. As layered defense, route raw values whose pre-colon prefix contains ASCII control
characters to the `Dangerous` treatment instead of emitting them unmodified.

-             let rlinks = vec![Rlink { href: raw_url, media_type: None }];
+             // Defense-in-depth: values whose scheme-like prefix embeds ASCII
+             // control characters (e.g. "jav\tascript:") evade Url::parse and
+             // the DANGEROUS_SCHEMES filter; treat them as dangerous rather
+             // than echoing them verbatim into href.
+             let sanitized = if raw_url.chars().take_while(|c| *c != ':').any(char::is_control) {
+                 tracing::warn!(citation_id = %citation.id, "Control-character-bearing pseudo-scheme treated as dangerous");
+                 String::new()
+             } else {
+                 raw_url
+             };
+             let rlinks = vec![Rlink { href: sanitized, media_type: None }];


══════ F0619 │ src/oscal/back_matter.rs:283-283 │ [bug · medium] ══════
[bug · medium] Duplicate citation IDs are silently swallowed: a later citation with the same `id`
overwrites the earlier entry in `resource_map`, so control links generated afterwards point the
first requirement's citations at the wrong (later) resource. This is invalid input just like the
empty-ID case handled above, so it should be rejected consistently (or at minimum logged loudly)
rather than overwritten invisibly.

-         resource_map.insert(citation.id.clone(), uuid);
+         if resource_map.insert(citation.id.clone(), uuid).is_some() {
+             return Err(ForgeError::BackMatter(format!(
+                 "duplicate citation id: {}",
+                 citation.id
+             )));
+         }


══════ F0612 │ src/oscal/catalog.rs:253-254 │ [maintainability · medium] ══════
[maintainability · medium] When `abbreviation` comes from resolve_abbreviation with a collision
suffix, the emitted ID is e.g. 'POL-AC-dd3e-001' — four dash-separated segments instead of the
documented POL-{ABBR}-{NNN} shape shown in the examples. Downstream parsers, regexes, or validators
that assume exactly three segments (or a numeric serial) will mis-handle collision-resolved
controls, yet the documented contract here never mentions this embedded-suffix form. Sanitize the
abbreviation segment on entry or document the accepted 'ABBR-hhhh' variant in the contract and
examples.

  #[must_use]
  pub fn generate_control_id(abbreviation: &str, requirement_index: usize, prefix: &str) -> String {
+     // NOTE: `abbreviation` may embed a 4-hex collision suffix ("AC-dd3e"),
+     // yielding four-segment IDs like POL-AC-dd3e-001. Callers/parsers must
+     // accept this form or the abbreviation must be sanitized here.
+     let display = requirement_index + 1;
+     format!("{prefix}-{abbreviation}-{display:03}")


══════ F0611 │ src/oscal/catalog.rs:358-364 │ [bug · medium] ══════
[bug · medium] On early error return (missing stable_id here, or a DuplicateElement/failure from
tl.record below) the function has already pushed entries into the caller-supplied
TraceLinkCollection in earlier iterations. The caller receives Err plus a half-populated collection
whose contents cannot be distinguished from a fully successful run, inviting double-recording on
retry (which record() rejects as DuplicateElement) or silent trace loss. Either roll back the links
recorded during this call (buffer locally and flush at the end) or drop them before returning Err.

+             // Buffer trace links locally; commit only if the whole build
+             // succeeds, otherwise clear the buffer so the collection stays
+             // transactional across the Err return.
              let stable_id = req.stable_id.as_ref().ok_or_else(|| {
+                 buffered_links.clear();
                  let preview: String = req.text.chars().take(60).collect();
                  ForgeError::CatalogBuild(format!(
                      "Requirement missing stable_id in section '{}': '{preview}'",
                      req_section.title,
                  ))
              })?;


══════ F0610 │ src/oscal/catalog.rs:473-476 │ [documentation · medium] ══════
[documentation · medium] The claimed order-independence is incorrect: the first title encountered
still takes the bare abbreviation (no suffix) and every later contender gets the content hash, so
swapping section order swaps which title is bare versus suffixed (POL-AC-001 vs POL-AC-dd3e-001
flip). Determinism holds only for a fixed input order, which the accompanying test actually verifies
('Same order should produce same result'); the doc overstates the guarantee and can mislead
consumers expecting reorder-stable IDs. Reword to document fixed-order determinism and its limit,
and note the triple-title duplicate hazard covered separately.

  /// titles that produce the same base abbreviation receive a hash suffix
  /// derived from their content (first 2 bytes of SHA-256, hex-encoded).
- /// This makes the disambiguation stable regardless of encounter order.
- pub(crate) fn resolve_abbreviation(
+ /// Note: output is deterministic only for a fixed section ordering — the
+ /// title encountered first keeps the bare abbreviation; later ones get
+ /// content-derived suffixes. Reordering the input can change every
+ /// affected control ID.


══════ F0601 │ src/oscal/component_definition.rs:124-135 │ [bug · medium] ══════
[bug · medium] Documented contract is broken: the doc comment states this "optionally records trace
links into the provided TraceLinkCollection", but the body never touches `_trace_links`. Any caller
passing `Some(collection)` receives a silent no-op and falsely assumes trace links were recorded.
Either wire the parameter through (emit links during component/citation construction) or remove it
from the public signature until trace recording is implemented; at minimum, correct the doc comment
so callers are not misled.

- /// Optionally records trace links into the provided [`TraceLinkCollection`].
- /// Pass `None` for backward compatibility.
+ /// NOTE: `trace_links` is accepted for API compatibility but is currently
+ /// ignored (no links are recorded). Do not rely on it until implemented.
+ /// Pass `None`.
  ///
  /// # Errors
  ///
  /// Returns `ForgeError::ComponentDefinitionBuild` if back matter generation fails.
  pub fn build_component_definition(
      document: &PolicyDocument,
      source_profile: Option<&str>,
      _trace_links: Option<&mut TraceLinkCollection>,
      source_file: Option<&str>,
  ) -> Result<ComponentDefinitionEnvelope, ForgeError> {


══════ F0605 │ src/oscal/component_definition.rs:148-157 │ [bug · medium] ══════
[bug · medium] `source_file.unwrap_or("")` uses an empty string as a sentinel for 'absent', which
leaks into `build_control_implementations` whenever `source_profile` is `Some`: with `source_file ==
None` but a profile supplied, downstream requirements receive `source_file: ""` and embed an empty
path in their trace props, conflating 'no source file provided' with 'an empty path'. Keep the
`Option` end-to-end (or forward the prop only when present) instead of relying on an untyped string
sentinel whose meaning depends on implicit convention.


══════ F0606 │ src/oscal/component_definition.rs:182-183 │ [bug · medium] ══════
[bug · medium] Deduplication here keys on `citation.id`, but `generate_back_matter` derives each
resource's UUID from the normalized citation text + URL. Two citations with distinct `id`s but
identical text/URL both survive this filter, so the generated back matter will contain multiple
resources sharing the same resource UUID — invalid OSCAL output that the existing tests never
exercise (they reuse the same id). Deduplicate on the same composite key used for resource identity
(or de-dupe produced resources by UUID) so back matter cannot contain duplicate resource UUIDs.


══════ F0602 │ src/oscal/component_definition.rs:187-189 │ [maintainability · medium] ══════
[maintainability · medium] The original error from `generate_back_matter` (already a structured
`ForgeError`) is converted to a bare string and re-wrapped, losing the source error's variant/type
information and all stage context: a caller seeing `ComponentDefinitionBuild("citation has empty
id")` cannot tell the failure originated in back-matter generation rather than component assembly.
Preserve the underlying error and add actionable boundary context instead of flattening it via
`to_string()`.

          let (resources, _resource_map) =
              crate::oscal::back_matter::generate_back_matter(&all_citations)
-                 .map_err(|e| ForgeError::ComponentDefinitionBuild(e.to_string()))?;
+                 .map_err(|e| {
+                     ForgeError::ComponentDefinitionBuild(format!(
+                         "back matter generation failed ({e}): {e}"
+                     ))
+                 })?;


══════ F0589 │ src/oscal/implemented_requirements.rs:128-129 │ [bug · medium] ══════
[bug · medium] The CI UUIDv5 is seeded from the LOSSY sanitized source: `sanitize_artifact_path`
keeps only the file name, so `--source-profile ./baselines/prod/baseline.json` and `--source-profile
./vendor/baseline.json` — two distinct profiles — produce the same basename, hence the same
`ControlImplementation.uuid` AND the same opaque `source` string. Two different
documents/implementations then become byte-wise indistinguishable identities, which corrupts
merge/diff tooling keyed on these deterministic UUIDs. Seed the UUID from the original
`source_profile` (before sanitization) so identity reflects the actual input, and/or reject inputs
whose basenames collide within a build.

      let sanitized_source = crate::io::sanitize_artifact_path(std::path::Path::new(source_profile));
-     let ci_uuid = generate_control_impl_uuid(&sanitized_source, title);
+     // Identity must track the real profile, not its lossily-collapsed filename.
+     let ci_uuid = generate_control_impl_uuid(source_profile, title);


══════ F0591 │ src/oscal/implemented_requirements.rs:155-157 │ [bug · medium] ══════
[bug · medium] Note how the `"no-stable-id"` sentinel compounds the index-seeding problem: every
requirement without a `stable_id` shares the same literal seed component, so with identical text the
ONLY thing separating them is positional `global_index` — meaning these entries are guaranteed to
churn on any edit anywhere in the document, precisely when they carry the weakest identity. Once the
missing-stable_id case is rejected upstream (see prior comment) this sentinel becomes dead; until
then, derive some content fingerprint (e.g., hash of text + section title) instead of a shared
constant.


══════ F0586 │ src/oscal/implemented_requirements.rs:233-238 │ [bug · medium] ══════
[bug · medium] This fallback diverges from the Catalog builder for the same input: `build_catalog`
returns `ForgeError::CatalogBuild` when `req.stable_id` is `None`, so no catalog control ever exists
for such requirements. Here the builder silently continues and emits
`implemented-requirement.control-id = "REQ-nnn"` — IDs outside the `POL-*` namespace that consumers
validate against, dangling references if a catalog was produced elsewhere, and a split artifact set
when the document contains a mix of IDs and non-ID requirements. Align the error policy (fail with
`ComponentDefinitionBuild` like the Catalog builder, or filter/skip such requirements before
mapping) instead of inventing a second identifier scheme.

-     if has_stable_id {
-         generate_control_id(abbreviation, req_index_in_section, "POL")
-     } else {
-         tracing::warn!(global_index, "Requirement missing stable_id — using fallback control-id");
-         format!("REQ-{:03}", global_index + 1)
+     let abbreviation = resolve_abbreviation(&section.title, &mut abbrev_counts);
+     let requirements = collect_requirements_with_section(section);
+
+     for (req_idx, (req, req_section)) in requirements.iter().enumerate() {
+         let stable_id = req.stable_id.as_deref().ok_or_else(|| {
+             ForgeError::ComponentDefinitionBuild(format!(
+                 "requirement at global index {global_index} lacks stable_id"
+             ))
+         })?;
+         let control_id = generate_control_id(&abbreviation, req_idx, "POL");
+         let entry = map_requirement_to_implemented(
+             req,
+             &control_id,
+             global_index,
+             source_file,
+             &req_section.title,
+         );
+         implemented_requirements.push(entry);
+         global_index += 1;
      }


══════ F0588 │ src/oscal/implemented_requirements.rs:92-92 │ [maintainability · medium] ══════
[maintainability · medium] The function is infallible — `ForgeError::ComponentDefinitionBuild` is
never constructed anywhere in this module — yet the signature, `# Errors` doc, and callers all
pretend failure is possible, forcing call sites to `unwrap()` an impossible `Err` or handle dead
error branches. Combined with finding 2 (which genuinely NEEDS this error for missing `stable_id`),
prefer propagating real validation failures through this `Result`; otherwise drop the wrapper and
return `Vec<ControlImplementation>` directly.

- ) -> Result<Vec<ControlImplementation>, ForgeError> {
+ pub fn build_control_implementations(
+     document: &PolicyDocument,
+     source_profile: &str,
+     source_file: &str,
+ ) -> Vec<ControlImplementation> {


══════ F0594 │ src/oscal/metadata.rs:56-58 │ [documentation · medium] ══════
[documentation · medium] "Production callers pass `None`" is false as written: `build_profile` in
`src/oscal/profile.rs` constructs `Some(MetadataOptions { timestamp_override, .. })` on its normal
production path for reproducible outputs. Additionally, `MetadataOptions` is fully public with a
free-form `uuid_override`, so nothing enforces the documented "test callers only" invariant —
passing a stale hard-coded UUID in production would emit artifacts with duplicate identities and no
diagnostic. Either restrict the visibility/ergonomics of the override (e.g., gate UUID override
behind a builder reserved for tests or log when it is used) or correct the docs to state the real
contract.

- /// Production callers pass `None` to `assemble_metadata`. Test callers
- /// construct `MetadataOptions` with fixed UUID and/or timestamp for
- /// deterministic assertions.
+ /// Options for overriding auto-generated metadata values.
+ ///
+ /// Timestamp overrides are used in production by deterministic builders
+ /// (e.g., Profile generation) to make output reproducible. UUID overrides
+ /// exist for tests; calling them with hard-coded values in production will
+ /// produce artifacts with colliding identifiers.


══════ F0593 │ src/oscal/metadata.rs:70-70 │ [documentation · medium] ══════
[documentation · medium] This doc contradicts both the surrounding reality and actual usage: (1) the
`OscalMetadata` docs above state only four fields are serialized (uuid is internal-only), yet this
says "all five required fields"; (2) `MetadataOptions` is NOT test-only — it is publicly exported
(`src/oscal/mod.rs` re-exports it) and `profile.rs::build_profile` uses `timestamp_override` in
production for deterministic/reproducible profile generation. Presenting overrides as "test callers"
invites accidental misuse, and an unconditional public `uuid_override` lets callers mint
arbitrary/duplicate artifact identities. Fix the field-count wording and rewrite the contract to
describe who may legitimately override UUID vs timestamp.

- /// Produces a complete `OscalMetadata` struct with all five required fields.
+ /// Produces a complete `OscalMetadata` struct: four serialized OSCAL fields
+ /// (`title`, `last-modified`, `version`, `oscal-version`) plus an internal-only
+ /// `uuid` (not serialized; the artifact UUID is emitted at the envelope root).
+ /// Overrides are legitimate for deterministic/reproducible output (e.g., Profile
+ /// builds); timestamp overrides are supported for that purpose, whereas UUID
+ /// overrides must preserve uniqueness — never reuse a UUID across artifacts.


══════ F0592 │ src/oscal/metadata.rs:95-97 │ [bug · medium] ══════
[bug · medium] An empty `title` (and likewise an empty `version`, which gets no diagnostic at all)
is accepted with only a warn-level trace and flows verbatim into the generated artifact. OSCAL
v1.2.3 requires `metadata/title` to be a non-empty string, so downstream artifacts can be silently
schema-invalid; the warning is easy to miss in library contexts where no subscriber collects
debug/warn output. This function deliberately returns `Result<ForgeError>` "for future
extensibility" — this is exactly the case a typed error should cover. Note that pinned test T017
(`assemble_empty_title_passes_through`) currently enshrines the non-compliant behavior, so
tightening this requires updating that test.

      if doc_metadata.title.is_empty() {
-         tracing::warn!("DocumentMetadata title is empty; OSCAL metadata.title will be empty");
+         return Err(ForgeError::InvalidInput(
+             "DocumentMetadata.title must be non-empty (required by OSCAL metadata/title)"
+                 .to_string(),
+         ));
+     }
+     if doc_metadata.version.is_empty() {
+         return Err(ForgeError::InvalidInput(
+             "DocumentMetadata.version must be non-empty (required by OSCAL metadata/version)"
+                 .to_string(),
+         ));
      }


══════ F0583 │ src/oscal/mod.rs:14-15 │ [maintainability · medium] ══════
[maintainability · medium] `implemented_requirements` and `trace_embedding` are `pub mod`s but are
the only submodules absent from the curated flat re-export block, unlike all eight siblings. Their
public types/functions (`ControlImplementation`, `ImplementedRequirement`,
`build_control_implementations`, `FORGE_TRACE_NS`, `build_trace_props`, `embed_trace_in_catalog`,
...) are genuinely part of the supported API — in-tree callers (component_definition.rs,
xml_serializer.rs/xml_deserializer.rs, trace/extractor.rs) are forced to use full module paths like
`crate::oscal::implemented_requirements::ControlImplementation`, which breaks the otherwise flat
public namespace this file curates and signals to readers that these modules are
second-class/internal when they are not. There is no name collision preventing flattening
(`ImplementedRequirement` vs `SspImplementedRequirement` are distinct); either add the missing
re-exports for symmetry, or document why these two modules stay unflattened.

  /// OSCAL implemented-requirements builder for control implementations.
  pub mod implemented_requirements;
+ // Consider also: `pub use implemented_requirements::{ControlImplementation, ImplementedRequirement, build_control_implementations};`
+ // and the analogous exports for `trace_embedding` so the whole facade stays flat.


══════ F0582 │ src/oscal/mod.rs:3-4 │ [documentation · medium] ══════
[documentation · medium] Module doc pins "OSCAL v1.2.0" but the actual constant re-exported below is
`metadata::OSCAL_VERSION = "1.2.3"` (see src/oscal/metadata.rs:11; `ssp::SSP_OSCAL_VERSION` just
aliases it), so every generated Catalog/SSP/Assessment Plan emits "oscal-version": "1.2.3". At a
crate-boundary facade, a hardcoded version claim that disagrees with the single source of truth
misleads consumers about schema conformance. Prefer referencing the constant instead of embedding a
literal.

- //! This module provides types and builders for producing OSCAL v1.2.0 artifacts
- //! including Catalogs, Component Definitions, Profiles, and Assessment Plans.
+ //! This module provides types and builders for producing OSCAL artifacts pinned
+ //! to [`metadata::OSCAL_VERSION`] (currently "1.2.3"), including Catalogs,
+ //! Component Definitions, Profiles, and Assessment Plans.


══════ F0595 │ src/oscal/profile.rs:253-254 │ [bug · medium] ══════
[bug · medium] The UUID-v5 seed is built by naively joining attacker-influenced strings (control
IDs, param IDs/values) with '|' as delimiter, so distinct inputs can collide: control_ids ["a", "b"]
vs ["a|b"], or pairs [("p|q", "v")] vs [("p", "q|v")], all produce the identical seed string and
thus identical profile UUIDs while representing semantically different profiles. OSCAL profiles
identified by colliding UUIDs cannot be told apart downstream (dedup/tracking/cache keying). Use a
length-prefixed or unambiguous encoding (e.g., length-prefix each part, or feed each part separately
into the hasher before new_v5 such as hashing lengths), so every input maps to a unique encoding.

-     let seed = seed_parts.join("|");
+ // Length-prefix each part so distinct inputs always map to distinct seeds
+ // (e.g. ["a", "b"] vs ["a|b"] no longer collide).
+ let mut seed = String::new();
+ for part in &seed_parts {
+     seed.push_str(&part.len().to_string());
+     seed.push(':');
+     seed.push_str(part);
+ }
      let uuid = Uuid::new_v5(&crate::uuid::PROFILE_NAMESPACE, seed.as_bytes());


══════ F0625 │ src/oscal/ssp.rs:370-373 │ [maintainability · medium] ══════
[maintainability · medium] This hand-written Serialize impl maintains a parallel field-count ledger
that must be updated in lock-step with every conditional branch below (it only stays correct today
because the `account-status` prop is pushed unconditionally), and it mutates semantics during
serialization: when `role_ids` is empty, no `role-ids` field is emitted and instead a fabricated
FORGE-namespaced `role-ids` prop carrying a TODO string is injected into `props`, so the JSON
diverges from the declared struct. A future field addition (or removing the unconditional prop) will
desync the ledger and corrupt the envelope shape for some serializer backends. Prefer building the
ordered set of emitted key/value pairs first, then serializing them, and express 'roles to be
defined' via a dedicated marker rather than repurposing output mid-flight.

-         let mut field_count = 2;
-         field_count += usize::from(self.short_name.is_some());
-         field_count += usize::from(self.description.is_some());
-         field_count += usize::from(!self.role_ids.is_empty());
+ // Derive the emitted field list, then serialize it; the length is computed,
+ // not tracked:
+ let entries = build_user_entries(self);
+ let mut map = serializer.serialize_map(Some(entries.len()))?;
+ for (key, value) in &entries {
+     map.serialize_entry(key, value)?;
+ }
+ map.end()


══════ F0629 │ src/oscal/ssp.rs:591-593 │ [other · medium] ══════
[other · medium] The SSP document UUID is a pure v5 hash of `ssp|{policy_title}`, so merely renaming
the source policy (an ordinary event between revisions) mints a brand-new document UUID. This breaks
cross-version traceability of the authorization record and forces re-keying of every implemented
requirement/by-component reference in downstream tooling. Anchor the identity to a stable business
key (or an explicit, caller-supplied UUID with fallback to the title hash for template generation).

-     // Deterministic UUID v5 from policy title
-     let seed = format!("{SSP_UUID_SEED}|{title}");
-     let uuid = generate_stable_id(&seed).to_string();
+ pub fn build_ssp_with_id(
+     policy_title: &str,
+     policy_version: &str,
+     explicit_uuid: Option<Uuid>,
+ ) -> Result<SystemSecurityPlanEnvelope, ForgeError> {
+     // ...
+     let uuid = explicit_uuid
+         .unwrap_or_else(|| generate_stable_id(&seed))
+         .to_string();


══════ F0627 │ src/oscal/ssp.rs:674-678 │ [bug · medium] ══════
[bug · medium] An empty/trailing-whitespace `source_profile` is silently downgraded to the
unresolvable href "TODO-profile.json" instead of being rejected, even though this function returns
`Result` and its signature implies input validation. Consumers will receive a structurally
valid-looking SSP whose baseline reference cannot resolve, which defeats the purpose of propagating
`ForgeError::SspBuild` elsewhere. Validate and return an error for an empty profile reference.


══════ F0628 │ src/oscal/ssp.rs:712-717 │ [other · medium] ══════
[other · medium] The placeholder system identifier pairs a plausible-looking invented scheme URI
(`https://example.com/system-identifiers`) with a `TODO-system-id` value. Because both fields pass
shape/schema checks, downstream validators and reviewers cannot distinguish this fabricated data
from a legitimately registered system identifier — unresolved templates ship looking complete. Emit
nothing (or a clearly non-authoritative marker such as an unset identifier) rather than a fake
scheme. Note also that `build_ssp` assigns this to the `#[serde(skip)]` legacy `system_id` where it
is never serialized, duplicating work for callers only.


══════ F0623 │ src/oscal/test_utils.rs:13-18 │ [test · medium] ══════
[test · medium] This guard silently ignores any "remarks" value that is not a JSON string (only
`val.as_str()` is collected) while still recursing into the non-string subtree. If a producer ever
serializes structured remarks (object/array, or an upgraded dependency model changes the shape),
`collect_remarks` returns zero entries for them and every downstream trace-leak assertion
(SEC-1/SEC-2/M-7 checks in component_definition.rs and trace_embedding.rs) passes vacuously —
weakening exactly the regression guard this test utility was created to enforce. All current
producers emit `Option<String>` remarks, so this is latent, but a fail-loud design is safer for a
security check: treat a non-string "remarks" node as an anomaly instead of swallowing it.

- if key == "remarks"
-                     && let Some(s) = val.as_str()
-                 {
+ if key == "remarks" {
+     let Some(s) = val.as_str() else {
+         panic!(
+             "non-string 'remarks' value ({val}); this violates the expected \
+              OscalRemarks shape and makes leak detection unsound"
+         );
+     };
                      collected.push(s.to_string());
                  }
                  collect_remarks(val, collected);


══════ F0634 │ src/oscal/trace_embedding.rs:120-123 │ [maintainability · medium] ══════
[maintainability · medium] Silent provenance degradation: when `file_name()` returns `None` (e.g.
path ends in `..` or is a bare `/`), the control silently receives an `unknown-file` prop and an
unresolvable link with no diagnostic, and `debug!` at the end reports it as successfully annotated.
Provenance links are precisely the artifact auditors resolve later; a missing/broken one should
surface. Emit at least a `warn!` on the fallback path so operators can detect degraded trace data.

-                 let file = loc.file_path.file_name().map_or_else(
-                     || "unknown-file".to_string(),
-                     |f| f.to_string_lossy().into_owned(),
+                 let file = match loc.file_path.file_name() {
+                     Some(f) => f.to_string_lossy().into_owned(),
+                     None => {
+                         tracing::warn!(
+                             path = %loc.file_path.display(),
+                             "Source path has no file name; emitting unresolvable 'unknown-file' trace reference"
                  );
+                         "unknown-file".to_string()
+                     }
+                 };


══════ F0635 │ src/oscal/trace_embedding.rs:133-135 │ [maintainability · medium] ══════
[maintainability · medium] The group-level `source-section` attribute is derived from whichever
traceable child control happens to appear first. Children of one group legitimately come from
different files/sections (partial trace data is explicitly supported elsewhere in this function),
yet the group then asserts a single attribution with no check that the children agree — a misleading
statement in compliance output. Either aggregate distinct values (e.g. de-duplicated list), pick
deterministically with a documented rule only when all children agree, or `debug!` when child
section titles conflict.

-                 if group_section_title.is_none() {
-                     group_section_title = Some(loc.section_title.clone());
+                 match &group_section_title {
+                     None => group_section_title = Some(loc.section_title.clone()),
+                     Some(prev) if prev != &loc.section_title => tracing::debug!(
+                         group_id = %group.id,
+                         prev = %prev,
+                         new = %loc.section_title,
+                         "Group children span multiple source sections; keeping first"
+                     ),
+                     _ => {}
                  }


══════ F0632 │ src/oscal/trace_embedding.rs:55-55 │ [bug · medium] ══════
[bug · medium] Partial percent-encoding corrupts href parsing for legal local filenames. `?` is a
valid byte in POSIX/macOS file names, but in a URI reference everything after `?` is parsed as a
query component (RFC 3986 §3.4). For a source named `policy?v2.md` the emitted href becomes
`policy?v2.md#line=7`, i.e. path=`policy`, query=`v2.md#line=7` — the `#line=` fragment is swallowed
into the query and consumers resolving the link land on the wrong resource. Same class of problem
applies to other gen-delims left untouched (`:`, `@`, `[`, `]`). Encode all RFC 3986 reserved
characters (e.g. via the `percent-encoding` crate / `utf8_percent_encode` with a PATH-compatible
set) rather than just `%`, space, and `#`.

-     safe_path.replace('%', "%25").replace(' ', "%20").replace('#', "%23")
+     // Percent-encode every character outside the RFC 3986 path-allowed set so the
+     // emitted href parses back to exactly the original file name.
+     utf8_percent_encode(safe_path, PATH_SEGMENT).to_string()


══════ F0637 │ src/oscal_cli/detector.rs:119-122 │ [bug · medium] ══════
[bug · medium] `run_version_check` waits on `Command::output()` with no timeout or retry bound. A
hung, corrupt, license-prompting, or slowly-starting (JVM) oscal-cli blocks `detect()` forever,
wedging CLI startup; any caller that invokes detection inside an async task blocks the runtime
worker. The feature contract itself mandates bounded execution for resolve (`--timeout`, default
60s, watchdog kills the child) but detection is unbounded — inconsistent and unsafe. Bound the child
with `wait_timeout` (sync) or `tokio::time::timeout` + `spawn_blocking` (async), kill and reap on
expiry, and return a descriptive timeout error.

-     let output = Command::new(exe_path)
+ const VERSION_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
+
+ let mut child = Command::new(exe_path)
          .arg("--version")
-         .output()
+     .stdout(Stdio::piped())
+     .stderr(Stdio::piped())
+     .spawn()
          .map_err(|e| format!("Failed to execute oscal-cli: {e}"))?;
+
+ // Drain piped output concurrently so the pipe buffer cannot deadlock the child,
+ // then enforce the deadline with `wait_timeout` (or tokio timeout + kill/reap).
+ match child.wait_timeout(VERSION_CHECK_TIMEOUT)? {
+     Some(status) => { /* collect buffered stdout/stderr and evaluate */ }
+     None => {
+         let _ = child.kill();
+         let _ = child.wait();
+         return Err(format!(
+             "oscal-cli --version timed out after {VERSION_CHECK_TIMEOUT:?}"
+         ));
+     }
+ }


══════ F0642 │ src/oscal_cli/detector.rs:136-145 │ [bug · medium] ══════
[bug · medium] The token heuristic ('starts with a digit and contains a dot') is too loose and too
trusting: multi-line startup banners routinely embed other digit-leading dotted tokens (embedded
JRE/build info like `openjdk 17.0.10`, Maven artifact coordinates, timestamps, multi-component IDs)
anywhere after the version, and because `split_whitespace` scans everything, whichever appears first
wins. Worse, on success with empty stdout the fallback yields `""`, and `detect()` then reports
`available: true, functional: true, version: Some("")` — a functional claim with a meaningless
version. Validate semver shape (accepting `-`/`+` suffixes only at token end) and treat empty or
unmatched output as a failed check rather than echoing the whole banner.

- fn parse_version_from_output(output: &str) -> String {
-     // Look for a version-like pattern (digits.digits.digits)
-     for word in output.split_whitespace() {
-         if word.chars().next().is_some_and(|c| c.is_ascii_digit()) && word.contains('.') {
-             return word.trim().to_string();
-         }
+ fn parse_version_from_output(output: &str) -> Option<String> {
+     /// Only accept semver-shaped tokens, e.g. `1.0.3`, `2.1.0-beta`, `1.2.3+build.5`.
+     fn is_semver_like(token: &str) -> bool {
+         let core = token.split(['-', '+']).next().unwrap_or(token);
+         let segments: Vec<&str> = core.split('.').collect();
+         segments.len() == 3
+             && segments.iter().all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
      }
-     // Fall back to trimmed output
-     output.trim().to_string()
+
+     output
+         .split_whitespace()
+         .find_map(|word| is_semver_like(word).then(|| word.to_string()))
+     // Callers must treat `None` (empty/unparseable stdout) as a failed
+     // version check instead of reporting Some("") on a healthy exit code.
  }


══════ F0641 │ src/oscal_cli/detector.rs:37-40 │ [maintainability · medium] ══════
[maintainability · medium] When an explicitly configured `--oscal-cli-path` cannot be canonicalized
(typo, missing file), it collapses into `OscalCliInfo::not_found()` — indistinguishable from
'nothing found on PATH' — so the user chasing the problem edits the wrong thing. Return (or record)
a detail carrying the configured path and the underlying canonicalize error, mirroring the
contract's differentiated `OscarCliNotFound` vs `OscarCliNotFunctional { detail }` reporting.

              Some(path) => match path.canonicalize() {
                  Ok(canonical) => Some(canonical),
-                 Err(_) => return OscalCliInfo::not_found(),
+                 Err(err) => {
+                     eprintln!(
+                         "error: --oscal-cli-path '{}' does not exist or cannot be resolved: {err}",
+                         path.display()
+                     );
+                     return OscalCliInfo::not_found();
+                 }
              },


══════ F0640 │ src/oscal_cli/detector.rs:56-61 │ [maintainability · medium] ══════
[maintainability · medium] The formatted failure reason from `run_version_check` (execute error,
exit code, no-timeout detail) is discarded with `Err(_)`, so `OscalCliInfo.functional == false`
gives the user nothing actionable even though the contract specifies messages like "oscal-cli found
at '{path}' but is not functional: {detail}", and the non-zero-exit branch currently omits stderr,
the likeliest place for the root cause. Preserve the detail — e.g. add `detail: Option<String>` to
`OscalCliInfo` or at minimum log it at debug level — and include captured stderr in the exit-status
error.

-             Err(_) => OscalCliInfo {
+             Err(detail) => {
+                 eprintln!("warning: oscal-cli at {} is not functional: {detail}", exe_path.display());
+                 OscalCliInfo {
                  available: true,
                  functional: false,
                  version: None,
                  executable_path: Some(exe_path),
-             },
+                     detail: Some(detail), // surfaced to --check output per contract
+                 }
+             }


══════ F0639 │ src/oscal_cli/detector.rs:86-91 │ [bug · medium] ══════
[bug · medium] On Windows this accepts every PATHEXT entry, but hardened Rust `Command`
(post-BatBadBut/CVE-2024-24576) no longer spawns `.bat`/`.cmd` wrappers transparently the way older
versions did — discovery "succeeds" and then the version check fails, reporting a broken install for
tools users run fine through the same wrapper. Additionally, the `.exe`-vs-wrapper precedence
depends entirely on a possibly-broken user PATHEXT, and `exists()` again matches directories.
Restrict the spawnable extension list (prefer `.exe`; invoke `.bat`/`.cmd` explicitly via `cmd.exe
/C` if wrapper support is required) and gate candidates on `is_file()`.

              for ext in extensions {
                  let candidate = dir_path.join(format!("oscal-cli{ext}"));
-                 if candidate.exists() {
-                     return candidate.canonicalize().ok().or(Some(candidate));
+                 if !candidate.is_file() {
+                     continue; // skip directories and stale entries
                  }
+                 // Wrappers cannot be spawned directly by hardened `Command`:
+                 // either limit discovery to `.exe`, or launch them via
+                 // `cmd.exe /C <wrapper>` in run_version_check.
+                 return candidate.canonicalize().ok().or(Some(candidate));
              }


══════ F0646 │ src/oscal_cli/invoker.rs:106-106 │ [bug · medium] ══════
[bug · medium] `tracing` macros treat a format string containing `{context}` as a LITERAL, not an
interpolated value — the actual logs will read "oscal-cli {context} failed with stderr output"
instead of "resolve"/"convert". The `format!` calls elsewhere in this file interpolate correctly,
making the mismatch easy to miss. Pass the field explicitly: `tracing::warn!(stderr = %stderr_str,
context, "oscal-cli failed with stderr output")`. (Same problem applies to the debug! line below.)

-         tracing::warn!(stderr = %stderr_str, "oscal-cli {context} failed with stderr output");
+         tracing::warn!(stderr = %stderr_str, context, "oscal-cli failed with stderr output");


══════ F0647 │ src/oscal_cli/invoker.rs:114-114 │ [bug · medium] ══════
[bug · medium] Same tracing formatting bug as the warn! above: `{context}` stays literal in emitted
logs. Use structured fields instead: `tracing::debug!(stderr = %stderr_str, context, "oscal-cli
stderr output");`

-         tracing::debug!(stderr = %stderr_str, "oscal-cli {context} stderr output");
+         tracing::debug!(stderr = %stderr_str, context, "oscal-cli stderr output");


══════ F0648 │ src/oscal_cli/invoker.rs:159-160 │ [performance · medium] ══════
[performance · medium] To learn only the ROOT KEY of the document, this reads the entire file into
memory and fully parses it into a generic Value tree (`serde_json::Value`/`serde_yaml::Value` build
the complete DOM). Large OSCAL catalogs/assessment-results (multi-MB) pay several full copies plus
parse time for information available in the first token/event. A streaming sniff (serde_json
StreamDeserializer/IgnoredAny, a YAML event API such as libyaml-safer/saphyr, or quick_xml's
NthEvent pattern already used below) avoids whole-tree allocation while producing identical results.


══════ F0649 │ src/oscal_cli/invoker.rs:163-172 │ [maintainability · medium] ══════
[maintainability · medium] Parse failures are deliberately collapsed (via `.ok()`) into `None`, so
corrupt/truncated input surfaces as the generic "Unable to detect OSCAL model", hiding the real root
cause (e.g., serde parse error with line/column info). Return or log the underlying parse error so
users see why their document was rejected. Additional subtlety: in the yaml branch, a mapping whose
first key is non-string (mixed-type map) makes the whole detection fail even though valid key naming
could exist; filtering string keys would be more forgiving than taking strictly the first entry.


══════ F0644 │ src/oscal_cli/invoker.rs:77-83 │ [bug · medium] ══════
[bug · medium] On the timeout path the already-drained stderr is thrown away (join result
discarded), which destroys the most valuable diagnostic — what the JVM was reporting right up to the
hang. Consider capturing any partial stderr, emitting it at warn level (after sanitization for
terminal safety), and optionally embedding a snippet in the timeout error to make 'why did it hang'
diagnosable.

              Ok(None) => {
                  if start.elapsed() >= timeout {
                      let _ = child.kill();
                      let _ = child.wait(); // Reap the zombie
-                     let _ = stderr_thread.join(); // Prevent thread leak
+                     let partial = stderr_thread.join().unwrap_or_else(|_| String::new());
+                     let partial = crate::sanitize::strip_control_chars(&partial);
+                     if !partial.is_empty() {
+                         tracing::warn!(stderr = %partial, "oscal-cli timed out; stderr captured up to the kill");
+                     }
                      return Err(ForgeError::OscalCliTimeout { timeout });
                  }


══════ F0652 │ src/oscal_cli/mod.rs:17-20 │ [maintainability · medium] ══════
[maintainability · medium] The independent `available`/`functional` booleans admit invalid states
(e.g., functional=true with available=false, or functional=true with executable_path=None). The
valid invariant is currently maintained only by convention at the three construction sites in
detector.rs (not-found / found-but-nonfunctional / functional), matching the spec matrix — but
nothing in the type prevents future call sites (or external consumers, since the fields are pub)
from building contradictory states. Model the outcome as an enum, or keep this struct but make the
fields private behind a validated constructor, so illegal detection outcomes become unrepresentable.

-     /// Whether oscal-cli was found on the system PATH (or via --oscal-cli-path).
-     pub available: bool,
-     /// Whether `oscal-cli --version` succeeded (binary is functional).
-     pub functional: bool,
+ pub struct OscalCliInfo {
+     state: DetectionState,
+ }
+
+ pub enum DetectionState {
+     /// Binary not located on PATH or via --oscal-cli-path.
+     NotFound,
+     /// Located but `--version` failed.
+     NonFunctional { executable_path: PathBuf },
+     /// Located and verified.
+     Functional { version: String, executable_path: PathBuf },
+ }
+
+ impl OscalCliInfo {
+     pub fn available(&self) -> bool { !matches!(self.state, DetectionState::NotFound) }
+     // ... accessors preserve the existing call-site API
+ }


══════ F0658 │ src/parameter/matchers.rs:102-105 │ [security · medium] ══════
[security · medium] The qualifier keywords have no left word boundary. Because Rust regex has no
implicit \b and this pattern is unanchored, the sub-token 'maximum'/'at most' can fire inside a
larger identifier (e.g., "MAXIMUMLoginAge 3", "FooAtMost2") producing phantom Maximum-threshold
parameters from unrelated tokens. Same absence of \b applies to THRESHOLD_MIN ('minimum', 'no less
than', ...) and lets the match land mid-word. Since these phantom matches flow through
extract_parameters_from_text() into generated OSCAL placeholders, prepend \b to the qualifier group.

  static THRESHOLD_MAX: LazyLock<Regex> = LazyLock::new(|| {
-     Regex::new(r"(?i)(?P<qualifier>no\s+more\s+than|maximum|at\s+most)\s+(?P<value>\d+[\w-]*)")
+     Regex::new(
+         r"(?i)(?P<qualifier>\bno\s+more\s+than|\bmaximum|\bat\s+most)\s+(?P<value>\d+[\w-]*)",
+     )
          .expect("THRESHOLD_MAX regex is valid")
  });


══════ F0660 │ src/parameter/matchers.rs:169-169 │ [bug · medium] ══════
[bug · medium] Alternatives are unanchored single words, so compound cadence terms used widely in
policy text produce a false inner match: "Backups run semi-monthly" / "Reviews are bi-weekly" match
bare 'monthly'/'weekly' (preceded by '-' is still a regex token boundary), yielding a bogus
Exact-frequency parameter even though the intended cadence is semi-. Note the pattern itself accepts
hyphenated 'semi-annually', so hyphenated compounds are demonstrably in scope. The regex crate does
not support lookbehind, so either enumerate the compounds ahead of the simple words in the
alternation AND guard the left edge manually (reject when text[..start] ends with '-' or an ASCII
letter), or pre-normalize such tokens before matching. Without this, extracted labels/values
contradict the sentence they came from.

- r"(?i)(?P<prefix>at\s+least\s+)?(?P<value>annually|quarterly|monthly|weekly|daily|biannually|semi-annually)",
+ static FREQUENCY: LazyLock<Regex> = LazyLock::new(|| {
+     // Longer compounds listed first; a trailing word guard also rejects
+     // 'annuallys'-style runs on the right.
+     Regex::new(
+         r"(?i)(?P<prefix>at\s+least\s+)?(?P<value>semi-annually|biannually|annually|quarterly|monthly|weekly|daily)(?!\w)",
+     )
+     .expect("FREQUENCY regex is valid")
+ });
+ // In find_parameters(): skip matches whose preceding character is '-'
+ // or an alphanumeric, filtering out 'semi-monthly' -> 'monthly' inner hits.
+ let preceded_by_wordish = m.start() > 0 && {
+     let prev = text[..m.start()].chars().next_back();
+     matches!(prev, Some('-') | Some(c) if c.is_alphanumeric()) || prev == Some('-')
+ };


══════ F0659 │ src/parameter/matchers.rs:95-97 │ [bug · medium] ══════
[bug · medium] The greedy suffix [\w-]* makes the captured value swallow standard-number/date
fragments: e.g., "retention period of no less than 90-180 days" yields value "90-180", and text like
"...per NIST SP 800-53, retain no less than 7 years" immediately followed by '800-53'-style content
(or "no less than 12/31/2026", "no less than 800-53 accounts") produces ParameterConstraint {
Minimum, value: "800-53" | "12" , ... }. "800-53" is not a numeric parameter value yet is stored as
a Minimum value-domain constraint, which downstream parsers (mod.rs replaces the span and publishes
this value verbatim) cannot interpret. Constrain the suffix to genuinely numeric-alphanumeric
continuations such as \d+(?:\.\d+)?(?:-bit)? (or add an explicit exclusion of hyphen-led
multi-hyphen tokens), or validate the captured value is plausibly numeric before emitting the match.
Applies identically to THRESHOLD_MIN above.

+ // Value: integer (optionally decimal), optionally followed by a suffix
+ // such as "-bit"; do NOT absorb arbitrary hyphenated text (e.g., "800-53").
  Regex::new(
-     r"(?i)(?P<qualifier>at\s+least|minimum|no\s+fewer\s+than|no\s+less\s+than)\s+(?P<value>\d+[\w-]*)",
+     r"(?i)(?P<qualifier>\bat\s+least|\bminimum|\bno\s+fewer\s+than|\bno\s+less\s+than)\s+(?P<value>\d+(?:\.\d+)?)",
  )


══════ F0655 │ src/parameter/mod.rs:201-203 │ [maintainability · medium] ══════
[maintainability · medium] This guard makes idempotence depend on implicit cross-module invariants
rather than explicit state. (1) Requirements whose first pass produced ZERO parameters do not hit
this branch, so they are re-scanned on every run, and their safety relies entirely on the SEC-6
claim that `{{ insert: param }}` placeholders cannot re-match — an invariant owned by the regexes in
`src/parameter/matchers.rs`, not enforced anywhere in this file. Any future matcher change that lets
a placeholder-shaped span match will silently double-insert placeholders and permanently corrupt
`req.text` (the corrupted text then persists because `parameters` is non-empty). (2) Conversely, if
a partial run or persistence round-trip ever leaves placeholders in `text` while `parameters` came
back empty, the module happily re-extracts into already-rewritten text. Prefer explicit
processed-state tracking: record that a requirement was enriched (e.g., a bool/newtype state on
PolicyRequirement, or a scanned-spans marker in the parent call), and/or add a cheap tripwire after
rewriting that warns/errors when placeholders are present but no parameters were recorded.

      if !req.parameters.is_empty() {
          return Ok(req.parameters.len());
      }
+
+     let had_placeholders = req.text.contains("insert: param");
+     // ... after successful extraction:
+     // if had_placeholders && params.is_empty() { /* placeholders survived a
+     // re-scan: state is inconsistent — log an error instead of proceeding */ }


══════ F0656 │ src/parameter/mod.rs:205-209 │ [bug · medium] ══════
[bug · medium] The OSCAL-safe ID prefix rule (`p{stable_id}` for non-alphabetic-leading IDs) creates
three correlated hazards: (a) `param.requirement_id` is stored as the RAW `stable_id` (line below)
while `PolicyParameter.id` embeds the PREFIXED form, so any consumer that reconstructs an `id` from
`requirement_id` (e.g., cross-referencing `id-ref`s back to requirements, or rebuilding catalog
links) gets a mismatched ID unless it independently re-implements this exact prefix heuristic — the
rule now lives in two implicit places. (b) Nothing here enforces `stable_id` uniqueness across
sections; two requirements sharing a `stable_id` emit identically numbered `param.id` values
(`X_prm_0` twice), and OSCAL catalogs treat `param.id` as unique — colliding IDs silently corrupt
catalog references with no error. Extract the sanitization into one named helper reused by both
producers and consumers, and make `extract_parameters` (which already walks the whole tree) fail
fast on duplicate parameter IDs.

-     let requirement_id = if stable_id.starts_with(|c: char| c.is_alphabetic() || c == '_') {
-         stable_id.clone()
+ // Single source of truth for the ID prefix rule, reusable by consumers that
+ // rebuild param ids from `PolicyParameter.requirement_id`.
+ pub(crate) fn oscal_base_id(stable_id: &str) -> String {
+     if stable_id.starts_with(|c: char| c.is_alphabetic() || c == '_') {
+         stable_id.to_string()
      } else {
          format!("p{stable_id}")
-     };
+     }
+ }
+
+ // And in extract_parameters(), track emitted ids across the walk:
+ // let mut seen_param_ids = std::collections::HashSet::new();
+ // if !seen_param_ids.insert(id.clone()) {
+ //     return Err(ForgeError::ParameterExtraction(format!(
+ //         "duplicate OSCAL param id '{id}' (duplicate stable_id?)"
+ //     )));
+ // }


══════ F0709 │ src/parse/atomize.rs:138-152 │ [bug · medium] ══════
[bug · medium] All rewrite paths hardcode `citations: vec![]`, `modality: None`, `parameters:
vec![]` instead of carrying them over from the input requirement — including this and the other two
"preserved as-is" branches (no-split and max-splits), where the requirement was explicitly supposed
to pass through unchanged, and also the split path (which forfeits any parent-level
citations/parameters). The shared pipeline currently extracts citations/modality/parameters AFTER
atomization (pipeline.rs steps 6→7b/7c/7d), so nothing breaks there today — but
`atomize_requirement`/`atomize_document` are public library APIs, and any caller invoking them on an
already-enriched document (tests, re-runs, or a future pipeline reordering) will silently destroy
populated fields with no signal. The preserve path should clone the original requirement and only
overwrite `stable_id`/`atom_index`; the split path should propagate the parent's
citations/parameters onto children.

+         // Preserved as-is: carry the original requirement through untouched
+         // (including citations/modality/parameters) and only fill the preliminary ID.
+         let mut preserved = requirement.clone();
+         preserved.stable_id = Some(preliminary_id(text, requirement.source_line, 0));
+         preserved.atom_index = 0;
          return Ok(AtomizationResult {
-             requirements: vec![PolicyRequirement {
-                 stable_id: Some(preliminary_id(text, requirement.source_line, 0)),
-                 text: text.clone(),
-                 source_line: requirement.source_line,
-                 nesting_depth: requirement.nesting_depth,
-                 atom_index: 0,
-                 parent_text: None,
-                 citations: vec![],
-                 modality: None,
-                 parameters: vec![],
-             }],
+             requirements: vec![preserved],
              was_split: false,
              original_text: None,
          });


══════ F0710 │ src/parse/atomize.rs:203-205 │ [bug · medium] ══════
[bug · medium] The shared subject is taken from the FIRST normative verb anywhere in the text, which
is not necessarily tied to the split boundaries — producing misattributed or garbled
reconstructions: (1) a lead-in sentence like "Section 4 will state: systems must enforce MFA and
must log access" yields subject "Section 4", and the tail fragment gets prefixed to "Section 4 log
access"; (2) worse, because FIRST_VERB_PATTERN is case-sensitive, a statement starting with a
capitalized "Must use strong passwords and must expire after 90 days" skips the initial "Must",
latches onto the later lowercase "must", derives the bogus subject "Must use strong passwords" (or
just "and" for text like "and must enforce X"), and silently rewrites every requirement in ways that
then flow into OSCAL control mapping. Safer approach: derive the subject from the text immediately
preceding the FIRST SPLIT_PATTERN match, and only prepend it when each clause fragment itself begins
with a normative verb; otherwise keep the fragment verbatim and log a diagnostic instead of
guessing.

-     // Find position of the first normative verb in the text to extract shared subject
-     let shared_subject =
-         FIRST_VERB_PATTERN.find(text).and_then(|m| extract_subject(text, m.start()));
+     // Subject = text preceding the FIRST split boundary (not the first modal verb
+     // anywhere in the text), so lead-in sentences or capitalized openings that the
+     // case-sensitive verb patterns miss cannot poison every reconstructed clause.
+     let shared_subject = SPLIT_PATTERN
+         .find(text)
+         .map(|m| m.start())
+         .and_then(|first_boundary| extract_subject(text, first_boundary));
+     // Additionally, in the reconstruction loop only prepend `shared_subject` when
+     // the clause starts with a normative verb (case-insensitive); otherwise emit
+     // the clause unchanged and log a warning that subject inference was skipped.


══════ F0664 │ src/parse/clauses.rs:211-214 │ [bug · medium] ══════
[bug · medium] Exclusion deny-list is incomplete, so EC-8 ("only paragraphs within the item") is
violated: only CodeBlock/BlockQuote toggle exclude_depth, but other container blocks legal inside
list items — most notably headings (`- Intro` followed by an indented `### Step title`) — pass
through unexcluded: Start(Heading) matches nothing here, and its inner Event::Text then flows
through handle_item_text and is concatenated into the item's text, merging heading prose into the
clause. Footnote/definition and similar future block tags would leak the same way. An allow-list
(only accumulate text while inside Tag::Paragraph, mirroring the exclude_depth technique) is more
robust than adding each offender to this pattern.

-         Event::Start(Tag::CodeBlock(_) | Tag::BlockQuote(_)) if !state.item_stack.is_empty() => {
-             state.exclude_depth += 1;
-             true
-         }
+         // Prefer an allow-list: track paragraph scope explicitly so headings,
+         // HTML blocks, footnotes, etc. inside an item cannot leak into text,
+         // instead of enumerating each construct to deny.


══════ F0665 │ src/parse/clauses.rs:376-379 │ [bug · medium] ══════
[bug · medium] HardBreak is silently dropped in paragraph and table-cell accumulation: only
Event::SoftBreak is matched here (and likewise in handle_table_event's cell handler), so a paragraph
or cell containing a hard break (backslash/EOL or <br>) renders adjacent tokens concatenated with no
separator ("line1" + "line2" -> "line1line2"), corrupting extracted text fidelity. handle_item_text
correctly maps both SoftBreak and HardBreak to a space — make this handler and the table handler
consistent with `Event::SoftBreak | Event::HardBreak`.

-         Event::SoftBreak if state.in_standalone => {
+         Event::SoftBreak | Event::HardBreak if state.in_standalone => {
              state.text.push(' ');
              true
          }


══════ F0705 │ src/parse/mod.rs:143-149 │ [bug · medium] ══════
[bug · medium] Line breaks inside a heading are dropped entirely. `Event::SoftBreak`/`HardBreak`
while `in_heading` fall through to `_ => {}`, so titles from multi-line headings are concatenated
without any separator: a setext-style title "## Access\nControl Overview" yields title
"AccessControl Overview" instead of "Access Control" (or "Access\nControl Overview"). The body
accumulator right below handles breaks by pushing '\n', so heading handling should be symmetrical —
add `Event::SoftBreak | Event::HardBreak if in_heading => title_buf.push(' ')` (or '\n') to the
match. Inline `Code` is captured, which makes the inconsistency easier to hit (e.g. "## `foo` bar").

+             Event::SoftBreak | Event::HardBreak if in_heading => {
+                 title_buf.push(' '); // preserve word separation in multi-line titles
+             }
              Event::Text(_)
              | Event::Code(_)
              | Event::SoftBreak
              | Event::HardBreak
              | Event::End(TagEnd::Paragraph | TagEnd::Item | TagEnd::List(_))
                  if !in_heading =>
              {


══════ F0684 │ src/parse/modality.rs:143-146 │ [security · medium] ══════
[security · medium] These `warn!` calls embed up to 120 characters of raw policy requirement text at
WARN level, contradicting SEC-1 of docs/SEC/033-sec-normative-advisory-detection.md: "Requirement
text shall not be logged at INFO level or above (DEBUG only)" and its claim that "Logs contain only
matched verb keywords". For documents where every requirement lacks modal verbs, this floods stderr
at WARN with verbatim excerpts of potentially confidential/internal policy content (note this crate
ships a dedicated `sanitize` module for exactly this class of data). Keep the WARN signal but drop
the excerpt from non-debug records, or gate the preview behind DEBUG level.

-         if result.is_default || result.has_conflict {
-             let preview: String = req.text.chars().take(120).collect();
-             if result.is_default {
-                 warn!(text = preview, "No modality verb detected — defaulting to Normative");
+         if result.has_conflict {
+             // SEC-1 (docs/SEC/033): requirement text is DEBUG-only; verbs suffice at WARN.
+             warn!(
+                 verbs = ?result.matched_verbs,
+                 "Conflicting normative/advisory verbs — Normative wins"
+             );
+         } else if result.is_default {
+             warn!("No modality verb detected — defaulting to Normative");
+         }


══════ F0688 │ src/parse/modality.rs:143-146 │ [security · medium] ══════
[security · medium] These `warn!` calls embed up to 120 characters of raw policy requirement text at
WARN level, contradicting SEC-1 of docs/SEC/033-sec-normative-advisory-detection.md: "Requirement
text shall not be logged at INFO level or above (DEBUG only)" and its claim that "Logs contain only
matched verb keywords". For documents where every requirement lacks modal verbs, this floods stderr
at WARN with verbatim excerpts of potentially confidential/internal policy content (note this crate
ships a dedicated `sanitize` module for exactly this class of data). Keep the WARN signal but drop
the excerpt from non-debug records, or gate the preview behind DEBUG level.

-         if result.is_default || result.has_conflict {
-             let preview: String = req.text.chars().take(120).collect();
-             if result.is_default {
-                 warn!(text = preview, "No modality verb detected — defaulting to Normative");
+         if result.has_conflict {
+             // SEC-1 (docs/SEC/033): requirement text is DEBUG-only; verbs suffice at WARN.
+             warn!(
+                 verbs = ?result.matched_verbs,
+                 "Conflicting normative/advisory verbs — Normative wins"
+             );
+         } else if result.is_default {
+             warn!("No modality verb detected — defaulting to Normative");
+         }


══════ F0728 │ src/pipeline.rs:142-151 │ [maintainability · medium] ══════
[maintainability · medium] `run_catalog_pipeline` and `run_component_pipeline` duplicate large
identical stretches beyond `prepare_document`: the version-defaulting warning,
`ConversionStatistics` assembly, the Json/Xml/Yaml format match, and nearly the whole
assessment-plan secondary-output block. Any future fix (e.g., the validation/formatting improvements
flagged in this file) must be applied twice and can drift — as has already happened with
stage-progress tracing being absent here but present in the component pipeline. Extract shared
helpers (e.g., `finalize_statistics(...)`, `serialize_in_format(...)`,
`build_ap_secondary_output(...)`) parameterized by strategy-specific values.

-     use crate::summary::{ValidationStatus, count_catalog_controls};
-
-     // Steps 1-9: shared pipeline stages
-     let doc_with_ids = prepare_document(input_path, max_size_bytes)?;
-     if doc_with_ids.metadata.version == "0.0.0" {
-         tracing::warn!(
-             source = %input_path.display(),
-             "document version not found; defaulting OSCAL metadata version to 0.0.0"
-         );
-     }
+     // Shared with run_component_pipeline:
+     let (stats_base, warnings) =
+         post_prepare_common(&doc_with_ids, input_path, Strategy::Catalog);


══════ F0731 │ src/pipeline.rs:238-244 │ [bug · medium] ══════
[bug · medium] The primary catalog is force-fed through `validate_and_serialize` (schema + semantic,
WI-20), but the assessment plan secondary artifact is serialized straight to JSON with no validation
at all — despite `complete_assessment_plan` documenting OSCAL minimum-content invariants and the
catalog semantic checks living one call away. An invalid AP can therefore ship silently alongside a
fully validated primary artifact, breaking the auto-validation guarantee the module advertises. At
minimum run the generated AP through the validator (an `OscalModelType::AssessmentPlan` variant +
schema) or fail loudly until AP validation is supported.

-         let subjects = crate::oscal::create_assessment_subjects(
-             None, // Catalog pipeline: no component definition UUID available
-             &envelope.catalog.metadata.title,
-         );
          crate::oscal::complete_assessment_plan(&mut ap_envelope, tasks, subjects)?;
+         // Validate the secondary artifact with the same rigor as the primary.
+         validate_and_serialize(
+             &ap_envelope,
+             "assessment plan",
+             crate::validate::OscalModelType::AssessmentPlan,
+         )?;
          let ap_json = serde_json::to_string_pretty(&ap_envelope)
              .map_err(|e| ForgeError::Serialization(e.to_string()))?;


══════ F0732 │ src/pipeline.rs:328-330 │ [security · medium] ══════
[security · medium] Falls back to `input_path.display()` when `file_name()` is `None` (paths ending
in `..`, `.` separators, or bare roots), which writes the caller's possibly-absolute path text
straight into the component-definition artifact — exactly what the SEC-1 comment two lines up
forbids. Note the parallel code in `trace_embedding::embed_trace_in_catalog` correctly falls back to
the constant `"unknown-file"`; do the same here.

-     let source_file_str = input_path
-         .file_name()
-         .map_or_else(|| input_path.display().to_string(), |f| f.to_string_lossy().into_owned());
+     // SEC-1: never fall back to the full path — emit a neutral placeholder
+     // instead of leaking absolute paths into the OSCAL output.
+     let source_file_str = input_path.file_name().map_or_else(
+         || "unknown-file".to_owned(),
+         |f| f.to_string_lossy().into_owned(),
+     );


══════ F0733 │ src/pipeline.rs:397-403 │ [bug · medium] ══════
[bug · medium] Same unvalidated-secondary-output gap as in `run_catalog_pipeline`: the assessment
plan is serialized directly with no schema/semantic validation, so an invalid AP document ships
silently next to a fully validated component definition. Validate it (or make the omission explicit
and consistent) before returning it as output.

-         let subjects = crate::oscal::create_assessment_subjects(
-             Some(&envelope.component_definition.uuid),
-             &envelope.component_definition.metadata.title,
-         );
          crate::oscal::complete_assessment_plan(&mut ap_envelope, tasks, subjects)?;
+         validate_and_serialize(
+             &ap_envelope,
+             "assessment plan",
+             crate::validate::OscalModelType::AssessmentPlan,
+         )?;
          let ap_json = serde_json::to_string_pretty(&ap_envelope)
              .map_err(|e| ForgeError::Serialization(e.to_string()))?;


══════ F0726 │ src/pipeline.rs:55-60 │ [maintainability · medium] ══════
[maintainability · medium] All diagnostic information from the `ValidationReport` is discarded: only
the error *count* is propagated, while `report.errors()` carries the structured details
(location/context, expected vs actual) that would explain why internally generated OSCAL failed its
own schema. Since these failures arise from bugs in this crate's own generators, the count alone
makes diagnosis very hard. Include at least a bounded summary of the individual errors in the
message (swap `{err}` for `{err:?}` if `ValidationError` lacks `Display`).

      if !report.is_valid() {
-         return Err(ForgeError::SchemaValidation(format!(
-             "{} validation error(s) in generated {label}",
+         let mut msg = format!(
+             "{} validation error(s) in generated {label}:",
              report.errors().len()
-         )));
+         );
+         for err in report.errors().iter().take(10) {
+             msg.push_str(&format!("\n  - {err}"));
+         }
+         return Err(ForgeError::SchemaValidation(msg));
      }
