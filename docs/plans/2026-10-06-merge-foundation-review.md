# Merge foundation review — October 6, 2026

This checkpoint preserves the original heads of PRs #165, #166, #168, #169 and #170 on main `aef0ab24f77559593b6d0b4fe5027f8a5eaa857e`. It introduces no dependency versions. The owner authorized execution of the checkpointed merge plan and separately approved the incremental dependency policy and the 20 exact exceptions in [the decision record](2026-10-06-merge-dependency-decisions.md).

## Source review and corrections

Codex reviewed the dependency consumer, graph preparation, filesystem helpers, API verification runner and their fixtures for identity/proof boundaries, bounded input and output, cleanup, scope and denominators. The consumer does not fetch or run Cargo. Qualification independently regenerates complete graph bytes; neither hashes alone nor native exemptions create audit credit. Full-rooted imported proof remains bound to the approved source pins. Runtime readiness and strict/full-store acceptance remain incomplete under the existing legacy exemptions.

The dependency reader now registers cleanup immediately with ExitStack, including held directory descriptors. Regression cases exercise read failure and continued cleanup after a close error. Do not use fdopen on directory descriptors. Removed an unused test import and explained the successful-publication cleanup case.

The API verifier retains its closed three-suite receipt and actual declared/observed operation counts. Removed the duplicate regex character and unused imports, explicitly imported unittest.mock, and explained intentional shutdown races. Windows descendant-process cleanup and broader platform/browser/runtime acceptance remain open as recorded in the original evidence.

CodeRabbit reviewed the original #169 head with no actionable comments. Its requested reviews of #166 and #168 were rate limited; they are not approvals. The coordinator source review and fixture execution are the current additional evidence. No human source-audit signoff is asserted.

The roadmap ledger verifies all 327 unique Must/Should requirements and 47 named gates against exact PRD wording, hashes and evidence references. The original #170 retained index had one stale summary entry: 3,006 bytes and SHA-256 `5cfb4b54d5496a96169200c07b6171cfcdd24aa914cae4605fe1112e3686fe06`. Its committed summary is 3,784 bytes with SHA-256 `bd525a4e45d338363c6c358348f10390c577b73f9cf9782f1c30289816968305`. The index now binds those actual bytes and retains the old entry in an explicit reconciliation field. All 152 indexed files verify; historical receipts were not edited.

## Verification and promotion boundary

The composed lint/dependency foundation passed 47 dependency fixtures, strict all-target/all-feature Clippy and 2,692 all-feature Rust tests with zero failures and three ignored tests. The API tooling passed all 24 fixtures. Its actual release binary passed all 12 maintained-client checks. Normal commit hooks remain enabled. The four release-target dependency graph matches 312 locked packages and canonical SHA-256 `4c3e80a2fa0419498063a95d208c06fd42a75c364fe1a4d17a1bb5451fdad48f`.

Native cargo-vet passes with the approved 20 exceptions. The stronger incremental inventory gate still requires a separate disposition for chacha20 0.10.2: its existing delta cannot inherit a full reviewed root from the exempt predecessor. The separate addendum is pending and is not covered by the original approval. No foundation promotion is accepted while this gate fails. Hosted checks must also verify the published final composition. Linux trust qualification is a separate repair candidate, not completion inferred from these macOS checks.

Code integration does not close roadmap, participant, accessibility, interoperability, human product or release gates. Historical receipts remain evidence of their original attempts, not claims about this candidate.
