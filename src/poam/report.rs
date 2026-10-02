//! Content-minimizing, deterministic source inventory for the POA&M foundation.

use std::fmt::Write as _;
use std::io::Write as _;

use serde::Serialize;

use super::manifest::SourceKind;
use crate::ForgeError;

/// Versioned source inventory contract. This report conveys no workflow approval.
pub const INVENTORY_SCHEMA_VERSION: &str = "forge.poam-source-inventory/1";
/// A detached source inventory is bounded independently of its source inputs.
pub const MAX_INVENTORY_BYTES: usize = 10 * 1024 * 1024;

/// Exact selected result and complete finding/risk inventory, without assessment prose.
///
/// [`super::source::load`] creates internally checked inventories. These public
/// fields permit detached use; serializers do not revalidate caller-constructed
/// values or confer authority on their scope or assertion fields.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceInventory {
    /// Exact detached inventory format identifier, [`INVENTORY_SCHEMA_VERSION`].
    pub schema_version: &'static str,
    /// Foundation loader scope, `source-integrity-only`; no workflow acceptance is implied.
    pub validation_scope: &'static str,
    /// Always `false` for foundation-loaded inventories; source consistency is not workflow validation.
    pub workflow_validated: bool,
    /// Lowercase SHA-256 of the complete captured AR file bytes, including formatting.
    pub source_sha256: String,
    /// Canonical native UUID of the explicitly selected result.
    pub result_uuid: String,
    /// Exact stable key paired with that result UUID; neither denotes review approval.
    pub result_key: String,
    /// All selected-result findings and risks, sorted by kind/key without eligibility filtering.
    pub objects: Vec<SourceObject>,
}

/// One original finding or risk, including satisfied findings and closed risks.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceObject {
    /// Original source domain, preserved even for satisfied findings or closed risks.
    pub kind: SourceKind,
    /// Exact Forge stable key from this source object, without prose-based matching.
    pub key: String,
    /// Original canonical native object UUID, preserved rather than regenerated.
    pub uuid: String,
    /// Canonical UUID of the explicitly selected result containing the object.
    pub result_uuid: String,
    /// Computed SHA-256 of the complete canonical object, including current prose and properties.
    /// Object keys are recursively sorted and array order is preserved; this differs
    /// from exact-file hashes and producer-declared digest properties.
    pub sha256: String,
    /// Original asserted finding state or risk status; `closed` does not verify remediation.
    pub state: String,
    /// Exact finding target controls, or sorted controls from this risk's related findings.
    /// An unlinked risk has an empty list; no applicability or eligibility judgment is inferred.
    pub control_ids: Vec<String>,
    /// Optional producer-declared lowercase content digest, retained as an assertion only.
    /// It cannot replace [`Self::sha256`] when qualifying an exact source tuple.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_content_sha256: Option<String>,
    /// Optional producer-declared lowercase rationale digest; this proves no review or approval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_rationale_sha256: Option<String>,
}

/// Serialize a deterministic JSON inventory with one trailing newline.
///
/// Output, including the newline, is capped at [`MAX_INVENTORY_BYTES`] before it
/// is returned. This renders supplied fields without revalidating a detached
/// inventory; use [`super::source::load`] to establish source consistency first.
///
/// # Errors
/// Returns an error if serialization fails or output exceeds the 10 MiB byte bound.
pub fn render_json(inventory: &SourceInventory) -> Result<String, ForgeError> {
    render_json_bounded(inventory, MAX_INVENTORY_BYTES)
}

fn render_json_bounded(inventory: &SourceInventory, limit: usize) -> Result<String, ForgeError> {
    let mut writer = BoundedJson { bytes: Vec::new(), limit };
    serde_json::to_writer_pretty(&mut writer, inventory)
        .map_err(|cause| error(format!("source inventory serialization failed: {cause}")))?;
    writer.write_all(b"\n").map_err(|cause| error(cause.to_string()))?;
    String::from_utf8(writer.bytes)
        .map_err(|cause| error(format!("source inventory is not UTF-8: {cause}")))
}

struct BoundedJson {
    bytes: Vec<u8>,
    limit: usize,
}

impl std::io::Write for BoundedJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("source inventory exceeds the byte bound"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Render identities and state only; no source assessment prose or actor identities.
///
/// Authored keys, status and control identifiers are escaped for terminal display.
/// Supplied fields are rendered without revalidating a detached inventory. The
/// closing boundary statement conveys source integrity only and grants no review,
/// remediation eligibility or risk-acceptance authority.
///
/// # Errors
/// Returns an error before growth beyond the 10 MiB inventory bound.
pub fn render_text(inventory: &SourceInventory) -> Result<String, ForgeError> {
    render_text_bounded(inventory, MAX_INVENTORY_BYTES)
}

fn render_text_bounded(inventory: &SourceInventory, limit: usize) -> Result<String, ForgeError> {
    let mut output = BoundedText { bytes: String::new(), limit };
    let rendered = (|| {
        writeln!(output, "FORGE POA&M source inventory")?;
        writeln!(output, "schema: {}", inventory.schema_version)?;
        writeln!(output, "source-sha256: {}", inventory.source_sha256)?;
        writeln!(output, "result: {} ({})", Escaped(&inventory.result_key), inventory.result_uuid)?;
        writeln!(output, "objects: {}", inventory.objects.len())?;
        for object in &inventory.objects {
            write!(
                output,
                "{} {}: uuid={}, state={}, controls=",
                object.kind.as_str(),
                Escaped(&object.key),
                object.uuid,
                Escaped(&object.state)
            )?;
            for (index, control) in object.control_ids.iter().enumerate() {
                if index != 0 {
                    output.write_str(",")?;
                }
                write!(output, "{}", Escaped(control))?;
            }
            writeln!(output, ", sha256={}", object.sha256)?;
        }
        writeln!(output, "Source integrity only; no review or remediation eligibility judgment.")
    })();
    rendered.map_err(|_| error("source inventory text exceeds the byte bound"))?;
    Ok(output.bytes)
}

struct BoundedText {
    bytes: String,
    limit: usize,
}

impl std::fmt::Write for BoundedText {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        if text.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::fmt::Error);
        }
        self.bytes.push_str(text);
        Ok(())
    }
}

struct Escaped<'a>(&'a str);

impl std::fmt::Display for Escaped<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for character in self.0.chars().flat_map(char::escape_default) {
            formatter.write_char(character)?;
        }
        Ok(())
    }
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PoamBuild(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escapes_authored_keys_and_json_ends_with_newline() {
        let inventory = SourceInventory {
            schema_version: INVENTORY_SCHEMA_VERSION,
            validation_scope: "source-integrity-only",
            workflow_validated: false,
            source_sha256: "a".repeat(64),
            result_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
            result_key: "result\ninjected".to_string(),
            objects: vec![SourceObject {
                kind: SourceKind::Finding,
                key: "finding\ninjected\u{1b}[31m".to_string(),
                uuid: "22222222-2222-4222-8222-222222222222".to_string(),
                result_uuid: "11111111-1111-4111-8111-111111111111".to_string(),
                sha256: "b".repeat(64),
                state: "state\rforged".to_string(),
                control_ids: vec!["AC-1".to_string(), "AC-2\tforged".to_string()],
                declared_content_sha256: Some("c".repeat(64)),
                declared_rationale_sha256: Some("d".repeat(64)),
            }],
        };
        let text = render_text(&inventory).unwrap();
        assert!(text.contains("result\\ninjected"));
        assert!(!text.contains("result\ninjected"));
        assert!(text.contains("finding\\ninjected\\u{1b}[31m"));
        assert!(text.contains("state\\rforged"));
        assert!(text.contains("controls=AC-1,AC-2\\tforged"));
        assert!(!text.contains('\u{1b}'));
        assert!(!text.contains('\r'));
        assert!(!text.contains('\t'));
        let json = render_json(&inventory).unwrap();
        assert!(json.ends_with('\n'));
        assert!(!json.contains("assessment rationale"));
        assert!(json.contains("source-integrity-only"));
        assert!(json.contains("\"workflow_validated\": false"));
        assert!(render_json_bounded(&inventory, json.len() - 1).is_err());
        assert_eq!(render_json_bounded(&inventory, json.len()).unwrap(), json);
        assert!(render_text_bounded(&inventory, text.len() - 1).is_err());
        assert_eq!(render_text_bounded(&inventory, text.len()).unwrap(), text);
    }
}
