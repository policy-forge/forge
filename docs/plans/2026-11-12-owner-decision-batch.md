# Owner decision batch — 2026-11-12

Status: **Awaiting owner decisions.** Prepared by the coordinator from
verified evidence; nothing here is approved. Answer inline or in the
linked tickets; each decision unblocks named work.

## D1. Approve four supply-chain exceptions (PRs #266 + #268) — unblocks F03 merge path

PR #266 (`fix/p0-supply-chain-security`, head `05fc7ca`) upgrades
zip 9.1.0→9.1.2, pulldown-cmark 0.13.4→0.13.6, rustix 1.1.4→1.1.6.
Full suite green; `cargo audit` 0 vulnerabilities; vet/graph gates pass.

- [ ] Approve the three PROPOSED exact-identity exceptions (owner
  brianluby, review by 2026-11-26, expiry 2026-12-26) in
  `supply-chain/dependency-exceptions.json` + `config.toml`.
  Evidence: `docs/plans/2026-11-12-supply-chain-security-decisions.md`
  (on the PR branch). On approval the PROPOSED markers are struck and
  the decision recorded; on rejection the branch is reverted whole.

- [ ] Approve the fourth PROPOSED exception: **quick-xml 0.42.0**
  (PR #268, head `e5feba4`, same owner/review/expiry terms). Migration
  verified: full suite green, no snapshot changes, XML output
  byte-identical for realistic inputs; the only reachable delta (tab in
  a heading → `&#9;` in `source-section` attrs) independently reproduced
  and confirmed to round-trip identically. Reader now rejects non-UTF-8
  at the parser (Forge already validates at ingest — refusal preserved).
  Note: #266 and #268 both touch `supply-chain/` — merge #266 first,
  then rebase #268 (coordinator handles).

## D1b. Renew the 47 stale exceptions — time-critical (expiry 2026-11-23)

Verified 2026-11-12: **all 47 inventory exceptions are past their
2026-11-07 review-by date** and all expire 2026-11-23 (11 days). The
gate already reports them `stale-exception`. Without renewal the
supply-chain gate hard-fails on expiry and blocks the release path.

- [ ] Renew all 47 for another term (owner brianluby, new review-by
  2026-11-26, expiry 2026-12-26 — matching the October pattern), or
  name exceptions to retire instead. Coordinator applies the renewal
  mechanically on approval; identities unchanged. The four new
  exceptions in D1 ride the same term.

## D2. Dismiss 25 retained-evidence Dependabot alerts — unblocks clean security dashboard

All 25 alerts are zip 8.6.0 in the immutable October review receipts
(`docs/dependency-reviews/2026-10-02/pr16*/Cargo.lock`), not the shipping
graph (root zip 9.1.0 is out of every advisory range; verified via
Dependabot API + `cargo audit`).

- [ ] Dismiss all 25 as "tolerable risk — retained historical review
  evidence, not a shipping manifest". Receipts stay byte-identical.

## D3. quick-xml 0.42 migration path — unblocks #264 disposition

#264 (0.41→0.42) breaks compile: 23 errors, byte-slice→&str API change.
The migration is DELIVERED as draft PR #268 (head `e5feba4`, verified —
see D1).

- [ ] Confirm: close #264 in favor of #268 (recommended), or hold 0.41
  until a later release.

## D3b. Close PR #265 without merge — protects evidence integrity

Verified: #265's entire diff is zip 8.6.0→9.0.0 inside the immutable
October evidence receipts (`docs/dependency-reviews/2026-10-02/pr16{2,3,4}/`);
the root lock is untouched. Merging corrupts the SHA-256-indexed F03
evidence chain.

- [ ] Close #265 without merge (recommended). The alerts it targets are
  resolved by D2 dismissal, not receipt edits.

## D4. Decision packets — DELIVERED, draft PR #270 (head `08115d7`)

`docs/plans/2026-11-12-owner-decision-packets.md` (659 lines, full
citations, pre-commit gate green: 4498 tests). Answer each item in the
packet; summary:

1. **D067** — ratify the implemented confined-roots MCP discovery +
   read-only threat model; name the two-client conformance pair
   (proposed: MCP Inspector `modern` + Claude Code v2 runtime, MCP
  2026-07-28). **Re-pin DELIVERED:** draft PR #275 (head `428c53e`),
  `docs/plans/2026-11-12-f11-discovery-threat-model-proposal.md`
  (625 lines, sha256 `53b9566f132e6e1b20f9191d1062b8929b8f8807805284e31c86982d98135c16`,
  44063 bytes, LF). The original approved bytes (sha256 `60c8dff1…`)
  were lost from `/tmp`; ratification must approve THIS document fresh
  and bind its hash — no approval is inherited.
2. **D068 scope split (F12/F13)** — already bound by the 2026-10-25
   disposition; verify by evidence, no re-decision. (The plan's former
   "D069" label collided with the dependency-policy decision ID and
   was relabelled.)
3. **F10 Q1–Q7** — Q1–Q6 answered by recorded dispositions; Q7 is a
   scope *confirmation* (PRD 054 never excluded AR validation; PR #253
   merged it — no override occurred).
4. **F12 M-10** — proposed-edit envelope with `target_subject_sha256`
   binding + AC-5 wrong-hash rejection; new closed family version.
5. **F12 M-13/M-15** — opt-in sensitive export (default retained);
   Windows no-replace file publisher already merged (#254); directory
   publication + qualification remain open.
6. **F16 confinement design** — Landlock/seccomp (Linux), sandbox-exec
   (macOS), Job Objects/AppContainer (Windows, dependency-gated);
   descendant cleanup strategy; no new crate without PRD 069 approval.
7. **D068 quorum** — `quorum_v3` verified against the disposition
   minimum: no mismatch; close by evidence.
8. **S-2 signed envelopes** — recommend Won't for 2.0.0 unless design
   supplied.

## D5. F15 Phase 2 dependency approval — unblocks live GitHub adapter

Phase 1 (change sets + fake-server contracts, no new deps) is in
progress. Phase 2 needs an outbound HTTPS client crate (none approved
today; hyper is server-only) + GitHub auth/TLS design.

- [ ] Defer to post-2.0.0 (recommended, keeps scope finite), or name an
  approved crate + design owner now.

## D6. F06 evaluation G0 dispositions — unblocks participant sessions

PR #271 delivered the session-ready protocol, but sessions cannot start
until the owner records the G0 readiness dispositions the design packet
left pending: adjudication rubric, per-task quality/safety thresholds,
session timing, and mapping-derivation rules (all marked
OWNER-APPROVAL-REQUIRED in the protocol; no numbers were invented).

- [ ] Record the G0 dispositions (protocol §G0 lists each), then
  recruit/consent/schedule participants. Human-only from there.

## D7. F22 support policy approval — unblocks release readiness

PR #269 delivered `docs/release/support-policy.md` as PROPOSED (support
window, PRD 069-grounded security policy, deprecation rules).

- [ ] Approve or amend the support policy. Also pending: MSRV 1.88 is
  declared but CI-unverified (add an MSRV job or accept the risk).

## Already recorded (no action needed)

- D064/D068/F10 add-only: owner-approved 2026-10-25, implemented, merged.
- Tickets #66–#70: close-ready (PRs #252/#253/#255/#256/#257 merged);
  human/merge hook closes.
- Measured coverage baseline: 92.18% lines on main `7dc20f2`
  (`docs/plans/2026-11-12-coverage-baseline.json`).
