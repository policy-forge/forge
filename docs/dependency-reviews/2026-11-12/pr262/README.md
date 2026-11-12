# PR 262: jsonschema 0.57.0 → 0.60.2 (MINOR)

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), base
`b4226bb35d796222c30f93235ac22118dd26a344`. The PR is OPEN; this note is not a
merge claim, an accepted source audit, or human signoff. No Cargo manifest,
lockfile or supply-chain policy was changed by this review.

## Exact transition

Root `Cargo.lock` **and** `Cargo.toml` (`jsonschema = "0.57"` → `"0.60"`,
`default-features = false` preserved). See [Cargo.lock.diff](Cargo.lock.diff)
and [lock-transitions.json](lock-transitions.json):

| Package | Old | New | New checksum |
|---|---|---|---|
| jsonschema | 0.57.0 | 0.60.2 | `a42a11b47492e6a2cc76e9c7bcc8077776cb31dac444e0795d7f61bcc6441c08` |
| jsonschema-regex | 0.57.0 | 0.60.2 | `107aeb6ff14b8deb5610096a8ce5c554ec89806b521147eaeb3cae05b07edb7a` |
| jsonschema-value | 0.57.0 | 0.60.2 | `e281d6b177bb4834f2ffd6f92aa0443cffe29c996b251c8d94eb6f456186feba` |
| referencing | 0.57.0 | 0.60.2 | `f42818036d3b37cda6bd4c494543158a36e431cb717015ef7c93acc6ba214d3b` |

`regex-syntax` stays at 0.8.11 (already in the base lock). Old checksums are in
the transitions file. Declared MSRV stays 1.85.0 (Forge requires 1.88). The
new optional `jsonb` feature (Postgres jsonb instances) is **not** enabled by
Forge's `default-features = false`; the default feature set (`resolve-http`,
`resolve-file`, `tls-aws-lc-rs`, `idna`) is unchanged between versions.

## Resolver / serialization behavior review

Forge compiles embedded OSCAL 1.2.3 **draft-07** schemas with
`jsonschema::validator_for` (`src/validate/mod.rs`), caches them in
`OnceLock`s, and formats errors via `schema_path`/`instance_path`
(`src/validate/formatter.rs`). All `$ref`s in the embedded schemas are
in-document; Forge never enables external resolution at validation time.
Offline-core invariant is therefore structurally preserved by this bump.

Deltas ([deltas/](deltas/)): [jsonschema](deltas/jsonschema-source.diff),
[referencing](deltas/referencing-source.diff),
[jsonschema-value](deltas/jsonschema-value-source.diff),
[jsonschema-regex](deltas/jsonschema-regex-source.diff). Security/behavior-
relevant observations:

- **referencing `find_vocabularies`**: now honors a subschema's own `$schema`
  declaration before falling back to the resource root's vocabulary. Affects
  draft/vocabulary selection for mixed-draft documents; Forge schemas are
  uniformly draft-07, so no expected effect.
- **referencing resolver**: new `at_uri` keeps dynamic scope when rebasing;
  anchor/URI handling refinements in `anchor.rs`/`registry.rs`. Resolution is
  still purely in-memory for Forge's usage.
- **`$ref` compilation** (`keywords/ref_.rs`): direct-ref targets now carry an
  absolute `target_location` and support deferred/recursive targets; error
  `schema_path` values for refs behind `$id` gain the resource prefix (upstream
  tests assert the new locations). Forge's formatter passes paths through;
  golden validation-error snapshots could shift for ref-heavy schemas — none
  did in the executed suite below.
- **New keywords machinery**: `discriminator`, `rfc3986` keyword modules,
  canonical-form algebra work, arbitrary-precision large-number tests gated
  behind a feature Forge does not enable. `unevaluated*`/`evaluate` remain
  unimplemented as before.
- **jsonschema-value**: representation-layer refactor supporting the optional
  jsonb backend; serde_json path used by Forge unchanged in shape.
- **jsonschema-regex**: small wrapper changes over regex-syntax (unchanged
  version); no new regex engine.

## Executed behavioral checks (throwaway worktree, macOS arm64, rustc 1.99.1)

`cargo test --locked` at the exact PR head against the candidate
([tests/cargo-test-validate.txt](tests/cargo-test-validate.txt)):

| Test file | Result |
|---|---|
| validate_test | 24 passed |
| api_contract_validation | 30 passed |
| schema_provenance_test | 11 passed |
| oscal_1_2_3_compatibility_test | 9 passed |
| fixture_validity_test | 2 passed |
| ocr_schema_regressions | 2 passed |

**78 tests, 0 failures** — embedded-schema compilation, validation verdicts,
error formatting and schema provenance all hold on 0.60.2.

`cargo audit --no-fetch` ([audit/cargo-audit.json](audit/cargo-audit.json)):
same 2 baseline pulldown-cmark findings as base
([../audit-base.json](../audit-base.json)); no new advisory. Inventory gate
drifts like every dependency bump (frozen F02 graph).

## Hosted CI (exact head, run 41067147939, 2026-10-30)

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
The minor bump passes Forge's entire embedded-validation surface, but the
resolver vocabulary precedence, ref-location error paths and the value-layer
refactor are real behavior deltas: a full human audit is still required before
any acceptance, and error-message snapshots should be re-checked if validation
output wording is contractual anywhere downstream.