# ADR-0004: Supported Browser and Accessibility Matrix

> **Document Type:** Architecture Decision Record
> **Audience:** LLM agents, human reviewers, engineering, accessibility
> **Status:** Accepted (PRD-062 Slice 0, 2026-09-08)
> **Last Updated:** 2026-09-08
> **Owner:** Brian Luby

Related: ADR-0003 (static shell and embedded assets), PRD 062 M-17, AC-15,
PRD Non-Goals ("Mobile-first or small-screen use"), the accessibility
requirements document `docs/accessibility/062-accessibility-requirements.md`
(Slice 0 companion artifact), and the PRD launch gate requiring "the supported
desktop-browser matrix is documented and exercised".

## Context

PRD 062 targets desktop browsers only, while still requiring zoom, reflow,
keyboard, and assistive-technology support for its desktop layouts (M-17,
AC-15). The launch gates require that the golden path conforms to WCAG 2.2 AA
through automated and human evaluation with no critical accessibility blocker,
and that the supported browser matrix be documented and exercised in both CI
and manual release gates. The front end is a static, framework-free shell
(ADR-0003), which makes browser-support and accessibility properties a direct
consequence of hand-written HTML/CSS/JS rather than of a framework support
tier. The MVP is a local, single-user tool: there is no telemetry, so support
coverage comes from declared testing, not usage analytics.

## Decision

### Supported desktop browser matrix

Support means: the golden path is executed and evaluated on the listed
browser during release testing, and release notes state the matrix in effect.
Browsers follow evergreen release trains; support is defined per release as
**current stable plus the previous major version**. The matrix is:

| Engine | Browser | Channels | Platforms | Depth |
|---|---|---|---|---|
| Blink (Chromium) | Google Chrome | current stable + previous major | Windows, macOS | Full golden path, automated + manual |
| Blink (Chromium) | Microsoft Edge | current stable | Windows | Smoke verification (same engine as Chrome) |
| Gecko | Mozilla Firefox | current stable + previous major | Windows, macOS | Full golden path, automated + manual |
| WebKit | Safari | current stable + previous major | macOS | Full golden path, automated + manual |

WebKit notes: WebKit is supported **via Safari only**. Safari is a macOS-only
browser in practice (the Windows build is long discontinued), so the Windows
matrix is Chromium (Chrome, Edge) and Firefox; no separate WebKit-on-Windows
slot is claimed. Other WebKit embeddings (Epiphany, mail clients, embedded
webviews) and all mobile browsers are explicitly unsupported (PRD non-goal).

The shell uses baseline web-platform features only (semantic HTML, modern
CSS, ES modules) with no engine-specific APIs, so engine coverage — not
version quirks — is the primary variable. A browser major is added to the
tested matrix when it becomes current stable; the previous-previous major is
dropped at the same time, with the change recorded in release notes.

### Accessibility requirements bound to the matrix

WCAG 2.2 Level AA is the normative target for the complete golden path
(M-17), per `docs/accessibility/062-accessibility-requirements.md`, which
remains the authoritative detailed requirements document. The matrix above
is exercised for, at minimum: complete keyboard-only operability (no
pointer-only P0 interaction, visible focus, logical focus order, no traps);
200% zoom with reflow so content and dialogs, tables, diffs, and
confirmations remain perceivable and operable without two-dimensional
scrolling where WCAG reflow applies; contrast and non-color cues; target
size; and announced status changes and errors (live regions) — matching
AC-15's given/when/then.

### Assistive technology matrix

| Screen reader | Browser | Platform | Gate |
|---|---|---|---|
| NVDA (current stable) | Chrome (current stable) | Windows | Manual release-gate evaluation |
| NVDA (current stable) | Firefox (current stable) | Windows | Manual release-gate evaluation |
| VoiceOver (current macOS stable) | Safari (current stable) | macOS | Manual release-gate evaluation |
| JAWS | any | Windows | **Explicitly excluded from the MVP gate** |

JAWS exclusion rationale: JAWS is commercial, license-gated, and has no
automation path suitable for CI; NVDA already covers the same
Windows screen-reader interaction surface for the two supported Windows
engines, and VoiceOver covers macOS/WebKit. Excluding JAWS keeps the gate
honest and executable rather than aspirational. The exclusion is documented
in user documentation as a known limitation; if pilot feedback shows JAWS is
required by target users, it is added as a best-effort manual evaluation
with findings tracked, via a revision to this ADR.

### CI versus manual gates

- **In CI (every run):** automated structural checks over the embedded shell
  and rendered golden-path views — validity of heading/landmark structure,
  label and name resolution, focus visibility and order invariants, image
  alternatives, and color-independence of state cues — executed against the
  packaged assets together with the browser end-to-end suite, using a
  vendored, offline rule engine (no runtime network, consistent with M-4).
  The headless conformance client additionally asserts accessible names for
  every operable control it drives, so coverage gaps fail CI (PRD M-22
  discipline applied to accessibility).
- **Manual, per release (retained evidence):** keyboard-only completion of
  the golden path; 200% zoom and reflow evaluation; NVDA and VoiceOver task
  completion per the matrix above; focus, dialog, diff/table, status, and
  error-pattern evaluation per the PRD accessibility test layer. Evidence is
  retained with release records alongside the security and transaction gate
  evidence.
- **Blocking rule:** any critical accessibility blocker, or a failed
  matrix-slot evaluation without disposition, blocks release (PRD launch
  gates). Manual evidence is tied to the exact release via the embedded-asset
  hash recorded in release provenance (ADR-0003), so evaluations identify the
  bytes they certified.

## Consequences

- Positive: a small, executable matrix that matches the product's actual
  single-user desktop audience; manual gates are feasible within a release
  cycle and produce retained, auditable evidence.
- Positive: the no-framework shell (ADR-0003) means conformance is earned in
  authored markup rather than fought through framework abstractions; the
  CI-checkable invariants (structure, names, focus) cover the classes of
  defect that regress silently.
- Negative: declaring only two engines plus Safari excludes a tail of users
  (older evergreen majors beyond previous, niche engines); acceptable for a
  local tool bundled with the binary the user already controls, and the
  baseline-features-only constraint makes incidental compatibility likely
  without it being claimed.
- Negative: manual AT evaluation is unrepeatably human; mitigated by tying
  evidence to asset hashes and by scripted keyboard-task checklists that
  evaluators execute verbatim.
- Constraint: every UI slice must land its automated accessibility checks in
  the same PR — accessibility work is not deferred behind visual polish
  (PRD principle 10, risk table).

## Alternatives considered

- **Current-stable-only matrix.** Rejected: evergreen trains move faster
  than enterprise-controlled browser fleets on Windows, where the target
  persona works; previous-major support is cheap here because the shell uses
  baseline features only.
- **Adding JAWS as a best-effort gate slot.** Rejected as a gate: a gate that
  can be silently skipped is worse than an explicit exclusion with a
  documented reversal path.
- **Core-mobility or a reduced conformance profile instead of WCAG 2.2 AA.**
  Rejected: M-17 and AC-15 name WCAG 2.2 AA as the normative target and
  "Accessible is shippable" is a product principle.
- **Automated-only accessibility verification.** Rejected: automated rules
  catch structure and naming defects but cannot evaluate AT behavior, zoom
  usability, or announcement quality; the PRD explicitly requires manual
  keyboard/screen-reader/zoom evaluation as release-gate evidence.
