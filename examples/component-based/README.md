# Component-Based Policy Example

This example demonstrates how FORGE converts a security policy with **two system components** (a web application and a database) into OSCAL artifacts, including a **Component Definition** and **System Security Plan (SSP)**.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Web Application                       │
│  - MFA Authentication    - RBAC Authorization            │
│  - TLS 1.3               - Audit Logging                 │
│  - Vulnerability Scanning - Alerting                     │
└─────────────────────┬───────────────────────────────────┘
                      │ mTLS (certificate auth)
                      ▼
┌─────────────────────────────────────────────────────────┐
│                      Database                            │
│  - AES-256 Encryption    - Key Rotation (90 days)        │
│  - Access Logging        - Automated Backups (6h)        │
│  - Restore Testing                                       │
└─────────────────────────────────────────────────────────┘
```

## Files

```
component-based/
├── policy.md              # Source security policy (7 controls, 2 components)
├── output/
│   ├── catalog.json           # OSCAL Catalog (17 controls)
│   ├── profile.json           # OSCAL Profile (10 selected controls)
│   ├── component-definition.json  # OSCAL Component Definition
│   └── ssp.json               # OSCAL System Security Plan
├── generate_ssp.py          # Script to generate SSP from component definition
└── README.md                # This walkthrough
```

## Step-by-Step Walkthrough

### Step 1: Generate the Catalog

The catalog converts all policy requirements into OSCAL controls:

```bash
forge convert policy.md --strategy catalog --format json --output output/catalog.json
```

**What happens:** FORGE parses the markdown, extracts 17 individual requirements across 7 control families (AC-1 through SI-2), assigns stable IDs (e.g., `POL-CSP-001`), and produces an OSCAL Catalog with traceability links back to the source policy lines.

**Key output structure:**
- Each control gets a deterministic ID like `POL-CSP-001`
- Controls are grouped by their section headings
- Each control has `links` pointing to `policy.md#line=N` for traceability
- `props` include modality (normative/advisory) and source section

### Step 2: Create a Profile (Baseline Selection)

A profile selects a subset of controls to form your compliance baseline:

```bash
forge profile --catalog output/catalog.json \
  --include "POL-CSP-001,POL-CSP-002,POL-CSP-004,POL-CSP-005,POL-CSP-007,POL-CSP-008,POL-CSP-009,POL-CSP-010,POL-CSP-012,POL-CSP-014" \
  --format json \
  --output output/profile.json
```

**What happens:** The profile references the catalog via `imports[0].href` and includes exactly 10 controls using `include-controls[0].with-ids`. This represents your organization's selected baseline — not every control from the catalog is required.

**Why use a profile:** In real compliance scenarios, you rarely implement every control. A profile lets you select the specific controls that apply to your system, creating a reusable baseline that can be shared across multiple systems.

### Step 3: Generate the Component Definition

The component definition maps policy controls to specific system components:

```bash
forge convert policy.md \
  --strategy component \
  --source-profile output/profile.json \
  --format json \
  --output output/component-definition.json
```

**What happens:** FORGE produces a Component Definition with:
- **One documentary component** of type `"policy"` representing the source policy
- **Control implementations** inside that component, one per selected control
- Each implemented requirement has a `control-id` matching the catalog and `props` for traceability (source file, section, line number, modality)

**How components are represented:**

```json
{
  "component-definition": {
    "components": [
      {
        "uuid": "<deterministic-v5>",
        "type": "policy",
        "title": "Component-Based Security Policy",
        "description": "Documentary component representing the Component-Based Security Policy policy document.",
        "control-implementations": [
          {
            "uuid": "...",
            "source": "output/profile.json",
            "description": "Controls implemented per the Component-Based Security Policy.",
            "implemented-requirements": [
              {
                "uuid": "...",
                "control-id": "POL-CSP-001",
                "description": "The application component SHALL enforce multi-factor authentication...",
                "props": [
                  {"name": "source-file", "value": "policy.md"},
                  {"name": "source-section", "value": "AC-1: Authentication"},
                  {"name": "modality", "value": "normative"}
                ]
              }
            ]
          }
        ]
      }
    ]
  }
}
```

The `type: "policy"` indicates this is a **documentary component** — it describes what the policy requires, not what a specific system implements. The actual system components (web application, database) are described in the SSP.

### Step 4: Generate the System Security Plan (SSP)

The SSP describes how the components implement the controls:

```bash
python3 generate_ssp.py
```

This script reads the component definition and produces an SSP with:

- **Two system components** in `system-implementation.components[]`:
  - `Web Application Component` (type: software)
  - `Database Component` (type: software)
- **Three user roles**: system administrator, security analyst, service account
- **One leveraged authorization**: AWS GovCloud (FedRAMP Moderate)
- **25 implemented requirements** mapped to the controls from the component definition

**How SSP components relate to the Component Definition:**

```
Component Definition (policy-level)          SSP (system-level)
┌──────────────────────────────┐             ┌──────────────────────────────┐
│ Documentary Component        │             │ System Implementation        │
│   type: "policy"             │────────────>│   components[]               │
│   title: "CSP"               │   maps to   │     - Web Application        │
│                              │             │     - Database               │
│ control-implementations[]    │             │   users[]                    │
│   implemented-requirements[] │             │   leveraged-authorizations[] │
└──────────────────────────────┘             └──────────────────────────────┘
```

The Component Definition says *what* the policy requires. The SSP says *how* each system component satisfies those requirements.

### Step 5: Validate Outputs

Validate the generated OSCAL artifacts against their schemas:

```bash
# Validate catalog
forge validate output/catalog.json --schema-type catalog

# Validate component definition
forge validate output/component-definition.json --schema-type component-definition
```

Both should report: `Valid: <type> artifact passes all validation.`

## Policy Controls

The source `policy.md` defines 7 control families with 25 individual requirements:

| Control ID | Family | Component | Requirements |
|------------|--------|-----------|--------------|
| AC-1 | Authentication | Application | MFA, session tokens, lockout |
| AC-2 | Authorization | Application | RBAC, API validation, approval workflow |
| DP-1 | Encryption at Rest | Database | AES-256, key management, rotation |
| DP-2 | Encryption in Transit | Both | TLS 1.3, mTLS, certificate auth |
| AL-1 | Audit Logging | Both | Auth events, DB access, retention, forwarding |
| AL-2 | Monitoring | Both | Auth failure alerts, unauthorized access alerts |
| SI-1 | Backup/Recovery | Database | Automated backups, retention, restore testing |
| SI-2 | Vulnerability Management | Both | Scanning, patching, dependency checks |

## Reproducing This Example

1. Ensure `forge` is built: `cargo build --release`
2. From the `examples/component-based/` directory, run the commands in Steps 1-5 above
3. All outputs will be written to `output/`
