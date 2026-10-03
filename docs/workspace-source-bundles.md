# Export and restore exact workspace sources

This guide describes the bounded inline source workflow introduced by workspace
API **2.3.0 / 57 operations**, also admitted by the separately captured
**2.4.0 / 65** working-tree successor under integration. This text supplies no
compiled, merged or hosted result for that successor. Select `--api-major 2` at launch. API v1 remains
**1.2.0 / 39 operations**. Contract versions describe the local API, not a product
release. The metadata-only prerequisite retains its historical **2.2.0 / 51**
verification records.

Source content is excluded by default. The source workflow requires an explicit
opt-in and includes exact registered file bytes together with the index label,
keys, roles, relative paths, sizes and hashes. **All of these can be sensitive.**
It supplies no blanket redaction, reviewer identity, approval or compliance
judgment. Use the [metadata workflow](workspace-bundle-receipts.md) when file
contents are unnecessary, and [API v2 migration](api/migration-v2.md) for launch
and registration.

## Choose what will be saved

| Workflow | Preparation | Subsequent confirmation |
| --- | --- | --- |
| Source export | An ordinary export operation prepares one JSON target | The existing single-file receipt writes that target |
| Source restore | A direct 200 response describes every target, input binding, directory and complete index replacement | A separate acknowledged batch confirmation durably accepts the restore operation |
| Outcome lookup | An authenticated read of a known restore operation ID | No write or renewed receipt authority |

Read-only sessions can look up known outcomes; they cannot prepare, confirm or
cancel these writes. Export completion means a preview is ready, not that the
artifact was saved. Restore preparation writes no project file and creates no
accepted durable transaction intent.

## Export with source content explicitly included

In **Trace & Reports**, open **Export or restore exact sources**. Enter a
**Source export project-relative target** and select **Include exact source
bytes and sensitive metadata in this export**, then activate **Prepare source
export**. The output cannot be the project index or a registered input.

After the operation prepares its preview, review the exact target, output hash,
bound inputs, validation and proposed write. **Keep editing** performs no commit.
**Confirm this exact write** publishes only the exported JSON artifact. It does
not restore another project.

**Download committed source bundle** retrieves the exact authenticated,
hash-checked source JSON from the retained original-session export receipt. Its
fixed local download name is `forge-workspace-index-and-source-content.json`;
the chosen project target can have another name. Source JSON, metadata JSON and
HTML report downloads use separate private receipt families. An unconfirmed
preview or another artifact family cannot be used as a committed source export.

## Review a complete source restore

Choose a file under **Choose an exact source bundle JSON file**, select **Source
restore target index schema**, and acknowledge all three statements about index
replacement, sensitive source content and creating or overwriting project files.
Then activate **Prepare source restore**.

The browser sends the original selected JSON bytes. It does not strip a BOM or
collapse duplicate keys with a parse/stringify round trip. The server rejects
malformed or duplicate-key input, unsupported profiles, mismatched pins/content,
noncanonical encoding, unsafe paths and an inadmissible proposed dependency
closure. File fingerprints alone do not establish current freshness or authority.

The **Review complete exact source restore** dialog contains the complete plan:

- Every resource target and the index target, with create/overwrite status,
  current and proposed hashes, byte lengths and versions.
- All current and proposed input bindings, including explicit absence.
- All parent-before-child directory intentions and the complete previous and
  proposed index objects, ordered removed keys and planned-file count.
- Bounded diffs with explicit binary or truncation explanations.
- A preallocated public operation ID for later outcome lookup.

Read every target and the complete index replacement before selecting **I
reviewed every target, input binding, directory and index replacement; restore
these exact bytes** and **Confirm this exact source restore**. Replacing index
membership is not a merge. Removed registrations' files are not deleted. Exact
resource bytes are restored; the proposed index is normalized typed JSON.

The numeric index selector is 1 or 2. Preserving index1 or explicitly migrating
index1 to index2 is supported. Current or supplied index2 cannot be downgraded.
Selecting a file or dismissing the review does not perform a migration or restore.
Changing a selection retires its acknowledgment; a newer view or preview prevents
an unsent old preparation from being dispatched. Dismissing a review does not
cancel work already dispatched to the server.

## Keep the operation ID when a reply is uncertain

The preview's operation ID is a lookup key, not authorization. Keep it before
confirmation. A 202 confirmation response records durable accepted intent, not
successful publication. The browser sends confirmation once; reopening the same
review after an uncertain attempt cannot resend the receipt.

Use **Check source restore status**, or enter the ID under **Source restore
outcome operation ID** and select **Look up source restore outcome**. After a lost
reply or restart, use a fresh same-project API2.3 or 2.4 session and that known ID.
Credentials, preview tokens and old session authority are never revived. A 404
may mean no accepted intent or an unavailable/expired outcome; **it does not prove
that no files changed and never justifies blind reexecution**.

Interpret the independently reported facts:

| Observation | Meaning |
| --- | --- |
| `pending` or `running` | Publication outcome remains unmeasured |
| `succeeded`, `write_outcome: committed` | A durable committed decision and exact result are recorded; inspect cleanup separately |
| `failed` or `cancelled`, `write_outcome: none`, cleanup verified | Conditional rollback was verified |
| `recovery-required` | Preserve the known ID; outcome may be none, committed or unknown and access may remain blocked |

**Staged files: X of Y reported.** counts verified staged files only. It is not a
publication percentage. Cancellation acknowledgment may remain pending/running;
only the terminal outcome establishes verified rollback. A request after the
durable commit decision cannot cancel the committed write.

## Publication, recovery and limits

The transaction stages exact new bytes, retains captured old generations and
publishes resources in authorial order and the index last, then verifies the complete proposed result
before the durable committed decision. Before that decision, an interruption
attempts bounded conditional rollback of owned instances. Foreign replacements
are not overwritten as recovery. After a committed decision, cleanup problems
cannot be relabelled as no write.

Participating workspace captures and commits share a project-data fence. External
CLI commands and editors do not participate in that fence and can observe mixed
whole-file generations. This is not globally atomic publication for external
readers. Stop another writer before confirming. Unknown ownership, durability or
cleanup blocks participating project-data access rather than bypassing recovery.

The consumed Unix port checks qualified private per-user transaction state before
credentials and listening; clients cannot choose its path. The Windows source
restore port returns typed unavailable while its native implementation and
qualification remain open. A writable API2 launch may settle
accepted intent. A read-only launch does not perform recovery writes and can
refuse startup when unresolved state needs settlement. Missing qualified native
primitives or unresolved state fails closed. Do not remove private state to turn
an unknown outcome into permission to resend.

The finite inline profile is `forge.workspace-index-bundle/3` with
`index-and-source-hex`. Lowercase hex chunks preserve exact resource bytes,
including non-text bytes and line endings. Index, pins and content keys form a
complete ordered bijection. The raw source artifact is at most **1,048,429 bytes**;
the acknowledged import wrapper adds **147 bytes**, within the **1 MiB** request
cap. This limit includes hex and JSON overhead, not just decoded file sizes.

The complete planning union has at most **100 paths**, including the index target
even when absent, all current/incoming/base paths, and a source export output
even when absent. A distinct export target and present index leave at most **98
registered resources**. Inline import permits at most **99 incoming resources**,
but its complete current/incoming union can still exceed 100. Portable aliases
are refused; an over-cap request is rejected whole, never accepted as a prefix.

One complete old/new capture budget is **50 MiB**, with **10 MiB** per resource.
Shared retained capacity stays **20 MiB / 256 records**, including private bytes
and conservative complete preview/reply charges. These are admission limits, not
a literal process-heap bound. Preparation receipts keep their original
**600-second** lifetime. Finished durable outcomes have bounded 600-second
retention; unresolved state is not silently evicted. New directory intentions are
bounded at 100, and all text diffs share a 200,000-byte budget.

Direct preparation has one cooperative ten-second body/queue/work budget.
Accepted restore keeps its original thirty-second forward budget. After forward
work stops, a queued worker may wait up to thirty seconds for the exclusive
workspace lease. After forward work returns without a finished committed outcome,
native settlement receives its own separate thirty-second cooperative budget;
it requires the exclusive lease. Neither renews forward work, and these separate
budgets do not establish one combined wall-clock bound.
Checkpoints do not preempt a parser or syscall. Keep the same original body and idempotency key for
an uncertain preparation reply; pending preparation returns
`bundle-preparation-in-progress`, while a ready retry returns the original receipt
without extending its lifetime. Confirmation uncertainty requires known-ID reads,
not automatic reconfirmation.

## API and maintained client

All six inline routes require selected API major2 and negotiated 2.3.0 or 2.4.0, with existing scoped authorization,
Host/Origin/Fetch Metadata and read-only checks. GETs use no idempotency key;
preparation and confirmation require one. Cancellation accepts the existing
client's empty-object POST convention and no idempotency key.

| Method and route | Successful response |
| --- | --- |
| POST `/api/v2/project/source-bundle-exports` | 202 ordinary export operation |
| GET `/api/v2/project/source-bundle-exports/{operation_id}/download` | 200 exact committed JSON |
| POST `/api/v2/project/source-bundle-imports` | 200 complete preview and index replacement |
| POST `/api/v2/project/bundle-restores/{preview_id}/commit` | 202 durably accepted restore operation |
| GET `/api/v2/project/bundle-restores/{operation_id}` | 200 sanitized current outcome |
| POST `/api/v2/project/bundle-restores/{operation_id}/cancel` | 200 cancellation acknowledgment/outcome |

The maintained `Workspace` client adds `prepare_source_bundle_export`,
`prepare_source_bundle_import`, `prepare_source_bundle_import_file`,
`download_source_bundle_export`, `commit_source_restore`, `source_restore_status`,
`cancel_source_restore` and `wait_source_restore`. The last method polls GET only;
`SourceRestoreUncertain` retains the known ID without automatically resending.
Keep credentials, source bytes and preview tokens out of logs.

Matching numeric major-2 bootstrap negotiation remains separate from feature
gates: inspection admits 2.1.0/2.2.0/2.3.0/2.4.0; metadata receipts admit
2.2.0/2.3.0/2.4.0; inline source routes admit 2.3.0/2.4.0. The separately
captured staged methods require exactly 2.4.0; see the [staged proposal](workspace-staged-source-bundles.md).
The original API1 surface remains available.

This bounded implementation does not close larger staged transfers, complete
capacity and cross-platform crash/rollback qualification, security/audit,
accessibility, participant, release or full integrated-documentation gates. Source
tests, mocked client/DOM controls and source review provide their own scoped
evidence; this guide supplies none of those execution or acceptance results.

See [source development verification](workspace-source-bundle-verification.md)
for the source-bound docstring, coverage, native and Chrome checkpoint.
