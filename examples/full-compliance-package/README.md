# Full Compliance Package

This example demonstrates a realistic end-to-end compliance workflow using FORGE. It transforms a human-written security policy into a complete set of OSCAL (Open Security Controls Assessment Language) artifacts, covering the full pipeline:

**Catalog → Profile → Component Definition → Assessment Plan**

## Prerequisites

- [FORGE](https://github.com/policy-forge/forge) CLI installed (`forge` on your PATH)
- A terminal with `cd` access to this directory

## Quick Start

Run the entire pipeline from this directory:

```bash
# Step 1: Generate an OSCAL Catalog from the policy
forge convert policy.md --strategy catalog --output output/cloud-security-baseline-catalog.json

# Step 2: Generate a Profile selecting a baseline of controls
forge profile --catalog output/cloud-security-baseline-catalog.json \
  --include "POL-CSBP-001,POL-CSBP-002,POL-CSBP-003,POL-CSBP-004,POL-CSBP-005,POL-CSBP-006,POL-CSBP-007,POL-CSBP-008,POL-CSBP-009,POL-CSBP-010,POL-CSBP-011,POL-CSBP-012,POL-CSBP-013,POL-CSBP-014,POL-CSBP-015" \
  --output output/cloud-security-baseline-profile.json \
  --timestamp "2026-05-18T00:00:00Z"

# Step 3: Generate a Component Definition with Assessment Plan
forge convert policy.md --strategy component \
  --source-profile output/cloud-security-baseline-profile.json \
  --output output/cloud-security-baseline-component.json \
  --import-ssp ./system-ssp.json

# Step 4: Validate all artifacts
forge validate output/cloud-security-baseline-catalog.json
forge validate output/cloud-security-baseline-profile.json
forge validate output/cloud-security-baseline-component.json
```

## Directory Structure

```
full-compliance-package/
├── policy.md                          # Source policy document (human-readable Markdown)
├── README.md                          # This walkthrough
└── output/
    ├── cloud-security-baseline-catalog.json        # OSCAL Catalog
    ├── cloud-security-baseline-profile.json        # OSCAL Profile (control baseline)
    ├── cloud-security-baseline-component.json      # OSCAL Component Definition
    └── policy-assessment-plan.json                 # OSCAL Assessment Plan
```

## Source Policy (`policy.md`)

The source policy is a **Cloud Security Baseline Policy** written in Markdown. It contains **26 individual controls** organized across **16 sections**, mapping to NIST SP 800-53 Rev. 5 control families:

| Section | NIST Family | Controls |
|---------|-------------|----------|
| Access Control Policy and Procedures | AC | Narrative |
| Account Management | AC | Narrative |
| Access Enforcement | AC | 3 numbered items |
| Audit Events | AU | 6 bullet items + narrative |
| Content of Audit Records | AU | Narrative |
| Audit Review, Analysis, and Reporting | AU | Narrative |
| Baseline Configuration | CM | Narrative |
| Least Functionality | CM | 4 numbered items |
| Identification and Authentication | IA | Narrative |
| Authenticator Management | IA | Narrative |
| Incident Handling | IR | Narrative |
| Incident Reporting | IR | Narrative |
| Boundary Protection | SC | 4 numbered items |
| Transmission Confidentiality and Integrity | SC | Narrative |
| Flaw Remediation | SI | 5 numbered items |
| System Monitoring | SI | Narrative |

Each section uses normative language (SHALL, MUST, MUST NOT) that FORGE detects to classify control strength. Numbered and bulleted list items are atomized into individual controls with stable UUIDs.

## Generated Artifacts

### 1. OSCAL Catalog (`cloud-security-baseline-catalog.json`)

The **Catalog** is the foundational artifact. It represents all controls extracted from the policy as a structured OSCAL Catalog.

**Key properties:**
- Root key: `"catalog"`
- Contains one group with 26 controls (POL-CSBP-001 through POL-CSBP-026)
- Each control has a unique ID, title, statement, and guidance parts
- Traceability properties link each control back to the source file, section, and line number
- Modality annotations (normative/advisory) classify control strength

**Purpose:** The Catalog serves as the complete control library. It is the machine-readable representation of the entire policy, suitable for control browsing, reference, and as input to downstream artifacts.

**Command used:**
```bash
forge convert policy.md --strategy catalog --output output/cloud-security-baseline-catalog.json
```

### 2. OSCAL Profile (`cloud-security-baseline-profile.json`)

The **Profile** selects a subset of controls from the Catalog to define a specific compliance baseline.

**Key properties:**
- Root key: `"profile"`
- Imports from `cloud-security-baseline-catalog.json`
- Includes 15 of 26 controls (POL-CSBP-001 through POL-CSBP-015)
- Deterministic UUID v5 based on selected control IDs

**Purpose:** Profiles define "which controls apply" for a given system or compliance framework. In real-world usage, different systems might select different subsets of the full catalog (e.g., a low-impact system vs. a high-impact system). The `--include` flag selects specific controls; `--exclude` can also be used to select all except specified controls.

**Command used:**
```bash
forge profile --catalog output/cloud-security-baseline-catalog.json \
  --include "POL-CSBP-001,...,POL-CSBP-015" \
  --output output/cloud-security-baseline-profile.json \
  --timestamp "2026-05-18T00:00:00Z"
```

### 3. OSCAL Component Definition (`cloud-security-baseline-component.json`)

The **Component Definition** describes how a specific system component (in this case, the policy document itself) implements the controls selected by the Profile.

**Key properties:**
- Root key: `"component-definition"`
- Contains one component of type `"policy"`
- References the Profile as the control source (`"source": "cloud-security-baseline-profile.json"`)
- Contains implemented-requirements with narrative descriptions for each control
- Full traceability back to source file, section, and line number

**Purpose:** Component Definitions are the bridge between abstract controls and concrete implementations. They answer "how does this system satisfy each control?" For a policy component, the implementation narrative is the control text itself. For a software component, it would describe specific technical measures.

**Command used:**
```bash
forge convert policy.md --strategy component \
  --source-profile output/cloud-security-baseline-profile.json \
  --output output/cloud-security-baseline-component.json \
  --import-ssp ./system-ssp.json
```

The `--source-profile` flag tells FORGE to use the Profile's control selections as the basis for the component definition, producing schema-valid output with proper control-implementations.

### 4. OSCAL Assessment Plan (`policy-assessment-plan.json`)

The **Assessment Plan** is a secondary artifact generated alongside the Component Definition. It specifies which controls are in scope for security assessment activities.

**Key properties:**
- Root key: `"assessment-plan"`
- Contains `"import-ssp"` referencing the System Security Plan (`system-ssp.json`)
- Lists all 26 controls under `"reviewed-controls" → "control-selections"`
- Deterministic UUID v5 based on control IDs and SSP reference

**Purpose:** The Assessment Plan is the starting point for assessors. It defines the scope of an assessment — which controls will be evaluated, and against which SSP (System Security Plan). The `--import-ssp` flag provides the SSP reference, and FORGE automatically generates the Assessment Plan skeleton.

**Command used:**
The Assessment Plan is generated automatically when `--import-ssp` is provided:
```bash
forge convert policy.md --strategy component \
  --source-profile output/cloud-security-baseline-profile.json \
  --output output/cloud-security-baseline-component.json \
  --import-ssp ./system-ssp.json
# Assessment Plan is written to: output/policy-assessment-plan.json
```

## The Full Pipeline Explained

```
policy.md (Markdown)
    │
    ▼  forge convert --strategy catalog
    │
cloud-security-baseline-catalog.json (OSCAL Catalog)
    │  All 26 controls extracted from policy
    │
    ▼  forge profile --include/--exclude
    │
cloud-security-baseline-profile.json (OSCAL Profile)
    │  15 controls selected as baseline
    │
    ▼  forge convert --strategy component --source-profile
    │
cloud-security-baseline-component.json (OSCAL Component Definition)
policy-assessment-plan.json        (OSCAL Assessment Plan)
    │  Implementation narratives + assessment scope
    │
    ▼  (downstream: System Security Plan, POA&M, etc.)
```

### How the Pieces Fit Together

1. **Catalog** is the source of truth for all controls in the organization's policy universe.
2. **Profile** carves out a specific subset of controls for a given system or compliance baseline.
3. **Component Definition** maps controls to actual system components and describes how each is implemented.
4. **Assessment Plan** uses the control set to define what will be assessed during a security review.
5. **System Security Plan (SSP)** (referenced but not generated here) would describe the complete system and how it satisfies all controls across all components.

Each artifact references the previous one: the Profile imports from the Catalog, the Component Definition references the Profile, and the Assessment Plan references the SSP. This creates a chain of traceability from human policy text to machine-executable compliance artifacts.

## Validation

All generated artifacts are validated against OSCAL JSON schemas during the conversion process. You can also validate them independently:

```bash
forge validate output/cloud-security-baseline-catalog.json
forge validate output/cloud-security-baseline-profile.json
forge validate output/cloud-security-baseline-component.json
```

The `forge validate` command auto-detects the OSCAL model type from the JSON root key and validates against the appropriate embedded schema.
