# API Capability Matrix

> Integration update — October 6, 2026: the applicable workspace, MCP, assessment
> and review implementations are now on main. See the [merge reconciliation](../plans/2026-10-06-merge-documentation-reconciliation.md)
> for exact commits and qualification scopes. Earlier draft states and failed
> receipts below retain their dated meanings; human and release gates remain separate.

> **Document Type:** Capability Matrix
> **Audience:** Product, engineering, LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-10-02
> **Owner:** Brian Luby

---

## Purpose

This matrix maps every in-release P0/P1 product action for the PRD 062 local
web workspace to one or more documented `/api/v2` operations, its golden-path
step, its user stories, and the authorization scope required. It is the
human-readable rendering of the machine-readable source of truth at
[capability-matrix-v2.json](capability-matrix-v2.json); the JSON file is what CI
checks. The governed product contract is
[../PRD/062-prd-local-web-workspace.md](../PRD/062-prd-local-web-workspace.md)
and the normative API definition is
[forge-workspace-v2.openapi.yaml](forge-workspace-v2.openapi.yaml).

## The Two-Directional Coverage Rule

1. **A browser feature without a matrix entry cannot enter implementation.**
   Every visible control, keyboard command, automatic refresh, retry, upload,
   download, cancellation, and shutdown action in the bundled UI must trace to
   an operation listed here before any UI code is written.
2. **No orphan API operations.** Every `operationId` in the OpenAPI document
   must appear in at least one matrix entry, and every operation listed in a
   matrix entry must exist verbatim (case-sensitive) in the OpenAPI document.
   CI enforces both directions.

## Authorization Values

| Value | Meaning |
|-------|---------|
| `browser-write` | Requires any write-scoped session capability (interactive writable browser session or write-capable machine session). In a read-only session these operations return the typed `read-only-session` error. |
| `browser-read` | Available to any authenticated session capability, including read-only browser sessions and read-only machine sessions. |
| `machine` | Bootstrap material delivered only through the `--machine-session` stdout descriptor, outside `/api/v2`. |
| `unlock` | The single unauthenticated `/api/v2` operation (`unlockSession`). |
| `none` | Unauthenticated static bootstrap assets served outside `/api/v2` with no project data. |

## Matrix

| ID | Action | Step | User stories | Operations | Authorization |
|----|--------|------|--------------|------------|---------------|
| CM-01 | Unlock the browser session with the per-launch passphrase | 1 | US-12 | `unlockSession` | unlock |
| CM-02 | Read session metadata and discover the supported API version | 1 | US-1, US-13 | `getSession` | browser-read |
| CM-03 | Stop the workspace from the UI | cross-cutting | US-10, US-12 | `shutdownSession` | browser-read |
| CM-04 | Serve the static locked bootstrap shell | 1 | US-12 | `unlockSession` | none |
| CM-05 | Bootstrap a machine session from the stdout descriptor | 1 | US-13 | `getSession` | machine |
| CM-06 | Understand project state and configuration | 3 | US-1 | `getProjectSummary`, `getProjectConfigStatus` | browser-read |
| CM-07 | Browse and inspect registered resources | 2 | US-1 | `listResources`, `getResource` | browser-read |
| CM-08 | Register an existing root-contained file | 2 | US-5 | `registerResource` | browser-write |
| CM-09 | Upload bounded bytes to a confirmed root-contained target | 2 | US-5 | `uploadResource` | browser-write |
| CM-10 | Prepare and inspect Markdown-to-OSCAL conversion | 2 | US-5 | `preparePolicyConversion`, `getConversion` | browser-write |
| CM-11 | Run validation and retrieve structured diagnostics | 3 | US-1 | `runValidation`, `getResourceValidation` | browser-read |
| CM-12 | Review the complete framework control inventory | 4 | US-2 | `listApplicabilityControls` | browser-read |
| CM-13 | Read and validate the applicability decisions draft | 4 | US-2 | `getApplicabilityDraft`, `validateApplicabilityDraft` | browser-read |
| CM-14 | Save applicability decisions as a manifest write preview | 4 | US-2 | `putApplicabilityDraft` | browser-write |
| CM-15 | Regenerate the deterministic gap analysis | 7 | US-4 | `analyzeApplicability` | browser-write |
| CM-16 | View the current gap report and its staleness | 3 | US-1, US-6 | `getApplicabilityReport` | browser-read |
| CM-17 | Compare policy and framework mapping subjects | 5 | US-3 | `listMappingSubjects` | browser-read |
| CM-18 | Read and validate the mapping decisions draft | 5 | US-3 | `getMappingDraft`, `validateMappingDraft` | browser-read |
| CM-19 | Save mapping decisions as a manifest write preview | 5 | US-3 | `putMappingDraft` | browser-write |
| CM-20 | Run mapping collection consistency checks | 5 | US-3, US-4 | `checkMapping` | browser-read |
| CM-21 | Build the mapping collection model | 7 | US-4 | `buildMapping` | browser-write |
| CM-22 | Work the review queue with filters, sorting, and paging | 3 | US-1, US-4 | `listReviewQueueItems` | browser-read |
| CM-23 | Reconcile review queue counts | 3 | US-1 | `getReviewQueueCounts` | browser-read |
| CM-24 | Traverse provenance and read bounded source excerpts | 8 | US-6 | `listProvenanceEntries`, `getProvenanceExcerpt` | browser-read |
| CM-25 | Inspect an effect preview before confirming | 6 | US-9 | `getEffectPreview` | browser-read |
| CM-26 | Commit the exact previewed bytes | 7 | US-9, US-10 | `commitEffectPreview` | browser-write |
| CM-27 | Query bounded operations | cross-cutting | US-10, US-13 | `getOperation` | browser-read |
| CM-28 | Cancel a running operation safely | cross-cutting | US-10 | `cancelOperation` | browser-write |
| CM-29 | Prepare a redacted static report export | 8 | US-8 | `prepareReportExport` | browser-write |
| CM-30 | Download a committed export | 8 | US-8 | `downloadExport` | browser-read |
| CM-31 | Enforce read-only launch mode | cross-cutting | US-7 | `registerResource`, `uploadResource`, `preparePolicyConversion`, `putApplicabilityDraft`, `putMappingDraft`, `analyzeApplicability`, `buildMapping`, `commitEffectPreview`, `prepareReportExport`, `initializeApplicabilityDraft`, `initializeMappingDraft`, `prepareProjectBundleExport`, `prepareProjectBundleImport`, `createProjectSourceTransferStage`, `putProjectSourceTransferChunk`, `prepareProjectStagedSourceRestore`, `discardProjectSourceTransferStage`, `prepareProjectSourceStreamExport` | browser-write |
| CM-32 | Render loading, refresh, and empty states from documented reads | cross-cutting | US-1 | `getProjectSummary`, `listResources`, `listReviewQueueItems` | browser-read |
| CM-33 | Recover from errors, conflicts, and lost responses | cross-cutting | US-10 | `commitEffectPreview`, `getOperation` | browser-write |
| CM-34 | Expose machine-readable status for accessible announcements | cross-cutting | US-11 | `getOperation`, `listReviewQueueItems` | browser-read |
| CM-35 | Drive the complete workspace through the published API | cross-cutting | US-13 | `analyzeApplicability`, `buildMapping`, `cancelOperation`, `checkMapping`, `commitEffectPreview`, `downloadExport`, `getApplicabilityDraft`, `getApplicabilityReport`, `getConversion`, `getEffectPreview`, `getMappingDraft`, `getOperation`, `getProjectConfigStatus`, `getProjectSummary`, `getProvenanceExcerpt`, `getResource`, `getResourceValidation`, `getReviewQueueCounts`, `getSession`, `listApplicabilityControls`, `listMappingSubjects`, `listProvenanceEntries`, `listResources`, `listReviewQueueItems`, `preparePolicyConversion`, `prepareReportExport`, `putApplicabilityDraft`, `putMappingDraft`, `registerResource`, `runValidation`, `shutdownSession`, `unlockSession`, `uploadResource`, `validateApplicabilityDraft`, `validateMappingDraft`, `initializeApplicabilityDraft`, `initializeMappingDraft`, `getProjectBundlePreview`, `verifyProjectBundle`, `getLifecycleRecords`, `getLifecycleRecord`, `getLifecycleHistory`, `getLifecycleQueue`, `getFrameworkImpactComparisons`, `getFrameworkImpactComparison`, `getFrameworkImpactChanges`, `getFrameworkImpactFindings`, `getFrameworkImpactPriorDispositions`, `prepareProjectBundleExport`, `downloadProjectBundleExport`, `prepareProjectBundleImport`, `prepareProjectSourceBundleExport`, `downloadProjectSourceBundleExport`, `prepareProjectSourceBundleImport`, `commitProjectSourceBundleRestore`, `getProjectSourceBundleRestore`, `cancelProjectSourceBundleRestore`, `createProjectSourceTransferStage`, `putProjectSourceTransferChunk`, `getProjectSourceTransferStage`, `prepareProjectStagedSourceRestore`, `discardProjectSourceTransferStage`, `prepareProjectSourceStreamExport`, `getProjectSourceStreamExportManifest`, `getProjectSourceStreamExportChunk` | browser-write |
| CM-36 | Initialize scope from a selected registered Catalog | 4 | US-2, US-13 | `initializeApplicabilityDraft` | browser-write |
| CM-37 | Initialize mapping from explicitly selected Catalogs and supplied review metadata | 5 | US-3, US-13 | `initializeMappingDraft` | browser-write |

| CM-38 | Preview complete metadata and acknowledge a local JSON download | cross-cutting | US-7, US-13 | `getProjectBundlePreview` | browser-read |
| CM-39 | Compare chosen metadata JSON bytes with registered fingerprints | cross-cutting | US-7, US-13 | `verifyProjectBundle` | browser-read |
| CM-40 | Inspect lifecycle status, history and owner review queue | cross-cutting | US-1, US-6, US-13 | `getLifecycleRecords`, `getLifecycleRecord`, `getLifecycleHistory`, `getLifecycleQueue` | browser-read |
| CM-41 | Inspect captured framework comparisons, changes, findings and prior dispositions | cross-cutting | US-1, US-6, US-13 | `getFrameworkImpactComparisons`, `getFrameworkImpactComparison`, `getFrameworkImpactChanges`, `getFrameworkImpactFindings`, `getFrameworkImpactPriorDispositions` | browser-read |

| CM-42 | Prepare and download an acknowledged metadata JSON export | cross-cutting | US-7, US-9, US-13 | `prepareProjectBundleExport`, `downloadProjectBundleExport` | browser-write |
| CM-43 | Inspect and confirm complete acknowledged index replacement | cross-cutting | US-7, US-9, US-10, US-13 | `prepareProjectBundleImport`, `getEffectPreview`, `commitEffectPreview` | browser-write |

| CM-44 | Prepare and download acknowledged exact source content | cross-cutting | US-7, US-9, US-10, US-13 | `prepareProjectSourceBundleExport`, `downloadProjectSourceBundleExport` | browser-write |
| CM-45 | Inspect and confirm the complete source restore | cross-cutting | US-7, US-9, US-10, US-13 | `prepareProjectSourceBundleImport`, `commitProjectSourceBundleRestore` | browser-write |
| CM-46 | Recover a source restore outcome after lost reply or restart | cross-cutting | US-7, US-9, US-10, US-13 | `getProjectSourceBundleRestore` | browser-read |
| CM-47 | Request safe source restore cancellation | cross-cutting | US-7, US-9, US-10, US-13 | `cancelProjectSourceBundleRestore` | browser-write |

Full notes for each entry, including the UX states and acceptance criteria
each one covers, live in [capability-matrix-v2.json](capability-matrix-v2.json).

## Coverage Summary

- **Golden path steps 1-8:** every step is covered by at least one entry
  (step 1: CM-01 through CM-05; step 2: CM-07 through CM-10; step 3: CM-06,
  CM-11, CM-16, CM-22, CM-23; step 4: CM-12 through CM-14; step 5: CM-17
  through CM-20; step 6: CM-25; step 7: CM-15, CM-21, CM-26; step 8: CM-16,
  CM-24, CM-29, CM-30).
- **Primary navigation views:** Overview (CM-06, CM-07, CM-16), Review Queue
  (CM-22, CM-23, CM-24), Framework Scope (CM-12, CM-13, CM-14), Mappings
  (CM-17 through CM-21), Policies & Artifacts (CM-07 through CM-11), Trace &
  Reports (CM-16, CM-24, CM-29, CM-30, CM-38, CM-39).
- **Cross-cutting UX states:** loading/empty/refresh (CM-32), error/retry
  (CM-33), cancel (CM-28), shutdown and process-stopped (CM-03), locked and
  throttled unlock (CM-01), permission denied and read-only mode (CM-31),
  expired preview and external conflict (CM-26), stale input (CM-16),
  invalid input (CM-11).
- **User stories:** US-1 through US-13 each appear in at least one entry.
- **MVP capability matrix rows (PRD 062):** Session (CM-01 through CM-05),
  Project (CM-06), Resource registration (CM-07 through CM-09), Policy
  onboarding (CM-10), Validation (CM-11), Applicability (CM-12 through
  CM-16), Mapping (CM-17 through CM-21), Review queues (CM-22, CM-23),
  Provenance (CM-24), Effects (CM-25, CM-26), Operations (CM-27, CM-28),
  Reports/exports (CM-29, CM-30).

## Change Control

Adding a browser-visible capability requires, in order: a PRD-level scope
decision, new or extended operations in the OpenAPI document, a matrix entry
in both files, and representative fixtures. Removing or narrowing any entry
is a compatibility change governed by
[compatibility.md](compatibility.md).

API v2 is selected explicitly with `--api-major 2`. The prior contract 2.1.0 declared 48 operations, adding the nine captured lifecycle and impact reads (CM-40 and CM-41) to the initial 2.0.0 foundation. Default API1/1.2.0 remains 39 operations. Native accessibility, platform, independent and human acceptance remain separate gates.

API 2.2.0 declares 51 operations: the 48 API 2.1 reads/effects retain their wire meanings, with three metadata-only preparation/download routes (CM-42/43). API1 remains 1.2.0/39. Index queries can inspect1000 registrations; these effect routes require the complete consumed physical-file union <=100 and 50 MiB without prefix admission. Source-inclusive restore, crash/rollback qualification, native parity and full S6 acceptance remain open.

API 2.3.0 declares57 operations, preserving the51-operation2.2 contract and adding six explicit source-bundle export/restore operations (CM-44 throughCM-47). Metadata bundle1/2 and all existing public Operation/EffectPreview enums retain their interpretation; source bundle3 is separately explicit and opt-in. The finite first profile retains1MiB request,100-file admitted union,50MiB capture and20MiB shared retention limits. Source restore is complete index-last publication with owned rollback/recovery, not globally atomic visibility for external readers. Larger source transfers, platform/native crash/rollback qualification, independent/human acceptance and full integrated documentation remain open. No API2.3 advertisement is allowed until all six routes are consumed and verified.

## Staged source successor: API2.4

The current draft contract declares eight staged-source operations (65 total) and five actions. API1/1.2.0 remains 39; the inline Bundle3/2.3 checkpoint remains 57. Staged controls require explicit 2.4; the maintained client admits 2.4 for inspection, metadata and inline-source features according to their separate version gates. These declarations do not establish a published release or acceptance; advertisement and conformance evidence must bind the selected runtime/artifact.

| ID | Action | Step | Stories | Operations | Authorization |
|---|---|---|---|---|---|
| CM-48 | Upload and prepare a complete acknowledged staged source artifact | cross-cutting | US-7, US-9, US-10, US-13 | `createProjectSourceTransferStage`, `putProjectSourceTransferChunk`, `prepareProjectStagedSourceRestore` | browser-write |
| CM-49 | Read the current session's retained stage status | cross-cutting | US-7, US-9, US-10, US-13 | `getProjectSourceTransferStage` | browser-read |
| CM-50 | Discard an unconfirmed source stage | cross-cutting | US-7, US-9, US-10, US-13 | `discardProjectSourceTransferStage` | browser-write |
| CM-51 | Prepare and confirm an exact staged-profile source export | cross-cutting | US-7, US-9, US-10, US-13 | `prepareProjectSourceStreamExport`, `getOperation`, `getEffectPreview`, `commitEffectPreview` | browser-write |
| CM-52 | Read manifest and exact parts of a committed staged-source export | cross-cutting | US-7, US-9, US-10, US-13 | `getProjectSourceStreamExportManifest`, `getProjectSourceStreamExportChunk` | browser-read |

**CM-48:** API2.4-only staged transport and complete off-Store preview. Whole declaration is reserved; exact part/full hashes, complete native proposed closure and100paths precede confirmation. Source bytes are sensitive; no prefix, native freshness or source approval from transport.

**CM-49:** Counters are observed retained transport facts.600-second lifetime never renews; unknown/expired returns404 without no-write inference.

**CM-50:** Sticky generation retirement. Actual held raw bytes stay charged until drop; cannot cancel/evict accepted restore authorities.

**CM-51:** Explicit source/sensitivity opt-in; existing Operation/preview/commit, complete conservative output slot. Preparation is not committed download authority.

**CM-52:** Same-session committed private family only. Every request recaptures and fully verifies exact artifact<=10MiB before returning manifest or<=32KiB part; no supplied-path or HTML-family fallback.

The10MiB artifact ceiling is not a cap increase for request/capture/retention. Every old and new admission must share20MiB and256 distinct retained entities; no accepted authority is evicted. Browser/client/native/platform/human/release/full integrated-documentation gates remain open; this matrix supplies traceability, not delivery proof.
