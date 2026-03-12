# Code Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix 18 code review issues (#68–#85) across OSCAL model completeness, XML round-trip, consistency, infrastructure, path sanitization, and behavioral fixes, plus 49 clippy warnings.

**Architecture:** Extend existing typed OSCAL structs for full v1.2.0 field coverage (Option A). Add shared `src/io.rs` utility module for atomic writes, size guardrails, and path sanitization. All changes are additive to existing structs — new fields use serde defaults so generation code is unaffected.

**Tech Stack:** Rust 1.93.0, serde 1.0.228, quick-xml 0.37, uuid 1.20.0, sha2 0.10.9, tempfile 3.25.0 (promoted from dev-dep), chrono 0.4

**Spec:** `docs/specs/2026-03-12-code-review-fixes-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/io.rs` | `write_atomic()`, `check_file_size()`, `sanitize_artifact_path()` utilities |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Promote `tempfile` from dev to production dependency |
| `src/lib.rs` | Register `io` module |
| `src/error.rs` | Add `AmbiguousArtifact` variant |
| `src/oscal/catalog.rs` | Add root `controls` to `OscalCatalog`, `groups` to `OscalGroup`, stable control IDs |
| `src/oscal/back_matter.rs` | Add `ns` to `Prop` |
| `src/oscal/component_definition.rs` | Add `Capability` struct, `capabilities` field |
| `src/oscal/profile.rs` | Deterministic UUID v5, path sanitization |
| `src/oscal/assessment_plan.rs` | Path sanitization |
| `src/oscal/implemented_requirements.rs` | Path sanitization |
| `src/export/xml_serializer.rs` | Implement control-implementation XML write |
| `src/export/xml_deserializer.rs` | Implement control-implementation XML read, reject invalid UUIDs |
| `src/cli/convert.rs` | Require `--source-profile` for component, respect `--quiet` in batch |
| `src/cli/export.rs` | Use full validation, add size guardrail |
| `src/cli/mod.rs` | Add `--timestamp` to profile subcommand |
| `src/cli/profile.rs` | Use atomic write, path sanitization |
| `src/cli/trace.rs` | Use atomic write |
| `src/pipeline.rs` | Use atomic write |
| `src/validate/mod.rs` | Add ambiguity detection to `detect_model_type()` |
| `src/diff/mod.rs` | Add size guardrail |
| `src/diff/extractor.rs` | Add capabilities extraction |
| `src/trace/mod.rs` | Add size guardrail |
| `src/summary/mod.rs` | Recurse nested groups + root controls |
| `README.md` | Fix component example, keep round-trip claim |
| Various test files | Clippy warning fixes |

---

## Execution Order Note

The spec declares Theme 1 → Theme 2 → Themes 3-6 → Clippy. This plan reorders slightly: **Infrastructure (Theme 4) comes first** because it creates `src/io.rs` with utilities (`write_atomic`, `check_file_size`, `sanitize_artifact_path`) that Themes 1, 2, 3, and 5 depend on. All other ordering matches the spec.

## Chunk 1: Infrastructure — `src/io.rs` (Theme 4, Issues #77, #78)

Creates the shared utility module used by later tasks.

### Task 1: Create `src/io.rs` with `write_atomic()`

**Files:**
- Create: `src/io.rs`
- Modify: `src/lib.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Promote tempfile to production dependency**

In `Cargo.toml`, move `tempfile` from `[dev-dependencies]` to `[dependencies]`:

```toml
# Add under [dependencies]
tempfile = "3.25.0"
```

Remove `tempfile = "3.25.0"` from `[dev-dependencies]`.

- [ ] **Step 2: Register io module**

In `src/lib.rs`, add after `pub mod ingest;`:

```rust
pub mod io;
```

- [ ] **Step 3: Write the failing test for `write_atomic`**

Create `src/io.rs` with the test:

```rust
//! Shared I/O utilities: atomic writes, size guardrails, path sanitization.

use std::path::Path;

use crate::error::ForgeError;

/// Maximum file size for all file-reading operations (50 MB).
pub const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Write content to a file atomically using temp-file + rename.
///
/// Writes to a `NamedTempFile` in the same directory as `path`,
/// then atomically renames. On failure, the temp file is cleaned up.
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), ForgeError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_file_with_correct_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        write_atomic(&path, b"hello world").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn write_atomic_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.json");
        std::fs::write(&path, "old").unwrap();
        write_atomic(&path, b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

    #[test]
    fn write_atomic_fails_on_nonexistent_parent() {
        let path = Path::new("/nonexistent/dir/out.json");
        assert!(write_atomic(path, b"data").is_err());
    }
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test --lib io::tests -- --exact`
Expected: FAIL with `not yet implemented`

- [ ] **Step 5: Implement `write_atomic`**

Replace `todo!()` with:

```rust
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<(), ForgeError> {
    use std::io::Write;

    let parent = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.persist(path).map_err(|e| {
        ForgeError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to persist temp file to '{}': {e}", path.display()),
        ))
    })?;
    Ok(())
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test --lib io::tests -- --exact`
Expected: PASS (3 tests)

- [ ] **Step 7: Commit**

```bash
git add src/io.rs src/lib.rs Cargo.toml
git commit -m "feat: add write_atomic utility in src/io.rs (#77)"
```

### Task 2: Add `check_file_size()` to `src/io.rs`

**Files:**
- Modify: `src/io.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/io.rs`:

```rust
/// Check that a file does not exceed `max_bytes` before reading.
///
/// Uses file metadata (no read) to check size. Returns the file size on success.
pub fn check_file_size(path: &Path, max_bytes: u64) -> Result<u64, ForgeError> {
    todo!()
}
```

Add tests:

```rust
#[test]
fn check_file_size_accepts_small_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("small.json");
    std::fs::write(&path, "{}").unwrap();
    assert!(check_file_size(&path, 1024).is_ok());
}

#[test]
fn check_file_size_rejects_oversized_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("big.json");
    std::fs::write(&path, vec![b'x'; 100]).unwrap();
    let result = check_file_size(&path, 50);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib io::tests::check_file_size -- --exact`
Expected: FAIL with `not yet implemented`

- [ ] **Step 3: Implement `check_file_size`**

```rust
pub fn check_file_size(path: &Path, max_bytes: u64) -> Result<u64, ForgeError> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();
    if size > max_bytes {
        return Err(ForgeError::FileTooLarge {
            path: path.to_path_buf(),
            size_bytes: size,
            limit_bytes: max_bytes,
        });
    }
    Ok(size)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib io::tests::check_file_size`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/io.rs
git commit -m "feat: add check_file_size utility in src/io.rs (#78)"
```

### Task 3: Add `sanitize_artifact_path()` to `src/io.rs`

**Files:**
- Modify: `src/io.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/io.rs`:

```rust
/// Extract filename from a path to prevent absolute path leaks in OSCAL artifacts.
///
/// Returns the file name component only. Falls back to the full path string
/// if `file_name()` returns `None` (e.g., root paths like `/`).
#[must_use]
pub fn sanitize_artifact_path(path: &Path) -> String {
    todo!()
}
```

Add tests:

```rust
#[test]
fn sanitize_extracts_filename_from_absolute_path() {
    let path = Path::new("/home/user/docs/catalog.json");
    assert_eq!(sanitize_artifact_path(path), "catalog.json");
}

#[test]
fn sanitize_preserves_bare_filename() {
    let path = Path::new("catalog.json");
    assert_eq!(sanitize_artifact_path(path), "catalog.json");
}

#[test]
fn sanitize_handles_relative_path() {
    let path = Path::new("../docs/catalog.json");
    assert_eq!(sanitize_artifact_path(path), "catalog.json");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib io::tests::sanitize`
Expected: FAIL with `not yet implemented`

- [ ] **Step 3: Implement `sanitize_artifact_path`**

```rust
#[must_use]
pub fn sanitize_artifact_path(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib io::tests::sanitize`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add src/io.rs
git commit -m "feat: add sanitize_artifact_path utility in src/io.rs (#79)"
```

### Task 4: Wire `write_atomic` into output paths

**Files:**
- Modify: `src/pipeline.rs:38`
- Modify: `src/cli/profile.rs:120`
- Modify: `src/cli/trace.rs:20`

- [ ] **Step 1: Update `pipeline.rs` — `write_output()`**

In `src/pipeline.rs:38`, replace:

```rust
std::fs::write(path, content)?;
```

with:

```rust
crate::io::write_atomic(path, content.as_bytes())?;
```

- [ ] **Step 2: Update `cli/profile.rs`**

In `src/cli/profile.rs`, replace the `std::fs::write(path, serialized)?` call with:

```rust
crate::io::write_atomic(path, serialized.as_bytes())?;
```

- [ ] **Step 3: Update `cli/trace.rs`**

In `src/cli/trace.rs`, replace `std::fs::write(path, &table).map_err(ForgeError::Io)?` with:

```rust
crate::io::write_atomic(path, table.as_bytes())?;
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All 1387+ tests pass

- [ ] **Step 5: Commit**

```bash
git add src/pipeline.rs src/cli/profile.rs src/cli/trace.rs
git commit -m "refactor: use write_atomic for all file output (#77)"
```

### Task 5: Wire `check_file_size` into reading paths

**Files:**
- Modify: `src/cli/export.rs:267`
- Modify: `src/diff/mod.rs:66`
- Modify: `src/trace/mod.rs:81`

- [ ] **Step 1: Add size check to `export.rs`**

In `src/cli/export.rs`, before the `std::fs::read(input_path)?` call, add:

```rust
crate::io::check_file_size(input_path, crate::io::MAX_FILE_SIZE)?;
```

- [ ] **Step 2: Add size check to `diff/mod.rs`**

In `src/diff/mod.rs`, at the start of `read_diff_file()`, add:

```rust
crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE)?;
```

- [ ] **Step 3: Add size check to `trace/mod.rs`**

In `src/trace/mod.rs`, at the start of `read_file()`, add:

```rust
crate::io::check_file_size(path, crate::io::MAX_FILE_SIZE)?;
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/cli/export.rs src/diff/mod.rs src/trace/mod.rs
git commit -m "feat: add file size guardrails to export, diff, trace (#78)"
```

---

## Chunk 2: OSCAL Model Completeness (Theme 1, Issues #70, #71, #82)

### Task 6: Add root-level `controls` to `OscalCatalog`

**Files:**
- Modify: `src/oscal/catalog.rs:31-42`

- [ ] **Step 1: Write the failing test**

Add to the test module in `src/oscal/catalog.rs`:

```rust
#[test]
fn catalog_round_trips_root_level_controls() {
    let json = r#"{
        "catalog": {
            "uuid": "test-uuid",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            },
            "controls": [
                {"id": "ctrl-1", "title": "Root Control"}
            ]
        }
    }"#;
    let envelope: CatalogEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(envelope.catalog.controls.len(), 1);
    assert_eq!(envelope.catalog.controls[0].id, "ctrl-1");

    // Round-trip
    let reserialized = serde_json::to_string(&envelope).unwrap();
    let re_parsed: CatalogEnvelope = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(re_parsed.catalog.controls.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib oscal::catalog::tests::catalog_round_trips_root_level_controls`
Expected: FAIL (no `controls` field on `OscalCatalog`)

- [ ] **Step 3: Add `controls` field to `OscalCatalog`**

In `src/oscal/catalog.rs`, add after the `groups` field (line 38):

```rust
    /// Root-level controls (not inside any group). OSCAL v1.2.0 allows this.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<OscalControl>,
```

- [ ] **Step 4: Fix `build_catalog()` — add `controls: vec![]` to the struct literal**

In `src/oscal/catalog.rs` around line 420, update the `OscalCatalog` construction to include `controls: vec![]`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib oscal::catalog::tests::catalog_round_trips_root_level_controls`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/oscal/catalog.rs
git commit -m "feat: add root-level controls to OscalCatalog (#71)"
```

### Task 7: Add nested `groups` to `OscalGroup`

**Files:**
- Modify: `src/oscal/catalog.rs:45-60`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn catalog_round_trips_nested_groups() {
    let json = r#"{
        "catalog": {
            "uuid": "test-uuid",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            },
            "groups": [{
                "id": "parent",
                "title": "Parent",
                "groups": [{
                    "id": "child",
                    "title": "Child",
                    "controls": [{"id": "ctrl-1", "title": "Nested"}]
                }]
            }]
        }
    }"#;
    let envelope: CatalogEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(envelope.catalog.groups[0].groups.len(), 1);
    assert_eq!(envelope.catalog.groups[0].groups[0].controls.len(), 1);

    let reserialized = serde_json::to_string(&envelope).unwrap();
    let re_parsed: CatalogEnvelope = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(re_parsed.catalog.groups[0].groups.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib oscal::catalog::tests::catalog_round_trips_nested_groups`
Expected: FAIL

- [ ] **Step 3: Add `groups` field to `OscalGroup`**

In `src/oscal/catalog.rs`, add after the `controls` field in `OscalGroup`:

```rust
    /// Nested sub-groups. OSCAL v1.2.0 allows groups within groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<OscalGroup>,
```

- [ ] **Step 4: Fix `build_catalog()` — add `groups: vec![]` to OscalGroup construction**

In the `groups.push(OscalGroup { ... })` block, add `groups: vec![]`.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib oscal::catalog::tests::catalog_round_trips_nested_groups`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/oscal/catalog.rs
git commit -m "feat: add nested groups to OscalGroup (#71)"
```

### Task 8: Update `count_catalog_controls()` for nested groups + root controls

**Files:**
- Modify: `src/summary/mod.rs:87-95`

- [ ] **Step 1: Write the failing test**

Add to `src/summary/mod.rs` test module:

```rust
#[test]
fn count_catalog_controls_includes_root_and_nested() {
    use crate::oscal::catalog::{OscalCatalog, OscalGroup, OscalControl, OscalMetadata};

    let catalog = OscalCatalog {
        uuid: "test".to_string(),
        metadata: OscalMetadata {
            title: "T".to_string(),
            last_modified: "2026-01-01T00:00:00Z".to_string(),
            version: "1.0".to_string(),
            oscal_version: "1.2.0".to_string(),
        },
        controls: vec![OscalControl {
            id: "root-1".to_string(),
            uuid: String::new(),
            title: "Root".to_string(),
            links: vec![], params: vec![], parts: vec![], props: vec![],
        }],
        groups: vec![OscalGroup {
            id: "g1".to_string(),
            title: "G1".to_string(),
            props: vec![], links: vec![],
            controls: vec![OscalControl {
                id: "g1-1".to_string(),
                uuid: String::new(),
                title: "G1C1".to_string(),
                links: vec![], params: vec![], parts: vec![], props: vec![],
            }],
            groups: vec![OscalGroup {
                id: "g1-sub".to_string(),
                title: "G1 Sub".to_string(),
                props: vec![], links: vec![], controls: vec![
                    OscalControl {
                        id: "g1s-1".to_string(),
                        uuid: String::new(),
                        title: "Nested".to_string(),
                        links: vec![], params: vec![], parts: vec![], props: vec![],
                    },
                ],
                groups: vec![],
            }],
        }],
        back_matter: None,
    };
    // 1 root + 1 in group + 1 in nested group = 3
    assert_eq!(count_catalog_controls(&catalog), 3);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib summary::tests::count_catalog_controls_includes_root_and_nested`
Expected: FAIL (returns 1 instead of 3)

- [ ] **Step 3: Update `count_catalog_controls` to recurse**

Replace the function body:

```rust
pub fn count_catalog_controls(catalog: &OscalCatalog) -> usize {
    fn count_group_controls(groups: &[crate::oscal::catalog::OscalGroup]) -> usize {
        groups.iter().map(|g| g.controls.len() + count_group_controls(&g.groups)).sum()
    }
    catalog.controls.len() + count_group_controls(&catalog.groups)
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib summary::tests::count_catalog_controls`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/summary/mod.rs
git commit -m "fix: count_catalog_controls recurses nested groups and root controls (#71)"
```

### Task 9: Add `ns` to `back_matter::Prop`

**Files:**
- Modify: `src/oscal/back_matter.rs:86-96`

- [ ] **Step 1: Write the failing test**

Add to `src/oscal/back_matter.rs` test module:

```rust
#[test]
fn prop_round_trips_namespace() {
    let json = r#"{"name":"custom","value":"val","ns":"https://example.com/ns"}"#;
    let prop: Prop = serde_json::from_str(json).unwrap();
    assert_eq!(prop.ns.as_deref(), Some("https://example.com/ns"));

    let reserialized = serde_json::to_string(&prop).unwrap();
    assert!(reserialized.contains("https://example.com/ns"));
}

#[test]
fn prop_omits_ns_when_none() {
    let prop = Prop { name: "x".to_string(), value: "y".to_string(), ns: None };
    let json = serde_json::to_string(&prop).unwrap();
    assert!(!json.contains("ns"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib oscal::back_matter::tests::prop_round_trips_namespace`
Expected: FAIL (no `ns` field)

- [ ] **Step 3: Add `ns` field to `Prop`**

In `src/oscal/back_matter.rs`, update the `Prop` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prop {
    pub name: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}
```

- [ ] **Step 4: Fix any compilation errors from existing Prop constructors**

Search for `Prop {` in `back_matter.rs` and add `ns: None` where needed.

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test --lib oscal::back_matter::tests::prop_round_trips_namespace`
Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add src/oscal/back_matter.rs
git commit -m "feat: add ns field to back_matter::Prop (#82)"
```

### Task 10: Add `Capability` struct and `capabilities` field to `ComponentDefinition`

**Files:**
- Modify: `src/oscal/component_definition.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn component_definition_round_trips_capabilities() {
    let json = r#"{
        "component-definition": {
            "uuid": "test-uuid",
            "metadata": {
                "title": "Test",
                "last-modified": "2026-01-01T00:00:00Z",
                "version": "1.0",
                "oscal-version": "1.2.0"
            },
            "components": [],
            "capabilities": [{
                "uuid": "cap-uuid",
                "name": "Encryption Capability",
                "description": "Provides data-at-rest encryption"
            }]
        }
    }"#;
    let envelope: ComponentDefinitionEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(envelope.component_definition.capabilities.len(), 1);
    assert_eq!(envelope.component_definition.capabilities[0].name, "Encryption Capability");

    let reserialized = serde_json::to_string(&envelope).unwrap();
    let re_parsed: ComponentDefinitionEnvelope = serde_json::from_str(&reserialized).unwrap();
    assert_eq!(re_parsed.component_definition.capabilities.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL (no `capabilities` field)

- [ ] **Step 3: Add `Capability` struct and field**

In `src/oscal/component_definition.rs`, add the struct:

```rust
/// OSCAL Capability within a Component Definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier for this capability.
    pub uuid: String,

    /// Capability name.
    pub name: String,

    /// Capability description.
    pub description: String,

    /// Control implementations under this capability.
    #[serde(default, rename = "control-implementations", skip_serializing_if = "Vec::is_empty")]
    pub control_implementations: Vec<crate::oscal::implemented_requirements::ControlImplementation>,
}
```

Add to `ComponentDefinition`:

```rust
    /// Capabilities grouping control implementations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<Capability>,
```

- [ ] **Step 4: Fix `build_component_definition()` — add `capabilities: vec![]` to struct literal**

- [ ] **Step 5: Run test to verify it passes**

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/oscal/component_definition.rs
git commit -m "feat: add Capability struct and capabilities field (#70, #81)"
```

---

## Chunk 3: XML Component Round-Trip (Theme 2, Issues #68, #69, #72)

### Task 11: Implement XML serialization for control-implementations

**Files:**
- Modify: `src/export/xml_serializer.rs`

- [ ] **Step 1: Read `xml_serializer.rs` to understand the `write_component()` function**

Read: `src/export/xml_serializer.rs` around line 307
Understand the `quick_xml::Writer` pattern used in existing functions.

- [ ] **Step 2: Implement `write_control_implementation()` and `write_implemented_requirement()`**

After the existing `write_component()` function, add two new functions following the same `Writer<W>` pattern:

```rust
fn write_control_implementation<W: std::io::Write>(
    writer: &mut Writer<W>,
    ci: &crate::oscal::implemented_requirements::ControlImplementation,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("control-implementation");
    elem.push_attribute(("uuid", ci.uuid.as_str()));
    elem.push_attribute(("source", ci.source.as_str()));
    writer.write_event(Event::Start(elem))?;

    write_text_element(writer, "description", &ci.description)?;

    for ir in &ci.implemented_requirements {
        write_implemented_requirement(writer, ir)?;
    }

    writer.write_event(Event::End(BytesEnd::new("control-implementation")))?;
    Ok(())
}

fn write_implemented_requirement<W: std::io::Write>(
    writer: &mut Writer<W>,
    ir: &crate::oscal::implemented_requirements::ImplementedRequirement,
) -> Result<(), ForgeError> {
    let mut elem = BytesStart::new("implemented-requirement");
    elem.push_attribute(("uuid", ir.uuid.as_str()));
    elem.push_attribute(("control-id", ir.control_id.as_str()));
    writer.write_event(Event::Start(elem))?;

    write_text_element(writer, "description", &ir.description)?;

    for prop in &ir.props {
        write_prop(writer, prop)?;
    }
    for link in &ir.links {
        write_link(writer, link)?;
    }

    writer.write_event(Event::End(BytesEnd::new("implemented-requirement")))?;
    Ok(())
}
```

- [ ] **Step 3: Update `write_component()` to call `write_control_implementation()`**

Remove the WI-26 skip comment. After writing props in `write_component()`, add:

```rust
for ci in &component.control_implementations {
    write_control_implementation(writer, ci)?;
}
```

- [ ] **Step 4: Run existing tests**

Run: `cargo test --lib export`
Expected: PASS (existing tests still work)

- [ ] **Step 5: Commit**

```bash
git add src/export/xml_serializer.rs
git commit -m "feat: implement XML serialization for control-implementations (#68)"
```

### Task 12: Implement XML deserialization for control-implementations

**Files:**
- Modify: `src/export/xml_deserializer.rs`

- [ ] **Step 1: Read `xml_deserializer.rs` to understand `convert_component()` and the XML struct pattern**

Read: `src/export/xml_deserializer.rs` around line 316

- [ ] **Step 2: Add XML deserialization structs**

Add before `convert_component()`:

```rust
#[derive(Deserialize)]
struct XmlControlImplementation {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(rename = "@source")]
    source: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "implemented-requirement")]
    implemented_requirements: Vec<XmlImplementedRequirement>,
}

#[derive(Deserialize)]
struct XmlImplementedRequirement {
    #[serde(rename = "@uuid")]
    uuid: String,
    #[serde(rename = "@control-id")]
    control_id: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "prop")]
    props: Vec<XmlProp>,
    #[serde(default, rename = "link")]
    links: Vec<XmlLink>,
}
```

- [ ] **Step 3: Update `convert_component()` to parse control-implementations**

Replace `control_implementations: vec![]` with actual parsing:

```rust
control_implementations: xml_component
    .control_implementations
    .into_iter()
    .map(|ci| {
        let uuid = Uuid::try_parse(&ci.uuid).map_err(|_| {
            ForgeError::ExportInvalidOscal {
                detail: format!("invalid UUID '{}' in control-implementation element", ci.uuid),
            }
        })?;
        Ok(crate::oscal::implemented_requirements::ControlImplementation {
            uuid: uuid.to_string(),
            source: ci.source,
            description: ci.description.unwrap_or_default(),
            implemented_requirements: ci.implemented_requirements.into_iter().map(|ir| {
                let ir_uuid = Uuid::try_parse(&ir.uuid).map_err(|_| {
                    ForgeError::ExportInvalidOscal {
                        detail: format!("invalid UUID '{}' in implemented-requirement", ir.uuid),
                    }
                })?;
                Ok(crate::oscal::implemented_requirements::ImplementedRequirement {
                    uuid: ir_uuid.to_string(),
                    control_id: ir.control_id,
                    description: ir.description.unwrap_or_default(),
                    props: ir.props.iter().map(convert_prop).collect(),
                    links: ir.links.iter().map(convert_link).collect(),
                })
            }).collect::<Result<Vec<_>, ForgeError>>()?,
        })
    })
    .collect::<Result<Vec<_>, ForgeError>>()?,
```

Also add `control-implementation` to the `XmlComponent` struct's fields.

- [ ] **Step 4: Write XML round-trip integration test**

Create a test that serializes a component with control-implementations to XML, then deserializes back, verifying all fields survive.

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/export/xml_deserializer.rs
git commit -m "feat: implement XML deserialization for control-implementations (#68)"
```

### Task 13: Require `--source-profile` for component strategy

**Files:**
- Modify: `src/cli/convert.rs:74-81`

- [ ] **Step 1: Write the failing test**

In the integration tests, add:

```rust
#[test]
fn component_without_source_profile_returns_error() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["convert", "tests/fixtures/sample_policy.md", "--strategy", "component", "--format", "json"])
        .output()
        .unwrap();
    assert!(!result.status.success());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("--source-profile is required"));
}
```

- [ ] **Step 2: Update `resolve_source_profile()` to return error**

In `src/cli/convert.rs`, change the `None` arm:

```rust
None => {
    Err(ForgeError::InvalidArgument(
        "--source-profile is required for component definitions to produce schema-valid output".to_string()
    ))
}
```

- [ ] **Step 3: Run test to verify it passes**

Run the new integration test.
Expected: PASS

- [ ] **Step 4: Fix any existing tests that rely on component without `--source-profile`**

Search for tests using `--strategy component` without `--source-profile` and update them.

- [ ] **Step 5: Commit**

```bash
git add src/cli/convert.rs tests/
git commit -m "fix: require --source-profile for component strategy (#72)"
```

### Task 14: Update README

**Files:**
- Modify: `README.md:64-65`

- [ ] **Step 1: Update component example to include `--source-profile`**

Change line 64-65 from:

```bash
# Convert to an OSCAL Component Definition
./target/release/forge convert tests/fixtures/sample_policy.md --strategy component --format json
```

to:

```bash
# Convert to an OSCAL Component Definition
./target/release/forge convert tests/fixtures/sample_policy.md --strategy component --format json --source-profile profile.json
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: update README component example with --source-profile (#69)"
```

---

## Chunk 4: Consistency Fixes (Theme 3, Issues #73, #74, #75, #81)

### Task 15: Add ambiguity detection to `detect_model_type()`

**Files:**
- Modify: `src/validate/mod.rs`
- Modify: `src/error.rs`

- [ ] **Step 1: Add `AmbiguousArtifact` error variant**

In `src/error.rs`, add after `TraceUnsupportedArtifact`:

```rust
#[error("Ambiguous OSCAL artifact: file contains multiple model types ({detail}). Each file must contain exactly one OSCAL model.")]
AmbiguousArtifact { detail: String },
```

- [ ] **Step 2: Write the failing test**

In `src/validate/mod.rs` tests:

```rust
#[test]
fn detect_model_type_rejects_ambiguous_artifact() {
    let json: serde_json::Value = serde_json::json!({
        "catalog": {"uuid": "x", "metadata": {}},
        "component-definition": {"uuid": "y", "metadata": {}}
    });
    let result = detect_model_type(&json);
    assert!(result.is_err());
}
```

- [ ] **Step 3: Update `detect_model_type()` to check for ambiguity**

Before returning the detected type, count how many OSCAL root keys are present. If > 1, return error:

```rust
let mut found = Vec::new();
if json.get("catalog").is_some() { found.push("catalog"); }
if json.get("component-definition").is_some() { found.push("component-definition"); }
if json.get("profile").is_some() { found.push("profile"); }
if found.len() > 1 {
    return Err(ForgeError::AmbiguousArtifact {
        detail: found.join(", "),
    });
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS (some existing tests may need updating if they relied on ambiguous inputs)

- [ ] **Step 5: Commit**

```bash
git add src/error.rs src/validate/mod.rs
git commit -m "fix: detect_model_type rejects ambiguous artifacts (#75)"
```

### Task 16: Reject invalid UUIDs in XML deserializer

**Files:**
- Modify: `src/export/xml_deserializer.rs:291`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn xml_deserialize_rejects_invalid_resource_uuid() {
    let xml = r#"<catalog xmlns="http://csrc.nist.gov/ns/oscal/1.0" uuid="valid-uuid">
        <metadata><title>T</title><last-modified>2026-01-01T00:00:00Z</last-modified><version>1.0</version><oscal-version>1.2.0</oscal-version></metadata>
        <back-matter><resource uuid="not-a-uuid"><title>R</title></resource></back-matter>
    </catalog>"#;
    let result = deserialize_catalog_from_xml(xml);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Change `unwrap_or_else` to error**

In `convert_resource()`, replace:

```rust
let uuid = Uuid::try_parse(&xml.uuid).unwrap_or_else(|_| Uuid::new_v4());
```

with:

```rust
let uuid = Uuid::try_parse(&xml.uuid).map_err(|_| ForgeError::ExportInvalidOscal {
    detail: format!("invalid UUID '{}' in resource element", xml.uuid),
})?;
```

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/export/xml_deserializer.rs
git commit -m "fix: reject invalid UUIDs in XML deserializer (#74)"
```

### Task 17: Use full validation in export

**Files:**
- Modify: `src/cli/export.rs`

- [ ] **Step 1: Locate the schema-only validation call in export**

Read `src/cli/export.rs` around the `validate_oscal_model()` function.

- [ ] **Step 2: Replace with `run_full_validation()`**

Update the validation call to use `run_full_validation()` instead of `validate_artifact()`, matching the pipeline's approach.

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/cli/export.rs
git commit -m "fix: export uses full semantic validation (#73)"
```

### Task 18: Add capabilities extraction to diff and AP

**Files:**
- Modify: `src/diff/extractor.rs:77`
- Modify: Assessment plan generation code

- [ ] **Step 1: Read `trace/walker.rs:129` to see the capabilities pattern**

- [ ] **Step 2: Update `extract_component_def_controls()` in `diff/extractor.rs`**

After the existing `components` iteration, add:

```rust
if let Some(capabilities) = json.pointer("/component-definition/capabilities")
    .and_then(Value::as_array)
{
    for capability in capabilities {
        // Same extraction pattern as components
        if let Some(impls) = capability.get("control-implementations").and_then(Value::as_array) {
            for ci in impls {
                // ... extract implemented-requirements same as components path
            }
        }
    }
}
```

- [ ] **Step 3: Write test with capabilities data**

Create a diff test fixture with `capabilities[]` containing implemented-requirements, verify they appear in diff output.

- [ ] **Step 4: Update AP generation similarly**

In `src/oscal/assessment_plan.rs`, the `build_assessment_plan()` function receives `control_ids` already extracted by the caller. The caller is in `src/pipeline.rs` (search for `build_assessment_plan`). The extraction happens in `src/oscal/component_definition.rs` where control IDs are gathered from `components[].control_implementations[].implemented_requirements[].control_id`. Add capabilities walking there:

```rust
// After iterating components for control IDs, also iterate capabilities
for capability in &compdef.capabilities {
    for ci in &capability.control_implementations {
        for ir in &ci.implemented_requirements {
            control_ids.push(ir.control_id.clone());
        }
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/diff/extractor.rs src/oscal/assessment_plan.rs
git commit -m "feat: diff and AP support capabilities[] in component definitions (#81)"
```

---

## Chunk 5: Path Sanitization (Theme 5, Issue #79)

### Task 19: Apply `sanitize_artifact_path` to profile generation

**Files:**
- Modify: `src/cli/profile.rs:96`
- Modify: `src/oscal/profile.rs:231-232`

- [ ] **Step 1: Write the failing test**

In profile tests:

```rust
#[test]
fn profile_import_href_uses_filename_only() {
    let profile = build_profile(
        "/absolute/path/to/catalog.json",
        vec!["AC-1".to_string()],
        SelectionMode::Include,
        &[],
    ).unwrap();
    assert_eq!(profile.imports[0].href, "catalog.json");
    assert!(!profile.imports[0].href.contains('/'));
}
```

- [ ] **Step 2: Sanitize in `build_profile()`**

In `src/oscal/profile.rs`, where `catalog_path` is used in `ProfileImport { href: catalog_path.to_string(), ... }`, change to:

```rust
href: crate::io::sanitize_artifact_path(std::path::Path::new(catalog_path)),
```

- [ ] **Step 3: Update `cli/profile.rs` similarly**

In `src/cli/profile.rs:96`, pass the sanitized path.

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS (some existing tests may need path expectations updated)

- [ ] **Step 5: Commit**

```bash
git add src/oscal/profile.rs src/cli/profile.rs
git commit -m "fix: profile generation uses filename-only paths (#79)"
```

### Task 20: Apply `sanitize_artifact_path` to AP and implemented requirements

**Files:**
- Modify: `src/oscal/assessment_plan.rs:136`
- Modify: `src/oscal/implemented_requirements.rs:131`

- [ ] **Step 1: Sanitize AP import-ssp href**

In `src/oscal/assessment_plan.rs`, where `import_ssp_href` is stored, sanitize it:

```rust
href: crate::io::sanitize_artifact_path(std::path::Path::new(&import_ssp_href)),
```

- [ ] **Step 2: Sanitize implemented requirements source**

In `src/oscal/implemented_requirements.rs`, where the source profile path is stored, sanitize it.

- [ ] **Step 3: Write tests verifying filename-only output**

```rust
#[test]
fn assessment_plan_import_ssp_uses_filename_only() {
    // Build an AP with absolute SSP path
    let envelope = build_assessment_plan(
        "Test Policy",
        &["AC-1".to_string()],
        "/absolute/path/to/ssp.json",
    ).unwrap();
    let href = &envelope.assessment_plan.import_ssp.href;
    assert_eq!(href, "ssp.json");
    assert!(!href.contains('/'));
}

#[test]
fn implemented_requirements_source_uses_filename_only() {
    // Build control implementation with absolute source path
    let ci = ControlImplementation::new(
        "/absolute/path/to/profile.json",
        "Test Policy",
        &requirements,
    );
    assert_eq!(ci.source, "profile.json");
    assert!(!ci.source.contains('/'));
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/oscal/assessment_plan.rs src/oscal/implemented_requirements.rs
git commit -m "fix: AP and implemented requirements use filename-only paths (#79)"
```

---

## Chunk 6: Behavioral Fixes (Theme 6, Issues #80, #83, #84)

### Task 21: Stable control IDs with content-based disambiguation

**Files:**
- Modify: `src/oscal/catalog.rs:445-459`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn abbreviation_collision_stable_under_reorder() {
    use std::collections::HashMap;
    // Order 1
    let mut counts1 = HashMap::new();
    let a1 = resolve_abbreviation("Access Control", &mut counts1);
    let b1 = resolve_abbreviation("Audit Control", &mut counts1);

    // Order 2 (reversed)
    let mut counts2 = HashMap::new();
    let b2 = resolve_abbreviation("Audit Control", &mut counts2);
    let a2 = resolve_abbreviation("Access Control", &mut counts2);

    // Same titles should get same IDs regardless of order
    assert_eq!(a1, a2, "Access Control ID changed with reorder");
    assert_eq!(b1, b2, "Audit Control ID changed with reorder");
}
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL (encounter-order gives different results)

- [ ] **Step 3: Implement content-based disambiguation**

Replace `resolve_abbreviation()`:

```rust
pub(crate) fn resolve_abbreviation(title: &str, counts: &mut HashMap<String, Vec<String>>) -> String {
    use sha2::{Sha256, Digest};

    let base = generate_section_abbreviation(title);
    let titles = counts.entry(base.clone()).or_default();
    titles.push(title.to_string());

    if titles.len() == 1 {
        // First occurrence — may need suffix later when a collision is found
        base
    } else {
        // Collision: use hash suffix
        let mut hasher = Sha256::new();
        hasher.update(title.as_bytes());
        let hash = hasher.finalize();
        let suffix = format!("{:02x}{:02x}", hash[0], hash[1]);
        debug!(
            abbreviation = %base,
            suffix = %suffix,
            section = %title,
            "Abbreviation collision resolved with hash suffix"
        );
        format!("{base}-{suffix}")
    }
}
```

Note: The first title to claim a bare abbreviation keeps it. When a second title collides, only the second gets a hash suffix. This is stable because the suffix derives from the title content, not encounter order.

- [ ] **Step 4: Update the `counts` type from `HashMap<String, usize>` to `HashMap<String, Vec<String>>`**

Update `build_catalog()` to use the new type.

- [ ] **Step 5: Run test to verify it passes**

Expected: PASS

- [ ] **Step 6: Run full test suite and update snapshots**

Run: `cargo test`
Some snapshot tests may need updating due to changed control IDs.

- [ ] **Step 7: Commit**

```bash
git add src/oscal/catalog.rs tests/
git commit -m "fix: stable control IDs using content-based hash disambiguation (#80)"
```

### Task 22: Respect `--quiet` in batch mode

**Files:**
- Modify: `src/cli/convert.rs:211-213`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn batch_quiet_suppresses_stderr() {
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(["convert", "tests/fixtures/", "--strategy", "catalog", "--format", "json", "--quiet", "--output", "/tmp/forge-test-batch"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&result.stderr);
    // With --quiet, no summary should be printed
    assert!(!stderr.contains("files processed"), "batch summary printed despite --quiet");
}
```

- [ ] **Step 2: Gate the eprint on `!opts.quiet`**

In `src/cli/convert.rs`, change:

```rust
let formatted = batch::format_batch_summary(&batch_summary);
eprint!("{formatted}");
```

to:

```rust
if !opts.quiet {
    let formatted = batch::format_batch_summary(&batch_summary);
    eprint!("{formatted}");
}
```

- [ ] **Step 3: Run test**

Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/cli/convert.rs
git commit -m "fix: batch mode respects --quiet flag (#83)"
```

### Task 23: Deterministic profile UUIDs

**Files:**
- Modify: `src/oscal/profile.rs:252`
- Modify: `src/cli/mod.rs` (add `--timestamp` flag)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn profile_same_inputs_produce_same_uuid() {
    let p1 = build_profile("catalog.json", vec!["AC-1".to_string()], SelectionMode::Include, &[]).unwrap();
    let p2 = build_profile("catalog.json", vec!["AC-1".to_string()], SelectionMode::Include, &[]).unwrap();
    assert_eq!(p1.uuid, p2.uuid, "Same inputs should produce same UUID");
}
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL (v4 UUIDs are random)

- [ ] **Step 3: Replace `Uuid::new_v4()` with `Uuid::new_v5()`**

In `src/oscal/profile.rs:252`, replace:

```rust
Ok(OscalProfile { uuid: Uuid::new_v4(), metadata, imports, modify })
```

with:

```rust
let mut seed_parts: Vec<&str> = vec![catalog_path];
seed_parts.extend(control_ids.iter().map(String::as_str));
let seed = seed_parts.join("|");
let uuid = Uuid::new_v5(&crate::uuid::PROFILE_NAMESPACE, seed.as_bytes());
Ok(OscalProfile { uuid, metadata, imports, modify })
```

Add `PROFILE_NAMESPACE` to `src/uuid.rs`:

```rust
/// Derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"profile")`.
pub const PROFILE_NAMESPACE: Uuid = Uuid::from_bytes([
    0xA6, 0x5B, 0xF3, 0x31, 0x28, 0x1C, 0x55, 0x86, 0x9F, 0x2E, 0x61, 0x31, 0x2E, 0x38, 0x55, 0x2F,
]);
```

- [ ] **Step 4: Add `--timestamp` flag to profile subcommand**

In `src/cli/mod.rs`, add to the profile command struct:

```rust
/// Override the last-modified timestamp (ISO 8601) for reproducible output.
#[arg(long)]
pub timestamp: Option<String>,
```

Thread this through to `assemble_metadata()`.

- [ ] **Step 5: Run test to verify it passes**

Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cargo test`
Expected: PASS (some profile tests may need updating)

- [ ] **Step 7: Commit**

```bash
git add src/oscal/profile.rs src/cli/mod.rs src/uuid.rs src/cli/profile.rs
git commit -m "feat: deterministic profile UUIDs and --timestamp flag (#84)"
```

---

## Chunk 7: Clippy Cleanup (Issue #85)

### Task 24: Fix all 49 clippy warnings

**Files:**
- Modify: Various (see clippy output)

- [ ] **Step 1: Run clippy and capture all warnings**

Run: `cargo clippy --all-targets --all-features 2>&1 | grep "warning\["`
Capture the full list.

- [ ] **Step 2: Fix doc backtick warnings (16x)**

Add backticks around code references in doc comments:
- `tests/export_integration.rs:9:62`
- `tests/profile_param_test.rs:16:62`
- `tests/integration_round_trip.rs:5:38`
- Plus 13 more locations (run `cargo clippy 2>&1 | grep "item in documentation"` for exact list)

- [ ] **Step 3: Fix `let...else` rewrites (5x)**

Replace `if let` + `panic!` with `let...else`:
- `tests/component_pipeline_test.rs:439`
- `tests/component_pipeline_test.rs:444`
- `tests/oscal_cli_round_trip.rs:154`
- Plus 2 more (run `cargo clippy 2>&1 | grep "let...else"`)

- [ ] **Step 4: Fix `map().unwrap_or(false)` → `is_some_and()` (4x)**

- `tests/integration_regression.rs:72:31`
- `tests/integration_profile_e2e.rs:186:9`
- `tests/integration_profile_e2e.rs:263:9`
- Plus 1 more

- [ ] **Step 5: Fix redundant closures (4x)**

- `src/diff/engine.rs:139:43`
- `tests/integration_cross_feature.rs:153:56`
- `tests/integration_cross_feature.rs:172:64`
- Plus 1 more

- [ ] **Step 6: Fix `panic!` in if → `assert!` (4x)**

- `tests/integration_round_trip.rs:19:5`
- `tests/integration_cross_feature.rs:57:5`
- `tests/integration_regression.rs:21:5`
- Plus 1 more

- [ ] **Step 7: Fix remaining warnings**

- `map().unwrap_or()` → `map_or()` (3x): `tests/integration_cross_feature.rs:169:5`, `tests/integration_cross_feature.rs:172:35`, `tests/integration_profile_e2e.rs:349:13`
- Format string inlining (2x): `benches/export_bench.rs:86:27`, `benches/export_bench.rs:98:27`
- Collapsible if (2x): `tests/integration_round_trip.rs:40:5`, `tests/integration_cross_feature.rs:116:17`
- Borrowed expression (2x): `benches/export_bench.rs:86:26`, `benches/export_bench.rs:98:26`
- Empty String (2x): `src/diff/engine.rs:386:19`, `src/diff/engine.rs:393:19`
- `format!` appended to String (1x): `src/oscal_cli/invoker.rs:242:13`
- `sort` on primitive (1x): `src/diff/engine.rs:279:9`
- Identical match arms (1x): `tests/golden_edge_case_tests.rs:476:21`
- `if let` instead of match (1x): `src/export/xml_deserializer.rs:451:9`
- `is_ok()` pattern (1x): `tests/assessment_plan_test.rs:201:12`

- [ ] **Step 8: Verify zero warnings**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: EXIT 0, zero warnings

- [ ] **Step 9: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "chore: fix all 49 clippy warnings (#85)"
```

---

## Final Verification

- [ ] **Run full test suite**: `cargo test` — all tests pass
- [ ] **Run clippy**: `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings
- [ ] **Run fmt check**: `cargo fmt --check` — no formatting issues
- [ ] **Verify all 18 issues addressed**: Cross-reference with issue-to-theme mapping in spec
