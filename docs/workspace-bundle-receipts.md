# Confirmed metadata bundles and index replacement

This guide describes metadata receipts introduced in workspace API **2.2.0 / 51
operations**, also admitted in **2.3.0 / 57** and **2.4.0 / 65**. The retained open
draft stack at `304b31f8ea3913eba4a7d776b833b9b3db4ad3a4` advertises 2.4.0;
it is no longer merely an uncommitted working-tree proposal. This source checkpoint
adds no compiled or hosted result to the earlier verification records and grants
no merge, release or acceptance. Select API v2 explicitly at launch.
API v1 remains **1.2.0 / 39 operations**. The three metadata operations and their
existing semantics are distinct from the six opt-in source operations in 2.3.0. This contract version does not announce a product
release or establish execution, platform, accessibility or human acceptance.
The initial API v2 foundation and the captured inspection implementation retain
their historical versions and verification records.

A metadata bundle contains the index label, registration keys, roles and
project-relative paths, plus hashes and sizes. **It contains no registered source
file contents, and its metadata can still be sensitive.** Acknowledging this
sensitivity is required before preparing an export. The exported category
exclusion is `source-excerpts`; it is not a promise to sanitize labels, keys or
other metadata.

See [API v2 migration](api/migration-v2.md) for launch and registration, and
[the metadata query guide](workspace-index-bundles.md) for local preview,
normalized hashing and registered-only comparison.

## Choose the workflow

| Workflow | Result | Writes to the project |
|---|---|---|
| Existing bundle preview and local JSON download | A metadata snapshot downloaded by the browser | None |
| Existing supplied-bundle verification | Comparison against current registered files | None; supplied unregistered paths are not opened |
| Metadata export preparation | A retained preview for one explicit project-relative JSON target | Only after confirming that exact preview |
| Index replacement preparation | A retained preview and complete previous/proposed index objects | Only the index, after confirming that exact preview |

The existing preview and verification queries continue to work in read-only
sessions. They can inspect a complete index of up to 1,000 registrations. That
query capacity does not increase the write preparation's whole-file limit.
Read-only sessions cannot prepare exports or index replacements. The committed
JSON download is an authenticated read of a retained receipt in its original
session and selected API major.

## Prepare and confirm a metadata export

In **Trace & Reports**, use **Export metadata or replace the index**. Enter the
**Metadata export project-relative target**, acknowledge that labels, keys,
paths and hashes are sensitive, then activate **Prepare metadata export**.
The index and any registered resource are forbidden output targets. The usual
portable-path and confinement rules apply to a new or existing destination.

Export preparation returns an ordinary export operation. Preparation is not a
saved file. Review its status and then **Review proposed write**. Inspect the
exact target, proposed hash, inputs and validation. **Keep editing** dismisses
the review without committing; **Confirm this exact write** performs the
conditional write. If an operation reply is lost, use its known operation ID to
check status rather than submit another preparation.

After the confirmed write, **Download committed metadata bundle** retrieves the
exact retained metadata bytes as `application/json`, with the fixed attachment
name `forge-workspace-index-and-hashes.json`. The client checks the receipt's
hash. The configured project target can have a different name; the download name
does not determine the artifact's type.

This JSON route accepts only committed metadata receipts. The existing HTML
report download accepts only report receipts. An HTML report preview, local
metadata download or unconfirmed metadata preview cannot be used as a committed
metadata receipt. A changed destination or unavailable receipt refuses the
request instead of producing a new serialization.

## Replace the complete index explicitly

Choose a metadata bundle under **Choose a bundle for index replacement**. Select
**Replacement target index schema** and acknowledge replacement of the index
label and all registrations. Activate **Prepare index replacement**. The direct
200 response means preparation succeeded; it does not mean the index was saved.

The response includes validation, the ordinary effect preview, and a complete
replacement description:

- `previous_index` is the complete current index, or null when no index exists.
- `proposed_index` is the complete proposed index, including its label and ordered registrations.
- `supplied_index_sha256` and `proposed_index_sha256` use normalized index bytes; the latter matches the prepared index output hash.
- `removed_resource_keys` lists old keys absent from the proposal, in old-index order. A reused key whose role or path changes remains visible in the complete index objects.
- `consumed_file_count` reports the complete distinct physical-file union used by preparation.

The index hash covers the typed index's ordered fields serialized as pretty JSON
with a trailing line feed; registration-array order is retained. It is distinct
from a hash of the original bundle file or wrapper.

Inspect all of these before **Confirm this exact write**. Replacement is not a
merge. Removed registrations disappear from the index; their files are not
removed. A bundle does not restore missing source files or copy supplied file
contents into the project. Every incoming path must already resolve through the
confined project root, match its pin and pass its existing role admission.
Fingerprint agreement alone does not establish schema validity, freshness or
reviewer authority. An applicability report may use the existing admitted-report
profile only when it exactly equals the captured current analysis; that exception
does not extend to other roles.

Bundle1 remains paired with index1 and its seven roles. Bundle2 remains paired
with index2 and its fifteen roles. The request's `target_index_schema_version`
is the numeric value **1** or **2**. Choosing 2 can explicitly migrate a supplied
index1 to index2 while preserving its label and ordered registrations. Index2
cannot be downgraded to index1. If the current project is index2, a supplied
bundle1 also requires selecting 2. Launching or merely selecting a file performs
no migration.

The import body retains the selected file's original JSON bytes inside the
wrapper. Duplicate or escaped duplicate keys are not collapsed by a
parse/stringify round trip. Malformed JSON or invalid version/profile pairs are
rejected. The import wrapper adds exactly 80 bytes, so the selected file limit is
**1,048,496 bytes** within the 1 MiB request limit. Existing verification keeps
its separate 11-byte wrapper and **1,048,565-byte** file limit.

## Bounds, conflicts and recovery

Preparation uses one complete union of at most **100 distinct physical files**
and **50 MiB** captured bytes. This includes a present raw index, all current
registrations (including registrations removed by replacement), incoming files,
and any distinct existing export destination. Exactly shared validated paths
count once; portable aliases are refused. The complete ordered effect input-pin
list also remains capped at 100; physical-path deduplication does not discard
distinct registration pins. Each resource is bounded to 10 MiB and the index to
1 MiB. The dedicated bundle-effect capture checks the complete
index/resource union before current resource reads; an over-cap index is refused
as a whole, never truncated to a 100-file prefix. The final destination and
whole-union checks still apply before receipt admission.

The existing **20 MiB** retained capacity applies to output, base and private
captured buffers. It also conservatively charges four times the complete final
preview and four times the complete wrapped reply. A request within the capture
limits can still exceed retention capacity. These conservative charges are
admission rules, not a literal process-heap bound. There are at most **256**
pending or ready records, and a prepared receipt's original lifetime is **600
seconds**; ready retries do not extend it or evict another record.

Retain the original admitted request body and idempotency key when a preparation
reply is uncertain. Retrying that same request while it is pending returns the
retryable 409 code `bundle-preparation-in-progress`. Wait and retry the original
request. Using the same key for a different request returns the existing
idempotency conflict. A ready retry returns the original receipt. Reservation
ownership prevents a late failed preparation from removing another attempt's
record; it is internal state, not an additional client credential.

Direct index preparation has one immutable, cooperative **10-second** budget
starting before body reading. It includes queueing, capture, validation, response
construction and final retention admission. `bundle-preparation-budget-exceeded`
or `bundle-preparation-interrupted` returns 503 without a partial prepared reply.
Checkpoints do not preempt a parser or syscall or guarantee response delivery by
the deadline. Export operations retain their separate 30-second accepted-operation budget
and cancellation behavior. Dismissing a browser review is not a server-side
cancellation of a dispatched preparation.

Confirmation rechecks the current snapshot, exact raw index presence and file
identity, every bound input's identity/hash/length, and the destination base and
parent. Changes refuse the exact write. Re-preview the intended replacement
rather than treating a new idempotency key as recovery. Matching confirmation
attempts that reach receipt use consume that receipt even if a later conflict or
expiry refuses the write. Earlier authorization, malformed/unmatched request or
capacity refusals do not consume it. Restarting the workspace does not recover
session-local previews, operation IDs or committed download records.

## Use the maintained client

Launch `Workspace` with `api_major=2` and `read_only=False` for preparation. Its
matching numeric API-major-2 bootstrap negotiation remains in place. In the
retained 2.4.0 draft source, metadata receipt methods admit 2.2.0, 2.3.0 or 2.4.0;
the nine captured inspection methods admit 2.1.0, 2.2.0, 2.3.0 or 2.4.0. Inline
source methods admit 2.3.0 or 2.4.0; staged methods require exactly 2.4.0.

| Method | Result |
|---|---|
| `prepare_bundle_export(target_path, acknowledge_sensitive_metadata=True, idempotency_key=...)` | Export operation preparation |
| `prepare_bundle_import(bundle_bytes, target_index_schema_version=1_or_2, acknowledge_index_replacement=True, idempotency_key=...)` | Validation, preview and complete replacement description |
| `download_bundle_export(operation_id, expected_sha256=...)` | Exact committed metadata JSON bytes |

Preparation and confirmation remain separate actions. Keep credentials and
sensitive metadata out of logs. The browser retains the original body/key for
uncertain dispatched requests; a retired file read or cancelled dirty-form
decision sends no new request.

## Remaining S-6 work

This metadata prerequisite does not complete full S-6 bundle import/export.
The separate [source workflow](workspace-source-bundles.md) describes the finite
opt-in source profile, complete batch confirmation and durable known-ID recovery.
The bounded [staged transfer implementation](workspace-staged-source-bundles.md)
is a separate draft-source workflow. Complete capacity and cross-platform
crash/rollback qualification remain required work; source implementation is not
full S-6 acceptance.
Native browser/platform interoperability, human accessibility and workflow
acceptance, security/release gates and the final integrated documentation review
remain open. Historical receipts for query-only bundles, API 2.0 or API 2.1
retain their original scope; they do not establish the new 51-operation behavior.

See the [development verification](workspace-bundle-receipts-verification.md)
for exact source, executed controls, documentation/coverage denominators and
retained failures. This focused verification does not close full S-6 acceptance.
