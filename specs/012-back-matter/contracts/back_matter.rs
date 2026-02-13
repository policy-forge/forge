//! Type contracts for OSCAL Back Matter Generation (WI-12).
//!
//! This file defines the public API types and function signatures.
//! Implementation follows after contract review and TDD test creation.

// Note: HashMap is used in function signatures (commented out below) but not in struct definitions.
// use std::collections::HashMap;

use serde::Serialize;
use uuid::Uuid;

// Assumes ForgeError and Citation are importable from their respective modules.
// use crate::error::ForgeError;
// use crate::model::Citation;

// ─── Back Matter Structs ────────────────────────────────────────────────

/// Top-level OSCAL back matter containing all reference resources.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BackMatter {
    /// All back matter resources generated from citations.
    pub resources: Vec<BackMatterResource>,
}

/// A single OSCAL back matter resource generated from a Citation.
///
/// Each resource has a deterministic UUID v5 derived from the
/// `BACK_MATTER_NAMESPACE` and the citation content.
#[derive(Debug, Clone, Serialize)]
pub struct BackMatterResource {
    /// Deterministic UUID v5 for this resource.
    pub uuid: Uuid,

    /// Title derived from citation text (preferred) or full URL (fallback).
    pub title: String,

    /// Optional description providing citation context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Bibliographic citation text (for non-URL citations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<ResourceCitation>,

    /// Resolvable links to external content (for URL-based citations).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rlinks: Vec<Rlink>,

    /// Property annotations (e.g., url-status for malformed URLs).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<Prop>,
}

/// Bibliographic citation text within a resource.
#[derive(Debug, Clone, Serialize)]
pub struct ResourceCitation {
    /// The bibliographic reference text.
    pub text: String,
}

/// Resolvable link to external content.
#[derive(Debug, Clone, Serialize)]
pub struct Rlink {
    /// URL to external content.
    pub href: String,

    /// Optional IANA media type inferred from URL extension.
    #[serde(rename = "media-type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// OSCAL link element for control bodies.
///
/// Links controls to back matter resources via `href="#<resource-uuid>"`.
#[derive(Debug, Clone, Serialize)]
pub struct OscalLink {
    /// Reference to back matter resource: `"#<resource-uuid>"`.
    pub href: String,

    /// Link relationship type: always `"reference"`.
    pub rel: String,

    /// Optional display text for the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// OSCAL property annotation (name-value pair).
///
/// Used for structured metadata instead of `remarks` per NIST guidance.
#[derive(Debug, Clone, Serialize)]
pub struct Prop {
    /// Property name (e.g., `"url-status"`).
    pub name: String,

    /// Property value (e.g., `"unvalidated"`).
    pub value: String,
}

// ─── Citation (Input Contract from WI-8) ────────────────────────────────

/// A citation extracted from a policy document by WI-8.
///
/// This is the input to back matter generation.
#[derive(Debug, Clone, Serialize)]
pub struct Citation {
    /// Unique identifier for this citation.
    pub id: String,

    /// Citation text (bibliographic reference or descriptive text).
    pub text: String,

    /// URL if this is a URL-based citation; None for bibliographic-only.
    pub url: Option<String>,

    /// stable_id of the PolicyRequirement that references this citation.
    pub source_requirement_id: Option<String>,
}

// ─── Function Signatures ────────────────────────────────────────────────

// /// Generate back matter resources from extracted citations.
// ///
// /// Returns a tuple of:
// /// - `Vec<BackMatterResource>`: The OSCAL resources for back matter
// /// - `HashMap<String, Uuid>`: Map from citation ID to resource UUID
// ///
// /// # Errors
// ///
// /// Returns `ForgeError::BackMatter` if citation data is invalid
// /// (e.g., citation with no text and no URL).
// pub fn generate_back_matter(
//     citations: &[Citation],
// ) -> Result<(Vec<BackMatterResource>, HashMap<String, Uuid>), ForgeError>;

// /// Generate link elements for a control given its associated citations.
// ///
// /// For each citation, looks up the resource UUID from the map and creates
// /// an `OscalLink` with `href="#<uuid>"` and `rel="reference"`.
// ///
// /// Citations not found in the resource map are skipped with a warning.
// pub fn generate_control_links(
//     citations: &[Citation],
//     resource_map: &HashMap<String, Uuid>,
// ) -> Vec<OscalLink>;
