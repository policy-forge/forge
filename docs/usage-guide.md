# FORGE Usage Guide

Complete end-to-end walkthrough for FORGE — Framework for OSCAL Risk & Governance Execution.

## 1. Installation

### From source (requires Rust 1.93.0+)

```bash
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build --release
./target/release/forge --version
```

### From binary release

Download the selected published binary for your platform from GitHub Releases. The [release workflow](../.github/workflows/release.yml) is configured to publish SHA-256 checksums and SLSA provenance. Check the evidence attached to the selected published tag for the archive hashes and any verified SLSA level.

Verify your installation:

```bash
forge --help
```

The help output lists conversion, validation, profile, mapping, migration,
configuration, drift, traceability, and lifecycle workflows. Use
`forge <SUBCOMMAND> --help` for the exact command contract in this release.

For local agent queries, see [the MCP guide](mcp.md). `forge mcp serve` exposes
seven read-only tools over stdio, with an explicit project and reviewed disclosure
scope. Static discovery does not authorize project data. The guide also records
the separate [F20 `/2` development checkpoint](mcp.md#f20-2-development-checkpoint);
those candidate inputs do not activate additional shipped MCP features.

## 2. Writing a Policy Document

FORGE accepts Markdown files (`.md` / `.markdown`) with optional YAML frontmatter. Headings become OSCAL groups; list items, tables, and paragraphs become control statements.

### Minimal policy example

Create a file named `policy.md`:

```markdown
---
title: "Access Control Policy"
version: "1.0.0"
author: "Security Team"
date: "2026-01-15"
---

# Access Control

All users must authenticate before accessing systems.

## Authentication Requirements

- Users must use multi-factor authentication
- Passwords must be at least 12 characters
- Sessions must timeout after 30 minutes of inactivity

## Authorization

- Access must follow principle of least privilege
- Role-based access control must be enforced

# Data Protection

## Encryption

- Data at rest must be encrypted using AES-256
- Data in transit must use TLS 1.2 or higher
- Encryption keys must be rotated annually
```

### How it works

- YAML frontmatter sets metadata: `title`, `version`, `author`, `date`
- Level-1 headings (`#`) become top-level OSCAL groups
- Level-2+ headings (`##`, `###`) become nested groups
- List items (`-`) become individual controls
- Paragraphs become control statements
- Compound requirements like "Systems must X and must Y" are automatically split into atomic controls

### Advanced features

FORGE detects and processes:
- **Requirement atomization** — splits "must X and must Y" into separate controls
- **Modality detection** — classifies statements as mandatory (MUST/SHALL) or advisory (SHOULD/MAY)
- **Parameter extraction** — turns prose thresholds (e.g., "12 characters", "30 minutes") into structured OSCAL parameters; enforcement depends on separately configured consumers
- **Citation extraction** — URLs and references become OSCAL back-matter resources
- **Content-derived identifiers** — UUID v5 tags repeat for the same normalized requirement text; substantive text or duplicate-occurrence changes can change them. These tags are not integrity or approval anchors.

25 sample policies are included in `example_data/` covering topics from acceptable use to incident response.

PDF and DOCX source documents are also accepted directly — Word heading and list styles are mapped to the document model automatically. For other formats, convert to Markdown first using [pandoc](https://pandoc.org/) or [markitdown](https://github.com/microsoft/markitdown).

## 3. CLI Workflows

### 3.1 `convert` — Convert Policy to OSCAL

Converts a Markdown policy document into an OSCAL Catalog or Component Definition.

#### Catalog strategy

Produces an OSCAL Catalog with groups, controls, and statements:

```bash
# Basic conversion — outputs JSON to stdout
forge convert policy.md --strategy catalog --format json

# Write to a file
forge convert policy.md --strategy catalog --format json --output catalog.json

# Output as XML
forge convert policy.md --strategy catalog --format xml

# Output as YAML
forge convert policy.md --strategy catalog --format yaml
```

#### Component Definition strategy

Produces an OSCAL Component Definition with implemented requirements. Requires `--source-profile` for schema-valid output:

```bash
# With a source profile (OSCAL Profile JSON)
forge convert policy.md --strategy component --format json \
  --source-profile baseline-profile.json

# With source profile + XML output
forge convert policy.md --strategy component --format xml \
  --source-profile baseline-profile.json --output component.xml
```

#### Additional convert options

```bash
# Override max input file size (default: 10 MB)
forge convert large-policy.md --strategy catalog --format json --max-size 20

# Enable verbose pipeline logging (shows each stage)
forge -v convert policy.md --strategy catalog --format json

# Suppress all non-essential output (OSCAL artifact only)
forge -q convert policy.md --strategy catalog --format json

# Detect substantive changes against a baseline
forge convert policy-v2.md --strategy catalog --format json \
  --stable-id-baseline policy-v1.md

# Generate an Assessment Plan alongside the Catalog
forge convert policy.md --strategy catalog --format json \
  --import-ssp system-security-plan.json

# Print a conversion summary dashboard to stderr
forge convert policy.md --strategy catalog --format json --summary

# Batch conversion (multiple files)
forge convert pol-*.md --strategy catalog --format json --output out/

# Batch with parallel jobs
forge convert pol-*.md --strategy catalog --format json --output out/ --jobs 4
```

### 3.2 `export` — Convert Between Formats

Converts a supported Catalog or Component Definition between JSON, XML and YAML. The input format is detected from its extension; other OSCAL models are not supported by `export`.

```bash
# JSON to XML
forge export catalog.json --format xml

# XML to YAML
forge export catalog.xml --format yaml

# YAML to JSON, written to a file
forge export catalog.yaml --format json --output catalog.json

# JSON Component Definition to XML
forge export component.json --format xml
```

File extensions recognized: `.json`, `.xml`, `.yaml`, `.yml`.

The Catalog or Component Definition model is detected from the document root. For JSON/YAML inputs and JSON/YAML targets, export validates the complete decoded JSON-compatible tree against the pinned OSCAL schema and semantic checks before publication. Supported native fields and array order remain in that tree; invalid unknown schema fields are rejected rather than silently discarded. Only numeric values represented exactly as i64/u64 integers are admitted: every f64 representation is refused, including `1.0`, `1e0` and oversized JSON integers decoded as floating point. JSON object order, whitespace, original bytes and YAML tags/comments/anchors are outside this decoded-tree contract. XML input/output and direct typed helper APIs still use a partial model projection, so retain the original for those paths. See the [prerequisite scope and verification obligations](plans/2026-10-03-f09-lossless-prerequisite-rebase.md).

### 3.3 `validate` — Schema and Semantic Validation

Validates supported OSCAL JSON artifacts against the pinned OSCAL v1.2.3 JSON schemas with semantic checks. Supported older declarations use the same pinned baseline; metadata never selects or downloads another schema.

```bash
# Basic validation with human-readable output
forge validate catalog.json

# Machine-parseable JSON output
forge validate catalog.json --format json

# Override auto-detected model type
forge validate artifact.json --schema-type catalog
forge validate artifact.json --schema-type component-definition

# Write validation results to a file
forge validate catalog.json --output validation-report.txt
```

On valid: prints "Valid: catalog artifact passes all validation." and exits 0.
On invalid: renders the error report to stderr and exits non-zero.

#### Round-trip validation

Tests format fidelity by running the artifact through a full conversion chain (JSON → XML → YAML → JSON) via `oscal-cli`, then comparing the result against the original:

```bash
# Requires oscal-cli on PATH
forge validate catalog.json --round-trip

# Custom oscal-cli path and timeout
forge validate catalog.json --round-trip \
  --oscal-cli-path /usr/local/bin/oscal-cli --timeout 60

# Machine-parseable round-trip results
forge validate catalog.json --round-trip --format json
```

Reports any divergences with classification markers: `FORGE-FIX`, `OSCAL-CLI`, `ACCEPT`.

### 3.4 `resolve` — Resolve OSCAL Profile to Catalog

Resolves an OSCAL Profile into a flat Catalog baseline by delegating to `oscal-cli`. Requires `oscal-cli` on PATH (Java-based).

```bash
# Resolve a Profile (requires .json input)
forge resolve nist-800-53-profile.json

# Custom output path
forge resolve profile.json --output resolved-catalog.json

# Custom timeout (default: 60s)
forge resolve profile.json --timeout 120

# Custom oscal-cli binary path
forge resolve profile.json --oscal-cli-path /usr/local/bin/oscal-cli

# Check oscal-cli availability without resolving
forge resolve --check
```

Default output path: `<input-stem>-resolved.json` in the same directory.

### 3.5 `trace` — Source-to-OSCAL Traceability

Generates a traceability report mapping OSCAL elements back to their source policy locations.

```bash
# Trace an OSCAL artifact against its source policy
forge trace catalog.json --source policy.md

# Write report to a file
forge trace catalog.json --source policy.md --output trace-report.txt
```

The output is a column-aligned table:

```
OSCAL Element ID    Element Type    Source Section           Source Line
----------------    ------------    --------------           -----------
access-control      group           Access Control           —
POL-AC-001          control         Access Control           10
POL-AC-002          control         Access Control           25
POL-DP-001          control         Data Protection          27
[unmapped]          control         [unmapped]               [unmapped]

Summary: 5 elements, 4 mapped, 1 unmapped (80.0% coverage)
```

- Groups with a section but no specific line show an em dash (—) for Source Line
- Unmapped elements show `[unmapped]` in source columns
- A staleness warning appears if the source file has been modified since conversion

### 3.6 `diff` — Compare OSCAL Artifacts

Compares two OSCAL artifacts (Catalogs or Component Definitions) and shows differences.

```bash
# Compare two catalogs
forge diff catalog-v1.json catalog-v2.json

# Compare two component definitions
forge diff component-old.json component-new.json
```

The output includes:

```
OSCAL Diff Report
=================
Old: catalog-v1.json  (catalog)
New: catalog-v2.json  (catalog)

Summary
-------
Controls (old): 5  |  Controls (new): 6
Added: 1  |  Removed: 0  |  Changed: 1  |  Unchanged: 4  |  UUID changes: 0

Added (1)
─────────
  + POL-DP-003  [uuid: ...]

Changed (1)
───────────
  ~ POL-AC-001
      title: "Old title"  →  "New title"
```

Exits 1 if differences are found (useful in CI pipelines).

### 3.7 `drift` — Content-Safe Generated-Artifact Check

Compares the complete parsed JSON value of committed and newly generated
Catalog or Component Definition artifacts. It ignores only the generated root
`uuid` and `metadata.last-modified`; every nested UUID and all other metadata and
policy-derived fields remain significant.

```bash
# Human-readable status
forge drift committed/catalog.json staged/catalog.json

# Machine-readable status for CI orchestration
forge drift committed/catalog.json staged/catalog.json --format json
```

Both output formats contain only status, artifact type, and comparison-contract
version. They do not include file paths, control identifiers, titles, prose, or
JSON excerpts. Exit `0` means clean, exit `1` means drift, and exit `2` means the
inputs could not be compared. This is a comparison primitive, not schema
validation; enforcement workflows must run `forge validate` first.

### 3.8 `profile` — Generate OSCAL Profile from Catalog

Creates an OSCAL Profile by selecting specific controls from a source Catalog.

```bash
# Include specific controls
forge profile --catalog nist-800-53.json \
  --include "ac-1,ac-2,ac-3,ia-1,ia-2"

# Exclude specific controls
forge profile --catalog nist-800-53.json \
  --exclude "ac-1,ac-2"

# Output as XML or YAML
forge profile --catalog catalog.json --include "ac-1" --format xml
forge profile --catalog catalog.json --include "ac-1" --format yaml

# Write to a file
forge profile --catalog catalog.json \
  --include "ac-1,ac-2" --output my-profile.json

# Set parameter overrides in the Profile's modify section
forge profile --catalog catalog.json --include "ac-2" \
  --set-param ac-2_prm_1 "30 days" \
  --set-param ac-2_prm_2 "12 characters"

# Override last-modified timestamp (ISO 8601) for reproducible output
forge profile --catalog catalog.json --include "ac-1" \
  --timestamp "2026-01-15T12:00:00Z"
```

`--include` and `--exclude` are mutually exclusive. At least one must be provided (unless using only `--set-param`, which produces a Profile with empty imports and a warning).

### 3.9 `mapping` — Publish Human-Reviewed Control Relationships

Control Mapping accepts explicit reviewer decisions from a closed,
versioned `forge.mapping-manifest/1` JSON document. It never downloads
framework content, follows OSCAL links, invokes `oscal-cli`, generates
relationship candidates, or interprets gaps as compliance failures.

```bash
# Deterministic unapproved scaffold; add reviewers, rationale, and maps before build
forge mapping init --source policy-catalog.json --target framework-catalog.json \
  --output mapping-manifest.json

# Profiles need a caller-produced resolved Catalog companion
forge mapping init --source policy-catalog.json --target framework-profile.json \
  --target-resolved-catalog framework-resolved.json \
  --output mapping-manifest.json

# OSCAL JSON and reports always use separate streams/files
forge mapping build --manifest mapping-manifest.json \
  --output mapping.json \
  --report mapping-report.json --report-format json

# Read-only baseline impact check for CI
forge mapping check --manifest mapping-manifest.json \
  --baseline mapping.json --report-format json --fail-on any
```

The manifest declares `control-only` or `control-plus-statement` review scope,
stable collection/mapping/map keys, resource paths and expected hashes,
reviewer parties, provenance, and one or more explicit maps. Many-to-many sets
remain one map and source/target direction is never reversed. Missing controls,
wrong semantic types, stale scaffold inventories, duplicate IDs/keys, invalid
vocabulary, and confidence/coverage values outside `0..=1` fail before output.
For a Profile, the reviewer must set `resolved_catalog_attestation: true` after
confirming that the explicit companion represents that Profile; FORGE records
both hashes but does not authenticate the reviewer or independently prove the
resolver lineage.

Reports label ratios as **review participation**, never compliance coverage.
By default they contain IDs, counts, hashes, and stable machine finding codes.
`--include-excerpts` adds bounded titles/prose and makes the report as sensitive
as its source frameworks. Exit `0` means analysis completed without the selected
review policy firing, exit `1` means completed analysis requires human review,
and exit `2` means no trustworthy artifact or report could be produced.

Reviewer names and rationale become durable artifact data. FORGE preserves but
does not authenticate identity, authority, approval, or signatures. Users are
responsible for permission to process and share framework content; FORGE ships
no third-party framework catalogs and defaults reports to IDs and hashes.

### 3.10 `lifecycle` — Manage Reviewable Policy State

New lifecycle records use the closed, bounded `forge.policy-lifecycle/2` JSON
contract. Legacy `/1` records remain readable by `check` and `status`; migrate
them non-destructively before transitions or attestations. One file describes
one immutable policy version, its source and
generated-artifact hashes, declared parties, versioned approval policy,
date-only review schedule, current state, optional replacement, and append-only
transition events. Unknown or duplicate JSON keys, unsupported versions,
sequence gaps, invalid event UUIDs, unsafe aliases, and exceeded limits fail.

```bash
forge lifecycle init --source policy.md --artifact catalog.json \
  --output policy-lifecycle.json --policy-key access-control \
  --version-key v1 --title "Access Control Policy" --owner alice \
  --party alice=owner,author --party bob=reviewer --party carol=approver \
  --next-review 2027-08-25 --separate-reviewer-approver

# Preserve a legacy /1 file and produce a context-bound /2 replacement.
forge lifecycle migrate --record policy-lifecycle-v1.json \
  --output policy-lifecycle-v2.json

# Without --apply, transition writes a complete proposal to stdout or --output.
forge lifecycle transition --record policy-lifecycle.json --to in-review \
  --actor bob --role reviewer --at 2026-08-25T17:00:00Z \
  --rationale "Review completed" --apply
forge lifecycle transition --record policy-lifecycle.json --to approved \
  --actor carol --role approver --at 2026-08-25T18:00:00Z \
  --rationale "Approved for publication" --apply

forge lifecycle check --record policy-lifecycle.json --format json
forge lifecycle status --record policy-lifecycle.json \
  --as-of 2026-08-25 --format json --gate publication

# Explicit portfolio review queue grouped by owner and date
forge lifecycle queue --record policy-lifecycle.json \
  --as-of 2026-08-25 --format json --gate publication

# Deterministic unsigned evidence suitable for a separate signing system
forge lifecycle attest --record policy-lifecycle.json \
  --output approval-attestation.json
```

Approval evidence must use identical fingerprints and meet every declared role
count. Additional distinct evidence can be supplied as repeatable
`--assertion ACTOR=ROLE` values. Separation rules compare declared actor keys;
they do not prove identity or authority. When author/reviewer or
author/approver separation is enabled, the review window must include at least
one declared `author` assertion; missing author evidence fails closed. An
approved record whose current bytes differ reports `approved-drifted`; a
generated artifact type or root-UUID change is also reported as action required
with `artifact-identity-changed`, rather than invalid JSON. `status` classifies
the next-review date itself as `due-soon` and only later dates as `overdue`.
`due-soon` is informational under the publication gate. The gate blocks
overdue, drifted, artifact-identity-changed, draft, in-review, superseded, and
retired records, so historical portfolios must be curated for publication.
`check` validates structure, artifact drift, and portfolio relationships without
inventing an `--as-of` date. Check/status JSON is always an array, even for one
record. Repeat `--record` for deterministic portfolio status and
supersession-cycle validation. `queue` emits `forge.policy-lifecycle-queue/1`
JSON grouped by owner and next-review date. A transition entering `in-review`
can preserve bounded, sorted PRD-057 reasons with repeatable
`--impact-finding-id` values; duplicate flags are collapsed. Those IDs are
included in status and queue output. `attest` emits deterministic
`forge.policy-approval-attestation/1` JSON containing the approved event,
declared assertions, approval policy, exact fingerprints, and review date. It
does not sign anything and refuses records that are not currently approved or
whose approved bytes have drifted. Exit `0` means valid under the selected gate,
exit `1` means valid but action is required, and exit `2` means the record,
artifacts, portfolio, or transition is invalid.

Event IDs bind policy metadata, parties, approval rules, review configuration,
and transition evidence. Direct edits to those surrounding fields invalidate
existing `/2` history. Migration retains each `/1` event ID as
`legacy_event_id` and calculates a new context-bound ID without modifying the
input record. Transition writes use durable atomic replacement and a final
byte comparison, but portable filesystems do not provide conditional rename;
concurrent lifecycle transitions or external writers therefore require
external serialization. FORGE does not claim multi-writer transaction safety.

### 3.11 `applicability` — Declare Scope and Analyze Policy Gaps

Applicability analysis consumes one closed `forge.applicability/1` manifest,
one exact local Catalog or Profile baseline, and zero or more PRD 055 Mapping
Collections. Omitted controls remain `under-review`. FORGE never derives scope
from mappings and never labels a mapped control as satisfied.

```bash
# Catalog scaffold: omitted decisions make every inventoried control under-review
forge applicability init --framework framework-catalog.json \
  --output applicability.json

# Profile scaffold: caller supplies the resolved Catalog used for inventory
forge applicability init --framework framework-profile.json \
  --resolved-catalog framework-resolved.json \
  --output applicability.json

# Text, JSON, and static HTML share the same deterministic report model
forge applicability analyze --manifest applicability.json --format text
forge applicability analyze --manifest applicability.json --format json \
  --output applicability-report.json
forge applicability analyze --manifest applicability.json --format html \
  --output applicability-report.html
```

The scaffold records resource type, relative label, raw SHA-256, root UUID,
metadata version, OSCAL version, resolved-Catalog hash when applicable, and a
sorted control inventory. To keep large valid inventories bounded, the scaffold
starts with an empty `decisions` array; the manifest contract classifies every
omitted control as `under-review`. For a Profile, a reviewer must change
`resolved_catalog_attestation` to `true` only after confirming the companion.
Each explicit decision uses one of `applicable`, `not-applicable`, `deferred`,
or `under-review`. Applicable decisions require reviewer and review time;
exclusions additionally require rationale; deferrals additionally require a
`YYYY-MM-DD` revisit date. Explicit `under-review` records may carry only an
optional assignee (`reviewer_key`) and note.

Every control receives exactly one primary classification:
`applicable-mapped`, `applicable-reviewed-no-relationship`,
`applicable-unmapped`, `not-applicable`, `deferred`, or `under-review`. Positive
mapping participation wins when a control has both positive and explicit
`no-relationship` edges, while both edge counts remain visible. The JSON/HTML
review queue uses stable reason codes and retains owner, revisit date, and
policy-source metadata.

Control and statement subjects retain PRD 055 granularity. A relationship that
targets only a statement is validated as part of its Mapping Collection but
does not implicitly classify the statement's parent control as mapped or
reviewed-no-relationship. Authors must map the control explicitly when that is
the reviewed conclusion.

Detail filters never change framework-wide totals. Persisted applicability
reports must reconcile the six classification counts with the inventory total;
invalid or overflowing sums are rejected. Unfiltered category counts match the
visible inventory exactly, while filtered details may be a subset. Workspace
currentness additionally requires comparison with recomputed analysis.

For example, select a subset without changing the complete denominator:

```bash
forge applicability analyze --manifest applicability.json --format json \
  --group access-control \
  --control-prefix ac- \
  --state applicable-unmapped \
  --reviewer scope-reviewer \
  --policy-source policy-catalog.json
```

The explicit CI gates are `never` (default), `applicable-unmapped`,
`any-review-action`, and `overdue-deferred`. The overdue gate requires a caller-
supplied date so repeated runs do not depend on the wall clock:

```bash
forge applicability analyze --manifest applicability.json \
  --fail-on overdue-deferred --as-of 2026-10-01
```

The gate treats a deferral as overdue only when its `revisit_date` is strictly
earlier than `--as-of`. It does not fire on the revisit date itself.

Analysis is offline and bounded. Unknown/duplicate manifest keys, unsupported
versions, stale inventories or subject fingerprints, conflicting decisions or
relationships, contradictory policy-source identities, duplicate or unstable
Mapping UUIDs, undeclared map reviewers, mismatched framework sides, absolute
local report paths, and output/input aliases fail before any report is written. Exit `0`
means valid analysis without the selected gate condition, exit `1` means a
valid report requires human review, and exit `2` means analysis failed.
Artifact and Mapping Collection paths are trusted local file instructions
resolved from the manifest directory and may intentionally contain `..`; the
manifest is not a filesystem-confinement boundary. Output parent directories
must already exist.

### 3.12 `framework impact` — Review a Framework Revision

The command compares two caller-supplied OSCAL Catalogs or two attested
Profile-plus-resolved-Catalog pairs. It traverses optional PRD 055 Mapping
Collections built against the exact old framework and can incorporate one raw
PRD 056 applicability manifest as the authoritative prior scope and gap state.
It does not fetch framework content, infer renamed controls, copy prose into the
report, or modify dependencies.

```json
{
  "schema_version": "forge.framework-impact/1",
  "old": {
    "type": "catalog",
    "artifact": "framework-v1.json",
    "expected_sha256": "<64 lowercase hex characters>",
    "root_uuid": "<catalog lineage UUID>",
    "document_version": "1.0.0",
    "oscal_version": "1.2.3"
  },
  "new": {
    "type": "catalog",
    "artifact": "framework-v2.json",
    "expected_sha256": "<64 lowercase hex characters>",
    "root_uuid": "<new Catalog root UUID>",
    "document_version": "2.0.0",
    "oscal_version": "1.2.3"
  },
  "mapping_collections": [
    {"artifact": "policy-mapping.json", "framework_role": "target"}
  ],
  "applicability_manifest": "applicability.json",
  "successor_map": "successor-map.json",
  "prior_report": "prior-impact-report.json",
  "disposition_file": "impact-dispositions.json"
}
```

For a Profile, each `old` and `new` object additionally requires
`resolved_catalog`, `resolved_catalog_attestation: true`, and
`expected_resolved_catalog_sha256`. The raw Profile provides resource identity;
the attested companion Catalog provides the control inventory and canonical
control hashes.

```bash
forge framework impact --manifest framework-impact.json \
  --format json --output framework-impact-report.json

# Fail on informational findings too; the default threshold is review-required
forge framework impact --manifest framework-impact.json \
  --format json --fail-on any

# Emit deterministic GitHub workflow commands without posting them
forge framework impact --manifest framework-impact.json --format github

# Produce deterministic review artifacts from the same report model
forge framework impact --manifest framework-impact.json --format markdown \
  --output framework-impact.md
forge framework impact --manifest framework-impact.json --format html \
  --output framework-impact.html

# Narrow displayed detail with exact-match filters
forge framework impact --manifest framework-impact.json --format json \
  --group access-control --decision-state applicable \
  --policy-source policies/access-control.json \
  --priority review-required --owner security-governance
```

Control identity is exact: a similar new ID is `added` while the missing old ID
is `removed`. Same-ID canonical subtree changes are `content-changed`; stable ID
and stable fingerprint are `unchanged`. Mapping inputs must carry the exact old
resource evidence and subject hashes or the complete analysis fails with exit
`2` and no report. If `applicability_manifest` is present, its old resource and
complete Mapping Collection set must match the impact manifest's target-side
portfolio exactly. Applicability findings retain the prior six-state gap
classification, reviewer owner, and policy-source labels. Removed mapped
controls are blocking; changed mapped controls, added controls, and affected
applicability decisions require review. Other unmapped removals and changes are
informational.

`successor_map` accepts the same closed `forge.successor-map/1` contract used by
`forge migrate`. Declared one-to-one successors, one-to-many splits, and
many-to-one merges become `identity-migrated` groups with sorted old/new control
IDs, hashes, cardinality, reviewer, timestamp, and rationale. A declaration is
evidence supplied by the caller, not an authenticated approval; FORGE never
infers a successor.

Durable review state requires `prior_report` and `disposition_file` together.
The closed `forge.framework-impact-dispositions/1` file binds itself to the
prior report's SHA-256 and assigns each prior finding exactly one of `resolved`,
`accepted-risk`, or `still-open`, with reviewer, time, and rationale. Raw current
findings and priority totals remain unchanged. Resolved and accepted-risk
findings no longer fire the selected gate; still-open and undispositioned
findings do. Dispositions whose findings are absent from the current raw result
remain visible as `prior_only_dispositions` for audit continuity.

Detail filters are available as `--group`, `--decision-state`,
`--policy-source`, `--priority`, and `--owner`. Multiple filters use AND
semantics. Group matching uses the deterministic old/new group-ID union for a
finding, including every side of an identity migration; decision state is the
validated prior PRD 056 applicability decision rather than its derived gap
classification. Policy source and owner use exact string matching. Filters
narrow the rendered review queue only: framework-wide change and priority
totals, raw disposition accounting, and gate evaluation still cover the full
validated analysis, including findings omitted from display.

The default `review-required` gate exits `1` for blocking or review-required
findings. `blocking` is a weaker threshold and `any` is stricter; there is no
disabled gate. Exit `0` means the selected threshold did not fire, not that the
organization remains compliant. Stable finding IDs can be carried into PRD 058
policy review history:

```bash
forge lifecycle transition --record lifecycle.json --to in-review \
  --actor reviewer --role reviewer --at 2026-08-25T14:00:00Z \
  --rationale "Framework impact requires review." \
  --impact-finding-id <finding-uuid> --apply
```

Markdown and static HTML escape caller-controlled table and markup content and
contain no scripts, remote assets, or runtime timestamps. The GitHub format
emits `error`, `warning`, and `notice` workflow commands for blocking,
review-required, and informational findings respectively. It escapes
workflow-command data, includes no framework prose or absolute paths, and does
not call GitHub or mutate repository state.

### 3.12.1 `migrate --successor-map` — Declare Reviewed Policy Identity

```bash
forge migrate old-policy.md new-policy.md --format json \
  --successor-map successor-map.json
```

The optional closed `forge.successor-map/1` JSON file uses the same
`successor`, `split`, and `merge` cardinalities accepted by framework impact.
Every declaration requires non-empty `approved_by`, RFC 3339 `approved_at`, and
`rationale`. Conflicting, reused, self-mapped, absent, malformed, oversized, or
unsafe declarations fail with exit `2` before output. Valid declarations remain
read-only and appear as declared—not authenticated—migration outcomes.

### 3.13 `assessment results` — Package Human Assessment Judgments

Create a context-bound draft, then add explicit assessor-authored observations,
findings, risks, provenance, and relationships:

```bash
forge assessment results init --assessment-plan assessment-plan.json \
  --ssp ssp.json --profile profile.json --catalog catalog.json \
  --output assessment-results-manifest.json

forge assessment results build --manifest assessment-results-manifest.json \
  --output assessment-results.json \
  --report assessment-results-review.json --report-format json
```

The optional `--evidence-index` input is an identity-only PRD 060
`forge.linkage-index/1` artifact. FORGE copies evidence keys and hashes, not
content, and does not treat a link as sufficient evidence. Use `--baseline` to
report stable-identity revision impacts; the default `--fail-on any` exits `1`
when review actions exist. The baseline must contain exactly one result epoch;
a plural baseline exits `2` before publication, including with `--fail-on never`.
Static HTML is available with `--report-format html`.
The build remains local, JSON-only, deterministic, and validated against the
pinned official OSCAL 1.2.3 Assessment Results schema. It records declared
judgments without authenticating assessors or inferring compliance,
effectiveness, certification, or remediation ownership. See
[OSCAL Assessment Results](assessment-results.md) for the complete contract and
trust boundaries.

#### Append a sealed assessment epoch

```bash
forge assessment results append-epoch \
  --request epoch-request.json \
  --output next-assessment-results.json \
  --report next-epoch-report.json
```

Both output names must be new portable JSON filenames on the request's parent.
The saved report is always JSON. Optional `--view-format json|text|html` selects a
complete stdout view after both files are published; omitted view leaves stdout
empty. A valid append returns `1` for descriptive review actions, or `0` with
`--fail-on never`; errors return `2` and a late error can leave complete files.
This first profile requires unchanged context/actors and sealed ordered windows.
See [sealed assessment epochs](assessment-results-epochs.md) for complete risk
classification, continuity, bounds, source rebinding and remaining S-4 gates.

#### Export explicitly reviewed risks to authored work

Start with an empty scaffold from `forge poam init`. Supply a closed
`forge.poam-risk-authoring/1` request containing the complete authored workflow
and exact reviewed risk references:

```bash
forge assessment results export-poam \
  --scaffold /absolute/bundle/empty-plan.json \
  --authoring requests/reviewed-risks.json \
  --as-of 2026-10-04 --output plan-workflow.json
```

`--authoring` is relative to the scaffold's parent. `--output` is a new single
`.json` filename in that same directory; omit it for stdout. You supply every
owner, date, milestone and planned initial history event. Export exits `0` for
complete admitted output and `2` for invalid input or output failure; subsequent
POA&M check/build retain their separate `0`/`1`/`2` meanings. See
[reviewed-risk authoring](assessment-results-reviewed-risks.md) for source
inventory selection, platform publication limits, consumption and remaining
acceptance gates.

### 3.14 `poam` — Check Sources and Authored Remediation Work

`forge poam init` emits an unselected `forge.poam/1` scaffold from one exact
Assessment Results result UUID and stable key plus its local Assessment Plan,
SSP, Profile and Catalog companions. Document metadata is explicit; no work,
actors, remediation ownership, deadlines or status assertions are inferred.

```bash
forge poam check --manifest ./assessment-bundle/poam.json \
  --source-only --format json
forge validate native-poam.json --schema-type poam
```

The first command verifies the captured source identities and emits a complete
finding/risk inventory. It requires `--source-only`, refuses nonempty items,
roles and parties. The second command performs
native POA&M structural validation and the system-identity assembly check; it
does not establish source binding or remediation workflow acceptance. POA&M
format conversion through `forge export`, source trace and generic diff remain
unsupported.
See [POA&M source foundation](poam-foundation.md) for the exact inputs, bounded
source profile, publication limits and remaining gates.

An explicitly authored plan uses a separate workflow scope:

```bash
forge poam check --manifest ./assessment-bundle/authored.json \
  --workflow --as-of 2026-02-06 --due-soon-days 7 --format json
forge poam build --manifest ./assessment-bundle/authored.json \
  --as-of 2026-02-06 --output native-poam.json --report schedule.json
```

Every item selects exact source finding/risk identities and supplies declared
owners, dates, ordered milestones and attributed append-only history. The as-of
date is required. Exit 0 means valid ungated state, 1 a valid schedule action,
and 2 invalid input or publication. Completion/risk acceptance is refused. Both
new output filenames use the manifest directory so relative native source links
retain their base. Artifact/report publication is separate and may leave a new
artifact if the later report fails. See [authored POA&M commands](poam-cli-workflow.md)
for explicit baselines, bounds, portable destinations and remaining acceptance.

Compare a prior declaration separately, including revisions refused by native
production:

```bash
forge poam baseline --manifest ./assessment-bundle/authored.json \
  --baseline prior.json --as-of 2026-02-06 --format json
```

Exit 0 means a complete comparison needs no review; 1 means its complete report
needs review; 2 means invalid input/output. Prior source tuples are checked against
the actual current source capture, without reconstructing historical freshness.
See [baseline comparison](poam-baseline-comparison.md) for complete denominators,
explicit reopening declarations, privacy and admission limits.

Review explicitly paired native plans together:

```bash
forge poam portfolio --manifest ./assessment-bundle/authored.json \
  --native native-poam.json --as-of 2026-02-06 --due-soon-days 7 \
  --output-root ./portfolio-reports --report portfolio.json --html portfolio.html
```

The output directory must already exist and both filenames must be unused. Repeat
`--manifest` and `--native` in matching order for up to 32 plans. Complete JSON
and HTML retain summed per-plan counts, null/cancelled/future rows and inert source
hrefs in their original bundle context. Outputs are separate: a later failure does
not roll back an earlier file. See [portfolio reports](poam-portfolio.md) for the
supported companion profile, stdout, dates, bounds and exit statuses.

Prepare explicitly selected local outbound intent from one current native plan:

```bash
forge poam outbound --manifest ./assessment-bundle/authored.json \
  --native native-poam.json --selection outbound-selection.json \
  --as-of 2026-02-06 --due-soon-days 7 --output-root ./outbound-reports \
  --report change-set.json
```

The closed selection chooses each item and `create`, `update`, or `close-request`
intent explicitly. Native and selection inputs are single JSON filenames beside
the manifest. The output directory must exist and the report filename must be new.
Omit `--report` for stdout. Valid local handoff exits 0 regardless of schedule;
invalid input/output exits 2. Eight actual originals are rechecked before output,
and remote apply authority remains false. See [local outbound change sets](poam-outbound.md)
for the selection format, complete native matching, minimized fields and bounds.

Inspect local evidence with `forge poam evidence --manifest ./assessment-bundle/authored.json --links review/evidence-links.json --as-of 2026-10-04`. Add `--report inspection.json` to publish a new report beside the plan on Linux or macOS. See [local evidence inspection](poam-evidence-inspection.md) for explicit assertion bindings, complete counts and unchanged closure gates.

### 3.15 `review` — Coordinate Portable Local Re-review

Portable review commands are an unreleased F12 implementation candidate and are
not included in the published v1.1.0 binary.

Use `forge review init` for selected Mapping assertions or explicit applicability
decisions in complete recorded Approved/current native closures. `respond` binds
to the exact recorded queue; `merge` and `status` check the complete current source
closure. Reviewer keys, roles, authors and times are asserted. A satisfied quorum
does not approve or modify the domain artifact.

`export-html` renders a recorded disposition bundle; `export-notifications`
exports recorded queue assignments without sending them or checking current
native sources. `supersede` creates a separate immutable old/new lineage companion
from explicit links and a complete native Impact comparison. The old queue stays
historical, and responses and quorum do not transfer to the new queue.

See the [portable review queue guide](review-queues.md) for the private source,
policy, Impact and link formats, exact commands, source requirements, sharing
limits and remaining acceptance gates. Inputs and outputs are explicit paths
under an absolute project root; output files must be new. Publication uses the
Linux/macOS no-replace publisher and refuses on Windows. Identifiers and hashes
can still contain sensitive project metadata.

Lifecycle re-review uses a separate [closed `/2` exchange candidate](lifecycle-review-exchange.md).
Its implementation candidate registers `forge review lifecycle
init/respond/merge/status`; the linked guide gives exact flags for use after
candidate integration. This section's seven commands retain their existing
Mapping/applicability scope; no review disposition changes native Lifecycle state.

## 4. Global Options

```bash
# Verbose: show each pipeline stage on stderr
forge -v convert policy.md --strategy catalog --format json

# Quiet: suppress all non-essential output (OSCAL artifact only on stdout)
forge -q convert policy.md --strategy catalog --format json
```

## 5. The FORGE Pipeline

Every `convert` execution runs through these stages:

```
Ingest → Parse → Extract → Assemble → Atomize → Assign IDs → Map to OSCAL → Serialize → Validate
```

1. **Ingest** — Read and validate the input file
2. **Parse** — Extract sections, clauses, and structure from Markdown
3. **Extract** — Pull out citations, modalities, and parameters
4. **Assemble** — Build the internal PolicyDocument model
5. **Atomize** — Split compound requirements into individual controls
6. **Assign IDs** — Generate deterministic UUID v5 identifiers
7. **Map to OSCAL** — Build OSCAL Catalog or Component Definition, embedding trace links
8. **Serialize** — Convert to JSON, XML, or YAML
9. **Validate** — Run JSON schema + semantic validation

Use `-v` to watch each stage execute.

## 6. End-to-End Walkthrough

Here is a complete workflow from a policy document to validated, cross-format OSCAL artifacts with diff and trace.

### Step 1: Write your policy

```bash
cat > my-policy.md << 'EOF'
---
title: "My Security Policy"
version: "1.0.0"
author: "Engineering"
date: "2026-05-01"
---

# Access Control

## Authentication

- All users must authenticate with multi-factor authentication
- Service accounts must use certificate-based authentication
- Failed login attempts must be limited to 5 before account lockout

## Authorization

- Access must be granted on a least-privilege basis
- Privileged access must require explicit approval

# Data Protection

## Encryption Standards

- Data at rest must be encrypted using AES-256 or stronger
- Data in transit must be protected with TLS 1.3
- Encryption keys must be rotated every 180 days
EOF
```

### Step 2: Convert to OSCAL Catalog

```bash
forge convert my-policy.md --strategy catalog --format json --output my-catalog.json
```

Output: `my-catalog.json` — an OSCAL v1.2.3 Catalog with groups for "Access Control" and "Data Protection", each containing atomized controls with stable UUIDs.

### Step 3: Validate the output

```bash
forge validate my-catalog.json
```

Expected output: `Valid: catalog artifact passes all validation.`

### Step 4: Export to multiple formats

```bash
forge export my-catalog.json --format xml --output my-catalog.xml
forge export my-catalog.json --format yaml --output my-catalog.yaml
```

### Step 5: Round-trip validation (requires oscal-cli)

```bash
forge validate my-catalog.json --round-trip
```

### Step 6: Generate a Profile from the Catalog

```bash
forge profile --catalog my-catalog.json \
  --include "POL-AC-001,POL-AC-002,POL-DP-001" \
  --output my-profile.json
```

### Step 7: Trace back to source

```bash
forge trace my-catalog.json --source my-policy.md
```

### Step 8: Compare versions after policy changes

```bash
# Edit the policy, bump the version
cp my-policy.md my-policy-v2.md
# ... make changes to my-policy-v2.md ...

forge convert my-policy-v2.md --strategy catalog --format json --output my-catalog-v2.json
forge diff my-catalog.json my-catalog-v2.json
```

### Step 9: Batch conversion (multiple policies)

```bash
mkdir -p output
forge convert example_data/POL-0[1-3]*.md --strategy catalog --format json --output output/ --jobs 4
```

## 7. Exit Codes

| Code | Maintained error family or completed action |
|------|---------------------------------------------|
| 0 | Success under the selected command/gate |
| 1 | Input/I/O, serialization or external execution failure; also completed diff, drift or review-action states |
| 2 | Parse/structure, required-argument or domain build/analysis failure |
| 3 | Schema validation or configuration failure |
| 4 | External dependency unavailable or nonfunctional |

CLI argument parsing also exits `2` for rejected usage. Use each command's
documented gate/action semantics before treating a nonzero code as a failed
analysis. The [maintained error mapping](../src/error.rs) defines these families;
there is no generic exit `5`.

## 8. Quality Gates

Run the same checks as CI locally:

```bash
./scripts/ci-local.sh
```

Install the pre-commit hook:

```bash
./scripts/install-hooks.sh
```

## Local workspace index bundles

For an explicit `forge.workspace.json`, the unreleased **Trace & Reports**
browser panel and maintained Python client's `bundle_preview()` and
`verify_bundle(bundle)` inspect complete registrations and compare supplied
fingerprints in either session mode. Preview metadata and acknowledge its
sensitivity before requesting the browser's local metadata JSON download. This
writes no project file or server receipt, includes no source content and
establishes no approval.

Choose a metadata JSON file, then explicitly compare registered fingerprints.
Its original bytes plus the 11-byte wrapper must fit 1 MiB, so the selected file
may be at most 1,048,565 bytes. The server rejects malformed/unsupported input
without opening supplied unregistered paths. Missing/empty index, subset
agreement/current-only entries, whole-index equality and observed validation
state remain separate. Queries keep the complete expected denominator up to
1,000 within byte limits and prepare no 100-input effect.

See [Workspace index bundles](workspace-index-bundles.md) for the Python example,
normalized index hash versus original resource hashes and safe errors.
Negotiated API2.2, 2.3 or 2.4 sessions offer [confirmed metadata receipts and index
replacement](workspace-bundle-receipts.md); API2.3 and 2.4 admit [opt-in exact-source
export and complete source restore](workspace-source-bundles.md). The current captured API2.4 source adds
[bounded staged part transfer](workspace-staged-source-bundles.md). Exact-tree
hosted API and Chrome staged-source development workflows have scoped successful
receipts; separate headless and dependency-audit jobs failed. This does not
establish full S-6, network-denial, platform or human acceptance. These writes require
preparation and a separate confirmation, with distinct known-ID recovery for
source restore. Full S-6 capacity, larger transfer and platform/crash/batch
qualification, human acceptance and the final integrated documentation review
remain open.

## Further Reading

- [README.md](../README.md) — project overview and quick start
- [Portable review queues](review-queues.md) — asserted review policy, immutable responses, recorded exports and explicit queue supersession
- [Lifecycle review exchange candidate](lifecycle-review-exchange.md) — separate `/2` formats, candidate CLI commands and qualification limits
- [Evidence and Implementation Linking](evidence-linkage.md) — exact subject/evidence metadata linkage, freshness, privacy, and baseline contracts
- [Evidence overlays](evidence-overlays.md) — new Catalog/Component JSON documents, complete source preservation and generated metadata boundaries
- [Contributing Guide](../CONTRIBUTING.md) — development setup and PR process
- [Architecture Guide](architecture.md) — pipeline details and crate structure
- `example_data/` — 25 sample policies
- `tests/fixtures/` — test fixtures for all subcommands
