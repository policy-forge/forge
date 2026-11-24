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

## `jsonschema` 0.61.1 → 0.61.2 (Dependabot PR #299)

- **Transition.** Lockfile only; `Cargo.toml` unchanged; one identity moves
  (archive checksum in the packet). `main` at `7cc0dbd` was merged into the
  Dependabot branch without conflicts, and `cargo fetch --locked` accepted the
  merged lockfile.
- **Source examination.** The crate ships no changelog, so this is read from
  `diff -ru` of `src/` (1328 added and 92 removed lines across 13 files) and
  the published manifest. It is a validation-performance change:
  - `$ref` targets are resolved through a cache filled on first use
    (`CachedTarget`, `RefTarget`, `lookup_cached_target`; `keywords/ref_.rs`,
    `node.rs`).
  - `allOf` branches that only check object members are merged into one pass
    over the object (`ObjectBranches`, `Merged`, `MergedNames`;
    `keywords/all_of.rs`, `properties.rs`).
  - `required` names are checked during the walk of a closed object
    (`additionalProperties: false`) using bit sets (`required_bits`,
    `CheckedByClosedObjectValidator`); the source states that only `is_valid`
    leans on that shortcut (`keywords/required.rs`,
    `keywords/additional_properties.rs`).
  - `unevaluatedProperties` tracks evaluated properties as a set plus
    conditional contributions (`Contributions`, `Part`;
    `keywords/unevaluated_properties.rs`, 333 added lines).
  - New tests in `tests/pattern_sequence.rs`. `unsafe` occurrences are
    unchanged at 2. The published manifest changes only the version and the
    versions of its helper crates; the locked `jsonschema-regex`,
    `jsonschema-value`, `referencing` 0.61.2 and `fancy-regex` 0.20.2 already
    satisfy them, so nothing else moves.
- **Exposure.** Forge uses `jsonschema` for runtime schema validation, so
  correctness, not only speed, depends on this crate. The full suite on the
  three CI platforms, including the validate and API contract tests, passes
  against 0.61.2.
- **Advisory scan.** `cargo audit` exits 0 with the three allowed unmaintained
  warnings; `cargo deny check` reports advisories, bans, licenses and sources ok.
- **Bookkeeping.** One new cargo-vet `safe-to-deploy` exemption and one exact
  owned exception for `jsonschema` 0.61.2. The owned record for `jsonschema`
  0.61.1, approved on 2026-11-14, is retired because that identity left
  `Cargo.lock`. The graph and the committed draft SBOM were regenerated.
- **Verification at the pushed head.** `cargo vet --locked` succeeded (298
  exempted); `dependency_graph.py check` qualified; `dependency_inventory.py
  gate` clean; `test_dependency_inventory.py` 47 passed;
  `test_generate_sbom.py` 21 passed; the pre-commit full suite passed. CI
  evidence is the run attached to this commit on PR #299.

## `clap` 4.6.7 → 4.6.8 with its proc-macro chain (Dependabot PR #296)

- **Transition.** Lockfile only; `Cargo.toml` unchanged. Five identities move
  (archive checksums in the packet): `clap` 4.6.8, `clap_derive` 4.6.8,
  `proc-macro2` 1.0.107, `quote` 1.0.47 and `syn` 3.0.10. `main` at
  `2b93cfa` was merged into the Dependabot branch without conflicts, and
  `cargo fetch --locked` accepted the merged lockfile.
- **Source examination.** None of these crates ships a changelog, so this is
  read from `diff -ru` of each crate's `src/` and published manifest.
  - `clap`: 42 changed lines, all doc comments in the tutorial and FAQ; no
    code change. The manifest pins `clap_derive` to exactly 4.6.8.
  - `clap_derive`: the generated `augment_args`, `augment_args_for_update`
    and subcommand code now assigns in place through a `mut` parameter instead
    of shadowing with `let`, and allows `clippy::self_assignment` on the
    generated code. Behaviour is unchanged. It requires `syn` 3.0.10,
    `quote` 1.0.47 and `proc-macro2` 1.0.107.
  - `syn` (55 added, 15 removed): a clearer error for an unexpected `<-`
    operator, a lookahead fix so a label followed by `::` is not parsed as a
    labelled expression, and a refactor of rightmost-subexpression precedence
    scanning. Parser fixes can change which code compiles; Forge's command
    line derives build and the full suite passes.
  - `quote` (146 added, 152 removed): doc links and macro documentation.
  - `proc-macro2` (7 added, 8 removed): doc links, `map_or` to `is_some_and`,
    one span-invalidation branch removed, and `rust-version` raised from 1.68
    to 1.71, below Forge's 1.88.
- **Exposure.** `clap` is Forge's command-line parser. The other four are
  build-time procedural-macro crates whose output is compiled into Forge;
  they sit outside the runtime closure for the inventory.
- **Advisory scan.** `cargo audit` exits 0 with the three allowed unmaintained
  warnings; `cargo deny check` reports advisories, bans, licenses and sources ok.
- **Bookkeeping.** Five cargo-vet `safe-to-deploy` exemptions, because
  procedural macros generate deployed code. One exact owned exception for
  `clap` 4.6.8, the only new runtime identity the gate names. The owned
  records for `clap` 4.6.7, `clap_derive` 4.6.7 and `syn` 3.0.6 are retired
  with their identities. The graph and the committed draft SBOM were
  regenerated.
- **Verification at the pushed head.** `cargo vet --locked` succeeded (298
  exempted); `dependency_graph.py check` qualified; `dependency_inventory.py
  gate` clean; `test_dependency_inventory.py` 47 passed;
  `test_generate_sbom.py` 21 passed; the pre-commit full suite passed. CI
  evidence is the run attached to this commit on PR #296.
