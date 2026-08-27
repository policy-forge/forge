//! OSCAL Back Matter generation: maps extracted citations to back matter resources
//! and generates control link elements.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ForgeError;
use crate::model::Citation;
use crate::uuid::BACK_MATTER_NAMESPACE;

// ─── Back Matter Structs ────────────────────────────────────────────────

/// Top-level OSCAL back matter containing all reference resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BackMatter {
    /// All back matter resources generated from citations.
    pub resources: Vec<BackMatterResource>,
}

/// A single OSCAL back matter resource generated from a Citation.
///
/// Each resource has a deterministic UUID v5 derived from the
/// `BACK_MATTER_NAMESPACE` and the citation content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackMatterResource {
    /// Deterministic UUID v5 for this resource.
    pub uuid: Uuid,

    /// Title derived from citation text (preferred) or full URL (fallback).
    pub title: String,

    /// Optional description providing citation context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Bibliographic citation text (for non-URL citations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation: Option<ResourceCitation>,

    /// Resolvable links to external content (for URL-based citations).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rlinks: Vec<Rlink>,

    /// Property annotations (e.g., url-status for malformed URLs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<Prop>,
}

/// Bibliographic citation text within a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCitation {
    /// The bibliographic reference text.
    pub text: String,
}

/// Resolvable link to external content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rlink {
    /// URL to external content.
    pub href: String,

    /// Optional IANA media type inferred from URL extension.
    #[serde(default, rename = "media-type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
}

/// OSCAL link element for control bodies.
///
/// Links controls to back matter resources via `href="#<resource-uuid>"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalLink {
    /// Reference to back matter resource: `"#<resource-uuid>"`.
    pub href: String,

    /// Link relationship type: always `"reference"`.
    pub rel: String,

    /// Optional display text for the link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// OSCAL property annotation (name-value pair).
///
/// Used for structured metadata instead of `remarks` per NIST guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prop {
    /// Property name (e.g., `"url-status"`).
    pub name: String,

    /// Property value (e.g., `"unvalidated"`).
    pub value: String,

    /// Optional namespace URI for the property.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ns: Option<String>,
}

// ─── URL Classification ─────────────────────────────────────────────────

/// URL schemes that are actively dangerous if rendered as clickable links.
/// These are stripped from rlink hrefs entirely (SEC-2).
const DANGEROUS_SCHEMES: &[&str] = &["javascript", "data", "vbscript"];

/// Classification of a citation's URL field.
enum UrlClassification {
    /// Valid http/https URL.
    Valid(url::Url),
    /// Non-http/https scheme that is NOT actively dangerous — preserve URL, annotate.
    Malformed(String),
    /// Dangerous scheme (javascript:, data:, vbscript:) — href stripped, annotated only.
    Dangerous(String),
    /// No URL — bibliographic-only citation.
    None,
}

/// Classify a citation's URL field.
///
/// Only `http` and `https` schemes are considered valid. All other schemes
/// (including `javascript:`, `data:`, `ftp:`, `mailto:`, `file:`) are classified
/// as malformed and annotated with `prop url-status="unvalidated"`.
///
/// # Security
///
/// Schemes like `javascript:` and `data:` pose XSS risks if rendered as
/// clickable links. Downstream consumers should treat any resource with
/// `url-status: "unvalidated"` as untrusted and avoid rendering the href
/// as a navigable link. A control character before the first colon can smuggle
/// a dangerous pseudo-scheme through parsers that normalize it, so it is
/// treated as dangerous and omitted from `rlinks`.
fn classify_url(url_opt: Option<&String>) -> UrlClassification {
    let Some(raw) = url_opt else {
        return UrlClassification::None;
    };

    if raw.trim().is_empty() {
        return UrlClassification::Malformed(raw.clone());
    }

    if raw.chars().take_while(|character| *character != ':').any(char::is_control) {
        return UrlClassification::Dangerous(raw.clone());
    }

    match url::Url::parse(raw) {
        Ok(parsed) if parsed.scheme() == "http" || parsed.scheme() == "https" => {
            UrlClassification::Valid(parsed)
        }
        Ok(parsed) if DANGEROUS_SCHEMES.contains(&parsed.scheme()) => {
            UrlClassification::Dangerous(raw.clone())
        }
        Ok(_) | Err(_) => UrlClassification::Malformed(raw.clone()),
    }
}

fn citation_field(text: &str) -> Option<ResourceCitation> {
    if text.is_empty() { None } else { Some(ResourceCitation { text: text.to_string() }) }
}

fn build_resource_parts(
    classification: UrlClassification,
    citation: &Citation,
) -> (Vec<Rlink>, Option<ResourceCitation>, Vec<Prop>) {
    match classification {
        UrlClassification::Valid(parsed_url) => {
            let media_type = infer_media_type(&parsed_url);
            let href = parsed_url.to_string();
            let rlinks = vec![Rlink { href, media_type }];
            (rlinks, citation_field(&citation.text), vec![])
        }
        UrlClassification::Malformed(raw_url) => {
            tracing::warn!(
                citation_id = %citation.id,
                url = %raw_url,
                "Malformed or non-http/https URL preserved with unvalidated annotation"
            );
            let rlinks = vec![Rlink { href: raw_url, media_type: None }];
            let props = vec![Prop {
                name: "url-status".to_string(),
                value: "unvalidated".to_string(),
                ns: None,
            }];
            (rlinks, citation_field(&citation.text), props)
        }
        UrlClassification::Dangerous(raw_url) => {
            tracing::warn!(
                citation_id = %citation.id,
                url = %raw_url,
                "Dangerous URL scheme stripped from rlink href (SEC-2)"
            );
            let props = vec![Prop {
                name: "url-status".to_string(),
                value: "dangerous-scheme-removed".to_string(),
                ns: None,
            }];
            (vec![], citation_field(&citation.text), props)
        }
        UrlClassification::None => {
            (vec![], Some(ResourceCitation { text: citation.text.clone() }), vec![])
        }
    }
}

/// Infer IANA media type from URL path extension.
fn infer_media_type(url: &url::Url) -> Option<String> {
    let path = std::path::Path::new(url.path());
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("pdf") => Some("application/pdf".to_string()),
        Some(ext) if ext.eq_ignore_ascii_case("json") => Some("application/json".to_string()),
        Some(ext) if ext.eq_ignore_ascii_case("xml") => Some("application/xml".to_string()),
        Some(ext) if ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm") => {
            Some("text/html".to_string())
        }
        Some(ext) if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") => {
            Some("application/yaml".to_string())
        }
        Some(ext) if ext.eq_ignore_ascii_case("txt") => Some("text/plain".to_string()),
        _ => None,
    }
}

// ─── Public Functions ───────────────────────────────────────────────────

/// Generate back matter resources from extracted citations.
///
/// Returns a tuple of:
/// - `Vec<BackMatterResource>`: The OSCAL resources for back matter
/// - `HashMap<String, Uuid>`: Map from citation ID to resource UUID
///
/// UUID generation uses [`crate::uuid::normalize_for_hashing`] to normalize
/// citation text (trim + collapse whitespace) before hashing, ensuring that
/// whitespace-only differences produce identical UUIDs. Valid URLs are hashed
/// and emitted in their canonical parsed form.
///
/// # Errors
///
/// Returns `ForgeError::BackMatter` if citation data is invalid
/// (for example, an empty or duplicate citation ID).
pub fn generate_back_matter(
    citations: &[Citation],
) -> Result<(Vec<BackMatterResource>, HashMap<String, Uuid>), ForgeError> {
    let mut resources = Vec::with_capacity(citations.len());
    let mut resource_map = HashMap::with_capacity(citations.len());
    let mut seen_citation_ids = std::collections::HashSet::with_capacity(citations.len());
    let mut seen_uuids = std::collections::HashSet::with_capacity(citations.len());

    for citation in citations {
        if citation.id.is_empty() {
            return Err(ForgeError::BackMatter("citation has empty id".to_string()));
        }
        if !seen_citation_ids.insert(citation.id.clone()) {
            return Err(ForgeError::BackMatter(format!("duplicate citation id: {}", citation.id)));
        }

        if citation.text.is_empty() && citation.url.is_none() {
            tracing::warn!(
                citation_id = %citation.id,
                "Citation has empty text and no URL — skipping resource generation"
            );
            continue;
        }

        // Classify before deriving identity or display fields so parsed http(s)
        // URLs use their canonical spelling throughout (F0621).
        let classification = classify_url(citation.url.as_ref());
        let canonical_url = match &classification {
            UrlClassification::Valid(parsed_url) => Some(parsed_url.as_str()),
            UrlClassification::Malformed(raw_url) | UrlClassification::Dangerous(raw_url) => {
                Some(raw_url.as_str())
            }
            UrlClassification::None => None,
        };
        let normalized = crate::uuid::normalize_for_hashing(&citation.text);
        let hash_input =
            canonical_url.map_or_else(|| normalized.clone(), |url| format!("{normalized}\n{url}"));
        let uuid = Uuid::new_v5(&BACK_MATTER_NAMESPACE, hash_input.as_bytes());

        // Identical normalized content derives the same UUID: reuse the
        // existing resource instead of emitting duplicates (F0618). Links via
        // resource_map keep resolving for every distinct citation id.
        if !seen_uuids.insert(uuid) {
            resource_map.insert(citation.id.clone(), uuid);
            continue;
        }

        let title = if citation.text.is_empty() {
            match &classification {
                UrlClassification::Valid(parsed_url) => parsed_url.to_string(),
                UrlClassification::Dangerous(_) => "[unsafe URL scheme removed]".to_string(),
                UrlClassification::Malformed(raw_url) => raw_url.clone(),
                UrlClassification::None => String::new(),
            }
        } else {
            citation.text.clone()
        };

        let description = citation
            .source_requirement_id
            .as_ref()
            .map(|req_id| format!("Referenced by requirement {req_id}"));

        let (rlinks, citation_field, props) = build_resource_parts(classification, citation);

        resources.push(BackMatterResource {
            uuid,
            title,
            description,
            citation: citation_field,
            rlinks,
            props,
        });

        resource_map.insert(citation.id.clone(), uuid);
    }

    Ok((resources, resource_map))
}

/// Generate link elements for a control given its associated citations.
///
/// For each citation, looks up the resource UUID from the map and creates
/// an `OscalLink` with `href="#<uuid>"` and `rel="reference"`.
///
/// Citations not found in the resource map are skipped with a warning.
pub fn generate_control_links<S: ::std::hash::BuildHasher>(
    citations: &[Citation],
    resource_map: &HashMap<String, Uuid, S>,
) -> Vec<OscalLink> {
    let mut links = Vec::with_capacity(citations.len());

    for citation in citations {
        let Some(uuid) = resource_map.get(&citation.id) else {
            tracing::warn!(
                citation_id = %citation.id,
                "Citation not found in resource map — skipping link generation"
            );
            continue;
        };

        let text = if citation.text.is_empty() { None } else { Some(citation.text.clone()) };

        links.push(OscalLink { href: format!("#{uuid}"), rel: "reference".to_string(), text });
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Citation;

    // ── Test helpers ────────────────────────────────────

    fn url_citation(id: &str, text: &str, url: &str) -> Citation {
        Citation {
            id: id.to_string(),
            text: text.to_string(),
            url: Some(url.to_string()),
            source_requirement_id: None,
        }
    }

    fn biblio_citation(id: &str, text: &str) -> Citation {
        Citation {
            id: id.to_string(),
            text: text.to_string(),
            url: None,
            source_requirement_id: None,
        }
    }

    fn citation_with_req(id: &str, text: &str, url: Option<&str>, req_id: &str) -> Citation {
        Citation {
            id: id.to_string(),
            text: text.to_string(),
            url: url.map(String::from),
            source_requirement_id: Some(req_id.into()),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T008 [US1] URL-based citations → rlinks resources
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn url_citation_produces_rlink_with_matching_href() {
        let citations = vec![url_citation("c1", "NIST SP 800-53", "https://nvd.nist.gov/800-53")];
        let (resources, _map) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].rlinks.len(), 1);
        assert_eq!(resources[0].rlinks[0].href, "https://nvd.nist.gov/800-53");
    }

    #[test]
    fn valid_url_uses_canonical_href_and_identity() {
        let canonical = url_citation("canonical", "NIST", "https://nist.gov/");
        let padded = url_citation("padded", "NIST", "  https://nist.gov/  ");
        let (resources, resource_map) = generate_back_matter(&[canonical, padded]).unwrap();

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].rlinks[0].href, "https://nist.gov/");
        assert_eq!(resource_map["canonical"], resource_map["padded"]);
    }

    #[test]
    fn pdf_url_gets_application_pdf_media_type() {
        let citations = vec![url_citation("c1", "PDF Guide", "https://example.com/guide.pdf")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].rlinks[0].media_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn url_with_query_params_and_fragments_preserved() {
        let url = "https://example.com/doc?version=5&lang=en#section-3";
        let citations = vec![url_citation("c1", "Doc", url)];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].rlinks[0].href, url);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T009 [US1] Bibliographic citations → citation.text resources
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn bibliographic_citation_produces_citation_text() {
        let citations = vec![biblio_citation("c1", "NIST SP 800-53 Rev 5")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources.len(), 1);
        assert!(resources[0].rlinks.is_empty());
        assert_eq!(resources[0].citation.as_ref().unwrap().text, "NIST SP 800-53 Rev 5");
    }

    #[test]
    fn long_citation_text_preserved_without_truncation() {
        let long_text = "A".repeat(600);
        let citations = vec![biblio_citation("c1", &long_text)];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].citation.as_ref().unwrap().text.len(), 600);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T010 [US1] Deterministic UUID v5 and title derivation
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn same_citation_produces_same_uuid_across_calls() {
        let citations = vec![url_citation("c1", "NIST", "https://nist.gov")];
        let (r1, _) = generate_back_matter(&citations).unwrap();
        let (r2, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(r1[0].uuid, r2[0].uuid);
    }

    #[test]
    fn title_equals_citation_text_when_text_available() {
        let citations = vec![url_citation("c1", "NIST SP 800-53", "https://nist.gov")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].title, "NIST SP 800-53");
    }

    #[test]
    fn title_equals_full_url_for_url_only_citations() {
        let citations = vec![Citation {
            id: "c1".to_string(),
            text: String::new(),
            url: Some("https://example.com/reference".to_string()),
            source_requirement_id: None,
        }];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].title, "https://example.com/reference");
    }

    #[test]
    fn title_prefers_text_over_url_when_both_present() {
        let citations = vec![url_citation("c1", "My Title", "https://example.com")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].title, "My Title");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T011 [US1] Malformed/empty/non-http URL handling
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn malformed_url_preserves_url_in_rlinks_with_unvalidated_prop() {
        let citations = vec![url_citation("c1", "Bad ref", "not a url")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].rlinks.len(), 1);
        assert_eq!(resources[0].rlinks[0].href, "not a url");
        assert_eq!(resources[0].props.len(), 1);
        assert_eq!(resources[0].props[0].name, "url-status");
        assert_eq!(resources[0].props[0].value, "unvalidated");
    }

    #[test]
    fn empty_url_treated_as_malformed() {
        let citations = vec![Citation {
            id: "c1".to_string(),
            text: "Empty URL ref".to_string(),
            url: Some(String::new()),
            source_requirement_id: None,
        }];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].props.len(), 1);
        assert_eq!(resources[0].props[0].name, "url-status");
        assert_eq!(resources[0].props[0].value, "unvalidated");
    }

    #[test]
    fn whitespace_only_url_treated_as_malformed() {
        let citations = vec![Citation {
            id: "c1".to_string(),
            text: "Whitespace URL ref".to_string(),
            url: Some("   ".to_string()),
            source_requirement_id: None,
        }];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources[0].props.len(), 1);
        assert_eq!(resources[0].props[0].name, "url-status");
        assert_eq!(resources[0].props[0].value, "unvalidated");
        assert_eq!(resources[0].rlinks[0].href, "   ");
    }

    #[test]
    fn ftp_scheme_gets_unvalidated_prop() {
        let citations = vec![url_citation("c1", "FTP ref", "ftp://files.example.com/doc")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(
            resources[0].props.iter().any(|p| p.name == "url-status" && p.value == "unvalidated")
        );
    }

    #[test]
    fn mailto_scheme_gets_unvalidated_prop() {
        let citations = vec![url_citation("c1", "Email ref", "mailto:admin@example.com")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(
            resources[0].props.iter().any(|p| p.name == "url-status" && p.value == "unvalidated")
        );
    }

    #[test]
    fn javascript_scheme_stripped_from_rlinks() {
        let citations = vec![url_citation("c1", "JS ref", "javascript:alert(1)")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(resources[0].rlinks.is_empty(), "Dangerous scheme must not appear in rlinks");
        assert!(
            resources[0]
                .props
                .iter()
                .any(|p| p.name == "url-status" && p.value == "dangerous-scheme-removed")
        );
    }

    #[test]
    fn control_character_pseudo_scheme_is_stripped_from_rlinks() {
        let citations = vec![url_citation("c1", "Obfuscated JS ref", "jav\tascript:alert(1)")];
        let (resources, _) = generate_back_matter(&citations).unwrap();

        assert!(resources[0].rlinks.is_empty());
        assert!(
            resources[0].props.iter().any(|prop| {
                prop.name == "url-status" && prop.value == "dangerous-scheme-removed"
            })
        );
    }

    #[test]
    fn data_scheme_stripped_from_rlinks() {
        let citations = vec![url_citation("c1", "Data ref", "data:text/plain;base64,SGVsbG8=")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(resources[0].rlinks.is_empty(), "Dangerous scheme must not appear in rlinks");
        assert!(
            resources[0]
                .props
                .iter()
                .any(|p| p.name == "url-status" && p.value == "dangerous-scheme-removed")
        );
    }

    #[test]
    fn malformed_url_citation_still_in_resource_map() {
        let citations = vec![url_citation("c1", "Bad ref", "not a url")];
        let (_, map) = generate_back_matter(&citations).unwrap();
        assert!(map.contains_key("c1"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T012 [US1] Edge cases
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn zero_citations_returns_empty() {
        let (resources, map) = generate_back_matter(&[]).unwrap();
        assert!(resources.is_empty());
        assert!(map.is_empty());
    }

    #[test]
    fn two_identical_citations_share_one_resource() {
        let citations = vec![
            url_citation("c1", "Same", "https://example.com"),
            url_citation("c2", "Same", "https://example.com"),
        ];
        let (resources, map) = generate_back_matter(&citations).unwrap();
        // Identical content yields ONE resource; both citation ids resolve to it (F0618).
        assert_eq!(resources.len(), 1);
        assert_eq!(map.get("c1"), map.get("c2"));
        assert_eq!(*map.get("c1").unwrap(), resources[0].uuid);
    }

    #[test]
    fn dangerous_url_with_empty_text_is_redacted_in_title() {
        let citations = vec![url_citation("c1", "", "javascript:alert(1)")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].title, "[unsafe URL scheme removed]");
        assert!(
            !resources[0].title.contains("javascript"),
            "payload must not reach the title (F0620)"
        );
    }

    #[test]
    fn resource_includes_description_when_source_requirement_id_present() {
        let citations =
            vec![citation_with_req("c1", "NIST ref", Some("https://nist.gov"), "req-123")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(resources[0].description.is_some());
        assert!(resources[0].description.as_ref().unwrap().contains("req-123"));
    }

    #[test]
    fn citation_with_empty_text_and_no_url_skipped() {
        let citations = vec![
            biblio_citation("c1", "Valid citation"),
            Citation {
                id: "c2".to_string(),
                text: String::new(),
                url: None,
                source_requirement_id: None,
            },
        ];
        let (resources, map) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].title, "Valid citation");
        assert!(!map.contains_key("c2"));
    }

    #[test]
    fn citation_with_empty_id_returns_error() {
        let citations = vec![Citation {
            id: String::new(),
            text: "Some text".to_string(),
            url: None,
            source_requirement_id: None,
        }];
        let result = generate_back_matter(&citations);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_citation_id_returns_error() {
        let citations = vec![
            url_citation("c1", "First reference", "https://example.com/first"),
            url_citation("c1", "Second reference", "https://example.com/second"),
        ];

        let error = generate_back_matter(&citations).unwrap_err();

        assert!(matches!(error, ForgeError::BackMatter(_)));
        assert!(error.to_string().contains("duplicate citation id: c1"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T015 [US2] generate_control_links — basic behavior
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn single_citation_produces_one_link_with_reference_rel() {
        let citations = vec![url_citation("c1", "NIST", "https://nist.gov")];
        let (_, map) = generate_back_matter(&citations).unwrap();
        let links = generate_control_links(&citations, &map);
        assert_eq!(links.len(), 1);
        assert!(links[0].href.starts_with('#'));
        assert_eq!(links[0].rel, "reference");
        let expected_href = format!("#{}", map["c1"]);
        assert_eq!(links[0].href, expected_href);
    }

    #[test]
    fn two_citations_produce_two_links_with_correct_hrefs() {
        let citations = vec![
            url_citation("c1", "Ref A", "https://example.com/a"),
            biblio_citation("c2", "Ref B"),
        ];
        let (_, map) = generate_back_matter(&citations).unwrap();
        let links = generate_control_links(&citations, &map);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, format!("#{}", map["c1"]));
        assert_eq!(links[1].href, format!("#{}", map["c2"]));
    }

    #[test]
    fn link_text_populated_from_citation_text() {
        let citations = vec![url_citation("c1", "NIST SP 800-53", "https://nist.gov")];
        let (_, map) = generate_back_matter(&citations).unwrap();
        let links = generate_control_links(&citations, &map);
        assert_eq!(links[0].text.as_deref(), Some("NIST SP 800-53"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T016 [US2] generate_control_links — orphan handling
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn orphan_citation_skipped_no_panic() {
        let citations = vec![biblio_citation("orphan", "Not in map")];
        let empty_map = HashMap::new();
        let links = generate_control_links(&citations, &empty_map);
        assert!(links.is_empty());
    }

    #[test]
    fn empty_citations_produces_empty_links() {
        let map = HashMap::new();
        let links = generate_control_links(&[], &map);
        assert!(links.is_empty());
    }

    #[test]
    fn only_requested_citations_generate_links() {
        let citations = vec![biblio_citation("c1", "Ref A")];
        let mut map = HashMap::new();
        map.insert("c1".to_string(), Uuid::new_v4());
        map.insert("c99".to_string(), Uuid::new_v4());
        let links = generate_control_links(&citations, &map);
        assert_eq!(links.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T019 [US3] No remarks in serialized output
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn url_resource_json_has_no_remarks() {
        let citations = vec![url_citation("c1", "NIST", "https://nist.gov")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        let json = serde_json::to_string(&resources[0]).unwrap();
        assert!(!json.contains("remarks"), "JSON should not contain 'remarks': {json}");
    }

    #[test]
    fn bibliographic_resource_json_has_no_remarks() {
        let citations = vec![biblio_citation("c1", "NIST SP 800-53 Rev 5")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        let json = serde_json::to_string(&resources[0]).unwrap();
        assert!(!json.contains("remarks"), "JSON should not contain 'remarks': {json}");
    }

    #[test]
    fn back_matter_with_multiple_resources_json_has_no_remarks() {
        let citations = vec![
            url_citation("c1", "URL Ref", "https://example.com"),
            biblio_citation("c2", "Biblio Ref"),
            url_citation("c3", "Malformed", "not-a-url"),
        ];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        let bm = BackMatter { resources };
        let json = serde_json::to_string(&bm).unwrap();
        assert!(!json.contains("remarks"), "JSON should not contain 'remarks': {json}");
    }

    #[test]
    fn metadata_stored_as_prop_not_remarks() {
        let citations = vec![url_citation("c1", "Bad", "not-a-url")];
        let (resources, _) = generate_back_matter(&citations).unwrap();
        assert!(!resources[0].props.is_empty(), "Should have prop annotations");
        let json = serde_json::to_string(&resources[0]).unwrap();
        assert!(json.contains("\"props\""), "Should use props: {json}");
        assert!(!json.contains("remarks"), "Should not use remarks: {json}");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T023 Integration test — end-to-end flow
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn integration_back_matter_and_control_links_end_to_end() {
        let citations = vec![
            url_citation("c1", "NIST SP 800-53", "https://nvd.nist.gov/800-53"),
            biblio_citation("c2", "ISO 27001:2022"),
            url_citation("c3", "OWASP Top 10", "https://owasp.org/top10"),
        ];

        let (resources, map) = generate_back_matter(&citations).unwrap();
        assert_eq!(resources.len(), 3);
        assert_eq!(map.len(), 3);

        let links = generate_control_links(&citations, &map);
        assert_eq!(links.len(), 3);

        // Verify link hrefs resolve to resource UUIDs
        for (link, resource) in links.iter().zip(resources.iter()) {
            let expected_href = format!("#{}", resource.uuid);
            assert_eq!(link.href, expected_href);
        }

        // Verify JSON serialization of full BackMatter struct
        let bm = BackMatter { resources };
        let json = serde_json::to_string_pretty(&bm).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let res_array = parsed["resources"].as_array().unwrap();
        assert_eq!(res_array.len(), 3);
        assert!(res_array[0]["uuid"].is_string());
        assert!(res_array[0]["title"].is_string());
        assert!(!json.contains("remarks"));
    }

    #[test]
    fn integration_zero_citations_produces_none_back_matter() {
        let (resources, map) = generate_back_matter(&[]).unwrap();
        assert!(resources.is_empty());
        assert!(map.is_empty());
        // Zero citations → back_matter should be None on catalog
        let bm: Option<BackMatter> =
            if resources.is_empty() { None } else { Some(BackMatter { resources }) };
        assert!(bm.is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // T028 Quickstart usage pattern validation
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn quickstart_usage_pattern_compiles_and_works() {
        use crate::model::Citation;
        use crate::oscal::back_matter::{generate_back_matter, generate_control_links};

        // Create citations (normally from WI-8 extraction)
        let citations = vec![Citation {
            id: "cit-1".into(),
            text: "NIST SP 800-53 Rev 5".into(),
            url: Some("https://nvd.nist.gov/800-53".into()),
            source_requirement_id: Some("req-uuid-here".into()),
        }];

        // Generate back matter resources + resource map
        let (resources, resource_map) = generate_back_matter(&citations).unwrap();

        // Generate control links using the resource map
        let links = generate_control_links(&citations, &resource_map);

        // Validate quickstart assertions
        assert_eq!(resources.len(), 1);
        assert_eq!(links.len(), 1);
        assert!(links[0].href.starts_with('#'));
        assert_eq!(resource_map.len(), 1);
        assert!(resource_map.contains_key("cit-1"));
    }

    // ── Task 9: ns field on Prop ─────────────────────────

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
}
