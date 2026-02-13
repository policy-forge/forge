# Quickstart: Citation Extraction (WI-8)

## Usage

```rust
use forge::citation::extract_citations;
use forge::model::PolicyDocument;

// After WI-5 assembly and WI-7 UUID assignment:
let mut document: PolicyDocument = /* ... */;

// Extract citations from all requirements
extract_citations(&mut document).unwrap();

// Each requirement now has citations populated and text cleaned
for section in &document.sections {
    for req in &section.requirements {
        println!("Requirement: {}", req.text);  // Clean prose (citations removed)
        for cit in &req.citations {
            println!("  Citation: {} (url: {:?})", cit.text, cit.url);
        }
    }
}
```

## Lower-Level API

```rust
use forge::citation::extract_citations_from_text;

let (cleaned_text, citations) = extract_citations_from_text(
    "req-uuid-here",
    "Access must comply with https://example.com/policy requirements"
).unwrap();

assert_eq!(cleaned_text, "Access must comply with requirements");
assert_eq!(citations.len(), 1);
assert_eq!(citations[0].url, Some("https://example.com/policy".to_string()));
```

## Pipeline Position

```
Markdown → WI-2 Ingest → WI-3/4 Parse → WI-5 Assemble → WI-6 Atomize
    → WI-7 UUID → **WI-8 Citations** → WI-9 Catalog → WI-12 Back Matter
```

## Testing

```bash
cargo test citation           # Run citation extraction tests
cargo test --lib              # Run all library tests
```
