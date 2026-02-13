//! WI-15: Component Implemented Requirements — Interface Contracts
//!
//! These are the function signatures and types that must be defined
//! BEFORE implementation (Constitution III: Contract-First Development).

use serde_json::Value;
use uuid::Uuid;

// ─── UUID Namespace Constants (in src/uuid.rs) ─────────────────────────

/// Fixed namespace UUID for control-implementation identifier generation.
///
/// Derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"control-implementation")`.
///
/// # Breaking Change Warning
///
/// Changing this value will change ALL generated control-implementation UUIDs.
pub const CONTROL_IMPL_NAMESPACE: Uuid = Uuid::from_bytes([/* computed bytes */]);

/// Fixed namespace UUID for implemented-requirement identifier generation.
///
/// Derived from `Uuid::new_v5(&FORGE_NAMESPACE_UUID, b"implemented-requirement")`.
///
/// # Breaking Change Warning
///
/// Changing this value will change ALL generated implemented-requirement UUIDs.
pub const IMPL_REQ_NAMESPACE: Uuid = Uuid::from_bytes([/* computed bytes */]);

// ─── Core Builder Functions (in src/oscal/implemented_requirements.rs) ──

/// Build the `control-implementations` JSON array for a Component Definition.
///
/// Walks the document's sections to generate control-ids consistent with the
/// Catalog builder (WI-9). Produces a JSON array containing one
/// `control-implementations` entry with the source profile reference and all
/// implemented-requirements mapped from PolicyRequirements.
///
/// # Arguments
/// * `document` - PolicyDocument with sections and requirements
/// * `source_profile` - Value of the `--source-profile` CLI flag
///
/// # Returns
/// A `serde_json::Value` array containing one control-implementations entry.
///
/// # Errors
/// Returns `ForgeError::ComponentDefinitionBuild` if mapping fails.
pub fn build_control_implementations(
    document: &PolicyDocument,
    source_profile: &str,
) -> Result<Value, ForgeError>;

/// Map a single PolicyRequirement to an OSCAL implemented-requirement JSON entry.
///
/// # Arguments
/// * `requirement` - A single PolicyRequirement from the domain model
/// * `control_id` - The pre-computed control-id for this requirement
/// * `global_index` - The requirement's global index (for UUID seed uniqueness)
///
/// # Returns
/// A `serde_json::Value` object with `uuid`, `control-id`, and `description` fields.
///
/// # Errors
/// Returns `ForgeError::ComponentDefinitionBuild` if UUID generation fails.
fn map_requirement_to_implemented(
    requirement: &PolicyRequirement,
    control_id: &str,
    global_index: usize,
) -> Result<Value, ForgeError>;

/// Generate a deterministic UUID v5 for a control-implementation entry.
///
/// Seed: `"{source_profile}\0{policy_title}"`
fn generate_control_impl_uuid(source_profile: &str, policy_title: &str) -> Uuid;

/// Generate a deterministic UUID v5 for an implemented-requirement entry.
///
/// Seed: `"{stable_id}\0{text}\0{index}"`
/// The index ensures uniqueness when two requirements have identical text (EC-5).
fn generate_impl_req_uuid(stable_id: &str, text: &str, index: usize) -> Uuid;

/// Derive a control-id for a requirement, with fallback for missing stable_id.
///
/// Normal case: uses section-based generation via `generate_control_id`
/// Fallback (EC-2): `REQ-{zero-padded global_index}` when stable_id is None
fn derive_control_id_or_fallback(
    abbreviation: &str,
    req_index_in_section: usize,
    global_index: usize,
    has_stable_id: bool,
) -> String;

// ─── Modified Existing Functions ────────────────────────────────────────

/// build_component_definition (MODIFIED — in src/oscal/component_definition.rs)
///
/// New signature adds source_profile parameter:
pub fn build_component_definition(
    document: &PolicyDocument,
    source_profile: Option<&str>,
) -> Result<ComponentDefinitionEnvelope, ForgeError>;

/// convert::execute (MODIFIED — in src/cli/convert.rs)
///
/// New signature adds source_profile parameter:
pub fn execute(
    input: &Path,
    strategy: &Strategy,
    format: &OutputFormat,
    output: Option<&Path>,
    max_size: u64,
    source_profile: Option<&str>,
) -> Result<(), ForgeError>;

// ─── New Pipeline Function (in src/pipeline.rs) ─────────────────────────

/// Orchestrates the full component pipeline: ingest -> parse -> normalize -> map -> serialize -> output.
///
/// # Arguments
/// * `input_path` - Path to the Markdown policy document
/// * `output_path` - Optional output file path; if None, writes JSON to stdout
/// * `max_size_bytes` - Maximum allowed input file size in bytes
/// * `source_profile` - The baseline profile reference for control-implementations
///
/// # Errors
/// * `Err(ForgeError)` if any pipeline stage fails
pub fn run_component_pipeline(
    input_path: &Path,
    output_path: Option<&Path>,
    max_size_bytes: u64,
    source_profile: &str,
) -> Result<(), ForgeError>;

// ─── CLI Changes (in src/cli/mod.rs) ────────────────────────────────────

// Add to Commands::Convert:
//
//     /// Source profile/baseline reference for component strategy
//     #[arg(long)]
//     source_profile: Option<String>,
//
// Validation in convert.rs (runtime, not clap-level):
// - strategy == Component && source_profile.is_none() -> ForgeError::Validation
// - strategy == Component && source_profile == Some("") -> ForgeError::Validation

// ─── Visibility Changes (in src/oscal/catalog.rs) ───────────────────────

// Change: fn resolve_abbreviation -> pub(crate) fn resolve_abbreviation
// Reason: Component builder needs same abbreviation collision resolution
