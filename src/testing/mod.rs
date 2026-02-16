//! Testing utilities for OSCAL round-trip fidelity verification.
//!
//! Provides semantic equivalence comparison for `serde_json::Value` trees,
//! enabling format-agnostic round-trip testing across JSON, XML, and YAML.

pub mod semantic_eq;

pub use semantic_eq::{EquivalenceDiff, EquivalenceResult, assert_semantic_equivalence};
