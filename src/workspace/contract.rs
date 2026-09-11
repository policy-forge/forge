//! Runtime validation against the single committed API contract.

use std::sync::LazyLock;

use serde::Serialize;
use serde_json::{Value, json};

/// Safe transport-neutral error. Never retain a parser or filesystem error string.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct Error {
    pub code: &'static str,
    pub message: &'static str,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
}

impl Error {
    pub(crate) const fn invalid() -> Self {
        Self::new("invalid-request", "The request does not match the supported contract.", false)
    }

    pub(crate) const fn containment() -> Self {
        Self::new(
            "resource-containment",
            "The resource cannot be accessed safely within this project.",
            false,
        )
    }

    pub(crate) const fn new(code: &'static str, message: &'static str, retryable: bool) -> Self {
        Self { code, message, retryable, resource_version: None }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for Error {}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) static DOCUMENT: LazyLock<Value> = LazyLock::new(|| {
    serde_yaml::from_str(include_str!("../../docs/api/forge-workspace-v1.openapi.yaml"))
        .expect("embedded OpenAPI is validated by the contract suite")
});

fn rewrite(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let value = if key == "$ref" {
                        value.as_str().map_or_else(
                            || value.clone(),
                            |reference| {
                                Value::String(
                                    reference.replace("#/components/schemas/", "#/$defs/"),
                                )
                            },
                        )
                    } else {
                        rewrite(value)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(rewrite).collect()),
        _ => value.clone(),
    }
}

static VALIDATORS: LazyLock<std::collections::BTreeMap<String, jsonschema::Validator>> =
    LazyLock::new(|| {
        let definitions = rewrite(&DOCUMENT["components"]["schemas"]);
        definitions
            .as_object()
            .expect("embedded schema definitions")
            .keys()
            .map(|name| {
                let schema = json!({"$ref": format!("#/$defs/{name}"), "$defs": definitions});
                let validator = jsonschema::options()
                    .with_draft(jsonschema::Draft::Draft202012)
                    .build(&schema)
                    .expect("embedded component schema is validated by the contract suite");
                (name.clone(), validator)
            })
            .collect()
    });

pub(crate) const VERSION: &str = "1.1.0";

pub(crate) fn validate(name: &str, value: &Value) -> Result<()> {
    if VALIDATORS.get(name).is_some_and(|validator| validator.is_valid(value)) {
        Ok(())
    } else {
        Err(Error::invalid())
    }
}

/// Raw size is checked before the shared duplicate-key-safe decoder allocates.
pub(crate) fn parse(bytes: &[u8], max_bytes: usize, max_string_bytes: usize) -> Result<Value> {
    if bytes.len() > max_bytes {
        return Err(Error::new(
            "payload-too-large",
            "The input exceeds the supported size limit.",
            false,
        ));
    }
    // A raw lexical work bound precedes tree allocation. Strings are skipped
    // without decoding; the strict parser below still validates every token.
    let mut quoted = false;
    let mut escaped = false;
    let mut separators = 0_usize;
    for byte in bytes {
        if quoted {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                quoted = false;
            }
        } else if *byte == b'"' {
            quoted = true;
        } else if matches!(byte, b'[' | b'{' | b',' | b':') {
            separators += 1;
            if separators > 100_000 {
                return Err(Error::invalid());
            }
        }
    }
    crate::json_strict::parse_value(
        bytes,
        "workspace input",
        crate::json_strict::Limits { max_depth: 64, max_string_bytes },
    )
    .map_err(|_| Error::invalid())
}

/// Match and validate transport parameters against the normative operation.
/// There is no independently maintained query/header/path schema in the server.
pub(crate) fn operation_request(
    method: &str,
    path: &str,
    query: &[(String, String)],
    idempotency: Option<&str>,
    body: Option<&Value>,
) -> Result<String> {
    let paths = DOCUMENT["paths"].as_object().ok_or_else(Error::invalid)?;
    let actual: Vec<_> = path.split('/').collect();
    for (template, item) in paths {
        let segments: Vec<_> = template.split('/').collect();
        if segments.len() != actual.len() {
            continue;
        }
        let mut path_values = std::collections::BTreeMap::new();
        let matched = segments.iter().zip(&actual).all(|(pattern, value)| {
            if let Some(key) = pattern.strip_prefix('{').and_then(|key| key.strip_suffix('}')) {
                path_values.insert(key, *value);
                !value.is_empty()
            } else {
                pattern == value
            }
        });
        if !matched {
            continue;
        }
        let Some(operation) = item.get(method.to_ascii_lowercase()) else {
            continue;
        };
        let mut allowed_queries = std::collections::BTreeSet::new();
        if let Some(parameters) = operation["parameters"].as_array() {
            for parameter in parameters {
                let parameter = parameter
                    .get("$ref")
                    .and_then(Value::as_str)
                    .and_then(|reference| reference.strip_prefix('#'))
                    .and_then(|pointer| DOCUMENT.pointer(pointer))
                    .unwrap_or(parameter);
                let name = parameter["name"].as_str().ok_or_else(Error::invalid)?;
                let location = parameter["in"].as_str().ok_or_else(Error::invalid)?;
                let value = match location {
                    "path" => path_values.get(name).copied(),
                    "header" if name == "Idempotency-Key" => idempotency,
                    "query" => {
                        allowed_queries.insert(name);
                        let mut values = query.iter().filter(|(key, _)| key == name);
                        let value = values.next().map(|(_, value)| value.as_str());
                        if values.next().is_some() {
                            return Err(Error::invalid());
                        }
                        value
                    }
                    _ => return Err(Error::invalid()),
                };
                if let Some(value) = value {
                    let schema = rewrite(&parameter["schema"]);
                    let value = if schema["type"] == "integer" {
                        json!(value.parse::<u64>().map_err(|_| Error::invalid())?)
                    } else if schema["type"] == "boolean" {
                        match value {
                            "true" => json!(true),
                            "false" => json!(false),
                            _ => return Err(Error::invalid()),
                        }
                    } else {
                        json!(value)
                    };
                    let full = json!({"allOf":[schema],"$defs":rewrite(&DOCUMENT["components"]["schemas"])});
                    let validator = jsonschema::options()
                        .with_draft(jsonschema::Draft::Draft202012)
                        .build(&full)
                        .map_err(|_| Error::invalid())?;
                    if !validator.is_valid(&value) {
                        return Err(Error::invalid());
                    }
                } else if parameter["required"] == true {
                    return Err(Error::invalid());
                }
            }
        }
        if query.iter().any(|(key, _)| !allowed_queries.contains(key.as_str())) {
            return Err(Error::invalid());
        }
        if let Some(request) = operation.get("requestBody") {
            let value = body.ok_or_else(Error::invalid)?;
            let name = request["content"]["application/json"]["schema"]["$ref"]
                .as_str()
                .and_then(|reference| reference.strip_prefix("#/components/schemas/"))
                .ok_or_else(Error::invalid)?;
            validate(name, value)?;
        } else if body.is_some_and(|value| value != &json!({})) {
            return Err(Error::invalid());
        }
        return operation["operationId"].as_str().map(str::to_owned).ok_or_else(Error::invalid);
    }
    Err(Error::new("not-found", "The requested API operation was not found.", false))
}

/// Serialize with a write-time allocation limit, including pretty JSON.
pub(crate) fn encode(value: &impl serde::Serialize, limit: usize, pretty: bool) -> Result<Vec<u8>> {
    struct Bounded {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len().saturating_add(bytes.len()) > self.limit {
                return Err(std::io::Error::other("bounded output"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = Bounded { bytes: Vec::new(), limit };
    if pretty {
        serde_json::to_writer_pretty(&mut output, value)
    } else {
        serde_json::to_writer(&mut output, value)
    }
    .map_err(|_| {
        Error::new("payload-too-large", "The prepared output exceeds its size limit.", false)
    })?;
    Ok(output.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicates_and_forward_closed_requests() {
        assert!(parse(br#"{"scope":"all","\u0073cope":"selected"}"#, 1024, 1024).is_err());
        assert!(validate("ValidationRunRequest", &json!({"scope":"all", "future":true})).is_err());
        assert!(validate("ValidationRunRequest", &json!({"scope":"all"})).is_ok());
        assert!(parse(b"{}", 1, 100).is_err());
    }

    #[test]
    fn diagnostics_never_reflect_untrusted_values() {
        let failure = parse(br#"{"private-secret":"\uZZZZ"}"#, 1024, 1024).unwrap_err();
        let rendered = serde_json::to_string(&failure).unwrap();
        assert!(!rendered.contains("private-secret"));
        validate("Error", &serde_json::to_value(failure).unwrap()).unwrap();
    }
}
