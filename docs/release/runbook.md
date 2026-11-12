# FORGE 2.0.0 release runbook — DRAFT

> **Status:** DRAFT — prepared 2026-11-12 by the F22 release-scaffolding slice
> for owner review. Basis: `b4226bb35d796222c30f93235ac22118dd26a344`.
> This runbook authorizes nothing. **Publication is an explicit owner act
> (§7); no agent may tag, publish, or close release status.**
>
> Existing conventions referenced: PRD 049 (cross-platform release), PRD 050
> (integration + release), PRD 069 (dependency audit), `docs/RUNBOOK.md`
> (tester recipes), `docs/OSCAL_COMPATIBILITY.md` (schema baseline),
> `docs/plans/` dated receipts (`format`, `basis`, job/log `sha256` fields).

## 0. Mechanism summary (what actually exists)

[`.github/workflows/release.yml`](../../.github/workflows/release.yml) is the
only publication mechanism. It triggers on **any `v*` tag push** and runs:

1. `test` — `cargo test`, strict Clippy, release build on
   `ubuntu-latest` / `macos-latest` / `windows-latest`.
2. `build` — four targets (`x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`,
   `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`); packages the binary +
   `docs/api` + the four workspace schemas into
   `forge-<tag>-<target>.tar.gz` / `.zip`. README/LICENSE/CHANGELOG are
   **not** included; PRD 049 expects documentation to accompany the release
   (WI-48 dependency) — owner decision whether to amend the workflow.
3. `sbom` — installs `cargo-cyclonedx` and generates
   `forge-<tag>-sbom.cdx.json` from the tagged tree.
4. `hash` — SHA-256 of every artifact, base64 for provenance subjects.
5. `provenance` — SLSA L3 via pinned
   `slsa-github-generator/generator_generic_slsa3.yml@f7dd8c5…` (v2.1.0),
   `upload-assets: false` (provenance stays a workflow artifact
   `multiple.intoto.jsonl`).
6. `release` — downloads artifacts + provenance, writes sorted `SHA256SUMS`
   over `*.tar.gz *.zip *.cdx.json`, then `softprops/action-gh-release@efb3536…`
   (v3.0.3) with `generate_release_notes: true` and `files: artifacts/*`.

**Known gaps in the mechanism (owner decisions, not agent edits):**

- The release action sets neither `draft` nor `prerelease`: **any `v*` tag,
  including an RC tag, publishes immediately and publicly.** Until the owner
  approves a workflow change (draft-first or an RC-tag guard), RC tags must
  not be pushed to the repository; use local builds for RC testing.
- Binaries are **not code-signed** (no Authenticode/notarization step exists);
  integrity rests on SHA256SUMS + SLSA provenance. PRD 049 lists signing as a
  Could; the owner must accept or schedule it.
- `cargo-cyclonedx` is installed unpinned-by-version (`--locked` only) inside
  the workflow. The offline draft generator
  (`scripts/generate_sbom.py`, §3) exists so the dependency inventory can be
  reviewed before the tag; it does not replace the workflow SBOM.

## 1. Pre-RC gate checklist (all must be green at the exact candidate head)

Record each item as a dated receipt under `docs/plans/` with `basis` = the
candidate commit SHA, following the existing receipt conventions (job/log
entries carry `path`, `bytes`, `sha256`).

- [ ] **PRD 055–069 evidence closed.** Every scoped Must/Should in PRDs
      055–069 has merged implementation plus its acceptance evidence; open
      human gates (participant judgments, audit sign-off, owner acceptance)
      are explicitly dispositioned by the owner in `docs/plans/`. Track via
      the F-ticket ledger in
      `docs/plans/2026-11-12-remaining-work-completion-plan.md`; agents never
      self-close tickets.
- [ ] **Full local CI replication** at the candidate head:
      `./scripts/ci-local.sh` (fmt, all-target Clippy `-D warnings`, full
      tests, API contract gate, `cargo audit`, `cargo deny check`,
      `cargo vet --locked`). The current-stable Clippy drift recorded in
      `docs/plans/2026-10-02-roadmap-requirement-ledger.md` must be closed or
      dispositioned first.
- [ ] **Supply-chain gates:**
      `python3 -B scripts/dependency_graph.py check` → `qualified`;
      `python3 -B scripts/dependency_inventory.py gate --as-of <date> --strict`
      → exit 0. **At the preparation snapshot this gate fails**
      (`review-required`: 214/215 runtime identities missing accepted
      reviews, 42 new unvetted runtime identities, 0 fresh owned exceptions).
      This is a hard blocker: audits/differential audits or owner-renewed
      exceptions must land first.
- [ ] **Exception discipline:** no entry in
      `supply-chain/dependency-exceptions.json` is expired or past
      `review_by` without a recorded owner renewal. All 47 standing entries
      had `review_by 2026-11-07` / `expires_on 2026-11-23` at preparation —
      renewal or audit closure is required before any tag.
- [ ] **Three-OS CI green** at the candidate head (push to main / PR runs of
      `.github/workflows/ci.yml`, matrix ubuntu/macos/windows), including the
      Python dependency-fixture suites.
- [ ] **Coverage receipt:** physical library LLVM coverage captured at the
      candidate head per the F19/F07 receipt convention (e.g.
      `docs/plans/2026-10-03-f07-foundation-integration-verification.json`
      `capture`/`cover` fields: `.cover` file `path`/`bytes`/`sha256`, changed
      declarations, explicit limits). A prose "tests pass" note is not a
      coverage receipt.
- [ ] **Schema baseline:** `cargo test --test schema_provenance_test` and
      `cargo test --test oscal_1_2_3_compatibility_test` pass; vendored asset
      pins unchanged or upgraded per `docs/OSCAL_COMPATIBILITY.md`.
- [ ] **SBOM draft reviewed:** regenerate
      `python3 -B scripts/generate_sbom.py --as-of <date> --output docs/release/sbom-<date>.json`
      at the candidate head; diff against the committed draft; owner reviews
      the dependency inventory (scope `required` = ships, `optional` =
      dev-only). The authoritative release SBOM is still the workflow's
      cargo-cyclonedx output (§0.3).
- [ ] **Support matrix + policy approved:** owner sign-off recorded for
      `docs/release/support-matrix.md` and `docs/release/support-policy.md`;
      unverified cells resolved or explicitly dispositioned.
- [ ] **CHANGELOG + version:** `Cargo.toml` version bumped to `2.0.0`,
      `Cargo.lock` regenerated, dependency graph refreshed and re-qualified,
      `CHANGELOG.md` complete for the release (PRD 050 S-2), release notes
      draft covering breaking changes and known limitations (PRD 050 M-8).
- [ ] **Rollback rehearsal current:** §8 dry-run receipt exists for the
      release-line mechanism; re-run if `release.yml` changed.

## 2. RC preparation (owner-gated)

1. Owner authorizes an RC in the ticket (veans `#76` thread or `docs/plans/`
   decision record).
2. Because of the §0 tag gap, **do not push `v2.0.0-rc.*` tags** unless the
   owner has first approved a draft/prerelease workflow guard. Build RC
   candidates locally: `cargo build --release --locked` per target, package
   with the same layout as the workflow `Package` steps, and smoke-test per
   PRD 050 S-3 (`forge --help`, sample conversion) on each available platform.
3. Record RC smoke receipts (platform, toolchain, commands, exit codes,
   artifact hashes) under `docs/plans/`.
4. Iterate until §1 is fully green. Every fix re-opens the affected receipts;
   receipts bind to exact heads.

## 3. SBOM procedure

- Draft (offline, any time): `scripts/generate_sbom.py` derives CycloneDX 1.6
  from `Cargo.lock` + `supply-chain/dependency-graph.json`, refusing any
  drift from the graph's input pins. Deterministic; tested by
  `python3 -B scripts/test_generate_sbom.py`. Drafts are marked
  `forge:draft=true` and are never release assets.
- Release (at tag): the workflow `sbom` job runs cargo-cyclonedx on the tagged
  tree. After publication, verify the released `.cdx.json` component set
  against the reviewed draft (same lock ⇒ same crate identities/versions/
  checksums); record any difference and its cause in the release receipt.
- PRD 069 S-4 (publish inventory with release artifacts) is satisfied by the
  workflow SBOM asset; the owner may additionally attach the reviewed
  dependency-inventory gate JSON.

## 4. Tag and publication — OWNER STOP POINT

**Only the owner (or a human the owner explicitly delegates in writing) may
execute this section. Agents stop here.**

1. Confirm every §1 checkbox has a receipt at the final head, and the owner
   publication authorization is recorded (ticket comment + `docs/plans/`
   decision naming the exact SHA and version).
2. Owner pushes the annotated tag: `git tag -a v2.0.0 -m "FORGE 2.0.0" <SHA>`
   then `git push origin v2.0.0`. This single act triggers publication (§0);
   there is no second confirmation in the workflow.
3. Watch the Release run to completion. Any job failure ⇒ §8 rollback
   decision; do not re-tag over a partially published release without owner
   decision.

## 5. Post-publication verification (agent-runnable, read-only)

- [ ] Release page lists 4 archives + `SHA256SUMS` + `.cdx.json` (+ provenance
      if the owner chose to attach it).
- [ ] Download all assets; `sha256sum -c SHA256SUMS` passes.
- [ ] Verify SLSA provenance with `slsa-verifier verify-artifact` against the
      expected tag/repository (tool install is owner-approved tooling).
- [ ] Per-platform smoke: download, `forge --help`, convert an
      `example_data/` policy, validate output (PRD 050 S-3). Record receipts.
- [ ] SBOM reconciliation per §3.
- [ ] Announce per owner direction; update `CHANGELOG.md` header and roadmap
      ledger; move tickets to In Review — **never close** (merge hook/owner
      closes).

## 6. Provenance and signing notes

- SLSA L3 provenance is generated by the pinned reusable workflow; subjects
  are the base64 SHA-256 list from the `hash` job. `upload-assets: false`
  means the `.intoto.jsonl` is a workflow artifact; the `release` job pulls it
  into `artifacts/` so it ships with the release files glob.
- No code signing exists (§0). If the owner later approves signing, it must
  land **before** the `hash` job so signatures are covered by SHA256SUMS and
  provenance.

## 7. Authorization record template

```
Decision: authorize publication of FORGE v2.0.0
Head: <full SHA>
Gates: docs/plans/<receipt files>
Exceptions disposition: <file or "none outstanding">
Authorized by: <owner> on <date>
```

## 8. Rollback procedure

Mechanism facts (from `release.yml`): publication is a GitHub Release created
by `softprops/action-gh-release`; re-running the workflow with the same tag
re-uploads same-named assets (replacement), and `SHA256SUMS` is regenerated
each run. There is no unpublish job; rollback is a deliberate human sequence.

**Offline rehearsal (done, agent-runnable):** the checksum-generation and
asset-replacement mechanics were dry-run in
`/tmp/forge-f22-rollback-rehearsal-v1` with synthetic archives; receipt:
`docs/plans/2026-11-12-f22-rollback-rehearsal-v1.json` (pass, including a
tamper-detection negative control). Re-run the same steps whenever the
`release` job changes.

**Live rollback (human gate — exact steps):**

1. Owner declares the release defective in the ticket; freeze further tags.
2. Mark the GitHub Release as a pre-release / edit notes with a prominent
   warning (GitHub UI or `gh release edit v2.0.0 --prerelease`), or delete
   the release entirely (`gh release delete v2.0.0`) — owner chooses based on
   exposure. Deleting the release does **not** delete the tag.
3. If the tag itself is bad: `git push origin :refs/tags/v2.0.0` then
   `git tag -d v2.0.0` (owner only). Re-tagging the same version is forbidden
   without an owner decision recorded per §7 — consumers may have cached
   artifacts; prefer a new patch version.
4. Fix forward on a branch, re-run §1 gates at the new head, publish the
   corrective version through §4.
5. If assets must be replaced in place (owner-approved exception): rebuild
   from the fixed head, replace the affected assets **and** regenerate
   `SHA256SUMS` in the same act (rehearsed mechanics), re-attach provenance
   for the new subjects, and record before/after hashes in a receipt. Old
   provenance no longer covers replaced bytes — never leave stale
   attestations attached.
6. Record the whole incident as a dated `docs/plans/` receipt: trigger,
   decision, commands, asset hashes before/after, final state.

## 9. What remains human-only (summary)

- Owner approval of support matrix, support policy, and this runbook.
- Exception renewals / audit sign-off (PRD 069) and PRD 055–069 acceptance.
- Any workflow change (RC guard, draft-first, signing).
- Tag push, publication, rollback decisions, ticket closure.
- Native Windows / Intel-macOS qualification runs and per-platform smoke on
  real hardware or hosted runners.