//! Human-authored, deterministic OSCAL Assessment Results workflow (PRD 063).

pub mod baseline;
pub mod context;
pub mod manifest;
pub mod model;
pub mod report;

use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::cli::{AssessmentResultsFailOn, AssessmentResultsReportFormat};
use crate::{ForgeError, io};

struct PreparedBuild {
    artifact_json: String,
    report: report::AssessmentResultsReport,
    input_paths: Vec<PathBuf>,
}

/// Execute `forge assessment results build`.
///
/// Returns `true` only when a valid baseline comparison produced review actions
/// selected by the exit gate. All input, schema, graph, alias, and serialization
/// checks complete before any destination is modified.
///
/// # Errors
///
/// Returns an error when preparation, validation, destination safety, rendering,
/// or an atomic output write fails.
pub fn execute_build(
    manifest_path: &Path,
    output: Option<&Path>,
    report_path: Option<&Path>,
    report_format: &AssessmentResultsReportFormat,
    baseline_path: Option<&Path>,
    fail_on: &AssessmentResultsFailOn,
) -> Result<bool, ForgeError> {
    let prepared = prepare(manifest_path, baseline_path)?;
    validate_destinations(&prepared.input_paths, output, report_path)?;
    let rendered_report = report::render(&prepared.report, report_format)?;
    if baseline_path.is_some() && report_path.is_none() {
        write_stderr(&rendered_report)?;
    }
    crate::cli::output::write_output(&prepared.artifact_json, output)?;
    if let Some(path) = report_path {
        crate::cli::output::write_output(&rendered_report, Some(path))?;
    }
    Ok(baseline_path.is_some()
        && !prepared.report.findings.is_empty()
        && matches!(fail_on, AssessmentResultsFailOn::Any))
}

/// Create a context-bound manifest scaffold with no observations, findings, or risks.
///
/// # Errors
///
/// Returns an error when any companion is unsafe, invalid, stale, inconsistent,
/// outside the scaffold directory, or when the destination cannot be written safely.
#[allow(clippy::too_many_arguments)]
#[allow(
    clippy::too_many_lines,
    reason = "scaffold assembly mirrors the exact four-artifact import chain in one sequence"
)]
pub fn execute_init(
    assessment_plan_path: &Path,
    ssp_path: &Path,
    profile_path: &Path,
    catalog_path: &Path,
    evidence_index_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let manifest_dir = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let manifest_dir = manifest_dir.canonicalize().map_err(|cause| {
        error(format!(
            "cannot resolve scaffold output directory '{}': {cause}",
            manifest_dir.display()
        ))
    })?;
    let assessment_plan = scaffold_artifact(
        &manifest_dir,
        assessment_plan_path,
        "assessment-plan",
        confined_href(&manifest_dir, assessment_plan_path, "assessment-plan")?,
    )?;
    let assessment_plan_value = read_json(assessment_plan_path, "Assessment Plan")?;
    let ssp_href = required_pointer(
        &assessment_plan_value,
        "/assessment-plan/import-ssp/href",
        "Assessment Plan import-ssp href",
    )?;
    let ssp = scaffold_artifact(&manifest_dir, ssp_path, "system-security-plan", ssp_href)?;
    let ssp_value = read_json(ssp_path, "SSP")?;
    let profile_href = required_pointer(
        &ssp_value,
        "/system-security-plan/import-profile/href",
        "SSP import-profile href",
    )?;
    let profile = scaffold_artifact(&manifest_dir, profile_path, "profile", profile_href)?;
    let profile_value = read_json(profile_path, "Profile")?;
    let imports = profile_value
        .pointer("/profile/imports")
        .and_then(Value::as_array)
        .ok_or_else(|| error("Profile imports are required for scaffold creation"))?;
    if imports.len() != 1 {
        return Err(error(
            "Assessment Results MVP scaffold requires exactly one Profile Catalog import",
        ));
    }
    let catalog_href =
        required_pointer(&profile_value, "/profile/imports/0/href", "Profile Catalog import href")?;
    let catalog = scaffold_artifact(&manifest_dir, catalog_path, "catalog", catalog_href)?;
    let evidence_index = evidence_index_path
        .map(|path| {
            let artifact = confined_relative(&manifest_dir, path, "evidence index")?;
            let bytes = io::read_bounded(path, io::MAX_FILE_SIZE)
                .map_err(|cause| error(format!("cannot read evidence index: {cause}")))?;
            Ok::<_, ForgeError>(manifest::EvidenceIndexManifest {
                artifact,
                expected_sha256: sha256(&bytes),
            })
        })
        .transpose()?;

    let context_manifest =
        manifest::ContextManifest { assessment_plan, ssp, profile, catalog, evidence_index };
    let loaded_context = context::load_from_root(&manifest_dir, &context_manifest)?;
    let mut control_ids: Vec<_> = loaded_context.reviewed_controls.iter().cloned().collect();
    let mut objective_ids: Vec<_> = loaded_context.reviewed_objectives.iter().cloned().collect();
    control_ids.sort();
    objective_ids.sort();

    let draft = manifest::AssessmentResultsManifest {
        schema_version: manifest::MANIFEST_SCHEMA_VERSION.to_string(),
        document: manifest::DocumentManifest {
            key: "replace-with-stable-assessment-results-key".to_string(),
            title: "REPLACE WITH ASSESSMENT RESULTS TITLE".to_string(),
            version: "0.1.0-draft".to_string(),
            last_modified: "REPLACE_WITH_RFC3339_REVIEW_TIME".to_string(),
        },
        context: context_manifest,
        roles: vec![manifest::RoleManifest {
            id: "assessor".to_string(),
            title: "Assessor".to_string(),
        }],
        parties: vec![manifest::PartyManifest {
            key: "replace-with-assessor-key".to_string(),
            party_type: manifest::PartyType::Person,
            name: "REPLACE WITH ASSESSOR NAME".to_string(),
        }],
        result: manifest::ResultManifest {
            key: "replace-with-stable-result-key".to_string(),
            title: "REPLACE WITH RESULT EPOCH TITLE".to_string(),
            description:
                "Draft scope scaffold only; no observations, findings, or risks have been recorded."
                    .to_string(),
            start: "REPLACE_WITH_RFC3339_ASSESSMENT_START".to_string(),
            end: None,
            control_ids,
            objective_ids,
            observations: Vec::new(),
            findings: Vec::new(),
            risks: Vec::new(),
            relationships: Vec::new(),
        },
    };
    let mut rendered = serde_json::to_string_pretty(&draft)
        .map_err(|cause| error(format!("manifest scaffold serialization failed: {cause}")))?;
    rendered.push('\n');
    let mut inputs = vec![
        assessment_plan_path.to_path_buf(),
        ssp_path.to_path_buf(),
        profile_path.to_path_buf(),
        catalog_path.to_path_buf(),
    ];
    inputs.extend(evidence_index_path.map(Path::to_path_buf));
    validate_destinations(&inputs, output, None)?;
    crate::cli::output::write_output(&rendered, output)
}

fn prepare(
    manifest_path: &Path,
    baseline_path: Option<&Path>,
) -> Result<PreparedBuild, ForgeError> {
    let manifest_bytes = io::read_bounded(manifest_path, manifest::MAX_MANIFEST_BYTES)
        .map_err(|cause| error(format!("cannot read manifest: {cause}")))?;
    let manifest = manifest::parse(&manifest_bytes)?;
    let context = context::load(manifest_path, &manifest.context)?;
    let built = model::build(&manifest, &context)?;
    let artifact_value = serde_json::to_value(&built.artifact).map_err(|cause| {
        error(format!("typed Assessment Results serialization failed: {cause}"))
    })?;
    validate_completed_json(&artifact_value)?;

    let mut review_report = report::AssessmentResultsReport::new(
        &manifest,
        built.artifact.assessment_results.uuid.clone(),
        context.artifact_identities().into_iter().map(report::ContextSummary::from),
    );
    if let Some(path) = baseline_path {
        let bytes = io::read_bounded(path, io::MAX_FILE_SIZE)
            .map_err(|cause| error(format!("cannot read baseline: {cause}")))?;
        baseline::analyze(&bytes, &built.object_snapshots, &context, &mut review_report)?;
    }
    review_report.finalize();
    let mut artifact_json = serde_json::to_string_pretty(&built.artifact)
        .map_err(|cause| error(format!("Assessment Results JSON serialization failed: {cause}")))?;
    artifact_json.push('\n');
    let mut input_paths = vec![manifest_path.to_path_buf()];
    input_paths.extend(context.input_paths);
    input_paths.extend(baseline_path.map(Path::to_path_buf));
    Ok(PreparedBuild { artifact_json, report: review_report, input_paths })
}

/// Validate a completed artifact against the pristine pinned v1.2.3 schema.
pub(crate) fn validate_completed_json(value: &Value) -> Result<(), ForgeError> {
    let schema: Value =
        serde_json::from_str(include_str!("../../schemas/oscal_assessment-results_schema.json"))
            .map_err(|cause| {
                error(format!("vendored Assessment Results schema is invalid: {cause}"))
            })?;
    let validator = jsonschema::validator_for(&schema).map_err(|cause| {
        error(format!("vendored Assessment Results schema failed to compile: {cause}"))
    })?;
    let mut errors: Vec<_> = validator
        .iter_errors(value)
        .take(101)
        .map(|schema_error| schema_error.to_string())
        .collect();
    let truncated = errors.len() > 100;
    errors.truncate(100);
    if errors.is_empty() {
        return Ok(());
    }
    let mut detail = errors.into_iter().take(10).collect::<Vec<_>>().join("; ");
    if truncated {
        detail.push_str("; additional schema errors omitted at configured bound");
    }
    Err(error(format!(
        "completed Assessment Results failed the pinned official OSCAL 1.2.3 schema: {detail}"
    )))
}

fn scaffold_artifact(
    manifest_dir: &Path,
    path: &Path,
    root_name: &str,
    href: String,
) -> Result<manifest::ArtifactManifest, ForgeError> {
    let artifact = confined_relative(manifest_dir, path, root_name)?;
    let bytes = io::read_bounded(path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("cannot read {root_name}: {cause}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|cause| error(format!("{root_name} is not valid JSON: {cause}")))?;
    let root = value.get(root_name).ok_or_else(|| error(format!("expected '{root_name}' root")))?;
    let root_uuid = required_value(root.get("uuid"), &format!("{root_name}.uuid"))?;
    let document_version = required_value(
        root.pointer("/metadata/version"),
        &format!("{root_name}.metadata.version"),
    )?;
    let oscal_version = required_value(
        root.pointer("/metadata/oscal-version"),
        &format!("{root_name}.metadata.oscal-version"),
    )?;
    Ok(manifest::ArtifactManifest {
        artifact,
        href,
        expected_sha256: sha256(&bytes),
        root_uuid,
        document_version,
        oscal_version,
    })
}

fn read_json(path: &Path, label: &str) -> Result<Value, ForgeError> {
    let bytes = io::read_bounded(path, io::MAX_FILE_SIZE)
        .map_err(|cause| error(format!("cannot read {label}: {cause}")))?;
    serde_json::from_slice(&bytes).map_err(|cause| error(format!("{label} is not JSON: {cause}")))
}

fn required_pointer(value: &Value, pointer: &str, label: &str) -> Result<String, ForgeError> {
    required_value(value.pointer(pointer), label)
}

fn required_value(value: Option<&Value>, label: &str) -> Result<String, ForgeError> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| error(format!("{label} must be a non-empty string")))
}

fn confined_relative(root: &Path, path: &Path, label: &str) -> Result<PathBuf, ForgeError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|cause| error(format!("cannot inspect {label} '{}': {cause}", path.display())))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(error(format!("{label} must be a regular non-symlink file")));
    }
    let canonical = path
        .canonicalize()
        .map_err(|cause| error(format!("cannot resolve {label} '{}': {cause}", path.display())))?;
    let relative = canonical.strip_prefix(root).map_err(|_| {
        error(format!(
            "{label} must be inside the manifest output directory for a confined scaffold"
        ))
    })?;
    if relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(error(format!("{label} cannot be expressed as a safe relative path")));
    }
    Ok(relative.to_path_buf())
}

fn validate_destinations(
    inputs: &[PathBuf],
    output: Option<&Path>,
    report: Option<&Path>,
) -> Result<(), ForgeError> {
    let destinations: Vec<_> = [output, report].into_iter().flatten().collect();
    for destination in &destinations {
        for input in inputs {
            if crate::mapping::paths_alias(destination, input).map_err(|cause| {
                error(format!("cannot verify destination alias safety: {cause}"))
            })? {
                return Err(error(format!(
                    "destination '{}' aliases an Assessment Results input",
                    destination.display()
                )));
            }
        }
    }
    if destinations.len() == 2
        && crate::mapping::paths_alias(destinations[0], destinations[1])
            .map_err(|cause| error(format!("cannot verify output/report alias safety: {cause}")))?
    {
        return Err(error("--output and --report must be different files"));
    }
    Ok(())
}

fn write_stderr(content: &str) -> Result<(), ForgeError> {
    let mut stderr = std::io::stderr().lock();
    stderr
        .write_all(content.as_bytes())
        .and_then(|()| stderr.flush())
        .map_err(|cause| error(format!("failed writing baseline report to stderr: {cause}")))
}

fn confined_href(root: &Path, path: &Path, label: &str) -> Result<String, ForgeError> {
    let relative = confined_relative(root, path, label)?;
    let href = relative
        .to_str()
        .ok_or_else(|| error(format!("{label} path cannot be represented as a UTF-8 href")))?;
    Ok(href.replace('\\', "/"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::AssessmentResultsBuild(message.into())
}
