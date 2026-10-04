# POA&M portfolio integration verification

This records the supplied-native portfolio and static HTML slice on the fixed
baseline parent `4df328577a1ab77217bb4b799e4ad14944720804`. The
[machine-readable receipt](plans/2026-10-03-f08-portfolio-integration-verification.json)
binds the final six Rust files and three command-guide files, preserves every
current lexical row and exact native alias, and separates historical cohorts.
Full F08 and the goal-wide final documentation review remain open.

| Lexical dimension | Selected new/changed | Whole six owned Rust files |
|---|---:|---:|
| Functions with direct Rustdoc | 83/83 | 165/217 |
| Types with direct Rustdoc | 19/19 | 62/62 |
| Members with direct Rustdoc | 128/128 | 503/586 |
| Modules with documentation, separate dimension | 3/6 | 16/28 |

Selected functions comprise 51 production, 14 cfg-test and 18 integration
declarations, with 24 new literal test attributes. Three new private cfg modules
have no adjacent outer documentation. All 52 whole-file function omissions are
inherited CLI declarations; member and module omissions remain explicit. The
prior baseline publication test is unchanged against the fixed parent and earns
no new portfolio docstring credit. Only `cli::execute` is an existing selected
function; the other 82 are introduced definitions and helpers.

Actual full managed LLVM passed **3,183 tests**, with zero failures and three
ignored in 70 summaries. Whole-workspace lines were **78,644/86,159 (91.28%)**;
functions were **7,058/8,325 (84.78%)**. Instantiations were 9,695/14,494 and
regions 137,606/151,808. These aggregates include production, tests and generated
code. Branch and MC/DC denominators are unavailable. The raw export retains
18,363 function records, including 6,000 count-zero records; those are separate
from aggregate functions and lexical declarations.

All 83 selected headers have positive exact entry observations, with zero
unmapped or zero-count headers. All 134 aliases remain, including 16 zero-count
instances. Across the six files, 217 headers are positive and 294 aliases include
29 zero-count instances. Association requires the identical actual filename and
literal header line/UTF8 column at each record's first regular code region.
Generated functions or nearby closures never fill an unmapped declaration.
Positive entries do not prove complete bodies, branches, every instantiation or
every platform. The fresh isolated target used Rust 1.99 on aarch64 macOS.

All final Rust and command-guide bytes match the full measured run. Integrated
formatting and strict all-target/all-feature Clippy passed without source changes.
The public command help was checked using the exact built binary; its additional
profile output was kept outside the frozen measured target. The mandatory commit
hook and later hosted checks are separate evidence recorded when executed.

## Preserved failures and controls

The original unit fixture first passed six controls and failed one because its
missing milestone prevented the intended closure-rule refusal. A cfg-only coherent
fixture successor passed all seven. Five genuine native-pair controls then passed.
The first tests-only public CLI run failed all seven new cases because the command
was absent. These are distinct source cohorts and never added to the full run.

After the CLI was supplied, six cases passed and one failed because the test
expected two overdue rows when only its milestone was overdue. Correcting that
expectation yielded 65 foundation cases. An earlier library cohort passed 82 cases.
Formatting and strict Clippy found separately retained formatting, doc spelling,
two checked allocation-bound conversions, scoped line-count lints and one cfg
assignment semicolon. Precise successors retain every failed receipt and preserve
unrelated bodies. The final full run executes the corrected integrated source.

Controls use actual five-source captures and freshly generated native artifacts.
They check complete counts, null as-of states and cancelled rows, final-state versus as-of
semantics, explicit multi-root pairs with different native basenames, full decoded
native equality, original-byte hashes, same-byte identity replacement, every held
original, stale/extra native data, duplicate plans, complete escaping and output
preflight. Successful JSON/HTML file publication is qualified to Linux/macOS;
portable stdout and refusal controls remain available on other platforms.

The actual CLI also generated a two-plan synthetic preview. Installed Chrome
loaded the resulting static HTML at widths 1440 and 320 with no page errors or
external requests. The observed document widths matched both viewports; there
were two plan rows, no scripts and no active links. These observations and the
screenshots do not establish representative-plan, accessibility, privacy, human
or hosted-platform acceptance.

## Remaining gates

The [command guide](poam-portfolio.md) describes explicit pairing, whole-value
native qualification, bounded complete reports, privacy and publication semantics.
IDs and hashes remain potentially sensitive. JSON and HTML are separate outputs;
a first output can remain when later publication fails.

D064 closure/supersession disposition, outbound adapters, fresh PRD060 evidence
references, independent OSCAL consumer qualification, three representative plans
and real remediation pilots remain open. Current platform/API/offline-runtime and
dependency-audit gates also require their own evidence. Existing native artifact
admission remains authoritative for supported declarations. No actor authority,
evidence sufficiency, audit approval, merge or release readiness is inferred here.
