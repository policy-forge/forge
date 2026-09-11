//! Closed, bounded `forge.reuse-corpus/1`: operator-supplied candidate documents.
//!
//! The corpus is a pin, not a discovery mechanism. Every document is named by a
//! portable descendant path and an exact SHA-256, carries an asserted rights and
//! source label, and is never fetched, inferred, or bundled with FORGE.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ForgeError;
use crate::json_strict::{self, Limits};

/// Closed corpus contract version.
pub const CORPUS_SCHEMA_VERSION: &str = "forge.reuse-corpus/1";
/// Maximum corpus manifest size.
pub const MAX_CORPUS_BYTES: u64 = 2 * 1024 * 1024;
/// Maximum candidate documents in one corpus.
pub const MAX_DOCUMENTS: usize = 512;
/// Maximum size of one candidate document.
pub const MAX_DOCUMENT_BYTES: u64 = 1024 * 1024;
/// Maximum bytes in one declared string.
pub const MAX_STRING_BYTES: usize = 16 * 1024;
/// Maximum bytes in one stable key.
pub const MAX_KEY_BYTES: usize = 64;
/// Maximum framework control identifier length.
pub const MAX_CONTROL_ID_BYTES: usize = 256;
/// Maximum topic or control hints per document.
pub const MAX_HINTS: usize = 128;

const LIMITS: Limits = Limits { max_depth: 16, max_string_bytes: MAX_STRING_BYTES };

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// Whether the operator has approved the document for reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentStatus {
    /// The operator labels the document approved. An assertion, not a grant.
    Approved,
    /// The operator labels the document draft; excluded unless explicitly included.
    Draft,
}

/// One operator-supplied candidate document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusDocument {
    /// Stable document key.
    pub key: String,
    /// Portable descendant path relative to the workspace root.
    pub path: PathBuf,
    /// Human title.
    pub title: String,
    /// Operator-declared status.
    pub status: DocumentStatus,
    /// Asserted rights label; FORGE does not grant rights.
    pub rights_label: String,
    /// Asserted provenance label for the document's origin.
    pub source_label: String,
    /// Exact expected SHA-256 of the document bytes.
    pub expected_sha256: String,
    /// Optional topic keys this document is known to serve.
    #[serde(default)]
    pub topic_keys: Vec<String>,
    /// Optional framework control identifiers this document is known to address.
    #[serde(default)]
    pub control_ids: Vec<String>,
    /// Optional key of the document this one replaces.
    #[serde(default)]
    pub supersedes: Option<String>,
}

/// A parsed, validated corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    /// Closed contract version.
    pub schema_version: String,
    /// Stable corpus key.
    pub corpus_key: String,
    /// Optional human title.
    #[serde(default)]
    pub title: Option<String>,
    /// Candidate documents.
    pub documents: Vec<CorpusDocument>,
}

impl Corpus {
    /// Parse and validate a bounded, closed corpus manifest.
    ///
    /// # Errors
    /// Returns an authoring error for oversized input, unknown, duplicate or
    /// null keys, an unsupported version, unsafe paths, malformed digests, or
    /// inconsistent references.
    pub fn parse(bytes: &[u8]) -> Result<Self, ForgeError> {
        if bytes.len() as u64 > MAX_CORPUS_BYTES {
            return Err(error(format!("reuse corpus exceeds the {MAX_CORPUS_BYTES} byte limit")));
        }
        let value = json_strict::parse_value(bytes, "reuse corpus", LIMITS)
            .map_err(|cause| error(cause.to_string()))?;
        // Every declared field is present or absent; null is never a legal value,
        // and Serde would otherwise fold it into an absent optional field.
        reject_nulls(&value, "reuse corpus")?;
        let corpus: Self = serde_json::from_value(value)
            .map_err(|cause| error(format!("invalid reuse corpus contract: {cause}")))?;
        corpus.validate()?;
        Ok(corpus)
    }

    /// Candidate documents.
    #[must_use]
    pub fn documents(&self) -> &[CorpusDocument] {
        &self.documents
    }

    fn validate(&self) -> Result<(), ForgeError> {
        if self.schema_version != CORPUS_SCHEMA_VERSION {
            return Err(error(format!(
                "reuse corpus schema_version must be {CORPUS_SCHEMA_VERSION}"
            )));
        }
        key("corpus_key", &self.corpus_key)?;
        if let Some(title) = &self.title {
            single_line("corpus title", title)?;
        }
        if self.documents.is_empty() {
            return Err(error("reuse corpus declares no documents"));
        }
        if self.documents.len() > MAX_DOCUMENTS {
            return Err(error(format!(
                "reuse corpus declares more than {MAX_DOCUMENTS} documents"
            )));
        }
        let mut keys = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for document in &self.documents {
            key("document key", &document.key)?;
            if !keys.insert(document.key.as_str()) {
                return Err(error("reuse corpus document keys must be unique"));
            }
            crate::authoring::manifest::validate_local_path("document path", &document.path)?;
            if document.path.extension().and_then(|value| value.to_str()) != Some("md") {
                return Err(error("document path must name a Markdown file"));
            }
            let path_key = document
                .path
                .to_str()
                .ok_or_else(|| error("document path must be UTF-8"))?
                .to_ascii_lowercase();
            if !paths.insert(path_key) {
                return Err(error("reuse corpus document paths must be distinct"));
            }
            single_line("document title", &document.title)?;
            label("rights label", &document.rights_label)?;
            label("source label", &document.source_label)?;
            json_strict::validate_lowercase_sha256(
                "document expected_sha256",
                &document.expected_sha256,
            )
            .map_err(error)?;
            unique_keys("document topic_keys", &document.topic_keys, true)?;
            unique_keys("document control_ids", &document.control_ids, false)?;
        }
        for document in &self.documents {
            if let Some(previous) = &document.supersedes {
                if !keys.contains(previous.as_str()) || previous == &document.key {
                    return Err(error(
                        "document supersedes must name another document in this corpus",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn reject_nulls(value: &Value, path: &str) -> Result<(), ForgeError> {
    match value {
        Value::Null => Err(error(format!("{path} must not be null; omit the key instead"))),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                reject_nulls(item, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        Value::Object(entries) => {
            for (key, item) in entries {
                reject_nulls(item, &format!("{path}.{}", json_strict::bounded(key)))?;
            }
            Ok(())
        }
        Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(()),
    }
}

fn unique_keys(name: &str, values: &[String], as_key: bool) -> Result<(), ForgeError> {
    if values.len() > MAX_HINTS {
        return Err(error(format!("{name} exceeds {MAX_HINTS} entries")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if as_key {
            key(name, value)?;
        } else {
            control_id(name, value)?;
        }
        if !seen.insert(value.as_str()) {
            return Err(error(format!("{name} entries must be unique")));
        }
    }
    Ok(())
}

fn key(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > MAX_KEY_BYTES
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(error(format!(
            "{name} must be lowercase ASCII kebab-case, at most {MAX_KEY_BYTES} bytes"
        )));
    }
    Ok(())
}

fn control_id(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.len() > MAX_CONTROL_ID_BYTES
        || value.chars().any(char::is_whitespace)
        || value.chars().any(char::is_control)
    {
        return Err(error(format!(
            "{name} must be a non-empty control identifier of at most {MAX_CONTROL_ID_BYTES} bytes"
        )));
    }
    Ok(())
}

fn single_line(name: &str, value: &str) -> Result<(), ForgeError> {
    if value.is_empty()
        || value.trim() != value
        || value.chars().any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(error(format!("{name} must be one non-empty line")));
    }
    Ok(())
}

fn label(name: &str, value: &str) -> Result<(), ForgeError> {
    single_line(name, value)?;
    if value.starts_with(['/', '\\'])
        || value.starts_with("file:")
        || value.as_bytes().get(1) == Some(&b':')
    {
        return Err(error(format!("{name} must not be a rooted local path")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn manifest() -> Value {
        json!({
            "schema_version": CORPUS_SCHEMA_VERSION,
            "corpus_key": "synthetic-corpus",
            "title": "Synthetic corpus",
            "documents": [{
                "key": "access-policy",
                "path": "prior/access-policy.md",
                "title": "Access policy",
                "status": "approved",
                "rights_label": "Repository synthetic fixture",
                "source_label": "Synthetic interview",
                "expected_sha256": DIGEST,
                "topic_keys": ["access-topic"],
                "control_ids": ["ac-1"]
            }, {
                "key": "old-access-policy",
                "path": "prior/old-access.md",
                "title": "Old access policy",
                "status": "draft",
                "rights_label": "Repository synthetic fixture",
                "source_label": "Synthetic interview",
                "expected_sha256": DIGEST,
                "supersedes": "access-policy"
            }]
        })
    }

    fn parse(value: &Value) -> Result<Corpus, ForgeError> {
        Corpus::parse(&serde_json::to_vec(value).unwrap())
    }

    #[test]
    fn valid_corpus_round_trips_with_every_field() {
        let corpus = parse(&manifest()).unwrap();
        assert_eq!(corpus.schema_version, CORPUS_SCHEMA_VERSION);
        assert_eq!(corpus.documents().len(), 2);
        assert_eq!(corpus.documents()[0].status, DocumentStatus::Approved);
        assert_eq!(corpus.documents()[1].status, DocumentStatus::Draft);
        assert_eq!(corpus.documents()[1].supersedes.as_deref(), Some("access-policy"));
        assert_eq!(corpus.documents()[0].path.to_str(), Some("prior/access-policy.md"));
    }

    #[test]
    fn closed_corpus_rejects_unknown_duplicate_null_and_forward_versions() {
        let mut unknown = manifest();
        unknown["documents"][0]["pointer"] = json!("nope");
        assert!(parse(&unknown).is_err(), "unknown field must fail");

        let mut forward = manifest();
        forward["schema_version"] = json!("forge.reuse-corpus/2");
        assert!(parse(&forward).is_err(), "forward version must fail");

        let mut null_title = manifest();
        null_title["title"] = Value::Null;
        assert!(parse(&null_title).is_err(), "null where absence is intended must fail");

        let decoded = format!(
            "{{\"schema_version\":\"{CORPUS_SCHEMA_VERSION}\",\"corpus_key\":\"a\",\"corpus_key\":\"b\",\"documents\":[]}}"
        );
        assert!(Corpus::parse(decoded.as_bytes()).is_err(), "duplicate decoded key must fail");
    }

    #[test]
    fn corpus_rejects_unsafe_or_ambiguous_documents() {
        for (pointer, replacement) in [
            ("/documents/0/key", json!("Access")),
            ("/documents/0/path", json!("/absolute/access.md")),
            ("/documents/0/path", json!("../outside.md")),
            ("/documents/0/path", json!("prior/access.txt")),
            ("/documents/0/expected_sha256", json!("ABCDEF")),
            ("/documents/0/title", json!("two\nlines")),
            ("/documents/0/rights_label", json!("/private/input")),
            ("/documents/0/status", json!("published")),
            ("/documents/0/control_ids", json!(["ac 1"])),
            ("/documents/1/supersedes", json!("missing-key")),
            ("/documents/1/supersedes", json!("old-access-policy")),
        ] {
            let mut value = manifest();
            let steps: Vec<&str> = pointer.trim_start_matches('/').split('/').collect();
            let (last, parents) = steps.split_last().expect("pointer has a leaf");
            let mut cursor = &mut value;
            for step in parents {
                if let Ok(index) = step.parse::<usize>() {
                    cursor = &mut cursor[index];
                } else {
                    cursor = &mut cursor[*step];
                }
            }
            cursor[*last] = replacement.clone();
            assert!(parse(&value).is_err(), "{pointer} = {replacement} must fail");
        }
    }

    #[test]
    fn corpus_rejects_duplicate_keys_and_paths_case_insensitively() {
        let mut duplicate_key = manifest();
        duplicate_key["documents"][1]["key"] = json!("access-policy");
        assert!(parse(&duplicate_key).is_err());

        let mut duplicate_path = manifest();
        duplicate_path["documents"][1]["path"] = json!("prior/ACCESS-Policy.md");
        assert!(parse(&duplicate_path).is_err());

        let mut empty = manifest();
        empty["documents"] = json!([]);
        assert!(parse(&empty).is_err());
    }

    #[test]
    fn corpus_rejects_oversized_input_before_parsing() {
        let oversized = vec![b' '; usize::try_from(MAX_CORPUS_BYTES).unwrap() + 1];
        assert!(Corpus::parse(&oversized).is_err());
    }

    #[test]
    fn corpus_schema_file_is_published_and_closed() {
        let schema: Value =
            serde_json::from_str(include_str!("../../schemas/forge.reuse-corpus-1.schema.json"))
                .unwrap();
        assert_eq!(schema["$id"], "https://policy-forge.github.io/schemas/reuse-corpus/1");
        assert_eq!(schema["additionalProperties"], json!(false));
    }
}
