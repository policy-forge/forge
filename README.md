FORGE 🦀

Framework for OSCAL Risk & Governance Execution

FORGE is a high-performance Rust CLI designed for the Agent-Native software era. It bridges the gap between human-written security policies and autonomous execution by converting Markdown governance into OSCAL (Open Security Controls Assessment Language)—the industry standard for machine-readable compliance.

🚀 Why FORGE?

In the world of Agentic AI, natural language documentation is a liability. Agents suffer from "semantic ambiguity," leading to hallucinations and inconsistent security enforcement. FORGE "forges" abstract policy into deterministic, schema-validated artifacts that provide AI agents with a Shared Truth Layer.

By providing a high-fidelity, machine-navigable roadmap of a system's rules, FORGE allows agents to not just write code, but to understand the guardrails they must operate within.

🛠️ Key Architectural Pillars

1. High-Fidelity Machine Readability

FORGE ingest human-centric Markdown and produces structured OSCAL Catalogs and Component Definitions.

Zero-Shot Success: Reduces token waste by providing agents with deterministic schemas rather than ambiguous prose.

Requirement Atomization: Automatically splits compound "Must X and Must Y" statements into individual, addressable controls.

2. Deterministic Agentic Guardrails

In agentic coding, an agent often has the power to modify its environment. FORGE creates the Guardrail Layer:

Stable Identifiers: UUID v5 generation ensures that every security control has a persistent identity, allowing agents to track compliance state across sessions.

Traceability: Source-to-OSCAL mapping ensures every machine rule is linked back to the original policy intent.

3. Agentic Interoperability & Enforcement

FORGE acts as a translation layer for the modern security stack. It enables the transition from "Policy-as-Prose" to Policy-as-Code:

Tooling Integration: Compatible with Open Policy Agent (OPA), GitHub Advanced Security, and standard CI/CD scanners.

MCP Native: Designed to feed into the Model Context Protocol (MCP), allowing agents to query system governance as easily as they query a database.

✨ Features

- **Markdown to OSCAL** — Convert policy documents into OSCAL Catalogs or Component Definitions
- **Multi-format output** — JSON, XML, and YAML with round-trip fidelity between all three
- **Schema validation** — Validate artifacts against OSCAL v1.2.0 JSON schemas with semantic checks
- **Format conversion** — Export existing OSCAL artifacts between JSON, XML, and YAML
- **Requirement atomization** — Automatically split compound policy statements into individual controls
- **Deterministic IDs** — UUID v5 generation ensures stable identifiers across re-conversions
- **Citation extraction** — URLs and references extracted into OSCAL back-matter resources
- **Traceability** — Source-to-OSCAL element mapping embedded as provenance metadata
- **Zero network dependencies** — Reads and writes local files only

🚦 Quick Start

```bash
# Install (requires Rust 1.93.0+)
git clone https://github.com/policy-forge/forge.git
cd forge
cargo build --release

# Convert a policy to an OSCAL Catalog (JSON)
./target/release/forge convert tests/fixtures/sample_policy.md --strategy catalog --format json

# Convert to an OSCAL Component Definition
./target/release/forge convert tests/fixtures/sample_policy.md --strategy component --format json

# Validate a generated OSCAL artifact
./target/release/forge validate catalog.json
```

## Usage

### Convert

Convert a Markdown policy document into an OSCAL artifact.

```bash
# OSCAL Catalog (groups, controls, statements)
forge convert policy.md --strategy catalog --format json

# OSCAL Component Definition (implemented requirements)
forge convert policy.md --strategy component --format json

# With a source profile reference for component strategy
forge convert policy.md --strategy component --source-profile baseline.json --format json

# Output as XML or YAML
forge convert policy.md --strategy catalog --format xml
forge convert policy.md --strategy catalog --format yaml

# Write to a file instead of stdout
forge convert policy.md --strategy catalog --format json --output catalog.json

# Override max input file size (default: 10 MB)
forge convert large-policy.md --strategy catalog --format json --max-size 20
```
### Export

Convert an existing OSCAL artifact between formats. Auto-detects the input format from the file extension.

```bash
# JSON to XML
forge export catalog.json --format xml

# XML to YAML
forge export catalog.xml --format yaml

# YAML to JSON, written to a file
forge export catalog.yaml --format json --output catalog.json
```

### Validate

Validate an OSCAL artifact against the OSCAL v1.2.0 JSON schema. Auto-detects the model type (Catalog or Component Definition) from the document structure.

```bash
# Validate with human-readable output
forge validate catalog.json

# Machine-parseable JSON output
forge validate catalog.json --format json

# Override auto-detected model type
forge validate artifact.json --schema-type catalog
```

### Global Options

```bash
# Verbose: show pipeline stage information on stderr
forge -v convert policy.md --strategy catalog --format json

# Quiet: suppress all non-essential output
forge -q convert policy.md --strategy catalog --format json
```

## Input Format

FORGE accepts Markdown files (`.md` / `.markdown`) with optional YAML frontmatter:

```markdown
---
title: "Access Control Policy"
version: "2.0"
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
```

Headings become OSCAL groups and controls. List items, tables, and paragraphs become control statements. Compound requirements like "Systems must X and must Y" are automatically split into atomic statements.

For other document formats (PDF, DOCX), convert to Markdown first using tools like [pandoc](https://pandoc.org/) or [markitdown](https://github.com/microsoft/markitdown).

25 sample policies are included in `example_data/` covering topics from acceptable use to incident response.

Each release includes SHA-256 checksums and [SLSA Level 3](https://slsa.dev/) provenance attestation.

🏗️ How It Works: The Deterministic Pipeline

FORGE processes governance through a rigorous nine-stage pipeline:
Ingest → Parse → Extract → Assemble → Atomize → Assign IDs → Map to OSCAL → Serialize → Validate

This ensures that the output is not just "valid JSON," but a semantically accurate representation of your security intent.

🗺️ Roadmap

Completed: Phase 1 — Foundation (v0.1.0)

Core Markdown-to-OSCAL pipeline. Focus on Requirement Atomization and Deterministic UUIDs.

Current: Phase 2 — Agentic Guardrails (v0.2.0)

Normative Detection: Using Rust-based NLP to differentiate "Must/Shall" from "Should/May."

Parameter Extraction: Turning prose thresholds (e.g., "30-day rotation") into machine-enforceable parameters.

Future: Phase 3 — Ecosystem (v0.3.0+)

Traceability Reports: Mapping every code line back to an OSCAL control.

Assessment Scaffolding: Generating automated test plans for AI agents.

See `docs/FORGE_PRODUCT_ROADMAP.md` for the full 50-item sprint plan.

🤝 Contributing

FORGE is built for the community. We welcome PRs for new language bindings, MCP adapters, and enhanced semantic validators.

Policy Forge: Forging the rules that power the agents.

## License

MIT
