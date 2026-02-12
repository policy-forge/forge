use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::instrument;
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::DocumentMetadata;

/// OSCAL specification version constant.
/// Single point of change when FORGE targets a new OSCAL version.
pub const OSCAL_VERSION: &str = "1.2.0";

/// OSCAL metadata for any artifact type (Catalog, Component Definition, Profile).
///
/// Serializes to OSCAL-compliant JSON with hyphenated field names where required.
/// All five fields are mandatory per OSCAL v1.2.0 specification.
///
/// # Examples
///
/// ```
/// use forge::oscal::metadata::{assemble_metadata, MetadataOptions, OscalMetadata};
/// use forge::model::DocumentMetadata;
///
/// let doc_meta = DocumentMetadata { title: "My Policy".into(), version: "1.0".into(), ..Default::default() };
/// let metadata = assemble_metadata(&doc_meta, None).unwrap();
/// assert_eq!(metadata.title, "My Policy");
/// assert_eq!(metadata.oscal_version, "1.2.0");
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct OscalMetadata {
    /// UUID v4 — unique per artifact generation instance.
    /// Not part of the OSCAL metadata section; used programmatically
    /// as the artifact-level UUID (e.g., `OscalCatalog.uuid`).
    #[serde(skip)]
    pub uuid: Uuid,

    /// Document title from PolicyDocument.metadata.title.
    pub title: String,

    /// ISO 8601 UTC timestamp of artifact generation.
    #[serde(rename = "last-modified")]
    pub last_modified: DateTime<Utc>,

    /// Document version from PolicyDocument.metadata.version.
    pub version: String,

    /// OSCAL specification version — always "1.2.0".
    #[serde(rename = "oscal-version")]
    pub oscal_version: String,
}

/// Options for overriding auto-generated metadata values (primarily for testing).
///
/// Production callers pass `None` to `assemble_metadata`. Test callers
/// construct `MetadataOptions` with fixed UUID and/or timestamp for
/// deterministic assertions.
#[derive(Debug, Default)]
pub struct MetadataOptions {
    /// Override the auto-generated UUID v4 (for deterministic tests).
    pub uuid_override: Option<Uuid>,

    /// Override the auto-generated timestamp (for deterministic tests).
    pub timestamp_override: Option<DateTime<Utc>>,
}

/// Assemble OSCAL metadata from a `PolicyDocument`'s `DocumentMetadata`.
///
/// Produces a complete `OscalMetadata` struct with all five required fields.
/// Uses UUID v4 for artifact identity and current UTC timestamp unless overridden.
///
/// # Arguments
///
/// * `doc_metadata` — Reference to the source document's metadata (title, version)
/// * `options` — Optional overrides for UUID and timestamp (for deterministic tests)
///
/// # Errors
///
/// Currently infallible. Returns `Result` for API consistency and future extensibility.
///
/// # Security
///
/// This function MUST NOT read environment variables, filesystem paths, hostnames,
/// or any system-identifying information (SEC-1, SEC-5). Only the provided
/// `DocumentMetadata` fields, `Uuid::new_v4()`, `Utc::now()`, and `OSCAL_VERSION`
/// constant are used.
#[instrument(skip(doc_metadata), level = "debug")]
pub fn assemble_metadata(
    doc_metadata: &DocumentMetadata,
    options: Option<MetadataOptions>,
) -> Result<OscalMetadata, ForgeError> {
    let opts = options.unwrap_or_default();

    if doc_metadata.title.is_empty() {
        tracing::warn!("DocumentMetadata title is empty; OSCAL metadata.title will be empty");
    }

    Ok(OscalMetadata {
        uuid: opts.uuid_override.unwrap_or_else(Uuid::new_v4),
        title: doc_metadata.title.clone(),
        last_modified: opts.timestamp_override.unwrap_or_else(Utc::now),
        version: doc_metadata.version.clone(),
        oscal_version: OSCAL_VERSION.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    /// T006: Inject fixed UUID and timestamp via MetadataOptions, assert both returned unchanged.
    #[test]
    fn assemble_with_overrides_uses_injected_values() {
        let fixed_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let fixed_ts = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap();

        let doc_meta = DocumentMetadata {
            title: "Test Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };
        let opts =
            MetadataOptions { uuid_override: Some(fixed_uuid), timestamp_override: Some(fixed_ts) };

        let result = assemble_metadata(&doc_meta, Some(opts)).unwrap();
        assert_eq!(result.uuid, fixed_uuid);
        assert_eq!(result.last_modified, fixed_ts);
    }

    /// T007: Set title to "Access Control Policy", assert metadata.title matches.
    #[test]
    fn assemble_populates_title_from_document_metadata() {
        let doc_meta = DocumentMetadata {
            title: "Access Control Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.title, "Access Control Policy");
    }

    /// T008: Set version to "2.1", assert metadata.version matches.
    #[test]
    fn assemble_populates_version_from_document_metadata() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "2.1".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.version, "2.1");
    }

    /// T009: Assert metadata.oscal_version equals "1.2.0".
    #[test]
    fn assemble_sets_oscal_version_to_1_2_0() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.oscal_version, "1.2.0");
    }

    /// T010: Call without override, parse last_modified, assert timezone is UTC.
    #[test]
    fn assemble_generates_utc_timestamp() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        let serialized = result.last_modified.to_rfc3339();
        assert!(
            serialized.ends_with('Z') || serialized.ends_with("+00:00"),
            "Timestamp should be UTC: {serialized}"
        );
    }

    /// T011: Serialize OscalMetadata to JSON, assert keys include hyphenated names.
    #[test]
    fn serialization_produces_correct_json_field_names() {
        let fixed_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let fixed_ts = Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap();

        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };
        let opts =
            MetadataOptions { uuid_override: Some(fixed_uuid), timestamp_override: Some(fixed_ts) };

        let result = assemble_metadata(&doc_meta, Some(opts)).unwrap();
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains("\"last-modified\""), "Should use hyphenated last-modified");
        assert!(json.contains("\"oscal-version\""), "Should use hyphenated oscal-version");
        assert!(json.contains("\"title\""), "Should contain title field");
        assert!(json.contains("\"version\""), "Should contain version field");
        // uuid is #[serde(skip)] — it's an artifact-level field, not part of metadata section
        assert!(!json.contains("\"uuid\""), "uuid should not serialize in metadata section");
    }

    // --- US2: UUID Uniqueness Tests ---

    /// T014: Call without override, verify UUID is valid v4 (version nibble = 4, variant = RFC4122).
    #[test]
    fn assemble_generates_valid_uuid_v4() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.uuid.get_version_num(), 4, "UUID should be version 4");
        assert_eq!(
            result.uuid.get_variant(),
            uuid::Variant::RFC4122,
            "UUID variant should be RFC4122"
        );
    }

    /// T015: Call twice without override, assert UUIDs differ.
    #[test]
    fn assemble_two_calls_produce_different_uuids() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "1.0".into(),
            ..Default::default()
        };

        let result1 = assemble_metadata(&doc_meta, None).unwrap();
        let result2 = assemble_metadata(&doc_meta, None).unwrap();
        assert_ne!(result1.uuid, result2.uuid, "Two calls should produce different UUIDs");
    }

    // --- US3: Edge Case Tests ---

    /// T017: Pass empty title, assert metadata.title is empty string (EC-1).
    #[test]
    fn assemble_empty_title_passes_through() {
        let doc_meta =
            DocumentMetadata { title: String::new(), version: "1.0".into(), ..Default::default() };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.title, "", "Empty title should pass through unchanged");
    }

    /// T018: Pass version "0.0.0", assert metadata.version equals "0.0.0" (EC-2).
    #[test]
    fn assemble_default_version_passthrough() {
        let doc_meta = DocumentMetadata {
            title: "Policy".into(),
            version: "0.0.0".into(),
            ..Default::default()
        };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.version, "0.0.0", "Default version should pass through unchanged");
    }

    /// T019: Pass title with quotes, ampersands, Unicode; assert preserved exactly (EC-5).
    #[test]
    fn assemble_special_characters_in_title() {
        let title = "Policy \"A\" & règles © 2026";
        let doc_meta =
            DocumentMetadata { title: title.into(), version: "1.0".into(), ..Default::default() };

        let result = assemble_metadata(&doc_meta, None).unwrap();
        assert_eq!(result.title, title, "Special characters should be preserved");
    }
}
