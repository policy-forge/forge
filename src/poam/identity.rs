//! Deterministic POA&M identities from immutable authored keys.
//!
//! Every UTF-8 segment is preceded by its byte length as an unsigned 64-bit
//! big-endian integer. The schema and object kind separate the identity domains;
//! mutable prose, sequence order, owners, dates and source assertions are absent.

use uuid::Uuid;

use super::manifest::MANIFEST_SCHEMA_VERSION;

/// Stable UUID v5 document identity for the exact UTF-8 plan key.
///
/// Uses the fixed Forge namespace and versioned, length-prefixed seed protocol.
/// Callers supply validated immutable keys; this function does not trim, normalize
/// or validate them. The resulting UUID conveys identity, not plan acceptance.
#[must_use]
pub fn document(plan_key: &str) -> Uuid {
    stable_uuid(&[MANIFEST_SCHEMA_VERSION, plan_key, "document"])
}

/// Stable UUID v5 item identity scoped by the exact plan and item keys.
///
/// Prose, source assertions, owners, dates and item ordering do not enter the seed.
/// Changing either immutable key changes identity; no remediation decision is inferred.
#[must_use]
pub fn item(plan_key: &str, item_key: &str) -> Uuid {
    stable_uuid(&[MANIFEST_SCHEMA_VERSION, plan_key, "item", item_key])
}

/// Stable UUID v5 milestone identity scoped by exact plan, item and milestone keys.
///
/// The milestone domain is distinct from document and item identities. Schedule
/// edits and completion assertions do not change identity or verify work completion.
#[must_use]
pub fn milestone(plan_key: &str, item_key: &str, milestone_key: &str) -> Uuid {
    stable_uuid(&[MANIFEST_SCHEMA_VERSION, plan_key, "milestone", item_key, milestone_key])
}

fn stable_uuid(segments: &[&str]) -> Uuid {
    let mut seed = Vec::new();
    for segment in segments {
        seed.extend_from_slice(&(segment.len() as u64).to_be_bytes());
        seed.extend_from_slice(segment.as_bytes());
    }
    Uuid::new_v5(&crate::uuid::FORGE_NAMESPACE_UUID, &seed)
}

#[cfg(test)]
mod tests {
    use uuid::{Variant, Version};

    use super::*;

    #[test]
    fn deterministic_identifiers_use_uuid_v5_and_distinct_domains() {
        let values = [document("plan"), item("plan", "work"), milestone("plan", "work", "step")];
        for value in values {
            assert_eq!(value.get_version(), Some(Version::Sha1));
            assert_eq!(value.get_variant(), Variant::RFC4122);
        }
        assert_eq!(document("plan"), document("plan"));
        assert_eq!(item("plan", "work"), item("plan", "work"));
        assert_eq!(milestone("plan", "work", "step"), milestone("plan", "work", "step"));
        assert_ne!(values[0], values[1]);
        assert_ne!(values[1], values[2]);
        assert_ne!(values[0], values[2]);
    }

    #[test]
    fn length_prefixes_prevent_delimiter_and_segment_boundary_aliases() {
        assert_ne!(item("ab", "c"), item("a", "bc"));
        assert_ne!(item("plan:work", "x"), item("plan", "work:x"));
        assert_ne!(milestone("plan", "ab", "c"), milestone("plan", "a", "bc"));
        assert_ne!(stable_uuid(&["a", "bc"]), stable_uuid(&["ab", "c"]));
        assert_ne!(stable_uuid(&["a", ""]), stable_uuid(&["a"]));
    }

    #[test]
    fn exact_keys_and_parent_scope_control_identity() {
        assert_ne!(document("plan"), document("other-plan"));
        assert_ne!(item("plan", "work"), item("other-plan", "work"));
        assert_ne!(item("plan", "work"), item("plan", "other-work"));
        assert_ne!(milestone("plan", "work", "step"), milestone("plan", "other-work", "step"));
        assert_ne!(item("plan", "work"), item("plan", " work"));
        assert_ne!(item("plan", "work"), item("plan", "WORK"));
        assert_ne!(item("plan", "é"), item("plan", "e\u{301}"));
    }

    #[test]
    fn utf8_lengths_and_frozen_seed_protocol_match_independent_fixtures() {
        // Independently calculated using Python hashlib over namespace + seed.
        assert_eq!(document("plan").to_string(), "7dbbab6a-71a2-5b85-b22c-ad1aa1cb4376");
        assert_eq!(item("plan", "work").to_string(), "f4ece507-56ad-50f1-bb12-99503e8a9f3b");
        assert_eq!(
            milestone("plan", "work", "step").to_string(),
            "6ef44ed2-b121-5c62-9f33-6bd36aa6c571"
        );
        assert_eq!(item("plan", "é").to_string(), "de48cf57-02e5-546a-9324-a5dd5fbe8923");
    }

    #[test]
    fn changing_document_prose_version_and_dates_preserves_key_identity() {
        let original = super::super::manifest::DocumentManifest {
            key: "plan".to_string(),
            title: "Original title".to_string(),
            version: "1.0.0".to_string(),
            last_modified: "2026-10-02T12:00:00Z".to_string(),
        };
        let mut edited = original.clone();
        edited.title = "Edited title".to_string();
        edited.version = "1.1.0".to_string();
        edited.last_modified = "2026-12-01T08:00:00Z".to_string();
        assert_eq!(document(&original.key), document(&edited.key));
        edited.key = "new-plan".to_string();
        assert_ne!(document(&original.key), document(&edited.key));
    }
}
