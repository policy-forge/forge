//! Bounded modern MCP framing and complete response-envelope serialization.
//!
//! Parsing admits only literal JSON trees and preserves exact integer/string IDs.
//! All source and approval authority stays in the separate captured query scope.

use std::cell::Cell;
use std::fmt;
use std::io::{self, Write};

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json::{Map, Number, Value};

/// Selected immutable primary protocol revision; no session negotiation exists.
pub(super) const REVISION: &str = "2026-07-28";
/// Complete input record, including its terminating LF.
pub(super) const MAX_FRAME: usize = 64 * 1024;
/// Complete output envelope, including its terminating LF.
pub(super) const MAX_RESPONSE: usize = 256 * 1024;
/// Complete strict JSON tree depth, with the root counted as one.
const MAX_DEPTH: usize = 64;
/// Values and object keys charged before tree insertion.
const MAX_NODES: usize = 100_000;
/// Exact string request-ID byte domain, separate from project keys.
const MAX_ID: usize = 128;
/// Bounded request method spelling; unknown valid methods remain protocol errors.
const MAX_METHOD: usize = 256;
/// Mandatory namespaced per-request selected protocol metadata.
const VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";
/// Mandatory per-request client capabilities; never cached as authority.
const CAPABILITIES_KEY: &str = "io.modelcontextprotocol/clientCapabilities";

/// Exact ID equality, with canonical nonnegative integer representation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(untagged)]
pub(super) enum RequestId {
    /// Signed negative integer within the exact i64 domain.
    Signed(i64),
    /// Nonnegative integer within the exact u64 domain.
    Unsigned(u64),
    /// Exact bounded UTF-8 string; numeric-looking strings remain strings.
    Text(String),
}

impl RequestId {
    /// Refuse fractional/exponent/f64/null/bool IDs without rounding or coercion.
    fn admit(value: &Value) -> Option<Self> {
        match value {
            Value::String(value) if value.len() <= MAX_ID => Some(Self::Text(value.clone())),
            Value::Number(number) if number.is_u64() => number.as_u64().map(Self::Unsigned),
            Value::Number(number) if number.is_i64() => number.as_i64().map(Self::Signed),
            _ => None,
        }
    }
}

/// Fixed protocol failures; no serde/native/private error strings are emitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Fault {
    /// Invalid JSON, UTF-8, duplicate keys, BOM, depth, nodes, or frame syntax.
    Parse,
    /// A JSON value does not form an admitted request envelope/ID.
    Request,
    /// Method has no consumed implementation in this server profile.
    Method,
    /// Missing or invalid request metadata, tool arguments, or tool name.
    Params,
    /// Internal bounded pipeline/serializer failure without private details.
    Internal,
    /// Exact unsupported revision, with the prescribed fixed data object.
    Version,
    /// All accepted queue slots or complete retained request IDs are occupied.
    Capacity,
    /// The original accepted absolute deadline elapsed before a static response.
    Deadline,
}

impl Fault {
    /// Return the literal standard/spec code or explicitly local server error.
    pub(super) const fn code(self) -> i32 {
        match self {
            Self::Parse => -32700,
            Self::Request => -32600,
            Self::Method => -32601,
            Self::Params => -32602,
            Self::Internal => -32603,
            Self::Version => -32022,
            Self::Capacity => -32000,
            Self::Deadline => -32001,
        }
    }
    /// Return only fixed redacted wire text.
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::Parse => "Parse error",
            Self::Request => "Invalid request",
            Self::Method => "Method not found",
            Self::Params => "Invalid params",
            Self::Internal => "Internal error",
            Self::Version => "Unsupported protocol version",
            Self::Capacity => "Bounded request capacity unavailable",
            Self::Deadline => "Accepted request deadline exceeded",
        }
    }
}

/// An admitted request; all metadata was validated independently for this frame.
pub(super) struct Request {
    /// Exact previously unseen correlation identity.
    pub(super) id: RequestId,
    /// Original bounded method spelling.
    pub(super) method: String,
    /// Owned bounded params including inert metadata, with no capture authority.
    pub(super) params: Map<String, Value>,
}

/// Only supported notification and admitted request are actionable.
pub(super) enum Message {
    /// Complete protocol request whose method-specific admission remains necessary.
    Request(Request),
    /// Exact prior-ID cancellation; its free-form reason is intentionally discarded.
    Cancel(RequestId),
    /// Invalid or unsupported notification; never send a notification response.
    Ignore,
}

/// Failure carries an ID only when its exact representation was safely admitted.
pub(super) struct Rejection {
    /// Fixed classification for a complete response.
    pub(super) fault: Fault,
    /// Optional exact ID, omitted rather than manufactured as null.
    pub(super) id: Option<RequestId>,
    /// Bounded original requested revision, only for the normative version error.
    pub(super) requested: Option<String>,
}

/// Frame state retains at most MAX_FRAME-1 payload bytes, including malformed input.
#[derive(Default)]
pub(super) struct Framer {
    /// Actual retained current raw payload, not an unbounded `read_line` buffer.
    bytes: Vec<u8>,
    /// Oversized record is drained to LF without storing any further payload.
    oversized: bool,
}

impl Framer {
    /// Feed one byte with admission before growth; LF completes exactly one record.
    pub(super) fn push(&mut self, byte: u8) -> Option<Result<Vec<u8>, Fault>> {
        if byte == b'\n' {
            let oversized = self.oversized;
            self.oversized = false;
            let bytes = std::mem::take(&mut self.bytes);
            return Some(if oversized { Err(Fault::Parse) } else { Ok(bytes) });
        }
        if !self.oversized {
            if self.bytes.len() >= MAX_FRAME - 1 {
                self.bytes.clear();
                self.oversized = true;
            } else {
                self.bytes.push(byte);
            }
        }
        None
    }
    /// EOF/shutdown releases unfinished input without inventing a complete frame.
    pub(super) fn clear(&mut self) {
        self.bytes.clear();
        self.oversized = false;
    }
}

/// Parse exactly one duplicate-free bounded object, never a JSON-RPC batch.
pub(super) fn parse(bytes: &[u8]) -> Result<Message, Rejection> {
    let fail = |fault| Rejection { fault, id: None, requested: None };
    if bytes.len() >= MAX_FRAME || bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(fail(Fault::Parse));
    }
    let value = strict_value(bytes, MAX_FRAME - 1).map_err(fail)?;
    let Value::Object(mut envelope) = value else {
        return Err(fail(Fault::Request));
    };
    let notification = !envelope.contains_key("id");
    let id = envelope.get("id").and_then(RequestId::admit);
    let reject = |fault| Rejection { fault, id: id.clone(), requested: None };
    if envelope.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || envelope
            .keys()
            .any(|key| !matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params"))
    {
        return if notification { Ok(Message::Ignore) } else { Err(reject(Fault::Request)) };
    }
    let Some(method) = envelope.get("method").and_then(Value::as_str) else {
        return if notification { Ok(Message::Ignore) } else { Err(reject(Fault::Request)) };
    };
    if method.is_empty() || method.len() > MAX_METHOD {
        return if notification { Ok(Message::Ignore) } else { Err(reject(Fault::Request)) };
    }
    let method = method.to_owned();
    let params = envelope.remove("params").unwrap_or_else(|| Value::Object(Map::new()));
    let Value::Object(params) = params else {
        return if notification { Ok(Message::Ignore) } else { Err(reject(Fault::Params)) };
    };
    if notification {
        if method != "notifications/cancelled"
            || params.keys().any(|key| !matches!(key.as_str(), "requestId" | "reason" | "_meta"))
            || params.get("reason").is_some_and(|value| !value.is_string())
            || params.get("_meta").is_some_and(|value| !value.as_object().is_some_and(valid_meta))
        {
            return Ok(Message::Ignore);
        }
        return Ok(params
            .get("requestId")
            .and_then(RequestId::admit)
            .map_or(Message::Ignore, Message::Cancel));
    }
    let Some(id) = id.clone() else {
        return Err(reject(Fault::Request));
    };
    let Some(meta) = params.get("_meta").and_then(Value::as_object) else {
        return Err(reject(Fault::Params));
    };
    if !valid_meta(meta) || !meta.get(CAPABILITIES_KEY).is_some_and(valid_capabilities) {
        return Err(reject(Fault::Params));
    }
    let Some(revision) = meta.get(VERSION_KEY).and_then(Value::as_str) else {
        return Err(reject(Fault::Params));
    };
    if revision.len() > 128 {
        return Err(reject(Fault::Params));
    }
    if revision != REVISION {
        return Err(Rejection {
            fault: Fault::Version,
            id: Some(id),
            requested: Some(revision.to_owned()),
        });
    }
    Ok(Message::Request(Request { id, method, params }))
}

/// Decode a bounded original/encoded wire tree without duplicate replacement.
/// The byte limit is admitted before parsing; nodes/depth precede derived growth.
pub(super) fn strict_value(bytes: &[u8], limit: usize) -> Result<Value, Fault> {
    if bytes.len() > limit || bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(Fault::Parse);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Fault::Parse)?;
    let nodes = Cell::new(0);
    let mut decoder = serde_json::Deserializer::from_str(text);
    let value =
        Seed { nodes: &nodes, depth: 1 }.deserialize(&mut decoder).map_err(|_| Fault::Parse)?;
    decoder.end().map_err(|_| Fault::Parse)?;
    Ok(value)
}

/// Bound and validate mandatory/known capability shapes while preserving extensions.
fn valid_capabilities(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.iter().all(|(key, value)| match key.as_str() {
        "experimental" => {
            value.as_object().is_some_and(|fields| fields.values().all(Value::is_object))
        }
        "roots" => value.is_object(),
        "sampling" => value.as_object().is_some_and(|fields| {
            fields.iter().all(|(name, value)| {
                !matches!(name.as_str(), "context" | "tools") || value.is_object()
            })
        }),
        "elicitation" => value.as_object().is_some_and(|fields| {
            fields
                .iter()
                .all(|(name, value)| !matches!(name.as_str(), "form" | "url") || value.is_object())
        }),
        "extensions" => value.as_object().is_some_and(|fields| {
            fields.iter().all(|(name, value)| meta_key(name, true) && value.is_object())
        }),
        _ => true,
    })
}

/// Preserve inert extension data while validating known namespaced metadata fields.
fn valid_meta(meta: &Map<String, Value>) -> bool {
    meta.iter().all(|(key, value)| {
        if !meta_key(key, false) {
            return false;
        }
        match key.as_str() {
            VERSION_KEY => value.is_string(),
            CAPABILITIES_KEY => valid_capabilities(value),
            "io.modelcontextprotocol/clientInfo" => value.as_object().is_some_and(|info| {
                info.get("name").is_some_and(Value::is_string)
                    && info.get("version").is_some_and(Value::is_string)
            }),
            "progressToken" => matches!(value, Value::String(_) | Value::Number(_)),
            "io.modelcontextprotocol/logLevel" => value.as_str().is_some_and(|level| {
                matches!(
                    level,
                    "debug"
                        | "info"
                        | "notice"
                        | "warning"
                        | "error"
                        | "critical"
                        | "alert"
                        | "emergency"
                )
            }),
            _ => true,
        }
    })
}

/// Apply literal primary-schema namespaced metadata syntax without interpreting values.
fn meta_key(key: &str, mandatory_prefix: bool) -> bool {
    let (prefix, suffix) = match key.split_once('/') {
        Some((prefix, suffix)) => (Some(prefix), suffix),
        None if !mandatory_prefix => (None, key),
        None => return false,
    };
    if !suffix.is_empty()
        && (!suffix.as_bytes()[0].is_ascii_alphanumeric()
            || !suffix.as_bytes()[suffix.len() - 1].is_ascii_alphanumeric()
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')))
    {
        return false;
    }
    prefix.is_none_or(|prefix| {
        let mut reserved = false;
        for (index, label) in prefix.split('.').enumerate() {
            if label.is_empty()
                || !label.as_bytes()[0].is_ascii_alphabetic()
                || !label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                || !label.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return false;
            }
            if index == 1 && matches!(label, "mcp" | "modelcontextprotocol") {
                reserved = true;
            }
        }
        !reserved || prefix == "io.modelcontextprotocol"
    })
}

/// Monotonic tree seed; every node is charged before its container insertion.
struct Seed<'a> {
    /// Complete keys/value denominator for this one admitted frame.
    nodes: &'a Cell<usize>,
    /// Actual traversal depth, independently limited before recursive parsing.
    depth: usize,
}

impl Seed<'_> {
    /// Refuse the next node before recursive visitor/container growth.
    fn charge<E: de::Error>(&self) -> Result<(), E> {
        let next = self.nodes.get().checked_add(1).ok_or_else(|| E::custom("bounded JSON"))?;
        if next > MAX_NODES || self.depth > MAX_DEPTH {
            return Err(E::custom("bounded JSON"));
        }
        self.nodes.set(next);
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    /// Parse through the strict duplicate/depth/node visitor, not a default Value decoder.
    fn deserialize<D: de::Deserializer<'de>>(self, decoder: D) -> Result<Value, D::Error> {
        self.charge()?;
        decoder.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    /// Supply fixed diagnostics that are never copied into public errors.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded JSON")
    }
    /// Preserve null without manufacturing a request identifier.
    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    /// Preserve an actual admitted boolean node.
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Value, E> {
        Ok(Value::Bool(value))
    }
    /// Preserve the exact signed integer representation.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    /// Preserve the exact unsigned integer representation.
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Number(value.into()))
    }
    /// Inert metadata may contain finite floats; request-ID admission refuses this representation.
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Value, E> {
        Number::from_f64(value).map(Value::Number).ok_or_else(|| E::custom("bounded JSON"))
    }
    /// The enclosing raw frame bounds a decoded string before its retained copy.
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
        Ok(Value::String(value.to_owned()))
    }
    /// Move an escaped string already bounded by the original raw frame.
    fn visit_string<E: de::Error>(self, value: String) -> Result<Value, E> {
        Ok(Value::String(value))
    }
    /// Charge and parse each complete child before appending it to the bounded array.
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Value, A::Error> {
        let mut array = Vec::new();
        while let Some(value) =
            sequence.next_element_seed(Seed { nodes: self.nodes, depth: self.depth + 1 })?
        {
            array.push(value);
        }
        Ok(Value::Array(array))
    }
    /// Refuse duplicate keys before replacement and precharge every key and child.
    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Value, A::Error> {
        let mut map = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            self.charge()?;
            if map.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON key"));
            }
            let value =
                object.next_value_seed(Seed { nodes: self.nodes, depth: self.depth + 1 })?;
            map.insert(key, value);
        }
        Ok(Value::Object(map))
    }
}

/// Complete fixed or captured-success JSON-RPC envelope.
#[derive(Serialize)]
struct Success<'a, T: Serialize> {
    /// Literal protocol envelope revision.
    jsonrpc: &'static str,
    /// Exact original correlation identity.
    id: &'a RequestId,
    /// Complete borrowed result; never a clipped trusted prefix.
    result: &'a T,
}

/// Fixed exception envelope with an optional actual admitted ID.
#[derive(Serialize)]
struct ErrorResponse<'a> {
    /// Literal JSON-RPC envelope version.
    jsonrpc: &'static str,
    /// Omitted when no exact ID was admitted; current primary schema permits absence.
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<&'a RequestId>,
    /// Fixed failure fields and only normative bounded unsupported-version data.
    error: ErrorBody<'a>,
}

/// Fixed protocol error payload; no project/native diagnostics are accepted.
#[derive(Serialize)]
struct ErrorBody<'a> {
    /// Exact standard/spec or declared local capacity/deadline code.
    code: i32,
    /// Fixed redacted human-readable text.
    message: &'static str,
    /// Required for -32022 only, absent on every other failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<VersionData<'a>>,
}

/// Exact current-schema unsupported-version data shape.
#[derive(Serialize)]
struct VersionData<'a> {
    /// Single consumed modern revision; no historical handshake advertisement.
    supported: [&'static str; 1],
    /// Bounded caller spelling, without fallback or silent negotiation.
    requested: &'a str,
}

/// Encode a complete success including LF before any publication.
pub(super) fn success(id: &RequestId, result: &impl Serialize) -> io::Result<Vec<u8>> {
    encode(&Success { jsonrpc: "2.0", id, result })
}

/// Encode one complete safe failure; invalid notifications never call this function.
pub(super) fn error(rejection: &Rejection) -> io::Result<Vec<u8>> {
    let data = if rejection.fault == Fault::Version {
        Some(VersionData {
            supported: [REVISION],
            requested: rejection.requested.as_deref().ok_or_else(invalid_encoding)?,
        })
    } else {
        None
    };
    encode(&ErrorResponse {
        jsonrpc: "2.0",
        id: rejection.id.as_ref(),
        error: ErrorBody { code: rejection.fault.code(), message: rejection.fault.message(), data },
    })
}

/// Nonretaining full-envelope precharge precedes every retained encoded byte.
fn encode(value: &impl Serialize) -> io::Result<Vec<u8>> {
    let mut count = Counter { used: 0 };
    serde_json::to_writer(&mut count, value).map_err(|_| invalid_encoding())?;
    let complete = count
        .used
        .checked_add(1)
        .filter(|size| *size <= MAX_RESPONSE)
        .ok_or_else(invalid_encoding)?;
    let mut retained = Buffer { bytes: Vec::with_capacity(complete), limit: complete - 1 };
    serde_json::to_writer(&mut retained, value).map_err(|_| invalid_encoding())?;
    if retained.bytes.len() != complete - 1 {
        return Err(invalid_encoding());
    }
    retained.bytes.push(b'\n');
    Ok(retained.bytes)
}

/// Full nonretaining serialized byte count; MAX includes the later LF.
struct Counter {
    /// Complete emitted byte count, checked before addition.
    used: usize,
}
impl Write for Counter {
    /// Charge each writer chunk before accepting it, with no retained payload.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.used = self
            .used
            .checked_add(bytes.len())
            .filter(|size| *size < MAX_RESPONSE)
            .ok_or_else(invalid_encoding)?;
        Ok(bytes.len())
    }
    /// Counting emits no external effect and needs no transport flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Actual complete response storage grows only inside its admitted exact count.
struct Buffer {
    /// Retained encoded bytes, never a decoded copied private scope.
    bytes: Vec<u8>,
    /// Exact nonretaining admitted count for this one envelope.
    limit: usize,
}
impl Write for Buffer {
    /// Refuse a complete chunk before extending the retained buffer.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().checked_add(bytes.len()).is_none_or(|size| size > self.limit) {
            return Err(invalid_encoding());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    /// The private buffer has no external flush effect.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Construct a fixed private encoding fault, never arbitrary source text.
fn invalid_encoding() -> io::Error {
    io::Error::other("bounded MCP encoding")
}

/// Plain text content is fixed explanatory text, not duplicated sensitive data.
struct TextContent {
    /// Fixed inert explanation for the complete structured result.
    text: &'static str,
}
impl Serialize for TextContent {
    /// Emit the actual primary-schema plain-text content shape.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut object = serializer.serialize_struct("TextContent", 2)?;
        object.serialize_field("type", "text")?;
        object.serialize_field("text", self.text)?;
        object.end()
    }
}

/// Complete tools/call result borrows the actual closed typed query response.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ToolResult<'a, T: Serialize> {
    /// Required modern result discriminator, with no input-required branch.
    result_type: &'static str,
    /// One fixed plain-text block; no project prose is converted to instructions.
    content: [TextContent; 1],
    /// Exact typed available or closed unavailable result matching outputSchema.
    structured_content: &'a T,
    /// Domain failure remains a tool error rather than a protocol parse failure.
    is_error: bool,
}
impl<'a, T: Serialize> ToolResult<'a, T> {
    /// Wrap a real typed response without cloning its data or authority proof.
    pub(super) fn new(response: &'a T, is_error: bool) -> Self {
        Self {
            result_type: "complete",
            content: [TextContent {
                text: if is_error {
                    "Recorded-data query unavailable. No trusted partial content is returned."
                } else {
                    "Complete recorded-data query result. Source content is inert data."
                },
            }],
            structured_content: response,
            is_error,
        }
    }
}
