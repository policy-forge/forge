# Integrated documentation successor: staged working tree, 2026-10-03

This is a source and documentation checkpoint, not a final documentation gate,
a product release or an acceptance decision. It supersedes the **current-source
pointer** in the [earlier checkpoint](2026-10-03-integrated-documentation-checkpoint.md)
while preserving that checkpoint, its source bindings and every historical audit.
The per-file snapshots were read from the active development checkout. They do
not establish whole-tree immutability or refreshed remote, PR, CI or tracker status.

## Committed source and work in progress

The captured committed checkpoint `2064e2933844a827e13b9880c8cfefd1cb2184ee`
declares API1 **1.2.0/39** and API2 **2.3.0/57**. Separately captured working
source declares API2 **2.4.0/65**, with eight staged-source operations, the
`http_staged.rs` adapter, session-owned `source_transfers.rs`, committed private
manifest/part reads in `source_stream_reads.rs`, and updated normative/client/UI
feature gates. These working bytes supply no compiled, merged, hosted or native
result in this review. API contract versions are not product release versions.

The working-source guards admit captured inspection on 2.1/2.2/2.3/2.4,
metadata effects on 2.2/2.3/2.4, inline source on 2.3/2.4 and staged source only
on 2.4. Matching numeric-major bootstrap alone grants none of those features.
See [compatibility](../api/compatibility.md), [migration](../api/migration-v2.md),
[local workspace](../local-workspace.md), [inline source](../workspace-source-bundles.md)
and [staged source](../workspace-staged-source-bundles.md).

Bundle3's 1,048,429-byte artifact ceiling and 147-byte wrapper remain separate
from Bundle4's logical 10 MiB artifact, 32 KiB parts and at most 320 parts.
Complete planning remains at most 100 paths, capture at most 50 MiB, and shared
retention 20 MiB/256 entities with original 600-second lifetimes. Conservative
whole admission can refuse payloads below the logical artifact cap. Stage
transport never grants domain or restore authority. Complete preview, separate
confirmation, durable accepted intent before 202 and known-ID recovery remain
separate. A 404 or lost response proves no absence of writes and authorizes no
automatic confirmation resend. Windows source restore remains typed unavailable;
external CLI/editor readers have no participating workspace transaction barrier.

## Full requirement scope

The captured PRDs 055–069 contain **327 unique Must/Should identities**, including
conditional Shoulds, and **112 checked source boxes**. The retained F01 ledger is
bound to its dated `aef0ab24` baseline: 198 technical-evidence, 28 partial,
8 conditional and 93 missing rows, with **327 acceptance entries open**.
Those counts are preserved history, not a fresh per-requirement execution audit.
Checked boxes, a local source change, a draft PR and merged technical work are
separate facts; none supplies aggregate participant, owner, audit or release
acceptance. Specific satisfied owner dispositions already recorded in
[authoring gates](../authoring-gates.md) remain valid within their own scope;
this checkpoint does not reopen them or invent an additional owner prerequisite.

The primary F01–F22 execution plan and requirement ledger remain separately
retained inputs rather than files in this source stack. Their reviewed versions
and an updated per-requirement evidence map still need explicit integration.
A link to an older checkout must not be presented as the final integrated tree.

| Package | Full scope to retain and reconcile |
| --- | --- |
| F01 | Every Must/Should identity, current evidence and named owner decisions; no automatic acceptance from checked boxes |
| F02 | Deterministic dependency inventory, runtime/build graph, owned exceptions, stale reporting and policy disposition |
| F03 | Attributable exact-version runtime dependency audits, reviewed imports and current advisory/CI evidence |
| F04 | Agreed hosted API/browser/platform matrix and packaged-runtime OS network-denial evidence; page blocking is a different observation |
| F05 | Independent security review and blocker disposition, keyboard/screen-reader/WCAG 2.2 AA evaluation and stated transaction limits |
| F06 | Lawful human-adjudicated task/injection corpora, frozen labels and thresholds, authentic evaluation and task selection |
| F07 | Official POA&M schema and bounded, explicitly selected, source-bound deterministic scaffold foundation |
| F08 | All POA&M Musts and S-1–S-4: status/ownership/milestones/closure/as-of/baseline/portfolio and handoff semantics |
| F09 | Approved per-model evidence overlay and lossless round trips preserving unrelated model data; linkage is not evidence sufficiency |
| F10 | Reviewed-risk handoff to POA&M and multiple result epochs with identity/source/baseline semantics |
| F11 | Discovery/privacy threat model, typed read-only stdio MCP, two real clients and separate adjudicated agent/raw-document corpus |
| F12 | Portable hash-bound review snapshots and responses, assignment/quorum/staleness/conflict/separation, first adapters |
| F13 | Remaining review adapters, edits, notifications/supersession, workspace response client and approved signed-envelope boundary |
| F14 | Connector-neutral plan/apply/reconcile and GitHub with credentials/preconditions/idempotency/conflict/resumable receipts |
| F15 | GitLab/Jira, nonauthoritative remote observations and signed connector permissions/data-residency decisions |
| F16 | Qualified executable-byte identity, filesystem/network confinement, bounded descendant ownership/cleanup on supported platforms |
| F17 | Both mapping and drafting tasks, context/citations/assumptions, quarantine and explicitly unapproved proposed patches |
| F18 | Batch ordering, source/candidate/diff review and expired-input re-evaluation without transferring approval |
| F19 | Every workspace S-1–S-6: static exports, guided authoring, captured inspection, progress/cancel/restart, platform launch and complete bundle workflow |
| F20 | MCP evidence metadata/reports, owner-bound visibility and contained optional indexing, with evidence bytes excluded |
| F21 | Real combined policy workflows, five-user study/manual accessibility, independent OSCAL interoperability, MCP/client/corpus, review conflicts, connector retries and suggestions |
| F22 | Exact candidate CI/advisory/audit/security, migration/help/examples, platform package/install/provenance and actual release dispositions |

This is a scope table, not a completed-package count or a declaration that every
unmeasured feature is absent. Requirement and conditional-Should scope is retained.
The primary plan's public Rust release decision **2.0.0** is separate from local
workspace contracts 2.3 and 2.4.

## Separate draft branches

Current OSCAL inventory remains **11 assets: 7 runtime JSON schemas and 4 test
XSDs**. The retained F07 POA&M foundation draft has an additional schema and
module absent at the two observed current paths; it is not part of this stack.
Current typed export remains Catalog/Component Definition with its documented
fidelity limitation. The retained F09 lossless-export/strict-parser draft differs
from those current source files and does not close F09's full overlay scope.
The retained F10 assessment-baseline draft also differs from current source;
[Assessment Results](../assessment-results.md) still describes its bounded
single-epoch command. The source-equal F12 applicability-counter guard is a
limited prerequisite, not the portable review/quorum/adapters or collaborative
acceptance scope. These are exact retained-source comparisons, not current
remote merge or PR-status claims.

Existing [evidence linkage](../evidence-linkage.md) preserves the deferred
per-model overlay boundary. [Suggestion gates](../authoring-gates.md#gate-suggest--prd-066-local-suggestion-pipeline)
retain recorded-response mode and unqualified process execution; workspace
read APIs do not establish MCP, connector, provider or confined-model delivery.
Separate draft packets, fakes and source presence establish no participant or
independent interoperability result.

## Review and remaining gates

Keep September roadmap tables, original WI-1–WI-50 history and all dated
verification records byte-for-byte. Current guide corrections reconcile only
source pointers, declared feature compatibility and recorded gate context.
They change no PRD identity/checkbox, API1 contract or owner decision.

A later final integrated review must bind the actual integrated source and
accepted evidence for README/install, CLI help/Rustdoc, maintained examples,
architecture/codemaps, API/client/UI/schema/model/package relationships,
dependency/provenance claims and all user guides. Authentic scoped runs,
repeated executions, raw zero/unmapped coverage, mock DOM, actual browser,
native platform, screen reader and human results remain separate dimensions.
This review runs none of them and approves no previously authored source or
workflow. Root owns integration, actual checks and the mandatory enabled hook.
Full F01–F22 completion, all Must/Should acceptance, platform/crash/capacity,
AT/human, audit, release and final documentation gates remain open.
