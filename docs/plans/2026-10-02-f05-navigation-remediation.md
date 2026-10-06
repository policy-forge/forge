# F05: preserve edits and focus across workspace navigation

## Scope and evidence boundary

This focused slice addresses two reproducible defects in the embedded workspace UI:
Inspect/Trace replaced forms without checking their unconfirmed edits, and ordinary
navigation moved focus from a failed-read error summary to the previous heading.
The coordinator selected `main` baseline `aef0ab24` for the isolated F05 branch.
The reviewed pre-change `ui/workspace.js` SHA-256 is
`c6b67c78e71113198c28059cf3ca5751638bc4149485d7ea6730f96d20ad9f81`.

The source review covered the affected UI callbacks and their existing tests. It
was a bounded review, not an exhaustive security scan. Fixture values and API
faults are synthetic development evidence. No owner disposition, independent audit
approval, assistive-technology result, or acceptance judgment is inferred.

## Behavior contract

- Inspect, Trace, Follow, primary navigation, and refresh check the same unsaved-edit
  decision before replacing the current view. Keep editing and native Escape/close
  retain the exact form values and dirty state, make no destination API request,
  and restore a connected enabled invoking control. Only Discard edits consents to
  removing those ephemeral values; saved files are unchanged.
- The discard dialog begins with deliberate focus on its heading. Escape cannot
  activate Discard edits. A button disabled during the decision is reenabled before
  cancellation restores its focus; removed or independently disabled controls are
  never focused as return targets.
- Successful current view installation focuses its destination heading. A failed
  navigation keeps focus on the error summary; the unlock handler likewise focuses
  main content only after its initial render succeeds.
- A provenance read keeps the original view inert while preparing its destination.
  A current failed read preserves the form and its dirty state, reenables the view,
  and surfaces the focused API error. A later retry still needs a discard decision.
  A successful read clears dirty state only when it installs the replacement.
- Render and provenance requests share a sequence counter. A superseded success or
  failure cannot replace the newer destination, announce a stale error, or steal
  its focus. The current request owns restoration of loading/inert controls.

These changes support the focused dialog, form preservation, and destination-focus
requirements in [the accessibility requirements](../accessibility/062-accessibility-requirements.md)
(A11Y-C-1/C-2, A11Y-F-3, and A11Y-FM-4), with keyboard regression cases relevant to
A11Y-K-1. They do not establish complete conformance with any of those requirements.

## Reproducible checks

`ui/tests/workspace-navigation.cjs` uses only Node standard-library modules. It
executes the entire `ui/workspace.js` asset through `vm.Script` with that file's
absolute filename. Its narrow fake DOM models connection, disabled/inert focus
eligibility, callback completion, and modal cancellation. It tests actual Inspect,
Trace, unlock, and navigation callbacks, cancellation and explicit discard, API
failure, safe focus restoration, and superseded success/failure. It does not model
browser rendering, native focus trapping, screen-reader announcements, or WCAG
acceptance.

Run the applied source probes without installing dependencies:

```sh
rtk proxy node --check ui/workspace.js
rtk proxy node --check ui/tests/workspace.cjs
rtk proxy node --test ui/tests/workspace-navigation.cjs
rtk proxy env NODE_V8_COVERAGE=/private/tmp/forge-f05-v8 node --test ui/tests/workspace-navigation.cjs
```

The applied actual-source suite passed 16/16 cases, with zero failures, skipped,
cancelled or todo cases. All seven changed named production functions executed and
have adjacent JSDoc; all four new browser helpers have JSDoc. Whole-asset V8 counts
are 28/64 executed function entries and 88/160 positive raw range entries, not line,
branch or condition coverage. The [verification record](../workspace-navigation-verification.md)
and [receipt](../workspace-navigation-verification.json) retain exact denominators,
uncovered ranges, source/tool/log hashes, and the applied-source path.

Both installed-Chrome POSIX wrappers exited 0 using actual embedded assets matched
byte-for-byte to the candidate source. Chrome 154.0.8037.97 completed 21 exact focus
checks in writable mode and two in read-only mode, with zero page errors or
unexpected page requests. Native dialog close is asynchronous: tests wait for exact
focus restoration rather than treating hidden state as completed cancellation.
The wrapper releases its owned PTY before reaping the synthetic server, resolving
the initial macOS cleanup stall. Initial failed diagnostics and proposal-copy runs
are historical development evidence and receive no acceptance credit.

The browser cases check failed navigation error focus, Inspect Keep editing and
Escape, exact registration and scope textarea preservation, failed provenance
retry, explicit discard and destination focus, and Trace cancellation with an
unsaved report path. These are development checks for one macOS Chrome version.
They do not prove the complete golden path is keyboard-only, manual assistive
technology behavior, the complete platform/browser matrix or WCAG acceptance.

## Remaining F05 and integration gates

The following concrete review observations remain open for separately scoped work:

- Pagination removes its focused controls without choosing a result focus target;
  count/loading/result announcements need review (A11Y-T-6/T-7).
- Long-operation polling changes a non-live connection label while the live status
  retains the initial pending state (A11Y-O-1/O-3).
- Draft-validation displays counts/state without the diagnostic field/resource
  linkage required by A11Y-F-1/F-2.
- Input borders `#8c9b95` measure about 2.90:1 against white and 2.70:1 against
  `#f6f7f4`, below the repository's 3:1 non-text contrast target (A11Y-V-2).
- Failed unlock currently clears the passphrase and focuses the error summary;
  throttled-state focus behavior needs the separate required-state review.
- Browser refresh/tab-close still relies on the native beforeunload prompt; the
  complete A11Y-C-4 requirement remains unresolved by this in-app navigation fix.

The [supported browser and assistive-technology matrix](../adr/0004-supported-browser-and-accessibility-matrix.md),
full keyboard golden path, real screen-reader checks, WCAG 2.2 AA assessment,
[workspace security review and existing F2–F5 dispositions](../SEC/062-sec-local-web-workspace.md),
ASVS scope, numeric/RSS/wall profile approvals, platform filesystem/transaction
limits, Windows qualification, and attributable owner acceptance remain open.
Existing evidence and owner decisions are preserved. This slice does not close
F05 or silently defer any Must Have or Should Have requirement.

After the remaining roadmap work, the coordinator must review and update all
applicable documentation against the integrated behavior and evidence. Passing
these focused checks, a draft PR, or this plan does not satisfy that final full
documentation review or the overall acceptance gate.
