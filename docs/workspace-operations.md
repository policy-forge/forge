# Workspace operation progress and recovery

Policy conversion, applicability analysis, mapping builds and report exports
return a session-owned operation when accepted. Preparation produces a preview;
the existing confirmed commit is the separate step that publishes a file.
The request/response schemas and API version remain unchanged.

## Observe the original operation

The acceptance reply has a stable `operation_id`. Use
`GET /api/v1/operations/{operation_id}` for current state. Retrying the same
method, path, raw query and parsed body with the original idempotency key returns
its original reply, which can still contain the original pending snapshot.
Retrying does not admit another worker or renew its deadline. A changed request
under that key remains a conflict.

During registered-file capture, `progress` contains `completed_items` and
`total_items`. The denominator is the full index, at most 1,000 registrations;
the numerator advances after a file is read, classified and installed in the
captured snapshot. Invalid classifications still count as captured. Failed reads
do not count. An empty index can report zero of zero. These units differ from the
100 consumed-input limit for a prepared effect.

Progress is null or absent outside the measured capture phase, and null after
background work settles. The producer uses a fixed denominator and a
nondecreasing completed prefix. Counts include no resource names, paths or
source content. Complete capture is followed by domain and output preparation;
it does not imply a successful operation, saved output or approval.

## Cancel preparation

`POST /api/v1/operations/{operation_id}/cancellation` acknowledges a request to
stop pending/running preparation. The acknowledgment can still be nonterminal
with `cancel_requested: true`; query until the worker settles. Cancelled work
discards local previews and receipts and publishes no project file. Successful,
failed and cancelled terminal states cannot be revived by a late worker update.
Cancellation of a terminal operation returns `operation-not-cancellable`.

One monotonic 30-second budget starts immediately after operation acceptance and
includes any wait before worker entry. At exact deadline equality it is expired.
The first observed cancellation, shutdown or deadline stop remains sticky.
Checks run before capture, around registered reads and classification, before and
after temporary staging copies, around bounded domain calls and before retaining
the prepared result. No subsequent progress or retry starts a new budget.

These checks are cooperative. An in-flight syscall, parser or domain call can
outlive the budget until its next boundary. This does not establish a hard
wall-time limit or preemption inside the shared engines. The existing wire shape
reports deadline/shutdown settlement as cancelled and does not identify an
internal stop reason.

## Recover within a session or start fresh

A lost preparation reply does not authorize a write. Retry its original key and
query the original operation. Socket disconnection alone does not cancel work.
Before committing, review the exact preview and confirm with its current receipt
and observed target version. A commit that has begun is not cancellable; query
its recorded result or retry its original key to determine whether it committed.
An already published write cannot become cancelled through a late update.

A fresh process has new capabilities and empty operation, preview and idempotency
stores. Old session IDs and receipts do not resume or approve work. Re-execution
captures current registered files, creates a new preview and requires fresh
confirmation. A process crash after publication has no durable operation journal;
observe the project files and prepare a fresh validation/review. This slice adds
neither durable continuation nor automatic commit replay.

## Verification boundary

The checkpoint design and regression plan are engineering prerequisites for
PRD 062 S-4. Source changes, component tests and scoped HTTP checks do not establish
full S-4 acceptance. The [operation browser verification](workspace-operation-browser-verification.md)
records the current integrated consumer, fresh docstring/test coverage and bounded
actual capture/cancellation observations, including an earlier incomplete probe.
Full S-4, platform qualification, security/accessibility and human acceptance
remain separate gates. All other F19 requirements and the final integrated
documentation review remain in scope.
