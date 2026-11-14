//! Strict separately closed edit-capable Response/4 and Dispositions/4 intake.
//! Queues remain the closed /3 shape; only the response envelope gains an optional
//! inert proposed edit. Decoding supplies no native facts or currentness owner.

pub(crate) use super::decode::{ContractError, ContractLedger};
pub(crate) use super::decode_v3::{DecodedV3, compared, hash, key, time, uuid};

/// Separately closed plain Response/4 and recorded Dispositions/4 intake.
#[path = "decode_v4_operational.rs"]
mod operational;
pub(crate) use operational::{decode_dispositions, decode_response, dispositions, response};

#[cfg(test)]
#[path = "decode_v4_tests.rs"]
mod tests;
