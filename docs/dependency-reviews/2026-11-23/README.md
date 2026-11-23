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
