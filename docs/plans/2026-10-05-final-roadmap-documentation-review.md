# Final roadmap documentation review — 2026-10-05

The full-content documentation review is complete for the frozen delivered-work
scope. It covers current guides, historical evidence and retained third-party
records across verified `main` and open candidates. The corrections update
source availability, commands, API navigation, identity/traceability claims and
publication/error semantics. Product acceptance and the 2.0.0 release candidate
remain open.

## Scope and source availability

| Reviewed group | Logical pages | Selected variants | Distinct contents |
| --- | ---: | ---: | ---: |
| Current references | 119 | 204 | 174 |
| Historical/evidence records | 103 | 113 | 105 |
| Retained third-party records | 21 | 21 | 16 |
| Total frozen population | 243 | 338 | 295 |

The 338 variants contain 5,083,724 bytes; distinct contents contain 4,221,229
bytes. These are documentation counts, not test or coverage denominators.
Current-page ownership is A34, C45, E34 and Root6. Readers reviewed complete
selected page contents; their narrower source follow-ups and byte pins do not
constitute fresh independent approval of historical Rust implementations.

The [page ledger](2026-10-05-final-roadmap-documentation-review.json) records
exact commits, byte sizes and SHA-256 values. The original scope has 81 pages
present on `main` and 162 candidate-only pages. A 2026-10-06 02:29 UTC refresh
confirmed `main` at `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e` and 61 open PRs:
the original 60 candidates retained their heads, bases and draft states, and
[PR #220](https://github.com/policy-forge/forge/pull/220) was the added sibling.
This is a dated source snapshot, not a claim about future repository state.

The documentation composition is based on Authoring draft
`304b31f8ea3913eba4a7d776b833b9b3db4ad3a4`
([PR #219](https://github.com/policy-forge/forge/pull/219)). It changes 82 existing
Markdown pages and adds this record and its JSON ledger. The 84 composition
layers explicitly preserve the Authoring checkpoint once and then apply its
later sentence correction; the roadmap checkpoint layers are ordered.
All original requirement/Ready checkbox sequences and human decisions remain.
The 130 added relative-file link occurrences resolve in that source roster;
the composition check does not validate remote pages or Markdown anchors.

The original WI-1–WI-50 completion record remains historical. PR #146 was
verified merged on 2026-09-12; its merge commit
`24d356ef14887660c285d95b635bbe00db13c933` is an ancestor of the verified main.
That merge does not imply acceptance of every later PRD 062 requirement.

## Separate MCP and platform observations

The sibling [MCP guide](https://github.com/policy-forge/forge/blob/855b3347d855b09ef318fb83e6d37166a0b7ea4f/docs/mcp.md) and [PRD 067](https://github.com/policy-forge/forge/blob/855b3347d855b09ef318fb83e6d37166a0b7ea4f/docs/PRD/067-prd-read-only-mcp-governance-interface.md)
retain their F20-specific corrections on PR #220. They are reviewed pages,
but the F20 implementation and its two-page update are not copied into the
Authoring-based documentation branch. Two later F20 page variants were checked
separately from the 338-row baseline. The side-PR #175 native-export evidence
page is absent from this branch; its original record and separate correction
remain on that source. Export stdout/distinct-output qualification does not
establish explicit in-place atomic replacement; see the current
[export preservation guide](../export-preservation-verification.md).

At MCP head `7bd8a6f336632660952085aae3574c83a2612e2a`, Windows ordinary tests
recorded 3,500 passes, zero failures and three ignored tests. Strict Clippy then
failed on the new geometry reader's intentional use of the private non-Unix
directory-handle field. The conditional lint annotation was integrated as
`855b3347d855b09ef318fb83e6d37166a0b7ea4f`. Actual local required hooks passed
format, strict all-target lint and 3,699 tests with zero failures and three
existing ignored fixtures. The normal push and exact open/draft PR readback
succeeded. All ten workflow jobs were in progress at the later 02:57 UTC
readback; that snapshot supplies no Windows qualification.

The original MCP library LLVM interval remains 2,611 passing library tests,
with physical-file lines 16,275/18,206 (89.394%) and functions 1,385/1,557
(88.953%). It includes tests/legacy/derived code and has unmeasured branches.
The selected adjacent-Rustdoc census is 602/602 callable bodies and 104/104
named types; unchanged legacy gaps retain their separate denominator.
These original measurements were not rerun by the editorial review or the
conditional annotation. Required commit-hook results are separate evidence.

The retained supply-chain log at 7bd records twenty missing safe-to-deploy
audits and exit 255. The Linux prerequisite remains incomplete: its independent
stdlib-gate observation is unavailable, with all other fields null. Its tested
merge was `f8c6f4535241410b5f18bc4ad22dc7aeab144396`, distinct from the requested
head. Neither result provides dependency approval, a successful IP-denial
experiment or attempted-egress evidence. Later runs must retain their own
heads and observations; prior failures are not rewritten.

## Delivery and remaining gates

The isolated documentation slice changes no Rust source, test, schema, Cargo
manifest or typed API contract. Frozen composition replay passed before
application; Root verifies the applied bytes and runs the actual required hooks
before drafting. The draft description records that delivery's exact head,
validation and hosted status. No new assertion-mirroring tests or fresh Rust
coverage campaign are supplied by this documentation-only slice.

Existing Forge task #33 retains its earlier rolling correction scope. Task #62
tracks this distinct final review under the completion epic; delivery belongs
in In Review, with `done=false`. Tracker state is not acceptance.

Authentic owner/reviewer authority, independent client/corpus and baseline
evaluation, dependency/MSRV/platform qualification, security/privacy and manual
accessibility review, real-team/pilot measurements and release authorization
remain governed by their existing records. Unsupported MCP resource families
and index construction/publication, further review adapters and other unmet
Must/Should requirements are not completed by documentation. Every later source
change requires its own documentation follow-up; this dated review does not
close the full goal or authorize a merge or release.
