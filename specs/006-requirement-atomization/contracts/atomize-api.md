# API Contract: Atomization Functions

**Feature**: 006-requirement-atomization
**Module**: `src/parse/atomize.rs`
**Generated**: 2026-02-11
**Source**: [spec.md](../spec.md), [AR-006](../../../docs/AR/006-ar-requirement-atomization.md)

## Public API

### `atomize_document`

**Signature**:
```rust
pub fn atomize_document(document: &PolicyDocument) -> Result<PolicyDocument, ForgeError>
```

**Purpose**: Atomize all requirements in a PolicyDocument, replacing compound requirements with their atomic parts.

**Algorithm** (high-level):
1. Iterate over all `PolicySection` in the `PolicyDocument`.
2. For each `PolicyRequirement` in each section:
   - Call `atomize_requirement(&requirement)`
   - If `was_split == true`: replace the original requirement with the N atomic requirements
   - If `was_split == false`: keep the original requirement unchanged
3. Return the updated `PolicyDocument`.

**Pre-conditions**:
- `document` must be a valid `PolicyDocument` (from WI-5 domain model)
- All `PolicyRequirement` instances must have valid `source_line` (>= 1)

**Post-conditions**:
- All compound requirements are replaced by atomic parts (1:N transformation)
- All atomic (non-compound) requirements are preserved unchanged (FR-003)
- Total requirement count >= original count (can only increase or stay same)
- All requirements have deterministic `stable_id` (FR-004)
- All requirements have `source_line` preserved from parent (FR-005)

**Errors**:
- `ForgeError::Parse` if regex compilation fails (should not happen with static patterns)
- `ForgeError::Parse` if subject extraction fails in an unrecoverable way (AR-006 error handling: preserve as-is, log warning — so this error should be rare)

**Examples**:
```rust
use forge::model::PolicyDocument;
use forge::parse::atomize::atomize_document;

// Given a PolicyDocument with 1 compound requirement
let document = PolicyDocument::new(/* ... */);
let original_count = document.total_requirement_count();

// When atomizing
let atomized = atomize_document(document)?;

// Then the count increases (or stays same if no compounds)
assert!(atomized.total_requirement_count() >= original_count);

// And all requirements have stable_ids
for section in &atomized.sections {
    for req in &section.requirements {
        assert!(!req.stable_id.is_empty());
        assert_eq!(req.stable_id.len(), 64); // SHA-256 hex
    }
}
```

**Performance**: O(n * m) where n = total requirements, m = average text length. Expected <1 second for documents with <1000 requirements.

**Thread Safety**: Safe (pure function, no shared state).

**Idempotency**: No (repeated calls on already-atomized requirements will attempt to re-atomize; should be called once per document).

---

### `atomize_requirement`

**Signature**:
```rust
pub fn atomize_requirement(requirement: &PolicyRequirement) -> Result<AtomizationResult, ForgeError>
```

**Purpose**: Atomize a single policy requirement, returning one or more atomic requirements.

**Algorithm** (detailed):
1. **Match regex pattern**: `\b(and|or)\s+(must|shall|should|will)\b` (case-sensitive)
2. **If no match**: Return `AtomizationResult { requirements: vec![requirement.clone()], was_split: false, original_text: None }`
3. **If match found**:
   - Count number of splits (number of conjunction + normative verb boundaries)
   - **If count > 50**: Preserve as-is, log warning (FR-010, EC-9), return non-split result
   - Extract shared subject (text before first normative verb occurrence)
   - **If subject extraction fails**: Preserve as-is, log warning (AR-006 error handling), return non-split result
   - Split text at each conjunction + normative verb boundary
   - For each clause fragment:
     - Reconstruct complete sentence by prepending shared subject (call `reconstruct_clause`)
     - Trim whitespace
     - Assign preliminary ID: `preliminary_id(text, requirement.source_line, atom_index)`
   - Return `AtomizationResult { requirements: vec![atomic_req_0, atomic_req_1, ...], was_split: true, original_text: Some(requirement.text.clone()) }`

**Pre-conditions**:
- `requirement` must have valid `source_line` (>= 1)
- `requirement.text` may be empty (EC-7: preserved as-is, no error)

**Post-conditions**:
- Returns 1 or more atomic requirements
- If `was_split == true`, all returned requirements have sequential `atom_index` (0, 1, 2, ...)
- If `was_split == false`, returned requirement has `atom_index == 0` and `parent_text == None`
- All returned requirements have the same `source_line` as the input requirement (FR-005)
- All returned requirements have deterministic `stable_id` (FR-004)

**Errors**:
- `ForgeError::Parse` if regex matching fails (should not happen with valid Rust regex crate)

**Examples**:
```rust
use forge::model::PolicyRequirement;
use forge::parse::atomize::atomize_requirement;

// Example 1: Compound statement
let compound = PolicyRequirement {
    stable_id: "temp".to_string(),
    text: "Systems must enforce MFA and must require complex passwords".to_string(),
    source_line: 42,
    atom_index: 0,
    parent_text: None,
};

let result = atomize_requirement(&compound)?;
assert!(result.was_split);
assert_eq!(result.requirements.len(), 2);
assert_eq!(result.requirements[0].text, "Systems must enforce MFA");
assert_eq!(result.requirements[1].text, "Systems must require complex passwords");
assert_eq!(result.requirements[0].atom_index, 0);
assert_eq!(result.requirements[1].atom_index, 1);
assert_eq!(result.requirements[0].source_line, 42);
assert_eq!(result.requirements[1].source_line, 42);

// Example 2: Atomic statement (no split)
let atomic = PolicyRequirement {
    stable_id: "temp".to_string(),
    text: "All systems must enforce MFA".to_string(),
    source_line: 10,
    atom_index: 0,
    parent_text: None,
};

let result = atomize_requirement(&atomic)?;
assert!(!result.was_split);
assert_eq!(result.requirements.len(), 1);
assert_eq!(result.requirements[0].text, "All systems must enforce MFA"); // Unchanged
assert_eq!(result.requirements[0].atom_index, 0);
assert_eq!(result.original_text, None);

// Example 3: Edge case — exceeds max split count
let pathological = PolicyRequirement {
    stable_id: "temp".to_string(),
    text: "must X and must Y and must Z ... (51+ times)".to_string(), // >50 splits
    source_line: 99,
    atom_index: 0,
    parent_text: None,
};

let result = atomize_requirement(&pathological)?;
assert!(!result.was_split); // Preserved as-is (EC-9)
assert_eq!(result.requirements.len(), 1);
// Expect warning logged: "Requirement at line 99 would produce >50 splits; preserved as-is"
```

**Performance**: O(m) where m = text length. Regex matching is linear-time (Rust regex crate guarantee).

**Thread Safety**: Safe (pure function, no shared state).

---

### `preliminary_id`

**Signature**:
```rust
pub fn preliminary_id(text: &str, source_line: usize, atom_index: usize) -> String
```

**Purpose**: Generate a preliminary stable ID for an atomic requirement using SHA-256 hash.

**Algorithm**:
1. Construct input string: `text + "|" + source_line.to_string() + "|" + atom_index.to_string()`
2. Compute SHA-256 hash of input string (using `sha2` crate)
3. Encode hash as lowercase hex string (64 characters)
4. Return hex string

**Pre-conditions**:
- `text` may be any string (including empty — EC-7)
- `source_line` should be >= 1 (but function does not validate; relies on domain model invariant)
- `atom_index` should be >= 0 (but function does not validate)

**Post-conditions**:
- Returns a 64-character lowercase hexadecimal string
- Deterministic: same inputs always produce the same output (FR-004)
- Unique: different inputs produce different outputs (SHA-256 collision resistance)

**Errors**: None (pure function, always succeeds)

**Examples**:
```rust
use forge::parse::atomize::preliminary_id;

let id1 = preliminary_id("Systems must enforce MFA", 42, 0);
assert_eq!(id1.len(), 64); // SHA-256 hex = 32 bytes * 2 hex chars/byte
assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));

// Determinism test
let id2 = preliminary_id("Systems must enforce MFA", 42, 0);
assert_eq!(id1, id2);

// Uniqueness test (different atom_index)
let id3 = preliminary_id("Systems must enforce MFA", 42, 1);
assert_ne!(id1, id3);
```

**Performance**: O(n) where n = text length (SHA-256 hashing).

**Thread Safety**: Safe (pure function, no shared state).

**Notes**:
- This ID is **temporary**. It will be replaced by a deterministic UUID v5 in WI-7.
- The delimiter "|" is used to separate components to avoid ambiguity (e.g., `"ab" + "12" + "0"` vs `"a" + "b12" + "0"`).
- The choice of SHA-256 (vs. std::hash or xxHash) was made for consistency with WI-2 (clarification 2026-02-11).

---

## Internal Helpers (Private API)

### `SPLIT_PATTERN` (static)

**Definition**:
```rust
static SPLIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(and|or)\s+(must|shall|should|will)\b")
        .expect("Failed to compile split pattern regex")
});
```

**Purpose**: Compile-once regex pattern for conjunction + normative verb detection. Initialized on first access via `std::sync::LazyLock`, providing thread-safe, zero-cost access after initialization.

**Pattern**: `\b(and|or)\s+(must|shall|should|will)\b`
- `\b`: Word boundary (ensures "and" is a full word, not part of "band")
- `(and|or)`: Conjunction (capturing group)
- `\s+`: One or more whitespace characters
- `(must|shall|should|will)`: Normative verb (capturing group)
- `\b`: Word boundary (ensures "must" is a full word, not part of "mustard")

**Case Sensitivity**: Case-sensitive (lowercase normative verbs only). Uppercase or mixed-case normative verbs (e.g., "MUST", "Must") will NOT match (EC-10).

**Thread Safety**: Thread-safe via `LazyLock` — the regex is compiled exactly once, even under concurrent access.

**Panics**: The initializer panics if regex compilation fails (should not happen with this static pattern).

---

### `extract_subject`

**Signature**:
```rust
fn extract_subject(text: &str, first_verb_pos: usize) -> Option<String>
```

**Purpose**: Extract the shared subject from a compound statement (text before the first normative verb).

**Algorithm**:
1. Substring `text[0..first_verb_pos]`
2. Trim leading/trailing whitespace
3. If resulting string is empty, return `None`
4. Otherwise, return `Some(trimmed_subject)`

**Pre-conditions**:
- `first_verb_pos` should be <= `text.len()` (caller's responsibility)

**Post-conditions**:
- Returns `Some(subject)` if a non-empty subject is found
- Returns `None` if subject extraction fails (empty or whitespace-only)

**Error Handling** (per AR-006):
- If `None` is returned, the caller (atomize_requirement) should preserve the original text as-is and log a warning.

**Examples**:
```rust
// Example 1: Clear subject
let subject = extract_subject("Systems must enforce MFA and must require passwords", 8); // "must" at pos 8
assert_eq!(subject, Some("Systems".to_string()));

// Example 2: No clear subject
let subject = extract_subject("must X and must Y", 0); // "must" at pos 0
assert_eq!(subject, None); // Empty subject before first verb
```

---

### `reconstruct_clause`

**Signature**:
```rust
fn reconstruct_clause(shared_subject: &str, clause: &str) -> String
```

**Purpose**: Reconstruct a complete sentence by prepending the shared subject to a clause fragment.

**Algorithm**:
1. Trim `clause`
2. Check if `clause` already starts with `shared_subject` (case-insensitive comparison to avoid duplication)
3. If yes: return `clause` unchanged
4. If no: return `shared_subject + " " + clause`

**Examples**:
```rust
// Example 1: Clause lacks subject
let complete = reconstruct_clause("Systems", "enforce MFA");
assert_eq!(complete, "Systems enforce MFA");

// Example 2: Clause already has subject (no duplication)
let complete = reconstruct_clause("Systems", "Systems require passwords");
assert_eq!(complete, "Systems require passwords"); // Not "Systems Systems require passwords"

// Example 3: Clause has different subject (prepend anyway — conservative)
let complete = reconstruct_clause("Systems", "The organization must review logs");
assert_eq!(complete, "Systems The organization must review logs");
// Note: This may produce awkward phrasing, but it's conservative (preserves original text)
// AR-006: Accept imperfect phrasing for edge cases
```

**Notes**:
- This function may produce grammatically awkward results for complex sentence structures (subordinate clauses, etc.). This is acceptable per AR-006 ("Subject reconstruction heuristic may produce awkward phrasing for edge cases").
- The priority is correctness (preserving all information) over fluency.

---

## Result Types

### `AtomizationResult`

**Definition**:
```rust
pub struct AtomizationResult {
    /// The atomic requirements produced (1 if already atomic, N if split)
    pub requirements: Vec<PolicyRequirement>,

    /// Whether the original statement was split
    pub was_split: bool,

    /// The original compound text (if split)
    pub original_text: Option<String>,
}
```

**Invariants**:
- If `was_split == true`:
  - `requirements.len() >= 2`
  - `original_text.is_some()`
  - All requirements have sequential `atom_index` (0, 1, 2, ...)
  - All requirements have the same `source_line`
  - All requirements have `parent_text == original_text`

- If `was_split == false`:
  - `requirements.len() == 1`
  - `original_text.is_none()`
  - `requirements[0].atom_index == 0`
  - `requirements[0].parent_text.is_none()`

---

## Security Contracts (from SEC-006)

| Requirement | Contract Enforcement |
|-------------|---------------------|
| **SEC-1**: Use Rust `regex` crate | `SPLIT_PATTERN` uses `regex::Regex` via `LazyLock` |
| **SEC-4**: Test with adversarial input | Test suite includes 10KB+ repetitive strings, unicode edge cases (SC-007) |
| **SEC-5**: Enforce max split count | `atomize_requirement` checks split count; if >50, preserve as-is + log warning (FR-010) |
| **SEC-7**: Pure function | All functions are side-effect-free; no global state, no I/O, no threading |
| **SEC-9**: Atomic statements unchanged | `atomize_requirement` returns original text when `was_split == false` (FR-003) |

---

## Testing Contracts

All functions must have:
1. **Unit tests** for happy path, edge cases (EC-1 through EC-10), and error conditions
2. **Property tests** for determinism (`preliminary_id` same input = same output)
3. **Adversarial tests** for ReDoS resistance (10KB+ repetitive strings, unicode edge cases)
4. **Integration tests** for `atomize_document` end-to-end (compound doc in, atomized doc out)

See [quickstart.md](../quickstart.md) for TDD workflow.

---

## Next Steps

1. Review this contract with stakeholders
2. Generate test fixtures (see [quickstart.md](../quickstart.md))
3. Write tests BEFORE implementation (TDD mandate)
4. Implement `src/parse/atomize.rs` following AR-006 guardrails
5. Run `cargo test` — all tests should pass

**Contract Status**: ✅ **READY FOR IMPLEMENTATION**
