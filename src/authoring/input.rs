//! Confined input capture and exact PRD-056 baseline verification.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use crate::ForgeError;
use crate::applicability::model::ReportFilters;
use crate::hashing::sha256_hex;

use super::error;
use super::manifest::{self, PinnedFile};
use super::model::{InputFingerprint, LoadedAuthorProject, LoadedClause};

const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_CLAUSE_BYTES: u64 = 1024 * 1024;
const MAX_SOURCE_BYTES: usize = 50 * 1024 * 1024;

pub(super) struct PreparedProject {
    pub root: PathBuf,
    pub loaded: LoadedAuthorProject,
    captures: CaptureSet,
}

struct CapturedFile {
    fingerprint: InputFingerprint,
    bytes: Vec<u8>,
    identity: (u64, u64),
    max_bytes: u64,
}

struct CaptureSet {
    root: PathBuf,
    files: BTreeMap<String, CapturedFile>,
    identities: BTreeSet<(u64, u64)>,
    byte_count: usize,
}

impl CaptureSet {
    fn new(root: PathBuf) -> Self {
        Self { root, files: BTreeMap::new(), identities: BTreeSet::new(), byte_count: 0 }
    }

    fn read(
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
        let (bytes, identity) =
            crate::linkage::read_confined_local_file(&self.root, path, max_bytes)
                .map_err(|cause| error(format!("{role}: {cause}")))?;
        let digest = sha256_hex(&bytes);
        if expected_sha256.is_some_and(|expected| expected != digest) {
            return Err(error(format!("{role} SHA-256 does not match its exact byte pin")));
        }
        if !self.identities.insert(identity) {
            return Err(error(format!("{role} aliases another input file")));
        }
        self.byte_count = self
            .byte_count
            .checked_add(bytes.len())
            .ok_or_else(|| error("input byte count overflow"))?;
        if self.byte_count > MAX_SOURCE_BYTES {
            return Err(error("captured project inputs exceed the 50 MiB total limit"));
        }
        let fingerprint = InputFingerprint {
            role: role.to_owned(),
            path: label.clone(),
            sha256: digest,
            byte_length: bytes.len() as u64,
        };
        self.files
            .insert(label, CapturedFile { fingerprint, bytes: bytes.clone(), identity, max_bytes });
        Ok(bytes)
    }

    fn pinned(&mut self, role: &str, pin: &PinnedFile, limit: u64) -> Result<Vec<u8>, ForgeError> {
        self.read(role, &pin.path, Some(&pin.expected_sha256), limit)
    }

    fn fingerprints(&self) -> Vec<InputFingerprint> {
        let mut entries: Vec<_> =
            self.files.values().map(|file| file.fingerprint.clone()).collect();
        entries.sort_by(|left, right| (&left.role, &left.path).cmp(&(&right.role, &right.path)));
        entries
    }

    fn verify(&self) -> Result<(), ForgeError> {
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
    /// Reopen every source through the same confined traversal before publication.
    pub(super) fn verify_inputs(&self) -> Result<(), ForgeError> {
        self.captures.verify()
    }
}

pub(super) fn prepare(manifest_path: &Path) -> Result<PreparedProject, ForgeError> {
    let absolute = absolute_manifest_path(manifest_path)?;
    let root = absolute
        .parent()
        .ok_or_else(|| error("manifest must have a parent directory"))?
        .to_path_buf();
    let manifest_name = absolute.file_name().ok_or_else(|| error("manifest must name a file"))?;
    let mut captures = CaptureSet::new(root.clone());
    let project_bytes =
        captures.read("author-project", Path::new(manifest_name), None, MAX_MANIFEST_BYTES)?;
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
        MAX_MANIFEST_BYTES,
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
        crate::json_strict::Limits { max_depth: 128, max_string_bytes: 16 * 1024 },
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
    let mut normalized = base.to_path_buf();
    for component in relative.components() {
        match component {
            Component::Normal(name) => normalized.push(name),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(error("baseline dependency escapes the author project root"));
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(error("absolute baseline dependency is forbidden"));
            }
        }
    }
    portable_label(&normalized)?;
    Ok(normalized)
}

fn absolute_manifest_path(path: &Path) -> Result<PathBuf, ForgeError> {
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
        for invalid in ["CON.json", "dir./catalog.json", "a?.json", "line\nbreak.json"] {
            assert!(contained_dependency(Path::new("nested"), Path::new(invalid)).is_err());
        }
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
}
