# Quickstart Guide: Requirement Atomization Implementation

**Feature**: 006-requirement-atomization
**Audience**: Developers implementing this feature
**Prerequisites**: Rust 1.93.0+, cargo, familiarity with TDD

---

## Overview

This guide walks you through implementing the requirement atomization feature using Test-Driven Development (TDD). You'll write tests first (red), implement the code to make them pass (green), then refactor as needed.

**Time Estimate**: 2-3 days for full implementation + testing

---

## Step 1: Read the Design Documents (30 minutes)

Before writing any code, understand the architecture and requirements:

1. **[spec.md](./spec.md)**: Feature specification (user stories, acceptance criteria, edge cases)
2. **[plan.md](./plan.md)**: Implementation plan (technical context, constitution check, structure)
3. **[data-model.md](./data-model.md)**: Domain model details (PolicyRequirement fields, validation rules)
4. **[contracts/atomize-api.md](./contracts/atomize-api.md)**: Function signatures and contracts
5. **[docs/AR/006-ar-requirement-atomization.md](../../../docs/AR/006-ar-requirement-atomization.md)**: Architecture decisions and implementation guardrails
6. **[docs/SEC/006-sec-requirement-atomization.md](../../../docs/SEC/006-sec-requirement-atomization.md)**: Security requirements

**Key Takeaways**:
- Conservative splitting: only split on clear patterns (conjunction + normative verb)
- Deterministic: same input always produces same output
- Maximum 50 splits per requirement (SEC-5)
- Case-sensitive: lowercase normative verbs only
- SHA-256 for preliminary IDs (replaced in WI-7)

---

## Step 2: Set Up the Module Structure (15 minutes)

Create the new atomization module:

```bash
# From repository root
cd src/parse
touch atomize.rs
```

Add the module to `src/parse/mod.rs` (or `src/lib.rs` if `parse` is a top-level module):

```rust
// src/parse/mod.rs (or appropriate parent module)
pub mod atomize;
```

Create test directories:

```bash
# From repository root
mkdir -p tests/fixtures
mkdir -p tests/unit
mkdir -p tests/integration
touch tests/fixtures/compound_statements.txt
touch tests/fixtures/atomic_statements.txt
touch tests/fixtures/edge_cases.txt
touch tests/unit/atomize_test.rs
touch tests/integration/atomize_integration_test.rs
```

---

## Step 3: Write Test Fixtures (1 hour)

Populate test fixtures with example policy statements from the spec.

**tests/fixtures/compound_statements.txt**:
```text
# Compound statements that should be split
# Format: LINE_NUMBER | STATEMENT | EXPECTED_SPLITS

42 | Systems must enforce MFA and must require complex passwords | 2
10 | The organization shall review access logs and shall revoke inactive accounts within 30 days | 2
5 | All employees must complete security training and must acknowledge the acceptable use policy or must request a waiver | 3
100 | Systems must X and must Y and must Z | 3
```

**tests/fixtures/atomic_statements.txt**:
```text
# Atomic statements that should NOT be split (preserved as-is)
# Format: LINE_NUMBER | STATEMENT

1 | All systems must enforce MFA
2 | Passwords must be at least 12 characters
3 | Systems must implement logging and monitoring
```

**tests/fixtures/edge_cases.txt**:
```text
# Edge cases from EC-1 through EC-10
# Format: EDGE_CASE_ID | LINE_NUMBER | STATEMENT | EXPECTED_BEHAVIOR

EC-1 | 50 | Systems must encrypt and store data securely | preserve_as_is
EC-2 | 51 | Systems must implement MFA or must use certificate-based authentication | split_into_2
EC-5 | 52 | Systems must implement logging and monitoring | preserve_as_is
EC-6 | 53 | Systems shall enforce MFA and shall require passwords | split_into_2
EC-7 | 54 |  | preserve_as_is
EC-9 | 55 | [51+ conjunction-verb pairs] | preserve_as_is
EC-10 | 56 | Systems MUST enforce MFA and MUST require passwords | preserve_as_is
```

---

## Step 4: Write Unit Tests (2-3 hours)

**tests/unit/atomize_test.rs**:

```rust
use forge::model::PolicyRequirement;
use forge::parse::atomize::{atomize_requirement, preliminary_id, AtomizationResult};

#[test]
fn test_preliminary_id_determinism() {
    // FR-004, AC-6: Same input produces same ID
    let id1 = preliminary_id("Systems must enforce MFA", 42, 0);
    let id2 = preliminary_id("Systems must enforce MFA", 42, 0);
    assert_eq!(id1, id2);
    assert_eq!(id1.len(), 64); // SHA-256 hex
}

#[test]
fn test_preliminary_id_uniqueness() {
    // Different atom_index produces different ID
    let id1 = preliminary_id("Systems must enforce MFA", 42, 0);
    let id2 = preliminary_id("Systems must enforce MFA", 42, 1);
    assert_ne!(id1, id2);
}

#[test]
fn test_atomize_compound_statement_two_parts() {
    // AC-1: "Systems must enforce MFA and must require complex passwords"
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "Systems must enforce MFA and must require complex passwords".to_string(),
        source_line: 42,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(result.was_split);
    assert_eq!(result.requirements.len(), 2);
    assert_eq!(result.original_text, Some(req.text.clone()));

    // Part 1
    assert_eq!(result.requirements[0].text, "Systems must enforce MFA");
    assert_eq!(result.requirements[0].source_line, 42);
    assert_eq!(result.requirements[0].atom_index, 0);
    assert_eq!(result.requirements[0].parent_text, Some(req.text.clone()));

    // Part 2
    assert_eq!(result.requirements[1].text, "Systems must require complex passwords");
    assert_eq!(result.requirements[1].source_line, 42);
    assert_eq!(result.requirements[1].atom_index, 1);
    assert_eq!(result.requirements[1].parent_text, Some(req.text));
}

#[test]
fn test_atomize_atomic_statement_unchanged() {
    // AC-3: "All systems must enforce MFA" (atomic)
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "All systems must enforce MFA".to_string(),
        source_line: 10,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(!result.was_split);
    assert_eq!(result.requirements.len(), 1);
    assert_eq!(result.requirements[0].text, "All systems must enforce MFA"); // Unchanged (FR-003)
    assert_eq!(result.requirements[0].atom_index, 0);
    assert_eq!(result.original_text, None);
}

#[test]
fn test_ec1_and_without_following_verb() {
    // EC-1: "must encrypt and store data securely" (no verb after "and")
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "Systems must encrypt and store data securely".to_string(),
        source_line: 50,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(!result.was_split); // Preserved as-is
    assert_eq!(result.requirements.len(), 1);
    assert_eq!(result.requirements[0].text, "Systems must encrypt and store data securely");
}

#[test]
fn test_ec2_or_conjunction_with_verbs() {
    // EC-2: "must implement MFA or must use certificate-based authentication"
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "Systems must implement MFA or must use certificate-based authentication".to_string(),
        source_line: 51,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(result.was_split);
    assert_eq!(result.requirements.len(), 2);
    assert_eq!(result.requirements[0].text, "Systems must implement MFA");
    assert_eq!(result.requirements[1].text, "Systems must use certificate-based authentication");
}

#[test]
fn test_ec7_empty_text() {
    // EC-7: Empty or whitespace-only text preserved as-is
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "   ".to_string(), // Whitespace-only
        source_line: 54,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(!result.was_split);
    assert_eq!(result.requirements.len(), 1);
    assert_eq!(result.requirements[0].text, "   "); // Unchanged
}

#[test]
fn test_ec9_max_split_count_exceeded() {
    // EC-9: Statement producing >50 splits preserved as-is
    // Generate a pathological statement with 51 "and must" pairs
    let text = format!("Systems must X{}", " and must Y".repeat(50));
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: text.clone(),
        source_line: 55,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(!result.was_split); // Preserved as-is (exceeds max)
    assert_eq!(result.requirements.len(), 1);
    assert_eq!(result.requirements[0].text, text);
    // Expect warning logged (check test output)
}

#[test]
fn test_ec10_uppercase_normative_verbs() {
    // EC-10: Uppercase normative verbs do NOT match (case-sensitive)
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "Systems MUST enforce MFA and MUST require passwords".to_string(),
        source_line: 56,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();

    assert!(!result.was_split); // Preserved as-is (case-sensitive matching)
    assert_eq!(result.requirements.len(), 1);
}

#[test]
fn test_sec4_redos_resistance_large_input() {
    // SC-007, SEC-4: Test with 10KB+ repetitive string
    let large_text = "Systems must X ".repeat(1000); // ~15KB
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: large_text.clone(),
        source_line: 100,
        atom_index: 0,
        parent_text: None,
    };

    // Should complete in linear time (no ReDoS)
    let start = std::time::Instant::now();
    let result = atomize_requirement(&req).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 1); // Should be fast (linear time)
    // May or may not split depending on pattern matching; just ensure it doesn't hang
}

#[test]
fn test_sec4_unicode_edge_cases() {
    // SC-007, SEC-4: Unicode edge cases
    let req = PolicyRequirement {
        stable_id: "temp".to_string(),
        text: "Systems must enforce MFA\u{200B}and must require passwords".to_string(), // Zero-width space
        source_line: 101,
        atom_index: 0,
        parent_text: None,
    };

    let result = atomize_requirement(&req).unwrap();
    // Should handle gracefully (likely preserved as-is since zero-width space breaks word boundary)
}
```

**Run tests** (they should all fail — RED):
```bash
cargo test atomize
```

Expected: All tests fail because `atomize_requirement` and `preliminary_id` are not implemented yet.

---

## Step 5: Implement the Atomization Logic (4-6 hours)

**src/parse/atomize.rs**:

```rust
use crate::model::{PolicyDocument, PolicyRequirement};
use crate::error::ForgeError;
use regex::Regex;
use sha2::{Sha256, Digest};
use once_cell::sync::Lazy;

/// Compiled regex pattern for conjunction + normative verb detection.
/// Pattern: \b(and|or)\s+(must|shall|should|will)\b
/// Case-sensitive (lowercase normative verbs only).
static SPLIT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(and|or)\s+(must|shall|should|will)\b")
        .expect("Failed to compile split pattern regex")
});

/// Maximum number of splits allowed per requirement (SEC-5, FR-010, EC-9).
const MAX_SPLITS_PER_REQUIREMENT: usize = 50;

/// Result of atomizing a single policy requirement.
#[derive(Debug, Clone)]
pub struct AtomizationResult {
    /// The atomic requirements produced (1 if already atomic, N if split).
    pub requirements: Vec<PolicyRequirement>,
    /// Whether the original statement was split.
    pub was_split: bool,
    /// The original compound text (if split).
    pub original_text: Option<String>,
}

/// Atomize all requirements in a PolicyDocument.
/// Replaces compound PolicyRequirements with their atomic parts.
///
/// # Errors
/// Returns ForgeError::Parse if atomization fails.
pub fn atomize_document(document: PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    // TODO: Implement
    // 1. Iterate over sections
    // 2. For each requirement, call atomize_requirement
    // 3. Replace compound requirements with atomic parts
    // 4. Return updated PolicyDocument
    todo!("Implement atomize_document")
}

/// Atomize a single policy requirement.
/// Returns one or more atomic requirement texts.
///
/// # Algorithm
/// See contracts/atomize-api.md for detailed algorithm.
pub fn atomize_requirement(requirement: &PolicyRequirement) -> Result<AtomizationResult, ForgeError> {
    // TODO: Implement
    // 1. Match regex pattern
    // 2. If no match: return as-is
    // 3. If match: check split count, extract subject, split, reconstruct, assign IDs
    todo!("Implement atomize_requirement")
}

/// Generate a preliminary stable ID for an atomic requirement.
/// Uses SHA-256 hash (hex-encoded) of text + source_line + atom_index.
///
/// # Format
/// Input: text + "|" + source_line + "|" + atom_index
/// Output: 64-character hex-encoded SHA-256 hash
pub fn preliminary_id(text: &str, source_line: usize, atom_index: usize) -> String {
    // TODO: Implement
    // 1. Construct input: text + "|" + source_line + "|" + atom_index
    // 2. Compute SHA-256 hash
    // 3. Encode as lowercase hex
    todo!("Implement preliminary_id")
}

/// Extract the shared subject from a compound statement.
/// Subject = text before the first normative verb occurrence.
fn extract_subject(text: &str, first_verb_pos: usize) -> Option<String> {
    // TODO: Implement
    // 1. Substring text[0..first_verb_pos]
    // 2. Trim whitespace
    // 3. Return Some(subject) if non-empty, else None
    todo!("Implement extract_subject")
}

/// Reconstruct a complete sentence by prepending the shared subject to a clause fragment.
fn reconstruct_clause(shared_subject: &str, clause: &str) -> String {
    // TODO: Implement
    // 1. Trim clause
    // 2. Check if clause already starts with shared_subject (case-insensitive)
    // 3. If yes: return clause unchanged
    // 4. If no: return shared_subject + " " + clause
    todo!("Implement reconstruct_clause")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Add inline unit tests here if desired (in addition to tests/unit/)
}
```

**Implementation Tips**:
1. Start with `preliminary_id` (simplest function)
2. Then `extract_subject` and `reconstruct_clause`
3. Then `atomize_requirement` (core logic)
4. Finally `atomize_document` (orchestration)

**Run tests incrementally**:
```bash
# After implementing preliminary_id:
cargo test test_preliminary_id

# After implementing atomize_requirement:
cargo test test_atomize
```

Expected: Tests turn GREEN as you implement each function.

---

## Step 6: Implement Integration Tests (1 hour)

**tests/integration/atomize_integration_test.rs**:

```rust
use forge::model::{PolicyDocument, PolicySection, PolicyRequirement};
use forge::parse::atomize::atomize_document;

#[test]
fn test_atomize_document_end_to_end() {
    // AC-8: Full document with mix of compound and atomic requirements
    let section = PolicySection {
        heading: "Access Control".to_string(),
        requirements: vec![
            PolicyRequirement {
                stable_id: "temp1".to_string(),
                text: "Systems must enforce MFA and must require complex passwords".to_string(),
                source_line: 10,
                atom_index: 0,
                parent_text: None,
            },
            PolicyRequirement {
                stable_id: "temp2".to_string(),
                text: "All systems must enforce MFA".to_string(), // Atomic
                source_line: 11,
                atom_index: 0,
                parent_text: None,
            },
        ],
    };

    let document = PolicyDocument {
        title: "Test Policy".to_string(),
        sections: vec![section],
    };

    let original_count = document.total_requirement_count();
    let atomized = atomize_document(document).unwrap();
    let atomized_count = atomized.total_requirement_count();

    // Total count increased (compound split into 2, atomic unchanged)
    assert_eq!(atomized_count, original_count + 1); // 2 + 1 = 3

    // Verify first section has 3 requirements (2 from split + 1 atomic)
    assert_eq!(atomized.sections[0].requirements.len(), 3);

    // Verify split requirements have sequential atom_index
    let split_reqs: Vec<_> = atomized.sections[0].requirements.iter()
        .filter(|r| r.source_line == 10)
        .collect();
    assert_eq!(split_reqs.len(), 2);
    assert_eq!(split_reqs[0].atom_index, 0);
    assert_eq!(split_reqs[1].atom_index, 1);

    // Verify atomic requirement unchanged
    let atomic_req = atomized.sections[0].requirements.iter()
        .find(|r| r.source_line == 11)
        .unwrap();
    assert_eq!(atomic_req.text, "All systems must enforce MFA");
    assert_eq!(atomic_req.atom_index, 0);
    assert!(atomic_req.parent_text.is_none());
}

#[test]
fn test_ec8_zero_requirements() {
    // EC-8: PolicyDocument with zero requirements returned unchanged
    let document = PolicyDocument {
        title: "Empty Policy".to_string(),
        sections: vec![],
    };

    let atomized = atomize_document(document).unwrap();
    assert_eq!(atomized.sections.len(), 0);
}
```

---

## Step 7: Refactor and Optimize (2 hours)

Once all tests pass (GREEN), refactor for clarity and performance:

1. **Extract constants**: Move magic numbers to named constants
2. **Simplify complex functions**: Break down long functions into smaller helpers
3. **Add logging**: Use `log::debug!` for FR-011 summary metrics
4. **Optimize regex**: Ensure `SPLIT_PATTERN` is compiled once (already done with `Lazy`)
5. **Handle warnings**: Use `tracing::warn!` for EC-9 (max splits) and subject extraction failures

**Example logging**:
```rust
use tracing::{debug, warn};

pub fn atomize_document(document: PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    let total_reqs = document.total_requirement_count();
    let mut split_count = 0;
    let mut preserved_count = 0;

    // ... atomization logic ...

    debug!("Atomization summary: {} requirements processed, {} split, {} preserved",
           total_reqs, split_count, preserved_count);

    Ok(updated_document)
}
```

**Run all tests**:
```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Expected: All tests pass, no clippy warnings, code is formatted.

---

## Step 8: Mutation Testing (1 hour)

Run mutation testing to verify test quality:

```bash
cargo mutants
```

Expected: High mutation score (>80%). If mutants survive, add more tests to kill them.

---

## Step 9: Final Validation (30 minutes)

**Checklist**:
- [x] All unit tests pass (AC-1 through AC-8, EC-1 through EC-10)
- [x] Integration tests pass (atomize_document end-to-end)
- [x] Adversarial input tests pass (SC-007, SEC-4)
- [x] Determinism tests pass (FR-004, AC-6)
- [x] No clippy warnings (`cargo clippy -- -D warnings`)
- [x] Code formatted (`cargo fmt`)
- [x] Mutation testing score >80% (`cargo mutants`)
- [x] Logging implemented (FR-011: DEBUG-level summary metrics)
- [x] All security requirements satisfied (SEC-1, SEC-4, SEC-5, SEC-7, SEC-9)

**Final command**:
```bash
cargo test --all && cargo clippy -- -D warnings && cargo fmt --check && cargo mutants
```

---

## Key Constraints (AR Implementation Guardrails)

**DO NOT**:
- Split on conjunctions without a following normative verb (EC-1, EC-5)
- Use NLP, ML, or any non-deterministic processing (P-3)
- Modify the text of atomic (non-compound) statements (FR-003, SEC-9)
- Generate UUID v5 identifiers — use preliminary SHA-256 IDs only (FR-010, W-1)

**MUST**:
- Reconstruct shared subject for each split clause (FR-002)
- Preserve source line numbers (FR-005)
- Test regex against adversarial input (SC-007, SEC-4)
- Enforce maximum 50 splits per requirement (FR-010, SEC-5, EC-9)
- Ensure pure functions (no side effects, no global state) (SEC-7)

---

## Integration Points

**Input**: `PolicyDocument` from WI-5 (after domain model construction)

**Output**: Updated `PolicyDocument` with atomized requirements

**Next Stage**: WI-7 (UUID generation) consumes atomized requirements and replaces preliminary IDs with deterministic UUID v5

**Pipeline Position**:
```text
WI-2 (Markdown Ingestion)
  → WI-3 (Heading Extraction)
  → WI-4 (Clause Extraction)
  → WI-5 (Domain Model)
  → **WI-6 (Atomization)** ← YOU ARE HERE
  → WI-7 (UUID Generation)
  → WI-9 (OSCAL Generation)
```

---

## Troubleshooting

**Problem**: Tests fail with "pattern did not match expected output"
- **Solution**: Check regex pattern is correct (`\b(and|or)\s+(must|shall|should|will)\b`)
- **Solution**: Verify case sensitivity (lowercase normative verbs only)

**Problem**: `preliminary_id` produces different IDs on repeated calls
- **Solution**: Check delimiter is "|" (not empty or inconsistent)
- **Solution**: Verify SHA-256 hash is hex-encoded (lowercase, 64 chars)

**Problem**: Subject reconstruction produces duplicated subjects
- **Solution**: Check `reconstruct_clause` for case-insensitive comparison
- **Solution**: Trim whitespace before comparison

**Problem**: Mutation testing score is low
- **Solution**: Add tests for boundary conditions (empty text, single word, etc.)
- **Solution**: Add tests for error paths (subject extraction failure, max splits)

---

## Next Steps

1. ✅ Implement atomization logic (this guide)
2. 🔜 Run `/speckit.tasks` to generate `tasks.md`
3. 🔜 Execute tasks in dependency order
4. 🔜 Create PR and request code review
5. 🔜 Address review feedback
6. 🔜 Merge to main

**Status**: ✅ **READY TO IMPLEMENT**
