# Source export and restore confirmation copy

Source export now leads with what is saved: the complete project index and exact
registered source bytes, including sensitive content and metadata. Restore leads
with replacing the index and restoring exact bytes to every listed target. It
explains possible folder creation and overwrites, retention of unlisted files,
and the confirmation boundary. Other local readers may see old and new files
during restore/recovery; confirming a write does not approve its data.

Complete target/index/input/directory/hash details remain visible. The complete
server summary is retained as text under “Server details”; its two spacing defects
are corrected for display. Staged export wording uses the original route-derived
family, including after the editable profile changes. Inline exports retain their
own summary. Restore copy is shared by inline and staged restore. Original review
acknowledgements, cancel/dismiss/confirm/download handlers, DTOs and routes remain.

Inline bundle capacity is 1,048,429 bytes. Staged capacity is 10 MiB with 32 KiB
parts; available session/preparation capacity may be smaller. Transfers remain
bounded to 100 distinct planned file paths, including the index and any export
destination. A lost reply still requires explicit same-upload retry and complete
review before the single restore confirmation.

## Verification before drafting

The [source-bound report](plans/2026-10-03-f04-source-confirmation-copy-verification.json)
was prepared on F07 parent `338082f78e7720dbb3028d27c177733b3e7a5bf9`
before the draft PR. The guarded two-path application preserved 1,581 other
tracked/configuration files. Production asset SHA256 is
`fa157dbfa0dc837c4f7a5936206f9300f8144c2736a10929cf1775ba7f05fc1b`;
the applied harness is `78b15722e1fe3acbef92dfdad1f28a50242575778eb1583fb5c89dd6b12b824b`.

The old asset reproduced four failures: 271 tests, 267 passed/four failed. The
applied asset passed all 271 with zero failures, skips or cancellations. Both
files passed Node syntax checks. New cases exercise original-route review after
profile edits, preservation of server facts and hashes, unchanged restore
acknowledgement/dismissal, and both supported source-panel versions. The original
harness is an exact prefix, with no baseline control adaptations. Red executed
the new TEMP harness; its managed old-harness pin is not substituted for that
executed path. Final passing and initial failing executions remain separate.

| Selected JSDoc sites | Documented | Total |
|---|---:|---:|
| Changed production declarations | 3 | 3 |
| New named control helper | 1 | 1 |
| Anonymous registration sites (five expanded cases) | 4 | 4 |

All eight sites map to positive V8 entries, with eight aliases, 80 complete range
records and **eight zero nested ranges retained**. Entry counts are preview 83,
restore details 15, source panel 73, helper two, callbacks one/one/one/two. Root
independently compared every selected red/green alias to the raw script/function
index, original source URL, UTF16 offsets and every range. A positive entry does
not establish whole-body, branch, line or byte coverage.

The bounded whole-file explicit-declaration classifier finds 148 documented out
of 163 production sites (15 omissions) and 78/78 harness sites. Direct-arrow
diagnostics are separate: 21/23 documented production bindings and 12/12 harness
bindings; two additional production pattern candidates are unmapped. These are
incomplete lexical categories, not an AST completeness claim or additive total.
Whole passing V8 retains all 345 asset function records (307 positive/38 zero)
and 1,935 ranges (297 zero), and all 803 harness records (794 positive/nine zero)
and 1,269 ranges (32 zero), including anonymous and synthetic bodies.

## Actual browser screenshots

A fresh Rust 1.99 native build embedded the final UI. The installed-Chrome
export/restore/fresh read-only lookup campaigns passed against that build with
the matching final source asset and no injected HTTP/application replies. Eight
PNG captures record 1440- and 320-pixel views. The synthetic 1,229,739-byte bundle
used 38 ordered parts; exact download bytes, all restored targets, retained
removed-registration files and one restore confirmation were verified. Fresh
read-only lookup performed zero project writes. All three Node/native children
exited normally, and browser/context closure was observed. No descendant-tree
emptiness or native per-user journal retirement is inferred.

Screenshots are retained at
`/private/tmp/forge-f04-confirmation-copy-browser-v1/{export,restore,lookup}/browser/`.
The report pins their exact bytes and geometry. Root visually inspected the
desktop export and mobile restore previews. This demonstrates the local rendered
implementation on synthetic owned fixtures; it is not hosted-platform, OS
network-denial, accessibility/AT, participant or human acceptance. Normal-browser
metadata is separate from Node fake-DOM V8; no native profiles were aggregated.

No dependency or protocol is added. Historical hosted receipts and source-specific
Rust coverage remain unchanged. Broader F04/F05, platform/audit/human acceptance,
the final goal-wide integrated documentation review and verified 2.0.0 release
candidate remain required.
