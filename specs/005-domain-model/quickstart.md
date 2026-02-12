# Quickstart: Internal Domain Model

**Feature**: 005-domain-model
**For**: Developers implementing or consuming the domain model
**Time to complete**: 5-10 minutes

## What You'll Learn

- How to construct a `PolicyDocument` from extraction outputs
- How the domain model fits into the FORGE pipeline
- How to write tests for domain model code
- How to extend the model for downstream work items

---

## Prerequisites

- Completed WI-2 (Markdown ingestion)
- Completed WI-3 (Section extraction)
- Completed WI-4 (Clause extraction)
- Rust stable 1.93.0+
- Familiarity with `cargo test`

---

## 1. Understanding the Domain Model

The domain model is the **canonical internal representation** of a parsed policy document. It decouples the extraction layer from OSCAL generation:

```
┌────────────────┐
│ Markdown Input │
└───────┬────────┘
        │ WI-2: Ingest
        ▼
┌────────────────────┐
│ IngestedDocument   │
└───────┬────────────┘
        │ WI-3: Extract Sections
        ▼
┌────────────────────┐     ┌──────────────────────┐
│ Vec<SectionNode>   │────▶│                      │
└────────────────────┘     │                      │
        │ WI-4: Extract Clauses │  WI-5: Assemble   │
        ▼                        │                      │
┌────────────────────┐     │  PolicyDocument      │
│ ExtractedContent   │────▶│                      │
└────────────────────┘     └──────────┬───────────┘
                                      │
                                      ▼
                           ┌────────────────────────┐
                           │ WI-6+: Downstream WIs  │
                           │ (UUID, OSCAL, etc.)    │
                           └────────────────────────┘
```

**Key Concept**: The domain model is **format-agnostic**. It has no Markdown-specific fields (no raw content) and no OSCAL-specific fields (no `control_id`). It's the clean boundary in the middle.

---

## 2. Core Structures

### PolicyDocument (Top-level)

```rust
pub struct PolicyDocument {
    pub id: String,                      // Document identifier
    pub metadata: DocumentMetadata,       // Title, version, author, etc.
    pub sections: Vec<PolicySection>,    // Hierarchical sections
}
```

### DocumentMetadata

```rust
pub struct DocumentMetadata {
    pub title: String,                   // From frontmatter or first H1 or filename
    pub version: String,                 // From frontmatter or "0.0.0"
    pub author: Option<String>,          // From frontmatter (optional)
    pub date: Option<String>,            // From frontmatter (optional)
    pub source_path: PathBuf,            // Source file path
    pub content_hash: Option<String>,    // SHA-256 hash (optional)
}
```

### PolicySection (Hierarchical)

```rust
pub struct PolicySection {
    pub title: String,                   // Heading text
    pub heading_level: u8,               // 1-6 (H1-H6)
    pub source_line: usize,              // 1-based line number
    pub body_text: Option<String>,       // Text content (optional)
    pub children: Vec<PolicySection>,    // Nested sections
    pub requirements: Vec<PolicyRequirement>, // Requirements in this section
}
```

### PolicyRequirement (Leaf)

```rust
pub struct PolicyRequirement {
    pub stable_id: Option<String>,       // UUID (None until WI-7)
    pub text: String,                    // Requirement text
    pub source_line: usize,              // 1-based line number
    pub nesting_depth: u8,               // 0 = top-level list item
}
```

---

## 3. Assembling a PolicyDocument

### Basic Usage

```rust
use forge::model::assemble_document;
use forge::ingest::ingest_file;
use forge::parse::{extract_sections, extract_clauses};

fn main() -> Result<(), ForgeError> {
    // Step 1: Ingest (WI-2)
    let ingested = ingest_file("policy.md")?;

    // Step 2: Extract sections (WI-3)
    let sections = extract_sections(&ingested.content)?;

    // Step 3: Extract clauses (WI-4)
    let clauses = extract_clauses(&ingested.content)?;

    // Step 4: Assemble domain model (WI-5)
    let document = assemble_document(&ingested, sections, clauses)?;

    // Use the domain model
    println!("Document: {} v{}", document.metadata.title, document.metadata.version);
    println!("Sections: {}", document.sections.len());

    Ok(())
}
```

### Handling Warnings

The assembly function emits warnings to stderr for recoverable issues:

```rust
let document = assemble_document(&ingested, sections, clauses)?;
// If YAML frontmatter is malformed:
// stderr: "Warning: Failed to parse YAML frontmatter: invalid syntax at line 3. Using fallback metadata."
// document.metadata.title will be from first H1 or filename
```

---

## 4. Working with the Domain Model

### Traversing the Section Tree

```rust
fn print_sections(sections: &[PolicySection], indent: usize) {
    for section in sections {
        println!("{:indent$}{} (level {})", "", section.title, section.heading_level);
        println!("{:indent$}  Requirements: {}", "", section.requirements.len());

        // Recurse into children
        print_sections(&section.children, indent + 2);
    }
}

print_sections(&document.sections, 0);
```

### Accessing Requirements

```rust
// Flatten all requirements from all sections
fn collect_all_requirements(sections: &[PolicySection]) -> Vec<&PolicyRequirement> {
    let mut all_reqs = Vec::new();

    for section in sections {
        all_reqs.extend(&section.requirements);
        all_reqs.extend(collect_all_requirements(&section.children));
    }

    all_reqs
}

let all_requirements = collect_all_requirements(&document.sections);
println!("Total requirements: {}", all_requirements.len());
```

### Checking for Temporary Identity

Before WI-7 assigns `stable_id`, you can identify requirements by `(source_line, text_hash)`:

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn temp_identity(req: &PolicyRequirement) -> (usize, u64) {
    let mut hasher = DefaultHasher::new();
    req.text[..req.text.len().min(64)].hash(&mut hasher);
    req.source_line.hash(&mut hasher);
    (req.source_line, hasher.finish())
}

for req in &section.requirements {
    let (line, hash) = temp_identity(req);
    println!("Requirement at line {}, hash {:x}", line, hash);
}
```

---

## 5. Writing Tests

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_document_with_frontmatter() {
        // Arrange
        let ingested = create_test_ingested_document(
            "policy.md",
            r#"---
title: "Security Policy"
version: "1.0"
---

# Access Control

- Users must authenticate
"#,
        );

        let sections = vec![
            SectionNode {
                title: "Access Control".to_string(),
                heading_level: 1,
                source_line: 6,
                body_text: None,
                children: vec![],
            },
        ];

        let clauses = ExtractedContent {
            list_items: vec![
                ExtractedListItem {
                    text: "Users must authenticate".to_string(),
                    source_line: 8,
                    nesting_depth: 0,
                },
            ],
            tables: vec![],
            paragraphs: vec![],
        };

        // Act
        let document = assemble_document(&ingested, sections, clauses).unwrap();

        // Assert
        assert_eq!(document.metadata.title, "Security Policy");
        assert_eq!(document.metadata.version, "1.0");
        assert_eq!(document.sections.len(), 1);
        assert_eq!(document.sections[0].requirements.len(), 1);
        assert_eq!(document.sections[0].requirements[0].text, "Users must authenticate");
        assert_eq!(document.sections[0].requirements[0].source_line, 8);
        assert!(document.sections[0].requirements[0].stable_id.is_none()); // Not yet populated by WI-7
    }

    #[test]
    fn test_assemble_document_with_malformed_yaml() {
        // Arrange: Document with malformed YAML frontmatter
        let ingested = create_test_ingested_document(
            "policy.md",
            r#"---
title: "Security Policy
version: 1.0
---

# Access Control
"#,
        );

        let sections = vec![
            SectionNode {
                title: "Access Control".to_string(),
                heading_level: 1,
                source_line: 6,
                body_text: None,
                children: vec![],
            },
        ];

        let clauses = ExtractedContent {
            list_items: vec![],
            tables: vec![],
            paragraphs: vec![],
        };

        // Act
        let document = assemble_document(&ingested, sections, clauses).unwrap();

        // Assert: Falls back to first H1 heading
        assert_eq!(document.metadata.title, "Access Control");
        assert_eq!(document.metadata.version, "0.0.0");
        // Expect warning to stderr (not asserted in test, but observable in test output)
    }
}
```

### Integration Test Example

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_pipeline_markdown_to_domain_model() {
        // Arrange: Real Markdown file
        let input_path = "tests/fixtures/sample_policy.md";

        // Act: Full pipeline
        let ingested = ingest_file(input_path).unwrap();
        let sections = extract_sections(&ingested.content).unwrap();
        let clauses = extract_clauses(&ingested.content).unwrap();
        let document = assemble_document(&ingested, sections, clauses).unwrap();

        // Assert: Verify all data preserved
        assert!(!document.sections.is_empty(), "Document should have sections");

        let total_requirements: usize = document.sections.iter()
            .map(|s| s.requirements.len())
            .sum();
        assert!(total_requirements > 0, "Document should have requirements");

        // Verify source line traceability
        for section in &document.sections {
            assert!(section.source_line >= 1, "Section source_line must be >= 1");
            for req in &section.requirements {
                assert!(req.source_line >= 1, "Requirement source_line must be >= 1");
            }
        }
    }
}
```

---

## 6. Extending for Downstream Work Items

### WI-6: Atomization

Atomization will consume `PolicyDocument` and return an enriched version with atomized requirements:

```rust
// WI-6 signature (example)
pub fn atomize_requirements(document: PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    // Takes ownership, returns enriched instance (functional transformation)
    let mut enriched = document;

    for section in &mut enriched.sections {
        let atomized = split_compound_requirements(&section.requirements);
        section.requirements = atomized;
    }

    Ok(enriched)
}
```

**Key**: WI-6 takes ownership and returns a new instance (functional transformation per Clarification Q1).

### WI-7: UUID Generation

UUID generation will populate `stable_id` fields:

```rust
// WI-7 signature (example)
pub fn assign_stable_ids(document: PolicyDocument) -> Result<PolicyDocument, ForgeError> {
    let mut enriched = document;

    for section in &mut enriched.sections {
        for req in &mut section.requirements {
            req.stable_id = Some(generate_stable_uuid(&req.text, req.source_line));
        }
    }

    Ok(enriched)
}
```

**After WI-7**: All `stable_id` fields are `Some(uuid_string)`.

### WI-9: OSCAL Generation

OSCAL generation will consume `PolicyDocument` and produce OSCAL JSON:

```rust
// WI-9 signature (example)
pub fn generate_oscal_catalog(document: &PolicyDocument) -> Result<OscalCatalog, ForgeError> {
    // Takes reference; does not consume document
    let mut catalog = OscalCatalog::new();

    for section in &document.sections {
        let group = map_section_to_oscal_group(section);
        catalog.groups.push(group);
    }

    Ok(catalog)
}
```

---

## 7. Common Patterns

### Pattern: Safe Option Unwrapping

Since `stable_id` is `Option<String>`, always handle the None case:

```rust
for req in &section.requirements {
    match &req.stable_id {
        Some(id) => println!("Requirement {}: {}", id, req.text),
        None => println!("Requirement (no ID yet): {}", req.text),
    }
}
```

### Pattern: Functional Pipeline Transformation

```rust
let document = assemble_document(&ingested, sections, clauses)?;
let atomized = atomize_requirements(document)?;
let with_uuids = assign_stable_ids(atomized)?;
let with_citations = extract_citations(with_uuids)?;

generate_oscal_catalog(&with_citations)?;
```

Each function takes ownership and returns an enriched instance. No in-place mutation.

### Pattern: Error Handling with Context

```rust
let document = assemble_document(&ingested, sections, clauses)
    .map_err(|e| ForgeError::Pipeline(format!("Failed to assemble domain model: {}", e)))?;
```

---

## 8. Troubleshooting

### Issue: "stable_id is None"

**Cause**: You're accessing `stable_id` before WI-7 has run.

**Solution**: Either handle the None case, or ensure your code runs after WI-7 in the pipeline.

### Issue: "Malformed YAML frontmatter warning"

**Cause**: YAML syntax error in frontmatter.

**Solution**: This is a warning, not an error. The assembly function falls back to heading-based metadata. Fix the YAML if you need frontmatter values.

### Issue: "Requirements missing from section"

**Cause**: Requirement's `source_line` doesn't fall within section's line range.

**Solution**: Verify extraction (WI-4) is correctly capturing `source_line` for list items.

---

## 9. Next Steps

- **Implement**: Follow TDD approach from AR testing strategy
- **Test**: Use test files in `tests/fixtures/` for integration tests
- **Extend**: Prepare for WI-6 (atomization) by understanding the functional transformation pattern
- **Document**: Add inline doc comments for public functions

---

## References

- [Feature Specification](./spec.md)
- [Architecture Review](../../docs/AR/005-ar-domain-model.md)
- [Security Review](../../docs/SEC/005-sec-domain-model.md)
- [Data Model](./data-model.md)
- [Rust Interface Contracts](./contracts/rust-interfaces.md)
