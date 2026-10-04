# Integrated JSON/YAML export preservation verification

The CLI now validates and emits the complete decoded Catalog or Component
Definition tree when converting JSON/YAML input to JSON/YAML output. Native
metadata, statements, capabilities, protocol ranges, back-matter, extension
fields and array order no longer disappear through the partial generation
structs. Duplicate keys, invalid schema data and unsupported numeric
representations are refused before output publication.

This integrates the prerequisite retained in draft #175 at
`512046e26650088a3d7048facd4a285c915579e7` onto the current stack at
`309b5e5f7b8265f5adec4e959152d26b44c3dfc9`, including the F10 baseline guard.
Current CLI, workspace and documentation changes are preserved; old draft
execution receipts remain historical. See the [compatibility contract](OSCAL_COMPATIBILITY.md),
[integration plan](plans/2026-10-03-f09-lossless-prerequisite-rebase.md) and
[source-bound report](plans/2026-10-03-f09-export-integration-verification.json).
The report was derived before drafting and uses format
`forge.f09-integrated-export-verification/1`. Its SHA-256 is `91c9c46eccf8be8cd9957f3618ffea1522284f0f555ac1485862dab7748b8fbf`.

## Numeric and representation boundary

Export admits only exact i64/u64 numeric representations. It refuses every
floating-point representation, including harmless `1.0` or `1e0`, port
`443.0`/`443e0`, high-precision decimal values, and JSON integer overflow
decoded through floating point. Numeric strings keep string semantics.
The pinned YAML decoder may treat an enormous untagged integer-looking scalar
as a string; the guard does not reinterpret it or claim universal lexical
numeric classification. Explicit `!!int` overflow remains a decoder refusal.
OSCAL schema and semantic validation can impose narrower value limits.

The existing offline OSCAL 1.2.3 validator and declarations 1.2.0 through 1.2.3
remain in use. The 50 MiB regular-file input limit is unchanged. Duplicate-safe
decoding limits decoded depth to 100 and each string/key to raw input length.
Those limits are checked after decoding; they do not establish allocation,
time or alias-expansion confinement. YAML URI-form tags can be erased by its
decoder. Original bytes, object ordering, YAML comments/tags/anchors/style and
arbitrary numeric-token exactness are not preservation claims. XML and direct
typed helper APIs retain their existing partial model projections.

Input preservation assertions use stdout or a distinct destination. Explicitly
naming the input as `--output` retains the existing in-place rewrite behavior.

## Actual final-source checks

Rust 1.99.0 and the existing locked graph were used; no dependency was added.

| Check | Actual outcome |
| --- | --- |
| Rich regressions before applying the fix | 6 passed, 16 failed; failures reproduced dropped fields, invalid input admission and numeric projection |
| Export unit controls | 35 passed, zero failed/ignored |
| Shared strict parser controls | 12 passed, zero failed/ignored |
| Rich CLI integration suite | 22 passed, zero failed/ignored |
| Strict all-target/all-feature Clippy | Passed with `-D warnings` |
| Final formatting | `cargo fmt --check` passed |
| Fresh whole-workspace/all-feature LLVM run | 3,029 passed, zero failed, three ignored across 69 summaries |

Rich synthetic Catalog and Component fixtures compare the complete decoded
tree through JSON to JSON and JSON to YAML to JSON. Controls also cover array
positions, exact integer boundaries, duplicate and non-string YAML keys,
invalid native fields, duplicate JSON keys, unsupported models and numeric
refusals. Rejection controls require unchanged source and absent/existing
sentinel destinations, with no stdout publication. These are actual synthetic
regressions; they do not establish independent OSCAL interoperability or
authentic assessor/evidence workflows.

All six passing jobs record identical before/after pins for the nine integrated
source, fixture and document paths. The initial formatting failure and original
16 regression failures remain preserved without pass credit. Earlier
pre-format focused runs are separate history. The enabled commit hook and
maintained full local CI are delivery checks recorded separately in the draft
description; this guide does not predict their outcomes. Per-job repeated
tests are not added to a unique-test denominator.

## Documentation and measured coverage

All 57 selected changed/introduced declarations have adjacent Rustdoc. Exact
native filename, physical declaration line and first header token's UTF-8 byte
column map 56 to positive first regular LLVM CodeRegion entries, one to zero,
and none to an unmapped row. The selected zero is
`StrictObjectKeyVisitor::visit_string`. All 77 matching native instances remain
recorded, including four zero instances; positive headers do not establish
every instantiation, function body or branch was exercised.

The complete three-file lexical cohort has 118 named declarations, 68 with
Rustdoc and 50 retained legacy omissions. Its header observations are 114
positive, four zero and none unmapped. The other zero headers are the existing
`StrictValueVisitor::visit_string`, `visit_none` and `visit_some`. Literal
`#[test]` counts are 35 in the export unit module, 12 in the strict parser and
22 in the integration file; these counts are separate from helpers and from
repeated job totals. Anonymous/generated bodies, types and fields are outside
this named-function documentation denominator.

The raw native export reports 6,560/7,699 covered functions and 72,755/79,981
covered lines. Export's native line summary is 509/605; the strict parser's
is 287/303. LLVM supplies no separate integration-file line summary in this
export. These mixed production/test/generated summaries are not repository
production-only coverage. Every raw zero-function index, header alias and
available file summary is retained. Branch and MC/DC denominators are zero
and unavailable. No binary/profile/process-wide writer or timestamp
qualification is inferred from source snapshots or pure report replay.

## Remaining goal work

PRD 060 S-3 and full F09 remain open: actual overlay emission, supported
models/representations, collision handling, evidence-byte exclusion, scope
authority, linkage freshness and independent per-model interoperability need
their own implementation and evidence. Floating/wider numeric support,
XML fidelity and YAML decoder resource qualification are separate work.

Dependency/owner decisions, qualified Linux OS denial, platform/crash/rollback,
independent security/privacy, assistive-technology/participant judgments and
verified 2.0.0 release provenance remain separate gates. The full integrated
documentation review across all completed roadmap work remains open. Tests,
coverage observations and draft PRs confer no task closure, acceptance, merge
or publication.
