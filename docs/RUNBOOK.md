# FORGE Tester Runbook

A hands-on manual for testing FORGE end to end: from the simplest
Markdown-to-OSCAL conversion to drafting policies from control standards with
local-AI suggestions. Every command below was run against the release binary
from the repository root; expected outputs are shown so you can confirm each
step worked.

Conventions:

- `forge` means `./target/release/forge` (build it with
  `cargo build --release --locked`).
- Work in a scratch directory for Chapters 3–4 so generated files stay out of
  the repo: `cp -r examples/authoring /tmp/forge-run && cd /tmp/forge-run`.
- Chapters 1–2 only read `example_data/` and write to `/tmp`, so they are safe
  to run anywhere.

## 1. Convert a Markdown policy to OSCAL (the simplest path)

Convert the acceptable-use policy to an OSCAL catalog:

```bash
forge convert example_data/POL-01_Acceptable_Use_Policy.md \
  --strategy catalog --format json --output /tmp/pol01-catalog.json
```

You will see modality warnings on stderr (the parser defaulting unmarked
sentences to Normative) — that is normal. Success is exit 0 and a
pretty-printed OSCAL catalog at `/tmp/pol01-catalog.json`.

Things to try:

- `--format xml` or `--format yaml` to serialize directly in another format.
- `--strategy component` needs a source profile:
  `forge convert policy.md --strategy component --source-profile profile.json`.
- Omit `--output` to print the artifact to stdout instead of a file.
- Key exit codes: `0` success, `1` input/IO, `2` parse/structure,
  `3` validation/config (e.g. missing `--strategy`).

## 2. Validate, export, trace, diff

Validate the artifact you just built:

```bash
forge validate /tmp/pol01-catalog.json
# Valid: catalog artifact passes all validation.
```

Convert between formats (input format is auto-detected from the extension):

```bash
forge export /tmp/pol01-catalog.json --format yaml --output /tmp/pol01-catalog.yaml
forge export /tmp/pol01-catalog.json --format xml --output /tmp/pol01-catalog.xml
```

Show where each OSCAL element came from in the source policy:

```bash
forge trace /tmp/pol01-catalog.json \
  --source example_data/POL-01_Acceptable_Use_Policy.md \
  --output /tmp/pol01-trace.txt
# OSCAL Element ID | Element Type | Source Section | Source Line ...
```

Compare two artifacts (build a second one first):

```bash
forge convert example_data/POL-02_IT_Risk_Assessment_Policy.md \
  --strategy catalog --format json --output /tmp/pol02-catalog.json
forge diff /tmp/pol01-catalog.json /tmp/pol02-catalog.json
# Summary: Controls (old): 7 | Controls (new): 13, Added/Removed lists...
```

## 3. Draft policies from control standards (`forge author`)

This is the framework-guided authoring flow: declare which controls apply,
find the gaps, then plan and build traceable policy skeletons from explicit
human inputs. Nothing is generated — every word comes from a file you supply.

Set up (all fixture paths resolve relative to the manifest directory, so copy
the whole directory and work inside the copy):

```bash
cp -r examples/authoring /tmp/forge-run && cd /tmp/forge-run && ls
# project.json framework.json applicability.json gap-report.json pack.json clause.md
```

Check the plan — what work is assigned, blocked, or deferred:

```bash
forge author plan --manifest project.json --format text
# FORGE policy drafting plan ... inputs / sections / gaps ...
```

Build the policy skeletons into a **new** directory (existing directories are
refused; publication is Linux/macOS only):

```bash
forge author build --manifest project.json --output-dir draft-run-1 --format json
ls draft-run-1 draft-run-1/policies
# plan.json plan.txt provenance.json policies/sample-policy.md
```

Determinism check — a second build to another new directory must be
byte-identical:

```bash
forge author build --manifest project.json --output-dir draft-run-2 --format json
diff -r draft-run-1 draft-run-2 && echo DETERMINISTIC
```

Exit codes: `0` everything assigned and context available, `1` valid report
with unresolved gaps or blocked sections, `2` bad pin, path, or output.

## 4. Reuse prior approved prose (`forge author reuse`)

Before involving any model, check whether already-approved prose covers your
gaps. Retrieval only: verbatim, byte-exact excerpts, ranked — never rewritten.

Create a tiny corpus of one approved document. First the document:

```bash
mkdir -p prior
printf '# Access drafting\n\nc-1 Approve access requests quarterly.\n' > prior/access.md
```

Then `corpus.json`, with the document's SHA-256 pin (compute it with
`sha256sum prior/access.md` and paste the hex digest where marked):

```json
{
  "schema_version": "forge.reuse-corpus/1",
  "corpus_key": "synthetic-corpus",
  "title": "Prior policies",
  "documents": [
    {
      "key": "prior-access",
      "path": "prior/access.md",
      "title": "Prior access policy",
      "status": "approved",
      "rights_label": "Operator asserted",
      "source_label": "Tester",
      "expected_sha256": "<sha256 of prior/access.md>",
      "topic_keys": ["sample-topic"],
      "control_ids": ["sample-1"]
    }
  ]
}
```

Run it:

```bash
forge author reuse --manifest project.json --corpus corpus.json
# FORGE reuse candidates ... ranked excerpts with exact spans and source hashes
```

Only documents with `"status": "approved"` are eligible. Nothing is written
into any pack, project, or plan.

## 5. Local-AI suggestions (`forge suggest`)

The five-step quarantined pipeline: `prepare` assembles the exact model
payload and records your consent; `run` imports the model output;
`validate` admits only cited, byte-exact claims into quarantine; `review`
records your per-suggestion decisions; `promote` proposes (never applies) a
downstream patch.

**About the "local AI" step.** Process execution is currently withheld until
an OS confinement boundary exists, so `run` takes a `--recorded-response`
file instead of calling a model. In real use, that file is the output your
local model produced for the exact payload in `prepared/payload.txt` (inspect
it — it is the bytes the model saw). Everything downstream treats that text
as untrusted data either way, so testing with a hand-written recorded
response exercises the real trust boundary.

### 5.1 Prepare (payload + consent)

You need a local adapter stand-in (fingerprinted only, never executed) plus
the corpus from Chapter 4:

```bash
printf '#!/bin/sh\ncat\n' > local-adapter && chmod +x local-adapter
forge suggest prepare --manifest project.json --output-dir prepared \
  --adapter ./local-adapter --model-id test-model \
  --corpus corpus.json --include-document prior-access \
  --consent --operator-key tester
# prepared payload.txt ... units: 2 ... consent token: consent.json
```

This writes `prepared/request.json` (the allowlist), `prepared/payload.txt`
(the exact bytes), and `prepared/consent.json` (bound to the payload and
adapter digests). `--consent` requires `--operator-key`.

### 5.2 Run (import the recorded response)

Look at `prepared/payload.txt`, then write the model's answer as
`recorded.json`. Citations must name allowlisted units; a `quote`, when
present, must equal the payload span byte for byte. Find the span bytes with:

```bash
python3 -c "
import json
r = json.load(open('prepared/request.json'))
for u in r['context']['units']: print(u['unit_id'], u['kind'], u['payload'])"
# unit-0001 plan-section ... unit-0002 source-span {'start': 580, 'end': 638}
```

The drafting section to address is in `request.json` under
`task.drafting_sections` (here `sample-policy` / `independent-topic`):

```json
{
  "schema_version": "forge.suggest-response/1",
  "task": {
    "kind": "policy-drafting",
    "schema_version": "forge.suggest-task-drafting/1",
    "draft_clauses": [
      {
        "policy_key": "sample-policy",
        "topic_key": "independent-topic",
        "draft_text": "Access requests are approved quarterly.",
        "citations": [
          {
            "unit_id": "unit-0002",
            "quote": "# Access drafting\n\nc-1 Approve access requests quarterly.\n"
          }
        ],
        "assumptions": [],
        "unresolved_questions": ["Who approves?"]
      }
    ]
  }
}
```

Import it:

```bash
forge suggest run --request prepared/request.json \
  --consent prepared/consent.json --output-dir run-1 \
  --recorded-response recorded.json
# response bytes ... output: run-1
```

### 5.3 Validate (quarantine gate)

```bash
forge suggest validate --request prepared/request.json \
  --run prepared/run-1/run.json --output-dir bundle-1
# evidence: high 1 medium 0 low 0 ... output: bundle-1
```

This publishes `prepared/bundle-1/suggestions.json`. Try breaking it to see
the refusal paths (each exits 2 and publishes nothing): alter one byte of the
`quote`, cite a unit that is not in the request, or point a clause at an
invented `policy_key`.

### 5.4 Review (your decisions)

Build `decisions.json` from the bundle. You need the bundle's id, the SHA-256
of the `suggestions.json` bytes, the task copied verbatim, and one record per
suggestion with the suggestion's `content_sha256`. **The `reviewer_key` must
be declared in the project** (here `synthetic-reviewer` — an unknown key
fails promotion later):

```bash
BUNDLE_SHA=$(sha256sum prepared/bundle-1/suggestions.json | cut -d' ' -f1)
python3 -c "
import json
b = json.load(open('prepared/bundle-1/suggestions.json'))
s = b['suggestions'][0]
d = {'schema_version': 'forge.suggest-dispositions/1',
     'bundle_id': b['bundle_id'], 'bundle_sha256': '$BUNDLE_SHA',
     'task': b['task'], 'as_of': '2026-09-08T00:00:00Z',
     'records': [{'suggestion_id': s['suggestion_id'], 'status': 'accept-as-is',
                  'reviewer_key': 'synthetic-reviewer',
                  'decided_as_of': '2026-09-08T00:00:00Z',
                  'rationale': 'Matches the supplied prior policy text.',
                  'original_sha256': s['content_sha256']}]}
json.dump(d, open('decisions.json', 'w'), indent=2)"
forge suggest review --bundle prepared/bundle-1/suggestions.json \
  --decisions decisions.json --output-dir review-1
# decisions: accept-as-is 1 ... undecided: 0 ... output: review-1
```

Statuses are `accept-as-is`, `accept-edited` (keeps both original and edited
hashes), `reject`. Any undecided suggestion exits 1. A wrong `bundle_sha256`,
unknown `suggestion_id`, or mismatched `original_sha256` exits 2.

### 5.5 Promote (proposal, not approval)

```bash
forge suggest promote \
  --bundle prepared/bundle-1/suggestions.json \
  --dispositions prepared/bundle-1/review-1/dispositions.json \
  --request prepared/request.json --destination project.json \
  --output-dir promotion-1
# proposed 1 clause(s) ... status: proposed-unapproved
ls promotion-1 promotion-1/promotion
# promotion.json promotion/proposed-project.json ...
```

`project.json` itself is untouched — the proposal lives beside it as
`proposed-project.json` plus one Markdown entry per suggestion, marked
`proposed-unapproved`. Applying it is a manual step, followed by
`forge author plan`/`build` on the result.

Exit-code summary for the whole pipeline: `0` complete, `1` action required
(execution unavailable, undecided items, nothing promotable — generations are
still published), `2` refusal, publishes nothing (consent mismatch, altered
quote, unknown unit, secret-shaped output, destination drift, existing output
directory — every `--output-dir` must be new).

## 6. If something looks wrong

- `diff -r` between two builds differing? That breaks the determinism
  guarantee — save both directories and report them.
- A refusal naming a SHA mismatch usually means a fixture file changed after
  its pin was recorded; re-copy `examples/authoring` fresh and retry.
- `suggest` steps resolve `run-1`/`bundle-1`/`review-1` beneath the request or
  bundle directory — pass the short names exactly as shown, not new paths.
- Anything that creates, modifies, or deletes files outside the generation
  directory you named is a quarantine bug — report the command and the tree
  diff.
