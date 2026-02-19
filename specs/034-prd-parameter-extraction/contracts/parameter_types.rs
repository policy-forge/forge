//! Domain types for WI-34 Parameter Extraction.
//!
//! These types are added to `src/model/mod.rs` alongside the existing
//! `Citation`, `PolicyRequirement`, `PolicySection`, and `PolicyDocument` types.
//!
//! # Contract
//! - All types are `Clone + Debug + PartialEq + Serialize`
//! - `PolicyParameter` must be `PartialEq` for idempotence verification
//! - Serde serialization must roundtrip without loss

use serde::Serialize;

// ─── New types in src/model/mod.rs ────────────────────────────────────────

/// A parameterizable value extracted from a policy requirement (WI-34).
///
/// Represents a configurable criterion (time window, threshold, frequency,
/// or quantity) extracted from requirement prose. Each parameter is linked
/// to its source `PolicyRequirement` via `requirement_id`.
///
/// # Deterministic Guarantee
/// Given the same source text and requirement ID, extraction always produces
/// the same `PolicyParameter` with the same `id` at the same position.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyParameter {
    /// Deterministic identifier: `"{requirement_id}_prm_{position}"`.
    ///
    /// `position` is the 0-based index of this parameter within the requirement.
    /// Example: `"POL-AC-001_prm_0"`
    pub id: String,

    /// `stable_id` of the `PolicyRequirement` this parameter was extracted from.
    pub requirement_id: String,

    /// Human-readable label describing this parameter.
    ///
    /// Derived from the matched text (e.g., `"within 30 days"`, `"at least 128-bit"`).
    pub label: String,

    /// The extracted parameter value as a string.
    ///
    /// Examples: `"30 days"`, `"128-bit"`, `"annually"`, `"3 factors"`
    pub value: String,

    /// The semantic category of this parameter.
    pub parameter_type: ParameterType,

    /// Value domain constraint inferred from qualifier words.
    ///
    /// `None` only for bare frequency words without an explicit qualifier
    /// (e.g., `"quarterly"` without `"at least"`), which receive `Exact` constraint.
    /// In practice, all extracted parameters will have a constraint.
    pub constraint: Option<ParameterConstraint>,
}

/// The semantic category of a policy parameter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ParameterType {
    /// Duration parameters specifying a deadline or period.
    ///
    /// Patterns: `"within N days"`, `"after N weeks"`, `"every N months"`
    TimeWindow,

    /// Numeric boundary parameters specifying a minimum or maximum value.
    ///
    /// Patterns: `"at least N"`, `"minimum N"`, `"no more than N"`, `"no fewer than N"`
    Threshold,

    /// Recurrence parameters specifying how often an action must occur.
    ///
    /// Patterns: `"annually"`, `"quarterly"`, `"at least monthly"`, `"daily"`
    Frequency,

    /// Count parameters specifying a number of items.
    ///
    /// Patterns: `"no fewer than 3 factors"`, `"at least 2 generations"`
    Quantity,
}

/// Value domain constraint on a `PolicyParameter`.
///
/// Maps to an OSCAL `param.constraint` element.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ParameterConstraint {
    /// The direction of the bound.
    pub constraint_type: ConstraintType,

    /// The bound value as a string (same string as `PolicyParameter.value`).
    pub value: String,
}

/// The direction of a value domain bound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ConstraintType {
    /// Value must be at least this amount.
    ///
    /// Triggered by: `"at least"`, `"minimum"`, `"no fewer than"`, `"no less than"`, `"within"`, `"after"`
    Minimum,

    /// Value must be at most this amount.
    ///
    /// Triggered by: `"no more than"`, `"maximum"`, `"at most"`
    Maximum,

    /// Value must equal exactly this.
    ///
    /// Triggered by: `"every N"`, bare frequency words, values without a directional qualifier
    Exact,
}

// ─── Modification to PolicyRequirement (src/model/mod.rs) ─────────────────

/// DIFF: Add `parameters` field to existing `PolicyRequirement`.
///
/// Before (current):
/// ```
/// pub struct PolicyRequirement {
///     pub stable_id: Option<String>,
///     pub text: String,
///     pub source_line: usize,
///     pub nesting_depth: u8,
///     pub atom_index: usize,
///     pub parent_text: Option<String>,
///     pub citations: Vec<Citation>,
/// }
/// ```
///
/// After (WI-34):
/// ```
/// pub struct PolicyRequirement {
///     pub stable_id: Option<String>,
///     pub text: String,           // Updated by WI-34: contains insertion placeholders
///     pub source_line: usize,
///     pub nesting_depth: u8,
///     pub atom_index: usize,
///     pub parent_text: Option<String>,
///     pub citations: Vec<Citation>,
///     pub parameters: Vec<PolicyParameter>,  // NEW: empty until WI-34 enrichment
/// }
/// ```
///
/// All existing tests constructing `PolicyRequirement` must add `parameters: vec![]`.
pub struct _PolicyRequirementDiff;
