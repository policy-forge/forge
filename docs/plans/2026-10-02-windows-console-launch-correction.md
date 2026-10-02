# Windows launch correction and retained first hosted failure

The native helper now uses `forge workspace --project <fixture> --read-only
--no-open`, matching the required long option in `src/cli/mod.rs`. The original
helper passed the project positionally. The retained local release returned 0
for `workspace --help` and 2 for the old positional command, before workspace
startup. A new control calls the real `worker_campaign` through fake adapters,
captures the complete launch tuple, and stops at process creation. It supplies
no synthetic input and starts no native process.

## First hosted attempt

[Workspace run 37062039556](https://github.com/policy-forge/forge/actions/runs/37062039556)
tested merge `b813b6bc3dcf235f587491f16fe9e74391316d7f`, with requested base
`a7fc14960027f2820805db838e2243588cd01765` and head
`54ae3ed9b01a22b86006de74d0c439e15aab114a`. The native Windows x64 job failed.
Its artifact records `terminal/native-api-failed`, zero prompts, no input,
no no-echo observations, no HTTP responses, and forced cleanup. It reports both
Job memberships, returned ConPTY close, terminal EOF, observed Job zero, and
unchanged fixture/release bytes. Windows build 26100 and Python 3.11.9 were
observed. The three separate API artifacts report passed results.

All four archive digests and bounded receipt bytes were checked and retained.
Remote merge-object and complete source-pin qualification are separate checks.
The demonstrated CLI mismatch is not proven to be the sole cause of the native
failure. The corrected hosted run remains pending at this precommit snapshot;
the failed attempt earns no native pass or product acceptance.

## Successor coverage checked before pushing

[The successor audit](2026-10-02-windows-console-control-audit-v2.json) retains
the exact four-file inventory, full mapped counts and all zeros. Both suites
ran against the applied sources on macOS/Python 3.14.8: **54 passed, no
failures, errors or skips**. Source bytes matched before and after execution.

| Four-file cohort | Documentation | Actual primary-thread observation |
| --- | ---: | ---: |
| Named functions | 158/158 | 153/158 called |
| Classes | 19/19 | no class-body completeness claim |
| Modules | 4/4 | imports preceded tracing |
| Compiler-mapped physical lines | separate metric | 1,341/1,791; 450 zero |

The helper's five unobserved functions are `NativeAPI.exited`, `NativeAPI.pipe`,
`worker_main`, `execute` and `main`. Calling `worker_campaign` through adapters
does not establish native execution or all of its statements. Native threads,
branch/MCDC coverage and whole-repository numeric coverage remain unmeasured.
Independent review checked the exact correction against the CLI/help receipts
and confirmed the original 33 native controls remain AST-identical. The original
53-control audit and initial static review remain unchanged historical evidence.

The mandatory commit hook is pending at this precommit snapshot. Veans child #29
is In Review under F04 #6, with draft PR #184. The existing wrapper/native exit
maps and API receipt `/2` remain as documented in
[the console guide](2026-10-02-windows-console-verification.md).
All remaining browser, parity, network-denial, dependency, security,
accessibility, human, pilot and release gates stay open. Full integrated docs
review is required after all roadmap work.
