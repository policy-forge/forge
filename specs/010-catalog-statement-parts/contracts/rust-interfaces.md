# Rust Interfaces: OSCAL Catalog Statement Parts & Prose

**Branch**: `010-catalog-statement-parts` | **Date**: 2026-02-12

## New Types (`src/oscal/parts.rs`)

```rust
use serde::Serialize;

/// An OSCAL control part (statement, guidance, objective).
///
/// Parts carry the actual content of a control. Every control has at least
/// one statement part; guidance and objective parts are optional.
///
/// # OSCAL v1.2.0
///
/// Serializes to the OSCAL JSON `parts` array structure:
/// ```json
/// { "id": "POL-AC-001_smt", "name": "statement", "prose": "..." }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OscalPart {
    /// Part ID following `{control-id}_{suffix}` convention.
    /// Example: `"POL-AC-001_smt"`, `"POL-AC-001_gdn"`.
    pub id: String,

    /// OSCAL part name: `"statement"`, `"guidance"`, `"objective"`, or `"item"`.
    pub name: String,

    /// Human-readable text content. Direct copy from source (SEC-1).
    pub prose: String,

    /// Nested sub-parts (e.g., enumerated sub-items within a statement).
    /// Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<OscalPart>,

    /// Properties on this part.
    /// Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,
}

/// An OSCAL property for structured metadata on controls or parts.
///
/// # OSCAL v1.2.0
///
/// Serializes to the OSCAL JSON `props` array structure:
/// ```json
/// { "name": "forge:source-line", "value": "42" }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OscalProp {
    /// Property name. FORGE-specific names use `forge:` prefix.
    pub name: String,

    /// Property value as string.
    pub value: String,
}
```

## New Functions (`src/oscal/parts.rs`)

```rust
use crate::model::PolicyRequirement;

/// Generate a part ID from a control ID and a suffix.
///
/// Convention: `{control-id}_{suffix}`
///
/// # Examples
///
/// ```
/// use forge::oscal::parts::generate_part_id;
///
/// assert_eq!(generate_part_id("POL-AC-001", "smt"), "POL-AC-001_smt");
/// assert_eq!(generate_part_id("POL-DP-003", "gdn"), "POL-DP-003_gdn");
/// ```
#[must_use]
pub fn generate_part_id(control_id: &str, suffix: &str) -> String;

/// Generate statement parts (and optionally guidance parts) for a control.
///
/// Always produces at least one part with `name: "statement"` and `prose`
/// from `requirement.text` (SEC-1: direct copy, no transformation).
///
/// When `guidance_text` is `Some(non_empty_text)`, also produces a guidance
/// part with `name: "guidance"` and prose from the guidance text.
///
/// Logs `tracing::warn` if `requirement.text` is empty (EC-1).
///
/// # Arguments
///
/// * `control_id` — The control's ID (e.g., `"POL-AC-001"`)
/// * `requirement` — The source `PolicyRequirement`
/// * `guidance_text` — Optional guidance text from `PolicySection.body_text`
///
/// # Examples
///
/// ```
/// use forge::oscal::parts::build_control_parts;
/// use forge::model::PolicyRequirement;
///
/// let req = PolicyRequirement {
///     text: "All users must use MFA.".to_string(),
///     source_line: 42,
///     stable_id: Some("uuid-1".to_string()),
///     nesting_depth: 0,
///     atom_index: 0,
///     parent_text: None,
/// };
///
/// let parts = build_control_parts("POL-AC-001", &req, None);
/// assert_eq!(parts.len(), 1);
/// assert_eq!(parts[0].name, "statement");
/// assert_eq!(parts[0].prose, "All users must use MFA.");
/// assert_eq!(parts[0].id, "POL-AC-001_smt");
/// ```
#[must_use]
pub fn build_control_parts(
    control_id: &str,
    requirement: &PolicyRequirement,
    guidance_text: Option<&str>,
) -> Vec<OscalPart>;

/// Generate props for a control from a `PolicyRequirement`.
///
/// Only emits `forge:source-line` when `requirement.source_line > 0` (EC-6).
/// Never stores structured data in remarks (SEC-3).
///
/// # Arguments
///
/// * `requirement` — The source `PolicyRequirement`
///
/// # Examples
///
/// ```
/// use forge::oscal::parts::build_control_props;
/// use forge::model::PolicyRequirement;
///
/// let req = PolicyRequirement {
///     text: "Encrypt data.".to_string(),
///     source_line: 42,
///     stable_id: Some("uuid-1".to_string()),
///     nesting_depth: 0,
///     atom_index: 0,
///     parent_text: None,
/// };
///
/// let props = build_control_props(&req);
/// assert_eq!(props.len(), 1);
/// assert_eq!(props[0].name, "forge:source-line");
/// assert_eq!(props[0].value, "42");
/// ```
#[must_use]
pub fn build_control_props(requirement: &PolicyRequirement) -> Vec<OscalProp>;
```

## Modified Types

### `OscalControl` (`src/oscal/catalog.rs`)

```rust
/// OSCAL Control mapped from a [`PolicyRequirement`].
#[derive(Debug, Clone, Serialize)]
pub struct OscalControl {
    /// Control ID following `POL-{ABBR}-{NNN}` pattern.
    pub id: String,
    /// UUID copied from `PolicyRequirement.stable_id`.
    pub uuid: String,
    /// Derived title (first sentence, 120-char cap).
    pub title: String,
    /// Parts array: statement part (mandatory) + optional guidance/objective.
    /// NOT skip-serialized — always present per FR-001.
    pub parts: Vec<OscalPart>,   // NEW in WI-10
    /// Props array: structured metadata (e.g., forge:source-line).
    /// Omitted from JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<OscalProp>,   // NEW in WI-10
}
```

### `build_catalog` integration (`src/oscal/catalog.rs`)

The existing `build_catalog` function is modified to call `build_control_parts` and `build_control_props` for each requirement, passing the section's `body_text` as the guidance source:

```rust
// Inside build_catalog, in the requirement loop:
for (req_idx, req) in requirements.iter().enumerate() {
    // ... existing stable_id check ...

    let control_id = generate_control_id(&abbreviation, req_idx, "POL");

    controls.push(OscalControl {
        id: control_id.clone(),
        uuid: stable_id.clone(),
        title: derive_control_title(&req.text),
        parts: build_control_parts(
            &control_id,
            req,
            section.body_text.as_deref(),
        ),
        props: build_control_props(req),
    });
}
```

### `src/oscal/mod.rs` updates

```rust
pub mod catalog;
pub mod metadata;
pub mod parts;    // NEW in WI-10

pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};
pub use parts::{OscalPart, OscalProp, build_control_parts, build_control_props};  // NEW
```

### `src/lib.rs` updates

```rust
pub use oscal::{OscalMetadata, OscalPart, OscalProp, assemble_metadata};  // OscalPart, OscalProp NEW
```

## Contract Guarantees

1. `build_control_parts` ALWAYS returns at least one part with `name: "statement"` (FR-001)
2. Statement part `prose` is an exact copy of `requirement.text` — no transformation (SEC-1)
3. Part IDs follow `{control-id}_{suffix}` convention deterministically (FR-003)
4. `build_control_props` returns empty `Vec` when `source_line == 0` (EC-6)
5. No function performs I/O or has side effects beyond tracing (SEC-4)
6. `forge:` prefix used for all FORGE-specific prop names (SEC-5)
