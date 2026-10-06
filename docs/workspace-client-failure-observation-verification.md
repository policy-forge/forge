# Client failure observation: source verification

The [failure-observation guide](workspace-client-failure-observation.md) describes the separate diagnostic and its hosted upload condition. This record reports one exact-source development check; it provides no native Windows, full F04 or human acceptance. The [machine-readable record](plans/2026-10-04-f04-client-failure-observation-binary-output-successor.json) retains the complete measured data, source pins, omissions and previous collector outcomes.

The final full scratch checkout copied all 1,609 tracked files from base `3c06b7fd0d47150d6d01af4918e223033cd446aa`. Only the verifier, its Python fixture suite and the workspace verification workflow changed. The maintained-client producer and library are byte-exact. All 1,609 measured inputs were unchanged before and after the final run.

The final run passed **76 tests**, with zero failures, errors or skips: the original 51 cases and 25 new cases. Fifty original test bodies are byte-exact. The remaining original case adds the explicit expected publication callback to its existing hosted-context forwarding assertion; its exact before/after identities remain retained. The selected AST test-method denominator is 26 because it includes that changed original case, whereas the new-test denominator is 25. These are separate counts.

## Documentation and measured coverage

The selected cohort has **46/46 function docstrings**: 43 new and three changed functions. All 46 entered an actual traced frame and had an observed body line. It has 3/3 documented classes. Across the four measured Python files there are 225/225 documented functions, 9/10 documented classes and 4/4 module docstrings. The inherited undocumented `ReceiptTests` class remains explicit.

| Source | Unique instruction-associated lines hit/available | Named function frames entered/available |
| --- | --- | --- |
| `scripts/verify_workspace.py` | 516/576 | 22/23 |
| `scripts/test_verify_workspace.py` | 1,049/1,056 | 100/100 |
| `scripts/test_workspace_client.py` | 128/631 | 4/28 |
| `scripts/workspace_client.py` | 143/942 | 3/74 |
| Complete four-file scope | 1,836/3,205 | 129/225 |

Selected function bodies hit 501/511 code-local line opportunities. The whole four-file scope retains all 96 unentered named functions and all 1,369 unique source-line zeros. All 306 compiled code records, including module/class/lambda/comprehension scopes, remain in the record. No exact headers or actual frame events were unmapped.

The collector fingerprinted exact path, lexical qualifier, first line and compiled code metadata. Executing a `def` declaration in enclosing code never earned a function-call hit. Raw line intervals and interpreter-instruction exclusions are retained. These instruction-associated line opportunities are not statement, branch, path, MC/DC or complete-body coverage. Calls can include generator resumes and are not invocation totals. Only the primary interpreter thread was traced: the verifier's `run_command.drain` worker and child Python/Git/Forge/Cargo processes remain outside the measurement. The client producer/library were imported and partially exercised by synthetic fixtures; this run did not execute live maintained-client conformance.

## Preserved collector history

The first collector refused a pinned, unrelated browser bytecode file already tracked in the source tree, before importing candidate modules. A successor preserved that file and refused only bytecode caches for the four measured modules. The next attempt stopped during compilation-metadata fingerprinting because CPython 3.14 emitted slice constants; deterministic start/stop/step fingerprints resolved the collector limitation. Neither attempt ran candidate tests.

A third attempt passed all 76 tests but exhausted its five-million-event trace bound while repeatedly parsing the API inventory. That trace remains incomplete. The fourth collector changed only the finite cap to twenty million events and completed with 5,261,201 events. Candidate source stayed identical across those collector attempts. Its [original complete record](plans/2026-10-04-f04-client-failure-observation-verification.json) is preserved byte-for-byte.

A final source successor adds explicit `O_BINARY` where available to the fixed runner-output writer, as required for binary Windows `os.open`. One existing new test now asserts a nonzero binary flag while writing through a genuine private descriptor. The same collector and 76-case cohort completed with 5,261,215 events; this source successor provides the current counts above. It does not constitute native Windows execution or explain the historical failure. Historical attempts are preserved separately and are not added to the final test count or treated as complete coverage.

## Unchanged Rust and remaining gates

All 318 tracked Rust/Cargo files remain byte-equal to the source generation of the preserved outbound whole-workspace LLVM cohort. That authentic earlier run passed 3,204 tests, with three ignored, and measured 79,372/86,925 lines, 7,138/8,432 functions and 138,929/153,211 regions. This is an unchanged-source binding, not a new Rust coverage run. Its uncovered/generated function records and unavailable branch/MC/DC denominator remain explicit in the [outbound verification record](poam-outbound-integration-verification.md).

The required enabled commit hook and resulting-head hosted statuses are separate delivery checks. This pre-commit measurement does not fabricate their outcomes. The diagnostic preserves the original failed producer, later selected suites and final input rechecks. Its upload requires successful new bounded publication; pre-existing files and publication faults receive no flag. A complete flag already written after fresh publication cannot be retracted by a later emitter fault.

PR #204's destroyed private Windows client receipt remains unavailable, so its first failure cause is still unknown. No current success transfers to that historical run. Dependency/security approval, full browser/platform and OS-denial qualification, accessibility/user/pilot evaluation, release provenance and the final integrated roadmap documentation review remain open.
