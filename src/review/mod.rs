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

// Root integration addition only; existing /1 module bodies stay byte-exact.
/// Strict raw/shape/semantic decoding under the original contract ledger.
pub(crate) mod decode_v2;
/// Fixed bounded binary profiles; digests issue no native capability.
pub(crate) mod hash_v2;
/// Separately closed inert Lifecycle exchange models.
pub(crate) mod wire_v2;

#[cfg(test)]
#[path = "lifecycle_wire_v2_tests.rs"]
/// Genuine component controls over explicitly inert synthetic operands.
mod lifecycle_wire_v2_tests;

/// Separate inert /2 chain consumer; no native currentness authority.
pub(crate) mod chain_v2;
/// Separate inert /2 merge consumer; no native currentness authority.
pub(crate) mod merge_v2;
#[cfg(test)]
#[path = "policy_v2_tests.rs"]
/// Genuine pending-policy controls; no native currentness or approval authority.
mod policy_v2_tests;
/// Separate inert /2 quorum consumer; no native currentness authority.
pub(crate) mod quorum_v2;

/// Genuine complete captured Lifecycle record and native artifact closure.
mod lifecycle_capture;

/// Pure finite /2 recorded JSON encoding and full inert typed readback.
pub(crate) mod encode_v2;

/// Root integration: register this additive pending leaf after genuine receiver/wire modules.
pub(crate) mod lifecycle_binding;

/// Genuine native Lifecycle currentness with the complete actual Queue/Response cohort.
pub(crate) mod lifecycle_finalize;

/// Genuine Lifecycle current and ordinary response operations with guarded output sinks.
pub(crate) mod lifecycle_commands;
/// Genuine native Lifecycle init queue export; distinct from captured input Queue binding.
pub(crate) mod lifecycle_queue_export;

/// Seven admitted plain Authoring-plan frame profiles; no native owner/currentness.
pub(crate) mod hash_v3;
/// Separately closed inert Authoring-plan /3 declarations; no native authority.
pub(crate) mod wire_v3;

/// Strict plain Authoring Queue/3 codec; no native owner or publication capability.
#[path = "decode_v3.rs"]
pub(crate) mod decode_v3;
/// Finite plain Authoring Queue/3 serializer with exact typed readback.
#[path = "encode_v3.rs"]
pub(crate) mod encode_v3;

/// Genuine whole native authoring receiver, distinct from inert Queue/3 codecs.
pub(crate) mod authoring_capture;
/// Strict fixed private authoring locator; no native owner issuer.
mod authoring_locator;
/// Same-original-ledger native authoring forwarding, without independent allowances.
mod authoring_work;

/// Genuine Authoring Queue and every actual Response original retained by one native owner.
pub(crate) mod authoring_binding;
/// Actual private Authoring current-output issuer over a genuine complete binding.
pub(crate) mod authoring_finalize;
/// Ordinary separately typed /3 Queue/Response/recorded-output assertion binding.
pub(crate) mod authoring_response_binding;
/// Complete plain /3 original identity and withdrawal chain, without native authority.
pub(crate) mod chain_v3;
/// Pending /3 classification and complete policy/history facts, without currentness.
pub(crate) mod merge_v3;
/// Independently typed asserted /3 distinct-key maximum matching.
pub(crate) mod quorum_v3;

/// Strict private asserted Authoring Init policy, without native authority.
pub(crate) mod authoring_init_policy;
/// Genuine native Init Queue preparation and finite output retaining the complete owner.
pub(crate) mod authoring_queue_export;

/// Whole native Authoring review command lifetime through actual output sinks.
pub(crate) mod authoring_commands;
