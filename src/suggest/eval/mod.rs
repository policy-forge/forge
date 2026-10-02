//! Bounded artifact-only suggestion corpus preflight.
//!
//! This module reads explicitly selected artifacts and never invokes an adapter,
//! network service, plugin, shell, or authoritative mutation. Opaque attestations
//! are hash-bound inventory only; authentic evaluation and acceptance remain open.

#![deny(missing_docs)]

mod capture;
pub mod manifest;
mod preflight;
pub mod report;

pub use preflight::{PREFLIGHT_ARTIFACT, preflight, preflight_to};
pub use report::Report;
