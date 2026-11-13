# PR 264: quick-xml 0.41.0 → 0.42.0 (MINOR) — RED FLAG: does not compile

This records a bounded agent-assisted differential review at the immutable head
in [receipt.json](receipt.json), base
`b4226bb35d796222c30f93235ac22118dd26a344`. The PR is OPEN; this note is not a
merge claim, an accepted source audit, or human signoff. No Cargo manifest,
lockfile or supply-chain policy was changed by this review.

**Bottom line: the candidate breaks Forge's build.** 22 compile errors in
`src/cli/export.rs`, `src/ingest/mod.rs` and `src/oscal_cli/invoker.rs`,
reproduced locally and confirmed by all three hosted Test jobs. This PR cannot
be merged as-is; it needs a source migration that is outside this review's
scope.

## Exact transition

Root `Cargo.lock` **and** `Cargo.toml` (`quick-xml = "0.41"` → `"0.42"`,
`features = ["serialize"]` preserved). See [Cargo.lock.diff](Cargo.lock.diff)
and [lock-transitions.json](lock-transitions.json):

| Package | Old | New | New checksum |
|---|---|---|---|
| quick-xml | 0.41.0 | 0.42.0 | `41b1177fdf999d2321d3fb46ff47159d9c1fb9ad66a4879f8c50a0b504615e9b` |

Old checksum `e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1`.

Declared MSRV rises **1.79 → 1.86** and edition 2021 → 2024; both are below
Forge's required 1.88, so the toolchain is not the blocker.

## Breaking API change (root cause of the build failure)

quick-xml 0.42 moves names and attribute values from bytes to validated UTF-8
strings:

- `QName`/`LocalName`/`Prefix` now wrap `&str`; `AsRef<str>` replaces the old
  byte-slice access (`src/name.rs`).
- `Attribute.value` is `Cow<'a, str>` instead of `Cow<'a, [u8]>`;
  `Attribute.key` is `QName` with str semantics (`src/events/attributes.rs`).
- `BytesStart::name()` etc. return the str-backed `QName`.

Forge call sites that assume bytes therefore fail:

- `src/cli/export.rs:171,183,185,190` — `String::from_utf8_lossy(...as_ref())`
  and `== ...as_bytes()` comparisons on names/attribute values (4 errors).
- `src/ingest/mod.rs:224…` — `match element.name().as_ref() { b"w:p" => … }`
  byte-literal patterns against `&str` (17 errors).
- `src/oscal_cli/invoker.rs` — 1 further byte/str mismatch.

Full local build log: [tests/cargo-test-xml.txt](tests/cargo-test-xml.txt)
(`cargo test --locked` at the exact head, exit 101, `could not compile forge
(lib) due to 22 previous errors`). The requested XML round-trip / export test
files **could not be executed** because the library does not build; the hosted
Test jobs fail identically (below), so no behavioral verdict on the
round-trip is possible until a migration lands.

## Source delta review

Full delta: [deltas/quick-xml-source.diff](deltas/quick-xml-source.diff)
(~2.7 MB; the release rewrites most of the crate). Security-relevant shape:

- The bytes→str migration is itself a hardening move: names/attribute values
  are UTF-8-validated at parse time instead of being handed to callers as raw
  bytes. It is also the source of every Forge compile error.
- `escape.rs` rewritten (escape/unescape paths, char-ref parsing);
  `writer.rs` keeps the `Writer`/`write_event`/`create_element` surface Forge
  uses, so the serializer side (`src/export/xml_serializer.rs`) is largely
  source-compatible; the deserializer (`de/`) and reader were restructured.
- New `se/` + `de/` module split with resolver/text/var handling; entity
  resolution API (`from_str_with_resolver`, `EntityResolver`) present in both
  versions.
- Given the breadth, this delta is **not** fully inspectable in a bounded
  review; a full human audit is required regardless of the compile fix.

## Executed checks (throwaway worktree, macOS arm64, rustc 1.99.1)

- `cargo test --locked` (xml_catalog_test, export_format_pairs,
  export_integration, export_lossless_integration, integration_round_trip,
  round_trip_test, oscal_cli_round_trip, golden_file_tests,
  profile_golden_file_tests, golden_edge_case_tests): **build failure**, exit
  101 — see log above. No tests ran.
- `cargo audit --no-fetch` ([audit/cargo-audit.json](audit/cargo-audit.json)):
  same 2 baseline pulldown-cmark findings as base
  ([../audit-base.json](../audit-base.json)); no new advisory. Inventory gate
  drifts like every dependency bump.

## Hosted CI (exact head, run 41067183887, 2026-10-30)

[ci/jobs.jsonl](ci/jobs.jsonl), [ci/run.json](ci/run.json):

- Test (ubuntu/macos/windows): **FAIL** — same 22 compile errors
  (`cargo test --locked --verbose`, exit 101).
- Workspace API (3 OS) and Chrome/headless/terminal prerequisites (run
  41067183934): **fail** — downstream of the broken build.
- Dependency fixtures (3 OS): pass (they do not build the candidate).
- Supply-chain audits: **fail** — baseline RUSTSEC-2026-0414
  ([ci/supply-chain-excerpt.txt](ci/supply-chain-excerpt.txt)).
- Dependency inventory gate: **fail** — frozen-graph drift.

Note: `gh pr checks` rollup text can lag; the job-level conclusions above are
the authoritative record.

## Disposition

Agent-assisted review only; no human audit approval, no merge permission.
**Recommend holding this PR**: a `src/cli/export.rs` + `src/ingest/mod.rs` +
`src/oscal_cli/invoker.rs` migration to the str-based name/attribute API is a
prerequisite, followed by the full XML round-trip suite and a human audit of
the rewritten escape/reader paths. That migration is tracked on branch
`feat/quick-xml-0.42-migration` (worktree `forge-quickxml`, branched from this
base; not yet pushed at review time).