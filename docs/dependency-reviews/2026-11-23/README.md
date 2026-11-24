# Dependency reviews — November 23, 2026

Approval: [the November 23 decision packet](../../plans/2026-11-23-dependency-renewal-and-bump-decisions.md).
Each section records the exact lockfile transition, a source examination of the
changed crates, the advisory scan and the verification for one change made under
that packet. These notes are evidence for the owner's decision. They are not a
source-audit signoff and grant no release acceptance.

## `url` 2.5.8 → 2.5.9 and `form_urlencoded` 1.2.2 → 1.2.3

Pull request: the upkeep change that also renews the 40 stale exceptions, cited in the packet's merge commit on `main`.

- **Transition.** `cargo update -p url --precise 2.5.9`. `Cargo.toml` is
  unchanged and no Forge source changed. Two lock identities move; archive
  checksums are in the packet.
- **Source examination** (`diff -ru` of `src/` between the crate sources in the
  local registry).
  - `url`: 44 added and 53 removed lines across `lib.rs`, `origin.rs`,
    `parser.rs` and `path_segments.rs`.
    - `origin.rs` carries the RUSTSEC-2026-0592 fix: for a `blob:` URL the origin
      of the inner URL is taken only when its scheme is `http`, `https` or
      `file`; every other inner scheme yields an opaque origin, so a nested
      `blob:` URL no longer recurses.
    - `parser.rs` is refactoring with no intended behaviour change: `filter`
      in place of a `match`, simpler character iteration, simplified file-host
      handling, and the path-segment pop restructured to avoid quadratic parsing
      of repeated `..` segments.
    - `path_segments.rs` re-expresses the `PATH` percent-encode set from
      `QUERY`. Its members are unchanged.
    - `unsafe` occurs once in each version. The manifest has no dependency,
      feature or build-script change. Standards test data was updated
      (`urltestdata.json`, `setters_tests.json`, `expected_failures.txt`,
      `unit.rs`, `wpt.rs`).
  - `form_urlencoded`: `Target::parse` clamps a `start_position` beyond the
    target length instead of panicking. No published dependency change.
- **Exposure.** Forge does not call `Url::origin()` and does not build `blob:`
  URLs; `url` is used in four source files. The advisory did not expose Forge,
  and `cargo audit` still required the fix.
- **Advisory scan.** `cargo audit` exits 0 with the three allowed unmaintained
  warnings (`chrono`, `ttf-parser`, `unsafe-libyaml`). `cargo deny check`
  reports advisories, bans, licenses and sources ok.
- **Renewal.** The 40 stale exceptions changed only `review_by`, `expires_on`,
  `approval_reference` and one appended rationale sentence. The gate validates
  every date and identity.
- **Verification at the pushed head.** `cargo vet --locked` succeeded (2 fully
  audited, 1 partially audited, 298 exempted). `dependency_graph.py check`
  qualified. `dependency_inventory.py gate` is clean for 2026-11-23 and
  2026-11-24. `test_dependency_inventory.py` passed 47 tests.
  `test_generate_sbom.py` passed 21 tests after `generate_sbom.py` regenerated
  the committed draft `docs/release/sbom-2026-11-12.json` (301 components,
  as-of 2026-11-12). `validate-store` exits 2 on this branch exactly as it does
  on `main`, because it rejects every legacy unowned exemption; CI does not run
  it as a gate.

## `sha2` 0.11.0 → 0.11.1 (Dependabot PR #300)

- **Transition.** Lockfile only; `Cargo.toml` unchanged; one identity moves
  (archive checksum in the packet). `main` at `405c8d3` was merged into the
  Dependabot branch without conflicts, and `cargo fetch --locked` accepted the
  merged lockfile.
- **Source examination.** Changelog entry 0.11.1 (2026-10-12): the workaround
  for unaligned loads in the `riscv-zknh` backends is removed and those backends
  no longer require the `zbkb` or `zbb` target features; loading of round
  constants in the AArch64 backends is fixed to satisfy the Stacked Borrows
  model; the MSRV rises from 1.85 to 1.88, equal to Forge's `rust-version`.
  The source moves hardware backends into per-algorithm modules (`src/sha256/`,
  `src/sha512/`) and refactors state (de)serialisation onto `as_chunks` and
  `array::from_fn`. `unsafe` occurrences in `src/` fall from 59 to 39. The
  published manifest changes only version, `rust-version` and `readme`; no
  dependency, feature or build-script change.
- **Exposure.** Forge uses `sha2` in 22 source files for runtime hashing. The
  digest algorithm and outputs are unchanged.
- **Advisory scan.** `cargo audit` exits 0 with the three allowed unmaintained
  warnings; `cargo deny check` reports advisories, bans, licenses and sources ok.
- **Bookkeeping.** One new cargo-vet `safe-to-deploy` exemption and one exact
  owned exception for `sha2` 0.11.1. `sha2` 0.11.0 had no owned record, so
  nothing was retired. The graph and the committed draft SBOM were regenerated.
- **Verification at the pushed head.** `cargo vet --locked` succeeded (298
  exempted); `dependency_graph.py check` qualified; `dependency_inventory.py
  gate` clean for 2026-11-23 and 2026-11-24; `test_dependency_inventory.py` 47
  passed; `test_generate_sbom.py` 21 passed; the pre-commit full suite passed.
  CI evidence is the run attached to this commit on PR #300.

## `zeroize` 1.9.0 → 1.9.1 (Dependabot PR #297)

- **Transition.** Lockfile only; `Cargo.toml` unchanged; one identity moves
  (archive checksum in the packet). `main` at `d733632` was merged into the
  Dependabot branch without conflicts, and `cargo fetch --locked` accepted the
  merged lockfile.
- **Source examination.** Changelog entry 1.9.1 (2026-10-06), confirmed against
  `diff -ru` of `src/`:
  - `Zeroizing<Z>` no longer derives `Debug`; a manual implementation prints
    only the struct name, so a protected value cannot leak through a `Debug`
    or error message. This matters to Forge, which keeps the workspace
    session salt, tag and hash in `Zeroizing` (`src/workspace/session.rs`).
  - `Vec`'s zeroization zeroes spare capacity before `clear()`, so initialized
    elements are no longer zeroed a second time.
  - The internal `optimization_barrier` is removed; the x86 and AArch64 paths
    use volatile reads through `MaybeUninit` instead.
  - Files touched: `src/lib.rs`, `src/x86.rs`, `src/aarch64.rs`,
    `src/barrier.rs`, `tests/alloc.rs`. `unsafe` occurrences are unchanged at
    21. The published manifest changes only the version.
- **Exposure.** Forge uses `zeroize` in two source files for session secret
    handling.
- **Advisory scan.** `cargo audit` exits 0 with the three allowed unmaintained
  warnings; `cargo deny check` reports advisories, bans, licenses and sources ok.
- **Bookkeeping.** One new cargo-vet `safe-to-deploy` exemption and one exact
  owned exception for `zeroize` 1.9.1. `zeroize` 1.9.0 had no owned record, so
  nothing was retired. The graph and the committed draft SBOM were regenerated.
- **Verification at the pushed head.** `cargo vet --locked` succeeded (298
  exempted); `dependency_graph.py check` qualified; `dependency_inventory.py
  gate` clean; `test_dependency_inventory.py` 47 passed;
  `test_generate_sbom.py` 21 passed; the pre-commit full suite passed. CI
  evidence is the run attached to this commit on PR #297.
