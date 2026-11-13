# PR 265: cargo-group bump — zip 8.6.0 → 9.0.0 in retained evidence manifests

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), base
`b4226bb35d796222c30f93235ac22118dd26a344`. The PR is OPEN; this note is not a
merge claim, an accepted source audit, or human signoff. No Cargo manifest,
lockfile or supply-chain policy was changed by this review.

**Scope warning: this PR edits immutable historical receipts.** The only
changed files are the retained 2026-10-02 review manifests
`docs/dependency-reviews/2026-10-02/pr16{2,3,4}/Cargo.lock` and matching
`Cargo.toml`. The shipping root `Cargo.lock`/`Cargo.toml` are untouched. The
2026-10-02 packet documents those lockfiles as frozen snapshots of the exact
reviewed heads; mutating them invalidates the packet's SHA-256 evidence index
and its replayability claim. **Recommend closing this PR without merge** (or
re-scoping it as an explicit, owner-approved re-baselining, which is a
different decision than a dependency bump).

## Exact transition

Per [Cargo.lock.diff](Cargo.lock.diff) and
[lock-transitions.json](lock-transitions.json), in each of the three evidence
manifests:

| Package | Old | New | New checksum |
|---|---|---|---|
| zip (×3 files) | 8.6.0 | 9.0.0 | `711e8c9646791c49b7eb799a19c4ae70a9b8372ddc6a1fa4778defd1e2aa5d5e` |

Old checksum `2d04a6b5381502aa6087c94c669499eb1602eb9c5e8198e534de571f7154809b`.
The manifests' `zip = "8"` requirement becomes `zip = "9"`. No other package
changes in any of the three files.

## Advisory status (observation only)

Local `cargo audit --no-fetch --file Cargo.lock` at the PR head, per evidence
manifest ([audit-evidence/](audit-evidence/)):

| Manifest | Vulnerabilities | Notes |
|---|---|---|
| pr162 | RUSTSEC-2026-0385 (ctutils 0.4.2), RUSTSEC-2026-0433 + 0414 (pulldown-cmark 0.13.4) | ctutils finding pre-exists at base; zip 9.0.0 itself is clean |
| pr163 | RUSTSEC-2026-0433 + 0414 (pulldown-cmark 0.13.4) | baseline |
| pr164 | RUSTSEC-2026-0433 + 0414 (pulldown-cmark 0.13.4) | baseline |

Warnings in all three: ttf-parser/unsafe-libyaml unmaintained; rustix 1.1.4
and tokio 1.53.1 unsound (informational). The zip advisories
RUSTSEC-2026-0367/0368/0369 (privilege escalation / path traversal classes,
patched ≥ 9.0.0) match zip 8.6.0 — i.e. they match the **retained evidence
lockfiles**, which is the provenance of the open Dependabot alerts (see
[packet index](../README.md)). The shipping root lock already carries
zip 9.1.0, outside every advisory range; this PR fixes no shipping
vulnerability. PR #260 (zip shipping-side work) is handled separately and is
out of scope here.

## Source delta review

The candidate crate delta is zip 8.6.0 → 9.0.0; because this PR only touches
historical receipts, no shipping code consumes it. The bounded review
therefore records the advisory ranges above rather than a shipping-impact
analysis. The one transitive delta visible in the packet's own evidence
directory is [deltas/uuid-source.diff](deltas/uuid-source.diff)
(uuid 1.23.2 → 1.24.1, from the base pr164 evidence context): parser rewritten
to a fixed-size `decode_hex32` with statically known bounds, serde/fmt
adjustments — no security-relevant regression identified in that bounded
inspection. It is retained for completeness, not as a PR #265 change.

## Executed checks (throwaway worktree, macOS arm64, rustc 1.99.1)

- Root lock at the PR head is byte-identical to base; `cargo audit --no-fetch`
  on it ([audit/cargo-audit.json](audit/cargo-audit.json)) shows only the 2
  baseline pulldown-cmark findings, same as [../audit-base.json](../audit-base.json).
- `python3 -B scripts/dependency_graph.py check`: **qualified** (exit 0) —
  expected, since the shipping graph is unchanged
  ([ci/dependency-graph-check.json](ci/dependency-graph-check.json)).
- `python3 -B scripts/dependency_inventory.py gate --as-of 2026-11-12`:
  **review-required** (exit 1), 42 new-unvetted-runtime entries, 214/215
  runtime packages missing accepted review — the standing F03 backlog, not a
  regression of this PR
  ([ci/inventory-gate-summary.json](ci/inventory-gate-summary.json)).

## Hosted CI (exact head, run 41756372632, 2026-11-04)

[ci/jobs.jsonl](ci/jobs.jsonl), [ci/run.json](ci/run.json):

- Test (ubuntu/macos/windows): **pass** (root graph unchanged).
- Dependency fixtures (3 OS): pass.
- Supply-chain audits: **fail** — root `cargo audit` stops on the 2 baseline
  pulldown-cmark advisories (1403-advisory DB at run time); deny/vet never ran
  ([ci/supply-chain-excerpt.txt](ci/supply-chain-excerpt.txt)). The hosted
  audit scans the root lock only, not the evidence manifests.
- Dependency inventory gate: **fail** — review-required backlog
  ([ci/inventory-gate-full.txt](ci/inventory-gate-full.txt)).

## Disposition

Agent-assisted review only; no human audit approval, no merge permission.
Beyond the standing audit backlog, the decisive objection is receipt
immutability: the 2026-10-02 evidence lockfiles must not be edited to silence
alerts. Alert dismissal or an explicit re-baselining is an owner action.