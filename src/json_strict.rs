//! Shared duplicate-key-safe JSON and JSON-compatible YAML parsing utilities.
//!
//! Limits apply to the decoded tree. Callers MUST still cap raw input bytes
//! before parsing because wide, shallow JSON may allocate before its structural
//! bounds can be inspected; `serde_json`'s own recursion limit can also reject a
//! document before `Limits::max_depth` when configured higher. YAML bounds likewise apply
//! after decoding; neither a raw-byte cap nor these bounds establishes confinement of
//! allocations or alias expansion during the YAML decoder itself.

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};

/// Structural limits applied after duplicate-key-safe JSON decoding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) max_depth: usize,
    pub(crate) max_string_bytes: usize,
}

/// Programmatic classification for strict JSON parsing failures.
#[derive(Debug, thiserror::Error)]
pub(crate) enum StrictJsonError {
    /// The input is not a complete JSON value.
    #[error("invalid {label} JSON: {source}")]
    InvalidJson {
        /// Caller-provided input label.
        label: String,
        /// Underlying JSON decoder error.
        #[source]
        source: serde_json::Error,
    },
    /// Valid JSON was followed by extra data.
    #[error("invalid trailing {label} data: {source}")]
    TrailingData {
        /// Caller-provided input label.
        label: String,
        /// Underlying JSON decoder error.
        #[source]
        source: serde_json::Error,
    },
    /// An object contains one duplicate key.
    #[error("duplicate object key '{key}'")]
    DuplicateKey {
        /// Bounded, escaped key for diagnostics.
        key: String,
    },
    /// A decoded value violates a configured structural bound.
    #[error("{message}")]
    BoundsViolation {
        /// Bounded diagnostic identifying the failed constraint.
        message: String,
    },
}

/// Preserve YAML decoder failures separately from decoded JSON-tree bounds.
///
/// Syntax, duplicate keys, unsupported scalar/container types, non-finite numbers,
/// and multiple documents retain the original decoder error and its location. A
/// bounds failure retains the same classification used by [`parse_value`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum StrictYamlError {
    /// YAML cannot be represented as one duplicate-free JSON-compatible value.
    #[error("invalid YAML: {source}")]
    Decode {
        /// Original YAML decoder error, including any available source location.
        #[source]
        source: serde_yaml::Error,
    },
    /// The decoded value exceeds a caller-supplied structural limit.
    #[error("{source}")]
    Bounds {
        /// Shared decoded-tree bound failure with an escaped diagnostic path.
        #[source]
        source: StrictJsonError,
    },
}

const DUPLICATE_KEY_PREFIX: &str = "forge-strict-json-duplicate-key:";

/// Parse one complete JSON value without duplicate object keys and enforce structural bounds.
pub(crate) fn parse_value(
    bytes: &[u8],
    label: &str,
    limits: Limits,
) -> Result<Value, StrictJsonError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let strict = StrictValue::deserialize(&mut deserializer).map_err(|source| {
        let message = source.to_string();
        if let Some(key) = message.strip_prefix(DUPLICATE_KEY_PREFIX) {
            StrictJsonError::DuplicateKey { key: key.to_string() }
        } else {
            StrictJsonError::InvalidJson { label: label.to_string(), source }
        }
    })?;
    deserializer
        .end()
        .map_err(|source| StrictJsonError::TrailingData { label: label.to_string(), source })?;
    enforce_bounds(&strict.0, &mut Vec::new(), 0, limits)?;
    Ok(strict.0)
}

/// Decode exactly one YAML document as a duplicate-free JSON-compatible value.
///
/// Mapping keys must decode as strings. Scalar types and array order are
/// preserved; supported ordinary aliases are expanded by the existing decoder.
/// Non-finite numbers and custom local tags have no JSON representation and are
/// rejected by the shared visitor. The decoder may erase nonlocal URI-form tags
/// before invoking the visitor; preservation of those tags or other YAML
/// presentation is not asserted. Callers must bound raw input separately.
///
/// The decoder has its own recursion/repetition limits; caller limits are checked
/// only after decoding. This helper does not establish allocation or alias-memory
/// confinement, even when the caller caps input bytes.
///
/// # Errors
///
/// Returns [`StrictYamlError::Decode`] with the original YAML error for decoder
/// failures and [`StrictYamlError::Bounds`] for decoded depth/string violations.
pub(crate) fn parse_yaml_value(content: &str, limits: Limits) -> Result<Value, StrictYamlError> {
    let strict = StrictValue::deserialize(serde_yaml::Deserializer::from_str(content))
        .map_err(|source| StrictYamlError::Decode { source })?;
    enforce_bounds(&strict.0, &mut Vec::new(), 0, limits)
        .map_err(|source| StrictYamlError::Bounds { source })?;
    Ok(strict.0)
}

/// Escape and truncate caller-controlled values before including them in diagnostics.
pub(crate) fn bounded(value: &str) -> String {
    value.chars().take(120).flat_map(char::escape_default).collect()
}

/// Validate the canonical lowercase hexadecimal representation of one SHA-256 digest.
pub(crate) fn validate_lowercase_sha256(path: &str, value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{path} must be 64 lowercase hexadecimal characters"));
    }
    Ok(())
}

enum PathSegment<'a> {
    Key(&'a str),
    Index(usize),
}

fn enforce_bounds<'a>(
    value: &'a Value,
    segments: &mut Vec<PathSegment<'a>>,
    depth: usize,
    limits: Limits,
) -> Result<(), StrictJsonError> {
    if depth > limits.max_depth {
        return Err(bounds_error(
            segments,
            format!("exceeds maximum JSON depth {}", limits.max_depth),
        ));
    }
    match value {
        Value::String(text) if text.len() > limits.max_string_bytes => Err(bounds_error(
            segments,
            format!("exceeds maximum string length {} bytes", limits.max_string_bytes),
        )),
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                segments.push(PathSegment::Index(index));
                let result = enforce_bounds(child, segments, depth + 1, limits);
                let _ = segments.pop();
                result?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for (key, child) in values {
                if key.len() > limits.max_string_bytes {
                    return Err(bounds_error(
                        segments,
                        format!(
                            "object key '{}' exceeds maximum string length {} bytes",
                            bounded(key),
                            limits.max_string_bytes
                        ),
                    ));
                }
                segments.push(PathSegment::Key(key));
                let result = enforce_bounds(child, segments, depth + 1, limits);
                let _ = segments.pop();
                result?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn bounds_error(segments: &[PathSegment<'_>], detail: impl AsRef<str>) -> StrictJsonError {
    StrictJsonError::BoundsViolation {
        message: format!("{} {}", render_path(segments), detail.as_ref()),
    }
}

fn render_path(segments: &[PathSegment<'_>]) -> String {
    let mut path = String::from("$");
    for segment in segments {
        match segment {
            PathSegment::Key(key) => {
                path.push('.');
                path.push_str(&bounded(key));
            }
            PathSegment::Index(index) => {
                path.push('[');
                path.push_str(&index.to_string());
                path.push(']');
            }
        }
    }
    path
}

/// An object key whose decoded type must be a string in every input format.
struct StrictObjectKey(String);

impl<'de> Deserialize<'de> for StrictObjectKey {
    /// Observe the decoded key type so YAML cannot coerce numbers into strings.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictObjectKeyVisitor)
    }
}

/// Accept string map keys while rejecting non-string YAML scalar/container keys.
struct StrictObjectKeyVisitor;

impl Visitor<'_> for StrictObjectKeyVisitor {
    type Value = StrictObjectKey;

    /// Describe the JSON-compatible mapping-key contract in decoder diagnostics.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string object key")
    }

    /// Retain a decoded borrowed string key without scalar coercion.
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictObjectKey(value.to_string()))
    }

    /// Retain a decoded owned string key without scalar coercion.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictObjectKey(value))
    }
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::String(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        StrictValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    /// Preserve string-keyed mappings and reject duplicates before inserting values.
    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((StrictObjectKey(key), value)) =
            object.next_entry::<StrictObjectKey, StrictValue>()?
        {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("{DUPLICATE_KEY_PREFIX}{}", bounded(&key))));
            }
            values.insert(key, value.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Limits, StrictYamlError, parse_value, parse_yaml_value, validate_lowercase_sha256,
    };
    use serde_json::json;

    const LIMITS: Limits = Limits { max_depth: 2, max_string_bytes: 3 };

    const YAML_LIMITS: Limits = Limits { max_depth: 16, max_string_bytes: 128 };

    /// Root and nested duplicate keys retain a YAML decoder error with source context.
    #[test]
    fn yaml_rejects_duplicate_keys_at_every_mapping_depth() {
        for content in [
            "a: 1\na: 2\n",
            "outer:\n  a: 1\n  a: 2\n",
            "name: &key a\nmapping: {*key : 1, a: 2}\n",
        ] {
            let error = parse_yaml_value(content, YAML_LIMITS).unwrap_err();
            let StrictYamlError::Decode { source } = error else {
                panic!("duplicate must retain the YAML decoder error");
            };
            assert!(source.to_string().contains("duplicate-key:a"));
            assert!(std::error::Error::source(&StrictYamlError::Decode { source }).is_some());
        }
    }

    /// YAML values retain JSON scalar types, quoted strings, and sequence order.
    #[test]
    fn yaml_preserves_scalar_types_and_core_scalar_tags() {
        let content = "null_value: null\nbool: true\nnegative: -7\npositive: 18446744073709551615\nfloat: 1.5\nquoted: 'true'\ntagged: !!str 12\nsequence: [false, null, 3, text]\n";
        assert_eq!(
            parse_yaml_value(content, YAML_LIMITS).unwrap(),
            json!({
                "null_value": null, "bool": true, "negative": -7,
                "positive": u64::MAX, "float": 1.5, "quoted": "true",
                "tagged": "12", "sequence": [false, null, 3, "text"]
            })
        );
    }

    /// Additional documents, including an empty successor, cannot be ignored.
    #[test]
    fn yaml_requires_one_complete_document() {
        for content in ["a: 1\n---\na: 2\n", "---\na: 1\n...\n---\n", "a: [1] trailing\n"] {
            assert!(matches!(
                parse_yaml_value(content, YAML_LIMITS),
                Err(StrictYamlError::Decode { .. })
            ));
        }
        assert_eq!(parse_yaml_value("---\na: 1\n...\n", YAML_LIMITS).unwrap(), json!({"a": 1}));
    }

    /// Non-string keys are rejected rather than coerced into string identifiers.
    #[test]
    fn yaml_rejects_non_string_keys_and_preserves_quoted_keys() {
        for content in [
            "1: value\n",
            "true: value\n",
            "null: value\n",
            "? [a, b]\n: value\n",
            "!!int 1: value\n",
        ] {
            assert!(matches!(
                parse_yaml_value(content, YAML_LIMITS),
                Err(StrictYamlError::Decode { .. })
            ));
        }
        assert_eq!(
            parse_yaml_value("'1': value\n'null': value\n", YAML_LIMITS).unwrap(),
            json!({"1": "value", "null": "value"})
        );
        assert_eq!(
            parse_value(br#"{"1":1,"null":null}"#, "test", YAML_LIMITS).unwrap(),
            json!({"1": 1, "null": null})
        );
    }

    /// Decoded depth and UTF-8 byte bounds retain the shared typed classification.
    #[test]
    fn yaml_reports_decoded_depth_and_string_bounds() {
        for content in ["a: {b: {c: null}}\n", "a: four\n", "a: éé\n", "long: 1\n"] {
            assert!(matches!(
                parse_yaml_value(content, LIMITS),
                Err(StrictYamlError::Bounds {
                    source: super::StrictJsonError::BoundsViolation { .. }
                })
            ));
        }
        assert_eq!(parse_yaml_value("a: {b: 1}\n", LIMITS).unwrap(), json!({"a": {"b": 1}}));
        assert_eq!(parse_yaml_value("a: é\n", LIMITS).unwrap(), json!({"a": "é"}));
    }

    /// Ordinary mapping, sequence, scalar, and string-key aliases expand faithfully.
    #[test]
    fn yaml_preserves_ordinary_alias_expansion() {
        let content = "original: &record {id: 7, flags: [true, false]}\ncopy: *record\nsequence: &items [1, two]\nsequence_copy: *items\nscalar: &name title\nscalar_copy: *name\nkey_map: {&key label: first}\nkey_copy: {*key : second}\n";
        assert_eq!(
            parse_yaml_value(content, YAML_LIMITS).unwrap(),
            json!({
                "original": {"id": 7, "flags": [true, false]},
                "copy": {"id": 7, "flags": [true, false]},
                "sequence": [1, "two"], "sequence_copy": [1, "two"],
                "scalar": "title", "scalar_copy": "title",
                "key_map": {"label": "first"}, "key_copy": {"label": "second"}
            })
        );
    }

    /// Expanded aliases remain subject to caller depth limits after decoding.
    #[test]
    fn yaml_aliases_are_checked_against_decoded_bounds() {
        let content = "a: &leaf {b: 1}\nx: {y: *leaf}\n";
        assert!(matches!(parse_yaml_value(content, LIMITS), Err(StrictYamlError::Bounds { .. })));
    }

    /// Decoder-erased URI-form tags limit the contract to the decoded JSON tree.
    ///
    /// This accepted example is a qualification of the existing dependency, not
    /// proof that unsupported YAML tags are rejected or preserved. Ordinary
    /// strings containing exclamation marks remain ordinary JSON strings.
    #[test]
    fn yaml_qualifies_erased_uri_tags_without_rejecting_ordinary_strings() {
        assert_eq!(
            parse_yaml_value("value: !<tag:example.invalid,2026:custom> 42\n", YAML_LIMITS,)
                .unwrap(),
            json!({"value": "42"})
        );
        assert_eq!(
            parse_yaml_value(
                "uri: https://example.invalid/!record\nprose: '!custom is quoted prose.'\n",
                YAML_LIMITS,
            )
            .unwrap(),
            json!({"uri": "https://example.invalid/!record", "prose": "!custom is quoted prose."})
        );
    }

    /// Malformed syntax, non-finite numbers, and local tagged values fail decoding.
    #[test]
    fn yaml_rejects_malformed_nonfinite_and_local_tagged_values() {
        for content in [
            "a: [1\n",
            "a: .nan\n",
            "a: .inf\n",
            "a: -.inf\n",
            "a: !!float .inf\n",
            "a: !custom value\n",
            "a: !custom [1]\n",
            "a: !custom {b: 1}\n",
            "a: *missing\n",
        ] {
            assert!(
                matches!(
                    parse_yaml_value(content, YAML_LIMITS),
                    Err(StrictYamlError::Decode { .. })
                ),
                "accepted {content:?}"
            );
        }
        assert_eq!(parse_yaml_value("a: '.inf'\n", YAML_LIMITS).unwrap(), json!({"a": ".inf"}));
    }

    #[test]
    fn classifies_duplicate_trailing_and_bound_violations_without_raw_keys() {
        assert!(matches!(
            parse_value(br#"{"a":1,"a":2}"#, "test", LIMITS),
            Err(super::StrictJsonError::DuplicateKey { .. })
        ));
        assert!(matches!(
            parse_value(b"{} {}", "test", LIMITS),
            Err(super::StrictJsonError::TrailingData { .. })
        ));
        assert!(matches!(
            parse_value(br#"{"a":"four"}"#, "test", LIMITS),
            Err(super::StrictJsonError::BoundsViolation { .. })
        ));
        assert!(matches!(
            parse_value(br#"{"a":{"b":{"c":null}}}"#, "test", LIMITS),
            Err(super::StrictJsonError::BoundsViolation { .. })
        ));
    }

    #[test]
    fn bounds_apply_to_object_keys_and_escape_diagnostic_paths() {
        let error = parse_value(br#"{"a\nverylong":1}"#, "test", LIMITS).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("maximum string length 3 bytes"));
        assert!(rendered.contains("a\\nverylong"));
        assert!(!rendered.contains("a\nverylong"));
    }

    #[test]
    fn lowercase_sha256_requires_exact_length_and_alphabet() {
        assert!(validate_lowercase_sha256("$.hash", &"0a".repeat(32)).is_ok());
        for invalid in ["0".repeat(63), "A".repeat(64), "g".repeat(64)] {
            assert_eq!(
                validate_lowercase_sha256("$.hash", &invalid).unwrap_err(),
                "$.hash must be 64 lowercase hexadecimal characters"
            );
        }
    }
}
