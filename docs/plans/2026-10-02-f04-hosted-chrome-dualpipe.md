# Hosted Chrome stdout and stderr capture correction

The second installed-Chrome job in [run 37068129035](https://github.com/policy-forge/forge/actions/runs/37068129035)
tested requested head `901542977285207f5fbba9675c33cedc3b53b753`, base
`058bb47b81316c4f6aecc8e855111df4709a34d2`, merge
`9353f81e198eefa57b14a67a39e98f809d4a279b`. The qualified artifact reports
`failed` / `browser-tool-capture-failed` with the exact closed diagnostic
`browser-tool-capture`, `chrome-version`, `chrome-product`, exit `0`.
Producer was `not-run`, browser tools were null, and zero campaigns started.
All 26 input pins match immutable requested-head Git bytes; ordered parents
and equal merge/head trees were verified. Its sibling Ubuntu API result is a
separate single-platform observation. The retained snapshot does not establish
the remaining siblings or complete CI.

The version helper merged stderr into stdout before applying its strict product
parser. Valid private diagnostics could contaminate valid stdout. Chromium's
[Linux channel fallback source](https://chromium.googlesource.com/chromium/src/+/main/chrome/app/chrome_main_linux.cc)
can log a warning when the direct executable uses an adjacent channel file.
That is a source-supported hypothesis for the failed run; raw output was not
retained, so its actual cause and the native effect of this fix remain unproved.
The earlier generic failure and its diagnostic successor stay historical.

## Capture contract

Only the production `command` body changes. Both pipes are owned, nonblocking
and independently drained. Each must reach EOF, and the actual direct child
must finish. Stdout alone is returned privately for unchanged strict parsing;
stderr is discarded but counts toward the same inclusive **262,144-byte
aggregate limit**. Stream activity never extends the original absolute drain
deadline. Existing natural-settle and failure-cleanup phases retain their
separate bounded budgets.

Capture or cleanup failure clears stdout. Direct-child status is retained from
Popen; EOF cannot manufacture a zero exit. Tree-close failure cannot skip either
pipe-close attempt, and failure of either close prevents pass. There is no raw
output added to public receipts. Wrapper `/2`, producer `/1`, shared API `/2`,
canonical named Chrome ELF identity, strict version parser, approved tool graph,
four campaigns and complete no-force/source/tool/pass gates remain unchanged.

## Applied-source evidence before push

[The versioned audit](2026-10-02-f04-hosted-chrome-control-audit-v3.json) retains
exact source bytes, the raw named test transcript, declarations, actual call
events and every physical compiler zero. The applied cohort on macOS/Python
3.14.8 passed **47 controls, no failures/errors/skips**, source unchanged.
Eight added controls cover diagnostic contamination, stderr-only spoofing,
aggregate inclusive limits, independent EOFs, real completion, a fixed deadline,
both-pipe cleanup and redacted stderr read failure. Of the 39 preceding direct
test bodies, 38 remain AST-identical; one disclosed mock adapter now provides
both EOFs while preserving all three original assertion calls.

| Four-file cohort | Documentation | Actual primary-thread observation |
| --- | ---: | ---: |
| Named functions | 112/112 | 109/112 called |
| Changed/new named functions | 16/16 | 16/16 called |
| Classes/modules | 3/3; 4/4 | imports preceded tracing |
| Compiler-mapped physical lines | separate metric | 766/1038; 272 zero |

The author reports 766/1034 with 268 zeros because its trace helper filters four
module/class docstring entries. Root retains them as unobserved compiler entries;
the common observed-line sets match. Both receipts and their exact reconciliation
are preserved. `OwnedTree.settle`, `file_limits` and `campaign` remain uncalled.
This is primary-thread mocked coverage; full bodies, branches/MCDC, background
threads, subprocesses, Linux/native/browser and whole-repository coverage are
unmeasured.

Root replayed the author packet's 35 pins and the independent source reviews'
59 and 83 pins before applying only the two Python files. Neither reviewer ran
the candidate. Existing installed-package probe and 105 fake-DOM observations
remain separate unchanged-source historical evidence. They are not native
browser results for this change.

## Delivery and remaining gates

This checked-in audit is a precommit snapshot; the enabled mandatory hook and
fresh immutable hosted outcome must be recorded separately. Draft PR #185 and
Veans #31 remain In Review under open F04 #6. Existing v1/v2 audits, diagnostic
guide, failed hosted packets, shared source/workflow and approved packages are
preserved. No GitHub merge or tracker closure is performed.

Actual Linux installed-Chrome campaigns, normative Windows/macOS browser slots,
IPv4/IPv6 OS denial and attempted-egress observation, full parity, accepted
dependency audits, accessibility/AT/human/security/pilot and release provenance
remain open. Linux adds prerequisite coverage and does not fill normative
browser slots. The full integrated docs review/update remains required at the
end of all roadmap work. No publication or participant contact is inferred.
