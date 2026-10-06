# Link-chain diagnostic integration: October 6, 2026

PR #209's exact head `bbb7f00e93ccca2cf876a7fe3a92d03fd11a249d` is composed onto
portable/preflight candidate `d1fdbb27593c359a72706f6a031e0453e282c80b`,
which retains MCP checkpoint `139d8cb0b1b2a749796db9f714fc0d84524ff0bc`. Its workflow conflict
is resolved by retaining the existing protected physical dispatcher and all
current prerequisite steps, plus the ordinary link-chain controls and the
original failure-only observation/upload conditions.

The original stat observer and link observer pinned older wrapper/helper source.
Both reporting-only readers now pin the reviewed helpers already on main:
`verify_workspace_os_denial.py` is 94,727 bytes, SHA-256
`649275b1a78500b480b7c19f3579888faf2267b94c42dc1cc93517109128b1a5`;
`test_workspace_os_denial.py` is 112,060 bytes, SHA-256
`26dbf51d45f053225307e53650243db13f62152da12a73f2e4c8d29b4a85b864`.
The link observer also pins the complete updated stat-observer buffer before
compiling that ordinary private module. Native admission and the protected
helper/wrapper algorithms are unchanged by this integration.

An initial integrated 45-control run found the stat/link pin mismatch. A later
stat-control run exposed its fixture's assumption that production pins were
stale. Both failures remain recorded. The corrected link run passes 46 controls,
including a new actual-source-buffer pin regression; the stat run passes all
31 controls. Its negative fixture now makes same-size mutations only to copied
temporary helper buffers and still checks exact-source and tested-HEAD refusal.
These macOS controls establish neither Linux runtime observation nor native
IP denial, loaded-byte provenance, target admission or product acceptance.
Normal repository hooks and fresh hosted qualification remain separate.

The original local/hosted verification JSON documents are preserved byte-for-byte
with their original populations and source hashes. Their 45-control and physical
line/docstring measurements are historical evidence, not measurements of this
successor. PR #222's historical support-root note is also copied byte-for-byte;
its workflow and all script changes were already present on main. No dependency,
release, tag, tracker acceptance or historical receipt is changed.
