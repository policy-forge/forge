// FORGE WI-41: Assessment Plan Builder — Interface Contracts
//
// This file defines the public API surface for the Assessment Plan feature.
// It is a contract document, not executable code. Implementations in
// src/oscal/assessment_plan.rs must match these signatures exactly.
//
// Contract version: 1.0.0 | Date: 2026-03-12

// ─── Primary Builder ─────────────────────────────────────────────────────

/// Build an OSCAL Assessment Plan JSON envelope from conversion output.
///
/// # Arguments
///
/// * `control_ids` — Control IDs from the conversion pipeline (Catalog or Component
///   Definition). May be empty (produces empty `include-controls` + warning).
///   Duplicates are removed. Order is normalized (sorted) for determinism.
/// * `import_ssp_href` — Path to the SSP, from `--import-ssp` CLI flag.
///   Must be non-empty; returns `ForgeError::Validation` if empty.
/// * `policy_title` — Title of the source policy document. Used in `metadata.title`
///   and `reviewed-controls.description`.
///
/// # Returns
///
/// `AssessmentPlanEnvelope` ready for `serde_json::to_string_pretty`.
///
/// # Errors
///
/// * `ForgeError::Validation` — if `import_ssp_href` is empty or whitespace-only
/// * `ForgeError::AssessmentPlanBuild` — if metadata assembly fails
///
/// # Guarantees
///
/// * Same inputs always produce identical output (deterministic UUID v5).
/// * `reviewed-controls.control-selections[0].include-controls` contains exactly
///   the deduplicated, sorted control IDs from the input slice.
/// * `metadata.version` is always `"1.0.0"`.
/// * `metadata.oscal-version` is always `"1.2.0"`.
///
/// # Example
///
/// ```rust
/// let ids = vec!["POL-AC-001".to_string(), "POL-AC-002".to_string()];
/// let envelope = build_assessment_plan(&ids, "./ssp/system-ssp.json", "Corporate Security Policy")?;
/// let json = serde_json::to_string_pretty(&envelope)?;
/// assert!(json.contains("\"assessment-plan\""));
/// assert!(json.contains("POL-AC-001"));
/// ```
pub fn build_assessment_plan(
    control_ids: &[String],
    import_ssp_href: &str,
    policy_title: &str,
) -> Result<AssessmentPlanEnvelope, ForgeError>;

// ─── Control ID Collectors ────────────────────────────────────────────────

/// Collect all control IDs from a built OSCAL Catalog.
///
/// Iterates `catalog.groups[].controls[].id` in declaration order.
/// Returns an empty Vec if the catalog has no groups or controls.
/// Does NOT deduplicate — deduplication is performed by `build_assessment_plan`.
///
/// # Example
///
/// ```rust
/// let catalog = build_catalog(&doc, None)?;
/// let ids = collect_control_ids_from_catalog(&catalog);
/// // ids = ["POL-AC-001", "POL-AC-002", ...]
/// ```
pub fn collect_control_ids_from_catalog(catalog: &OscalCatalog) -> Vec<String>;

/// Collect all control IDs from a built Component Definition.
///
/// Iterates all `components[].control_implementations[].implemented_requirements[].control_id`.
/// Returns an empty Vec if the Component Definition has no implemented requirements
/// (e.g., when `--source-profile` was not provided).
/// Does NOT deduplicate — deduplication is performed by `build_assessment_plan`.
///
/// # Example
///
/// ```rust
/// let envelope = build_component_definition(&doc, Some(profile_path), None, None)?;
/// let ids = collect_control_ids_from_component_def(&envelope);
/// // ids = ["ac-1", "ac-2", ...]
/// ```
pub fn collect_control_ids_from_component_def(
    envelope: &ComponentDefinitionEnvelope,
) -> Vec<String>;

// ─── Output Path Derivation ───────────────────────────────────────────────

/// Derive the Assessment Plan output file path from the input and primary output paths.
///
/// # Rules
///
/// * Output filename: `{input_stem}-assessment-plan.json`
/// * Output directory: parent of `primary_output` if `Some`; else `.` (cwd)
///
/// # Examples
///
/// ```rust
/// // No primary output (stdout mode):
/// let ap = derive_ap_output_path(Path::new("policy.md"), None);
/// assert_eq!(ap, PathBuf::from("./policy-assessment-plan.json"));
///
/// // Primary output to a specific path:
/// let ap = derive_ap_output_path(Path::new("policy.md"), Some(Path::new("out/catalog.json")));
/// assert_eq!(ap, PathBuf::from("out/policy-assessment-plan.json"));
/// ```
pub fn derive_ap_output_path(input: &Path, primary_output: Option<&Path>) -> PathBuf;

// ─── Structs ──────────────────────────────────────────────────────────────

/// Top-level Assessment Plan JSON envelope.
/// Serializes to `{"assessment-plan": {...}}`.
pub struct AssessmentPlanEnvelope {
    pub assessment_plan: AssessmentPlan,  // serde rename: "assessment-plan"
}

/// OSCAL Assessment Plan root object.
pub struct AssessmentPlan {
    pub uuid: String,                   // UUID v5, deterministic
    pub metadata: ApMetadata,
    pub import_ssp: ImportSsp,          // serde rename: "import-ssp"
    pub reviewed_controls: ReviewedControls, // serde rename: "reviewed-controls"
}

/// OSCAL metadata for the Assessment Plan.
pub struct ApMetadata {
    pub title: String,           // "Assessment Plan for {policy_title}"
    pub last_modified: String,   // serde rename: "last-modified"; ISO 8601 UTC
    pub version: String,         // "1.0.0"
    pub oscal_version: String,   // serde rename: "oscal-version"; "1.2.0"
}

/// SSP reference — href passed through verbatim from CLI flag.
pub struct ImportSsp {
    pub href: String,
}

/// Container defining assessment scope with one control-selections group.
pub struct ReviewedControls {
    pub description: Option<String>,             // "Controls derived from {title} for assessment review."
    pub control_selections: Vec<ApControlSelection>, // serde rename: "control-selections"
}

/// A single control-selection group listing included controls.
pub struct ApControlSelection {
    pub include_controls: Vec<ApIncludeControl>, // serde rename: "include-controls"
}

/// A single control identifier entry.
pub struct ApIncludeControl {
    pub control_id: String, // serde rename: "control-id"; e.g., "POL-AC-001"
}

// ─── CLI Extension ────────────────────────────────────────────────────────

// New field added to ConvertOptions in src/cli/convert.rs:
//
// pub struct ConvertOptions<'a> {
//     ...existing fields...
//     /// Optional SSP reference for Assessment Plan generation.
//     /// When Some(href), an Assessment Plan is written alongside the primary artifact.
//     /// When None, AP generation is skipped (backward compatible).
//     pub import_ssp: Option<&'a str>,  // NEW
// }

// New flag added to Commands::Convert in src/cli/mod.rs:
//
// /// SSP reference for Assessment Plan generation.
// /// When provided, an Assessment Plan skeleton is written to
// /// {output_dir}/{policy_stem}-assessment-plan.json alongside the converted artifact.
// /// Mutually exclusive with batch mode (2+ input files).
// #[arg(long)]
// import_ssp: Option<String>,

// ─── Pipeline Extensions ──────────────────────────────────────────────────

// run_catalog_pipeline extended signature (src/pipeline.rs):
//
// pub fn run_catalog_pipeline(
//     input_path: &Path,
//     output_path: Option<&Path>,
//     max_size_bytes: u64,
//     format: &OutputFormat,
//     import_ssp_href: Option<&str>,  // NEW — None = no AP generation
// ) -> Result<ConversionStatistics, ForgeError>;

// run_component_pipeline extended signature (src/pipeline.rs):
//
// pub fn run_component_pipeline(
//     input_path: &Path,
//     output_path: Option<&Path>,
//     max_size_bytes: u64,
//     source_profile: Option<&str>,
//     format: &OutputFormat,
//     import_ssp_href: Option<&str>,  // NEW — None = no AP generation
// ) -> Result<ConversionStatistics, ForgeError>;

// ─── ForgeError Extension ─────────────────────────────────────────────────

// New variant added to ForgeError in src/error.rs:
//
// #[error("Assessment plan build error: {0}")]
// AssessmentPlanBuild(String),
//
// Mapped to exit code 2 in exit_code() (parse/structure errors category).
