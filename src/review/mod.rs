//! Portable review contracts and pending asserted-policy evaluation.
//! Native currentness, publication, authentication and domain promotion require
//! their separate genuine capture and finalization consumers.

/// Exact borrowed originals, shared bounded decode work and fixed failures.
pub(crate) mod decode;
/// Complete structural correlations and versioned typed fingerprints.
pub(crate) mod validate;
/// Closed inert typed declarations.
pub(crate) mod wire;

#[cfg(test)]
/// Structural contract controls, not native workflow qualification.
mod tests;

/// Complete exact original identity and self-supersession graph validation.
pub(crate) mod chain;
/// Pending declared-policy evaluation; no successful native proof constructor.
pub(crate) mod merge;
#[cfg(test)]
/// Pure core controls; no captured native authority is fabricated.
mod merge_tests;
/// Bounded deterministic one-asserted-key-per-seat maximum matching.
mod quorum;

/// One genuinely held confined capture and complete final input fence.
pub(crate) mod capture;

/// Static redacted recorded-dispositions HTML; no currentness or domain authority.
pub(crate) mod html;

/// Domain-specific source-bound item facts; no detached success constructor.
pub(crate) mod adapters;
/// Currentness-qualified dispositions retaining the selected domain and complete original owner.
pub(crate) mod finalize;
/// Complete native Mapping originals and neutral recorded approval/current tuple.
pub(crate) mod mapping_capture;

/// Genuine full applicability report-source capture and recorded-current lifecycle gate.
pub(crate) mod applicability_capture;

/// Plain legacy preparations under the existing command ledger and cooperative control.
pub(crate) mod legacy_work;

/// Six explicit-path review flows using one actual capture/ledger/control owner.
pub(crate) mod commands;
/// Bounded complete closed JSON output under the same command ledger.
pub(crate) mod encode;

/// Complete local notifications from one captured recorded queue; no delivery authority.
pub(crate) mod notifications;

/// Complete actual captured legacy Impact roster and strict stored-current comparison.
pub(crate) mod impact_capture;
/// Strict full stored/native Impact equality returning non-authorizing borrowed data.
pub(crate) mod impact_report;

/// Strict explicit links and complete plain old/new item accounting; no native proof.
pub(crate) mod links;

/// Inert fixed-whitelist companion endpoint and original/queue pin fragments.
pub(crate) mod supersession_wire;

/// Inert closed recorded supersession decoding and exact queue-data binding.
pub(crate) mod supersession_decode;
