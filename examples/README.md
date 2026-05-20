# FORGE Examples

This directory contains three worked examples that demonstrate the full FORGE pipeline — from human-written Markdown policy documents to validated OSCAL artifacts. Each example increases in scope and complexity, showing progressively more of the FORGE feature set.

## Comparison

| Example | Key Features | Controls (Catalog) | Controls (Profile) | Components | OSCAL Artifacts |
|---|---|---|---|---|---|
| [simple-access-control](./simple-access-control/) | Minimal policy, catalog+profile generation, `forge export` (XML/YAML) | 9 | 5 | 1 (policy) | catalog, profile |
| [component-based](./component-based/) | Multi-component architecture, component definition, SSP generation, 25 requirements | 17 | 10 | 2 (web-application, database) | catalog, profile, component-definition, ssp |
| [full-compliance-package](./full-compliance-package/) | Complete pipeline: catalog→profile→component→assessment-plan, 26 controls, 16 sections | 26 | 15 | 1 (policy) | catalog, profile, component-definition, assessment-plan |

## Which Example Should I Start With?

**Start with [simple-access-control](./simple-access-control/)** if:
- You're new to FORGE or OSCAL
- You want to see the core `forge convert` and `forge validate` workflow in under 5 minutes
- You need a minimal, copy-paste reproducible example with 4 NIST-style controls

**Move to [component-based](./component-based/)** if:
- You're modeling a real system with multiple software components (e.g. web app + database)
- You need a System Security Plan (SSP) alongside the Component Definition
- You want to see how control requirements map to specific system components

**Explore the [full-compliance-package](./full-compliance-package/)** if:
- You're evaluating FORGE for production use
- You need the complete four-artifact pipeline: Catalog → Profile → Component Definition → Assessment Plan
- You're building a compliance program with dozens of controls across multiple NIST families
- You want to understand the end-to-end traceability chain from policy text to assessment scope

All examples assume you have `forge` installed and on your `PATH`. See the [project README](../README.md) for installation instructions.
