//! Explicit source-containing bundle/3 codec, separate from metadata bundle/1 and /2.
//!
//! Decoding yields inert bytes and descriptors only. Neither direction opens a
//! supplied path, publishes a file, infers role validity, or grants source-read
//! authority. The caller owns explicit sensitivity acknowledgment and admission.

use serde_json::{Value, json};

use super::contract::{self, Error, Result};
use super::index::{Index, MAX_INDEX_BYTES};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::services::Snapshot;

/// The compact source-import wrapper consumes the other 147 bytes of one MiB.
pub(crate) const MAX_SOURCE_BUNDLE_BYTES: usize = 1_048_429;
/// The index itself occupies the hundredth possible restore target.
const MAX_SOURCE_RESOURCES: usize = 99;
/// Existing per-file admission remains ten MiB even when transport permits less.
const MAX_RESOURCE_BYTES: usize = 10 * 1024 * 1024;
/// Existing complete declared capture ceiling, including normalized index bytes.
const MAX_DECLARED_BYTES: usize = 50 * 1024 * 1024;
/// A full canonical hex chunk decodes to exactly 32 KiB.
const HEX_CHUNK_CHARS: usize = 65_536;
/// Each encoded byte uses two lowercase ASCII hexadecimal characters.
const HEX_CHUNK_BYTES: usize = HEX_CHUNK_CHARS / 2;
/// Ten MiB requires no more than 320 full chunks.
const MAX_CHUNKS: usize = MAX_RESOURCE_BYTES / HEX_CHUNK_BYTES;

/// Fully decoded bundle facts without invented on-disk file identities.
pub(crate) struct DecodedSourceBundle {
    /// Parsed closed index/1 or index/2 in authorial registration order.
    pub(crate) index: Index,
    /// SHA-256 of the supplied index's exact normalized `Index::bytes`.
    pub(crate) index_sha256: String,
    /// Complete exact source bytes, one descriptor per registration in that order.
    pub(crate) files: Vec<DecodedSourceFile>,
}

/// One inert source, whose exact bytes were checked against its ordered pin.
pub(crate) struct DecodedSourceFile {
    /// The index key; role and confined relative path remain in the index.
    pub(crate) key: String,
    /// Lowercase SHA-256 independently verified against these original bytes.
    pub(crate) sha256: String,
    /// Exact decoded byte count, not encoded characters or allocated capacity.
    pub(crate) size_bytes: usize,
    /// Original binary bytes without UTF-8, BOM, newline or domain normalization.
    pub(crate) bytes: Vec<u8>,
}

/// Borrowed, fully preflighted content; no decoded source buffer exists yet.
struct SourcePlan<'a> {
    /// Complete key already matched to the same-position index registration.
    key: String,
    /// Canonical expected hash, checked again after decoding.
    sha256: String,
    /// Exact total decoded length checked before source buffer reservation.
    size: usize,
    /// Canonical bounded chunks borrowed from the strict parsed JSON tree.
    chunks: &'a [Value],
}

/// Complete cross-field admission before allocating any decoded source buffer.
struct DecodePlan<'a> {
    /// Validated independent existing index grammar.
    index: Index,
    /// Normalized index hash, already independently recomputed.
    index_sha256: String,
    /// All rows admitted together; no prefix can escape as a decoded bundle.
    files: Vec<SourcePlan<'a>>,
}

/// Return a fixed safe capacity error without leaking source or decoder details.
fn oversized() -> Error {
    Error::new("payload-too-large", "The source bundle exceeds its size limit.", false)
}

/// Admit exactly the named object members, requiring every member and no extras.
fn closed_object<'a>(
    value: &'a Value,
    names: &[&str],
) -> Result<&'a serde_json::Map<String, Value>> {
    let object = value.as_object().ok_or_else(Error::invalid)?;
    if object.len() != names.len() || names.iter().any(|name| !object.contains_key(*name)) {
        return Err(Error::invalid());
    }
    Ok(object)
}

/// Check the canonical wire spelling of a SHA-256 digest without normalization.
fn canonical_hash(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Admit the existing strict index before permitting any source descriptor allocation.
fn decode_index(value: &Value) -> Result<Index> {
    if value["resources"].as_array().ok_or_else(Error::invalid)?.len() > MAX_SOURCE_RESOURCES {
        return Err(Error::invalid());
    }
    Index::parse(&contract::encode(value, MAX_INDEX_BYTES, false)?)
}

/// Check canonical chunk lengths, characters and exact declared decoded size.
/// Empty bytes require zero chunks; every non-final chunk must be full width.
fn admit_chunks(chunks: &[Value], declared: usize) -> Result<()> {
    if chunks.len() > MAX_CHUNKS {
        return Err(Error::invalid());
    }
    let mut decoded = 0_usize;
    for (position, value) in chunks.iter().enumerate() {
        let chunk = value.as_str().ok_or_else(Error::invalid)?;
        if chunk.is_empty()
            || chunk.len() > HEX_CHUNK_CHARS
            || chunk.len() % 2 != 0
            || (position + 1 < chunks.len() && chunk.len() != HEX_CHUNK_CHARS)
            || !chunk.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Error::invalid());
        }
        decoded = decoded.checked_add(chunk.len() / 2).ok_or_else(oversized)?;
        if decoded > MAX_RESOURCE_BYTES {
            return Err(oversized());
        }
    }
    if decoded != declared {
        return Err(Error::invalid());
    }
    Ok(())
}

/// Validate every ordered pin/content row and the complete aggregate before decoding.
fn preflight<'a>(value: &'a Value, control: &mut dyn WorkControl) -> WorkResult<DecodePlan<'a>> {
    closed_object(
        value,
        &[
            "schema_version",
            "profile",
            "source_content_included",
            "index",
            "index_sha256",
            "pins",
            "contents",
        ],
    )?;
    if value["schema_version"] != "forge.workspace-index-bundle/3"
        || value["profile"] != "index-and-source-hex"
        || value["source_content_included"] != true
    {
        return Err(Error::invalid().into());
    }
    let index = decode_index(&value["index"])?;
    let normalized = index.bytes()?;
    let index_sha256 = value["index_sha256"].as_str().ok_or_else(Error::invalid)?;
    if !canonical_hash(index_sha256) || index_sha256 != crate::hashing::sha256_hex(&normalized) {
        return Err(Error::invalid().into());
    }
    let pins = value["pins"].as_array().ok_or_else(Error::invalid)?;
    let contents = value["contents"].as_array().ok_or_else(Error::invalid)?;
    if pins.len() != index.resources.len() || contents.len() != pins.len() {
        return Err(Error::invalid().into());
    }
    let mut total = normalized.len();
    let mut files = Vec::new();
    files.try_reserve_exact(pins.len()).map_err(|_| oversized())?;
    for ((resource, pin), content) in index.resources.iter().zip(pins).zip(contents) {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        closed_object(pin, &["key", "sha256", "size"])?;
        closed_object(content, &["key", "encoding", "chunks"])?;
        let sha256 = pin["sha256"].as_str().ok_or_else(Error::invalid)?;
        let size = pin["size"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or_else(Error::invalid)?;
        if pin["key"] != resource.key
            || content["key"] != resource.key
            || content["encoding"] != "hex"
            || !canonical_hash(sha256)
            || size > MAX_RESOURCE_BYTES
        {
            return Err(Error::invalid().into());
        }
        let chunks = content["chunks"].as_array().ok_or_else(Error::invalid)?;
        admit_chunks(chunks, size)?;
        total = total.checked_add(size).ok_or_else(oversized)?;
        if total > MAX_DECLARED_BYTES {
            return Err(oversized().into());
        }
        files.push(SourcePlan { key: resource.key.clone(), sha256: sha256.into(), size, chunks });
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    }
    Ok(DecodePlan { index, index_sha256: index_sha256.into(), files })
}

/// Convert one previously admitted lowercase ASCII nibble defensively.
fn nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(Error::invalid()),
    }
}

/// Decode one complete admitted row and verify the hash before exposing its bytes.
fn decode_file(
    plan: SourcePlan<'_>,
    control: &mut dyn WorkControl,
) -> WorkResult<DecodedSourceFile> {
    control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(plan.size).map_err(|_| oversized())?;
    for value in plan.chunks {
        let chunk = value.as_str().ok_or_else(Error::invalid)?;
        for pair in chunk.as_bytes().chunks_exact(2) {
            bytes.push((nibble(pair[0])? << 4) | nibble(pair[1])?);
        }
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
    }
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let sha256 = crate::hashing::sha256_hex(&bytes);
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if bytes.len() != plan.size || sha256 != plan.sha256 {
        return Err(Error::invalid().into());
    }
    Ok(DecodedSourceFile { key: plan.key, sha256, size_bytes: plan.size, bytes })
}

/// Decode strict raw JSON before any caller can read a supplied destination.
///
/// Duplicate decoded keys, BOM, malformed UTF-8, multiple documents, noncanonical
/// hex and all cross-field mismatches reject the whole bundle. Decoded bytes are
/// inert and still require role and complete proposed-closure admission by the caller.
pub(crate) fn decode(raw: &[u8], control: &mut dyn WorkControl) -> WorkResult<DecodedSourceBundle> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let value = contract::parse(raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let plan = preflight(&value, control)?;
    let mut files = Vec::new();
    files.try_reserve_exact(plan.files.len()).map_err(|_| oversized())?;
    for file in plan.files {
        files.push(decode_file(file, control)?);
    }
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    Ok(DecodedSourceBundle { index: plan.index, index_sha256: plan.index_sha256, files })
}

/// Build a small empty-content wire skeleton from exact ordered captured observations.
/// This checks complete capture consistency without claiming native validity or freshness.
fn export_skeleton(snapshot: &Snapshot, control: &mut dyn WorkControl) -> WorkResult<Value> {
    if !snapshot.index_present {
        return Err(
            Error::new("not-found", "The workspace resource index is not present.", false).into()
        );
    }
    if snapshot.index.resources.len() > MAX_SOURCE_RESOURCES
        || snapshot.items.len() != snapshot.index.resources.len()
    {
        return Err(Error::invalid().into());
    }
    let normalized = snapshot.index.bytes()?;
    let mut total = normalized.len();
    let mut pins = Vec::new();
    let mut contents = Vec::new();
    pins.try_reserve_exact(snapshot.items.len()).map_err(|_| oversized())?;
    contents.try_reserve_exact(snapshot.items.len()).map_err(|_| oversized())?;
    for (registration, item) in snapshot.index.resources.iter().zip(&snapshot.items) {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        if registration.key != item.registration.key
            || registration.role != item.registration.role
            || registration.path != item.registration.path
            || item.captured.bytes.len() > MAX_RESOURCE_BYTES
            || item.captured.sha256 != crate::hashing::sha256_hex(&item.captured.bytes)
        {
            return Err(Error::invalid().into());
        }
        total = total.checked_add(item.captured.bytes.len()).ok_or_else(oversized)?;
        if total > MAX_DECLARED_BYTES {
            return Err(oversized().into());
        }
        pins.push(json!({"key":registration.key, "sha256":item.captured.sha256, "size":item.captured.bytes.len()}));
        contents.push(json!({"key":registration.key, "encoding":"hex", "chunks":[]}));
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    }
    Ok(json!({"schema_version":"forge.workspace-index-bundle/3", "profile":"index-and-source-hex",
        "source_content_included":true, "index":snapshot.index,
        "index_sha256":crate::hashing::sha256_hex(&normalized), "pins":pins, "contents":contents}))
}

/// Compute exact compact wire size before allocating source hex strings.
/// The empty-array skeleton already includes brackets; add only strings and commas.
fn encoded_size(skeleton: &Value, snapshot: &Snapshot) -> Result<usize> {
    let mut size = contract::encode(skeleton, MAX_SOURCE_BUNDLE_BYTES, false)?.len();
    for item in &snapshot.items {
        let bytes = item.captured.bytes.len();
        let chunks = bytes.div_ceil(HEX_CHUNK_BYTES);
        let extra = bytes
            .checked_mul(2)
            .and_then(|n| chunks.checked_mul(2).and_then(|quotes| n.checked_add(quotes)))
            .and_then(|n| n.checked_add(chunks.saturating_sub(1)))
            .ok_or_else(oversized)?;
        size = size.checked_add(extra).ok_or_else(oversized)?;
        if size > MAX_SOURCE_BUNDLE_BYTES {
            return Err(oversized());
        }
    }
    Ok(size)
}

/// Encode at most one canonical chunk using checked bounded string reservation.
fn hex_chunk(bytes: &[u8]) -> Result<String> {
    if bytes.is_empty() || bytes.len() > HEX_CHUNK_BYTES {
        return Err(Error::invalid());
    }
    let capacity = bytes.len().checked_mul(2).ok_or_else(oversized)?;
    let mut encoded = String::new();
    encoded.try_reserve_exact(capacity).map_err(|_| oversized())?;
    let digits = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(char::from(digits[usize::from(byte >> 4)]));
        encoded.push(char::from(digits[usize::from(byte & 15)]));
    }
    Ok(encoded)
}

/// Encode only the complete captured inventory; sensitivity acknowledgment is caller-owned.
///
/// No role content-reader permission, native validation, freshness, publication or
/// hidden resource discovery is inferred. The complete encoded artifact must fit
/// the smaller source-import transport ceiling; no source is truncated to fit.
pub(crate) fn encode(snapshot: &Snapshot, control: &mut dyn WorkControl) -> WorkResult<Vec<u8>> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    let mut value = export_skeleton(snapshot, control)?;
    let expected_size = encoded_size(&value, snapshot)?;
    for (position, item) in snapshot.items.iter().enumerate() {
        control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        let mut chunks = Vec::new();
        chunks
            .try_reserve_exact(item.captured.bytes.len().div_ceil(HEX_CHUNK_BYTES))
            .map_err(|_| oversized())?;
        for bytes in item.captured.bytes.chunks(HEX_CHUNK_BYTES) {
            chunks.push(Value::String(hex_chunk(bytes)?));
            control.checkpoint(Stage::CopyInputs, ProgressUpdate::Unchanged)?;
        }
        let content = value["contents"]
            .as_array_mut()
            .and_then(|contents| contents.get_mut(position))
            .ok_or_else(Error::invalid)?;
        *content.get_mut("chunks").ok_or_else(Error::invalid)? = Value::Array(chunks);
    }
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    let bytes = contract::encode(&value, MAX_SOURCE_BUNDLE_BYTES, false)?;
    if bytes.len() != expected_size {
        return Err(Error::invalid().into());
    }
    // Reuse the strict raw codec for complete output parity, never a filesystem read.
    decode(&bytes, control)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    Ok(bytes)
}

#[cfg(test)]
/// Codec controls use actual Root/Snapshot captures and never publish an effect.
mod tests {
    use super::super::contract::ApiMajor;
    use super::super::index::{Resource, Role};
    use super::super::preparation::{Interruption, NoopControl, WorkError, test_support::Recorder};
    use super::super::root::Root;
    use super::*;

    /// Capture exact fixture files through the real confined Snapshot entry point.
    fn fixture(version: &str, role: Role, sources: &[&[u8]]) -> (tempfile::TempDir, Snapshot) {
        let directory = tempfile::tempdir().unwrap();
        let resources = sources
            .iter()
            .enumerate()
            .map(|(position, bytes)| {
                let key = format!("input-{position}");
                let path =
                    format!("{key}.{}", if role == Role::PolicySource { "md" } else { "bin" });
                std::fs::write(directory.path().join(&path), bytes).unwrap();
                Resource { key, role, path }
            })
            .collect();
        let index = Index {
            schema_version: version.into(),
            label: "Exact source fixture".into(),
            resources,
        };
        std::fs::write(
            directory.path().join(super::super::index::INDEX_PATH),
            index.bytes().unwrap(),
        )
        .unwrap();
        let root = Root::open(directory.path()).unwrap();
        (directory, Snapshot::capture(&root).unwrap())
    }

    /// Obtain a complete actual encoder result for small malformed-wire controls.
    fn small_wire() -> Value {
        let (_directory, snapshot) =
            fixture("forge.workspace/2", Role::LifecycleSource, &[b"\0\xff\r\nA", b"second"]);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap()
    }

    /// Serialize a mutated raw control without invoking the decoder or repairing its facts.
    fn raw_wire(value: &Value) -> Vec<u8> {
        contract::encode(value, MAX_SOURCE_BUNDLE_BYTES, false).unwrap()
    }

    /// Require the existing safe invalid-request classification without printing source data.
    fn assert_invalid(raw: &[u8]) {
        match decode(raw, &mut NoopControl) {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "invalid-request"),
            _ => panic!("expected a complete safe invalid-request refusal"),
        }
    }

    /// Empty, all-byte, BOM/CRLF and a multi-chunk file preserve complete original bytes.
    #[test]
    fn binary_and_empty_sources_round_trip_in_exact_authorial_order() {
        let all_bytes: Vec<u8> = (0..=255).collect();
        let across_chunk = vec![0xa5; HEX_CHUNK_BYTES + 1];
        let sources: [&[u8]; 4] = [b"", &all_bytes, b"\xef\xbb\xbftext\r\n", &across_chunk];
        let (directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &sources);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        let decoded = decode(&raw, &mut NoopControl).unwrap();
        assert_eq!(
            decoded.index_sha256,
            crate::hashing::sha256_hex(&snapshot.index.bytes().unwrap())
        );
        assert_eq!(decoded.index.schema_version, "forge.workspace/2");
        for ((file, resource), expected) in
            decoded.files.iter().zip(&snapshot.index.resources).zip(sources)
        {
            assert_eq!(file.key, resource.key);
            assert_eq!(file.bytes, expected);
            assert_eq!(file.size_bytes, expected.len());
            assert_eq!(file.sha256, crate::hashing::sha256_hex(expected));
            assert_eq!(std::fs::read(directory.path().join(&resource.path)).unwrap(), expected);
        }
        let wire = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        assert_eq!(wire["contents"][0]["chunks"], json!([]));
        let chunks = wire["contents"][3]["chunks"].as_array().unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].as_str().unwrap().len(), HEX_CHUNK_CHARS);
        assert_eq!(chunks[1].as_str().unwrap().len(), 2);
    }

    /// Bundle/3 carries either existing index grammar without changing metadata route admission.
    #[test]
    fn existing_metadata_versions_stay_separate_from_source_profile() {
        for version in ["forge.workspace/1", "forge.workspace/2"] {
            let (_directory, snapshot) =
                fixture(version, Role::PolicySource, &[b"# Policy\n\nExact text.\n"]);
            let metadata =
                super::super::bundles::encode_metadata_for_api(&snapshot, ApiMajor::V2).unwrap();
            let value = contract::parse(&metadata, 1024 * 1024, HEX_CHUNK_CHARS).unwrap();
            assert!(super::super::bundles::decode_bundle_for_api(&value, ApiMajor::V2).is_ok());
            assert_invalid(&metadata);
            let source = encode(&snapshot, &mut NoopControl).unwrap();
            let source_value =
                contract::parse(&source, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
            assert!(
                super::super::bundles::decode_bundle_for_api(&source_value, ApiMajor::V2).is_err()
            );
            assert_eq!(decode(&source, &mut NoopControl).unwrap().index.schema_version, version);
        }
    }

    /// Source transfer preserves structurally capturable invalid input without conferring domain validity.
    #[test]
    fn captured_invalid_policy_can_export_without_becoming_import_ready() {
        let (_directory, snapshot) = fixture("forge.workspace/1", Role::PolicySource, &[b"\0\xff"]);
        assert_eq!(snapshot.items[0].metadata["validation_state"], "invalid");
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        assert_eq!(decode(&raw, &mut NoopControl).unwrap().files[0].bytes, b"\0\xff");
    }

    /// Strict raw parsing rejects duplicate decoded names, outer BOM, malformed UTF-8 and trailing documents.
    #[test]
    fn raw_parser_refuses_lossy_transport_inputs_before_descriptors() {
        let raw = raw_wire(&small_wire());
        let text = std::str::from_utf8(&raw).unwrap();
        for duplicate in
            [r#""profile":"index-and-source-hex", "#, r#""\u0070rofile":"index-and-source-hex", "#]
        {
            let broken = format!("{{{duplicate}{}", &text[1..]);
            assert_invalid(broken.as_bytes());
        }
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&raw);
        assert_invalid(&bom);
        assert_invalid(&[0xff]);
        let mut documents = raw;
        documents.extend_from_slice(b" {}");
        assert_invalid(&documents);
    }

    /// Unknown members and wrong version/profile/source opt-in are closed at every new wire level.
    #[test]
    fn closed_wire_fields_and_profile_are_required() {
        let original = small_wire();
        for pointer in ["", "/pins/0", "/contents/0"] {
            let mut wire = original.clone();
            wire.pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("extra".into(), json!(true));
            assert_invalid(&raw_wire(&wire));
        }
        for (field, value) in [
            ("schema_version", json!("forge.workspace-index-bundle/2")),
            ("profile", json!("index-and-hashes")),
            ("source_content_included", json!(false)),
        ] {
            let mut wire = original.clone();
            wire[field] = value;
            assert_invalid(&raw_wire(&wire));
        }
        for field in ["pins", "contents", "index_sha256"] {
            let mut wire = original.clone();
            wire.as_object_mut().unwrap().remove(field);
            assert_invalid(&raw_wire(&wire));
        }
    }

    /// Missing, extra, duplicated and reordered rows cannot attach source bytes to another registration.
    #[test]
    fn ordered_index_pin_content_bijection_is_complete() {
        let original = small_wire();
        for field in ["pins", "contents"] {
            let mut reversed = original.clone();
            reversed[field].as_array_mut().unwrap().reverse();
            assert_invalid(&raw_wire(&reversed));
            let mut missing = original.clone();
            missing[field].as_array_mut().unwrap().pop();
            assert_invalid(&raw_wire(&missing));
            let mut extra = original.clone();
            let first = extra[field][0].clone();
            extra[field].as_array_mut().unwrap().push(first);
            assert_invalid(&raw_wire(&extra));
            let mut repeated = original.clone();
            repeated[field][1]["key"] = repeated[field][0]["key"].clone();
            assert_invalid(&raw_wire(&repeated));
        }
        let mut wrong_key = original;
        wrong_key["contents"][0]["key"] = json!("foreign");
        assert_invalid(&raw_wire(&wrong_key));
    }

    /// Normalized index identity, source hash and byte length are independent mandatory bindings.
    #[test]
    fn hashes_sizes_and_normalized_index_identity_are_independent() {
        let original = small_wire();
        let mut compact_hash = original.clone();
        compact_hash["index_sha256"] = json!(crate::hashing::sha256_hex(
            &contract::encode(&original["index"], MAX_INDEX_BYTES, false).unwrap()
        ));
        assert_invalid(&raw_wire(&compact_hash));
        let mut source_hash = original.clone();
        source_hash["pins"][0]["sha256"] = json!("0".repeat(64));
        assert_invalid(&raw_wire(&source_hash));
        let mut upper_hash = original.clone();
        upper_hash["pins"][0]["sha256"] =
            json!(original["pins"][0]["sha256"].as_str().unwrap().to_uppercase());
        assert_invalid(&raw_wire(&upper_hash));
        for size in [json!(0), json!(6), json!(-1), json!(5.0), json!(true), json!(u64::MAX)] {
            let mut wire = original.clone();
            wire["pins"][0]["size"] = size;
            assert_invalid(&raw_wire(&wire));
        }
    }

    /// Equivalent noncanonical splits and uppercase spellings reject even when decoded bytes/hash agree.
    #[test]
    fn canonical_hex_width_and_spelling_are_not_repaired() {
        let (_directory, snapshot) = fixture(
            "forge.workspace/2",
            Role::LifecycleSource,
            &[&vec![0xab; HEX_CHUNK_BYTES + 1]],
        );
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        let original = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        let mut split = original.clone();
        let first = split["contents"][0]["chunks"][0].as_str().unwrap().to_owned();
        let second = split["contents"][0]["chunks"][1].as_str().unwrap().to_owned();
        split["contents"][0]["chunks"] =
            json!([&first[..first.len() - 2], format!("{}{second}", &first[first.len() - 2..])]);
        assert_invalid(&raw_wire(&split));
        let mut uppercase = original.clone();
        uppercase["contents"][0]["chunks"][0] = json!(first.to_uppercase());
        assert_invalid(&raw_wire(&uppercase));
        for chunks in [
            json!([""]),
            json!(["a"]),
            json!(["zz"]),
            json!(["ab".repeat(HEX_CHUNK_BYTES + 1)]),
            json!(vec!["ab"; MAX_CHUNKS + 1]),
        ] {
            let mut wire = original.clone();
            wire["contents"][0]["chunks"] = chunks;
            assert_invalid(&raw_wire(&wire));
        }
    }

    /// A zero-length pin cannot hide a redundant empty content chunk.
    #[test]
    fn empty_file_has_one_canonical_zero_chunk_form() {
        let (_directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &[b""]);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        assert_eq!(decode(&raw, &mut NoopControl).unwrap().files[0].size_bytes, 0);
        let mut wire = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        wire["contents"][0]["chunks"] = json!([""]);
        assert_invalid(&raw_wire(&wire));
    }

    /// Exactly 99 actual registrations export completely; a hundredth rejects without a prefix.
    #[test]
    fn complete_registration_limit_is_not_the_existing_thousand_query_limit() {
        let sources: Vec<&[u8]> = vec![b""; MAX_SOURCE_RESOURCES];
        let (_directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &sources);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        assert_eq!(decode(&raw, &mut NoopControl).unwrap().files.len(), MAX_SOURCE_RESOURCES);
        let hundred: Vec<&[u8]> = vec![b""; MAX_SOURCE_RESOURCES + 1];
        let (_directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &hundred);
        assert_eq!(snapshot.items.len(), 100);
        let mut wire = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        wire["index"] = serde_json::to_value(&snapshot.index).unwrap();
        wire["index_sha256"] = json!(crate::hashing::sha256_hex(&snapshot.index.bytes().unwrap()));
        let last = snapshot.items.last().unwrap();
        wire["pins"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":last.registration.key, "sha256":last.captured.sha256, "size":0}));
        wire["contents"]
            .as_array_mut()
            .unwrap()
            .push(json!({"key":last.registration.key, "encoding":"hex", "chunks":[]}));
        assert_invalid(&raw_wire(&wire));
        match encode(&snapshot, &mut NoopControl) {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "invalid-request"),
            _ => panic!("expected complete hundred-resource refusal"),
        }
    }

    /// Exact raw artifact ceiling accepts legal trailing whitespace; one extra byte refuses before parsing.
    #[test]
    fn raw_artifact_limit_reserves_the_full_import_wrapper() {
        let mut raw = raw_wire(&small_wire());
        raw.resize(MAX_SOURCE_BUNDLE_BYTES, b' ');
        assert_eq!(decode(&raw, &mut NoopControl).unwrap().files.len(), 2);
        raw.push(b' ');
        match decode(&raw, &mut NoopControl) {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "payload-too-large"),
            _ => panic!("expected one-byte-over artifact refusal"),
        }
        assert_eq!(MAX_SOURCE_BUNDLE_BYTES.checked_add(147), Some(1024 * 1024));
    }

    /// A valid captured source below ten MiB still refuses if its whole hex artifact cannot fit.
    #[test]
    fn encoded_transport_limit_refuses_the_whole_export() {
        let source = vec![0xff; MAX_SOURCE_BUNDLE_BYTES / 2];
        let (directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &[&source]);
        match encode(&snapshot, &mut NoopControl) {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "payload-too-large"),
            _ => panic!("expected complete encoded-artifact refusal"),
        }
        assert_eq!(
            std::fs::read(directory.path().join(&snapshot.index.resources[0].path)).unwrap(),
            source
        );
    }

    /// Exact source bytes/hash cannot be forged by a direct internal Snapshot consumer.
    #[test]
    fn inconsistent_capture_facts_cannot_be_exported() {
        let (_directory, mut snapshot) =
            fixture("forge.workspace/2", Role::LifecycleSource, &[b"actual", b"second"]);
        snapshot.items[0].captured.sha256 = "0".repeat(64);
        assert!(encode(&snapshot, &mut NoopControl).is_err());
        let (_directory, mut snapshot) =
            fixture("forge.workspace/2", Role::LifecycleSource, &[b"actual", b"second"]);
        snapshot.items.reverse();
        assert!(encode(&snapshot, &mut NoopControl).is_err());
    }

    /// The absent-index setup state is distinct from a present explicitly empty source bundle.
    #[test]
    fn explicit_empty_index_does_not_infer_missing_index_presence() {
        let (_directory, snapshot) = fixture("forge.workspace/2", Role::LifecycleSource, &[]);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        assert!(decode(&raw, &mut NoopControl).unwrap().files.is_empty());
        let directory = tempfile::tempdir().unwrap();
        let snapshot = Snapshot::capture(&Root::open(directory.path()).unwrap()).unwrap();
        match encode(&snapshot, &mut NoopControl) {
            Err(WorkError::Failed(error)) => assert_eq!(error.code, "not-found"),
            _ => panic!("expected absent-index setup refusal"),
        }
    }

    /// Cooperative cancellation before parsing and during actual chunk copy returns no partial bundle.
    #[test]
    fn controlled_decode_preserves_interruption_before_and_after_partial_copy() {
        let (_directory, snapshot) = fixture(
            "forge.workspace/2",
            Role::LifecycleSource,
            &[&vec![0xac; HEX_CHUNK_BYTES + 1]],
        );
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        for mut control in
            [Recorder::at(Stage::PrepareDomain, 1), Recorder::at(Stage::CopyInputs, 2)]
        {
            assert!(matches!(
                decode(&raw, &mut control),
                Err(WorkError::Interrupted(Interruption::CancelRequested))
            ));
            assert_eq!(control.interruption(), Some(Interruption::CancelRequested));
        }
    }

    /// Cancelled export never reaches publication and preserves source/index fixture bytes.
    #[test]
    fn controlled_export_discards_local_prefix_without_changing_sources() {
        let (directory, snapshot) =
            fixture("forge.workspace/2", Role::LifecycleSource, &[b"first", b"second"]);
        let index_before =
            std::fs::read(directory.path().join(super::super::index::INDEX_PATH)).unwrap();
        let mut control = Recorder::at(Stage::CopyInputs, 2);
        assert!(matches!(
            encode(&snapshot, &mut control),
            Err(WorkError::Interrupted(Interruption::CancelRequested))
        ));
        assert_eq!(
            std::fs::read(directory.path().join(super::super::index::INDEX_PATH)).unwrap(),
            index_before
        );
        for item in snapshot.items {
            assert_eq!(
                std::fs::read(directory.path().join(&item.registration.path)).unwrap(),
                item.captured.bytes
            );
        }
    }
}
