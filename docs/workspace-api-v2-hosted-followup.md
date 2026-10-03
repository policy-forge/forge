# API v2 hosted receipt fixture follow-up

Hosted workspace run `37100568423` at requested head `702fab68` exposed one
synthetic fixture missed by the earlier maintained-client controls. The existing
receipt shutdown test constructs `RecordingWorkspace` without its initializer.
After selected-major client negotiation was added, that unstarted fixture lacked
`_api_prefix`; shutdown raised `AttributeError` before the delegated request could
exercise sticky accounting. All three platform API jobs failed receipt tooling
before building their tested binary. Their failed API receipts contain null
checkout/identity and all suites are not-run; they earn no API execution credit.

Initialize the synthetic fixture's explicit original `/api/v1` prefix. The
production client, negotiated-version checks, delegated errors, accounting latch
and existing assertions remain unchanged. The original failure was reproduced
locally and retained. All **51 receipt controls now pass**. The changed helper
retains its docstring, and all **four executable body statements** have observed
trace counts. A bounded independent source review found no material concern.

The [follow-up audit](plans/2026-10-02-f19-api-v2-receipt-fixture-verification.json)
binds the repaired source, actual trace/log and historical failure. All 16 Rust
source pins from the [initial API v2 coverage audit](workspace-api-v2-verification.md)
still match exactly. Its 1,958-test LLVM run and 105 selected documentation/entry
observations are unchanged evidence for those bytes; this fixture correction
does not replace them with new Rust coverage or claim whole-function bodies.

The enabled commit hook, immutable delivery and hosted successor must be bound
separately. In the original hosted workflow the Chrome and Windows-terminal jobs
reported success, while Linux network-denial verification remained incomplete.
These job outcomes do not establish the complete browser/platform matrix,
native accessibility, human or dependency-audit acceptance. Full S-3, S-6,
release provenance and the final integrated documentation review remain open.
