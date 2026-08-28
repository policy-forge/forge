//! Closed, bounded PRD 059 component and composition manifest contracts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::ForgeError;

pub const COMPONENT_SCHEMA_VERSION: &str = "forge.policy-component/1";
pub const COMPOSITION_SCHEMA_VERSION: &str = "forge.policy-composition/1";
pub const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
pub const MAX_PARAMETERS: usize = 256;
pub const MAX_INSTANCES: usize = 1_000;
pub const MAX_STRING_BYTES: usize = 16 * 1024;
const MAX_JSON_DEPTH: usize = 32;
const STRICT_JSON_LIMITS: crate::json_strict::Limits =
    crate::json_strict::Limits { max_depth: MAX_JSON_DEPTH, max_string_bytes: MAX_STRING_BYTES };
static SEMANTIC_VERSION_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(
        r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$",
    )
    .expect("static semantic-version grammar must compile")
});

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ComponentManifest {
    pub schema_version: String,
    pub component_key: String,
    pub version: String,
    pub title: String,
    pub owner: String,
    pub status: ComponentStatus,
    pub source: PathBuf,
    pub expected_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement_component_key: Option<String>,
    #[serde(default)]
    pub parameters: Vec<ParameterDeclaration>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentStatus {
    Draft,
    Approved,
    Deprecated,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParameterDeclaration {
    pub name: String,
    #[serde(rename = "type")]
    pub parameter_type: ParameterType,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<ParameterValue>,
    #[serde(default)]
    pub constraints: ParameterConstraints,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ParameterType {
    String,
    Integer,
    Boolean,
    StringList,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    StringList(Vec<String>),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ParameterConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regex: Option<String>,
    #[serde(default, rename = "enum", skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<ParameterValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompositionManifest {
    pub schema_version: String,
    pub project_root: PathBuf,
    pub policy_key: String,
    pub title: String,
    pub version: String,
    pub outputs: CompositionOutputs,
    pub components: Vec<ComponentInstance>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CompositionOutputs {
    pub markdown: PathBuf,
    pub lock: PathBuf,
    pub provenance: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ComponentInstance {
    pub instance_key: String,
    pub component_manifest: PathBuf,
    #[serde(default)]
    pub parameters: BTreeMap<String, ParameterValue>,
}

/// Parse and validate one duplicate-key-safe, closed component manifest.
///
/// # Errors
///
/// Returns [`ForgeError::PolicyComposition`] for invalid JSON, fields, bounds, or contracts.
pub fn parse_component(bytes: &[u8]) -> Result<ComponentManifest, ForgeError> {
    let manifest: ComponentManifest = parse_closed(bytes, "component manifest")?;
    validate_component(&manifest)?;
    Ok(manifest)
}

/// Parse and validate one duplicate-key-safe, closed composition manifest.
///
/// # Errors
///
/// Returns [`ForgeError::PolicyComposition`] for invalid JSON, fields, bounds, or contracts.
pub fn parse_composition(bytes: &[u8]) -> Result<CompositionManifest, ForgeError> {
    let manifest: CompositionManifest = parse_closed(bytes, "composition manifest")?;
    validate_composition(&manifest)?;
    Ok(manifest)
}

fn parse_closed<T>(bytes: &[u8], label: &str) -> Result<T, ForgeError>
where
    T: serde::de::DeserializeOwned,
{
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(error(format!("{label} exceeds the {MAX_MANIFEST_BYTES} byte limit")));
    }
    let value = crate::json_strict::parse_value(bytes, label, STRICT_JSON_LIMITS)
        .map_err(|source| error(source.to_string()))?;
    serde_json::from_value(value)
        .map_err(|source| error(format!("invalid {label} contract: {source}")))
}

fn validate_component(manifest: &ComponentManifest) -> Result<(), ForgeError> {
    schema_version(&manifest.schema_version, COMPONENT_SCHEMA_VERSION)?;
    key("$.component_key", &manifest.component_key)?;
    semantic_version("$.version", &manifest.version)?;
    non_empty("$.title", &manifest.title)?;
    key("$.owner", &manifest.owner)?;
    validate_local_path("$.source", &manifest.source, Some("md"))?;
    crate::json_strict::validate_lowercase_sha256("$.expected_sha256", &manifest.expected_sha256)
        .map_err(error)?;
    if let Some(replacement) = &manifest.replacement_component_key {
        if manifest.status != ComponentStatus::Deprecated {
            return Err(error(
                "$.replacement_component_key is only valid when $.status is deprecated",
            ));
        }
        key("$.replacement_component_key", replacement)?;
    }
    if manifest.parameters.len() > MAX_PARAMETERS {
        return Err(error(format!("$.parameters exceeds the {MAX_PARAMETERS} entry limit")));
    }
    let mut names = BTreeSet::new();
    for (index, declaration) in manifest.parameters.iter().enumerate() {
        let path = format!("$.parameters[{index}]");
        parameter_name(&format!("{path}.name"), &declaration.name)?;
        if !names.insert(&declaration.name) {
            return Err(error(format!(
                "{path}.name duplicates parameter '{}'",
                crate::json_strict::bounded(&declaration.name)
            )));
        }
        if is_sensitive_parameter_name(&declaration.name) {
            tracing::warn!(parameter = %declaration.name, "secret-like policy component parameter rejected");
            return Err(error(format!(
                "{path}.name '{}' matches a secret-like pattern; policy composition parameters must not contain secrets",
                crate::json_strict::bounded(&declaration.name)
            )));
        }
        validate_constraints(&path, declaration)?;
        if declaration.required && declaration.default.is_some() {
            return Err(error(format!("{path} cannot be both required and have a default")));
        }
        if let Some(default) = &declaration.default {
            validate_value(&format!("{path}.default"), declaration, default)?;
        }
    }
    Ok(())
}

fn validate_composition(manifest: &CompositionManifest) -> Result<(), ForgeError> {
    schema_version(&manifest.schema_version, COMPOSITION_SCHEMA_VERSION)?;
    validate_project_root(&manifest.project_root)?;
    key("$.policy_key", &manifest.policy_key)?;
    non_empty("$.title", &manifest.title)?;
    if manifest.title.contains(['\r', '\n']) {
        return Err(error("$.title must be one line"));
    }
    non_empty("$.version", &manifest.version)?;
    for (path, output) in [
        ("$.outputs.markdown", &manifest.outputs.markdown),
        ("$.outputs.lock", &manifest.outputs.lock),
        ("$.outputs.provenance", &manifest.outputs.provenance),
    ] {
        validate_local_path(path, output, None)?;
    }
    let mut output_keys = BTreeSet::new();
    for output in [&manifest.outputs.markdown, &manifest.outputs.lock, &manifest.outputs.provenance]
    {
        if !output_keys.insert(normalized_path(output)?) {
            return Err(error("composition output paths must be distinct"));
        }
    }
    if manifest.components.is_empty() {
        return Err(error("$.components must contain at least one component instance"));
    }
    if manifest.components.len() > MAX_INSTANCES {
        return Err(error(format!("$.components exceeds the {MAX_INSTANCES} entry limit")));
    }
    let mut instances = BTreeSet::new();
    for (index, instance) in manifest.components.iter().enumerate() {
        let path = format!("$.components[{index}]");
        key(&format!("{path}.instance_key"), &instance.instance_key)?;
        if !instances.insert(&instance.instance_key) {
            return Err(error(format!(
                "{path}.instance_key duplicates instance key '{}'",
                crate::json_strict::bounded(&instance.instance_key)
            )));
        }
        validate_local_path(
            &format!("{path}.component_manifest"),
            &instance.component_manifest,
            Some("json"),
        )?;
        for name in instance.parameters.keys() {
            parameter_name(&format!("{path}.parameters.{name}"), name)?;
            if is_sensitive_parameter_name(name) {
                tracing::warn!(parameter = %name, "secret-like policy component parameter rejected");
                return Err(error(format!(
                    "{path}.parameters.{} matches a secret-like pattern; parameters must not contain secrets",
                    crate::json_strict::bounded(name)
                )));
            }
        }
    }
    Ok(())
}

fn validate_constraints(path: &str, declaration: &ParameterDeclaration) -> Result<(), ForgeError> {
    let constraints = &declaration.constraints;
    if constraints.min_length.zip(constraints.max_length).is_some_and(|(min, max)| min > max) {
        return Err(error(format!("{path}.constraints min_length exceeds max_length")));
    }
    if constraints.minimum.zip(constraints.maximum).is_some_and(|(min, max)| min > max) {
        return Err(error(format!("{path}.constraints minimum exceeds maximum")));
    }
    if constraints.min_items.zip(constraints.max_items).is_some_and(|(min, max)| min > max) {
        return Err(error(format!("{path}.constraints min_items exceeds max_items")));
    }
    match declaration.parameter_type {
        ParameterType::String => {
            if constraints.minimum.is_some()
                || constraints.maximum.is_some()
                || constraints.min_items.is_some()
                || constraints.max_items.is_some()
            {
                return Err(error(format!(
                    "{path}.constraints contains fields invalid for string"
                )));
            }
        }
        ParameterType::Integer => {
            if constraints.min_length.is_some()
                || constraints.max_length.is_some()
                || constraints.min_items.is_some()
                || constraints.max_items.is_some()
                || constraints.regex.is_some()
            {
                return Err(error(format!(
                    "{path}.constraints contains fields invalid for integer"
                )));
            }
        }
        ParameterType::Boolean => {
            if *constraints
                != (ParameterConstraints {
                    allowed_values: constraints.allowed_values.clone(),
                    ..ParameterConstraints::default()
                })
            {
                return Err(error(format!(
                    "{path}.constraints contains fields invalid for boolean"
                )));
            }
        }
        ParameterType::StringList => {
            if constraints.minimum.is_some() || constraints.maximum.is_some() {
                return Err(error(format!(
                    "{path}.constraints contains fields invalid for string-list"
                )));
            }
        }
    }
    if let Some(pattern) = &constraints.regex {
        if pattern.len() > 1024 {
            return Err(error(format!("{path}.constraints.regex exceeds 1024 bytes")));
        }
        regex::Regex::new(pattern)
            .map_err(|source| error(format!("{path}.constraints.regex is invalid: {source}")))?;
    }
    for (index, allowed) in constraints.allowed_values.iter().enumerate() {
        validate_value_type(
            &format!("{path}.constraints.enum[{index}]"),
            declaration.parameter_type,
            allowed,
        )?;
    }
    Ok(())
}

/// Validate one typed parameter value against its declaration and constraints.
///
/// # Errors
///
/// Returns [`ForgeError::PolicyComposition`] for type, bound, regex, or enum violations.
pub fn validate_value(
    path: &str,
    declaration: &ParameterDeclaration,
    value: &ParameterValue,
) -> Result<(), ForgeError> {
    validate_value_type(path, declaration.parameter_type, value)?;
    let constraints = &declaration.constraints;
    let declared_regex = constraints
        .regex
        .as_deref()
        .map(regex::Regex::new)
        .transpose()
        .map_err(|source| error(source.to_string()))?;
    match value {
        ParameterValue::String(value) => {
            validate_text_value(path, value)?;
            let length = value.chars().count();
            range(path, "length", length, constraints.min_length, constraints.max_length)?;
            if let Some(pattern) = &declared_regex
                && !pattern.is_match(value)
            {
                return Err(error(format!("{path} does not match the declared regex")));
            }
        }
        ParameterValue::Integer(value) => {
            if constraints.minimum.is_some_and(|minimum| *value < minimum)
                || constraints.maximum.is_some_and(|maximum| *value > maximum)
            {
                return Err(error(format!("{path} is outside the declared integer range")));
            }
        }
        ParameterValue::Boolean(_) => {}
        ParameterValue::StringList(values) => {
            range(path, "item count", values.len(), constraints.min_items, constraints.max_items)?;
            for (index, value) in values.iter().enumerate() {
                validate_text_value(&format!("{path}[{index}]"), value)?;
                range(
                    &format!("{path}[{index}]"),
                    "length",
                    value.chars().count(),
                    constraints.min_length,
                    constraints.max_length,
                )?;
                if let Some(pattern) = &declared_regex
                    && !pattern.is_match(value)
                {
                    return Err(error(format!(
                        "{path}[{index}] does not match the declared regex"
                    )));
                }
            }
        }
    }
    if !constraints.allowed_values.is_empty() && !constraints.allowed_values.contains(value) {
        return Err(error(format!("{path} is not one of the declared enum values")));
    }
    Ok(())
}

fn validate_value_type(
    path: &str,
    expected: ParameterType,
    value: &ParameterValue,
) -> Result<(), ForgeError> {
    let matches = matches!(
        (expected, value),
        (ParameterType::String, ParameterValue::String(_))
            | (ParameterType::Integer, ParameterValue::Integer(_))
            | (ParameterType::Boolean, ParameterValue::Boolean(_))
            | (ParameterType::StringList, ParameterValue::StringList(_))
    );
    if matches { Ok(()) } else { Err(error(format!("{path} has the wrong parameter type"))) }
}

fn validate_text_value(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.len() > MAX_STRING_BYTES {
        return Err(error(format!("{path} exceeds {MAX_STRING_BYTES} bytes")));
    }
    if value.contains(['\n', '\r']) || value.chars().any(char::is_control) {
        return Err(error(format!("{path} must be single-line text without control characters")));
    }
    Ok(())
}

fn range(
    path: &str,
    label: &str,
    actual: usize,
    minimum: Option<usize>,
    maximum: Option<usize>,
) -> Result<(), ForgeError> {
    if minimum.is_some_and(|value| actual < value) || maximum.is_some_and(|value| actual > value) {
        Err(error(format!("{path} {label} is outside the declared range")))
    } else {
        Ok(())
    }
}

fn schema_version(actual: &str, expected: &str) -> Result<(), ForgeError> {
    if actual == expected {
        Ok(())
    } else {
        Err(error(format!(
            "unsupported schema_version '{}'; expected {expected}",
            crate::json_strict::bounded(actual)
        )))
    }
}

fn non_empty(path: &str, value: &str) -> Result<(), ForgeError> {
    if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        Err(error(format!("{path} must be non-empty, bounded text without control characters")))
    } else {
        Ok(())
    }
}

fn semantic_version(path: &str, value: &str) -> Result<(), ForgeError> {
    non_empty(path, value)?;
    if SEMANTIC_VERSION_PATTERN.is_match(value) {
        Ok(())
    } else {
        Err(error(format!("{path} must be a semantic version such as 1.2.3")))
    }
}

fn key(path: &str, value: &str) -> Result<(), ForgeError> {
    if valid_kebab_key(value) {
        Ok(())
    } else {
        Err(error(format!("{path} must be ASCII kebab-case")))
    }
}

fn parameter_name(path: &str, value: &str) -> Result<(), ForgeError> {
    key(path, value)
}

fn valid_kebab_key(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
}

#[must_use]
pub fn is_sensitive_parameter_name(name: &str) -> bool {
    let normalized = name.replace('_', "-").to_ascii_lowercase();
    ["password", "passphrase", "secret", "token", "api-key", "private-key", "credential"]
        .iter()
        .any(|pattern| {
            normalized == *pattern
                || normalized.contains(&format!("-{pattern}"))
                || normalized.contains(&format!("{pattern}-"))
        })
}

fn validate_project_root(path: &Path) -> Result<(), ForgeError> {
    if path == Path::new(".") { Ok(()) } else { validate_local_path("$.project_root", path, None) }
}

/// Validate the platform-independent spelling of a contained relative path.
///
/// # Errors
///
/// Returns [`ForgeError::PolicyComposition`] for empty, absolute, traversal, drive, backslash,
/// alternate stream, invalid UTF-8, or wrong-extension spellings.
pub fn validate_local_path(
    path: &str,
    value: &Path,
    extension: Option<&str>,
) -> Result<(), ForgeError> {
    if value.as_os_str().is_empty() {
        return Err(error(format!("{path} must not be empty")));
    }
    let spelling = value.to_str().ok_or_else(|| error(format!("{path} must be valid UTF-8")))?;
    let bytes = spelling.as_bytes();
    let windows_drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let unsafe_component = value.components().any(|component| {
        matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
    }) || spelling
        .split(['/', '\\'])
        .any(|component| component == ".." || component.is_empty());
    if value.is_absolute()
        || windows_drive
        || spelling.starts_with(['/', '\\'])
        || unsafe_component
        || spelling.contains('\\')
        || spelling.contains(':')
    {
        return Err(error(format!(
            "{path} must be a contained relative local path without parent, root, drive, backslash, or alternate-stream components"
        )));
    }
    if let Some(extension) = extension
        && value.extension().and_then(|part| part.to_str()) != Some(extension)
    {
        return Err(error(format!("{path} must use the .{extension} extension")));
    }
    Ok(())
}

fn normalized_path(path: &Path) -> Result<String, ForgeError> {
    path.to_str()
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| error(format!("path '{}' is not valid UTF-8", path.display())))
}

pub(crate) fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::PolicyComposition(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_component() -> serde_json::Value {
        json!({
            "schema_version": COMPONENT_SCHEMA_VERSION,
            "component_key": "access-review",
            "version": "1.0.0",
            "title": "Access review",
            "owner": "security",
            "status": "approved",
            "source": "access-review.md",
            "expected_sha256": "0".repeat(64),
            "parameters": []
        })
    }

    #[test]
    fn component_contract_rejects_duplicate_unknown_and_unsupported_fields() {
        let duplicate = br#"{"schema_version":"forge.policy-component/1","schema_version":"forge.policy-component/1"}"#;
        assert!(
            parse_component(duplicate).unwrap_err().to_string().contains("duplicate object key")
        );

        let mut unknown = valid_component();
        unknown["include"] = json!("nested.json");
        assert!(
            parse_component(&serde_json::to_vec(&unknown).unwrap())
                .unwrap_err()
                .to_string()
                .contains("unknown field")
        );

        let mut unsupported = valid_component();
        unsupported["schema_version"] = json!("forge.policy-component/2");
        assert!(
            parse_component(&serde_json::to_vec(&unsupported).unwrap())
                .unwrap_err()
                .to_string()
                .contains("unsupported schema_version")
        );
    }

    #[test]
    fn secret_like_and_duplicate_parameter_names_are_rejected() {
        let mut manifest = valid_component();
        manifest["parameters"] = json!([
            {"name":"owner-role","type":"string"},
            {"name":"owner-role","type":"string"}
        ]);
        assert!(
            parse_component(&serde_json::to_vec(&manifest).unwrap())
                .unwrap_err()
                .to_string()
                .contains("duplicates parameter")
        );

        manifest["parameters"] = json!([{"name":"api-token","type":"string"}]);
        assert!(
            parse_component(&serde_json::to_vec(&manifest).unwrap())
                .unwrap_err()
                .to_string()
                .contains("secret-like")
        );
    }

    #[test]
    fn path_spelling_rejects_cross_platform_escape_forms() {
        for value in [
            "../outside.md",
            r"..\outside.md",
            r"components\clause.md",
            "/tmp/x.md",
            r"C:\x.md",
            r"\\server\x.md",
        ] {
            assert!(
                validate_local_path("$.source", Path::new(value), Some("md")).is_err(),
                "accepted {value}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_spelling_rejects_invalid_utf8() {
        use std::os::unix::ffi::OsStrExt;

        let value = Path::new(std::ffi::OsStr::from_bytes(b"component-\xff.md"));
        assert!(
            validate_local_path("$.source", value, Some("md"))
                .unwrap_err()
                .to_string()
                .contains("valid UTF-8")
        );
    }

    #[test]
    fn component_version_is_semantic() {
        let mut manifest = valid_component();
        for invalid in ["version-one", "1.0.0-01", "1.0.0-alpha.01"] {
            manifest["version"] = json!(invalid);
            assert!(
                parse_component(&serde_json::to_vec(&manifest).unwrap())
                    .unwrap_err()
                    .to_string()
                    .contains("semantic version"),
                "accepted invalid semantic version {invalid}"
            );
        }
        for valid in ["1.0.0-0", "1.0.0-alpha.1", "1.0.0-alpha-01", "1.0.0+01"] {
            manifest["version"] = json!(valid);
            assert!(
                parse_component(&serde_json::to_vec(&manifest).unwrap()).is_ok(),
                "rejected valid semantic version {valid}"
            );
        }
    }
}
