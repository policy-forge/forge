# F19 browser metadata consumer

This partial PRD 062 S-6 slice adds **Trace & Reports** metadata preview, an
acknowledged local JSON download and explicit chosen-file fingerprint comparison
in writable and read-only sessions. It consumes the existing API 1.2.0 GET/POST
queries; there is no new dependency, operation, role, schema or write authority.
The base is `66c2002dc910f64292c9fb936155a0ca43ca1b01` (draft PR #180).

The browser retains complete index/pin order. File bytes are bounded before
reading and sent unchanged inside the fixed 11-byte wrapper. Preview and
comparison have separate ownership sequences bound to the connected view and
session. Refresh, navigation and shutdown retire obsolete results,
acknowledgment and download URLs. Superseded file reads cannot release newer
busy state, and pacing cannot allow an obsolete comparison POST. Failures keep
unrelated unsaved edits and global Saved status intact.

The long-content probe exposed actual page overflow at 640 and 320 pixels.
`overflow-wrap: anywhere` on the metadata panel corrected that failure. The
optional fixture is separate from ordinary writable/read-only fixture state;
it preserves all four measurements before an overflow assertion.

Before drafting, audit production and helper docstrings against the immutable
base, capture fresh source V8 coverage, exercise actual download/raw POST/strict
duplicate rejection in both native modes, retain the failed reflow predecessor,
and run the enabled commit hook. The
[verification record](../workspace-bundle-browser-verification.md) gives exact
denominators and qualifications. Updated capability, usage and bundle guides
describe this consumer without changing normalized hashing.

Full S-6 still requires server publication/export receipts, source-content
opt-in, confirmed writable import, reviewed batch binding and capacity/retention
qualification. Platform/interoperability, OS-wide offline, security/privacy,
human accessibility/AT/WCAG, pilot and release acceptance remain separate. The
requested full integrated documentation review is an end gate across all
completed roadmap work; these focused guide updates do not close it.
