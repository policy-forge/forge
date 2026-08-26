//! Shared duplicate-key-safe, bounded JSON parsing utilities.

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};

/// Structural limits applied after duplicate-key-safe JSON decoding.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) max_depth: usize,
    pub(crate) max_string_bytes: usize,
}

/// Parse one complete JSON value without duplicate object keys and enforce structural bounds.
pub(crate) fn parse_value(bytes: &[u8], label: &str, limits: Limits) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let strict = StrictValue::deserialize(&mut deserializer)
        .map_err(|cause| format!("invalid {label} JSON: {cause}"))?;
    deserializer.end().map_err(|cause| format!("invalid trailing {label} data: {cause}"))?;
    enforce_bounds(&strict.0, "$", 0, limits)?;
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

fn enforce_bounds(value: &Value, path: &str, depth: usize, limits: Limits) -> Result<(), String> {
    if depth > limits.max_depth {
        return Err(format!("{path} exceeds maximum JSON depth {}", limits.max_depth));
    }
    match value {
        Value::String(text) if text.len() > limits.max_string_bytes => {
            Err(format!("{path} exceeds maximum string length {} bytes", limits.max_string_bytes))
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                enforce_bounds(child, &format!("{path}[{index}]"), depth + 1, limits)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            for (key, child) in values {
                enforce_bounds(child, &format!("{path}.{key}"), depth + 1, limits)?;
            }
            Ok(())
        }
        _ => Ok(()),
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

    fn visit_map<A>(self, mut object: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, value)) = object.next_entry::<String, StrictValue>()? {
            if values.insert(key.clone(), value.0).is_some() {
                return Err(de::Error::custom(format!("duplicate object key '{key}'")));
            }
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Limits, parse_value, validate_lowercase_sha256};

    const LIMITS: Limits = Limits { max_depth: 2, max_string_bytes: 3 };

    #[test]
    fn rejects_duplicate_keys_trailing_data_and_structural_bound_violations() {
        assert!(
            parse_value(br#"{"a":1,"a":2}"#, "test", LIMITS)
                .unwrap_err()
                .contains("duplicate object key")
        );
        assert!(
            parse_value(b"{} {}", "test", LIMITS)
                .unwrap_err()
                .contains("invalid trailing test data")
        );
        assert!(
            parse_value(br#"{"a":"four"}"#, "test", LIMITS)
                .unwrap_err()
                .contains("maximum string length 3 bytes")
        );
        assert!(
            parse_value(br#"{"a":{"b":{"c":null}}}"#, "test", LIMITS)
                .unwrap_err()
                .contains("maximum JSON depth 2")
        );
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
