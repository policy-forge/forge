---
description: Bootstrap a feature spec from existing PRD, AR, and SEC documents by number.
handoffs:
  - label: Clarify Spec Requirements
    agent: speckit.clarify
    prompt: Clarify specification requirements
    send: true
  - label: Build Technical Plan
    agent: speckit.plan
    prompt: Create a plan for the spec. I am building with...
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

The user has provided a feature number (e.g. `037`). Use it to auto-discover existing PRD, AR, and SEC documents, then run the full `speckit.specify` workflow using those documents as the source of truth.

### Step 1 — Parse the feature number

Extract the numeric prefix from `$ARGUMENTS` (e.g. `037`, `42`, `100`). Normalise it to the zero-padded form used in this repo if needed (check existing filenames to determine padding convention).

If `$ARGUMENTS` is empty or non-numeric: ERROR "Usage: /speckit.kickoff <feature-number> (e.g. /speckit.kickoff 037)"

### Step 2 — Discover source documents

Glob for each document type under the repo root. At least one of PRD, AR, or SEC must exist; all three are preferred.

| Type | Glob pattern |
|------|-------------|
| PRD  | `docs/PRD/<number>-prd-*.md` |
| AR   | `docs/AR/<number>-ar-*.md`   |
| SEC  | `docs/SEC/<number>-sec-*.md` |

Rules:
- If multiple files match a pattern (shouldn't happen), use the first match and warn the user.
- If a document type is missing, proceed without it and note the gap.
- If **no documents** match any pattern: ERROR "No PRD, AR, or SEC documents found for number <number>. Expected files like docs/PRD/<number>-prd-*.md"

### Step 3 — Extract feature description

Read the discovered documents to determine:
- **Feature name** — use the document title (first `#` heading) from the PRD if available, otherwise AR, otherwise SEC.
- **Feature description** — synthesise a 2–4 sentence natural-language description of what the feature does and why, drawn from the PRD Overview / Problem Statement section (or equivalent in AR/SEC).

This description becomes the input to the spec workflow.

### Step 4 — Run the speckit.specify workflow

Execute the full `speckit.specify` workflow (all steps) using:

- **Feature description** = the description synthesised in Step 3
- **Pre-loaded context** = the PRD, AR, and SEC documents already read

When generating the spec, treat these documents as authoritative sources:
- **PRD** → functional requirements, user stories, success criteria, scope
- **AR** → technical constraints, architecture decisions, implementation guardrails
- **SEC** → security requirements (SEC-* IDs), trust boundaries, data classifications

Do **not** ask the user to provide information that is already present in these documents. Minimise `[NEEDS CLARIFICATION]` markers (max 3, only for genuine gaps).

**CRITICAL — feature number:** When invoking `.specify/scripts/bash/create-new-feature.sh`, pass `--number <N>` using the **exact feature number from the user's input** (e.g. `037` → `--number 37`). Do NOT let the script auto-assign a number by scanning for the next available one — the number must match the PRD/AR/SEC document number.

Refer to `speckit.specify` instructions for the full workflow (branch creation, spec template, quality validation checklist, worktree handling).

### Step 5 — Report completion

After the spec is written and validated, report:
- Feature number and discovered files
- Branch name and spec file path
- Any documents that were missing
- Checklist results
- Readiness for next phase (`/speckit.clarify` or `/speckit.plan`)
