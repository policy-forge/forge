//! Confined input capture and exact PRD-056 baseline verification.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::ForgeError;
use crate::applicability::model::ReportFilters;
use crate::hashing::sha256_hex;

use super::error;
use super::manifest::{self, MAX_CLAUSE_BYTES, MAX_MANIFEST_BYTES, MAX_TOTAL_BYTES, PinnedFile};
use super::model::{InputFingerprint, LoadedAuthorProject, LoadedClause};

pub(super) struct PreparedProject {
    pub root: PathBuf,
    pub loaded: LoadedAuthorProject,
    pub(super) captures: CaptureSet,
}

struct CapturedFile {
    fingerprint: InputFingerprint,
    bytes: Vec<u8>,
    identity: (u64, u64),
    max_bytes: u64,
}

pub(super) struct CaptureSet {
    root: PathBuf,
    files: BTreeMap<String, CapturedFile>,
    identities: BTreeSet<(u64, u64)>,
    byte_count: u64,
    byte_limit: u64,
}

impl CaptureSet {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: BTreeMap::new(),
            identities: BTreeSet::new(),
            byte_count: 0,
            byte_limit: MAX_TOTAL_BYTES,
        }
    }

    pub(super) fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub(super) fn restrict_budget(&mut self, limit: u64) -> Result<(), ForgeError> {
        self.byte_limit = self.byte_limit.min(limit);
        if self.byte_count > self.byte_limit {
            return Err(error("captured requests exceed the aggregate input budget"));
        }
        Ok(())
    }

    pub(super) fn reject_cross_aliases(&self, other: &Self) -> Result<(), ForgeError> {
        let identities: BTreeMap<_, _> =
            other.files.iter().map(|(path, file)| (file.identity, path)).collect();
        for (path, file) in &self.files {
            if let Some(other_path) = identities.get(&file.identity)
                && self.root.join(path) != other.root.join(other_path)
            {
                return Err(error("snapshot inputs alias one file through distinct paths"));
            }
        }
        Ok(())
    }

    pub(super) fn read(
        &mut self,
        role: &str,
        path: &Path,
        expected_sha256: Option<&str>,
        max_bytes: u64,
    ) -> Result<Vec<u8>, ForgeError> {
        let label = portable_label(path)?;
        if self.files.keys().any(|existing| existing.eq_ignore_ascii_case(&label)) {
            return Err(error(format!("{role} aliases another input")));
        }
        let remaining = self.byte_limit.saturating_sub(self.byte_count);
        if remaining == 0 {
            return Err(error(format!(
                "{role}: captured project inputs have exhausted the 50 MiB total source budget"
            )));
        }
        let effective_limit = max_bytes.min(remaining);
        let (bytes, identity) =
            crate::linkage::read_confined_local_file(&self.root, path, effective_limit).map_err(
                |cause| {
                    if effective_limit < max_bytes {
                        // Report the known limit without guessing why the read failed.
                        // Missing-file and permission failures must retain their cause too.
                        error(format!(
                            "{role}: input read failed with {remaining} bytes remaining in the \
                             50 MiB total source budget (per-file limit: {max_bytes} bytes): {cause}"
                        ))
                    } else {
                        error(format!("{role}: {cause}"))
                    }
                },
            )?;
        let digest = sha256_hex(&bytes);
        if expected_sha256.is_some_and(|expected| expected != digest) {
            return Err(error(format!("{role} SHA-256 does not match its exact byte pin")));
        }
        if !self.identities.insert(identity) {
            return Err(error(format!("{role} aliases another input file")));
        }
        self.byte_count = self
            .byte_count
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| error("input byte count overflow"))?;
        // Retain the aggregate invariant independently of the confined reader's bounds.
        if self.byte_count > self.byte_limit {
            return Err(error("captured project inputs exceed the 50 MiB total limit"));
        }
        let fingerprint = InputFingerprint {
            role: role.to_owned(),
            path: label.clone(),
            sha256: digest,
            byte_length: bytes.len() as u64,
        };
        self.files.insert(
            label,
            CapturedFile {
                fingerprint,
                bytes: bytes.clone(),
                identity,
                max_bytes: effective_limit,
            },
        );
        Ok(bytes)
    }

    pub(super) fn pinned(
        &mut self,
        role: &str,
        pin: &PinnedFile,
        limit: u64,
    ) -> Result<Vec<u8>, ForgeError> {
        self.read(role, &pin.path, Some(&pin.expected_sha256), limit)
    }

    pub(super) fn fingerprints(&self) -> Vec<InputFingerprint> {
        let mut entries: Vec<_> =
            self.files.values().map(|file| file.fingerprint.clone()).collect();
        entries.sort_by(|left, right| (&left.role, &left.path).cmp(&(&right.role, &right.path)));
        entries
    }

    pub(super) fn verify(&self) -> Result<(), ForgeError> {
        for (path, capture) in &self.files {
            let (bytes, identity) = crate::linkage::read_confined_local_file(
                &self.root,
                Path::new(path),
                capture.max_bytes,
            )
            .map_err(|cause| error(format!("input revalidation failed: {cause}")))?;
            if identity != capture.identity || bytes != capture.bytes {
                return Err(error(format!(
                    "{} changed after input capture",
                    capture.fingerprint.role
                )));
            }
        }
        Ok(())
    }
}

impl PreparedProject {
    pub(super) fn byte_count(&self) -> u64 {
        self.captures.byte_count
    }

    pub(super) fn control_fingerprints(&self) -> Result<BTreeMap<String, String>, ForgeError> {
        let snapshot =
            tempfile::tempdir().map_err(|_| error("cannot create inventory snapshot"))?;
        for (path, captured) in &self.captures.files {
            let target = snapshot.path().join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|_| error("cannot prepare inventory snapshot"))?;
            }
            std::fs::write(target, &captured.bytes)
                .map_err(|_| error("cannot write inventory snapshot"))?;
        }
        let baseline = &self.loaded.project.applicability_manifest.path;
        let capture = self
            .captures
            .files
            .get(&portable_label(baseline)?)
            .ok_or_else(|| error("missing captured baseline"))?;
        let manifest = crate::applicability::manifest::parse(&capture.bytes)?;
        let root = snapshot.path().join(baseline.parent().unwrap_or_else(|| Path::new("")));
        let resource = crate::mapping::inventory::load(&root, "framework", &manifest.framework)?;
        let kind = crate::mapping::manifest::SubjectType::Control;
        resource
            .inventory
            .ids_of_type(kind)
            .into_iter()
            .map(|id| {
                let digest = resource
                    .inventory
                    .fingerprint(kind, &id)
                    .ok_or_else(|| error("validated control lacks a fingerprint"))?
                    .to_owned();
                Ok((id, digest))
            })
            .collect()
    }

    /// Reopen every source through the same confined traversal before publication.
    pub(super) fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.captures.verify()
    }
}

pub(super) fn prepare(manifest_path: &Path) -> Result<PreparedProject, ForgeError> {
    prepare_bounded(manifest_path, MAX_TOTAL_BYTES)
}

pub(super) fn prepare_bounded(
    manifest_path: &Path,
    budget: u64,
) -> Result<PreparedProject, ForgeError> {
    prepare_pinned(manifest_path, budget, None)
}

pub(super) fn prepare_pinned(
    manifest_path: &Path,
    budget: u64,
    expected: Option<&str>,
) -> Result<PreparedProject, ForgeError> {
    let absolute = absolute_manifest_path(manifest_path)?;
    let root = absolute
        .parent()
        .ok_or_else(|| error("manifest must have a parent directory"))?
        .to_path_buf();
    let manifest_name = absolute.file_name().ok_or_else(|| error("manifest must name a file"))?;
    let mut captures = CaptureSet::new(root.clone());
    captures.byte_limit = budget.min(MAX_TOTAL_BYTES);
    let project_bytes =
        captures.read("author-project", Path::new(manifest_name), expected, MAX_MANIFEST_BYTES)?;
    let project = manifest::parse_project(&project_bytes)?;
    if project.project_root != Path::new(".") {
        return Err(error("Phase 1 project_root must be '.' (the author manifest directory)"));
    }
    let pack_bytes =
        captures.pinned("authoring-pack", &project.authoring_pack, MAX_MANIFEST_BYTES)?;
    let pack = manifest::parse_pack(&pack_bytes)?;
    let report_bytes =
        captures.pinned("gap-report", &project.gap_report, crate::io::MAX_FILE_SIZE)?;
    let supplied_report = strict_json(&report_bytes, "gap report")?;
    let applicability_bytes = captures.pinned(
        "applicability-manifest",
        &project.applicability_manifest,
        crate::applicability::manifest::MAX_MANIFEST_BYTES,
    )?;
    let applicability = crate::applicability::manifest::parse(&applicability_bytes)
        .map_err(|cause| error(format!("applicability manifest: {cause}")))?;
    let baseline_dir =
        project.applicability_manifest.path.parent().unwrap_or_else(|| Path::new(""));
    let framework_path = contained_dependency(baseline_dir, &applicability.framework.artifact)?;
    let framework_bytes = captures.read(
        "framework",
        &framework_path,
        Some(&project.baseline.framework_sha256),
        crate::io::MAX_FILE_SIZE,
    )?;
    strict_json(&framework_bytes, "framework")?;
    if let Some(companion) = &applicability.framework.resolved_catalog {
        let pin =
            project.baseline.resolved_catalog_sha256.as_deref().ok_or_else(|| {
                error("Profile baseline requires the resolved Catalog fingerprint")
            })?;
        let companion_path = contained_dependency(baseline_dir, companion)?;
        let companion_bytes = captures.read(
            "resolved-catalog",
            &companion_path,
            Some(pin),
            crate::io::MAX_FILE_SIZE,
        )?;
        strict_json(&companion_bytes, "resolved Catalog")?;
    } else if project.baseline.resolved_catalog_sha256.is_some() {
        return Err(error("Catalog baseline must not declare a resolved Catalog fingerprint"));
    }
    for (index, mapping) in applicability.mapping_collections.iter().enumerate() {
        let path = contained_dependency(baseline_dir, mapping)?;
        let bytes = captures.read(
            &format!("mapping-collection-{index}"),
            &path,
            None,
            crate::io::MAX_FILE_SIZE,
        )?;
        strict_json(&bytes, "Mapping Collection")?;
    }

    let baseline_report = analyze_snapshot(&captures, &project.applicability_manifest.path)?;
    let regenerated = serde_json::to_value(&baseline_report)
        .map_err(|cause| error(format!("cannot serialize baseline: {cause}")))?;
    if regenerated != supplied_report {
        return Err(error(
            "gap report does not exactly represent the complete current unfiltered PRD-056 analysis",
        ));
    }
    if pack.baseline != project.baseline
        || project.baseline.report_sha256 != sha256_hex(&report_bytes)
        || project.baseline.framework_sha256 != baseline_report.framework.raw_sha256
        || project.baseline.resolved_catalog_sha256
            != baseline_report.framework.resolved_catalog_sha256
    {
        return Err(error(
            "pack, project, framework and report baseline fingerprints must match exactly",
        ));
    }
    manifest::validate_relationships(&pack, &project, &baseline_report)?;
    let clauses = load_clauses(&mut captures, &project)?;
    captures.verify()?;
    let loaded = LoadedAuthorProject {
        project,
        pack,
        baseline_report,
        project_sha256: sha256_hex(&project_bytes),
        pack_sha256: sha256_hex(&pack_bytes),
        report_sha256: sha256_hex(&report_bytes),
        inputs: captures.fingerprints(),
        clauses,
    };
    Ok(PreparedProject { root, loaded, captures })
}

fn load_clauses(
    captures: &mut CaptureSet,
    project: &manifest::AuthorProject,
) -> Result<BTreeMap<String, LoadedClause>, ForgeError> {
    let mut clauses = BTreeMap::new();
    for clause in &project.human_clauses {
        let bytes = captures.pinned(
            &format!("human-clause-{}", clause.key),
            &clause.source,
            MAX_CLAUSE_BYTES,
        )?;
        super::render::validate_clause(&bytes)?;
        clauses.insert(clause.key.clone(), LoadedClause { source: clause.clone(), bytes });
    }
    Ok(clauses)
}

fn analyze_snapshot(
    captures: &CaptureSet,
    manifest_path: &Path,
) -> Result<crate::applicability::model::ApplicabilityReport, ForgeError> {
    // The older applicability loader accepts traversal and follows path-based reads. Give it
    // only privately captured files, preserving relative layout and every exact source byte.
    let snapshot = tempfile::tempdir()
        .map_err(|cause| error(format!("cannot create baseline snapshot: {cause}")))?;
    for (path, captured) in &captures.files {
        let target = snapshot.path().join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|cause| error(format!("cannot prepare snapshot: {cause}")))?;
        }
        std::fs::write(target, &captured.bytes)
            .map_err(|cause| error(format!("cannot capture snapshot: {cause}")))?;
    }
    let analysis = crate::applicability::prepare_analysis(
        &snapshot.path().join(manifest_path),
        ReportFilters::default(),
    )
    .map_err(|cause| error(format!("baseline analysis: {cause}")))?;
    Ok(analysis.report)
}

fn strict_json(bytes: &[u8], label: &str) -> Result<serde_json::Value, ForgeError> {
    crate::json_strict::parse_value(
        bytes,
        label,
        // Imported artifacts retain their upstream semantic bounds. The confined
        // capture already bounds the complete byte buffer; authoring's narrower
        // string limit applies only to the new pack/project contracts.
        crate::json_strict::Limits { max_depth: 128, max_string_bytes: bytes.len() },
    )
    .map_err(|cause| error(cause.to_string()))
}

fn portable_label(path: &Path) -> Result<String, ForgeError> {
    manifest::validate_local_path("captured input", path)?;
    let raw = path.to_str().ok_or_else(|| error("input paths must be UTF-8"))?;
    if raw.is_empty() || raw.contains(['\\', ':', '\0']) || path.is_absolute() {
        return Err(error("input paths must be portable relative descendants"));
    }
    if path.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(error("input path is not a canonical descendant"));
    }
    Ok(raw.to_owned())
}

fn contained_dependency(base: &Path, relative: &Path) -> Result<PathBuf, ForgeError> {
    let raw = relative.to_str().ok_or_else(|| error("baseline paths must be UTF-8"))?;
    if raw.contains(['\\', ':', '\0']) || relative.is_absolute() || raw.is_empty() {
        return Err(error("baseline dependencies must be portable local paths"));
    }
    let base_raw = base.to_str().ok_or_else(|| error("baseline directory must be UTF-8"))?;
    if !base_raw.is_empty() {
        manifest::validate_local_path("baseline directory", base)?;
    }
    let mut normalized =
        if base_raw.is_empty() { Vec::new() } else { base_raw.split('/').collect::<Vec<_>>() };
    let mut descended = false;
    // Only leading parent hops from a nested manifest are supported. Discarding
    // an internal `sub/..` would hide whether `sub` is a symlink or invalid path,
    // and would change how the unchanged source reference resolves in a snapshot.
    for component in raw.split('/') {
        match component {
            ".." if !descended => {
                if normalized.pop().is_none() {
                    return Err(error("baseline dependency escapes the author project root"));
                }
            }
            "" | "." | ".." => {
                return Err(error("baseline dependency has a non-canonical path spelling"));
            }
            name => {
                descended = true;
                normalized.push(name);
            }
        }
    }
    // PathBuf::push would introduce native backslashes on Windows, while the
    // captured input label must keep the portable spelling from the contract.
    let normalized = PathBuf::from(normalized.join("/"));
    portable_label(&normalized)?;
    Ok(normalized)
}

pub(super) fn absolute_manifest_path(path: &Path) -> Result<PathBuf, ForgeError> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()
            .map_err(|cause| error(format!("cannot determine current directory: {cause}")))?
    };
    let mut absolute = base;
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return Err(error("manifest path must not contain '..'")),
            _ => absolute.push(component.as_os_str()),
        }
    }
    Ok(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_parent_references_remain_contained() {
        assert_eq!(
            contained_dependency(Path::new("baselines"), Path::new("../framework.json")).unwrap(),
            Path::new("framework.json")
        );
        assert!(contained_dependency(Path::new(""), Path::new("../outside.json")).is_err());
        assert!(
            contained_dependency(Path::new("baselines"), Path::new("../../outside.json")).is_err()
        );
        assert!(contained_dependency(Path::new(""), Path::new("C:\\outside.json")).is_err());
        for invalid in [
            "CON.json",
            "dir./catalog.json",
            "a?.json",
            "line\nbreak.json",
            "sub/../framework.json",
            "./framework.json",
            "sub//framework.json",
            "../sub/../framework.json",
            "framework.json/",
        ] {
            assert!(contained_dependency(Path::new("nested"), Path::new(invalid)).is_err());
        }
    }

    #[test]
    fn baseline_dependencies_preserve_exact_portable_separators_and_utf8() {
        for (base, relative, expected) in [
            ("", "resources/framework.json", "resources/framework.json"),
            ("baselines", "framework.json", "baselines/framework.json"),
            ("baselines", "resources/framework.json", "baselines/resources/framework.json"),
            (
                "baselines/nested",
                "../resources/framework.json",
                "baselines/resources/framework.json",
            ),
            ("baselines/nested", "../../resources/framework.json", "resources/framework.json"),
            ("baselines/équipe", "../資料/é.json", "baselines/資料/é.json"),
        ] {
            let actual = contained_dependency(Path::new(base), Path::new(relative)).unwrap();
            assert_eq!(actual.to_str(), Some(expected), "{base} + {relative}");
            assert_eq!(portable_label(&actual).unwrap(), expected);
        }
    }

    #[test]
    fn baseline_dependencies_reject_noncanonical_base_directory_spellings() {
        for base in ["./baselines", "baselines//nested", "baselines/../nested", "base\\nested"] {
            assert!(contained_dependency(Path::new(base), Path::new("framework.json")).is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn baseline_dependencies_reject_non_utf8_without_lossy_conversion() {
        use std::os::unix::ffi::OsStrExt;

        let invalid = Path::new(std::ffi::OsStr::from_bytes(b"invalid-\xff"));
        assert!(contained_dependency(invalid, Path::new("framework.json")).is_err());
        assert!(contained_dependency(Path::new("baselines"), invalid).is_err());
    }

    #[test]
    fn prepared_inputs_detect_changes_before_output() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("source.json"), b"old").unwrap();
        let mut capture = CaptureSet::new(root.clone());
        capture.read("source", Path::new("source.json"), None, 20).unwrap();
        std::fs::write(root.join("source.json"), b"new").unwrap();
        assert!(capture.verify().is_err());
    }

    #[test]
    fn captures_reject_case_collisions_before_opening_an_alias() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("framework.json"), b"{}").unwrap();
        let mut captures = CaptureSet::new(root);
        captures.read("framework", Path::new("framework.json"), None, 20).unwrap();
        let failure = captures.read("mapping", Path::new("FRAMEWORK.json"), None, 20).unwrap_err();
        assert!(failure.to_string().contains("aliases another input"));
    }

    #[test]
    fn remaining_source_budget_limits_each_read_before_capture() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("source.json"), b"123").unwrap();
        let original = crate::linkage::read_confined_local_file(&root, Path::new("source.json"), 2)
            .unwrap_err()
            .to_string();
        let mut captures = CaptureSet::new(root);
        captures.byte_count = MAX_TOTAL_BYTES - 2;
        let failure = captures.read("source", Path::new("source.json"), None, 100).unwrap_err();
        let message = failure.to_string();
        assert!(message.contains("2 bytes remaining in the 50 MiB total source budget"));
        assert!(message.contains("per-file limit: 100 bytes"));
        assert!(message.ends_with(&original));
        assert!(captures.files.is_empty());
        assert!(captures.identities.is_empty());
        assert_eq!(captures.byte_count, MAX_TOTAL_BYTES - 2);
    }

    #[test]
    fn remaining_source_budget_accepts_an_exactly_fitting_input() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("source.json"), b"{}").unwrap();
        let mut captures = CaptureSet::new(root);
        captures.byte_count = MAX_TOTAL_BYTES - 2;
        assert_eq!(captures.read("source", Path::new("source.json"), None, 100).unwrap(), b"{}");
        assert_eq!(captures.byte_count, MAX_TOTAL_BYTES);
        assert_eq!(captures.files.len(), 1);
        assert_eq!(captures.identities.len(), 1);
        assert_eq!(captures.files["source.json"].max_bytes, 2);
        captures.verify().unwrap();
    }

    #[test]
    fn exhausted_source_budget_is_reported_before_reading_another_input() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut captures = CaptureSet::new(root);
        captures.byte_count = MAX_TOTAL_BYTES;
        let failure =
            captures.read("next source", Path::new("missing.json"), None, 100).unwrap_err();
        assert!(failure.to_string().contains(
            "next source: captured project inputs have exhausted the 50 MiB total source budget"
        ));
        assert!(captures.files.is_empty());
        assert!(captures.identities.is_empty());
        assert_eq!(captures.byte_count, MAX_TOTAL_BYTES);
    }

    #[test]
    fn partial_source_budget_keeps_the_original_missing_file_cause() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let original =
            crate::linkage::read_confined_local_file(&root, Path::new("missing.json"), 2)
                .unwrap_err()
                .to_string();
        let mut captures = CaptureSet::new(root);
        captures.byte_count = MAX_TOTAL_BYTES - 2;
        let failure = captures.read("source", Path::new("missing.json"), None, 100).unwrap_err();
        let message = failure.to_string();
        assert!(message.contains("2 bytes remaining in the 50 MiB total source budget"));
        assert!(message.ends_with(&original));
        assert!(!message.contains("exhausted"));
        assert!(captures.files.is_empty());
        assert!(captures.identities.is_empty());
        assert_eq!(captures.byte_count, MAX_TOTAL_BYTES - 2);
    }

    #[test]
    fn captured_identity_rejects_unicode_aliases_on_normalizing_filesystems() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        std::fs::write(root.join("Ä.json"), b"{}").unwrap();
        let mut captures = CaptureSet::new(root.clone());
        captures.read("framework", Path::new("Ä.json"), None, 20).unwrap();
        for alias in ["ä.json", "A\u{308}.json"] {
            let exists = root.join(alias).exists();
            let failure = captures.read("mapping", Path::new(alias), None, 20).unwrap_err();
            if exists {
                assert!(failure.to_string().contains("aliases another input file"));
            }
        }
    }
}
