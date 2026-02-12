# Quickstart: OSCAL Catalog Statement Parts & Prose

**Branch**: `010-catalog-statement-parts` | **Date**: 2026-02-12

## Overview

WI-10 adds statement parts and metadata props to OSCAL controls. After this feature, `build_catalog` produces controls that contain the actual policy requirement text, not just IDs and titles.

---

## 1. Building Parts for a Single Control

```rust
use forge::oscal::parts::{build_control_parts, build_control_props};
use forge::model::PolicyRequirement;

let req = PolicyRequirement {
    text: "All users must use multi-factor authentication for privileged access.".to_string(),
    source_line: 42,
    stable_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
    nesting_depth: 0,
    atom_index: 0,
    parent_text: None,
};

// Statement part only (no guidance)
let parts = build_control_parts("POL-AC-001", &req, None);
assert_eq!(parts.len(), 1);
assert_eq!(parts[0].id, "POL-AC-001_smt");
assert_eq!(parts[0].name, "statement");
assert_eq!(parts[0].prose, req.text);

// Props
let props = build_control_props(&req);
assert_eq!(props.len(), 1);
assert_eq!(props[0].name, "forge:source-line");
assert_eq!(props[0].value, "42");
```

---

## 2. Multi-Part Control (with Guidance)

When a `PolicySection` has `body_text`, it becomes the guidance prose for all controls in that section:

```rust
use forge::oscal::parts::build_control_parts;
use forge::model::PolicyRequirement;

let req = PolicyRequirement {
    text: "Systems shall require MFA.".to_string(),
    source_line: 10,
    stable_id: Some("uuid-1".to_string()),
    nesting_depth: 0,
    atom_index: 0,
    parent_text: None,
};

let guidance = "Organizations should implement MFA using hardware tokens where possible.";

let parts = build_control_parts("POL-AC-001", &req, Some(guidance));
assert_eq!(parts.len(), 2);

// Statement part
assert_eq!(parts[0].name, "statement");
assert_eq!(parts[0].id, "POL-AC-001_smt");
assert_eq!(parts[0].prose, "Systems shall require MFA.");

// Guidance part
assert_eq!(parts[1].name, "guidance");
assert_eq!(parts[1].id, "POL-AC-001_gdn");
assert_eq!(parts[1].prose, guidance);
```

---

## 3. Full Catalog with Parts

```rust
use forge::oscal::catalog::{build_catalog, CatalogEnvelope};
use forge::model::{PolicyDocument, PolicySection, PolicyRequirement, DocumentMetadata};
use std::path::PathBuf;

let doc = PolicyDocument {
    id: "test-policy".to_string(),
    metadata: DocumentMetadata {
        title: "Security Policy".to_string(),
        version: "1.0".to_string(),
        ..Default::default()
    },
    sections: vec![PolicySection {
        title: "Access Control".to_string(),
        heading_level: 1,
        source_line: 1,
        body_text: Some("MFA implementation guidance.".to_string()),
        children: vec![],
        requirements: vec![PolicyRequirement {
            text: "All users must use MFA.".to_string(),
            source_line: 5,
            stable_id: Some("uuid-1".to_string()),
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
        }],
    }],
};

let catalog = build_catalog(&doc).unwrap();
let control = &catalog.groups[0].controls[0];

// Control has parts
assert_eq!(control.parts.len(), 2);  // statement + guidance
assert_eq!(control.parts[0].name, "statement");
assert_eq!(control.parts[1].name, "guidance");

// Control has props
assert_eq!(control.props.len(), 1);
assert_eq!(control.props[0].name, "forge:source-line");
assert_eq!(control.props[0].value, "5");

// JSON output
let envelope = CatalogEnvelope { catalog };
let json = serde_json::to_string_pretty(&envelope).unwrap();
println!("{json}");
```

### Expected JSON output:

```json
{
  "catalog": {
    "uuid": "00000000-0000-0000-0000-000000000000",
    "metadata": {
      "title": "placeholder",
      "last-modified": "1970-01-01T00:00:00Z",
      "version": "0.0.0",
      "oscal-version": "1.2.0"
    },
    "groups": [
      {
        "id": "access-control",
        "title": "Access Control",
        "controls": [
          {
            "id": "POL-AC-001",
            "uuid": "uuid-1",
            "title": "All users must use MFA.",
            "parts": [
              {
                "id": "POL-AC-001_smt",
                "name": "statement",
                "prose": "All users must use MFA."
              },
              {
                "id": "POL-AC-001_gdn",
                "name": "guidance",
                "prose": "MFA implementation guidance."
              }
            ],
            "props": [
              {
                "name": "forge:source-line",
                "value": "5"
              }
            ]
          }
        ]
      }
    ]
  }
}
```

---

## 4. Edge Cases

### Empty requirement text (EC-1)

```rust
let req = PolicyRequirement {
    text: String::new(),  // empty
    source_line: 10,
    stable_id: Some("uuid".to_string()),
    ..Default::default()
};

let parts = build_control_parts("POL-AC-001", &req, None);
assert_eq!(parts[0].prose, "");  // empty prose, warning logged
```

### Source line 0 — no prop generated (EC-6)

```rust
let req = PolicyRequirement {
    text: "Some requirement.".to_string(),
    source_line: 0,  // unknown
    ..Default::default()
};

let props = build_control_props(&req);
assert!(props.is_empty());  // no forge:source-line prop
```
