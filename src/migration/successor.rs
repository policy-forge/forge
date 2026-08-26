//! Strict reviewer-authored successor, split, and merge declarations.

use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::Read as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ForgeError;

pub const SUCCESSOR_MAP_SCHEMA_VERSION: &str = "forge.successor-map/1";
pub const MAX_SUCCESSOR_MAP_BYTES: u64 = 2 * 1024 * 1024;
const MAX_RELATIONSHIPS: usize = 10_000;
const MAX_IDS_PER_RELATIONSHIP: usize = 1_000;
const MAX_STRING_BYTES: usize = 64 * 1024;
const MAX_JSON_DEPTH: usize = 64;
const STRICT_JSON_LIMITS: crate::json_strict::Limits =
    crate::json_strict::Limits { max_depth: MAX_JSON_DEPTH, max_string_bytes: MAX_STRING_BYTES };

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorMap {
    pub schema_version: String,
    pub relationships: Vec<SuccessorRelationship>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorRelationship {
    pub relationship: RelationshipType,
    pub old_ids: Vec<String>,
    pub new_ids: Vec<String>,
    pub approved_by: String,
    pub approved_at: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    Successor,
    Split,
    Merge,
}

impl RelationshipType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Successor => "successor",
            Self::Split => "split",
            Self::Merge => "merge",
        }
    }
}

/// Read and validate a local successor map.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] for unsafe files, invalid JSON, unsupported contracts,
/// conflicting declarations, invalid cardinality, or invalid approval evidence.
pub fn load(path: &Path) -> Result<SuccessorMap, ForgeError> {
    let file = open_regular_nofollow(path)?;
    load_open_file(file, path)
}

fn open_regular_nofollow(path: &Path) -> Result<File, ForgeError> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    #[cfg(not(any(unix, windows)))]
    return Err(error("atomic no-follow successor-map opening is unsupported on this platform"));

    let file = options.open(path).map_err(|cause| {
        error(format!("cannot open successor map '{}': {cause}", path.display()))
    })?;
    let metadata = file.metadata().map_err(|cause| {
        error(format!("cannot inspect successor map '{}': {cause}", path.display()))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(error("successor map must not be a symbolic link"));
    }
    if !metadata.is_file() {
        return Err(error("successor map must be a regular file"));
    }
    Ok(file)
}

fn load_open_file(file: File, path: &Path) -> Result<SuccessorMap, ForgeError> {
    let metadata = file.metadata().map_err(|cause| {
        error(format!("cannot inspect successor map '{}': {cause}", path.display()))
    })?;
    if metadata.len() > MAX_SUCCESSOR_MAP_BYTES {
        return Err(error(format!(
            "successor map exceeds the {MAX_SUCCESSOR_MAP_BYTES} byte limit"
        )));
    }
    let mut bytes = Vec::new();
    file.take(MAX_SUCCESSOR_MAP_BYTES + 1).read_to_end(&mut bytes).map_err(|cause| {
        error(format!("cannot read successor map '{}': {cause}", path.display()))
    })?;
    if bytes.len() as u64 > MAX_SUCCESSOR_MAP_BYTES {
        return Err(error(format!(
            "successor map exceeds the {MAX_SUCCESSOR_MAP_BYTES} byte limit"
        )));
    }
    parse(&bytes)
}

/// Parse and validate a duplicate-key-safe successor map.
///
/// # Errors
///
/// Returns [`ForgeError::MigrationError`] when the map is invalid.
pub fn parse(bytes: &[u8]) -> Result<SuccessorMap, ForgeError> {
    if bytes.len() as u64 > MAX_SUCCESSOR_MAP_BYTES {
        return Err(error(format!(
            "successor map exceeds the {MAX_SUCCESSOR_MAP_BYTES} byte limit"
        )));
    }
    let value = crate::json_strict::parse_value(bytes, "successor map", STRICT_JSON_LIMITS)
        .map_err(error)?;
    let mut map: SuccessorMap = serde_json::from_value(value)
        .map_err(|cause| error(format!("invalid successor map contract: {cause}")))?;
    validate(&mut map)?;
    Ok(map)
}

fn validate(map: &mut SuccessorMap) -> Result<(), ForgeError> {
    if map.schema_version != SUCCESSOR_MAP_SCHEMA_VERSION {
        return Err(error(format!(
            "unsupported successor map schema_version '{}'; expected {SUCCESSOR_MAP_SCHEMA_VERSION}",
            crate::json_strict::bounded(&map.schema_version)
        )));
    }
    if map.relationships.is_empty() || map.relationships.len() > MAX_RELATIONSHIPS {
        return Err(error(format!(
            "$.relationships must contain between 1 and {MAX_RELATIONSHIPS} declarations"
        )));
    }
    let mut used_old = BTreeSet::new();
    let mut used_new = BTreeSet::new();
    for (index, relationship) in map.relationships.iter_mut().enumerate() {
        let path = format!("$.relationships[{index}]");
        validate_cardinality(&path, relationship)?;
        normalize_ids(&format!("{path}.old_ids"), &mut relationship.old_ids)?;
        normalize_ids(&format!("{path}.new_ids"), &mut relationship.new_ids)?;
        if relationship.old_ids.iter().any(|id| relationship.new_ids.binary_search(id).is_ok()) {
            return Err(error(format!("{path} must not map an identifier to itself")));
        }
        for id in &relationship.old_ids {
            if !used_old.insert(id.clone()) {
                return Err(error(format!(
                    "{path}.old_ids reuses identifier '{}' in conflicting declarations",
                    crate::json_strict::bounded(id)
                )));
            }
        }
        for id in &relationship.new_ids {
            if !used_new.insert(id.clone()) {
                return Err(error(format!(
                    "{path}.new_ids reuses identifier '{}' in conflicting declarations",
                    crate::json_strict::bounded(id)
                )));
            }
        }
        validate_nonempty(&format!("{path}.approved_by"), &relationship.approved_by)?;
        validate_nonempty(&format!("{path}.rationale"), &relationship.rationale)?;
        validate_nonempty(&format!("{path}.approved_at"), &relationship.approved_at)?;
        chrono::DateTime::parse_from_rfc3339(&relationship.approved_at)
            .map_err(|_| error(format!("{path}.approved_at must be an RFC 3339 timestamp")))?;
    }
    map.relationships.sort_by(|left, right| {
        (left.relationship, left.old_ids.as_slice(), left.new_ids.as_slice()).cmp(&(
            right.relationship,
            right.old_ids.as_slice(),
            right.new_ids.as_slice(),
        ))
    });
    Ok(())
}

fn validate_cardinality(
    path: &str,
    relationship: &SuccessorRelationship,
) -> Result<(), ForgeError> {
    let valid = match relationship.relationship {
        RelationshipType::Successor => {
            relationship.old_ids.len() == 1 && relationship.new_ids.len() == 1
        }
        RelationshipType::Split => {
            relationship.old_ids.len() == 1 && relationship.new_ids.len() >= 2
        }
        RelationshipType::Merge => {
            relationship.old_ids.len() >= 2 && relationship.new_ids.len() == 1
        }
    };
    if !valid
        || relationship.old_ids.len() > MAX_IDS_PER_RELATIONSHIP
        || relationship.new_ids.len() > MAX_IDS_PER_RELATIONSHIP
    {
        return Err(error(format!(
            "{path} has invalid {} cardinality",
            relationship.relationship.as_str()
        )));
    }
    Ok(())
}

fn normalize_ids(path: &str, ids: &mut [String]) -> Result<(), ForgeError> {
    for (index, id) in ids.iter().enumerate() {
        validate_nonempty(&format!("{path}[{index}]"), id)?;
    }
    ids.sort();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(error(format!("{path} contains a duplicate identifier")));
    }
    Ok(())
}

fn validate_nonempty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() || value.len() > MAX_STRING_BYTES {
        return Err(error(format!("{path} must contain between 1 and {MAX_STRING_BYTES} bytes")));
    }
    Ok(())
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::MigrationError(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_relationship_order_and_rejects_conflicts() {
        let parsed = parse(
            br#"{"schema_version":"forge.successor-map/1","relationships":[{"relationship":"split","old_ids":["old"],"new_ids":["new-b","new-a"],"approved_by":"reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"reviewed split"}]}"#,
        )
        .unwrap();
        assert_eq!(parsed.relationships[0].new_ids, ["new-a", "new-b"]);

        let duplicate = parse(
            br#"{"schema_version":"forge.successor-map/1","schema_version":"forge.successor-map/1","relationships":[]}"#,
        )
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate object key"));
    }

    #[test]
    fn relationship_as_str_values_match_their_serialized_contracts() {
        for value in [RelationshipType::Successor, RelationshipType::Split, RelationshipType::Merge]
        {
            assert_eq!(serde_json::to_value(value).unwrap(), value.as_str());
        }
    }

    #[test]
    fn load_reads_the_validated_handle_with_a_hard_byte_limit() {
        let directory = tempfile::tempdir().unwrap();
        let valid_path = directory.path().join("valid.json");
        std::fs::write(
            &valid_path,
            br#"{"schema_version":"forge.successor-map/1","relationships":[{"relationship":"successor","old_ids":["old"],"new_ids":["new"],"approved_by":"reviewer","approved_at":"2026-08-25T12:00:00Z","rationale":"reviewed successor"}]}"#,
        )
        .unwrap();
        assert_eq!(load(&valid_path).unwrap().relationships.len(), 1);

        let oversized_path = directory.path().join("oversized.json");
        let oversized = std::fs::File::create(&oversized_path).unwrap();
        oversized.set_len(MAX_SUCCESSOR_MAP_BYTES + 1).unwrap();
        let error = load(&oversized_path).unwrap_err();
        assert!(error.to_string().contains("byte limit"));
    }

    #[cfg(unix)]
    #[test]
    fn opened_handle_survives_directory_entry_replacement_without_following_the_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("successor.json");
        let moved_path = directory.path().join("opened-successor.json");
        let replacement = directory.path().join("replacement.json");
        std::fs::write(&path, successor_json("original-old", "original-new")).unwrap();
        std::fs::write(&replacement, successor_json("replacement-old", "replacement-new")).unwrap();

        let opened = open_regular_nofollow(&path).unwrap();
        std::fs::rename(&path, &moved_path).unwrap();
        std::os::unix::fs::symlink(&replacement, &path).unwrap();

        let parsed = load_open_file(opened, &path).unwrap();
        assert_eq!(parsed.relationships[0].old_ids, ["original-old"]);
        assert_eq!(parsed.relationships[0].new_ids, ["original-new"]);
        assert!(load(&path).unwrap_err().to_string().contains("cannot open successor map"));
    }

    fn successor_json(old_id: &str, new_id: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": SUCCESSOR_MAP_SCHEMA_VERSION,
            "relationships": [{
                "relationship": "successor",
                "old_ids": [old_id],
                "new_ids": [new_id],
                "approved_by": "reviewer",
                "approved_at": "2026-08-25T12:00:00Z",
                "rationale": "reviewed successor"
            }]
        }))
        .unwrap()
    }
}
