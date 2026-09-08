# 062-accessibility-requirements

> **Document Type:** Accessibility Requirements
> **Audience:** Product, design, engineering, accessibility evaluators, LLM agents, human reviewers
> **Status:** Draft
> **Last Updated:** 2026-09-08 <!-- @auto -->
> **Owner:** Brian Luby <!-- @human-required -->

---

## Review Tier Legend

| Marker | Tier | Speckit Behavior |
|--------|------|------------------|
| 🔴 `@human-required` | Human Generated | Prompt human to author; blocks until complete |
| 🟡 `@human-review` | LLM + Human Review | LLM drafts → prompt human to confirm/edit; blocks until confirmed |
| 🟢 `@llm-autonomous` | LLM Autonomous | LLM completes; no prompt; logged for audit |
| ⚪ `@auto` | Auto-generated | System fills (timestamps, links); no prompt |

---

## Scope and Normative References 🟡 `@human-review`

This document defines the **normative accessibility interaction requirements** for the complete
PRD 062 golden path in the bundled web workspace. It specifies requirements, not visual designs;
component-level designs must satisfy these requirements and be confirmed in human review. There
is no UI implementation in Slice 0 — these requirements gate the UI when it is built and are the
basis of the accessibility evaluation plan.

**Normative target:** WCAG 2.2 Level AA. "Accessible is shippable" (PRD Product Principles and
Guardrails 10); accessibility is not deferred to visual polish.

**In scope:** every view and state of the golden path (launch/locked, register, diagnose, review
scope, review mappings, confirm, regenerate, trace/export, shutdown), in both writable and
`--read-only` sessions, and in setup mode (no `forge.workspace.json` index).

**Out of scope:** mobile/small-screen layouts (PRD Non-Goals; desktop layouts still meet zoom,
reflow, keyboard, and assistive-technology requirements), and any UI beyond the MVP golden path.

**References:**

| Reference | Use |
|-----------|-----|
| [062-prd-local-web-workspace.md](../PRD/062-prd-local-web-workspace.md) | Parent PRD: M-17, AC-15, US-11, "Required UX States", Verification Plan layer 8 |
| [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | Normative conformance target (Level AA) |
| [0004-supported-browser-and-accessibility-matrix.md](../adr/0004-supported-browser-and-accessibility-matrix.md) | Supported browser and assistive-technology matrix (mirrored/extended below) |
| [062-sec-local-web-workspace.md](../SEC/062-sec-local-web-workspace.md) | Security rendering constraints (SEC-REN) that share ownership of the rendered-content allowlist |
| [forge-workspace-v1.openapi.yaml](../api/forge-workspace-v1.openapi.yaml) | Normative API contract: error envelopes carry the machine-readable codes and retryability the UI must surface |

---

## Component Interaction Patterns 🟡 `@human-review`

Each pattern states requirements with RFC 2119 keywords and traces to PRD clause IDs and WCAG
2.2 success criteria (informative mapping; WCAG is the normative target). "The UI" means the
bundled web application operating as a client of the published `/api/v1` contract.

### Data Tables (review queue, control inventory, mappings)

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-T-1 | Every data table MUST expose a programmatic caption or an associated summary (visible or programmatically related) identifying its content (e.g., "Review queue — unresolved applicability items") | US-1; US-2; M-7 | 1.3.1 |
| A11Y-T-2 | Row and column headers MUST be programmatically associated (`th`/scope semantics); header relationships MUST survive sorting and filtering | M-11 | 1.3.1 |
| A11Y-T-3 | Sortable columns MUST expose sort state programmatically (e.g., `aria-sort`) and MUST announce the new sort order and resulting count when activated | M-11; Review queues capability (filter/sort fields) | 4.1.3 |
| A11Y-T-4 | Row actions MUST be reachable and operable by keyboard in a documented, consistent order; a row's actions MUST be discoverable without pointer hover | US-2; US-3 | 2.1.1 |
| A11Y-T-5 | Tables MUST support cell-by-cell navigation by keyboard (documented arrow-key pattern) or an equivalent documented non-pointer traversal; the chosen pattern MUST be uniform across all tables | US-1 through US-3 | 2.1.1 |
| A11Y-T-6 | Queue counts MUST be accurate, announced on change, and reconcile to the unfiltered denominator; activating a count MUST move focus to the resulting filtered item list, not merely load it | AC-9; Review queues capability | 4.1.3, 2.4.3 |
| A11Y-T-7 | Loading, empty, and error states within table regions MUST be rendered in place and announced, without silently blanking the table | "Required UX States" | 4.1.3 |

### Diffs (preview confirmation)

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-D-1 | A semantic change summary MUST be presented and reachable before the text diff; a user MUST be able to understand what will change without reading raw diff text | Write Preview and Transaction Model step 3; M-13; US-9 | 1.3.1 |
| A11Y-D-2 | Diff regions MUST be labeled (additions, deletions, unchanged context); additions and deletions MUST be distinguishable non-visually as well as by color (for example text markers or labels in addition to background color) | M-13; US-9 | 1.4.1 |
| A11Y-D-3 | Diff lines/context MUST expose anchors or stable references so a user can navigate and re-locate a change; keyboard navigation MUST reach every changed region | US-9 | 2.1.1, 2.4.3 |
| A11Y-D-4 | The preview MUST present the exact target, create/overwrite status, current target version/hash, and validation outcome in text form (not color or icon alone), restated at confirmation | M-13; AC-10; Write Preview step 3 | 1.4.1, 3.3.4 |

### Source Excerpts (provenance)

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-E-1 | Excerpts MUST be bounded and rendered as inert, escaped text (per the security allowlist) while preserving quoted context needed to interpret them | M-12; Provenance capability (bounded excerpts); SEC-REN | 1.3.2 |
| A11Y-E-2 | Opening an excerpt MUST move focus to a focusable excerpt target, and the excerpt MUST provide a back-reference (return link) to the originating item that restores focus on activation | AC-9; US-6 | 2.4.3 |
| A11Y-E-3 | Excerpt loading MUST NOT cause layout shift that moves the focus target or the reading position; reserved space or in-place rendering MUST be used | US-6 | 1.4.10 (reflow stability) |
| A11Y-E-4 | The link from any count or classification to its evidence MUST be operable by keyboard and MUST announce its destination (control, decision, mapping edge, policy subject, source location, fingerprint) | AC-9; M-12 | 2.4.4, 4.1.2 |

### Dialogs and Confirmations

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-C-1 | Modal dialogs MUST trap focus within them while open, set initial focus deliberately (to the primary content or first control, documented per dialog), and restore focus to the invoking element on close | AC-13; "Required UX States" | 2.1.2, 2.4.3 |
| A11Y-C-2 | Esc MUST close or cancel a dialog (per its documented semantics); Esc MUST NEVER activate a destructive confirmation by itself | M-13; AC-10 | 2.1.2, 3.3.4 |
| A11Y-C-3 | Confirmations of pending actions MUST restate the target (project-relative destination), the base/target hashes, and the observed resource version in text before the confirming control; the confirming control MUST be distinct from and follow a cancel affordance in focus order | M-13; AC-10; AC-21 | 3.3.4 |
| A11Y-C-4 | The uncommitted-edits warning on refresh/close MUST be an accessible dialog (not a browser-native-only prompt) whose text explains that only ephemeral form state is lost and files are unchanged | AC-13 | 3.3.4 |
| A11Y-C-5 | Dialog open/close and pending-operation state MUST be announced | M-24 | 4.1.3 |

### Forms and Validation Errors

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-F-1 | On submit failure, an error summary MUST appear at the top of the form, link each issue to its field or resource, and receive focus | "Required UX States"; M-18 | 3.3.1 |
| A11Y-F-2 | Each field with an issue MUST expose a programmatic per-field message; messages MUST state whether retrying can succeed where the API error carries retryability | M-18; HTTP and Resource Semantics (stable error codes) | 3.3.1, 3.3.3 |
| A11Y-F-3 | Failed submissions MUST preserve user-entered in-memory form values when safe; the user MUST never re-enter lost decisions | "Required UX States"; AC-13 | 3.3.7 (Redundant Entry) |
| A11Y-F-4 | Validation results MUST be announced on submit (status message), not only on later inspection; asynchronous draft validation results MUST be announced when they arrive | M-9; M-10; "Required UX States" | 4.1.3 |
| A11Y-F-5 | Error content MUST surface the machine-readable code, safe message, affected resource/field, and retryability from the API error envelope verbatim in semantics — the UI MUST NOT re-derive or paraphrase error meaning beyond the contract | M-18; HTTP and Resource Semantics; OpenAPI contract reference | 3.3.1, 3.3.3 |
| A11Y-F-6 | Required decision fields (state, rationale, reviewer, review time) MUST be programmatically identified as required before submission, not only rejected after | M-9; M-10 | 3.3.2 |

### Status, Progress, and Operations

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-O-1 | Operation state transitions (pending, running, cancelled, failed, succeeded) MUST be rendered through a live region or status role so they are announced without focus change | M-24; AC-23 | 4.1.3 |
| A11Y-O-2 | Progress facts, when available, MUST be text-conveyable; where only indeterminate progress exists, the state text MUST say so | M-24; S-4 | 4.1.3 |
| A11Y-O-3 | Cancellation MUST be keyboard-operable and its request and terminal cancellation status MUST be announced; completion of a long operation MUST be announced | M-24; S-4; AC-23 | 4.1.3 |
| A11Y-O-4 | Operation state MUST NEVER be conveyed by color or motion alone (for example, a spinner or red badge without text) | "Required UX States"; M-17 | 1.4.1 |
| A11Y-O-5 | After response loss or disconnect, the UI MUST present the queryable operation state (per AC-23) with an announced path to determine whether an effect committed | M-24; AC-22; AC-23 | 4.1.3 |

### Focus Management

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-FM-1 | The keyboard focus indicator MUST be visible at all times and MUST NOT be obscured by sticky headers, dialogs, or overlays | M-17 | 2.4.7, 2.4.11 |
| A11Y-FM-2 | Focus order MUST follow the logical reading/operating order of each view; primary navigation precedes view content | M-17 | 2.4.3 |
| A11Y-FM-3 | A skip link to main content MUST be the first focusable element on every view | M-17 | 2.4.1 |
| A11Y-FM-4 | View changes MUST move focus deliberately to a defined target (heading or primary control) and the change MUST be announced; focus MUST NOT be dropped to the document start on SPA-style navigation | M-17 | 2.4.3, 4.1.3 |

### Keyboard Operation

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-K-1 | The complete golden path — launch/unlock, registration, review, preview, confirm, export, and shutdown — MUST be operable by keyboard alone with no pointer-required interaction at any step | US-11; M-17; AC-15 | 2.1.1 |
| A11Y-K-2 | Any dragging operation (for example reordering) MUST have a non-dragging keyboard equivalent, or dragging MUST NOT be used | M-17 | 2.5.7 |
| A11Y-K-3 | The UI MUST NOT trap keyboard focus; Esc or a documented sequence MUST always exit any mode | US-11; M-17 | 2.1.2 |
| A11Y-K-4 | All shortcuts and escape hatches MUST be documented in-product or in linked documentation; shortcuts MUST NOT conflict with assistive-technology and browser shortcuts and MUST be remappable or avoidable where conflict is reported | US-11; M-17 | 2.1.4 |

### Zoom and Reflow

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-Z-1 | The workspace MUST remain functional and conformant at 200% browser zoom with content reflowing; content MUST NOT require horizontal scrolling in one dimension at 200% zoom / 1280 CSS px equivalent viewport (~640 px) | M-17; AC-15; In Scope (responsive through 200% zoom) | 1.4.4, 1.4.10 |
| A11Y-Z-2 | Text MUST remain resizable to 200% (including via user-agent text scaling) without loss of content or function | M-17 | 1.4.4 |
| A11Y-Z-3 | Data-dense regions (tables, diffs) MAY scroll within the region at high zoom, but labels, actions, and status MUST remain perceivable and operable without bidirectional hunting | M-17 | 1.4.10 |

### Contrast, Target Size, and Non-Color Cues

| Req ID | Requirement | PRD clause | WCAG SC (informative) |
|--------|-------------|-----------|------------------------|
| A11Y-V-1 | Text MUST meet 4.5:1 contrast (3:1 for large text) against its background in all themes/shipped variants | M-17 | 1.4.3 |
| A11Y-V-2 | UI component states, icons, focus indicators, and diff/status boundaries MUST meet 3:1 non-text contrast | M-17 | 1.4.11 |
| A11Y-V-3 | Pointer targets SHOULD be at least 24 x 24 CSS px, or have equivalent spacing; smaller targets MUST remain keyboard-operable alternatives where they exist alongside | M-17 | 2.5.8 |
| A11Y-V-4 | Every state — including under-review, conflict, invalid, stale, read-only, and locked — MUST carry a redundant non-color indicator (text label, icon with text alternative, or pattern) | M-7; M-11; "Required UX States"; US-11 | 1.4.1 |

---

## Required UX States — Accessibility Rules 🟡 `@human-review`

PRD "Required UX States" requires every primary view to define and test these states; errors
preserve user-entered in-memory form values when safe, move focus to a summary, link each issue
to its field or resource, and state whether retrying can succeed. For each state below, the UI
MUST announce the state change (live region/status), direct focus appropriately, and offer the
next action. "Next action" MUST be a keyboard-reachable control unless noted.

| State | Announced as | Focus direction | Required next action |
|-------|--------------|-----------------|----------------------|
| Locked | "Workspace locked — passphrase required" on view load | To the unlock form's primary field | Unlock submit; guidance to the launch terminal if unlock cannot proceed |
| Throttled unlock | "Too many attempts — retry available in N seconds" (delay stated) | Remain on passphrase field (do not steal focus) | Wait then retry; stop-and-relaunch guidance if repeated |
| Loading | Region-specific "Loading <resource>" without focus theft | Preserved on the triggering control (disabled with text, not silently) | None required; cancellable where the operation model allows |
| Empty | "No <items> yet" with a one-sentence explanation | To the empty-state's primary action | The state's primary action (for example register a resource); setup mode explains registration/upload without scanning (AC-5) |
| Ready | View title/heading of the loaded state | To view heading or first content control | Golden-path continuation (next recommended human action per Overview) |
| Invalid input | Error summary with count and per-field links | To the error summary, then linked fields | Field correction; values preserved (A11Y-F-3); retryability stated |
| Review required | Item classification with its reason code in text | To the first review item | Record decision (rationale/reviewer fields required, M-9/M-10) |
| Stale input | "Inputs changed since this view was loaded" with affected resource | To the affected resource link | Refresh/reload the affected view; no silent revalidation into decisions |
| External conflict | Typed conflict text including the new version (AC-21) | To the conflict explanation | Reload/re-preview against current version; original files preserved |
| Permission denied | "Permission denied" with affected target and retryability | To the message | Retry (if retryable) or remediate outside the workspace; no data changed |
| Expired session | "Session expired — workspace stopped or restarted" | To the explanation | Stop-and-relaunch guidance; in-progress form state warning per AC-13 semantics |
| Process stopped | "Workspace process stopped" on the final state | To the terminal message | Terminal guidance (relaunch); confirmation that files were left unchanged or fully committed (AC-17) |

---

## Assistive Technology and Browser Matrix 🟢 `@llm-autonomous`

This matrix mirrors [ADR-0004](../adr/0004-supported-browser-and-accessibility-matrix.md) and
records the MVP evaluation commitments; where ADR-0004 extends it, ADR-0004 governs.

| Combination | Platform | Role in MVP | Evidence |
|-------------|----------|-------------|----------|
| NVDA + Chrome (latest supported) | Windows | Primary screen-reader target | Manual golden-path pass each release gate |
| NVDA + Firefox (latest supported) | Windows | Secondary screen-reader verification | Manual golden-path pass each release gate |
| VoiceOver + Safari (latest supported) | macOS | Primary macOS screen-reader target | Manual golden-path pass each release gate |
| JAWS (any version) | Windows | **Explicitly excluded from the MVP verification matrix.** FORGE MUST NOT claim or imply JAWS support in release notes or documentation until a JAWS pass is executed and evidence is retained. JAWS-specific defect reports are handled as normal WCAG 2.2 AA defects, but no conformance claim through JAWS is made. Revisit trigger: first enterprise/pilot request or any hosted-release planning | None in MVP; disposition recorded here |
| Keyboard-only (no AT) | Across the supported browser matrix (per ADR-0004) | Full golden-path keyboard operability | Manual pass + automated focus-order checks in CI |
| 200% zoom / reflow | Across the supported browser matrix | Zoom and reflow conformance | Automated layout assertions in CI + manual spot checks |
| High-contrast modes (Windows High Contrast; macOS Increase Contrast / reduce transparency) | Windows, macOS | Best effort: spot-check the golden path; blocking issues are logged and dispositioned but high-contrast mode is not a claim surface in MVP | Manual spot check, non-blocking with disposition |

The accessibility evaluation MUST cover every state in "Required UX States — Accessibility
Rules" on at least the primary screen-reader targets, and the complete golden path (including
read-only mode and setup mode) on all manual rows.

---

## Verification Mapping 🟡 `@human-review`

Maps requirement areas to PRD Verification Plan **layer 8 (accessibility tests)** and **AC-15**.
Automated checks catch only a subset of WCAG failures; automated passes alone are never
sufficient evidence for AC-15.

| Requirement area | Automated in CI (feasible) | Manual at release gate | Layer 8 tie-in | AC |
|------------------|---------------------------|------------------------|----------------|----|
| Data tables (A11Y-T-*) | Table/header/`aria-sort` presence; caption association; count-announcement hooks | Screen-reader navigation of queue/inventory/mapping tables; cell traversal pattern | "table" testing | AC-15 (via AC-9 behavior) |
| Diffs (A11Y-D-*) | Label/marker presence on diff regions; summary-before-diff ordering in DOM | Screen-reader comprehension of additions/deletions; hash/target restatement | "diff" testing | AC-15 (via AC-10 behavior) |
| Source excerpts (A11Y-E-*) | Focus target and back-reference presence; bounded rendering (with security corpus) | Focus round-trip; no layout shift under AT; context comprehension | "table/diff" adjacent + provenance | AC-15 (via AC-9 behavior) |
| Dialogs/confirmations (A11Y-C-*) | Focus trap/restore smoke tests; dialog role presence | Esc semantics; confirmation restatement audit (target/hashes); uncommitted-edits warning | "dialog" testing | AC-15 (via AC-10/AC-13 behavior) |
| Forms/errors (A11Y-F-*) | Label/programmatic-association rules; error-summary link presence; code surfacing from contract fixtures | Announcement on submit; value preservation after failure; retryability clarity | "error" testing | AC-15 (via M-18) |
| Status/operations (A11Y-O-*) | Live-region/status-role presence on operation views | Announcement of transitions, cancellation, completion; disconnect recovery flow | "status" testing | AC-15 (via AC-23 behavior) |
| Focus management (A11Y-FM-*) | Focus-visible, skip-link, focus-order automation | Focus movement on view change; non-obscured focus under overlays | "focus" testing | AC-15 |
| Keyboard operation (A11Y-K-*) | Keyboard-only scripted golden-path run (registration → review → preview → confirm → export → shutdown) | Keyboard-only full pass per browser matrix; shortcut conflict review | "keyboard" testing | AC-15 (US-11) |
| Zoom/reflow (A11Y-Z-*) | 200% zoom layout assertions; no-horizontal-scroll checks | Human verification of readability and operability at 200% | "zoom/reflow" testing | AC-15 |
| Contrast/target/non-color (A11Y-V-*) | Contrast rule sweeps on shipped variants; non-color-state marker presence | Non-color cue audit for under-review/conflict/invalid/read-only/locked; target-size spot checks | Included in manual pass | AC-15 |
| Required UX states | State-role/announcement hooks present per state | Full state-matrix walkthrough with AT on primary targets (all 12 states) | Layer 8 covers "status" and "error" | AC-15 (with M-18) |
| AT/browser matrix | Zoom/keyboard automation across browser matrix | NVDA+Chrome, NVDA+Firefox, VoiceOver+Safari golden-path passes; high-contrast best-effort spot checks; JAWS exclusion upheld in comms | Layer 8 definition | AC-15 |

**Release-gate rule:** AC-15 passes only when (a) automated accessibility checks pass in CI for
the shipped UI, (b) the manual matrix rows above have retained evidence for the current
release, and (c) no critical accessibility blocker is open (PRD Launch Gates).

---

## Cross-References ⚪ `@auto`

- **Security rendering interlock:** the escaped-text / allowlisted-Markdown decision
  ([062-sec-local-web-workspace.md](../SEC/062-sec-local-web-workspace.md), SEC-REN-1/4) jointly
  owns the rendered-content allowlist with this document: the allowlist MUST retain the semantic
  structure (headings, lists, emphasis, table semantics) that accessible rendering depends on,
  while excluding everything the security review prohibits. Neither review may change the
  allowlist alone.
- **Error envelopes:** all user-facing error semantics originate in
  [forge-workspace-v1.openapi.yaml](../api/forge-workspace-v1.openapi.yaml) (stable codes, safe
  messages, affected resource/field, retryability, correlation ID, resource version). The UI
  surfaces these; it MUST NOT re-derive error meaning (A11Y-F-5).

---

## Changelog ⚪ `@auto`

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-09-08 | ZCode (LLM) | Initial accessible interaction requirements, UX-state rules, AT/browser matrix (with explicit JAWS disposition), and verification mapping for PRD 062 Slice 0 |
