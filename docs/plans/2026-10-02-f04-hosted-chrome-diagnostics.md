# Hosted Chrome tool diagnostics after the first failed attempt

The [first hosted run 37065119949](https://github.com/policy-forge/forge/actions/runs/37065119949)
tested requested head `886f5cef8c3a2f71bdd3c7c76a96d4ae2eed85fa`, base
`19232d667e726b24701f5dc555149bd1ab92ef09`, merge
`191245fb788a5bebce359d0dd217a391a9eaf8c6`. The installed-Chrome job
111030941348 failed during verification after its mocked controls, approved
tool provisioning and release build succeeded. Its retained archive digest and
single receipt member were qualified, along with all 26 source inputs against
immutable Git bytes and the ordered merge parents. The captured merge/head trees
were equal. This is a job-level outcome; the original run snapshot was still in
progress, so it does not qualify all other jobs or general CI.

The original receipt reports `verification-input-invalid`, null browser tools,
unverified input/tool stability and producer `not-run`. Zero browser campaigns
started. Its provided release pin is retained, but it establishes no browser
identity, native pass or startup-loaded-byte provenance. A generic exception
boundary concealed the safe tool failure detail; the actual underlying cause
remains unknown. The original receipt schema `/1`, source, archive and logs
remain unchanged historical evidence.

## Versioned diagnostic contract

The current wrapper emits `forge.workspace-hosted-chrome-verification/2`.
Its additive `diagnostic` is null or exactly four fields: fixed phase
`browser-tool-capture`, allowlisted `step`, allowlisted `reason`, and the exact
signed 32-bit `exit_code` or null when no process status was available. The
producer constructs and the wrapper revalidates these facts. Arbitrary command
output, exception strings, paths, credentials and process IDs are excluded.
Executable resolution, Chrome ELF identity, Node/npm versions, Chrome product,
local package probe/validation and final input pins have distinct safe steps.
Known lifecycle failures retain their fixed reason instead of a generic label.
This contract does not infer a root cause or manufacture an exit status.

Producer schema `/1` and shared API schema `/2` are unchanged. Missing tools,
unknown process status, forced cleanup, unstable input, incomplete campaign
prefixes or a later tool-capture failure cannot pass. Node/npm/package/Chrome
identity checks, four campaigns, no-echo, EOF, actual zero exit and owned-tree
requirements remain intact. The approved development-tool scope is unchanged.

## Applied-source checks before the next push

[The successor audit](2026-10-02-f04-hosted-chrome-control-audit-v2.json) retains
exact sources, raw transcript, every declaration and all compiler-mapped zeros.
The applied four-file cohort on macOS/Python 3.14.8 passed **39 controls, no
failures/errors/skips**, with source bytes unchanged. All 30 original direct
test bodies remain AST-identical; nine added controls cover typed diagnostic
construction/revalidation, failed output redaction, exact/null status and
cleanup/pre-/post-capture failure boundaries through mocked adapters.

| Four-file cohort | Documentation | Actual primary-thread observation |
| --- | ---: | ---: |
| Named functions | 98/98 | 95/98 called |
| Changed/new named functions | 21/21 | 21/21 called |
| Classes/modules | 3/3; 4/4 | imports preceded tracing |
| Compiler-mapped physical lines | separate metric | 700/961; 261 zero |

The author observation reports 700/957 with 257 zeros because its Python trace
helper filters four module/class docstring lines. Root's recursive compiler
inventory retains those four unobserved entries. The common observed-line sets
match; both receipts and the exact reconciliation are preserved. Unobserved
functions are `OwnedTree.settle`, `file_limits` and `campaign`. Primary-thread
calls do not establish every body statement, branch/MCDC, background/native
execution or whole-repository coverage.

Both independent source reviewers found no remaining material finding within
this bounded correction; root replayed 44 and 52 review pins. Root restored the
same approved package installation byte-for-byte from its temporary parking
directory and reran the unchanged actual package probe with approved Node: exit
0, source unchanged. This starts no browser. The earlier 105 fake-DOM checks
remain historical unchanged-asset evidence; they were not rerun for this Python
change.

## Integration and remaining gates

Root locally integrates Windows diagnostic prerequisite
`058bb47b81316c4f6aecc8e855111df4709a34d2` into the Chrome branch. The additive
Chrome workflow and complete Windows/API prefix are preserved. Draft PR #185 and
Veans #31 remain In Review under open F04 #6. This precommit audit leaves the
mandatory hook and fresh hosted outcome pending; subsequent evidence is separate.
No GitHub merge or tracker closure is performed.

Fresh immutable Linux/Chrome qualification is required. Named browser/current-
previous/platform/architecture coverage, OS network denial, accepted dependency
audits, security/accessibility/AT/human/pilot judgments and release provenance
remain open. The full integrated docs review/update remains required after all
roadmap work. No publication, participant contact or human acceptance is inferred.
