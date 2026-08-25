//! Human-reviewed OSCAL Control Mapping build and impact-check workflows.

pub mod baseline;
pub mod inventory;
pub mod manifest;
pub mod model;

use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use crate::cli::{MappingFailOn, MappingReportFormat};
use crate::{ForgeError, io, validate};

struct PreparedBuild {
    artifact_json: String,
    report: model::MappingReport,
    input_paths: Vec<PathBuf>,
}

/// Build a Mapping artifact and optional separate review report.
///
/// # Errors
///
/// Returns [`ForgeError`] when any input, validation, baseline, alias, serialization, or write
/// check fails. No output is written before all trust checks complete.
pub fn execute_build(
    manifest_path: &Path,
    output: Option<&Path>,
    report_path: Option<&Path>,
    report_format: &MappingReportFormat,
    baseline_path: Option<&Path>,
    fail_on: &MappingFailOn,
    include_excerpts: bool,
) -> Result<bool, ForgeError> {
    let prepared = prepare(manifest_path, baseline_path, include_excerpts)?;
    validate_destinations(&prepared.input_paths, output, report_path)?;
    crate::cli::output::write_output(&prepared.artifact_json, output)?;
    if let Some(path) = report_path {
        let rendered = render_report(&prepared.report, report_format)?;
        crate::cli::output::write_output(&rendered, Some(path))?;
    }
    Ok(review_required(&prepared.report, fail_on))
}

/// Perform read-only Mapping baseline impact analysis.
///
/// # Errors
///
/// Returns [`ForgeError`] when current inputs or the baseline cannot be analyzed completely.
pub fn execute_check(
    manifest_path: &Path,
    baseline_path: &Path,
    report_path: Option<&Path>,
    report_format: &MappingReportFormat,
    fail_on: &MappingFailOn,
    include_excerpts: bool,
) -> Result<bool, ForgeError> {
    let prepared = prepare(manifest_path, Some(baseline_path), include_excerpts)?;
    validate_destinations(&prepared.input_paths, None, report_path)?;
    let rendered = render_report(&prepared.report, report_format)?;
    crate::cli::output::write_output(&rendered, report_path)?;
    Ok(review_required(&prepared.report, fail_on))
}

/// Create a deterministic, unapproved manifest skeleton with inventories and fingerprints.
///
/// # Errors
///
/// Returns [`ForgeError`] for invalid resources, missing Profile companions, unsafe aliases, or
/// serialization/write failures.
pub fn execute_init(
    source_path: &Path,
    target_path: &Path,
    source_resolved_catalog: Option<&Path>,
    target_resolved_catalog: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    let source = scaffold_resource(source_path, source_resolved_catalog, output, "source")?;
    let target = scaffold_resource(target_path, target_resolved_catalog, output, "target")?;
    let mut input_paths = vec![source_path.to_path_buf(), target_path.to_path_buf()];
    input_paths.extend(source_resolved_catalog.map(Path::to_path_buf));
    input_paths.extend(target_resolved_catalog.map(Path::to_path_buf));
    validate_destinations(&input_paths, output, None)?;
    let manifest = manifest::MappingManifest {
        schema_version: manifest::MANIFEST_SCHEMA_VERSION.to_string(),
        collection: manifest::CollectionManifest {
            key: "replace-with-stable-collection-key".to_string(),
            title: "REPLACE WITH REVIEWED MAPPING TITLE".to_string(),
            version: "0.1.0-draft".to_string(),
            last_modified: "REPLACE_WITH_RFC3339_REVIEW_TIME".to_string(),
        },
        reviewers: Vec::new(),
        provenance: manifest::ProvenanceManifest {
            method: manifest::MappingMethod::Human,
            matching_rationale: manifest::MatchingRationale::Semantic,
            status: manifest::MappingStatus::Draft,
            mapping_description: "REPLACE WITH INTENDED USE AND LIMITATIONS".to_string(),
            reviewer_keys: Vec::new(),
            reviewed_at: "REPLACE_WITH_RFC3339_REVIEW_TIME".to_string(),
        },
        mapping: manifest::MappingManifestBody {
            key: "replace-with-stable-mapping-key".to_string(),
            scope: manifest::ReviewScope::ControlPlusStatement,
            method: None,
            matching_rationale: None,
            status: None,
            mapping_description: None,
            confidence_score: None,
            coverage: None,
            source,
            target,
            maps: Vec::new(),
        },
    };
    let mut rendered = serde_json::to_string_pretty(&manifest).map_err(|error| {
        mapping_error(format!("manifest scaffold serialization failed: {error}"))
    })?;
    rendered.push('\n');
    crate::cli::output::write_output(&rendered, output)
}

fn scaffold_resource(
    path: &Path,
    resolved_catalog: Option<&Path>,
    output: Option<&Path>,
    label: &str,
) -> Result<manifest::ResourceManifest, ForgeError> {
    io::check_file_size(path, io::MAX_FILE_SIZE)
        .map_err(|error| mapping_error(format!("{label} resource: {error}")))?;
    let bytes =
        std::fs::read(path).map_err(|error| mapping_error(format!("{label} resource: {error}")))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| mapping_error(format!("{label} resource is not JSON: {error}")))?;
    let resource_type = match validate::detect_model_type(&value)
        .map_err(|error| mapping_error(format!("{label} resource: {error}")))?
    {
        crate::OscalModelType::Catalog => manifest::ResourceType::Catalog,
        crate::OscalModelType::Profile => manifest::ResourceType::Profile,
        other => {
            return Err(mapping_error(format!(
                "{label} resource uses unsupported '{}' model; expected Catalog or Profile",
                other.as_str()
            )));
        }
    };
    if resource_type == manifest::ResourceType::Profile && resolved_catalog.is_none() {
        return Err(mapping_error(format!(
            "--{label}-resolved-catalog is required when {label} is a Profile"
        )));
    }
    let temporary = manifest::ResourceManifest {
        resource_type,
        artifact: path.to_path_buf(),
        href: safe_file_label(path),
        resolved_catalog: resolved_catalog.map(Path::to_path_buf),
        resolved_catalog_attestation: resolved_catalog.map(|_| true),
        expected_sha256: None,
        inventory: None,
    };
    let loaded = inventory::load(Path::new("."), &format!("$.mapping.{label}"), &temporary)?;
    Ok(manifest::ResourceManifest {
        resource_type,
        artifact: manifest_relative_path(path, output)?,
        href: safe_file_label(path),
        resolved_catalog: resolved_catalog
            .map(|companion| manifest_relative_path(companion, output))
            .transpose()?,
        resolved_catalog_attestation: resolved_catalog.map(|_| false),
        expected_sha256: Some(loaded.evidence.raw_sha256.clone()),
        inventory: Some(loaded.snapshot()),
    })
}

fn manifest_relative_path(path: &Path, output: Option<&Path>) -> Result<PathBuf, ForgeError> {
    let target = path.canonicalize().map_err(|error| {
        mapping_error(format!("cannot resolve mapping resource '{}': {error}", path.display()))
    })?;
    let manifest_dir = output
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|error| mapping_error(format!("cannot resolve manifest directory: {error}")))?;
    relative_path(&manifest_dir, &target).ok_or_else(|| {
        mapping_error(format!(
            "cannot express mapping resource '{}' relative to manifest directory '{}'",
            path.display(),
            manifest_dir.display()
        ))
    })
}

fn relative_path(base: &Path, target: &Path) -> Option<PathBuf> {
    let base_components: Vec<_> = base.components().collect();
    let target_components: Vec<_> = target.components().collect();
    let common = base_components
        .iter()
        .zip(&target_components)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return None;
    }
    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        if matches!(component, Component::Normal(_)) {
            relative.push("..");
        }
    }
    for component in &target_components[common..] {
        relative.push(component.as_os_str());
    }
    Some(relative)
}

fn safe_file_label(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| "resource.json".to_string(), |name| name.to_string_lossy().into_owned())
}

fn prepare(
    manifest_path: &Path,
    baseline_path: Option<&Path>,
    include_excerpts: bool,
) -> Result<PreparedBuild, ForgeError> {
    io::check_file_size(manifest_path, manifest::MAX_MANIFEST_BYTES)
        .map_err(|error| mapping_error(format!("manifest: {error}")))?;
    let manifest_bytes = std::fs::read(manifest_path)
        .map_err(|error| mapping_error(format!("manifest: {error}")))?;
    let manifest = manifest::parse(&manifest_bytes)?;
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let source = inventory::load(manifest_dir, "$.mapping.source", &manifest.mapping.source)?;
    let target = inventory::load(manifest_dir, "$.mapping.target", &manifest.mapping.target)?;
    let mut product = model::build(&manifest, &source, &target, include_excerpts)?;
    let artifact_value = serde_json::to_value(&product.artifact)
        .map_err(|error| mapping_error(format!("typed Mapping serialization failed: {error}")))?;
    inventory::validate_schema(
        "completed Mapping",
        &artifact_value,
        crate::OscalModelType::Mapping,
    )?;
    product.report.validation.mapping_schema_valid = true;
    if let Some(path) = baseline_path {
        baseline::analyze(
            path,
            &product.artifact,
            &source.inventory,
            &target.inventory,
            &mut product.report,
        )?;
    }
    let mut artifact_json = serde_json::to_string_pretty(&product.artifact)
        .map_err(|error| mapping_error(format!("Mapping JSON serialization failed: {error}")))?;
    artifact_json.push('\n');
    let mut input_paths = vec![manifest_path.to_path_buf(), source.path, target.path];
    for resource in [&manifest.mapping.source, &manifest.mapping.target] {
        if let Some(companion) = &resource.resolved_catalog {
            input_paths.push(manifest_dir.join(companion));
        }
    }
    if let Some(path) = baseline_path {
        input_paths.push(path.to_path_buf());
    }
    Ok(PreparedBuild { artifact_json, report: product.report, input_paths })
}

fn render_report(
    report: &model::MappingReport,
    format: &MappingReportFormat,
) -> Result<String, ForgeError> {
    match format {
        MappingReportFormat::Json => {
            let mut rendered = serde_json::to_string_pretty(report)
                .map_err(|error| mapping_error(format!("report serialization failed: {error}")))?;
            rendered.push('\n');
            Ok(rendered)
        }
        MappingReportFormat::Text => Ok(render_text_report(report)),
    }
}

fn render_text_report(report: &model::MappingReport) -> String {
    let mut output = String::new();
    output.push_str("FORGE Control Mapping review report\n");
    let _ = writeln!(output, "schema: {}", report.schema_version);
    let _ = writeln!(output, "status: {}", report.status);
    let scope = match report.scope {
        manifest::ReviewScope::ControlOnly => "control-only",
        manifest::ReviewScope::ControlPlusStatement => "control-plus-statement",
    };
    let _ = writeln!(output, "scope: {scope}");
    append_resource_evidence(&mut output, "source", &report.source);
    append_resource_evidence(&mut output, "target", &report.target);
    append_participation(&mut output, "source controls", &report.source_controls);
    append_participation(&mut output, "target controls", &report.target_controls);
    append_participation(&mut output, "source statements", &report.source_statements);
    append_participation(&mut output, "target statements", &report.target_statements);
    let _ = writeln!(
        output,
        "validation: manifest={} resources={} references={} mapping-schema={}",
        report.validation.manifest_valid,
        report.validation.resources_valid,
        report.validation.references_valid,
        report.validation.mapping_schema_valid
    );
    for estimate in &report.author_estimates {
        append_author_estimate(&mut output, estimate);
    }
    let _ = writeln!(output, "review findings: {}", report.findings.len());
    for finding in &report.findings {
        let _ = writeln!(
            output,
            "- {} {}: {}",
            escape(&finding.code),
            escape(&finding.path),
            escape(&finding.message)
        );
    }
    if !report.excerpts.is_empty() {
        output.push_str("subject excerpts (sensitive; explicitly requested):\n");
        for excerpt in &report.excerpts {
            let _ = writeln!(
                output,
                "- {} {} {}: {}",
                excerpt.side,
                excerpt.subject_type.as_str(),
                escape(&excerpt.id),
                escape(&excerpt.excerpt)
            );
        }
    }
    output
}

fn append_resource_evidence(
    output: &mut String,
    label: &str,
    evidence: &inventory::ResourceEvidence,
) {
    let _ = writeln!(
        output,
        "{label} resource: type={} href={} raw-sha256={} root-uuid={} document-version={} oscal-version={}",
        evidence.resource_type.as_str(),
        escape(&evidence.href),
        evidence.raw_sha256,
        evidence.root_uuid,
        escape(&evidence.document_version),
        escape(&evidence.oscal_version)
    );
    if let Some(hash) = &evidence.resolved_catalog_sha256 {
        let _ = writeln!(output, "{label} resolved-catalog-sha256: {hash}");
    }
}

fn append_author_estimate(output: &mut String, estimate: &model::AuthorEstimate) {
    let confidence = estimate.confidence.as_ref().map_or_else(
        || "none".to_string(),
        |score| match (score.category, score.percentage) {
            (Some(manifest::ConfidenceCategory::Low), None) => "low".to_string(),
            (Some(manifest::ConfidenceCategory::Medium), None) => "medium".to_string(),
            (Some(manifest::ConfidenceCategory::High), None) => "high".to_string(),
            (None, Some(percentage)) => format!("{percentage}"),
            _ => "invalid".to_string(),
        },
    );
    let target_coverage = estimate
        .target_coverage
        .map_or_else(|| "none".to_string(), |coverage| coverage.to_string());
    let _ = writeln!(
        output,
        "author estimate: map-key={} label={} confidence={} target-coverage={}",
        escape(&estimate.map_key),
        estimate.label,
        confidence,
        target_coverage
    );
}

fn append_participation(output: &mut String, label: &str, participation: &model::Participation) {
    let percent = participation.ratio * 100.0;
    let _ = writeln!(
        output,
        "{label} review participation: {}/{} ({percent:.2}%)",
        participation.referenced, participation.eligible
    );
    if !participation.unmapped_ids.is_empty() {
        let _ = writeln!(
            output,
            "{label} unmapped IDs: {}",
            participation.unmapped_ids.iter().map(|id| escape(id)).collect::<Vec<_>>().join(", ")
        );
    }
}

fn review_required(report: &model::MappingReport, fail_on: &MappingFailOn) -> bool {
    match fail_on {
        MappingFailOn::Never => false,
        MappingFailOn::Any => !report.findings.is_empty(),
        MappingFailOn::Stale => report.findings.iter().any(|finding| {
            matches!(finding.code.as_str(), "stale_reference" | "subject_type_changed")
        }),
        MappingFailOn::SubjectChange => {
            report.findings.iter().any(|finding| finding.code == "subject_changed")
        }
        MappingFailOn::GapIncrease => {
            report.findings.iter().any(|finding| finding.code == "new_gap")
        }
    }
}

fn validate_destinations(
    inputs: &[PathBuf],
    output: Option<&Path>,
    report: Option<&Path>,
) -> Result<(), ForgeError> {
    let destinations: Vec<_> = [output, report].into_iter().flatten().collect();
    for destination in &destinations {
        for input in inputs {
            if paths_alias(destination, input)? {
                return Err(mapping_error(format!(
                    "destination '{}' aliases a mapping input",
                    destination.display()
                )));
            }
        }
    }
    if destinations.len() == 2 && paths_alias(destinations[0], destinations[1])? {
        return Err(mapping_error("--output and --report must be different files"));
    }
    Ok(())
}

fn paths_alias(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    if path_identity(left)? == path_identity(right)? {
        return Ok(true);
    }
    if !left.exists() || !right.exists() {
        return Ok(false);
    }
    same_file_identity(left, right)
}

#[cfg(unix)]
fn same_file_identity(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    use std::os::unix::fs::MetadataExt;

    let left_metadata = std::fs::metadata(left)
        .map_err(|error| mapping_error(format!("cannot inspect '{}': {error}", left.display())))?;
    let right_metadata = std::fs::metadata(right)
        .map_err(|error| mapping_error(format!("cannot inspect '{}': {error}", right.display())))?;
    Ok(left_metadata.dev() == right_metadata.dev() && left_metadata.ino() == right_metadata.ino())
}

#[cfg(windows)]
fn same_file_identity(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    windows_file_identity::same_file(left, right)
        .map_err(|error| mapping_error(format!("cannot compare file identities: {error}")))
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(_left: &Path, _right: &Path) -> Result<bool, ForgeError> {
    Ok(false)
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_file_identity {
    use std::ffi::c_void;
    use std::fs::File;
    use std::io;
    use std::mem::MaybeUninit;
    use std::os::windows::io::AsRawHandle;
    use std::path::Path;

    #[repr(C)]
    struct FileTime {
        low_date_time: u32,
        high_date_time: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    pub(super) fn same_file(left: &Path, right: &Path) -> io::Result<bool> {
        let left = identity(&File::open(left)?)?;
        let right = identity(&File::open(right)?)?;
        Ok(left == right)
    }

    fn identity(file: &File) -> io::Result<(u32, u64)> {
        let mut information = MaybeUninit::<ByHandleFileInformation>::uninit();
        // SAFETY: `file` remains open for the call, its raw handle is valid, and `information`
        // points to writable storage with the exact documented C layout. The value is read only
        // after the API reports success and has initialized the complete structure.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: the successful API call above initialized the complete structure.
        let information = unsafe { information.assume_init() };
        let file_index =
            (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
        Ok((information.volume_serial_number, file_index))
    }
}

fn path_identity(path: &Path) -> Result<PathBuf, ForgeError> {
    if path.exists() {
        return path.canonicalize().map_err(|error| {
            mapping_error(format!("cannot resolve '{}': {error}", path.display()))
        });
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let canonical_parent = parent.canonicalize().map_err(|error| {
        mapping_error(format!("cannot resolve parent of '{}': {error}", path.display()))
    })?;
    Ok(canonical_parent.join(path.file_name().unwrap_or_default()))
}

fn escape(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn mapping_error(message: impl Into<String>) -> ForgeError {
    ForgeError::MappingBuild(message.into())
}
