# F05: unlock feedback and confirmed-write focus

## Scope and baseline

Veans Forge child #16 is the bounded follow-up to draft PR #173 under active
F05 #7 and epic #2. The branch is `codex/f05-keyboard-completion`, based on
`b4207ea8280304c655cda2ad018f28324403a0d0`. It reuses the clean owned F05
worktree; the pushed live-state branch and its historical receipts remain intact.
The previous slice's preserved browser executable remains a separate artifact.

This slice addresses two concrete keyboard behaviors: unlocking silently disabled
its submit and sent typed throttling through a focus-stealing generic summary;
a verified write replaced the preview's invoking form without a deliberate
post-close destination. It also covers the saved-but-refresh-failed distinction
when an inline table error occurs while the confirmation modal is open.

No new crate, API field, authoritative decision, effect bypass or backoff policy
is introduced. Final publication still requires the exact confirmed receipt and
original idempotency key. A late UI completion does not cancel a confirmed write
or authorize replay; guards apply to installing state and moving focus.

## Unlock state

Initial load announces locked state and focuses the passphrase field. A pending
unlock announces its region state and uses a guarded unavailable submit action
without removing the focused native control. Repeated activation must not start
another request. Busy state clears when that one request settles.

A typed `unlock-throttled` error announces the supplied safe server message. It
preserves the owned retry-field focus; a late response cannot pull focus back from
another intentional target. Failed submissions still clear the credential field.
Ordinary unlock failures retain the focused safe summary. Successful unlock and
initial project-load failure keep their separate destination/error behavior.

The server computes remaining seconds from the existing monotonic `unlock_after`
deadline. Positive fractional intervals round up; expired intervals are zero.
The caller reports throttling only while the deadline is in the future. Existing
failed-attempt delays remain 2–64 seconds, with the existing successful-unlock
rate limit unchanged. The browser displays the message verbatim; it does not
estimate, parse a countdown or invent a local delay.

The private transport error stores static authored text by borrowing and only the
dedicated throttling constructor owns a bounded server-authored message. Code,
message, retryability and optional resource-version fields retain the closed
OpenAPI Error shape. No parser, filesystem or credential text enters the message.
Regression checks must validate both static and timed errors against that schema.

## Confirmed write and native close

A current verified successful commit refreshes its current destination, installs
the committed download action when applicable, and finishes the modal's queued
native close lifecycle before selecting heading focus. A verified write with a
failed refresh remains saved and focuses the connected refresh-error summary
after close. A rejected write keeps its current preview open and its error focused.

Preview/view/session ownership must survive the whole async path. Escape,
dismissal, navigation, a newer preview or shutdown revokes older UI publication
and focus. A current refresh may not install rows or errors after revocation.
A successor preview waits for the previous native close event before reuse, so
its listeners and content cannot be mistaken for the earlier modal.

A dismissed guarded refresh resumes only its connected installed pagers, adopting
the current monotonic view fence and advancing local read generations. Older
row/error/focus replies stay obsolete. A settled read releases controls only
while it owns the latest local sequence, so its finally cannot unlock a newer
request. Retained rows are explicitly previous results until a fresh read.

An already-ineligible background preview must return before reserving a new
preview generation or reading its receipt. Otherwise a late direct-preparation
reply while another modal is visible would silently revoke that visible modal's
confirmation. Initially eligible reads still need later generation and ownership
checks. Operation GET recovery and initial same-key replay remain separate paths.

## Verification to collect before drafting

Use whole-source Node regressions with the exact asset filename/hash, including
queued close behavior and disconnected invokers. Preserve all earlier tests and
add deferred field/submit unlock, duplicate activation, typed throttle and generic
failure cases; confirmed registration/export heading and native Tab continuation;
saved-but-refresh-failed global/inline error focus; rejection; late Escape,
navigation/new-preview/shutdown; and ineligible background preparation against a
visible still-confirmable preview. A fake DOM is not native browser or AT evidence.

Run installed Chrome through the maintained POSIX harness with exact served JS/CSS
and a separately preserved candidate binary. Label injected 429/refresh faults as
synthetic UI cases; a later actual unlock and real confirmed writes must succeed.
Keep exact runtime versions, logs, focus and reflow scopes, raw source coverage and
all earlier failures. Server rounding/expiry and schema compatibility tests are
separate from synthetic browser timing. No passed count or coverage is asserted
until its final candidate run completes.

Audit adjacent documentation for every changed/new named production and test
helper, distinguishing JS/Rust and anonymous/unchanged exclusions. Record actual
V8 and scoped LLVM observations with complete covered/uncovered denominators;
raw V8 ranges are not line/branch/condition coverage. Run required formatting,
strict default-feature all-target Clippy and full Rust hooks without bypass.
Root coordinates all Cargo processes and shared error/contract changes.

## Remaining gates

This supports the recorded locked/throttled/loading rules and deliberate
post-transition focus in PRD-062 accessibility requirements. It does not close
full keyboard golden-path coverage, screen-reader announcement timing, diagnostic
field/resource linkage, browser refresh/tab-close semantics, the supported
browser/platform/AT matrix, WCAG/ASVS assessment, security acceptance, dependency
audits, numeric/RSS/wall approvals, Windows qualification, real-user evaluation or
owner acceptance. Must/Should and conditional requirements retain their gates.

After all scoped roadmap work, review and update the complete integrated docs:
CLI/API, examples, architecture, schemas, dependencies/provenance, platform
support, verification instructions, requirements and acceptance status. This
focused plan and its eventual tests/draft PR do not satisfy the final docs gate.

## Observed development checks

The [keyboard verification successor](../workspace-keyboard-verification.md)
records 86/86 source tests, documentation for 16/16 selected JS and 8/8 Rust
functions, 13/13 test helpers, raw V8 and corrected Rust production-line
denominators, 47 library and 11 HTTP checks, and 60 writable / 5 read-only
Chrome focus observations against the preserved executable. Historical failures and fixture limits remain.
Required commit checks are reported separately by the hook/PR; this record does
not close normative accessibility, human or integration gates.
