# Dependency exception renewal and update decision — approved October 24, 2026

This packet covers two owner decisions made on October 24, 2026:

1. Renewal of the 34 existing owned exceptions whose review deadline
   (October 20, 2026) had passed.
2. Integration of five Dependabot patch updates (PRs #234–238) with exact
   owned exceptions for their new runtime identities.

Neither decision signs off a source audit or releases the product.

## Why action was required

From October 21 the inventory gate (`dependency_inventory.py gate`, run by the
CI `Dependency inventory gate` job with the current UTC date) reported all 34
owned exceptions as stale, because each had `review_by: 2026-10-20`. Every pull
request failed that gate whatever it changed. Separately, PR #244 changed
`Cargo.toml` without refreshing `supply-chain/dependency-graph.json`, so the
graph check reported a stale manifest digest.

## Decision 1: renewal of 34 existing exceptions

The owner renewed the 34 entries in `supply-chain/dependency-exceptions.json`
without changing any identity, checksum, criteria or owner:

- `review_by` moves from 2026-10-20 to **2026-11-07**.
- `expires_on` moves from 2026-11-05 to **2026-11-23**.
- `created_on` stays 2026-10-06, so every term stays within the 90-day
  `max_exception_days` policy.
- `approval_reference` points to this packet. Each rationale keeps its
  original text and names its original approval:
  [initial decision](2026-10-06-merge-dependency-decisions.md) (21 entries) or
  [update decision](2026-10-06-merge-dependency-update-decisions.md) (13 entries).

The renewed identities are unchanged from those packets. Their full-rooted
source audits remain outstanding.

## Decision 2: Dependabot patch updates

The owner approved these lockfile-only patch updates. They add no crate names,
change no `Cargo.toml` requirement and add no direct dependency.

All identities use source `registry+https://github.com/rust-lang/crates.io-index`.
Owner GitHub `brianluby`; created October 24, review by November 7, expiry
November 23, 2026.

| Crate | Version | Criteria | PR | Archive SHA-256 |
| --- | --- | --- | --- | --- |
| hyper-util | 0.1.20 → 0.1.21 | safe-to-deploy | #235 | `ddc03d96684f9226b8a787cdb71488417b53ab5ea8fdb1dac946cb9431cc8bff` |
| serde | 1.0.228 → 1.0.229 | safe-to-deploy | #238 | `4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba` |
| serde_core | 1.0.228 → 1.0.229 | safe-to-deploy | #238 | `67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48` |
| serde_derive | 1.0.228 → 1.0.229 | safe-to-run + safe-to-deploy (generated code) | #238 | `e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348` |
| tokio | 1.53.1 → 1.53.2 | safe-to-deploy | #237 | `e95f91fcc7a621e8b030f6aa23c71fe9838ae2fb4d8118b75602a328f5144044` |
| toml | 1.1.5+spec-1.1.0 → 1.1.6+spec-1.1.0 | safe-to-deploy | #236 | `920602543f0911ab71da12c50d59701da54c196d1a2bf5cb4b75667f137a406a` |

`insta` 1.48.0 → 1.49.0 (PR #234, checksum
`b67d3d2e287e4b86c10b3f3b641033d1f89b74bdb39d05f34952e2b9a6fe21cd`) is a
development dependency outside the runtime closure. It receives only a native
cargo-vet `safe-to-run` exemption, matching its prior versions; no inventory
exception is needed.

Each new identity has a matching native cargo-vet exemption in
`supply-chain/config.toml` whose notes reference this packet. Older-version
exemptions remain until `cargo vet prune`.

## Evidence and limits

- `python3 scripts/dependency_graph.py check`: exit 0 after refresh.
- `python3 scripts/dependency_inventory.py gate --as-of 2026-10-24`: exit 0,
  outcome `clean`, no errors. It still passes on 2026-11-07 and returns exit 1
  from 2026-11-08, when the next review is due.
- `cargo vet --locked`: succeeded (2 fully audited, 1 partially audited,
  294 exempted).
- `validate-store` still reports the 294 legacy-unowned exemptions as invalid,
  exactly as it does on `main` before this change; that transitional state is
  unchanged and is not a CI gate.
- These are integration exceptions. They are not audits, and passing gates do
  not establish release readiness.

## Human decision receipt

In a Claude Code session on October 24, 2026, the assistant asked the owner to
approve two sets: renewal of the 34 existing exceptions (review date October 20
passed) and approval of the new versions serde 1.0.229, tokio 1.53.2,
toml 1.1.6, hyper-util 0.1.21 and insta 1.49.0 (test-only). It asked: "do you
approve both sets, just the renewal, or neither?" The owner answered **"Yes"**.
The specific review and expiry dates above were chosen by the assistant within
the existing 14-day review and 90-day policy limits; the owner can shorten them.
