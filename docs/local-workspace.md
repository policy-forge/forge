# Local web workspace

The workspace is a local, single-user review surface over registered project
files. It does not establish reviewer identity, approval, implementation,
effectiveness, or compliance. This technical implementation does not complete
the PRD-062 human studies, accessibility evaluation, security signoff, or pilot
release gates.

## Launch and stop

```sh
forge workspace --project ./example
forge workspace --project ./example --read-only --no-open
```

Choose and confirm a 15–128 character passphrase in the terminal. Input is not
echoed. Enter that same passphrase in the browser. The listener binds only
`127.0.0.1` on an OS-assigned port. The URL itself grants no access. Passwords
cannot be supplied through arguments, environment variables, project files, or
redirected standard input. Each successful unlock creates a distinct scoped
capability retained in page memory. There are no cookies, local/session
storage, service workers, accounts, or remote services.

On page load, focus starts in **Workspace passphrase**. Unlocking shows
“Unlocking workspace…”; repeated activation while that request is pending sends
no second unlock request. The submitting field or **Unlock workspace** button
keeps focus while waiting. Completed attempts clear the passphrase field. If the
server throttles an attempt, the page shows its stated remaining wait and retry
guidance verbatim. Re-enter the passphrase after that wait and retry. The browser
has no countdown; repeated throttling directs you to stop and relaunch from the
terminal. Moving focus elsewhere while waiting does not authorize a late
throttling response to move it back.

Use **Stop workspace** or Ctrl-C. Stopping discards unconfirmed work and
invalidates session capabilities. Completed files remain saved. Reloading the
page loses its capability and unsaved forms; unlock again. Restarting the
process requires a new passphrase. The workspace never recovers a password or
persists a verifier. Machine sessions cannot unlock the browser.

## Register explicit resources

The workspace reads `forge.workspace.json`, if present, and only its registered
resources. It never scans the directory. The closed `forge.workspace/1` index
contains a project label and stable key, typed role, and portable relative path
for each resource. See [the schema](../schemas/forge.workspace-1.schema.json).
No applicability or mapping decisions belong in the index.

**Policies & Artifacts** can register an existing file, upload a local file, or
convert a registered Markdown policy using the existing conversion engine.
Upload and registration are separate confirmed writes. Conversion does not
register its output automatically. Markdown extraction follows the existing
CLI grammar, including list-item and table clauses; a heading alone is not a
policy requirement. Catalog and Component Definition outputs are supported;
secondary multi-file generation is excluded from this workspace tranche.

Paths use `/` on every platform, even Windows. Paths must stay under the chosen
root and use the index's portable ASCII segment grammar. Symlinks, reparse
points, hard links, aliases, special files, absolute paths, `..`, Windows device
names, and leading-dot segments are rejected. Destination parent directories
must already exist. No arbitrary file read, directory listing, or shell route
exists. A missing or unsafe registered file fails the snapshot closed; repair
that explicit file or its index entry outside the workspace before continuing.

## Review scope and mappings

**Framework Scope** initializes an applicability manifest from an explicitly
selected registered Catalog. The initializer pins the exact resource hash and
inventory. It creates no decisions or reviewers: omitted controls remain under
review. After confirming, register the new applicability manifest. Select a
control, supply its decision and review evidence, apply it to the unsaved
manifest, validate, and preview the changes. Deferred decisions require a
revisit date. Full JSON editing remains available for advanced domain fields,
reviewer records, and advanced fields. A separate guided control links an
explicitly selected registered built mapping collection into the unsaved scope
manifest; validate and confirm before analyzing it.

**Mappings** initializes a manifest from explicitly selected policy and framework
Catalogs. Supply collection/reviewer metadata, a review time, scope, rationale,
and the first reviewed relationship. Load the selected Catalog subjects and
choose the exact source and target. Existing PRD-055 `/1` rules require at least
one relationship; the initializer never invents one or treats an empty scaffold
as a valid reviewed manifest. After registration, the guided editor can update a
reviewed relationship or add another explicit source/target relationship. Full
JSON editing supports multi-subject relationships and optional fields. Validation uses the existing PRD-055 engine and rejects stale
pins or nonexistent subjects before preparing a write.

Reviewer labels are asserted provenance. Neither local unlock nor a mapping
status authenticates a reviewer or approves a policy. Scope decisions and
positive mapping participation remain separate facts. The Review Queue includes
invalid/stale resources, explicit scope-review work, and unmapped subjects from
validated mapping inputs. Filters and cursor pagination show matching items and
total inventory separately, together with the page number. A filtered count is
shown only when its page and unfiltered total belong to the same input version.
Next, Previous and Apply retain the previous rows during loading; a failed read
labels them as previous results and offers Retry. A changed-input cursor offers
Restart from first page. Successful page/filter actions focus their result list;
failed reads focus the local error. The Overview review count also focuses its
result list after the ordinary unsaved-edit decision.

Rebuild the committed mapping collection or analyze committed scope decisions
explicitly. A build/analysis prepares a report; confirming its receipt publishes
one file. Default generated paths are `mapping-collection.json` and
`applicability-report.json`. No pins, decisions, or reports refresh automatically.
One applicability manifest and one mapping manifest are selected in this MVP;
ambiguous manifest or applicability-report destination selection fails closed.

## Preview, conflicts, and recovery

Every material write presents its target, create/overwrite status, validation,
semantic summary, exact proposed hash, current hash/version, bound input hashes,
and text diff before confirmation. Diffs are bounded and explicitly marked when
truncated; the hash still binds the complete bytes. No autosave occurs.
Receipts expire after ten minutes and are single-use, including failed commit
attempts. Re-preview after an expired receipt, stale version, changed input,
changed target, or failed confirmation.

For a current preview, a verified confirmed write refreshes its view and closes
the preview before focus moves to the view heading. A failed refresh keeps the
saved outcome and focuses an error explaining that the write was saved but the
view could not be refreshed. A rejected commit keeps its preview and error open.
Closing a preview after confirmation does not cancel an in-flight write. A late
reply cannot replace a newer view, form or preview. When a post-write refresh is
dismissed, retained tables label their previous results and remain usable;
refresh to load the latest state. Confirmed exports keep their download action
keyboard reachable.

Operation results and idempotency keys remain queryable for the process's
lifetime, within the documented session bounds. **Retry the same request**
reuses its key; an identical commit retry returns the recorded result. A lost
HTTP response does not authorize a new effect. Persistent operation rows remain
available across views. They announce the reported preparation state and supplied
progress facts; missing, null or unreconciled counters remain indeterminate. No
completion percentage or finish time is estimated. Cancel pending operation
acknowledges a request separately from a terminal cancelled state. Cancellation
discards proposed results and never publishes files.

If the operation ID is known, Check operation status reads that same operation;
it does not submit a new preparation. If the first response was lost before an ID
was known, Retry the same request reuses the original key and body. Recovery and
errors stay with their operation across navigation. Background completion offers
Review prepared write; it does not open a preview in another view. Review uses
the normal unsaved-edit decision. A failed preview read preserves the verified
preparation for retry. Preparation success is not Saved: only confirmation of the
exact receipt can publish the write. Stopping the workspace ends status recovery
for that session. Once a commit has begun, it is not cancellable.

Background preparation uses one 30-second monotonic budget from acceptance,
including time waiting for its worker. Equality with the deadline exhausts that
budget. Cooperative checks run around registered reads, classification,
temporary staging copies, domain calls and preview retention. A blocking read,
parser or domain call finishes before its next check; the deadline is not a hard
process timeout.

While capture is measured, the browser reports "Captured registrations: X of Y
reported." The `progress` pair counts complete captured and classified
registrations against the full registered set (up to 1,000), including inputs
classified as invalid. These units differ from the 100 consumed-input limit for
a prepared effect. Progress clears before unmeasured work and at termination.
Capture completion does not mean preparation or publication succeeded. A cancel
acknowledgment may still show pending/running; poll the original operation for its
terminal state. See [workspace operations](workspace-operations.md) for recovery
and the distinction between session retry and fresh-process re-execution.

Publication is a single-file atomic rename, with exclusive staging, flush, input
revalidation, and destination/parent identity revalidation immediately before
rename. New targets use no-replace publication. Unix writes use directory-held
file descriptors; Windows holds parent handles without delete sharing and
renames the held source handle. Failure before publication preserves existing
bytes. An interrupted process may leave an inert temporary file; never treat it
as a committed output. Successful rename is not reported as a failed commit if
a subsequent directory flush fails. Power-loss durability is distinct from
process-interruption atomicity.

Do not run another writer against the same destination during confirmation.
Ordinary filesystems do not provide a portable compare-and-swap rename against
an arbitrary uncooperating writer; a final recheck-to-rename race remains for
external writers. This implementation does not claim isolation against a
malicious same-user process or interruption-safe multi-file transactions.
Windows runtime behavior must pass hosted Windows tests before platform support
is claimed; cross-compilation alone is insufficient evidence.

## Trace and export

Provenance views connect supplied scope decisions, reviewed mapping edges,
explicit subjects, registered artifacts, and bounded source excerpts. OSCAL
trace properties use the existing trace walker. A source-file reference resolves
only to the exact registered portable path, never a matching basename. Excerpts
bind the current captured source hash and exact line range. Historical OSCAL
trace properties do not supply an original source hash: their labels explicitly
state that limitation rather than claiming the source is unchanged.

Static exports use the closed `forge.workspace-report/1` envelope in a canonical
inert HTML document. They contain bounded numeric review/trace counts and sorted
opaque resource IDs with SHA-256 pins. They contain no policy excerpts, framework
prose, reviewer names, rationale, native absolute paths, scripts, remote assets,
forms, or arbitrary report strings. The strict default redaction profile is the
only profile. Download is available only after confirming that export's write,
and the server rechecks the exact published hash before returning it.

A `trace-report` registration accepts this canonical trace export, not arbitrary
HTML or a CLI text table. Its complete closed envelope, count invariants, exact
canonical bytes, input pins, and recomputed counts are validated. A structurally
valid historical report with changed inputs is stale. Unsupported versions and
extra content fail closed. Applicability JSON reports retain their PRD-056
contract and are compared with a freshly validated analysis. Their six
classification counts must sum exactly to the declared inventory total;
overflowing sums are invalid. Unfiltered reports restate each category exactly,
while filtered details may show a subset of the complete inventory. A valid
historical report still requires recomputation to establish currentness.
Historical report fingerprints identify the manifest and, when that exact manifest is still
present, its referenced framework and mapping files. If the manifest changed,
only its historical hash is shown; old file paths cannot be reconstructed or
inferred from current registrations.

## Supported local clients and bounds

```sh
forge workspace --project ./example --machine-session --read-only
python3 scripts/workspace_client.py --forge ./target/debug/forge --project ./example
```

Machine launch emits one JSON descriptor on stdout, including a secret
capability. Do not log or persist it. The maintained Python standard-library
client captures it in memory, checks the loopback endpoint and API version,
and uses the same documented API as the UI. Its `request`, `wait`, and explicit
`commit(..., confirmed=True, idempotency_key=...)` methods cover the shared
workflow; clients must retain their own idempotency key across response loss.
Python cannot guarantee physical zeroization of immutable strings.

The normative contract is [OpenAPI](api/forge-workspace-v1.openapi.yaml), with
[compatibility policy](api/compatibility.md) and [capability matrix](api/capability-matrix.md).
The additive unreleased contract is 1.2.0. It includes closed initialization,
optional provenance references and read-only index-bundle queries. Existing domain
`/1` meanings remain unchanged.
The public Rust `Commands::Workspace` variant extends an exhaustive enum and
requires a release compatibility decision for downstream matches. No release
compatibility or new release is asserted here.

Conservative bounds include: 1 MiB index/ordinary request, 1,000 registered
resources, 10 MiB per resource, 50 MiB captured inputs, 100 consumed inputs per
prepared effect/report, 10,000 visible controls/graph entries, 64 JSON nesting
levels, 100,000 structural JSON separators, 20 MiB retained prepared data, 256
session receipts/operations/idempotency results, 16 capabilities, 16 connections,
2 request workers, 1 background preparation, and 30 requests/second per
capability. Upload has a separate bounded base64 request envelope. API responses
are at most 4 MiB. Reaching a session retention bound requires finishing work
and restarting; results are not silently evicted.

### Workspace index bundle inspection

In **Trace & Reports**, use **Preview metadata** to inspect every registered
resource key, role, project-relative path, SHA-256 fingerprint and byte length.
This read is available in writable and read-only sessions. Labels, paths, keys
and stable hashes can reveal project information even though source content is
excluded. Acknowledge that sensitivity before **Download metadata bundle**, which
requests a local `forge-workspace-index-and-hashes.json` file. It creates no
project file, server publication or receipt. A fresh preview or view/session
retirement clears acknowledgment for the old preview.

To compare an existing file, use **Choose a metadata bundle JSON file**, then
**Compare registered fingerprints**. The file may contain at most **1,048,565
bytes**: its original bytes and fixed 11-byte JSON wrapper must fit the 1 MiB
request bound. The browser sends those bytes without decoding, stripping a BOM,
removing duplicate keys or rewriting JSON. The server validates the input and
compares only current registered captures; supplied unregistered paths are not
opened. An explicit retry captures current state again.

Queries preserve complete denominators through the existing 1,000-registration
and byte bounds, with no 100-input preparation prefix. Missing versus explicitly
empty current index, expected fingerprint agreement versus current-only
registrations, whole-index equality and observed valid/stale/invalid content
remain distinct. Matching hashes or the local sensitivity acknowledgment
establish neither domain approval nor import readiness or later byte stability.

The maintained Python client still exposes `bundle_preview()` and
`verify_bundle(bundle)` for the same read-only queries. See
[Workspace index bundles](workspace-index-bundles.md) for its example, normalized
index hashing, limits and errors. Receipt-backed server export/publication,
source-content opt-in, confirmed writable import, reviewed batch binding,
retention/capacity qualification and full S-6 acceptance remain open.

## Verification and remaining gates

Rust contract, domain, security, transaction, and headless workflow tests run
through `cargo test --locked`. Browser tests in `ui/tests/workspace.cjs` exercise
the embedded assets through the published API; the POSIX harness is
`scripts/test_workspace_browser.py`. Playwright is development-only. The browser
harness runs locally only: it is not executed by CI, and installing the browser
test dependencies in CI remains pending. These tests check request coverage,
blocked non-loopback page traffic, storage absence, workflow behavior, and
narrow-viewport reflow. This is not proof that all browser background traffic or
all OS processes are network-denied.

Current API/client verification is configured separately on Linux, macOS and
Windows, including ordinary PRs with stacked bases. Versioned `/2` receipts
retain the current 16 client assertion groups and complete 39-operation
inventory, exact inputs, requested head/base and tested Git parents. A passed
receipt covers the declared three-suite slice only. See the
[current verification guide](plans/2026-10-02-f04-current-api-verification.md)
and [development record](workspace-current-api-verification.md) for status,
coverage gaps, publication failure controls and remaining gates.

Focused Inspect/Trace navigation uses the same unsaved-edit decision as ordinary
navigation. Keep editing or Escape preserves values and returns focus to the
connected enabled invoker; a failed provenance read preserves the form for retry.
Successful destinations focus their heading, while failed navigation retains the
error summary's focus. The historical [navigation verification record](workspace-navigation-verification.md)
and historical [live-state verification record](workspace-live-state-verification.md)
report exact source/docstring scopes and local macOS Chrome development runs.
The live-state slice also exercises pagination, persistent operation recovery, rendered
input borders and final-page reflow at 640 and 320 CSS pixels. Those focused checks
do not establish full accessibility acceptance. The separate
[keyboard verification record](workspace-keyboard-verification.md) records unlock
feedback, confirmation focus and retained-table recovery checks.

The current [operation browser verification](workspace-operation-browser-verification.md)
records fresh integrated source, docstring and test coverage checks, both native
modes and a bounded actual capture/cancellation probe. It preserves the earlier
incomplete probe and all zero/unmapped coverage dimensions. These observations
remain scoped development evidence.

The [browser metadata verification record](workspace-bundle-browser-verification.md)
records the fresh source/docstring coverage audit, actual local downloads and raw
comparisons in both modes, and long-content reflow observations. The earlier
overflow failure and remaining coverage gaps are preserved there.

Manual screen-reader/keyboard evaluation, WCAG 2.2 AA acceptance, the complete
supported browser/platform matrix, security approval, five target-user studies,
pilot metrics, packaging/release approval, and PRD-062 overall completion remain
pending. See [accessibility requirements](accessibility/062-accessibility-requirements.md)
and [the security record](SEC/062-sec-local-web-workspace.md).

After the remaining roadmap work, a full documentation review must reconcile the
integrated CLI/API, examples, architecture, schemas, dependency/provenance claims,
platform support, verification instructions, requirements and acceptance status.
