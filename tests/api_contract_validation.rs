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
use serde_json::{Map, Number, Value};
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
    serde_yaml::from_str(&text)
        .unwrap_or_else(|error| panic!("parse {} as YAML: {error}", path.display()))
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
                let resolved = match parameter.get("$ref").and_then(Value::as_str) {
                    Some(target) => document
                        .pointer(target.strip_prefix('#').unwrap_or(target))
                        .unwrap_or_else(|| panic!("unresolved parameter ref {target}")),
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
        assert!(url.starts_with("http://127.0.0.1"), "server url {url} must be loopback-only");
        assert!(
            !url.contains("localhost"),
            "server url {url} must use the literal 127.0.0.1, not a DNS name"
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
    for required in ["schema_version", "resources"] {
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
                // An invalid-expectation fixture may legitimately fail strict parsing.
                if expectation == "invalid" {
                    kinds.entry(kind.to_string()).or_insert(0);
                    listed_files.insert(file.to_string());
                    continue;
                }
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
