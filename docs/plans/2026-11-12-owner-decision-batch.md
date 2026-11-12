# Owner decision batch — 2026-11-12

Status: **Awaiting owner decisions.** Prepared by the coordinator from
verified evidence; nothing here is approved. Answer inline or in the
linked tickets; each decision unblocks named work.

## D1. Approve three supply-chain exceptions (PR #266) — unblocks F03 merge path

PR #266 (`fix/p0-supply-chain-security`, head `05fc7ca`) upgrades
zip 9.1.0→9.1.2, pulldown-cmark 0.13.4→0.13.6, rustix 1.1.4→1.1.6.
Full suite green; `cargo audit` 0 vulnerabilities; vet/graph gates pass.

- [ ] Approve the three PROPOSED exact-identity exceptions (owner
  brianluby, review by 2026-11-26, expiry 2026-12-26) in
  `supply-chain/dependency-exceptions.json` + `config.toml`.
  Evidence: `docs/plans/2026-11-12-supply-chain-security-decisions.md`
  (on the PR branch). On approval the PROPOSED markers are struck and
  the decision recorded; on rejection the branch is reverted whole.

## D2. Dismiss 25 retained-evidence Dependabot alerts — unblocks clean security dashboard

All 25 alerts are zip 8.6.0 in the immutable October review receipts
(`docs/dependency-reviews/2026-10-02/pr16*/Cargo.lock`), not the shipping
graph (root zip 9.1.0 is out of every advisory range; verified via
Dependabot API + `cargo audit`).

- [ ] Dismiss all 25 as "tolerable risk — retained historical review
  evidence, not a shipping manifest". Receipts stay byte-identical.

## D3. quick-xml 0.42 migration path — unblocks #264 disposition

#264 (0.41→0.42) breaks compile: 23 errors, byte-slice→&str API change.
Migration slice `feat/quick-xml-0.42-migration` is in progress
(byte-exact output contract enforced).

- [ ] Confirm: close #264 in favor of the migration PR when delivered
  (recommended), or hold 0.41 until a later release.

## D4. Decision packets (landing shortly on `docs/decision-packets-2026-11-12`)

The packet file covers, with recommendations and citations:

1. **D067** — F11 MCP discovery/threat model approval (unblocks #55
   conformance work). Recommendation: approve the implemented
   confined-roots model.
2. **D069** — F12/F13 scope split for remaining adapters + interactive
   client. Recommendation: portable-file adapters → F12, client → F13.
3. **F10 Q1–Q7** — mostly answered by D064/add-only dispositions;
   Q7 is a scope *confirmation* (PRD 054 never excluded AR validation;
   PR #253 merged it — no override occurred).
4. **F12 M-10** proposed-edit patch contract (target-hash binding).
5. **F12 M-13/M-15** — opt-in sensitive export; keep Windows publication
   refusal until qualification.
6. **F16 confinement design** — per-platform mechanism choice (largest
   unknown; names options with tradeoffs).
7. **S-2 signed envelopes** — recommend Won't for 2.0.0 unless design
   supplied.

## D5. F15 Phase 2 dependency approval — unblocks live GitHub adapter

Phase 1 (change sets + fake-server contracts, no new deps) is in
progress. Phase 2 needs an outbound HTTPS client crate (none approved
today; hyper is server-only) + GitHub auth/TLS design.

- [ ] Defer to post-2.0.0 (recommended, keeps scope finite), or name an
  approved crate + design owner now.

## Already recorded (no action needed)

- D064/D068/F10 add-only: owner-approved 2026-10-25, implemented, merged.
- Tickets #66–#70: close-ready (PRs #252/#253/#255/#256/#257 merged);
  human/merge hook closes.
- Measured coverage baseline: 92.18% lines on main `7dc20f2`
  (`docs/plans/2026-11-12-coverage-baseline.json`).
