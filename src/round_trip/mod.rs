//! Round-trip validation module — compares FORGE output against oscal-cli canonical conversions.
//!
//! Provides OSCAL-aware semantic comparison, a multi-format conversion chain
//! (JSON → XML → YAML → JSON), and structured divergence logging.

mod chain;
mod comparator;
mod divergence;
mod divergence_log;
mod rules;

pub use chain::run_round_trip_chain;
pub use comparator::compare_oscal_json;
pub use divergence::{
    ArtifactType, CompatibilityClassification, Divergence, DivergenceClass, ResolutionStatus,
    RoundTripResult, classify_oscal_cli_compatibility,
};
pub use divergence_log::write_divergence_log;
pub use rules::OscalComparisonRules;
