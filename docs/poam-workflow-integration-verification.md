# POA&M workflow integration verification

This report binds the integrated nonterminal workflow and actual nested AR receipt
producer/consumer correction to final source bytes. Full F08 and the goal-wide final
documentation review remain open. The [machine-readable receipt](plans/2026-10-03-f08-item-workflow-integration-verification.json)
retains source-copy identities, exact denominators, historical failures, aliases and gaps.

## Documentation and measured coverage

| Lexical dimension | New or changed | Whole 12 owned Rust files |
|---|---:|---:|
| Functions with adjacent Rustdoc | 124/124 | 243/407 |
| Types with direct Rustdoc | 32/32 | 79/115 |
| Members with direct Rustdoc | 140/140 | 602/825 |
| Modules with Rustdoc, separate optional dimension | 3/5 | See receipt |

The selected functions comprise 69 production, 32 cfg-test and 23 integration
declarations. They contain 42 literal test declarations: 40 newly added controls
and two explicitly adapted pre-existing controls. The two selected module gaps are
private cfg-test modules. Unchanged omissions are enumerated separately.

Actual LLVM recorded 123 positive selected function headers, one zero and no
unmapped header. The zero is the bounded in-memory writer's `flush`. All 202 exact
header aliases are retained, including 23 zero aliases. Whole-file association
retains 404 positive, two zero and one unmapped declaration among 407, with 598
aliases. The unmapped declaration is an unsupported-platform publisher test.
Nearby generated functions or closure entries never provide header credit.

The full LLVM run passed **3,136 tests**, with zero failures and three ignored in
70 summaries. Whole-workspace lines were **76,452/83,888 (91.14%)** and functions
**6,869/8,100 (84.80%)**. Those totals include tests and generated code. Positive
entry counts do not prove whole-body, branch, every-instantiation or platform
coverage. Branch and MC/DC denominators are unavailable, not zero coverage.

Final formatting and strict all-target/all-feature Clippy passed. Every completed
job retains before/after source and raw-log pins. The later enabled mandatory
commit hook is separate evidence. Historical failures and green controls are not
summed into the final run's unique cases.

## Corrections established by actual controls

The original producer emitted importer-relative context hrefs. Its real nested
bundle control failed; the unchanged consumer then still refused corrected
producer output. The integrated pair passes actual source-only checks and refuses
wrong-target decoys even when their bytes match the declared companion.

The original workflow had 16 passing controls and one actual native-schema
failure. Pretty inner declaration JSON placed literal line breaks in native
string properties. Compact bounded declaration serialization now passes all 18
workflow controls, preserving exact authored newline/tab content. Outer native
JSON and schedules retain pretty formatting and one final newline.

The first full CLI suite had 41 passes and one scope-control failure. A diagnostic
identified clap admitting source-only with an as-of option; dispatch already
refused the combination. Explicit conflicts now cover every workflow-only option.
The full 42-case foundation/CLI suite passes, including real build/check, schedule
action exits, destination preflight, no-replacement publication and source drift.

The independent semantic doc review corrected decoded event-prefix wording,
historical nested-href limitations and the optional zero-day interval. Only three
guides changed after the measured source capture; all 12 Rust files still match
that run byte-for-byte. Both tested and current source pins remain explicit.

## Admission and remaining work

Completion and risk-acceptance history is refused at every public workflow entry;
there is no bypass flag. Cancellation changes a plan assertion and closes no
source finding. Exact captures establish integrity, not assessor/owner authority,
remediation effectiveness or evidence sufficiency. Native files retain authored
prose and party names; minimized schedules omit them.

Native artifacts and optional reports are new portable filenames in the manifest
directory. Both destinations are qualified before either write, and declarations
plus five source generations are rechecked before output. Publications are
separate: a later report failure may leave the newly created artifact. Linux/macOS
publisher support is preserved; no Windows publication result is inferred.

Full baseline impacts/removal/reopening, the D064 closure disposition, fresh
evidence links, portfolio/HTML/connector outputs, independent consumer qualification,
representative plans, human pilots and release acceptance remain open. No audit,
human approval or complete goal-wide documentation review is inferred.
