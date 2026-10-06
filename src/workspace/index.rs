//! The closed resource index: explicit registration, no filesystem discovery.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use super::contract::{self, Error, Result};

/// Fixed project-relative index target; domain resources cannot register it.
pub(crate) const INDEX_PATH: &str = "forge.workspace.json";
/// Raw and normalized index byte ceiling, including its final newline.
pub(crate) const MAX_INDEX_BYTES: usize = 1024 * 1024;

/// Closed registered input families; new families require an explicit /2 index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Role {
    /// Supported human-authored Markdown conversion input.
    PolicySource,
    /// Native OSCAL Catalog, independently schema validated.
    OscalCatalogArtifact,
    /// Native OSCAL Component Definition, independently schema validated.
    OscalComponentArtifact,
    /// Mapping authoring manifest or native Mapping Collection; edges constrain representation.
    MappingCollection,
    /// Raw applicability authoring manifest, not a summarized report.
    ApplicabilityManifest,
    /// Stored applicability report with its existing admission checks.
    ApplicabilityReport,
    /// Existing bounded workspace trace report.
    TraceReport,
    /// Locally declared lifecycle record; actors are not authenticated.
    LifecycleRecord,
    /// Opaque bounded source fingerprint bytes, with no content-reader permission.
    LifecycleSource,
    /// Native OSCAL Profile; no implicit resolution or remote import.
    OscalProfileArtifact,
    /// Native OSCAL System Security Plan.
    OscalSspArtifact,
    /// Intrinsic framework-impact manifest; full captured dependency closure remains a read-service prerequisite.
    FrameworkImpactManifest,
    /// Supplied successor map with unauthenticated declared migration assertions.
    SuccessorMap,
    /// Historical prior report admitted structurally, without current freshness proof.
    FrameworkImpactReport,
    /// Supplied framework-impact dispositions, retaining raw findings and local assertions.
    FrameworkImpactDispositions,
}

impl Role {
    /// Check schema-specific role admission before an unregistered file can be read.
    pub(crate) fn admitted_by(self, schema_version: &str) -> bool {
        match schema_version {
            "forge.workspace/1" => matches!(
                self,
                Self::PolicySource
                    | Self::OscalCatalogArtifact
                    | Self::OscalComponentArtifact
                    | Self::MappingCollection
                    | Self::ApplicabilityManifest
                    | Self::ApplicabilityReport
                    | Self::TraceReport
            ),
            "forge.workspace/2" => true,
            _ => false,
        }
    }

    /// Name the narrower validation profile for new roles without asserting freshness.
    pub(crate) fn validation_profile(self) -> Option<&'static str> {
        match self {
            Self::LifecycleRecord => Some("lifecycle-record-structure"),
            Self::LifecycleSource => Some("opaque-fingerprint-bytes"),
            Self::OscalProfileArtifact | Self::OscalSspArtifact => Some("native-oscal-schema"),
            Self::FrameworkImpactManifest => Some("framework-impact-manifest"),
            Self::SuccessorMap => Some("successor-map"),
            Self::FrameworkImpactReport => Some("framework-impact-prior-admission"),
            Self::FrameworkImpactDispositions => Some("framework-impact-dispositions"),
            _ => None,
        }
    }
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

/// Independently compiled version schemas preserve the closed original /1 interpretation.
static SCHEMAS: LazyLock<BTreeMap<&'static str, jsonschema::Validator>> = LazyLock::new(|| {
    [
        ("forge.workspace/1", include_str!("../../schemas/forge.workspace-1.schema.json")),
        ("forge.workspace/2", include_str!("../../schemas/forge.workspace-2.schema.json")),
    ]
    .into_iter()
    .map(|(version, bytes)| {
        let schema: serde_json::Value =
            serde_json::from_str(bytes).expect("embedded index schema is tested");
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .build(&schema)
            .expect("embedded index schema compiles");
        (version, validator)
    })
    .collect()
});

impl Index {
    /// Preserve legacy absent-index behavior until a request explicitly selects /2.
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: "forge.workspace/1".into(),
            label: "Local project".into(),
            resources: Vec::new(),
        }
    }

    /// Admit one closed version without relaxing duplicate, portable-path or alias checks.
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self> {
        let value = contract::parse(bytes, MAX_INDEX_BYTES, 4096)?;
        let version = value["schema_version"].as_str().ok_or_else(Error::invalid)?;
        let schema = SCHEMAS.get(version).ok_or_else(Error::invalid)?;
        if !schema.is_valid(&value) {
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

    /// Encode ordered authorial registrations deterministically, validate and append newline.
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

    /// Unknown versions still reject after /2 admission; duplicate keys, path aliases and index self-registration remain closed.
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
            Index::parse(br#"{"schema_version":"forge.workspace/3","label":"X","resources":[]}"#)
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
