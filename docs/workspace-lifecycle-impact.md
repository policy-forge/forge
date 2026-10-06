# Inspect lifecycle and framework impact

> Integration update — October 6, 2026: the applicable workspace, MCP, assessment
> and review implementations are now on main. See the [merge reconciliation](plans/2026-10-06-merge-documentation-reconciliation.md)
> for exact commits and qualification scopes. Earlier draft states and failed
> receipts below retain their dated meanings; human and release gates remain separate.

In the open draft stack, launch `forge workspace --project ./example --api-major 2` and open **Lifecycle & Impact**. The exact stack-219 source at `304b31f8ea3913eba4a7d776b833b9b3db4ad3a4` advertises API2 contract **2.4.0** and admits this read-only view in both writable and read-only sessions. Earlier captured 2.1.0, 2.2.0 and 2.3.0 revisions also provided inspection. Default API1 sessions retain their existing navigation. API2/2.0.0 sessions supply registration and bundles; restart with the draft server for inspection. This is draft-source availability, not shipped-main or release acceptance.

Create or explicitly migrate to index2, then register its inputs through **Policies & Artifacts**. Launching or inspecting never migrates an index. An absent index, an index1 awaiting explicit migration, an empty family and registered inputs needing attention have separate states.

## Lifecycle records and owner queue

The recorded inventory lists exact policy/version and owner keys. Without an **Explicit lifecycle review date**, it publishes no derived current status. Status details and the owner queue require a real `YYYY-MM-DD` date; they never choose today's date for you. Computed status validates the complete registered portfolio before filtering or paging. A missing dependency, invalid record or replacement cycle prevents a partial status result.

Source and generated-artifact fingerprints come from captured registered bytes. Status uses the existing lifecycle model and UUID identity rules. Captured schema validity is a separate fact: an identifiable but schema-invalid generated artifact can have matching fingerprints and still need attention. A recorded approval or actor role is locally declared metadata; FORGE does not authenticate the actor or grant approval.

Recorded history shows event identity, sequence, timestamps, state changes and declared actors. It asserts neither current freshness nor authenticated approval. Lifecycle finding references remain unresolved when no comparison pair is supplied. Titles, rationale, party names and source content are excluded. Owner queues count distinct records and owner memberships separately; one policy can have several owners.

## Framework comparisons

The comparison inventory reports declared availability without claiming a current analysis. Selecting a comparison validates its exact registered input closure and reruns the shared framework-impact engine over captured copies. Detail reports a captured-current comparison, complete change/finding counts and provenance.

Change pages accept an exact change class. Finding pages combine **group**, **decision state**, **policy source**, **priority** and **owner** filters with AND semantics. Complete summaries retain hidden findings and their dispositions. A zero-row filtered page does not establish that the comparison has no findings or blockers.

Prior-only dispositions are historical local assertions from the exact hash- and pair-bound prior report. Dispositions matched to current findings remain attached to those findings. Prior-report admission is labelled limited-structural and does not establish complete schema validity, reviewer identity or approval of risk. No inspection action writes a disposition, transitions a lifecycle record or applies a migration.

## Paging, capture and limits

Pages default to 50 and allow 1–200 rows. Each complete response identifies its immutable snapshot and view version; comparison subpages also identify the unfiltered comparison. Cursors bind the endpoint, resource, date, filters, page size and capture. If any bound input changes, restart from the first page after a 409 version-conflict. Complete totals come from the same response as its page.

The existing capture limits are 1,000 registrations, 10 MiB per resource and 50 MiB in total. Lifecycle owner placements are bounded at 64,000; complete impact collections at 100,000. Responses are at most 4 MiB. Excess is rejected rather than silently truncated. The existing raw request-target limit is 2,048 bytes; percent encoding counts toward that limit. Decoded owner/group/policy-source tokens additionally require trimmed strings without control characters and at most 4,096 UTF-8 bytes.

A ten-second immutable read budget includes blocking-worker wait, capture, domain work and response validation/encoding. Checkpoints are cooperative; they do not preempt an internal parser or syscall. Budget expiry returns 503 `query-budget-exceeded` with no partial result. Typed cancellation returns 503 `query-interrupted`; shutdown retains 400 `shutdown-in-progress`. Inspection publishes no operation progress, effect receipt or project file.

Keys, declared actor identities, resource identities and hashes can still reveal project information. Treat captured metadata as sensitive even though source excerpts and absolute filesystem paths from runtime errors are excluded. Declared relative artifact identities can remain visible.

The [API2 contract](api/forge-workspace-v2.openapi.yaml), [capability matrix](api/capability-matrix-v2.md) and [migration guide](api/migration-v2.md) define the current surface. Source tests, function-entry coverage, native browser/accessibility checks, independent review, human acceptance and release qualification provide separate evidence. The final documentation review across the full roadmap remains open until all implementation work has been reconciled.

The [development verification record](workspace-lifecycle-impact-verification.md) gives exact executed checks, selected documentation and coverage denominators, retained gaps and native observations.
