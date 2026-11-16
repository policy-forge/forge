// Contract: OSCAL Metadata Assembly
// Phase 1 output — type signatures and function contracts
// This file is a reference; actual implementation in src/oscal/metadata.rs

use chrono::{DateTime, Utc};
use serde::Serialize;
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
    /// UUID v4 — unique per artifact generation instance. Internal only:
    /// the OSCAL metadata schema has no `uuid` property (additionalProperties:
    /// false); the artifact UUID belongs at the envelope root. Never serialized.
    #[serde(skip_serializing)]
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

/// Assemble OSCAL metadata from a PolicyDocument's DocumentMetadata.
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
/// # Errors
///
/// Returns `ForgeError::Validation` if `doc_metadata.title` or
/// `doc_metadata.version` is empty: both are mandatory OSCAL metadata fields,
/// so empty input is refused rather than emitted as non-compliant metadata.
///
/// # Security
///
/// This function MUST NOT read environment variables, filesystem paths, hostnames,
/// or any system-identifying information (SEC-1, SEC-5). Only the provided
/// `DocumentMetadata` fields, `Uuid::new_v4()`, `Utc::now()`, and `OSCAL_VERSION`
/// constant are used.
pub fn assemble_metadata(
    doc_metadata: &DocumentMetadata,
    options: Option<MetadataOptions>,
) -> Result<OscalMetadata, ForgeError> {
    if doc_metadata.title.is_empty() {
        return Err(ForgeError::Validation(
            "OSCAL metadata title must not be empty".to_string(),
        ));
    }
    if doc_metadata.version.is_empty() {
        return Err(ForgeError::Validation(
            "OSCAL metadata version must not be empty".to_string(),
        ));
    }

    let opts = options.unwrap_or_default();
    Ok(OscalMetadata {
        uuid: opts.uuid_override.unwrap_or_else(Uuid::new_v4),
        title: doc_metadata.title.clone(),
        last_modified: opts.timestamp_override.unwrap_or_else(Utc::now),
        version: doc_metadata.version.clone(),
        oscal_version: OSCAL_VERSION.to_string(),
    })
}
