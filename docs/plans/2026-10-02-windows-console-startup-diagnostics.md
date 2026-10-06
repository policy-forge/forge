# Windows startup diagnostics and retained failed campaigns

This successor corrects two source-proven diagnostic defects in the terminal
helper. An idle writer queue could become a generic native API failure, masking
an earlier prompt timeout. An unexpected reader error could report terminal EOF.
The writer now polls the queue for at most 100 ms at a time, shares the unchanged
absolute 60-second worker deadline, and observes cancellation before teardown.
An earlier typed failure survives later reader and cleanup faults. The reader
records EOF only for a zero-byte successful read or the declared broken-pipe
results 109/232. Cancellation cannot preempt a synchronous native write already
in progress; the existing controller and Job deadline remain responsible for
bounded cleanup.

The first prompt uses the stable ASCII prefix `Set workspace passphrase (`.
The unchanged owned-console mode check must still observe processed input with
echo and line input disabled before every synthetic input. The prefix grants no
input authority by itself. Output code page and cursor rendering are possible
compatibility explanations supported by source review; neither was measured in
the failed hosted campaigns. The native startup cause remains unknown.

## Retained hosted outcomes

The [first run 37062039556](https://github.com/policy-forge/forge/actions/runs/37062039556)
tested merge `b813b6bc3dcf235f587491f16fe9e74391316d7f`, requested head
`54ae3ed9b01a22b86006de74d0c439e15aab114a`. The
[second run 37064244874](https://github.com/policy-forge/forge/actions/runs/37064244874)
tested the corrected CLI launch at requested head
`19232d667e726b24701f5dc555149bd1ab92ef09`. Both used requested base
`a7fc14960027f2820805db838e2243588cd01765`. Their archive digests, bounded
receipt shapes, ordered merge parents and all 72 source pins per run were
qualified against immutable Git objects.

Both native jobs failed with `terminal/native-api-failed`, zero prompts, zero
no-echo observations, no input or HTTP responses, and forced cleanup. They
reported 370 and 138 terminal bytes respectively, unchanged fixture/release
bytes, Job admission and observed Job zero. Their old EOF flags are reported
observations and do not independently prove EOF because of the diagnosed reader
path. Raw terminal text was excluded. Neither count identifies the failure cause.
All three separate API jobs passed in each run. The corresponding general CI
runs passed all three test jobs and failed the separate supply-chain audit.
No native or full-CI pass is inferred.

## Coverage checked before the next push

[The v3 audit](2026-10-02-windows-console-control-audit-v3.json) retains exact
sources, collector and raw transcript pins, every named declaration, all mapped
physical line counts and every zero. Against the applied four-file cohort on
macOS/Python 3.14.8, **63 controls passed with no failures, errors or skips**;
source bytes were unchanged before and after execution.

| Four-file cohort | Documentation | Actual primary-thread observation |
| --- | ---: | ---: |
| Named functions | 183/183 | 178/183 called |
| Classes | 23/23 | no class-body completeness claim |
| Modules | 4/4 | imports preceded tracing |
| Compiler-mapped physical lines | separate metric | 1,530/1,974; 444 zero |

The five unobserved functions remain `NativeAPI.exited`, `NativeAPI.pipe`,
`worker_main`, `execute` and `main`. Native/thread, branch/MCDC, function-body
completeness and whole-repository numeric coverage remain unmeasured.
Independent source review reproduced all 27 author input pins and found no
remaining material finding within this narrow correction. All 34 original
direct native test bodies remain AST-identical. Nine added controls exercise
queue readiness/cancellation/absolute expiry, reader error versus EOF, real
worker timeout preservation, Unicode suffix compatibility, mode rejection and
malformed UTF-8 rejection through fake adapters. The changed/new documentation
cohort is 30/30; its two-file named-function inventory is 144/144.

## Evidence and remaining gates

The [original console guide](2026-10-02-windows-console-verification.md),
[launch correction](2026-10-02-windows-console-launch-correction.md), and v1/v2
audits preserve their historical snapshots. This precommit v3 snapshot records
the mandatory hook and fresh hosted outcome as pending. Draft PR #184 and Veans
child #29 remain In Review under F04 #6. The Win32 ABI, Job containment, six HTTP
expectations, receipt schemas, no-echo requirement and complete pass guard are
unchanged. Fresh hosted native qualification is required.

Browser matrix, platform/API parity, OS-enforced offline runtime, dependency
approval/audit, security/accessibility/AT/user/pilot judgments and release
provenance gates remain open. The full integrated docs review and update is
required after all roadmap work. No merge, publication, contact or human
acceptance is inferred.
