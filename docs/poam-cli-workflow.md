# Authored POA&M commands

The integrated commands check explicitly authored nonterminal work and generate
a native schema-validated POA&M. The original foundation init and source-only
checks retain their scope. Full F08, D064 closure disposition, independent consumer
qualification and human acceptance remain open; these commands authenticate no
actor and infer no remediation effectiveness.

## Explicit modes and dates

`forge poam check --manifest poam.json --source-only` keeps the existing minimized
source-integrity inventory. It does not admit selected remediation items or workflow
fields, and cannot be combined with workflow options.

An authored workflow is explicitly selected:

```sh
forge poam check --manifest poam.json --workflow --as-of 2026-02-06 \
  --due-soon-days 7 --format json
forge poam build --manifest poam.json --as-of 2026-02-06 \
  --due-soon-days 7 --output native-poam.json --report schedule.json
```

The `--as-of` date is mandatory, canonical `YYYY-MM-DD` and calendar-valid. It is
never derived from the clock. The inclusive `--due-soon-days` interval is 0–365;
omitting it selects zero, which still includes open records due on the as-of date.
JSON is the build report default; text is the existing check default. Text escapes
all authored keys into printable ASCII. Native output always remains OSCAL JSON.

An optional `--baseline prior.json` is a normalized confined descendant of the
manifest directory, rather than an unrelated current-directory input. It supplies
an exact author-declaration history prefix comparison, not authenticated history,
a full impact report or evidence freshness. Existing decoded attributed events
must remain an exact prefix; new nonterminal events can be appended. JSON
whitespace and escape spelling are not compared across revisions. Each originally
captured raw manifest/baseline generation is separately rechecked during output.

Completion and risk-acceptance history are refused by the public workflow parser,
preparer and CLI, even with adequate proposed review metadata. There is no bypass
flag. Explicit cancellation changes only the authoring plan assertion and never
closes an Assessment Results finding or risk.

## Actual file and link base

The captured manifest directory is the native relative-link base. `--output` is a
required new single `.json` filename there. `--report`, when supplied, is another
new single filename there, with `.json` for JSON or `.txt` for text. Absolute paths,
nested destinations, `./` aliases, wrong suffixes, existing files and ASCII
case-folded artifact/report or source/manifest/baseline aliases are refused. Both
filenames consume the exact existing publisher validator before either write: at
most 128 bytes per component and ASCII letters/digits, dash, underscore or dot;
reserved device names and other unsafe portable spellings are refused. An absolute
manifest path invoked from another directory still publishes at the manifest directory. No source href
is invented, relocated or silently rewritten.

Without an explicit report path, the complete minimized schedule goes to stdout.
A workflow check never creates a native artifact. A build creates no report file
unless an explicit report path was supplied. Native artifacts retain authored
prose, party names and workflow declarations and may be sensitive; they are not
content-minimized report substitutes.

Current manifest and optional baseline original bytes/file identities are retained
through existing confined bounded readers. The producer separately captures all
five actual native source generations and exact selection tuples. Both declaration
generations and all sources are rechecked before the first output, and again before
a subsequent report. No-replacement publication uses the existing authoring
publisher; a filename preflight does not itself grant write authority.

The two outputs are separate publications. If artifact publication succeeds and a
later report recheck, publication or stdout write fails, the artifact may remain.
The command reports an invalid exit 2 and promises neither rollback nor atomic
visibility across the pair. Existing files and sources are never intentionally
replaced. The current publisher supports Linux/macOS; other platforms fail closed
for file publication until a separate qualified implementation exists. Stdout
workflow checks use existing confinement support and remain independently testable.

## Outcome and scope

Exit 0 means a valid admitted workflow with no schedule/review action. Exit 1 means
a valid complete schedule contains overdue, due-soon or blocked records; it emits
no generic error message. Exit 2 means invalid usage, authoring/source/history,
placement, revalidation, schema or output failure. No partial report supplies
valid action credit. Each report retains complete source/item/milestone/row counts
and distinguishes current as-of assertions from missing historical assertions.

The [adapter proposal](plans/2026-10-03-f08-workflow-cli-adapter-proposal.md) records
precise source guards, required nested-href and compact native-declaration repairs,
and historical proposal guards. The integration applies the nested producer/consumer
path correction, compact bounded native declarations and explicit clap conflicts
for every workflow-only option in source-only mode. Full baseline impacts/reopening, closure disposition,
fresh evidence, portfolio/HTML/connector packages, independent-tool interpretation,
representative plans and owner/human/platform acceptance remain open.
