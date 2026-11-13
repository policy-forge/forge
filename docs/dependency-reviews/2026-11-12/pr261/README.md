# PR 261: hyper 1.11.1 → 1.12.0

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
| hyper | 1.11.1 | 1.12.0 | `2c3e324da4c95177d6291d4c8730197c0d1822f8a9766814a4a44fa5ab797c9c` |

`futures-channel 0.3.34` is removed from the graph (hyper's `http1` feature no
longer depends on it). Old hyper checksum
`27b501faa50e7a26c3d3560ca625132f4078a17771f4810baf70475ae48cbe43`.
Forge declares `hyper = { version = "1", features = ["server", "http1"] }`, so
the candidate resolves without a manifest edit. Declared MSRV stays 1.63
(Forge requires 1.88).

## Source delta review

Full delta: [deltas/hyper-source.diff](deltas/hyper-source.diff) (old→new
registry sources, `diff -ru`, `.cargo_vcs_info.json` omitted). Security-relevant
observations, bounded to Forge's server/http1 usage (`src/workspace/http.rs`):

- **Connection-header keep-alive fix** (`src/proto/h1/role.rs`): once any
  `Connection` line asks to close, a later `keep-alive` line can no longer undo
  it (`saw_connection_close` latch). Corrects a request-smuggling-adjacent
  keep-alive desync class; strictly tightening.
- **New opt-in `http1_max_header_size`** (`role.rs`, `io.rs`, `conn.rs`,
  `server/conn/http1.rs`): enforces a parsed-header byte cap when configured.
  Forge does not set it; default behavior unchanged. The old `TODO` placeholder
  on the expect-continue path is replaced by the real cap check.
- **Internal body channel rewrite** (`src/body/chan.rs` new, replaces
  `futures-channel` mpsc): hand-rolled `Arc<Mutex<State>>` + `AtomicWaker`
  channel; comment documents that an error must not displace already-accepted
  data. Concurrency-sensitive internal code; upstream tests were read in the
  delta, not executed here.
- Remainder: h2 ping/upgrade adjustments, `want` requirement raised 0.3 → 0.3.2
  (`want` is not in Forge's resolved graph — the feature that pulls it is
  inactive), lint-table and `finish_non_exhaustive` cosmetics. No new
  network-facing surface; the only graph change is the removed
  `futures-channel`.

No actionable regression for Forge's loopback workspace listener was identified
in this bounded inspection. The channel rewrite is the one area a full human
audit should concentrate on.

## Executed checks (throwaway worktree, macOS arm64, rustc 1.99.1)

- `cargo check --locked --lib` at the exact PR head: **pass**
  ([checks/cargo-check-lib.txt](checks/cargo-check-lib.txt)).
- `cargo audit --no-fetch` at the exact PR head
  ([audit/cargo-audit.json](audit/cargo-audit.json)): 2 vulnerabilities, both
  pre-existing baseline `pulldown-cmark 0.13.4` findings (RUSTSEC-2026-0433,
  RUSTSEC-2026-0414), identical to base
  ([../audit-base.json](../audit-base.json)). **No new advisory introduced or
  fixed by this bump.** Warnings: ttf-parser/unsafe-libyaml unmaintained,
  rustix 1.1.4 unsound (informational), all baseline.
- `python3 -B scripts/dependency_graph.py check`: **drift** (exit 1), as for
  every dependency bump against the frozen F02 inventory
  ([ci/dependency-graph-check.json](ci/dependency-graph-check.json)).

## Hosted CI (exact head, run 41067129491, 2026-10-30)

Authoritative job conclusions: [ci/jobs.jsonl](ci/jobs.jsonl),
[ci/run.json](ci/run.json).

- Test (ubuntu/macos/windows): **pass** — full suite on the candidate.
- Dependency fixtures (3 OS): pass.
- Supply-chain audits: **fail** — `cargo audit` stops on baseline
  RUSTSEC-2026-0414 (published 2026-10-18); deny/vet steps never ran
  ([ci/supply-chain-excerpt.txt](ci/supply-chain-excerpt.txt)).
- Dependency inventory gate: **fail** — frozen-graph drift (reproduced above).
- Workspace API (3 OS), Chrome/headless/terminal prerequisites (separate run
  41067129240): pass.

The advisory failures are baseline state handled on the P0 supply-chain branch,
not regressions of this PR. Alert provenance is recorded in the
[packet index](../README.md).

## Disposition

Agent-assisted review only; no human audit approval, no merge permission.
Behaviorally clean against Forge's test surface; full human audit of the new
body channel remains open. Predecessor hyper versions are exempted, not locally
source-audited; this differential cannot inherit that approval.