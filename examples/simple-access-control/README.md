# Simple Access Control Policy — FORGE Example

This example demonstrates the end-to-end FORGE workflow: writing a minimal access control policy in Markdown, converting it to OSCAL artifacts, generating a baseline profile, and validating the outputs.

## Policy Overview

The policy (`policy.md`) defines four NIST SP 800-53-style access controls:

| Control | Description |
|---------|-------------|
| AC-1 | Access Control Policy and Procedures — develop, document, and review the policy |
| AC-2 | Account Management — manage system accounts, identify users, require approvals |
| AC-3 | Access Enforcement — enforce approved authorizations for logical access and information flow |
| IA-1 | Identification and Authentication Policy — require identification before granting access |

FORGE processes this Markdown document through a nine-stage pipeline (Ingest → Parse → Extract → Assemble → Atomize → Assign IDs → Map to OSCAL → Serialize → Validate) to produce machine-readable OSCAL artifacts.

## Prerequisites

- [Rust](https://rustup.rs) 1.93.0 or later
- The `forge` CLI built from source or installed via `cargo install forge`

## Reproducing This Example

### Step 1: Generate the OSCAL Catalog

Convert the Markdown policy into an OSCAL Catalog (JSON):

```bash
forge convert policy.md --strategy catalog --format json --output output/catalog.json
```

**What this does:** FORGE reads `policy.md`, extracts headings as OSCAL groups, list items as controls, and produces a fully structured OSCAL v1.2.0 Catalog. Each control gets a deterministic UUID v5 identifier (e.g., `POL-AC-001`) and source traceability links back to the original Markdown file.

**Output:** `output/catalog.json` — an OSCAL Catalog containing 9 atomized controls organized under the "Access Control" group.

### Step 2: Validate the Catalog

Verify the generated catalog against the OSCAL v1.2.0 JSON schema:

```bash
forge validate output/catalog.json
```

Expected output:
```
Valid: catalog artifact passes all validation.
```

### Step 3: Generate an OSCAL Profile

Select a subset of controls from the catalog to create a baseline profile:

```bash
forge profile --catalog output/catalog.json \
  --include "POL-AC-001,POL-AC-002,POL-AC-003,POL-AC-004,POL-AC-005" \
  --output output/profile.json
```

**What this does:** The `profile` command creates an OSCAL Profile by selecting specific controls from the source Catalog. Profiles are used to define which controls an organization actually implements from a larger catalog.

**Output:** `output/profile.json` — an OSCAL Profile selecting 5 controls from the catalog.

### Step 4: Validate the Profile

```bash
forge validate output/profile.json
```

Expected output:
```
Valid: profile artifact passes all validation.
```

### Step 5: Export to Other Formats

Export the catalog to other formats:

```bash
forge export output/catalog.json --format xml --output output/catalog.xml
forge export output/catalog.json --format yaml --output output/catalog.yaml
```

## File Structure

```
examples/simple-access-control/
├── policy.md           # Source Markdown policy (4 controls)
├── README.md           # This walkthrough
└── output/
    ├── catalog.json    # OSCAL Catalog (validated)
    └── profile.json    # OSCAL Profile (validated)
```

## Why FORGE Produces These Outputs

- **Catalog**: The catalog is the machine-readable representation of your entire policy. Every heading becomes a group, every bullet becomes a control, and compound requirements are atomized into individual enforceable statements. This is what downstream tools (compliance scanners, AI agents, CI/CD gates) consume.

- **Profile**: Organizations rarely implement every control from a catalog. A profile selects a subset — your "baseline." FORGE generates the profile from the catalog so you can iteratively refine which controls matter for your environment.

- **Deterministic IDs**: FORGE uses UUID v5 to generate stable identifiers. Re-converting the same policy text always produces the same IDs, enabling change detection and version comparison via `forge diff`.

- **Traceability**: Every OSCAL element carries `source-section`, `source-line`, and `source-file` properties linking it back to the original Markdown. This is critical for audits — you can always show the human-readable policy that produced a given machine rule.
