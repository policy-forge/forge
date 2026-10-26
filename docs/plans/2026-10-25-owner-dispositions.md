# Owner dispositions: D064, D068 and F10

Date: 2026-10-25 (America/Los_Angeles). Baseline: `99cfd54e0d1ffa042599f3c3ef6dfccc51eae341`.
Status: **Recorded.** The project owner approved every disposition below on 2026-10-25.

This record is an asserted human disposition. It does not authenticate identity.
It records no Compliance or Legal review separate from the project owner's approval.
It settles design choices only: no requirement is accepted, no participant,
pilot, interoperability or release gate passes, and no implementation is
authorized to claim verified closure. The proposals these dispositions resolve
are in the [owner decision packets](2026-10-02-roadmap-owner-decisions.md).

## D064 — POA&M editable authority and terminal states (F08)

The [D064 packet](2026-10-02-roadmap-owner-decisions.md#d064--poam-editable-authority-and-scaffold-f07f08)
is approved with these specifics:

- **Editable source of truth.** The closed local `forge.poam/1` manifest is the
  only editable source. OSCAL POA&M is deterministic generated output; there is no
  round-trip editing of native files.
- **Terminal states.** `completed-asserted` and `accepted-risk-asserted` are
  admitted. Each requires an explicit assertion by a reviewer who is independent of
  the item's owners, with the reviewer key, role, time and rationale, plus at least
  one evidence reference that carries its hash and capture time. An assertion
  records that someone asserted the outcome; it never claims verified closure or
  authorized risk acceptance.
- **Reopen.** Reopening creates a new item that explicitly links to the terminal
  item it supersedes. Terminal history is never rewritten.
- **Native projection.** Assertion status appears as Forge-namespaced props on the
  generated POA&M item. An assertion alone never sets a native risk status of
  `closed` or `deviation-approved`.
- **Dates.** The manifest keeps full dates. Where the native schema requires a
  date-time, the author must supply an explicit date-time. Forge never fabricates a
  time of day or time zone.
- **Terminology.** Documentation and output say "declared" or "asserted" risk
  acceptance, never "authorized".

## D068 — Portable review authority and the F12/F13 split

The [D068 packet](2026-10-02-roadmap-owner-decisions.md#d068--portable-review-authority-f12f13)
is approved:

- **Minimum contract.** The implemented contract is the approved minimum: Mapping
  first, then Applicability; immutable snapshots plus hash-bound source references;
  unique declared reviewers, role counts, explicit substitutions, separation of
  authors from reviewers by declared key, and abstentions that never approve.
- **Scope split.** F12 is the 055/056 portable review core. The remaining M-3
  adapters, M-10 proposed edits, S-1 response-file client, S-2 signed envelopes,
  and S-4 supersession for the Lifecycle `/2` and Authoring `/3` families belong to
  F13. Ticket #21's "21 acceptance entries" means 17 Musts plus 4 Shoulds.
- **M-10.** Zero adapters supporting proposed edits is acceptable in F12. When F13
  adds edit support, each response is validated separately: a response with an
  invalid or stale edit is classified on that response, and the rest of the merge
  continues (AC-5).
- **M-13.** No sensitive export mode is required while Forge exports only IDs,
  hashes, reason codes and bounded context. A mode is added only when a real
  workflow needs excerpts or personal data, and it then requires explicit opt-in.
- **S-2.** Deferred until the F13 core is complete. The intended design direction
  is Ed25519 detached signatures over canonical response bytes, with reviewer
  public keys pinned in the queue policy; a signature proves possession of a key,
  not identity. This direction does not approve any crate. A new signature
  dependency still requires its own approval and supply-chain review.

## F10 — Assessment epochs and reviewed-risk handoff (PRD 063 S-3/S-4)

- **S-3 initial states.** Export seeds new `planned` work only. Later status
  changes go through the manifest's append-only history.
- **S-3 existing plans.** Export into an existing plan is approved as add-only. It
  creates new items with new stable identities, never changes existing items, and
  the M-13 baseline reports the new items as added.
- **S-4 windows.** Epoch windows stay sealed and non-overlapping.
- **S-4 identity and continuity.** Native UUIDs are never reused across epochs;
  OSCAL requires them to be unique within a document. Continuity is carried as a
  stable family key and a predecessor UUID in Forge-namespaced props, so tools
  that read only native files can see it. The companion report remains the
  analysis record. Families stay strictly linear: no branching or merging.
- **S-4 mixed epochs.** An appended epoch may add parties, actors and evidence
  receipts, without changing any earlier one. A different Assessment Plan or
  context generation starts a new Assessment Results document, because OSCAL
  Assessment Results has a single `import-ap`.
- **S-4 references.** New findings may reference observations from earlier epochs,
  backward in time only. References to earlier findings or risks stay refused.
- **`forge validate`.** Adding Assessment Results to `forge validate` is approved
  as a small scope expansion beyond PRD 054's original boundary.

## Cross-cutting

- **Windows publication.** Refusing review, POA&M and epoch publication on Windows
  is acceptable while it is documented. The shared fix is one no-replace
  publication path for Windows in the authoring output layer, tracked as its own
  task.
- **Source restore recovery (#42).** No compatibility shim is added for journal
  records left by the pre-fix created-target rollback defect. Source restore has
  not shipped in any release (`v1.1.0` predates it), so only development builds
  can hold that state; a documented manual recovery note is sufficient.
