# Merge documentation reconciliation — October 6, 2026

This is a later integration observation. It preserves the October 5 full-content
review ledger, historical failures, populations and original source hashes.
The current merge train supplies its own review and qualification evidence;
these records are not combined into new coverage or acceptance denominators.

## Verified main integration

At this observation, main is `7643553dc4565731ee9acdaef7c62985db28fed2`.
The following checkpoints are merged, with actual merge parents and candidate
tree equality verified after each merge:

| Checkpoint | Integrated scope | Main merge commit |
| --- | --- | --- |
| [#223](https://github.com/policy-forge/forge/pull/223) | Dependency policy, applicability and evidence foundations | `54ccd6c68f5fb79d75de804f011e5dc399b84825` |
| [#224](https://github.com/policy-forge/forge/pull/224) | Core workspace and native report composition | `14351e790e1311f27ccbc488e1ccf57ad7d11b67` |
| [#225](https://github.com/policy-forge/forge/pull/225) | API 2.4 staged-source transfer and platform prerequisites | `f7e5846225367c30c8b227b2ab5d03cf1ac98175` |
| [#226](https://github.com/policy-forge/forge/pull/226) | Assessment Results and nonterminal POA&M workflows | `67d1d70554110c8b80c133f9c78c262e3823c9c5` |
| [#227](https://github.com/policy-forge/forge/pull/227) | Read-only MCP queries and App geometry | `139d8cb0b1b2a749796db9f714fc0d84524ff0bc` |
| [#228](https://github.com/policy-forge/forge/pull/228) | Portable, Lifecycle and saved-plan Authoring review | `dd028efec34fff6ffaa40f8f3826f21b1154660b` |
| [#229](https://github.com/policy-forge/forge/pull/229) | Read-only suggestion-evaluation preflight | `7643553dc4565731ee9acdaef7c62985db28fed2` |

The earlier #219 and #220 open-draft paragraphs describe their dated snapshots.
Their implementations are now incorporated through #228 and #227 respectively;
#215–219 and #214/#220 were closed after exact-head ancestry verification.
GitHub marked original #171 merged when #229 incorporated its exact head.
Branches and historical evidence remain retained. Published release availability
is separate: no tag or release was created by this merge train.

## Qualification scope

Final #228 head `2a42338485c6eb1df268ea1ee24a4ef1ab668f3b` passed all 22
checks, native CI run `37494996802` and Workspace run `37494996881`.
Its normal hooks passed 4,387 tests, zero failures and three existing ignored
fixtures. Separate all-feature tests and strict all-target/all-feature Clippy
passed on its composition parent, with Rust/schema/Cargo/workflow bytes unchanged
in the final documentation successor.

Final #229 head `d1fdbb27593c359a72706f6a031e0453e282c80b` passed all 22
checks, native CI run `37497012597` and Workspace run `37497012586`.
Normal hooks and separate all-feature tests each passed 4,421 tests, zero failures
and three existing ignored fixtures; strict all-target/all-feature Clippy passed.
These overlapping test populations are not added together.

Each of those Workspace runs supplied 17 fresh artifacts. Six principal receipts
and 15 additional format rows were replayed against maintained validators and
actual producer/source pins, tool/release identities, exits and natural cleanup.
Each run keeps its own tested merge and ordered parents; earlier failures and
receipts are not relabeled. Four final-head CodeQL language analyses for each PR
completed without errors or results; existing main findings remain separate.
Native denial and Chrome/Windows prerequisite evidence retains its measured
profile. It does not establish arbitrary attempted-egress or product acceptance.

[PR #230](https://github.com/policy-forge/forge/pull/230) separately integrates
reporting-only Linux link-chain diagnostics at
`a3455528a2b838e4e15ec58e3b46bf870c1f0a28`; its fresh hosted qualification and
merge remain pending at this snapshot. This documentation candidate is composed
on that source and preserves #222's historical support-root note.
The [diagnostic integration note](2026-10-06-link-chain-integration.md) retains
initial pin/fixture failures and corrected 46 link/31 stat-control results.

## Preserved boundaries

The [October 5 documentation ledger](2026-10-05-final-roadmap-documentation-review.json)
remains byte-for-byte intact. All 338 historical variants and 82 corrected-page
before/candidate pins replayed successfully against their recorded Git objects:
243 logical pages, 295 distinct contents, 5,083,724 total bytes and 4,221,229
distinct-content bytes. This is integrity replay, not a new full-content reading
claim or fresh historical test/coverage qualification. The coordinator reviewed
this incoming 84-path documentation diff and its integration changes separately.

The owner-approved [incremental dependency policy and 21 exact exceptions](2026-10-06-merge-dependency-decisions.md)
permit integration within their existing scope and dates. They do not establish
full source audits, strict-store acceptance or approval of newer identities.
The five pending dependency-update PRs require their own graph and audit records.
Manifest `rust-version = "1.85"` is a declared minimum, not measured MSRV support.

Original requirement/Ready checkboxes, owner decisions, tracker acceptance,
D064 terminal/reopening refusal, independent client/corpus and baseline evaluation,
security/privacy, manual accessibility, real-team/pilot and release gates retain
their existing authority. This implementation-merge work does not close the full
product roadmap or qualify unsupported commands, capacities or platforms.
