# Synthetic guided authoring project

Every framework control and clause in this example is original synthetic fixture
content. The named reviewer and organization context are fictional. The example
demonstrates input contracts; it is not a framework pack for operational use.

From this directory with a built `forge` executable:

```sh
forge author plan --manifest project.json --format text
forge author build --manifest project.json --output-dir draft-run-1 --format json
forge author build --manifest project.json --output-dir draft-run-2 --format json
diff -r draft-run-1 draft-run-2
```

Both applicable gaps are assigned. The first section contains an explicitly
pinned clause with an answer-record pin. The independent second section contains
a visible missing-clause marker. Planning exits 0 because required context is
available and no gap is unresolved; this does not mean the policy text is finished.

Linux and macOS support atomic directory publication. Other platforms support
read-only planning and refuse directory publication before creating output.
Each destination must be new. Delete example output only after reviewing it, or
choose a fresh directory name for the next build.

The files pin one another by exact SHA-256. If editing a question, pack, answer,
baseline, or clause, update every affected explicit pin as part of a new human
review; FORGE never refreshes pins automatically. Question and answer hashes use
the canonical domain-separated algorithm documented in
[the implementation plan](../../docs/authoring-phase1-plan.md).

See [usage and trust boundaries](../../docs/authoring.md) for the complete
contract. All PRD-061 human readiness, pilot, and release gates remain pending;
M-13 impact analysis and PRD-059 composition remain Phase 2.
