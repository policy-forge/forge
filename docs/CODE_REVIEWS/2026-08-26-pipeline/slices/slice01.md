# Validation slice slice01 — 60 findings
Severity mix: critical×3, unspecified×6, high×51


══════ F1054 │ .github/workflows/release.yml:170-170 │ [security · critical] ══════
[security · critical] This reusable workflow is granted id-token:write and contents:write yet is
pinned by the mutable tag `v2.1.0`, unlike every other action in this file which is pinned to a full
commit SHA. A moved/hijacked tag here would hand full control of the signing/provenance pipeline to
an attacker, compromising the entire supply-chain attestation chain this workflow exists to provide.

-     uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0
+     # Pin to the full commit SHA that corresponds to v2.1.0.
+     uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@<full-commit-sha-of-v2.1.0>


══════ F0383 │ src/config.rs:0-0 │ [security · critical] ══════
[security · critical] The symlink-containment walk stops at the DEEPEST EXISTING ancestor and trusts
its canonicalization. If every directory along the tail (`resolved`) already exists, each iteration
canonicalizes its own parent, so the final `canonical_ancestor` ends up being just `project_root`
itself when it should be the real target. Attack example: root contains directory symlink `esc ->
../..`; a hostile `.forge.toml` sets `output = "esc/d/e/out.json"`. Walk: 'd' does not exist -> jump
to `root/esc` -> canonicalize -> `<parent-of-root>`; rebuilt = `<parent-of-root>/d/e/out.json`. Now
suppose `/etc` contains `d/e` (or `/tmp` where anyone can pre-create them):
`rebuilt.starts_with(canonical_root)` is FALSE only because `d` happens to exist there — flip it
around: choose `output = "d/../.."`-style non-existing mixes, or simply point `esc ->
/home/user/.forge-out` with config `output = "esc/sub/new.json"`: rebuilt =
`/home/user/.forge-out/sub/new.json`, which passes entirely because nothing checks that the
symlink-tail was expanded rather than kept lexically. Net effect: paths whose whole purpose is
symlink expansion end up validated against the WRONG ancestor (or accepted outright when
intermediate components also exist inside the external target), defeating the M-9 trust boundary the
function exists to enforce.


══════ F0480 │ src/lifecycle/mod.rs:600-607 │ [security · critical] ══════
[security · critical] Record-controlled artifact/source paths are used to build filesystem reads
without confinement. `record::validate` (validate_fingerprint in record.rs) only checks `non_empty`
on stored paths; it does not reject absolute paths, drive prefixes, or `..` components. Here
`base.join(&record.policy.source.path)` and `base.join(&expected.path)` will DISCARD `base` entirely
when the stored path is absolute (e.g. "/etc/passwd" or "C:\\evil\\policy.md"), and `../..` escapes
reach outside the record directory just as easily. `fingerprint()` then happily reads, sizes, and
SHA-256 hashes those out-of-tree bytes, and `relative_path()` accepts any pair of absolute paths
(they share at least the root component), producing the reported fingerprints from attacker-chosen
files. Consequences: (1) arbitrary file read smuggled into a supposedly closed local evidence chain,
with the digest emitted in status/attestation output acting as a hash oracle; (2) trust decisions
(approved-drift detection, transitions) can be steered to whichever bytes the record author names.
The same join pattern recurs in the proposal-output alias loop of `execute_transition` and in
`validate_report_destination`. Fix: enforce confinement whenever consuming stored paths — reject any
path whose components include RootDir/Prefix/ParentDir/CurDir, and additionally assert
`path.canonicalize()?.starts_with(&base)` before opening.

      let base = record_directory(record_path)?;
-     let source_path = base.join(&record.policy.source.path);
+     let source_path = confined_join(&base, &record.policy.source.path)?;
      let source = fingerprint(&source_path, &base, false)?;
      let mut generated = Vec::new();
      let mut identity_changes = Vec::new();
      for expected in &record.policy.generated_artifacts {
-         let path = base.join(&expected.path);
+         let path = confined_join(&base, &expected.path)?;
          let actual = fingerprint(&path, &base, true)?;


══════ F1062 │ .github/workflows/ci.yml:66-68 │ [bug · high] ══════
[bug · high] Steps execute sequentially, so if any earlier step fails (tests, provenance check,
clippy, release build), every remaining step is skipped — including `Security audit`, `License and
advisory check`, and `Supply-chain audit`. A routine test failure therefore silently suppresses all
security signals, meaning new advisories/licenses/supply-chain problems introduced alongside the
regression go undetected until the next fully green commit. Either make the three audit steps
resilient with `if: always()` — noting they still require the repo built, so add `--frozen` installs
won't help without a successful build — or (better) move audits into a dedicated `audit` job that
depends only on checkout/toolchain, so they always produce a signal.

        - name: Security audit
-         if: matrix.os == 'ubuntu-latest'
+         if: matrix.os == 'ubuntu-latest' && always()
          run: cargo audit
+
+ # Or preferred: a standalone job so audits don't depend on build success
+ #  audit:
+ #    runs-on: ubuntu-latest
+ #    timeout-minutes: 30
+ #    steps:
+ #      - uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd
+ #      - run: cargo install cargo-audit cargo-deny cargo-vet --locked
+ #      - run: cargo audit && cargo deny check && cargo vet --locked


══════ F1056 │ .github/workflows/release.yml:199-199 │ [bug · high] ══════
[bug · high] `sha256sum` failures are silently swallowed in two places because these steps run under
the default `bash -e` shell without `pipefail`. In the SHA256SUMS step, `2>/dev/null` hides 'No such
file or directory' errors when a glob class (*.tar.gz, *.zip, *.cdx.json) matches nothing, and the
missing pipefail means a failing `sha256sum | sort` pipeline still reports success — yielding an
incomplete SHA256SUMS file that is released and no longer matches the subjects attested by the SLSA
provenance. In the follow-up step exporting hashes to `$GITHUB_OUTPUT`, a partial sha256sum failure
is likewise masked (its exit status hidden behind `sort`/the outer `echo`), so truncated or empty
hashes could be published as provenance subjects without failing the job. Fix both by declaring
`shell: bash` with `set -euo pipefail`, removing the `2>/dev/null`, and failing fast if any expected
artifact class is absent.

-           sha256sum -- *.tar.gz *.zip *.cdx.json 2>/dev/null | sort > SHA256SUMS
+           shell: bash
+           run: |
+             cd artifacts
+             # Fail loudly instead of hiding unmatched-glob errors;
+             # optionally assert the manifest matches needs.hash.outputs.hashes.
+             sha256sum -- *.tar.gz *.zip *.cdx.json | sort > SHA256SUMS


══════ F1068 │ .gitignore:6-9 │ [security · high] ══════
[security · high] Ignoring Cargo.lock is wrong for this project: the root Cargo.toml is a binary
application (clap CLI + six bench targets), and the project's own security requirement SEC-9
(docs/SEC/049-sec-cross-platform-release.md, lines 243/254/415) mandates "Cargo.lock must be
committed to the repository and verified during CI builds" to pin dependency versions against
supply-chain attacks. Committing the lockfile guarantees reproducible builds for
binaries/applications (the standard Cargo guidance for executable packages). Practical side effects
today: (a) documented release steps `git add Cargo.toml Cargo.lock ...`
(specs/035-prd-phase2-release/tasks.md T038, plan.md line 276, quickstart.md line 92) will silently
skip the ignored lockfile; (b) CI cache keys in .github/workflows/ci.yml and release.yml use
hashFiles('**/Cargo.lock'), which resolves to an empty string when the file is absent, collapsing
every run onto one shared, potentially stale cache. Remove this line and commit Cargo.lock.

  *.rlib
  *.prof*
- Cargo.lock
  *.log
+ # NOTE: Cargo.lock must be committed (SEC-9); do NOT list it here.


══════ F0029 │ benches/export_bench.rs:70-74 │ [test · high] ══════
[test · high] Silent skip hides broken CI signal: if the fixture is missing or gets renamed, `cargo
bench` exits successfully having run zero benchmarks, and historical Criterion data just stops
growing. A guard against a missing committed fixture should fail loudly (panic / return an error) or
record an explicit failed/skipped sample so regressions cannot go unnoticed.

      let fixture_path = Path::new(FIXTURE_PATH);
      if !fixture_path.exists() {
-         tracing::warn!(fixture = %FIXTURE_PATH, "Skipping export benchmark: fixture not found");
-         return;
+         panic!("benchmark fixture missing: {FIXTURE_PATH} (commit it or fix FIXTURE_PATH)");
      }


══════ F0025 │ benches/parameter_extraction.rs:108-113 │ [performance · high] ══════
[performance · high] Same timed-region clone problem as the 500-requirement bench: the deep clone of
a 100-requirement document (plus all its Strings/Vectors) is included in the measured time,
inflating the reported throughput for extraction itself.

      c.bench_function("extract_parameters/100_requirements", |b| {
-         b.iter(|| {
-             let mut d = black_box(doc.clone());
-             extract_parameters(&mut d).expect("extract_parameters must not fail");
-         });
+         b.iter_batched(
+             || doc.clone(),
+             |mut d| {
+                 black_box(extract_parameters(&mut d))
+                     .expect("extract_parameters must not fail");
+             },
+             BatchSize::SmallInput,
+         );
      });


══════ F0024 │ benches/parameter_extraction.rs:96-101 │ [performance · high] ══════
[performance · high] `doc.clone()` runs inside the timed region, so every measured iteration pays a
full deep clone of a 500-requirement document (hundreds of String/Vec allocations) on top of the
actual extraction. The reported timing therefore conflates clone cost with extractor cost and cannot
honestly be compared against the PRD NF-1 p95 ≤ 1s target. Since `extract_parameters` mutates the
document in place, you do need a fresh copy per iteration — but create it in the setup closure of
`iter_batched` so Criterion excludes it from timing. Also consider `black_box`ing the returned
Result so the optimizer cannot elide the measured work.

      c.bench_function("extract_parameters/500_requirements", |b| {
-         b.iter(|| {
-             let mut d = black_box(doc.clone());
-             extract_parameters(&mut d).expect("extract_parameters must not fail");
-         });
+         b.iter_batched(
+             || doc.clone(),
+             |mut d| {
+                 black_box(extract_parameters(&mut d))
+                     .expect("extract_parameters must not fail");
+             },
+             BatchSize::SmallInput,
+         );
      });


══════ F0051 │ benches/pipeline_benchmark.rs:107-108 │ [bug · high] ══════
[bug · high] Parity bug with production: PolicyDocument::collect_citations() clones EVERY
requirement's citations with no deduplication, whereas run_catalog_pipeline uses
forge::oscal::component_definition::collect_all_citations(&doc_with_ids.sections), which dedups by
citation id and caps at MAX_CITATIONS (10_000). On the 50-page fixture (~200 requirements citing
overlapping sources) this inflates duplicate back-matter resources, so generate_back_matter and the
serialization_json/serialization_yaml stage benchmarks measure larger output than any production run
produces. Since build_catalog_envelope feeds the full-pipeline, catalog_assembly, and both
serialization benchmarks, every one of those numbers skews high.

-     let citations = doc.collect_citations();
+     let citations =
+         forge::oscal::component_definition::collect_all_citations(&doc.sections);
      let (back_matter_resources, _) = forge::oscal::generate_back_matter(&citations)?;


══════ F0050 │ benches/pipeline_benchmark.rs:38-38 │ [bug · high] ══════
[bug · high] This hand-copied "mirror" of src/pipeline.rs::prepare_document/run_catalog_pipeline has
drifted: production additionally runs Step 7c annotate_modalities(), Step 7d extract_parameters(),
and the EC-6 no-structure guard, none of which are present here. Production also recomputes
counts/warnings ignored here. As a result, 'full_pipeline/catalog_50page' reports timings for a
pipeline that no longer exists — modality annotation and parameter extraction cost (both post-ID
stages in production) are silently excluded. If src/pipeline.rs is refactored again, this copy will
keep aging. Prefer calling a shared crate entry point (or at least prepare_document if made public)
so the benchmark cannot drift.


══════ F0099 │ benches/xml_benchmark.rs:35-38 │ [bug · high] ══════
[bug · high] Benchmark-validity: this replicated pipeline diverges from the real one. Production
goes through forge::pipeline::prepare_document(), which after extract_citations() additionally runs
parse::annotate_modalities() (WI-33) and parameter::extract_parameters() (WI-34) before
build_catalog/build_component_definition. prepare_document is `pub(crate)` so benches cannot call
it, but re-implementing only part of the stages here means the controls being serialized lack the
`modality` prop and all OSCAL `param` elements (labels/values/constraints) that every real export
contains. For a ~200-requirement fixture that omits a meaningful share of XML payload, so this
benchmark systematically understates the work serialize_catalog_to_xml /
serialize_component_definition_to_xml perform on real artifacts — potentially masking violations of
the <50ms budget. Add the two missing stages (annotated doc must flow into the catalog/component
build; apply to build_component_def_from_fixture identically), or better, make prepare_document
accessible to benches/tests.

      let doc = forge::uuid::assign_stable_ids(atomized);
      let doc = forge::citation::extract_citations(doc).unwrap();
+
+     // Match forge::pipeline::prepare_document() stages 7c/7d:
+     let doc = forge::parse::annotate_modalities(doc).expect("annotate modalities");
+     let mut doc = doc;
+     forge::parameter::extract_parameters(&mut doc).expect("extract parameters");

      let mut trace_links = forge::TraceLinkCollection::new();


══════ F0909 │ ci/integration-test.sh:154-159 │ [bug · high] ══════
[bug · high] This pipeline is assigned inside command substitution under `set -euo pipefail`, so it
can kill the whole script before the intended `[[ -z "${CONTROL_IDS}" ]]` fallback is ever reached:
(a) with >20 extracted IDs, `head -20` closes the read end and the upstream stages die with SIGPIPE
(exit 141); (b) if the first grep finds no match, or the inverse `grep -v` filters out everything,
it exits 1 and pipefail propagates the failure. The graceful 'no control IDs found' branch below is
effectively dead code for these failure modes. Additionally, regex-scraping JSON for `"id"` keys is
brittle (it can pick up nested/unrelated `id` fields anywhere in the catalog) and silently truncates
the included-control list at 20, which may diverge from what golden fixtures expect. Consider `sed
-n '1,20p'` instead of `head` (consumes the whole stream, no SIGPIPE), tolerate grep exit 1, and
ideally parse the catalog structurally (e.g. with jq/python) limited to `groups[].controls[].id`.

+ # `|| true` tolerates grep exit 1 (no match); sed -n '1,20p' replaces `head`
+ # so the upstream side is never SIGPIPE-killed (exit 141) by an early reader
+ # close under `set -o pipefail`.
  CONTROL_IDS=$(grep -o '"id" *: *"[A-Za-z0-9_-]*"' "${CATALOG_JSON}" \
      | sed 's/"id" *: *"//;s/"//' \
-     | grep -v '_smt$\|_gdn$\|_obj$\|title-' \
-     | head -20 \
+     | { grep -v '_smt$\|_gdn$\|_obj$\|title-' || true; } \
+     | sed -n '1,20p' \
      | tr '\n' ',' \
      | sed 's/,$//')


══════ F0911 │ ci/integration-test.sh:215-217 │ [bug · high] ══════
[bug · high] `--import-ssp` points at './system-ssp.json', a file this script never creates, copies,
or verifies (and there is no CLI way to produce an SSP — see the adjacent Step 1h skip note that
says SSP is library-only). Relative to the repo-root `cd`, the file almost certainly does not exist,
so when the flag IS supported this named check either spuriously fails CI or, if forge tolerates the
missing input, validates nothing while still printing 'PASS'. Generate a minimal valid SSP fixture
into ${TMPDIR} first (library/integration setup), point the flag at it, or make the absence of a
usable SSP input an explicit SKIP like the other optional-feature paths.

+     MINIMAL_SSP="${TMPDIR}/minimal-system-ssp.json"
+     # TODO: provision a valid minimal SSP fixture at ${MINIMAL_SSP} before
+     # invoking --import-ssp; otherwise record a SKIP instead.
      check "Generate assessment-plan (JSON)" \
          ${FORGE} convert "${FIXTURE}" --strategy catalog --format json \
-             --output "${TMPDIR}/catalog-ap.json" --import-ssp "./system-ssp.json"
+             --output "${TMPDIR}/catalog-ap.json" --import-ssp "${MINIMAL_SSP}"


══════ F0910 │ ci/integration-test.sh:308-308 │ [bug · high] ══════
[bug · high] In ci/integration-test.sh both ${AP_JSON} and ${PROFILE_JSON} are expanded without the
':-' guard the other late-bound artifacts use; when the corresponding generation stages are skipped
on older builds, set -u aborts mid-suite ('unbound variable') instead of reaching the intended SKIP
paths. Initialize both alongside CATALOG_XML or expand with ${VAR:-}. Also anchor the forge-validate
help detection to a distinctive token such as --schema-type.*profile rather than the bare substring
'profile' to avoid false positives.

- if [[ -f "${AP_JSON}" && -s "${AP_JSON}" ]]; then
+ if [[ -f "${AP_JSON:-}" && -s "${AP_JSON:-}" ]]; then


══════ F0017 │ examples/component-based/generate_ssp.py:31-33 │ [bug · high] ══════
[bug · high] Every implemented requirement unconditionally links to the '#component-web-application'
fragment, regardless of which component supplied the control, so controls inherited from the
database component are mis-attributed. Worse, the fragment anchor does not match any emitted
identifier: the SSP components are assigned uuid5 values (stable_uuid("web-application") /
stable_uuid("database")), so '#component-web-application' dangles and cannot be resolved by OSCAL
validators or consumers expecting href targets to reference valid component/back-matter UUIDs. The
link should carry the concrete component UUID chosen for that control (or be dropped).

  "links": [
-             {"href": "#component-web-application", "rel": "implements"}
+             {"href": "#" + component_uuid_for(cid), "rel": "implements"}
          ]


══════ F0006 │ examples/component-based/output/catalog-new.json:12-13 │ [bug · high] ══════
[bug · high] Generation defect: the raw YAML front-matter of policy.md was mis-parsed as a policy
section and emitted as its own catalog group. The id is a slugified dump of the front-matter text,
and the title concatenates unprocessed key/value pairs with no separator between fields
('...logins"version: ...'), which is neither valid markdown nor human-readable. Downstream OSCAL
consumers will surface this as a bogus, empty policy group that duplicates the document title (also
in the next group and in catalog.metadata.title). The generator should skip/handle front-matter
instead of converting it into a group.


══════ F0007 │ examples/component-based/output/catalog-new.json:156-157 │ [bug · high] ══════
[bug · high] Control titles are stored truncated with a literal '...' tail by the generator, while
the authoritative `_smt` prose holds the full requirement — and truncation can break markup, not
just omit words. Confirmed instances: POL-CSP-004 (clipped at 'admin, editor, and v...', factually
omitting the third role 'viewer'); POL-CSP-021 (cut at 'of backup ...'); POL-CSP-025 (cut
mid-parameter id-ref); and POL-CSP-022, where the clip lands inside a placeholder token ('{{ insert:
param, id-ref: p7bc6fb2f-376d-57ab-97c1-...') so the reference no longer resolves to the declared
param 'p7bc6fb2f-376d-57ab-97c1-46fbdb109150_prm_0' — any template engine honoring insert:param
markers will fail or leave dangling markup, while only the `_smt` prose carries the well-formed
reference. Long titles must be stored in full (or wrapped), never clipped with an ellipsis by the
generator.

              "id": "POL-CSP-004",
-             "title": "The application component SHALL enforce role-based access control (RBAC) with at least three roles: admin, editor, and v...",
+             "title": "The application component SHALL enforce role-based access control (RBAC) with at least three roles: admin, editor, and viewer",


══════ F0005 │ examples/simple-access-control/output/profile.json:12-12 │ [maintainability · high] ══════
[maintainability · high] Import href embeds an absolute, user-specific filesystem path
(/Users/bluby/.hermes/...) instead of a portable URI. This path will not resolve on any other
machine, CI runner, or checkout, so the profile fails to load outside the original author's
environment. Since catalog.json lives in the same output/ directory as this profile.json, use a
relative href.

- "href": "/Users/bluby/.hermes/kanban/boards/forge/workspaces/t_86841893/examples/simple-access-control/output/catalog.json",
+ "href": "catalog.json",


══════ F0903 │ scripts/pre-commit.sh:22-24 │ [bug · high] ══════
[bug · high] Checks validate the working tree, not the staged snapshot. With partial staging, `cargo
fmt/clippy/test` run over uncommitted edits while the commit ships other content, so the committed
code was never verified. Typical fixes: bail when `git diff --quiet` reports unstaged changes, or
temporarily stash unstaged hunks before running the gates.

+ # Gates below validate the working tree, not the staged snapshot. Refuse to run
+ # when unstaged edits exist, otherwise a partially staged commit can ship code
+ # that was never formatted/linted/tested.
+ if ! git diff --quiet; then
+     echo "[pre-commit] unstaged changes detected; stage (or stash) everything first" >&2
+     exit 1
+ fi
+
  run_step "cargo fmt --check" cargo fmt --check
  run_step "cargo clippy -- -D warnings" cargo clippy -- -D warnings
  run_step "cargo test" cargo test


══════ F0014 │ sonar-project.properties:17-17 │ [bug · high] ══════
[bug · high] This coverage setting is fragile in two ways and will fail silently (quality gate shows
0% coverage instead of raising an error). (1) Property key: neither SonarCloud's built-in Rust
analyzer nor the common community Rust plugin documents "sonar.rust.lcov.reportPaths" — the
community plugin expects "community.rust.lcov.reportPaths"; if the running analyzer does not
recognize this exact key, the report is quietly ignored. (2) Report path: "lcov.info" is resolved
relative to the project base dir, so the CI step must write the LCOV to exactly
<repo-root>/lcov.info (default cargo-llvm-cov output goes elsewhere). Pin the CI command, e.g.
`cargo llvm-cov --lcov --output-path lcov.info` executed at the workspace root, or point this
property at the artifact's actual location.

+ # Ensure CI runs (at repo root):
+ #   cargo llvm-cov --lcov --output-path lcov.info
+ # and confirm the property key is honored by the Rust analyzer in use
+ # (the community plugin uses community.rust.lcov.reportPaths).
  sonar.rust.lcov.reportPaths=lcov.info


══════ F0308 │ src/applicability/mod.rs:305-305 │ [security · high] ══════
[security · high] Edge reviewer authorization accepts ANY declared metadata party, not just parties
holding the 'mapping-reviewer' role. reviewer_uuids is correctly restricted to
role='mapping-reviewer' (and used only to require at least one such party exists), but the set
passed to validate_mapping_edges is parties.keys() — i.e., every party. An attacker-controlled
Mapping Collection can therefore declare an unrelated person/organization party and cite it as FORGE
'reviewer-key', passing review-provenance validation with no actual mapping-reviewer involvement,
defeating the guarantee the role check exists to provide. Return/consume the reviewer_uuids set
instead.

-     let party_uuids = parties.keys().copied().collect();
+     // Only parties holding the 'mapping-reviewer' role may author reviewed edges.
+     let reviewer_uuids_set = reviewer_uuids.clone();
+     let reviewers = reviewer_uuids
+         .into_iter()
+         .map(|uuid| /* ... unchanged ... */)
+         .collect::<Result<Vec<_>, _>>()?;
+     Ok((reviewers, reviewer_uuids_set))


══════ F0286 │ src/batch/output_naming.rs:32-34 │ [bug · high] ══════
[bug · high] Collision avoidance ignores state on disk, which enables silent data loss. `claimed`
only tracks names minted in this call, so: (a) any `policy.json` already present under `base_dir` is
silently overwritten with no suffix; (b) worse, when `output_dir` is None an input such as
`notes.json` with the same target format derives its output onto itself (`./notes.json`), clobbering
the source file — the alias is invisible to `claimed` because the input path text differs from the
`.`-prefixed derived path. Either probe the filesystem before finalizing names (rename-with-suffix
or fail), surface an explicit overwrite policy (e.g. rename_if_exists / overwrite / error), or at
minimum detect and reject `input == output` cases upstream. Right now nothing in this unit guards
it.


══════ F0298 │ src/citation.rs:206-209 │ [bug · high] ══════
[bug · high] Two occurrences of the same citation text within one requirement (e.g., the
duplicate-URL case exercised by us1_duplicate_urls_produce_separate_citations) hash to the identical
UUID v5, because the name input only covers "{requirement_id}:{citation_text}". The resulting
separate Citation entries share the same `id`; if citation ids are later used as OSCAL references,
map keys, or anchors, entries silently collide (last-write-wins) and per-occurrence provenance is
lost. Make the id generation collision-free by mixing in the occurrence ordinal (or byte offset) of
the match.

- pub fn generate_citation_id(requirement_id: &str, citation_text: &str) -> String {
-     let input = format!("{requirement_id}:{citation_text}");
+ pub fn generate_citation_id(
+     requirement_id: &str,
+     citation_text: &str,
+     occurrence: usize,
+ ) -> String {
+     let input = format!("{requirement_id}:{occurrence}:{citation_text}");
      Uuid::new_v5(&FORGE_NAMESPACE_UUID, input.as_bytes()).to_string()
  }


══════ F0348 │ src/cli/resolve.rs:68-72 │ [bug · high] ══════
[bug · high] Data-integrity hazard: the user-supplied (or derived) output path is neither validated
against the input nor rejected when identical. `forge resolve profile.json --output profile.json`
lets oscal-cli write the resolved catalog onto the very profile it is reading, destroying the
source. There is also no existence/parent-dir check to catch a mistake early, and the
non-canonicalized user path contradicts ResolveResult's documented contract of returning an absolute
output path. Add equivalence guards (raw and canonicalized) and keep the recorded output path
consistent.

      // Derive default output path if not provided
      let output_path = match output {
          Some(p) => p.to_path_buf(),
          None => derive_default_output_path(&canonical_input),
      };
+
+     // SEC/data-integrity guard: never allow the resolved Catalog to overwrite
+     // the source Profile (compare raw and canonicalized forms, since the output
+     // may not exist yet).
+     if output_path == canonical_input
+         || std::fs::canonicalize(&output_path)
+             .ok()
+             .is_some_and(|canon_out| canon_out == canonical_input)
+     {
+         return Err(ForgeError::InvalidArgument(format!(
+             "Output path '{}' must differ from the input profile '{}'",
+             output_path.display(),
+             canonical_input.display()
+         )));
+     }


══════ F0384 │ src/config.rs:0-0 │ [security · high] ══════
[security · high] TOCTOU / unbounded-read gap (also Pre-scan focus #1): size, regular-file, and
symlink verdicts come from `fs::symlink_metadata` taken BEFORE `fs::read`, but the byte vector is
never re-checked. A file that passes the metadata gate can grow beyond MAX_CONFIG_SIZE (or be
swapped for a fifo/procfs-style pseudo-file whose len reads as 0) before the read, letting a hostile
environment commit the whole contents to memory (≤1 MiB assumption broken, unbounded allocation)
while diagnostics still claim the cap was enforced. Enforce the cap on the data actually parsed.


══════ F0448 │ src/export/xml_serializer.rs:294-297 │ [bug · high] ══════
[bug · high] `write_group` never serializes `group.groups` — every nested sub-group (and its entire
subtree of controls/parts) is silently dropped from exported catalogs. Nested <group> elements are
valid per the OSCAL Catalog XSD (GroupType permits group* after part* and before control*), so this
is irrecoverable data loss whenever the model contains a nested hierarchy. Because no test fixture
populates `groups: vec![...]` non-empty, this went undetected.

-     // Controls (position 7)
+     // Nested sub-groups — OSCAL XSD places group* after parts and before controls
+     for subgroup in &group.groups {
+         write_group(writer, subgroup)?;
+     }
+
+     // Controls
      for control in &group.controls {
          write_control(writer, control)?;
      }


══════ F0449 │ src/export/xml_serializer.rs:550-556 │ [bug · high] ══════
[bug · high] `serialize_catalog_to_xml` only iterates `catalog.groups` and never emits
`catalog.controls`, so top-level controls held by `OscalCatalog.controls` (the OSCAL XSD allows
control* on CatalogType alongside group*) vanish from every export. An OSCAL document consisting
solely of top-level controls exports as a catalog with zero controls. Add the top-level controls
loop after groups (matching CatalogType child order), and add a regression test where
`catalog.controls` is non-empty.

      for group in &catalog.groups {
          write_group(&mut writer, group)?;
+     }
+
+     // Top-level controls — CatalogType allows control* siblings after group*
+     for control in &catalog.controls {
+         write_control(&mut writer, control)?;
      }

      if let Some(bm) = &catalog.back_matter {
          write_back_matter(&mut writer, bm)?;
      }


══════ F0402 │ src/framework/disposition.rs:103-107 │ [bug · high] ══════
[bug · high] `uuid::Uuid::parse_str` accepts many spellings of the same UUID (uppercase/lowercase
hex, brace-delimited, urn:uuid prefix, and unhyphenated 32-digit forms). These pass as *distinct*
entries here because both the duplicate detection set and the stored `finding_id` operate on the raw
string, so `{…}`/uppercase variants of the same ID defeat dedup and won't lexically match canonical
IDs in the report downstream. Parse once, key the uniqueness set on the parsed `Uuid`, and persist
the canonical `to_string()` form (or model the field as `typed` `uuid::Uuid`, which serializes
canonically lower-case) — this requires collecting parsed values in the loop and canonicalizing the
records afterwards.

-         uuid::Uuid::parse_str(&disposition.finding_id)
+         // Compare identity on the parsed value, not on spelling.
+         let finding_id = uuid::Uuid::parse_str(&disposition.finding_id)
              .map_err(|_| error(format!("{path}.finding_id must be a UUID")))?;
-         if !finding_ids.insert(disposition.finding_id.as_str()) {
+         if !finding_ids.insert(finding_id) {
              return Err(error(format!("{path}.finding_id duplicates another disposition")));
          }
+         canonical_ids.push(finding_id);
+         // …then, in a follow-up pass over `&mut file.dispositions`:
+         // disposition.finding_id = finding_id.to_string();


══════ F0463 │ src/framework/model.rs:173-175 │ [bug · high] ══════
[bug · high] `FindingPriority` derives `PartialOrd`/`Ord`, so the derived order simply follows
variant declaration order: `Informational > ReviewRequired > Blocking`. That declares the LOWEST
severity as the maximum. This is confirmed consequential: `src/framework/analysis.rs:120-135` keys
its published-report sort on `(left.priority, ...)`, which only produces sensible output because an
ascending sort surfaces Blocking first. Any other consumer relying on the derived order gets silent
inverted semantics: `findings.iter().max_by_key(|f| f.priority)` returns an Informational finding as
'most severe', and range/binary-search partitions split backwards. Encode the severity precedence
explicitly (e.g. a documented `rank()` used as the sort key at the call site, or a manual `Ord` impl
preserving the current comparator output so report ordering does not change), and add a unit test
asserting the intended precedence so future reordering of variants cannot silently invert severity.

- #[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
- #[serde(rename_all = "kebab-case")]
- pub enum FindingPriority {
+ impl FindingPriority {
+     /// Explicit severity rank; documented so ordering never depends on
+     /// variant declaration order. Values chosen to keep the existing
+     /// ascending sort in `crate::framework::analysis` byte-for-byte stable.
+     #[must_use]
+     pub const fn rank(self) -> u8 {
+         self as u8 // Blocking = 0, ReviewRequired = 1, Informational = 2
+     }
+ }
+
+ // Then sort callers use `f.priority.rank()` as the key instead of the derived `Ord`.


══════ F0433 │ src/ingest/mod.rs:184-188 │ [security · high] ══════
[security · high] Security/DoS: max_size_bytes bounds the compressed archive, but
'word/document.xml' is fully decompressed into a String with no cap. A deliberately crafted
(zip-bomb) DOCX can be tiny on disk yet expand to gigabytes here, exhausting memory even though it
passed the size validation. Enforce a decompression budget, e.g. wrap the Read in
take(max_decompressed_bytes) sized relative to max_size_bytes, and fail with a Parse error when the
limit is hit instead of buffering indefinitely.

-     archive
+     let entry = archive
          .by_name("word/document.xml")
-         .map_err(|e| ForgeError::Parse(format!("DOCX missing word/document.xml: {e}")))?
-         .read_to_string(&mut document_xml)
+         .map_err(|e| ForgeError::Parse(format!("DOCX missing word/document.xml: {e}")))?;
+     // Bound decompressed size to a sane multiple of the compressed limit.
+     let cap = max_size_bytes.saturating_mul(64);
+     let mut document_bytes = Vec::new();
+     (&mut entry.take(cap))
+         .take(cap)
+         .read_to_end(&mut document_bytes)
          .map_err(|e| ForgeError::Parse(format!("failed to read DOCX document.xml: {e}")))?;
+     let document_xml = String::from_utf8(document_bytes)
+         .map_err(|_| ForgeError::Parse("DOCX document.xml is not valid UTF-8".to_string()))?;


══════ F0490 │ src/lifecycle/record.rs:410-414 │ [security · high] ══════
[security · high] The "assertions must be unique and sorted" invariant is enforced only for `/2`
records whose events carry no `legacy_event_id`, and the equivalent relaxation is baked into
`validate_separation` via `require_author_evidence` (`record.schema_version == SCHEMA_VERSION && ...
legacy_event_id.is_none()`). Consequences: (a) any `/1` record is accepted even if `assertions`
contains duplicate or conflicting (actor, role) attestations, and (b) on `/2`, an event that merely
carries a `legacy_event_id` opts itself (and every future approval in whose evidence window it sits,
see `validate_approval`) out of the author-evidence prerequisite and out of sortedness checks. Since
`event_id` deliberately bridges both schemas and `validate_approval` unions assertions from the
whole review window, an unmigrated/migrated event lets an approver satisfy `required_roles` counts
without ever producing declared-author evidence, defeating author/reviewer and author/approver
separation even when those rules are configured `true`. If trusting legacy/M-format evidence is an
intentional migration-time trust boundary, please document it at these two gates (and in the PRD)
and consider tightening it for `/2` records: enforce sortedness/uniqueness of `assertions`
unconditionally regardless of `legacy_event_id`, and/or require that once the record itself is `/2`
the author-evidence rule applies to windows containing legacy-carried events.

-         if record.schema_version == SCHEMA_VERSION
-             && event.legacy_event_id.is_none()
-             && event.assertions.windows(2).any(|items| items[0] >= items[1])
-         {
+ // Tightened: structural integrity of assertions does not depend on which ID scheme an event uses.
+ if event.assertions.windows(2).any(|items| items[0] >= items[1]) {
              return Err(error(format!("{path}.assertions must be unique and sorted")));
+ }
+ // Only the author-evidence prerequisite remains relaxed, for genuinely legacy approval windows.
+ let require_author_evidence =
+     record.schema_version == SCHEMA_VERSION && record.history[event_index].legacy_event_id.is_none();


══════ F0536 │ src/mapping/inventory.rs:165-167 │ [security · high] ══════
[security · high] Trust asymmetry: the Profile artifact itself is hash-pinned (expected_sha256
compared above), but the companion resolved_catalog — whose bytes actually produce the Profile's
entire inventory, fingerprints, excerpts and group metadata consumed by the build — is only hashed
and echoed as evidence; no expected-companion-hash is ever checked. A stale, regenerated or
maliciously substituted companion passes every gate whenever its id-set happens to match
manifest.inventory: changed control titles/prose (subject-sha256 props embedded in the published
Mapping), changed prose excerpts, changed group hierarchies and changed ineligible-part metadata all
go undetected and become unattested 'source of truth' evidence. The framework loader enforces
exactly this pin (framework/manifest.rs requires expected_resolved_catalog_sha256 for Profiles and
re-verifies it after load), so the mapping pipeline is the unguarded path. Add an
expected_resolved_catalog_sha256 field to ResourceManifest and enforce it here alongside
expected_sha256.

          let companion_path = manifest_dir.join(companion);
          let companion_bytes = read_bounded_json(&companion_path, path_label)?;
-         evidence.resolved_catalog_sha256 = Some(sha256(&companion_bytes));
+         let companion_sha256 = sha256(&companion_bytes);
+         if let Some(expected) = &resource.expected_resolved_catalog_sha256
+             && expected != &companion_sha256
+         {
+             return Err(mapping_error(format!(
+                 "{path_label}.expected_resolved_catalog_sha256 mismatch: expected {expected}, got {companion_sha256}"
+             )));
+         }
+         evidence.resolved_catalog_sha256 = Some(companion_sha256);


══════ F0476 │ src/mapping/manifest.rs:429-431 │ [security · high] ══════
[security · high] Security: manifest-supplied filesystem paths are only extension-checked here.
`artifact` (and `resolved_catalog`) accept absolute paths and parent-directory traversal such as
"/etc/cron.d/evil.json" or "../../evil.json", since the sole gate is `extension() == Some("json")`.
If any consumer feeds these straight to the filesystem, the reviewer-authored manifest becomes a
path-traversal primitive. Tighten the contract in this validator (reject `is_absolute()` and any
component besides `Normal`) or explicitly delegate to a sandboxed resolution layer with a documented
invariant.

+     // Refuse paths that are absolute or contain traversal segments.
+     fn ensure_local_relative(label: &str, candidate: &std::path::Path) -> Result<(), ForgeError> {
+         if candidate.is_absolute()
+             || candidate
+                 .components()
+                 .any(|component| !matches!(component, std::path::Component::Normal(_)))
+         {
+             return Err(mapping_error(format!(
+                 "{label} must be a relative path without '..', '.' or leading separators"
+             )));
+         }
+         Ok(())
+     }
+     ensure_local_relative(
+         &format!("{path}.artifact"),
+         resource.artifact.as_path(),
+     )?;
      if resource.artifact.extension().and_then(|value| value.to_str()) != Some("json") {
          return Err(mapping_error(format!("{path}.artifact must be a local .json file")));
      }


══════ F0518 │ src/migration/engine.rs:249-251 │ [bug · high] ══════
[bug · high] Skipping locator-coincident pairs with identical text strands valid 1:1 pairs
downstream. Repro: old={X("dup"@secA/L1), Y("other"@secB/L2)}, new={P("dup"@secB/L2),
Q("dup"@secA/L1)}. Pass 2 (unique text) skips T="dup" (1 old × 2 new); the locator pass consumes Y↔P
(differing text); at group_ambiguities the residual text group is now 1×1 ({X},{Q}), which
append_candidate_groups also drops — so X and Q, with byte-identical text AND identical locator, are
silently reported as Retired/Added instead of being grouped or matched. Whenever a text-twin on one
side is consumed by the locator pass, the surviving twin pool collapses to 1×1 and the identical
pair vanishes from every classification bucket. Fix: re-run match_unique_normalized_text over the
residual unmatched set (between match_unique_locators and group_ambiguities). Choosing a 1×1 text
match there is unambiguous by construction (all real competitors are either already matched or enter
the ambiguity stage), so this preserves the "never resolve arbitrarily" rule.

-         if old[old_index].normalized_text == new[new_index].normalized_text {
-             continue;
-         }
+ // In classify(), between match_unique_locators(..) and group_ambiguities(..):
+ match_unique_normalized_text(
+     &old.requirements,
+     &new.requirements,
+     &mut old_matched,
+     &mut new_matched,
+     &mut entries,
+ );


══════ F0513 │ src/migration/inventory.rs:0-0 │ [bug · unspecified] ══════
[bug] Section paths are joined with `/` and titles are never escaped, so a title such as "Access
Control / Audit" produces `


══════ F0530 │ src/migration/types.rs:58-59 │ [bug · high] ══════
[bug · high] `InventoryRequirement` derives `PartialEq`/`Eq` over ALL fields, but `normalized_text`
is both `#[serde(skip)]` (invisible in the report JSON) and `pub(crate)` (mutable outside type
controls). This breaks the natural invariant that values with identical serialized output compare
equal: two requirements built with the same stable_id/sha256/location but different hidden text will
serialize identically yet differ under `==` and `Eq`-based dedup. Everywhere the report is compared
byte-for-byte (diffs, snapshot tests, round-trip checks) `PartialEq` gives a contradicting answer.
Implement `PartialEq`/`Hash` manually over the serializable identity fields (stable_id,
normalized_text_sha256, location) so structural equality always matches what is written to disk.

+ #[derive(Debug, Clone)]
+ pub struct InventoryRequirement {
+     pub stable_id: String,
+     pub normalized_text_sha256: String,
+     pub location: RequirementLocation,
      #[serde(skip)]
      pub(crate) normalized_text: String,
+ }
+
+ impl PartialEq for InventoryRequirement {
+     fn eq(&self, other: &Self) -> bool {
+         // Compare only the externally visible identity fields so that
+         // `==` agrees with serialized report output.
+         self.stable_id == other.stable_id
+             && self.normalized_text_sha256 == other.normalized_text_sha256
+             && self.location == other.location
+     }
+ }
+ impl Eq for InventoryRequirement {}


══════ F0573 │ src/model/assemble.rs:113-113 │ [bug · high] ══════
[bug · high] All range math here assumes sibling source_lines are strictly ascending and
duplicate-free, but nothing verifies it — not even a debug_assert. For out-of-order siblings,
range_end < range_start makes the half-open filter `item.source_line >= range_start &&
item.source_line < range_end` never match, so those sections' list items vanish silently (violating
the SEC-5 no-silent-drop guarantee advertised elsewhere in this file). Duplicate source_lines
likewise produce inverted/degenerate child ranges via build_child_ranges/is_in_child_range,
misattributing or dropping items between parent and child. The parse layer happens to emit ascending
lines today so tests pass, but this function is the guarantor of the association and must be
defensive on its own terms.

+         // Hard contract for the range math below; fail loudly rather than
+         // silently misattributing/dropping list items (SEC-5).
+         debug_assert!(
+             section_nodes.windows(2).all(|w| w[0].source_line < w[1].source_line),
+             "sibling SectionNodes must have strictly ascending source_lines"
+         );
      let mut result = Vec::with_capacity(section_nodes.len());


══════ F0568 │ src/model/frontmatter.rs:54-55 │ [bug · high] ══════
[bug · high] Opener/closer handling is asymmetric: files written with CRLF endings are rejected
outright by strip_prefix("---\n"), while the closing-delimiter search below goes out of its way to
accept "\r\n" delimiters. Any document saved with Windows line endings therefore silently loses its
entire frontmatter (returns None), which contradicts the fault-tolerance intent of SEC-005 and the
implied \r\n support. Normalize both openers up front (and make the closer search consistently
CR-aware).

-     // Frontmatter must start with "---\n" at the beginning of the document
-     let rest = content.strip_prefix("---\n")?;
+     // Accept both LF and CRLF openers; the closer search below already tolerates CRLF.
+     let rest = content
+         .strip_prefix("---\r\n")
+         .or_else(|| content.strip_prefix("---\n"))?;


══════ F0561 │ src/oscal/assessment_plan.rs:329-333 │ [bug · high] ══════
[bug · high] The index-based fallback seed `req-{i}` can collide with a literal `stable_id`: a
requirement without a `stable_id` at index 3 gets the same seed as a different requirement whose
`stable_id` is exactly "req-3", yielding duplicate task AND activity UUIDs in one document.
`complete_assessment_plan` will then silently drop the second activity definition while two distinct
tasks reference the same activity UUID, breaking referential integrity of the emitted OSCAL. The
fallback also makes UUIDs order-dependent (same requirements reordered produce different UUIDs),
contradicting the "so UUIDs stay unique" comment. Use a namespaced fallback that cannot collide (or
return a typed error / derive from a content hash), and document the ordering caveat.

+             // Namespaced so a literal stable_id like "req-3" can never collide;
+             // note UUIDs remain sensitive to requirement ordering for inputs
+             // without stable_id.
              let id_seed = req
                  .stable_id
                  .as_deref()
                  .filter(|s| !s.is_empty())
-                 .map_or_else(|| format!("req-{i}"), str::to_owned);
+                 .map_or_else(|| format!("<unset-stable-id>{i}>"), str::to_owned);


══════ F0618 │ src/oscal/back_matter.rs:253-258 │ [bug · high] ══════
[bug · high] Distinct citations with identical text+URL hash to the same UUID v5, yet each is pushed
as a separate BackMatterResource — the output can contain multiple resources sharing one identifier
(this is even codified by the `two_identical_citations_produce_same_uuid` test). Downstream,
`href="#<uuid>"` links become ambiguous, violating OSCAL's expectation that back-matter resource
UUIDs uniquely identify a resource within the document. Track already-emitted UUIDs and reuse the
existing resource (inserting into the map without pushing a duplicate resource), or return an error.

-         let normalized = crate::uuid::normalize_for_hashing(&citation.text);
-         let hash_input = match &citation.url {
-             Some(url) => format!("{normalized}\n{url}"),
-             None => normalized.clone(),
-         };
          let uuid = Uuid::new_v5(&BACK_MATTER_NAMESPACE, hash_input.as_bytes());
+         if !seen_uuids.insert(uuid) {
+             // Identical content already produced a resource; reuse its UUID
+             // instead of emitting a second resource with the same identifier.
+             resource_map.insert(citation.id.clone(), uuid);
+             continue;
+         }


══════ F0620 │ src/oscal/back_matter.rs:260-264 │ [security · high] ══════
[security · high] Incomplete SEC-2 sanitization: for `javascript:`/`data:`/`vbscript:` citations the
href is stripped from `rlinks`, but when `text` is empty the raw URL payload (e.g.
`"javascript:alert(1)"`) is copied verbatim into `title`. Titles are descriptive display strings
that downstream renderers are free to print as-is, which reintroduces the sanitized hostile payload
into rendered output and re-opens the XSS vector SEC-2 was meant to close. Only fall back to the raw
URL for validated http(s)/benign values; otherwise emit a redacted placeholder.

-         let title = if citation.text.is_empty() {
+         let title = if !citation.text.is_empty() {
+             citation.text.clone()
+         } else if matches!(
+             classification,
+             UrlClassification::Valid(_) | UrlClassification::Malformed(_)
+         ) {
              citation.url.clone().unwrap_or_default()
          } else {
-             citation.text.clone()
+             "[unsafe URL scheme removed]".to_string()
          };


══════ F0609 │ src/oscal/catalog.rs:462-470 │ [bug · high] ══════
[bug · high] The 4-hex-char suffix is not checked against previously issued IDs, so collisions among
suffixes are possible and, worse, deterministic: any title appearing three or more times with the
same base slug receives the identical '{base}-{hash}' for occurrences 2..=n, producing duplicate
group IDs (and via resolve_abbreviation, duplicate control IDs like POL-AC-c5c6-001 twice), which
violates the SC-003 uniqueness invariant. The 16-bit space (65,536 values) is small under birthday
collisions for real-world corpora of similar titles. Re-check uniqueness after applying the suffix
and extend/iterate the hash until the resulting ID is unused.

+         // Ensure the suffixed ID itself does not collide: re-hash with a
+         // salt until we find an unused ID (handles >=3 same-title sections
+         // and 16-bit birthday collisions).
+         let mut salt = 0u64;
+         loop {
          let mut hasher = Sha256::new();
          hasher.update(title.as_bytes());
+             hasher.update(salt.to_le_bytes());
          let hash = hasher.finalize();
-         let suffix = format!("{:02x}{:02x}", hash[0], hash[1]);
-         format!("{base}-{suffix}")
+             let candidate = format!("{base}-{:02x}{:02x}", hash[0], hash[1]);
+             if !counts.values().flatten().any(|t| t == &candidate) && !issued.contains(&candidate) {
+                 // track `issued` alongside titles to make the check sound
+                 break candidate;
+             }
+             salt += 1;
+         }
      }
  }

  /// Resolve an abbreviation with content-based collision tracking.


══════ F0608 │ src/oscal/catalog.rs:511-513 │ [bug · high] ══════
[bug · high] collect_control_ids_from_catalog silently drops control IDs stored outside top-level
groups: it never traverses OscalCatalog.controls (root-level controls, supported per OSCAL v1.2.0
and proven by the catalog_round_trips_root_level_controls test in this file) nor OscalGroup.groups
(nested sub-groups, proven by catalog_round_trips_nested_groups). Any consumer such as
build_assessment_plan relying on this collector will omit root/nested controls from uniqueness
checks and assessments, producing incomplete plans or false-positive 'unique ID' results. Make the
walk recursive over groups and include catalog.controls.

  pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String> {
-     catalog.groups.iter().flat_map(|g| g.controls.iter()).map(|c| c.id.clone()).collect()
+     fn walk_groups(groups: &[OscalGroup], out: &mut Vec<String>) {
+         for g in groups {
+             out.extend(g.controls.iter().map(|c| c.id.clone()));
+             walk_groups(&g.groups, out);
+         }
+     }
+     let mut ids: Vec<String> =
+         catalog.controls.iter().map(|c| c.id.clone()).collect();
+     walk_groups(&catalog.groups, &mut ids);
+     ids
  }


══════ F0585 │ src/oscal/implemented_requirements.rs:210-213 │ [bug · high] ══════
[bug · high] Folding the positional `global_index` into the UUIDv5 seed defeats the purpose of
deterministic identifiers: inserting or deleting ANY requirement re-rolls the UUIDs of every
following requirement whose `stable_id`/text are unchanged, breaking traceability links and
producing noisy diffs between document versions. The index also isn't needed for uniqueness in the
normal case, because `stable_id` is content-derived; note the interplay with the `"no-stable-id"`
fallback seed above — blindly removing the index would make those collide. Prefer seeding from
`stable_id` + text and disambiguating only genuine duplicate `(stable_id, text)` pairs with a
per-pair occurrence counter (or the content-derived `atom_index` field already present on
`PolicyRequirement`), and update the T022/T-uuid-index tests accordingly.

- fn generate_impl_req_uuid(stable_id: &str, text: &str, index: usize) -> Uuid {
-     let seed = format!("{stable_id}\0{text}\0{index}");
+ /// Seed from content only; bump `occurrence` solely when the same
+ /// (stable_id, text) pair appears more than once, so inserting or
+ /// removing unrelated requirements never shifts existing UUIDs.
+ fn generate_impl_req_uuid(stable_id: &str, text: &str, occurrence: usize) -> Uuid {
+     let seed = format!("{stable_id}\0{text}\0{occurrence}");
      Uuid::new_v5(&IMPL_REQ_NAMESPACE, seed.as_bytes())
  }


══════ F0615 │ src/oscal/parts.rs:141-143 │ [medium · unspecified] ══════
[medium] An empty/whitespace-only requirement text only logs a `warn!` (EC-1/EC-2) and still pushes
a statement part whose `prose` is `""`. Callers who don't consume tracing output get an OSCAL part
with empty prose — content the OSCAL schema/model treats as malformed (prose is required to be
meaningful) — with no programmatic signal. Either propagate this condition (e.g., return `Result`
with a typed error like the rest of the pipeline's `ForgeError` paths), record a marker such as an
empty-prose flag/prop, or explicitly document and enforce that upstream guarantees non-empty text so
the runtime warning isn't the only defense.

      if requirement.text.trim().is_empty() {
          warn!(control_id, "Empty/whitespace requirement text — statement prose preserved as-is");
+         // Surface the anomaly to callers instead of relying solely on tracing,
+         // e.g. by returning `Result<Vec<OscalPart>, ForgeError>` and bubbling up,
+         // so artifact generation can fail loudly rather than emit empty prose.
      }


══════ F0614 │ src/oscal/parts.rs:156-158 │ [medium · unspecified] ══════
[medium] The em guards guidance with `!text.is_empty()` but never trims, while the requirement-text
check above uses `.trim().is_empty()`. A whitespace-only `Some("   ")` therefore slips through the
documented `Some(non_empty_text)` contract and still emits a guidance part whose entire prose is
whitespace — inconsistent behavior relative to how empty statement text is handled. Align the
checks, e.g. gate on `!text.trim().is_empty()` (and decide whether prose should be stored trimmed or
preserved), or update the doc comment to state that only truly-empty strings are filtered.

      if let Some(text) = guidance_text
-         && !text.is_empty()
+         && !text.trim().is_empty()
      {


══════ F0616 │ src/oscal/parts.rs:181-183 │ [low · unspecified] ══════
[low] `build_control_props` is a public API that accepts a full `PolicyRequirement` but
unconditionally returns an empty vector; its only purpose is stated as retaining "API compatibility
with existing callers" after trace-prop logic moved to post-processing. Dead-in-effect public stubs
invite new misuse: callers may assume props are populated here and double-add them post-embedding,
or silently drop real metadata expectations. If all internal call sites are migrated, prefer
removing it from the public re-export (see src/oscal/mod.rs) — or at minimum annotate it
`#[deprecated(since = "…", note = "trace props are added by embed_trace_in_catalog; this always
returns empty")]` so misuse is caught at compile time.

+ /// **Deprecated:** always returns an empty vec; trace props are added by
+ /// [`crate::oscal::trace_embedding::embed_trace_in_catalog`] post-processing.
+ #[deprecated(
+     since = "0.x",
+     note = "returns empty; use trace_embedding::embed_trace_in_catalog for trace props"
+ )]
  pub fn build_control_props(_requirement: &PolicyRequirement) -> Vec<OscalProp> {
      vec![]
  }


══════ F0617 │ src/oscal/parts.rs:31-32 │ [low · unspecified] ══════
[low] `name` is modeled as a free-form `String` although this type's own documentation restricts it
to `"statement"`, `"guidance"`, `"objective"`, or `"item"` — four well-known OSCAL part-name values.
With a plain `String`, any typo'd or arbitrary name round-trips into the serialized `parts` array
with no compile-time or runtime guard, producing quietly invalid OSCAL. Model it as a dedicated enum
(serialized to/from these strings) or at minimum validate in builders, reserving `String` for
genuinely free-form OSCAL extensions.

      /// OSCAL part name: `"statement"`, `"guidance"`, `"objective"`, or `"item"`.
-     pub name: String,
+     pub name: OscalPartName,


══════ F0613 │ src/oscal/parts.rs:88-90 │ [high · unspecified] ══════
[high] `generate_part_id` concatenates a raw `control_id` into the OSCAL part `id` with no character
validation. OSCAL `id` values follow a NCName-like pattern (letters/digits plus `-`, `_`, `.`), and
control IDs come from parsing arbitrary policy markdown, so characters like `&`, spaces, `/`, or `#`
flow straight into `"id": "POL-DP&P-001_smt"`. That yields schema-invalid catalogs and can break
downstream XML/id-reference consumers (e.g., cross-references between `<param>`/links and part ids).
The unit test `test_generate_part_id_special_chars` even cements `&` passing through unchanged.
Consider sanitizing/rejecting non-conforming characters here (or at call sites), and updating the
test accordingly.

- pub fn generate_part_id(control_id: &str, suffix: &str) -> String {
-     format!("{control_id}_{suffix}")
+ #[must_use]
+ pub fn sanitize_oscal_id(raw: &str) -> String {
+     let mut out: String = raw
+         .chars()
+         .map(|c| {
+             if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
+                 c
+             } else {
+                 '-'
+             }
+         })
+         .collect();
+     // OSCAL ids must begin with a letter.
+     if !out.starts_with(|c: char| c.is_ascii_alphabetic()) {
+         out.insert_str(0, "ctrl-");
+     }
+     out
  }
+
+ #[must_use]
+ pub fn generate_part_id(control_id: &str, suffix: &str) -> String {
+     format!(


══════ F0624 │ src/oscal/ssp.rs:315-315 │ [bug · high] ══════
[bug · high] Component UUIDs are minted solely from the component title, so two inputs with the same
title produce identical v5 UUIDs. In `build_ssp_skeleton`, `component_uuids` then contains
duplicates and `build_control_impl_reqs` emits multiple by-component entries referencing the same
(merged) component, silently corrupting the SSP rather than failing loudly. Either derive the seed
from a stronger tuple (title + description + ordinal) or reject duplicate titles instead of
synthesizing collisions.

-             let uuid = Uuid::new_v5(&COMPONENT_NAMESPACE, def.title.as_bytes()).to_string();
+ for def in definitions {
+     if seen_titles.insert(def.title.trim().to_lowercase()) == false {
+         return Err(ForgeError::SspBuild(format!(
+             "duplicate component title {:?}: title-derived UUIDv5 would collide",
+             def.title
+         )));
+     }
+ }


══════ F0631 │ src/oscal/trace_embedding.rs:108-112 │ [bug · high] ══════
[bug · high] The traversal misses two kinds of elements that the OSCAL model explicitly allows. Per
src/oscal/catalog.rs: OscalCatalog has a top-level `controls: Vec<OscalControl>` ('Root-level
controls (not inside any group). OSCAL v1.2.0 allows this.') and OscalGroup has a nested `groups:
Vec<OscalGroup>` ('Nested sub-groups'). This function only walks `catalog.groups[*].controls`, so
controls living at catalog root or inside nested sub-groups are emitted without any trace metadata,
contradicting the doc claim 'Walk catalog groups and controls' and making the completion log
(`annotated_controls`) understate coverage. Extract a shared `annotate_control` helper, run it over
`catalog.controls`, and recurse into `group.groups`.

  pub fn embed_trace_in_catalog(catalog: &mut OscalCatalog, trace_links: &TraceLinkCollection) {
-     let mut annotated_controls = 0usize;
-     let mut annotated_groups = 0usize;
+     // Root-level controls are valid OSCAL v1.2.0 and must be annotated too.
+     for control in &mut catalog.controls {
+         annotate_control(control, trace_links);
+     }

      for group in &mut catalog.groups {
+         annotate_group(group, trace_links);
+     }
+ }
+
+ /// Recursively annotates a group, its controls, and nested sub-groups.
+ fn annotate_group(group: &mut OscalGroup, trace_links: &TraceLinkCollection) {
+     let mut group_section_title: Option<String> = None;
+     for control in &mut group.controls {
+         if let Some(section) = annotate_control(control, trace_links) {
+             group_section_title.get_or_insert(section);
+         }
+     }
+     for sub in &mut group.groups {
+         annotate_group(sub, trace_links);
+     }
+     // ... push group PROP_SOURCE_SECTION as before ...
+ }


══════ F0638 │ src/oscal_cli/detector.rs:93-96 │ [bug · high] ══════
[bug · high] Candidate selection uses only `candidate.exists()`, deviating from shell PATH
semantics: (1) a *directory* named `oscal-cli` (common when someone unpacks a release archive)
matches and ends the search, so later valid binaries on PATH are never considered and the result
flips to `functional: false`; (2) there is no executability check, so a non-executable match also
terminates the loop prematurely; (3) `exists()` followed by `canonicalize()`/spawn is TOCTOU-prone.
Check metadata (`is_file()` and permission bits `mode & 0o111`) and `continue` the search when a
candidate is not usable.

              let candidate = dir_path.join("oscal-cli");
-             if candidate.exists() {
+             match std::fs::metadata(&candidate) {
+                 Ok(meta)
+                     if meta.is_file()
+                         && std::os::unix::fs::PermissionsExt::mode(&meta.permissions())
+                             & 0o111
+                             != 0 =>
+                 {
                  return candidate.canonicalize().ok().or(Some(candidate));
+                 }
+                 _ => continue, // not a regular executable file: keep searching PATH
              }


══════ F0643 │ src/oscal_cli/invoker.rs:86-91 │ [bug · high] ══════
[bug · high] The `try_wait` Err arm leaks resources: unlike the timeout arm, it neither kills/reaps
the child nor joins the stderr-drain thread. If `try_wait` fails after spawn (e.g., transient OS
error), oscal-cli keeps running unsupervised, an unreaped child (potential zombie) remains, and a
detached reader thread holding the pipe persists for the life of the process. Mirror the cleanup
done on the timeout path.

              Err(e) => {
+                 let _ = child.kill();
+                 let _ = child.wait(); // Reap the child
+                 let _ = stderr_thread.join(); // Prevent thread leak
                  return Err(ForgeError::OscalCliExecution {
                      exit_code: None,
                      message: format!("Failed to wait for oscal-cli {context}: {e}"),
                  });
              }


══════ F0663 │ src/parse/clauses.rs:449-454 │ [bug · high] ══════
[bug · high] Dispatch-order bug: when a GFM table is nested inside a list item (legal Markdown: an
indented table under "- Item\n\n    | A | B |") , the chain calls handle_item_text BEFORE
handle_table_event. While in_table is true the list handler consumes Start(Table) normally, but
subsequent cell Text/Code events match handle_item_text first (item_stack non-empty, exclude_depth
== 0) and return true, so `continue` skips the table handler entirely. Every cell's text is appended
into the list item's buffer (duplicated prose in list_items) and never reaches current_cell,
producing a table with empty/garbled headers and rows. Route events to handle_table_event first
whenever table_state.in_table is set.

-         if handle_item_text(&event, &mut list_state) {
-             continue;
-         }
-         if handle_table_event(&event, &range, &mut table_state, &mut tables, &line_starts) {
+         // Inside a GFM table the table handler must own every event
+         // (including cell Text/Code); otherwise cells nested in list items
+         // leak into the item text and disappear from the extracted table.
+         let handled = if table_state.in_table {
+             handle_table_event(&event, &range, &mut table_state, &mut tables, &line_starts)
+                 || handle_list_event(&event, &range, &mut list_state, &mut list_items, &line_starts)
+                 || handle_item_text(&event, &mut list_state)
+         } else {
+             handle_list_event(&event, &range, &mut list_state, &mut list_items, &line_starts)
+                 || handle_item_text(&event, &mut list_state)
+                 || handle_table_event(&event, &range, &mut table_state, &mut tables, &line_starts)
+         };
+         if handled {
              continue;
          }


══════ F0692 │ src/round_trip/comparator.rs:138-148 │ [bug · high] ══════
[bug · high] The soft-line-break whitelist can misclassify meaning-altering whitespace collapses as
Acceptable. Lines consisting solely of '=' or '-' (setext heading underlines / thematic breaks) are
not caught: the list Only checks '- ' with a trailing space, so '-----' slips through, and
pipe-table rows ('| a |', '| --- |') are not checked at all. Joining such lines turns a heading or
table into plain paragraph text, so e.g. expected "Overview\n-----" vs actual "Overview -----" is
reported as DivergenceClass::Acceptable even though the rendered Markdown differs fundamentally.
Refuse normalization when a continuation line starts with '=' or '|'.

- fn normalize_soft_line_breaks(value: &str) -> Option<String> {
-     if !value.contains('\n')
-         || value.contains('\r')
-         || value.contains('\t')
-         || value.contains("<pre")
-         || value.contains("</pre")
-         || value.contains('`')
-         || value.contains("~~~")
+ for (index, line) in lines.iter().enumerate() {
+     if line.ends_with("  ") || line.ends_with('\\') {
+         return None;
+     }
+     if index > 0
+         && (is_markdown_block_start(line)
+             || line.trim_start().starts_with(['=', '|']))
      {
          return None;
+     }
      }


══════ F0766 │ src/trace/walker.rs:122-127 │ [security · high] ══════
[security · high] `walk_group` and `walk_control` recurse on untrusted `serde_json::Value` input
with no depth limit. A crafted/corrupt deeply nested document (e.g., thousands of nested
`groups`/`controls` arrays) will overflow the call stack and abort the process — a denial-of-service
vector for a parser-facing tool. Thread a `depth: usize` parameter (and a max-depth error, or stop
recursing with a warning) so the recursion is bounded.

      // OSCAL allows nested controls (e.g. control enhancements)
      if let Some(children) = control.get("controls").and_then(|c| c.as_array()) {
          for child in children {
-             walk_control(child, entries);
+             walk_control_at_depth(child, entries, depth + 1);
          }
      }


══════ F0774 │ src/uuid.rs:256-259 │ [bug · high] ══════
[bug · high] The v5 seed couples the identifier to volatile layout data: section_path (titles),
source_line, and atom_index. Any cosmetic edit — a paragraph reflow that shifts lines,
renaming/moving a section, or a re-parse that changes atom order — regenerates the stable_id for
requirement text that did NOT change. This contradicts the module's "Determinism Guarantee" ('Same
text produces the same UUID, always', as documented on generate_stable_id) and defeats the intended
Substantive Change Detection downstream: one inserted line early in the document rewrites IDs for
every following requirement, producing mass false positives in diff/change tracking. Prefer deriving
the ID from content-stable fields only and disambiguating exact duplicates with a deterministic
content-based ordinal (e.g., nth occurrence of this normalized text), or, at minimum, document
loudly that these IDs are position-sensitive despite being called 'stable'.

-         let hash_input = format!(
-             "{normalized}\0{section_path}\0{}\0{}",
-             requirement.source_line, requirement.atom_index
-         );
+         // Seed only with content-stable fields. Resolve exact duplicates
+         // deterministically (ordinal = index of this occurrence of the same
+         // normalized text among prior requirements), so unrelated edits or
+         // line shifts never rewrite unchanged identifiers.
+         let hash_input = format!("{normalized}\0{occurrence}");


══════ F0790 │ src/validate/formatter.rs:112-118 │ [maintainability · high] ══════
[maintainability · high] The entire classification pipeline dispatches on exact English substrings
of the `jsonschema` crate's human-readable messages. Any crate upgrade that rewords even one phrase
silently drops every affected error into the generic `schema validation failed` fallback, losing
field names and constraints with no compile-time signal. The crate exposes structured context via
`raw_error.kind()` (`ValidationErrorKind::Required { property }`, `Type { got, expected_types }`,
`AdditionalProperties { unexpected }`, `Constant`, `Enum`, `MaxLength`, `Minimum`, `Pattern`,
`Format`, ...) which carries the extracted data type-safely and eliminates the need for the
hand-written message-parsing helpers below (`extract_quoted_value`, `extract_trailing_quoted`,
`extract_length_constraint`, ...). Thread the `&jsonschema::ValidationError` (or its kind) into
`classify_error` and match on the enum instead of the `Display` string.

-     None.or_else(|| classify_required_property(raw_message))
-         .or_else(|| classify_type_mismatch(raw_message))
-         .or_else(|| classify_schema_mismatch(raw_message))
-         .or_else(|| classify_length_constraint(raw_message))
-         .or_else(|| classify_pattern_or_format(raw_message))
-         .or_else(|| classify_additional_properties(raw_message))
-         .or_else(|| classify_enum_constraint(raw_message))
+ use jsonschema::error::ValidationErrorKind;
+
+ fn classify_error(err: &jsonschema::ValidationError) -> (String, String) {
+     match err.kind() {
+         ValidationErrorKind::Required { .. } => {
+             ("required field missing".to_string(), "required field".to_string())
+         }
+         ValidationErrorKind::Type { got, .. } => (
+             format!("wrong type: got {got}"),
+             "valid type per schema".to_string(),
+         ),
+         // ... remaining structured variants ...
+         _ => (
+             "schema validation failed".to_string(),
+             "valid value per schema".to_string(),
+         ),
+     }
+ }


══════ F0791 │ src/validate/formatter.rs:165-171 │ [bug · high] ══════
[bug · high] These predicates require the literal tokens `pattern`/`format`, which do not appear in
the jsonschema crate's shipped messages: pattern violations print like `"banana" does not match
"^[apples]+C"
