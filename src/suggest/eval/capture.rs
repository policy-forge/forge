//! Confined exact-byte capture with per-case logical byte bounds and final pin verification.

use super::manifest::{ArtifactRef, Corpus, Outcome, error};
use crate::{ForgeError, hashing::sha256_hex, suggest::shared};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};

// Proposed engineering ceilings, not approved operating-system RSS/wall profiles.
const MAX_CASE_BYTES: u64 = 50 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ARTIFACTS: usize = 4096;

struct Pin {
    reference: ArtifactRef,
    identity: (u64, u64),
    limit: u64,
}
pub(super) struct Session {
    root: PathBuf,
    pins: BTreeMap<String, Pin>,
    identities: BTreeMap<(u64, u64), String>,
    folded_paths: BTreeMap<String, String>,
    total_bytes: u64,
    case_bytes: u64,
    case_paths: BTreeSet<String>,
}
impl Session {
    pub(super) fn new(root: &Path) -> Result<Self, ForgeError> {
        let metadata =
            std::fs::symlink_metadata(root).map_err(|_| error("evaluation root is unavailable"))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(error("evaluation root must be a directory, not a symbolic link"));
        }
        let root =
            std::fs::canonicalize(root).map_err(|_| error("evaluation root cannot be resolved"))?;
        Ok(Self {
            root,
            pins: BTreeMap::new(),
            identities: BTreeMap::new(),
            folded_paths: BTreeMap::new(),
            total_bytes: 0,
            case_bytes: 0,
            case_paths: BTreeSet::new(),
        })
    }
    pub(super) fn root(&self) -> &Path {
        &self.root
    }
    pub(super) fn begin_case(&mut self) {
        self.case_bytes = 0;
        self.case_paths.clear();
    }
    pub(super) fn manifest(&mut self, path: &Path, limit: u64) -> Result<Arc<[u8]>, ForgeError> {
        let path = path.to_str().ok_or_else(|| error("evaluation manifest path must be UTF-8"))?;
        self.capture(path, None, limit)
    }
    pub(super) fn validate_declared(&self, corpus: &Corpus) -> Result<(), ForgeError> {
        // Inventory known manifest references before any case artifact is read.
        // Imported payload/retained-response links are discovered and bounded later.
        let mut references = BTreeMap::new();
        let mut folded = BTreeMap::new();
        let mut total = 0_u64;
        for pin in self.pins.values() {
            declare(&pin.reference, &mut references, &mut folded, &mut total)?;
        }
        for case in &corpus.cases {
            let mut local_paths = BTreeSet::new();
            let mut local_bytes = 0_u64;
            for reference in
                case.sources.iter().map(|source| &source.artifact).chain(case.request.iter()).chain(
                    match &case.outcome {
                        Outcome::Response { run, response, bundle, .. } => {
                            vec![Some(run), Some(response), bundle.as_ref()]
                        }
                        _ => Vec::new(),
                    }
                    .into_iter()
                    .flatten(),
                )
            {
                declare(reference, &mut references, &mut folded, &mut total)?;
                declare_group(reference, &mut local_paths, &mut local_bytes)?;
            }
        }
        for (_, group) in corpus.evidence.groups() {
            let mut local_paths = BTreeSet::new();
            let mut local_bytes = 0_u64;
            for reference in group {
                declare(reference, &mut references, &mut folded, &mut total)?;
                declare_group(reference, &mut local_paths, &mut local_bytes)?;
            }
        }
        Ok(())
    }
    pub(super) fn read(
        &mut self,
        reference: &ArtifactRef,
        limit: u64,
    ) -> Result<Arc<[u8]>, ForgeError> {
        reference.validate(limit)?;
        self.capture(&reference.path, Some(reference), limit)
    }
    fn capture(
        &mut self,
        path: &str,
        expected: Option<&ArtifactRef>,
        limit: u64,
    ) -> Result<Arc<[u8]>, ForgeError> {
        shared::relative_path("evaluation artifact path", path)?;
        let folded = path.to_ascii_lowercase();
        if self.folded_paths.get(&folded).is_some_and(|previous| previous != path) {
            return Err(error("evaluation artifacts alias through case-distinct paths"));
        }
        let existing = self.pins.get(path);
        if existing.is_none() && self.pins.len() >= MAX_ARTIFACTS {
            return Err(error("evaluation artifact inventory exceeds its bound"));
        }
        if let (Some(pin), Some(expected)) = (existing, expected)
            && pin.reference != *expected
        {
            return Err(error("evaluation references disagree about the same artifact"));
        }
        let total_remaining = if existing.is_some() {
            limit
        } else {
            MAX_TOTAL_BYTES.saturating_sub(self.total_bytes)
        };
        let case_remaining = if self.case_paths.contains(path) {
            limit
        } else {
            MAX_CASE_BYTES.saturating_sub(self.case_bytes)
        };
        let effective_limit = limit.min(total_remaining).min(case_remaining);
        if effective_limit == 0 || expected.is_some_and(|pin| pin.bytes > effective_limit) {
            return Err(error("evaluation capture exceeds its case or total byte bound"));
        }
        let (bytes, identity) =
            crate::linkage::read_confined_local_file(&self.root, Path::new(path), effective_limit)
                .map_err(|_| {
                    error("evaluation artifact is unsafe, unreadable or exceeds its capture bound")
                })?;
        let actual = ArtifactRef {
            path: path.to_owned(),
            sha256: sha256_hex(&bytes),
            bytes: bytes.len() as u64,
        };
        if expected.is_some_and(|expected| expected != &actual) {
            return Err(error("evaluation artifact does not match its exact SHA-256 and byte pin"));
        }
        if let Some(pin) = existing {
            if pin.reference != actual || pin.identity != identity {
                return Err(error("evaluation artifact changed after capture"));
            }
        } else {
            if self.identities.get(&identity).is_some_and(|previous| previous != path) {
                return Err(error("evaluation artifacts alias one file through distinct paths"));
            }
            self.total_bytes = self
                .total_bytes
                .checked_add(actual.bytes)
                .ok_or_else(|| error("evaluation total byte count overflow"))?;
            self.identities.insert(identity, path.to_owned());
            self.folded_paths.insert(folded, path.to_owned());
            self.pins.insert(path.to_owned(), Pin { reference: actual.clone(), identity, limit });
        }
        if self.case_paths.insert(path.to_owned()) {
            self.case_bytes = self
                .case_bytes
                .checked_add(actual.bytes)
                .ok_or_else(|| error("evaluation case byte count overflow"))?;
        }
        Ok(Arc::from(bytes))
    }
    pub(super) fn verify(&self) -> Result<(), ForgeError> {
        // Only pins are retained across cases; each file is reopened and released.
        for pin in self.pins.values() {
            let (bytes, identity) = crate::linkage::read_confined_local_file(
                &self.root,
                Path::new(&pin.reference.path),
                pin.limit,
            )
            .map_err(|_| error("evaluation input cannot be safely revalidated"))?;
            if identity != pin.identity
                || bytes.len() as u64 != pin.reference.bytes
                || sha256_hex(&bytes) != pin.reference.sha256
            {
                return Err(error("evaluation input changed before report publication"));
            }
        }
        Ok(())
    }
}

fn declare<'a>(
    reference: &'a ArtifactRef,
    references: &mut BTreeMap<&'a str, &'a ArtifactRef>,
    folded: &mut BTreeMap<String, &'a str>,
    total: &mut u64,
) -> Result<(), ForgeError> {
    let key = reference.path.to_ascii_lowercase();
    if folded.get(&key).is_some_and(|previous| *previous != reference.path) {
        return Err(error("evaluation artifacts alias through case-distinct paths"));
    }
    if let Some(previous) = references.get(reference.path.as_str()) {
        if *previous != reference {
            return Err(error("evaluation references disagree about the same artifact"));
        }
        return Ok(());
    }
    if references.len() >= MAX_ARTIFACTS {
        return Err(error("evaluation declared artifact inventory exceeds its bound"));
    }
    if reference.bytes > MAX_TOTAL_BYTES.saturating_sub(*total) {
        return Err(error("evaluation declared unique artifact bytes exceed their total bound"));
    }
    *total += reference.bytes;
    folded.insert(key, reference.path.as_str());
    references.insert(reference.path.as_str(), reference);
    Ok(())
}
fn declare_group<'a>(
    reference: &'a ArtifactRef,
    paths: &mut BTreeSet<&'a str>,
    bytes: &mut u64,
) -> Result<(), ForgeError> {
    if paths.insert(reference.path.as_str()) {
        if reference.bytes > MAX_CASE_BYTES.saturating_sub(*bytes) {
            return Err(error("evaluation declared case or evidence bytes exceed their bound"));
        }
        *bytes += reference.bytes;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_verification_refuses_changed_bytes_and_replaced_file_identity() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source.txt");
        std::fs::write(&path, b"original").unwrap();
        let reference = ArtifactRef {
            path: "source.txt".to_owned(),
            sha256: sha256_hex(b"original"),
            bytes: 8,
        };
        let mut captures = Session::new(directory.path()).unwrap();
        captures.read(&reference, 64).unwrap();
        assert!(captures.verify().is_ok());
        std::fs::write(&path, b"modified").unwrap();
        assert!(
            captures
                .verify()
                .unwrap_err()
                .to_string()
                .contains("changed before report publication")
        );
        std::fs::rename(&path, directory.path().join("old.txt")).unwrap();
        std::fs::write(&path, b"original").unwrap();
        assert!(captures.verify().is_err());
    }

    #[test]
    fn repeat_path_preserves_one_pin_and_conflicting_reference_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("source.txt"), b"original").unwrap();
        let reference = ArtifactRef {
            path: "source.txt".to_owned(),
            sha256: sha256_hex(b"original"),
            bytes: 8,
        };
        let mut captures = Session::new(directory.path()).unwrap();
        captures.read(&reference, 64).unwrap();
        captures.read(&reference, 64).unwrap();
        assert_eq!(captures.pins.len(), 1);
        assert_eq!(captures.total_bytes, 8);
        assert_eq!(captures.case_bytes, 8);
        captures.begin_case();
        captures.read(&reference, 64).unwrap();
        assert_eq!(captures.total_bytes, 8);
        assert_eq!(captures.case_bytes, 8);
        let mut altered = reference;
        altered.sha256 = "b".repeat(64);
        assert!(
            captures.read(&altered, 64).unwrap_err().to_string().contains("references disagree")
        );
    }
}
