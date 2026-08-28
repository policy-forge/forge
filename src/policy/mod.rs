//! Reusable, local, hash-pinned Markdown policy component workflows (PRD 059).

pub mod manifest;
pub mod render;

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use manifest::{ComponentInstance, ComponentManifest, CompositionManifest};
use render::{RenderedComposition, sha256, validate_static_component};
use serde::Serialize;

use crate::{ForgeError, io};

const MAX_COMPONENT_BYTES: u64 = 10 * 1024 * 1024;
const MAX_COMPOSITION_SOURCE_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct LoadedComponent {
    pub(crate) instance: ComponentInstance,
    pub(crate) manifest: ComponentManifest,
    pub(crate) manifest_sha256: String,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) source_sha256: String,
    pub(crate) source_label: String,
}

struct PreparedComposition {
    root: PathBuf,
    outputs: [PathBuf; 3],
    rendered: RenderedComposition,
}

#[derive(Default)]
struct IdentityRegistry {
    identities: BTreeMap<String, String>,
    #[cfg(windows)]
    paths: Vec<(PathBuf, String)>,
}

/// Validate and atomically emit Markdown, lock, and provenance outputs.
///
/// # Errors
///
/// Returns [`ForgeError`] when any input, pin, contract, render, optional conversion, or
/// coordinated output write fails.
pub fn compose(manifest_path: &Path, validate_conversion: bool) -> Result<(), ForgeError> {
    let prepared = prepare_composition(manifest_path)?;
    if validate_conversion {
        validate_existing_conversion_chain(&prepared.root, &prepared.rendered.markdown)?;
    }
    commit_outputs(
        &prepared.outputs,
        [
            prepared.rendered.markdown.as_slice(),
            prepared.rendered.lock.as_slice(),
            prepared.rendered.provenance.as_slice(),
        ],
    )
}

/// Validate the complete composition without creating or replacing outputs.
///
/// # Errors
///
/// Returns [`ForgeError`] when any input, pin, contract, render, or optional conversion fails.
pub fn check_composition(
    manifest_path: &Path,
    validate_conversion: bool,
) -> Result<(), ForgeError> {
    let prepared = prepare_composition(manifest_path)?;
    if validate_conversion {
        validate_existing_conversion_chain(&prepared.root, &prepared.rendered.markdown)?;
    }
    Ok(())
}

/// Validate one component sidecar, its pin, Markdown structure, and placeholder grammar.
///
/// # Errors
///
/// Returns [`ForgeError`] when the sidecar, source path, pin, structure, or grammar is invalid.
pub fn check_component(manifest_path: &Path) -> Result<(), ForgeError> {
    let manifest_path = regular_non_symlink(manifest_path, "component manifest")?;
    let bytes = io::read_bounded(&manifest_path, manifest::MAX_MANIFEST_BYTES)?;
    let manifest = manifest::parse_component(&bytes)?;
    let root = manifest_path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(ForgeError::Io)?;
    let source = resolve_input(&root, &manifest.source, "component source")?;
    let source_bytes = io::read_bounded(&source, MAX_COMPONENT_BYTES)?;
    let actual = sha256(&source_bytes);
    if actual != manifest.expected_sha256 {
        return Err(composition_error(format!(
            "component '{}' SHA-256 mismatch: expected {}, found {actual}",
            manifest.component_key, manifest.expected_sha256
        )));
    }
    let label = manifest.source.to_string_lossy();
    validate_static_component(&manifest, &label, &source_bytes)
}

/// Create a closed draft sidecar for one existing level-two Markdown component.
///
/// # Errors
///
/// Returns [`ForgeError`] when paths alias or escape, the source is invalid, the requested
/// contract fields are invalid, or the sidecar cannot be written atomically.
pub fn scaffold_component(
    source: &Path,
    output: &Path,
    component_key: &str,
    version: &str,
    title: &str,
    owner: &str,
) -> Result<(), ForgeError> {
    let source = regular_non_symlink(source, "component source")?;
    let source_parent =
        source.parent().ok_or_else(|| composition_error("component source has no parent"))?;
    let output_parent = output
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(ForgeError::Io)?;
    if output_parent != source_parent {
        return Err(composition_error(
            "scaffold output must be adjacent to the source so the sidecar contains no parent traversal",
        ));
    }
    if paths_alias(output, &source)? {
        return Err(composition_error("scaffold output aliases the component source"));
    }
    if let Ok(metadata) = std::fs::symlink_metadata(output)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(composition_error("scaffold output must be a regular non-symlink file"));
    }
    let source_bytes = io::read_bounded(&source, MAX_COMPONENT_BYTES)?;
    let source_name = source
        .file_name()
        .ok_or_else(|| composition_error("component source must have a filename"))?;
    let scaffold = ComponentManifest {
        schema_version: manifest::COMPONENT_SCHEMA_VERSION.to_string(),
        component_key: component_key.to_string(),
        version: version.to_string(),
        title: title.to_string(),
        owner: owner.to_string(),
        status: manifest::ComponentStatus::Draft,
        source: PathBuf::from(source_name),
        expected_sha256: sha256(&source_bytes),
        replacement_component_key: None,
        parameters: Vec::new(),
    };
    let bytes = pretty_json(&scaffold)?;
    let validated = manifest::parse_component(&bytes)?;
    validate_static_component(&validated, &source_name.to_string_lossy(), &source_bytes)?;
    io::write_atomic(output, &bytes)
}

#[derive(Debug, Serialize)]
struct ComponentImpactReport {
    schema_version: &'static str,
    component_key: String,
    affected_policy_count: usize,
    affected_instance_count: usize,
    affected_instances: Vec<ComponentDependency>,
}

#[derive(Debug, Serialize)]
struct ComponentDependency {
    composition_manifest: String,
    policy_key: String,
    policy_version: String,
    instance_key: String,
    component_version: String,
    expected_sha256: String,
    current_sha256: String,
    pin_matches: bool,
}

/// Build a deterministic, read-only reverse dependency and component-update impact report.
///
/// # Errors
///
/// Returns [`ForgeError`] when a supplied manifest/resource is invalid or the report cannot be
/// rendered or written.
pub fn component_impact(
    component_key: &str,
    composition_manifests: &[PathBuf],
    output: Option<&Path>,
) -> Result<(), ForgeError> {
    if composition_manifests.is_empty() {
        return Err(composition_error("component impact requires at least one --manifest"));
    }
    let mut dependencies = Vec::new();
    for manifest_path in composition_manifests {
        let manifest_path = regular_non_symlink(manifest_path, "composition manifest")?;
        let bytes = io::read_bounded(&manifest_path, manifest::MAX_MANIFEST_BYTES)?;
        let composition = manifest::parse_composition(&bytes)?;
        let parent = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        let root = resolve_root(parent, &composition.project_root)?;
        let manifest_label = crate::io::sanitize_artifact_path(&manifest_path);
        for instance in &composition.components {
            let sidecar_path =
                resolve_input(&root, &instance.component_manifest, "component manifest")?;
            let sidecar_bytes = io::read_bounded(&sidecar_path, manifest::MAX_MANIFEST_BYTES)?;
            let sidecar = manifest::parse_component(&sidecar_bytes)?;
            if sidecar.component_key != component_key {
                continue;
            }
            let sidecar_parent = sidecar_path.parent().ok_or_else(|| {
                composition_error("component manifest must have a containing directory")
            })?;
            let source =
                resolve_input_from(&root, sidecar_parent, &sidecar.source, "component source")?;
            let source_bytes = io::read_bounded(&source, MAX_COMPONENT_BYTES)?;
            let current_sha256 = sha256(&source_bytes);
            dependencies.push(ComponentDependency {
                composition_manifest: manifest_label.clone(),
                policy_key: composition.policy_key.clone(),
                policy_version: composition.version.clone(),
                instance_key: instance.instance_key.clone(),
                component_version: sidecar.version,
                pin_matches: current_sha256 == sidecar.expected_sha256,
                expected_sha256: sidecar.expected_sha256,
                current_sha256,
            });
        }
    }
    dependencies.sort_by(|left, right| {
        (&left.policy_key, &left.policy_version, &left.composition_manifest, &left.instance_key)
            .cmp(&(
                &right.policy_key,
                &right.policy_version,
                &right.composition_manifest,
                &right.instance_key,
            ))
    });
    let affected_policy_count = dependencies
        .iter()
        .map(|dependency| (&dependency.composition_manifest, &dependency.policy_key))
        .collect::<BTreeSet<_>>()
        .len();
    let report = ComponentImpactReport {
        schema_version: "forge.policy-component-impact/1",
        component_key: component_key.to_string(),
        affected_policy_count,
        affected_instance_count: dependencies.len(),
        affected_instances: dependencies,
    };
    let rendered = String::from_utf8(pretty_json(&report)?).map_err(|source| {
        composition_error(format!("impact serialization was not UTF-8: {source}"))
    })?;
    crate::cli::output::write_output(&rendered, output)
}

/// Resolve assembled source lines in an OSCAL trace report back to component instances.
///
/// # Errors
///
/// Returns [`ForgeError`] when provenance is invalid, stale, oversized, or mismatched to the
/// supplied assembled source.
pub fn format_composition_trace_origins(
    provenance_path: &Path,
    source_path: &Path,
    report: &crate::trace::report::TraceReport,
) -> Result<String, ForgeError> {
    let provenance_bytes = io::read_bounded(provenance_path, io::MAX_FILE_SIZE)?;
    let value = crate::json_strict::parse_value(
        &provenance_bytes,
        "composition provenance",
        crate::json_strict::Limits { max_depth: 32, max_string_bytes: 16 * 1024 },
    )
    .map_err(|source| composition_error(source.to_string()))?;
    let provenance: render::ProvenanceMap = serde_json::from_value(value)
        .map_err(|source| composition_error(format!("invalid composition provenance: {source}")))?;
    if provenance.schema_version != "forge.policy-composition-provenance/1" {
        return Err(composition_error(format!(
            "unsupported composition provenance schema_version '{}'",
            crate::json_strict::bounded(&provenance.schema_version)
        )));
    }
    if provenance.spans.len() > 1_000_000 {
        return Err(composition_error("composition provenance exceeds 1000000 spans"));
    }
    let source_bytes = io::read_bounded(source_path, io::MAX_FILE_SIZE)?;
    if sha256(&source_bytes) != provenance.output_sha256 {
        return Err(composition_error(
            "composition provenance output_sha256 does not match the supplied trace source",
        ));
    }

    let mut rows = BTreeSet::new();
    for entry in &report.entries {
        let Some(line) = entry.trace.as_ref().and_then(|trace| trace.source_line) else {
            continue;
        };
        for span in provenance.spans.iter().filter(|span| span.output.line == line) {
            let (component_file, instance_key, source_line) = match &span.origin {
                render::ProvenanceOrigin::Component {
                    component_file,
                    instance_key,
                    source,
                    ..
                }
                | render::ProvenanceOrigin::Parameter {
                    component_file,
                    instance_key,
                    source,
                    ..
                } => (component_file, instance_key, source.line),
                render::ProvenanceOrigin::GeneratedMetadata { .. } => continue,
            };
            rows.insert((
                entry.element_id.clone(),
                instance_key.clone(),
                component_file.clone(),
                source_line,
            ));
        }
    }
    let mut output = String::from(
        "\nComposition provenance:\nOSCAL Element ID\tInstance Key\tComponent File\tComponent Line\n",
    );
    for (element, instance, component, line) in rows {
        use std::fmt::Write as _;
        let _ = writeln!(
            output,
            "{}\t{}\t{}\t{}",
            crate::sanitize::strip_control_chars(&element),
            crate::sanitize::strip_control_chars(&instance),
            crate::sanitize::strip_control_chars(&component),
            line
        );
    }
    Ok(output)
}

fn prepare_composition(manifest_path: &Path) -> Result<PreparedComposition, ForgeError> {
    let manifest_path = regular_non_symlink(manifest_path, "composition manifest")?;
    let manifest_bytes = io::read_bounded(&manifest_path, manifest::MAX_MANIFEST_BYTES)?;
    let manifest = manifest::parse_composition(&manifest_bytes)?;
    let manifest_parent = manifest_path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let root = resolve_root(manifest_parent, &manifest.project_root)?;
    if !manifest_path.canonicalize().map_err(ForgeError::Io)?.starts_with(&root) {
        return Err(composition_error(
            "composition manifest must be contained by the declared project root",
        ));
    }
    let outputs = [
        resolve_output(&root, &manifest.outputs.markdown, "Markdown output")?,
        resolve_output(&root, &manifest.outputs.lock, "lock output")?,
        resolve_output(&root, &manifest.outputs.provenance, "provenance output")?,
    ];
    let components = load_components(&root, &manifest_path, &manifest, &outputs)?;
    let rendered = render::render(
        &manifest.policy_key,
        &manifest.title,
        &manifest.version,
        &sha256(&manifest_bytes),
        &components,
    )?;
    Ok(PreparedComposition { root, outputs, rendered })
}

fn load_components(
    root: &Path,
    composition_path: &Path,
    composition: &CompositionManifest,
    outputs: &[PathBuf; 3],
) -> Result<Vec<LoadedComponent>, ForgeError> {
    let mut loaded = Vec::with_capacity(composition.components.len());
    let mut identities = IdentityRegistry::default();
    let mut source_owners: BTreeMap<String, (PathBuf, String)> = BTreeMap::new();
    let mut total_source_bytes = 0usize;
    register_identity(&mut identities, composition_path, "composition manifest", false)?;
    for (index, output) in outputs.iter().enumerate() {
        register_identity(&mut identities, output, &format!("output {index}"), false)?;
    }
    for (index, instance) in composition.components.iter().enumerate() {
        let path = resolve_input(root, &instance.component_manifest, "component manifest")?;
        register_identity(
            &mut identities,
            &path,
            &format!("$.components[{index}].component_manifest"),
            true,
        )?;
        let manifest_bytes = io::read_bounded(&path, manifest::MAX_MANIFEST_BYTES)?;
        let component = manifest::parse_component(&manifest_bytes)?;
        let component_parent = path.parent().ok_or_else(|| {
            composition_error(format!("component manifest '{}' has no parent", path.display()))
        })?;
        let source =
            resolve_input_from(root, component_parent, &component.source, "component source")?;
        let component_identity = path.to_string_lossy().to_ascii_lowercase();
        let source_identity = output_identity_key(&source)?;
        #[cfg(windows)]
        for (previous_source, previous_component) in source_owners.values() {
            if source_identity != output_identity_key(previous_source)?
                && paths_alias(&source, previous_source)?
                && previous_component != &component_identity
            {
                return Err(composition_error(format!(
                    "component source for instance '{}' aliases the source of a different component sidecar",
                    instance.instance_key
                )));
            }
        }
        if let Some((_, previous_component)) = source_owners.get(&source_identity) {
            if previous_component != &component_identity {
                return Err(composition_error(format!(
                    "component source for instance '{}' aliases the source of a different component sidecar",
                    instance.instance_key
                )));
            }
        } else {
            source_owners.insert(source_identity, (source.clone(), component_identity));
        }
        register_identity(
            &mut identities,
            &source,
            &format!("component source for instance '{}'", instance.instance_key),
            true,
        )?;
        let source_bytes = io::read_bounded(&source, MAX_COMPONENT_BYTES)?;
        total_source_bytes = total_source_bytes
            .checked_add(source_bytes.len())
            .ok_or_else(|| composition_error("composition source byte count overflowed"))?;
        if total_source_bytes > MAX_COMPOSITION_SOURCE_BYTES {
            return Err(composition_error(format!(
                "composition sources exceed the {MAX_COMPOSITION_SOURCE_BYTES} byte aggregate limit"
            )));
        }
        let source_sha256 = sha256(&source_bytes);
        if source_sha256 != component.expected_sha256 {
            return Err(composition_error(format!(
                "component '{}' SHA-256 mismatch: expected {}, found {source_sha256}",
                component.component_key, component.expected_sha256
            )));
        }
        let source_label = source
            .strip_prefix(root)
            .map_err(|_| composition_error("component source escaped the declared project root"))?
            .to_string_lossy()
            .replace('\\', "/");
        validate_static_component(&component, &source_label, &source_bytes)?;
        loaded.push(LoadedComponent {
            instance: instance.clone(),
            manifest: component,
            manifest_sha256: sha256(&manifest_bytes),
            source_bytes,
            source_sha256,
            source_label,
        });
    }
    Ok(loaded)
}

fn resolve_root(base: &Path, relative: &Path) -> Result<PathBuf, ForgeError> {
    reject_symlink_components(base, relative, "project root")?;
    let path = base.join(relative);
    let metadata = std::fs::symlink_metadata(&path).map_err(ForgeError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(composition_error("declared project root must be a non-symlink directory"));
    }
    path.canonicalize().map_err(ForgeError::Io)
}

fn resolve_input(root: &Path, relative: &Path, label: &str) -> Result<PathBuf, ForgeError> {
    resolve_input_from(root, root, relative, label)
}

fn resolve_input_from(
    root: &Path,
    base: &Path,
    relative: &Path,
    label: &str,
) -> Result<PathBuf, ForgeError> {
    manifest::validate_local_path(label, relative, None)?;
    reject_symlink_components(base, relative, label)?;
    let path = base.join(relative);
    let metadata = std::fs::symlink_metadata(&path).map_err(|source| {
        ForgeError::Io(std::io::Error::new(
            source.kind(),
            format!("cannot inspect {label} '{}': {source}", path.display()),
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(composition_error(format!("{label} must be a regular non-symlink file")));
    }
    let canonical = path.canonicalize().map_err(ForgeError::Io)?;
    if !canonical.starts_with(root) {
        return Err(composition_error(format!("{label} escapes the declared project root")));
    }
    Ok(canonical)
}

fn resolve_output(root: &Path, relative: &Path, label: &str) -> Result<PathBuf, ForgeError> {
    manifest::validate_local_path(label, relative, None)?;
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new("."));
    reject_symlink_components(root, parent_relative, label)?;
    let parent = root.join(parent_relative);
    let metadata = std::fs::symlink_metadata(&parent).map_err(|source| {
        ForgeError::Io(std::io::Error::new(
            source.kind(),
            format!("cannot inspect parent for {label} '{}': {source}", parent.display()),
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(composition_error(format!(
            "parent for {label} must be a non-symlink directory"
        )));
    }
    let canonical_parent = parent.canonicalize().map_err(ForgeError::Io)?;
    if !canonical_parent.starts_with(root) {
        return Err(composition_error(format!("{label} escapes the declared project root")));
    }
    let file_name = relative
        .file_name()
        .ok_or_else(|| composition_error(format!("{label} must name a file")))?;
    let output = canonical_parent.join(file_name);
    if let Ok(metadata) = std::fs::symlink_metadata(&output)
        && (metadata.file_type().is_symlink() || !metadata.is_file())
    {
        return Err(composition_error(format!(
            "existing {label} must be a regular non-symlink file"
        )));
    }
    Ok(output)
}

fn reject_symlink_components(base: &Path, relative: &Path, label: &str) -> Result<(), ForgeError> {
    let mut current = base.to_path_buf();
    for component in relative.components() {
        match component {
            Component::CurDir => continue,
            Component::Normal(part) => current.push(part),
            _ => {
                return Err(composition_error(format!(
                    "{label} contains an unsafe path component"
                )));
            }
        }
        if let Ok(metadata) = std::fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            return Err(composition_error(format!(
                "{label} traverses symbolic link '{}'",
                current.display()
            )));
        }
    }
    Ok(())
}

fn regular_non_symlink(path: &Path, label: &str) -> Result<PathBuf, ForgeError> {
    let metadata = std::fs::symlink_metadata(path).map_err(ForgeError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(composition_error(format!("{label} must be a regular non-symlink file")));
    }
    path.canonicalize().map_err(ForgeError::Io)
}

fn register_identity(
    registry: &mut IdentityRegistry,
    path: &Path,
    label: &str,
    allow_exact_reuse: bool,
) -> Result<(), ForgeError> {
    let identity = output_identity_key(path)?;
    let canonical_label = path.to_string_lossy().to_ascii_lowercase();
    if let Some(previous_label) = registry.identities.get(&identity) {
        if allow_exact_reuse && previous_label == &canonical_label {
            return Ok(());
        }
        return Err(composition_error(format!("{label} aliases another input or output")));
    }
    #[cfg(windows)]
    for (previous_path, previous_label) in &registry.paths {
        if paths_alias(path, previous_path)? {
            if allow_exact_reuse && previous_label == &canonical_label {
                return Ok(());
            }
            return Err(composition_error(format!("{label} aliases another input or output")));
        }
    }
    registry.identities.insert(identity, canonical_label.clone());
    #[cfg(windows)]
    registry.paths.push((path.to_path_buf(), canonical_label));
    Ok(())
}

#[cfg(unix)]
fn file_identity_key(path: &Path) -> Result<String, ForgeError> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::metadata(path).map_err(ForgeError::Io)?;
    Ok(format!("inode:{}:{}", metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn file_identity_key(path: &Path) -> Result<String, ForgeError> {
    path.canonicalize()
        .map(|value| format!("path:{}", value.to_string_lossy().to_ascii_lowercase()))
        .map_err(ForgeError::Io)
}

fn output_identity_key(path: &Path) -> Result<String, ForgeError> {
    if path.exists() {
        file_identity_key(path)
    } else {
        Ok(format!("path:{}", path.to_string_lossy().to_ascii_lowercase()))
    }
}

fn paths_alias(left: &Path, right: &Path) -> Result<bool, ForgeError> {
    crate::mapping::paths_alias(left, right)
        .map_err(|source| composition_error(format!("cannot compare path identities: {source}")))
}

fn validate_existing_conversion_chain(root: &Path, markdown: &[u8]) -> Result<(), ForgeError> {
    let mut temp = tempfile::Builder::new()
        .prefix(".forge-policy-compose-")
        .suffix(".md")
        .tempfile_in(root)
        .map_err(ForgeError::Io)?;
    temp.write_all(markdown).map_err(ForgeError::Io)?;
    temp.as_file().sync_all().map_err(ForgeError::Io)?;
    crate::pipeline::run_catalog_pipeline(
        temp.path(),
        MAX_COMPONENT_BYTES.saturating_mul(10),
        &crate::types::OutputFormat::Json,
        None,
    )?;
    Ok(())
}

fn commit_outputs(paths: &[PathBuf; 3], contents: [&[u8]; 3]) -> Result<(), ForgeError> {
    let mut staged = Vec::with_capacity(3);
    for (path, content) in paths.iter().zip(contents) {
        let parent = path.parent().ok_or_else(|| composition_error("output has no parent"))?;
        let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(ForgeError::Io)?;
        temp.write_all(content).map_err(ForgeError::Io)?;
        temp.as_file().sync_all().map_err(ForgeError::Io)?;
        staged.push((path, temp));
    }

    let mut backups: Vec<(PathBuf, Option<tempfile::NamedTempFile>)> = Vec::with_capacity(3);
    for path in paths {
        if path.exists() {
            let parent = path.parent().ok_or_else(|| composition_error("output has no parent"))?;
            let backup = match tempfile::NamedTempFile::new_in(parent) {
                Ok(backup) => backup,
                Err(error) => {
                    rollback_outputs(&[], &mut backups);
                    return Err(ForgeError::Io(error));
                }
            };
            if let Err(error) = std::fs::rename(path, backup.path()) {
                rollback_outputs(&[], &mut backups);
                return Err(ForgeError::Io(error));
            }
            backups.push((path.clone(), Some(backup)));
        } else {
            backups.push((path.clone(), None));
        }
    }

    let mut persisted = Vec::new();
    for (path, temp) in staged {
        match temp.persist(path) {
            Ok(file) => {
                persisted.push(path.clone());
                if let Err(error) = file.sync_all() {
                    rollback_outputs(&persisted, &mut backups);
                    return Err(ForgeError::Io(error));
                }
            }
            Err(error) => {
                rollback_outputs(&persisted, &mut backups);
                return Err(ForgeError::Io(error.error));
            }
        }
    }
    for parent in paths.iter().filter_map(|path| path.parent()).collect::<BTreeSet<_>>() {
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
    }
    Ok(())
}

fn rollback_outputs(
    persisted: &[PathBuf],
    backups: &mut [(PathBuf, Option<tempfile::NamedTempFile>)],
) {
    for path in persisted {
        let _ = std::fs::remove_file(path);
    }
    for (path, backup) in backups.iter_mut() {
        if let Some(backup) = backup.take() {
            let _ = std::fs::rename(backup.path(), path);
        }
    }
}

fn pretty_json<T: Serialize>(value: &T) -> Result<Vec<u8>, ForgeError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|source| {
        composition_error(format!("failed to serialize policy component artifact: {source}"))
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn composition_error(message: impl Into<String>) -> ForgeError {
    ForgeError::PolicyComposition(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn coordinated_write_stages_every_output_before_replacing_any() {
        let directory = tempfile::tempdir().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        let missing = directory.path().join("missing").join("third");
        std::fs::write(&first, "old-1").unwrap();
        std::fs::write(&second, "old-2").unwrap();
        let result = commit_outputs(
            &[first.clone(), second.clone(), missing],
            [b"new-1", b"new-2", b"new-3"],
        );
        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(first).unwrap(), "old-1");
        assert_eq!(std::fs::read_to_string(second).unwrap(), "old-2");
    }

    #[test]
    fn component_check_detects_pin_drift_before_rendering() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("component.md");
        let sidecar = directory.path().join("component.json");
        std::fs::write(&source, "## Clause\n\nText.\n").unwrap();
        std::fs::write(
            &sidecar,
            serde_json::to_vec(&json!({
                "schema_version": "forge.policy-component/1",
                "component_key": "clause",
                "version": "1.0.0",
                "title": "Clause",
                "owner": "security",
                "status": "approved",
                "source": "component.md",
                "expected_sha256": "0".repeat(64)
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(check_component(&sidecar).unwrap_err().to_string().contains("SHA-256 mismatch"));
    }
}
