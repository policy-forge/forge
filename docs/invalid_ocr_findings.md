# Invalid OCR findings

Validated 2026-10-06 against the working tree. `cargo test --test api_contract_validation` (30 tests) passes, confirming the fixture `valid`/`invalid` classifications. Items that were true but low-value have been moved to the "Low priority" section of the valid-findings file (L1 to L5), and are not repeated here as defects. Valid findings are in [`valid_ocr_to_be_fixed.md`](valid_ocr_to_be_fixed.md). Findings are grouped by file and rejected for the reason given.

## `scripts/workspace_browser_tool_probe.cjs`

- **TOCTOU in `readBounded`:**
  - The code already opens with `O_NOFOLLOW`, then compares `lstat`, `fstat` and a final `lstat` (dev/ino/size/mtime/ctime). The swap case in the finding fails closed (`ELOSS`/`changed-file`).
  - Intermediate-directory swaps and timestamp granularity need an attacker with write access to `ui/node_modules` during a local, read-only, dev-tool run. The script's documented scope excludes that.
  - `openat`-style opens are not available in Node. Rejected as a design-level suggestion.
- **Prototype pollution via `JSON.parse`:** `JSON.parse` does not pollute `Object.prototype`. The code only reads named fields, and the finding itself concedes this. The proposed reviver adds nothing.
- **Symlink validation gap in `packagePin`:** the finding concludes the code is safe. `lstatSync` in `visit()` plus `realpathSync` already cover it. A comment would be optional.
- **Swallowed error detail:** the single fixed message is deliberate. The probe emits a closed, non-disclosing observation, and the tracker design elsewhere avoids reflecting error content.
- **Hardcoded versions and Node pin, plus comment requests:** the exact pins are the point of an attestation script. Mismatch is meant to be a hard failure. Comment-only suggestions are not defects.
- **Non-atomic multi-file verification:** the threat model is a local process with no concurrent writer. The "re-pin before output" suggestion adds no real assurance.
- **Misleading `same()` comment:** the comment says it compares metadata without treating a later unchanged hash as attestation. That matches the code.
- **`symlink-in-package` error naming and `directoryNames` rename:** cosmetic. Failing closed already happens.
- **64 KiB buffer allocation:** negligible at roughly 10k files, one allocation per call. No evidence of a performance problem.
- **`existsSync` and dangling symlink:** the finding states the allowlist check already covers it ("No change needed").

## `ui/tests/workspace-bundle-browser.cjs`

- **Fixture not restored on failure:** `scripts/test_workspace_bundle_browser.py:110` runs the test in a `tempfile.TemporaryDirectory`. The fixture is disposable, so there is no persistent corruption to roll back.
- **Failure receipt keeps only `error.constructor.name`:** deliberate. The receipt is designed not to retain content or paths. Capturing `message` and `stack` would reintroduce them.
- **Top-level `assert(!existsSync(out))` crashes without a receipt:** deliberate fail-closed behaviour. It refuses to overwrite existing evidence. Overwriting with `recursive:true` would defeat that.
- **`waitForResponse` awaited after `goto`:** the waiters are registered before `goto` and awaited immediately after. This is the correct Playwright ordering.
- **`focused()` polling:** the helper deliberately observes focus without assigning it, per its doc comment. Playwright's `waitForFunction` suggestion is equivalent.
- **Minified style and prettier:** style preference. The file pins its own hash.
- **`files()` duplicate keys, ID regex fragility, and `==` in a `waitForFunction` string:**
  - Keys are unique per path.
  - The ID regex comment is speculative.
  - The cited code uses `===`.

## `ui/tests/workspace.cjs`

- **All `==`, `assert.equal`, `assert.deepEqual` and `assert.strictEqual` findings:**
  - `grep` finds no loose `==` in the file.
  - The file imports `node:assert/strict`, in which `equal` is `strictEqual` and `deepEqual` is `deepStrictEqual`.
  - The reviewer's own findings at lines 3, 194, 13 and 340 concede this.
- **Null `page` in the catch block, null `context` and `browser` in `reconcileCleanup`, and error handling in the outer `.catch`:**
  - `createTracker.capture` is first-fault-wins, and the original error is printed first via `console.error(error.stack)`. A secondary `TypeError` cannot replace it.
  - `reconcileCleanup` already skips null owners (`workspace_failure.cjs:80`).
  - The outer `.catch` leaves `exitCode=1`.
- **Throttle gate "deadlock":** if the test fails, the `finally` closes the context and the process ends, so the pending route handler is abandoned. The proposed `setTimeout` adds an uncleared timer.
- **`{times:1}` on self-unrouting handlers:**
  - These handlers act only on a specific GET, or on `page_size=50` with a `cursor`, and pass other requests through.
  - `{times:1}` would consume the one-shot on a non-matching request and break the fault injection.
- **`?*` route globs:** these routes inject faults and the injected faults are asserted. If they did not match, the tests would fail.
- **`waitForResponse` with `response.ok()`, `elementHandle()` null checks, `?.` chaining, and "wait until status text" hardening:** speculative test-robustness suggestions with no failing scenario. A missing element already fails the test clearly through Playwright's timeouts.
- **`expectedOrigin`, port-range check, `favicon.ico`, and request-handler origin filter:**
  - `url` is validated by an anchored regex, and the origin is compared as the same string.
  - `/favicon.ico` is not requested in this suite.
  - The origin-less counters are guarded by the abort route.
- **Reviewer name `<script>`:** the suite already asserts `#view script` count 0 and literal rendering elsewhere (`panel.locator("script").count()` at `workspace.cjs:463`).
- **YAML regex vs `js-yaml`:** adding a new dependency to a pinned, no-new-dependencies test environment is out of scope. The regex is bound to a repository-owned file.
- **API-path constants, long `verifyMetadataConsumer`, step-name helpers, `console.error` vs `console.log`, `—` escape, em-dash and ellipsis text, sorting, symbol keys and download cleanup:** style or refactor preferences. Several would change pinned observation output.
- **Self-contradicting findings:** several items conclude "No change needed" or "Good" themselves (lines 13, 340, 589, 590, 595, 598 and others). They are noise.
- **`measureMetadataLongContent` focus assert:** unconditional failure on a disconnected element would turn a measurement helper into a flaky gate.

## `ui/workspace.css`

- **`[hidden]` specificity:** no element with a `display`-setting class (`.cards`, `.view-heading`) is toggled with `hidden` in `ui/workspace.js`. The hidden elements are buttons and divs that no later rule overrides. Latent only, and a hardening nicety.
- **`prefers-reduced-motion`:** the stylesheet has no animations or transitions to suppress.
- **`forced-colors`:** browsers already force border and text colours in forced-colors mode. An enhancement request, not a defect.
- **`cursor: wait` vs `default`:** deliberate. Native `disabled` is used during pending work, and `aria-disabled` elsewhere. `workspace.cjs` asserts this split.
- **`100vh` / `165px` and `--border` contrast:** cosmetic. The border is decorative, and form controls use the darker `#52605b`.

## Tool config

- **`.specify/config.json` empty `worktree_custom_path` and key spelling:** generated by the specify tool. The finding says it cannot be verified without that schema.
- **`.claude/settings.json` hook with bare `veans prime`:**
  - This is the hook the veans CLI itself generates.
  - PATH-hijack requires an attacker already controlling the user's PATH.
  - An absolute path would break other machines.
- **`.claude/settings.json` wildcard `cargo` permissions:**
  - This is the user's own local allowlist.
  - `cargo build`, `cargo test` and `cargo check` run project code by design.
  - The suggested replacement would block normal use.

## `docs/api/fixtures-v2` and fixture indexes

- **"Fixture named `-invalid` has `validation.state: valid` or looks valid" (about 25 fixtures in `project-bundle-effects` and `project-staged-source`):**
  - The invalidity in these fixtures is schema-level: an extra property, a missing envelope key, a wrong type, or an out-of-range count. It is not a runtime `validation` field.
  - `index.json` assigns `expectation: invalid` and the schema to check.
  - `validation.state: valid` is just the embedded sample preview.
  - The proposed edits would instead make them valid-envelope fixtures that no longer test the intended rule.
  - The README says valid and invalid labels are wire-shape classifications.
- **`-missing-snapshot-invalid` fixtures:** each is invalid because `snapshot_version` is absent. A `grep` confirms 0 occurrences in the three cited files. This is the intended defect.
- **Filename or expectation mismatches (`-native-invalid` marked `valid`, error fixtures without the `-valid` suffix, `version-conflict` kind):**
  - `index.json` is the authority.
  - The `-native-invalid` fixtures are schema-valid and rejected only by the native decoder, as their descriptions say.
  - No test infers expectation from filename.
- **Expired `expires_at` timestamps:** static synthetic fixtures. Wire validity does not imply freshness (README).
- **Identical or placeholder hashes (empty-string SHA-256, all-zero hashes, `aaaa...`):** README states valid means wire-shape only, and that wire validity does not establish exact hashes.
- **`preview-valid.json` overwrite with empty-string `base_sha256`:** an existing empty file is a valid overwrite.
- **`metadata-export-result-valid.json` create with non-null `target_version`:** the schema doc says "when overwriting" but does not require null for create. Wire-shape fixture, no schema violation shown.
- **Intentionally wrong values** (`prepare-selector-invalid` `3`, `stage-bool-counter-invalid`, `consumed_file_count: true`, `Bad_Key`, `unexpected`, `bundle-debug-path`, `forge.workspace/2` in `invalid-schema-version`, path traversal, absolute path): these are the negative cases.
- **`chunk-*` hex payload `"ins"` typo:** decoding all chunk fixtures shows `"pins"` present and `"ins"` absent. The finding misread the hex.
- **`chunk-empty-invalid` sha and `chunk-over-invalid` hash length:** the fixtures are schema-invalid by empty or oversize hex. Their hashes are not meant to be correct, and the "63-character" and "spaces" claims are a misreading of the encoded payload.
- **`create-size-over-invalid` / `create-count-over-invalid` / `manifest-empty-invalid` chunk-count inconsistencies:** schema-only negative fixtures. Ceiling relationships are native-only checks (`manifest-ceil-native-invalid`).
- **`bundle-index1-new-role-invalid` stale `index_sha256`:** no test or script loads this fixture natively (grep finds it only in `index.json`). Schema rejection by role happens first. Could be tidied but is not a defect today.
- **`stage-preparing-valid` counters:** the state-machine claim is not documented in the OpenAPI. It is a wire-shape fixture.
- **`resources/page-valid.json` empty-hash with `size_bytes`:** wire-shape fixture. Hash and size consistency is explicitly not asserted.
- **`index.json` missing spaces ("Only1 or2"):** repository-wide compact style also used in the README and OpenAPI text. Description text only.
- **JSON key spelling "checks":** these were requested by the review checklist and found no errors. They are informational.
- **`getLifecycleQueue-unknown-property-invalid` key ordering:** JSON key order has no semantic effect.
- **`getFrameworkImpactComparison-unknown-property-invalid` `inferred_approval` and duplicated provenance hash:** the unknown property is the intended invalidity, and the rest is synthetic data.
- **`session/unlock-request-valid.json` key check, `workspace/*` fixtures:** the findings themselves conclude "no issue".

## Frozen evidence under `docs/dependency-reviews/` and `docs/plans/`

These are dated, hash-pinned records. For example, `pr162/historical/artifact-index.json` pins file hashes, and `docs/plans/*audit*.json` assert byte preservation of earlier versions. Edits would break the pinned evidence chain. Findings in this group are rejected as a class unless noted.

- **CI failure, `BLOCKED` merge state, and failed `cargo-vet` records (PR 160 to 164):** the records are accurate snapshots of a failing run. They are not file defects.
- **Identical hashes across versions** (`argon2-0.5.3` and `0.6.0` `raw-kat-tags.json`, and `version.rs`): the KAT output is expected to be the same when the algorithm is unchanged. The index hashes real files, so identical hashes are plausible.
- **`source-receipts.json` `build: null` vs `false`, and `rust_version: null`:** frozen captured data from `cargo metadata`, where the field is absent for some crates.
- **`upstream-license-tree.json` empty `license_paths`, minimal `getrandom-0.3.4` and `tempfile` snapshots:** record the upstream state at capture. Not a scan failure signal.
- **`advisory-index-pr160-pr161.json` format variance:** shared evidence file, not read by any script (grep finds it only in other plan JSONs). Out of scope for a code fix.
- **Machine-specific absolute paths in audit JSON:** deliberate evidence provenance, tied to dated hashes.
- **`notcovered`, `mcdc`, and `isBlockCoverage` keys:** verbatim keys from `llvm-cov` and V8 coverage output.
- **Key naming `AST`, `V8`, `API`, `UI` casing, hyphenated scope keys, `summaries`, `readonly_...` and similar naming cleanups in `docs/plans/2026-10-0x-*.json`:** frozen versioned artifacts. Renaming breaks the recorded hashes and cross-version comparisons.
- **`f19-api-v2-verification.json` "missing 2 src files in `source_before_verification_documents`":** the finding assumes the before-manifest must equal the instrumented set. The document does not state that. Frozen record, so left as-is. Revisit only if the plan itself requires equality.
- **`f19-api-v2-verification.json` browser/function-record counts, selected-files note:** counts measure different things (controls vs function records). Frozen.
- **`slice_index.json` `n` vs `files.length`:** `n` equals the sum of `sev`, so it counts findings. It does not count files. The findings are consistent. `unspecified` is simply a severity present in slice01.
- **`hosted-chrome.json` `introducing_pr`, `node_path`, `browser` and `approved_by` wording:** deliberate approval-record wording. The file is consumed and hash-checked by the probe and plans, so edits need a coordinated re-pin and are not a defect.
- **`2026-10-04-f04-protected-dispatcher-source-review-v1.json` naming and duplicate sections:** frozen review artifact.
