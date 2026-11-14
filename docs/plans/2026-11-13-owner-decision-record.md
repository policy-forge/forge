# Owner decision record — 2026-11-13

Status: **Recorded owner dispositions.** Captured interactively from the
project owner (Brian Luby) on 2026-11-13 via three decision batches. This
record asserts the owner's answers; it does not authenticate identity. It
settles the decision-register items in
[the 2026-11-12 owner decision batch](2026-11-12-owner-decision-batch.md)
and [the decision packets](2026-11-12-owner-decision-packets.md) (PR #270).
Implementation, human gates, and publication authorization remain separate.

## Dispositions

| Item | Decision | Effect |
| --- | --- | --- |
| D1 — four supply-chain exceptions | **Approved** (2026-11-13, recorded on `fix/p0-supply-chain-security` `e38211c`) | zip 9.1.2 / pulldown-cmark 0.13.6 / rustix 1.1.6 / quick-xml 0.42 exceptions in force |
| D1b — 47 stale exception renewals | **Approved** (2026-11-13, `e22a24d` review-date refresh) | expiry extended; supply-chain gate no longer time-critical |
| D2 — 13 retained-evidence Dependabot alerts | **Dismiss** as tolerable risk | Executed 2026-11-13: all 13 dismissed (`tolerable_risk`); shipping graph clean |
| D3 — quick-xml migration path | **Settled** — #264 closed 2026-11-13 in favor of #268 (merged) | migration delivered |
| D3b — #265 edited immutable evidence | **Keep merged; add dated F03 note** | #265 stays merged; F03 audit record must note the authorized receipt alteration (zip 8.6.0→9.0.0/9.0.5 in pr162/163/164 locks); SHA-256 index discrepancy is authorized, not tampering |
| D4 / D067 — F11 MCP discovery + threat model | **Ratify** implemented confined-roots model + re-pin #275 (hash `a15bac36…`); two-client pair = MCP Inspector (modern) + Claude Code (v2), MCP 2026-07-28 | unblocks F11 (#55/#56) conformance harness |
| D4 / F16 — confinement mechanism | **Unix-first**: macOS sandbox-exec/Seatbelt + Linux Landlock+seccomp-BPF now (no new deps); Windows stays refused until a separate `windows-sys` PRD 069 approval | unblocks F16 (#72) Unix implementation |
| D4 / F12 M-10 — proposed-edit contract | **Approve** envelope as specified (`target_subject_sha256` binding, AC-5 wrong-hash rejection, per-response classification, new closed family version) | unblocks F13 M-10 |
| D4 / F12 M-13/M-15 | **M-13 opt-in** (conservative default retained); **M-15 keep "qualified Windows" refused** until P7 hardware qualification | #71 stays In Review |
| D4 / F10 Q1–Q7 | **Confirm** Q1–Q6 binding (D064/add-only); Q7 AR validation in scope; close #68/#69/#70 by evidence, no new code | no PRD 054 amendment required |
| D4 / D068 — F12/F13 split + quorum | **Confirm** recorded split + quorum_v3 by evidence; relabel plan "D069"→"D068 scope split" | reconcile, no re-decision |
| D4 / S-2 — signed envelopes | **Won't for 2.0.0** — record as deferred Should | removed from 2.0.0 scope |
| D5 — F15 Phase 2 (live GitHub HTTPS) | **Defer to post-2.0.0** | Phase 1 (#276) ships; Phase 2 out of 2.0.0 scope |
| D6 — F06 authentic evaluation | **Delegate**: draft proposed G0 values for owner review first, then approve; recruit participants in parallel | G0 draft requested (see below) |
| D7 — MSRV verification | **Add an MSRV CI job** enforcing 1.88 | unblocks MSRV gate |
| D7 — F22 support policy | **Approve as drafted** (latest minor only; 1.x security-only 90 days; triage 7d/48h; no release with expired/unreviewed runtime exception) | support policy in force on approval record |
| D7b — private disclosure channel | **Defer to post-2.0.0** — tracked Should | out of 2.0.0 scope |

## Unblocked implementation (next batch)

- **F16 (#72)** Unix confinement (macOS + Linux), no new dependencies.
- **F12/F13 (#21/#59)** M-10 proposed edits + remaining M-3 adapters per the
  confirmed D068 split; S-1 interactive client; S-4 supersession.
- **F11 (#55/#56)** two-client MCP conformance harness (Inspector + Claude
  Code) against the ratified model.
- **MSRV CI job** (1.88 enforcement).
- **F03 note** documenting the authorized #265 receipt alteration.

## Still gated (not unblocked by these decisions)

- **F17/F18 (#74/#75)** — need F16 + F06 authentic evaluation gates.
- **F06 (#8)** — G0 dispositions (draft requested) + real participants.
- **P7 human qualification** — macOS/Windows hardware, WCAG/AT, real users,
  independent-tool interop, security/privacy review, human audit sign-off.
- **F22 (#76) publication** — owner authorization remains the only exit.

## Deferred to post-2.0.0 (recorded)

F15 Phase 2 (live GitHub adapter), S-2 signed envelopes, private disclosure
channel, Windows confinement (`windows-sys` approval).
