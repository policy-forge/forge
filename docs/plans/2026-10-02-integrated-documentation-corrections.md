# Integrated documentation corrections — rolling delivery

This is task #33 (id 1428), stacked on source basis
`3d9345bda6f8c7eb99e69186b6897a874eff4579`. It prepares a rolling correction
slice from the immutable `532c9e8` review and the `0e2b803` successor. It does not
close the user's final all-work integrated documentation review.

The original 39-path application is followed by a six-path Rustdoc/help/README
supplement, overlapping two original files and adding four, for 43 distinct
paths. Three dated F04 guide checkpoints bring the intended set to 46. This plan
and the [raw validation audit](2026-10-02-rolling-docs-checks-v1.json) bring the
intended delivery to 48. Proposed, applied, validated, committed and hosted states
must be read separately from their exact source receipts; path arithmetic is not
a claim that pending files or checks have already been delivered.

## Original correction groups

| Group | Concrete correction |
| --- | --- |
| R-DOC-01 | Describe the pinned OSCAL v1.2.3 validation baseline and current generated declarations; retain older checked-in example output as historical. |
| R-DOC-02 | Count eleven pinned assets: seven runtime JSON schemas and four compatibility-test XSDs; include Mapping and Assessment Results in the upgrade download patterns. |
| R-DOC-03 | Describe the existing metadata preview/local download and registered-resource comparison UI; retain the boundary against writable import, authority and project publication. |
| R-DOC-04 | Limit existing `export` to Catalog and Component Definition typed projections across JSON/XML/YAML; remove arbitrary native-tree/all-format fidelity claims. The separate F09 preservation draft is not silently integrated. |
| R-DOC-05 | Match release archive naming configured by the workflow using explicit tag/target placeholders and download-only examples; claim no live tag or verified release. |
| R-DOC-06 | Correct the documented existing `jsonschema` version to 0.57 without changing dependencies or audit dispositions. |
| R-DOC-07 | Separate implemented technical tranches from the completed WI history, full PRD acceptance and still-separate roadmap scope. |
| R-DOC-08 | Refresh domain/workspace/export codemaps and architecture descriptions against actual modules and interfaces, without advertising separate unintegrated drafts. |
| R-DOC-09 | Describe Markdown/PDF/DOCX policy ingestion and supported output projections in crate/convert documentation. |
| R-DOC-10 | Reference recorded authoring owner dispositions and the 2.0.0 migration target while keeping rights authentication, design-partner/pilot and release approval gates open. |
| R-DOC-11 | Qualify the historical initial workspace API 1.1.0 versus current unreleased 1.2.0 wording. |
| R-DOC-12 | Repair confirmed relative navigation targets, including PRD links; preserve requirement text, checkbox state and owner dispositions. |

These groups repair current descriptions and navigation. Historical audits,
receipts, fixtures, outputs, completion history and acceptance snapshots keep their
original bytes and dates. Current source claims remain separate from unintegrated
POA&M, suggestion-evaluation, export-preservation and other draft work.

## Separately disclosed help and Rustdoc changes

ROLL-HELP-01 changes the CLI `long_about` literal to describe policy-document
inputs (Markdown, PDF or DOCX) and Catalog/Component Definition outputs in JSON,
XML or YAML. This is an intentional generated-help change, not a comment-only
non-comment-token equality claim. It does not expand runtime format/model support,
SSP output formats, command arguments or dispatch.

The inherited convert-input comment, the supplementary two-model export command
and input comments, and the lifecycle-init Party grammar code span also change
Clap-generated help intentionally. The Party grammar remains `KEY=ROLE[,ROLE]`;
putting it in a code span prevents `[,ROLE]` from being interpreted as a Rustdoc
item link. No argument grammar or parser change follows.

The preserved baseline combined library/binary Rustdoc invocation exited 101,
with three broken links, two private-link warnings and a same-name output
collision. The repairs target the actual defining files:

- `src/diff/canonical.rs` links to public `crate::diff::diff_artifacts`.
- `src/oscal/mod.rs` links to public `crate::oscal::metadata::OSCAL_VERSION`.
- `src/mapping/model.rs` renders the private `BuildProduct::finalize_report` name
  as a code span rather than a public link.
- `src/parse/modality.rs` renders private `detect_modality` as a code span.
- `src/cli/mod.rs` renders the Party declaration grammar as a code span.

The OSCAL namespace repair is in `src/oscal/mod.rs`, not a speculative `src/lib.rs`
metadata change. Public visibility is unchanged. The root must record separate
strict library and binary Rustdoc outputs to avoid the collision; the old failure
remains evidence rather than being rewritten as a pass.

README now states that the release workflow is configured to publish SHA-256
checksums and SLSA provenance. A selected release's artifacts and verified level
need their own evidence. Configuration of the SLSA generator does not establish
that every release has verified Level 3 provenance.

## Dated F04 checkpoints

The three guide additions preserve their preceding historical sections and append
only scoped source-bound checkpoints:

- [Maintained-client extension](../development-tools/maintained-client-extension.md):
  requested `63ab9c2`, tested ordered merge `7985b011`; three qualified platform
  API receipts each record nine contract tests, 22 workflow tests, sixteen groups
  and 37/39 declared operations observed. Cancellation and unlock are unobserved.
  Windows has 242 requests; Ubuntu/macOS each have 238. Native siblings and full
  CI are outside that qualification.
- [Installed Chrome](../development-tools/hosted-chrome.md): requested `0e2b803`,
  tested ordered merge `ff2d3252`; all four scoped default/long read-only/writable
  campaigns pass in the qualified viewport checkpoint. All reported capture
  counters are zero, earning no counter-correlation coverage. Earlier failed
  screenshot outcomes and their unknown cause remain preserved.
- [Linux headless OS denial](../development-tools/linux-headless-os-denial.md):
  requested `3d9345b`, tested ordered merge `4921c756`; the qualified wrapper/2
  result is incomplete, `tool-untrusted`, at `stdlib-entry` / `worker-writable`.
  Tools are null, the native producer is not run, cleanup is nonforced
  `verified-not-created`, and attempted egress is unmeasured/null. No underlying
  object identity, effective write authority or native repair is inferred.

These dated checkpoints do not qualify a new documentation head or turn sibling,
partial status, workflow assertions or producer-observed hashes into full CI,
startup attestation, release or human acceptance.

## Validation and remaining gates

Recorded validation is in the raw audit. The earlier four passing help captures
belong to the 39-path snapshot and remain historical after the supplementary help
comments. Fresh final-source checks must cover top-level, convert, export,
validate and lifecycle-init help, separate strict library/binary Rustdoc, relative
links and the enabled repository hook. Future commands are not recorded here as
passing. The root integration owner records actual source/tool/binary/exit bindings in
the immutable pre-commit audit, and records enabled-hook results and final delivery
state in a separate fresh delivery receipt.

The source scope adds no production function body, dependency, public API,
visibility or signature. Static non-comment/token preservation excludes the
explicit CLI help literal; generated help changes are intentional. Documentation
and help checks provide no new production line/function/branch coverage. Existing
raw mock/native profiles retain their exact selected/whole denominators, zero and
unmapped records, dates and authorship exclusions; neither they nor this document
are a new coverage campaign or repository-wide documentation claim.

This plan is authored preparation; its author also authored earlier rolling
proposals and cannot independently approve their correctness. Root review,
application, fresh validation, mandatory hook, commit/PR and any hosted
qualification are separately evidenced. Full F04 Must/Should, remaining model and
workspace roadmap implementation, owner/dependency-audit and human/accessibility
acceptance, release/pilot gates, and the final integrated all-work documentation
review remain open.
