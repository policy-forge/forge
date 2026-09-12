//! Deterministic reuse candidates for unresolved authoring sections.
//!
//! Retrieval only: every byte returned to a caller exists in an
//! operator-supplied source document, with an exact span and hash. Nothing in
//! this module generates, rewrites, or merges text, so a candidate can never
//! become prose that no human supplied.

pub mod blocks;
pub mod capture;
pub mod corpus;
mod execute;
pub mod rank;
pub mod report;

pub use execute::execute;
pub(crate) use execute::{corpus_location, read_manifest};
