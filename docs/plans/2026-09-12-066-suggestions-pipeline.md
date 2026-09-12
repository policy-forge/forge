# 066 suggestions pipeline: local-only prepare → run → validate → review → promote

Status: implementation plan for review. No implementation, release, or acceptance
claim.
Owner: Brian Luby. Author: coordinating agent, 2026-09-12.

This plan carves the **task-agnostic, offline** part of
[PRD-066](../PRD/066-prd-ai-assisted-suggestions.md) into one reviewable PR. It
builds the pipeline both candidate generation tasks share, without choosing a
task and without any network boundary. Phase 0 retrieval
(`forge author reuse`) already shipped in PR #153; this plan is the tranche that
follows it, and it supersedes the PRD's external-provider framing with the
owner's 2026-09-12 decisions.

## Inherited owner decisions (2026-09-12)

Recorded in the PRD's Decision Log; not open for re-litigation here.

1. **Local-only model boundary.** No outbound HTTP/TLS, no provider hosts,
   endpoints or keys, no `[ai]` network configuration. Any model call is a local
   adapter launched as a child process, mirroring `src/oscal_cli`, or the
   offline recorded-response workflow. `Cargo.toml`/`Cargo.lock` stay untouched
   unless the owner first approves a dependency.
2. **Corpus-first evaluation, no numeric claim.** An adjudicated corpus and a
   prompt-injection corpus precede task selection; per-task thresholds come from
   them. No time-savings or quality number is published. M-14/M-15 stay open.
3. **Nothing leaves the machine.** No payload egress in any tranche. Only
   verbatim operator-supplied content and hashes are produced.
4. **One PR surfaces the whole PRD-066 work**, not just retrieval.
5. **The first generation task is deferred.** Build the pipeline both tasks
   share; M-2 still requires both versioned task schemas.

## Decision summary

- Five explicit commands: `forge suggest prepare | run | validate | review |
  promote`. Each is a separate process invocation; no command chains into
  another, and no command writes into a pack, project, plan, or manifest.
- The model boundary is a **typed trait over a local child process**. The
  operator supplies the executable path; FORGE never resolves a provider, never
  opens a socket, and never reads a credential.
- Every model byte is untrusted. The adapter's stdout is bounded, decoded
  against a closed schema, citation-checked against supplied spans, and either
  admitted to the `forge.suggestions/1` quarantine bundle or rejected. Nothing
  is repaired, and no rejected content is retained unless the operator asks.
- A run requires a **consent token bound to the exact payload hash and adapter
  identity**. The payload preview is the exact bytes the adapter will receive.
- Promotion emits a **proposal**, validated with the destination's own existing
  contract validator in-process and never written to the destination.

## Why this shape

1. **It is the whole PRD minus the two things nobody can decide yet.** Task
   choice and thresholds need corpora; the pipeline needs neither. Building the
   pipeline first is what makes the corpus work possible: the tester programme
   cannot run without `prepare`/`run`/`validate`/`review`.
2. **Local-only removes the security review as a blocker, not as a topic.** With
   no egress, M-3/M-4/M-5/M-6 collapse to "show the operator the exact bytes you
   are about to hand to a process on their machine, and record their go-ahead".
   The preview and consent are still required — a local adapter is still a
   boundary the operator must see across.
3. **The quarantine contract is the durable part.** Whatever task or model
   arrives later, `forge.suggestions/1`, the disposition manifest and the
   promotion proposal stay the same. The reuse tranche already proved the
   closed-bounded-contract + atomic-publication machinery this needs.

## Command surface

| Command | Reads | Writes | Purpose |
|---|---|---|---|
| `forge suggest prepare` | project manifest + task inputs + adapter path | `forge.suggest-request/1` (+ optional `forge.suggest-consent/1`) | Build the exact payload, allowlisted context, preview, size estimate |
| `forge suggest run` | request + consent token + adapter | `forge.suggest-response/1` raw capture (quarantined staging) | Invoke the local adapter once, bounded and timed |
| `forge suggest validate` | request + raw response | `forge.suggestions/1` bundle | Closed decode, citation check, assumptions, redaction; reject rather than repair |
| `forge suggest review` | bundle + reviewer-supplied dispositions | `forge.suggest-dispositions/1` | Record accept/edit/reject/expiry with reviewer key, time and rationale |
| `forge suggest promote` | bundle + dispositions + destination artifact | `forge.suggest-promotion/1` | Proposed downstream patch, validated, unapproved |

`--format text|json` everywhere JSON is emitted. Text is a rendering of the same
facts. `--output-dir` publishes through the existing atomic no-replace
directory path; existing destinations are refused.

### Data flow

```text
project + task inputs + adapter path
        │
        ▼  prepare ──── request.json  (payload, allowlist, preview, estimates)
        │            └─ consent.json  (only with an explicit --consent)
        ▼  run ──────── response.raw  (bounded stdout capture, quarantined)
        ▼  validate ── suggestions.json  (validated, cited, hash-sealed)
        ▼  review ──── dispositions.json (+ original and edited content hashes)
        ▼  promote ─── promotion.json  (proposed patch, unapproved)
```

No arrow writes into `forge.authoring-pack/1`, `forge.author-project/1`, a
PRD-055 manifest, a PRD-061 clause file, or any lifecycle/evidence/assessment
artifact.

## Frozen contracts

| Contract | Direction | Notes |
|---|---|---|
| `forge.suggest-request/1` | input to `run`/`validate` | Payload artifact digest and byte/unit counts, allowlisted context, adapter target, redaction records |
| `forge.suggest-consent/1` | input to `run` | Payload hash, adapter identity hash, model id, retention notice, operator key, `as_of` |
| `forge.suggest-task-mapping/1` | embedded in request/response | Versioned task schema for mapping candidates (M-2) |
| `forge.suggest-task-drafting/1` | embedded in request/response | Versioned task schema for policy drafting (M-2) |
| `forge.suggest-response/1` | adapter stdout | Untrusted, closed, bounded; tool-call/unknown/oversized content rejected |
| `forge.suggest-run/1` | output of `run` | Receipt binding request, payload, response and adapter digests, with measured time and exit code |
| `forge.suggestions/1` | output of `validate` | Quarantine bundle: validated suggestions + provenance |
| `forge.suggest-dispositions/1` | output of `review` | Reviewer key/time/rationale; original + edited content and hashes |
| `forge.suggest-promotion/1` | output of `promote` | Proposed downstream patch + destination hash; explicitly unapproved |

House rules, matching the existing authoring and reuse tranches:

- Closed serde structs with `#[serde(deny_unknown_fields)]`; absence is
  omission, never `null`; `reject_nulls` after the strict parse.
- `crate::json_strict::parse_value` with an explicit `Limits`, after a raw-byte
  cap, before `serde_json::from_value`.
- Runtime validation is authoritative; the published
  `schemas/forge.*-1.schema.json` files are contracts with
  `additionalProperties: false` and a `$comment` naming runtime-only bounds.
- Failure is `ForgeError::Authoring(String)` (exit 2) or
  `ForgeError::AuthoringActionRequired` (exit 1). New variants are added only if
  a genuinely distinct exit code is needed, and then under the already-committed
  `2.0.0` breaking release.
- Deterministic bytes: no wall clock, locale, environment, absolute path or
  hash-map iteration order. `as_of` is copied from the supplied project input,
  as `forge author reuse` already does.

## Stage detail

### prepare — M-3, M-4, M-5

- **Context allowlist (M-3).** The request contains only spans the operator
  selected: unresolved plan sections, gap IDs, approved answers with
  `public`/`internal` sensitivity, control IDs, and bounded metadata. Files are
  read through the existing confined capture path (`authoring::input::CaptureSet`)
  so aliases, symlinks, pins and non-portable paths are rejected the way
  authoring already rejects them. Answers whose sensitivity is `confidential` or
  `restricted` require an explicit `--include-sensitive-answer` acknowledgement
  and are otherwise refused, not silently dropped.
- **Exact preview (M-4).** The preview is byte-identical to the payload the
  adapter will receive, written as a separate artifact, with target
  (adapter path + hash, task schema version), a data-handling notice, and byte /
  token / span counts. "Estimate" is a **count of supplied bytes and spans**,
  never a cost or token estimate we cannot compute locally — state that in the
  preview text.
- **Redaction.** A secret-like scan (the existing authoring secret-name and
  protected-value rules) runs over the assembled payload. A match **refuses the
  prepare** unless the operator supplied a redaction rule that removes it; after
  redaction the preview shows the redacted bytes exactly. Redaction is
  substitution the operator declared; FORGE never invents replacement text and
  never transmits a match it could not remove.
- **Consent (M-5).** With `--consent`, `prepare` writes
  `forge.suggest-consent/1`: `payload_sha256`, `adapter_sha256` (hash of the
  adapter executable at prepare time), model id (operator-supplied string, never
  probed), retention notice, operator key, `as_of`. Without `--consent`, no
  token is written and `run` refuses.
- **Payload span, not a second copy.** A unit names a byte range inside the
  payload artifact rather than embedding its own text, so the exact bytes the
  adapter receives are recorded exactly once, spans must be ascending and
  non-overlapping, and a citation can be checked against those bytes. The
  payload is deterministic framing around the rendered task payload and the
  selected unit texts; nothing in it is generated prose.
- **The mapping task is not selectable yet.** `--task mapping` is refused with
  a message naming the owner's 2026-09-12 deferral: the mapping task's subject
  derivation is the task-selection decision, and inventing one here would pick
  the deferred task by accident. Its contract, schema and validation tests ship
  and are exercised; only the `prepare` derivation is withheld.
- Exit codes: `0` request written; `1` the selected context is empty, so no
  request is written at all (`AuthoringActionRequired`); `2` invalid, unsafe,
  sensitive, or oversized input.

### run — M-6, M-13

- **Adapter boundary (M-6).** A new crate-level trait mirroring
  `OscalCliInvoke`, object-safe and synchronous:

  ```rust
  pub trait LocalModelInvoke {
      fn invoke(&self, request: &SuggestRequest, timeout: Duration)
          -> Result<AdapterOutput, ForgeError>;
  }
  ```

  `ProcessModelAdapter::new(executable: PathBuf)` implements it:
  `Command::new(&self.executable)` with `env_clear()` and the same environment
  allowlist the oscal-cli invoker uses; the payload goes on **stdin** (not
  argv), stdout is piped and read with an explicit byte cap, stderr is piped to
  a drain thread that **also enforces a cap** (the oscal-cli drain is unbounded;
  this tranche must not copy that gap), and the same `Instant` + `try_wait`
  poll loop with `kill()` + `wait()` + thread join on every early return.
- **No network.** The adapter is a local executable. FORGE adds no HTTP client,
  resolves no host, and passes no proxy or credential variable — the allowlist
  is an explicit transfer list, and `env_clear` is the default.
- **Recorded-response mode.** `RecordedResponseAdapter::new(path)` implements
  the same trait from a fixture file. Every CI test uses it, so the test suite
  needs no model and no binary. The `ProcessModelAdapter` path is exercised only
  in tests that self-exec a trivial helper process (`sh -c 'cat'`, and a
  deliberately hanging/slow variant) — no real model is required for CI.
- **Bounds and failure.** Distinct errors for: adapter missing/not executable,
  spawn failure, non-zero exit, timeout, stdout over the byte cap, and stderr
  over the cap. Each maps to `ForgeError::Authoring` (exit 2) except a
  cleanly-reported adapter failure, which is exit 1: the request was valid, the
  environment could not complete it. Nothing partial is retained.
- **Provenance (M-13).** `run` publishes the exact response beside a
  `forge.suggest-run/1` receipt: request digest, payload digest, response digest
  and length, adapter executable digest, model id as supplied, argv, the
  redaction records, measured `elapsed_ms` and the adapter's exit code. Elapsed
  time is a measurement, not a timestamp; no clock is read for identity. A
  receipt is only written for a successful invocation, so its exit code is 0 by
  construction.
- **Exit codes for `run`.** A contract violation (bad arguments, an unreadable
  request, a consent that does not authorise the payload, a payload or adapter
  that changed after consent) is exit 2. A valid, consented request the local
  environment could not complete — missing, failing, hanging or over-bound
  adapter — is exit 1 with the reason on stdout, and nothing is published.

### validate — M-7, M-8, M-9

- **Closed structured output (M-7).** The raw capture is strict-parsed, size- and
  depth-bounded, then decoded into the task struct. Unknown fields, nulls,
  forward versions, oversized strings, tool-call/function-call shapes, and
  anything that is not the task's closed schema are rejected. Rejection is
  terminal for that response: no repair, no partial admission.
- **Citation validation (M-8).** Every substantive suggestion must carry at
  least one citation naming a supplied source ID and a span inside that source.
  Validation re-reads the captured source bytes and requires the cited bytes to
  match; a missing, nonexistent, out-of-range, re-ordered, or altered citation
  fails the whole response.
- **Assumptions (M-9).** Each suggestion carries explicit `assumptions` and
  `unresolved_questions`. Organization facts are only admissible when they come
  from a supplied answer or literal span; a claim that asserts an org fact with
  no citation is rejected as unsupported.
- **Redaction.** Model-emitted secret-like strings are rejected, not scrubbed
  into the bundle.
- **Whole-response rejection.** A citation that names no allowlisted unit, a
  quote that is not byte-identical to the cited unit's payload span, a shape from
  the other task, a non-zero exit code in the receipt, a receipt that does not
  authorise the request, payload and response, or any decode failure refuses the
  entire response: no partial bundle is published, and the diagnostic names the
  suggestion index without echoing model text.
- **Evidence rating is computed, never self-reported.** *high*: every citation
  cites a selected source span; *medium*: every citation resolves and at least
  one cites supplied metadata rather than a span; *low*: no citation cites a
  span. A suggestion with no citation is refused by the contract itself.
- Output: `forge.suggestions/1`. Every admitted suggestion is pending; a
  suggestion with no disposition record stays pending, so the bundle carries no
  status field at all. Suggestion and bundle identifiers are UUID v5 values over
  the request, response and suggestion content, so the same suggestion always
  receives the same identifier. When `--retain-raw` is passed the exact raw bytes
  are published beside the bundle and `response.retained` is true; the default
  stores hashes only, because raw model text is untrusted and must not ride into
  the default artifact.
- Exit codes: `0` every returned suggestion was admitted; `1` the report is
  complete but empty (nothing to review, `AuthoringActionRequired`); `2` a
  refusal or an unsafe output. Nothing is published on any refusal.
- Contract note: `citations` requires at least one entry and `assumptions` /
  `unresolved_questions` are required fields (`M-8`, `M-9`), in the runtime
  validator and the published schemas alike — the two never disagree.

### review and quarantine — M-10, M-11

- **Quarantine only (M-10).** The only artifact family these commands write is
  `forge.suggest-*`. There is no code path that constructs a mapping manifest,
  clause file, lifecycle record, evidence record, assessment result or POA&M.
- **Disposition (M-11).** For each suggestion: `accept-as-is`, `accept-edited`,
  `reject`, or `expired`, with reviewer key, `as_of` (supplied, never a clock),
  and a non-empty rationale. `accept-edited` requires the edited content and its
  hash; the original content and hash are always preserved. Expiry is computed
  only against a supplied `--as-of`; FORGE never consults the system clock.
- Rendered reports never use approved, compliant, certified, effective or
  validated language about a suggestion; a disposition is a review record, not
  an approval of the underlying control relationship or policy text.

### promote — M-12

- `promote` selects only suggestions with an `accept-as-is` or `accept-edited`
  disposition (expired and rejected ones never promote).
- It constructs the downstream record for the named destination and **runs the
  destination's existing validator in-process** on the proposed bytes: the
  PRD-055 manifest parser or the PRD-061 pack/clause contract, whichever the
  request named. If validation fails, `promote` fails; it does not emit a
  proposal that could not be applied.
- Output `forge.suggest-promotion/1` names the destination, its expected hash,
  the proposed record bytes and hashes, and carries an explicit
  `approved: false`-style marker in its own vocabulary (`status:
  "proposed-unapproved"`). It is never written to the destination.

## Provenance, evaluation and consent boundaries

- **M-13** provenance lives in the bundle and the promotion proposal; every
  field is either operator-supplied, measured, or a hash.
- **M-14/M-15 remain open.** This plan builds what the gates will need
  (recorded responses, deterministic packaging, per-suggestion evidence rating
  by deterministic citation checks) and claims no threshold. No numeric
  time-savings or quality statement is made anywhere in the tranche's output.
- **M-16** telemetry and training are absent rather than configurable off:
  there is no telemetry code path, no corpus upload, and no training use. The
  request records the operator's retention notice verbatim; FORGE has no other
  channel.

## Safe I/O and bounds

All bounds are runtime-checked and reported in the error path; the JSON Schema
`$comment`s name those the schema cannot express.

| Bound | Value |
|---|---|
| Project/request manifest | 2 MiB |
| Aggregate captured input | 50 MiB |
| Per captured document | 1 MiB |
| Per string | 16 KiB |
| Payload preview / adapter stdin | 8 MiB (payload is a strict subset of captured input) |
| Adapter stdout (`forge.suggest-response/1`) | 8 MiB |
| Adapter stderr (drained) | 256 KiB |
| Adapter timeout | operator `--timeout`, default 60 s, range 1–3600 s |
| Suggestions per bundle | 1 000 |
| Citations per suggestion | 64 |
| Spans per citation | 1 |
| Dispositions per bundle | 1 000 |
| Published generation | 50 MiB over ≤2 048 artifacts (existing limit) |
| Publication | atomic no-replace directory rename, Linux/macOS; other platforms fail closed |

## Determinism rules

- Generation is not deterministic; **packaging is**. Re-running `validate`,
  `review` and `promote` on the same inputs must produce byte-identical output.
- No system clock anywhere: `as_of` comes from the project manifest. Elapsed
  time is recorded as a duration measurement and never used for identity.
- Ordering is total and explicit: suggestions sort by `(section order, source
  key, span start, suggestion id)`, dispositions by `(suggestion id)`,
  citations by `(source key, span start)`. All indexes are `BTreeMap`/`BTreeSet`.
- Paths in output are portable and relative, as reuse already enforces.
- Every output byte is written through `write_output` and the bounded encoder
  with a single trailing newline.

## Verification matrix

| Requirement | Executable evidence |
|---|---|
| M-1 separate steps | CLI test drives each command alone in a fresh process and asserts no other command runs implicitly |
| M-2 versioned task schemas | Both schemas parse, reject unknown versions, and are `include_str!`-checked for `$id` + `additionalProperties: false` |
| M-3 allowlist | Request built from a project with unrelated files and sensitive answers contains none of them; confidential answer requires the explicit flag |
| M-4 exact preview | Preview bytes equal the bytes written to the adapter's stdin, byte for byte, including after redaction; target/notice/estimates present |
| M-5 consent | `run` without a token fails; token with wrong payload hash, wrong adapter hash, wrong model id or wrong retention notice fails; matching token runs |
| M-6 local boundary | `env_clear` + allowlist asserted; adapter is a local path; test asserts no `http`/`https`/`TcpStream`/`Client` symbol is reachable from the adapter module |
| M-7 closed output | Unknown field, null, forward version, oversized string, oversized stdout, truncated JSON, tool-call shape and non-task shape each rejected |
| M-8 citations | Missing, nonexistent source, out-of-range span, altered span bytes, and re-ordered citation rejected; valid citation admitted with exact bytes |
| M-9 assumptions | Suggestion without assumptions/questions rejected; org fact without citation rejected; supplied-answer-backed fact admitted |
| M-10 quarantine | No `suggest` invocation creates, modifies or deletes any non-`suggest` artifact; repository tree compared before/after |
| M-11 disposition | Each status requires rationale + reviewer key + `as_of`; `accept-edited` preserves original and edited hashes; expiry uses only supplied `as_of` |
| M-12 promotion | Promotion fails when the destination validator rejects the proposed bytes; succeeded proposal is `proposed-unapproved` and the destination file is untouched |
| M-13 provenance | Bundle contains adapter/model hashes, request/response hashes, byte counts, exit status and redaction policy; no field reads a clock |
| M-14/M-15 | Not claimed. The corpus runner harness is exercised with synthetic fixtures only; thresholds stay open |
| M-16 | Asserted by absence: no telemetry symbol, no upload path, no training hook in the crate's adapter or bundle modules |
| M-17 adversarial suite | Dedicated `tests/suggest_adversarial_test.rs`: prompt injection in supplied source text, fabricated/altered citations, exfiltration requests (absolute paths, env values, unrelated files), malformed/oversized output, adapter missing/hang/crash/stderr flood, seeded secrets, quarantine escape attempts, promotion validity |
| Determinism | Repeated `validate`/`review`/`promote` runs in two different directories produce byte-identical outputs |
| Legacy behaviour | Full locked suite unchanged; existing command outputs byte-identical; no new default artifacts |
| Delivery | `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `git diff --check` |

## Phase slices

One reviewable commit slice per row, tests alongside each.

| # | Slice | Contents | Gate row |
|---|---|---|---|
| 1 | Contracts + skeleton | `forge.suggest-request/1`, `forge.suggest-consent/1`, both task schemas, `forge.suggest-response/1`, `forge.suggestions/1`, `forge.suggest-dispositions/1`, `forge.suggest-promotion/1`: closed structs, strict decode, bounds, published schemas | GATE-SUGGEST-CONTRACT |
| 2 | `prepare` | Context allowlist, redaction scan, exact preview, consent token, estimates | GATE-SUGGEST-CONTEXT |
| 3 | `run` | `LocalModelInvoke`, `ProcessModelAdapter`, `RecordedResponseAdapter`, bounds, timeout, provenance | GATE-SUGGEST-ADAPTER |
| 4 | `validate` | Closed decode, citation re-validation, assumptions, redaction of model text, quarantine bundle | GATE-SUGGEST-VALIDATION |
| 5 | `review` | Disposition manifest, reviewer metadata, expiry against supplied `as_of` | GATE-SUGGEST-DISPOSITION |
| 6 | `promote` | Downstream proposal, in-process destination validation, unapproved marker | GATE-SUGGEST-PROMOTION |
| 7 | Safe I/O + provenance | Bounds audit, atomic no-replace publication, determinism tests, provenance completeness | GATE-SUGGEST-SAFETY |
| 8 | Docs + gates | PRD-066 Decision Log/DoR reconciliation, `GATE-SUGGEST-*` rows in [authoring-gates](../authoring-gates.md), CHANGELOG, roadmap, usage docs | GATE-SUGGEST-DOCS |

Slices 2–6 depend on 1; 7 depends on 2–6. 8 lands last and claims only what 1–7
proved. `Cargo.toml`/`Cargo.lock` are unchanged by every slice.

## Tester programme

Who: the three design-partner reviewers recorded under GATE-PARTNERS, plus the
maintainer. No new recruitment, no names in the repository.

What each tester does, on a real project of their own:

1. Pick the task input they already have (an unresolved mapping set or an
   authoring plan) and run `prepare` with their own local adapter or recorded
   fixture.
2. Read the preview, confirm the payload contains nothing they did not choose,
   then consent and run.
3. Review every suggestion: accept, edit, reject, or let it expire. Rate each as
   useful / irrelevant / wrong or fabricated.
4. Attempt promotion on anything they accepted, and report whether the proposed
   patch would have been acceptable had they applied it themselves.

Returned to the project: the request/preview, bundle, disposition and promotion
JSON (all contain paths, hashes, spans and metadata — no confidential prose is
required), plus the ratings.

What we measure (hypotheses, not results): citation-validity rate, unsupported
or fabricated claim rate, useful-suggestion rate, disposition distribution, and
the share of promotions that pass the destination validator. Stage exit
criteria: every tester completes one project, zero citation mismatches in
admitted bundles, every adversarial case in the suite passes, and no gate above
is claimed as satisfied by the exercise.

Feedback capture: the same opt-in plain-text summary mechanism the reuse tester
programme already documents. No telemetry, no automatic upload.

## Open items for the owner

1. **Which generation task ships first** — still deferred (inherited decision 5).
2. **M-14/M-15 numeric thresholds** — after adjudicated corpora exist.
3. **Local adapter shipping in this PR or a follow-on tranche.** This plan
   assumes this PR (slice 3, child process plus recorded-response fixture), and
   adds no dependency. The alternative is to ship recorded-response only and
   defer `ProcessModelAdapter`; that would leave M-6's boundary unproven.
4. **Whether to name the five `SuggestCommand` variants publicly now.** As with
   `AuthorCommand`, adding a public clap enum is a breaking change for
   downstream `match`es, carried by the already-committed `2.0.0` release.

## Decisions

Proposed 2026-09-12; owner agreement required before slice 1.

- **D1 — Top-level `forge suggest`.** One canonical command family, crate-level
  entry points per stage (`suggest::prepare`, `run`, `validate`, `review`,
  `promote`) so a later MCP tool or subcommand can wrap them. No duplicate
  spellings.
- **D2 — Adapter transport is stdin/stdout, not argv or temp files.** The
  payload can be large and must be exactly previewed; stdin is the only channel
  with no argv length limit, no filename in the process table, and no
  intermediate file to leave behind.
- **D3 — Recorded-response adapter is a first-class implementation**, not a test
  shim: it is the offline workflow the PRD calls for, and CI depends on it.
- **D4 — Default bundle stores hashes, not raw model text.** Raw retention is
  opt-in per run. Untrusted text must not become the default artifact.
- **D5 — Redaction is refusal-first.** A secret-like match stops `prepare` unless
  the operator declares the substitution; FORGE never invents replacement text.
- **D6 — `promote` validates with the destination's own code path** rather than
  a copy of its rules, so a proposal cannot pass a validator the destination
  would reject.
- **D7 — Bounds above are the initial values**, raised only with a measured need
  and a test; they are not placeholders.
- **D8 — Gates are recorded as `GATE-SUGGEST-*`** in
  [authoring-gates](../authoring-gates.md), tracked like `GATE-REUSE-*`: contract,
  context, adapter, validation, disposition, promotion, safety, docs, plus open
  usefulness and release gates.

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1 | 2026-09-12 | coordinating agent | Initial plan: local-only offline pipeline, contracts, bounds, verification matrix, tester programme, owner decisions D1–D8 |
