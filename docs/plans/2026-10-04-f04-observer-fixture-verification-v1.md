# F04 historical observer fixture: verification successor v1

Draft #213's initial Linux run failed one ordinary observer test before the physical campaign. The production observer pins historical qualifier bytes. Copying the newer qualifiers into its test fixture could not satisfy those historical pins. All 22 physical controls were skipped, the build/workload did not run and attempted egress stayed unmeasured. The original job log and artifact bytes are preserved in [the complete successor receipt](2026-10-04-f04-observer-fixture-verification-v1.json).

Only one existing test changes. It first proves that the historical production pin map refuses the newer actual source bytes, then temporarily selects exact synthetic fixture pins to exercise real bounded source reads, tested-HEAD refusal before source reads and a same-size source mutation. Production observer bytes, pins and stat algorithm remain unchanged; this fixture does not establish current observer or native qualification.

Fresh actual execution passed **174 ordinary controls, 0 failures/errors**, from 196 registered IDs. The 22 Linux UID0 cases were skipped as one class on macOS and receive zero local credit. Docstrings cover **550/550 functions, 210/210 new or changed functions, 38/38 classes and 10/10 modules** across eight physical scripts and two actual embedded programs. Measured lines are **4,643/6,713**; the protected runner remains **0/102**. Complete zeros, declaration comparisons, actual call identities and 114 uncalled functions are retained.

Rust/Cargo inputs remain byte-identical to the fresh LLVM run retained by the prior receipt: 3,407 passed, 0 failed, 3 ignored; lines 89,459/97,614. The earlier enabled commit hook passed 3,431/0/3. The correction's normal hook and hosted outcomes are separate pending delivery evidence. Earlier receipts remain unchanged.

Real hosted physical/stdlib/IP-denial/egress, exact-source CI, audit, owner/platform/product acceptance and the full documentation review at the end of the entire roadmap remain open.
