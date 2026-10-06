# F19 operation progress with browser recovery

This is the S4 browser consumer prerequisite, tracked by Forge task #26 under
F19 task #23. It integrates the committed checkpoint producer
`f9a74ce347718021b0af3385584b45c4671f9398` and the complete F05 browser lineage
`04bf2aac1a948c5b1380566d90a5aef04761c175`. The final local integration commit
must contain both exact ancestors. Draft PR delivery does not establish full S4
or F19 acceptance.

The browser describes measured progress as captured registrations. A supplied
1/101 or 101/101 pair says how many registered inputs were captured; it does not
mean preparation succeeded or output was saved. Absent/null counters remain
indeterminate, zero/zero remains reported facts, and completed greater than total
remains explicitly unreconciled. Existing nonnegative safe-integer validation is
preserved, including generic future-compatible 1001/1001 observations. The
producer's full-index limit of 1,000 registrations and the effect limit of 100
consumed inputs do not become new consumer schema limits.

Persistent rows, known-ID GET recovery, original-key/body retry before ID
acknowledgment, sticky cancellation acknowledgment, terminal/generation guards,
unsaved-edit preservation, explicit Review and queued preview-close focus come
from the retained F05 lineage. Server-authored unlock retry guidance keeps the
closed Error wire shape and the existing monotonic retry deadline. No API route,
schema field, role, dependency, authority or persistent state is added.

Fresh validation binds the complete integrated source and exact served embedded
JS/CSS, rather than treating historical producer or F05 results as new execution.
The full Rust suite, instrumented Rust scope, maintained client, whole-source
Node/V8 suite and writable/read-only native Chrome runs have separate overlapping
denominators. Before drafting, check selected and whole docstring inventories,
mapped zero/unmapped Rust lines, V8 zero/absent functions and unavailable branch
or MC/DC dimensions. Preserve the original receipts and all failed or missed
observations. Native API-counter and installed DOM tuples require actual responses;
supplied source-test states or an empty collector do not establish native progress.

Use the dedicated verification document and machine-readable evidence for exact
results. The native companion uses a separately prepared synthetic project and
actual UI actions; no production pause, state/counter injection or polling change
may create a positive result. A fast terminal operation or missed observation
remains incomplete evidence.

The mandatory formatting, strict Clippy and full-suite commit hook remains
required. Root owns shared Git, Cargo, tracker and PR operations. Historical F05
verification packets remain unchanged, and the source/doc conflict resolution is
recorded separately. This branch is stacked on the checkpoint producer; hosted
main-only CI does not automatically qualify its exact combined head.

Full S4, all six PRD062 Should Have requirements, S6 browser metadata export/raw-file
verification and confirmed writable import, platform/MSRV/offline/signed launcher,
security/privacy/assistive-technology/WCAG and owner/human acceptance remain open.
Every scoped Must/Should in PRDs055–069 stays in the goal. After implementation and
acceptance gates are satisfied, perform the user's full documentation review and
update against the immutable integrated release candidate, including README,
usage, architecture/CODEMAPS, schemas, contributor guidance, API, roadmap and
release/provenance material. This local plan and preparatory inventory do not
complete that final review. No GitHub PR merge, release publication or participant
contact is authorized by this slice.
