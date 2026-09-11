//! Deterministic reuse candidates for unresolved authoring sections.
//!
//! Retrieval only: every byte returned to a caller exists in an
//! operator-supplied source document, with an exact span and hash. Nothing in
//! this module generates, rewrites, or merges text, so a candidate can never
//! become prose that no human supplied.

pub mod corpus;
