//! Shared duplicate-key-safe, bounded JSON parsing utilities.
//!
//! Limits apply to the decoded tree. Callers MUST still cap raw input bytes
//! before parsing because wide, shallow JSON may allocate before its structural
//! bounds can be inspected; `serde_json`'s own recursion limit can also reject a
//! document before `Limits::max_depth` when configured higher.

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

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = object.next_entry::<String, StrictValue>()? {
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
    use super::{Limits, parse_value, validate_lowercase_sha256};

    const LIMITS: Limits = Limits { max_depth: 2, max_string_bytes: 3 };

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
