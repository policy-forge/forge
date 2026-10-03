//! Explicit staged Bundle4 codec; existing inline Bundle3 keeps its separate limits.
//!
//! API2.4 adapters own transport admission, native authority and shared retention.
//! The intrinsic codec shares the inline bundle grammar while keeping explicit
//! staged transport limits; no existing inline raw limit is widened.
//!
//! Decoding yields inert bytes and descriptors only. Neither direction opens a
//! supplied path, publishes a file, infers role validity, or grants source-read
//! authority. The caller owns explicit sensitivity acknowledgment and admission.

use serde_json::{Value, json};

use super::contract::{self, Error, Result};
use super::index::{Index, MAX_INDEX_BYTES};
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::services::Snapshot;
use super::source_bundles::{DecodedSourceBundle, DecodedSourceFile};

/// Whole raw Bundle4 engineering cap, separate from each one-MiB chunk request.
pub(crate) const MAX_SOURCE_BUNDLE_BYTES: usize = 10 * 1024 * 1024;
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
    if value["schema_version"] != "forge.workspace-index-bundle/4"
        || value["profile"] != "index-and-source-hex-staged"
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
    Ok(
        json!({"schema_version":"forge.workspace-index-bundle/4", "profile":"index-and-source-hex-staged",
        "source_content_included":true, "index":snapshot.index,
        "index_sha256":crate::hashing::sha256_hex(&normalized), "pins":pins, "contents":contents}),
    )
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
/// the explicit staged logical-artifact ceiling; no source is truncated to fit.
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
/// Actual confined fixture captures prove codec bytes, not native-role or restore approval.
mod tests {
    use super::*;
    use crate::workspace::index::{INDEX_PATH, Resource, Role};
    use crate::workspace::preparation::{NoopControl, WorkError};
    use crate::workspace::root::Root;

    /// Capture every explicitly registered fixture via the actual native Root/Snapshot path.
    fn fixture(sources: &[&[u8]]) -> (tempfile::TempDir, Snapshot) {
        let directory = tempfile::tempdir().unwrap();
        let mut resources = Vec::new();
        for (position, bytes) in sources.iter().enumerate() {
            let key = format!("source-{position}");
            let path = format!("{key}.md");
            std::fs::write(directory.path().join(&path), bytes).unwrap();
            resources.push(Resource { key, role: Role::PolicySource, path });
        }
        let index = Index {
            schema_version: "forge.workspace/2".into(),
            label: "staged fixture".into(),
            resources,
        };
        std::fs::write(directory.path().join(INDEX_PATH), index.bytes().unwrap()).unwrap();
        let root = Root::open(directory.path()).unwrap();
        (directory, Snapshot::capture(&root).unwrap())
    }

    /// Obtain an exact complete real-capture codec result for strict malformed controls.
    fn small_raw() -> Vec<u8> {
        let (_directory, snapshot) = fixture(&[b"# Fixture\r\npolicy text\n"]);
        encode(&snapshot, &mut NoopControl).unwrap()
    }

    /// Require typed codec failure without printing raw private file or parser details.
    fn invalid(raw: &[u8]) {
        assert!(
            matches!(decode(raw,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="invalid-request")
        );
    }

    /// Genuine captured larger output exceeds Bundle3 yet preserves every ordered exact byte.
    #[test]
    fn larger_real_capture_roundtrips_without_widening_inline_bundle3() {
        let source = b"# Source\ntext\n".repeat(50_000);
        let (_directory, snapshot) = fixture(&[&source, b"second\r\n"]);
        assert!(
            matches!(super::super::source_bundles::encode(&snapshot,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="payload-too-large")
        );
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        assert!(raw.len() > super::super::source_bundles::MAX_SOURCE_BUNDLE_BYTES);
        let decoded = decode(&raw, &mut NoopControl).unwrap();
        assert_eq!(decoded.files.len(), 2);
        assert_eq!(decoded.files[0].bytes, source);
        assert_eq!(decoded.files[1].bytes, b"second\r\n");
        invalid_inline(&raw);
    }

    /// Existing strict Bundle3 grammar refuses the new profile instead of silently reinterpreting it.
    fn invalid_inline(raw: &[u8]) {
        assert!(
            matches!(super::super::source_bundles::decode(raw,&mut NoopControl),Err(WorkError::Failed(error)) if matches!(error.code,"invalid-request"|"payload-too-large"))
        );
    }

    /// All binary source bytes and CRLF survive independently of UTF8 role-validation labels.
    #[test]
    fn inert_binary_exactness_does_not_claim_native_role_admission() {
        let bytes = (0_u8..=255).collect::<Vec<_>>();
        let (_directory, snapshot) = fixture(&[&bytes, b"\r\n"]);
        let raw = encode(&snapshot, &mut NoopControl).unwrap();
        let decoded = decode(&raw, &mut NoopControl).unwrap();
        assert_eq!(decoded.files[0].bytes, bytes);
        assert_eq!(decoded.files[1].bytes, b"\r\n");
    }

    /// The two source profile tags are exact and never an implicit same-field upgrade.
    #[test]
    fn inline_and_staged_profiles_remain_separate_strict_grammars() {
        let raw = small_raw();
        invalid_inline(&raw);
        let value = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        let mut wrong = value.clone();
        wrong["profile"] = json!("index-and-source-hex");
        invalid(&contract::encode(&wrong, MAX_SOURCE_BUNDLE_BYTES, false).unwrap());
        let mut old = value;
        old["schema_version"] = json!("forge.workspace-index-bundle/3");
        invalid(&contract::encode(&old, MAX_SOURCE_BUNDLE_BYTES, false).unwrap());
    }

    /// Strict raw parsing keeps duplicates/BOM/trailing bytes and escaped key collisions visible.
    #[test]
    fn raw_grammar_rejects_before_any_partial_decoded_bundle() {
        let raw = small_raw();
        let body = String::from_utf8(raw.clone()).unwrap();
        let dup =
            body.replacen('{', r#"{"\u0073chema_version":"forge.workspace-index-bundle/4","#, 1);
        invalid(dup.as_bytes());
        let mut bom = vec![0xef, 0xbb, 0xbf];
        bom.extend_from_slice(&raw);
        invalid(&bom);
        let mut trailing = raw;
        trailing.extend_from_slice(b" {}");
        invalid(&trailing);
    }

    /// Exact logical raw bound remains whole; no body or decoded capture cap is raised.
    #[test]
    fn whole_logical_artifact_limit_and_one_over_are_explicit() {
        let (_directory, snapshot) = fixture(&[]);
        let mut raw = encode(&snapshot, &mut NoopControl).unwrap();
        raw.resize(MAX_SOURCE_BUNDLE_BYTES, b' ');
        assert!(decode(&raw, &mut NoopControl).is_ok());
        raw.push(b' ');
        assert!(
            matches!(decode(&raw,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="payload-too-large")
        );
    }

    /// Every ordered descriptor and content row must match complete authorial membership.
    #[test]
    fn whole_pin_content_hash_and_length_bijection_is_preserved() {
        let raw = small_raw();
        let good = contract::parse(&raw, MAX_SOURCE_BUNDLE_BYTES, HEX_CHUNK_CHARS).unwrap();
        for target in ["pin-key", "content-key", "hash", "size", "extra-content"] {
            let mut bad = good.clone();
            match target {
                "pin-key" => bad["pins"][0]["key"] = json!("other"),
                "content-key" => bad["contents"][0]["key"] = json!("other"),
                "hash" => bad["pins"][0]["sha256"] = json!("0".repeat(64)),
                "size" => bad["pins"][0]["size"] = json!(0),
                _ => {
                    bad["contents"].as_array_mut().unwrap().push(good["contents"][0].clone());
                }
            }
            invalid(&contract::encode(&bad, MAX_SOURCE_BUNDLE_BYTES, false).unwrap());
        }
    }

    /// Codec99 registrations preserves the index slot, and refuses100 rather than a prefix.
    #[test]
    fn unchanged_complete_registration_bound_is_not_larger_transfer_capacity() {
        let sources = vec![b"policy\n".as_slice(); 99];
        let (_directory, snapshot) = fixture(&sources);
        assert_eq!(
            decode(&encode(&snapshot, &mut NoopControl).unwrap(), &mut NoopControl)
                .unwrap()
                .files
                .len(),
            99
        );
        let sources = vec![b"policy\n".as_slice(); 100];
        let (_directory, snapshot) = fixture(&sources);
        assert!(
            matches!(encode(&snapshot,&mut NoopControl),Err(WorkError::Failed(error)) if error.code=="invalid-request")
        );
    }
}
