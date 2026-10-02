# Windows controlling-terminal verification prerequisite

This F04 child verifies Forge's browser-mode terminal launch, synthetic
passphrase setup, locked and authenticated API session, and shutdown in an
actual Windows x64 console. It adds a dedicated ordinary-PR job alongside the
three-platform API verifier. The first hosted native
attempt failed; the corrected launch and successor coverage are recorded in
[the launch correction](2026-10-02-windows-console-launch-correction.md).
Local controls do not establish Windows acceptance.

The source base is `a7fc14960027f2820805db838e2243588cd01765` (draft PR #183).
Veans child #29 is In Review under F04 #6 with draft PR #184. No production
Rust behavior, browser provisioning or new dependency is introduced.

## Run and interpret

The hosted `windows-console` job in
[workspace-verification.yml](../../.github/workflows/workspace-verification.yml)
uses the existing pinned checkout, Rust and Python actions. It checks both
standard-library control suites, fetches locked Cargo inputs, builds the release
binary with `--locked --offline --release`, and runs:

```sh
python -B scripts/verify_workspace_windows_console.py \
  --forge target/release/forge.exe \
  --output-dir windows-console-evidence \
  --expected-commit "$FORGE_TEST_COMMIT" \
  --build-outcome "$FORGE_TEST_BUILD" \
  --event "$FORGE_TEST_EVENT" \
  --checkout-kind "$FORGE_TEST_CHECKOUT" \
  --requested-head "$FORGE_TEST_HEAD" \
  --requested-base "$FORGE_TEST_BASE"
```

The last argument applies to a PR. Head/push runs omit `--requested-base`.
An ordinary PR tests its merge checkout; requested head/base, tested commit and
ordered parents are retained separately. The wrapper binds the supplied release
bytes, source inputs and tool observations before and after execution. A build
success field is a workflow-step assertion, not binary loader attestation.
Outputs must be fresh; existing receipts are never replaced.

The retained public artifact is
`windows-console-evidence/workspace-windows-console-verification.json`, schema
`forge.workspace-windows-console-verification/1`. Its native nested result is
`forge.windows-console-smoke/1`. The documented wrapper command returns 0 for
passed, 1 for incomplete/unsupported, and 2 for failed. The nested native producer
returns 0 for passed, 1 for failed, and 2 for incomplete/unsupported. Both retain
synthetic-development evidence
with `acceptance_eligible=false`. Missing, malformed, unstable, mismatched or
failed producer evidence cannot become a passed wrapper receipt. Raw terminal
output, credentials, capabilities and project/native paths are excluded from
the public receipt. API receipt schema `/2` and its three suites are unchanged.

## Native contract

The controller owns a non-inheritable, kill-on-close Windows Job, without
breakaway. A suspended worker is admitted atomically through
`PROC_THREAD_ATTRIBUTE_JOB_LIST` (`0x2000d`). Containment is proved before resume.
The worker owns its ConPTY and creates suspended Forge through
`PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` (`0x20016`); it proves exact current-Job
membership before resume. Failed membership proof terminates the suspended child.

The worker observes the owned Forge console with `AttachConsole` and
`GetConsoleMode` before each synthetic input. Processed input must be enabled,
with echo and line input disabled. Printed prompts or a timer do not establish
no-echo readiness. Concurrent terminal draining is bounded; any synthetic secret
or capability echo fails. No production console mode is changed by the helper.

The six ordered loopback requests are:

| Request | Expected status |
| --- | --- |
| GET project summary while locked | 401 |
| GET session while locked | 401 |
| POST session unlock | 200 |
| GET authenticated session | 200 |
| GET authenticated project summary | 200 |
| POST session shutdown | 200 |

Pass requires the empty read-only fixture and binary bytes unchanged, graceful
Forge exit 0, ConPTY close returned, terminal EOF, worker exit 0, observed Job
active-process count 0, and no forced cleanup. An independent controller deadline
contains blocking ConPTY close on older Windows builds. After successful worker
exit, remaining Job members immediately cause forced cleanup and a failed result;
transient conhost rundown may conservatively fail. Native timing, ABI, nested
Job admission and no-echo behavior require hosted observations.

Primary API references: [ConPTY creation](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session),
[ConPTY close](https://learn.microsoft.com/en-us/windows/console/closepseudoconsole),
[Job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects),
[process attributes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
and [native architecture detection](https://learn.microsoft.com/en-us/windows/win32/api/wow64apiset/nf-wow64apiset-iswow64process2).
The frozen design checked SDK enum values and the locked `rpassword` console path.

## Original precommit coverage snapshot

[The precommit audit](2026-10-02-windows-console-control-audit-v1.json) retains
every literal declaration and compiler-mapped physical line, including zeros.
Both control suites ran against exact integrated sources on macOS/Python 3.14.8:
**53 passed, 0 failed, 0 errors, 0 skipped**. Before/after source bytes matched.

| Four-file cohort | Documented | Primary-thread observations |
| --- | ---: | ---: |
| Named functions | 154/154 | 148/154 called |
| Classes | 19/19 | no native/class-body completeness claim |
| Modules | 4/4 | imports occurred before tracing |
| Compiler-mapped physical lines | not a docstring metric | 1,278 observed / 1,771 total; 493 zero |

The helper's six unobserved functions are `NativeAPI.exited`, `NativeAPI.pipe`,
`worker_campaign`, `worker_main`, `execute` and `main`. The unsupported CLI
subprocess is outside the parent trace. Fake Win32/HTTP adapters exercise
containment, cleanup, strict receipt shape, response ordering, overwrite,
redaction, stability and producer-exit boundaries; calls do not establish every
function-body statement or branch. Native threads, branch/MCDC, Windows behavior
and whole-repository numeric coverage are unmeasured.

Independent frozen source review reproduced all 211 review pins and found the
earlier cleanup-receipt defect fixed in v3. It found no additional concrete open
defect within its static scope. Static review is not runtime or human acceptance. This original snapshot
predates the successful enabled commit hook and failed first hosted attempt.
It did not compare the process argv against the required CLI project option.
The linked successor retains that scope gap, the correction and current counts.

## Remaining completion gates

The metadata file `.gitattributes` is now explicitly LF-pinned for future
checkout identity. Historical Windows CRLF receipts remain byte-preserved.
This prerequisite covers native terminal/session smoke only. Named browser
current/previous product slots, all platform/architecture/API parity,
OS-enforced packaged-runtime network denial, dependency/audit decisions,
security/accessibility/AT/user/pilot judgments and release provenance remain open.
The full integrated documentation review and update is required at the end of
all roadmap work. No GitHub merge, publication, participant contact or acceptance
approval is inferred.
