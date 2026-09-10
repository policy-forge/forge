# PRD-061 Phase 2 review and verification

This record describes technical checks, not human readiness, legal/content
acceptance, design-partner validation, pilot metrics or release approval.
PR #144's retained public Rust API compatibility gate remains pending.

## Independent complete-diff review

The three focused agents first completed read-only discovery. After interface
freeze and implementation they reviewed independent lanes: components reviewed
impact; impact reviewed components and shared PRD-059 rendering; presentation
and handoff reviewed all coordinator CLI, capture, rendering, reporting and
publication changes. The coordinator reviewed integration, tests, docs and the
complete diff. All Git operations remained coordinator-only.

| Finding | Disposition |
|---|---|
| Optional question markers could precede a component H2 | Fixed marker placement after the component establishes its section. CLI regression checks marker lies between the correct topic headings. |
| Aggregate output checks happened after serializers allocated output | Added remaining-budget serializers, HTML encoders, rendering limits and handoff record/receipt budgets; exact-fit and one-byte-short tests. |
| Second snapshot request pin reads could exceed the remaining input budget | Restrict the shared capture budget before those reads; independently validated snapshots count conservatively against one invocation budget. |
| Same-path new snapshot pins were checked after rendering | Pins now enter confined project/component capture itself, before parsing/planning/rendering. Post-capture checks remain defense in depth. |
| Answer expiry suppressed component output but appeared substantive | Separate blocked output/state changes from supplied source/value content changes. Regression covers block and unblock. |
| Review time without an expiry dependency was labeled an expiry change | Only relevant freshness dependencies affect that axis; review metadata remains provenance-only. |
| Component extension whitespace changes were invisible to impact | Capture the exact extension SHA and emit a global binding finding without falsely affecting identical local content. CLI and unit regressions. |
| Incomplete comparison identity omitted component selection | Exact request hash binds every finding and its policy/section references; closed failure-phase reasons avoid raw diagnostic disclosure. |
| Component schema guards differed from runtime | Align timestamp/nonblank/control-character guards and add parser/schema adversarial cases. |
| Component provenance span limits applied after allocation | Pass remaining span/output budgets into shared rendering and conversion before pushes. |
| Stale component answer pins failed the whole invocation | Treat a same-question record hash mismatch as a stale binding and block only dependent sections; retain expected/observed hashes. Wrong relationships and source/sidecar drift still fail closed. Existing Phase 1 human-clause pin behavior is preserved. |

The dependency manifest and lockfile remain unchanged; stale dependency-downgrade
advice from Phase 1 was not followed. Portable output labels use `/`. Linux/macOS
retain single no-replace directory publication; other platforms fail closed.

## External review intake and dispositions

OCR reviews each candidate's exact commit with explicit Rust/JSON/Markdown
selection, including changed tests. Terminal counts and exact-head hosted results
are recorded on the PR after completion; a preview or running session is not
completion evidence.

| Intake | Disposition |
|---|---|
| Effective rendering budget errors named global constants | Report caller-effective byte/span limits; assertions also verify rejected writes preserve buffers. |
| Repeated scans made long-line span projection quadratic | Replace projection with monotonic UTF-8 cursors and advance source columns from known literal/token lengths; Unicode, CRLF and empty-substitution regressions preserve exact provenance. |
| Repeated blocked-instance scan per section | Precompute the explicit blocked policy/topic pair set. |
| Separate title escaping could drift | Use the existing authoring heading escape helper in the component adapter. |
| Empty-substitution and literal-variant schema probes | Add direct fragment-versus-legacy zero-width evidence and literal unknown/null rejection cases. |
| Shared renderer should reject all sidecars containing defaults | Not adopted: PRD-059 deliberately supports defaults, while PRD-061 requires every declared parameter explicitly bound before calling it. Rejecting declared defaults would break consciously supplied selections. Document the pure renderer/caller boundary. |
| Extension bytes might evade final capture verification | Not present in integrated source: generation captures the extension through the prepared capture set before adapter parsing and verifies that set before publication. Document the pure adapter's capture requirement. |
| Constructor and positional-budget refactors | Optional refactors deferred; title-bearing composition and title-free fragments intentionally initialize different accounting. Tests protect effective bounds and existing bytes. |
| Comment-anchor probe | Non-finding: provider tooling probe contains no source defect. |
| Impact schema end anchors and byte/cardinality documentation | Harden actual-end/control checks, test newline/control suffixes and explain independent 2 MiB, record and UTF-8 byte limits. |
| Changelog/roadmap status dates, sequencing and gates | Add actionable CLI/contracts/platform notes, current reconciliation date, retire resolved Phase 1 sequencing question and retain API/human/release gates. |
| CLI help omitted new option descriptions | Add descriptions; retain frozen handoff shape with machine-readable `handoff.json` receipt in the mandatory generation. |
| Component/impact/handoff documentation absent in early code candidates | Already supplied by the following documentation commit; Phase 1 historical plans remain historical. Add direct extension-schema links in the authoring guide. |
| CLI attack and handoff privacy coverage gaps | Derive prior answer pins from fixtures; exercise build rejection without publication for every path attack; cover HTML option guards and plan-only views; assert every handoff non-policy artifact excludes rendered answer values. |
| Partial correspondence could hide surviving control changes | Require nonempty pairs and explicit coverage of surviving IDs, report true control additions/removals, and index pack control/topic dependencies without current gaps. Unsupported comparisons have no unaffected scopes. |
| Missing component answer edges and unknown question references | Validate known question relationships while retaining declared question dependencies for missing answers; blocked context stays scoped. |
| Empty unverified report hashes and unbudgeted scope records | Use explicit null for incomplete comparisons and reserve scope/ID evidence before allocation. |
| Same-project component location failure bypassed incomplete report | Keep the failure inside the snapshot result so it follows the closed incomplete-report path. |
| Component could override or omit topic heading | False positive: `components::validate_structure` requires the component source's first line to equal `## <escaped topic title>` before fragment preparation, including blocked sections; shared helper and heading regressions enforce this. |
| Repeated component search in rendering and section hashes | Build a borrowed policy/topic index once for each operation. |
| Inventory snapshot diagnostics omitted OS cause | Include the non-sensitive I/O error kind; retain private path and raw diagnostic suppression. |
| Handoff exit 1 after publication | Intentional existing authoring convention; explicitly document successful publication with remaining drafting work and verify receipt/output in CLI tests. |
| CodeRabbit: typed HTML paths lacked the shared raw-value guard | Apply the same bounded schema/redaction guard to plan/impact types; future nested raw-value regression covers both contracts. |
| CodeRabbit: vacuous spans and ambiguous overwrite regression | Assert the exact policy, nonempty spans and full Markdown partition; require existing-destination rejection while prior handoff inputs still exist. |
| Peer review: new no-gap edges could spend budget reserved for later edges | Reserve against the full planned dependency count before adding no-gap edges; exact-fit/overflow checks preserve the counter on rejection. |
| Final audit: failed old snapshot refunded unknown captured bytes | Conservatively reserve its unavailable partial-capture allowance; same/different-project CLI regressions prove the otherwise valid new snapshot remains unverified instead of spending the allowance again. |
| Copilot: plan-buffer copy before hashing | Hash the borrowed bytes directly, preserving the digest without the extra allocation. |
| Copilot: generic serializers used plan-specific errors | Use report wording for shared bounded serialization failures. |


## Final-candidate OCR follow-up

The exact `9099ef3` review prompted these additional source-validated changes.
Its terminal coverage and any provider limitations are recorded on the PR.

| Intake | Disposition |
|---|---|
| Changelog guide label, output inventory and API note | Link the complete Phase 2 contract table and add a Changed entry. Phase 2 changes `AuthorCommand` variants and fields; `ForgeError` is unchanged in this diff and remains part of the carried Phase 1 API gate. |
| Changelog publication novelty and opt-in values | Explicitly identify the reused Phase 1 publication primitive and public/internal substitutions into draft Markdown; report/HTML raw-value omission remains distinct. |
| Shared PRD-059 error diagnostics | Record effective-budget diagnostic wording changes; successful composition bytes and existing provenance semantics remain preserved. |
| Roadmap chronology and product/compliance gates | Restore chronological changelog rows and retain the same pending human gates in both roadmap views. |
| Evidence subject, digest terminology and first-line heading rule | Name the remediated test candidate, use lowercase-hex digest terminology and describe the exact first-line component H2 check. |
| Optional exact-resource correspondence lacked a direct case | Clarify that every supplied map replaces automatic matching and requires surviving-ID coverage; regression covers omitted, complete, partial and empty mappings with identical framework hashes. |
| Same-project component-location failures used the generic category | Mark the component validation phase before path confinement; CLI regression requires component-input findings for both same/different project locations and preserves private incomplete reports. |
| HTML typed-schema mismatch lacked a direct case | Add explicit mismatched-contract coverage through the typed adapter. |
| HTML's 16 KiB guard rejected valid imported control IDs | Reuse the existing 64 KiB imported-metadata limit, retaining depth and aggregate byte caps; exercise the real validated long-control planning path and over-limit rejection. |
| HTML DOM duplication | Streaming guard is an optional optimization: current serialized input is bounded before parsing and producer graph/cardinality limits remain enforced. The byte limit is not claimed as an exact peak-heap limit. |
| Additional PRD requirement checkboxes | Intentionally unchanged: this tranche closes M-13's technical evidence and explicitly preserves other acceptance/human gates. Expanding checkbox scope would claim evidence outside this delivery. |
| PRD revision history | Add a dated technical-disposition row without changing human acceptance status. |
| Hypothetical duplicate component index | Not reachable through supported callers: `render_with_components` and `section_hashes` are confined to the authoring module, generation calls `components::prepare`, and closed `/1` parsing rejects duplicate policy/topic pairs. Public construction of `LoadedComponents` cannot invoke those private rendering adapters. No last/first-wins behavior is supported. |
| Repeated HTML serialize/parse advice | Duplicate of the DOM optimization item; retain the runtime privacy/schema guard requested by the preceding review, with bounded producer inputs and the corrected imported-string ceiling. |
| Claimed `ErrorKind` formatting compile failure | False positive: `ErrorKind` has implemented Display since Rust 1.60, predating the declared MSRV ([standard-library documentation](https://doc.rust-lang.org/std/io/enum.ErrorKind.html#impl-Display-for-ErrorKind)). Exact-candidate locked clippy/tests and Linux/macOS/Windows CI passed; a standalone rustc program using the disputed `{}` formatting also compiled and printed `permission denied`. No source change needed. |
| Fragment default/span wording | Clarify shared grammar/default selection rules separately from intentional zero-width fragment spans; legacy composition semantics stay unchanged. |
| CLI exit helper assumed only one domain error variant | Check the application diagnostic prefix to exclude clap usage errors while allowing shared composition/lifecycle errors that also map to exit 2. Scenario-specific assertions continue to check the intended failure. |

## Verification boundary

The freshly fetched baseline passed 2,234 tests with three ignored. The integrated
suite passed 2,300 tests with three ignored after initial independent remediation,
before the external review fixes recorded above. The remediated candidate then
passed 2,322 tests with three ignored. The final OCR follow-up passed 2,325
tests with three ignored. Formatting,
strict locked all-target clippy and `git diff --check` passed. The final exact
commit, test counts, OCR terminal coverage and hosted CI/review dispositions are
recorded with the pull request. A skipped or incomplete bot review is not counted
as completed review. No merge or release is authorized.
