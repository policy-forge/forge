# Validation slice slice02 — 60 findings
Severity mix: high×9, medium×51


══════ F0785 │ src/validate/semantic.rs:25-25 │ [bug · high] ══════
[bug · high] Same defect from the caller side: `check_orphaned_links(json)` discards `model_type`,
so the resource collection cannot pick the right root key for the artifact being validated. Thread
`model_type` through so resources are read from the model's actual root, and consider skipping
models without a recognized root instead of silently treating every '#...' href as unresolved.

-         let mut errors = check_orphaned_links(json);
+         let mut errors = check_orphaned_links(json, model_type);


══════ F0784 │ src/validate/semantic.rs:35-37 │ [bug · high] ══════
[bug · high] The root-key list is incomplete relative to the OSCAL models this validator claims to
handle: `validate()` runs the orphaned-link walk for *every* artifact, but back-matter resources are
only collected under 'catalog', 'component-definition', and 'mapping-collection'. A well-formed
Profile ('profile' root) — or SSP / assessment-plan / assessment-results — whose links legitimately
resolve to its own back-matter resources will have every local link falsely reported as orphaned.
Derive the root key(s) from OscalModelType (or enumerate all supported model roots) instead of
hardcoding three.

-     // Try known OSCAL root keys
-     let root_keys = ["catalog", "component-definition", "mapping-collection"];
-     for key in &root_keys {
+     // Resolve the model-specific root key so back-matter is found for every supported model
+     let root_keys = model_type.supported_root_keys();
+     for key in root_keys {


══════ F1017 │ supply-chain/audits.toml:4-4 │ [security · high] ══════
[security · high] The [audits] table is completely empty. Combined with supply-chain/config.toml
(which carries ~100+ crates under [[exemptions.*]] with 'safe-to-deploy'/'safe-to-run', and declares
no [imports] from any shared registry), every single dependency in the graph is trusted via blanket
local exemptions with zero recorded human review. Under this setup `cargo vet --locked` passes
vacuously: the CI supply-chain gate emits no real review signal, and each newly added crate simply
accumulates another silent exemption instead of being vetted. This defeats the project's stated
control ('All new crates must be registered... convert exemptions to imported audits as they become
available upstream', ADR 051 / SEC 051). Recommend importing a curated audit registry (e.g., the
cargo-vet community registry) via [imports] in config.toml and starting to populate real
[[audits.<crate>]] entries here — or at minimum documenting why a deliberately zero-audit posture is
acceptable.

  [audits]
+
+ # Example entry once reviews begin:
+ # [[audits.anyhow]]
+ # who = "<reviewer identity>"
+ # criteria = "safe-to-deploy"
+ # version = "1.0.104"
+ # notes = "Reviewed source in accordance with safe-to-deploy criteria"


══════ F1049 │ supply-chain/config.toml:439-441 │ [security · high] ══════
[security · high] Blanket 'safe-to-deploy' exemptions on hostile-input parsers: lopdf/pdf-extract
(PDF object streams), postscript + cff-parser + type1-encoding-parser (font programs), quick-xml
(XML), zip 8.6.0 (archive extraction — zip-slip/decompression-bomb class),
wasmparser/wit-component/wit-parser (WebAssembly binaries), unsafe-libyaml (its very name flags
unsafe pointer-heavy YAML parsing), nom and fancy-regex. These accept attacker-controlled bytes at
runtime, so an exemption attests nothing about hardening against malformed input — it only covers
author-side review. Treat these rows as the highest-priority audit backlog: record genuine imported
audits for them when available ('cargo vet import' from Firefox/Embark-Studios repos covers many),
and consider pinning `fail()`-on-error / resource-limit configurations in application code
regardless of vetting status.


══════ F0815 │ tests/cli_integration.rs:444-447 │ [test · high] ══════
[test · high] Silent skip makes the test vacuous: if the XML fixture is ever deleted, renamed, or
not committed, this test returns early and CI reports PASS, permanently disabling all coverage of
the `--format xml` code path with zero signal. Every other fixture-dependent test in this file
hard-fails when its fixture is missing; this one is uniquely self-disabling. Replace the early
return with a hard assertion so a missing fixture is a loud setup failure.

      let fixture = std::path::Path::new("tests/fixtures/sample_policy.md");
-     if !fixture.exists() {
-         return;
-     }
+     assert!(
+         fixture.exists(),
+         "Required fixture '{}' is missing — XML format coverage is disabled",
+         fixture.display()
+     );


══════ F0832 │ tests/common/mod.rs:77-84 │ [test · high] ══════
[test · high] This helper converts a missing fixture into a quiet green test via an opt-in
boolean-return convention: every caller must remember to `return` when it yields true, and nothing
forces them to. Across the ~40 call sites in tests/*_test.rs a regression in
tests/common/fixture_generator.rs (or a renamed fixture directory) starves entire suites of coverage
with zero failure signal — only stderr noise that CI logs typically drop. Make skipping loud:
assert!(!path.exists(), "fixture missing: {}", path.display()); at generation setup, or return
Result<(), String>/Option<()> so '?' propagates, or provide a require_fixture! macro that bakes in
the return.

- pub fn skip_if_missing(path: &Path) -> bool {
-     if path.exists() {
-         false
-     } else {
-         eprintln!("Skipping test: fixture not found at {}", path.display());
-         true
-     }
+ /// Panics when a generated fixture is absent, so a broken fixture
+ /// generator fails loudly instead of silently emptying test coverage.
+ #[track_caller]
+ pub fn require_fixture(path: &Path) {
+     assert!(
+         path.exists(),
+         "required fixture missing (run fixture_generator?): {}",
+         path.display()
+     );
  }


══════ F0836 │ tests/export_integration.rs:120-120 │ [test · high] ══════
[test · high] Mode 0o444 does not prevent writes when the test process runs as root, which is the
default in most privileged Docker/CI containers; here run_export would then succeed and the
assert_ne!(exit_code, 0) below fails spuriously. Guard the environment: skip early if a probe write
into readonly_dir succeeds (covers root and CAP_DAC_OVERRIDE situations), so the test only asserts
behavior it can actually induce.

      std::fs::set_permissions(&readonly_dir, std::fs::Permissions::from_mode(0o444)).unwrap();
+
+     // Permission bits do not restrict root (common in Docker CI runners);
+     // bail out instead of failing spuriously if the sandbox lets us write.
+     let probe = readonly_dir.join(".probe");
+     if std::fs::write(&probe, b"x").is_ok() {
+         let _ = std::fs::remove_file(&probe);
+         let _ = std::fs::set_permissions(&readonly_dir, Permissions::from_mode(0o755));
+         eprintln!("skipping: cannot enforce read-only dir for this user");
+         return;
+     }


══════ F0844 │ tests/integration_cross_feature.rs:299-301 │ [test · high] ══════
[test · high] This test's stated purpose is verifying that EACH atomized half of the compound bullet
4 carries its own modality, but the assertions only check global presence of 'some normative' and
'some advisory'. Since bullet 1 ('must enforce multi-factor authentication') already yields
normative and bullet 3 ('should review access logs') already yields advisory, the test passes
unchanged if the atomizer assigns both halves the WRONG modality, drops props on the new controls,
or stops attributing them at all — only the raw `>= 5` count would catch full non-splitting, and
that proxy itself is unreliable (see count_controls). Assert attribution directly: locate the two
controls produced by bullet 4 (e.g., by their requirement/statement text 'enforce MFA' and 'should
notify', ideally sharing the same original_text/parent link) and assert their respective modality
props.

      // Each atomized control must carry its own modality prop.
-     // After the split, at least one control must be normative and one advisory.
-     let modalities = collect_modality_props(&catalog);
+     // Assert per-control attribution for the bullet-4 split, not mere global presence.
+     let mfa_ctrl = find_control_by_text(&catalog, "must enforce MFA")
+         .expect("atomized normative half of compound bullet 4 missing");
+     let notify_ctrl = find_control_by_text(&catalog, "should notify administrators")
+         .expect("atomized advisory half of compound bullet 4 missing");
+     assert_eq!(modality_of(&mfa_ctrl), Some("normative"));
+     assert_eq!(modality_of(&notify_ctrl), Some("advisory"));


══════ F0874 │ tests/oscal_cli_round_trip.rs:66-70 │ [bug · high] ══════
[bug · high] Contract contradiction with validate_divergence_log (T019): reclassify only assigns
Accepted to Acceptable-class divergences, while validate_divergence_log asserts that every
ForgeFix/OscalCliDiff divergence has a non-null resolution. However, compare_oscal_json produces all
divergences with resolution: None (src/round_trip/comparator.rs), so the required invariant can
never be satisfied by data produced in this file. Consequence: when the suite actually detects a
ForgeFix regression — precisely what SC-001/SC-002 exist to catch — the run dies inside
validate_divergence_log with the misleading 'must have a non-null resolution' message instead of
surfacing the classified divergence, and the OscalCliDiff reporting path (ReportedUpstream) can
never be exercised end-to-end. Align the two sides: either fill resolutions for
ForgeFix/OscalCliDiff here (e.g., Fixed / ReportedUpstream per investigation outcome), or drop the
non-null-resolution assertion for those classes in validate_divergence_log.

-             if d.classification == DivergenceClass::Acceptable && d.resolution.is_none() {
+             match d.classification {
+                 DivergenceClass::Acceptable if d.resolution.is_none() => {
                  Divergence { resolution: Some(ResolutionStatus::Accepted), ..d }
-             } else {
-                 d
+                 }
+                 // Keep the reclassify step consistent with the resolution
+                 // contract enforced by validate_divergence_log (T019).
+                 DivergenceClass::ForgeFix if d.resolution.is_none() => {
+                     Divergence { resolution: Some(ResolutionStatus::Fixed), ..d }
+                 }
+                 DivergenceClass::OscalCliDiff if d.resolution.is_none() => {
+                     Divergence { resolution: Some(ResolutionStatus::ReportedUpstream), ..d }
+                 }
+                 _ => d,
              }


══════ F0947 │ .cargo/config.toml:1-2 │ [performance · medium] ══════
[performance · medium] Raising net.retry from Cargo's default of 3 to 10 makes every
index/download/git operation attempt ~11 requests with backoff before failing. When the cause is not
transient (registry outage, expired token, misconfigured mirror URL), each build stalls for minutes
instead of failing fast with a clear error — which hurts time-bounded CI pipelines and hides
systemic problems (e.g., authentication failures get retried uselessly all 10 times). Consider
keeping retries modest (near the default) here, and letting individual developers or specific CI
jobs opt into higher retry counts via the CARGO_NET_RETRY environment variable where their network
genuinely needs it.

  [net]
- retry = 10
+ # Fail relatively fast on non-transient network errors (registry/auth
+ # outages); CI jobs with known-flaky connectivity can raise this via
+ # CARGO_NET_RETRY instead.
+ retry = 3


══════ F1073 │ .gitattributes:16-21 │ [maintainability · medium] ══════
[maintainability · medium] `whitespace` is not an attribute that Git honors for these paths:
built-in line-ending behavior is governed solely by `text`/`eol`/`binary`, and whitespace checking
is controlled by `core.whitespace` / `--check`, not by a `whitespace` attribute. Repository-wide
search (.github/, scripts/, CI configs, toml/rs files) finds no custom tooling, hook, or linter in
this repo that consumes it either, so `-whitespace` here is a silent no-op that creates a false
impression of extra enforcement — the byte-pinning guarantee rests entirely on `-text`, and `-text`
alone fully disables eol conversion anyway. Either remove the pseudo-attribute and rely on the real
mechanism, or add a comment naming the external tool that consumes it. Additionally, note that these
exclusion rules MUST keep appearing after `*.json text eol=lf` (last-matching-line wins); their
correctness is invisible to casual editors, so that coupling deserves an explicit warning here.

  # Official NIST OSCAL release assets are byte-pinned by the provenance manifest.
- # Do not normalize or whitespace-check their upstream formatting.
- schemas/oscal_catalog_schema.json -text -whitespace
- schemas/oscal_component_schema.json -text -whitespace
- schemas/oscal_profile_schema.json -text -whitespace
- tests/fixtures/schemas/* -text -whitespace
+ # NOTE: keeps AFTER '*.json text eol=lf' above — gitattributes is last-match-wins,
+ # so moving the block above that line would re-enable normalization and corrupt
+ # the pinned assets.
+ # '-whitespace' has no effect for git (nor is it consumed by any repo tooling);
+ # '-text' alone guarantees no eol conversion.
+ tests/fixtures/schemas/* -text
+ tests/fixtures/xsd/*.xsd -text
+ schemas/oscal_*.json -text


══════ F1072 │ .gitattributes:21-25 │ [bug · medium] ══════
[bug · medium] Coverage gap risks silent corruption of byte-pinned assets. Git treats patterns
containing '/' as pathname globs, so `tests/fixtures/schemas/*` matches ONLY immediate children of
that directory — any schema later nested in a subfolder, or a new manifest-pinned asset placed
outside these directories, falls through to `* text=auto` and gets eol-normalized again (the later
exclusion never sees it), silently breaking byte-for-byte provenance checks and `git diff
--no-index` comparisons (see docs/OSCAL_COMPATIBILITY.md). Today this happens to work because
schemas/oscal-schema-manifest.json pins exactly 2 flat JSON fixtures and 4 flat XSDs, which are all
directly covered — but the guard relies on incidental flat layout rather than on pattern intent.
Suggest widening coverage with recursive/directory forms and keeping it aligned with the manifest.

- tests/fixtures/schemas/* -text -whitespace
- tests/fixtures/xsd/oscal_catalog_schema.xsd -text -whitespace
- tests/fixtures/xsd/oscal_component_schema.xsd -text -whitespace
- tests/fixtures/xsd/oscal_profile_schema.xsd -text -whitespace
- tests/fixtures/xsd/oscal_complete_schema.xsd -text -whitespace
+ # Pin entire provenance-governed trees so newly added/renamed pinned files
+ # cannot silently fall back to '* text=auto' normalization.
+ tests/fixtures/schemas/** -text
+ tests/fixtures/xsd/*.xsd -text


══════ F1052 │ .github/dependabot.yml:7-11 │ [maintainability · medium] ══════
[maintainability · medium] The repository has GitHub Actions workflows (.github/workflows/ci.yml and
.github/workflows/release.yml), but this Dependabot configuration only enables updates for the
'cargo' ecosystem. Pinned action versions (e.g., actions/checkout@v4) used by these workflows will
never receive automated dependency-update PRs, leaving Actions versions to drift until they
eventually hit deprecated-node deprecation warnings or break. Add a second updates entry for the
'github-actions' ecosystem:

  updates:
    - package-ecosystem: "cargo" # See documentation for possible values
      directory: "/" # Location of package manifests
+     schedule:
+       interval: "weekly"
+   - package-ecosystem: "github-actions"
+     directory: "/" # Covers .github/workflows
      schedule:
        interval: "weekly"


══════ F1063 │ .github/workflows/ci.yml:26-27 │ [security · medium] ══════
[security · medium] Hard-coded commit SHAs with hand-written version comments cannot be verified
from this file alone — a hijacked or typo'd SHA would look exactly like this and precisely defeat
the supply-chain checks this workflow itself runs. Confirm each SHA matches its annotated tag
upstream (e.g., `git ls-remote https://github.com/dtolnay/rust-toolchain 631a55b...`). Special care
with `dtolnay/rust-toolchain`: a frozen master-branch SHA may stop working when GitHub runner images
rotate their bundled Rust/cargo versions, causing sudden CI breakage. Consider automating pin
integrity (e.g., a `dependency-review`/actionlint job or Dependabot, which updates both tag comment
and SHA together).


══════ F1065 │ .github/workflows/ci.yml:62-64 │ [performance · medium] ══════
[performance · medium] `cargo install` compiles cargo-audit, cargo-deny, and cargo-vet (and their
large dependency trees) from source on every Ubuntu run — typically 5–15+ minutes combined — even
though a cache exists. Worse, this job's cache key is `${{ runner.os }}-cargo-${{
hashFiles('**/Cargo.lock') }}`: bumping *any* project dependency invalidates the cache and wipes
`~/.cargo/bin`, forcing a full recompile of all three audit tools simultaneously. Prefer prebuilt
binaries (e.g., `taiki-e/install-action`, SHA-pinned), or split audit tooling into its own job
cached by tool version instead of `Cargo.lock`.

-       - name: Install cargo-audit
+       - name: Install audit tooling (prebuilt)
          if: matrix.os == 'ubuntu-latest'
-         run: cargo install cargo-audit --locked
+         uses: taiki-e/install-action@<full-commit-sha>
+         with:
+           tool: cargo-audit,cargo-deny,cargo-vet


══════ F1057 │ .github/workflows/release.yml:14-17 │ [other · medium] ══════
[other · medium] No job in either workflow sets `timeout-minutes`: all five jobs in
.github/workflows/release.yml and the matrix test job in .github/workflows/ci.yml can hang
indefinitely on a stalled cargo build, test suite, benchmark, or `cargo install` compilation,
burning runner time until GitHub's global 6-hour limit — multiplied across the 3-OS matrices.
Declare an explicit bounded `timeout-minutes` per job in both workflows.

  jobs:
    test:
      name: Test (${{ matrix.os }})
-     runs-on: ${{ matrix.os }}
+     timeout-minutes: 60
+     runs-on: ${{ matrix.os }}, plus e.g. 45-60 for build, 30 for sbom/hash/release


══════ F1055 │ .github/workflows/release.yml:164-168 │ [security · medium] ══════
[security · medium] `contents: write` is excessive here: with `upload-assets: false` the SLSA
generator only publishes the attestation as a workflow artifact and never writes to the
repository/releases, so it only needs `actions: read` and `id-token: write`. Granting `contents:
write` widens the blast radius if the referenced workflow (currently only tag-pinned) is ever
compromised.

      permissions:
        actions: read
        id-token: write
-       contents: write
      # SLSA L3 provenance — tamper-proof build attestation.


══════ F1058 │ .github/workflows/release.yml:3-6 │ [other · medium] ══════
[other · medium] Neither workflow defines concurrency control. In .github/workflows/release.yml,
pushing several v* tags (or re-pushing a tag) fans out full 9-runner pipelines in parallel with no
grouping, wasting compute and risking duplicate/out-of-order releases. In .github/workflows/ci.yml,
push-to-main and pull_request sync events overlap: each pushed commit to an open PR triggers a fresh
run alongside the previous run, and merging re-runs everything against main — amplified by three
source-compiled `cargo install`s per Ubuntu leg. Add a `concurrency` block to both workflows with a
group keyed per ref (per tag ref for releases; per branch/PR ref for CI) and `cancel-in-progress:
true` so superseded runs are cancelled.

  on:
    push:
      tags:
        - "v*"
+
+ concurrency:
+   group: ${{ github.workflow }}-${{ github.ref }}
+   cancel-in-progress: true


══════ F1059 │ .github/workflows/release.yml:82-85 │ [performance · medium] ══════
[performance · medium] Unlike the test job, the build job has no cargo caching at all, so every tag
rebuilds all dependencies from scratch across 4 targets/3 OSes. Add actions/cache keyed on
Cargo.lock and the target triple (path-style approach already proven above) to cut build time
substantially.

+       - name: Cache cargo registry and build
+         uses: actions/cache@668228422ae6a00e4ad889ee87cd7109ec5666a7  # v5
+         with:
+           path: |
+             ~/.cargo/registry
+             ~/.cargo/git
+             target
+           key: ${{ runner.os }}-${{ matrix.target }}-cargo-${{ hashFiles('**/Cargo.lock') }}
+           restore-keys: ${{ runner.os }}-${{ matrix.target }}-cargo-
+
        - name: Build release binary
          shell: bash
          env:
            BUILD_TARGET: ${{ matrix.target }}


══════ F0995 │ .rustfmt.toml:1-1 │ [maintainability · medium] ══════
[maintainability · medium] The "edition" option is only recognized by rustfmt >= 1.8.0 (shipped with
Rust 1.85). The repo has no rust-toolchain(.toml) pinning the toolchain, so environments (dev
machines, CI, editors) running an older stable toolchain will make every rustfmt invocation
hard-fail with an invalid-option error instead of formatting. Ensure the pinned/installed toolchain
supports Rust 2024, ideally by adding a rust-toolchain.toml, to keep `cargo fmt` consistent across
environments.

+ # Requires rustfmt >= 1.8.0 (Rust 1.85+); keep in sync with Cargo.toml edition.
+ # Consider adding rust-toolchain.toml so all contributors run a compatible toolchain.
  edition = "2024"


══════ F1020 │ Cargo.toml:27-27 │ [bug · medium] ══════
[bug · medium] Disabling default features leaves only `deflate-flate2` compiled in, so `ZipArchive`
(src/ingest/mod.rs) will reject otherwise-valid archives that use bzip2/zstd/xz compression, AES
encryption, or ZipCrypto passwords with "unsupported compression method" at runtime. Since ingest
feeds on externally supplied `.forgepack` uploads, any archive produced with a different method (or
by tools defaulting to another codec) will be rejected during ingestion even though the payload
itself is harmless. Confirm this reduction is intentional; if broader codec acceptance is desired,
add the relevant methods/features or keep the narrow set but surface a clear, actionable ingest
error instead of a raw zip failure.

+ # Intentionally minimal: only stored/deflate entries are supported.
+ # Extend when needed:
+ # zip = { version = "8.6.0", default-features = false, features = ["deflate-flate2", "bzip2"] }
  zip = { version = "8.6.0", default-features = false, features = ["deflate-flate2"] }


══════ F1024 │ Cargo.toml:37-38 │ [security · medium] ══════
[security · medium] This crate ingests untrusted PDF/XML/zip/Markdown payloads (pdf-extract,
quick-xml, zip are history-rich attack surfaces), yet `unsafe_code` stays at `warn`, so new `unsafe`
blocks sail through a normal build with a warning and can easily land in CI that doesn't run with
`-D warnings`. Harden the lint gate so at least this crate's own unsafe usage is structurally
forbidden rather than merely flagged.

  [lints.rust]
- unsafe_code = "warn"
+ unsafe_code = "forbid"


══════ F0041 │ benches/atomize.rs:10-23 │ [maintainability · medium] ══════
[maintainability · medium] This hand-copied fixture duplicates tests/common/mod.rs::make_req
(verified byte-for-byte today). PolicyRequirement already grew several optional fields (citations,
modality, parameters), so the duplication will keep drifting: the next added field either breaks
compilation of every fixture construction site, or — worse — gets a different default here and the
benchmarks silently measure a shape of input the real/test pipelines never produce. Since Cargo does
not compile submodule files under benches/ as separate targets, the helper can be shared verbatim
via a non-target module instead of being mirrored.

- // NOTE: mirrors tests/common/mod.rs — kept local since benches cannot import test modules
- fn make_req(text: &str, source_line: usize) -> PolicyRequirement {
-     PolicyRequirement {
-         stable_id: None,
-         text: text.to_string(),
-         source_line,
-         nesting_depth: 0,
-         atom_index: 0,
-         parent_text: None,
-         citations: vec![],
-         modality: None,
-         parameters: vec![],
-     }
- }
+ // benches/common/mod.rs (not a bench target, so safe to reference from every bench)
+ mod common;
+ use common::{make_doc, make_section, make_req};
+
+ // Then delete the local copy of make_req and reuse the single definition
+ // shared with tests/common/mod.rs.


══════ F0042 │ benches/atomize.rs:40-47 │ [performance · medium] ══════
[performance · medium] The "mixed" 100-requirement workload contains only two distinct requirement
texts repeated 50 times each, all with identical zero-valued metadata (nesting_depth 0, empty
citations/parameters) in one flat section with no children, body_text, or content_hash. Repetition
maximizes allocator/regex/engine warm-cache effects on identical input slices, and the flat
structure means hierarchy traversal, dedup-style behavior over heterogeneous content, and
metadata-dependent branches are never exercised — results will not transfer to realistic policies.
Vary text, nesting depth, and section structure in the generated fixture.

      let mut requirements = Vec::with_capacity(100);
-     for i in 0..50 {
-         requirements
-             .push(make_req("Systems must enforce MFA and must require complex passwords", i + 1));
-     }
-     for i in 50..100 {
-         requirements.push(make_req("All systems must enforce MFA", i + 1));
+     for i in 0..100 {
+         let text = match i % 3 {
+             0 => format!("System-{i} must enforce MFA"),
+             1 => format!("Component {i} shall log access events and shall alert admins"),
+             _ => format!("Operators {i} should rotate keys quarterly"),
+         };
+         let mut req = make_req(&text, i + 1);
+         req.nesting_depth = (i % 3) as u8;
+         requirements.push(req);
      }


══════ F0031 │ benches/export_bench.rs:52-63 │ [bug · medium] ══════
[bug · medium] Hand-rebuilding `OscalCatalog` here hardcodes `controls: vec![]` and only forwards
`groups`, bypassing whatever `build_catalog`, `embed_trace_in_catalog`, and the real CLI pipeline
produce. Today root-level controls happen to be empty, but if `build_catalog` ever emits them (or
the envelope assembly logic in `pipeline.rs` changes), this benchmark silently benchmarks a
different artifact than production exports — sizes shrink, timings skew, and export bugs around
control/back-matter data stay hidden. Prefer reusing the assembled catalog/envelope (e.g.,
structural update `..catalog`, or the shared pipeline helper used by `forge export`) instead of
reconstructing field-by-field.

+     // Keep the artifact identical to what the real pipeline produces; only override
+     // the placeholder metadata populated later by assemble_metadata.
      let oscal_catalog = forge::oscal::OscalCatalog {
          uuid: metadata.uuid.to_string(),
          metadata: forge::oscal::catalog::OscalMetadata {
              title: metadata.title,
              last_modified: metadata.last_modified.to_rfc3339(),
              version: metadata.version,
              oscal_version: metadata.oscal_version,
          },
-         controls: vec![],
-         groups: catalog.groups,
-         back_matter,
+         ..catalog
      };


══════ F0032 │ benches/export_bench.rs:87-87 │ [test · medium] ══════
[test · medium] Embedding the runtime-derived payload size in the Criterion function id means any
fixture edit that shifts the byte count forks a brand-new time series (`json_to_xml_503kb` vs
`json_to_xml_498kb`), orphaning stored baselines and breaking cross-commit comparisons with
`--baseline`. Use stable ids (e.g., `catalog_json_to_xml`) and log/measure the size separately.

-     group.bench_function(format!("json_to_xml_{json_size_kb}kb"), |b| {
+     tracing::info!(size_kb = json_size_kb, "catalog JSON fixture size");
+     group.bench_function("catalog_json_to_xml", |b| {


══════ F0030 │ benches/export_bench.rs:88-95 │ [performance · medium] ══════
[performance · medium] Each iteration performs `deserialize → validate → serialize` PLUS a full
filesystem write (and implicit std::fs read) via `write_output`, so the measured time is
dominated/contaminated by page-cache-dependent disk I/O variance, not just the conversion cost. That
may be intentional if SC-005 is defined end-to-end, but then the noise floor makes the `< 1s`
comparison unreliable; otherwise cache the output as a string once outside the loop (or call
serialize-only APIs) to measure the actual transform.


══════ F0026 │ benches/parameter_extraction.rs:120-125 │ [other · medium] ══════
[other · medium] Same timed-region clone problem applies here. Additionally, `extract_parameters`
genuinely returns `Err(ForgeError::ParameterExtraction)` (not just a theoretical invariant): an
`Err` makes the benchmark abort via panic instead of recording/surfacing which input class failed,
silently narrowing the set of inputs the "p95 ≤ 1s" result was validated against. That may be
acceptable fail-fast behavior for a bench, but it should be a deliberate choice — e.g.,
counting/measuring the error path or documenting why `Err` is unreachable for this corpus.

      c.bench_function("extract_parameters/1_requirement", |b| {
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


══════ F0052 │ benches/pipeline_benchmark.rs:56-58 │ [performance · medium] ══════
[performance · medium] Production serializes through validate_and_serialize(), which first converts
the envelope to serde_json::Value and runs full OSCAL schema+semantic validation before
pretty-printing; that validation cost is a material part of the real end-to-end latency. Here Step 5
goes straight to serde_json::to_string_pretty, so the full-pipeline number understates production by
the entire validation phase. Either include the same validate step (via whatever of forge::validate
is publicly callable) or document explicitly that this benchmark excludes validation so readers
don't treat it as end-to-end wall time.


══════ F0053 │ benches/pipeline_benchmark.rs:72-75 │ [test · medium] ══════
[test · medium] The existence assertion validates the file is present but not that its content
parses into meaningful output. If the fixture generator output or parsing heuristics change such
that extract_sections returns zero sections (fixture regenerated with different heading style, parse
rules changed), every downstream benchmark still runs successfully — measuring near-trivial work on
empty documents and producing plausible-looking but worthless numbers. After pre-computing, assert
!sections.is_empty() (and ideally >0 requirements) so a broken fixture fails loudly instead of
silently benchmarking empty structures.


══════ F0021 │ benches/uuid_benchmark.rs:5-5 │ [test · medium] ══════
[test · medium] Input divergence across benches: bench_normalize_for_hashing measures
whitespace-padded text while both generate_stable_id benches measure cleanly spaced text. As a
result the reported numbers are not comparable across benchmarks — you cannot tell how much of
generate_stable_id's cost comes from the normalization step it presumably performs, and comparing
"normalize" vs "generate" baselines is misleading. If this split is deliberate (e.g. isolating
worst-case whitespace handling), add a comment stating so; otherwise benchmark each helper on the
same set of representative samples.

-     let text = "  All  users  must  use  multi-factor  authentication  ";
+ // Shared samples so all benches measure comparable work.
+ const PADDED_SAMPLE: &str = "  All  users  must  use  multi-factor  authentication  ";
+
+ fn bench_normalize_for_hashing(c: &mut Criterion) {
+     let text = PADDED_SAMPLE;
+     c.bench_function("normalize_for_hashing", |b| {


══════ F0101 │ benches/xml_benchmark.rs:28-29 │ [maintainability · medium] ══════
[maintainability · medium] Copy-pasted pipeline setup: these ~14 ingest→atomize→assign_ids→cite
stages are duplicated verbatim in build_component_def_from_fixture below, and eight chained unwraps
give no indication of which stage failed if the fixture ever breaks the pipeline. Since
forge::pipeline::prepare_document is pub(crate) and unavailable to benches, extract one local helper
(e.g. fn prepared_document(fixture_path) -> PolicyDocument) consumed by both builders; that removes
the duplication drift risk and lets each step carry an expect() message naming the stage.

- fn build_catalog_from_fixture(fixture_path: &Path) -> forge::oscal::catalog::OscalCatalog {
-     let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES).unwrap();
+ fn prepared_document(
+     fixture_path: &Path,
+ ) -> forge::model::PolicyDocument {
+     let ingested = forge::ingest::ingest_file(fixture_path, MAX_SIZE_BYTES)
+         .expect("ingest stage failed");
+     let content = ingested.reconstruct_content();
+     let sections = forge::parse::extract_sections(&content).expect("section extraction failed");
+     // ... remaining shared stages ...
+ }


══════ F0103 │ benches/xml_benchmark.rs:85-88 │ [test · medium] ══════
[test · medium] Silent no-op bench: if the fixture is missing, tracing::warn! is almost certainly
invisible (criterion harnesses typically install no tracing subscriber) and the early return means
cargo bench exits green having produced zero measurements — a CI regression (renamed/deleted
fixture, wrong cwd) would go unnoticed. FIXTURE_PATH refers to a fixture that is committed alongside
this bench, so treat its absence as a hard error rather than an optional skip.

      if !fixture_path.exists() {
-         tracing::warn!(fixture = %FIXTURE_PATH, "Skipping XML benchmark: fixture not found");
-         return;
+         eprintln!("FATAL: XML benchmark fixture not found: {FIXTURE_PATH}");
+         panic!("required fixture missing: {FIXTURE_PATH}");
      }


══════ F0914 │ ci/integration-test.sh:236-240 │ [maintainability · medium] ══════
[maintainability · medium] Every conditional branch re-implements the bookkeeping that
`check`/`check_skip` already encapsulate, with hand-maintained combinations of TOTAL/FAILED/SKIPPED
increments (here +2 twice, +1 in Steps 1c–1f, Step 3 and Step 4 skip blocks). Any drift between a
`skip`/`fail` printout and the surrounding arithmetic silently corrupts the final tally (e.g.
double-counting when someone adds a third skip line but forgets `TOTAL=$((TOTAL+2))` → stale
constant). Factor explicit `record_fail`/`record_skip` helpers (mirroring `check`/`check_skip`) and
replace all six-plus manual blocks so counters have a single update site.

-     TOTAL=$((TOTAL + 2))
-     SKIPPED=$((SKIPPED + 2))
-     skip "Generate assessment-plan (JSON) — --import-ssp not available in this build"
-     skip "Assessment-plan file exists — --import-ssp not available"
+ record_fail() { TOTAL=$((TOTAL + 1)); FAILED=$((FAILED + 1)); fail "$*"; }
+ record_skip() { TOTAL=$((TOTAL + 1)); SKIPPED=$((SKIPPED + 1)); skip "$*"; }
+ ...
+     record_skip "Generate assessment-plan (JSON) — --import-ssp not available in this build"
+     record_skip "Assessment-plan file exists — --import-ssp not available"
  fi


══════ F0912 │ ci/integration-test.sh:294-295 │ [bug · medium] ══════
[bug · medium] Unlike xmllint, `python3` is never feature-detected. On hosts without it, these
structural checks return 127 from the command-not-found and are counted as FAILED by `check`,
hard-failing the entire run — contradicting the header's stated dependency policy ('xmllint optional
… graceful skip', implying missing extras degrade, not fail). Detect `HAS_PYTHON3` once (mirroring
the xmllint check) and route through `check_skip`. Separately, interpolating `${PROFILE_JSON}` /
`${AP_JSON}` directly into the `python3 -c` source is a fragile quoting/injection pattern; passing
the path as an argv argument keeps the code static and safe.

+ HAS_PYTHON3=false
+ command -v python3 >/dev/null 2>&1 && HAS_PYTHON3=true
+ ...
      check "Profile JSON has 'profile' root key" \
-         python3 -c "
+         python3 -c '
+ import json, sys
+ with open(sys.argv[1]) as f:
+     data = json.load(f)
+ assert "profile" in data, "Missing profile root key"
+ ' "${PROFILE_JSON}"


══════ F0913 │ ci/integration-test.sh:39-40 │ [performance · medium] ══════
[performance · medium] `FORGE` is deliberately left unquoted so the multi-word 'cargo run --quiet
--release --' fallback expands as separate words, but that breaks the documented usage
`./ci/integration-test.sh [FORGE_BIN]` as soon as the binary path contains whitespace
(word-splitting mangles it into several arguments), and it forces every one of the ~20 forge
invocations in this suite to go through cargo again, repeating dependency/rebuild checks each time
and dominating wall-clock time. Resolve to a plain executable path once (auto-building first if
needed) so expansions can be safely quoted and cargo is paid for exactly once.

- FORGE="${1:-}"
- if [[ -z "${FORGE}" ]]; then
+ if [[ -n "$1" ]]; then
+     FORGE="$1"
+ elif [[ -x "target/release/forge" ]]; then
+     FORGE="target/release/forge"
+ elif [[ -x "target/debug/forge" ]]; then
+     FORGE="target/debug/forge"
+ else
+     cargo build --quiet --release   # single up-front build instead of N re-launches
+     FORGE="target/release/forge"
+ fi


══════ F0915 │ ci/integration-test.sh:405-405 │ [test · medium] ══════
[test · medium] The summary gates solely on FAILED > 0, so a badly degraded environment (old forge
without diff/trace/import-ssp, missing xmllint/python3, schema fixtures absent) can reduce nearly
every check to SKIP and still exit 0, green-lighting CI while validating essentially nothing — a
quiet masking vector for regressions. Either treat feature-detection shortfalls as fail-worthy for
checks this script declares core (catalog/profile/component generation), or additionally fail when
PASSED falls below a floor (including the degenerate PASSED == 0 case) so inconclusive runs cannot
masquerade as successes.

  if [[ ${FAILED} -gt 0 ]]; then
+     echo -e "${RED}Integration test FAILED${NC}"
+     exit 1
+ elif [[ ${PASSED} -eq 0 ]]; then
+     echo -e "${RED}Integration test INCONCLUSIVE: every check was skipped${NC}"
+     exit 1


══════ F1034 │ deny.toml:21-23 │ [security · medium] ══════
[security · medium] wildcards = "allow" fully disables the diagnostic for dependencies specified
with '*'/loose version requirements anywhere in the tree, so an accidental '*' pin (which silently
tracks breaking major releases and unreviewed transitive upgrades) is never surfaced by this policy
— the opposite of what a supply-chain gate usually wants. Combined with multiple-versions = "warn"
(which also never hard-fails), duplicate-version drift is accepted by default. Recommend at minimum
wildcards = "warn" (pair with [bans.highlight] so hits are visible), and prefer "deny" plus targeted
skips for any wildcard that can't be eliminated yet.

  [bans]
  multiple-versions = "warn"
- wildcards = "allow"
+ # Fail on '*' requirements so accidental wildcard pins cannot enter the tree.
+ wildcards = "deny"


══════ F1035 │ deny.toml:25-27 │ [security · medium] ══════
[security · medium] unknown-registry and unknown-git are only "warn", while the allow lists below
declare a crates.io-only policy. At warn level, a cargo-deny run completes successfully (non-zero
exit comes only from Error-level diagnostics), meaning a dependency redirected to a git fork or
third-party registry passes CI silently — the declared source policy is effectively decorative,
leaving room for typo-squatting-style redirection. If this file is meant to enforce sources, use
"deny" so disallowed sources actually fail the check.

  [sources]
- unknown-registry = "warn"
- unknown-git = "warn"
+ # Hard-fail builds pulling from any registry/git origin not on the allow lists.
+ unknown-registry = "deny"
+ unknown-git = "deny"


══════ F1032 │ deny.toml:4-6 │ [maintainability · medium] ══════
[maintainability · medium] The rationale matches this repo's actual state (pdf-extract is a direct
dep in Cargo.toml:26, and supply-chain/config.toml carries vet exemptions for lopdf, pdf-extract,
and ttf-parser 0.25.1), so the suppression itself is justified. However, an ignore keyed only by
RUSTSEC ID is permanent: it stays active forever even after lopdf updates or the advisory is
superseded/withdrawn, and nothing in CI ever flags a stale no-longer-applicable ignore. Add a
concrete, dated review trigger (e.g., REVIEW-BY date plus an owning GitHub issue) and keep the verb
"Revisit" actionable; also consider upgrading to cargo-deny's inline-table form { id = ..., reason =
... } (supported by recent cargo-deny) so the justification is machine-readable in diagnostic
output.

      # ttf-parser 0.25.x is unmaintained but still pulled transitively by lopdf 0.42 (via pdf-extract);
-     # unmaintained is informational, not a vulnerability. Revisit when lopdf drops it.
+     # unmaintained is informational, not a vulnerability.
+     # REVIEW-BY 2026-12-31 (owner: deps-rotation): drop this ignore once lopdf
+     # upgrades past ttf-parser 0.25.x, or bump ttf-parser directly.
      "RUSTSEC-2026-0192",


══════ F0016 │ examples/component-based/generate_ssp.py:17-18 │ [bug · medium] ══════
[bug · medium] Duplicate control-ID collision: in a component-based architecture the same control-id
frequently appears under several components (e.g., a control jointly implemented by the web app and
the database). This flat append keeps every occurrence, so the later loop mints the *same* uuid5
seed (`stable_uuid(f"ir-{cid.lower()}")`) once per occurrence, producing repeated
implemented-requirements entries with non-unique UUIDs and duplicated control statements in the SSP.
Deduplicate while preserving order.

+ seen = set()
  for ir in ci.get("implemented-requirements", []):
-             control_ids.append(ir["control-id"])
+             cid = ir["control-id"]
+             if cid not in seen:
+                 seen.add(cid)
+                 control_ids.append(cid)


══════ F0018 │ examples/component-based/generate_ssp.py:59-62 │ [bug · medium] ══════
[bug · medium] Not schema-conformant OSCAL 1.1.3: 'responsible-roles' entries are defined with
'role-id' plus 'party-uuids' references (free-text parties belong in metadata with UUIDs), yet this
document invents a 'responsible-party' string field that the metaschema does not allow, and it never
defines these role-ids (system-admin, security-officer, ...) in metadata.roles nor any parties/users
the roles refer to. The generated document will fail validation against the declared oscal-version
despite the hard-coded claim.

  "responsible-roles": [
-                         {"role-id": "system-admin", "responsible-party": "Application Engineering Team"},
-                         {"role-id": "security-officer", "responsible-party": "Security Engineering Team"}
+                         {"role-id": "system-admin", "party-uuids": [stable_uuid("user-system-admin")]}
                      ],


══════ F0008 │ examples/component-based/output/catalog-new.json:428-429 │ [bug · medium] ══════
[bug · medium] Title reads 'TLS 1.' because a decimal number was treated as a sentence terminator
during title extraction: the '.3' was dropped. This contradicts the control's own statement/guidance
prose ('TLS 1.3 for all external communications') and yields a meaningless security requirement.
Splitting on '.' for title detection needs to be avoided for numeric decimals.

              "id": "POL-CSP-010",
-             "title": "The application component SHALL require TLS 1.",
+             "title": "The application component SHALL require TLS 1.3 for all external communications",


══════ F0004 │ examples/component-based/output/ssp.json:114-119 │ [bug · medium] ══════
[bug · medium] This `href` fragment (`#component-web-application`) resolves to nothing in this
document. Fragments must point at a locally-defined identifier — for OSCAL elements that is the
element's UUID, and neither component's `uuid` (`895e13fb-...`, `6114bc33-...`) nor any other node
matches `#component-web-application`. Because the identical dangling reference is copied into all 25
implemented requirements, every `implements` association is silently unresolvable; the link should
target the Web Application Component's actual UUID.

-           "links": [
-             {
-               "href": "#component-web-application",
-               "rel": "implements"
-             }
-           ]
+               "href": "#895e13fb-c041-593d-a571-55b167922209",


══════ F0001 │ examples/component-based/output/ssp.json:23-24 │ [bug · medium] ══════
[bug · medium] `responsible-party` is not a valid key inside an OSCAL `responsible-roles` entry. In
OSCAL 1.1.x, a ResponsibleRole consists of `role-id` plus `party-uuids` (an array of UUIDs
referencing parties declared in `metadata.parties`). This file instead binds plain-text team names
(e.g., `system-admin` -> "Application Engineering Team", `database-admin` -> "Database Operations
Team"), and `metadata` declares no `parties` block at all, so nothing these bindings refer to exists
in the document. Any OSCAL-conformant validator (the header advertises `oscal-version: 1.1.3`) will
reject these keys or leave the roles unresolvable. This invented-key pattern is duplicated
machine-generated style across every component block, so fixing it will require regenerating all
component blocks consistently.

-               "role-id": "system-admin",
-               "responsible-party": "Application Engineering Team"
+               "party-uuids": [
+                 "00000000-0000-4000-8000-000000000001"
+               ]


══════ F0002 │ examples/component-based/output/ssp.json:75-77 │ [bug · medium] ══════
[bug · medium] The `role-id`/`role-ids` values used throughout the plan (`system-admin`,
`security-officer`, `database-admin`, `analyst`, `service-account`) are never declared. OSCAL
requires every referenced role id to resolve to a `role` defined in `metadata.roles`; with no
`roles` block in this document's metadata, none of the responsible-role or user-to-role mappings
validate. Add a `metadata.roles` array declaring all five role IDs (with titles/descriptions).

-           "role-ids": [
-             "analyst"
-           ],
+ // additionally, in metadata:
+ "roles": [
+   { "id": "analyst", "title": "Security Analyst" },
+   ...
+ ]


══════ F0893 │ scripts/ci-local.sh:20-21 │ [performance · medium] ══════
[performance · medium] Running the benchmark suite on every local pass substantially inflates wall
time, and benchmark variance/flakiness can block trivially correct changes. Make benchmarks opt-in
(env-gated) rather than a mandatory gate on each invocation.

+ if [[ "${CI_LOCAL_BENCH:-0}" == "1" ]]; then
  run_step "cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3" \
      cargo bench --bench pipeline_benchmark -- --warm-up-time 1 --measurement-time 3
+ else
+     echo "[ci-local] skipping benchmarks (set CI_LOCAL_BENCH=1 to run them)"
+ fi


══════ F0894 │ scripts/ci-local.sh:22-24 │ [maintainability · medium] ══════
[maintainability · medium] The audit/deny/vet steps in ci-local.sh and the strict audit/deny block
in pre-commit.sh hard-assume cargo-audit/cargo-deny/vet are installed (and, for audit, that the
RustSec advisory DB is reachable); on fresh or air-gapped machines under set -e these die steps-deep
with bare 'no such command', sync errors, or a confusing unknown-bench-target message for the
hardcoded pipeline_benchmark name. Pre-flight the external tools once with command -v in both
scripts and emit actionable hints ('cargo install ...' / unset FORGE_PRECOMMIT_STRICT / disable
benchmarks), ideally discovering or asserting the bench target instead of hardcoding it.

+ for subcommand in audit deny vet; do
+     if ! cargo "${subcommand}" --version >/dev/null 2>&1; then
+         echo "[ci-local] required subcommand 'cargo ${subcommand}' is not installed; install it before running ci-local" >&2
+         exit 1
+     fi
+ done
+
  run_step "cargo audit" cargo audit
  run_step "cargo deny check" cargo deny check
  run_step "cargo vet --locked" cargo vet --locked


══════ F0890 │ scripts/ci-local.sh:5-6 │ [bug · medium] ══════
[bug · medium] scripts/ci-local.sh and scripts/pre-commit.sh derive REPO_ROOT from dirname of
${BASH_SOURCE[0]}, which points at the symlink's directory when launched through a link (e.g.
~/.local/bin/ci-local, or .git/hooks/pre-commit linking to scripts/pre-commit.sh), so the gates run
against the wrong project tree. Resolve the real path of the entrypoint; for the pre-commit/hook
context prefer the authoritative git rev-parse --show-toplevel.

- SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
+ SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"
  REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"


══════ F0895 │ scripts/install-hooks.sh:7-8 │ [bug · medium] ══════
[bug · medium] Hardcoding `${REPO_ROOT}/.git/hooks` breaks in several common setups: repositories
that configure `core.hooksPath` (e.g. husky or a checked-in hooks directory), linked worktrees
(where `.git` is a file, so the `[ ! -d ... ]` guard exits early and other worktrees keep their old
behavior), and repos accessed via a non-default `GIT_DIR`/submodule layout. Git will simply ignore a
hook installed here, so required pre-commit validations silently stop running. Resolve the hooks
directory through git itself instead.

- HOOKS_DIR="${REPO_ROOT}/.git/hooks"
+ # Ask git where hooks live: honors core.hooksPath, linked worktrees,
+ # and non-default GIT_DIR locations.
+ if ! HOOKS_DIR="$(git -C "${REPO_ROOT}" rev-parse --git-path hooks)"; then
+     echo "[install-hooks] not inside a git repository." >&2
+     exit 1
+ fi
  PRE_COMMIT_HOOK="${HOOKS_DIR}/pre-commit"


══════ F0905 │ scripts/pre-commit.sh:23-24 │ [performance · medium] ══════
[performance · medium] Running the full workspace lint + test suite on every commit makes the hook
slow on large repos, which pushes developers toward SKIP_FORGE_PRECOMMIT and undermines the gate
entirely. Additionally, without `--locked` cargo may opportunistically update Cargo.lock mid-commit,
dirtying a tracked file as a side effect of the hook. Add `--locked`, and consider trimming what
runs here (e.g., `cargo test --lib` subset here, full suite in CI).

- run_step "cargo clippy -- -D warnings" cargo clippy -- -D warnings
- run_step "cargo test" cargo test
+ run_step "cargo clippy --locked -- -D warnings" cargo clippy --locked -- -D warnings
+ run_step "cargo test --locked" cargo test --locked


══════ F0904 │ scripts/pre-commit.sh:8-11 │ [security · medium] ══════
[security · medium] A single environment variable silently disables every quality gate (fmt, clippy,
test, and strict-mode audit/deny). Because env vars are inherited ad hoc, an IDE terminal, wrapper
script, or shared CI image exporting SKIP_FORGE_PRECOMMIT=1 will skip all checks with almost no
trace. Mitigations: write the notice to stderr, refuse to honor the override in CI environments,
and/or offer finer-grained skippers (e.g., SKIP_FORGE_TESTS) so heavyweight gates like cargo
audit/deny cannot be dismissed wholesale. Note also the comparison accepts only the literal "1", so
"true"/"yes" silently do NOT skip — surprising but arguably safer; document the accepted values.

  if [[ "${SKIP_FORGE_PRECOMMIT:-0}" == "1" ]]; then
-     echo "[pre-commit] SKIP_FORGE_PRECOMMIT=1, skipping checks"
+     echo "[pre-commit] WARNING: SKIP_FORGE_PRECOMMIT=1, skipping ALL checks" >&2
+     # Quality gates exist to protect shared history; don't let an inherited
+     # environment variable disable them on CI runners.
+     if [[ -n "${CI:-}" ]]; then
+         echo "[pre-commit] refusing to skip checks in CI; unset SKIP_FORGE_PRECOMMIT" >&2
+         exit 1
+     fi
      exit 0
  fi


══════ F0290 │ src/applicability/model.rs:106-110 │ [maintainability · medium] ══════
[maintainability · medium] reason_code is a bare &'static str emitted into serialized,
machine-readable output, yet nothing compiler-checks the vocabulary: a typo or future edit in the
match arms below compiles cleanly and silently changes the contract downstream parsers rely on (the
sibling GapClassification avoids exactly this via a typed, kebab-case-serde enum). Model the queue
reasons as an enum with #[serde(rename_all = "kebab-case")], so reason codes share the same checked
lifecycle as classifications.

- /// Stable, machine-readable human review queue entry.
- #[derive(Debug, Clone, Serialize)]
- pub struct ReviewQueueItem {
-     pub control_id: String,
-     pub reason_code: &'static str,
+ #[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
+ #[serde(rename_all = "kebab-case")]
+ pub enum ReviewReason {
+     ReviewedNoPositiveRelationship,
+     NoReviewedMapping,
+     DeferredScopeDecision,
+     ScopeDecisionRequired,
+ }
+
+ // in ReviewQueueItem:
+ //     pub reason_code: ReviewReason,


══════ F0289 │ src/applicability/model.rs:144-151 │ [maintainability · medium] ══════
[maintainability · medium] The 'sorted' determinism contract is self-enforced for
controls/policy_sources (BTreeSet iteration) but NOT for mapping_collections: the vector is stored
verbatim, so report determinism silently depends on each call site pre-sorting (today only
applicability/mod.rs sorts by uuid right before calling). Any future caller passing unsorted
evidence will violate the schema's determinism guarantee with no error. Enforce the ordering inside
the builder, or at minimum document the pre-condition and protect it with a debug_assert.

- /// Build a sorted, reconciled report from validated inputs.
- #[must_use]
- pub fn build_report(
-     manifest: &ApplicabilityManifest,
-     manifest_sha256: String,
-     framework: ResourceEvidence,
-     inventory: &Inventory,
-     mapping_collections: Vec<MappingEvidence>,
+ let mut mapping_collections = mapping_collections;
+ mapping_collections.sort_by(|left, right| left.uuid.cmp(&right.uuid));
+
+ let mut counts = ClassificationCounts::default();


══════ F0277 │ src/batch/formatter.rs:42-47 │ [security · medium] ══════
[security · medium] `error_message` is interpolated verbatim. Messages produced by the parser/IO
layer frequently echo fragments of input file content or paths, so an embedded `\n`/`\r` breaks the
assumed one-line-per-result layout (breaking anything that parses this stderr block), and raw
control bytes such as ANSI escape sequences enable terminal output forging/log injection. Sanitize
to printable content before rendering.

              FileOutcome::Failure { error_message } => {
+                 // Flatten to one line and neutralize control characters (newlines,
+                 // ANSI escapes) so each result stays on its own line and the
+                 // terminal output cannot be forged by input file contents.
+                 let safe_message: String = error_message
+                     .chars()
+                     .map(|c| if c.is_control() { ' ' } else { c })
+                     .collect();
                  let _ = writeln!(
                      buf,
-                     "  \u{2717} {input_name} \u{2014} {error_message} ({duration_secs:.2}s)"
+                     "  \u{2717} {input_name} \u{2014} {safe_message} ({duration_secs:.2}s)"
                  );
              }


══════ F0293 │ src/batch/orchestrator.rs:108-110 │ [other · medium] ══════
[other · medium] `AssertUnwindSafe` deliberately suppresses the compiler's `UnwindSafe` proof, but
there is no safety rationale here documenting why observing pipeline state after a caught panic is
sound. If `run_pipeline` mutates any shared state (locks, caches, OSCTR model registries), a later
task on the rayon worker threads may observe that state in a half-mutated, inconsistent condition
because panics no longer propagate/poison anything. Under a `panic=abort` build profile
`catch_unwind` never returns either, which changes failure semantics silently. Please add a
documented invariant (what shared state the pipeline touches and why it is reset-safe) and/or reset
per-file mutable state after a caught panic.


══════ F0294 │ src/batch/orchestrator.rs:116-120 │ [maintainability · medium] ══════
[maintainability · medium] The typed `ForgeError` is flattened to a `String` here, permanently
discarding the error variant and its source chain for batch mode. Downstream code that needs
structured classification cannot recover it: e.g. `ForgeError::exit_code()` distinguishes validation
(exit 3), external-dependency (exit 4), etc., yet `execute_dispatch` can only emit a generic "N of M
files failed" `BatchConversion` error because every per-file cause was reduced to prose before it
left this function. Any future consumer wanting per-category exit codes, structured reports (--json
summaries of failures), or retry classification hits this wall. Prefer carrying the `ForgeError`
(already sized/owned after `with_max_size_guidance()`) inside `FileOutcome::Failure` and rendering
`to_string()` only at display boundaries.


══════ F0295 │ src/batch/orchestrator.rs:87-90 │ [other · medium] ══════
[other · medium] When the custom thread pool fails to build, execution degrades silently from
parallel to sequential: the only trace is a log line, and `BatchSummary` is indistinguishable from a
successful parallel run. For large batches this collapses throughput (jobs=1 equivalent) with no
user-visible or machine-readable signal, so callers cannot detect or compensate for the degraded
mode. At minimum record that the batch ran degraded (e.g. a flag/field on `BatchSummary` or a
failed-files-independent metric emitted with the summary); you can also avoid the whole failure path
cheaply by keeping the built pool alive (rayon pools are reusable) rather than rebuilding per
invocation.


══════ F0287 │ src/batch/output_naming.rs:34-34 │ [bug · medium] ══════
[bug · medium] Uniqueness rests on byte-exact `PathBuf` equality, which does not model real
filesystems: on case-insensitive volumes (macOS default APFS/HFS+, Windows NTFS) or
normalization-insensitive ones (macOS NFD normalization), two derived paths like `policy.json` and
`POLICY.json` compare unequal here but resolve to the same physical file, so the second write
silently overwrites the first despite the claimed anti-collision guarantee. Consider folding stems
through a case-insensitive comparison (e.g. lowercase ASCII fold for the `claimed`/suffix keys) or
documenting the byte-exactness assumption.


══════ F0282 │ src/batch/summary.rs:60-63 │ [maintainability · medium] ══════
[maintainability · medium] Every field is public, so callers can build `BatchSummary { .. }`
literally or mutate/push/remove `results` after construction, silently desynchronizing
`total_files`/`succeeded`/`failed` from the list and breaking the documented sort invariant —
`has_failures()` then reports stale results. Encapsulate the state behind read-only accessors (e.g.
`fn results(&self) -> &[FileResult]`) so an inconsistent summary is unrepresentable.

-     /// Total wall-clock duration of the entire batch run.
-     pub total_duration: Duration,
-     /// Per-file results, sorted by input filename (full path as tie-breaker).
-     pub results: Vec<FileResult>,
+     total_files: usize,
+     succeeded: usize,
+     failed: usize,
+     total_duration: Duration,
+     results: Vec<FileResult>,
+ }
+
+ impl BatchSummary {
+     /// Read-only view of the per-file results.
+     #[must_use]
+     pub fn results(&self) -> &[FileResult] {
+         &self.results
+     }
