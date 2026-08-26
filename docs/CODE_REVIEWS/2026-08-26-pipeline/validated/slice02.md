# Validation slice slice02 — validated findings (60)

**Repo HEAD:** b22e2d5 "Harden successor map opening against symlink races"
**Review vintage:** 2026-08-16 (docs/CODE_REVIEWS/ocr_review_2026-08-16.md)
**Validation date:** 2026-08-26

## Verdict summary

| Status | Count | Finding IDs |
|---|---|---|
| VALID | 54 | F0784, F1017, F1049, F0815, F0832, F0836, F0844, F0874, F0947, F1073, F1072, F1052, F1063, F1065, F1057, F1055, F1058, F1059, F0995, F0041, F0042, F0032, F0030, F0026, F0052, F0053, F0021, F0101, F0103, F0914, F0912, F0913, F0915, F1034, F1035, F1032, F0016, F0018, F0008, F0004, F0002, F0893, F0894, F0890, F0895, F0905, F0904, F0290, F0289, F0277, F0294, F0295, F0287, F0282 |
| PARTIAL | 4 | F1020, F1024, F0031, F0293 |
| DUPLICATE | 2 | F0785 (of F0784), F0001 (of F0018) |
| INVALID | 0 | — |

Every cited file/region was re-read at HEAD; line numbers below are current. Four PARTIAL findings had their severity re-assessed from medium to low (their stated harm premises are only partially verified against current code, or the defect is latent-only). No finding was fully invalidated; the two duplicates share a root cause with a stronger primary finding in this slice.

## Verdict table (per finding)

| ID | File (current lines) | Category | Original sev | Assessed sev | Status |
|---|---|---|---|---|---|
| F0784 | src/validate/semantic.rs:25,32-51 | bug | high | high | valid |
| F0785 | src/validate/semantic.rs:25 | bug | high | high | duplicate (of F0784) |
| F1017 | supply-chain/audits.toml:1-3 | security | high | high | valid |
| F1049 | supply-chain/config.toml (12 exemption rows) | security | high | high | valid |
| F0815 | tests/cli_integration.rs:444-447 | test | high | high | valid |
| F0832 | tests/common/mod.rs:75-84 | test | high | high | valid |
| F0836 | tests/export_integration.rs:114-136 | test | high | high | valid |
| F0844 | tests/integration_cross_feature.rs:283-310 | test | high | high | valid |
| F0874 | tests/oscal_cli_round_trip.rs:64-75,131-173 | bug | high | high | valid |
| F0947 | .cargo/config.toml:1-2 | performance | medium | medium | valid |
| F1073 | .gitattributes:16-25 | maintainability | medium | medium | valid |
| F1072 | .gitattributes:18-25 | bug | medium | medium | valid |
| F1052 | .github/dependabot.yml:6-11 | maintainability | medium | medium | valid |
| F1063 | .github/workflows/ci.yml:24-27 | security | medium | medium | valid |
| F1065 | .github/workflows/ci.yml:31-39,62-80 | performance | medium | medium | valid |
| F1057 | .github/workflows/release.yml + ci.yml (all jobs) | other | medium | medium | valid |
| F1055 | .github/workflows/release.yml:161-173 | security | medium | medium | valid |
| F1058 | .github/workflows/release.yml:1-13, ci.yml:1-13 | other | medium | medium | valid |
| F1059 | .github/workflows/release.yml:51-88 | performance | medium | medium | valid |
| F0995 | .rustfmt.toml:1 | maintainability | medium | medium | valid |
| F1020 | Cargo.toml:27 | bug | medium | low | partial |
| F1024 | Cargo.toml:37-38 | security | medium | low | partial |
| F0041 | benches/atomize.rs:10-23 | maintainability | medium | medium | valid |
| F0042 | benches/atomize.rs:40-48 | performance | medium | medium | valid |
| F0031 | benches/export_bench.rs:52-63 | bug | medium | low | partial |
| F0032 | benches/export_bench.rs:87,94 | test | medium | medium | valid |
| F0030 | benches/export_bench.rs:87-112 | performance | medium | medium | valid |
| F0026 | benches/parameter_extraction.rs:104-126 | other | medium | medium | valid |
| F0052 | benches/pipeline_benchmark.rs:37-59 | performance | medium | medium | valid |
| F0053 | benches/pipeline_benchmark.rs:70-74,150-175 | test | medium | medium | valid |
| F0021 | benches/uuid_benchmark.rs:4-24 | test | medium | medium | valid |
| F0101 | benches/xml_benchmark.rs:27-83 | maintainability | medium | medium | valid |
| F0103 | benches/xml_benchmark.rs:85-90 | test | medium | medium | valid |
| F0914 | ci/integration-test.sh (manual counter blocks) | maintainability | medium | medium | valid |
| F0912 | ci/integration-test.sh:294-303,308-320 | bug | medium | medium | valid |
| F0913 | ci/integration-test.sh:39-60 | performance | medium | medium | valid |
| F0915 | ci/integration-test.sh:404-411 | test | medium | medium | valid |
| F1034 | deny.toml:20-22 | security | medium | medium | valid |
| F1035 | deny.toml:24-28 | security | medium | medium | valid |
| F1032 | deny.toml:3-7 | maintainability | medium | medium | valid |
| F0016 | examples/component-based/generate_ssp.py:14-18 | bug | medium | medium | valid |
| F0018 | examples/component-based/generate_ssp.py:59-62,69-72 | bug | medium | medium | valid |
| F0008 | output/catalog-new.json:428 (root cause src/oscal/catalog.rs:283-297) | bug | medium | medium | valid |
| F0004 | output/ssp.json:114-119 (root cause generate_ssp.py:31-35) | bug | medium | medium | valid |
| F0001 | output/ssp.json:23-24 (root cause generate_ssp.py:59-62,69-72) | bug | medium | medium | duplicate (of F0018) |
| F0002 | output/ssp.json:75-77 (root cause generate_ssp.py:38-49) | bug | medium | medium | valid |
| F0893 | scripts/ci-local.sh:20-21 | performance | medium | medium | valid |
| F0894 | scripts/ci-local.sh:22-24 | maintainability | medium | medium | valid |
| F0890 | scripts/ci-local.sh:5-6, scripts/pre-commit.sh:5-6 | bug | medium | medium | valid |
| F0895 | scripts/install-hooks.sh:7-13 | bug | medium | medium | valid |
| F0905 | scripts/pre-commit.sh:22-24 | performance | medium | medium | valid |
| F0904 | scripts/pre-commit.sh:8-11 | security | medium | medium | valid |
| F0290 | src/applicability/model.rs:108-117,239-253 | maintainability | medium | medium | valid |
| F0289 | src/applicability/model.rs:145-216 (caller mod.rs:129-131) | maintainability | medium | medium | valid |
| F0277 | src/batch/formatter.rs:46-52 | security | medium | medium | valid |
| F0293 | src/batch/orchestrator.rs:104-108 | other | medium | low | partial |
| F0294 | src/batch/orchestrator.rs:115-119 | maintainability | medium | medium | valid |
| F0295 | src/batch/orchestrator.rs:84-91 | other | medium | medium | valid |
| F0287 | src/batch/output_naming.rs:21-46 | bug | medium | medium | valid |
| F0282 | src/batch/summary.rs:52-65 | maintainability | medium | medium | valid |

---

## Detailed findings — VALID

### High severity

══════ F0784 │ src/validate/semantic.rs:25, 32-51 │ [bug · high] — VALID ══════
**Symbols:** `collect_resource_uuids` (32-51), `check_orphaned_links` (58-66), `SemanticValidator::validate` (25-29), callers `run_full_validation` (src/validate/mod.rs:314-316), `validate_oscal_model` (src/cli/export.rs:228).
**Root cause:** `validate()` runs `check_orphaned_links(json)` for every artifact type (Catalog, ComponentDefinition, Profile, Mapping — all reach `SemanticValidator` via `run_full_validation`), but `collect_resource_uuids` hardcodes `root_keys = ["catalog", "component-definition", "mapping-collection"]` (line 35). A schema-valid Profile (root key `profile`; detected at src/validate/mod.rs:142-143, schema-validated against the embedded oscal_profile_schema.json) whose links legitimately point at its own back-matter resources yields an empty `resource_uuids` set, so `walk_for_orphaned_links_inner` (lines 84-140) flags every `#…` href as orphaned. Same for any SSP/assessment-plan document routed through this validator.
**Evidence:** hardcoded list confirmed at line 35; `validate()` receives `model_type` but passes it only to `check_missing_references` (line 26). Profile validation is a live path: `forge validate` auto-detects `profile` root and runs full schema+semantic validation (src/cli/validate.rs:73-83).
**Remediation:** Add `OscalModelType::root_key(&self) -> &'static str` returning `"catalog" | "component-definition" | "profile" | "mapping-collection"` (the exact strings already asserted in src/validate/mod.rs:578-581). Change `collect_resource_uuids(json, model_type)` to read only `json.get(model_type.root_key()).pointer("/back-matter/resources")`; thread `model_type` through `check_orphaned_links(json, model_type)` and the call at line 25. Update the three direct unit-test call sites (`orphaned_link_detected`, `valid_links_no_errors`, `no_back_matter_links_with_hash_all_orphaned` in semantic.rs tests) to pass `OscalModelType::Catalog`. Add regression test: profile artifact with a back-matter resource and an internal `#uuid` link yields zero semantic errors; a profile with `#missing` yields one. No snapshot impact (semantic errors are not snapshot-tested for profiles today).

══════ F1017 │ supply-chain/audits.toml:1-3 │ [security · high] — VALID ══════
**Symbols:** `[audits]` table; `supply-chain/config.toml` (no `[imports]`, ~200 `[[exemptions.*]]`).
**Root cause:** audits.toml is literally two lines (`# cargo-vet audits file` + `[audits]`). config.toml declares no `[imports]` and exempts every crate in the graph with blanket `safe-to-deploy`/`safe-to-run`. `cargo vet --locked` (.github/workflows/ci.yml:82-85) therefore passes vacuously: zero recorded human review, and newly added crates silently accumulate another exemption instead of being vetted. This defeats the project's stated supply-chain control (convert exemptions to real audits over time).
**Evidence:** file contents verified (3 lines); grep confirms no `[imports]` and no `[[audits.` anywhere in supply-chain/.
**Remediation:** Add `[imports]` to config.toml referencing a community audit registry (e.g. the cargo-vet community registry / Firefox or Embark-Studios published audits) and begin populating real `[[audits.<crate>]]` entries (who/criteria/version/notes), starting with direct dependencies. At minimum, add a README note in supply-chain/ documenting that the zero-audit posture is deliberate until registries are imported. No CI workflow change required.

══════ F1049 │ supply-chain/config.toml — exemption rows: cff-parser 115-117, fancy-regex 259-261, lopdf 441-443, nom 459-461, pdf-extract 527-529, postscript 555-557, quick-xml 591-593, type1-encoding-parser 839-841, unsafe-libyaml 887-889, wasmparser 963-965, wit-component 1083-1085, wit-parser 1087-1089, zip 1131-1133 │ [security · high] — VALID ══════
**Symbols:** the named `[[exemptions.*]]` entries.
**Root cause:** These crates parse attacker-controlled bytes at runtime (PDF object streams, font programs, XML, zip archives, WebAssembly, YAML). Blanket `safe-to-deploy` exemptions attest only to author-side review, nothing about malformed-input hardening, so the CI vet gate provides no assurance for exactly the highest-risk parsing surface. Distinct from F1017 (empty audits table) — this finding prioritizes WHICH exemptions to convert first.
**Evidence:** all rows grep-confirmed at the listed lines, all `safe-to-deploy`; none has a corresponding real audit. pdf-extract, quick-xml, and zip are direct dependencies (Cargo.toml:26,23,27); the rest are transitive (lopdf, font parsers, wasmparser via tooling/deps).
**Remediation:** Treat these rows as the highest-priority audit backlog: run `cargo vet import` against registries that already audit them (Firefox/Embark) and record genuine audits for lopdf, pdf-extract, quick-xml, zip, unsafe-libyaml first. Independently of vetting status, keep/extend application-side resource limits on hostile-input paths (src/ingest/mod.rs already enforces `max_size_bytes`; consider bounded decompression when reading DOCX `word/document.xml` at src/ingest/mod.rs:179-195).

══════ F0815 │ tests/cli_integration.rs:444-447 │ [test · high] — VALID ══════
**Symbol:** test `convert_format_xml_produces_valid_xml`.
**Root cause:** `if !fixture.exists() { return; }` turns a missing/renamed/uncommitted `tests/fixtures/sample_policy.md` into a silent PASS, permanently disabling the only `--format xml` end-to-end CLI coverage with zero CI signal. It is the ONLY exists-guarded early return in this 1406-line file (verified by grep); every other fixture-dependent test hard-fails (via `.arg(...)` on a literal path or unwrap-on-read).
**Evidence:** lines 444-447 confirmed; fixture is currently committed (tests/fixtures/sample_policy.md exists).
**Remediation:** Replace the guard with `assert!(fixture.exists(), "Required fixture '{}' is missing — XML format coverage disabled", fixture.display());`. No other call sites.

══════ F0832 │ tests/common/mod.rs:75-84 │ [test · high] — VALID ══════
**Symbol:** `pub fn skip_if_missing(path: &Path) -> bool`.
**Root cause:** The helper converts a missing fixture into a quiet green test via an opt-in boolean convention: every one of the ~31 call sites must remember `if common::skip_if_missing(p) { return; }`, and nothing enforces it. A regression in tests/common/fixture_generator.rs or a renamed fixture directory starves entire suites (assessment_plan_test ×14, xml_catalog_test ×8, xml_component_test ×5, xml_validation_test ×4 call sites verified) of coverage with only an `eprintln!` trace that CI logs typically drop.
**Evidence:** helper body confirmed at lines 77-84; call-site pattern grep-confirmed in four test files.
**Remediation:** Replace with a panicking helper: `#[track_caller] pub fn require_fixture(path: &Path) { assert!(path.exists(), "required fixture missing (run fixture generator?): {}", path.display()); }` and migrate every `if common::skip_if_missing(p) { return; }` to `common::require_fixture(p);` (mechanical, ~31 sites). The synthetic fixture generator is deterministic (no randomness/time, per its own header), so absence is always a genuine defect. Delete `skip_if_missing` once migrated.

══════ F0836 │ tests/export_integration.rs:114-136 │ [test · high] — VALID ══════
**Symbol:** test `cli_export_read_only_output_path` (T046, EC-4).
**Root cause:** The test chmods a dir to 0o444 (line 121) and asserts `run_export` fails (line 133). Mode bits do not restrict root (or CAP_DAC_OVERRIDE) — the default user in most privileged Docker CI containers — so there the write succeeds and `assert_ne!(exit_code, 0)` fails spuriously.
**Evidence:** test body confirmed at lines 114-136; no environment probe exists; cleanup restores 0o755 at line 130.
**Remediation:** After `set_permissions`, probe with `std::fs::write(readonly_dir.join(".probe"), b"x")`; if `Ok`, remove the probe, restore 0o755, `eprintln!` a skip notice, and return — so the assertion only runs where the sandbox can actually induce EACCES.

══════ F0844 │ tests/integration_cross_feature.rs:283-310 │ [test · high] — VALID ══════
**Symbol:** test `atomized_normative_advisory_each_gets_correct_prop`; helpers `collect_modality_props` (98-110), `count_controls` (166-171); fixture `MIXED_POLICY` (34-47).
**Root cause:** The test's stated purpose is verifying EACH atomized half of compound bullet 4 ("Systems must enforce MFA and should notify administrators of policy violations.") carries its own modality. But the assertions only check GLOBAL presence of some "normative" and some "advisory". Bullet 1 ("must enforce multi-factor…") already yields normative and bullet 3 ("should review access logs…") already yields advisory, so the test passes unchanged if the atomizer assigns both halves of bullet 4 the wrong modality, drops props on the split controls, or stops attributing them. Only the `total_controls >= 5` guard catches full non-splitting, and that is a blunt proxy.
**Evidence:** MIXED_POLICY bullets confirmed at lines 43-46; assertions confirmed at lines ~300-309.
**Remediation:** Add `fn find_control_by_text(catalog: &Value, needle: &str) -> Option<&Value>` scanning `catalog.groups[].controls[]` for controls whose title/parts prose contains the needle. Then: `let mfa = find_control_by_text(&catalog, "must enforce MFA").expect(...); let notify = find_control_by_text(&catalog, "should notify administrators").expect(...);` and assert each control's own modality prop equals "normative"/"advisory" respectively (read `props` where `name == "modality"`). Keep the `>= 5` count as a secondary guard.

══════ F0874 │ tests/oscal_cli_round_trip.rs:64-75, 131-173 │ [bug · high] — VALID ══════
**Symbols:** `reclassify` (64-75), `validate_divergence_log` (131-173); producer `compare_oscal_json` (src/round_trip/comparator.rs); types `DivergenceClass`/`ResolutionStatus` (src/round_trip/divergence.rs:26-45).
**Root cause:** Contract contradiction. `reclassify` assigns a resolution ONLY for `DivergenceClass::Acceptable` (`Some(ResolutionStatus::Accepted)`). `validate_divergence_log` asserts every divergence classified `ForgeFix` or `OscalCliDiff` has a NON-null resolution (lines 164-169). But `compare_oscal_json` constructs every divergence with `resolution: None` (src/round_trip/comparator.rs:100, 199, 222, 233, 265, 281), and nothing else fills it. Consequence: exactly when the suite detects a FORGE regression — the SC-001/SC-002 purpose — the run dies inside `validate_divergence_log` with the misleading "must have a non-null resolution" message BEFORE the SC-001 `forge_fix_count == 0` assertion (line ~200), and the OscalCliDiff/ReportedUpstream path can never be exercised end-to-end. The suite is gated on oscal-cli availability (`skip_if_no_oscal_cli`, line ~37), which is why this hasn't fired in CI.
**Evidence:** reclassify body confirmed at 64-75; the non-null assertion confirmed at 164-169; `resolution: None` at all six comparator construction sites (grep).
**Remediation:** Extend `reclassify` to satisfy the T019 contract for all classes: match on `d.classification` and assign `ResolutionStatus::Fixed` for `ForgeFix` and `ResolutionStatus::ReportedUpstream` for `OscalCliDiff` when `resolution.is_none()` (both variants already exist in src/round_trip/divergence.rs:37-45). This keeps `validate_divergence_log` strict. If the team instead wants unresolved ForgeFix divergences to fail loudly, relax the log assertion to field-presence only — but option (a) is the contract-aligned fix. No snapshot impact (round_trip log snapshots use pre-filled resolutions already).

### Medium severity — repo config & CI

══════ F0947 │ .cargo/config.toml:1-2 │ [performance · medium] — VALID ══════
**Symbol:** `[net] retry = 10`.
**Root cause:** File is exactly `[net]` + `retry = 10`. Cargo's default is 3; raising to 10 makes every index/download/git operation attempt ~11 requests with backoff before failing. For non-transient causes (registry outage, expired token, bad mirror) builds stall for minutes instead of failing fast, hurting time-bounded CI and masking systemic problems (auth failures retried uselessly 10×).
**Evidence:** file contents confirmed (2 lines).
**Remediation:** Set `retry = 3` (default) with a comment; document `CARGO_NET_RETRY` env override for flaky-network CI jobs.

══════ F1073 │ .gitattributes:16-25 │ [maintainability · medium] — VALID ══════
**Symbols:** pinned-asset block lines 16-25.
**Root cause:** `-whitespace` is not an attribute git honors for line-ending/normalization purposes (built-in behavior is governed by `text`/`eol`/`binary`; whitespace checking is `core.whitespace`/`git diff --check` configuration, not a path attribute). A repo-wide search finds no hook/tooling consuming it, so `-whitespace` here is a silent no-op creating a false impression of enforcement; `-text` alone already fully disables eol conversion. Additionally the block MUST stay after `*.json text eol=lf` (line 9) because gitattributes is last-match-wins — an undocumented coupling.
**Evidence:** .gitattributes lines 1-25 confirmed; no consumer of a `whitespace` attribute in .github/, scripts/, ci/, src/.
**Remediation:** Drop `-whitespace` from the pinned lines, keep `-text`, and add a comment warning that the block must remain after `*.json text eol=lf` (last-match-wins) or the pinned assets get re-normalized.

══════ F1072 │ .gitattributes:18-25 │ [bug · medium] — VALID ══════
**Symbols:** same pinned-asset block.
**Root cause:** Git treats patterns containing `/` as pathname globs, so `tests/fixtures/schemas/*` matches ONLY immediate children. Any schema later nested in a subfolder — or a new manifest-pinned asset placed outside the enumerated paths — falls through to `* text=auto` (line 2) and gets eol-normalized, silently breaking byte-for-byte provenance (`tests/schema_provenance_test.rs` pins sizes + SHA-256 for all 10 assets in schemas/oscal-schema-manifest.json). Today it happens to work because the manifest's 10 assets (4 runtime JSON in schemas/, 2 test JSON + 4 XSDs in tests/fixtures/) are all flat and individually enumerated.
**Evidence:** manifest assets verified (schemas/oscal-schema-manifest.json); gitattributes patterns verified; provenance test reads each asset and checks size+sha256 (tests/schema_provenance_test.rs:101-105).
**Remediation:** Widen to recursive forms (`tests/fixtures/schemas/** -text`, `tests/fixtures/xsd/*.xsd -text`, `schemas/oscal_*.json -text`) and keep the block aligned with the manifest; distinct from F1073 (pattern coverage vs no-op attribute) — fix both in one edit.

══════ F1052 │ .github/dependabot.yml:6-11 │ [maintainability · medium] — VALID ══════
**Symbol:** `updates:` list.
**Root cause:** Only `package-ecosystem: "cargo"` is configured, but the repo has two workflows with SHA-pinned actions (.github/workflows/ci.yml, release.yml). Pinned action versions never receive automated update PRs, so pins drift until deprecated-node warnings or breakage; manual SHA+comment updates are exactly the error-prone flow F1063 warns about.
**Evidence:** dependabot.yml confirmed (single cargo entry); workflows exist with `uses:` pins.
**Remediation:** Add a second `updates` entry: `package-ecosystem: "github-actions"`, `directory: "/"`, weekly schedule. Dependabot updates SHA pins and version comments together.

══════ F1063 │ .github/workflows/ci.yml:24-27 (and release.yml mirrors) │ [security · medium] — VALID ══════
**Symbols:** `actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd  # v5`, `dtolnay/rust-toolchain@631a55b12751854ce901bb631d5902ceb48146f7 # stable`, `actions/cache@668228422ae6a00e4ad889ee87cd7109ec5666a7  # v5`.
**Root cause:** Hard-coded SHAs with hand-written version comments cannot be verified from the file alone — a hijacked or typo'd SHA looks identical and would precisely defeat the supply-chain checks this workflow runs. Notably, upstream lookup of the checkout SHA `93cb6efe…` associates it with v6-era commits, while the comment claims `# v5` — illustrating that comment/SHA drift already happens. `dtolnay/rust-toolchain` is pinned to a frozen master-branch SHA, which can stop working when GitHub runner images rotate bundled toolchains.
**Evidence:** ci.yml:24,27,32 confirmed; release.yml uses the same pins (lines 25,28,33,80,84,119,122).
**Remediation:** Verify each SHA against its annotated tag upstream (`git ls-remote`); correct stale comments; add automation (the F1052 github-actions Dependabot entry updates SHA+comment atomically, or an actionlint/dependency-review job). For dtolnay/rust-toolchain consider tag-pinning (`@stable` is upstream-recommended since it tracks) or scheduled re-pin.

══════ F1065 │ .github/workflows/ci.yml:31-39, 62-80 │ [performance · medium] — VALID ══════
**Symbols:** `Cache cargo registry and build` step (31-39), `cargo install cargo-audit|cargo-deny|cargo-vet --locked` steps (62-64, 70-72, 78-80).
**Root cause:** The three audit tools are compiled from source on every Ubuntu run — the cache path list (`~/.cargo/registry`, `~/.cargo/git`, `target`) does NOT include `~/.cargo/bin`, and even if it did, the cache key `runner.os-cargo-hashFiles('**/Cargo.lock')` is invalidated by ANY dependency bump, wiping tool binaries and forcing 5-15+ min of recompilation exactly when unrelated changes land.
**Evidence:** cache `path:` block confirmed (lines 34-37) with no `~/.cargo/bin`; install steps confirmed.
**Remediation:** Prefer prebuilt binaries via `taiki-e/install-action` (SHA-pinned) for cargo-audit/cargo-deny/cargo-vet, or split audit tooling into its own job with a cache keyed on tool versions instead of Cargo.lock.

══════ F1057 │ .github/workflows/release.yml (all 5 jobs), .github/workflows/ci.yml (test job) │ [other · medium] — VALID ══════
**Symbols:** jobs `test`, `build`, `sbom`, `hash`, `provenance`, `release` (release.yml); job `test` (ci.yml).
**Root cause:** No job in either workflow sets `timeout-minutes` (grep confirmed zero occurrences). A stalled cargo build/test/bench/`cargo install` hangs until GitHub's 6-hour global limit, multiplied across the 3-OS and 4-target matrices.
**Evidence:** grep for `timeout-minutes` in both workflows: no matches.
**Remediation:** Add bounded `timeout-minutes` per job (e.g. 60 test, 45 build, 30 sbom/hash, 15 provenance, 30 release).

══════ F1055 │ .github/workflows/release.yml:161-173 │ [security · medium] — VALID ══════
**Symbol:** job `provenance` permissions block (164-167).
**Root cause:** The SLSA generator invocation sets `contents: write` although `upload-assets: false` (line 173) means it only publishes the attestation as a workflow artifact — it needs `actions: read` + `id-token: write` only. The extra write scope widens blast radius if the referenced reusable workflow (tag-pinned `@v2.1.0`, line 170) is ever compromised.
**Evidence:** permissions block and `upload-assets: false` confirmed.
**Remediation:** Delete `contents: write` from the `provenance` job's permissions (the `release` job legitimately keeps its own `contents: write` at lines 179-180).

══════ F1058 │ .github/workflows/release.yml:1-13, .github/workflows/ci.yml:1-13 │ [other · medium] — VALID ══════
**Symbols:** `on:` triggers of both workflows.
**Root cause:** Neither workflow defines `concurrency` (grep confirmed). release.yml: pushing/re-pushing several `v*` tags fans out full 9-runner pipelines in parallel, risking duplicate/out-of-order releases. ci.yml: push-to-main + pull_request overlap means each PR commit starts a fresh run beside the previous one, amplified by three source-compiled `cargo install`s per Ubuntu leg.
**Evidence:** no `concurrency:` key in either file.
**Remediation:** Add `concurrency: { group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true }` to both (per-tag grouping naturally serializes releases; per-PR grouping cancels superseded CI runs; consider `cancel-in-progress: false` for release.yml if in-flight releases should complete).

══════ F1059 │ .github/workflows/release.yml:51-88 │ [performance · medium] — VALID ══════
**Symbol:** job `build` steps.
**Root cause:** Unlike the `test` job (cache at release.yml:32-40), the `build` job has NO actions/cache step, so every tag rebuilds all dependencies from scratch across 4 targets × 3 OSes.
**Evidence:** build job steps confirmed (checkout → rust-toolchain → cargo build); no cache step.
**Remediation:** Add actions/cache keyed `${{ runner.os }}-${{ matrix.target }}-cargo-${{ hashFiles('**/Cargo.lock') }}` with restore-keys prefix, paths `~/.cargo/registry`, `~/.cargo/git`, `target`, placed before the build step.

══════ F0995 │ .rustfmt.toml:1 │ [maintainability · medium] — VALID ══════
**Symbol:** `edition = "2024"`.
**Root cause:** `edition` in rustfmt config is only recognized by rustfmt ≥ 1.8.0 (Rust 1.85+). The repo has no rust-toolchain(.toml) pin (glob confirmed absent) and Cargo.toml declares no `rust-version`, so dev machines/editors on older stable toolchains hard-fail every `cargo fmt` invocation with an invalid-option error instead of formatting. CI is fine (dtolnay/rust-toolchain@stable), but the gate's consistency across environments is unanchored.
**Evidence:** .rustfmt.toml = 3 lines (`edition = "2024"`, `max_width = 100`, `use_small_heuristics = "Max"`); no rust-toolchain* file at repo root.
**Remediation:** Add `rust-toolchain.toml` pinning stable ≥ 1.85 (channel = "stable") and/or `rust-version = "1.85"` in Cargo.toml [package]; comment .rustfmt.toml noting the toolchain floor.

### Medium severity — benches

══════ F0041 │ benches/atomize.rs:10-23 │ [maintainability · medium] — VALID ══════
**Symbol:** `fn make_req` (benches/atomize.rs:11-23) vs tests/common/mod.rs:86-99.
**Root cause:** Hand-copied fixture constructor, verified field-for-field identical to tests/common/mod.rs::make_req today. PolicyRequirement has grown optional fields (citations, modality, parameters); the next added field either breaks every construction site or — worse — gets a different default here, making benchmarks measure an input shape the real pipeline never produces. Benches cannot import the tests/ module, but files in benches/ subdirectories are not compiled as bench targets, so a shared helper module works.
**Evidence:** both bodies compared; identical field set. Cargo.toml declares six [[bench]] targets, none named `common`.
**Remediation:** Create benches/common/mod.rs with the shared `make_req`/`make_section`/`make_doc` (copied once from tests/common/mod.rs) and `mod common; use common::*;` in each bench that needs them; delete the local copy in benches/atomize.rs. Also dedupe the `MAX_SIZE_BYTES` constant duplicated in benches/xml_benchmark.rs:24-25 and benches/export_bench.rs:22-23 into the same module.

══════ F0042 │ benches/atomize.rs:40-48 │ [performance · medium] — VALID ══════
**Symbol:** `bench_atomize_document_100`.
**Root cause:** The "mixed" 100-requirement workload is only two distinct texts repeated 50× each, all with zero-valued metadata in one flat section (no children, body_text, citations, or parameters). Repetition maximizes warm-cache effects on identical slices and never exercises hierarchy traversal or metadata-dependent branches; results won't transfer to realistic policies.
**Evidence:** lines 40-48 confirmed (two loops, two literals).
**Remediation:** Generate 100 distinct texts (e.g. format! with the index), vary `nesting_depth` (i % 3), and give some requirements citations/parameters so the benchmark covers the branches atomize actually has.

══════ F0032 │ benches/export_bench.rs:87, ~100 │ [test · medium] — VALID ══════
**Symbol:** `bench_export_pipeline` bench ids.
**Root cause:** Both bench ids embed the runtime-derived fixture size (`format!("json_to_xml_{json_size_kb}kb")` and the yaml twin). Any fixture edit that shifts the byte count forks a new Criterion time series, orphaning stored baselines and breaking `--baseline` cross-commit comparisons.
**Evidence:** both `bench_function(format!(...))` calls confirmed.
**Remediation:** Use stable ids (`catalog_json_to_xml`, `catalog_json_to_yaml`); log the size separately (eprintln or tracing) outside the measured region.

══════ F0030 │ benches/export_bench.rs:87-112 │ [performance · medium] — VALID ══════
**Symbol:** `bench_export_pipeline` iterations.
**Root cause:** Each iteration runs `export_artifact(input, format, Some(output_path))`, so the measured region includes a full filesystem write per iteration (plus the temp-dir file). Measured time is contaminated by page-cache-dependent disk I/O variance, undermining the stated `< 1s` SC-005 comparison — unless the benchmark is deliberately end-to-end, which isn't documented.
**Evidence:** `Some(black_box(&xml_output))` / `Some(black_box(&yaml_output))` confirmed inside both `b.iter` closures.
**Remediation:** Either call `export_artifact(..., None)` (stdout/serialize-only path) to measure the transform, or document in the file header that SC-005 is measured end-to-end including file I/O.

══════ F0026 │ benches/parameter_extraction.rs:104-126 │ [other · medium] — VALID ══════
**Symbol:** `bench_extract_parameters_500/_100/_single`.
**Root cause:** All three benches clone the 500/100/1-requirement document INSIDE `b.iter` (`let mut d = black_box(doc.clone());`), so clone cost pollutes the measurement; and `extract_parameters` returns `Result` (src/parameter/mod.rs:142 — it genuinely can return `Err(ForgeError::ParameterExtraction)`, see lines 212-216), so an `Err` panics the benchmark instead of surfacing which input class failed, silently narrowing the validated input set for the p95 ≤ 1s claim.
**Evidence:** pattern confirmed in all three benches; error path confirmed in src/parameter/mod.rs.
**Remediation:** Switch to `b.iter_batched(|| doc.clone(), |mut d| black_box(extract_parameters(&mut d)).expect("..."), BatchSize::SmallInput)`, and either document why Err is unreachable for this corpus (valid UTF-8) or count/measure the error path deliberately.

══════ F0052 │ benches/pipeline_benchmark.rs:37-59 │ [performance · medium] — VALID ══════
**Symbol:** `run_full_catalog_pipeline` (bench helper).
**Root cause:** Production serializes through `validate_and_serialize()` (src/pipeline.rs:40-63), which converts the envelope to a serde_json::Value and runs full OSCAL schema + semantic validation before returning the pretty-printed JSON. The bench's Step 5 goes straight to `serde_json::to_string_pretty`, so the full-pipeline number understates production by the entire validation phase (schema compilation is cached, but per-document validation is not free at 500KB).
**Evidence:** bench Step 5 confirmed at lines 56-58; production validate_and_serialize confirmed at src/pipeline.rs:46-62 calling `crate::validate::run_full_validation`.
**Remediation:** Include the same validation in the bench helper (call `forge::validate::run_full_validation` on the serialized value), or state explicitly in the doc comment that the benchmark excludes validation so readers don't treat it as end-to-end wall time.

══════ F0053 │ benches/pipeline_benchmark.rs:70-74, 150-175 │ [test · medium] — VALID ══════
**Symbol:** `bench_full_pipeline` and `bench_per_stage` pre-computation.
**Root cause:** The fixture-existence assertions (lines 71-74, 152) verify presence but not that parsing yields content. If the fixture is regenerated with a different heading style or parse rules change so `extract_sections` returns zero sections, every downstream stage benchmark still runs green — measuring near-trivial work on an empty document and producing plausible-looking worthless numbers.
**Evidence:** no `!sections.is_empty()` assertion anywhere in the file.
**Remediation:** After pre-computation in `bench_per_stage`, add `assert!(!sections.is_empty(), "fixture produced no sections");` and `assert!(sections.iter().map(|s| s.requirements.len()).sum::<usize>() > 0, ...)`; likewise assert the atomized doc has requirements in `bench_full_pipeline`'s setup.

══════ F0021 │ benches/uuid_benchmark.rs:4-24 │ [test · medium] — VALID ══════
**Symbol:** `bench_normalize_for_hashing` vs `bench_generate_stable_id(_long)`.
**Root cause:** `bench_normalize_for_hashing` measures whitespace-padded text while both `generate_stable_id` benches measure cleanly spaced text. The reported numbers aren't comparable across benchmarks — you can't attribute generate_stable_id's cost to the normalization step it performs internally.
**Evidence:** lines 5 vs 12 vs 19 inputs confirmed.
**Remediation:** Define shared `PADDED_SAMPLE`/`CLEAN_SAMPLE` consts and benchmark each helper on the same sample set, or add a comment stating the split is deliberate worst-case isolation.

══════ F0101 │ benches/xml_benchmark.rs:27-83 │ [maintainability · medium] — VALID ══════
**Symbols:** `build_catalog_from_fixture` (27-63), `build_component_def_from_fixture` (66-83).
**Root cause:** The ingest→parse→assemble→atomize→assign_ids→cite stage chain is duplicated verbatim between the two builders (8 chained `.unwrap()`s each), giving no indication which stage failed if the fixture breaks the pipeline. `forge::pipeline::prepare_document` is `pub(crate)` (src/pipeline.rs:76), so benches can't reuse it.
**Evidence:** both function bodies confirmed; `pub(crate) fn prepare_document` confirmed.
**Remediation:** Extract one local `fn prepared_document(fixture_path: &Path) -> forge::model::PolicyDocument` consumed by both builders, with `.expect("<stage> failed")` on each step (ingest, extract_sections, extract_clauses, assemble_document, atomize_document, extract_citations).

══════ F0103 │ benches/xml_benchmark.rs:85-90 │ [test · medium] — VALID ══════
**Symbol:** `bench_xml_serialization` fixture guard.
**Root cause:** If the fixture is missing, `tracing::warn!` + early `return` — but criterion harnesses install no tracing subscriber, so the message is invisible and `cargo bench` exits green with ZERO measurements. A renamed/deleted fixture or wrong cwd would go unnoticed in CI (which runs `cargo bench --bench pipeline_benchmark` only, so xml/export bench regressions are even less visible). FIXTURE_PATH is committed, so absence is always a defect.
**Evidence:** lines 85-90 confirmed; fixture tests/fixtures/synthetic-50page-policy.md is committed. The identical pattern exists at benches/export_bench.rs:74-77.
**Remediation:** Replace with `panic!("required fixture missing: {FIXTURE_PATH}")` (or assert!) in both benches/xml_benchmark.rs:85-90 AND benches/export_bench.rs:74-77.

### Medium severity — ci/integration-test.sh

══════ F0914 │ ci/integration-test.sh — manual counter blocks at 177-178, 189-190, 201-202, 238-241, 246-249, 360-363, 388-391 │ [maintainability · medium] — VALID ══════
**Symbols:** `check`/`check_skip` (108-130) vs hand-maintained `TOTAL=$((TOTAL + N)); SKIPPED=$((SKIPPED + N)); skip ...` blocks.
**Root cause:** Every conditional branch re-implements the bookkeeping `check`/`check_skip` already encapsulate, with hand-maintained +1/+2 combinations across seven blocks. Any drift between a skip/fail printout and the arithmetic silently corrupts the final tally (add a third skip line, forget `TOTAL=$((TOTAL+2))` → stale constant).
**Evidence:** `check`/`check_skip` definitions confirmed at lines 108-130; manual blocks confirmed at all seven locations.
**Remediation:** Add `record_fail() { TOTAL=$((TOTAL + 1)); FAILED=$((FAILED + 1)); fail "$*"; }` and `record_skip() { TOTAL=$((TOTAL + 1)); SKIPPED=$((SKIPPED + 1)); skip "$*"; }` next to check/check_skip, and replace every manual block (each `skip` line + its counter pair) with single `record_skip`/`record_fail` calls.

══════ F0912 │ ci/integration-test.sh:294-303, 308-320 │ [bug · medium] — VALID ══════
**Symbols:** Step 2e/2f python3 structural checks.
**Root cause:** `python3` is never feature-detected (unlike xmllint at lines 80-83). On hosts without it, the checks return 127 (command-not-found) and `check` counts them FAILED, hard-failing the run — contradicting the header's stated policy that missing extras degrade gracefully (line 18). Additionally `${PROFILE_JSON}`/`${AP_JSON}` are interpolated directly into `python3 -c` source — fragile quoting/injection pattern.
**Evidence:** xmllint detection at 80-83 confirmed; no python3 detection anywhere (grep); interpolation confirmed at lines 297, 312.
**Remediation:** Add `HAS_PYTHON3=false; command -v python3 >/dev/null 2>&1 && HAS_PYTHON3=true` beside the xmllint check; route both structural checks through `check_skip` when absent; pass the file path as an argv argument (`python3 -c '...sys.argv[1]...' "${PROFILE_JSON}"`) instead of interpolating it into the code string.

══════ F0913 │ ci/integration-test.sh:39-60 │ [performance · medium] — VALID ══════
**Symbol:** FORGE resolution block.
**Root cause:** `FORGE` is deliberately unquoted so the multi-word `cargo run --quiet --release --` fallback expands as separate words — but that breaks the documented usage `./ci/integration-test.sh [FORGE_BIN]` whenever the binary path contains whitespace, and forces every one of the ~20 forge invocations through cargo (dependency/rebuild re-check each time), dominating wall time.
**Evidence:** lines 39-48 confirmed; ~20 unquoted `${FORGE}` call sites confirmed by grep (lines 54, 60, 65, 145, 150, 168, 175, 186, 198, 210, 216, 260, 264, 321-323, 343, 352, 377, 384).
**Remediation:** Resolve to a plain executable path once: use `$1` if given, else target/release/forge, else target/debug/forge, else `cargo build --quiet --release` up front and use target/release/forge. Then quote all expansions (`"${FORGE}"`).

══════ F0915 │ ci/integration-test.sh:404-411 │ [test · medium] — VALID ══════
**Symbol:** Summary gate.
**Root cause:** The summary gates solely on `FAILED > 0`. A degraded environment (old forge without diff/trace/import-ssp, missing xmllint/python3, absent schema fixtures) can reduce nearly every check to SKIP and still exit 0 — green-lighting CI while validating essentially nothing, including the degenerate PASSED == 0 case.
**Evidence:** final gate confirmed at lines 404-411 (`if [[ ${FAILED} -gt 0 ]]; then ... exit 1; else PASSED; exit 0`).
**Remediation:** Add `elif [[ ${PASSED} -eq 0 ]]; then echo "Integration test INCONCLUSIVE: every check skipped"; exit 1;` (optionally a higher PASSED floor for the core catalog/profile/component generation checks).

### Medium severity — deny.toml / supply-chain policy

══════ F1034 │ deny.toml:20-22 │ [security · medium] — VALID ══════
**Symbol:** `[bans]` section.
**Root cause:** `wildcards = "allow"` fully disables the diagnostic for `*`/loose version requirements anywhere in the tree, so an accidental `*` pin (silently tracking breaking majors and unreviewed transitive upgrades) is never surfaced. Combined with `multiple-versions = "warn"` (never hard-fails), duplicate-version drift is accepted by default.
**Evidence:** deny.toml lines 20-22 confirmed.
**Remediation:** Set `wildcards = "deny"` (with targeted `[bans.skip]` for any unavoidable wildcard) or at least `"warn"` paired with `[bans.highlight]`.

══════ F1035 │ deny.toml:24-28 │ [security · medium] — VALID ══════
**Symbol:** `[sources]` section.
**Root cause:** `unknown-registry = "warn"` and `unknown-git = "warn"` while the allow lists declare a crates.io-only policy. At warn level cargo-deny exits 0 (only Error-level diagnostics fail the run), so a dependency redirected to a git fork or third-party registry passes CI silently — the declared source policy is decorative, leaving typo-squatting-style redirection room.
**Evidence:** deny.toml lines 24-28 confirmed.
**Remediation:** Set both to `"deny"` so disallowed sources actually fail `cargo deny check`.

══════ F1032 │ deny.toml:3-7 │ [maintainability · medium] — VALID ══════
**Symbol:** `[advisories] ignore` entry `"RUSTSEC-2026-0192"`.
**Root cause:** The suppression rationale matches reality (ttf-parser 0.25.x pulled transitively by lopdf 0.42 via pdf-extract; exemptions for all three exist in supply-chain/config.toml), so the ignore itself is justified. But an ignore keyed only by RUSTSEC ID is permanent: it stays active after lopdf updates or the advisory is superseded/withdrawn, and nothing in CI ever flags a stale ignore.
**Evidence:** ignore list confirmed at lines 3-7 with the "Revisit when lopdf drops it" comment.
**Remediation:** Add a concrete dated review trigger (REVIEW-BY date + owning issue) in the comment, and migrate to cargo-deny's inline-table form `{ id = "RUSTSEC-2026-0192", reason = "..." }` so the justification is machine-readable.

### Medium severity — examples/ (generator-rooted defects)

Per the batch contract, defects in examples/ outputs are redirected to their GENERATOR: ssp.json is produced by examples/component-based/generate_ssp.py (README Step 4), catalog-new.json by `forge convert` (README Step 1). Fixing the generator and regenerating is the remediation; hand-editing generated JSON is not.

══════ F0016 │ examples/component-based/generate_ssp.py:14-18 │ [bug · medium] — VALID ══════
**Symbol:** control-ID collection loop.
**Root cause:** The loop appends every `control-id` from every component's control-implementations with no dedup. In a component-based architecture the same control frequently appears under several components; the later `for cid in control_ids` loop then mints the SAME uuid5 seed (`stable_uuid(f"ir-{cid.lower()}")`) per occurrence, producing repeated implemented-requirements entries with non-unique UUIDs and duplicated statements in ssp.json. Today the example's component-definition.json happens to carry exactly ONE component (type "policy"), so no collision manifests yet — but the generator is advertised as the component-based pattern and will silently corrupt the first multi-component input.
**Evidence:** loop confirmed at lines 14-18; single-component input confirmed in output/component-definition.json (one component, 25 control-ids).
**Remediation:** Deduplicate preserving order: `seen = set(); for ir in ci.get("implemented-requirements", []): cid = ir["control-id"]; if cid not in seen: seen.add(cid); control_ids.append(cid)`. Regenerate output/ssp.json.

══════ F0018 │ examples/component-based/generate_ssp.py:59-62, 69-72 │ [bug · medium] — VALID ══════
**Symbols:** `responsible-roles` literals for both components.
**Root cause:** The generator emits `{"role-id": ..., "responsible-party": "<free text>"}` entries. OSCAL 1.1.x (the declared oscal-version, line 48) defines responsible-role as `role-id` + `party-uuids` referencing parties in metadata.parties — `responsible-party` is not a metaschema key, and the document defines neither metadata.roles for these role-ids nor any parties. Any conformant validator rejects these entries. This is the generator root cause of the output-level findings F0001 (ssp.json responsible-party keys) and, together with the missing roles/parties blocks (lines 38-49 define neither), of F0002.
**Evidence:** generator literals confirmed; ssp.json metadata has no `roles` and no `parties` keys (grep confirmed absent).
**Remediation:** In generate_ssp.py: (1) add `metadata.roles` declaring system-admin, security-officer, database-admin, analyst, service-account; (2) add `metadata.parties` (e.g. team parties with uuid5 ids); (3) replace `responsible-party` with `"party-uuids": [<party uuid>]`; regenerate ssp.json. F0001/F0002 resolve with this single generator fix.

══════ F0008 │ examples/component-based/output/catalog-new.json:428-429 │ [bug · medium] — VALID ══════
**Symbol (locus redirected):** `derive_control_title` — src/oscal/catalog.rs:283-297.
**Root cause:** Output artifact shows title "The application component SHALL require TLS 1." while the control's own statement prose reads "TLS 1.3 for all external communications". Root cause is in forge, not the example: `derive_control_title` takes the first sentence via `requirement_text.find(['.', '!', '?'])`, treating the decimal point in "TLS 1.3" as a sentence terminator and dropping ".3 …". The source bullet (examples/component-based/policy.md:36) has no trailing period, so a correct implementation would keep the full text. This affects ANY policy line containing a decimal mid-sentence (e.g. example_data/POL-09 line 81 "TLS 1.2", POL-18 lines 61-64 CVSS ranges).
**Evidence:** src/oscal/catalog.rs:284-286 confirmed (`find(['.', '!', '?'])`); mismatch between title and `_smt` prose confirmed in catalog-new.json.
**Remediation:** In `derive_control_title`, only treat a terminator as sentence-ending when it is NOT part of a decimal number: skip `.` positions whose next char is an ASCII digit (and whose previous char is a digit). Add unit tests next to the existing ones (src/oscal/catalog.rs:695-723): "require TLS 1.3 for all communications" (no trailing period) → full text; "Version 2.0 released. More." → "Version 2.0 released.". Then regenerate examples/component-based/output/catalog-new.json and catalog.json via the README Step 1 command; check snapshot impact on any title-bearing snapshots (`cargo insta review`).

══════ F0004 │ examples/component-based/output/ssp.json:114-119 │ [bug · medium] — VALID ══════
**Symbol (locus redirected):** generate_ssp.py implemented-requirements builder, lines 26-35 (hardcoded `"href": "#component-web-application"`).
**Root cause:** Every implemented requirement links `href: "#component-web-application"`, but OSCAL fragment references must target a locally-defined UUID; the Web Application component's uuid is `stable_uuid("web-application")` (a v5 UUID, shown as `895e13fb-…` in ssp.json), and no element carries the literal id "component-web-application". All 25 `implements` associations are therefore dangling.
**Evidence:** generator literal confirmed at line ~33; ssp.json component uuids confirmed as UUIDs; no node matches the fragment.
**Remediation:** In generate_ssp.py compute `WEB_APP_UUID = stable_uuid("web-application")` once and emit `{"href": f"#{WEB_APP_UUID}", "rel": "implements"}`; regenerate ssp.json.

══════ F0002 │ examples/component-based/output/ssp.json:75-77 │ [bug · medium] — VALID ══════
**Symbol (locus redirected):** generate_ssp.py SSP skeleton, lines 38-49 (metadata) and 84-110 (users with `role-ids`).
**Root cause:** The role ids used throughout (`system-admin`, `security-officer`, `database-admin`, `analyst`, `service-account`) are never declared: ssp.json metadata has no `roles` block, so neither the responsible-role entries nor the user `role-ids` resolve — the document fails OSCAL 1.1.3 validation despite claiming it.
**Evidence:** users' role-ids confirmed at ssp.json:75-77; metadata lacks `roles` (generator emits only title/last-modified/version/oscal-version, lines 42-48).
**Remediation:** Same generator fix as F0018: add a `metadata.roles` array declaring all five role ids with titles, then regenerate. (F0018 is the primary; implement both roles and party-uuids in one pass.)

### Medium severity — scripts/

══════ F0893 │ scripts/ci-local.sh:20-21 │ [performance · medium] — VALID ══════
**Symbol:** bench `run_step`.
**Root cause:** `cargo bench --bench pipeline_benchmark …` runs on EVERY local pass, inflating wall time and letting benchmark variance/flakiness block trivially correct changes.
**Evidence:** scripts/ci-local.sh lines 20-21 confirmed (mandatory, un-gated).
**Remediation:** Gate on `CI_LOCAL_BENCH=1` (run when set, else print "skipping benchmarks (set CI_LOCAL_BENCH=1)"). Keep it mandatory in scripts/ci-local.sh's documented CI-mirror role only if the team wants exact parity — the finding recommends opt-in.

══════ F0894 │ scripts/ci-local.sh:22-24 │ [maintainability · medium] — VALID ══════
**Symbols:** audit/deny/vet `run_step`s; also scripts/pre-commit.sh:26-31 strict block.
**Root cause:** Both scripts hard-assume cargo-audit/cargo-deny/cargo-vet are installed (and, for audit, that the RustSec DB is reachable). Under `set -euo pipefail` a fresh or air-gapped machine dies steps-deep with a bare "no such command" — and the hardcoded `pipeline_benchmark` name yields a confusing unknown-target error if benches change.
**Evidence:** ci-local.sh lines 22-24 and pre-commit.sh lines 26-31 confirmed; both start with `set -euo pipefail`; no `command -v` pre-flight anywhere.
**Remediation:** Pre-flight once: `for sub in audit deny vet; do cargo "$sub" --version >/dev/null 2>&1 || { echo "[ci-local] 'cargo $sub' not installed; cargo install cargo-$sub --locked" >&2; exit 1; }; done` (in pre-commit.sh only when FORGE_PRECOMMIT_STRICT=1), with actionable hints.

══════ F0890 │ scripts/ci-local.sh:5-6, scripts/pre-commit.sh:5-6 │ [bug · medium] — VALID ══════
**Symbol:** `SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"` + `REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"`.
**Root cause:** When launched through a symlink (e.g. ~/.local/bin/ci-local, or .git/hooks/pre-commit → scripts/pre-commit.sh), `dirname ${BASH_SOURCE[0]}` points at the LINK's directory, so the gates run against the wrong project tree. Note: the hook installed by scripts/install-hooks.sh resolves via `git rev-parse --show-toplevel` at runtime (install-hooks.sh:19), which masks this for the default hook path — but ci-local.sh symlink usage is directly broken.
**Evidence:** identical two lines confirmed in both scripts.
**Remediation:** Resolve the real path: `SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")" && pwd)"` (macOS 12.3+/Linux have readlink -f; otherwise a POSIX fallback), or prefer `REPO_ROOT="$(git rev-parse --show-toplevel)"` where a git context is guaranteed.

══════ F0895 │ scripts/install-hooks.sh:7-13 │ [bug · medium] — VALID ══════
**Symbol:** `HOOKS_DIR="${REPO_ROOT}/.git/hooks"`.
**Root cause:** Hardcoding `.git/hooks` breaks for repos with `core.hooksPath` configured (husky, checked-in hooks dirs), linked worktrees (`.git` is a file, so the `[ ! -d ]` guard at line 10 exits early and other worktrees silently keep old behavior), and non-default GIT_DIR layouts. Git ignores a hook installed in the wrong place, so the required pre-commit validations silently stop running.
**Evidence:** lines 7-13 confirmed.
**Remediation:** `HOOKS_DIR="$(git -C "${REPO_ROOT}" rev-parse --git-path hooks)" || { echo "not inside a git repository" >&2; exit 1; }` — honors core.hooksPath, worktrees, and GIT_DIR.

══════ F0905 │ scripts/pre-commit.sh:22-24 │ [performance · medium] — VALID ══════
**Symbols:** `cargo clippy -- -D warnings` and `cargo test` run_steps.
**Root cause:** Full workspace lint + full test suite on every commit makes the hook slow, pushing developers toward SKIP_FORGE_PRECOMMIT and undermining the gate. Additionally, without `--locked`, cargo may opportunistically update Cargo.lock mid-commit, dirtying a tracked file as a side effect of the hook.
**Evidence:** lines 22-24 confirmed (no `--locked`).
**Remediation:** Add `--locked` to both (`cargo clippy --locked -- -D warnings`, `cargo test --locked`); consider trimming to `cargo test --lib` in the hook with the full suite left to CI (optional second half).

══════ F0904 │ scripts/pre-commit.sh:8-11 │ [security · medium] — VALID ══════
**Symbol:** SKIP_FORGE_PRECOMMIT gate.
**Root cause:** One environment variable silently disables EVERY quality gate (fmt, clippy, test, strict audit/deny). Env vars are inherited ad hoc, so an IDE terminal, wrapper, or shared CI image exporting SKIP_FORGE_PRECOMMIT=1 skips all checks with almost no trace (the notice goes to stdout only). The comparison accepts only literal "1" (so "true"/"yes" do NOT skip — surprising but safer; worth documenting).
**Evidence:** lines 8-11 confirmed (`[[ "${SKIP_FORGE_PRECOMMIT:-0}" == "1" ]]` → echo + exit 0).
**Remediation:** Emit the notice to stderr; refuse the override when `CI` is set (exit 1 with a message); document accepted values; optionally add finer-grained skippers so audit/deny can't be dismissed wholesale.

### Medium severity — src/applicability & src/batch

══════ F0290 │ src/applicability/model.rs:108-117 (ReviewQueueItem), 239-253 (review_queue_item) │ [maintainability · medium] — VALID ══════
**Symbols:** `pub reason_code: &'static str`, producer `review_queue_item`.
**Root cause:** `reason_code` is a bare `&'static str` emitted into serialized, machine-readable review-queue output, yet nothing compiler-checks the vocabulary: a typo or future edit in the four match arms (lines 241-245: "reviewed-no-positive-relationship", "no-reviewed-mapping", "deferred-scope-decision", "scope-decision-required") compiles cleanly and silently changes the contract downstream parsers rely on. The sibling `GapClassification` (model.rs:119-131) avoids exactly this via a typed kebab-case-serde enum with a checked `as_str()`.
**Evidence:** struct field at line 112; match arms at 241-245; GapClassification pattern at 120-131.
**Remediation:** Add `#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)] #[serde(rename_all = "kebab-case")] pub enum ReviewReason { ReviewedNoPositiveRelationship, NoReviewedMapping, DeferredScopeDecision, ScopeDecisionRequired }` (variant names must serialize to the exact current strings), change the field to `pub reason_code: ReviewReason`, and simplify `review_queue_item` to map classification → enum variant. Snapshot impact: tests/snapshots/applicability_cli_test__*.snap contain reason_code values — the kebab-case serialization is byte-identical, so snapshots should NOT change; verify with `cargo insta test` and report if any drift.

══════ F0289 │ src/applicability/model.rs:145-216 (build_report); caller src/applicability/mod.rs:129-131 │ [maintainability · medium] — VALID ══════
**Symbols:** `build_report(..., mapping_collections: Vec<MappingEvidence>, ...)`.
**Root cause:** The report's determinism contract is self-enforced for controls (BTreeMap/BTreeSet iteration) and policy_sources, but `mapping_collections` is stored verbatim (line ~204 `mapping_collections,`). Sorting happens only at today's single call site (`evidence.sort_by(|left, right| left.uuid.cmp(&right.uuid));` at mod.rs:129). Any future caller passing unsorted evidence violates the schema's determinism guarantee with no error.
**Evidence:** build_report takes the Vec by value and forwards it unsorted; the sort is confirmed at mod.rs:129 outside the builder.
**Remediation:** Move the invariant inside: at the top of `build_report`, `let mut mapping_collections = mapping_collections; mapping_collections.sort_by(|left, right| left.uuid.cmp(&right.uuid));` (or add a `debug_assert!(mapping_collections.is_sorted_by_key(|e| &e.uuid))` plus a doc-comment precondition if the team prefers keeping the sort at call sites — moving it is safer). The caller's sort then becomes redundant; remove it for a single enforcement point. Snapshot check: applicability report snapshots order mapping_collections by uuid already, so no drift expected.

══════ F0277 │ src/batch/formatter.rs:46-52 │ [security · medium] — VALID ══════
**Symbol:** `format_batch_summary`, FileOutcome::Failure arm.
**Root cause:** `error_message` is interpolated verbatim into the one-line-per-result stderr layout: `"  \u{2717} {input_name} \u{2014} {error_message} ({duration_secs:.2}s)"`. Error messages from the IO/parser layer echo input paths and content fragments (e.g. ForgeError::Parse strings embed zip/parser errors, ForgeError::FileNotFound embeds paths), so an embedded newline breaks the assumed line layout for anything parsing this block, and raw control bytes (ANSI escapes) enable terminal output forging/log injection from crafted filenames or content.
**Evidence:** interpolation confirmed at lines 47-51; error constructors embedding input-derived text confirmed (src/ingest/mod.rs:182, 186, 188; src/batch/orchestrator.rs validate_inputs formats paths into BatchConversion messages).
**Remediation:** Before rendering, map control chars to spaces: `let safe_message: String = error_message.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();` and interpolate `safe_message`. Add a unit test in formatter.rs tests: a FileResult::failure whose message contains "\n\u{1b}[31mFORGED" renders on exactly one line with no ESC byte.

══════ F0294 │ src/batch/orchestrator.rs:115-119 │ [maintainability · medium] — VALID ══════
**Symbol:** `convert_single_file` Ok(Err(e)) arm.
**Root cause:** The typed `ForgeError` is flattened via `e.with_max_size_guidance().to_string()` into `FileOutcome::Failure { error_message: String }`, permanently discarding the variant and its source chain. `ForgeError::exit_code()` distinguishes validation (3), external-dependency (4), etc., but `execute_dispatch` (src/cli/convert.rs:345-348) can only emit a generic "N of M files failed" BatchConversion error because every per-file cause was reduced to prose. Per-category exit codes, structured --json failure summaries, or retry classification all hit this wall.
**Evidence:** flattening confirmed at lines 115-119; FileOutcome::Failure carries only String (src/batch/summary.rs:13-17); execute_dispatch generic message confirmed.
**Remediation:** Change `FileOutcome::Failure` to carry the owned error (e.g. `Failure { error: ForgeError }` — `with_max_size_guidance()` already returns an owned ForgeError), keep `error_message()`/Display rendering at the display boundary in formatter.rs, and let future consumers match on the variant. Update FileResult::failure constructor and the formatter/tests accordingly.

══════ F0295 │ src/batch/orchestrator.rs:84-91 │ [other · medium] — VALID ══════
**Symbol:** `run_batch_conversion` thread-pool fallback.
**Root cause:** When `rayon::ThreadPoolBuilder::new().num_threads(jobs).build()` fails, execution silently degrades to sequential: the only trace is a `tracing::error!` log line, and the returned `BatchSummary` is structurally indistinguishable from a successful parallel run. For large batches this collapses throughput with no user-visible or machine-readable signal.
**Evidence:** fallback confirmed at lines 87-90; BatchSummary (src/batch/summary.rs:52-65) has no mode/degraded field.
**Remediation:** Record degradation: add a field (e.g. `pub parallel: bool` or `degraded_reason: Option<String>`) to BatchSummary set by `from_results`, surface it in format_batch_summary ("(ran sequentially: pool creation failed)"), and consider building the pool once per invocation set instead of per call.

══════ F0287 │ src/batch/output_naming.rs:21-46 │ [bug · medium] — VALID ══════
**Symbol:** `derive_output_paths` (`claimed: HashSet<PathBuf>`, `next_suffix`).
**Root cause:** Uniqueness rests on byte-exact PathBuf equality, which does not model case-insensitive (macOS default APFS/HFS+, Windows NTFS) or normalization-insensitive filesystems: inputs `policy.md` and `POLICY.md` derive `policy.json` vs `POLICY.json` — unequal in the HashSet but the same physical file — so the second write silently overwrites the first despite the documented anti-collision guarantee (doc comment lines 6-12).
**Evidence:** HashSet<PathBuf> membership test confirmed at lines 21, 33; no case folding anywhere.
**Remediation:** Key the `claimed` set and `next_suffix` map on a folded stem (e.g. ASCII-lowercased `format!("{stem}.{ext}")`) on platforms with case-insensitive defaults (cfg target_os windows/macos), or unconditionally fold and document the assumption; add a regression test with mixed-case stems expecting a `_2` suffix.

══════ F0282 │ src/batch/summary.rs:52-65 │ [maintainability · medium] — VALID ══════
**Symbol:** `BatchSummary` (all-pub fields).
**Root cause:** Every field is public, so callers can build `BatchSummary { .. }` literally or mutate/push/remove `results` after construction, silently desynchronizing `total_files`/`succeeded`/`failed` from the list and breaking the documented sort invariant — `has_failures()` then reports stale state.
**Evidence:** struct at lines 53-65 all `pub`; construction happens only via `from_results` today (orchestrator.rs:93) plus tests.
**Remediation:** Make the five fields private, expose read-only accessors (`total_files()`, `succeeded()`, `failed()`, `total_duration()`, `results(&self) -> &[FileResult]`), and update the two consumers (src/batch/formatter.rs:18-27 field reads; src/cli/convert.rs:347 `batch_summary.failed`/`total_files`) plus the summary.rs unit tests that read fields directly. `has_failures()` stays.

---

## B. PARTIAL findings

══════ F1020 │ Cargo.toml:27 │ [bug · medium → low] — PARTIAL ══════
**Symbols:** `zip = { version = "8.6.0", default-features = false, features = ["deflate-flate2"] }`; consumer `extract_docx_content` (src/ingest/mod.rs:179-195).
**Why partial:** The factual core holds — with only `deflate-flate2`, ZipArchive rejects bzip2/zstd/xz-compressed or encrypted entries at runtime. But the finding's harm premise ("ingest feeds on externally supplied `.forgepack` uploads") is stale/wrong: no `.forgepack` concept exists anywhere in the current codebase (grep: only the review docs mention it), and the zip crate is used ONLY to open `.docx` files (src/ingest/mod.rs:14, 181). OOXML packages are conventionally deflate/stored, so real-world DOCX rejection risk is low; the error surfaced is already wrapped as ForgeError::Parse("failed to open DOCX archive: {e}") (line 182), which is reasonably actionable. Severity lowered to low: latent acceptance gap, not a live defect.
**Remediation:** Confirm the narrow codec set is intentional (it appears deliberate for attack-surface reduction). If broader acceptance is wanted, add `bzip2`/`zstd` features; otherwise add a doc comment on Cargo.toml:27 stating only stored/deflate DOCX entries are supported and ensure the Parse error message names the unsupported-method cause clearly.

══════ F1024 │ Cargo.toml:37-38 │ [security · medium → low] — PARTIAL ══════
**Symbols:** `[lints.rust] unsafe_code = "warn"`; existing unsafe in src/lifecycle/mod.rs:1100-1162 and src/mapping/mod.rs:427-485.
**Why partial:** `unsafe_code = "warn"` is confirmed. However the claim that new unsafe "can easily land in CI that doesn't run with -D warnings" does not hold for THIS repo: CI runs `cargo clippy -- -D warnings` (ci.yml:50-51) which promotes the warn to deny on every CI build, and the crate already contains exactly two narrowly-scoped, SAFETY-commented Windows FFI blocks that carry `#[allow(unsafe_code)]` module attributes (lifecycle/mod.rs:1100, mapping/mod.rs:428) — so flipping to `forbid` would compile today only if those keeps their allow (allow cannot override forbid; it would FAIL, requiring `forbid` → keep `deny` at crate level or restructure). The hardening is real but incremental given the CI gate; severity lowered to low.
**Remediation:** Change to `unsafe_code = "deny"` (not `forbid`, because the two Windows modules use `#[allow(unsafe_code)]` which deny permits but forbid rejects), keeping the scoped allows documented. If the team insists on forbid, the Windows FFI blocks must move behind a separate crate or cfg-gated lint attributes first.

══════ F0031 │ benches/export_bench.rs:52-63 │ [bug · medium → low] — PARTIAL ══════
**Symbol:** hand-built `OscalCatalog` in `build_catalog_json`.
**Why partial:** The bench does rebuild the envelope field-by-field with `controls: vec![]` and forwards only `groups`, bypassing `build_catalog`'s full output. BUT today `build_catalog` itself always emits `controls: vec![]` (src/oscal/catalog.rs:435) and the production pipeline assembles the envelope identically (src/pipeline.rs:190-195 — same `controls: vec![]`, groups, back_matter). So the benchmarked artifact is currently byte-identical to production; the divergence risk is real but latent, triggered only if envelope assembly changes. Severity lowered to low.
**Remediation:** Still worth doing cheaply: refactor the bench to use a shared envelope-assembly helper (the same pattern duplicated in benches/pipeline_benchmark.rs:102-134 `build_catalog_envelope` and benches/xml_benchmark.rs:27-63 — see also F0101/F0052), e.g. expose the pipeline's envelope assembly as a public test-support helper or duplicate via the F0041-style benches/common module, so a future pipeline change cannot silently desync the benches.

══════ F0293 │ src/batch/orchestrator.rs:108-110 │ [other · medium → low] — PARTIAL ══════
**Symbol:** `std::panic::catch_unwind(std::panic::AssertUnwindSafe(...))` in `convert_single_file`.
**Why partial:** The missing-safety-rationale critique stands as a documentation gap. But the substantive unsoundness scenario does not hold under inspection of current code: `run_pipeline` (lines 124-142) only calls `run_catalog_pipeline`/`run_component_pipeline` (pure functions of their inputs producing an owned PipelineOutput) and `write_output` to the per-file output path — it touches no shared mutable state, no locks or caches (schema validators are read-only once compiled in per-model OnceLock cells, src/validate/mod.rs:196-204). A caught panic therefore cannot leave half-mutated shared state for sibling rayon tasks; the panic=abort caveat is inherent to catch_unwind generally. Remaining valid ask: document the invariant (pipeline is shared-state-free; catch_unwind is best-effort isolation) so future edits don't silently introduce shared state behind the AssertUnwindSafe.
**Remediation:** Add a doc comment on `convert_single_file` stating: (1) run_pipeline must remain free of shared mutable state for catch_unwind isolation to be sound, (2) under panic=abort the catch never returns. No behavioral change.

---

## C. DUPLICATE findings

══════ F0785 │ src/validate/semantic.rs:25 │ [bug · high] — DUPLICATE of F0784 ══════
Same root cause as F0784: the orphaned-link pass is model-type-unaware. F0785 describes the caller side (validate() discards model_type at line 25); F0784 describes the resource-collection side (hardcoded root_keys at line 35). One change — threading model_type through check_orphaned_links/collect_resource_uuids — fixes both; F0784 is primary because it also demands enumerating the correct root key ("profile") and its remediation subsumes the F0785 signature change. Implement per F0784.

══════ F0001 │ examples/component-based/output/ssp.json:23-24 │ [bug · medium] — DUPLICATE of F0018 ══════
Same root cause as F0018: generate_ssp.py emits the non-schema `responsible-party` key (lines 59-62, 69-72). F0001 is the output-artifact view (ssp.json:23-24 and every component block), F0018 is the generator view that additionally captures the missing metadata.roles/parties context. Fixing the generator per F0018 and regenerating ssp.json resolves F0001; hand-editing the JSON would be wrong (generated artifact).

---

## D. INVALID findings

None. All 60 findings describe defects that exist in the current codebase to at least some degree; the four PARTIAL findings had only their severity/harm-premise adjusted, and the two DUPLICATE findings share root causes with stronger in-slice primaries.
