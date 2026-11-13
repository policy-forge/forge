# PR 263: serde_json 1.0.151 → 1.0.154

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), base
`b4226bb35d796222c30f93235ac22118dd26a344`. The PR is OPEN; this note is not a
merge claim, an accepted source audit, or human signoff. No Cargo manifest,
lockfile or supply-chain policy was changed by this review.

## Exact transition

Root `Cargo.lock` only (see [Cargo.lock.diff](Cargo.lock.diff) and
[lock-transitions.json](lock-transitions.json)):

| Package | Old | New | New checksum |
|---|---|---|---|
| serde_json | 1.0.151 | 1.0.154 | `e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6` |

Old checksum `c841b55ecdae098c80dcae9cf767f6f8a0c2cdb3416bbef72181df4d0fe73f14`.
Forge declares `serde_json = "1"` with default features; no
manifest edit, no feature change, declared MSRV stays 1.71 (Forge requires
1.88).

## Source delta review

Full delta: [deltas/serde_json-source.diff](deltas/serde_json-source.diff).
Security/behavior-relevant observations:

- **`-0` integer parsing fix** (`src/de.rs`): the integer fast path now
  excludes the literal `"-0"` so it parses as a float (`-0.0`) instead of the
  integer `0`. This is a value-representation correction at a trust boundary
  (untrusted JSON input). Forge's OSCAL documents do not rely on `-0`
  round-tripping as an integer; golden output is unchanged (below).
- **`number.rs` `as_f32`**: now filters non-finite results
  (`Some(n as f32).filter(|f| f.is_finite())`), returning `None` instead of an
  infinity when an `f64` overflows `f32`. Stricter, additive-safe accessor.
- **`map.rs` `preserve_order` hasher**: the `IndexMap` backend gains an
  explicit `BuildHasherImpl` — `std` `RandomState` under `std`, `foldhash`
  only under `no_std`. This adds an **optional** `foldhash 0.2` dependency
  gated behind `preserve_order` + `not(std)`. Forge uses default features
  (`std`, no `preserve_order`), so the BTreeMap backend is unchanged and
  `foldhash` is not activated by this bump (it already exists in the base
  graph via another crate; the lock diff adds no package).
- **`read.rs`**: improved error line/column reporting for invalid Unicode
  code points and `raw_value` buffering position tracking (diagnostics only).
  `lib.rs` is a doc-URL bump; `lexical/math.rs` is `max_value` → `MAX`
  modernization (no arithmetic change). No new unsafe.
- Upstream tests in the delta cover the `-0` case and float parsing; they were
  read, not executed here (Forge's own suite ran instead).

No actionable regression for Forge's serialization surface was identified in
this bounded inspection.

## Executed checks (throwaway worktree, macOS arm64, rustc 1.99.1)

- `cargo check --locked --lib` at the exact PR head: **pass**
  ([checks/cargo-check-lib.txt](checks/cargo-check-lib.txt)).
- `cargo test --locked` golden/determinism surface at the exact head
  ([tests/cargo-test-golden.txt](tests/cargo-test-golden.txt)):

| Test file | Result |
|---|---|
| golden_file_tests | 29 passed |
| golden_edge_case_tests | 14 passed |
| fixture_determinism_test | 7 passed |

**50 tests, 0 failures** — byte-stable JSON output and determinism hold on
1.0.154, so the `-0` correction does not shift any retained snapshot.

- `cargo audit --no-fetch` ([audit/cargo-audit.json](audit/cargo-audit.json)):
  same 2 baseline pulldown-cmark findings as base
  ([../audit-base.json](../audit-base.json)); no new advisory. Inventory gate
  drifts like every dependency bump (frozen F02 graph).

## Hosted CI (exact head, run 41067162528, 2026-10-30)

[ci/jobs.jsonl](ci/jobs.jsonl), [ci/run.json](ci/run.json):

- Test (ubuntu/macos/windows): **pass** — full suite on the candidate.
- Dependency fixtures (3 OS): pass.
- Supply-chain audits: **fail** — baseline RUSTSEC-2026-0414 stops `cargo
  audit` before deny/vet ([ci/supply-chain-excerpt.txt](ci/supply-chain-excerpt.txt)).
- Dependency inventory gate: **fail** — frozen-graph drift.

Baseline advisories are handled on the P0 supply-chain branch; provenance in
the [packet index](../README.md).

## Disposition

Agent-assisted review only; no human audit approval, no merge permission.
Lowest-risk of the five candidates: patch-level, byte-stable output verified,
one trust-boundary parsing correction that is strictly more correct. Full
human audit of the parser change remains the acceptance gate.