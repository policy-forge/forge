//! The closed resource index: explicit registration, no filesystem discovery.

use std::collections::BTreeSet;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::contract::{self, Error, Result};

pub(crate) const INDEX_PATH: &str = "forge.workspace.json";
pub(crate) const MAX_INDEX_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Role {
    PolicySource,
    OscalCatalogArtifact,
    OscalComponentArtifact,
    MappingCollection,
    ApplicabilityManifest,
    ApplicabilityReport,
    TraceReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Resource {
    pub key: String,
    pub role: Role,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Index {
    pub schema_version: String,
    pub label: String,
    pub resources: Vec<Resource>,
}

static SCHEMA: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../../schemas/forge.workspace-1.schema.json"))
            .expect("embedded index schema is tested");
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .expect("embedded index schema compiles")
});

impl Index {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: "forge.workspace/1".into(),
            label: "Local project".into(),
            resources: Vec::new(),
        }
    }

    pub(crate) fn parse(bytes: &[u8]) -> Result<Self> {
        let value = contract::parse(bytes, MAX_INDEX_BYTES, 4096)?;
        if !SCHEMA.is_valid(&value) {
            return Err(Error::invalid());
        }
        let index: Self = serde_json::from_value(value).map_err(|_| Error::invalid())?;
        if index.label.chars().any(char::is_control) {
            return Err(Error::invalid());
        }
        let mut keys = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for resource in &index.resources {
            validate_path(&resource.path)?;
            if !keys.insert(&resource.key) || !paths.insert(resource.path.to_ascii_lowercase()) {
                return Err(Error::invalid());
            }
            if resource.path.eq_ignore_ascii_case(INDEX_PATH) {
                return Err(Error::invalid());
            }
        }
        Ok(index)
    }

    pub(crate) fn bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = contract::encode(self, MAX_INDEX_BYTES - 1, true)?;
        bytes.push(b'\n');
        Self::parse(&bytes)?;
        Ok(bytes)
    }
}

/// Portable spelling is narrower than OS paths. Reject device names and aliases
/// on every platform so a project cannot change meaning when moved to Windows.
pub(crate) fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() || path.len() > 512 {
        return Err(Error::containment());
    }
    for segment in path.split('/') {
        if !segment.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
            || !segment.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || segment.ends_with('.')
        {
            return Err(Error::containment());
        }
        let stem = segment.split('.').next().unwrap_or_default().to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (stem.len() == 4
                && (stem.starts_with("COM") || stem.starts_with("LPT"))
                && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(Error::containment());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ordered_closed_index_round_trips() {
        let value = json!({"schema_version":"forge.workspace/1", "label":"Example", "resources":[
            {"key":"z", "role":"policy-source", "path":"policies/Z.md"},
            {"key":"a", "role":"oscal-catalog-artifact", "path":"catalog.json"}
        ]});
        let index = Index::parse(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert_eq!(index.resources[0].key, "z");
        assert_eq!(
            index.bytes().unwrap(),
            Index::parse(&index.bytes().unwrap()).unwrap().bytes().unwrap()
        );
    }

    #[test]
    fn closed_versions_duplicates_and_aliases_fail() {
        for resources in [
            json!([{"key":"a","role":"policy-source","path":"x.md"},{"key":"a","role":"policy-source","path":"y.md"}]),
            json!([{"key":"a","role":"policy-source","path":"X.md"},{"key":"b","role":"policy-source","path":"x.md"}]),
            json!([{"key":"a","role":"policy-source","path":"forge.workspace.json"}]),
        ] {
            assert!(Index::parse(&serde_json::to_vec(&json!({"schema_version":"forge.workspace/1","label":"X","resources":resources})).unwrap()).is_err());
        }
        assert!(
            Index::parse(br#"{"schema_version":"forge.workspace/2","label":"X","resources":[]}"#)
                .is_err()
        );
        assert!(Index::parse(br#"{"schema_version":"forge.workspace/1","label":"X","resources":[],"secret":"x"}"#).is_err());
    }

    #[test]
    fn hostile_portable_paths_fail_before_io() {
        for path in [
            "",
            "/etc/passwd",
            "../x",
            "a/./x",
            "C:/x",
            "a\\x",
            "a//x",
            "a/",
            "a/CON.txt",
            "COM1",
            "x.",
            "a/%2e%2e/x",
            "a/é.md",
        ] {
            assert!(validate_path(path).is_err(), "{path}");
        }
        validate_path("policies/access-control_v1.md").unwrap();
    }
}
