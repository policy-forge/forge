# slice01 — Validated Findings Report

- Slice: slice01 (60 findings: 3 critical, 6 unspecified-severity, 51 high)
- Validated against HEAD b22e2d5 "Harden successor map opening against symlink races" (2026-08-26)
- Verdict counts: valid=54, partial=5, invalid=1, duplicate=0
- Severity remaps (unspecified): F0513→medium; F0615→medium; F0614→low; F0616→low; F0617→low; F0613→medium (downgraded from the review's own 'high' label — see entry)
- Downgrades from original severity (with justification in entries): F1068 high→low, F0910 high→low, F0005 high→low, F0014 high→medium, F0463 high→medium, F0530 high→medium, F0573 high→medium, F0613 high→medium, F0766 high→low

All line numbers verified against the current tree; several cited ranges drifted slightly from the review text and are given here as observed at HEAD.

---

## CRITICAL FINDINGS

### F1054 — VALID (security · critical)
- Location: `.github/workflows/release.yml:163-172` (cited `uses:` line at 170)
- Symbols: job `provenance`; `uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0`; permissions `id-token: write`, `contents: write`, `actions: read`
- Category: security (supply chain)
- Root cause: The SLSA provenance job delegates the entire attestation pipeline to a reusable workflow pinned by the **mutable tag** `v2.1.0`. Every other action reference in both release.yml and ci.yml is pinned to a full 40-char commit SHA (checkout `93cb6efe…`, rust-toolchain `631a55b1…`, cache `66822842…`, upload/download-artifact `330a01c4…`/`634f93cb…`, gh-release `153bb8e0…`). The provenance job is the single most privileged step in the file: it holds `id-token: write` (OIDC signing) and `contents: write`, and its output (`base64-subjects` from `needs.hash.outputs.hashes`) is what the release publishes as attested subjects. A moved or hijacked `v2.1.0` tag hands an attacker full control of the signing/provenance stage, defeating the SLSA L3 guarantee the workflow exists to provide.
- Evidence: release.yml line 170 reads `uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0`; no comment records the corresponding SHA. Contrast with lines 24, 27, 33, 74, 77, 117, 125, 151, 158, 181, 189 (all SHA-pinned).
- Remediation: Replace the tag pin with the full commit SHA of slsa-github-generator v2.1.0 and keep a `# v2.1.0` comment, exactly like the other actions. No test impact; add/extend a workflow-lint or CI check that rejects non-SHA `uses:` refs (e.g., pinact/dependabot ecosystem or a grep gate in ci-local.sh) so regressions are caught. Snapshot impact: none.

### F0383 — INVALID (security · critical)
- Location: `src/config.rs:671-705` (`ensure_symlink_containment`)
- Category: security
- Rationale: The described bypass does not exist in the current implementation. The walk descends from the fully joined `resolved` path to the deepest **existing** ancestor (`symlink_metadata` succeeds even on symlinks), canonicalizes **that ancestor itself**, then lexically rebuilds only the non-existing tail and requires `rebuilt.starts_with(canonical_root)`. For the review's `esc -> ../..` attack with `output = "esc/d/e/out.json"`: `root/esc` exists as a symlink, so `ancestor = root/esc`, `canonicalize(root/esc)` resolves to the external target, `rebuilt = <external>/d/e/out.json`, and `starts_with(canonical_root)` is false → rejected with "resolves outside the project root through a symbolic link". The variant where every tail component already exists inside the external target canonicalizes the deepest existing component (still external) → likewise rejected. The walk never degenerates to trusting `project_root` itself unless the final component genuinely exists directly inside the project root, which is containment-correct. Residual TOCTOU (a symlink planted between validation and use) is inherent to any path check and is not what this finding describes; the size/read-side TOCTOU is separately tracked as F0384 (valid). No action for F0383.

### F0480 — VALID (security · critical)
- Location: `src/lifecycle/mod.rs:596-612` (`current_artifacts`, joins at 601 and 606), plus the same unconfined join pattern at `src/lifecycle/mod.rs:482` (`execute_transition` proposal-output alias loop), `:921` (`validate_report_destination`), and the consumers `fingerprint` (~540-577) and `relative_path` (1013-1031)
- Symbols: `current_artifacts`, `fingerprint`, `relative_path`, `record_directory`, `record::validate_fingerprint` (`src/lifecycle/record.rs:637-648`)
- Category: security (path traversal / arbitrary file read)
- Root cause: Lifecycle records are reviewer/attacker-influenced JSON; `record::validate` checks fingerprint paths with `non_empty` only (`record.rs:638`, `:659`) — no rejection of absolute paths, Windows prefixes, `..`, or `.` components. `current_artifacts` then does `base.join(&record.policy.source.path)` (line 601) and `base.join(&expected.path)` (line 606). `PathBuf::join` discards `base` entirely when the joined path is absolute, and lexical `..` components escape the record directory. `fingerprint` reads, sizes, and SHA-256 hashes whatever bytes that resolves to and reports them via `relative_path` (which accepts any two absolute paths sharing a root), so the digest surfaces in status/attestation output — an arbitrary-read hash oracle feeding drift/approval trust decisions.
- Evidence: record.rs `validate_fingerprint` (637-648) = non_empty + sha256 shape only. mod.rs:601/606 plain joins. `paths_alias` (used at 482/921) compares canonicalized identities but only for the *output* aliasing check, not for confining the stored artifact paths themselves.
- Remediation: Add a `confined_join(base: &Path, stored: &str) -> Result<PathBuf, ForgeError>` helper in lifecycle/mod.rs: (1) reject any `Path::components()` that is not `Component::Normal` (this rejects RootDir, Prefix, ParentDir, CurDir, and leading-separator absolutes on all platforms); (2) `let joined = base.join(stored); let canon = joined.canonicalize()?;` require `canon.starts_with(base_canonical)` (canonicalize `base` once via `record_directory`). Call it at mod.rs:601, :606, and use it for the artifact paths compared in the alias loops at :482 and :921. Also add a structural check in `record::validate_fingerprint` rejecting stored paths containing traversal components so bad records fail at load, not at fingerprint time. Tests: unit tests in lifecycle/mod.rs — record with `source.path = "/etc/passwd"` rejected; `../../x` rejected; valid relative path still fingerprints. Snapshot impact: none (error-path only).

---

## CI / RELEASE / BUILD CONFIG

### F1062 — VALID (bug · high)
- Location: `.github/workflows/ci.yml:41-82` (single `test` job; audit steps at 60-82)
- Symbols: steps `Run tests`, `Verify OSCAL schema provenance`, `Lint`, `Build release`, `Check formatting`, `Run benchmarks`, `Install cargo-audit`/`Security audit`, `Install cargo-deny`/`License and advisory check`, `Install cargo-vet`/`Supply-chain audit`
- Category: bug (CI signal suppression)
- Root cause: All steps live in one job and GitHub Actions runs steps sequentially with fail-fast semantics: any failing step cancels the rest. A routine test/lint/build failure therefore silently skips `cargo audit`, `cargo deny check`, and `cargo vet --locked`, so advisories, license problems, and supply-chain vet regressions introduced in the same commit go undetected until the next fully green commit. The three audit steps have no `if:` resilience beyond `matrix.os == 'ubuntu-latest'`.
- Evidence: ci.yml lines 41-47 (tests), 49-50 (schema provenance), 52-53 (clippy), 55-56 (release build), then 58-82 are all conditional only on the OS; none have `if: always()`; jobs list contains only `test`.
- Remediation: Preferred: split into a dedicated `audit` job (`runs-on: ubuntu-latest`, `timeout-minutes: 30`) that depends only on checkout + toolchain and runs `cargo audit`, `cargo deny check`, `cargo vet --locked`; these need no build artifacts (cargo audit/deny read Cargo.lock; cargo vet needs the lockfile — which is committed, see F1068). If kept in-job, add `if: matrix.os == 'ubuntu-latest' && always()` to the three audit steps, noting they still require a successful toolchain install. Tests: none applicable; verify by pushing a deliberately failing test commit and observing audit steps still run. Snapshot impact: none.

### F1056 — VALID (bug · high)
- Location: `.github/workflows/release.yml:194-199` (SHA256SUMS step; the hash-output step at 150-156 uses `sha256sum ./* | base64` in the `hash` job and has the same pipefail gap)
- Symbols: step "Generate SHA-256 checksums"; command `sha256sum -- *.tar.gz *.zip *.cdx.json 2>/dev/null | sort > SHA256SUMS`
- Category: bug (silent data loss)
- Root cause: The default step shell is `bash -e` without `pipefail`. Two defects: (1) `2>/dev/null` hides "No such file or directory" when a glob class (e.g. `*.cdx.json` on a run where the SBOM artifact failed to download, or `*.zip` when the Windows build was excluded) matches nothing — the unmatched glob literal is passed to sha256sum, which fails, but the pipeline's exit status is `sort`'s (success), so an incomplete SHA256SUMS is released; (2) a failing `sha256sum` mid-pipe likewise reports success without pipefail. The released checksum file then no longer matches the subjects attested in the SLSA provenance (`needs.hash.outputs.hashes` is computed over `./*` in the `hash` job — a different glob — so the two manifests can silently diverge).
- Evidence: release.yml lines 196-199 exactly as cited; `hash` job lines 150-156 compute subjects from `sha256sum ./*` while the release step globs only `*.tar.gz *.zip *.cdx.json`.
- Remediation: On the SHA256SUMS step set `shell: bash` and `run: |` with `set -euo pipefail`; remove `2>/dev/null`; assert each glob class matches at least one file (`shopt -s nullglob; files=(*.tar.gz *.zip *.cdx.json); [ ${#files[@]} -gt 0 ]`) before `sha256sum -- "${files[@]}" | sort > SHA256SUMS`. Optionally cross-check the manifest against `needs.hash.outputs.hashes` (base64-decode and diff) so the attested subjects and the published checksums cannot diverge. Tests: none applicable. Snapshot impact: none.

### F1068 — PARTIAL (security · high → assessed low)
- Location: `.gitignore:8`
- Category: security (supply chain hygiene)
- Root cause/verification: The finding's central premise — that Cargo.lock is not committed — is FALSE at HEAD: `git ls-files Cargo.lock` shows it tracked, `git cat-file -s HEAD:Cargo.lock` = 67355 bytes, last touched in b22e2d5 itself. `git check-ignore Cargo.lock` exits 1 (ignored patterns do not affect already-tracked files). So the SEC-9 "lockfile must be committed" requirement is satisfied, and CI `hashFiles('**/Cargo.lock')` cache keys resolve correctly. What remains: the `.gitignore` line `Cargo.lock` is stale and actively misleading — it would (a) silently prevent re-adding the lockfile if it were ever accidentally removed (`git add Cargo.lock` no-ops on an ignored path), and (b) contradict the documented release task list (`git add Cargo.toml Cargo.lock …`) by making that add silently skip. The documented release step's "silently skip" hazard is real only in that recovery scenario.
- Remediation: Delete the `Cargo.lock` line from `.gitignore` (line 8) and add a comment `# Cargo.lock must stay committed (SEC-9); do not ignore it.` No code change; no snapshot impact. Downgraded high→low because the operative supply-chain risk (unpinned builds) does not currently exist.

### F0014 — PARTIAL (bug · high → assessed medium)
- Location: `sonar-project.properties:17`
- Category: bug (config)
- Root cause/verification: `sonar.rust.lcov.reportPaths=lcov.info` is present, but no CI workflow, script, or ci/ file references sonar, llvm-cov, or lcov at all (grep across `.github`, `scripts`, `ci` returned zero matches), so the property is currently inert — there is no coverage pipeline for it to misroute. The finding's two structural points stand for whenever SonarCloud analysis is wired up: (1) the property key is not the one the community Rust plugin documents (`community.rust.lcov.reportPaths`), so a plugin-based analyzer would silently ignore it; (2) `lcov.info` resolves relative to project base dir, so the (future) CI step must emit exactly `<repo-root>/lcov.info` (e.g. `cargo llvm-cov --lcov --output-path lcov.info` at the workspace root). The "silently fails showing 0% coverage" consequence is real but latent.
- Remediation: When adding the SonarCloud job, run `cargo llvm-cov --lcov --output-path lcov.info` at repo root and either switch the property to the analyzer's documented key or verify the built-in analyzer honors `sonar.rust.lcov.reportPaths` and comment which analyzer is authoritative. Mark assessed severity medium (latent misconfiguration, no current consumer). Snapshot impact: none.

### F0903 — VALID (bug · high)
- Location: `scripts/pre-commit.sh:22-24` (the three `run_step` gates; strict block 26-31 shares the defect)
- Symbols: `run_step`, gates `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- Category: bug (hook correctness)
- Root cause: The script `cd`s to the repo root and runs all gates against the working tree. With partial staging (`git add -p`, or staged file A while B is modified unstaged), the gates verify a tree that differs from what the commit will contain — committed code may never have been formatted/linted/tested. There is no check for unstaged changes (the only guard is the `SKIP_FORGE_PRECOMMIT` env escape hatch at lines 8-11).
- Evidence: scripts/pre-commit.sh lines 20-24: `cd "${REPO_ROOT}"` immediately followed by the three gates; no `git diff --quiet` / `git status --porcelain` inspection anywhere.
- Remediation: Insert after the `cd`: `if ! git diff --quiet; then echo "[pre-commit] unstaged changes present — gates would validate the working tree, not the staged snapshot; stage or stash first" >&2; exit 1; fi` (optionally also `git diff --cached --quiet` → nothing staged → exit 0). Alternative: stash unstaged (`git stash push --keep-index`), run gates, `git stash pop` with restore-on-failure. Tests: manual — stage a formatted change, dirty another file, expect refusal. Snapshot impact: none.

---

## BENCHMARKS

### F0029 — VALID (test · high)
- Location: `benches/export_bench.rs:74-78` (guard inside `bench_export_pipeline`)
- Symbols: `bench_export_pipeline`, const `FIXTURE_PATH`
- Category: test (silent skip)
- Root cause: If the fixture is missing/renamed, the function logs `tracing::warn!` and `return`s, so `cargo bench` exits 0 with zero benchmarks executed and Criterion history just stops growing — a broken fixture is indistinguishable from success. Contrast `benches/pipeline_benchmark.rs:71-75` and `:152`, which correctly `assert!` fixture existence.
- Remediation: Replace the `if !fixture_path.exists() { tracing::warn!(...); return; }` block with `assert!(fixture_path.exists(), "benchmark fixture missing: {FIXTURE_PATH} (commit it or fix FIXTURE_PATH)")`, matching pipeline_benchmark's pattern. Tests: none needed; the bench harness itself is the surface. Snapshot impact: none.

### F0024 — VALID (performance · high)
- Location: `benches/parameter_extraction.rs:95-101` (`bench_extract_parameters_500`)
- Symbols: `bench_extract_parameters_500`, `extract_parameters`, `make_synthetic_document`
- Category: performance (measurement validity)
- Root cause: `b.iter(|| { let mut d = black_box(doc.clone()); extract_parameters(&mut d)... })` includes a full deep clone of a 500-requirement `PolicyDocument` (hundreds of String/Vec allocations) inside the timed region, so the reported time conflates clone cost with extractor cost and cannot be compared against the PRD NF-1 p95 ≤ 1s target. Because `extract_parameters` mutates in place, a fresh copy per iteration is needed — but it belongs in the setup closure.
- Remediation: Switch to `b.iter_batched(|| doc.clone(), |mut d| { black_box(extract_parameters(&mut d)).expect("extract_parameters must not fail"); }, BatchSize::SmallInput);` (import `criterion::BatchSize`). Same change applies to `bench_extract_parameters_single` at lines 121-127. Tests: bench-only. Snapshot impact: none (criterion baselines will shift — that is the fix).

### F0025 — VALID (performance · high)
- Location: `benches/parameter_extraction.rs:107-113` (`bench_extract_parameters_100`)
- Symbols: `bench_extract_parameters_100`
- Category: performance (measurement validity)
- Root cause: Identical timed-region clone defect as F0024, on the 100-requirement document. (Same root-cause *class* but a distinct benchmark with its own measurement series; not marked duplicate because each bench needs its own fix and each inflates a separately reported metric.)
- Remediation: Same `iter_batched` transformation as F0024. Snapshot impact: none.

### F0051 — VALID (bug · high)
- Location: `benches/pipeline_benchmark.rs:107-108` (`build_catalog_envelope`)
- Symbols: `build_catalog_envelope`; production counterpart `run_catalog_pipeline` (`src/pipeline.rs:172-175`); `PolicyDocument::collect_citations` (`src/model/mod.rs:265-283`); `collect_all_citations` + `MAX_CITATIONS` (`src/oscal/component_definition.rs:237-277`)
- Category: bug (benchmark parity)
- Root cause: `build_catalog_envelope` calls `doc.collect_citations()`, which clones every requirement's citations with **no dedup and no cap**, whereas production uses `collect_all_citations`, which dedups by `citation.id` (HashSet `seen`) and caps at `MAX_CITATIONS = 10_000`. On the 50-page fixture (overlapping citations across ~200 requirements) the bench inflates back-matter resources, skewing `generate_back_matter`, `catalog_assembly`, `serialization_json`, `serialization_yaml`, and the full-pipeline numbers high relative to any real run.
- Evidence: model/mod.rs:265-283 has no seen-set; component_definition.rs:275 `if seen.insert(citation.id.clone())`; pipeline.rs:172-173 uses the deduping collector.
- Remediation: In `build_catalog_envelope` replace `let citations = doc.collect_citations();` with `let citations = forge::oscal::component_definition::collect_all_citations(&doc.sections);`. Apply the same fix to the identical copies in `benches/export_bench.rs:44-45` and `benches/xml_benchmark.rs:43-44` (same defect, same root cause — fix all three call sites together). Tests: bench-only. Snapshot impact: criterion baselines shrink toward production values — intended.

### F0050 — VALID (bug · high)
- Location: `benches/pipeline_benchmark.rs:38-58` (`run_full_catalog_pipeline`)
- Symbols: `run_full_catalog_pipeline`; production `prepare_document` (`src/pipeline.rs:76-121`, `pub(crate)`)
- Category: bug (benchmark drift)
- Root cause: The hand-mirrored pipeline omits production stages: EC-6 no-structure guard (pipeline.rs:92-95), Step 7c `parse::annotate_modalities` (pipeline.rs:113-114), Step 7d `parameter::extract_parameters` (pipeline.rs:116-118), and the version-0.0.0 warning/recomputation. `full_pipeline/catalog_50page` therefore times a pipeline that no longer exists — modality annotation and parameter extraction costs are excluded — and the mirror will keep aging on further refactors.
- Remediation: Best: make `prepare_document` reachable from benches (either `pub` behind `#[doc(hidden)]` like the `testing` module, or a `#[cfg(any(test, feature = "bench-support"))]` re-export) and call it. Minimal: add the missing stages to the mirror in production order: after `extract_citations`, `let doc = forge::parse::annotate_modalities(doc)?; let mut doc = doc; forge::parameter::extract_parameters(&mut doc)?;` before `build_catalog_envelope`. Keep the EC-6 guard too. Tests: bench-only. Snapshot impact: criterion baselines rise — intended.

### F0099 — VALID (bug · high)
- Location: `benches/xml_benchmark.rs:29-45` (`build_catalog_from_fixture`)
- Symbols: `build_catalog_from_fixture`, `build_component_def_from_fixture` (same file, second builder)
- Category: bug (benchmark drift)
- Root cause: Same drift as F0050: the fixture pipeline stops at `extract_citations` and never runs `annotate_modalities`/`extract_parameters`, so serialized controls lack the `modality` prop and all `param` elements that real exports contain. XML serialization benchmarks (`serialize_catalog_to_xml`/`serialize_component_definition_to_xml`) therefore understate real payload work — potentially masking violations of the <50ms budget. Not a duplicate of F0050: different file, different measured stage (serialization vs pipeline), distinct fix surface.
- Remediation: Insert after `let doc = forge::citation::extract_citations(doc).unwrap();` in both builders: `let doc = forge::parse::annotate_modalities(doc).expect("annotate modalities"); let mut doc = doc; forge::parameter::extract_parameters(&mut doc).expect("extract parameters");` — or share the fix from F0050 by exposing `prepare_document`. Tests: bench-only. Snapshot impact: criterion baselines rise — intended.

---

## CI SCRIPT (ci/integration-test.sh)

### F0909 — VALID (bug · high)
- Location: `ci/integration-test.sh:154-160` (CONTROL_IDS pipeline; verified lines at HEAD)
- Category: bug (shell/pipefail interaction)
- Root cause: The script runs under `set -euo pipefail` (line 20). The assignment `CONTROL_IDS=$(grep -o '"id" *: *"[A-Za-z0-9_-]*"' ... | sed ... | grep -v ... | head -20 | tr ... | sed ...)` has two abort modes that fire **before** the `[[ -z "${CONTROL_IDS}" ]]` fallback at line 162 is reached: (a) `head -20` closes its input after 20 lines; upstream stages die with SIGPIPE (exit 141) and pipefail fails the assignment; (b) when the first grep matches nothing or `grep -v` filters everything, grep exits 1 and pipefail propagates. Both kill the script mid-suite (set -e) instead of reaching the intended graceful "no control IDs found" fail branch. Additionally the regex scrapes any `"id"` key anywhere in the catalog JSON (fragile) and silently truncates at 20.
- Remediation: Replace `head -20` with `sed -n '1,20p'` (consumes the full stream — no SIGPIPE), wrap the inverse filter as `{ grep -v ... || true; }` to tolerate no-match, and/or append `|| true` to the whole command substitution if empty is an acceptable outcome for the fallback branch. Longer term, parse structurally (python3/jq: `jq -r '.catalog.groups[].controls[].id'`) limited to control ids. Tests: run the script against a catalog with >20 controls and with a control-free fixture; both must reach the fallback branch instead of aborting. Snapshot impact: none.

### F0911 — VALID (bug · high)
- Location: `ci/integration-test.sh:215-217` (Step 1g assessment-plan generation)
- Category: bug (missing fixture)
- Root cause: The check invokes `forge convert ... --import-ssp "./system-ssp.json"` but no `system-ssp.json` exists anywhere in the repo (glob confirmed zero matches), the script never creates/copies it, and the script `cd`s to repo root so the relative path resolves to a nonexistent file. Because forge treats `--import-ssp` as a reference string (it is sanitized to a filename-only href via `sanitize_artifact_path` and never opened — `src/oscal/assessment_plan.rs:252-254`), the check passes while validating nothing: the named "Generate assessment-plan" PASS is meaningless for the import-ssp contract, and if the semantics ever tighten to open the file the step flips to a spurious hard failure.
- Remediation: Either provision a minimal valid SSP fixture into `${TMPDIR}` before the check and pass its path, or — since the current CLI only embeds the href — keep a constant sentinel href but rename the check to reflect what it actually verifies, and add an explicit `test -f` precondition + SKIP path mirroring the xmllint pattern when a real SSP input is unavailable. Tests: script run. Snapshot impact: none.

### F0910 — PARTIAL (bug · high → assessed low)
- Location: `ci/integration-test.sh:297` (`[[ -f "${AP_JSON}" && -s "${AP_JSON}" ]]`; `${PROFILE_JSON}` unguarded at line 297 of the 2e block)
- Category: bug (set -u hygiene)
- Root cause/verification: The unguarded expansions are real (`${AP_JSON}` at the 2f block and `${PROFILE_JSON}` in 2e/2g lack the `:-` guard other late-bound vars use, e.g. `${PROFILE_JSON:-}` at line 185). However the review's claimed failure mode — `set -u` aborting mid-suite — is largely refuted: (1) `AP_JSON` is assigned unconditionally at line 209 before the `HAS_IMPORT_SSP` branch, so it is never unbound; (2) `PROFILE_JSON` is only assigned in the success branch, but the 2e guard `-f "${PROFILE_JSON:-}"` short-circuits `&&` before the unguarded `-s "${PROFILE_JSON}"` is ever expanded when the file is missing (verified empirically: `bash -c 'set -u; [[ -f "${X:-}" && -s "${X}" ]]'` does not abort). An abort requires a hypothetical code path where the variable is unset AND expansion is reached — not present today. The secondary point (grep -q "profile" on `forge validate --help` is a loose substring) is moot: the `SchemaType` enum (src/cli/mod.rs:786-795) has no `profile` variant, so the branch correctly SKIPs — but it would misfire if the word "profile" appeared in help prose.
- Remediation: Harden anyway (cheap, defensive): expand as `${AP_JSON:-}` / `${PROFILE_JSON:-}` in both conditions, and anchor the capability probe to a distinctive token (e.g. grep -q -- '--schema-type' plus an explicit value check) instead of bare 'profile'. Severity assessed low: latent hygiene, no current abort path. Snapshot impact: none.

---

## EXAMPLES (generated artifacts — root cause redirected to src/)

### F0017 — VALID (bug · high)
- Location: `examples/component-based/generate_ssp.py:30-34` (the generator; the stale output lives in `examples/component-based/output/ssp.json`)
- Symbols: loop building `implemented_reqs`; `stable_uuid(seed)` (uuid5 NAMESPACE_DNS, prefix `forge-ssp-`)
- Category: bug (example-generator correctness)
- Root cause: Every implemented requirement hard-codes `"links": [{"href": "#component-web-application", "rel": "implements"}]` regardless of which component supplied the control, and the fragment `component-web-application` matches no emitted identifier — the SSP's components are identified by `stable_uuid("web-application")` / `stable_uuid("database")` (uuid5 values). So (a) database-inherited controls are mis-attributed to the web component, and (b) the href is a dangling fragment no OSCAL resolver can match. The loop has no record of which component owns each control-id because it flattens all control-ids from the component definition first.
- Remediation: In generate_ssp.py, iterate `cd["components"]` and their `control-implementations[].implemented-requirements[]` preserving component identity; emit `"href": "#" + stable_uuid(component_seed)` where `component_seed` is the seed used for that SSP component (derive a per-component seed map, e.g. from component title, consistent with the two components emitted below). Regenerate `examples/component-based/output/ssp.json` and commit the updated artifact. Tests: none (example script); optionally assert every `links[].href` fragment resolves to a component uuid in the same file. Snapshot impact: regenerated ssp.json example only.

### F0006 — VALID (bug · high; root cause in src/parse)
- Location: root cause `src/parse/mod.rs:106-164` (`extract_sections`); symptom in `examples/component-based/output/catalog-new.json:12-13`
- Symbols: `extract_sections`; pulldown-cmark `Parser::new` without metadata-block handling
- Category: bug (parse)
- Root cause: The example policy starts with YAML front matter (`---\ntitle: …\n---`). `extract_sections` parses raw content with default pulldown-cmark options; the front-matter block is seen as a paragraph followed by a setext heading (the closing `---` underlines the paragraph), so a bogus SectionNode is created whose title is the concatenated key/value text and whose id slugifies to `title-component-based-security-policy-version-1-0-0-author-…` — exactly the bogus first group in catalog-new.json. `assemble_document` separately parses front matter for metadata (model/frontmatter.rs) but the *section extractor* never strips it, so the same text is double-consumed: once as metadata, once as a phantom group duplicating the document title.
- Remediation: In `extract_sections` (and in `extract_clauses`, which has the same blind spot), strip a leading YAML front-matter block before parsing: reuse `model::frontmatter` detection (content starts with `---\n` … closing `---` line) and pass only the remainder to `Parser::new` — adjusting line-start offsets by the number of stripped lines so `source_line` stays correct, or enable pulldown-cmark's metadata handling and ignore the metadata event. Add a unit test in parse/mod.rs: document with front matter yields no phantom section and correct source lines. Regenerate examples/component-based/output/*.json (catalog-new.json, profile-new.json) — their leading bogus group disappears and line annotations shift. Snapshot impact: any snapshot asserting the phantom group must be updated (expected, intentional).

### F0007 — VALID (bug · high; root cause in src/oscal/catalog.rs)
- Location: root cause `src/oscal/catalog.rs:283-297` (`derive_control_title`); symptom in `examples/component-based/output/catalog-new.json` (e.g. POL-CSP-004, POL-CSP-021, POL-CSP-022, POL-CSP-025)
- Symbols: `derive_control_title`, called from `build_catalog` (catalog.rs:387)
- Category: bug (data loss in generated titles)
- Root cause: `derive_control_title` clips the first sentence at 120 chars and appends a literal `...`. Confirmed in the generated example: POL-CSP-004 is cut at "admin, editor, and v..." (factually dropping 'viewer'); POL-CSP-022 is cut inside an `{{ insert: param, id-ref: p7bc6fb2f-376d-57ab-97c1-...` token, breaking the reference to declared param `p7bc6fb2f-376d-57ab-97c1-46fbdb109150_prm_0`; the full text survives only in the `_smt` part. Clipping can bisect markup/param tokens, not just words.
- Remediation: Remove the truncation branch in `derive_control_title` — store the full first-sentence text (or full text when no sentence punctuation). Update the doc comment (the "truncate and append `...`" contract) and the unit tests asserting the ellipsis behavior (catalog.rs tests around line 714-717, including `multibyte_requirement_text_title_truncates_without_panic` in assessment_plan.rs:752-759 if it asserts the 120-char cap — that one is for assessment task titles via `assessment_task_title`, a different function with MAX_REQUIREMENT_CHARS=77; decide separately whether AP task titles keep their cap, but catalog control titles must be full). Regenerate examples/component-based/output/catalog-new.json. Snapshot impact: snapshot tests embedding truncated titles need regeneration.

### F0005 — PARTIAL (maintainability · high → assessed low; root cause already fixed)
- Location: stale artifact `examples/simple-access-control/output/profile.json:12`
- Symbols: current generator `build_profile` → `sanitize_artifact_path` (`src/io.rs:115-118`)
- Category: maintainability (stale generated artifact)
- Root cause/verification: The committed profile.json embeds the author's absolute path `/Users/bluby/.hermes/.../catalog.json` as `imports[0].href`. However the current src/ code already prevents this: `build_profile` (src/oscal/profile.rs:241, 262) passes the catalog path through `sanitize_artifact_path`, which returns the file name only — proven by tests `build_profile_href_uses_filename_only` and `profile_import_href_uses_filename_only` asserting `href == "catalog.json"` with no `/`. The defect survives only in the committed example artifact generated by an older build.
- Remediation: Regenerate `examples/simple-access-control/output/profile.json` with the current binary (`forge profile --catalog output/catalog.json --include … --format json --output output/profile.json`); the href becomes `catalog.json`. No src/ change needed. Downgraded high→low: root cause fixed in code; only a misleading example remains. Snapshot impact: example artifact only.

---

## src/applicability

### F0308 — VALID (security · high)
- Location: `src/applicability/mod.rs:284-313` (`mapping_reviewer_evidence`; the `let party_uuids = parties.keys().copied().collect();` at ~line 305) consumed by `validate_mapping_edges` (~325-350, reviewer-key membership check at 344-349)
- Symbols: `mapping_reviewer_evidence`, `validate_mapping_edges`, `reviewer_uuids` (BTreeSet of mapping-reviewer party UUIDs)
- Category: security (authorization bypass)
- Root cause: `mapping_reviewer_evidence` computes `reviewer_uuids` correctly restricted to `responsible.role_id == "mapping-reviewer"` — but only to require at least one such party exists. It then returns `party_uuids = parties.keys()` (ALL declared parties) as the authorization set, and `validate_mapping_edges` validates an edge's FORGE `reviewer-key` prop merely against that full party set (`if !party_uuids.contains(&reviewer_uuid)` at line 345). A collection author can therefore declare any person/organization party, cite it as `reviewer-key`, and pass review-provenance validation without any mapping-reviewer involvement — defeating the role check the code appears to enforce.
- Remediation: Return the reviewer-restricted set instead: keep `reviewer_uuids` alive (clone before the `into_iter()` at line 306 consumes it) and return `(reviewers, reviewer_uuids)`; in `validate_mapping_edges` the membership check then enforces that `reviewer-key` resolves to a party that actually holds the mapping-reviewer role. Adjust the error message to "references a party that is not a mapping-reviewer". Tests: unit/integration test — collection with a declared non-reviewer party cited as reviewer-key must fail validation; same edge citing a mapping-reviewer party must pass. Snapshot impact: none (error-path only).

---

## src/batch, src/citation, src/cli/resolve, src/config (TOCTOU)

### F0286 — VALID (bug · high)
- Location: `src/batch/output_naming.rs:14-51` (`derive_output_paths`)
- Symbols: `derive_output_paths`, `claimed: HashSet<PathBuf>`, `next_suffix`
- Category: bug (silent overwrite / self-clobber)
- Root cause: Collision avoidance tracks only names minted in this call (`claimed`), never the filesystem: (a) a pre-existing `policy.json` under `base_dir` is silently overwritten without suffix; (b) when `output_dir` is `None`, an input named `notes.json` converted to JSON derives output `./notes.json` — textually distinct from the input path, invisible to `claimed`, and the write clobbers the source file. `validate_inputs` (orchestrator.rs:25-48) checks existence/regular-file only; nothing upstream detects input==output.
- Remediation: In `derive_output_paths`: probe existence before finalizing (`if candidate.exists()` enter the suffix loop, re-probing each suffixed candidate); and reject/avoid self-clobber by canonicalizing inputs and comparing against derived outputs (error `ForgeError::BatchConversion("output would overwrite input '<path>'")`) when `output_dir` is None and `input.file_stem().ext == format.as_extension()`. Alternatively surface an explicit overwrite policy enum, but the minimal contract is: never silently overwrite existing files, never overwrite an input. Tests: unit tests in output_naming.rs — pre-existing file forces suffix; `notes.json` input with Json format errors (or suffixes) instead of aliasing; update existing tests that assert bare overwrite. Snapshot impact: none.

### F0298 — VALID (bug · high)
- Location: `src/citation.rs:206-215` (`generate_citation_id`), called from `extract_citations_from_text` (citation.rs:136, 152, 167, 179)
- Symbols: `generate_citation_id(requirement_id, citation_text)`; seed `format!("{requirement_id}:{citation_text}")`
- Category: bug (identifier collision)
- Root cause: Two occurrences of identical citation text within one requirement (e.g. duplicate URL — the exact case pinned by test `us1_duplicate_urls_produce_separate_citations`, citation.rs:326-330) produce the same UUID v5, so two separate `Citation` entries share one `id`. Downstream, `collect_all_citations` dedups by `citation.id` (component_definition.rs:275) — so the second occurrence is silently dropped from back matter, and any id-keyed map/anchor collides last-write-wins, losing per-occurrence provenance.
- Remediation: Add an occurrence ordinal to the seed: `generate_citation_id(requirement_id, citation_text, occurrence: usize)` with input `format!("{requirement_id}:{occurrence}:{citation_text}")`; in `extract_citations_from_text` maintain a per-text counter (HashMap<&str, usize>) across the four match loops and pass the count. Note: this changes generated citation IDs — snapshot tests embedding citation uuids need regeneration (expected; document as a deliberate ID-scheme change). Update `us1_duplicate_urls_produce_separate_citations` to assert distinct ids.

### F0348 — VALID (bug · high)
- Location: `src/cli/resolve.rs:69-73` (output path derivation in `execute`)
- Symbols: `execute`, `derive_default_output_path`, `canonical_input`
- Category: bug (data-integrity / self-overwrite)
- Root cause: The user-supplied `--output` is used verbatim with no equivalence check against the input: `forge resolve profile.json --output profile.json` makes oscal-cli write the resolved catalog over the very profile being read, destroying the source. There is also no canonicalized comparison (a relative output aliasing the canonical input slips through), and the recorded output path stays non-canonical while ResolveResult documents an absolute path.
- Remediation: After deriving `output_path`, add: `if output_path == canonical_input || std::fs::canonicalize(&output_path).ok().is_some_and(|c| c == canonical_input) { return Err(ForgeError::InvalidArgument(format!("output path '{}' must differ from the input profile '{}'", …))); }` and canonicalize the recorded path when it exists. Tests: unit test — identical output/input rejected; canonicalized alias rejected. Snapshot impact: none.

### F0384 — VALID (security · high)
- Location: `src/config.rs:300-335` (`load_file`)
- Symbols: `load_file`, `fs::symlink_metadata` gate, `fs::read`
- Category: security (TOCTOU / unbounded read)
- Root cause: The M-10 safety verdict (regular file, not symlink, ≤ MAX_CONFIG_SIZE) comes from `fs::symlink_metadata` taken BEFORE `fs::read`, but the returned byte vector is never re-checked against the cap. Between the metadata gate and the read, the file can grow past 1 MiB (or be swapped for a pseudo-file whose `len()` reads as 0 while the read returns megabytes), letting a hostile environment commit an unbounded allocation to memory while diagnostics still claim the cap was enforced. Note the hardening commit b22e2d5 touched the *successor map* symlink handling, not this path — the gap persists.
- Remediation: In `load_file`, after `let bytes = fs::read(path)?`, enforce `if u64::try_from(bytes.len()).is_ok_and(|len| len > MAX_CONFIG_SIZE) { return Err(ForgeError::Config(format!("…configuration exceeds the {} MiB limit after read ({} bytes)", …))); }` before UTF-8 conversion; optionally also re-stat after read and reject kind changes. Tests: unit test with a config written, then truncated/grown between phases is inherently racy — instead test the post-read check directly (e.g. factor a `check_bytes` helper and unit-test it with >1 MiB input). Snapshot impact: none.

---

## src/export (XML serializer data loss)

### F0448 — VALID (bug · high)
- Location: `src/export/xml_serializer.rs:276-305` (`write_group`)
- Symbols: `write_group`; field `OscalGroup.groups` (src/oscal/catalog.rs:63-65, "Nested sub-groups. OSCAL v1.2.0 allows groups within groups.")
- Category: bug (irrecoverable data loss on export)
- Root cause: `write_group` serializes title, props, links, and controls — but never `group.groups`. Every nested sub-group (and its entire subtree of controls/parts) is silently dropped from exported XML catalogs. Nested `<group>` is valid per the OSCAL Catalog XSD (GroupType: title?, prop*, link*, part*, group*, control*). Undetected because no unit test populates a non-empty nested `groups` (the serializer's own test catalog at xml_serializer.rs:705-709 uses `groups` with empty children).
- Reachability: the `forge export` command deserializes JSON catalogs into `OscalCatalog` (cli/export.rs:64-70 → detect_model_type) and can therefore carry nested groups into `serialize_catalog_to_xml` (cli/export.rs:204-206); the JSON round-trip test `catalog_round_trips_nested_groups` (catalog.rs:1326-1357) proves the model supports them. So a user-supplied nested catalog loses data on JSON→XML export.
- Remediation: In `write_group`, after links and before controls, add: `for subgroup in &group.groups { write_group(writer, subgroup)?; }` (XSD child order: part*, group*, control* — parts aren't modeled on OscalGroup today, so group-before-control is correct). Add a regression test: catalog with a group containing a nested group with controls → XML contains both `<group>` levels and the inner control. Snapshot impact: none until a fixture uses nested groups.

### F0449 — VALID (bug · high)
- Location: `src/export/xml_serializer.rs:535-565` (`serialize_catalog_to_xml`; the groups loop at ~551-553)
- Symbols: `serialize_catalog_to_xml`; field `OscalCatalog.controls` (catalog.rs:37-38, "Root-level controls (not inside any group)")
- Category: bug (data loss on export)
- Root cause: The serializer iterates only `catalog.groups` and never emits `catalog.controls`, though the OSCAL CatalogType permits `control*` at catalog level and the model explicitly supports it (proven by `catalog_round_trips_root_level_controls`, catalog.rs:1358+). A catalog consisting solely of top-level controls exports as a catalog with zero controls.
- Remediation: After the groups loop, add `for control in &catalog.controls { write_control(&mut writer, control)?; }` (matching CatalogType child order group* then control*), plus a regression test with non-empty `catalog.controls` asserting the `<control>` elements appear. Snapshot impact: none until exercised.

---

## src/framework

### F0402 — VALID (bug · high)
- Location: `src/framework/disposition.rs:101-108` (dedup loop inside `validate`)
- Symbols: `validate(file: &mut DispositionFile)`, `finding_ids: BTreeSet<&str>`, field `DispositionRecord.finding_id`
- Category: bug (dedup bypass via UUID spelling)
- Root cause: `uuid::Uuid::parse_str` accepts many spellings of the same UUID (upper/lowercase hex, brace-delimited, urn:uuid:, unhyphenated 32-hex), but the uniqueness set keys on the raw string (`finding_ids.insert(disposition.finding_id.as_str())`), and downstream sorting/report matching also uses raw strings. Two dispositions spelling the same UUID differently pass dedup, then sort/match lexicographically against canonical lowercase ids elsewhere — silent double disposition of one finding.
- Remediation: Parse once per record; key the BTreeSet on the parsed `uuid::Uuid`; collect canonical forms in a parallel `Vec<String>` (or re-normalize in a second pass over `&mut file.dispositions`: `disposition.finding_id = finding_id.to_string()` before the final sort). Stronger alternative: type the field as `uuid::Uuid` with serde (serializes canonical lowercase). Tests: unit test — duplicate id in different spellings (uppercase vs canonical, braced vs bare) rejected. Snapshot impact: none.

### F0463 — VALID (bug · high → assessed medium)
- Location: `src/framework/model.rs:173-179` (`FindingPriority` derive), consumer `src/framework/analysis.rs:120-135` (report sort)
- Symbols: `FindingPriority { Blocking, ReviewRequired, Informational }`, derived `PartialOrd/Ord`
- Category: bug (inverted severity semantics)
- Root cause: The derived `Ord` follows declaration order, making `Informational` the maximum — i.e. the derived ordering declares the LOWEST severity as "greatest". The one real consumer (analysis.rs:120-128 sorts ascending on `(priority, subject_id, …)`) happens to produce sensible output only because ascending puts Blocking first; any other consumer (`max_by_key`, range partitions, binary search) gets silently inverted semantics. Downgraded high→medium because the shipped report ordering is currently correct — the hazard is a semantic trap for future code and a wrong public contract.
- Remediation: Replace the derived ordering with an explicit, documented rank used as the sort key: `impl FindingPriority { pub const fn rank(self) -> u8 { self as u8 } }` (Blocking=0 < ReviewRequired=1 < Informational=2, preserving the existing ascending sort byte-for-byte), change analysis.rs:122/129 to sort on `.priority.rank()`, and drop `PartialOrd/Ord` from the derive (compile-time enforcement that no caller relies on the derived order). Add a unit test asserting `Blocking.rank() < ReviewRequired.rank() < Informational.rank()`. Snapshot impact: none (sort key values unchanged).

---

## src/ingest

### F0433 — VALID (security · high)
- Location: `src/ingest/mod.rs:182-191` (`extract_docx_content`)
- Symbols: `extract_docx_content`, `ZipArchive::by_name("word/document.xml")`, `read_to_string`
- Category: security (zip-bomb DoS)
- Root cause: `max_size_bytes` bounds the compressed DOCX on disk, but `word/document.xml` is decompressed via unbounded `read_to_string` into a String. A crafted zip bomb (tiny on disk, gigabytes decompressed) exhausts memory despite passing size validation.
- Remediation: Bound the decompressed read: `let entry = archive.by_name("word/document.xml")…; let cap = max_size_bytes.saturating_mul(64);` read into a `Vec<u8>` via `entry.take(cap + 1).read_to_end(&mut bytes)` and error `ForgeError::Parse("DOCX word/document.xml exceeds decompression budget")` when `bytes.len() > cap`; then `String::from_utf8(bytes)` with a UTF-8 Parse error on failure. (The finding's suggested snippet double-applied `take`; use a single `Read::take`.) Tests: unit test building an in-memory zip (tempfile) whose document.xml exceeds the budget → Parse error, no unbounded allocation. Snapshot impact: none.

---

## src/lifecycle (record gates)

### F0490 — VALID (security · high)
- Location: `src/lifecycle/record.rs:410-414` (assertions sortedness gate in `validate_history`), plus the parallel relaxation in `validate_approval` → `validate_separation` at record.rs:595-599 (`require_author_evidence = record.schema_version == SCHEMA_VERSION && record.history[event_index].legacy_event_id.is_none()`)
- Symbols: `validate_history`, `validate_approval`, `validate_separation(rules, evidence, require_author_evidence)`
- Category: security (trust-boundary relaxation)
- Root cause: The "assertions must be unique and sorted" invariant is enforced only when `schema_version == SCHEMA_VERSION && legacy_event_id.is_none()`; any `/1` (legacy) record — and any `/2` event that merely carries a `legacy_event_id` — opts out of sortedness/uniqueness checks entirely. `validate_approval` unions assertions across the whole review window and passes the same `require_author_evidence` relaxation to `validate_separation`, so a legacy-carried event lets an approver satisfy `required_roles` counts without author evidence even when `author_reviewer`/`author_approver` separation is configured true. The `event_id` bridge makes this reachable on `/2` records by design, but the trust consequence is undocumented at both gates.
- Remediation: (1) Enforce sortedness/uniqueness of `event.assertions` unconditionally (structural integrity does not depend on the ID scheme): drop the `schema_version`/`legacy_event_id` conjuncts from the gate at 410-414. (2) For author-evidence, either keep the relaxation for genuinely legacy approval windows but document it at `validate_separation`'s call site and in the PRD, or tighten: when `record.schema_version == SCHEMA_VERSION`, apply `require_author_evidence` to windows even if they contain legacy-carried events. Tests: unit tests — `/2` record with legacy_event_id event carrying unsorted/duplicate assertions rejected; separation rule honored across legacy-carried windows per chosen policy. Snapshot impact: none (error-path only).

---

## src/mapping

### F0536 — VALID (security · high)
- Location: `src/mapping/inventory.rs:163-169` (companion load in `load`)
- Symbols: `load(path_label, resource)`, `ResourceManifest` (src/mapping/manifest.rs:107-118), `evidence.resolved_catalog_sha256`; framework-side counterpart `src/framework/manifest.rs:153-169` + `src/framework/analysis.rs:395-399`
- Category: security (missing integrity pin)
- Root cause: The Profile artifact itself is hash-pinned (`expected_sha256` checked at inventory.rs:137-142), but the companion `resolved_catalog` — whose bytes actually produce the Profile's entire inventory, fingerprints, excerpts, and group metadata consumed downstream — is only hashed and echoed as evidence (`evidence.resolved_catalog_sha256 = Some(sha256(&companion_bytes))` at line 168); no expected-companion hash is ever compared. A stale/regenerated/maliciously substituted companion passes every gate whenever its id-set matches `manifest.inventory` — changed titles/prose/excerpts/hierarchies become unattested source-of-truth. The framework loader already enforces exactly this pin: `framework/manifest.rs:163-169` requires `expected_resolved_catalog_sha256` for Profiles and `analysis.rs:395-399` re-verifies it after load — the mapping pipeline is the unguarded path.
- Remediation: Add `#[serde(default)] pub expected_resolved_catalog_sha256: Option<String>` to `mapping::manifest::ResourceManifest` (make it REQUIRED for Profile in `validate_resource`, mirroring framework/manifest.rs:163-169), and in `mapping/inventory.rs` compare before recording: on mismatch, `Err(mapping_error(format!("{path_label}.expected_resolved_catalog_sha256 mismatch: expected {expected}, got {actual}")))`. Tests: unit test — companion hash mismatch rejected; matching accepted. Snapshot impact: none.

### F0476 — VALID (security · high)
- Location: `src/mapping/manifest.rs:428-441` (`validate_resource`; extension checks at 429-431 and 439-441)
- Symbols: `validate_resource`, `ResourceManifest.artifact: PathBuf`, `ResourceManifest.resolved_catalog: Option<PathBuf>`; consumers `mapping/inventory.rs:134` (`manifest_dir.join(&resource.artifact)`) and `:166` (companion join)
- Category: security (path traversal via manifest)
- Root cause: Reviewer-authored manifest paths are gated only by `extension() == Some("json")`. Absolute paths (`/etc/cron.d/evil.json`) and traversal (`../../evil.json`) pass validation and are later joined onto `manifest_dir` and opened (`read_bounded_json`), making the manifest a path-traversal read primitive. The framework side has the same shape (framework/manifest.rs:157 `validate_json_path` — check whether it already confines; if not, fix both).
- Remediation: Add a local-relative guard in `validate_resource` applied to both `artifact` and `resolved_catalog`: reject `is_absolute()` and any component that is not `Component::Normal` (`CurDir`/`ParentDir`/`Prefix`/`RootDir`), with error `"{path}.artifact must be a relative path without '..', '.' or leading separators"`. Verify the framework manifest validator confines identically and align. Tests: unit tests — absolute and `../` paths rejected for both fields. Snapshot impact: none.

---

## src/migration

### F0518 — VALID (bug · high)
- Location: `src/migration/engine.rs:249-251` (identical-text skip inside `match_unique_locators`); pipeline order in `classify` (engine.rs:30-66); droppers `match_unique_normalized_text` (196-229) and `append_candidate_groups` (306-323, skips 1×1 groups)
- Symbols: `classify`, `match_exact_ids`, `match_unique_normalized_text`, `match_unique_locators`, `group_ambiguities`, `append_candidate_groups`
- Category: bug (classification loss)
- Root cause: `match_unique_locators` deliberately skips locator-coincident pairs whose text is identical (lines 249-251), expecting the text pass to pick them up — but the text pass already ran earlier in `classify` (engine.rs:48-54), where the text group was ambiguous (1 old × 2 new) and skipped. When the locator pass then consumes one text-twin (with differing text), the residual text group collapses to 1×1, which both `match_unique_normalized_text` (already finished) and `append_candidate_groups` (drops 1×1 at lines 317-319) ignore. Net: a pair with byte-identical text AND identical locator is silently classified Retired/Added instead of matched. Repro per the finding: old={X("dup"@A), Y("other"@B)}, new={P("dup"@B), Q("dup"@A)} → X↔Q vanishes from every bucket.
- Remediation: In `classify`, between `match_unique_locators(..)` (engine.rs:55-61) and `group_ambiguities(..)` (62-66), re-run `match_unique_normalized_text(&old.requirements, &new.requirements, &mut old_matched, &mut new_matched, &mut entries);`. A 1×1 text match at that point is unambiguous by construction (all real competitors are matched or will enter the ambiguity stage), preserving the never-resolve-arbitrarily rule. Add a regression test reproducing the four-requirement scenario above and asserting X↔Q is matched (ObservedIdChange or Unchanged per locator evidence), not Retired/Added. Snapshot impact: migration reports for corpora containing locator-consumed text-twins change — intended.

### F0513 — VALID (bug · unspecified → assessed low)
- Location: `src/migration/inventory.rs:78` (`collect_section` child path construction); the slice's cited :0-0 reflects a truncated original
- Symbols: `collect_section`, `RequirementLocation.section_path` (src/migration/types.rs:46), `locator_key` (src/migration/engine.rs:439-441)
- Category: bug (ambiguous serialization)
- Root cause: Section paths are built as `format!("{section_path}/{}", child.title)` with no escaping, so a title containing `/` (e.g. "Access Control / Audit") yields a `section_path` string indistinguishable from a real nesting hierarchy ('Access Control' → child 'Audit'). Downstream consumers (migration reports, and `locator_key` which keys matching on `(section_path, line, atom_index)`) cannot unambiguously map a recorded `section_path` back to one unique document location. Internal matching stays self-consistent (both inventories use the same scheme), so impact is limited to report interpretability/reverse mapping — assessed low (the original entry was truncated/unspecified; a fuller sibling entry in the review was rated low).
- Remediation: Escape or replace separator-ambiguous characters when joining: e.g. `child.title.replace('/', "∕")` (or percent-encode `/` as `%2F`) in `collect_section` (inventory.rs:78), keeping the same transformation wherever section_path is reconstructed (uuid.rs section paths use the same pattern at uuid.rs:283 — align both or document they are display-only). Add a unit test: title with `/` produces a section_path that cannot be parsed as deeper nesting (assert no extra separator). Snapshot impact: migration reports containing such titles change — intentional.

### F0530 — VALID (bug · high → assessed medium)
- Location: `src/migration/types.rs:54-61` (`InventoryRequirement`)
- Symbols: `InventoryRequirement { stable_id, normalized_text_sha256, location, #[serde(skip)] pub(crate) normalized_text }`, derived `PartialEq, Eq`
- Category: bug (equality/serialization mismatch)
- Root cause: `PartialEq`/`Eq` are derived over ALL fields including `normalized_text`, which is `#[serde(skip)]` (invisible in report JSON) and `pub(crate)`-mutable. Two requirements identical in every serialized field but differing in hidden text serialize identically yet compare unequal — so `==` and Eq-based dedup/compare disagree with byte-for-byte report comparison. Downgraded high→medium: grep found no current `BTreeSet`/`HashSet` keyed on `InventoryRequirement` and no `==` consumer in src/ — the invariant violation is latent, not yet exploited by shipped logic.
- Remediation: Implement manually: `impl PartialEq for InventoryRequirement { fn eq(&self, other: &Self) -> bool { self.stable_id == other.stable_id && self.normalized_text_sha256 == other.normalized_text_sha256 && self.location == other.location } }` + `impl Eq` (remove `PartialEq, Eq` from the derive; `RequirementLocation` keeps its derive — all its fields serialize). Unit test: two values differing only in `normalized_text` compare equal and serialize identically. Snapshot impact: none.

---

## src/model

### F0573 — VALID (bug · high → assessed medium)
- Location: `src/model/assemble.rs:113-121` (range math in `map_sections_recursive`), helpers `build_child_ranges` (167-178) and `is_in_child_range` (181-183)
- Symbols: `map_sections_recursive`, `build_child_ranges`, `is_in_child_range`
- Category: bug (defensive-contract gap)
- Root cause: The half-open range filter `item.source_line >= range_start && item.source_line < range_end` assumes sibling `SectionNode.source_line` values are strictly ascending and duplicate-free. For out-of-order siblings, `range_end < range_start` makes the filter match nothing — that section's list items vanish silently (violating the SEC-5 no-silent-drop guarantee invoked elsewhere in this file). Duplicate source_lines produce inverted/degenerate child ranges in `build_child_ranges`, misattributing items between parent and child. The parse layer happens to emit ascending lines today, so nothing fails — but this function is the association guarantor and carries no `debug_assert`, not even one.
- Remediation: Add at the top of `map_sections_recursive` (assemble.rs:~113): `debug_assert!(section_nodes.windows(2).all(|w| w[0].source_line < w[1].source_line), "sibling SectionNodes must have strictly ascending source_lines");` (children slices are contiguous sub-slices of sorted parents, so one assert per level suffices since the function recurses). Optionally also harden `build_child_ranges` with the same debug_assert. Downgraded high→medium: latent (parser guarantees order today), and the fix is a contract guard rather than a behavior change. Tests: unit test feeding hand-built out-of-order SectionNodes panics in debug builds (should_panic) — or assert items are dropped nowhere for an in-order corpus. Snapshot impact: none.

### F0568 — VALID (bug · high)
- Location: `src/model/frontmatter.rs:55` (opener), closer search at 58-63
- Symbols: `parse_frontmatter`, `strip_prefix("---\n")`
- Category: bug (CRLF asymmetry)
- Root cause: The opener accepts only LF (`content.strip_prefix("---\n")?` at line 55 returns None for `---\r\n`), while the closer search explicitly tolerates CRLF (`find("\n---\r\n")`, `strip_suffix("\r\n---")` at lines 58-63). A document saved with Windows line endings therefore silently loses its entire front matter (metadata falls back to H1/filename defaults and version to "0.0.0") — contradicting the SEC-005 fault-tolerance intent the closer branch evidences.
- Remediation: Normalize both openers up front: `let rest = content.strip_prefix("---\r\n").or_else(|| content.strip_prefix("---\n"))?;` and keep the closer search as-is (already CR-aware). Add unit tests: CRLF front matter parses identically to the LF twin; mixed-opener edge (LF opener, CRLF closer) parses. Snapshot impact: documents with CRLF front matter gain metadata they previously lacked — intended; regenerate any affected fixture-derived snapshots.

---

## src/oscal

### F0561 — VALID (bug · high)
- Location: `src/oscal/assessment_plan.rs:329-333` (`id_seed` fallback in `generate_assessment_tasks`)
- Symbols: `generate_assessment_tasks`, seed `format!("req-{i}")`, `generate_stable_id("assessment-task|{id_seed}")` / `"assessment-activity|{id_seed}"`, dedup in `complete_assessment_plan` (435-471, activities deduplicated by uuid — test `complete_plan_deduplicates_activity_definitions_by_uuid` at 870-883)
- Category: bug (UUID collision via fallback seed)
- Root cause: A requirement without `stable_id` at index i gets seed `req-{i}`, which is byte-identical to the seed of a different requirement whose literal `stable_id` is `"req-{i}"`. Both then derive the same task UUID and the same activity UUID; `complete_assessment_plan` dedupes activity definitions by UUID (silently dropping the second), while two distinct tasks reference the same activity UUID — breaking referential integrity of the emitted OSCAL. The index fallback also makes UUIDs order-dependent, contradicting the "so UUIDs stay unique" comment.
- Remediation: Use a namespaced fallback that cannot collide with any real stable_id, e.g. `.map_or_else(|| format!("<unset-stable-id:{i}>"), str::to_owned)` (stable_ids are content-derived UUID strings and can never contain `<>`); document in the comment that UUIDs remain ordering-sensitive for inputs lacking stable_id. Tests: unit test — one req with `stable_id = "req-3"` and one without stable_id at index 3 produce distinct task and activity UUIDs. Snapshot impact: AP outputs for documents mixing fallbacks with literal 'req-N' ids change — intended.

### F0618 — VALID (bug · high)
- Location: `src/oscal/back_matter.rs:253-287` (`generate_back_matter` resource loop; uuid computation at ~260)
- Symbols: `generate_back_matter`, `BACK_MATTER_NAMESPACE`, test `two_identical_citations_produce_same_uuid` (545-548+)
- Category: bug (duplicate resource UUIDs)
- Root cause: Distinct citations with identical normalized text+URL hash to the same UUID v5 (determinism by design), but each is pushed as a separate `BackMatterResource` — the output can contain multiple resources sharing one identifier (codified by the `two_identical_citations_produce_same_uuid` test). `href="#<uuid>"` links (generate_control_links, ~295-320) then become ambiguous, violating OSCAL's expectation that back-matter resource UUIDs uniquely identify a resource within a document.
- Remediation: Track emitted UUIDs in the loop: `let mut seen_uuids: HashSet<Uuid> = HashSet::new();` and after computing `uuid`, `if !seen_uuids.insert(uuid) { resource_map.insert(citation.id.clone(), uuid); continue; }` — reusing the existing resource instead of pushing a duplicate (resource_map already maps citation.id → uuid, so links keep working). Update the codifying test to assert ONE resource with both citation ids mapping to the same uuid. Callers: production pipeline (pipeline.rs:175, component_definition.rs:188) and benches all benefit automatically. Snapshot impact: snapshots with duplicate resources shrink — intended.

### F0620 — VALID (security · high)
- Location: `src/oscal/back_matter.rs:262-266` (title fallback in `generate_back_matter`); classification `classify_url` (132-149), `DANGEROUS_SCHEMES` (106)
- Symbols: `generate_back_matter`, `UrlClassification::Dangerous`, `build_resource_parts`
- Category: security (incomplete SEC-2 sanitization)
- Root cause: For `javascript:`/`data:`/`vbscript:` citations, `build_resource_parts` strips the href from `rlinks` and marks `url-status: dangerous-scheme-removed` (verified by tests at 502-528) — but when `citation.text` is empty, the title fallback `citation.url.clone().unwrap_or_default()` (lines 262-264, evaluated BEFORE classification is consulted for the title) copies the raw payload (e.g. `"javascript:alert(1)"`) verbatim into `title`. Titles are display strings downstream renderers print as-is, reintroducing the sanitized payload into rendered output.
- Remediation: Move classification before the title decision and gate the fallback: when text is empty, use the raw URL only for `UrlClassification::Valid(_)` or `Malformed(_)` (benign/unvalidated-but-not-dangerous); for `Dangerous(_)` emit a redacted placeholder like `"[unsafe URL scheme removed]"`; `None` already yields empty. Add test: citation with empty text and `javascript:` URL produces a title that does not contain the scheme payload. Snapshot impact: resources matching this shape change title — intended.

### F0609 — VALID (bug · high)
- Location: `src/oscal/catalog.rs:447-469` (`resolve_group_id`); twin logic `resolve_abbreviation` (477-503) feeds control IDs via `generate_control_id`
- Symbols: `resolve_group_id`, `resolve_abbreviation`, `generate_group_id`, `generate_section_abbreviation`
- Category: bug (deterministic ID collision)
- Root cause: The 4-hex (16-bit) disambiguation suffix is `SHA-256(title)[0..2]` — a pure function of the title with no check against already-issued IDs. Any title appearing ≥3 times under one base slug: occurrences 2..=n all receive the IDENTICAL `{base}-{hash}`, producing duplicate group IDs (and, via `resolve_abbreviation`, duplicate control-id prefixes like `POL-AC-c5c6-001` twice), violating the SC-003 uniqueness invariant. The 16-bit space is also birthday-fragile for large corpora of similar titles. No `issued` set exists to re-check against.
- Remediation: Track issued IDs alongside the counts map (e.g. `HashSet<String>` threaded through `build_catalog`) and loop: `let mut salt = 0u64; loop { hash SHA-256(title || salt.to_le_bytes()); candidate = format!("{base}-{hash[0]:02x}{hash[1]:02x}"); if !issued.contains(&candidate) { issued.insert(...); break candidate; } salt += 1; }`. Apply identically to `resolve_abbreviation`. Tests: unit test — three sections with the same title yield three distinct group ids and control prefixes; ≥3 duplicate-title catalog builds without duplicate IDs across 1000 random titles (property-ish). Snapshot impact: only affects ≥3-duplicate corpora; regenerate such snapshots if any.

### F0608 — VALID (bug · high)
- Location: `src/oscal/catalog.rs:511-513` (`collect_control_ids_from_catalog`)
- Symbols: `collect_control_ids_from_catalog`; consumers `build_assessment_plan` via pipeline.rs:227-228
- Category: bug (incomplete traversal)
- Root cause: The collector iterates only `catalog.groups[].controls[]`, silently dropping control IDs stored in `OscalCatalog.controls` (root-level controls — supported per OSCAL v1.2.0, proven by `catalog_round_trips_root_level_controls`) and in `OscalGroup.groups` (nested sub-groups, proven by `catalog_round_trips_nested_groups`). Consumers like the Assessment Plan builder (pipeline.rs:227-228 uses this collector to derive include-controls) then omit root/nested controls — incomplete plans and false 'all controls included' results.
- Remediation: Make the walk recursive and include root controls:
```rust
pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String> {
    fn walk(groups: &[OscalGroup], out: &mut Vec<String>) {
        for g in groups {
            out.extend(g.controls.iter().map(|c| c.id.clone()));
            walk(&g.groups, out);
        }
    }
    let mut ids: Vec<String> = catalog.controls.iter().map(|c| c.id.clone()).collect();
    walk(&catalog.groups, &mut ids);
    ids
}
```
(Keep the existing no-dedup contract — dedup happens in `build_assessment_plan`.) Tests: unit tests — root-level controls and nested-group controls both collected. Snapshot impact: AP outputs for nested/root-control catalogs gain controls — intended.

### F0585 — VALID (bug · high)
- Location: `src/oscal/implemented_requirements.rs:210-213` (`generate_impl_req_uuid`), called with `global_index` from `map_requirement_to_implemented` (157) and the loop counter at 95/108/113
- Symbols: `generate_impl_req_uuid(stable_id, text, index)`, seed `format!("{stable_id}\0{text}\0{index}")`, `global_index`
- Category: bug (unstable deterministic identifiers)
- Root cause: The positional `global_index` is folded into the UUIDv5 seed, so inserting or deleting ANY requirement re-rolls the implemented-requirement UUIDs of every following requirement even when their stable_id/text are unchanged — breaking traceability links across document versions and producing noisy diffs. The index exists only for uniqueness of the `"no-stable-id"` fallback (derive_control_id_or_fallback at 227-239 uses it too), but `stable_id` is already content-derived, so the index is redundant for the normal case.
- Remediation: Seed from content only and disambiguate genuine duplicates with an occurrence counter: change the signature to `generate_impl_req_uuid(stable_id: &str, text: &str, occurrence: usize)` (seed unchanged in shape) and in the build loop maintain a `HashMap<(String,String), usize>` (or use the existing content-derived `atom_index` on `PolicyRequirement`) so `occurrence` is 0 for unique pairs and increments only for repeated `(stable_id, text)`. For requirements lacking stable_id, keep a positional disambiguator scoped to the fallback group only. Update the T022/T-uuid-index unit tests that assert index-sensitivity. Snapshot impact: component-definition implemented-requirement UUIDs change on multi-requirement documents — intentional; regenerate snapshots.

### F0615 — VALID (medium · unspecified → assessed medium)
- Location: `src/oscal/parts.rs:140-143` (empty-text warn in `build_control_parts`)
- Symbols: `build_control_parts`, EC-1/EC-2 warn
- Category: medium (silent malformed output)
- Root cause: Empty/whitespace requirement text only logs `warn!` and still pushes a statement part with `prose: ""` — OSCAL treats empty prose as malformed, and callers not consuming tracing get no programmatic signal.
- Remediation: Either propagate the condition (return `Result<Vec<OscalPart>, ForgeError>` and bubble through `build_catalog`, failing loudly) or attach a marker prop (e.g. `empty-text: true`) so downstream can detect it; at minimum document that upstream guarantees non-empty text and add a test pinning the chosen behavior. Preferred: typed error, since the pipeline already threads ForgeError. Snapshot impact: if erroring, add an error-path test; if prop marker, snapshots gain a prop.

### F0614 — VALID (medium · unspecified → assessed low)
- Location: `src/oscal/parts.rs:156-158` (guidance gate in `build_control_parts`)
- Symbols: `build_control_parts` guidance branch
- Category: medium (inconsistent empty handling)
- Root cause: The guidance gate is `!text.is_empty()` (never trims) while the requirement-text check above uses `.trim().is_empty()`; whitespace-only guidance `Some("   ")` slips through the documented `Some(non_empty_text)` contract and emits a guidance part of pure whitespace.
- Remediation: Gate on `!text.trim().is_empty()` and decide storage (trimmed vs preserved) consistently; or update the doc comment to state only truly-empty strings are filtered. Add unit test: `Some("   ")` produces no guidance part (if gating). Snapshot impact: whitespace-only guidance bodies disappear from output — intended.

### F0616 — VALID (low · unspecified)
- Location: `src/oscal/parts.rs:176-184` (`build_control_props`), re-exported at `src/oscal/mod.rs:49`
- Symbols: `build_control_props(_requirement) -> Vec<OscalProp>` (always empty)
- Category: low (dead-effect public API)
- Root cause: Public stub retained "for API compatibility" after trace-prop logic moved to `embed_trace_in_catalog`; invites misuse (callers assuming props are populated here and double-adding or dropping metadata). `build_catalog` still calls it (catalog.rs:368) before appending modality props.
- Remediation: Migrate the one internal caller (`build_catalog` at catalog.rs:368 — inline `let mut control_props = Vec::new();`) and either remove `build_control_props` from `parts.rs` and the `pub use` in oscal/mod.rs:49, or annotate `#[deprecated(since = "<next version>", note = "always returns empty; trace props are added by trace_embedding::embed_trace_in_catalog")]`. Removal preferred (clean cutover). Snapshot impact: none.

### F0617 — VALID (low · unspecified)
- Location: `src/oscal/parts.rs:31-33` (`OscalPart.name: String`)
- Symbols: `OscalPart`
- Category: low (weak typing)
- Root cause: `name` is a free-form String although the type's own doc restricts it to statement/guidance/objective/item; arbitrary names round-trip into serialized parts with no guard, producing quietly invalid OSCAL.
- Remediation: Model as `enum OscalPartName { Statement, Guidance, Objective, Item }` with serde rename to the four strings (reject unknowns on deserialize), update the two builders in parts.rs, and audit other constructors (grep `name: "` in oscal/). If genuinely free-form extension names must round-trip, keep a `#[serde(other)] Custom(String)` variant deliberately. Tests: deserialize of unknown name errors (or maps to Custom). Snapshot impact: none if renames match.

### F0613 — PARTIAL (high · unspecified → assessed medium)
- Location: `src/oscal/parts.rs:87-90` (`generate_part_id`), cemented by test `test_generate_part_id_special_chars` (parts.rs:221-224)
- Symbols: `generate_part_id(control_id, suffix)`
- Category: high→medium (schema-validity gap, currently unreachable via shipped pipeline)
- Root cause/verification: `generate_part_id` concatenates a raw control_id with no character validation, and OSCAL ids follow an NCName-like pattern; the unit test even cements `&` passing through. However, reachability is narrower than claimed: control IDs in shipped catalogs come from `generate_control_id` (`POL-{abbreviation}-{NNN}`) where `generate_section_abbreviation` filters to alphanumeric initials (catalog.rs:210-236) — so `&`/spaces cannot reach part ids through build_catalog. The function is nonetheless public API (re-exported via oscal/mod.rs:49), and callers like assessment-plan part construction or third-party use of the library could feed arbitrary control ids (e.g. hand-written OSCAL round-tripped through these builders), so the validation gap is real at the API boundary but not currently producing invalid artifacts from markdown.
- Remediation: Add `sanitize_oscal_id(raw)` mapping non `[A-Za-z0-9._-]` chars to `-` and prefixing when not starting with a letter; apply inside `generate_part_id`; update `test_generate_part_id_special_chars` to assert the sanitized output. Snapshot impact: none for current pipeline output (ids already clean); third-party-visible behavior change — document.

### F0624 — VALID (bug · high)
- Location: `src/oscal/ssp.rs:314-315` (component UUID minting), consumed by `build_ssp_skeleton` (650+)
- Symbols: `Uuid::new_v5(&COMPONENT_NAMESPACE, def.title.as_bytes())`, `build_ssp_skeleton`, `build_control_impl_reqs`
- Category: bug (silent UUID collision)
- Root cause: Component UUIDs derive solely from the title; two `SspComponentInput`s with the same title mint identical v5 UUIDs, `component_uuids` contains duplicates, and per-component implemented-requirement entries silently reference/merge the same component instead of failing — corrupting the SSP with no error.
- Remediation: Pre-check before minting: `let mut seen_titles = HashSet::new(); for def in definitions { if !seen_titles.insert(def.title.trim().to_lowercase()) { return Err(ForgeError::SspBuild(format!("duplicate component title {:?}: title-derived UUIDv5 would collide", def.title))); } }` (add the `SspBuild` variant if absent, or reuse an existing build-error variant). Stronger alternative: seed from (title, ordinal). Tests: duplicate titles rejected with a clear error. Snapshot impact: none (error path).

### F0631 — VALID (bug · high)
- Location: `src/oscal/trace_embedding.rs:108-156` (`embed_trace_in_catalog`)
- Symbols: `embed_trace_in_catalog`, `annotate_controls` walk over `catalog.groups[*].controls`
- Category: bug (incomplete traversal)
- Root cause: The embedding walk covers only top-level groups and their direct controls. Per the model, `OscalCatalog.controls` (root-level controls, catalog.rs:37-38) and `OscalGroup.groups` (nested sub-groups, catalog.rs:63-65) are valid OSCAL v1.2.0; controls there are emitted without trace metadata, contradicting the doc comment 'Walk catalog groups and controls' and understating the `annotated_controls` completion log. Same traversal family as F0608 (distinct function, distinct consequence — not a duplicate).
- Remediation: Extract an `annotate_control(&mut OscalControl, &TraceLinkCollection) -> Option<String>` helper from the loop body; run it over `catalog.controls` first; make the group walk recursive (`fn annotate_group(&mut OscalGroup, …)` recursing into `group.groups`), preserving the group `source-section` prop derivation from the first traceable child control per group (EC-4 skip rule). Tests: root-level control gets 3 props + link; nested-group control annotated; group prop derivation unchanged. Snapshot impact: catalogs with root/nested controls gain trace props — intended.

### F0638 — VALID (bug · high)
- Location: `src/oscal_cli/detector.rs:84-98` (`search_path_with`)
- Symbols: `search_path_with`, candidate `dir_path.join("oscal-cli")`, `candidate.exists()`
- Category: bug (PATH semantics deviation)
- Root cause: Candidate selection uses only `exists()`: (1) a DIRECTORY named `oscal-cli` matches and ends the search (common when a release archive is unpacked onto PATH), so later valid binaries are never considered and detection flips to `functional: false`; (2) no executability check — a non-executable file also terminates the search; (3) exists→canonicalize/spawn is TOCTOU-prone. Both the Windows branch (with PATHEXT) and Unix branch share the defect.
- Remediation: Replace `if candidate.exists()` with metadata inspection: `match std::fs::metadata(&candidate) { Ok(meta) if meta.is_file() && is_executable(&meta) => return candidate.canonicalize().ok().or(Some(candidate)), _ => continue }` where `is_executable` checks `PermissionsExt::mode & 0o111 != 0` on Unix (cfg-gated; on Windows `is_file()` suffices since PATHEXT candidates are inherently executable). Tests: temp PATH dir containing a directory named oscal-cli followed by a real executable → detects the executable. Snapshot impact: none.

### F0643 — VALID (bug · high)
- Location: `src/oscal_cli/invoker.rs:91-97` (the `try_wait` `Err` arm inside `run_oscal_cli`)
- Symbols: `run_oscal_cli`, timeout arm (83-89: kill/wait/join) vs Err arm
- Category: bug (resource leak on error)
- Root cause: The `Err(e)` arm of `child.try_wait()` returns immediately without killing/reaping the child or joining the stderr-drain thread — unlike the timeout arm which does all three. On a transient OS error the child keeps running unsupervised (potential zombie) and a detached reader thread holding the pipe persists.
- Remediation: Mirror the timeout cleanup in the Err arm: `let _ = child.kill(); let _ = child.wait(); let _ = stderr_thread.join();` before returning `ForgeError::OscalCliExecution { exit_code: None, … }`. Tests: hard to unit-test the Err path (try_wait errors are rare) — at minimum keep the timeout-path test green; consider a comment pinning the invariant that every exit path reaps. Snapshot impact: none.

### F0663 — VALID (bug · high)
- Location: `src/parse/clauses.rs:446-455` (event dispatch loop in `extract_clauses`); helpers `handle_item_text` (242-277) and `handle_table_event` (279+)
- Symbols: `extract_clauses`, `handle_list_event`, `handle_item_text`, `handle_table_event`
- Category: bug (dispatch order)
- Root cause: When a GFM table is nested inside a list item (legal markdown: indented table under a list item), `in_table` becomes true but cell `Text`/`Code` events hit `handle_item_text` FIRST (item_stack non-empty, exclude_depth == 0 — tables don't bump exclude_depth, only CodeBlock/BlockQuote do at lines 223-232) and return true, so `continue` skips the table handler entirely. Cell text lands in the list item buffer (duplicated prose) and never reaches `current_cell` — the extracted table gets empty/garbled headers/rows.
- Remediation: Reorder dispatch so the table handler owns events while in a table: `let handled = if table_state.in_table { handle_table_event(…) || handle_list_event(…) || handle_item_text(…) } else { handle_list_event(…) || handle_item_text(…) || handle_table_event(…) }; if handled { continue; }` (table Start/End must still reach the table handler to maintain in_table). Tests: fixture markdown with a 2×2 table indented under a list item → table extracted with correct headers/rows AND the list item does not contain cell text; keep existing non-nested table/list tests passing. Snapshot impact: any fixture combining lists and tables changes — intended.

### F0692 — VALID (bug · high)
- Location: `src/round_trip/comparator.rs:149-178` (`normalize_soft_line_breaks`), guard list at 163-172
- Symbols: `normalize_soft_line_breaks`, `is_markdown_block_start` (180-196), `soft_line_breaks_equivalent`
- Category: bug (misclassification of meaning-altering joins)
- Root cause: The soft-line-break whitelist checks continuation lines only via `is_markdown_block_start`, which recognizes `- ` (with trailing space), `* `, `+ `, `> `, `# `, ordered markers, and indented lines — but NOT setext underlines (`=====`, `-----`), thematic breaks (`---`/`***`), or pipe-table rows (`| a |`). So `"Overview\n-----"` vs `"Overview -----"` is normalized (joined) and reported `DivergenceClass::Acceptable` even though the join turns a setext heading into paragraph text — a fundamental render difference. Same for table rows.
- Remediation: Extend the continuation-line guard: `if index > 0 && (is_markdown_block_start(line) || line.trim_start().starts_with(['=', '|']) || line.chars().all(|c| matches!(c, '-' | '*' | '_')) && !line.is_empty()) { return None; }` (bare dash/star/underscore runs are setext/thematic breaks; require ≥3 chars per CommonMark if being strict). Tests: comparator unit tests — setext heading vs joined line classified as divergence (not Acceptable); pipe-table row likewise; genuine prose soft-break still Acceptable. Snapshot impact: round-trip reports for prose containing such constructs may flip Acceptable→Divergence — intended.

---

## src/trace, src/uuid, src/validate

### F0766 — PARTIAL (security · high → assessed low)
- Location: `src/trace/walker.rs:78-130` (`walk_group` ~78-104, `walk_control` 107-130)
- Symbols: `walk_group`, `walk_control`, input parsed at `src/trace/mod.rs:42-44` (`serde_json::from_str`)
- Category: security (DoS via recursion)
- Root cause/verification: The recursion on untrusted `serde_json::Value` has no depth limit — true as stated. However the practical DoS vector is largely gated: the input must first parse with `serde_json::from_str` (trace/mod.rs:43-44), and serde_json enforces a recursion limit (default 128) during parsing, so a document nested beyond that fails to parse before the walker runs; documents nested within ~128 levels cannot overflow the call stack. The residual hazard is defense-in-depth: the walker is a public-shaped library function (`walk_catalog_elements`), could later be fed a programmatically constructed deep `Value`, and an explicit bound is cheap.
- Remediation: Thread `depth: usize` through `walk_group`/`walk_control` (`walk_*_at_depth`), stop recursing (with `tracing::warn!`) at a generous cap (e.g. 100, well above OSCAL sanity and below stack risk), and keep the parse-time limit as the outer gate. Tests: unit test with a Value nested beyond the cap → walker terminates without panic and logs. Downgraded high→low because the shipped entry path is already bounded by serde_json's parse limit. Snapshot impact: none.

### F0774 — VALID (bug · high)
- Location: `src/uuid.rs:252-262` (hash seed in `assign_stable_ids_to_section_inner`)
- Symbols: `assign_stable_ids_to_section_inner`, seed `format!("{normalized}\0{section_path}\0{source_line}\0{atom_index}")`, module 'Determinism Guarantee' doc on `generate_stable_id` (uuid.rs:147-151)
- Category: bug (volatile identifier coupling)
- Root cause: The v5 seed couples the 'stable' id to volatile layout data: `section_path` (heading titles), `source_line`, and `atom_index`. Any cosmetic edit — paragraph reflow shifting lines, renaming/moving a section, re-parse changing atom order — regenerates stable_id for unchanged requirement text, contradicting the module's documented guarantee ('Same text produces the same UUID, always') and defeating Substantive Change Detection: one inserted line early in a document rewrites IDs for every following requirement → mass false positives in diff/change tracking (and churns lifecycle fingerprints/migration exact-id matching keyed on stable_id).
- Remediation: Seed from content-stable fields only and disambiguate exact duplicates deterministically: compute, during `assign_stable_ids`, an occurrence ordinal per normalized text (index of this occurrence among prior requirements with the same normalized text, document-order) and hash `format!("{normalized}\0{occurrence}")`. This preserves uniqueness for repeated text while making IDs immune to line shifts/section renames. If full migration is too disruptive now, the minimum is amending the doc guarantee to state position-sensitivity explicitly — but the substantive fix is preferred. CAUTION: this changes every generated stable_id — coordinate with the release notes, migration tooling, and snapshot regeneration (all snapshots embedding stable_ids churn). Tests: reflowing whitespace/renaming a section leaves IDs unchanged; duplicate texts get distinct, order-stable IDs. Snapshot impact: pervasive, intentional.

### F0790 — VALID (maintainability · high)
- Location: `src/validate/formatter.rs:110-124` (`classify_error` dispatch chain), helpers `classify_required_property`..`classify_enum_constraint` (126-192), extraction helpers (194+)
- Symbols: `classify_error(raw_message: &str)`, `format_schema_error` (94-106)
- Category: maintainability (brittle string dispatch)
- Root cause: The entire classification pipeline matches exact English substrings of the `jsonschema` crate's human-readable `Display` messages. Verified against the vendored crate (Cargo.toml pins jsonschema 0.45.0; registry source 0.45.1 `src/error.rs` Display impl at lines ~1401-1595): any rewording across a crate upgrade silently drops affected errors into the generic `("schema validation failed", "valid value per schema")` fallback with no compile-time signal, losing field names and constraints. The crate exposes structured `ValidationError::kind()` (`ValidationErrorKind::Required { property }`, `Type { kind }`, `AdditionalProperties { unexpected }`, `Constant`, `Enum`, `MaxLength`, `Minimum`, `Pattern { pattern }`, `Format { format }`, …) which carries the same data type-safely.
- Remediation: Thread `&jsonschema::ValidationError` into classification: change `format_schema_error` to pass `raw_error` (not just `raw_message`) into a `classify_error(err: &jsonschema::ValidationError)` that matches `err.kind()` variants, mapping each to the existing (message, expected) tuples; delete the substring predicates and extraction helpers (`extract_quoted_value`, `extract_trailing_quoted`, `extract_length_constraint`, `extract_parenthesized`) once unused. Keep SEC-2's no-raw-message-passthrough invariant. Tests: existing formatter unit tests should keep passing unchanged (same output strings); add one test per kind variant using a real validator. Snapshot impact: none if strings preserved.

### F0791 — VALID (bug · high)
- Location: `src/validate/formatter.rs:165-177` (`classify_pattern_or_format`)
- Symbols: `classify_pattern_or_format`
- Category: bug (dead classification predicates)
- Root cause: The predicate requires BOTH `"does not match"` AND the literal token `"pattern"` (and for format: `"is not a"` AND `"format"`). Verified against jsonschema 0.45.1 (error.rs:1547-1549): pattern violations Display as `{instance} does not match "{pattern}"` — the word 'pattern' NEVER appears; format violations (error.rs:1407-1409) Display as `{instance} is not a "{format}"` — the word 'format' never appears. So both branches are dead code: every pattern/format violation falls through to the generic `schema validation failed` fallback, losing the specific classification the function exists to provide. (Subsumed by F0790's structural-kind rewrite, but a distinct present-tense bug: the predicates fail TODAY on the pinned crate, not merely after an upgrade.)
- Remediation: Immediate: drop the `&& msg.contains("pattern")` / `&& msg.contains("format")` conjuncts — but note `"does not match"` alone would also collide with any future message; the durable fix is F0790's `kind()` match (`ValidationErrorKind::Pattern { .. }` → ("value does not match required pattern", "pattern match"); `ValidationErrorKind::Format { format }` → (format!("invalid format: expected {format}"), format!("format: {format}"))). Add unit tests driving a real validator with a `pattern` and a `format` schema violation asserting the specific classifications. Snapshot impact: validation reports for pattern/format failures gain specific messages — intended improvement.

---

## INVALID FINDINGS

- **F0383** (security · critical) — src/config.rs `ensure_symlink_containment`: no bypass exists. The walk canonicalizes the deepest existing ancestor itself (not merely its parent), so every attack path in the finding — `esc -> ../..` with non-existing tail, fully-existing external tail, mixed `d/../..` shapes — resolves to a canonical path outside the root and is rejected by the `starts_with` check. The only residual is inherent TOCTOU between validation and use, which is outside this finding's described mechanism and partially covered by the separate, VALID F0384 (read-side re-check).

## DUPLICATE FINDINGS

- None. Candidates were examined and kept distinct: F0024/F0025 (same defect class, separate benchmarks/metrics), F0050/F0099 (same drift class, different files/stages), F0608/F0631 (same traversal gap family, different functions/consequences), F0790/F0791 (F0791 is a live bug on the pinned crate; F0790 is the systemic rewrite), F0298/F0618 (same collision theme at different layers: citation IDs vs back-matter resources; fixing one does not fix the other).
