//! PRD-062 Slice 0 contract validation harness.
//!
//! Deterministic, offline validation of the normative `/api/v1` `OpenAPI` 3.1
//! contract, the `forge.workspace/1` resource-index schema, the representative
//! fixture suite, and the browser capability matrix. These checks are the
//! contract/schema/fixture drift gate that must pass before browser UI
//! implementation begins (PRD-062 "Contract-First Artifacts" and M-21).
//!
//! Ownership and phasing are recorded in `docs/adr/0001-openapi-contract-ownership.md`:
//! the committed `OpenAPI` document is the single normative artifact; fixtures and
//! the capability matrix are validated against it live, so any drift between
//! them fails `cargo test` on every supported platform with no network access
//! and no external toolchain.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use serde_json::{Map, Number, Value, json};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

const OPENAPI_REL: &str = "docs/api/forge-workspace-v1.openapi.yaml";
const META_SCHEMA_REL: &str = "schemas/openapi-3.1-schema-2022-10-07.json";
const META_SCHEMA_SHA256: &str = "da01ba28852cac0de53893797cb8d1942bc3b05084f526dcc216717dec314ed0";
const META_SCHEMA_ID: &str = "https://spec.openapis.org/oas/3.1/schema/2022-10-07";
const WORKSPACE_SCHEMA_REL: &str = "schemas/forge.workspace-1.schema.json";
const WORKSPACE_SCHEMA_VERSION: &str = "forge.workspace/1";
const JSON_SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
const FIXTURES_REL: &str = "docs/api/fixtures";
const FIXTURE_INDEX_REL: &str = "docs/api/fixtures/index.json";
const FIXTURE_FORMAT: &str = "forge.api-fixtures/1";
const MATRIX_JSON_REL: &str = "docs/api/capability-matrix.json";
const MATRIX_MD_REL: &str = "docs/api/capability-matrix.md";
const UNLOCK_PATH: &str = "/api/v1/session/unlock";

const HTTP_METHODS: [&str; 8] =
    ["get", "put", "post", "delete", "options", "head", "patch", "trace"];
const REQUIRED_FIXTURE_KINDS: [&str; 7] =
    ["request", "response", "workspace", "error", "pagination", "version-conflict", "operation"];
const ALLOWED_AUTHORIZATIONS: [&str; 5] =
    ["browser-write", "browser-read", "machine", "unlock", "none"];
const GOLDEN_PATH_STEPS: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

/// Minimal structural summary of one `OpenAPI` operation.
#[derive(Clone, Debug)]
struct OperationRef {
    method: &'static str,
    path: String,
    id: String,
    unauthenticated: bool,
}

fn load_yaml_as_json(path: &Path) -> Value {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    // Parse through the duplicate-key-rejecting visitor: deserializing
    // directly into serde_json::Value would silently keep the last value for
    // duplicated YAML mapping keys.
    let deserializer = serde_yaml::Deserializer::from_str(&text);
    StrictValue::deserialize(deserializer).map_or_else(
        |error| panic!("parse {} as strict YAML: {error}", path.display()),
        |value| value.0,
    )
}

fn load_openapi() -> Value {
    load_yaml_as_json(&repo_path(OPENAPI_REL))
}

/// All operations in the document as `(method, path, operation)` tuples.
fn operations(document: &Value) -> Vec<OperationRef> {
    let mut found = Vec::new();
    let Some(paths) = document.get("paths").and_then(Value::as_object) else {
        return found;
    };
    for (path, path_item) in paths {
        let Some(items) = path_item.as_object() else {
            continue;
        };
        for method in HTTP_METHODS {
            let Some(operation) = items.get(method).and_then(Value::as_object) else {
                continue;
            };
            found.push(OperationRef {
                method,
                path: path.clone(),
                id: operation
                    .get("operationId")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                unauthenticated: matches!(
                    operation.get("security"),
                    Some(Value::Array(entries)) if entries.is_empty()
                ),
            });
        }
    }
    found
}

fn unlock_operation_id(document: &Value) -> String {
    operations(document)
        .iter()
        .find(|operation| operation.path == UNLOCK_PATH && operation.method == "post")
        .map(|operation| operation.id.clone())
        .unwrap_or_default()
}

/// True when any security requirement object is `{}`, which `OpenAPI` treats as
/// "authentication optional" rather than "authenticated".
fn has_empty_security_requirement(requirements: &[Value]) -> bool {
    requirements.iter().any(|requirement| requirement.as_object().is_some_and(Map::is_empty))
}

/// Rewrite `#/components/schemas/<Name>` references to `#/$defs/<Name>` so one
/// component schema can be compiled standalone together with its siblings.
fn rewrite_component_refs(value: &Value) -> Value {
    match value {
        Value::Object(entries) => {
            let rewritten: Map<String, Value> = entries
                .iter()
                .map(|(key, child)| {
                    if key == "$ref" {
                        if let Some(target) = child.as_str() {
                            if let Some(name) = target.strip_prefix("#/components/schemas/") {
                                return (key.clone(), Value::String(format!("#/$defs/{name}")));
                            }
                        }
                    }
                    (key.clone(), rewrite_component_refs(child))
                })
                .collect();
            Value::Object(rewritten)
        }
        Value::Array(items) => Value::Array(items.iter().map(rewrite_component_refs).collect()),
        other => other.clone(),
    }
}

/// Compile one `components/schemas` entry as a standalone 2020-12 schema.
fn component_validator(document: &Value, name: &str) -> jsonschema::Validator {
    let schemas = document
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{OPENAPI_REL} has no components/schemas object"));
    assert!(
        schemas.contains_key(name),
        "{OPENAPI_REL} references unknown components/schemas/{name}"
    );
    let wrapper = serde_json::json!({
        "$schema": JSON_SCHEMA_DIALECT,
        "$ref": format!("#/$defs/{name}"),
        "$defs": rewrite_component_refs(document.pointer("/components/schemas").unwrap_or(&Value::Null)),
    });
    jsonschema::validator_for(&wrapper).unwrap_or_else(|error| {
        panic!("components/schemas/{name} is not valid JSON Schema: {error}")
    })
}

fn workspace_schema() -> Value {
    let path = repo_path(WORKSPACE_SCHEMA_REL);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    read_json_strict(&bytes, WORKSPACE_SCHEMA_REL)
        .unwrap_or_else(|error| panic!("{WORKSPACE_SCHEMA_REL}: {error}"))
}

fn collect_files(dir: &Path, prefix: &Path, found: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let entry = entry.expect("directory entry");
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, prefix, found);
        } else {
            let relative = path
                .strip_prefix(prefix)
                .expect("path below fixtures root")
                .to_string_lossy()
                .replace('\\', "/");
            found.push(relative);
        }
    }
}

// ---------------------------------------------------------------------------
// Strict JSON parsing (duplicate object keys are a validation failure)
// ---------------------------------------------------------------------------

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
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
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
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
        D: serde::Deserializer<'de>,
    {
        StrictValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
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
                return Err(serde::de::Error::custom(format!(
                    "duplicate object key '{}'",
                    key.chars().take(120).collect::<String>()
                )));
            }
            values.insert(key, value.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

fn read_json_strict(bytes: &[u8], label: &str) -> Result<Value, String> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValue::deserialize(&mut deserializer)
        .map_err(|error| format!("{label}: invalid JSON: {error}"))?;
    deserializer.end().map_err(|error| format!("{label}: trailing data: {error}"))?;
    Ok(value.0)
}

// ---------------------------------------------------------------------------
// Vendored OpenAPI 3.1 meta-schema
// ---------------------------------------------------------------------------

#[test]
fn vendored_openapi_meta_schema_is_pinned_and_intact() {
    let bytes = fs::read(repo_path(META_SCHEMA_REL)).expect("vendored meta-schema exists");
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(hex, "{byte:02x}");
    }
    assert_eq!(
        hex, META_SCHEMA_SHA256,
        "{META_SCHEMA_REL} changed; update the pinned digest deliberately and record why"
    );
    let schema = read_json_strict(&bytes, META_SCHEMA_REL).expect("meta-schema parses strictly");
    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some(META_SCHEMA_ID),
        "vendored meta-schema must self-identify as the official 3.1 schema"
    );
    jsonschema::validator_for(&schema).expect("meta-schema compiles offline");
}

// ---------------------------------------------------------------------------
// Normative OpenAPI document
// ---------------------------------------------------------------------------

#[test]
fn openapi_document_is_valid_openapi_3_1() {
    let document = load_openapi();
    assert!(
        document
            .get("openapi")
            .and_then(Value::as_str)
            .is_some_and(|version| version.starts_with("3.1.")),
        "contract must declare OpenAPI 3.1.x"
    );
    assert_eq!(
        document.get("jsonSchemaDialect").and_then(Value::as_str),
        Some(JSON_SCHEMA_DIALECT),
        "contract must pin the draft 2020-12 JSON Schema dialect"
    );
    for field in ["title", "version"] {
        assert!(
            document
                .pointer(&format!("/info/{field}"))
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty()),
            "info.{field} must be a non-empty string"
        );
    }
    let meta_bytes = fs::read(repo_path(META_SCHEMA_REL)).expect("vendored meta-schema exists");
    let meta = read_json_strict(&meta_bytes, META_SCHEMA_REL).expect("meta-schema parses");
    let validator = jsonschema::validator_for(&meta).expect("meta-schema compiles");
    validator
        .validate(&document)
        .expect("normative document validates against the official OpenAPI 3.1 schema");
}

#[test]
#[allow(clippy::too_many_lines)] // Keeps the complete namespace/security/parameter gate auditable in one place.
fn openapi_paths_operations_and_security_invariants_hold() {
    let document = load_openapi();
    let paths = document.get("paths").and_then(Value::as_object).expect("document has paths");
    let mut errors: Vec<String> = Vec::new();

    for path in paths.keys() {
        if !path.starts_with("/api/v1/") {
            errors.push(format!("path {path} is outside the /api/v1 namespace"));
        }
    }

    let found = operations(&document);
    assert!(
        found.len() >= 20,
        "the contract must cover the full PRD-062 MVP capability matrix ({} operations found)",
        found.len()
    );

    let mut seen_ids: BTreeMap<&str, &str> = BTreeMap::new();
    for operation in &found {
        if operation.id.is_empty() {
            errors.push(format!("{} {} has no operationId", operation.method, operation.path));
            continue;
        }
        if let Some(previous) = seen_ids.get(operation.id.as_str()) {
            errors.push(format!(
                "operationId {} is duplicated ({} and {} {})",
                operation.id, previous, operation.method, operation.path
            ));
        } else {
            seen_ids.insert(operation.id.as_str(), operation.path.as_str());
        }
        let path_item = &document["paths"][&operation.path][operation.method];
        if path_item.get("responses").and_then(Value::as_object).is_none() {
            errors.push(format!("{} {} has no responses object", operation.method, operation.path));
        }
    }

    let global_security = document
        .get("security")
        .and_then(Value::as_array)
        .filter(|requirements| !requirements.is_empty());
    if global_security.is_none() {
        errors.push("document-level security must require a capability for all operations".into());
    }
    // An empty security-requirement object `{}` silently makes authentication
    // optional for an operation; only the unlock operation may opt out, and it
    // must do so with an explicit empty requirements list (`security: []`).
    if let Some(requirements) = global_security {
        if has_empty_security_requirement(requirements) {
            errors.push("document-level security contains an empty requirement object".into());
        }
    }
    for operation in &found {
        let operation_object = &document["paths"][&operation.path][operation.method];
        if let Some(Value::Array(requirements)) = operation_object.get("security") {
            if !requirements.is_empty() && has_empty_security_requirement(requirements) {
                errors.push(format!(
                    "{} {} declares an empty security requirement object",
                    operation.method, operation.path
                ));
            }
        }
    }

    // Every `{template}` path segment must be declared as an in-path parameter
    // on each of the path item's operations (inline or via components/parameters).
    for (path, path_item) in paths {
        let template_names: Vec<&str> = path
            .split('/')
            .filter_map(|segment| segment.strip_prefix('{')?.strip_suffix('}'))
            .collect();
        if template_names.is_empty() {
            continue;
        }
        let Some(items) = path_item.as_object() else {
            continue;
        };
        let mut path_level: Vec<&Value> = Vec::new();
        if let Some(Value::Array(parameters)) = items.get("parameters") {
            path_level.extend(parameters.iter());
        }
        for method in HTTP_METHODS {
            let Some(Value::Object(operation)) = items.get(method) else {
                continue;
            };
            let mut declared: BTreeSet<String> = BTreeSet::new();
            let parameter_lists = path_level.iter().copied().chain(
                operation
                    .get("parameters")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            );
            for parameter in parameter_lists {
                let resolved_parameter = parameter.get("$ref").and_then(Value::as_str);
                let resolved = match resolved_parameter.and_then(|target| {
                    target
                        .strip_prefix('#')
                        .filter(|pointer| !pointer.is_empty())
                        .and_then(|pointer| document.pointer(pointer))
                }) {
                    Some(resolved) => resolved,
                    None if resolved_parameter.is_some() => {
                        errors.push(format!(
                            "{method} {path}: parameter reference {} is not a resolvable local pointer",
                            resolved_parameter.unwrap_or_default()
                        ));
                        continue;
                    }
                    None => parameter,
                };
                if resolved.get("in").and_then(Value::as_str) == Some("path") {
                    if let Some(name) = resolved.get("name").and_then(Value::as_str) {
                        declared.insert(name.to_string());
                    }
                }
            }
            for name in &template_names {
                if !declared.contains(*name) {
                    errors.push(format!(
                        "{method} {path} does not declare path parameter {{{name}}}"
                    ));
                }
            }
        }
    }

    let unauthenticated: Vec<_> =
        found.iter().filter(|operation| operation.unauthenticated).collect();
    match unauthenticated.as_slice() {
        [only] if only.method == "post" && only.path == UNLOCK_PATH => {}
        [] => errors
            .push(format!("POST {UNLOCK_PATH} must be explicitly unauthenticated (security: [])")),
        other => errors.push(format!(
            "exactly one unauthenticated operation is allowed (POST {UNLOCK_PATH}); found {other:?}"
        )),
    }

    assert!(
        errors.is_empty(),
        "OpenAPI structural/security invariants violated:\n{}",
        errors.join("\n")
    );
}

#[test]
fn openapi_servers_are_loopback_only() {
    let document = load_openapi();
    let servers = document
        .get("servers")
        .and_then(Value::as_array)
        .filter(|servers| !servers.is_empty())
        .expect("document declares at least one server");
    for server in servers {
        let url = server.get("url").and_then(Value::as_str).unwrap_or_default();
        // Anchor the host on a terminator so DNS names that merely start with
        // the loopback literal (e.g. http://127.0.0.1.attacker.example) fail.
        let Some(rest) = url.strip_prefix("http://127.0.0.1") else {
            panic!("server url {url} must be the loopback literal 127.0.0.1");
        };
        assert!(
            rest.is_empty()
                || rest
                    .chars()
                    .next()
                    .is_some_and(|first| first == ':' || first == '/' || first == '{'),
            "server url {url} must end the host after 127.0.0.1 (port, path, or variable only)"
        );
    }
}

#[test]
fn openapi_internal_refs_resolve_and_every_component_schema_compiles() {
    let document = load_openapi();
    let mut errors = collect_internal_ref_errors(&document, &document, "$");

    if let Some(schemas) = document.pointer("/components/schemas").and_then(Value::as_object) {
        for name in schemas.keys() {
            // Compilation itself rejects malformed 2020-12 schemas.
            if let Err(error) = jsonschema::validator_for(&serde_json::json!({
                "$schema": JSON_SCHEMA_DIALECT,
                "$ref": format!("#/$defs/{name}"),
                "$defs": rewrite_component_refs(document.pointer("/components/schemas").unwrap_or(&Value::Null)),
            })) {
                errors.push(format!("components/schemas/{name} does not compile: {error}"));
            }
        }
    } else {
        errors.push("document has no components/schemas".into());
    }

    assert!(
        errors.is_empty(),
        "OpenAPI reference/schema integrity violations:\n{}",
        errors.join("\n")
    );
}

fn collect_internal_ref_errors(root: &Value, node: &Value, path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    match node {
        Value::Object(entries) => {
            if let Some(Value::String(target)) = entries.get("$ref") {
                if let Some(pointer) = target.strip_prefix('#') {
                    if root.pointer(pointer).is_none() {
                        errors.push(format!("{path}: unresolved $ref {target}"));
                    }
                }
            }
            for (key, child) in entries {
                if key != "$ref" {
                    errors.extend(collect_internal_ref_errors(
                        root,
                        child,
                        &format!("{path}.{key}"),
                    ));
                }
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                errors.extend(collect_internal_ref_errors(
                    root,
                    child,
                    &format!("{path}[{index}]"),
                ));
            }
        }
        _ => {}
    }
    errors
}

// ---------------------------------------------------------------------------
// forge.workspace/1 resource index schema
// ---------------------------------------------------------------------------

#[test]
fn workspace_schema_is_closed_versioned_and_bounded() {
    let schema = workspace_schema();
    assert_eq!(
        schema.get("$schema").and_then(Value::as_str),
        Some(JSON_SCHEMA_DIALECT),
        "workspace schema must pin draft 2020-12"
    );
    assert_eq!(
        schema.pointer("/properties/schema_version/const"),
        Some(&Value::String(WORKSPACE_SCHEMA_VERSION.to_string())),
        "workspace schema must const-pin schema_version to {WORKSPACE_SCHEMA_VERSION}"
    );
    assert_eq!(
        schema.get("type").and_then(Value::as_str),
        Some("object"),
        "workspace schema root must be an object"
    );
    assert_eq!(
        schema.get("additionalProperties"),
        Some(&Value::Bool(false)),
        "workspace schema must be closed at the root"
    );
    for required in ["schema_version", "label", "resources"] {
        assert!(
            schema
                .get("required")
                .and_then(Value::as_array)
                .is_some_and(|fields| fields.iter().any(|field| field == required)),
            "workspace schema must require {required}"
        );
    }
    jsonschema::validator_for(&schema).expect("workspace schema compiles");
}

// ---------------------------------------------------------------------------
// Fixture suite
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)] // Keeps the complete fixture-suite gate auditable in one place.
fn fixtures_validate_against_the_live_contract() {
    let document = load_openapi();
    let workspace = workspace_schema();
    let workspace_validator =
        jsonschema::validator_for(&workspace).expect("workspace schema compiles");

    let index_bytes = fs::read(repo_path(FIXTURE_INDEX_REL)).expect("fixture index exists");
    let index = read_json_strict(&index_bytes, FIXTURE_INDEX_REL)
        .unwrap_or_else(|error| panic!("{FIXTURE_INDEX_REL}: {error}"));
    assert_eq!(
        index.get("fixture_format").and_then(Value::as_str),
        Some(FIXTURE_FORMAT),
        "fixture index must declare {FIXTURE_FORMAT}"
    );
    let entries =
        index.get("fixtures").and_then(Value::as_array).expect("fixture index lists fixtures");
    assert!(
        entries.len() >= 25,
        "representative fixture coverage expected (found {})",
        entries.len()
    );

    let mut errors: Vec<String> = Vec::new();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut listed_files: BTreeSet<String> = BTreeSet::new();
    let mut workspace_valid = 0;
    let mut workspace_invalid = 0;

    for entry in entries {
        let label =
            entry.get("file").and_then(Value::as_str).unwrap_or("<missing file field>").to_string();
        let Some(file) = entry.get("file").and_then(Value::as_str) else {
            errors.push(format!("fixture entry without file: {entry}"));
            continue;
        };
        if file.is_empty()
            || file.starts_with('/')
            || file.contains("..")
            || file.contains('\\')
            || file.starts_with('.')
        {
            errors.push(format!("fixture path {file} must be a safe relative path"));
            continue;
        }
        let kind = entry.get("kind").and_then(Value::as_str).unwrap_or_default();
        if !REQUIRED_FIXTURE_KINDS.contains(&kind) {
            errors.push(format!("{label}: unknown kind {kind}"));
            continue;
        }
        let expectation = entry.get("expectation").and_then(Value::as_str).unwrap_or_default();
        if !matches!(expectation, "valid" | "invalid") {
            errors.push(format!("{label}: expectation must be valid|invalid"));
            continue;
        }
        let schema_ref =
            entry.get("schema").and_then(Value::as_str).unwrap_or_default().to_string();
        if entry.get("description").and_then(Value::as_str).is_none_or(str::is_empty) {
            errors.push(format!("{label}: description is required"));
        }

        let path = repo_path(FIXTURES_REL).join(file);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => {
                errors.push(format!("{label}: missing fixture file: {error}"));
                continue;
            }
        };
        let instance = match read_json_strict(&bytes, &label) {
            Ok(value) => value,
            Err(error) => {
                // Every fixture must be complete, well-formed strict JSON
                // (fixtures README rule 3: an invalid fixture fails schema
                // validation for its single documented reason, not parsing).
                errors.push(error);
                continue;
            }
        };

        let outcome = if schema_ref == "workspace:forge.workspace/1" {
            workspace_validator.validate(&instance).map_err(|error| error.to_string())
        } else if let Some(name) = schema_ref.strip_prefix("openapi:components/schemas/") {
            component_validator(&document, name)
                .validate(&instance)
                .map_err(|error| error.to_string())
        } else {
            errors.push(format!("{label}: unsupported schema reference {schema_ref}"));
            continue;
        };

        match (expectation, outcome) {
            ("valid", Err(reason)) => {
                errors.push(format!("{label}: expected valid, got: {reason}"));
            }
            ("invalid", Ok(())) => {
                errors.push(format!("{label}: expected invalid, but it validates"));
            }
            ("valid", Ok(())) | ("invalid", Err(_)) => {}
            _ => unreachable!("expectation is constrained above"),
        }

        match kinds.entry(kind.to_string()) {
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                *slot.get_mut() += 1;
            }
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(1);
            }
        }
        if !listed_files.insert(file.to_string()) {
            errors.push(format!("{label}: fixture listed more than once"));
        }
        if kind == "workspace" {
            if expectation == "valid" {
                workspace_valid += 1;
            } else {
                workspace_invalid += 1;
            }
        }
    }

    for kind in REQUIRED_FIXTURE_KINDS {
        if !kinds.contains_key(kind) {
            errors.push(format!("fixture suite lacks required kind {kind}"));
        }
    }
    assert!(workspace_valid >= 1, "at least one valid workspace fixture is required");
    assert!(workspace_invalid >= 2, "at least two invalid workspace fixtures are required");

    // Drift check: every fixture file on disk must be listed in the index.
    let mut on_disk = Vec::new();
    let fixtures_root = repo_path(FIXTURES_REL);
    collect_files(&fixtures_root, &fixtures_root, &mut on_disk);
    for file in on_disk {
        if file != "index.json" && file != "README.md" && !listed_files.contains(&file) {
            errors.push(format!("orphan fixture file not listed in index.json: {file}"));
        }
    }

    assert!(errors.is_empty(), "fixture validation failures:\n{}", errors.join("\n"));
}

// ---------------------------------------------------------------------------
// Capability matrix
// ---------------------------------------------------------------------------

#[test]
#[allow(clippy::too_many_lines)] // Keeps the complete bidirectional coverage gate auditable in one place.
fn capability_matrix_covers_the_contract_in_both_directions() {
    let document = load_openapi();
    let matrix_bytes = fs::read(repo_path(MATRIX_JSON_REL)).expect("capability matrix JSON exists");
    let matrix = read_json_strict(&matrix_bytes, MATRIX_JSON_REL)
        .unwrap_or_else(|error| panic!("{MATRIX_JSON_REL}: {error}"));
    let entries = matrix
        .get("entries")
        .and_then(Value::as_array)
        .or_else(|| matrix.as_array())
        .expect("capability matrix lists entries");
    assert!(
        entries.len() >= 15,
        "capability matrix must cover the golden path and cross-cutting actions (found {})",
        entries.len()
    );

    let openapi_ids: BTreeSet<String> = operations(&document)
        .into_iter()
        .map(|operation| operation.id)
        .filter(|id| !id.is_empty())
        .collect();
    let unlock_id = unlock_operation_id(&document);
    let mut errors: Vec<String> = Vec::new();
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut covered: BTreeSet<String> = BTreeSet::new();
    let mut steps: BTreeSet<u32> = BTreeSet::new();

    for entry in entries {
        let id = entry.get("id").and_then(Value::as_str).unwrap_or_default();
        if id.len() != 5 || !id.starts_with("CM-") || !id[3..].chars().all(|c| c.is_ascii_digit()) {
            errors.push(format!("capability id {id} must match CM-NN"));
            continue;
        }
        if !ids.insert(id.to_string()) {
            errors.push(format!("duplicate capability id {id}"));
        }
        if entry.get("action").and_then(Value::as_str).is_none_or(str::is_empty) {
            errors.push(format!("{id}: action is required"));
        }
        match entry.get("golden_path_step") {
            Some(Value::Number(number)) => match number.as_u64() {
                Some(step) if (1..=8).contains(&step) => {
                    steps.insert(u32::try_from(step).expect("bounded by the range check"));
                }
                _ => errors.push(format!(
                    "{id}: golden_path_step must be 1-8 or cross-cutting, got {number}"
                )),
            },
            Some(Value::String(step)) if step == "cross-cutting" => {}
            other => errors.push(format!(
                "{id}: golden_path_step must be 1-8 or cross-cutting, got {other:?}"
            )),
        }
        for story in entry.get("user_stories").and_then(Value::as_array).unwrap_or(&Vec::new()) {
            let story = story.as_str().unwrap_or_default();
            if !(story.starts_with("US-")
                && story[3..].chars().all(|c| c.is_ascii_digit())
                && story.len() >= 4)
            {
                errors.push(format!("{id}: invalid user story {story}"));
            }
        }
        let authorization = entry.get("authorization").and_then(Value::as_str).unwrap_or_default();
        if !ALLOWED_AUTHORIZATIONS.contains(&authorization) {
            errors.push(format!(
                "{id}: authorization {authorization} must be one of {ALLOWED_AUTHORIZATIONS:?}"
            ));
        }
        let operations_list =
            entry.get("operations").and_then(Value::as_array).cloned().unwrap_or_default();
        if operations_list.is_empty() {
            errors.push(format!("{id}: at least one operation is required"));
        }
        for operation in &operations_list {
            let operation = operation.as_str().unwrap_or_default().to_string();
            if !openapi_ids.contains(&operation) {
                errors.push(format!(
                    "{id}: references operationId {operation} missing from the contract"
                ));
            }
            if authorization == "unlock" && operation != unlock_id {
                errors.push(format!(
                    "{id}: unlock-authorized entries may only use the unlock operation"
                ));
            }
            covered.insert(operation);
        }
    }

    for step in GOLDEN_PATH_STEPS {
        if !steps.contains(&step) {
            errors.push(format!("golden path step {step} has no capability matrix entry"));
        }
    }
    let orphans: Vec<_> = openapi_ids.difference(&covered).collect();
    if !orphans.is_empty() {
        errors
            .push(format!("contract operations without any capability matrix entry: {orphans:?}"));
    }

    // The human-readable matrix must stay in sync with the machine-readable one.
    let markdown = fs::read_to_string(repo_path(MATRIX_MD_REL))
        .unwrap_or_else(|error| panic!("read {MATRIX_MD_REL}: {error}"));
    for id in &ids {
        if !markdown.contains(id) {
            errors.push(format!("{MATRIX_MD_REL} does not mention capability {id}"));
        }
    }
    for operation in &covered {
        if !markdown.contains(operation) {
            errors.push(format!("{MATRIX_MD_REL} does not mention operationId {operation}"));
        }
    }

    assert!(errors.is_empty(), "capability matrix coverage failures:\n{}", errors.join("\n"));
}

/// Rewrite only the original index schema's local definitions into API components.
/// The equality test below prevents a separately maintained nested index contract.
fn bundle_index_api_mirror(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let rewritten = if key == "$ref" {
                        match value.as_str() {
                            Some("#/$defs/resource") => {
                                json!("#/components/schemas/WorkspaceBundleIndexResource")
                            }
                            Some("#/$defs/role") => {
                                json!("#/components/schemas/WorkspaceBundleIndexRole")
                            }
                            _ => value.clone(),
                        }
                    } else {
                        bundle_index_api_mirror(value)
                    };
                    (key.clone(), rewritten)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(bundle_index_api_mirror).collect()),
        _ => value.clone(),
    }
}

/// Keep all nested index constraints identical to the existing closed on-disk schema.
#[test]
fn bundle_index_components_are_composed_from_the_unchanged_index_schema() {
    let document = load_openapi();
    let raw = fs::read(repo_path("schemas/forge.workspace-1.schema.json")).unwrap();
    let original = read_json_strict(&raw, "workspace index schema").unwrap();
    let mut expected = original.clone();
    for key in ["$schema", "$id", "$defs"] {
        expected.as_object_mut().unwrap().remove(key);
    }
    for (name, definition) in [
        ("WorkspaceBundleIndex", &expected),
        ("WorkspaceBundleIndexResource", &original["$defs"]["resource"]),
        ("WorkspaceBundleIndexRole", &original["$defs"]["role"]),
    ] {
        assert_eq!(
            document["components"]["schemas"][name],
            bundle_index_api_mirror(definition),
            "{name} must preserve the original index contract"
        );
    }
    assert_eq!(document["info"]["version"], "1.2.0");
}
/// Validate the separately published API2 contracts without renumbering or weakening original API1 checks.
mod s3_api2_contracts {
    use super::{
        bundle_index_api_mirror, component_validator, load_openapi, load_yaml_as_json, operations,
        read_json_strict, repo_path, workspace_schema,
    };
    use serde_json::{Value, json};
    use std::fs;

    /// Load the selected major's own normative document rather than rewriting API1 at test time.
    fn s3_api2_document() -> Value {
        load_yaml_as_json(&repo_path("docs/api/forge-workspace-v2.openapi.yaml"))
    }

    /// Parse the separate closed index2 artifact using the existing duplicate-safe fixture decoder.
    fn s3_index2_schema() -> Value {
        let relative = "schemas/forge.workspace-2.schema.json";
        read_json_strict(&fs::read(repo_path(relative)).unwrap(), relative).unwrap()
    }

    /// Mirror only local index2 references into its explicit API2 component siblings.
    fn s3_index2_mirror(value: &Value) -> Value {
        match value {
            Value::Object(map) => Value::Object(
                map.iter()
                    .map(|(key, child)| {
                        let rewritten = if key == "$ref" {
                            match child.as_str() {
                                Some("#/$defs/resource") => {
                                    json!("#/components/schemas/WorkspaceBundleIndexResourceV2")
                                }
                                Some("#/$defs/role") => {
                                    json!("#/components/schemas/WorkspaceBundleIndexRoleV2")
                                }
                                _ => child.clone(),
                            }
                        } else {
                            s3_index2_mirror(child)
                        };
                        (key.clone(), rewritten)
                    })
                    .collect(),
            ),
            Value::Array(items) => Value::Array(items.iter().map(s3_index2_mirror).collect()),
            _ => value.clone(),
        }
    }

    /// Construct schema-only metadata bundles; zero pins and arbitrary digest are not native capture or hash proof.
    fn s3_schema_bundle(version: u8) -> Value {
        json!({"schema_version":format!("forge.workspace-index-bundle/{version}"),"content_profile":"index-and-hashes",
            "index":{"schema_version":format!("forge.workspace/{version}"),"label":"Schema-only empty index","resources":[]},
            "index_sha256":"a".repeat(64),"pins":[]})
    }

    /// Preserve API1 and API2 namespaces, bootstrap versions and operation identities as separate documents.
    #[test]
    fn s3_api2_document_namespace_and_bootstrap_do_not_change_api1() {
        let one = load_openapi();
        let two = s3_api2_document();
        assert_eq!(one["info"]["version"], "1.2.0");
        assert_eq!(two["info"]["version"], "2.2.0");
        let first = operations(&one);
        let second = operations(&two);
        assert_eq!(first.len(), 39);
        assert_eq!(second.len(), 51);
        let mut left: Vec<_> = first
            .iter()
            .map(|operation| {
                (
                    operation.method,
                    operation.path.strip_prefix("/api/v1").unwrap(),
                    operation.id.as_str(),
                )
            })
            .collect();
        let mut right: Vec<_> = second
            .iter()
            .filter(|operation| first.iter().any(|original| original.id == operation.id))
            .map(|operation| {
                (
                    operation.method,
                    operation.path.strip_prefix("/api/v2").unwrap(),
                    operation.id.as_str(),
                )
            })
            .collect();
        left.sort_unstable();
        right.sort_unstable();
        assert_eq!(left, right);
        assert_eq!(two["components"]["schemas"]["Session"]["properties"]["api_major"]["const"], 2);
        assert_eq!(one["components"]["schemas"]["Session"]["properties"]["api_major"]["const"], 1);
        // The additive contract retains all39 original operations; execution and full parity are separate evidence.
        assert!(second.iter().all(|operation| !operation.path.starts_with("/api/v1/")));
    }

    /// Old nullable keys remain valid; version/migration alternatives are closed and confined to API2.
    #[test]
    fn s3_api2_register_schema_keeps_null_key_compatibility_and_closed_alternatives() {
        let one = load_openapi();
        let two = s3_api2_document();
        let old = component_validator(&one, "RegisterResourceRequest");
        let new = component_validator(&two, "RegisterResourceRequest");
        let legacy = json!({"path":"policy.md","role":"policy-source","key":null});
        assert!(old.is_valid(&legacy));
        assert!(new.is_valid(&legacy));
        let selected = json!({"path":"opaque.bin","role":"lifecycle-source","key":null,"index_schema_version":"forge.workspace/2"});
        let migration = json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2"}});
        assert!(new.is_valid(&selected));
        assert!(new.is_valid(&migration));
        assert!(!old.is_valid(&selected));
        assert!(!old.is_valid(&migration));
        let current_two = json!({"path":"opaque.bin","role":"lifecycle-source"});
        assert!(new.is_valid(&current_two));
        assert!(!old.is_valid(&current_two));
        for invalid in [
            json!({"path":"policy.md","role":"policy-source","index_schema_version":null}),
            json!({"path":"policy.md","role":"policy-source","index_schema_version":"forge.workspace/1"}),
            json!({"path":"policy.md","role":"policy-source","index_schema_version":"forge.workspace/3"}),
            json!({"path":"policy.md","role":"policy-source","unknown":true}),
            json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2"},"path":"policy.md","role":"policy-source"}),
            json!({"migration":null}),
            json!({"migration":{"from":null,"to":"forge.workspace/2"}}),
            json!({"migration":{"from":"forge.workspace/2","to":"forge.workspace/2"}}),
            json!({"migration":{"from":"forge.workspace/1","to":"forge.workspace/2","approval":true}}),
            json!({"path":"policy.md","role":"unsupported-role"}),
        ] {
            assert!(!new.is_valid(&invalid), "invalid API2 alternative: {invalid}");
        }
    }

    /// Separate index mirrors preserve original seven roles and /2's full fifteen-role and 1000-entry shape.
    #[test]
    fn s3_api2_index_mirrors_keep_version_roles_and_closed_bounds() {
        let document = s3_api2_document();
        let one = workspace_schema();
        let two = s3_index2_schema();
        let old = jsonschema::validator_for(&one).unwrap();
        let new = jsonschema::validator_for(&two).unwrap();
        let roles_one = one["$defs"]["role"]["enum"].as_array().unwrap();
        let roles_two = two["$defs"]["role"]["enum"].as_array().unwrap();
        assert_eq!(roles_one.len(), 7);
        assert_eq!(roles_two.len(), 15);
        assert_eq!(&roles_two[..7], roles_one);
        for (schema, names, mirror) in [
            (
                &one,
                [
                    "WorkspaceBundleIndex",
                    "WorkspaceBundleIndexResource",
                    "WorkspaceBundleIndexRole",
                ],
                bundle_index_api_mirror as fn(&Value) -> Value,
            ),
            (
                &two,
                [
                    "WorkspaceBundleIndexV2",
                    "WorkspaceBundleIndexResourceV2",
                    "WorkspaceBundleIndexRoleV2",
                ],
                s3_index2_mirror as fn(&Value) -> Value,
            ),
        ] {
            let mut index = schema.clone();
            for field in ["$schema", "$id", "$defs"] {
                index.as_object_mut().unwrap().remove(field);
            }
            for (name, value) in names.into_iter().zip([
                &index,
                &schema["$defs"]["resource"],
                &schema["$defs"]["role"],
            ]) {
                assert_eq!(document["components"]["schemas"][name], mirror(value), "{name}");
            }
        }
        for (number, role) in roles_two.iter().enumerate() {
            let mut index = json!({"schema_version":"forge.workspace/2","label":"Role admission","resources":[{"key":"resource","role":role,"path":"source.bin"}]});
            assert!(new.is_valid(&index), "index2 role {role}");
            index["schema_version"] = json!("forge.workspace/1");
            assert_eq!(old.is_valid(&index), number < 7, "index1 role {role}");
        }
        let mut empty =
            json!({"schema_version":"forge.workspace/2","label":"Bounds","resources":[]});
        assert!(new.is_valid(&empty));
        empty["resources"] = json!((0..1000).map(|number| json!({"key":format!("resource-{number}"),"role":"lifecycle-source","path":format!("source-{number}.bin")})).collect::<Vec<_>>());
        assert!(new.is_valid(&empty));
        empty["resources"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":"excess","role":"lifecycle-source","path":"excess.bin"}));
        assert!(!new.is_valid(&empty));
        empty["resources"] = json!([]);
        for (field, value) in [
            ("schema_version", json!("forge.workspace/3")),
            ("label", json!("")),
            ("unknown", json!(true)),
        ] {
            let mut invalid = empty.clone();
            invalid[field] = value;
            assert!(!new.is_valid(&invalid));
        }
    }

    /// Schema-only bundle pairing stays closed; actual pin bijection/hash/IO controls remain in real HTTP tests.
    #[test]
    fn s3_api2_bundle_components_pair_only_matching_index_versions() {
        let one = load_openapi();
        let two = s3_api2_document();
        let old = component_validator(&one, "WorkspaceBundleVerificationRequest");
        let new = component_validator(&two, "WorkspaceBundleVerificationRequest");
        let first = json!({"bundle":s3_schema_bundle(1)});
        let second = json!({"bundle":s3_schema_bundle(2)});
        assert!(old.is_valid(&first));
        assert!(!old.is_valid(&second));
        assert!(new.is_valid(&first));
        assert!(new.is_valid(&second));
        for (pointer, value) in [
            ("/bundle/schema_version", json!("forge.workspace-index-bundle/1")),
            ("/bundle/index/schema_version", json!("forge.workspace/1")),
            ("/bundle/content_profile", json!("source-inclusive")),
            ("/bundle/schema_version", json!("forge.workspace-index-bundle/3")),
        ] {
            let mut invalid = second.clone();
            *invalid.pointer_mut(pointer).unwrap() = value;
            assert!(!new.is_valid(&invalid));
        }
        let mut unknown = second;
        unknown["bundle"]["approval"] = json!(true);
        assert!(!new.is_valid(&unknown));
        let mut mixed = first;
        mixed["content"] = json!("PRIVATE REJECTED VALUE");
        assert!(!new.is_valid(&mixed));
    }

    /// Optional validation profiles are paired to the exact new role and remain absent from legacy metadata.
    #[test]
    fn s3_api2_resource_profiles_are_closed_and_role_specific() {
        let one = load_openapi();
        let two = s3_api2_document();
        let old = component_validator(&one, "Resource");
        let new = component_validator(&two, "Resource");
        let base = json!({"resource_id":"res_abcdefghijkl","key":"resource","role":"policy-source","path":"source.bin","sha256":"a".repeat(64),"size_bytes":0,"validation_state":"valid","stale":false,"version":"a".repeat(64)});
        assert!(old.is_valid(&base));
        assert!(new.is_valid(&base));
        for (role, profile) in [
            ("lifecycle-record", "lifecycle-record-structure"),
            ("lifecycle-source", "opaque-fingerprint-bytes"),
            ("oscal-profile-artifact", "native-oscal-schema"),
            ("oscal-ssp-artifact", "native-oscal-schema"),
            ("framework-impact-manifest", "framework-impact-manifest"),
            ("successor-map", "successor-map"),
            ("framework-impact-report", "framework-impact-prior-admission"),
            ("framework-impact-dispositions", "framework-impact-dispositions"),
        ] {
            let mut resource = base.clone();
            resource["role"] = json!(role);
            assert!(new.is_valid(&resource));
            assert!(!old.is_valid(&resource));
            resource["validation_profile"] = json!(profile);
            assert!(new.is_valid(&resource));
            let mut wrong = resource.clone();
            wrong["validation_profile"] = json!(if profile == "opaque-fingerprint-bytes" {
                "lifecycle-record-structure"
            } else {
                "opaque-fingerprint-bytes"
            });
            assert!(!new.is_valid(&wrong));
            resource["validation_profile"] = json!(null);
            assert!(!new.is_valid(&resource));
        }
        let mut legacy = base;
        legacy["validation_profile"] = json!("opaque-fingerprint-bytes");
        assert!(!new.is_valid(&legacy));
        assert!(!old.is_valid(&legacy));
    }
}

/// Apply the original artifact drift gates to the independently committed API2 family.
mod api2_artifact_drift {
    use super::*;

    const FIXTURES_REL: &str = "docs/api/fixtures-v2";
    const FIXTURE_INDEX_REL: &str = "docs/api/fixtures-v2/index.json";
    const MATRIX_JSON_REL: &str = "docs/api/capability-matrix-v2.json";
    const MATRIX_MD_REL: &str = "docs/api/capability-matrix-v2.md";

    /// Resolve the sole unauthenticated operation within this v2 artifact family.
    fn unlock_operation_id(document: &Value) -> String {
        document["paths"]["/api/v2/session/unlock"]["post"]["operationId"]
            .as_str()
            .expect("v2 unlock operation identifier")
            .to_owned()
    }

    /// Load the committed v2 document directly; preserve the original v1 helper.
    fn load_openapi() -> Value {
        load_yaml_as_json(&repo_path("docs/api/forge-workspace-v2.openapi.yaml"))
    }

    /// Validate the entire v2 document against the pinned official meta-schema offline.
    #[test]
    fn openapi_document_is_valid_openapi_3_1() {
        let document = load_openapi();
        assert!(
            document
                .get("openapi")
                .and_then(Value::as_str)
                .is_some_and(|version| version.starts_with("3.1.")),
            "contract must declare OpenAPI 3.1.x"
        );
        assert_eq!(
            document.get("jsonSchemaDialect").and_then(Value::as_str),
            Some(JSON_SCHEMA_DIALECT),
            "contract must pin the draft 2020-12 JSON Schema dialect"
        );
        for field in ["title", "version"] {
            assert!(
                document
                    .pointer(&format!("/info/{field}"))
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty()),
                "info.{field} must be a non-empty string"
            );
        }
        let meta_bytes = fs::read(repo_path(META_SCHEMA_REL)).expect("vendored meta-schema exists");
        let meta = read_json_strict(&meta_bytes, META_SCHEMA_REL).expect("meta-schema parses");
        let validator = jsonschema::validator_for(&meta).expect("meta-schema compiles");
        validator
            .validate(&document)
            .expect("normative document validates against the official OpenAPI 3.1 schema");
    }

    /// Resolve every v2 reference and compile every component rather than sampling five schemas.
    #[test]
    fn openapi_internal_refs_resolve_and_every_component_schema_compiles() {
        let document = load_openapi();
        let mut errors = collect_internal_ref_errors(&document, &document, "$");

        if let Some(schemas) = document.pointer("/components/schemas").and_then(Value::as_object) {
            for name in schemas.keys() {
                // Compilation itself rejects malformed 2020-12 schemas.
                if let Err(error) = jsonschema::validator_for(&serde_json::json!({
                    "$schema": JSON_SCHEMA_DIALECT,
                    "$ref": format!("#/$defs/{name}"),
                    "$defs": rewrite_component_refs(document.pointer("/components/schemas").unwrap_or(&Value::Null)),
                })) {
                    errors.push(format!("components/schemas/{name} does not compile: {error}"));
                }
            }
        } else {
            errors.push("document has no components/schemas".into());
        }

        assert!(
            errors.is_empty(),
            "OpenAPI reference/schema integrity violations:\n{}",
            errors.join("\n")
        );
    }

    /// Validate every separate v2 fixture and its exact declared family/expectation.
    #[test]
    #[allow(clippy::too_many_lines)] // One complete artifact drift check, matching the existing v1 harness.
    fn fixtures_validate_against_the_live_contract() {
        let document = load_openapi();
        let workspace = workspace_schema();
        let workspace_validator =
            jsonschema::validator_for(&workspace).expect("workspace schema compiles");

        let index_bytes = fs::read(repo_path(FIXTURE_INDEX_REL)).expect("fixture index exists");
        let index = read_json_strict(&index_bytes, FIXTURE_INDEX_REL)
            .unwrap_or_else(|error| panic!("{FIXTURE_INDEX_REL}: {error}"));
        assert_eq!(
            index.get("fixture_format").and_then(Value::as_str),
            Some(FIXTURE_FORMAT),
            "fixture index must declare {FIXTURE_FORMAT}"
        );
        let entries =
            index.get("fixtures").and_then(Value::as_array).expect("fixture index lists fixtures");
        assert!(
            entries.len() >= 25,
            "representative fixture coverage expected (found {})",
            entries.len()
        );

        let mut errors: Vec<String> = Vec::new();
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        let mut listed_files: BTreeSet<String> = BTreeSet::new();
        let mut workspace_valid = 0;
        let mut workspace_invalid = 0;

        for entry in entries {
            let label = entry
                .get("file")
                .and_then(Value::as_str)
                .unwrap_or("<missing file field>")
                .to_string();
            let Some(file) = entry.get("file").and_then(Value::as_str) else {
                errors.push(format!("fixture entry without file: {entry}"));
                continue;
            };
            if file.is_empty()
                || file.starts_with('/')
                || file.contains("..")
                || file.contains('\\')
                || file.starts_with('.')
            {
                errors.push(format!("fixture path {file} must be a safe relative path"));
                continue;
            }
            let kind = entry.get("kind").and_then(Value::as_str).unwrap_or_default();
            if !REQUIRED_FIXTURE_KINDS.contains(&kind) {
                errors.push(format!("{label}: unknown kind {kind}"));
                continue;
            }
            let expectation = entry.get("expectation").and_then(Value::as_str).unwrap_or_default();
            if !matches!(expectation, "valid" | "invalid") {
                errors.push(format!("{label}: expectation must be valid|invalid"));
                continue;
            }
            let schema_ref =
                entry.get("schema").and_then(Value::as_str).unwrap_or_default().to_string();
            if entry.get("description").and_then(Value::as_str).is_none_or(str::is_empty) {
                errors.push(format!("{label}: description is required"));
            }

            let path = repo_path(FIXTURES_REL).join(file);
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    errors.push(format!("{label}: missing fixture file: {error}"));
                    continue;
                }
            };
            let instance = match read_json_strict(&bytes, &label) {
                Ok(value) => value,
                Err(error) => {
                    // Every fixture must be complete, well-formed strict JSON
                    // (fixtures README rule 3: an invalid fixture fails schema
                    // validation for its single documented reason, not parsing).
                    errors.push(error);
                    continue;
                }
            };

            let outcome = if schema_ref == "workspace:forge.workspace/1" {
                workspace_validator.validate(&instance).map_err(|error| error.to_string())
            } else if let Some(name) = schema_ref.strip_prefix("openapi:components/schemas/") {
                component_validator(&document, name)
                    .validate(&instance)
                    .map_err(|error| error.to_string())
            } else {
                errors.push(format!("{label}: unsupported schema reference {schema_ref}"));
                continue;
            };

            match (expectation, outcome) {
                ("valid", Err(reason)) => {
                    errors.push(format!("{label}: expected valid, got: {reason}"));
                }
                ("invalid", Ok(())) => {
                    errors.push(format!("{label}: expected invalid, but it validates"));
                }
                ("valid", Ok(())) | ("invalid", Err(_)) => {}
                _ => unreachable!("expectation is constrained above"),
            }

            match kinds.entry(kind.to_string()) {
                std::collections::btree_map::Entry::Occupied(mut slot) => {
                    *slot.get_mut() += 1;
                }
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(1);
                }
            }
            if !listed_files.insert(file.to_string()) {
                errors.push(format!("{label}: fixture listed more than once"));
            }
            if kind == "workspace" {
                if expectation == "valid" {
                    workspace_valid += 1;
                } else {
                    workspace_invalid += 1;
                }
            }
        }

        for kind in REQUIRED_FIXTURE_KINDS {
            if !kinds.contains_key(kind) {
                errors.push(format!("fixture suite lacks required kind {kind}"));
            }
        }
        assert!(workspace_valid >= 1, "at least one valid workspace fixture is required");
        assert!(workspace_invalid >= 2, "at least two invalid workspace fixtures are required");

        // Drift check: every fixture file on disk must be listed in the index.
        let mut on_disk = Vec::new();
        let fixtures_root = repo_path(FIXTURES_REL);
        collect_files(&fixtures_root, &fixtures_root, &mut on_disk);
        for file in on_disk {
            if file != "index.json" && file != "README.md" && !listed_files.contains(&file) {
                errors.push(format!("orphan fixture file not listed in index.json: {file}"));
            }
        }

        assert!(errors.is_empty(), "fixture validation failures:\n{}", errors.join("\n"));
    }

    /// Require the v2 capability matrix to cover every normative operation and match its readable companion.
    #[test]
    #[allow(clippy::too_many_lines)] // One complete artifact drift check, matching the existing v1 harness.
    fn capability_matrix_covers_the_contract_in_both_directions() {
        let document = load_openapi();
        let matrix_bytes =
            fs::read(repo_path(MATRIX_JSON_REL)).expect("capability matrix JSON exists");
        let matrix = read_json_strict(&matrix_bytes, MATRIX_JSON_REL)
            .unwrap_or_else(|error| panic!("{MATRIX_JSON_REL}: {error}"));
        let entries = matrix
            .get("entries")
            .and_then(Value::as_array)
            .or_else(|| matrix.as_array())
            .expect("capability matrix lists entries");
        assert!(
            entries.len() >= 15,
            "capability matrix must cover the golden path and cross-cutting actions (found {})",
            entries.len()
        );

        let openapi_ids: BTreeSet<String> = operations(&document)
            .into_iter()
            .map(|operation| operation.id)
            .filter(|id| !id.is_empty())
            .collect();
        let unlock_id = unlock_operation_id(&document);
        let mut errors: Vec<String> = Vec::new();
        let mut ids: BTreeSet<String> = BTreeSet::new();
        let mut covered: BTreeSet<String> = BTreeSet::new();
        let mut steps: BTreeSet<u32> = BTreeSet::new();

        for entry in entries {
            let id = entry.get("id").and_then(Value::as_str).unwrap_or_default();
            if id.len() != 5
                || !id.starts_with("CM-")
                || !id[3..].chars().all(|c| c.is_ascii_digit())
            {
                errors.push(format!("capability id {id} must match CM-NN"));
                continue;
            }
            if !ids.insert(id.to_string()) {
                errors.push(format!("duplicate capability id {id}"));
            }
            if entry.get("action").and_then(Value::as_str).is_none_or(str::is_empty) {
                errors.push(format!("{id}: action is required"));
            }
            match entry.get("golden_path_step") {
                Some(Value::Number(number)) => match number.as_u64() {
                    Some(step) if (1..=8).contains(&step) => {
                        steps.insert(u32::try_from(step).expect("bounded by the range check"));
                    }
                    _ => errors.push(format!(
                        "{id}: golden_path_step must be 1-8 or cross-cutting, got {number}"
                    )),
                },
                Some(Value::String(step)) if step == "cross-cutting" => {}
                other => errors.push(format!(
                    "{id}: golden_path_step must be 1-8 or cross-cutting, got {other:?}"
                )),
            }
            for story in entry.get("user_stories").and_then(Value::as_array).unwrap_or(&Vec::new())
            {
                let story = story.as_str().unwrap_or_default();
                if !(story.starts_with("US-")
                    && story[3..].chars().all(|c| c.is_ascii_digit())
                    && story.len() >= 4)
                {
                    errors.push(format!("{id}: invalid user story {story}"));
                }
            }
            let authorization =
                entry.get("authorization").and_then(Value::as_str).unwrap_or_default();
            if !ALLOWED_AUTHORIZATIONS.contains(&authorization) {
                errors.push(format!(
                    "{id}: authorization {authorization} must be one of {ALLOWED_AUTHORIZATIONS:?}"
                ));
            }
            let operations_list =
                entry.get("operations").and_then(Value::as_array).cloned().unwrap_or_default();
            if operations_list.is_empty() {
                errors.push(format!("{id}: at least one operation is required"));
            }
            for operation in &operations_list {
                let operation = operation.as_str().unwrap_or_default().to_string();
                if !openapi_ids.contains(&operation) {
                    errors.push(format!(
                        "{id}: references operationId {operation} missing from the contract"
                    ));
                }
                if authorization == "unlock" && operation != unlock_id {
                    errors.push(format!(
                        "{id}: unlock-authorized entries may only use the unlock operation"
                    ));
                }
                covered.insert(operation);
            }
        }

        for step in GOLDEN_PATH_STEPS {
            if !steps.contains(&step) {
                errors.push(format!("golden path step {step} has no capability matrix entry"));
            }
        }
        let orphans: Vec<_> = openapi_ids.difference(&covered).collect();
        if !orphans.is_empty() {
            errors.push(format!(
                "contract operations without any capability matrix entry: {orphans:?}"
            ));
        }

        // The human-readable matrix must stay in sync with the machine-readable one.
        let markdown = fs::read_to_string(repo_path(MATRIX_MD_REL))
            .unwrap_or_else(|error| panic!("read {MATRIX_MD_REL}: {error}"));
        for id in &ids {
            if !markdown.contains(id) {
                errors.push(format!("{MATRIX_MD_REL} does not mention capability {id}"));
            }
        }
        for operation in &covered {
            if !markdown.contains(operation) {
                errors.push(format!("{MATRIX_MD_REL} does not mention operationId {operation}"));
            }
        }

        assert!(errors.is_empty(), "capability matrix coverage failures:\n{}", errors.join("\n"));
    }
}

/// Bind release inventory declarations to the independently versioned normative families.
#[test]
fn release_inventory_contains_both_current_api_families() {
    let inventory = load_yaml_as_json(&repo_path("docs/api/release-artifacts.json"));
    assert_eq!(inventory["schema_version"], "forge.workspace-api-release-artifacts/1");
    let families = inventory["api_majors"].as_array().expect("API families");
    assert_eq!(families.len(), 2);
    let package_paths: Vec<&str> = inventory["package_paths"]
        .as_array()
        .expect("package paths")
        .iter()
        .map(|path| path.as_str().expect("relative package path"))
        .collect();
    assert_eq!(
        package_paths,
        [
            "docs/api",
            "schemas/forge.workspace-1.schema.json",
            "schemas/forge.workspace-2.schema.json"
        ]
    );
    for (family, (major, version, index_version)) in
        families.iter().zip([(1, "1.2.0", "forge.workspace/1"), (2, "2.2.0", "forge.workspace/2")])
    {
        assert_eq!(family["api_major"], major);
        assert_eq!(family["contract_version"], version);
        let document_path = family["document"].as_str().expect("document path");
        assert_eq!(document_path, format!("docs/api/forge-workspace-v{major}.openapi.yaml"));
        let document = load_yaml_as_json(&repo_path(document_path));
        assert_eq!(document["info"]["version"], version);
        let declared_operations = operations(&document);
        assert_eq!(declared_operations.len(), if major == 1 { 39 } else { 51 });
        assert!(
            declared_operations
                .iter()
                .all(|operation| operation.path.starts_with(&format!("/api/v{major}/")))
        );
        let fixture_path = family["fixtures"].as_str().expect("fixture directory");
        let suffix = if major == 1 { "" } else { "-v2" };
        assert_eq!(fixture_path, format!("docs/api/fixtures{suffix}"));
        assert_eq!(
            family["capability_matrix"],
            json!([
                format!("docs/api/capability-matrix{suffix}.json"),
                format!("docs/api/capability-matrix{suffix}.md"),
            ])
        );
        let expected_schemas: Vec<String> = (1..=major)
            .map(|version| format!("schemas/forge.workspace-{version}.schema.json"))
            .collect();
        assert_eq!(family["index_schemas"], json!(expected_schemas));
        assert!(repo_path(fixture_path).join("index.json").is_file());
        let schemas = family["index_schemas"].as_array().expect("paired index schemas");
        let current_schema =
            schemas.last().expect("current index schema").as_str().expect("schema path");
        assert_eq!(
            load_yaml_as_json(&repo_path(current_schema))["properties"]["schema_version"]["const"],
            index_version
        );
        let mut assets = vec![document_path, fixture_path];
        assets.extend(
            family["capability_matrix"]
                .as_array()
                .expect("matrix artifacts")
                .iter()
                .map(|path| path.as_str().expect("matrix path")),
        );
        assets.extend(schemas.iter().map(|path| path.as_str().expect("index schema path")));
        for asset in assets {
            assert!(repo_path(asset).exists(), "declared release asset missing: {asset}");
            assert!(
                package_paths
                    .iter()
                    .any(|parent| asset == *parent || asset.starts_with(&format!("{parent}/"))),
                "declared asset outside package inventory: {asset}"
            );
        }
    }
}

/// Keep the nine captured reads authenticated, finite and confined to the API2 successor.
#[test]
fn lifecycle_impact_routes_have_exact_dates_queries_and_safe_stops() {
    let one = load_openapi();
    let two = load_yaml_as_json(&repo_path("docs/api/forge-workspace-v2.openapi.yaml"));
    let routes: [(&str, &str, &[&str], bool); 9] = [
        (
            "/lifecycle/records",
            "LifecycleRecordPage",
            &["as_of", "owner", "state", "page_size", "cursor"],
            false,
        ),
        ("/lifecycle/records/{record_id}", "LifecycleRecordDetail", &["as_of"], true),
        (
            "/lifecycle/records/{record_id}/history",
            "LifecycleHistoryPage",
            &["page_size", "cursor"],
            false,
        ),
        (
            "/lifecycle/queue",
            "LifecycleQueuePage",
            &["as_of", "owner", "page_size", "cursor"],
            true,
        ),
        (
            "/framework-impact/comparisons",
            "FrameworkImpactComparisonPage",
            &["page_size", "cursor"],
            false,
        ),
        (
            "/framework-impact/comparisons/{comparison_id}",
            "FrameworkImpactComparisonDetail",
            &[],
            false,
        ),
        (
            "/framework-impact/comparisons/{comparison_id}/changes",
            "FrameworkImpactChangePage",
            &["change_class", "page_size", "cursor"],
            false,
        ),
        (
            "/framework-impact/comparisons/{comparison_id}/findings",
            "FrameworkImpactFindingPage",
            &[
                "group",
                "decision_state",
                "policy_source",
                "priority",
                "owner",
                "page_size",
                "cursor",
            ],
            false,
        ),
        (
            "/framework-impact/comparisons/{comparison_id}/prior-dispositions",
            "FrameworkImpactPriorDispositionPage",
            &["page_size", "cursor"],
            false,
        ),
    ];
    for (suffix, schema, keys, required_date) in routes {
        assert!(one["paths"].get(format!("/api/v1{suffix}")).is_none());
        let operation = &two["paths"][format!("/api/v2{suffix}")]["get"];
        assert_eq!(operation["security"], json!([{"capabilityBearer": []}]));
        assert_eq!(
            operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"],
            format!("#/components/schemas/{schema}")
        );
        assert!(operation["responses"].get("503").is_some());
        let parameters = operation["parameters"].as_array().unwrap();
        let resolved: Vec<&Value> = parameters
            .iter()
            .map(|value| {
                value.get("$ref").map_or(value, |reference| {
                    let name = reference
                        .as_str()
                        .unwrap()
                        .strip_prefix("#/components/parameters/")
                        .unwrap();
                    &two["components"]["parameters"][name]
                })
            })
            .filter(|value| value["in"] == "query")
            .collect();
        let actual: Vec<&str> =
            resolved.iter().map(|value| value["name"].as_str().unwrap()).collect();
        assert_eq!(actual, keys);
        if let Some(date) = resolved.iter().find(|value| value["name"] == "as_of") {
            assert_eq!(date["required"], required_date);
            assert_eq!(date["schema"]["maxLength"], 10);
        }
        let dto = &two["components"]["schemas"][schema];
        assert_eq!(dto["additionalProperties"], false);
        assert!(dto["required"].as_array().unwrap().contains(&json!("snapshot_version")));
        let _compiled = component_validator(&two, schema);
    }
}

/// Admit the retryable inspection stop envelope only in the selected API2 contract.
#[test]
fn lifecycle_impact_stop_errors_preserve_the_api1_boundary() {
    let one = load_openapi();
    let two = load_yaml_as_json(&repo_path("docs/api/forge-workspace-v2.openapi.yaml"));
    let old_error = component_validator(&one, "Error");
    let new_error = component_validator(&two, "Error");
    let mut sample =
        load_yaml_as_json(&repo_path("docs/api/fixtures-v2/error/unauthorized-401.json"));
    for code in ["query-budget-exceeded", "query-interrupted"] {
        sample["code"] = json!(code);
        sample["retryable"] = json!(true);
        assert!(new_error.is_valid(&sample));
        assert!(!old_error.is_valid(&sample));
    }
}

/// Preserve actual native recorded dates while explicit query dates retain their narrower grammar.
#[test]
fn lifecycle_recorded_date_schema_accepts_native_negative_and_expanded_years() {
    let two = load_yaml_as_json(&repo_path("docs/api/forge-workspace-v2.openapi.yaml"));
    let recorded = component_validator(&two, "S3RecordedDate");
    let query = component_validator(&two, "S3Date");
    for year in [-10_000, -1, 1, 10_000] {
        let actual = chrono::NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
        let value = serde_json::to_value(actual).unwrap();
        assert!(recorded.is_valid(&value), "native recorded date excluded: {value}");
        if !(0..=9999).contains(&year) {
            assert!(!query.is_valid(&value), "expanded recorded year became a query date");
        }
    }
    assert!(!recorded.is_valid(&json!("-000001-01-01-extra")));
}
