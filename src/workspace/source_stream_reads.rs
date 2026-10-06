//! Committed staged-source manifest/part reader, never an unconfirmed source read.
//!
//! ROOT admits exact selected-major route/session/parameters and participating project
//! lease first. Store descriptor selection occurs before this off-lock I/O port.

use super::contract::{self, ApiMajor, Error};
use super::effects::StagedExportDescriptor;
use super::preparation::{ProgressUpdate, Stage, WorkControl, WorkResult};
use super::root::{Root, conflict};
use super::source_transfers::{MAX_ARTIFACT_BYTES, PART_BYTES};
use serde_json::{Value, json};

/// Off-lock exact committed generation, private until a complete bounded response is ready.
pub(crate) struct CommittedStagedArtifact {
    /// Exact observed committed output bytes; not a declared or reconstructed raw artifact.
    bytes: Vec<u8>,
    /// Full independent observed hash checked against the original receipt descriptor.
    sha256: String,
}

/// Capture exactly one already selected committed output and reject changed whole generations.
/// Callback fences are cooperative before/after a bounded read, not syscall preemption.
pub(crate) fn capture(
    root: &Root,
    descriptor: &StagedExportDescriptor,
    control: &mut dyn WorkControl,
) -> WorkResult<CommittedStagedArtifact> {
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
    if descriptor.bytes == 0 || descriptor.bytes > MAX_ARTIFACT_BYTES {
        return Err(Error::invalid().into());
    }
    let captured = root.read(&descriptor.path, MAX_ARTIFACT_BYTES)?;
    control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
    if captured.bytes.len() != descriptor.bytes || captured.sha256 != descriptor.sha256 {
        return Err(conflict().into());
    }
    Ok(CommittedStagedArtifact { bytes: captured.bytes, sha256: captured.sha256 })
}

impl CommittedStagedArtifact {
    /// Project original complete bytes into a fixed small manifest; no paths/content appear.
    pub(crate) fn manifest(&self, operation_id: &str) -> WorkResult<Value> {
        let value = json!({"operation_id":operation_id,
            "schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged","source_content_included":true,
            "artifact_sha256":self.sha256,"artifact_size_bytes":self.bytes.len(),
            "chunk_size_bytes":PART_BYTES,"chunk_count":self.bytes.len().div_ceil(PART_BYTES)});
        contract::validate_for(ApiMajor::V2, "SourceStreamManifest", &value)?;
        Ok(value)
    }

    /// Return one exact ordinal from this fully verified generation, or no partial response.
    pub(crate) fn part(
        &self,
        operation_id: &str,
        ordinal: usize,
        control: &mut dyn WorkControl,
    ) -> WorkResult<Value> {
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Clear)?;
        if ordinal >= self.bytes.len().div_ceil(PART_BYTES) {
            return Err(Error::invalid().into());
        }
        let start = ordinal.checked_mul(PART_BYTES).ok_or_else(Error::invalid)?;
        let end = start.checked_add(PART_BYTES).ok_or_else(Error::invalid)?.min(self.bytes.len());
        let bytes = &self.bytes[start..end];
        let value = json!({"operation_id":operation_id,"artifact_sha256":self.sha256,
            "chunk_ordinal":ordinal,"sha256":crate::hashing::sha256_hex(bytes),
            "size_bytes":bytes.len(),"hex":canonical_hex(bytes)?});
        contract::validate_for(ApiMajor::V2, "SourceStreamChunk", &value)?;
        control.checkpoint(Stage::PrepareDomain, ProgressUpdate::Unchanged)?;
        Ok(value)
    }
}

/// Encode at most32KiB exact original bytes into canonical lower hex, without raw diagnostics.
fn canonical_hex(bytes: &[u8]) -> WorkResult<String> {
    const DIGITS: &[u8] = b"0123456789abcdef";
    if bytes.is_empty() || bytes.len() > PART_BYTES {
        return Err(Error::invalid().into());
    }
    let mut output = String::new();
    output
        .try_reserve_exact(bytes.len().checked_mul(2).ok_or_else(Error::invalid)?)
        .map_err(|_| Error::invalid())?;
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 15)]));
    }
    Ok(output)
}

/// Off-lock reader controls with actual confined bytes, separate from authenticated export proof.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::index::Index;
    use crate::workspace::preparation::test_support::Recorder;
    use crate::workspace::preparation::{NoopControl, WorkError};

    /// Author a valid empty Bundle4 with exact legal raw padding across two transport parts.
    fn artifact() -> Vec<u8> {
        let index = Index {
            schema_version: "forge.workspace/2".into(),
            label: "reader fixture".into(),
            resources: Vec::new(),
        };
        let value = json!({"schema_version":"forge.workspace-index-bundle/4",
            "profile":"index-and-source-hex-staged","source_content_included":true,
            "index":index,"index_sha256":crate::hashing::sha256_hex(&index.bytes().unwrap()),
            "pins":[],"contents":[]});
        let mut raw = contract::encode(&value, MAX_ARTIFACT_BYTES, false).unwrap();
        raw.extend(std::iter::repeat_n(b' ', PART_BYTES + 13));
        raw
    }

    /// Create one actual confined output and a synthetic private descriptor for isolated reader tests.
    fn fixture(raw: &[u8]) -> (tempfile::TempDir, Root, StagedExportDescriptor) {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("staged.json"), raw).unwrap();
        let root = Root::open(directory.path()).unwrap();
        let descriptor = StagedExportDescriptor {
            path: "staged.json".into(),
            sha256: crate::hashing::sha256_hex(raw),
            bytes: raw.len(),
        };
        (directory, root, descriptor)
    }

    /// All manifest/ordinal facts come from complete actual original bytes with no raw rewrite.
    #[test]
    fn manifest_and_exact_parts_reassemble_the_original_generation() {
        let raw = artifact();
        let (_directory, root, descriptor) = fixture(&raw);
        let capture = capture(&root, &descriptor, &mut NoopControl).unwrap();
        let id = "op_0123456789abcdef";
        let manifest = capture.manifest(id).unwrap();
        assert_eq!(manifest["artifact_size_bytes"], raw.len());
        let mut joined = Vec::new();
        for ordinal in 0..raw.len().div_ceil(PART_BYTES) {
            let value = capture.part(id, ordinal, &mut NoopControl).unwrap();
            let part = crate::workspace::source_transfers::DecodedPart::parse(
                &json!({"sha256":value["sha256"],"size_bytes":value["size_bytes"],"hex":value["hex"]}),
                &mut NoopControl,
            );
            assert!(part.is_ok());
            let hex = value["hex"].as_str().unwrap();
            for offset in (0..hex.len()).step_by(2) {
                joined.push(u8::from_str_radix(&hex[offset..offset + 2], 16).unwrap());
            }
        }
        assert_eq!(joined, raw);
        assert_eq!(manifest["artifact_sha256"], crate::hashing::sha256_hex(&joined));
    }

    /// Changed observed output refuses before any manifest/part is projected.
    #[test]
    fn changed_generation_is_not_a_partial_download_success() {
        let raw = artifact();
        let (directory, root, descriptor) = fixture(&raw);
        std::fs::write(directory.path().join("staged.json"), b"changed").unwrap();
        assert!(capture(&root, &descriptor, &mut NoopControl).is_err());
    }

    /// An ordinal outside the complete ceil(size/32768) range returns no prefix or empty part.
    #[test]
    fn missing_ordinal_never_projects_an_empty_successful_chunk() {
        let raw = artifact();
        let (_directory, root, descriptor) = fixture(&raw);
        let capture = capture(&root, &descriptor, &mut NoopControl).unwrap();
        assert!(
            capture
                .part("op_0123456789abcdef", raw.len().div_ceil(PART_BYTES), &mut NoopControl)
                .is_err()
        );
        assert!(canonical_hex(&[]).is_err());
        assert!(canonical_hex(&vec![0; PART_BYTES + 1]).is_err());
    }

    /// Existing typed cooperative stops before/after the bounded read cannot become wire success.
    #[test]
    fn off_lock_reader_retains_typed_interruption() {
        let raw = artifact();
        for visit in [1, 2] {
            let (_directory, root, descriptor) = fixture(&raw);
            let mut recorder = Recorder::at(Stage::PrepareDomain, visit);
            assert!(matches!(
                capture(&root, &descriptor, &mut recorder),
                Err(WorkError::Interrupted(_))
            ));
        }
    }
}
