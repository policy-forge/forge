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

Download the latest binary for your platform from GitHub Releases. Each release includes SHA-256 checksums and SLSA Level 3 provenance attestation.

Verify your installation:

```bash
forge --help
```

You should see seven subcommands: `convert`, `export`, `validate`, `resolve`, `trace`, `diff`, and `profile`.

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
- **Parameter extraction** — turns prose thresholds (e.g., "12 characters", "30 minutes") into machine-enforceable parameters
- **Citation extraction** — URLs and references become OSCAL back-matter resources
- **Stable identifiers** — UUID v5 generation ensures every control has a persistent identity across re-conversions

25 sample policies are included in `example_data/` covering topics from acceptable use to incident response.

PDF and DOCX source documents are also accepted directly — Word heading and list styles are mapped to the document model automatically. For other formats, convert to Markdown first using [pandoc](https://pandoc.org/) or [markitdown](https://github.com/microsoft/markitdown).

## 3. The Seven CLI Subcommands

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

Converts an existing OSCAL artifact between JSON, XML, and YAML. Auto-detects the input format from the file extension.

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

Input OSCAL model type (Catalog vs Component Definition) is auto-detected from the document structure. The pipeline validates the artifact against OSCAL JSON schemas before serializing to the target format.

### 3.3 `validate` — Schema and Semantic Validation

Validates an OSCAL JSON artifact against the OSCAL v1.2.0 JSON schema with semantic checks.

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

### 3.7 `profile` — Generate OSCAL Profile from Catalog

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

Output: `my-catalog.json` — an OSCAL v1.2.0 Catalog with groups for "Access Control" and "Data Protection", each containing atomized controls with stable UUIDs.

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

| Code | Meaning |
|------|---------|
| 0    | Success |
| 1    | Validation failure or diff found changes |
| 2    | File not found |
| 3    | Invalid argument |
| 4    | oscal-cli not found (resolve/round-trip) |
| 5    | oscal-cli execution failure |

## 8. Quality Gates

Run the same checks as CI locally:

```bash
./scripts/ci-local.sh
```

Install the pre-commit hook:

```bash
./scripts/install-hooks.sh
```

## Further Reading

- [README.md](../README.md) — project overview and quick start
- [Contributing Guide](CONTRIBUTING.md) — development setup and PR process
- [Architecture Guide](architecture.md) — pipeline details and crate structure
- `example_data/` — 25 sample policies
- `tests/fixtures/` — test fixtures for all subcommands
