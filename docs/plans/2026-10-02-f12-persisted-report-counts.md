# F12 prerequisite: persisted applicability report counts

This slice makes the existing persisted applicability parser reject an overflowing sum of its six classification counts with the existing validation error. It preserves the report envelope, representable filtered denominators, unfiltered category equality, and the separate recomputation required for workspace currentness.

The immutable implementation baseline is `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. The delivery source is `src/applicability/mod.rs`, SHA-256 `5d1c246314497af08c33882a3cda7ce27a390181a066c77b0cfd6fda3a1cff72`, 75,602 bytes. Recording-time branch: `codex/f12-persisted-report-counts`; tracker scope: parent #21, child #22. These identities do not establish tracker closure or approval of the portable review workflow.

## Existing contract and change

`parse_stored_report` at lines 1321–1426 already validates the closed `forge.applicability-report/1` envelope, hashes, visible controls and classification counts. Lines 1360–1370 now accumulate the six existing counters with `usize::checked_add`; overflow returns `ForgeError::Validation("Invalid persisted applicability report")`. Line 1371 retains the exact comparison with the declared total. The subsequent hash, duplicate-control, unfiltered equality and filtered subset checks are unchanged.

Workspace registration reaches the parser through `services::validate_bytes` at line 86. Snapshot metadata, report rendering and input staleness also consume it at lines 181, 563 and 635. The change adds no count cap, queue envelope, public command, schema version, dependency, identity rule or quorum policy.

## Executable checks and evidence

- All 30 ordered distinct `usize::MAX` plus one pairs reach the direct parser. The final implementation rejects each with the exact existing Validation variant/message without a caught panic.
- Zero and each of the six representable MAX-plus-zero filtered denominators preserve the entire parsed JSON report. These are synthetic historical-parser boundary fixtures, not authentic inventories.
- The actual `ApplicabilityReport` registration consumer rejects overflow without panic, accepts an ordinary report and rejects a representable total mismatch.
- Both final default instrumented execution and the explicit Forge test-profile `overflow-checks=false` execution pass. The latter is an overflow-mode regression check, not a release build or other-platform result.
- The final selected LLVM run reports 1,868 passes; the standalone default-feature suite reports 2,695 passes, zero failures and three existing WI-31 ignores. Final standalone strict all-target Clippy exits zero. Retained failed lint/format attempts and exact source successors are recorded in the verification packet.

Coverage uses every unique LCOV DA entry before `#[cfg(test)]` line 1428: 1,024/1,160 production lines, including all 136 zeros. The parser is a separate 79/82 metric; the added production guard is 12/12 instrumented lines. Native whole-file 1,311/1,497 includes test mappings and uses a different aggregation. Branch and MC/DC evidence is unavailable. Selected adjacent Rustdoc is 6/6; whole production is 6/50 with 44 legacy undocumented declarations. See [the focused verification](../persisted-applicability-verification.md) and its JSON for exact source/raw pins, line counts and exclusions.

## Remaining gates

The mandatory commit hook must run after these artifacts are applied, without bypass. Its authentic outcome will be retained outside the commit and reported with the PR; the pre-commit record cannot claim that future run. Exact-head hosted CI and review remain separate gates.

This prerequisite does not complete PRD 068/F12 portable review queues or satisfy its M-2/M-9 authority and conflict/quorum requirements. Product must select the first workflow; Compliance must approve dispositions, reviewer/role counting, substitution, separation, abstention and quorum; Engineering must approve immutable snapshots versus bound references and queue/response/merge/promotion contracts; Security/privacy must review package contents and asserted identity wording. Signed envelopes remain conditional on a separate identity/key/trust/revocation/cryptography design and any dependency approval.

All 17 PRD 068 Must requirements, S-1/S-3/S-4, the conditional S-2, AC-1–AC-6, three representative team workflows, pilot metrics, owner acceptance, later adapters/F13 and F21/F22 qualification remain open. F01 records are proposed planning evidence, not owner dispositions. The full integrated documentation review after the scoped roadmap packages remains open. No human acceptance is asserted by this plan or its synthetic evidence.
