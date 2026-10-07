# Valid OCR findings to be fixed

**Status 2026-10-06:** V1 to V4 and L1 to L4 are fixed in the working tree (uncommitted). L5 is left for the plan's author. V3 and V4 were syntax-checked only: `ui/node_modules` (Playwright) is not installed here, so the browser run still needs executing on a machine that has it.

Validated 2026-10-06 against the working tree. Each finding was checked against the code or data it cites. Findings are ordered by priority. The rejected findings are in [`ivalid_ocr_findings.md`](ivalid_ocr_findings.md).

## V1. `receipt-expired-410.json` fixture says `retryable: true`; the runtime says `false` — FIXED

- **File:** `docs/api/fixtures-v2/error/receipt-expired-410.json:4`
- **Severity:** medium
- **Evidence:**
  - `src/workspace/effects.rs:649` returns `Error::new("receipt-expired", "The preview expired. Prepare a new preview.", false)`.
  - `src/workspace/source_transfers.rs:523` and `:539` also use `false`.
  - `ui/tests/workspace.cjs` injects `receipt-expired` with `retryable:false`.
  - Only this fixture disagrees. Resending the same receipt cannot succeed, so a client that trusts the fixture would retry blindly.
- **Proposed fix:**
  1. Set `"retryable": false` in the fixture.
  2. Make the fixture message match the runtime wording, or leave it if the contract only pins the code.
  3. Check `docs/api/forge-workspace-v2.openapi.yaml` for any `receipt-expired` example that repeats `retryable: true`.
  4. Run `cargo test --test api_contract_validation`. Update the recorded fixture hashes or counts in `docs/api/fixtures-v2/index.json` if that test checks them.

## V2. `capability-matrix{,-v2}.json` CM-31 lists too few read-only mutations — FIXED

- **Files:** `docs/api/capability-matrix.json` (CM-31), `docs/api/capability-matrix-v2.json` (CM-31), and the matching `.md` files.
- **Severity:** medium (contract documentation; CM-31 is the AC-2 read-only enforcement entry)
- **Evidence:**
  - CM-31 says every mutation returns the typed `read-only-session` error. Its `operations` list omits browser-write operations that are defined elsewhere in the same matrix.
  - v1 matrix: `cancelOperation` (CM-28) is missing.
  - v2 matrix: `cancelOperation` (CM-28), `prepareProjectSourceBundleExport` (CM-44), `prepareProjectSourceBundleImport` (CM-45), `commitProjectSourceBundleRestore` (CM-45) and `cancelProjectSourceBundleRestore` (CM-47) are missing.
  - The CM-31 note's read list is also incomplete. `shutdownSession` (CM-03, whose own note says it stays available read-only), `getOperation` (CM-27), `getProjectBundlePreview` (CM-38) and `verifyProjectBundle` (CM-39) are `browser-read` but not named.
- **Not valid, same review:**
  - The suggestion to split CM-42 and CM-44 is rejected. Their notes already say which operation is read scope.
  - CM-35's "browser or machine" wording is rejected for the same reason.
- **Proposed fix:**
  1. Append the missing operation IDs to CM-31 `operations` in both JSON files.
  2. Update the CM-31 `notes` read list to include CM-03, CM-27, CM-38 and CM-39. In v1, include only entries that exist there.
  3. Mirror the changes in `capability-matrix.md` and `capability-matrix-v2.md`.
  4. Before editing, confirm in the handlers that read-only sessions reject each added mutation, so the doc is not made wrong in the other direction.
  5. Run `cargo test --test api_contract_validation`. It loads these matrices (`tests/api_contract_validation.rs:35,1243`).

## V3. `ui/tests/workspace-bundle-browser.cjs` hardcodes the macOS Chrome path — FIXED

- **File:** `ui/tests/workspace-bundle-browser.cjs:26`
- **Severity:** medium
- **Evidence:**
  - The script launches `/Applications/Google Chrome.app/Contents/MacOS/Google Chrome` unconditionally.
  - The sibling `ui/tests/workspace.cjs:15` already honours `FORGE_TEST_BROWSER_EXECUTABLE`.
  - The hosted-Chrome manifest targets `/opt/google/chrome/chrome` on Linux runners.
- **Proposed fix:** mirror `workspace.cjs`.
  1. Use `process.env.FORGE_TEST_BROWSER_EXECUTABLE || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'` as `executablePath`. This keeps current local behaviour and adds an override.
  2. Do not fall back to Playwright's bundled Chromium. The probe and `docs/development-tools/hosted-chrome.json` forbid browser downloads.
  3. The script records its own `script_sha256` in receipts. After the change, any pinned script hash must be regenerated. Check `scripts/test_workspace_bundle_browser.py` and the `docs/plans/*f19*` receipts.

## V4. `ui/tests/workspace-bundle-browser.cjs` asserts inside the route handler (low) — FIXED

- **File:** `ui/tests/workspace-bundle-browser.cjs:30-36`
- **Severity:** low
- **Evidence:**
  - `assert(target.pathname.startsWith('/api/v2/'))` throws inside `context.route`. The throw is not routed to the main `try/catch`, and the request is neither continued nor aborted. A violation therefore surfaces as a timeout or an unhandled rejection, not a clear phase-tagged failure and receipt.
  - The read-only guard at line 95 matches `POST /api/v2/...` strings from the request set. A mutation using another method would not be caught.
- **Proposed fix:**
  1. Replace the assert with `receipt.route_violations` bookkeeping, then `route.abort()`.
  2. After navigation, assert `route_violations` is empty.
  3. Optionally add a `mutating_api_requests` counter, incremented for any non-GET/HEAD/OPTIONS `/api/` request. In read-only mode, assert the counter is 0.
  4. Same hash-regeneration caveat as V3. Apply both fixes in one change.

---

# Low priority: true, but optional hardening or cleanup

These were first rejected in `ivalid_ocr_findings.md` as not worth fixing. On re-triage they are accurate observations with no failing scenario today. Do them opportunistically.

- **[FIXED] L1. `ui/workspace.css:25` `[hidden]` can lose to later `display` rules.**
  - `[hidden]` has specificity (0,1,0), the same as `.cards` and `.view-heading`, and those rules come later.
  - Nothing in `ui/workspace.js` currently toggles `hidden` on those classes, so there is no visible bug.
  - Fix: `[hidden] { display: none !important; }`. Re-run `ui/tests/workspace.cjs` and `workspace-navigation.cjs`.
- **[FIXED] L2. `docs/api/fixtures-v2/project-staged-source/bundle-index1-new-role-invalid.json` has a stale `index_sha256`.**
  - It was copied from the `policy-source` bundle, but this fixture's index uses `lifecycle-source`.
  - Today it is rejected by the schema on the role first, and nothing loads it natively.
  - Fix: recompute the hash with the same canonicalisation as `src/workspace/staged_source_bundles.rs` (pretty JSON plus trailing newline). That way a later native test isolates the role rejection.
- **[FIXED] L3. `docs/api/fixtures-v2/index.json` descriptions have missing spaces** ("Only1 or2", "Bundle1/index2", "cannot exceed100", and similar).
  - The README and OpenAPI use the same compact style, so fix them all or none.
  - Cosmetic; descriptions only.
- **[FIXED] L4. `scripts/workspace_browser_tool_probe.cjs` pin comments.**
  - Add comments saying the `1.62.1` and `v24.19.0` pins and the `forge.hosted-chrome-tools/1` schema string must change in lockstep with `docs/development-tools/hosted-chrome.json`.
  - Add a comment recording the residual threat model: local run, no concurrent writer to `ui/node_modules`.
  - Comment-only, but check whether anything hashes this script before editing.
- **L5. `docs/plans/2026-10-02-f19-api-v2-verification.json` pinned-source gap.**
  - `rust_llvm.current_exact_src_pins` lists 16 `src/` files and `source_before_verification_documents` lists 14. `src/applicability/mod.rs` and `src/workspace/root.rs` are only in the first.
  - This is arithmetically confirmed. Whether it is an error depends on whether the before-manifest was meant to cover the instrumented set, and the document does not say.
  - Decide with the plan's author. If it is a gap, add a successor record rather than editing this one, because the file is a hash-pinned frozen plan.
