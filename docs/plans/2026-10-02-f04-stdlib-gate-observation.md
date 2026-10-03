# F04: separate bounded stdlib stat observation

The Linux headless-denial gate currently retains an incomplete tool-trust
failure without the rejected object's stat identity. This slice adds a
separate ordinary-user observer after that gate fails. It keeps the original
experiment, trust rules and singleton artifact unchanged. The original job
failure and incomplete receipt remain unchanged.
The [guide](../workspace-stdlib-gate-observation.md) describes the diagnostic;
the [root checks](2026-10-02-f04-stdlib-gate-observation-checks-v4.json) retain
actual control results and full coverage inventories.

The integration basis is immutable
`3594b1c4d3185adbcaaf470b870e7258cc86cf90`, parent
`e52ae8cc5c52ff6734fb47bc5dc17436b5bd9a55`. Only two scripts, the additive
workflow, guide, plan and static audit were initially applied. The unchanged
wrapper, native helper and shared verifier in the TEMP candidate were protected
fixtures and were not applied. All 1,282 other tracked files and the existing
tracker configuration were preserved. This root checks successor adds one
audit path; prior plans, failed receipts and historical audits remain intact.

The closed `forge.stdlib-gate-observation/1` sidecar is at most 2,048 bytes
and has twelve fields. It retains one original stat predicate only after a
same-instance recheck and unchanged ordinary source identity. Source change,
missing/untrusted tools, limits, malformed probe output or observation faults
yield unavailable. No-rejection remains incomplete. The observer reports no
qualification pass, inventory, effective authority or cause. It never runs ip,
sudo, native denial, Forge or a browser, changes permissions or installs tools.

The workflow adds a verification-step ID, one control command and separate
diagnostic/upload steps conditioned exactly on the existing verification
failure. The observer still uses the fixed distro-Python path and its existing
stat trust rules. Its walk is streamed and bounded by root count, depth, entry
count, aggregate stat size and a single deadline. Separate probe streams share
the cap; EOF, actual exit, original-owner wait and independent cleanup remain
part of admission.

Root ran the actual controls twice, retaining both source and raw outputs.
The original 29 controls passed with 84/84 documented functions, 83 observed
named entries and 608 positive / 220 zero-hit physical compiler lines. The
sole uncalled helper was `administrative_tool`. One additional control then
exercised its real trust and ancestor logic with ten mocked metadata/link
cases, including the sixteen-link limit and no process or content opening.
The final 30 controls passed with no failures, errors or skips. Production
bytes remained unchanged.

The final inventory has 86/86 documented named functions, 86 observed named
entries, 7/7 documented classes and 2/2 documented modules. It retains every
compiler-mapped physical line, including zeros: 676 positive / 190 zero-hit
across 866 mapped lines. Production contributes 323/420 positive; controls
353/446. Imports happened before tracing and the profiler covers the primary
thread. Function entry does not prove whole-body or branch coverage; branch
and MC/DC remain unmeasured. Fake OS/process/Git seams and temporary synthetic
fixtures do not qualify the Linux runtime or native-denial experiment.

The [static author audit v2](2026-10-02-f04-stdlib-gate-observation-static-audit-v2.json)
is retained byte-for-byte as the unexecuted 29-control rebase preparation.
Its 84-function inventory and null runtime metrics describe that historical
source. The root successor binds the applied source, expanded controls, tool,
collector, raw job receipts and all named-call/line counts. The bounded
independent source review covered the frozen original candidate and rebased
docs; it did not execute controls or approve its own historical authorship.

Historical run `37087523880` tested merge `4921c756` of requested head
`3d9345bda6f8c7eb99e69186b6897a874eff4579`; it remains incomplete and
native-not-run, with no original-object or cause attribution. A later hosted
sidecar must be independently source-bound with actual producer status.
Mandatory hooks, PR and tracker readbacks are separate delivery evidence
after this precommit checks checkpoint. Full F04, complete S-3, all scoped
Must/Should requirements, the [supported matrix](../adr/0004-supported-browser-and-accessibility-matrix.md),
[PRD 062](../PRD/062-prd-local-web-workspace.md),
[PRD 069](../PRD/069-prd-dependency-security-audit.md), human acceptance, release
and final integrated documentation review remain required. This slice does
not grant owner, audit or product acceptance.
