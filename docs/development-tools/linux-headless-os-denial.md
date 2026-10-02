# Linux headless OS-denial prerequisite

This F04 development slice prepares a calibrated Linux x86_64 IP-denial experiment for a provided Forge release and the maintained headless client. The final source has passed mocked controls and bounded static review. **No native Linux experiment, Forge/client run, or attempted-egress observation is recorded by this packet.** A future scoped `passed` receipt will still set `acceptance_eligible=false`; it cannot close full F04 or human acceptance.

The original frozen design and source foundation is `ad149144b98fadd22f32c7db84cdf839644f890e`. Root has bound the explicit source-baseline successor on Chrome `cffbb70b4d6a50e73eaf430867bb18f5b0369329`. Its frozen candidate appends the unchanged 2,777-byte OS job to the exact 10,652-byte workflow prefix: 13,429 bytes, SHA256 `b6cf57b5f2035bf7472415cbbddd2609929bb7be9fa956036cefefd6efcf4d0d`. This is completed source/workflow reconciliation; root has also run the exact applied 75 mocked controls successfully with unchanged source. Required-hook and hosted-outcome evidence remain separate gates. The original 10,373-byte workflow and its proposal remain historical evidence. Code, shared API `/2`, client, Chrome/Windows jobs and existing dependencies must retain their intended bytes across reconciliation.

[The implementation plan](../plans/2026-10-02-f04-linux-headless-os-denial.md) and [the v1 control audit](../plans/2026-10-02-f04-os-denial-control-audit-v1.json) record exact source and evidence pins. [Installed Chrome](hosted-chrome.md) is a separate prerequisite: its current diagnostic controls are 60 Python and 22 Node mocks; a fresh native successor remains pending at this snapshot.

## Calibrated experiment and complete denominator

The helper owns three private namespaces: canary, calibration and DUT. One veth pair links only canary and calibration. The DUT has loopback only, including explicit IPv4/IPv6 loopback. The experiment changes no host interfaces, routes, sysctls or firewall rules. Peers are controlled addresses outside the DUT, not proof of public Internet connectivity or of a Forge IPv6 listener.

All 12 ordered rows must pass their actual checks. No successful subset replaces this denominator:

| Phase | Family | Ordered roles and required observations |
| --- | --- | --- |
| before | IPv4 | calibration external challenge connection; DUT external denial with Linux ENETUNREACH 101; DUT loopback challenge connection |
| before | IPv6 | calibration external challenge connection; DUT external denial with Linux ENETUNREACH 101; DUT loopback challenge connection |
| after | IPv4 | calibration external challenge connection; DUT external denial with Linux ENETUNREACH 101; DUT loopback challenge connection |
| after | IPv6 | calibration external challenge connection; DUT external denial with Linux ENETUNREACH 101; DUT loopback challenge connection |

Each connected row requires its fresh challenge to be verified. Each denied row requires exact errno 101. An unreachable IPv6 configuration alone supplies no usable-egress calibration. Fresh link/address inventory and both actual family probes remain required.

The actual unchanged maintained client must run inside the DUT against the protected release and complete its `/2` contract. The native stdlib reader validates the original private client bytes; the ordinary wrapper independently uses the unchanged shared reader, requiring exact canonical byte/hash equality and actual client exit 0. Header or hash equality alone supplies no functional pass. This is the existing 16-group, 39-operation contract, not complete functional exercise of all 39 operations.

`attempted_egress` remains `{ "state": "unmeasured", "count": null }`. Explicit calibration probes do not measure every attempt by Forge. GUI browser denial, DNS, Unix sockets, other channels, exhaustive transient-fork observation and full offline-runtime acceptance remain outside this prerequisite.

## Trust, deadlines and cleanup

Only already-present `ip`, `sudo` and a separately qualified distro `/usr/bin/python3` are used. The native experiment installs no package/tool and introduces no additional development dependency or broad fallback. The ordinary job retains pinned checkout/Rust/setup-Python steps, locked Cargo fetch and locked offline release build; only its closed outer receipt is uploaded. The privileged interpreter, stdlib and their ancestors must be root-owned and non-worker-writable; setup-Python toolcache identity does not establish privileged trust. Administration paths are independently qualified and rechecked by the helper. Native execution also requires Linux x86_64 GNU libc; a bounded actual `gnu_get_libc_version` result qualifies GNU identity before querying the fixed glibc `sigaction` layout. This remains an unexecuted native gate in the mocked evidence.

Noninteractive sudo runs a fixed isolated bootstrap. It reads the bounded helper once, verifies its exact SHA256, then compiles/executes those same bytes with stdlib imports and no checkout imports as root. The private closed plan permits fixed targets, not arbitrary commands. Root-protected release and four client/shared/API copies retain exact hashes and a non-worker-writable hierarchy. The dropped client uses `-E -s -S -B`. File/source bindings are distinct from native startup attestation or reproducible-build proof.

Before release, the blocked-child fence verifies namespace, UID/GID/groups, all capability sets including bounding/ambient zero, no-new-privileges, one thread, descriptor bounds and positive PID/start identity with a held pidfd. Later checks cover only observed instances. Normal and adopted signaling uses held instance-bound pidfds; the sole creator owns wait status. The narrow pidfd-acquisition rollback exception may signal only the original positive, never-reaped direct child after proving sole single-thread wait ownership and normal SIGCHLD handling. It revokes pass before cleanup; adopted PIDs, process groups and a sent signal cannot establish cleanup.

Owned namespace/veth deletion requires detailed link identity, expected ifindex/MAC/veth kind and held no-follow directory authority. Natural exit, both stream EOFs and actual reconciliation of owned resources are required. Forced termination, uncertain wait, failed scans or unreconciled resources cannot earn pass. Selector and every stdin/stdout/stderr close are attempted independently; a close fault discards private output and revokes otherwise-success while preserving an earlier fixed primary failure.

| Engineering bound | Limit |
| --- | ---: |
| Shared absolute native/owned-command budget, including startup and reserved cleanup | 600 seconds |
| Setup / client / each probe / natural grace / cleanup | 30 / 300 / 2 / 5 / 10 seconds |
| Observed owned process instances / proc entries / per-process FD entries | 128 / 20,000 / 256 |
| Private capture / native receipt / outer receipt | 256 KiB each |
| Closed private plan / helper source / provided release | 64 KiB / 128 KiB / 1 GiB |
| Bounded traversal depth / fixed probe rows | 16 / 12 |
| Privileged stdlib inventory entries / bytes / depth | 20,000 / 256 MiB / 32 |

Owned command dispatch checks `now >= deadline` before selector creation and immediately before Popen. Native dispatch is fenced before private-directory work and immediately before sudo; tool observation is fenced at entry and return. Stdout/stderr have separate EOF requirements and a combined output bound. Existing shared identity/tool helpers retain their per-command timeout behavior, so this is not a whole-wrapper 600-second wallclock, preemptible syscall/hash or unchanged-shared-helper process-confinement guarantee.

## Closed receipts and scoped evidence

The ordinary entrypoint is `scripts/verify_workspace_os_denial.py --forge --output-dir`, with existing build and checkout context flags. It writes `workspace-os-denial-verification.json` (`forge.workspace-os-denial-verification/1`). The privileged helper consumes only its bounded `--plan-stdin` plan and writes private `os-denial-smoke.json` (`forge.packaged-runtime-os-denial/1`). Both map passed/failed/incomplete to exits **0/1/2**. Strict readers reject open fields, duplicate/nonfinite data, contradictory status, wrong versions and missing bounds. Publication never replaces evidence; a post-link cleanup fault can leave a passed-looking file, but its nonzero actual exit cannot be accepted.

Missing prerequisites before experiment creation can be incomplete only with verified-not-created facts. After dispatch, that status additionally requires actual exit 2 and a closed helper receipt proving no experiment resources were created; it does not mean no helper process existed. Unknown sudo/helper faults, crash, missing receipt or unknown/forced cleanup fail closed. Uploaded output contains only the closed outer receipt; raw console, HTTP, exceptions, paths, tokens and PIDs remain private.

Author and root each recorded **75 controls, 0 failures, 0 errors, 0 skips**, on exact unchanged sources with fake native/process/tool adapters and bounded synthetic files. None executed privileged Linux facilities, Forge, the client or a browser. All **221/221 named functions**, **12/12 classes** and **3/3 modules** have docstrings; primary-thread profiling observed **199/221 named calls**, leaving 22 unobserved. Control `load()` ran before the collection window. Anonymous callbacks, generated methods, external collectors and unchanged shared/client/API files are outside this introduced-source denominator.

Root's unfiltered compiler table retains **1,855/2,796 physical mapped lines** and **all 941 zeros**. The historical author's `trace._find_executable_linenos` method retains **1,855/2,784** and **929 zeros**, filtering 12 zero class-docstring entries. Common line event counts and all named body-call counts match. The applied-control rerun binds its identical source to that retained profile; it adds no new coverage instrumentation or unique test denominator. Both original methods stay preserved; neither implies all-function-body, branch/MCDC, thread, subprocess, syscall or native coverage.

The copied raw root collector still calls its schema `forge.hosted-chrome-mocked-controls/1` and mentions four native campaigns. Those strings remain byte-original and are inaccurate, non-normative labels here. The OS-specific audit identifies the actual three-file, 75-mock scope; it grants no campaign or Chrome execution credit.

The ordinary wrapper/cleanup reader and root's native-helper reader had separate source-review scopes. Documentation authorship is not independent approval of these guides; static code review is not human security acceptance. Authentic Linux qualification, native execution bindings, the required hook/PR delivery, full F04/platform/browser/AT/human/audit/release gates and the final integrated roadmap documentation review remain open.
