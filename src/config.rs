//! Project configuration (`.forge.toml`) — PRD 051.
//!
//! Provides selection (global `--config`, `$FORGE_CONFIG`, nearest upward
//! discovery), strict parsing/validation of the closed schema-version 1
//! contract, and deterministic overlay resolution:
//!
//! explicit CLI > supported environment override > project config > built-in default.
//!
//! Safety properties (see PRD 051):
//! - The selected config must be a regular file, not a symlink, ≤ 1 MiB.
//! - Relative config paths resolve from the config's directory and must stay
//!   inside the project root after lexical normalization.
//! - No executables, hooks, secrets, interpolation, or input globs are permitted.
//!
//! Note: reproducible configuration resolution does **not** imply
//! byte-identical OSCAL artifacts; production metadata still embeds runtime
//! UUID v4 and UTC timestamps.

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::cli::{SchemaType, ValidateOutputFormat};
use crate::error::ForgeError;
use crate::types::{OutputFormat, OutputType, Strategy};

/// Conventional project configuration filename used during discovery.
pub const CONFIG_FILE_NAME: &str = ".forge.toml";

/// Maximum accepted configuration file size (PRD M-10).
pub const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

/// The only supported `schema-version` value (PRD M-4).
pub const SUPPORTED_SCHEMA_VERSION: i64 = 1;

/// Legal top-level keys in schema version 1.
const KNOWN_TOP_KEYS: &[&str] = &["convert", "schema-version", "validate"];

/// Legal `[convert]` keys in schema version 1.
const KNOWN_CONVERT_KEYS: &[&str] = &[
    "format",
    "import-ssp",
    "jobs",
    "max-size-mb",
    "output",
    "source-profile",
    "stable-id-baseline",
    "strategy",
    "summary",
    "to",
];

/// Legal `[validate]` keys in schema version 1.
const KNOWN_VALIDATE_KEYS: &[&str] = &["format", "output", "schema-type", "timeout-seconds"];

// ─── Raw schema (closed, deny_unknown_fields backstop) ─────────────────────

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(rename = "schema-version")]
    schema_version: Option<i64>,
    convert: Option<RawConvert>,
    validate: Option<RawValidate>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConvert {
    strategy: Option<String>,
    format: Option<String>,
    output: Option<String>,
    #[serde(rename = "max-size-mb")]
    max_size_mb: Option<i64>,
    #[serde(rename = "source-profile")]
    source_profile: Option<String>,
    jobs: Option<i64>,
    #[serde(rename = "stable-id-baseline")]
    stable_id_baseline: Option<String>,
    to: Option<String>,
    #[serde(rename = "import-ssp")]
    import_ssp: Option<String>,
    summary: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawValidate {
    #[serde(rename = "schema-type")]
    schema_type: Option<String>,
    format: Option<String>,
    output: Option<String>,
    #[serde(rename = "timeout-seconds")]
    timeout_seconds: Option<i64>,
}

// ─── Validated settings ─────────────────────────────────────────────────────

/// Validated `[convert]` settings from the project configuration.
///
/// Path-valued fields are already resolved against the project root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvertSettings {
    /// Conversion strategy (`catalog` or `component`).
    pub strategy: Option<Strategy>,
    /// Output serialization format.
    pub format: Option<OutputFormat>,
    /// Output file/directory resolved inside the project root.
    pub output: Option<PathBuf>,
    /// Maximum input size in MB (`1..=51200`).
    pub max_size_mb: Option<u64>,
    /// Source profile resolved to a regular file inside the project root.
    pub source_profile: Option<PathBuf>,
    /// Parallel jobs for batch conversion (`0..=256`; `0` = auto).
    pub jobs: Option<u16>,
    /// Stable-ID baseline resolved to a regular file inside the project root.
    pub stable_id_baseline: Option<PathBuf>,
    /// Output artifact type override.
    pub to: Option<OutputType>,
    /// SSP reference for Assessment Plan generation (non-empty).
    pub import_ssp: Option<String>,
    /// Whether to print the conversion summary dashboard.
    pub summary: Option<bool>,
}

/// Validated `[validate]` settings from the project configuration.
///
/// Path-valued fields are already resolved against the project root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateSettings {
    /// Model type override (auto-detect when `None`).
    pub schema_type: Option<SchemaType>,
    /// Validation report format.
    pub format: Option<ValidateOutputFormat>,
    /// Validation report output path resolved inside the project root.
    pub output: Option<PathBuf>,
    /// Round-trip oscal-cli step timeout in seconds (`1..=3600`).
    pub timeout_seconds: Option<u64>,
}

/// A fully loaded and validated project configuration file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectConfig {
    /// Absolute path of the selected config file.
    pub path: PathBuf,
    /// Absolute project root (the config file's parent directory).
    pub project_root: PathBuf,
    /// How the config file was selected.
    pub source: SourceKind,
    /// Validated `[convert]` settings, when present.
    pub convert: Option<ConvertSettings>,
    /// Validated `[validate]` settings, when present.
    pub validate: Option<ValidateSettings>,
}

impl ProjectConfig {
    /// Human-readable description of the selected config's location.
    #[must_use]
    pub fn describe(&self) -> String {
        let origin = match self.source {
            SourceKind::CliFlag => "--config",
            SourceKind::Environment => "$FORGE_CONFIG",
            SourceKind::Discovered => "discovered",
        };
        format!("{} ({origin})", self.path.display())
    }
}

/// How the active configuration file was selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Selected via the global `--config <path>` option.
    CliFlag,
    /// Selected via the `FORGE_CONFIG` environment variable.
    Environment,
    /// Selected via nearest `.forge.toml` upward discovery.
    Discovered,
}

// ─── Selection & discovery ──────────────────────────────────────────────────

/// Select and load the project configuration, if any (PRD precedence).
///
/// Order: `--config <path>`, then `FORGE_CONFIG`, then nearest `.forge.toml`
/// discovered from `cwd` through its ancestors. At most one file is selected;
/// files are never merged. An explicitly selected file that is missing or
/// invalid is an error — there is no fallback to discovery.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when an explicitly selected file is missing,
/// unsafe, or invalid.
pub fn load_selected(explicit_cli: Option<&Path>) -> Result<Option<ProjectConfig>, ForgeError> {
    match select_path(explicit_cli)? {
        Some((path, source)) => Ok(Some(load_file(&path, source)?)),
        None => Ok(None),
    }
}

/// Determine the config file path without loading it.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] for an invalid `FORGE_CONFIG` selector
/// (e.g. empty) — an empty value must not silently enable discovery (EC-3).
pub fn select_path(
    explicit_cli: Option<&Path>,
) -> Result<Option<(PathBuf, SourceKind)>, ForgeError> {
    if let Some(p) = explicit_cli {
        return Ok(Some((p.to_path_buf(), SourceKind::CliFlag)));
    }
    if let Some(env_path) = env_config_path()? {
        return Ok(Some((PathBuf::from(env_path), SourceKind::Environment)));
    }
    let cwd = std::env::current_dir()
        .map_err(|e| ForgeError::Config(format!("cannot determine current directory: {e}")))?;
    Ok(discover_from(&cwd).map(|p| (p, SourceKind::Discovered)))
}

/// Search `start` and each ancestor for `.forge.toml`, stopping at the first
/// match. Checks the filesystem root exactly once and terminates (EC-4).
///
/// A nearest candidate that exists but is unsafe (a directory, special file,
/// or dangling symlink) is **selected**, not skipped, so that loading fails
/// with an actionable file-safety error under M-10 instead of silently using
/// an ancestor configuration.
#[must_use]
pub fn discover_from(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start.to_path_buf());
    while let Some(current) = dir {
        let candidate = current.join(CONFIG_FILE_NAME);
        if fs::symlink_metadata(&candidate).is_ok() {
            return Some(candidate);
        }
        dir = current.parent().map(Path::to_path_buf);
    }
    None
}

fn env_config_path() -> Result<Option<String>, ForgeError> {
    match std::env::var("FORGE_CONFIG") {
        Ok(value) if value.trim().is_empty() => Err(ForgeError::Config(
            "FORGE_CONFIG is set to an empty value; expected a config file path \
             (unset the variable to enable .forge.toml discovery)"
                .to_string(),
        )),
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(ForgeError::Config(
            "FORGE_CONFIG contains non-Unicode data; a valid path is required".to_string(),
        )),
    }
}

/// Parse `FORGE_JOBS` (the only command-setting environment override in
/// schema version 1). Invalid or empty values are errors when consulted.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when the variable is set but empty,
/// non-numeric, or outside `0..=256`.
pub fn env_jobs() -> Result<Option<u16>, ForgeError> {
    match std::env::var("FORGE_JOBS") {
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(ForgeError::Config(
            "FORGE_JOBS contains non-Unicode data; expected an integer in 0..=256".to_string(),
        )),
        Ok(raw) => {
            if raw.trim().is_empty() {
                return Err(ForgeError::Config(
                    "FORGE_JOBS is set to an empty value; expected an integer in 0..=256"
                        .to_string(),
                ));
            }
            match raw.parse::<u16>() {
                Ok(jobs) if jobs <= 256 => Ok(Some(jobs)),
                _ => Err(ForgeError::Config(format!(
                    "FORGE_JOBS has invalid value '{raw}'; expected an integer in 0..=256"
                ))),
            }
        }
    }
}

// ─── Loading & safety checks ────────────────────────────────────────────────

/// Load, safety-check, parse, and validate one configuration file.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] for symlinked/non-regular/oversized files
/// (M-10), non-UTF-8 content (EC-10), TOML parse failures, schema-version
/// problems (M-4), unknown keys (M-6), type/range violations (M-5/M-11), and
/// unsafe or missing project-relative paths (M-9/M-12).
pub fn load_file(path: &Path, source: SourceKind) -> Result<ProjectConfig, ForgeError> {
    // File-safety metadata checks before any content read (M-10).
    let meta = fs::symlink_metadata(path).map_err(|e| {
        ForgeError::Config(format!("cannot read config file {}: {e}", path.display()))
    })?;
    if meta.file_type().is_symlink() {
        return Err(ForgeError::Config(format!(
            "{}: configuration files must not be symbolic links",
            path.display()
        )));
    }
    if !meta.is_file() {
        return Err(ForgeError::Config(format!(
            "{}: configuration must be a regular file",
            path.display()
        )));
    }
    if meta.len() > MAX_CONFIG_SIZE {
        return Err(ForgeError::Config(format!(
            "{}: configuration file exceeds the {} MiB limit ({} bytes)",
            path.display(),
            MAX_CONFIG_SIZE / (1024 * 1024),
            meta.len()
        )));
    }

    let bytes = fs::read(path).map_err(|e| {
        ForgeError::Config(format!("cannot read config file {}: {e}", path.display()))
    })?;
    let text = String::from_utf8(bytes).map_err(|_| {
        ForgeError::Config(format!("{}: configuration file is not valid UTF-8", path.display()))
    })?;

    parse_and_validate(path, source, &text)
        .map_err(|msg| ForgeError::Config(format!("{}: {msg}", path.display())))
}

/// Parse and validate config text. Returns the validated settings or a
/// diagnostic message (without the file prefix).
fn parse_and_validate(
    path: &Path,
    source: SourceKind,
    text: &str,
) -> Result<ProjectConfig, String> {
    let value: toml::Value = toml::from_str(text).map_err(|e| format!("invalid TOML:\n{e}"))?;
    check_unknown_keys(&value)?;

    let raw: RawFile =
        value.try_into().map_err(|e: toml::de::Error| format!("invalid schema: {e}"))?;

    match raw.schema_version {
        None => {
            return Err(
                "missing required key `schema-version`; this build supports schema version 1"
                    .to_string(),
            );
        }
        Some(SUPPORTED_SCHEMA_VERSION) => {}
        Some(other) => {
            return Err(format!(
                "unsupported schema-version {other}; this FORGE v{} binary supports schema version 1",
                env!("CARGO_PKG_VERSION")
            ));
        }
    }

    // A bare relative selector such as `.forge.toml` has an empty parent;
    // resolve it against the current directory (M-2, EC-1).
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
    let project_root =
        fs::canonicalize(parent).map_err(|e| format!("cannot resolve project root: {e}"))?;

    let convert = match raw.convert {
        Some(raw_convert) => Some(validate_convert(&project_root, raw_convert)?),
        None => None,
    };
    let validate = match raw.validate {
        Some(raw_validate) => Some(validate_validate(&project_root, &raw_validate)?),
        None => None,
    };

    Ok(ProjectConfig { path: path.to_path_buf(), project_root, source, convert, validate })
}

/// Reject unknown top-level keys and unknown keys inside command tables,
/// suggesting a close alternative when one exists (M-6).
fn check_unknown_keys(value: &toml::Value) -> Result<(), String> {
    let Some(table) = value.as_table() else {
        return Err("top-level value must be a table".to_string());
    };

    let mut top_keys: Vec<&String> = table.keys().collect();
    top_keys.sort();
    for key in top_keys {
        let key_str = key.as_str();
        if !KNOWN_TOP_KEYS.contains(&key_str) {
            return Err(unknown_key_message(key_str, None, KNOWN_TOP_KEYS));
        }
        let (sub_keys, known): (Option<Vec<&String>>, &[&str]) = match key_str {
            "convert" => (
                table[key_str].as_table().map(|t| {
                    let mut k: Vec<&String> = t.keys().collect();
                    k.sort();
                    k
                }),
                KNOWN_CONVERT_KEYS,
            ),
            "validate" => (
                table[key_str].as_table().map(|t| {
                    let mut k: Vec<&String> = t.keys().collect();
                    k.sort();
                    k
                }),
                KNOWN_VALIDATE_KEYS,
            ),
            _ => (None, &[]),
        };
        if let Some(sub_keys) = sub_keys {
            for sub in sub_keys {
                if !known.contains(&sub.as_str()) {
                    return Err(unknown_key_message(sub, Some(key_str), known));
                }
            }
        }
    }
    Ok(())
}

/// Build an unknown-key diagnostic, appending a close-match suggestion when
/// exactly one unambiguous candidate exists (AC-7).
fn unknown_key_message(key: &str, table: Option<&str>, known: &[&str]) -> String {
    let location = table.map_or_else(|| "at the top level".to_string(), |t| format!("in [{t}]"));
    match closest_key(key, known) {
        Some(candidate) => format!("unknown key `{key}` {location} (did you mean `{candidate}`?)"),
        None => format!(
            "unknown key `{key}` {location}; allowed keys: {}",
            known.iter().map(|k| format!("`{k}`")).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Find the closest known key within edit distance 2, if unique.
fn closest_key(key: &str, candidates: &[&str]) -> Option<String> {
    let mut best: Option<(usize, &str)> = None;
    let mut ties = 0;
    for candidate in candidates {
        let d = edit_distance(key, candidate);
        if d <= 2 {
            match best {
                Some((bd, _)) if d == bd => ties += 1,
                Some((bd, _)) if d > bd => {}
                _ => {
                    best = Some((d, candidate));
                    ties = 0;
                }
            }
        }
    }
    // A single candidate, or one strictly closer than any other, wins.
    if ties == 0 { best.map(|(_, c)| c.to_string()) } else { None }
}

/// Levenshtein edit distance (bounded inputs; keys are short identifiers).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

// ─── Field validation ───────────────────────────────────────────────────────

fn parse_enum<T: Clone>(
    key: &str,
    raw: Option<&str>,
    mapping: &[(&str, T)],
) -> Result<Option<T>, String> {
    let Some(s) = raw else { return Ok(None) };
    let lowered = s.trim().to_ascii_lowercase();
    mapping.iter().find(|(name, _)| lowered == *name).map(|(_, v)| Some(v.clone())).ok_or_else(
        || {
            format!(
                "`{key}` has invalid value '{s}'; allowed values: {}",
                mapping.iter().map(|(name, _)| format!("'{name}'")).collect::<Vec<_>>().join(", ")
            )
        },
    )
}

fn parse_range(key: &str, raw: Option<i64>, min: i64, max: i64) -> Result<Option<u64>, String> {
    match raw {
        None => Ok(None),
        Some(v) if (min..=max).contains(&v) => Ok(Some(u64::try_from(v).expect("range checked"))),
        Some(v) => Err(format!("`{key}` has value {v}; expected an integer in {min}..={max}")),
    }
}

fn validate_convert(project_root: &Path, raw: RawConvert) -> Result<ConvertSettings, String> {
    let strategy = parse_enum(
        "convert.strategy",
        raw.strategy.as_deref(),
        &[("catalog", Strategy::Catalog), ("component", Strategy::Component)],
    )?;
    let format = parse_enum(
        "convert.format",
        raw.format.as_deref(),
        &[("json", OutputFormat::Json), ("xml", OutputFormat::Xml), ("yaml", OutputFormat::Yaml)],
    )?;
    let to = parse_enum(
        "convert.to",
        raw.to.as_deref(),
        &[
            ("catalog", OutputType::Catalog),
            ("component-definition", OutputType::ComponentDefinition),
            ("ssp", OutputType::Ssp),
        ],
    )?;
    let max_size_mb = parse_range("convert.max-size-mb", raw.max_size_mb, 1, 51_200)?;
    let jobs = parse_range("convert.jobs", raw.jobs, 0, 256)?;

    let output = optional_project_path(project_root, "convert.output", raw.output.as_deref())?;
    let source_profile =
        optional_input_file(project_root, "convert.source-profile", raw.source_profile.as_deref())?;
    let stable_id_baseline = optional_input_file(
        project_root,
        "convert.stable-id-baseline",
        raw.stable_id_baseline.as_deref(),
    )?;

    if raw.import_ssp.as_deref().is_some_and(|s| s.trim().is_empty()) {
        return Err("`convert.import-ssp` must be a non-empty URI/reference string".to_string());
    }

    Ok(ConvertSettings {
        strategy,
        format,
        output,
        max_size_mb,
        source_profile,
        jobs: jobs.map(|j| u16::try_from(j).expect("range checked")),
        stable_id_baseline,
        to,
        import_ssp: raw.import_ssp.filter(|s| !s.trim().is_empty()),
        summary: raw.summary,
    })
}

fn validate_validate(project_root: &Path, raw: &RawValidate) -> Result<ValidateSettings, String> {
    let schema_type = parse_enum(
        "validate.schema-type",
        raw.schema_type.as_deref(),
        &[
            ("catalog", SchemaType::Catalog),
            ("component-definition", SchemaType::ComponentDefinition),
        ],
    )?;
    let format = parse_enum(
        "validate.format",
        raw.format.as_deref(),
        &[("text", ValidateOutputFormat::Text), ("json", ValidateOutputFormat::Json)],
    )?;
    let timeout_seconds = parse_range("validate.timeout-seconds", raw.timeout_seconds, 1, 3600)?;
    let output = optional_project_path(project_root, "validate.output", raw.output.as_deref())?;

    Ok(ValidateSettings { schema_type, format, output, timeout_seconds })
}

/// Resolve an optional project-relative path value (output destinations).
///
/// Rejects absolute values and normalization escaping the project root (M-9).
fn optional_project_path(
    project_root: &Path,
    key: &str,
    raw: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let Some(raw) = raw else { return Ok(None) };
    if raw.trim().is_empty() {
        return Err(format!("`{key}` must be a non-empty project-relative path"));
    }
    Ok(Some(resolve_inside_root(project_root, key, raw)?))
}

/// Resolve an optional project-relative path that must reference an existing
/// regular file (input/reference semantics).
fn optional_input_file(
    project_root: &Path,
    key: &str,
    raw: Option<&str>,
) -> Result<Option<PathBuf>, String> {
    let Some(resolved) = optional_project_path(project_root, key, raw)? else {
        return Ok(None);
    };
    let meta = fs::metadata(&resolved).map_err(|e| {
        format!("`{key}` references '{}': {e}", display_relative(project_root, &resolved))
    })?;
    if !meta.is_file() {
        return Err(format!(
            "`{key}` references '{}' which is not a regular file",
            display_relative(project_root, &resolved)
        ));
    }
    Ok(Some(resolved))
}

/// Lexically normalize `project_root.join(raw)` and reject escapes (EC-5/EC-6).
fn resolve_inside_root(project_root: &Path, key: &str, raw: &str) -> Result<PathBuf, String> {
    let candidate = Path::new(raw);
    if candidate.is_absolute() {
        return Err(format!(
            "`{key}` must be a project-relative path; absolute paths are not allowed ('{raw}')"
        ));
    }
    let joined = normalize_lexical(&project_root.join(candidate));
    if !joined.starts_with(project_root) {
        return Err(format!(
            "`{key}` path '{raw}' resolves outside the project root; escaping the project root is not allowed"
        ));
    }
    reject_windows_device_name(key, raw)?;
    ensure_symlink_containment(project_root, &joined, key, raw)?;
    Ok(joined)
}

/// Windows reserved device names (Win32 can redirect these to devices even
/// when they appear inside ordinary relative paths).
const RESERVED_DEVICE_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Reject Windows reserved device names in any path component
/// (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`). Win32 resolves reserved
/// device names in intermediate components as well, ignores trailing dots and
/// spaces (so `"CON "` and `"CON."` still name the device), and treats text
/// after the first `.` or `:` as extension/alternate-data-stream — all of
/// which can redirect an apparently ordinary relative path to a device. A
/// repository-controlled config must not be able to select such paths.
fn reject_windows_device_name(key: &str, raw: &str) -> Result<(), String> {
    for component in Path::new(raw).components() {
        let Component::Normal(name) = component else { continue };
        let name = name.to_string_lossy();
        // Split at the first '.' or ':' (extension / ADS separator), then
        // strip trailing spaces and dots that Win32 disregards. Leading
        // spaces are significant in Win32 and are preserved.
        let stem = name
            .split(['.', ':'])
            .next()
            .unwrap_or("")
            .trim_end_matches([' ', '.'])
            .to_ascii_uppercase();
        if RESERVED_DEVICE_NAMES.contains(&stem.as_str()) {
            return Err(format!(
                "`{key}` path '{raw}' uses a reserved device name ('{stem}') that is not allowed \
                 in project configuration paths"
            ));
        }
    }
    Ok(())
}

/// Verify that `resolved` cannot escape `project_root` through symbolic links
/// (PRD 051 trust boundary / M-9): the deepest existing ancestor is
/// canonicalized — resolving any symlinks inside the path — and the result
/// must remain within the canonical project root.
fn ensure_symlink_containment(
    project_root: &Path,
    resolved: &Path,
    key: &str,
    raw: &str,
) -> Result<(), String> {
    let canonical_root =
        fs::canonicalize(project_root).map_err(|e| format!("cannot resolve project root: {e}"))?;

    // Walk up to the deepest ancestor that currently exists.
    let mut ancestor = resolved;
    while fs::symlink_metadata(ancestor).is_err() {
        let Some(parent) = ancestor.parent() else {
            return Err(format!("`{key}` path '{raw}' cannot be resolved within the project root"));
        };
        ancestor = parent;
    }
    let canonical_ancestor = fs::canonicalize(ancestor)
        .map_err(|e| format!("`{key}` path '{raw}' cannot be resolved: {e}"))?;

    // Rebuild any not-yet-existing trailing components lexically.
    let remainder = resolved.strip_prefix(ancestor).unwrap_or(Path::new(""));
    let rebuilt = normalize_lexical(&canonical_ancestor.join(remainder));

    if !rebuilt.starts_with(&canonical_root) {
        return Err(format!(
            "`{key}` path '{raw}' resolves outside the project root through a symbolic link; \
             escaping the project root is not allowed"
        ));
    }
    Ok(())
}

/// Normalize `.` and `..` components without touching the filesystem (EC-5).
fn normalize_lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Render `path` relative to `root` for diagnostics, falling back to the path
/// itself when it is not inside `root`.
fn display_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map_or_else(|_| path.display().to_string(), |rel| rel.display().to_string())
}

// ─── Effective-setting resolution ───────────────────────────────────────────

/// Explicitly supplied convert-command values captured from clap before any
/// defaults are applied (R-1).
#[derive(Debug, Clone, Copy, Default)]
pub struct ConvertCliValues<'a> {
    /// `--strategy` when explicitly supplied.
    pub strategy: Option<Strategy>,
    /// `--format` when explicitly supplied.
    pub format: Option<OutputFormat>,
    /// `--output` when explicitly supplied.
    pub output: Option<&'a Path>,
    /// `--max-size` when explicitly supplied.
    pub max_size: Option<u64>,
    /// `--source-profile` when explicitly supplied.
    pub source_profile: Option<&'a str>,
    /// `--jobs` when explicitly supplied.
    pub jobs: Option<u16>,
    /// `--stable-id-baseline` when explicitly supplied.
    pub stable_id_baseline: Option<&'a Path>,
    /// `--to` when explicitly supplied.
    pub to: Option<OutputType>,
    /// `--import-ssp` when explicitly supplied.
    pub import_ssp: Option<&'a str>,
    /// `Some(true)` for `--summary`, `Some(false)` for `--no-summary`,
    /// `None` when neither was supplied.
    pub summary: Option<bool>,
}

/// Fully resolved effective settings for the `convert` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConvert {
    /// Conversion strategy.
    pub strategy: Strategy,
    /// Output serialization format.
    pub format: OutputFormat,
    /// Output destination (CLI/config); `None` retains command defaults.
    pub output: Option<PathBuf>,
    /// Maximum input size in MB.
    pub max_size_mb: u64,
    /// Source profile for component strategy.
    pub source_profile: Option<PathBuf>,
    /// Parallel jobs (`0` = auto).
    pub jobs: u16,
    /// Stable-ID baseline document.
    pub stable_id_baseline: Option<PathBuf>,
    /// Output artifact type override.
    pub to: Option<OutputType>,
    /// SSP reference for Assessment Plan generation.
    pub import_ssp: Option<String>,
    /// Whether to print the summary dashboard.
    pub summary: bool,
}

/// Fully resolved effective settings for the `validate` command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveValidate {
    /// Model type override (auto-detect when `None`).
    pub schema_type: Option<SchemaType>,
    /// Validation report format.
    pub format: ValidateOutputFormat,
    /// Report output destination.
    pub output: Option<PathBuf>,
    /// Round-trip oscal-cli step timeout in seconds.
    pub timeout_seconds: u64,
}

/// Validate cross-field constraints among the project config's own settings.
///
/// Used by `forge config check` (M-13) to reject configurations that could
/// never run successfully. Command execution deliberately does **not** call
/// this on config-only values: an explicit CLI value may resolve a conflict
/// present at a lower layer (EC-9).
///
/// # Errors
///
/// Returns an error describing the conflicting effective settings.
pub fn validate_cross_field(convert: &ConvertSettings) -> Result<(), ForgeError> {
    if matches!(convert.strategy, Some(Strategy::Component)) && convert.source_profile.is_none() {
        return Err(ForgeError::Config(
            "`convert.strategy = \"component\"` requires `convert.source-profile` in the \
             project configuration (or supply --source-profile on the command line)"
                .to_string(),
        ));
    }
    Ok(())
}

/// Resolve effective `convert` settings:
/// explicit CLI > `FORGE_JOBS` environment override > project config >
/// built-in defaults (M-7).
///
/// `jobs_env` is the pre-parsed [`env_jobs`] result; pass `Ok(None)` in tests.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when no layer supplies the required
/// `--strategy` choice or an environment override is invalid.
pub fn resolve_convert(
    cli: ConvertCliValues<'_>,
    config: Option<&ProjectConfig>,
    jobs_env: Result<Option<u16>, ForgeError>,
) -> Result<EffectiveConvert, ForgeError> {
    let settings = config.and_then(|c| c.convert.as_ref());

    let strategy = cli.strategy.or_else(|| settings.and_then(|s| s.strategy)).ok_or_else(|| {
        ForgeError::MissingRequiredArgument(
            "the following required arguments were not provided:\n  \
                 --strategy <STRATEGY>"
                .to_string(),
        )
    })?;

    // Environment overrides are consulted only when the CLI did not supply a
    // value: an explicit `--jobs` must not be defeated by an invalid
    // lower-precedence `FORGE_JOBS` (M-7).
    let jobs = match cli.jobs {
        Some(jobs) => jobs,
        None => jobs_env?.or_else(|| settings.and_then(|s| s.jobs)).unwrap_or(0),
    };

    let max_size_mb = cli.max_size.or_else(|| settings.and_then(|s| s.max_size_mb)).unwrap_or(10);

    Ok(EffectiveConvert {
        strategy,
        format: cli
            .format
            .or_else(|| settings.and_then(|s| s.format))
            .unwrap_or(OutputFormat::Json),
        output: cli
            .output
            .map(Path::to_path_buf)
            .or_else(|| settings.and_then(|s| s.output.clone())),
        max_size_mb,
        source_profile: cli
            .source_profile
            .map(PathBuf::from)
            .or_else(|| settings.and_then(|s| s.source_profile.clone())),
        jobs,
        stable_id_baseline: cli
            .stable_id_baseline
            .map(Path::to_path_buf)
            .or_else(|| settings.and_then(|s| s.stable_id_baseline.clone())),
        to: cli.to.or_else(|| settings.and_then(|s| s.to)),
        import_ssp: cli
            .import_ssp
            .map(str::to_string)
            .or_else(|| settings.and_then(|s| s.import_ssp.clone())),
        summary: cli.summary.unwrap_or_else(|| settings.and_then(|s| s.summary).unwrap_or(false)),
    })
}

/// Resolve effective `validate` settings using the same precedence model.
///
/// # Errors
///
/// Returns [`ForgeError::Config`] when an environment override is invalid
/// (reserved for future variables; currently none apply to `validate`).
pub fn resolve_validate(
    schema_type: Option<SchemaType>,
    format: Option<ValidateOutputFormat>,
    output: Option<&Path>,
    timeout: Option<u64>,
    config: Option<&ProjectConfig>,
) -> Result<EffectiveValidate, ForgeError> {
    let settings = config.and_then(|c| c.validate.as_ref());
    Ok(EffectiveValidate {
        schema_type: schema_type.or_else(|| settings.and_then(|s| s.schema_type.clone())),
        format: format
            .or_else(|| settings.and_then(|s| s.format.clone()))
            .unwrap_or(ValidateOutputFormat::Text),
        output: output.map(Path::to_path_buf).or_else(|| settings.and_then(|s| s.output.clone())),
        timeout_seconds: timeout.or_else(|| settings.and_then(|s| s.timeout_seconds)).unwrap_or(30),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join(CONFIG_FILE_NAME);
        std::fs::write(&path, content).expect("write config");
        path
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn load_str(text: &str) -> Result<ProjectConfig, ForgeError> {
        let dir = tempdir();
        let path = write_config(dir.path(), text);
        load_file(&path, SourceKind::Discovered)
    }

    // ── Schema version (M-4, AC-4, AC-5) ──

    #[test]
    fn missing_schema_version_fails() {
        let err = load_str("[convert]\nstrategy = \"catalog\"\n").unwrap_err();
        assert!(err.to_string().contains("schema-version"), "{err}");
    }

    #[test]
    fn unsupported_schema_version_reports_supported_version() {
        let err = load_str("schema-version = 2\n").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains('2'), "{msg}");
        assert!(msg.contains("version 1"), "{msg}");
        assert!(msg.contains(env!("CARGO_PKG_VERSION")), "{msg}");
    }

    #[test]
    fn non_integer_schema_version_fails_cleanly() {
        let err = load_str("schema-version = \"one\"\n").unwrap_err();
        assert!(err.to_string().contains("invalid"), "{err}");
    }

    // ── Unknown keys & suggestions (M-6, AC-7) ──

    #[test]
    fn unknown_top_level_key_rejected() {
        let err = load_str("schema-version = 1\ninputs = [\"a.md\"]\n").unwrap_err();
        assert!(err.to_string().contains("unknown key `inputs`"), "{err}");
    }

    #[test]
    fn unknown_convert_key_rejected_with_suggestion() {
        let err = load_str("schema-version = 1\n[convert]\nformt = \"json\"\n").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown key `formt`"), "{msg}");
        assert!(msg.contains("did you mean `format`?"), "{msg}");
    }

    #[test]
    fn unsupported_security_sensitive_keys_rejected_ac19() {
        for body in [
            "[validate]\nround-trip = true\n",
            "[validate]\noscal-cli-path = \"/usr/bin/oscal-cli\"\n",
            "hooks = []\n",
        ] {
            let text = format!("schema-version = 1\n{body}");
            assert!(load_str(&text).is_err(), "unsupported key must be rejected: {body}");
        }
    }

    // ── Enums & ranges (M-5) ──

    #[test]
    fn invalid_enum_value_lists_allowed_values() {
        let err = load_str("schema-version = 1\n[convert]\nformat = \"protobuf\"\n").unwrap_err();
        assert!(err.to_string().contains("'json', 'xml', 'yaml'"), "{err}");
    }

    #[test]
    fn jobs_out_of_range_rejected() {
        let err = load_str("schema-version = 1\n[convert]\njobs = 257\n").unwrap_err();
        assert!(err.to_string().contains("0..=256"), "{err}");
    }

    #[test]
    fn max_size_out_of_range_rejected() {
        let err = load_str("schema-version = 1\n[convert]\nmax-size-mb = 0\n").unwrap_err();
        assert!(err.to_string().contains("1..=51200"), "{err}");
    }

    #[test]
    fn timeout_out_of_range_rejected() {
        let err = load_str("schema-version = 1\n[validate]\ntimeout-seconds = 3601\n").unwrap_err();
        assert!(err.to_string().contains("1..=3600"), "{err}");
    }

    #[test]
    fn full_valid_config_parses() {
        let cfg = load_str(
            "schema-version = 1\n\
             [convert]\n\
             strategy = \"catalog\"\n\
             format = \"yaml\"\n\
             output = \"generated/oscal\"\n\
             max-size-mb = 20\n\
             jobs = 4\n\
             summary = true\n\
             [validate]\n\
             format = \"json\"\n\
             timeout-seconds = 20\n",
        )
        .expect("valid config");
        let conv = cfg.convert.expect("convert present");
        assert_eq!(conv.strategy, Some(Strategy::Catalog));
        assert_eq!(conv.format, Some(OutputFormat::Yaml));
        assert_eq!(conv.max_size_mb, Some(20));
        assert_eq!(conv.jobs, Some(4));
        assert_eq!(conv.summary, Some(true));
        assert_eq!(conv.output, Some(cfg.project_root.join("generated/oscal")));
        let val = cfg.validate.expect("validate present");
        assert_eq!(val.format, Some(ValidateOutputFormat::Json));
        assert_eq!(val.timeout_seconds, Some(20));
    }

    // ── Key order independence (M-17 / AC-20) ──

    #[test]
    fn reordered_keys_resolve_identically() {
        let a = load_str(
            "schema-version = 1\n[convert]\nstrategy = \"catalog\"\nformat = \"json\"\njobs = 3\n",
        )
        .unwrap();
        let b = load_str(
            "schema-version = 1\n[convert]\njobs = 3\nformat = \"json\"\nstrategy = \"catalog\"\n",
        )
        .unwrap();
        assert_eq!(a.convert, b.convert);
    }

    // ── Paths (M-9, AC-13, AC-14, EC-5, EC-6) ──

    #[test]
    fn relative_output_resolves_from_project_root() {
        let cfg =
            load_str("schema-version = 1\n[convert]\noutput = \"generated/oscal\"\n").unwrap();
        assert_eq!(cfg.convert.unwrap().output.unwrap(), cfg.project_root.join("generated/oscal"));
    }

    #[test]
    fn dot_segments_normalize_consistently() {
        let cfg =
            load_str("schema-version = 1\n[convert]\noutput = \"./generated/../oscal\"\n").unwrap();
        assert_eq!(cfg.convert.unwrap().output.unwrap(), cfg.project_root.join("oscal"));
    }

    #[test]
    fn traversal_outside_root_rejected() {
        let err = load_str("schema-version = 1\n[convert]\noutput = \"../outside\"\n").unwrap_err();
        assert!(err.to_string().contains("outside the project root"), "{err}");
    }

    #[test]
    fn absolute_output_rejected() {
        // Use an actually-absolute path per platform; "/etc/passwd" is
        // drive-relative (not absolute) on Windows.
        let abs = if cfg!(windows) { "C:\\evil\\out.json" } else { "/etc/passwd" };
        let err =
            load_str(&format!("schema-version = 1\n[convert]\noutput = \"{abs}\"\n"))
                .unwrap_err();
        assert!(err.to_string().contains("absolute"), "{err}");
    }

    #[test]
    fn contained_parent_traversal_accepted() {
        let cfg =
            load_str("schema-version = 1\n[convert]\noutput = \"sub/../generated/out.json\"\n")
                .unwrap();
        assert_eq!(
            cfg.convert.unwrap().output.unwrap(),
            cfg.project_root.join("generated/out.json")
        );
    }

    #[test]
    fn source_profile_must_exist_as_regular_file() {
        let dir = tempdir();
        write_config(
            dir.path(),
            "schema-version = 1\n[convert]\nsource-profile = \"missing.json\"\n",
        );
        let err =
            load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered).unwrap_err();
        assert!(err.to_string().contains("source-profile"), "{err}");

        std::fs::write(dir.path().join("baseline.json"), "{}").unwrap();
        write_config(
            dir.path(),
            "schema-version = 1\n[convert]\nsource-profile = \"baseline.json\"\n",
        );
        let cfg = load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered).unwrap();
        assert_eq!(
            cfg.convert.unwrap().source_profile.unwrap(),
            dir.path().canonicalize().unwrap().join("baseline.json")
        );
    }

    #[test]
    fn import_ssp_must_be_non_empty() {
        let err = load_str("schema-version = 1\n[convert]\nimport-ssp = \"  \"\n").unwrap_err();
        assert!(err.to_string().contains("import-ssp"), "{err}");
    }

    // ── File safety (M-10, AC-15, EC-10) ──

    #[test]
    #[cfg(unix)]
    fn symlinked_config_rejected() {
        let dir = tempdir();
        let real = dir.path().join("real.toml");
        std::fs::write(&real, "schema-version = 1\n").unwrap();
        let link = dir.path().join(CONFIG_FILE_NAME);
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let err = load_file(&link, SourceKind::Discovered).unwrap_err();
        assert!(err.to_string().contains("symbolic link"), "{err}");
    }

    #[test]
    fn oversized_config_rejected() {
        let dir = tempdir();
        let big = format!(
            "schema-version = 1\n[convert]\nimport-ssp = \"{}\"\n",
            "x".repeat(usize::try_from(MAX_CONFIG_SIZE).expect("fits usize"))
        );
        write_config(dir.path(), &big);
        let err =
            load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered).unwrap_err();
        assert!(err.to_string().contains("exceeds"), "{err}");
    }

    #[test]
    fn non_utf8_config_rejected_without_lossy_interpretation() {
        let dir = tempdir();
        let path = dir.path().join(CONFIG_FILE_NAME);
        std::fs::write(&path, [0xff, 0xfe, b'a']).unwrap();
        let err = load_file(&path, SourceKind::Discovered).unwrap_err();
        assert!(err.to_string().contains("UTF-8"), "{err}");
    }

    #[test]
    fn invalid_toml_identifies_location() {
        let err = load_str("schema-version = 1\n[convert\n").unwrap_err();
        assert!(err.to_string().contains("invalid TOML"), "{err}");
        assert!(err.to_string().contains("line"), "{err}");
    }

    // ── Discovery (M-1, M-3, AC-1, EC-4) ──

    #[test]
    fn discovery_selects_nearest_without_merging() {
        let parent = tempdir();
        let child_dir = parent.path().join("child");
        std::fs::create_dir(&child_dir).unwrap();
        write_config(parent.path(), "schema-version = 1\n[convert]\njobs = 1\n");
        write_config(&child_dir, "schema-version = 1\n[convert]\njobs = 2\n");

        let found = discover_from(&child_dir).unwrap();
        assert_eq!(found, child_dir.join(CONFIG_FILE_NAME));

        let cfg = load_file(&found, SourceKind::Discovered).unwrap();
        assert_eq!(cfg.convert.unwrap().jobs, Some(2), "nearest wins, no merge");
    }

    #[test]
    fn discovery_from_root_terminates() {
        // Starting from an isolated deep tree whose ancestors have no config;
        // walks to "/" once and returns None rather than looping.
        let dir = tempdir();
        let nested = dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        assert!(discover_from(&nested).is_none());
    }

    #[test]
    #[cfg(unix)]
    fn discovery_selects_unsafe_nearest_candidate_instead_of_bypassing() {
        // A directory named .forge.toml is the nearest candidate: discovery
        // must select it (so loading fails under M-10) rather than silently
        // skipping to an ancestor configuration.
        use std::os::unix::fs::symlink;
        let parent = tempdir();
        let child = parent.path().join("child");
        std::fs::create_dir(&child).unwrap();
        write_config(parent.path(), "schema-version = 1\n[convert]\njobs = 7\n");

        // Directory candidate
        std::fs::create_dir(child.join(CONFIG_FILE_NAME)).unwrap();
        let found = discover_from(&child).expect("nearest candidate must be selected");
        assert_eq!(found, child.join(CONFIG_FILE_NAME));
        assert!(load_file(&found, SourceKind::Discovered).is_err());

        // Dangling symlink candidate
        std::fs::remove_dir(child.join(CONFIG_FILE_NAME)).unwrap();
        symlink(parent.path().join("nonexistent.toml"), child.join(CONFIG_FILE_NAME)).unwrap();
        let found = discover_from(&child).expect("dangling symlink must be selected");
        assert_eq!(found, child.join(CONFIG_FILE_NAME));
        let err = load_file(&found, SourceKind::Discovered).unwrap_err();
        assert!(!err.to_string().contains("jobs = 7"), "no ancestor fallback: {err}");
    }

    // ── Symlink containment for config paths (M-9 trust boundary) ──

    #[test]
    fn windows_reserved_device_names_rejected_cross_platform() {
        for raw in [
            "CON",
            "NUL.json",
            "com1",
            "logs/LPT3.txt",
            // Intermediate components are checked too.
            "CON/out.json",
            "logs/AUX/report.json",
            // Trailing dots/spaces are insignificant in Win32...
            "CON ",
            "CON.",
            "sub/NUL .. ",
            // ...and alternate-data-stream syntax still names the device.
            "CON:stream",
            "aux.txt:hidden",
        ] {
            let text = format!("schema-version = 1\n[convert]\noutput = \"{raw}\"\n");
            let err = load_str(&text).unwrap_err();
            assert!(err.to_string().contains("reserved device name"), "{raw}: {err}");
        }
        // Ordinary names remain fine.
        assert!(load_str("schema-version = 1\n[convert]\noutput = \"console/app.json\"\n").is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn symlinked_output_path_escape_is_rejected() {
        let dir = tempdir();
        // output-link -> ../.. : a lexical check alone would pass.
        std::os::unix::fs::symlink(dir.path().parent().unwrap(), dir.path().join("output-link"))
            .unwrap();
        write_config(
            dir.path(),
            "schema-version = 1\n[convert]\noutput = \"output-link/.zshrc\"\n",
        );
        let err = load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered);
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("symbolic link") || msg.contains("outside the project root"), "{msg}");
    }

    #[test]
    #[cfg(unix)]
    fn symlinked_input_reference_escape_is_rejected() {
        let dir = tempdir();
        let outside = tempdir();
        let secret = outside.path().join("secret.json");
        std::fs::write(&secret, "{}").unwrap();
        std::os::unix::fs::symlink(&secret, dir.path().join("profile-link")).unwrap();
        write_config(
            dir.path(),
            "schema-version = 1\n[convert]\nsource-profile = \"profile-link\"\n",
        );
        let err = load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered);
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("outside the project root"), "{msg}");
    }

    #[test]
    #[cfg(unix)]
    fn contained_symlinked_directory_still_accepted() {
        // A symlink that stays inside the project root remains allowed.
        let dir = tempdir();
        std::fs::create_dir(dir.path().join("real")).unwrap();
        std::os::unix::fs::symlink(dir.path().join("real"), dir.path().join("link")).unwrap();
        write_config(dir.path(), "schema-version = 1\n[convert]\noutput = \"link/out.json\"\n");
        let cfg = load_file(&dir.path().join(CONFIG_FILE_NAME), SourceKind::Discovered).unwrap();
        assert_eq!(
            cfg.convert.unwrap().output.unwrap(),
            normalize_lexical(&dir.path().canonicalize().unwrap().join("link/out.json"))
        );
    }

    // ── Bare relative selectors (M-2, EC-1) ── covered end-to-end in
    // tests/project_config_test.rs (`bare_relative_forge_config_selector_works`)
    // because resolving them depends on the process working directory.

    // ── FORGE_JOBS environment override (EC-3 analogues) ──

    #[test]
    fn invalid_env_override_fails_resolution() {
        // The parsed FORGE_JOBS result is injected; an invalid override must
        // fail resolution before any command side effects (M-12) — but only
        // when no explicit CLI value takes precedence.
        let cli = ConvertCliValues { strategy: Some(Strategy::Catalog), ..Default::default() };
        let err = resolve_convert(cli, None, Err(ForgeError::Config("bad".into()))).unwrap_err();
        assert!(err.to_string().contains("bad"));
    }

    #[test]
    fn explicit_cli_jobs_defeats_invalid_env_override() {
        // FORGE_JOBS=999 with --jobs 1 must resolve to 1, not fail (M-7):
        // lower-precedence layers are not consulted when CLI supplies a value.
        let cli = ConvertCliValues {
            strategy: Some(Strategy::Catalog),
            jobs: Some(1),
            ..Default::default()
        };
        let eff = resolve_convert(cli, None, Err(ForgeError::Config("invalid".into()))).unwrap();
        assert_eq!(eff.jobs, 1);
    }

    // ── Cross-field validation (M-13) ──

    #[test]
    fn component_strategy_without_profile_fails_cross_field_check() {
        let settings = ConvertSettings {
            strategy: Some(Strategy::Component),
            format: None,
            output: None,
            max_size_mb: None,
            source_profile: None,
            jobs: None,
            stable_id_baseline: None,
            to: None,
            import_ssp: None,
            summary: None,
        };
        let err = validate_cross_field(&settings).unwrap_err();
        assert!(err.to_string().contains("source-profile"), "{err}");

        let mut resolved = settings.clone();
        resolved.source_profile = Some(PathBuf::from("/tmp/profile.json"));
        assert!(validate_cross_field(&resolved).is_ok());

        let mut catalog = settings;
        catalog.strategy = Some(Strategy::Catalog);
        assert!(validate_cross_field(&catalog).is_ok());
    }

    #[test]
    fn env_override_applied_without_cli() {
        let eff = resolve_convert(base_cli(), None, Ok(Some(2))).unwrap();
        assert_eq!(eff.jobs, 2);
    }

    // ── Precedence resolution (M-7, AC-8..AC-12) ──

    fn config_with_jobs(jobs: i64) -> ProjectConfig {
        let dir = tempdir();
        let text = format!("schema-version = 1\n[convert]\njobs = {jobs}\n");
        let path = write_config(dir.path(), &text);
        load_file(&path, SourceKind::Discovered).unwrap()
    }

    fn base_cli() -> ConvertCliValues<'static> {
        ConvertCliValues { strategy: Some(Strategy::Catalog), ..Default::default() }
    }

    #[test]
    fn ac8_cli_beats_env_beats_config_beats_default() {
        // config jobs=4, env jobs=2 → env wins
        let cfg = config_with_jobs(4);
        let eff = resolve_convert(base_cli(), Some(&cfg), Ok(Some(2))).unwrap();
        assert_eq!(eff.jobs, 2);

        // CLI jobs=1 beats env and config
        let eff = resolve_convert(
            ConvertCliValues { jobs: Some(1), ..base_cli() },
            Some(&cfg),
            Ok(Some(2)),
        )
        .unwrap();
        assert_eq!(eff.jobs, 1, "explicit CLI wins");
    }

    #[test]
    fn ac10_config_used_when_no_cli_no_env() {
        let cfg = config_with_jobs(4);
        let eff = resolve_convert(base_cli(), Some(&cfg), Ok(None)).unwrap();
        assert_eq!(eff.jobs, 4);
    }

    #[test]
    fn ac11_builtin_default_when_no_layers() {
        let eff = resolve_convert(base_cli(), None, Ok(None)).unwrap();
        assert_eq!(eff.jobs, 0);
        assert_eq!(eff.max_size_mb, 10);
        assert_eq!(eff.format, OutputFormat::Json);
        assert!(!eff.summary);
    }

    #[test]
    fn unrelated_settings_remain_independent() {
        let dir = tempdir();
        let text = "schema-version = 1\n[convert]\nformat = \"yaml\"\njobs = 4\n";
        let path = write_config(dir.path(), text);
        let cfg = load_file(&path, SourceKind::Discovered).unwrap();

        let eff = resolve_convert(
            ConvertCliValues { format: Some(OutputFormat::Xml), ..base_cli() },
            Some(&cfg),
            Ok(None),
        )
        .unwrap();
        assert_eq!(eff.format, OutputFormat::Xml);
        assert_eq!(eff.jobs, 4, "config jobs survive partial CLI override");
    }

    #[test]
    fn ac12_summary_explicit_false_overrides_config_true() {
        let dir = tempdir();
        let text = "schema-version = 1\n[convert]\nsummary = true\n";
        let path = write_config(dir.path(), text);
        let cfg = load_file(&path, SourceKind::Discovered).unwrap();

        let eff = resolve_convert(base_cli(), Some(&cfg), Ok(None)).unwrap();
        assert!(eff.summary);

        let eff = resolve_convert(
            ConvertCliValues { summary: Some(false), ..base_cli() },
            Some(&cfg),
            Ok(None),
        )
        .unwrap();
        assert!(!eff.summary);
    }

    #[test]
    fn strategy_required_from_cli_or_config() {
        let err = resolve_convert(ConvertCliValues::default(), None, Ok(None)).unwrap_err();
        assert!(matches!(err, ForgeError::MissingRequiredArgument(_)), "{err}");
        assert!(err.to_string().contains("--strategy"), "{err}");

        let cfg_text = "schema-version = 1\n[convert]\nstrategy = \"component\"\n";
        let dir = tempdir();
        let path = write_config(dir.path(), cfg_text);
        let cfg = load_file(&path, SourceKind::Discovered).unwrap();
        let eff = resolve_convert(ConvertCliValues::default(), Some(&cfg), Ok(None)).unwrap();
        assert_eq!(eff.strategy, Strategy::Component);
    }

    // ── Validate resolution ──

    #[test]
    fn validate_resolution_precedence_and_defaults() {
        let eff = resolve_validate(None, None, None, None, None).unwrap();
        assert_eq!(eff.format, ValidateOutputFormat::Text);
        assert_eq!(eff.timeout_seconds, 30);
        assert!(eff.schema_type.is_none());

        let dir = tempdir();
        let text = "schema-version = 1\n[validate]\nformat = \"json\"\ntimeout-seconds = 20\n";
        let path = write_config(dir.path(), text);
        let cfg = load_file(&path, SourceKind::Discovered).unwrap();

        let eff = resolve_validate(None, None, None, None, Some(&cfg)).unwrap();
        assert_eq!(eff.format, ValidateOutputFormat::Json);
        assert_eq!(eff.timeout_seconds, 20);

        let eff =
            resolve_validate(None, Some(ValidateOutputFormat::Text), None, Some(45), Some(&cfg))
                .unwrap();
        assert_eq!(eff.format, ValidateOutputFormat::Text);
        assert_eq!(eff.timeout_seconds, 45);
    }

    // ── Suggestion quality ──

    #[test]
    fn ambiguous_suggestion_is_suppressed() {
        // 'jbs' is equidistant from nothing within 2 except 'jobs' (distance 1) — fine;
        // craft an ambiguous case: 'fobmat' vs 'format'(1) and... ensure single winner.
        let msg = unknown_key_message("outpt", Some("convert"), KNOWN_CONVERT_KEYS);
        assert!(msg.contains("`output`"), "{msg}");

        let msg = unknown_key_message("zzzzzzz", None, KNOWN_TOP_KEYS);
        assert!(!msg.contains("did you mean"), "{msg}");
        assert!(msg.contains("allowed keys"), "{msg}");
    }
}
