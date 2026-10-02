# Current API verification: development record

The current F04 integration preserves PR #181's client metadata workflows and PR #168's
verification foundation. Historical receipt documents remain byte-for-byte intact.
This record binds pre-commit source observations; it does not certify F04 acceptance.
The [machine-readable record](workspace-current-api-verification.json) retains every
source pin, line count, zero line, named/anonymous function and operation partition.

| Check | Actual result |
| --- | --- |
| Final-source receipt/checkout units | 51 passed; 0 failed; 0 skipped |
| API contract suite | 9 passed; 0 failed/ignored/measured/filtered |
| Workspace process suite | 22 passed; 0 failed/ignored/measured/filtered |
| Release maintained client | 16 assertion groups passed |
| Current operation inventory | 39 declared; 23 observed; 16 unobserved |
| Actual final client attempts | 54: 51 succeeded, 1 rejected, 2 post-close transport failures |
| Pre-commit wrapper | incomplete, exit 1; all suites passed; tracked source dirty; inputs unchanged |
| Hosted context | none; local commit-object-only binding |

The observed release build used locked/offline Cargo, default features and the release
profile. Preserved binary SHA-256 is
`4fdc10a84ee345d3371a8976261ce7805b52b94b81938ae3e4638555f4bb5e6d`
(16,574,128 bytes). All 219 retained shipping source pins equal first parent
`7ecd47df395a1646d13445ab54b240b30f0c2ac6`. Build occurred during the local
integration; scripts were reconciled afterward. Cargo suites use test-profile
executables with unmeasured hashes. These facts establish neither reproducibility
nor startup, package or publication attestation.

## Python coverage and documentation

These are authentic primary-thread line events from the final 51-case unit run,
three-suite wrapper and separate live client run. All source hashes agree.
The table counts physical executable source lines once across those runs;
definition/signature hits cannot substitute for owned function body execution.
Background threads and subprocess line events are untraced. Anonymous lambdas
remain separate from the named-function docstring denominator.

| Source | Hit/executable lines | Whole named docs | PR cohort docs | Cohort owned-body hits |
| --- | --- | --- | --- | --- |
| `scripts/test_verify_workspace.py` | 720/725 | 63/63 | 63/63 | 63/63 |
| `scripts/test_workspace_client.py` | 230/245 | 6/6 | 6/6 | 6/6 |
| `scripts/verify_workspace.py` | 395/448 | 17/17 | 17/17 | 16/17 |
| `scripts/workspace_client.py` | 94/114 | 2/11 | 0/0 | 0/0 |

The introduced/changed cohort has 86/86 docstrings and 85/86 traced owned bodies.
Whole-source named docs are 88/97; the unchanged client library retains nine
legacy docstring gaps. The zero-body member is `scripts/verify_workspace.py:run_command.drain`;
its worker-thread execution is outside this collector. All 93 zero source lines
remain in the JSON. Line hits do not establish branch/MC/DC, Rust, browser,
assistive-technology or complete-runtime coverage.

## Preserved failures and qualifications

Independent source review identified three receipt-validation gaps and a leaked
shared test budget. The source successors address them; actual controls cover
swallowed shutdown errors, aggregate request limits, exact integer headers and
per-case reset. A real-filesystem control injects an unlink failure after final
hard-link publication: passed-looking bytes remain while client main returns 1.
A separate control proves the wrapper rejects that failed producer before reading
the file. Temporary shallow-Git fixture commands use a private empty hooks
directory; the Forge project's mandatory commit hook is unchanged.

Two earlier coverage-collector jobs exited 1 after their 41 unit cases passed,
because Python 3.14's executable-line inventory included artificial None/zero
entries. The failed jobs remain retained. Corrected collection succeeded.
The final derived/2 body map also excludes multiline signature continuations and
nested definition spans; the authentic raw line events remain unchanged. Prior
unit/client runs and static candidate reviews are retained as separate history,
not additional current cases or runtime acceptance.

This snapshot precedes the mandatory enabled hook, clean-commit verifier and
remote/PR/hosted readbacks. Delivery records must bind those checks separately;
the pre-commit incomplete receipt is preserved. Wider ordinary-PR triggers add
eligibility, not an invented hosted result. See the
[current verification guide](plans/2026-10-02-f04-current-api-verification.md) for
receipt contracts, reproduction, status semantics and remaining gates.

The complete browser/platform/API-parity/offline-runtime, dependency/security,
accessibility/user/pilot and release gates remain open. The full documentation
review/update after all remaining roadmap work is still required.
