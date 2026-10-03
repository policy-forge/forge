# Separate stdlib stat-gate observation

When the Linux headless IP-denial verification fails, this standalone observer
can retain one source-bound stat rejection in a separate diagnostic artifact.
The existing experiment, failure status and singleton receipt remain intact.
See the [implementation plan](plans/2026-10-02-f04-stdlib-gate-observation.md)
and [root checks](plans/2026-10-02-f04-stdlib-gate-observation-checks-v4.json).

The workflow identifies the existing verification step as
`headless-denial-verification`. Both the diagnostic and its upload use exactly
`failure() && steps.headless-denial-verification.outcome == 'failure'`.
The sidecar is `stdlib-gate-evidence/stdlib-gate-observation.json`, uploaded as
a separate `workspace-stdlib-gate-<tested SHA>` artifact. Its success cannot
change the original job failure.

The observer runs as the ordinary CI user. It checks the existing fixed
`/usr/bin/python3`, `/usr/bin/ip` and `/usr/bin/sudo` stat and ancestor rules,
then runs only the isolated distro-Python version and `sys.path` probe.
It hashes four fixed public repository inputs for source binding. It reads no
regular stdlib file content and executes neither ip nor sudo. It starts no
namespace, Forge, client or browser, and performs no installation or permission
change. The existing tool trust rules remain in force.

The current `/2` observer checks UID, rejects symlinks by kind, then applies
the `0o022` mask to non-link entries. The preserved `/1` observer used UID,
mode, then kind order.
It accepts two to eight distinct absolute roots and bounds the walk at 20,000
entries, depth 32 and 256 MiB of aggregate regular-file stat size under one
600-second monotonic deadline. A missing optional `.zip` root uses the current
root insertion index for its ancestor check. Probe stdout and stderr share a
256 KiB cap; only stdout reaches strict JSON decoding. Both EOFs, actual exit,
original-owner wait and independent close attempts remain required. These
bounds do not establish syscall preemption or decoder allocation confinement.

The compact canonical ASCII JSON plus LF is at most 2 KiB and has exactly
twelve fields. It can expose one fixed phase/reason pair, source predicate,
zero-based root index, coarse kind, root-UID boolean, `0o022` mask, opaque
object ID, stat stability and ordinary source identity. A metadata recheck
must identify the same instance before rejection facts are retained. Names,
paths, link targets, file contents, raw UID/GID, command output and arbitrary
exception details stay out of the artifact. The digest is an instance
identifier; it does not prevent guessing common filenames.

Identity binds the requested and tested commit assertions to observed checkout
HEAD and before/after SHA-256 and size pins for the observer, unchanged wrapper,
unchanged native producer and workflow. Wrapper and native hashes must match
the protected basis. Changed or unverified identity clears all object claims.
Authenticating requested head, base and ordered parents remains an external
receipt-reader responsibility. Ordinary source pins do not attest loaded or
privileged execution bytes.

`rejected` with stable stat and unchanged identity exits 0: a diagnostic is
available. `unavailable` and `no-rejection` exit 2 and remain incomplete.
There is no passed outcome, complete inventory pin or qualifying tool tuple.
A publication error exits 1. No-replacement hardlink publication can leave a
canonical-looking sidecar after a later cleanup fault; the actual nonzero
producer exit prevents diagnostic credit.

## Historical observer /1 controls

Root executed 30 controls over the applied source using fake process/stat/Git
seams and isolated synthetic fixtures: 30 passed, with no failures, errors or
skips. All 86 named functions/methods, seven classes and two modules have
docstrings; all 86 named functions had observed primary-thread calls.
Physical compiler-line events were positive for 676 of 866 mapped lines;
all 190 zero-hit lines are retained. Production alone had 323 positive and
97 zero-hit lines; controls had 353 positive and 93 zero-hit lines. Imports
preceded tracing. Named entries, physical lines, whole bodies and branches
have different denominators; branch and MC/DC coverage remain unmeasured.
The first 29-control run and its uncovered administrative helper are preserved;
the successor adds one actual-helper control with ten mocked trust/link cases.
These runs do not establish Linux or native-denial qualification.

The immutable integration basis is
`3594b1c4d3185adbcaaf470b870e7258cc86cf90`, parent
`e52ae8cc5c52ff6734fb47bc5dc17436b5bd9a55`. The
[static author audit](plans/2026-10-02-f04-stdlib-gate-observation-static-audit-v2.json)
remains the unexecuted 29-control preparation, with its original 84-function
inventory. It is superseded for execution by the root checks, rather than
silently rewritten.

Historical requested head `3d9345bda6f8c7eb99e69186b6897a874eff4579`
and tested merge `4921c756` in run `37087523880` produced an incomplete wrapper
`/2` result: `tool-untrusted`, `stdlib-entry / worker-writable`, null command
exit and tools, and native producer not run. Cleanup was
`verified-not-created`, not forced; attempted egress was unmeasured. The
original object and cause were not retained. A new sidecar cannot identify
that historical object or cause retrospectively.

The source-parent `/1` sidecar from run `37149736538` has now been read back
with the successful observer and upload steps. It binds requested head
`2064e2933844a827e13b9880c8cfefd1cb2184ee` and tested merge
`8d79c79eb8dc7090e1a99a9274387659e67c4ff7`. Its actual tuple is a stable,
root-owned symlink at root ordinal 0, with mode-mask decimal 18 and the original
`entry-mode-022 / worker-writable` predicate. These are historical stat facts,
not evidence that the target is writable. Its exact target is unobserved.
The wrapper remained incomplete and the native producer did not run.

## Current observer /2

Linux ordinary symlink permission bits are ignored. A root-owned symlink can
have mode `0777` without granting write access to its target. The current wrapper
and observer report `unsupported-link` before testing those ineffective bits;
every such link remains rejected. Non-root ownership retains first priority.
The native producer and its admission rules are unchanged. See
[Linux symlink semantics](https://man7.org/linux/man-pages/man7/symlink.7.html).

Sidecar `/2` retains the actual masked mode on a symlink-kind rejection. It
rejects stale `/1` records and mode-predicate records that claim a symlink is
worker-writable. Non-link kind records still require a zero mask. The existing
source stability, redaction, publication and lifecycle rules remain in force.
The original `/1` artifacts and source pins are preserved rather than relabelled.

Before drafting this successor, the full synthetic wrapper and observer suites
passed **107 and 31 tests**, respectively, with no failures, errors or skips.
All **298 named functions, 17 classes and four modules** have docstrings; all
**14 selected functions** have observed calls and first executable body lines.
The whole-file cohort retains one unobserved function, wrapper `capture_identity`.
Physical compiler lines are **3,004 positive of 3,110**, with all **106 zeros**
retained. Imports were traced; secondary threads and unselected files are excluded.
These controls use explicit mock seams and do not run the privileged experiment.

The [source-bound report](plans/2026-10-03-f04-stdlib-link-diagnostic-verification.json)
retains every function and physical-line record, the failed initial priority
assertions, the historical sidecar binding and the unchanged native/workflow/
dependency pins. Two original first-body rows identify `nonlocal` declarations;
their raw zeros are retained separately from the first executable statement
metric. The successor derives those statement mappings from the same events;
it adds no execution. Positive calls or statements do not prove whole-body,
branch or MC/DC coverage.

Reproduce the mock controls with:

```sh
python3 -B scripts/test_verify_workspace_os_denial.py
python3 -B scripts/test_observe_workspace_stdlib_gate.py
```

Hosted execution of the new diagnostic, any qualified link-target resolution,
and native OS-denial remain open. Full F04, complete S-3, all Must/Should roadmap requirements,
the [supported matrix](adr/0004-supported-browser-and-accessibility-matrix.md),
[PRD 062](PRD/062-prd-local-web-workspace.md),
[PRD 069](PRD/069-prd-dependency-security-audit.md), human acceptance, release
and the final integrated documentation review remain open.
